#![no_main]

use libfuzzer_sys::fuzz_target;
use lpdf_engine_switcher::fuzz::fuzz_fs_and_ai_rpc_bytes;

fuzz_target!(|data: &[u8]| {
    fuzz_fs_and_ai_rpc_bytes(data);
});
