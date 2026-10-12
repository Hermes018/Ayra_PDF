#![no_main]

use libfuzzer_sys::fuzz_target;
use lpdf_engine_switcher::fuzz::fuzz_engine_switcher_bytes;

fuzz_target!(|data: &[u8]| {
    fuzz_engine_switcher_bytes(data);
});
