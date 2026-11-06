//! F4 resource guards (protocol 03, F4), applied to every file an engine entry point opens.
//!
//! Two layers, both bounded:
//! 1. **Structure scan** ([`scan_reader`]): a streaming PDF lexer over 64 KiB buffers (memory does
//!    not grow with the file). It tracks `/Length` in stream dictionaries, with names normalised
//!    (`#xx` escapes decoded, so `/Len#67th` is `/Length`), resolves indirect lengths
//!    (`/Length 5 0 R` → `5 0 obj 123 endobj`) when the target is a plain integer object, skips
//!    strings, comments and stream data, and trips `declared-stream-length` when a stream declares
//!    more bytes than the whole file holds. Lengths it cannot resolve (inside object streams) are
//!    left to layer 2.
//! 2. **Decompression-ratio cap** ([`check_decompression_ratio`]): every stream is decoded with
//!    pdfcraft's own bounded decoder (`pdfcraft_cos::Stream::decoded_within`), with a limit of
//!    [`RATIO_CAP`] x its encoded size (at least [`RATIO_FLOOR`]), and the decoded total is capped
//!    at [`TOTAL_DECODED_CAP`]. This is where the bytes are actually decompressed, so a bomb with an
//!    honest small `/Length` trips `decompression-ratio`. Note: pdfcraft's internal decodes keep its
//!    own fixed 1 GiB cap (`pdfcraft_cos::MAX_DECODED`); making the ratio native to pdfcraft needs a
//!    pdfcraft-cos patch (lead decision, see docs/ENGINE_REUSE_MATRIX.md).
//!
//! [`read_guarded`] is the one way engine code reads an input PDF: it reads the file once, runs
//! both layers the first time a given (path, size, mtime) is seen, and caches the verdict, so a
//! RenderPage after a load does not rescan.

use lpdf_core_types::{LpdfError, Result};
use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::SystemTime;

/// A tripped F4 guard: its stable name and a plain-language detail.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuardTrip {
    pub guard: &'static str,
    pub detail: String,
}

impl std::fmt::Display for GuardTrip {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "F4 guard '{}' tripped: {}", self.guard, self.detail)
    }
}

impl From<GuardTrip> for LpdfError {
    fn from(t: GuardTrip) -> Self {
        LpdfError::ResourceLimit(t.to_string())
    }
}

pub const DECLARED_STREAM_LENGTH: &str = "declared-stream-length";
pub const DECOMPRESSION_RATIO: &str = "decompression-ratio";
pub const DECOMPRESSION_TOTAL: &str = "decompression-total";

/// Most a stream may expand relative to its encoded size.
pub const RATIO_CAP: usize = 200;
/// Streams may always decode to this much, whatever their ratio (small, highly compressible
/// streams such as blank image masks are legitimate).
pub const RATIO_FLOOR: usize = 32 << 20;
/// Most all streams of one file may decode to in total.
pub const TOTAL_DECODED_CAP: usize = 4 << 30;
/// Object-stream decode bound when opening a file for the ratio check.
const CHECK_OPEN_STREAM_LIMIT: usize = 256 << 20;
/// Read buffer of the structure scan.
pub const SCAN_BUFFER: usize = 64 << 10;
/// Bounds on what the scan remembers (beyond them, lengths are "unknown", left to layer 2).
const MAX_INT_OBJECTS: usize = 200_000;
const MAX_LENGTH_REFS: usize = 200_000;
const MAX_NAME: usize = 127;
const MAX_WORD: usize = 24;

// ------------------------------------------------------------------------------------------
// Layer 1: streaming structure scan
// ------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Name(Vec<u8>),
    Int(u64),
    Word(Vec<u8>),
    Other,
}

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Normal,
    Name,
    Number,
    Word,
    Comment,
    LitString { depth: u32, escape: bool },
    LessThan,
    HexString,
    Stream { matched: usize },
}

const ENDSTREAM: &[u8] = b"endstream";
/// KMP failure table of `endstream` (the only self-overlap is the leading `e`).
const ENDSTREAM_FAIL: [usize; 9] = [0, 0, 0, 0, 0, 0, 1, 0, 0];

fn is_ws(b: u8) -> bool {
    matches!(b, b' ' | b'\n' | b'\r' | b'\t' | b'\x0c' | b'\0')
}
fn is_delim(b: u8) -> bool {
    matches!(
        b,
        b'(' | b')' | b'<' | b'>' | b'[' | b']' | b'{' | b'}' | b'/' | b'%'
    )
}
fn hex(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

/// Incremental scanner state. Feed bytes with [`Scanner::feed`], then call [`Scanner::finish`].
struct Scanner {
    file_len: u64,
    offset: u64,
    mode: Mode,
    buf: Vec<u8>,
    name_hex: Option<(u8, Option<u8>)>,
    number_is_int: bool,
    ring: Vec<Tok>,
    int_objects: HashMap<u32, u64>,
    length_refs: Vec<(u32, u64)>,
    trip: Option<GuardTrip>,
}

impl Scanner {
    fn new(file_len: u64) -> Self {
        Self {
            file_len,
            offset: 0,
            mode: Mode::Normal,
            buf: Vec::with_capacity(MAX_NAME + 1),
            name_hex: None,
            number_is_int: true,
            ring: Vec::with_capacity(8),
            int_objects: HashMap::new(),
            length_refs: Vec::new(),
            trip: None,
        }
    }

    fn too_long(&mut self, value: u64, how: &str) {
        if self.trip.is_none() && value > self.file_len {
            self.trip = Some(GuardTrip {
                guard: DECLARED_STREAM_LENGTH,
                detail: format!(
                    "a stream near byte {} declares {how}/Length {value} bytes, but the whole file is {} bytes \
                     (decompression bomb or truncated file)",
                    self.offset, self.file_len
                ),
            });
        }
    }

    /// Patterns over the last tokens: `/Length v X` (direct), `/Length v g R` (indirect),
    /// `n g obj v endobj` (integer object).
    fn token(&mut self, t: Tok) {
        self.ring.push(t);
        if self.ring.len() > 6 {
            self.ring.remove(0);
        }
        enum Act {
            None,
            Direct(u64),
            Indirect(u32),
            IntObject(u32, u64),
        }
        let act = {
            let n = self.ring.len();
            let at = |i: usize| if i <= n { self.ring.get(n - i) } else { None };
            let is_len =
                |t: Option<&Tok>| matches!(t, Some(Tok::Name(v)) if v.as_slice() == b"Length");
            let word =
                |t: Option<&Tok>, w: &[u8]| matches!(t, Some(Tok::Word(x)) if x.as_slice() == w);
            match (at(5), at(4), at(3), at(2), at(1)) {
                // `n g obj v endobj`
                (Some(Tok::Int(num)), Some(Tok::Int(_)), o, Some(Tok::Int(v)), e)
                    if word(o, b"obj") && word(e, b"endobj") =>
                {
                    u32::try_from(*num)
                        .map(|num| Act::IntObject(num, *v))
                        .unwrap_or(Act::None)
                }
                // `/Length v g R`: indirect
                (_, l, Some(Tok::Int(v)), Some(Tok::Int(_)), r) if is_len(l) && word(r, b"R") => {
                    Act::Indirect(u32::try_from(*v).unwrap_or(u32::MAX))
                }
                // `/Length v g Y`, Y != R: direct v
                (_, l, Some(Tok::Int(v)), Some(Tok::Int(_)), Some(_)) if is_len(l) => {
                    Act::Direct(*v)
                }
                // `/Length v X`, X not an integer: direct v
                (_, _, l, Some(Tok::Int(v)), Some(x)) if is_len(l) && !matches!(x, Tok::Int(_)) => {
                    Act::Direct(*v)
                }
                _ => Act::None,
            }
        };
        match act {
            Act::None => {}
            Act::Direct(v) => self.too_long(v, ""),
            Act::Indirect(num) => {
                if self.length_refs.len() < MAX_LENGTH_REFS {
                    self.length_refs.push((num, self.offset));
                }
            }
            Act::IntObject(num, v) => {
                if self.int_objects.len() < MAX_INT_OBJECTS {
                    self.int_objects.insert(num, v);
                }
            }
        }
    }

    fn end_token(&mut self) {
        let t = match self.mode {
            Mode::Name => {
                if let Some((_, _)) = self.name_hex.take() {
                    // A dangling `#` / `#x` is kept literally (lenient readers do the same).
                    self.buf.push(b'#');
                }
                Tok::Name(std::mem::take(&mut self.buf))
            }
            Mode::Number => {
                let ok = self.number_is_int && !self.buf.is_empty();
                let v = if ok {
                    self.buf.iter().fold(0u64, |acc, d| {
                        acc.saturating_mul(10).saturating_add(u64::from(d - b'0'))
                    })
                } else {
                    0
                };
                self.buf.clear();
                if ok {
                    Tok::Int(v)
                } else {
                    Tok::Other
                }
            }
            Mode::Word => Tok::Word(std::mem::take(&mut self.buf)),
            _ => return,
        };
        let starts_stream = matches!(&t, Tok::Word(w) if w.as_slice() == b"stream");
        self.mode = Mode::Normal;
        self.token(t);
        if starts_stream {
            // The dictionary before `stream` has been evaluated; skip the data.
            self.ring.clear();
            self.mode = Mode::Stream { matched: 0 };
        }
    }

    fn push_capped(&mut self, b: u8, cap: usize) {
        if self.buf.len() < cap {
            self.buf.push(b);
        } else if self.buf.len() == cap {
            self.buf.push(0); // marks "too long": can never equal a keyword or `Length`
        }
    }

    fn byte(&mut self, b: u8) {
        loop {
            match self.mode {
                Mode::Stream { matched } => {
                    let mut m = matched;
                    while m > 0 && ENDSTREAM.get(m) != Some(&b) {
                        m = ENDSTREAM_FAIL.get(m - 1).copied().unwrap_or(0);
                    }
                    if ENDSTREAM.get(m) == Some(&b) {
                        m += 1;
                    }
                    self.mode = if m == ENDSTREAM.len() {
                        Mode::Normal
                    } else {
                        Mode::Stream { matched: m }
                    };
                    return;
                }
                Mode::Comment => {
                    if b == b'\n' || b == b'\r' {
                        self.mode = Mode::Normal;
                    }
                    return;
                }
                Mode::LitString { depth, escape } => {
                    self.mode = if escape {
                        Mode::LitString {
                            depth,
                            escape: false,
                        }
                    } else {
                        match b {
                            b'\\' => Mode::LitString {
                                depth,
                                escape: true,
                            },
                            b'(' => Mode::LitString {
                                depth: depth.saturating_add(1),
                                escape: false,
                            },
                            b')' if depth <= 1 => {
                                self.token(Tok::Other);
                                Mode::Normal
                            }
                            b')' => Mode::LitString {
                                depth: depth - 1,
                                escape: false,
                            },
                            _ => Mode::LitString {
                                depth,
                                escape: false,
                            },
                        }
                    };
                    return;
                }
                Mode::LessThan => {
                    if b == b'<' {
                        self.mode = Mode::Normal; // `<<`
                        return;
                    }
                    self.mode = Mode::HexString;
                    continue;
                }
                Mode::HexString => {
                    if b == b'>' {
                        self.mode = Mode::Normal;
                        self.token(Tok::Other);
                    }
                    return;
                }
                Mode::Name => {
                    if is_ws(b) || is_delim(b) {
                        self.end_token();
                        continue;
                    }
                    match self.name_hex {
                        Some((_, None)) => match hex(b) {
                            Some(h) => self.name_hex = Some((b, Some(h))),
                            None => {
                                self.name_hex = None;
                                self.push_capped(b'#', MAX_NAME);
                                continue;
                            }
                        },
                        Some((first, Some(hi))) => {
                            self.name_hex = None;
                            match hex(b) {
                                Some(lo) => self.push_capped(hi * 16 + lo, MAX_NAME),
                                None => {
                                    self.push_capped(b'#', MAX_NAME);
                                    self.push_capped(first, MAX_NAME);
                                    continue;
                                }
                            }
                        }
                        None if b == b'#' => self.name_hex = Some((b'#', None)),
                        None => self.push_capped(b, MAX_NAME),
                    }
                    return;
                }
                Mode::Number => {
                    if b.is_ascii_digit() {
                        self.push_capped(b, MAX_WORD);
                        if self.buf.len() > MAX_WORD {
                            // Saturate: keep a value larger than any file.
                            self.buf.truncate(MAX_WORD);
                        }
                        return;
                    }
                    if b == b'.' || b == b'+' || b == b'-' {
                        self.number_is_int = false;
                        return;
                    }
                    self.end_token();
                    continue;
                }
                Mode::Word => {
                    if is_ws(b) || is_delim(b) {
                        self.end_token();
                        continue;
                    }
                    self.push_capped(b, MAX_WORD);
                    return;
                }
                Mode::Normal => {
                    match b {
                        _ if is_ws(b) => {}
                        b'/' => {
                            self.buf.clear();
                            self.name_hex = None;
                            self.mode = Mode::Name;
                        }
                        b'0'..=b'9' => {
                            self.buf.clear();
                            self.buf.push(b);
                            self.number_is_int = true;
                            self.mode = Mode::Number;
                        }
                        b'+' | b'-' | b'.' => {
                            self.buf.clear();
                            self.number_is_int = false;
                            self.mode = Mode::Number;
                        }
                        b'%' => self.mode = Mode::Comment,
                        b'(' => {
                            self.mode = Mode::LitString {
                                depth: 1,
                                escape: false,
                            }
                        }
                        b'<' => self.mode = Mode::LessThan,
                        b'>' | b'[' | b']' | b'{' | b'}' | b')' => self.token(Tok::Other),
                        _ => {
                            self.buf.clear();
                            self.buf.push(b);
                            self.mode = Mode::Word;
                        }
                    }
                    return;
                }
            }
        }
    }

    fn feed(&mut self, chunk: &[u8]) {
        for &b in chunk {
            if self.trip.is_some() {
                return;
            }
            self.byte(b);
            self.offset += 1;
        }
    }

    fn finish(mut self) -> std::result::Result<(), GuardTrip> {
        self.end_token();
        // Flush a trailing `/Length v` at end of input.
        self.token(Tok::Other);
        if let Some(t) = self.trip.take() {
            return Err(t);
        }
        for (num, offset) in std::mem::take(&mut self.length_refs) {
            if let Some(&v) = self.int_objects.get(&num) {
                if v > self.file_len {
                    return Err(GuardTrip {
                        guard: DECLARED_STREAM_LENGTH,
                        detail: format!(
                            "a stream near byte {offset} declares (via indirect object {num}) /Length {v} bytes, \
                             but the whole file is {} bytes (decompression bomb or truncated file)",
                            self.file_len
                        ),
                    });
                }
            }
        }
        Ok(())
    }
}

/// Layer 1 over any reader, in [`SCAN_BUFFER`] chunks; `file_len` is the file's size.
pub fn scan_reader(mut reader: impl Read, file_len: u64) -> std::result::Result<(), GuardTrip> {
    let mut s = Scanner::new(file_len);
    let mut buf = vec![0u8; SCAN_BUFFER];
    loop {
        let n = match reader.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => n,
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(e) => {
                return Err(GuardTrip {
                    guard: DECLARED_STREAM_LENGTH,
                    detail: format!("the file could not be read: {e}"),
                })
            }
        };
        s.feed(buf.get(..n).unwrap_or(&[]));
        if s.trip.is_some() {
            break;
        }
    }
    s.finish()
}

/// Layer 1 over bytes already in memory.
pub fn check_declared_stream_lengths(bytes: &[u8]) -> std::result::Result<(), GuardTrip> {
    scan_reader(bytes, bytes.len() as u64)
}

// ------------------------------------------------------------------------------------------
// Layer 2: decompression-ratio cap
// ------------------------------------------------------------------------------------------

/// Decode every stream with pdfcraft's bounded decoder within `RATIO_CAP` x its size (at least
/// `RATIO_FLOOR`) and the per-file total within `TOTAL_DECODED_CAP`. Files pdfcraft cannot open
/// (damaged beyond repair, or encrypted without an empty password) pass: the engines decide on
/// them, and pdfcraft's own 1 GiB cap still applies to their decodes.
pub fn check_decompression_ratio(bytes: Arc<Vec<u8>>) -> std::result::Result<(), GuardTrip> {
    let Ok(doc) =
        pdfcraft_cos::Document::open_with_stream_limit(bytes, None, CHECK_OPEN_STREAM_LIMIT)
    else {
        return Ok(());
    };
    let mut total: usize = 0;
    for num in doc.object_numbers() {
        let obj = doc.get(pdfcraft_cos::ObjRef {
            num,
            generation: doc.generation(num),
        });
        let pdfcraft_cos::Object::Stream(s) = &*obj else {
            continue;
        };
        let raw = s.raw.len();
        let limit = raw
            .saturating_mul(RATIO_CAP)
            .clamp(RATIO_FLOOR, pdfcraft_cos::MAX_DECODED);
        match s.decoded_within(limit) {
            Ok(v) => {
                total = total.saturating_add(v.len());
                if total > TOTAL_DECODED_CAP {
                    return Err(GuardTrip {
                        guard: DECOMPRESSION_TOTAL,
                        detail: format!(
                            "the file's streams decode to more than {} bytes in total",
                            TOTAL_DECODED_CAP
                        ),
                    });
                }
            }
            Err(pdfcraft_cos::CosError::Filter(msg)) if msg.contains("exceeds the limit") => {
                return Err(GuardTrip {
                    guard: DECOMPRESSION_RATIO,
                    detail: format!(
                        "object {num}: {raw} encoded bytes decode to more than {limit} bytes (over {RATIO_CAP}:1, decompression bomb)"
                    ),
                });
            }
            // Image codecs, corrupt data, unsupported filters: not a size question.
            Err(_) => {}
        }
    }
    Ok(())
}

/// Both layers on bytes in memory (tests, in-memory callers).
pub fn preflight(bytes: &[u8]) -> std::result::Result<(), GuardTrip> {
    check_declared_stream_lengths(bytes)?;
    check_decompression_ratio(Arc::new(bytes.to_vec()))
}

// ------------------------------------------------------------------------------------------
// Entry point for engine code, with the verdict cache
// ------------------------------------------------------------------------------------------

type Key = (PathBuf, u64, Option<SystemTime>);
const CACHE_CAP: usize = 512;

fn cache() -> &'static Mutex<HashMap<Key, std::result::Result<(), GuardTrip>>> {
    static C: OnceLock<Mutex<HashMap<Key, std::result::Result<(), GuardTrip>>>> = OnceLock::new();
    C.get_or_init(|| Mutex::new(HashMap::new()))
}

fn key_of(path: &Path) -> Result<Key> {
    let meta = std::fs::metadata(path).map_err(LpdfError::Io)?;
    let canon = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    Ok((canon, meta.len(), meta.modified().ok()))
}

/// How many times layer 1+2 actually ran (cache misses), for tests and the benchmark.
pub fn scan_count() -> usize {
    SCANS.load(std::sync::atomic::Ordering::Relaxed)
}
static SCANS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// Run both guards on `path` unless this (path, size, mtime) already has a verdict.
/// Layer 1 streams the file; layer 2 uses `bytes` when given (the bytes the caller will parse),
/// else reads the file.
fn verdict(
    path: &Path,
    bytes: Option<&Arc<Vec<u8>>>,
) -> Result<std::result::Result<(), GuardTrip>> {
    let key = key_of(path)?;
    if let Some(v) = cache().lock().unwrap_or_else(|e| e.into_inner()).get(&key) {
        return Ok(v.clone());
    }
    SCANS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let v = match bytes {
        // The caller has the bytes it will parse: scan those (no second read of the file).
        Some(b) => {
            scan_reader(b.as_slice(), key.1).and_then(|()| check_decompression_ratio(Arc::clone(b)))
        }
        // Check only: stream the file through the scanner (bounded memory), then read it for the
        // ratio layer only if the structure is sound.
        None => {
            let file = std::fs::File::open(path).map_err(LpdfError::Io)?;
            match scan_reader(std::io::BufReader::with_capacity(SCAN_BUFFER, file), key.1) {
                Ok(()) => {
                    check_decompression_ratio(Arc::new(std::fs::read(path).map_err(LpdfError::Io)?))
                }
                Err(t) => Err(t),
            }
        }
    };
    let mut c = cache().lock().unwrap_or_else(|e| e.into_inner());
    if c.len() >= CACHE_CAP {
        c.clear();
    }
    c.insert(key, v.clone());
    Ok(v)
}

/// Check `path` (cached). `Err(LpdfError::ResourceLimit)` names the guard that tripped.
pub fn check_file(path: &Path) -> Result<()> {
    verdict(path, None)?.map_err(LpdfError::from)
}

/// Read an input PDF for parsing, refusing it if an F4 guard trips. The file is read once; the
/// guards run only the first time this (path, size, mtime) is seen.
pub fn read_guarded(path: &Path) -> Result<Vec<u8>> {
    let bytes = Arc::new(std::fs::read(path).map_err(LpdfError::Io)?);
    verdict(path, Some(&bytes))?.map_err(LpdfError::from)?;
    Ok(Arc::try_unwrap(bytes).unwrap_or_else(|a| (*a).clone()))
}
