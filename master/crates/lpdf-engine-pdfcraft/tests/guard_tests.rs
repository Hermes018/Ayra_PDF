//! F4 guards (BUG-018/029/030). Each bypass below was accepted by the CP2b scanner (a plain
//! `/Length <int>` byte search over the whole file read into memory): an indirect length, a
//! `#xx`-escaped name, an honest-length flate bomb, and a token straddling a read boundary.
//! Hostile files are generated in the tests; corpus files check for false positives.

use lpdf_core_types::LpdfError;
use lpdf_engine_pdfcraft::guard::{
    self, preflight, scan_reader, DECLARED_STREAM_LENGTH, DECOMPRESSION_RATIO,
};
use std::io::Read;
use std::path::{Path, PathBuf};

/// A minimal PDF from object bodies (`n 0 obj … endobj` numbered from 1), with a correct xref.
fn pdf(objects: &[Vec<u8>]) -> Vec<u8> {
    let mut out = b"%PDF-1.7\n%\xe2\xe3\xcf\xd3\n".to_vec();
    let mut offsets = Vec::new();
    for (i, body) in objects.iter().enumerate() {
        offsets.push(out.len());
        out.extend_from_slice(format!("{} 0 obj\n", i + 1).as_bytes());
        out.extend_from_slice(body);
        out.extend_from_slice(b"\nendobj\n");
    }
    let xref = out.len();
    out.extend_from_slice(
        format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1).as_bytes(),
    );
    for o in offsets {
        out.extend_from_slice(format!("{o:010} 00000 n \n").as_bytes());
    }
    out.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
            objects.len() + 1
        )
        .as_bytes(),
    );
    out
}

fn page_with_contents(contents: Vec<u8>) -> Vec<u8> {
    pdf(&[
        b"<< /Type /Catalog /Pages 2 0 R >>".to_vec(),
        b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_vec(),
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents 4 0 R >>".to_vec(),
        contents,
    ])
}

fn stream(dict: &str, data: &[u8]) -> Vec<u8> {
    let mut v = format!("<< {dict} >>\nstream\n").into_bytes();
    v.extend_from_slice(data);
    v.extend_from_slice(b"\nendstream");
    v
}

fn trip_name(bytes: &[u8]) -> Option<&'static str> {
    preflight(bytes).err().map(|t| t.guard)
}

fn corpus(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../corpus")
        .join(name)
}

fn temp_dir(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!(
        "lpdf_guard_{tag}_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    std::fs::create_dir_all(&d).unwrap();
    d
}

#[test]
fn bypass_indirect_length_is_resolved() {
    let mut objs = vec![
        b"<< /Type /Catalog /Pages 2 0 R >>".to_vec(),
        b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_vec(),
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents 4 0 R >>".to_vec(),
        stream(
            "/Length 5 0 R /Filter /FlateDecode",
            b"x\x9c\x03\x00\x00\x00\x00\x01",
        ),
    ];
    objs.push(b"1073741824".to_vec());
    let bytes = pdf(&objs);
    let t = preflight(&bytes).unwrap_err();
    assert_eq!(t.guard, DECLARED_STREAM_LENGTH);
    assert!(t.detail.contains("indirect object 5"), "{t}");
    // The same file with an honest indirect length passes.
    let mut ok = objs.clone();
    ok[4] = b"8".to_vec();
    assert_eq!(trip_name(&pdf(&ok)), None);
    // An indirect length that cannot be resolved is unknown, not a trip (left to the ratio cap).
    let mut missing = objs;
    missing[3] = stream("/Length 99 0 R", b"BT ET");
    assert_eq!(guard::check_declared_stream_lengths(&pdf(&missing)), Ok(()));
}

#[test]
fn bypass_escaped_name_is_normalised() {
    for key in ["/Len#67th", "/#4Cength", "/L#65#6Egth"] {
        let bytes = page_with_contents(stream(&format!("{key} 1073741824"), b"BT ET"));
        assert_eq!(trip_name(&bytes), Some(DECLARED_STREAM_LENGTH), "{key}");
    }
    // Different names stay different: `/Length1` and `/Len#67th1` are font lengths, not checked.
    let bytes = page_with_contents(stream(
        "/Length 5 /Length1 1073741824 /Len#67th1 1073741824",
        b"BT ET",
    ));
    assert_eq!(trip_name(&bytes), None);
}

#[test]
fn bypass_honest_length_flate_bomb_trips_the_ratio_cap() {
    // 48 MiB of zeros compress to ~48 KiB: honest /Length, ratio ~1000:1, above the 32 MiB floor.
    let zeros = vec![0u8; 48 << 20];
    let packed = miniz_oxide::deflate::compress_to_vec_zlib(&zeros, 9);
    drop(zeros);
    let bytes = page_with_contents(stream(
        &format!("/Length {} /Filter /FlateDecode", packed.len()),
        &packed,
    ));
    assert_eq!(
        guard::check_declared_stream_lengths(&bytes),
        Ok(()),
        "layer 1 cannot see it"
    );
    let t = preflight(&bytes).unwrap_err();
    assert_eq!(t.guard, DECOMPRESSION_RATIO, "{t}");
    // Through the engine entry point too.
    let dir = temp_dir("bomb");
    let p = dir.join("bomb.pdf");
    std::fs::write(&p, &bytes).unwrap();
    match guard::read_guarded(&p) {
        Err(LpdfError::ResourceLimit(m)) => assert!(m.contains("decompression-ratio"), "{m}"),
        other => panic!("expected ResourceLimit, got {:?}", other.map(|b| b.len())),
    }
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn small_highly_compressible_streams_are_allowed() {
    // 4 MiB of zeros (ratio ~1000:1 but under the 32 MiB floor): a blank mask, legitimate.
    let zeros = vec![0u8; 4 << 20];
    let packed = miniz_oxide::deflate::compress_to_vec_zlib(&zeros, 9);
    let bytes = page_with_contents(stream(
        &format!("/Length {} /Filter /FlateDecode", packed.len()),
        &packed,
    ));
    assert_eq!(trip_name(&bytes), None);
}

/// A reader that yields 7 bytes at a time, so every token straddles read boundaries.
struct Trickle<'a>(&'a [u8]);
impl Read for Trickle<'_> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let n = self.0.len().min(buf.len()).min(7);
        buf[..n].copy_from_slice(&self.0[..n]);
        self.0 = &self.0[n..];
        Ok(n)
    }
}

#[test]
fn tokens_straddling_buffer_boundaries_are_seen() {
    let bomb = page_with_contents(stream("/Len#67th 1073741824", b"BT ET"));
    assert!(scan_reader(Trickle(&bomb), bomb.len() as u64).is_err());
    // Exactly across the 64 KiB boundary of the real buffer.
    let pad = guard::SCAN_BUFFER - 30;
    let mut objs = vec![
        b"<< /Type /Catalog /Pages 2 0 R >>".to_vec(),
        b"<< /Type /Pages /Kids [] /Count 0 >>".to_vec(),
    ];
    objs.push(
        [
            b"% ".as_slice(),
            &vec![b'x'; pad],
            b"\n<< /Length 1073741824 >>\nstream\nX\nendstream",
        ]
        .concat(),
    );
    let bytes = pdf(&objs);
    let at = bytes.windows(7).position(|w| w == b"/Length").unwrap_or(0);
    assert!(
        at < guard::SCAN_BUFFER && at + 18 > guard::SCAN_BUFFER,
        "token must straddle (at {at})"
    );
    let file = temp_dir("straddle").join("s.pdf");
    std::fs::write(&file, &bytes).unwrap();
    let f = std::fs::File::open(&file).unwrap();
    assert!(scan_reader(
        std::io::BufReader::with_capacity(guard::SCAN_BUFFER, f),
        bytes.len() as u64
    )
    .is_err());
}

#[test]
fn stream_data_strings_and_comments_are_not_misread() {
    let data = b"BT (/Length 99999999999) Tj ET\n/Length 99999999999 % binary-ish\n";
    let bytes = pdf(&[
        b"<< /Type /Catalog /Pages 2 0 R /Note (a /Length 999999999 in a string) >>".to_vec(),
        b"<< /Type /Pages /Kids [3 0 R] /Count 1 >> % /Length 999999999 in a comment".to_vec(),
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents 4 0 R >>".to_vec(),
        stream(&format!("/Length {}", data.len()), data),
    ]);
    assert_eq!(trip_name(&bytes), None);
}

#[test]
fn no_false_positives_on_the_corpus_and_generated_equivalents() {
    for name in [
        "standard-text.pdf",
        "standard-form.pdf",
        "synthetic-1k.pdf",
        "hostile-broken-xref.pdf",
    ] {
        let p = corpus(name);
        assert!(p.exists(), "{name} missing (corpus/fetch.sh)");
        assert_eq!(
            guard::check_file(&p).map_err(|e| e.to_string()),
            Ok(()),
            "{name}"
        );
    }
    // scanned.pdf / encrypted.pdf come from wt/release's fetch.sh; use them when present,
    // and always check generated equivalents.
    for name in ["scanned.pdf", "encrypted.pdf"] {
        let p = corpus(name);
        if p.exists() {
            assert_eq!(
                guard::check_file(&p).map_err(|e| e.to_string()),
                Ok(()),
                "{name}"
            );
        }
    }
    let dir = temp_dir("equiv");
    // Scanned equivalent: one page with a 1200x1600 8-bit grey noise image, Flate-encoded.
    let mut seed: u32 = 12345;
    let pixels: Vec<u8> = (0..1200 * 1600)
        .map(|_| {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            (seed >> 24) as u8
        })
        .collect();
    let packed = miniz_oxide::deflate::compress_to_vec_zlib(&pixels, 6);
    let scanned = pdf(&[
        b"<< /Type /Catalog /Pages 2 0 R >>".to_vec(),
        b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_vec(),
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources << /XObject << /Im0 5 0 R >> >> /Contents 4 0 R >>".to_vec(),
        stream("/Length 27", b"q 612 0 0 792 0 0 cm /Im0 Do Q"[..27].as_ref()),
        stream(
            &format!("/Type /XObject /Subtype /Image /Width 1200 /Height 1600 /ColorSpace /DeviceGray /BitsPerComponent 8 /Filter /FlateDecode /Length {}", packed.len()),
            &packed,
        ),
    ]);
    let sp = dir.join("scanned-equivalent.pdf");
    std::fs::write(&sp, &scanned).unwrap();
    assert_eq!(guard::check_file(&sp).map_err(|e| e.to_string()), Ok(()));
    // Encrypted equivalent: AES-256 with a user password, written by pdfcraft itself.
    let ep = dir.join("encrypted-equivalent.pdf");
    let adapter = lpdf_engine_pdfcraft::PdfcraftAdapter::new();
    let perms = lpdf_engine_port::Permissions {
        allow_print: true,
        allow_copy: false,
        allow_edit: false,
    };
    lpdf_engine_port::PdfEngine::protect(
        &adapter,
        &corpus("standard-text.pdf"),
        "lpdf-test",
        perms,
        &ep,
    )
    .unwrap();
    assert_eq!(guard::check_file(&ep).map_err(|e| e.to_string()), Ok(()));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn verdict_is_cached_per_path_size_and_mtime() {
    let dir = temp_dir("cache");
    let p = dir.join("c.pdf");
    std::fs::copy(corpus("standard-text.pdf"), &p).unwrap();
    let before = guard::scan_count();
    for _ in 0..5 {
        guard::read_guarded(&p).unwrap();
        guard::check_file(&p).unwrap();
    }
    let after_first = guard::scan_count();
    assert_eq!(
        after_first - before,
        1,
        "one scan for ten reads of the same file"
    );
    // Changing the file (size and mtime) invalidates the verdict.
    std::thread::sleep(std::time::Duration::from_millis(20));
    let mut bytes = std::fs::read(&p).unwrap();
    bytes.extend_from_slice(b"\n% appended\n");
    std::fs::write(&p, &bytes).unwrap();
    guard::check_file(&p).unwrap();
    assert!(guard::scan_count() > after_first);
    let _ = std::fs::remove_dir_all(&dir);
}

fn bench(label: &str, f: impl Fn()) -> f64 {
    let t = std::time::Instant::now();
    f();
    let ms = t.elapsed().as_secs_f64() * 1000.0;
    println!("bench {label}: {ms:.1} ms");
    ms
}

/// Micro-benchmark on corpus/synthetic-1k.pdf: first (uncached) check vs cached RenderPage-style rechecks.
#[test]
fn bench_synthetic_1k_first_scan_and_cached_recheck() {
    let dir = temp_dir("bench1k");
    let p = dir.join("synthetic-1k.pdf");
    std::fs::copy(corpus("synthetic-1k.pdf"), &p).unwrap();
    let len = std::fs::metadata(&p).map(|m| m.len()).unwrap_or(0);
    let structure = bench("synthetic-1k structure scan (streamed)", || {
        let f = std::fs::File::open(&p).unwrap();
        scan_reader(
            std::io::BufReader::with_capacity(guard::SCAN_BUFFER, f),
            len,
        )
        .unwrap();
    });
    let first = bench("synthetic-1k check_file first (scan + ratio)", || {
        guard::check_file(&p).unwrap()
    });
    let cached = bench("synthetic-1k check_file x100 cached", || {
        for _ in 0..100 {
            guard::check_file(&p).unwrap();
        }
    });
    println!("synthetic-1k: {len} bytes; structure {structure:.1} ms, first {first:.1} ms, cached x100 {cached:.1} ms");
    assert!(
        cached < first.max(1.0) * 10.0,
        "cached checks must not rescan"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// 100 MB synthetic file, generated in the temp dir: the structure scan streams it with a fixed
/// 64 KiB buffer. Ignored by default (disk/RAM priority to training); run with `--ignored`.
#[test]
#[ignore]
fn bench_100_mb_streaming_scan() {
    let dir = temp_dir("bench100");
    let p = dir.join("big.pdf");
    {
        use std::io::Write;
        let mut f = std::io::BufWriter::new(std::fs::File::create(&p).unwrap());
        let chunk: Vec<u8> = (0..(1 << 20)).map(|i| (i % 251) as u8).collect();
        f.write_all(b"%PDF-1.7\n1 0 obj << /Type /Catalog /Pages 2 0 R >> endobj\n2 0 obj << /Type /Pages /Kids [] /Count 0 >> endobj\n").unwrap();
        for k in 0..100 {
            f.write_all(
                format!("{} 0 obj << /Length {} >>\nstream\n", k + 3, chunk.len()).as_bytes(),
            )
            .unwrap();
            f.write_all(&chunk).unwrap();
            f.write_all(b"\nendstream endobj\n").unwrap();
        }
        f.write_all(b"trailer << /Root 1 0 R >>\n%%EOF\n").unwrap();
    }
    let len = std::fs::metadata(&p).map(|m| m.len()).unwrap_or(0);
    assert!(len > 100_000_000);
    let ms = bench("100 MB structure scan (streamed, 64 KiB buffer)", || {
        let f = std::fs::File::open(&p).unwrap();
        scan_reader(
            std::io::BufReader::with_capacity(guard::SCAN_BUFFER, f),
            len,
        )
        .unwrap();
    });
    println!(
        "100 MB: {len} bytes in {ms:.0} ms = {:.0} MB/s",
        len as f64 / 1e6 / (ms / 1000.0)
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn hostile_bomb_corpus_still_trips() {
    match guard::check_file(Path::new(&corpus("hostile-bombs.pdf"))) {
        Err(LpdfError::ResourceLimit(m)) => assert!(m.contains(DECLARED_STREAM_LENGTH), "{m}"),
        other => panic!("{other:?}"),
    }
}
