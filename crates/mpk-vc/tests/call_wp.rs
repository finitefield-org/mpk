use mpk_vc::vir::{
    VirContractBinaryExpr, VirPanicPolicy, VirResultRef, VirTermination, VirVariableRef,
};
use mpk_vc::{
    contract_hash, generate_program_vcs, program_declaration_name, vir_hash, BitVectorWidth,
    LowercaseSha256, MpkExprTerm, OverflowMode, PanicMode, PointerWidth, ProgramDeclarationKind,
    ProgramVcMemberKind, ProgramWpError, RustCheckedParameters, SemanticParameters,
    SemanticProfile, SourceLanguage, VirBinaryOperator, VirBinding, VirBlock, VirContract,
    VirContractExpr, VirFeature, VirFunction, VirInstruction, VirModule, VirTerminator, VirType,
    VirUnit, VirValue, VIR_SCHEMA_VERSION,
};

const UNIT_ID: &str = "demo";
const CALLEE_ID: &str = "demo::callee";
const CALLER_ID: &str = "demo::caller";

fn i8_ty() -> VirType {
    VirType::Bv {
        width: BitVectorWidth::Bits8,
        signed: true,
    }
}

fn bool_ty() -> VirType {
    VirType::Bool {}
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

fn variable_expr(id: &str) -> VirContractExpr {
    VirContractExpr::Variable(VirVariableRef { var: id.to_owned() })
}

fn result_expr(index: u32) -> VirContractExpr {
    VirContractExpr::Result(VirResultRef { result: index })
}

fn equal(lhs: VirContractExpr, rhs: VirContractExpr) -> VirContractExpr {
    VirContractExpr::Binary(VirContractBinaryExpr {
        op: VirBinaryOperator::Eq,
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
    })
}

fn parameters() -> SemanticParameters {
    SemanticParameters::RustChecked(RustCheckedParameters {
        target_id: "x86_64-unknown-linux-gnu".to_owned(),
        pointer_width: PointerWidth::Bits64,
        overflow_mode: OverflowMode::Checked,
        panic_mode: PanicMode::Abort,
    })
}

fn zero_hash() -> LowercaseSha256 {
    LowercaseSha256::new("0".repeat(64)).expect("zero hash")
}

fn contract(function_id: &str, params: &[VirBinding]) -> VirContract {
    let first_param = params.first().map(|parameter| variable_expr(&parameter.id));
    let mut contract = VirContract {
        unit_id: UNIT_ID.to_owned(),
        function_id: function_id.to_owned(),
        semantic_profile: SemanticProfile::RustCheckedV0,
        semantic_parameters: parameters(),
        requires: first_param
            .clone()
            .into_iter()
            .map(|value| equal(value.clone(), value))
            .collect(),
        ensures: vec![first_param.map_or_else(
            || equal(result_expr(0), result_expr(0)),
            |value| equal(result_expr(0), value),
        )],
        modifies: Vec::new(),
        panic: VirPanicPolicy::Forbidden,
        termination: VirTermination::Total,
        loops: Vec::new(),
        contract_hash: zero_hash(),
    };
    contract.contract_hash = contract_hash(&contract).expect("contract hash");
    contract
}

fn callee() -> VirFunction {
    let params = vec![binding("arg0", i8_ty())];
    VirFunction {
        id: CALLEE_ID.to_owned(),
        unit_id: UNIT_ID.to_owned(),
        name: "callee".to_owned(),
        params: params.clone(),
        results: vec![binding("result0", i8_ty())],
        locals: Vec::new(),
        blocks: vec![VirBlock {
            label: "bb0".to_owned(),
            parameters: Vec::new(),
            instructions: Vec::new(),
            terminator: VirTerminator::Return {
                values: vec![variable("arg0")],
            },
        }],
        contracts: contract(CALLEE_ID, &params),
        features_used: Vec::new(),
    }
}

fn call(id: &str, argument: &str, callee_hash: LowercaseSha256) -> VirInstruction {
    VirInstruction::CallStatic {
        id: id.to_owned(),
        r#type: i8_ty(),
        function: CALLEE_ID.to_owned(),
        contract_hash: callee_hash,
        args: vec![variable(argument)],
        safety_checks: Vec::new(),
    }
}

fn direct_caller(callee_hash: LowercaseSha256) -> VirFunction {
    let params = vec![binding("arg0", i8_ty())];
    VirFunction {
        id: CALLER_ID.to_owned(),
        unit_id: UNIT_ID.to_owned(),
        name: "caller".to_owned(),
        params: params.clone(),
        results: vec![binding("result0", i8_ty())],
        locals: Vec::new(),
        blocks: vec![VirBlock {
            label: "bb0".to_owned(),
            parameters: Vec::new(),
            instructions: vec![call("t0", "arg0", callee_hash)],
            terminator: VirTerminator::Return {
                values: vec![variable("t0")],
            },
        }],
        contracts: contract(CALLER_ID, &params),
        features_used: vec![VirFeature::CallStatic],
    }
}

fn module_with(functions: Vec<VirFunction>) -> VirModule {
    let mut module = VirModule {
        schema: VIR_SCHEMA_VERSION.to_owned(),
        source_language: SourceLanguage::Rust,
        semantic_profile: SemanticProfile::RustCheckedV0,
        semantic_parameters: parameters(),
        units: vec![VirUnit {
            id: UNIT_ID.to_owned(),
            name: "demo".to_owned(),
            type_decls: Vec::new(),
            const_decls: Vec::new(),
            functions,
        }],
        vir_hash: zero_hash(),
    };
    module.vir_hash = vir_hash(&module).expect("VIR hash");
    module
}

fn direct_module() -> VirModule {
    let callee = callee();
    let caller = direct_caller(callee.contracts.contract_hash.clone());
    module_with(vec![callee, caller])
}

fn function<'a>(output: &'a mpk_vc::ProgramVcModule, id: &str) -> &'a mpk_vc::ProgramVcFunction {
    output
        .functions
        .iter()
        .find(|function| function.function_id == id)
        .expect("function output")
}

fn members(
    function: &mpk_vc::ProgramVcFunction,
    kind: ProgramVcMemberKind,
) -> Vec<&mpk_vc::ProgramVcMember> {
    function
        .members
        .iter()
        .filter(|member| member.kind == kind)
        .collect()
}

fn contains_bound(term: &MpkExprTerm, expected: u32) -> bool {
    match term {
        MpkExprTerm::Bound { index } => *index == expected,
        MpkExprTerm::Apply { args, .. } => args.iter().any(|term| contains_bound(term, expected)),
        MpkExprTerm::Convert { value, .. } => contains_bound(value, expected),
        MpkExprTerm::Forall { body, .. } => contains_bound(body, expected),
        _ => false,
    }
}

fn forall_depth(term: &MpkExprTerm) -> usize {
    match term {
        MpkExprTerm::Forall { body, .. } => 1 + forall_depth(body),
        MpkExprTerm::Apply { args, .. } => args.iter().map(forall_depth).max().unwrap_or(0),
        MpkExprTerm::Convert { value, .. } => forall_depth(value),
        _ => 0,
    }
}

fn encoded_equal(lhs: MpkExprTerm, rhs: MpkExprTerm) -> MpkExprTerm {
    MpkExprTerm::Apply {
        function: "Std.Eq".to_owned(),
        args: vec![lhs, rhs],
    }
}

#[test]
fn direct_call_generates_contract_bound_members_and_fresh_result_continuation() {
    let output = generate_program_vcs(&direct_module()).expect("direct call generates");
    assert_eq!(
        output
            .functions
            .iter()
            .map(|function| function.function_id.as_str())
            .collect::<Vec<_>>(),
        vec![CALLEE_ID, CALLER_ID]
    );
    let caller = function(&output, CALLER_ID);

    assert_eq!(
        members(caller, ProgramVcMemberKind::CalleePrecondition).len(),
        1
    );
    let panic = members(caller, ProgramVcMemberKind::CalleePanicFree);
    assert_eq!(panic.len(), 1);
    assert_eq!(panic[0].group_id, format!("{CALLER_ID}.panic_free"));
    let post = members(caller, ProgramVcMemberKind::Postcondition);
    assert_eq!(post.len(), 1);
    let result_equals_argument = encoded_equal(
        MpkExprTerm::Bound { index: 0 },
        MpkExprTerm::Var {
            name: "arg0".to_owned(),
        },
    );
    assert_eq!(
        post[0].conclusion,
        MpkExprTerm::Forall {
            binder_type: mpk_vc::MpkTypeTerm::Constant {
                name: "Std.Program.Base.Int8".to_owned(),
            },
            body: Box::new(MpkExprTerm::Apply {
                function: "Std.Logic.Imp".to_owned(),
                args: vec![result_equals_argument.clone(), result_equals_argument],
            }),
        }
    );

    let callee_contract = program_declaration_name(CALLEE_ID, ProgramDeclarationKind::Contract);
    let callee_panic = program_declaration_name(CALLEE_ID, ProgramDeclarationKind::PanicFree);
    assert_eq!(
        callee_contract,
        "VC.Function.f64656d6f3a3a63616c6c6565.contract"
    );
    assert_eq!(caller.direct_callees, vec![CALLEE_ID]);
    assert_eq!(caller.contract_dependencies, vec![callee_contract.clone()]);
    let mut expected_panic_dependencies = vec![
        program_declaration_name(CALLER_ID, ProgramDeclarationKind::Contract),
        callee_contract,
        callee_panic,
    ];
    expected_panic_dependencies.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
    assert_eq!(caller.panic_free_dependencies, expected_panic_dependencies);
}

#[test]
fn branched_call_members_retain_the_call_path_guard() {
    let callee = callee();
    let mut caller = direct_caller(callee.contracts.contract_hash.clone());
    caller.params.push(binding("arg1", bool_ty()));
    caller.contracts = contract(CALLER_ID, &caller.params);
    caller.blocks = vec![
        VirBlock {
            label: "bb0".to_owned(),
            parameters: Vec::new(),
            instructions: Vec::new(),
            terminator: VirTerminator::Branch {
                cond: variable("arg1"),
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
                values: vec![variable("arg0")],
            },
        },
        VirBlock {
            label: "bb2".to_owned(),
            parameters: Vec::new(),
            instructions: vec![call("t0", "arg0", callee.contracts.contract_hash.clone())],
            terminator: VirTerminator::Return {
                values: vec![variable("t0")],
            },
        },
    ];
    caller.features_used = vec![VirFeature::Branch, VirFeature::CallStatic];
    let output =
        generate_program_vcs(&module_with(vec![callee, caller])).expect("branch generates");
    let caller = function(&output, CALLER_ID);

    for member in members(caller, ProgramVcMemberKind::CalleePrecondition)
        .into_iter()
        .chain(members(caller, ProgramVcMemberKind::CalleePanicFree))
    {
        assert_eq!(
            member.assumptions,
            vec![MpkExprTerm::Var {
                name: "arg1".to_owned()
            }]
        );
    }
    assert_eq!(members(caller, ProgramVcMemberKind::Postcondition).len(), 2);
}

#[test]
fn branch_join_keeps_call_and_non_call_continuations_separate() {
    let callee = callee();
    let mut caller = direct_caller(callee.contracts.contract_hash.clone());
    caller.params.push(binding("arg1", bool_ty()));
    caller.contracts = contract(CALLER_ID, &caller.params);
    caller.blocks = vec![
        VirBlock {
            label: "bb0".to_owned(),
            parameters: Vec::new(),
            instructions: Vec::new(),
            terminator: VirTerminator::Branch {
                cond: variable("arg1"),
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
            terminator: VirTerminator::Jump {
                label: "bb3".to_owned(),
                args: vec![variable("arg0")],
            },
        },
        VirBlock {
            label: "bb2".to_owned(),
            parameters: Vec::new(),
            instructions: vec![call("t0", "arg0", callee.contracts.contract_hash.clone())],
            terminator: VirTerminator::Jump {
                label: "bb3".to_owned(),
                args: vec![variable("t0")],
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
    caller.features_used = vec![VirFeature::Branch, VirFeature::CallStatic];
    let output = generate_program_vcs(&module_with(vec![callee, caller]))
        .expect("branch join generates path-specific continuations");
    let caller = function(&output, CALLER_ID);

    let postconditions = members(caller, ProgramVcMemberKind::Postcondition);
    assert_eq!(postconditions.len(), 2);
    assert_eq!(
        postconditions
            .iter()
            .filter(|member| matches!(member.conclusion, MpkExprTerm::Forall { .. }))
            .count(),
        1
    );
    assert_eq!(
        members(caller, ProgramVcMemberKind::CalleePrecondition).len(),
        1
    );
    assert_eq!(
        members(caller, ProgramVcMemberKind::CalleePanicFree).len(),
        1
    );
}

#[test]
fn repeated_reachable_calls_keep_members_but_deduplicate_dependencies_and_nest_results() {
    let callee = callee();
    let mut caller = direct_caller(callee.contracts.contract_hash.clone());
    caller.blocks[0]
        .instructions
        .push(call("t1", "t0", callee.contracts.contract_hash.clone()));
    caller.blocks[0].terminator = VirTerminator::Return {
        values: vec![variable("t1")],
    };
    let output = generate_program_vcs(&module_with(vec![callee, caller])).expect("calls generate");
    let caller = function(&output, CALLER_ID);

    assert_eq!(
        members(caller, ProgramVcMemberKind::CalleePrecondition).len(),
        2
    );
    assert_eq!(
        members(caller, ProgramVcMemberKind::CalleePanicFree).len(),
        2
    );
    assert_eq!(caller.direct_callees, vec![CALLEE_ID]);
    assert_eq!(caller.contract_dependencies.len(), 1);
    let post = members(caller, ProgramVcMemberKind::Postcondition);
    assert_eq!(forall_depth(&post[0].conclusion), 2);
    assert!(contains_bound(&post[0].conclusion, 0));
    assert!(contains_bound(&post[0].conclusion, 1));
}

#[test]
fn hash_signature_and_recursive_calls_reject_before_wp_artifact_generation() {
    let mut hash_mismatch = direct_module();
    let VirInstruction::CallStatic { contract_hash, .. } =
        &mut hash_mismatch.units[0].functions[1].blocks[0].instructions[0]
    else {
        panic!("caller begins with CallStatic");
    };
    *contract_hash = zero_hash();
    hash_mismatch.vir_hash = vir_hash(&hash_mismatch).expect("VIR hash");
    let error = generate_program_vcs(&hash_mismatch).expect_err("hash mismatch rejects");
    assert!(matches!(
        error,
        ProgramWpError::Validation(source) if source.code() == "VIR_CALLEE_CONTRACT_HASH"
    ));

    let mut signature_mismatch = direct_module();
    let VirInstruction::CallStatic { r#type, .. } =
        &mut signature_mismatch.units[0].functions[1].blocks[0].instructions[0]
    else {
        panic!("caller begins with CallStatic");
    };
    *r#type = bool_ty();
    signature_mismatch.vir_hash = vir_hash(&signature_mismatch).expect("VIR hash");
    let error = generate_program_vcs(&signature_mismatch).expect_err("signature rejects");
    assert!(matches!(
        error,
        ProgramWpError::Validation(source) if source.code() == "VIR_CALL_SIGNATURE"
    ));

    let mut recursive = direct_module();
    let caller_hash = recursive.units[0].functions[1]
        .contracts
        .contract_hash
        .clone();
    let callee = &mut recursive.units[0].functions[0];
    callee.blocks[0].instructions = vec![VirInstruction::CallStatic {
        id: "t0".to_owned(),
        r#type: i8_ty(),
        function: CALLER_ID.to_owned(),
        contract_hash: caller_hash,
        args: vec![variable("arg0")],
        safety_checks: Vec::new(),
    }];
    callee.blocks[0].terminator = VirTerminator::Return {
        values: vec![variable("t0")],
    };
    callee.features_used = vec![VirFeature::CallStatic];
    recursive.vir_hash = vir_hash(&recursive).expect("VIR hash");
    let error = generate_program_vcs(&recursive).expect_err("recursion rejects");
    assert!(matches!(
        error,
        ProgramWpError::Validation(source) if source.code() == "VIR_CALL_CYCLE"
    ));
}

#[test]
fn source_dead_callee_remains_standalone_without_reachable_call_dependency() {
    let mut dead = callee();
    dead.id = "demo::dead".to_owned();
    dead.name = "dead".to_owned();
    dead.contracts = contract(&dead.id, &dead.params);
    let mut root = callee();
    root.id = "demo::root".to_owned();
    root.name = "root".to_owned();
    root.contracts = contract(&root.id, &root.params);
    let output =
        generate_program_vcs(&module_with(vec![dead, root])).expect("dead closure generates");

    assert_eq!(output.functions.len(), 2);
    for function in &output.functions {
        assert!(function.direct_callees.is_empty());
        assert!(function.contract_dependencies.is_empty());
        assert_eq!(
            function.panic_free_dependencies,
            vec![program_declaration_name(
                &function.function_id,
                ProgramDeclarationKind::Contract
            )]
        );
        assert!(members(function, ProgramVcMemberKind::CalleePrecondition).is_empty());
        assert!(members(function, ProgramVcMemberKind::CalleePanicFree).is_empty());
    }
}

#[test]
fn forall_term_serializes_with_the_vc_v1_shape() {
    let term = MpkExprTerm::Forall {
        binder_type: mpk_vc::MpkTypeTerm::Constant {
            name: "Std.Program.Base.Int8".to_owned(),
        },
        body: Box::new(MpkExprTerm::Bound { index: 0 }),
    };
    assert_eq!(
        serde_json::to_value(term).expect("forall serializes"),
        serde_json::json!({
            "kind":"forall",
            "binder_type":{"kind":"constant","name":"Std.Program.Base.Int8"},
            "body":{"kind":"bound","index":0}
        })
    );
}
