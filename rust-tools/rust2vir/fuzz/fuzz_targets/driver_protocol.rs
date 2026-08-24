#![no_main]

use libfuzzer_sys::fuzz_target;
use rust2vir_internal::fuzz_entrypoints::exercise_driver_protocol;

fuzz_target!(|data: &[u8]| exercise_driver_protocol(data));
