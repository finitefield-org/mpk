#![no_main]

use libfuzzer_sys::fuzz_target;
use rust2vir_internal::fuzz_entrypoints::exercise_rust_contract;

fuzz_target!(|data: &[u8]| exercise_rust_contract(data));
