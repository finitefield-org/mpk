#[path = "support/vir_interpreter.rs"]
mod vir_interpreter;

use std::fs;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::PathBuf;

use mpk_vc::{
    evaluate_total_bitvector_operation, import_vir_json, BitVectorWidth, DivRemOperation,
    OverflowOperation, TotalBitVectorResult, VirBinaryOperator, VirModule, VirSafetyCheck, VirType,
};
use serde_json::Value;
use vir_interpreter::{
    evaluate_modeled_safety, execute, total_binary, total_convert, total_unary, ExecutionOutcome,
    ModeledPanic, RuntimeValue,
};

const RUST_FUNCTION_PREFIX: &str = "vector::";
const GO_ARITH_PREFIX: &str = "github.com/finitefield-org/mpk/fixtures/go-alpha/arith.";
const GO_BRANCH_PREFIX: &str = "github.com/finitefield-org/mpk/fixtures/go-alpha/branch.";
const GO_STRUCT_PREFIX: &str =
    "github.com/finitefield-org/mpk/fixtures/go-basic/positive/structarray.";
const GO_PAIR_ID: &str =
    "github.com/finitefield-org/mpk/fixtures/go-basic/positive/structarray.Pair";

#[test]
fn interpreter_remains_unreachable_from_production_sources() {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let support = crate_root.join("tests/support/vir_interpreter.rs");
    assert!(support.is_file(), "test-only interpreter location");
    for source in rust_sources(&crate_root.join("src")) {
        let text = fs::read_to_string(&source)
            .unwrap_or_else(|error| panic!("read {}: {error}", source.display()));
        assert!(
            !text.contains("vir_interpreter"),
            "production source references test interpreter: {}",
            source.display()
        );
    }
}

#[test]
fn total_bitvector_equations_match_exhaustively_at_eight_bits() {
    use VirBinaryOperator as Op;

    let operations = [
        Op::Eq,
        Op::NotEq,
        Op::BvAdd,
        Op::BvSub,
        Op::BvMul,
        Op::BvSdiv,
        Op::BvSrem,
        Op::BvUdiv,
        Op::BvUrem,
        Op::BvAnd,
        Op::BvOr,
        Op::BvXor,
        Op::BvShl,
        Op::BvAshr,
        Op::BvLshr,
        Op::SignedLt,
        Op::SignedLe,
        Op::SignedGt,
        Op::SignedGe,
        Op::UnsignedLt,
        Op::UnsignedLe,
        Op::UnsignedGt,
        Op::UnsignedGe,
    ];
    for operation in operations {
        let signed = matches!(
            operation,
            Op::BvSdiv
                | Op::BvSrem
                | Op::BvAshr
                | Op::SignedLt
                | Op::SignedLe
                | Op::SignedGt
                | Op::SignedGe
        );
        for lhs in 0_u64..=u8::MAX.into() {
            for rhs in 0_u64..=u8::MAX.into() {
                let lhs_value = bit_pattern(8, signed, lhs);
                let rhs_value = bit_pattern(8, signed, rhs);
                let interpreted = total_binary(operation, &lhs_value, &rhs_value);
                let expected = evaluate_total_bitvector_operation(
                    operation,
                    mpk_vc::BitVectorWidth::Bits8,
                    lhs,
                    mpk_vc::BitVectorWidth::Bits8,
                    rhs,
                )
                .expect("total operation reference");
                match expected {
                    TotalBitVectorResult::BitVector(bits) => {
                        assert_eq!(interpreted.as_unsigned(), bits, "{operation:?} {lhs} {rhs}")
                    }
                    TotalBitVectorResult::Boolean(value) => {
                        assert_eq!(interpreted.as_bool(), value, "{operation:?} {lhs} {rhs}")
                    }
                }
            }
        }
    }

    assert_eq!(
        total_binary(
            Op::BvSdiv,
            &RuntimeValue::signed(8, -7),
            &RuntimeValue::signed(8, 0),
        )
        .as_signed(),
        1
    );
    assert_eq!(
        total_binary(
            Op::BvSdiv,
            &RuntimeValue::signed(8, 7),
            &RuntimeValue::signed(8, 0),
        )
        .as_signed(),
        -1
    );
    assert_eq!(
        total_binary(
            Op::BvSrem,
            &RuntimeValue::signed(8, -7),
            &RuntimeValue::signed(8, 0),
        )
        .as_signed(),
        -7
    );
    assert_eq!(
        total_binary(
            Op::BvSdiv,
            &RuntimeValue::signed(8, i8::MIN.into()),
            &RuntimeValue::signed(8, -1),
        )
        .as_signed(),
        i128::from(i8::MIN)
    );
    assert_eq!(
        total_binary(
            Op::BvShl,
            &RuntimeValue::unsigned(8, 1),
            &RuntimeValue::unsigned(16, 9),
        )
        .as_unsigned(),
        0
    );
    assert_eq!(
        total_binary(
            Op::BvAshr,
            &RuntimeValue::signed(8, -2),
            &RuntimeValue::unsigned(16, 9),
        )
        .as_signed(),
        -1
    );

    for bits in u8::MIN..=u8::MAX {
        let signed = RuntimeValue::signed(8, (bits as i8).into());
        let unsigned = RuntimeValue::unsigned(8, bits.into());
        assert_eq!(
            total_unary(mpk_vc::VirUnaryOperator::BvNeg, &signed).as_signed(),
            i128::from((bits as i8).wrapping_neg())
        );
        assert_eq!(
            total_unary(mpk_vc::VirUnaryOperator::BvNot, &unsigned).as_unsigned(),
            u64::from(!bits)
        );
    }
    assert_eq!(
        total_unary(mpk_vc::VirUnaryOperator::Not, &RuntimeValue::bool(true)),
        RuntimeValue::bool(false)
    );

    assert_eq!(
        total_convert(
            &RuntimeValue::signed(8, -1),
            &VirType::Bv {
                width: BitVectorWidth::Bits64,
                signed: true,
            },
        )
        .as_signed(),
        -1
    );
    assert_eq!(
        total_convert(
            &RuntimeValue::unsigned(8, u8::MAX.into()),
            &VirType::Bv {
                width: BitVectorWidth::Bits64,
                signed: true,
            },
        )
        .as_signed(),
        i128::from(u8::MAX)
    );
    assert_eq!(
        total_convert(
            &RuntimeValue::unsigned(64, 0x1234),
            &VirType::Bv {
                width: BitVectorWidth::Bits8,
                signed: false,
            },
        )
        .as_unsigned(),
        0x34
    );
}

#[test]
fn rust_checked_u8_addition_and_signed_division_match_exhaustively() {
    let addition = rust_module("checked-addition", "artifacts/vir.json");
    let division = rust_module("signed-division", "artifacts/vir.json");

    for lhs in u8::MIN..=u8::MAX {
        for rhs in u8::MIN..=u8::MAX {
            let observed = scalar(execute(
                &addition,
                "vector::checked_addition",
                vec![
                    RuntimeValue::unsigned(8, lhs.into()),
                    RuntimeValue::unsigned(8, rhs.into()),
                ],
            ));
            match lhs.checked_add(rhs) {
                Some(value) => assert_eq!(observed.unwrap().as_unsigned(), u64::from(value)),
                None => assert_eq!(observed, Err(ModeledPanic::IntegerOverflow)),
            }

            let signed_lhs = lhs as i8;
            let signed_rhs = rhs as i8;
            let observed = scalar(execute(
                &division,
                "vector::signed_division",
                vec![
                    RuntimeValue::signed(8, signed_lhs.into()),
                    RuntimeValue::signed(8, signed_rhs.into()),
                ],
            ));
            match signed_lhs.checked_div(signed_rhs) {
                Some(value) => assert_eq!(observed.unwrap().as_signed(), i128::from(value)),
                None if signed_rhs == 0 => {
                    assert_eq!(observed, Err(ModeledPanic::DivisionByZero))
                }
                None => assert_eq!(observed, Err(ModeledPanic::SignedDivisionOverflow)),
            }
        }
    }
}

#[test]
fn every_rust_positive_category_matches_values_and_modeled_panics() {
    let short_circuit = rust_module("boolean-short-circuit", "artifacts/vir.json");
    for enabled in [false, true] {
        for left in [0, 1, 7, u32::MAX] {
            for right in [0, 1, 3, u32::MAX] {
                assert_same_bool(
                    execute(
                        &short_circuit,
                        "vector::boolean_short_circuit",
                        vec![
                            RuntimeValue::bool(enabled),
                            RuntimeValue::unsigned(32, left.into()),
                            RuntimeValue::unsigned(32, right.into()),
                        ],
                    ),
                    native(|| rust_boolean_short_circuit(enabled, left, right)),
                );
            }
        }
    }

    let maximum = rust_module("signed-unsigned-max", "artifacts/vir.json");
    let mut state = 0x6a09_e667_f3bc_c909_u64;
    for _ in 0..4_096 {
        state = next_state(state);
        let unsigned_left = state as u8;
        let unsigned_right = (state >> 8) as u8;
        let signed_left = (state >> 16) as u8 as i8;
        let signed_right = (state >> 24) as u8 as i8;
        assert_unsigned(
            execute(
                &maximum,
                "vector::max_values",
                vec![
                    RuntimeValue::unsigned(8, unsigned_left.into()),
                    RuntimeValue::unsigned(8, unsigned_right.into()),
                    RuntimeValue::signed(8, signed_left.into()),
                    RuntimeValue::signed(8, signed_right.into()),
                ],
            ),
            Ok(rust_max_values(unsigned_left, unsigned_right, signed_left, signed_right).into()),
        );
    }

    let minimum = rust_module("minimum-negation", "artifacts/vir.json");
    let early = rust_module("early-return", "artifacts/vir.json");
    for bits in u8::MIN..=u8::MAX {
        let value = bits as i8;
        for choose_minimum in [false, true] {
            let expected = if choose_minimum {
                Ok(i8::MIN)
            } else {
                value.checked_neg().ok_or(ModeledPanic::IntegerOverflow)
            };
            assert_signed(
                execute(
                    &minimum,
                    "vector::minimum_or_negate",
                    vec![
                        RuntimeValue::signed(8, value.into()),
                        RuntimeValue::bool(choose_minimum),
                    ],
                ),
                expected.map(i128::from),
            );

            let expected = if choose_minimum {
                Ok(value)
            } else {
                value.checked_neg().ok_or(ModeledPanic::IntegerOverflow)
            };
            assert_signed(
                execute(
                    &early,
                    "vector::early_return",
                    vec![
                        RuntimeValue::signed(8, value.into()),
                        RuntimeValue::bool(choose_minimum),
                    ],
                ),
                expected.map(i128::from),
            );
        }
    }

    let shifts = rust_module("cross-width-shifts", "artifacts/vir.json");
    let shift_cases = [
        (0x8123_4567, 0, 0, 0, 0),
        (0x8123_4567, 3, 2, 1, 4),
        (1, 31, 31, 31, 31),
        (1, 32, 0, 0, 0),
        (1, 0, -1, 0, 0),
        (1, 0, 0, -1, 0),
        (1, 0, 0, 0, 32),
        (u32::MAX, u8::MAX, i64::MAX, i8::MIN, u64::MAX),
    ];
    for (value, narrow_unsigned, wide_signed, narrow_signed, wide_unsigned) in shift_cases {
        let observed = execute(
            &shifts,
            "vector::cross_width_shifts",
            vec![
                RuntimeValue::unsigned(32, value.into()),
                RuntimeValue::unsigned(8, narrow_unsigned.into()),
                RuntimeValue::signed(64, wide_signed.into()),
                RuntimeValue::signed(8, narrow_signed.into()),
                RuntimeValue::unsigned(64, wide_unsigned),
            ],
        );
        assert_unsigned(
            observed,
            checked_cross_width_shifts(
                value,
                narrow_unsigned,
                wide_signed,
                narrow_signed,
                wide_unsigned,
            )
            .map(u64::from),
        );
    }

    let array = rust_module("array-bounds", "artifacts/vir.json");
    let usize32 = rust_module("usize-targets", "artifacts/i686/vir.json");
    let usize64 = rust_module("usize-targets", "artifacts/x86_64/vir.json");
    let elements = [3_u8, 5, 8, 13];
    for index in 0_u64..=5 {
        let expected = elements
            .get(index as usize)
            .copied()
            .map(u64::from)
            .ok_or(ModeledPanic::IndexOutOfBounds);
        let args64 = || {
            vec![
                RuntimeValue::array(elements.map(|value| RuntimeValue::unsigned(8, value.into()))),
                RuntimeValue::unsigned(64, index),
            ]
        };
        assert_unsigned(execute(&array, "vector::array_bounds", args64()), expected);
        assert_unsigned(execute(&usize64, "vector::usize_index", args64()), expected);
        let args32 = vec![
            RuntimeValue::array(elements.map(|value| RuntimeValue::unsigned(8, value.into()))),
            RuntimeValue::unsigned(32, index),
        ];
        assert_unsigned(execute(&usize32, "vector::usize_index", args32), expected);
    }

    let structure = rust_module("struct-move", "artifacts/vir.json");
    for (left, right) in [(0, 0), (1, 2), (100, 155), (200, 56), (255, 0), (255, 1)] {
        assert_unsigned(
            execute(
                &structure,
                "vector::construct_move_read",
                vec![
                    RuntimeValue::unsigned(8, left),
                    RuntimeValue::unsigned(8, right),
                ],
            ),
            (left as u8)
                .checked_add(right as u8)
                .map(u64::from)
                .ok_or(ModeledPanic::IntegerOverflow),
        );
    }

    for (fixture, function) in [
        ("module-calls", "cross_module"),
        ("multi-file-closure", "multi_file"),
    ] {
        let module = rust_module(fixture, "artifacts/vir.json");
        for value in u8::MIN..=u8::MAX {
            assert_unsigned(
                execute(
                    &module,
                    &format!("{RUST_FUNCTION_PREFIX}{function}"),
                    vec![RuntimeValue::unsigned(8, value.into())],
                ),
                Ok(value.into()),
            );
        }
    }
}

#[test]
fn native_rust_panics_on_the_same_guarded_edges() {
    assert_eq!(native(|| rust_checked_addition(3, 4)), Ok(7));
    assert_eq!(native(|| rust_checked_addition(u8::MAX, 1)), Err(()));
    assert_eq!(native(|| rust_signed_division(-12, 3)), Ok(-4));
    assert_eq!(native(|| rust_signed_division(1, 0)), Err(()));
    assert_eq!(native(|| rust_signed_division(i8::MIN, -1)), Err(()));
    assert_eq!(native(|| rust_boolean_short_circuit(false, 1, 0)), Ok(true));
    assert_eq!(native(|| rust_boolean_short_circuit(true, 1, 0)), Err(()));
    assert_eq!(native(|| rust_early_return(i8::MIN, true)), Ok(i8::MIN));
    assert_eq!(native(|| rust_early_return(i8::MIN, false)), Err(()));
    assert_eq!(native(|| rust_array_bounds([1, 2, 3, 4], 3)), Ok(4));
    assert_eq!(native(|| rust_array_bounds([1, 2, 3, 4], 4)), Err(()));
    assert_eq!(native(|| rust_cross_width_shifts(1, 32, 0, 0, 0)), Err(()));
    assert_eq!(native(|| rust_cross_width_shifts(1, 0, -1, 0, 0)), Err(()));
}

#[test]
fn migrated_go_operations_branches_and_aggregates_match() {
    let arithmetic = go_module("alpha-arith");
    for lhs_bits in u8::MIN..=u8::MAX {
        for rhs_bits in u8::MIN..=u8::MAX {
            let lhs = lhs_bits as i8;
            let rhs = rhs_bits as i8;
            for (name, expected) in [
                ("Add8", lhs.wrapping_add(rhs)),
                ("Sub8", lhs.wrapping_sub(rhs)),
            ] {
                assert_signed(
                    execute(
                        &arithmetic,
                        &format!("{GO_ARITH_PREFIX}{name}"),
                        vec![
                            RuntimeValue::signed(8, lhs.into()),
                            RuntimeValue::signed(8, rhs.into()),
                        ],
                    ),
                    Ok(expected.into()),
                );
            }
            for (name, expected) in [
                ("AndU8", lhs_bits & rhs_bits),
                ("OrU8", lhs_bits | rhs_bits),
                ("XorU8", lhs_bits ^ rhs_bits),
            ] {
                assert_unsigned(
                    execute(
                        &arithmetic,
                        &format!("{GO_ARITH_PREFIX}{name}"),
                        vec![
                            RuntimeValue::unsigned(8, lhs_bits.into()),
                            RuntimeValue::unsigned(8, rhs_bits.into()),
                        ],
                    ),
                    Ok(expected.into()),
                );
            }
        }
        assert_unsigned(
            execute(
                &arithmetic,
                &format!("{GO_ARITH_PREFIX}NotU8"),
                vec![RuntimeValue::unsigned(8, lhs_bits.into())],
            ),
            Ok((!lhs_bits).into()),
        );
    }

    for count in [0, 1, 31, 63, 64, 65, u64::MAX] {
        for value in [0, 1, 0x8000_0000_0000_0000, u64::MAX] {
            assert_unsigned(
                execute(
                    &arithmetic,
                    &format!("{GO_ARITH_PREFIX}ShiftLeftU64"),
                    vec![
                        RuntimeValue::unsigned(64, value),
                        RuntimeValue::unsigned(8, count),
                    ],
                ),
                Ok(if count < 64 { value << count } else { 0 }),
            );
            assert_unsigned(
                execute(
                    &arithmetic,
                    &format!("{GO_ARITH_PREFIX}ShiftRightU64"),
                    vec![
                        RuntimeValue::unsigned(64, value),
                        RuntimeValue::unsigned(8, count),
                    ],
                ),
                Ok(if count < 64 { value >> count } else { 0 }),
            );
        }
    }

    for left in [false, true] {
        for right in [false, true] {
            assert_bool(
                execute(
                    &arithmetic,
                    &format!("{GO_ARITH_PREFIX}BoolAnd"),
                    vec![RuntimeValue::bool(left), RuntimeValue::bool(right)],
                ),
                Ok(left && right),
            );
        }
    }

    let branches = go_module("alpha-branch");
    for lhs_bits in u8::MIN..=u8::MAX {
        for rhs_bits in u8::MIN..=u8::MAX {
            let lhs = lhs_bits as i8;
            let rhs = rhs_bits as i8;
            assert_signed(
                execute(
                    &branches,
                    &format!("{GO_BRANCH_PREFIX}Max8"),
                    vec![
                        RuntimeValue::signed(8, lhs.into()),
                        RuntimeValue::signed(8, rhs.into()),
                    ],
                ),
                Ok(i128::from(lhs.max(rhs))),
            );
        }
    }

    let aggregates = go_module("basic-structarray");
    let pair = RuntimeValue::structure(
        GO_PAIR_ID,
        [
            ("Left", RuntimeValue::signed(64, -12)),
            ("Right", RuntimeValue::signed(64, 20)),
        ],
    );
    assert_eq!(
        scalar(execute(
            &aggregates,
            &format!("{GO_STRUCT_PREFIX}BuildPair"),
            vec![RuntimeValue::signed(64, -12), RuntimeValue::signed(64, 20)],
        )),
        Ok(pair.clone())
    );
    assert_signed(
        execute(
            &aggregates,
            &format!("{GO_STRUCT_PREFIX}SumPair"),
            vec![pair],
        ),
        Ok(8),
    );
    let array = RuntimeValue::array([RuntimeValue::signed(64, -5), RuntimeValue::signed(64, 9)]);
    assert_eq!(
        scalar(execute(
            &aggregates,
            &format!("{GO_STRUCT_PREFIX}BuildArray"),
            vec![RuntimeValue::signed(64, -5), RuntimeValue::signed(64, 9)],
        )),
        Ok(array.clone())
    );
    assert_signed(
        execute(
            &aggregates,
            &format!("{GO_STRUCT_PREFIX}PickFirst"),
            vec![array],
        ),
        Ok(-5),
    );
}

#[test]
fn intentional_go_rust_differences_match_the_frozen_profile_vectors() {
    let vectors: Value = serde_json::from_slice(
        &fs::read(repo_root().join("develop/specs/vectors/vir-v0.json")).expect("read VIR vectors"),
    )
    .expect("parse VIR vectors");
    for (id, result, checks) in [
        ("profile.go_signed_min_div_negative_one", "-128", 1),
        ("profile.rust_signed_min_div_negative_one", "-128", 2),
        ("profile.go_negative_signed_shift_count", "0", 1),
        ("profile.rust_negative_signed_shift_count", "0", 2),
    ] {
        let case = profile_case(&vectors, id);
        assert_eq!(case["expect"]["result"].as_str(), Some(result));
        assert_eq!(
            case["expect"]["check_results"]
                .as_array()
                .expect("check results")
                .len(),
            checks
        );
    }

    let minimum = RuntimeValue::signed(8, i8::MIN.into());
    let minus_one = RuntimeValue::signed(8, -1);
    let go_division = [VirSafetyCheck::DivisorNonzero {}];
    let rust_division = [
        VirSafetyCheck::DivisorNonzero {},
        VirSafetyCheck::SignedDivremRepresentable {
            operation: DivRemOperation::Div,
        },
    ];
    assert_eq!(
        evaluate_modeled_safety(&go_division, &minimum, Some(&minus_one)),
        Ok(())
    );
    assert_eq!(
        evaluate_modeled_safety(&rust_division, &minimum, Some(&minus_one)),
        Err(ModeledPanic::SignedDivisionOverflow)
    );

    let add_left = RuntimeValue::signed(8, i8::MAX.into());
    let add_right = RuntimeValue::signed(8, 1);
    let rust_addition = [VirSafetyCheck::IntegerNoOverflow {
        operation: OverflowOperation::Add,
        signed: true,
    }];
    assert_eq!(
        total_binary(VirBinaryOperator::BvAdd, &add_left, &add_right).as_signed(),
        i128::from(i8::MIN)
    );
    assert_eq!(
        evaluate_modeled_safety(&[], &add_left, Some(&add_right)),
        Ok(())
    );
    assert_eq!(
        evaluate_modeled_safety(&rust_addition, &add_left, Some(&add_right)),
        Err(ModeledPanic::IntegerOverflow)
    );

    let shift_value = RuntimeValue::unsigned(8, 1);
    let over_width = RuntimeValue::unsigned(16, 9);
    let rust_shift = [VirSafetyCheck::ShiftCountLessThanWidth {}];
    assert_eq!(
        total_binary(VirBinaryOperator::BvShl, &shift_value, &over_width).as_unsigned(),
        0
    );
    assert_eq!(
        evaluate_modeled_safety(&[], &shift_value, Some(&over_width)),
        Ok(())
    );
    assert_eq!(
        evaluate_modeled_safety(&rust_shift, &shift_value, Some(&over_width)),
        Err(ModeledPanic::ShiftOutOfRange)
    );

    let negative = RuntimeValue::signed(8, -1);
    let signed_shift = [VirSafetyCheck::ShiftCountNonnegative {}];
    assert_eq!(
        evaluate_modeled_safety(&signed_shift, &shift_value, Some(&negative)),
        Err(ModeledPanic::NegativeShift)
    );

    let array = RuntimeValue::array([RuntimeValue::bool(true), RuntimeValue::bool(false)]);
    let bounds = [VirSafetyCheck::IndexInBounds {}];
    assert_eq!(
        evaluate_modeled_safety(&bounds, &array, Some(&RuntimeValue::signed(8, -1))),
        Err(ModeledPanic::IndexOutOfBounds)
    );
    assert_eq!(
        evaluate_modeled_safety(&bounds, &array, Some(&RuntimeValue::unsigned(8, 1))),
        Ok(())
    );
    assert_eq!(
        evaluate_modeled_safety(&bounds, &array, Some(&RuntimeValue::unsigned(8, 2))),
        Err(ModeledPanic::IndexOutOfBounds)
    );
}

fn rust_module(fixture: &str, artifact: &str) -> VirModule {
    load_module(
        repo_root()
            .join("fixtures/rust-basic/positive")
            .join(fixture)
            .join(artifact),
    )
}

fn go_module(fixture: &str) -> VirModule {
    load_module(
        repo_root()
            .join("fixtures/vir-go/frontend")
            .join(fixture)
            .join("vir.json"),
    )
}

fn load_module(path: PathBuf) -> VirModule {
    import_vir_json(
        &fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display())),
    )
    .unwrap_or_else(|error| panic!("import {}: {error}", path.display()))
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn rust_sources(directory: &std::path::Path) -> Vec<PathBuf> {
    let mut sources = Vec::new();
    let mut entries = fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("read {}: {error}", directory.display()))
        .map(|entry| entry.expect("source directory entry"))
        .collect::<Vec<_>>();
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let path = entry.path();
        if entry.file_type().expect("source file type").is_dir() {
            sources.extend(rust_sources(&path));
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
            sources.push(path);
        }
    }
    sources
}

fn profile_case<'a>(vectors: &'a Value, id: &str) -> &'a Value {
    vectors["profile_cases"]
        .as_array()
        .expect("profile cases")
        .iter()
        .find(|case| case["id"].as_str() == Some(id))
        .unwrap_or_else(|| panic!("profile case {id}"))
}

fn scalar(outcome: ExecutionOutcome) -> Result<RuntimeValue, ModeledPanic> {
    match outcome {
        ExecutionOutcome::Returned(mut values) => {
            assert_eq!(values.len(), 1);
            Ok(values.pop().unwrap())
        }
        ExecutionOutcome::Panicked(panic) => Err(panic),
    }
}

fn assert_unsigned(outcome: ExecutionOutcome, expected: Result<u64, ModeledPanic>) {
    assert_eq!(scalar(outcome).map(|value| value.as_unsigned()), expected);
}

fn assert_signed(outcome: ExecutionOutcome, expected: Result<i128, ModeledPanic>) {
    assert_eq!(scalar(outcome).map(|value| value.as_signed()), expected);
}

fn assert_bool(outcome: ExecutionOutcome, expected: Result<bool, ModeledPanic>) {
    assert_eq!(scalar(outcome).map(|value| value.as_bool()), expected);
}

fn assert_same_bool(outcome: ExecutionOutcome, expected: Result<bool, ()>) {
    match expected {
        Ok(value) => assert_bool(outcome, Ok(value)),
        Err(()) => assert!(matches!(outcome, ExecutionOutcome::Panicked(_))),
    }
}

fn bit_pattern(width: u32, signed: bool, bits: u64) -> RuntimeValue {
    if signed {
        RuntimeValue::signed(width, bits as i8 as i128)
    } else {
        RuntimeValue::unsigned(width, bits)
    }
}

fn next_state(mut state: u64) -> u64 {
    state ^= state << 13;
    state ^= state >> 7;
    state ^ (state << 17)
}

fn native<T>(function: impl FnOnce() -> T) -> Result<T, ()> {
    catch_unwind(AssertUnwindSafe(function)).map_err(|_| ())
}

fn checked_cross_width_shifts(
    value: u32,
    narrow_unsigned: u8,
    wide_signed: i64,
    narrow_signed: i8,
    wide_unsigned: u64,
) -> Result<u32, ModeledPanic> {
    let left_narrow = value
        .checked_shl(narrow_unsigned.into())
        .ok_or(ModeledPanic::ShiftOutOfRange)?;
    if wide_signed < 0 {
        return Err(ModeledPanic::NegativeShift);
    }
    let right_wide = left_narrow
        .checked_shr(wide_signed as u32)
        .ok_or(ModeledPanic::ShiftOutOfRange)?;
    if narrow_signed < 0 {
        return Err(ModeledPanic::NegativeShift);
    }
    let left_signed = right_wide
        .checked_shl(narrow_signed as u32)
        .ok_or(ModeledPanic::ShiftOutOfRange)?;
    let count = u32::try_from(wide_unsigned).unwrap_or(u32::MAX);
    left_signed
        .checked_shr(count)
        .ok_or(ModeledPanic::ShiftOutOfRange)
}

#[inline(never)]
fn rust_checked_addition(left: u8, right: u8) -> u8 {
    left + right
}

#[inline(never)]
fn rust_signed_division(left: i8, right: i8) -> i8 {
    left / right
}

#[inline(never)]
fn rust_boolean_short_circuit(enabled: bool, left: u32, right: u32) -> bool {
    let identity = enabled;
    let negated = !enabled;
    if identity {
        identity && ((left / right) > 0)
    } else {
        negated || ((left / right) > 0)
    }
}

fn rust_max_values(unsigned_left: u8, unsigned_right: u8, signed_left: i8, signed_right: i8) -> u8 {
    let signed = signed_left.max(signed_right);
    if signed >= 0 {
        unsigned_left.max(unsigned_right)
    } else if unsigned_left > unsigned_right {
        unsigned_left
    } else {
        unsigned_right
    }
}

#[inline(never)]
fn rust_early_return(value: i8, return_early: bool) -> i8 {
    if return_early {
        return value;
    }
    -value
}

#[inline(never)]
fn rust_array_bounds(values: [u8; 4], index: usize) -> u8 {
    values[index]
}

#[inline(never)]
fn rust_cross_width_shifts(
    value: u32,
    narrow_unsigned: u8,
    wide_signed: i64,
    narrow_signed: i8,
    wide_unsigned: u64,
) -> u32 {
    let left_narrow = value << narrow_unsigned;
    let right_wide = left_narrow >> wide_signed;
    let left_signed = right_wide << narrow_signed;
    left_signed >> wide_unsigned
}
