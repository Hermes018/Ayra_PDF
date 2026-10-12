#![no_main]

use libfuzzer_sys::fuzz_target;
use lpdf_engine_switcher::fuzz::fuzz_fallback_bytes;

fuzz_target!(|data: &[u8]| {
    fuzz_fallback_bytes(data);
});
