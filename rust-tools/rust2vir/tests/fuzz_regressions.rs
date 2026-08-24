use rust2vir_internal::fuzz_entrypoints::{exercise_driver_protocol, exercise_rust_contract};

#[test]
fn checked_in_driver_protocol_regressions_are_parser_only() {
    for seed in [
        include_bytes!("../fuzz/seeds/driver_protocol/duplicate-key.json").as_slice(),
        include_bytes!("../fuzz/seeds/driver_protocol/truncated.json").as_slice(),
        include_bytes!("../fuzz/seeds/driver_protocol/valid-lowered.json").as_slice(),
        include_bytes!("../fuzz/seeds/driver_protocol/valid-request.json").as_slice(),
    ] {
        exercise_driver_protocol(seed);
    }
}

#[test]
fn checked_in_contract_regressions_are_parser_only() {
    for seed in [
        include_bytes!("../fuzz/seeds/rust_contract/depth-boundary.json").as_slice(),
        include_bytes!("../fuzz/seeds/rust_contract/duplicate-key.json").as_slice(),
        include_bytes!("../fuzz/seeds/rust_contract/valid-identity.json").as_slice(),
        include_bytes!("../fuzz/seeds/rust_contract/whitespace.json").as_slice(),
    ] {
        exercise_rust_contract(seed);
    }
}
