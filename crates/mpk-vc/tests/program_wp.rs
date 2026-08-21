use mpk_vc::vir::{
    VirBooleanLiteral, VirContractBinaryExpr, VirIntegerLiteral, VirPanicPolicy, VirResultRef,
    VirTermination, VirVariableRef,
};
use mpk_vc::{
    contract_hash, generate_program_vcs, validate_vir, vir_hash, BitVectorWidth, DecimalInteger,
    GoFixedParameters, LowercaseSha256, MpkExprTerm, OverflowMode, PanicMode, PointerWidth,
    ProgramVcMemberKind, RustCheckedParameters, SafetyEvidenceRoute, SemanticParameters,
    SemanticProfile, SourceLanguage, VirBinaryOperator, VirBinding, VirBlock, VirContract,
    VirContractExpr, VirFeature, VirFunction, VirInstruction, VirIntLiteral, VirModule,
    VirSafetyCheck, VirTerminator, VirType, VirUnit, VirValue, SAFETY_PROOF_PENDING_OWNER,
    VIR_SCHEMA_VERSION,
};

fn bool_ty() -> VirType {
    VirType::Bool {}
}

fn i8_ty() -> VirType {
    VirType::Bv {
        width: BitVectorWidth::Bits8,
        signed: true,
    }
}

fn i16_ty() -> VirType {
    VirType::Bv {
        width: BitVectorWidth::Bits16,
        signed: true,
    }
}

fn u8_ty() -> VirType {
    VirType::Bv {
        width: BitVectorWidth::Bits8,
        signed: false,
    }
}

fn binding(id: &str, r#type: VirType) -> VirBinding {
    VirBinding {
        id: id.to_owned(),
        r#type,
    }
}

fn variable(id: &str) -> VirValue {
    VirValue::Variable(VirVariableRef { var: id.to_owned() })
}

fn boolean(value: bool) -> VirValue {
    VirValue::Boolean(VirBooleanLiteral { value })
}

fn integer(value: impl ToString, signed: bool) -> VirValue {
    VirValue::Integer(VirIntegerLiteral {
        int: VirIntLiteral {
            value: DecimalInteger::new(value.to_string()).expect("canonical integer"),
            width: BitVectorWidth::Bits8,
            signed,
        },
    })
}

fn result_expr(index: u32) -> VirContractExpr {
    VirContractExpr::Result(VirResultRef { result: index })
}

fn variable_expr(id: &str) -> VirContractExpr {
    VirContractExpr::Variable(VirVariableRef { var: id.to_owned() })
}

fn integer_expr(value: impl ToString, signed: bool) -> VirContractExpr {
    VirContractExpr::Integer(VirIntegerLiteral {
        int: VirIntLiteral {
            value: DecimalInteger::new(value.to_string()).expect("canonical integer"),
            width: BitVectorWidth::Bits8,
            signed,
        },
    })
}

fn equal(lhs: VirContractExpr, rhs: VirContractExpr) -> VirContractExpr {
    VirContractExpr::Binary(VirContractBinaryExpr {
        op: VirBinaryOperator::Eq,
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
    })
}

fn constant_instruction(id: &str, value: i32) -> VirInstruction {
    VirInstruction::Const {
        id: id.to_owned(),
        r#type: i8_ty(),
        value: match integer(value, true) {
            VirValue::Integer(literal) => mpk_vc::VirLiteral::Integer(literal),
            _ => unreachable!(),
        },
        safety_checks: Vec::new(),
    }
}

#[allow(clippy::too_many_arguments)]
fn module(
    profile: SemanticProfile,
    params: Vec<VirBinding>,
    result_type: VirType,
    locals: Vec<VirBinding>,
    blocks: Vec<VirBlock>,
    features_used: Vec<VirFeature>,
    requires: Vec<VirContractExpr>,
    ensures: Vec<VirContractExpr>,
) -> VirModule {
    let (source_language, unit_id, function_id, parameters) = match profile {
        SemanticProfile::GoFixedV0 => (
            SourceLanguage::Go,
            "example.com/demo",
            "example.com/demo.f",
            SemanticParameters::GoFixed(GoFixedParameters {
                target_id: "linux/amd64".to_owned(),
                pointer_width: PointerWidth::Bits64,
            }),
        ),
        SemanticProfile::RustCheckedV0 => (
            SourceLanguage::Rust,
            "demo",
            "demo::f",
            SemanticParameters::RustChecked(RustCheckedParameters {
                target_id: "x86_64-unknown-linux-gnu".to_owned(),
                pointer_width: PointerWidth::Bits64,
                overflow_mode: OverflowMode::Checked,
                panic_mode: PanicMode::Abort,
            }),
        ),
    };
    let mut contract = VirContract {
        unit_id: unit_id.to_owned(),
        function_id: function_id.to_owned(),
        semantic_profile: profile,
        semantic_parameters: parameters.clone(),
        requires,
        ensures,
        modifies: Vec::new(),
        panic: VirPanicPolicy::Forbidden,
        termination: VirTermination::Total,
        loops: Vec::new(),
        contract_hash: zero_hash(),
    };
    contract.contract_hash = contract_hash(&contract).expect("contract hash");
    let function = VirFunction {
        id: function_id.to_owned(),
        unit_id: unit_id.to_owned(),
        name: "f".to_owned(),
        params,
        results: vec![binding("result0", result_type)],
        locals,
        blocks,
        contracts: contract,
        features_used,
    };
    let mut module = VirModule {
        schema: VIR_SCHEMA_VERSION.to_owned(),
        source_language,
        semantic_profile: profile,
        semantic_parameters: parameters,
        units: vec![VirUnit {
            id: unit_id.to_owned(),
            name: "demo".to_owned(),
            type_decls: Vec::new(),
            const_decls: Vec::new(),
            functions: vec![function],
        }],
        vir_hash: zero_hash(),
    };
    module.vir_hash = vir_hash(&module).expect("VIR hash");
    validate_vir(&module).expect("test VIR validates");
    module
}

fn zero_hash() -> LowercaseSha256 {
    LowercaseSha256::new("0".repeat(64)).expect("zero hash")
}

fn post_members(output: &mpk_vc::ProgramVcModule) -> Vec<&mpk_vc::ProgramVcMember> {
    output.functions[0]
        .members
        .iter()
        .filter(|member| member.kind == ProgramVcMemberKind::Postcondition)
        .collect()
}

fn function_id(profile: SemanticProfile) -> &'static str {
    match profile {
        SemanticProfile::GoFixedV0 => "example.com/demo.f",
        SemanticProfile::RustCheckedV0 => "demo::f",
    }
}

fn safety_members(output: &mpk_vc::ProgramVcModule) -> Vec<&mpk_vc::ProgramVcMember> {
    output.functions[0]
        .members
        .iter()
        .filter(|member| member.kind == ProgramVcMemberKind::OperationSafety)
        .collect()
}

#[test]
fn diamond_join_substitutes_block_parameters_into_one_postcondition() {
    for profile in [SemanticProfile::GoFixedV0, SemanticProfile::RustCheckedV0] {
        let blocks = vec![
            VirBlock {
                label: "bb0".to_owned(),
                parameters: Vec::new(),
                instructions: Vec::new(),
                terminator: VirTerminator::Branch {
                    cond: variable("arg0"),
                    then_label: "bb2".to_owned(),
                    then_args: Vec::new(),
                    else_label: "bb1".to_owned(),
                    else_args: Vec::new(),
                },
            },
            VirBlock {
                label: "bb1".to_owned(),
                parameters: Vec::new(),
                instructions: vec![constant_instruction("t0", 1)],
                terminator: VirTerminator::Jump {
                    label: "bb3".to_owned(),
                    args: vec![variable("t0")],
                },
            },
            VirBlock {
                label: "bb2".to_owned(),
                parameters: Vec::new(),
                instructions: vec![constant_instruction("t1", 2)],
                terminator: VirTerminator::Jump {
                    label: "bb3".to_owned(),
                    args: vec![variable("t1")],
                },
            },
            VirBlock {
                label: "bb3".to_owned(),
                parameters: vec![binding("p0", i8_ty())],
                instructions: Vec::new(),
                terminator: VirTerminator::Return {
                    values: vec![variable("p0")],
                },
            },
        ];
        let input = module(
            profile,
            vec![binding("arg0", bool_ty())],
            i8_ty(),
            Vec::new(),
            blocks,
            vec![VirFeature::Branch],
            Vec::new(),
            vec![equal(result_expr(0), integer_expr(2, true))],
        );

        let output = generate_program_vcs(&input).expect("diamond generates");
        let posts = post_members(&output);
        assert_eq!(posts.len(), 1);
        assert_eq!(
            posts[0].id,
            format!("{}#postcondition#000000", function_id(profile))
        );
        assert_eq!(
            posts[0].group_id,
            format!("{}.contract", function_id(profile))
        );
        assert!(posts[0].local_binders.is_empty());
        assert!(posts[0].assumptions.is_empty());
        let MpkExprTerm::Apply { function, args } = &posts[0].conclusion else {
            panic!("postcondition is equality");
        };
        assert_eq!(function, "Std.Eq");
        assert_eq!(args.len(), 2);
        assert_eq!(
            args[0],
            MpkExprTerm::apply(
                "Std.Bool.if",
                [
                    MpkExprTerm::Var {
                        name: "arg0".to_owned()
                    },
                    MpkExprTerm::BitVecLiteral {
                        value: "2".to_owned(),
                        width: 8,
                        signed: true
                    },
                    MpkExprTerm::BitVecLiteral {
                        value: "1".to_owned(),
                        width: 8,
                        signed: true
                    }
                ]
            )
        );
    }
}

#[test]
fn nested_short_circuit_and_early_returns_keep_guard_order() {
    for profile in [SemanticProfile::GoFixedV0, SemanticProfile::RustCheckedV0] {
        let checked_divide = VirInstruction::BinOp {
            id: "t0".to_owned(),
            op: VirBinaryOperator::BvUdiv,
            r#type: u8_ty(),
            lhs: variable("arg2"),
            rhs: variable("arg3"),
            safety_checks: vec![VirSafetyCheck::DivisorNonzero {}],
        };
        let blocks = vec![
            VirBlock {
                label: "bb0".to_owned(),
                parameters: Vec::new(),
                instructions: Vec::new(),
                terminator: VirTerminator::Branch {
                    cond: variable("arg0"),
                    then_label: "bb2".to_owned(),
                    then_args: Vec::new(),
                    else_label: "bb1".to_owned(),
                    else_args: Vec::new(),
                },
            },
            VirBlock {
                label: "bb1".to_owned(),
                parameters: Vec::new(),
                instructions: Vec::new(),
                terminator: VirTerminator::Return {
                    values: vec![variable("arg2")],
                },
            },
            VirBlock {
                label: "bb2".to_owned(),
                parameters: Vec::new(),
                instructions: vec![checked_divide],
                terminator: VirTerminator::Branch {
                    cond: variable("arg1"),
                    then_label: "bb4".to_owned(),
                    then_args: vec![variable("t0")],
                    else_label: "bb3".to_owned(),
                    else_args: vec![variable("t0")],
                },
            },
            VirBlock {
                label: "bb3".to_owned(),
                parameters: vec![binding("p0", u8_ty())],
                instructions: Vec::new(),
                terminator: VirTerminator::Return {
                    values: vec![variable("p0")],
                },
            },
            VirBlock {
                label: "bb4".to_owned(),
                parameters: vec![binding("p1", u8_ty())],
                instructions: Vec::new(),
                terminator: VirTerminator::Return {
                    values: vec![variable("p1")],
                },
            },
        ];
        let input = module(
            profile,
            vec![
                binding("arg0", bool_ty()),
                binding("arg1", bool_ty()),
                binding("arg2", u8_ty()),
                binding("arg3", u8_ty()),
            ],
            u8_ty(),
            Vec::new(),
            blocks,
            vec![VirFeature::Branch],
            Vec::new(),
            vec![equal(result_expr(0), result_expr(0))],
        );

        let output = generate_program_vcs(&input).expect("nested CFG generates");
        let safety = safety_members(&output);
        assert_eq!(safety.len(), 1);
        assert_eq!(
            safety[0].id,
            format!("{}#operation_safety#000000", function_id(profile))
        );
        assert_eq!(
            safety[0].group_id,
            format!("{}.panic_free", function_id(profile))
        );
        assert!(safety[0].local_binders.is_empty());
        assert_eq!(
            safety[0].assumptions,
            vec![MpkExprTerm::Var {
                name: "arg0".to_owned()
            }]
        );
        assert_eq!(
            safety[0].safety_evidence,
            Some(SafetyEvidenceRoute::ProofPending {
                owner: SAFETY_PROOF_PENDING_OWNER
            })
        );

        let posts = post_members(&output);
        assert_eq!(posts.len(), 3);
        assert_eq!(
            posts[0].id,
            format!("{}#postcondition#000000", function_id(profile))
        );
        assert_eq!(
            posts[1].id,
            format!("{}#postcondition#000001", function_id(profile))
        );
        assert_eq!(
            posts[2].id,
            format!("{}#postcondition#000002", function_id(profile))
        );
        assert_eq!(posts[1].assumptions.len(), 2);
        assert_eq!(
            posts[1].assumptions[0],
            MpkExprTerm::Var {
                name: "arg0".to_owned()
            }
        );
        assert_eq!(
            posts[1].assumptions[1],
            MpkExprTerm::apply(
                "Std.Bool.not",
                [MpkExprTerm::Var {
                    name: "arg1".to_owned()
                }]
            )
        );
    }
}

#[test]
fn mutable_local_reassignment_merges_at_join_deterministically() {
    for profile in [SemanticProfile::GoFixedV0, SemanticProfile::RustCheckedV0] {
        let copy = |id: &str, value: VirValue| VirInstruction::Copy {
            id: id.to_owned(),
            r#type: i8_ty(),
            target: "local0".to_owned(),
            value,
            safety_checks: Vec::new(),
        };
        let blocks = vec![
            VirBlock {
                label: "bb0".to_owned(),
                parameters: Vec::new(),
                instructions: vec![copy("t0", integer(0, true))],
                terminator: VirTerminator::Branch {
                    cond: variable("arg0"),
                    then_label: "bb2".to_owned(),
                    then_args: Vec::new(),
                    else_label: "bb1".to_owned(),
                    else_args: Vec::new(),
                },
            },
            VirBlock {
                label: "bb1".to_owned(),
                parameters: Vec::new(),
                instructions: vec![copy("t1", integer(1, true))],
                terminator: VirTerminator::Jump {
                    label: "bb3".to_owned(),
                    args: vec![variable("local0")],
                },
            },
            VirBlock {
                label: "bb2".to_owned(),
                parameters: Vec::new(),
                instructions: vec![copy("t2", integer(2, true))],
                terminator: VirTerminator::Jump {
                    label: "bb3".to_owned(),
                    args: vec![variable("local0")],
                },
            },
            VirBlock {
                label: "bb3".to_owned(),
                parameters: vec![binding("p0", i8_ty())],
                instructions: vec![copy("t3", variable("p0"))],
                terminator: VirTerminator::Return {
                    values: vec![variable("local0")],
                },
            },
        ];
        let input = module(
            profile,
            vec![binding("arg0", bool_ty())],
            i8_ty(),
            vec![binding("local0", i8_ty())],
            blocks,
            vec![VirFeature::Branch, VirFeature::MutableLocal],
            Vec::new(),
            vec![equal(result_expr(0), integer_expr(2, true))],
        );

        let first = generate_program_vcs(&input).expect("mutable join generates");
        let second = generate_program_vcs(&input).expect("repeat generation");
        assert_eq!(first, second);
        assert_eq!(post_members(&first).len(), 1);
        assert!(safety_members(&first).is_empty());
        assert!(contains_function(
            &post_members(&first)[0].conclusion,
            "Std.Bool.if"
        ));
    }
}

#[test]
fn empty_safety_paths_are_identical_for_go_and_rust() {
    for profile in [SemanticProfile::GoFixedV0, SemanticProfile::RustCheckedV0] {
        let blocks = vec![VirBlock {
            label: "bb0".to_owned(),
            parameters: Vec::new(),
            instructions: vec![VirInstruction::BinOp {
                id: "t0".to_owned(),
                op: VirBinaryOperator::BvAnd,
                r#type: u8_ty(),
                lhs: variable("arg0"),
                rhs: integer(1, false),
                safety_checks: Vec::new(),
            }],
            terminator: VirTerminator::Return {
                values: vec![variable("t0")],
            },
        }];
        let input = module(
            profile,
            vec![binding("arg0", u8_ty())],
            u8_ty(),
            Vec::new(),
            blocks,
            Vec::new(),
            vec![VirContractExpr::Boolean(VirBooleanLiteral { value: true })],
            vec![equal(result_expr(0), variable_expr("arg0"))],
        );
        let output = generate_program_vcs(&input).expect("empty safety path generates");
        assert!(safety_members(&output).is_empty());
        assert_eq!(post_members(&output).len(), 1);
        assert_eq!(
            output.functions[0].requires,
            vec![MpkExprTerm::Constant {
                name: "Std.Bool.true".to_owned()
            }]
        );
        assert!(post_members(&output)[0].assumptions.is_empty());
    }
}

#[test]
fn symbolic_go_safety_is_proof_pending_without_changing_value_semantics() {
    let blocks = vec![VirBlock {
        label: "bb0".to_owned(),
        parameters: Vec::new(),
        instructions: vec![VirInstruction::BinOp {
            id: "t0".to_owned(),
            op: VirBinaryOperator::BvShl,
            r#type: u8_ty(),
            lhs: variable("arg0"),
            rhs: variable("arg1"),
            safety_checks: vec![VirSafetyCheck::ShiftCountNonnegative {}],
        }],
        terminator: VirTerminator::Return {
            values: vec![variable("t0")],
        },
    }];
    let input = module(
        SemanticProfile::GoFixedV0,
        vec![binding("arg0", u8_ty()), binding("arg1", i16_ty())],
        u8_ty(),
        Vec::new(),
        blocks,
        Vec::new(),
        Vec::new(),
        vec![equal(result_expr(0), result_expr(0))],
    );

    let output = generate_program_vcs(&input).expect("Go shift generates");
    let safety = safety_members(&output);
    assert_eq!(safety.len(), 1);
    assert_eq!(
        safety[0].safety_evidence,
        Some(SafetyEvidenceRoute::ProofPending {
            owner: SAFETY_PROOF_PENDING_OWNER
        })
    );
    assert!(contains_function(
        &post_members(&output)[0].conclusion,
        "Std.Bool.if"
    ));
}

#[test]
fn invalid_unreachable_block_never_produces_a_partial_artifact() {
    let blocks = vec![VirBlock {
        label: "bb0".to_owned(),
        parameters: Vec::new(),
        instructions: Vec::new(),
        terminator: VirTerminator::Return {
            values: vec![boolean(true)],
        },
    }];
    let mut input = module(
        SemanticProfile::RustCheckedV0,
        Vec::new(),
        bool_ty(),
        Vec::new(),
        blocks,
        Vec::new(),
        Vec::new(),
        vec![VirContractExpr::Boolean(VirBooleanLiteral { value: true })],
    );
    input.units[0].functions[0].blocks.push(VirBlock {
        label: "bb1".to_owned(),
        parameters: Vec::new(),
        instructions: Vec::new(),
        terminator: VirTerminator::Return {
            values: vec![boolean(false)],
        },
    });
    input.vir_hash = vir_hash(&input).expect("mutated VIR hash");

    let error = generate_program_vcs(&input).unwrap_err();
    assert_eq!(error.code(), "VC_PROGRAM_VIR_INVALID");
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
