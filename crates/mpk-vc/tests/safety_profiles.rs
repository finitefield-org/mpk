use std::fs;
use std::path::{Path, PathBuf};

use mpk_cert::decode_canonical_certificate;
use mpk_vc::vir::{VirIntegerLiteral, VirVariableRef};
use mpk_vc::{
    encode_instruction_safety, evaluate_total_bitvector_operation, required_safety_checks,
    validate_safety_check_sequence, ArrayLength, BitVectorWidth, DecimalInteger, DivRemOperation,
    EncodedSafetyPredicate, GoFixedParameters, MpkExprTerm, OverflowMode, OverflowOperation,
    PanicMode, PointerWidth, ProgramExprContext, RustCheckedParameters, SafetyEvidenceRoute,
    SafetyObligationKind, SemanticParameters, SemanticProfile, TotalBitVectorResult,
    VirBinaryOperator, VirInstruction, VirInstructionKind, VirIntLiteral, VirSafetyCheck,
    VirSafetyOperation, VirType, VirUnaryOperator, VirValue, SAFETY_BITVEC_THEORY_FORMAT,
    SAFETY_GROUPED_CERTIFICATE_FOUNDATION, SAFETY_OBLIGATION_KIND_COMPONENT,
};
use serde_json::Value;

const RUST_FRONTEND_ARITHMETIC_VIR: &[u8] =
    include_bytes!("../../../rust-tools/rust2vir/testdata/arithmetic/expected-vir.json");

#[test]
fn rust_frontend_checked_arithmetic_fixture_is_profile_complete() {
    let modules: Vec<Value> =
        serde_json::from_slice(RUST_FRONTEND_ARITHMETIC_VIR).expect("arithmetic VIR fixture");
    assert!(!modules.is_empty());
    for module in modules {
        let bytes = serde_json::to_vec(&module).expect("serialize fixture module");
        mpk_vc::import_vir_json(&bytes).expect("frontend VIR passes independent validation");
    }
}

fn go_parameters() -> SemanticParameters {
    SemanticParameters::GoFixed(GoFixedParameters {
        target_id: "linux/amd64".to_owned(),
        pointer_width: PointerWidth::Bits64,
    })
}

fn rust_parameters(pointer_width: PointerWidth) -> SemanticParameters {
    SemanticParameters::RustChecked(RustCheckedParameters {
        target_id: match pointer_width {
            PointerWidth::Bits32 => "i686-unknown-linux-gnu",
            PointerWidth::Bits64 => "x86_64-unknown-linux-gnu",
        }
        .to_owned(),
        pointer_width,
        overflow_mode: OverflowMode::Checked,
        panic_mode: PanicMode::Abort,
    })
}

fn bv(width: BitVectorWidth, signed: bool) -> VirType {
    VirType::Bv { width, signed }
}

fn int(value: impl ToString, width: BitVectorWidth, signed: bool) -> VirValue {
    VirValue::Integer(VirIntegerLiteral {
        int: VirIntLiteral {
            value: DecimalInteger::new(value.to_string()).expect("canonical integer"),
            width,
            signed,
        },
    })
}

fn variable(id: &str) -> VirValue {
    VirValue::Variable(VirVariableRef { var: id.to_owned() })
}

fn overflow(operation: OverflowOperation, signed: bool) -> VirSafetyCheck {
    VirSafetyCheck::IntegerNoOverflow { operation, signed }
}

fn binary_instruction(
    op: VirBinaryOperator,
    r#type: VirType,
    lhs: VirValue,
    rhs: VirValue,
    safety_checks: Vec<VirSafetyCheck>,
) -> VirInstruction {
    VirInstruction::BinOp {
        id: "t0".to_owned(),
        op,
        r#type,
        lhs,
        rhs,
        safety_checks,
    }
}

fn unary_instruction(
    r#type: VirType,
    value: VirValue,
    safety_checks: Vec<VirSafetyCheck>,
) -> VirInstruction {
    VirInstruction::UnaryOp {
        id: "t0".to_owned(),
        op: VirUnaryOperator::BvNeg,
        r#type,
        value,
        safety_checks,
    }
}

fn context(profile: SemanticProfile) -> ProgramExprContext {
    let parameters = match profile {
        SemanticProfile::GoFixedV0 => go_parameters(),
        SemanticProfile::RustCheckedV0 => rust_parameters(PointerWidth::Bits64),
    };
    ProgramExprContext::new(profile, parameters, Vec::new()).expect("semantic context")
}

#[test]
fn required_check_matrix_is_exact_at_every_width() {
    use BitVectorWidth::{Bits16, Bits32, Bits64, Bits8};
    use OverflowOperation::{Add, Mul, Neg, Sub};
    use VirBinaryOperator::{
        BvAdd, BvAshr, BvLshr, BvMul, BvSdiv, BvShl, BvSrem, BvSub, BvUdiv, BvUrem,
    };

    for width in [Bits8, Bits16, Bits32, Bits64] {
        for signed in [false, true] {
            let ty = bv(width, signed);
            for (operator, operation) in [(BvAdd, Add), (BvSub, Sub), (BvMul, Mul)] {
                assert_eq!(
                    required_safety_checks(
                        SemanticProfile::GoFixedV0,
                        VirSafetyOperation::Binary(operator),
                        &[ty.clone(), ty.clone()],
                    )
                    .unwrap(),
                    Vec::<VirSafetyCheck>::new()
                );
                assert_eq!(
                    required_safety_checks(
                        SemanticProfile::RustCheckedV0,
                        VirSafetyOperation::Binary(operator),
                        &[ty.clone(), ty.clone()],
                    )
                    .unwrap(),
                    vec![overflow(operation, signed)]
                );
            }
        }

        let signed = bv(width, true);
        assert!(required_safety_checks(
            SemanticProfile::GoFixedV0,
            VirSafetyOperation::Unary(VirUnaryOperator::BvNeg),
            std::slice::from_ref(&signed),
        )
        .unwrap()
        .is_empty());
        assert_eq!(
            required_safety_checks(
                SemanticProfile::RustCheckedV0,
                VirSafetyOperation::Unary(VirUnaryOperator::BvNeg),
                std::slice::from_ref(&signed),
            )
            .unwrap(),
            vec![overflow(Neg, true)]
        );
        for (operator, operation) in [
            (BvSdiv, DivRemOperation::Div),
            (BvSrem, DivRemOperation::Rem),
        ] {
            assert_eq!(
                required_safety_checks(
                    SemanticProfile::GoFixedV0,
                    VirSafetyOperation::Binary(operator),
                    &[signed.clone(), signed.clone()],
                )
                .unwrap(),
                vec![VirSafetyCheck::DivisorNonzero {}]
            );
            assert_eq!(
                required_safety_checks(
                    SemanticProfile::RustCheckedV0,
                    VirSafetyOperation::Binary(operator),
                    &[signed.clone(), signed.clone()],
                )
                .unwrap(),
                vec![
                    VirSafetyCheck::DivisorNonzero {},
                    VirSafetyCheck::SignedDivremRepresentable { operation }
                ]
            );
        }

        let unsigned = bv(width, false);
        assert!(required_safety_checks(
            SemanticProfile::GoFixedV0,
            VirSafetyOperation::Unary(VirUnaryOperator::BvNeg),
            std::slice::from_ref(&unsigned),
        )
        .unwrap()
        .is_empty());
        for operator in [BvUdiv, BvUrem] {
            for profile in [SemanticProfile::GoFixedV0, SemanticProfile::RustCheckedV0] {
                assert_eq!(
                    required_safety_checks(
                        profile,
                        VirSafetyOperation::Binary(operator),
                        &[unsigned.clone(), unsigned.clone()],
                    )
                    .unwrap(),
                    vec![VirSafetyCheck::DivisorNonzero {}]
                );
            }
        }
    }

    let lhs_signed = bv(Bits8, true);
    let lhs_unsigned = bv(Bits8, false);
    let rhs_signed = bv(Bits16, true);
    let rhs_unsigned = bv(Bits64, false);
    for (operator, lhs) in [
        (BvShl, &lhs_signed),
        (BvAshr, &lhs_signed),
        (BvLshr, &lhs_unsigned),
    ] {
        assert_eq!(
            required_safety_checks(
                SemanticProfile::GoFixedV0,
                VirSafetyOperation::Binary(operator),
                &[lhs.clone(), rhs_signed.clone()],
            )
            .unwrap(),
            vec![VirSafetyCheck::ShiftCountNonnegative {}]
        );
        assert_eq!(
            required_safety_checks(
                SemanticProfile::RustCheckedV0,
                VirSafetyOperation::Binary(operator),
                &[lhs.clone(), rhs_signed.clone()],
            )
            .unwrap(),
            vec![
                VirSafetyCheck::ShiftCountNonnegative {},
                VirSafetyCheck::ShiftCountLessThanWidth {}
            ]
        );
        assert_eq!(
            required_safety_checks(
                SemanticProfile::RustCheckedV0,
                VirSafetyOperation::Binary(operator),
                &[lhs.clone(), rhs_unsigned.clone()],
            )
            .unwrap(),
            vec![VirSafetyCheck::ShiftCountLessThanWidth {}]
        );
        assert!(required_safety_checks(
            SemanticProfile::GoFixedV0,
            VirSafetyOperation::Binary(operator),
            &[lhs.clone(), rhs_unsigned.clone()],
        )
        .unwrap()
        .is_empty());
    }

    let array = VirType::Array {
        length: ArrayLength::try_from(2).unwrap(),
        element: Box::new(VirType::Bool {}),
    };
    assert_eq!(
        required_safety_checks(
            SemanticProfile::GoFixedV0,
            VirSafetyOperation::Index,
            &[array.clone(), bv(Bits8, true)],
        )
        .unwrap(),
        vec![VirSafetyCheck::IndexInBounds {}]
    );
    assert_eq!(
        required_safety_checks(
            SemanticProfile::RustCheckedV0,
            VirSafetyOperation::Index,
            &[array, bv(Bits64, false)],
        )
        .unwrap(),
        vec![VirSafetyCheck::IndexInBounds {}]
    );
    assert!(required_safety_checks(
        SemanticProfile::GoFixedV0,
        VirSafetyOperation::None(VirInstructionKind::Const),
        &[],
    )
    .unwrap()
    .is_empty());
}

#[test]
fn division_shift_and_index_predicates_match_boundaries_at_every_width() {
    use BitVectorWidth::{Bits16, Bits32, Bits64, Bits8};
    use VirBinaryOperator::{BvAshr, BvLshr, BvShl};

    for width in [Bits8, Bits16, Bits32, Bits64] {
        let minimum = -(1_i128 << (width.bits() - 1));
        for (operator, operation) in [
            (VirBinaryOperator::BvSdiv, DivRemOperation::Div),
            (VirBinaryOperator::BvSrem, DivRemOperation::Rem),
        ] {
            let instruction = binary_instruction(
                operator,
                bv(width, true),
                int(minimum, width, true),
                int(-1, width, true),
                vec![
                    VirSafetyCheck::DivisorNonzero {},
                    VirSafetyCheck::SignedDivremRepresentable { operation },
                ],
            );
            let predicates =
                encode_instruction_safety(&context(SemanticProfile::RustCheckedV0), &instruction)
                    .unwrap();
            assert!(eval_bool(&predicates[0].proposition));
            assert!(!eval_bool(&predicates[1].proposition));
        }

        for operator in [BvShl, BvAshr, BvLshr] {
            let lhs_signed = operator == BvAshr;
            for (count, expected_nonnegative, expected_in_width) in [
                (-1_i128, false, false),
                (i128::from(width.bits() - 1), true, true),
                (i128::from(width.bits()), true, false),
            ] {
                let instruction = binary_instruction(
                    operator,
                    bv(width, lhs_signed),
                    int(1, width, lhs_signed),
                    int(count, width, true),
                    vec![
                        VirSafetyCheck::ShiftCountNonnegative {},
                        VirSafetyCheck::ShiftCountLessThanWidth {},
                    ],
                );
                let predicates = encode_instruction_safety(
                    &context(SemanticProfile::RustCheckedV0),
                    &instruction,
                )
                .unwrap();
                assert_eq!(eval_bool(&predicates[0].proposition), expected_nonnegative);
                assert_eq!(eval_bool(&predicates[1].proposition), expected_in_width);
            }
        }

        for signed in [false, true] {
            for (index_value, expected) in [(-1_i128, false), (1, true), (2, false)] {
                if !signed && index_value < 0 {
                    continue;
                }
                let go = context(SemanticProfile::GoFixedV0).with_variable(
                    "array",
                    VirType::Array {
                        length: ArrayLength::try_from(2).unwrap(),
                        element: Box::new(VirType::Bool {}),
                    },
                );
                let instruction = VirInstruction::Index {
                    id: "t0".to_owned(),
                    r#type: VirType::Bool {},
                    base: variable("array"),
                    index: int(index_value, width, signed),
                    safety_checks: vec![VirSafetyCheck::IndexInBounds {}],
                };
                let [predicate] = encode_instruction_safety(&go, &instruction)
                    .unwrap()
                    .try_into()
                    .expect("one index predicate");
                assert_eq!(eval_bool(&predicate.proposition), expected);
            }
        }
    }
}

#[test]
fn malformed_metadata_and_operand_shapes_fail_closed() {
    let expected = vec![
        VirSafetyCheck::DivisorNonzero {},
        VirSafetyCheck::SignedDivremRepresentable {
            operation: DivRemOperation::Div,
        },
    ];
    let cases = [
        (Vec::new(), "VIR_SAFETY_CHECK_MISSING"),
        (
            vec![
                VirSafetyCheck::DivisorNonzero {},
                VirSafetyCheck::SignedDivremRepresentable {
                    operation: DivRemOperation::Div,
                },
                VirSafetyCheck::ShiftCountLessThanWidth {},
            ],
            "VIR_SAFETY_CHECK_EXTRA",
        ),
        (
            vec![
                VirSafetyCheck::DivisorNonzero {},
                VirSafetyCheck::DivisorNonzero {},
            ],
            "VIR_SAFETY_CHECK_DUPLICATE",
        ),
        (
            vec![
                VirSafetyCheck::SignedDivremRepresentable {
                    operation: DivRemOperation::Div,
                },
                VirSafetyCheck::DivisorNonzero {},
            ],
            "VIR_SAFETY_CHECK_ORDER",
        ),
    ];
    for (actual, code) in cases {
        assert_eq!(
            validate_safety_check_sequence(&actual, &expected)
                .unwrap_err()
                .code(),
            code
        );
    }
    assert_eq!(
        validate_safety_check_sequence(
            &[overflow(OverflowOperation::Sub, true)],
            &[overflow(OverflowOperation::Add, true)],
        )
        .unwrap_err()
        .code(),
        "VIR_SAFETY_CHECK_OPERATION"
    );
    assert_eq!(
        validate_safety_check_sequence(
            &[overflow(OverflowOperation::Add, false)],
            &[overflow(OverflowOperation::Add, true)],
        )
        .unwrap_err()
        .code(),
        "VIR_SAFETY_CHECK_SIGNEDNESS"
    );

    assert_eq!(
        required_safety_checks(
            SemanticProfile::RustCheckedV0,
            VirSafetyOperation::Binary(VirBinaryOperator::BvAdd),
            &[
                bv(BitVectorWidth::Bits8, true),
                bv(BitVectorWidth::Bits16, true)
            ],
        )
        .unwrap_err()
        .code(),
        "VIR_INSTRUCTION_TYPE"
    );
    assert_eq!(
        required_safety_checks(
            SemanticProfile::RustCheckedV0,
            VirSafetyOperation::Binary(VirBinaryOperator::BvSdiv),
            &[
                bv(BitVectorWidth::Bits8, false),
                bv(BitVectorWidth::Bits8, false)
            ],
        )
        .unwrap_err()
        .code(),
        "VIR_INSTRUCTION_TYPE"
    );
    assert_eq!(
        required_safety_checks(
            SemanticProfile::RustCheckedV0,
            VirSafetyOperation::Unary(VirUnaryOperator::BvNeg),
            &[bv(BitVectorWidth::Bits8, false)],
        )
        .unwrap_err()
        .code(),
        "VIR_PROFILE_OPERATION"
    );
}

#[test]
fn overflow_predicates_match_exact_boundaries_at_every_width() {
    use BitVectorWidth::{Bits16, Bits32, Bits64, Bits8};
    use OverflowOperation::{Add, Mul, Neg, Sub};
    use VirBinaryOperator::{BvAdd, BvMul, BvSub};

    for (width, signed_minimum, signed_maximum, unsigned_maximum) in [
        (Bits8, -(1_i128 << 7), (1_i128 << 7) - 1, u8::MAX as u64),
        (Bits16, -(1_i128 << 15), (1_i128 << 15) - 1, u16::MAX as u64),
        (Bits32, -(1_i128 << 31), (1_i128 << 31) - 1, u32::MAX as u64),
        (Bits64, -(1_i128 << 63), (1_i128 << 63) - 1, u64::MAX),
    ] {
        let unsigned = bv(width, false);
        for (operator, operation, lhs, rhs, expected) in [
            (BvAdd, Add, unsigned_maximum - 1, 1, true),
            (BvAdd, Add, unsigned_maximum, 1, false),
            (BvSub, Sub, 1, 1, true),
            (BvSub, Sub, 0, 1, false),
            (BvMul, Mul, unsigned_maximum, 1, true),
            (BvMul, Mul, unsigned_maximum, 2, false),
        ] {
            let instruction = binary_instruction(
                operator,
                unsigned.clone(),
                int(lhs, width, false),
                int(rhs, width, false),
                vec![overflow(operation, false)],
            );
            let [predicate] =
                encode_instruction_safety(&context(SemanticProfile::RustCheckedV0), &instruction)
                    .unwrap()
                    .try_into()
                    .expect("one overflow predicate");
            assert_eq!(eval_bool(&predicate.proposition), expected);
            assert_common_metadata(&predicate);
        }

        let signed = bv(width, true);
        for (operator, operation, lhs, rhs, expected) in [
            (BvAdd, Add, signed_maximum, 0, true),
            (BvAdd, Add, signed_maximum, 1, false),
            (BvSub, Sub, signed_minimum, 0, true),
            (BvSub, Sub, signed_minimum, 1, false),
            (BvMul, Mul, 2, 3, true),
            (BvMul, Mul, signed_maximum, 2, false),
            (BvMul, Mul, -1, signed_minimum, false),
        ] {
            let instruction = binary_instruction(
                operator,
                signed.clone(),
                int(lhs, width, true),
                int(rhs, width, true),
                vec![overflow(operation, true)],
            );
            let [predicate] =
                encode_instruction_safety(&context(SemanticProfile::RustCheckedV0), &instruction)
                    .unwrap()
                    .try_into()
                    .expect("one overflow predicate");
            assert_eq!(eval_bool(&predicate.proposition), expected);
        }

        for (value, expected) in [(signed_minimum, false), (-1, true)] {
            let instruction = unary_instruction(
                signed.clone(),
                int(value, width, true),
                vec![overflow(Neg, true)],
            );
            let [predicate] =
                encode_instruction_safety(&context(SemanticProfile::RustCheckedV0), &instruction)
                    .unwrap()
                    .try_into()
                    .expect("one negation predicate");
            assert_eq!(eval_bool(&predicate.proposition), expected);
        }
    }
}

#[test]
fn division_shift_and_index_predicates_cover_total_corner_cases() {
    use BitVectorWidth::{Bits16, Bits8};

    let signed = bv(Bits8, true);
    let rust_div = binary_instruction(
        VirBinaryOperator::BvSdiv,
        signed.clone(),
        int(-128, Bits8, true),
        int(-1, Bits8, true),
        vec![
            VirSafetyCheck::DivisorNonzero {},
            VirSafetyCheck::SignedDivremRepresentable {
                operation: DivRemOperation::Div,
            },
        ],
    );
    let predicates =
        encode_instruction_safety(&context(SemanticProfile::RustCheckedV0), &rust_div).unwrap();
    assert_eq!(predicates.len(), 2);
    assert!(eval_bool(&predicates[0].proposition));
    assert!(!eval_bool(&predicates[1].proposition));
    assert_eq!(
        predicates[1].stable_id_component,
        SAFETY_OBLIGATION_KIND_COMPONENT
    );
    assert_eq!(
        predicates[1].evidence_route,
        SafetyEvidenceRoute::GroupedCertificate {
            foundation: SAFETY_GROUPED_CERTIFICATE_FOUNDATION
        }
    );

    let go_div_zero = binary_instruction(
        VirBinaryOperator::BvUdiv,
        bv(Bits8, false),
        int(7, Bits8, false),
        int(0, Bits8, false),
        vec![VirSafetyCheck::DivisorNonzero {}],
    );
    let [predicate] = encode_instruction_safety(&context(SemanticProfile::GoFixedV0), &go_div_zero)
        .unwrap()
        .try_into()
        .expect("one divisor predicate");
    assert!(!eval_bool(&predicate.proposition));
    assert_eq!(
        predicate.evidence_route,
        SafetyEvidenceRoute::ZeroAxiomGround
    );

    let rust_shift = binary_instruction(
        VirBinaryOperator::BvShl,
        bv(Bits8, false),
        int(1, Bits8, false),
        int(8, Bits16, true),
        vec![
            VirSafetyCheck::ShiftCountNonnegative {},
            VirSafetyCheck::ShiftCountLessThanWidth {},
        ],
    );
    let predicates =
        encode_instruction_safety(&context(SemanticProfile::RustCheckedV0), &rust_shift).unwrap();
    assert!(eval_bool(&predicates[0].proposition));
    assert!(!eval_bool(&predicates[1].proposition));
    assert!(contains_function(
        &predicates[1].proposition,
        "Std.BitVec.BV16.ult"
    ));

    let go_context = context(SemanticProfile::GoFixedV0)
        .with_variable(
            "array",
            VirType::Array {
                length: ArrayLength::try_from(256).unwrap(),
                element: Box::new(VirType::Bool {}),
            },
        )
        .with_variable("index", bv(Bits8, false));
    let index = VirInstruction::Index {
        id: "t0".to_owned(),
        r#type: VirType::Bool {},
        base: variable("array"),
        index: int(255, Bits8, false),
        safety_checks: vec![VirSafetyCheck::IndexInBounds {}],
    };
    let [predicate] = encode_instruction_safety(&go_context, &index)
        .unwrap()
        .try_into()
        .expect("one index predicate");
    assert!(eval_bool(&predicate.proposition));
    assert_eq!(
        predicate.proposition,
        MpkExprTerm::Constant {
            name: "Std.Bool.true".to_owned()
        }
    );

    let rust_context = ProgramExprContext::new(
        SemanticProfile::RustCheckedV0,
        rust_parameters(PointerWidth::Bits64),
        Vec::new(),
    )
    .unwrap()
    .with_variable(
        "array",
        VirType::Array {
            length: ArrayLength::try_from(2).unwrap(),
            element: Box::new(VirType::Bool {}),
        },
    )
    .with_variable("index", bv(BitVectorWidth::Bits32, false));
    let bad_index = VirInstruction::Index {
        id: "t0".to_owned(),
        r#type: VirType::Bool {},
        base: variable("array"),
        index: variable("index"),
        safety_checks: vec![VirSafetyCheck::IndexInBounds {}],
    };
    assert_eq!(
        encode_instruction_safety(&rust_context, &bad_index)
            .unwrap_err()
            .code(),
        "VIR_RUST_INDEX_TYPE"
    );
}

#[test]
fn symbolic_predicates_use_the_grouped_checked_certificate_path() {
    let ty = bv(BitVectorWidth::Bits8, false);
    let instruction = binary_instruction(
        VirBinaryOperator::BvAdd,
        ty.clone(),
        variable("lhs"),
        variable("rhs"),
        vec![overflow(OverflowOperation::Add, false)],
    );
    let rust = context(SemanticProfile::RustCheckedV0)
        .with_variable("lhs", ty.clone())
        .with_variable("rhs", ty);
    let [predicate] = encode_instruction_safety(&rust, &instruction)
        .unwrap()
        .try_into()
        .expect("one predicate");
    assert_eq!(
        predicate.evidence_route,
        SafetyEvidenceRoute::GroupedCertificate {
            foundation: SAFETY_GROUPED_CERTIFICATE_FOUNDATION
        }
    );
    assert_eq!(
        predicate.stable_id_component,
        SAFETY_OBLIGATION_KIND_COMPONENT
    );

    let ground = binary_instruction(
        VirBinaryOperator::BvAdd,
        bv(BitVectorWidth::Bits8, false),
        int(1, BitVectorWidth::Bits8, false),
        int(2, BitVectorWidth::Bits8, false),
        vec![overflow(OverflowOperation::Add, false)],
    );
    let [predicate] = encode_instruction_safety(&context(SemanticProfile::RustCheckedV0), &ground)
        .unwrap()
        .try_into()
        .expect("one predicate");
    assert_eq!(
        predicate.evidence_route,
        SafetyEvidenceRoute::MvpTheoryGround {
            format: SAFETY_BITVEC_THEORY_FORMAT
        }
    );

    let unsupported = binary_instruction(
        VirBinaryOperator::BvMul,
        bv(BitVectorWidth::Bits8, false),
        int(128, BitVectorWidth::Bits8, false),
        int(2, BitVectorWidth::Bits8, false),
        vec![overflow(OverflowOperation::Mul, false)],
    );
    let [predicate] =
        encode_instruction_safety(&context(SemanticProfile::RustCheckedV0), &unsupported)
            .unwrap()
            .try_into()
            .expect("one predicate");
    assert_eq!(
        predicate.evidence_route,
        SafetyEvidenceRoute::GroupedCertificate {
            foundation: SAFETY_GROUPED_CERTIFICATE_FOUNDATION
        }
    );
}

#[test]
fn safety_path_ledger_covers_every_check_without_unchecked_axioms() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let ledger: Value = serde_json::from_slice(
        &fs::read(root.join("fixtures/program-safety/expected.json")).expect("safety ledger"),
    )
    .expect("valid safety ledger");
    assert_eq!(ledger["schema"], "mpk.program_safety_paths.v0");
    assert_eq!(ledger["unchecked_axioms"], 0);
    assert!(ledger["proof_pending_owner"].is_null());
    assert_eq!(
        ledger["rust_checked_theory_format"],
        SAFETY_BITVEC_THEORY_FORMAT
    );

    let checks = ledger["check_kinds"]
        .as_array()
        .expect("check kind array")
        .iter()
        .map(|value| value.as_str().expect("check kind"))
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        checks,
        [
            "integer_no_overflow",
            "divisor_nonzero",
            "signed_divrem_representable",
            "shift_count_nonnegative",
            "shift_count_less_than_width",
            "index_in_bounds",
        ]
        .into_iter()
        .collect()
    );
    for vector in ledger["vectors"].as_array().expect("vector array") {
        let route = vector["route"].as_str().expect("route");
        assert!(matches!(
            route,
            "zero_axiom" | "mvp_theory" | "grouped_certificate"
        ));
        if route == "grouped_certificate" {
            assert_eq!(vector["foundation"], SAFETY_GROUPED_CERTIFICATE_FOUNDATION);
            assert_eq!(vector["checked_path"], "mpk.vc.cert_skeleton.v1");
        } else if route == "mvp_theory" {
            assert_eq!(vector["format"], SAFETY_BITVEC_THEORY_FORMAT);
        } else {
            let fixture = vector["fixture"].as_str().expect("zero-axiom fixture");
            let certificate = decode_canonical_certificate(&decode_hex(&root.join(fixture)))
                .expect("canonical zero-axiom fixture");
            assert_eq!(certificate.axiom_report.summary.total_axiom_count, 0);
        }
    }
}

fn decode_hex(path: &Path) -> Vec<u8> {
    let input = fs::read_to_string(path).expect("hex fixture");
    let input = input.trim();
    assert_eq!(input.len() % 2, 0, "hex fixture uses complete bytes");
    input
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            u8::from_str_radix(std::str::from_utf8(pair).expect("ASCII hex"), 16)
                .expect("valid hex")
        })
        .collect()
}

fn assert_common_metadata(predicate: &EncodedSafetyPredicate) {
    assert_eq!(
        predicate.obligation_kind,
        SafetyObligationKind::OperationSafety
    );
    assert_eq!(
        predicate.obligation_kind.as_str(),
        SAFETY_OBLIGATION_KIND_COMPONENT
    );
    assert_eq!(
        predicate.stable_id_component,
        SAFETY_OBLIGATION_KIND_COMPONENT
    );
}

fn contains_function(term: &MpkExprTerm, expected: &str) -> bool {
    match term {
        MpkExprTerm::Apply { function, args } => {
            function == expected || args.iter().any(|arg| contains_function(arg, expected))
        }
        MpkExprTerm::Convert { value, .. } => contains_function(value, expected),
        _ => false,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Evaluated {
    BitVector { bits: u64, width: BitVectorWidth },
    Boolean(bool),
}

fn eval_bool(term: &MpkExprTerm) -> bool {
    match eval(term) {
        Evaluated::Boolean(value) => value,
        Evaluated::BitVector { .. } => panic!("predicate evaluated to a bitvector"),
    }
}

fn eval(term: &MpkExprTerm) -> Evaluated {
    match term {
        MpkExprTerm::Constant { name } if name == "Std.Bool.true" => Evaluated::Boolean(true),
        MpkExprTerm::Constant { name } if name == "Std.Bool.false" => Evaluated::Boolean(false),
        MpkExprTerm::BitVecLiteral {
            value,
            width,
            signed: _,
        } => {
            let width = BitVectorWidth::try_from(*width).expect("supported width");
            let value = value.parse::<i128>().expect("integer literal");
            let modulus = 1_i128 << width.bits();
            Evaluated::BitVector {
                bits: value.rem_euclid(modulus) as u64,
                width,
            }
        }
        MpkExprTerm::Apply { function, args } if function == "Std.Bool.not" => {
            let [value] = args.as_slice() else {
                panic!("not arity")
            };
            Evaluated::Boolean(!eval_bool(value))
        }
        MpkExprTerm::Apply { function, args }
            if function == "Std.Bool.and" || function == "Std.Bool.or" =>
        {
            let [lhs, rhs] = args.as_slice() else {
                panic!("boolean arity")
            };
            Evaluated::Boolean(if function == "Std.Bool.and" {
                eval_bool(lhs) && eval_bool(rhs)
            } else {
                eval_bool(lhs) || eval_bool(rhs)
            })
        }
        MpkExprTerm::Apply { function, args } if function == "Std.Eq" => {
            let [lhs, rhs] = args.as_slice() else {
                panic!("equality arity")
            };
            Evaluated::Boolean(eval(lhs) == eval(rhs))
        }
        MpkExprTerm::Apply { function, args } if function.starts_with("Std.BitVec.BV") => {
            let suffix = function
                .strip_prefix("Std.BitVec.BV")
                .expect("bitvector prefix");
            let (width, operation) = suffix.split_once('.').expect("bitvector operation");
            let width = BitVectorWidth::try_from(width.parse::<u32>().expect("numeric width"))
                .expect("supported width");
            let [lhs, rhs] = args.as_slice() else {
                panic!("binary bitvector arity")
            };
            let lhs = eval_bv(lhs, width);
            let rhs = eval_bv(rhs, width);
            let operator = match operation {
                "add" => VirBinaryOperator::BvAdd,
                "sub" => VirBinaryOperator::BvSub,
                "mul" => VirBinaryOperator::BvMul,
                "sdiv" => VirBinaryOperator::BvSdiv,
                "udiv" => VirBinaryOperator::BvUdiv,
                "slt" => VirBinaryOperator::SignedLt,
                "sgt" => VirBinaryOperator::SignedGt,
                "sge" => VirBinaryOperator::SignedGe,
                "ult" => VirBinaryOperator::UnsignedLt,
                "ugt" => VirBinaryOperator::UnsignedGt,
                "uge" => VirBinaryOperator::UnsignedGe,
                other => panic!("unsupported evaluator operation {other}"),
            };
            match evaluate_total_bitvector_operation(operator, width, lhs, width, rhs)
                .expect("total operation")
            {
                TotalBitVectorResult::BitVector(bits) => Evaluated::BitVector { bits, width },
                TotalBitVectorResult::Boolean(value) => Evaluated::Boolean(value),
            }
        }
        other => panic!("unsupported test term {other:?}"),
    }
}

fn eval_bv(term: &MpkExprTerm, expected_width: BitVectorWidth) -> u64 {
    match eval(term) {
        Evaluated::BitVector { bits, width } if width == expected_width => bits,
        other => panic!(
            "expected BV{} value, found {other:?}",
            expected_width.bits()
        ),
    }
}
