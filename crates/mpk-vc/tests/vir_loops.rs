use mpk_vc::vir::{
    VirBooleanLiteral, VirContractBinaryExpr, VirIntegerLiteral, VirPanicPolicy, VirResultRef,
    VirTermination, VirVariableRef,
};
use mpk_vc::{
    contract_hash, generate_program_vcs, validate_vir, vir_hash, BitVectorWidth, DecimalInteger,
    GoFixedParameters, LowercaseSha256, MpkExprTerm, PanicMode, PointerWidth, ProgramVcMemberKind,
    ProgramWpError, RustCheckedParameters, SafetyEvidenceRoute, SemanticParameters,
    SemanticProfile, SourceLanguage, VirBinaryOperator, VirBinding, VirBlock, VirContract,
    VirContractExpr, VirFeature, VirFunction, VirInstruction, VirIntLiteral, VirLiteral,
    VirLoopContract, VirModule, VirSafetyCheck, VirTerminator, VirType, VirUnit, VirValue,
    SAFETY_GROUPED_CERTIFICATE_FOUNDATION, VIR_SCHEMA_VERSION,
};

fn bool_ty() -> VirType {
    VirType::Bool {}
}

fn bv_ty(signed: bool) -> VirType {
    VirType::Bv {
        width: BitVectorWidth::Bits8,
        signed,
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

fn integer(value: impl ToString, signed: bool) -> VirValue {
    VirValue::Integer(VirIntegerLiteral {
        int: VirIntLiteral {
            value: DecimalInteger::new(value.to_string()).expect("canonical integer"),
            width: BitVectorWidth::Bits8,
            signed,
        },
    })
}

fn boolean(value: bool) -> VirValue {
    VirValue::Boolean(VirBooleanLiteral { value })
}

fn variable_expr(id: &str) -> VirContractExpr {
    VirContractExpr::Variable(VirVariableRef { var: id.to_owned() })
}

fn result_expr(index: u32) -> VirContractExpr {
    VirContractExpr::Result(VirResultRef { result: index })
}

fn binary_expr(
    op: VirBinaryOperator,
    lhs: VirContractExpr,
    rhs: VirContractExpr,
) -> VirContractExpr {
    VirContractExpr::Binary(VirContractBinaryExpr {
        op,
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
    })
}

fn zero_hash() -> LowercaseSha256 {
    LowercaseSha256::new("0".repeat(64)).expect("zero hash")
}

fn rehash(module: &mut VirModule) {
    let contract = &mut module.units[0].functions[0].contracts;
    contract.contract_hash = contract_hash(contract).expect("contract hash");
    module.vir_hash = vir_hash(module).expect("VIR hash");
}

fn loop_module(total: bool, signed: bool) -> VirModule {
    let r#type = bv_ty(signed);
    let less = if signed {
        VirBinaryOperator::SignedLt
    } else {
        VirBinaryOperator::UnsignedLt
    };
    let less_equal = if signed {
        VirBinaryOperator::SignedLe
    } else {
        VirBinaryOperator::UnsignedLe
    };
    let blocks = vec![
        VirBlock {
            label: "bb0".to_owned(),
            parameters: Vec::new(),
            instructions: vec![
                VirInstruction::Const {
                    id: "t0".to_owned(),
                    r#type: r#type.clone(),
                    value: match integer(0, signed) {
                        VirValue::Integer(value) => VirLiteral::Integer(value),
                        _ => unreachable!(),
                    },
                    safety_checks: Vec::new(),
                },
                VirInstruction::Copy {
                    id: "t1".to_owned(),
                    r#type: r#type.clone(),
                    target: "local0".to_owned(),
                    value: variable("t0"),
                    safety_checks: Vec::new(),
                },
            ],
            terminator: VirTerminator::Jump {
                label: "bb1".to_owned(),
                args: vec![variable("arg0")],
            },
        },
        VirBlock {
            label: "bb1".to_owned(),
            parameters: vec![binding("p0", r#type.clone())],
            instructions: vec![VirInstruction::BinOp {
                id: "t2".to_owned(),
                op: less,
                r#type: bool_ty(),
                lhs: variable("local0"),
                rhs: variable("p0"),
                safety_checks: Vec::new(),
            }],
            terminator: VirTerminator::Branch {
                cond: variable("t2"),
                then_label: "bb3".to_owned(),
                then_args: vec![variable("p0")],
                else_label: "bb2".to_owned(),
                else_args: vec![variable("p0")],
            },
        },
        VirBlock {
            label: "bb2".to_owned(),
            parameters: vec![binding("p1", r#type.clone())],
            instructions: Vec::new(),
            terminator: VirTerminator::Return {
                values: vec![variable("local0")],
            },
        },
        VirBlock {
            label: "bb3".to_owned(),
            parameters: vec![binding("p2", r#type.clone())],
            instructions: vec![
                VirInstruction::BinOp {
                    id: "t3".to_owned(),
                    op: VirBinaryOperator::BvAdd,
                    r#type: r#type.clone(),
                    lhs: variable("local0"),
                    rhs: integer(1, signed),
                    safety_checks: Vec::new(),
                },
                VirInstruction::Copy {
                    id: "t4".to_owned(),
                    r#type: r#type.clone(),
                    target: "local0".to_owned(),
                    value: variable("t3"),
                    safety_checks: Vec::new(),
                },
            ],
            terminator: VirTerminator::Jump {
                label: "bb1".to_owned(),
                args: vec![variable("p2")],
            },
        },
    ];
    let invariant = binary_expr(less_equal, variable_expr("local0"), variable_expr("p0"));
    let decreases = total
        .then(|| {
            binary_expr(
                VirBinaryOperator::BvSub,
                variable_expr("p0"),
                variable_expr("local0"),
            )
        })
        .into_iter()
        .collect();
    let parameters = SemanticParameters::GoFixed(GoFixedParameters {
        target_id: "linux/amd64".to_owned(),
        pointer_width: PointerWidth::Bits64,
    });
    let mut contract = VirContract {
        unit_id: "example.com/demo".to_owned(),
        function_id: "example.com/demo.Loop".to_owned(),
        semantic_profile: SemanticProfile::GoFixedV0,
        semantic_parameters: parameters.clone(),
        requires: Vec::new(),
        ensures: vec![binary_expr(
            less_equal,
            result_expr(0),
            variable_expr("arg0"),
        )],
        modifies: Vec::new(),
        panic: VirPanicPolicy::Forbidden,
        termination: if total {
            VirTermination::Total
        } else {
            VirTermination::Partial
        },
        loops: vec![VirLoopContract {
            header: "bb1".to_owned(),
            invariants: vec![invariant],
            decreases,
        }],
        contract_hash: zero_hash(),
    };
    contract.contract_hash = contract_hash(&contract).expect("contract hash");
    let function = VirFunction {
        id: "example.com/demo.Loop".to_owned(),
        unit_id: "example.com/demo".to_owned(),
        name: "Loop".to_owned(),
        params: vec![binding("arg0", r#type.clone())],
        results: vec![binding("result0", r#type.clone())],
        locals: vec![binding("local0", r#type)],
        blocks,
        contracts: contract,
        features_used: vec![
            VirFeature::Branch,
            VirFeature::CyclicCfg,
            VirFeature::MutableLocal,
        ],
    };
    let mut module = VirModule {
        schema: VIR_SCHEMA_VERSION.to_owned(),
        source_language: SourceLanguage::Go,
        semantic_profile: SemanticProfile::GoFixedV0,
        semantic_parameters: parameters,
        units: vec![VirUnit {
            id: "example.com/demo".to_owned(),
            name: "demo".to_owned(),
            type_decls: Vec::new(),
            const_decls: Vec::new(),
            functions: vec![function],
        }],
        vir_hash: zero_hash(),
    };
    module.vir_hash = vir_hash(&module).expect("VIR hash");
    validate_vir(&module).expect("loop VIR validates");
    module
}

fn two_backedge_loop_module() -> VirModule {
    let mut module = loop_module(true, false);
    let function = &mut module.units[0].functions[0];
    function.blocks[3] = VirBlock {
        label: "bb3".to_owned(),
        parameters: vec![binding("p2", bv_ty(false))],
        instructions: Vec::new(),
        terminator: VirTerminator::Branch {
            cond: boolean(true),
            then_label: "bb5".to_owned(),
            then_args: vec![variable("p2")],
            else_label: "bb4".to_owned(),
            else_args: vec![variable("p2")],
        },
    };
    function
        .blocks
        .push(backedge_block("bb4", "p3", "t3", "t4", 1));
    function
        .blocks
        .push(backedge_block("bb5", "p4", "t5", "t6", 2));
    rehash(&mut module);
    validate_vir(&module).expect("two-backedge loop validates");
    module
}

fn two_return_loop_module() -> VirModule {
    let mut module = loop_module(false, false);
    let function = &mut module.units[0].functions[0];
    function.blocks[2].terminator = VirTerminator::Branch {
        cond: boolean(true),
        then_label: "bb5".to_owned(),
        then_args: vec![variable("p1")],
        else_label: "bb4".to_owned(),
        else_args: vec![variable("p1")],
    };
    function.blocks.push(VirBlock {
        label: "bb4".to_owned(),
        parameters: vec![binding("p3", bv_ty(false))],
        instructions: Vec::new(),
        terminator: VirTerminator::Return {
            values: vec![variable("local0")],
        },
    });
    function.blocks.push(VirBlock {
        label: "bb5".to_owned(),
        parameters: vec![binding("p4", bv_ty(false))],
        instructions: Vec::new(),
        terminator: VirTerminator::Return {
            values: vec![variable("p4")],
        },
    });
    rehash(&mut module);
    validate_vir(&module).expect("two-return loop validates");
    module
}

fn backedge_block(
    label: &str,
    parameter: &str,
    value_id: &str,
    copy_id: &str,
    increment: u8,
) -> VirBlock {
    VirBlock {
        label: label.to_owned(),
        parameters: vec![binding(parameter, bv_ty(false))],
        instructions: vec![
            VirInstruction::BinOp {
                id: value_id.to_owned(),
                op: VirBinaryOperator::BvAdd,
                r#type: bv_ty(false),
                lhs: variable("local0"),
                rhs: integer(increment, false),
                safety_checks: Vec::new(),
            },
            VirInstruction::Copy {
                id: copy_id.to_owned(),
                r#type: bv_ty(false),
                target: "local0".to_owned(),
                value: variable(value_id),
                safety_checks: Vec::new(),
            },
        ],
        terminator: VirTerminator::Jump {
            label: "bb1".to_owned(),
            args: vec![variable(parameter)],
        },
    }
}

fn members(
    output: &mpk_vc::ProgramVcModule,
    kind: ProgramVcMemberKind,
) -> Vec<&mpk_vc::ProgramVcMember> {
    output.functions[0]
        .members
        .iter()
        .filter(|member| member.kind == kind)
        .collect()
}

fn bound(index: u32) -> MpkExprTerm {
    MpkExprTerm::Bound { index }
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

fn contains_literal(term: &MpkExprTerm, expected: &str) -> bool {
    match term {
        MpkExprTerm::BitVecLiteral { value, .. } => value == expected,
        MpkExprTerm::Apply { args, .. } => args
            .iter()
            .any(|argument| contains_literal(argument, expected)),
        MpkExprTerm::Convert { value, .. } => contains_literal(value, expected),
        _ => false,
    }
}

#[test]
fn partial_loop_generates_initialization_preservation_and_exit_members() {
    let output = generate_program_vcs(&loop_module(false, false)).expect("partial loop generates");
    let function = &output.functions[0];

    assert_eq!(function.members.len(), 3);
    assert!(members(&output, ProgramVcMemberKind::LoopDecreases).is_empty());
    assert!(members(&output, ProgramVcMemberKind::Postcondition).is_empty());

    let initialization = members(&output, ProgramVcMemberKind::LoopInitialization);
    assert_eq!(initialization.len(), 1);
    assert_eq!(
        initialization[0].id,
        "example.com/demo.Loop#loop_initialization#000000"
    );
    assert!(initialization[0].local_binders.is_empty());
    assert!(initialization[0].assumptions.is_empty());

    let preservation = members(&output, ProgramVcMemberKind::LoopPreservation);
    assert_eq!(preservation.len(), 1);
    assert_eq!(preservation[0].local_binders.len(), 2);
    assert_eq!(preservation[0].assumptions.len(), 2);
    assert!(contains_function(
        &preservation[0].conclusion,
        "Std.BitVec.BV8.add"
    ));
    assert!(format!("{:?}", preservation[0]).contains("Bound { index: 1 }"));
    assert!(format!("{:?}", preservation[0]).contains("Bound { index: 0 }"));

    let exit = members(&output, ProgramVcMemberKind::LoopExit);
    assert_eq!(exit.len(), 1);
    assert_eq!(exit[0].local_binders.len(), 2);
    assert_eq!(exit[0].assumptions.len(), 2);
    assert_eq!(exit[0].group_id, "example.com/demo.Loop.contract");
}

#[test]
fn total_unsigned_loop_adds_one_strict_decrease_member() {
    let output = generate_program_vcs(&loop_module(true, false)).expect("total loop generates");
    let decreases = members(&output, ProgramVcMemberKind::LoopDecreases);

    assert_eq!(decreases.len(), 1);
    assert_eq!(
        decreases[0].id,
        "example.com/demo.Loop#loop_decreases#000000"
    );
    assert_eq!(decreases[0].local_binders.len(), 2);
    assert!(contains_function(
        &decreases[0].conclusion,
        "Std.BitVec.BV8.ult"
    ));
}

#[test]
fn total_signed_loop_orders_nonnegative_before_strict_decrease() {
    let output = generate_program_vcs(&loop_module(true, true)).expect("signed loop generates");
    let decreases = members(&output, ProgramVcMemberKind::LoopDecreases);

    assert_eq!(decreases.len(), 2);
    assert!(contains_function(
        &decreases[0].conclusion,
        "Std.BitVec.BV8.sge"
    ));
    assert!(contains_function(
        &decreases[1].conclusion,
        "Std.BitVec.BV8.slt"
    ));
}

#[test]
fn every_backedge_generates_one_preservation_and_strict_decrease_member_in_block_order() {
    let output = generate_program_vcs(&two_backedge_loop_module()).expect("two backedges generate");
    let preservation = members(&output, ProgramVcMemberKind::LoopPreservation);
    let decreases = members(&output, ProgramVcMemberKind::LoopDecreases);

    assert_eq!(preservation.len(), 2);
    assert!(contains_literal(&preservation[0].conclusion, "1"));
    assert!(contains_literal(&preservation[1].conclusion, "2"));
    assert_eq!(decreases.len(), 2);
    assert!(contains_literal(&decreases[0].conclusion, "1"));
    assert!(contains_literal(&decreases[1].conclusion, "2"));
}

#[test]
fn every_ordered_loop_clause_generates_exactly_one_member_per_required_origin() {
    let mut input = loop_module(true, false);
    let contract = &mut input.units[0].functions[0].contracts;
    let invariant = contract.loops[0].invariants[0].clone();
    let decrease = contract.loops[0].decreases[0].clone();
    contract.loops[0].invariants.push(invariant);
    contract.loops[0].decreases.push(decrease);
    contract.ensures.push(contract.ensures[0].clone());
    rehash(&mut input);
    validate_vir(&input).expect("multi-clause loop validates");

    let output = generate_program_vcs(&input).expect("multi-clause loop generates");

    assert_eq!(
        members(&output, ProgramVcMemberKind::LoopInitialization).len(),
        2
    );
    assert_eq!(
        members(&output, ProgramVcMemberKind::LoopPreservation).len(),
        2
    );
    assert_eq!(members(&output, ProgramVcMemberKind::LoopExit).len(), 2);
    assert_eq!(
        members(&output, ProgramVcMemberKind::LoopDecreases).len(),
        2
    );
}

#[test]
fn multiple_return_blocks_share_one_exit_member_per_header_and_ensure() {
    let output = generate_program_vcs(&two_return_loop_module()).expect("two exits generate");
    let exits = members(&output, ProgramVcMemberKind::LoopExit);

    assert_eq!(exits.len(), 1);
    assert!(contains_function(&exits[0].conclusion, "Std.Bool.if"));
}

#[test]
fn safety_inside_loop_uses_the_same_cutpoint_scope_and_path_guards() {
    let mut input = loop_module(false, false);
    let VirInstruction::BinOp {
        op,
        rhs,
        safety_checks,
        ..
    } = &mut input.units[0].functions[0].blocks[3].instructions[0]
    else {
        panic!("body begins with BinOp");
    };
    *op = VirBinaryOperator::BvUdiv;
    *rhs = variable("p2");
    *safety_checks = vec![VirSafetyCheck::DivisorNonzero {}];
    rehash(&mut input);
    validate_vir(&input).expect("division loop validates");

    let output = generate_program_vcs(&input).expect("division loop generates");
    let safety = members(&output, ProgramVcMemberKind::OperationSafety);

    assert_eq!(safety.len(), 1);
    assert_eq!(safety[0].local_binders.len(), 2);
    assert_eq!(safety[0].assumptions.len(), 2);
    assert_eq!(
        safety[0].safety_evidence,
        Some(SafetyEvidenceRoute::GroupedCertificate {
            foundation: SAFETY_GROUPED_CERTIFICATE_FOUNDATION
        })
    );
}

#[test]
fn validation_rejects_uncovered_loop_and_partial_decreases_without_artifact() {
    let mut uncovered = loop_module(false, false);
    uncovered.units[0].functions[0].contracts.loops.clear();
    uncovered.units[0].functions[0].contracts.termination = VirTermination::Total;
    rehash(&mut uncovered);
    let error = generate_program_vcs(&uncovered).expect_err("uncovered cycle rejects");
    assert!(matches!(
        error,
        ProgramWpError::Validation(source) if source.code() == "VIR_LOOP_CUTPOINT"
    ));

    let mut mixed = loop_module(true, false);
    mixed.units[0].functions[0].contracts.termination = VirTermination::Partial;
    rehash(&mut mixed);
    let error = generate_program_vcs(&mixed).expect_err("partial decreases rejects");
    assert!(matches!(
        error,
        ProgramWpError::Validation(source) if source.code() == "VIR_LOOP_TERMINATION"
    ));
}

#[test]
fn rust_profile_rejects_the_identical_cyclic_cfg_before_wp_generation() {
    let mut input = loop_module(true, false);
    input.source_language = SourceLanguage::Rust;
    input.semantic_profile = SemanticProfile::RustCheckedV0;
    input.semantic_parameters = SemanticParameters::RustChecked(RustCheckedParameters {
        target_id: "x86_64-unknown-linux-gnu".to_owned(),
        pointer_width: PointerWidth::Bits64,
        overflow_mode: mpk_vc::OverflowMode::Checked,
        panic_mode: PanicMode::Abort,
    });
    let unit = &mut input.units[0];
    unit.id = "demo".to_owned();
    let function = &mut unit.functions[0];
    function.id = "demo::Loop".to_owned();
    function.unit_id = "demo".to_owned();
    function.contracts.unit_id = "demo".to_owned();
    function.contracts.function_id = "demo::Loop".to_owned();
    function.contracts.semantic_profile = input.semantic_profile;
    function.contracts.semantic_parameters = input.semantic_parameters.clone();
    rehash(&mut input);

    let error = generate_program_vcs(&input).expect_err("Rust cycle rejects");
    assert!(matches!(
        error,
        ProgramWpError::Validation(source) if source.code() == "VIR_RUST_CYCLIC_CFG"
    ));
}

#[test]
fn bound_term_serializes_with_the_vc_v1_discriminator() {
    assert_eq!(
        serde_json::to_value(bound(3)).expect("bound serializes"),
        serde_json::json!({"kind":"bound","index":3})
    );
}
