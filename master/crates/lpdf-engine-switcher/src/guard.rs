//! F4 resource guards run before any engine parses a file (protocol 03, F4).
//!
//! Each guard is a cheap, bounded byte scan that rejects a file whose own structure proves it is
//! hostile or truncated, so neither engine (nor the fallback) ever starts on it. A tripped guard
//! is reported as [`lpdf_core_types::LpdfError::ResourceLimit`] with the guard's name, so the UI
//! can tell the user which protection stopped the file (DON'T-7: never fail silently).

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

/// Name of the declared-stream-length guard.
pub const DECLARED_STREAM_LENGTH: &str = "declared-stream-length";

fn is_ws(b: u8) -> bool {
    matches!(b, b' ' | b'\n' | b'\r' | b'\t' | b'\x0c' | b'\0')
}

/// Parse an unsigned decimal at `i`; returns (value saturated at u64::MAX, index after digits).
fn parse_uint(bytes: &[u8], mut i: usize) -> Option<(u64, usize)> {
    let start = i;
    let mut v: u64 = 0;
    while let Some(&b) = bytes.get(i) {
        if !b.is_ascii_digit() {
            break;
        }
        v = v.saturating_mul(10).saturating_add(u64::from(b - b'0'));
        i += 1;
    }
    (i > start).then_some((v, i))
}

fn skip_ws(bytes: &[u8], mut i: usize) -> usize {
    while bytes.get(i).copied().is_some_and(is_ws) {
        i += 1;
    }
    i
}

/// `declared-stream-length`: a direct `/Length N` (stream dictionary) larger than the whole file
/// is impossible for a well-formed stream, whose bytes lie inside the file. It is the signature
/// of a decompression bomb / allocation attack or a truncated file (e.g. `corpus/hostile-bombs.pdf`
/// declares 1 GiB in a ~400-byte file). Indirect lengths (`/Length 5 0 R`) and `/Length1..3`
/// (font program parts) are not checked.
pub fn check_declared_stream_lengths(bytes: &[u8]) -> Result<(), GuardTrip> {
    const KEY: &[u8] = b"/Length";
    let file_len = bytes.len() as u64;
    let mut from = 0;
    while let Some(pos) = bytes
        .get(from..)
        .and_then(|rest| rest.windows(KEY.len()).position(|w| w == KEY))
    {
        let at = from + pos;
        let after_key = at + KEY.len();
        from = after_key;
        // Exactly `/Length`, then whitespace, then a number.
        if !bytes.get(after_key).copied().is_some_and(is_ws) {
            continue;
        }
        let num_at = skip_ws(bytes, after_key);
        let Some((value, end)) = parse_uint(bytes, num_at) else {
            continue;
        };
        // Indirect reference `N G R`: the real length is elsewhere.
        let g_at = skip_ws(bytes, end);
        if g_at > end {
            if let Some((_, g_end)) = parse_uint(bytes, g_at) {
                let r_at = skip_ws(bytes, g_end);
                if g_end > g_at && r_at > g_end && bytes.get(r_at) == Some(&b'R') {
                    continue;
                }
            }
        }
        if value > file_len {
            return Err(GuardTrip {
                guard: DECLARED_STREAM_LENGTH,
                detail: format!(
                    "a stream at byte {at} declares /Length {value} bytes, but the whole file is {file_len} bytes \
                     (decompression bomb or truncated file)"
                ),
            });
        }
    }
    Ok(())
}

/// Every F4 preflight guard, in order.
pub fn preflight(bytes: &[u8]) -> Result<(), GuardTrip> {
    check_declared_stream_lengths(bytes)
}
