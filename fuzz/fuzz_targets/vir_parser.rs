#![no_main]

use libfuzzer_sys::fuzz_target;
use mpk_vc::import_vir_json;

const MAX_FUZZ_INPUT: usize = 1_048_576;

fuzz_target!(|data: &[u8]| {
    let data = &data[..data.len().min(MAX_FUZZ_INPUT)];
    let first = import_vir_json(data);
    let second = import_vir_json(data);
    assert_eq!(first, second);
});
