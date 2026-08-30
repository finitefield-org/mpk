#[path = "support/successor_projection.rs"]
mod successor_projection;

use std::fs;
use std::path::{Path, PathBuf};

use mpk_cert::{
    axiom_report_hash_for_report, build_axiom_report, build_export_block, certificate_hash,
    decode_canonical_certificate, export_block_hash, hash_hex,
};
use mpk_vc::program_encode::{
    encode_vir_contract_expr, encode_vir_instruction_expr, encode_vir_value,
    evaluate_total_bitvector_operation, ProgramExprContext, ProgramExprEncodeError,
    TotalBitVectorResult,
};
use mpk_vc::semantic_profile::{
    GoFixedParameters, OverflowMode, PanicMode, PointerWidth, RustCheckedParameters,
    SemanticParameters, SemanticProfile,
};
use mpk_vc::type_encode::{
    encode_vir_type, MpkTypeTerm, ProgramTypeEncoder, TypeEncodeError, STD_PROGRAM_BASE_ARRAY,
    STD_PROGRAM_BASE_ARRAY_LENGTH, STD_PROGRAM_BASE_BOOL, STD_PROGRAM_BASE_STRUCT_FIELD,
    STD_PROGRAM_BASE_STRUCT_FIELD_TYPE, STD_PROGRAM_BASE_STRUCT_SHAPE,
    STD_PROGRAM_BASE_STRUCT_VALUE,
};
use mpk_vc::vir::{
    ArrayLength, BitVectorWidth, DecimalInteger, OverflowOperation, VirBinaryOperator,
    VirConstDecl, VirConstantRef, VirContractBinaryExpr, VirContractExpr, VirContractNaryExpr,
    VirContractNaryOperator, VirContractUnaryExpr, VirContractUnaryOperator, VirInstruction,
    VirIntLiteral, VirIntegerLiteral, VirLiteral, VirSafetyCheck, VirStructDecl, VirStructField,
    VirType, VirValue, VirVariableRef,
};
use mpk_vc::{
    generate_program_vcs, validate_vir_safety_fragment, MpkExprTerm, VirSafetyOperation,
    STD_BITVEC_MODULE, STD_BOOL_AND, STD_BOOL_IF, STD_BOOL_NOT, STD_EQ,
};
use serde_json::Value;
use successor_projection::import_successor_rust_vir_projection;

const RUST_ARRAYS_VIR: &[u8] =
    include_bytes!("../../../rust-tools/rust2vir/testdata/arrays/expected-vir.json");
const RUST_STRUCTS_VIR: &[u8] =
    include_bytes!("../../../rust-tools/rust2vir/testdata/structs/expected-vir.json");

fn go_parameters(pointer_width: PointerWidth) -> SemanticParameters {
    SemanticParameters::GoFixed(GoFixedParameters {
        target_id: match pointer_width {
            PointerWidth::Bits32 => "linux/386",
            PointerWidth::Bits64 => "linux/amd64",
        }
        .to_owned(),
        pointer_width,
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

fn variable(id: &str) -> VirValue {
    VirValue::Variable(VirVariableRef { var: id.to_owned() })
}

fn contract_variable(id: &str) -> VirContractExpr {
    VirContractExpr::Variable(VirVariableRef { var: id.to_owned() })
}

fn binop(
    op: VirBinaryOperator,
    result_type: VirType,
    checks: Vec<VirSafetyCheck>,
) -> VirInstruction {
    VirInstruction::BinOp {
        id: "t0".to_owned(),
        op,
        r#type: result_type,
        lhs: variable("arg0"),
        rhs: variable("arg1"),
        safety_checks: checks,
    }
}

fn pair_declaration() -> VirStructDecl {
    VirStructDecl {
        id: "example::Pair".to_owned(),
        name: "Pair".to_owned(),
        fields: vec![
            VirStructField {
                name: "left".to_owned(),
                r#type: bv(BitVectorWidth::Bits64, true),
            },
            VirStructField {
                name: "flags".to_owned(),
                r#type: VirType::Array {
                    length: ArrayLength::try_from(2).expect("valid array length"),
                    element: Box::new(VirType::Bool {}),
                },
            },
        ],
    }
}

#[test]
fn foundation_fixture_manifest_matches_canonical_certificates() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let manifest: Value = serde_json::from_slice(
        &fs::read(root.join("fixtures/program-base/expected.json")).expect("fixture manifest"),
    )
    .expect("valid fixture manifest");

    for fixture in ["foundation", "type_map"] {
        let record = &manifest[fixture];
        assert_eq!(record["axiom_count"], 0);
        let relative = record["path"].as_str().expect("fixture path");
        let certificate_bytes = decode_hex(&root.join(relative));
        let certificate = decode_canonical_certificate(&certificate_bytes)
            .expect("program-base certificate is canonical");
        assert_eq!(certificate.module, record["module"]);
        assert_eq!(certificate.axiom_report.summary.total_axiom_count, 0);
        assert_eq!(
            build_export_block(&certificate).expect("rebuild export block"),
            certificate.export_block
        );
        assert_eq!(
            build_axiom_report(&certificate).expect("rebuild axiom report"),
            certificate.axiom_report
        );
        assert_eq!(
            export_block_hash(&certificate.export_block),
            certificate.hashes.export_hash
        );
        assert_eq!(
            axiom_report_hash_for_report(&certificate.axiom_report),
            certificate.hashes.axiom_report_hash
        );
        assert_eq!(
            hash_hex(&certificate_hash(&certificate_bytes)),
            record["certificate_hash"]
                .as_str()
                .expect("certificate hash")
        );
    }

    let foundation = decode_canonical_certificate(&decode_hex(
        &root.join(
            manifest["foundation"]["path"]
                .as_str()
                .expect("foundation path"),
        ),
    ))
    .expect("foundation is canonical");
    let exports = foundation
        .export_block
        .iter()
        .map(|entry| {
            foundation.name_table[usize::try_from(entry.name).expect("name ID fits usize")].as_str()
        })
        .collect::<std::collections::BTreeSet<_>>();
    let expected_exports = manifest["aliases"]
        .as_array()
        .expect("alias array")
        .iter()
        .chain(
            manifest["self_contained_exports"]
                .as_array()
                .expect("self-contained export array"),
        )
        .map(|alias| alias.as_str().expect("alias string"))
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(exports, expected_exports);
}

#[test]
fn rust_constant_array_golden_imports_and_generates_stable_vcs() {
    let fixtures: Value = serde_json::from_slice(RUST_ARRAYS_VIR).expect("array VIR JSON");
    let fixtures = fixtures.as_array().expect("dual-target array VIR fixtures");
    assert_eq!(fixtures.len(), 2);
    for fixture in fixtures {
        let bytes = serde_json::to_vec(fixture).expect("successor array VIR serializes");
        let module = import_successor_rust_vir_projection(&bytes);
        let first = generate_program_vcs(&module).expect("array construction generates VCs");
        let second = generate_program_vcs(&module).expect("repeat array construction VCs");
        assert_eq!(first, second);
        assert_eq!(first.functions.len(), 1);
        assert_eq!(first.functions[0].members.len(), 1);
    }
}

#[test]
fn rust_nominal_struct_golden_imports_and_generates_stable_contract_groups() {
    let fixtures: Value = serde_json::from_slice(RUST_STRUCTS_VIR).expect("struct VIR JSON");
    let fixtures = fixtures
        .as_array()
        .expect("dual-target struct VIR fixtures");
    assert_eq!(fixtures.len(), 2);
    for fixture in fixtures {
        let bytes = serde_json::to_vec(fixture).expect("successor struct VIR serializes");
        let module = import_successor_rust_vir_projection(&bytes);
        assert_eq!(
            module.units[0].type_decls,
            vec![pair_like_point_declaration()]
        );
        let first = generate_program_vcs(&module).expect("struct construction generates VCs");
        let second = generate_program_vcs(&module).expect("repeat struct construction VCs");
        assert_eq!(first, second);
        assert_eq!(first.functions.len(), 1);
        assert_eq!(first.functions[0].members.len(), 1);
        assert_eq!(
            first.functions[0].members[0].kind,
            mpk_vc::ProgramVcMemberKind::Postcondition
        );
        assert!(first.functions[0].members[0]
            .group_id
            .ends_with(".contract"));
    }
}

fn pair_like_point_declaration() -> VirStructDecl {
    VirStructDecl {
        id: "vector::Point".to_owned(),
        name: "Point".to_owned(),
        fields: ["x", "y"]
            .into_iter()
            .map(|name| VirStructField {
                name: name.to_owned(),
                r#type: bv(BitVectorWidth::Bits8, false),
            })
            .collect(),
    }
}

#[test]
fn vir_types_and_target_sized_integers_use_program_base() {
    let declarations = vec![pair_declaration()];
    let go_parameters = go_parameters(PointerWidth::Bits32);
    let go = ProgramTypeEncoder::new(SemanticProfile::GoFixedV0, &go_parameters, &declarations)
        .expect("Go type encoder");
    let rust_parameters = rust_parameters(PointerWidth::Bits64);
    let rust = ProgramTypeEncoder::new(
        SemanticProfile::RustCheckedV0,
        &rust_parameters,
        &declarations,
    )
    .expect("Rust type encoder");

    assert_eq!(
        go.encode(&VirType::Bool {}).unwrap(),
        MpkTypeTerm::constant(STD_PROGRAM_BASE_BOOL)
    );
    assert_eq!(
        go.encode_target_sized_integer(true).unwrap(),
        MpkTypeTerm::constant("Std.Program.Base.Int32")
    );
    assert_eq!(
        rust.encode_target_sized_integer(false).unwrap(),
        MpkTypeTerm::constant("Std.Program.Base.Uint64")
    );

    for (width, unsigned_name, signed_name) in [
        (
            BitVectorWidth::Bits8,
            "Std.Program.Base.Uint8",
            "Std.Program.Base.Int8",
        ),
        (
            BitVectorWidth::Bits16,
            "Std.Program.Base.Uint16",
            "Std.Program.Base.Int16",
        ),
        (
            BitVectorWidth::Bits32,
            "Std.Program.Base.Uint32",
            "Std.Program.Base.Int32",
        ),
        (
            BitVectorWidth::Bits64,
            "Std.Program.Base.Uint64",
            "Std.Program.Base.Int64",
        ),
    ] {
        assert_eq!(
            go.encode(&bv(width, false)).expect("BV type encodes"),
            MpkTypeTerm::constant(unsigned_name)
        );
        assert_eq!(
            go.encode(&bv(width, true)).expect("BV type encodes"),
            MpkTypeTerm::constant(signed_name)
        );
    }

    let array = VirType::Array {
        length: ArrayLength::try_from(3).unwrap(),
        element: Box::new(bv(BitVectorWidth::Bits16, false)),
    };
    assert_eq!(
        go.encode(&array).unwrap(),
        MpkTypeTerm::apply(
            STD_PROGRAM_BASE_ARRAY,
            [
                MpkTypeTerm::constant("Std.Program.Base.Uint16"),
                MpkTypeTerm::apply(STD_PROGRAM_BASE_ARRAY_LENGTH, [MpkTypeTerm::nat_literal(3)])
            ]
        )
    );

    let struct_term = go
        .encode(&VirType::Struct {
            id: "example::Pair".to_owned(),
        })
        .unwrap();
    let expected_shape = MpkTypeTerm::apply(
        STD_PROGRAM_BASE_STRUCT_SHAPE,
        [
            MpkTypeTerm::string_literal("example::Pair"),
            MpkTypeTerm::apply(
                STD_PROGRAM_BASE_STRUCT_FIELD,
                [
                    MpkTypeTerm::string_literal("left"),
                    MpkTypeTerm::apply(
                        STD_PROGRAM_BASE_STRUCT_FIELD_TYPE,
                        [MpkTypeTerm::constant("Std.Program.Base.Int64")],
                    ),
                ],
            ),
            MpkTypeTerm::apply(
                STD_PROGRAM_BASE_STRUCT_FIELD,
                [
                    MpkTypeTerm::string_literal("flags"),
                    MpkTypeTerm::apply(
                        STD_PROGRAM_BASE_STRUCT_FIELD_TYPE,
                        [MpkTypeTerm::apply(
                            STD_PROGRAM_BASE_ARRAY,
                            [
                                MpkTypeTerm::constant(STD_PROGRAM_BASE_BOOL),
                                MpkTypeTerm::apply(
                                    STD_PROGRAM_BASE_ARRAY_LENGTH,
                                    [MpkTypeTerm::nat_literal(2)],
                                ),
                            ],
                        )],
                    ),
                ],
            ),
        ],
    );
    assert_eq!(
        struct_term,
        MpkTypeTerm::apply(STD_PROGRAM_BASE_STRUCT_VALUE, [expected_shape])
    );

    assert_eq!(
        encode_vir_type(
            SemanticProfile::RustCheckedV0,
            &rust_parameters,
            &declarations,
            &VirType::Bool {}
        )
        .unwrap(),
        MpkTypeTerm::constant(STD_PROGRAM_BASE_BOOL)
    );
}

#[test]
fn go_and_rust_share_value_encoding_after_check_metadata_validation() {
    let ty = bv(BitVectorWidth::Bits8, true);
    let go_checks = Vec::new();
    let rust_checks = vec![VirSafetyCheck::IntegerNoOverflow {
        operation: OverflowOperation::Add,
        signed: true,
    }];
    validate_vir_safety_fragment(
        SemanticProfile::GoFixedV0,
        VirSafetyOperation::Binary(VirBinaryOperator::BvAdd),
        &[ty.clone(), ty.clone()],
        &go_checks,
    )
    .expect("Go check metadata");
    validate_vir_safety_fragment(
        SemanticProfile::RustCheckedV0,
        VirSafetyOperation::Binary(VirBinaryOperator::BvAdd),
        &[ty.clone(), ty.clone()],
        &rust_checks,
    )
    .expect("Rust check metadata");

    let go_context = ProgramExprContext::new(
        SemanticProfile::GoFixedV0,
        go_parameters(PointerWidth::Bits64),
        Vec::new(),
    )
    .unwrap()
    .with_variable("arg0", ty.clone())
    .with_variable("arg1", ty.clone());
    let rust_context = ProgramExprContext::new(
        SemanticProfile::RustCheckedV0,
        rust_parameters(PointerWidth::Bits64),
        Vec::new(),
    )
    .unwrap()
    .with_variable("arg0", ty.clone())
    .with_variable("arg1", ty.clone());

    let go_term = encode_vir_instruction_expr(
        &go_context,
        &binop(VirBinaryOperator::BvAdd, ty.clone(), go_checks),
    )
    .unwrap();
    let rust_term = encode_vir_instruction_expr(
        &rust_context,
        &binop(VirBinaryOperator::BvAdd, ty, rust_checks),
    )
    .unwrap();
    assert_eq!(go_term, rust_term);
    assert_eq!(
        go_term,
        MpkExprTerm::apply(
            format!("{STD_BITVEC_MODULE}.BV8.add"),
            [
                MpkExprTerm::Var {
                    name: "arg0".to_owned()
                },
                MpkExprTerm::Var {
                    name: "arg1".to_owned()
                }
            ]
        )
    );
}

#[test]
fn fixed_array_index_value_waits_for_the_checked_array_read_foundation() {
    let array_type = VirType::Array {
        length: ArrayLength::try_from(4).unwrap(),
        element: Box::new(bv(BitVectorWidth::Bits8, false)),
    };
    let context = ProgramExprContext::new(
        SemanticProfile::RustCheckedV0,
        rust_parameters(PointerWidth::Bits64),
        Vec::new(),
    )
    .unwrap()
    .with_variable("arg0", array_type)
    .with_variable("arg1", bv(BitVectorWidth::Bits64, false));
    let error = encode_vir_instruction_expr(
        &context,
        &VirInstruction::Index {
            id: "t0".to_owned(),
            r#type: bv(BitVectorWidth::Bits8, false),
            base: variable("arg0"),
            index: variable("arg1"),
            safety_checks: vec![VirSafetyCheck::IndexInBounds {}],
        },
    )
    .expect_err("array-read values are not encoded before foundation integration");

    assert!(matches!(
        error,
        ProgramExprEncodeError::UnsupportedInstruction {
            kind: mpk_vc::VirInstructionKind::Index,
        }
    ));
}

#[test]
fn cross_width_shifts_guard_before_count_conversion() {
    let lhs_type = bv(BitVectorWidth::Bits8, false);
    let rhs_type = bv(BitVectorWidth::Bits16, false);
    let context = ProgramExprContext::new(
        SemanticProfile::GoFixedV0,
        go_parameters(PointerWidth::Bits64),
        Vec::new(),
    )
    .unwrap()
    .with_variable("arg0", lhs_type.clone())
    .with_variable("arg1", rhs_type);
    let term = encode_vir_instruction_expr(
        &context,
        &binop(VirBinaryOperator::BvShl, lhs_type, Vec::new()),
    )
    .unwrap();

    let MpkExprTerm::Apply { function, args } = term else {
        panic!("cross-width shift must be guarded")
    };
    assert_eq!(function, STD_BOOL_IF);
    assert_eq!(args.len(), 3);
    assert_eq!(
        args[0],
        MpkExprTerm::apply(
            format!("{STD_BITVEC_MODULE}.BV16.ult"),
            [
                MpkExprTerm::Var {
                    name: "arg1".to_owned()
                },
                MpkExprTerm::BitVecLiteral {
                    value: "8".to_owned(),
                    width: 16,
                    signed: false
                }
            ]
        )
    );
    assert!(matches!(args[1], MpkExprTerm::Apply { .. }));
    assert_eq!(
        args[2],
        MpkExprTerm::BitVecLiteral {
            value: "0".to_owned(),
            width: 8,
            signed: false
        }
    );
}

#[test]
fn total_bitvector_equations_cover_shift_division_and_comparison_corners() {
    use BitVectorWidth::{Bits16, Bits32, Bits64, Bits8};
    use TotalBitVectorResult::{BitVector as Bv, Boolean};
    use VirBinaryOperator::*;

    assert_eq!(
        evaluate_total_bitvector_operation(BvShl, Bits8, 1, Bits16, 9).unwrap(),
        Bv(0)
    );
    assert_eq!(
        evaluate_total_bitvector_operation(BvAshr, Bits8, 254, Bits16, 8).unwrap(),
        Bv(255)
    );
    assert_eq!(
        evaluate_total_bitvector_operation(BvLshr, Bits8, 254, Bits16, 8).unwrap(),
        Bv(0)
    );
    assert_eq!(
        evaluate_total_bitvector_operation(BvSdiv, Bits8, 249, Bits8, 0).unwrap(),
        Bv(1)
    );
    assert_eq!(
        evaluate_total_bitvector_operation(BvSdiv, Bits8, 7, Bits8, 0).unwrap(),
        Bv(255)
    );
    assert_eq!(
        evaluate_total_bitvector_operation(BvSrem, Bits8, 249, Bits8, 0).unwrap(),
        Bv(249)
    );
    assert_eq!(
        evaluate_total_bitvector_operation(BvUdiv, Bits8, 7, Bits8, 0).unwrap(),
        Bv(255)
    );
    assert_eq!(
        evaluate_total_bitvector_operation(BvUrem, Bits8, 7, Bits8, 0).unwrap(),
        Bv(7)
    );
    assert_eq!(
        evaluate_total_bitvector_operation(BvSdiv, Bits8, 128, Bits8, 255).unwrap(),
        Bv(128)
    );
    assert_eq!(
        evaluate_total_bitvector_operation(BvSrem, Bits8, 128, Bits8, 255).unwrap(),
        Bv(0)
    );
    assert_eq!(
        evaluate_total_bitvector_operation(SignedLt, Bits8, 255, Bits8, 0).unwrap(),
        Boolean(true)
    );
    assert_eq!(
        evaluate_total_bitvector_operation(UnsignedLt, Bits8, 255, Bits8, 0).unwrap(),
        Boolean(false)
    );

    for (width, sign_bit, all_ones) in [
        (Bits8, 1_u64 << 7, u8::MAX.into()),
        (Bits16, 1_u64 << 15, u16::MAX.into()),
        (Bits32, 1_u64 << 31, u32::MAX.into()),
        (Bits64, 1_u64 << 63, u64::MAX),
    ] {
        assert_eq!(
            evaluate_total_bitvector_operation(BvSdiv, width, sign_bit, width, all_ones).unwrap(),
            Bv(sign_bit)
        );
        assert_eq!(
            evaluate_total_bitvector_operation(BvSrem, width, sign_bit, width, all_ones).unwrap(),
            Bv(0)
        );
        assert_eq!(
            evaluate_total_bitvector_operation(BvUdiv, width, 1, width, 0).unwrap(),
            Bv(all_ones)
        );
        assert_eq!(
            evaluate_total_bitvector_operation(BvUrem, width, all_ones, width, 0).unwrap(),
            Bv(all_ones)
        );
    }
}

#[test]
fn contract_boolean_tree_rejects_oversized_input_before_visiting_arguments() {
    let context = ProgramExprContext::new(
        SemanticProfile::GoFixedV0,
        go_parameters(PointerWidth::Bits64),
        Vec::new(),
    )
    .unwrap();
    let expression = VirContractExpr::Nary(VirContractNaryExpr {
        op: VirContractNaryOperator::Or,
        args: (0..65)
            .map(|index| contract_variable(&format!("unknown{index}")))
            .collect(),
    });

    assert_eq!(
        encode_vir_contract_expr(&context, &expression),
        Err(ProgramExprEncodeError::ContractArity {
            operation: "or",
            actual: 65,
        })
    );
}

#[test]
fn program_type_encoder_rejects_forward_nominal_references() {
    let declarations = vec![
        VirStructDecl {
            id: "example::Outer".to_owned(),
            name: "Outer".to_owned(),
            fields: vec![VirStructField {
                name: "inner".to_owned(),
                r#type: VirType::Struct {
                    id: "example::Inner".to_owned(),
                },
            }],
        },
        VirStructDecl {
            id: "example::Inner".to_owned(),
            name: "Inner".to_owned(),
            fields: Vec::new(),
        },
    ];
    let parameters = go_parameters(PointerWidth::Bits64);

    assert_eq!(
        ProgramTypeEncoder::new(SemanticProfile::GoFixedV0, &parameters, &declarations)
            .unwrap_err(),
        TypeEncodeError::StructDeclarationOrder {
            declaration_id: "example::Outer".to_owned(),
            referenced_id: "example::Inner".to_owned(),
        }
    );
}

#[test]
fn constants_aggregate_equality_and_boolean_tree_are_canonical() {
    let array_type = VirType::Array {
        length: ArrayLength::try_from(2).unwrap(),
        element: Box::new(VirType::Bool {}),
    };
    let struct_type = VirType::Struct {
        id: "example::Pair".to_owned(),
    };
    let constant = VirConstDecl {
        id: "example::LIMIT".to_owned(),
        name: "LIMIT".to_owned(),
        r#type: bv(BitVectorWidth::Bits8, false),
        value: VirLiteral::Integer(VirIntegerLiteral {
            int: VirIntLiteral {
                value: DecimalInteger::new("7".to_owned()).unwrap(),
                width: BitVectorWidth::Bits8,
                signed: false,
            },
        }),
    };
    let context = ProgramExprContext::new(
        SemanticProfile::RustCheckedV0,
        rust_parameters(PointerWidth::Bits64),
        vec![pair_declaration()],
    )
    .unwrap()
    .with_variable("a", VirType::Bool {})
    .with_variable("b", VirType::Bool {})
    .with_variable("c", VirType::Bool {})
    .with_variable("array0", array_type.clone())
    .with_variable("array1", array_type)
    .with_variable("pair0", struct_type.clone())
    .with_variable("pair1", struct_type)
    .with_constant(constant);

    assert_eq!(
        encode_vir_value(
            &context,
            &VirValue::Constant(VirConstantRef {
                constant: "example::LIMIT".to_owned()
            })
        )
        .unwrap(),
        MpkExprTerm::BitVecLiteral {
            value: "7".to_owned(),
            width: 8,
            signed: false
        }
    );

    let boolean_tree = VirContractExpr::Nary(VirContractNaryExpr {
        op: VirContractNaryOperator::And,
        args: vec![
            contract_variable("a"),
            VirContractExpr::Unary(VirContractUnaryExpr {
                op: VirContractUnaryOperator::Not,
                value: Box::new(contract_variable("b")),
            }),
            contract_variable("c"),
        ],
    });
    let encoded = encode_vir_contract_expr(&context, &boolean_tree).unwrap();
    assert_eq!(
        encoded,
        MpkExprTerm::apply(
            STD_BOOL_AND,
            [
                MpkExprTerm::apply(
                    STD_BOOL_AND,
                    [
                        MpkExprTerm::Var {
                            name: "a".to_owned()
                        },
                        MpkExprTerm::apply(
                            STD_BOOL_NOT,
                            [MpkExprTerm::Var {
                                name: "b".to_owned()
                            }]
                        )
                    ]
                ),
                MpkExprTerm::Var {
                    name: "c".to_owned()
                }
            ]
        )
    );

    let array_equality = VirContractExpr::Binary(VirContractBinaryExpr {
        op: VirBinaryOperator::Eq,
        lhs: Box::new(contract_variable("array0")),
        rhs: Box::new(contract_variable("array1")),
    });
    assert_eq!(
        encode_vir_contract_expr(&context, &array_equality).unwrap(),
        MpkExprTerm::apply(
            STD_EQ,
            [
                MpkExprTerm::Var {
                    name: "array0".to_owned()
                },
                MpkExprTerm::Var {
                    name: "array1".to_owned()
                }
            ]
        )
    );

    let struct_inequality = VirContractExpr::Binary(VirContractBinaryExpr {
        op: VirBinaryOperator::NotEq,
        lhs: Box::new(contract_variable("pair0")),
        rhs: Box::new(contract_variable("pair1")),
    });
    assert_eq!(
        encode_vir_contract_expr(&context, &struct_inequality).unwrap(),
        MpkExprTerm::apply(
            STD_BOOL_NOT,
            [MpkExprTerm::apply(
                STD_EQ,
                [
                    MpkExprTerm::Var {
                        name: "pair0".to_owned()
                    },
                    MpkExprTerm::Var {
                        name: "pair1".to_owned()
                    }
                ]
            )]
        )
    );
}

#[test]
fn go_contract_division_uses_total_hook_and_rust_contract_rejects_it() {
    let ty = bv(BitVectorWidth::Bits8, true);
    let expression = VirContractExpr::Binary(VirContractBinaryExpr {
        op: VirBinaryOperator::BvSdiv,
        lhs: Box::new(contract_variable("arg0")),
        rhs: Box::new(contract_variable("arg1")),
    });
    let go = ProgramExprContext::new(
        SemanticProfile::GoFixedV0,
        go_parameters(PointerWidth::Bits64),
        Vec::new(),
    )
    .unwrap()
    .with_variable("arg0", ty.clone())
    .with_variable("arg1", ty.clone());
    let rust = ProgramExprContext::new(
        SemanticProfile::RustCheckedV0,
        rust_parameters(PointerWidth::Bits64),
        Vec::new(),
    )
    .unwrap()
    .with_variable("arg0", ty.clone())
    .with_variable("arg1", ty);

    assert_eq!(
        encode_vir_contract_expr(&go, &expression).unwrap(),
        MpkExprTerm::apply(
            format!("{STD_BITVEC_MODULE}.BV8.sdiv"),
            [
                MpkExprTerm::Var {
                    name: "arg0".to_owned()
                },
                MpkExprTerm::Var {
                    name: "arg1".to_owned()
                }
            ]
        )
    );
    assert_eq!(
        encode_vir_contract_expr(&rust, &expression),
        Err(ProgramExprEncodeError::ProfileOperation {
            profile: SemanticProfile::RustCheckedV0,
            operation: "bv_sdiv"
        })
    );
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
