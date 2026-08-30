#[path = "support/successor_projection.rs"]
mod successor_projection;

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;

use mpk_cert::{hash_hex, hash_with_domain, HashDomain as CertificateHashDomain};
use mpk_vc::vir::{
    VirContractBinaryExpr, VirPanicPolicy, VirResultRef, VirTermination, VirVariableRef,
};
use mpk_vc::{
    canonical_json_bytes, canonical_vc_json, contract_hash, emit_validated_vc_skeleton_v1,
    encode_vir_type, generate_program_vcs, generate_vc_v1_from_context, import_vc_v1_json,
    parse_strict_json, program_declaration_name, validate_policy_member_binding, vir_hash,
    BitVectorWidth, GroupedTheoremDeclaration, LowercaseSha256, MpkExprTerm, OverflowMode,
    PanicMode, PointerWidth, ProgramDeclarationKind, ProgramVcFunction, ProgramVcMemberKind,
    ProgramVcModule, RustCheckedParameters, SemanticParameters, SemanticProfile, SourceLanguage,
    StrictJsonLimits, ValidatedVcCertificateSkeleton, ValidatedVcDocument, VcBinder, VcDocument,
    VcGroup, VcGroupKind, VcMember, VcSourceContext, VcSourceFunction, VcTerm, VcTypeTerm,
    VcValidationPhase, VirBinaryOperator, VirBinding, VirBlock, VirContract, VirContractExpr,
    VirFeature, VirFunction, VirInstruction, VirModule, VirSafetyCheck, VirTerminator, VirType,
    VirUnit, VirValue, VERIFICATION_LIMIT_PROFILE, VIR_SCHEMA_VERSION,
};
use successor_projection::import_successor_rust_vir_projection;

const UNIT_ID: &str = "vector";
const LEAF_ID: &str = "vector::a_leaf";
const LEFT_ID: &str = "vector::b_left";
const RIGHT_ID: &str = "vector::c_right";
const DIAMOND_ID: &str = "vector::z_diamond";
const REPEATED_ID: &str = "vector::z_repeated";
const FRONTEND_VIR: &[u8] =
    include_bytes!("../../../rust-tools/rust2vir/testdata/static-calls/expected-vir.json");

struct Fixture {
    program: ProgramVcModule,
    context: VcSourceContext,
    vc: ValidatedVcDocument,
    skeleton: ValidatedVcCertificateSkeleton,
}

#[test]
fn diamond_calls_generate_path_bound_members_and_minimal_topological_dependencies() {
    let fixture = build_fixture(diamond_module());
    assert_eq!(
        fixture
            .program
            .functions
            .iter()
            .map(|function| function.function_id.as_str())
            .collect::<Vec<_>>(),
        vec![LEAF_ID, LEFT_ID, RIGHT_ID, DIAMOND_ID]
    );

    let leaf = program_function(&fixture.program, LEAF_ID);
    assert!(leaf.direct_callees.is_empty());
    assert_eq!(members(leaf, ProgramVcMemberKind::Postcondition).len(), 1);
    assert!(members(leaf, ProgramVcMemberKind::CalleePrecondition).is_empty());
    assert!(members(leaf, ProgramVcMemberKind::CalleePanicFree).is_empty());

    for helper in [LEFT_ID, RIGHT_ID] {
        let function = program_function(&fixture.program, helper);
        assert_eq!(function.direct_callees, vec![LEAF_ID]);
        assert_eq!(
            members(function, ProgramVcMemberKind::CalleePrecondition).len(),
            1
        );
        assert_eq!(
            members(function, ProgramVcMemberKind::CalleePanicFree).len(),
            1
        );
    }

    let selected = program_function(&fixture.program, DIAMOND_ID);
    assert_eq!(selected.direct_callees, vec![LEFT_ID, RIGHT_ID]);
    let preconditions = members(selected, ProgramVcMemberKind::CalleePrecondition);
    let panic_free = members(selected, ProgramVcMemberKind::CalleePanicFree);
    assert_eq!(preconditions.len(), 2);
    assert_eq!(panic_free.len(), 2);
    let disabled = MpkExprTerm::Apply {
        function: "Std.Bool.not".to_owned(),
        args: vec![MpkExprTerm::Var {
            name: "arg1".to_owned(),
        }],
    };
    let enabled = MpkExprTerm::Var {
        name: "arg1".to_owned(),
    };
    for call_members in [preconditions, panic_free] {
        assert_eq!(call_members[0].assumptions, vec![disabled.clone()]);
        assert_eq!(call_members[1].assumptions, vec![enabled.clone()]);
    }

    assert_minimal_program_dependencies(&fixture.program);
    assert_canonical_declaration_order_and_edges(&fixture);

    let leaf_panic = declaration(&fixture.skeleton, LEAF_ID, VcGroupKind::PanicFree);
    assert!(leaf_panic.member_ids.is_empty());
    assert!(matches!(
        &leaf_panic.theorem_type.body,
        VcTerm::Apply { function, args }
            if function == "Std.Logic.Imp"
                && matches!(args.as_slice(), [_, VcTerm::Constant { name }] if name == "Std.Bool.true")
    ));

    let repeat = build_fixture(diamond_module());
    assert_eq!(fixture.vc.canonical_bytes(), repeat.vc.canonical_bytes());
    assert_eq!(
        fixture.skeleton.canonical_bytes(),
        repeat.skeleton.canonical_bytes()
    );
}

#[test]
fn repeated_calls_keep_distinct_members_but_deduplicate_declaration_edges() {
    let fixture = build_fixture(repeated_module());
    assert_eq!(
        fixture
            .program
            .functions
            .iter()
            .map(|function| function.function_id.as_str())
            .collect::<Vec<_>>(),
        vec![LEAF_ID, RIGHT_ID, REPEATED_ID]
    );
    let selected = program_function(&fixture.program, REPEATED_ID);
    assert_eq!(selected.direct_callees, vec![RIGHT_ID]);
    assert_eq!(
        members(selected, ProgramVcMemberKind::CalleePrecondition).len(),
        2
    );
    assert_eq!(
        members(selected, ProgramVcMemberKind::CalleePanicFree).len(),
        2
    );
    assert_eq!(selected.contract_dependencies.len(), 1);
    assert_eq!(selected.panic_free_dependencies.len(), 3);
    let postconditions = members(selected, ProgramVcMemberKind::Postcondition);
    assert_eq!(postconditions.len(), 1);
    assert_eq!(forall_depth(&postconditions[0].conclusion), 2);
    assert!(contains_bound(&postconditions[0].conclusion, 0));
    assert!(contains_bound(&postconditions[0].conclusion, 1));
    let safety = members(selected, ProgramVcMemberKind::OperationSafety);
    assert_eq!(safety.len(), 1);
    assert_eq!(forall_depth(&safety[0].conclusion), 2);
    assert!(contains_bound(&safety[0].conclusion, 0));
    assert!(contains_bound(&safety[0].conclusion, 1));

    let selected_contract = declaration(&fixture.skeleton, REPEATED_ID, VcGroupKind::Contract);
    assert_eq!(
        selected_contract
            .member_ids
            .iter()
            .filter(|member| member.contains("#callee_precondition#"))
            .count(),
        2
    );
    assert_eq!(
        selected_contract
            .dependencies
            .iter()
            .filter(|dependency| {
                **dependency == program_declaration_name(RIGHT_ID, ProgramDeclarationKind::Contract)
            })
            .count(),
        1
    );
    assert_minimal_program_dependencies(&fixture.program);
}

#[test]
fn missing_reversed_duplicate_and_extra_edges_reject_with_stable_codes() {
    let fixture = build_fixture(diamond_module());

    let mut missing = fixture.vc.document().clone();
    group_mut(&mut missing, DIAMOND_ID, VcGroupKind::Contract)
        .dependencies
        .remove(0);
    assert_dependency_error(missing, &fixture.context, "VC_DEPENDENCY_SET");

    let selected_contract = program_declaration_name(DIAMOND_ID, ProgramDeclarationKind::Contract);
    let mut reversed = fixture.vc.document().clone();
    group_mut(&mut reversed, LEAF_ID, VcGroupKind::Contract)
        .dependencies
        .push(selected_contract);
    assert_dependency_error(reversed, &fixture.context, "VC_DEPENDENCY_CYCLE");

    let mut duplicate = fixture.vc.document().clone();
    let duplicate_group = group_mut(&mut duplicate, DIAMOND_ID, VcGroupKind::Contract);
    duplicate_group
        .dependencies
        .push(duplicate_group.dependencies[0].clone());
    duplicate_group
        .dependencies
        .sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
    assert_dependency_error(duplicate, &fixture.context, "VC_DEPENDENCY_ORDER");

    let mut extra = fixture.vc.document().clone();
    let extra_group = group_mut(&mut extra, DIAMOND_ID, VcGroupKind::Contract);
    extra_group.dependencies.push(program_declaration_name(
        LEAF_ID,
        ProgramDeclarationKind::PanicFree,
    ));
    extra_group
        .dependencies
        .sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
    assert_dependency_error(extra, &fixture.context, "VC_DEPENDENCY_SET");
}

#[test]
fn policy_members_bind_to_one_containing_declaration_name_hash_and_full_closure() {
    let fixture = build_fixture(diamond_module());
    let selected = fixture
        .vc
        .document()
        .functions
        .iter()
        .find(|function| function.function_id == DIAMOND_ID)
        .expect("selected VC function");
    let hashes = fixture
        .skeleton
        .skeleton()
        .theorem_declarations
        .iter()
        .map(|declaration| (declaration.name.clone(), declaration_hash(declaration)))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(
        hashes.len(),
        fixture.skeleton.skeleton().theorem_declarations.len()
    );

    for member in &selected.members {
        let containing = fixture
            .skeleton
            .skeleton()
            .theorem_declarations
            .iter()
            .filter(|declaration| declaration.member_ids.contains(&member.id))
            .collect::<Vec<_>>();
        assert_eq!(containing.len(), 1, "one grouped declaration per conjunct");
        let declaration = containing[0];
        validate_policy_member_binding(
            &fixture.skeleton,
            &member.id,
            &member.group_id,
            &declaration.name,
        )
        .expect("exact member containment tuple");
        assert!(policy_binding_matches(
            &fixture.skeleton,
            &member.id,
            &member.group_id,
            &declaration.name,
            &hashes[&declaration.name]
        ));
        assert!(!policy_binding_matches(
            &fixture.skeleton,
            &member.id,
            &member.group_id,
            &declaration.name,
            &"0".repeat(64)
        ));
    }

    let selected_contract = declaration(&fixture.skeleton, DIAMOND_ID, VcGroupKind::Contract);
    assert!(selected_contract.member_ids.len() > 1);
    assert!(selected_contract
        .member_ids
        .iter()
        .all(|member| member != &selected_contract.name));

    let roots = BTreeSet::from([
        program_declaration_name(DIAMOND_ID, ProgramDeclarationKind::Contract),
        program_declaration_name(DIAMOND_ID, ProgramDeclarationKind::PanicFree),
    ]);
    let required = declaration_closure(&fixture.skeleton, roots.clone());
    let all = fixture
        .skeleton
        .skeleton()
        .theorem_declarations
        .iter()
        .map(|declaration| declaration.name.clone())
        .collect::<BTreeSet<_>>();
    assert_eq!(required, all);
    assert!(roots.is_subset(&required));

    let mut missing_selected_group = all.clone();
    missing_selected_group.remove(&program_declaration_name(
        DIAMOND_ID,
        ProgramDeclarationKind::PanicFree,
    ));
    assert!(!required.is_subset(&missing_selected_group));
    let mut missing_transitive = all;
    missing_transitive.remove(&program_declaration_name(
        LEAF_ID,
        ProgramDeclarationKind::PanicFree,
    ));
    assert!(!required.is_subset(&missing_transitive));
}

#[test]
fn public_and_frontend_call_graph_fixtures_are_identical() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    assert_eq!(
        fs::read(root.join("fixtures/rust-basic/calls/wp_graph.rs")).unwrap(),
        fs::read(root.join("rust-tools/rust2vir/testdata/static-calls/wp_graph.rs")).unwrap()
    );
    assert_eq!(
        fs::read(root.join("fixtures/rust-basic/calls/expected-vir.json")).unwrap(),
        fs::read(root.join("rust-tools/rust2vir/testdata/static-calls/expected-vir.json")).unwrap()
    );
}

fn build_fixture(module: VirModule) -> Fixture {
    let program = generate_program_vcs(&module).expect("Rust CallStatic program VCs generate");
    let context = source_context(&module, &program);
    let vc = generate_vc_v1_from_context(&context).expect("Rust call VC v1 generates");
    let skeleton =
        emit_validated_vc_skeleton_v1(&vc).expect("Rust call grouped skeleton generates");
    Fixture {
        program,
        context,
        vc,
        skeleton,
    }
}

fn source_context(module: &VirModule, program: &ProgramVcModule) -> VcSourceContext {
    let mut functions = Vec::with_capacity(program.functions.len());
    for generated in &program.functions {
        let (unit, function) = module
            .units
            .iter()
            .find_map(|unit| {
                unit.functions
                    .iter()
                    .find(|function| function.id == generated.function_id)
                    .map(|function| (unit, function))
            })
            .expect("generated function belongs to VIR");
        let parameters = function
            .params
            .iter()
            .map(|parameter| VcBinder {
                id: parameter.id.clone(),
                r#type: VcTypeTerm::from(
                    &encode_vir_type(
                        module.semantic_profile,
                        &module.semantic_parameters,
                        &unit.type_decls,
                        &parameter.r#type,
                    )
                    .expect("parameter type encodes"),
                ),
            })
            .collect();
        let requires = generated
            .requires
            .iter()
            .map(VcTerm::try_from)
            .collect::<Result<Vec<_>, _>>()
            .expect("requirements close over parameters");
        let regenerated_members = generated
            .members
            .iter()
            .map(VcMember::try_from)
            .collect::<Result<Vec<_>, _>>()
            .expect("members close over parameters");
        functions.push(VcSourceFunction {
            function_id: function.id.clone(),
            contract_hash: contract_hash(&function.contracts)
                .expect("contract hash")
                .as_str()
                .to_owned(),
            direct_callees: generated.direct_callees.clone(),
            parameters,
            requires,
            regenerated_members,
        });
    }
    VcSourceContext {
        id: "rust.calls.integration".to_owned(),
        source_ir_schema: VIR_SCHEMA_VERSION.to_owned(),
        source_ir_hash: module.vir_hash.as_str().to_owned(),
        input_set_hash: "0".repeat(64),
        semantic_profile: module.semantic_profile,
        semantic_parameters: module.semantic_parameters.clone(),
        verification_limit_profile: VERIFICATION_LIMIT_PROFILE.to_owned(),
        functions,
    }
}

fn assert_minimal_program_dependencies(program: &ProgramVcModule) {
    for function in &program.functions {
        let mut contract = function
            .direct_callees
            .iter()
            .map(|callee| program_declaration_name(callee, ProgramDeclarationKind::Contract))
            .collect::<Vec<_>>();
        contract.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
        let mut panic_free = vec![program_declaration_name(
            &function.function_id,
            ProgramDeclarationKind::Contract,
        )];
        for callee in &function.direct_callees {
            panic_free.push(program_declaration_name(
                callee,
                ProgramDeclarationKind::Contract,
            ));
            panic_free.push(program_declaration_name(
                callee,
                ProgramDeclarationKind::PanicFree,
            ));
        }
        panic_free.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
        panic_free.dedup();
        assert_eq!(function.contract_dependencies, contract);
        assert_eq!(function.panic_free_dependencies, panic_free);
    }
}

fn assert_canonical_declaration_order_and_edges(fixture: &Fixture) {
    let declarations = &fixture.skeleton.skeleton().theorem_declarations;
    assert_eq!(declarations.len(), fixture.program.functions.len() * 2);
    let mut earlier = BTreeSet::new();
    for (function, pair) in fixture
        .program
        .functions
        .iter()
        .zip(declarations.chunks_exact(2))
    {
        assert_eq!(pair[0].function_id, function.function_id);
        assert_eq!(pair[0].group_kind, VcGroupKind::Contract);
        assert_eq!(pair[1].function_id, function.function_id);
        assert_eq!(pair[1].group_kind, VcGroupKind::PanicFree);
        assert_eq!(pair[0].dependencies, function.contract_dependencies);
        assert_eq!(pair[1].dependencies, function.panic_free_dependencies);
        for declaration in pair {
            assert!(declaration
                .dependencies
                .iter()
                .all(|dependency| earlier.contains(dependency)));
            assert!(declaration
                .dependencies
                .windows(2)
                .all(|window| window[0].as_bytes() < window[1].as_bytes()));
            assert!(earlier.insert(declaration.name.clone()));
        }
    }
}

fn assert_dependency_error(document: VcDocument, context: &VcSourceContext, code: &str) {
    let bytes = canonical_vc_json(&document).expect("mutated VC serializes canonically");
    let error = import_vc_v1_json(&bytes, context).expect_err("dependency mutation rejects");
    assert_eq!(error.phase(), VcValidationPhase::Dependencies);
    assert_eq!(error.code(), code);
}

fn declaration<'a>(
    skeleton: &'a ValidatedVcCertificateSkeleton,
    function_id: &str,
    kind: VcGroupKind,
) -> &'a GroupedTheoremDeclaration {
    skeleton
        .skeleton()
        .theorem_declarations
        .iter()
        .find(|declaration| {
            declaration.function_id == function_id && declaration.group_kind == kind
        })
        .unwrap_or_else(|| panic!("missing {function_id} {} declaration", kind.as_str()))
}

fn group_mut<'a>(
    document: &'a mut VcDocument,
    function_id: &str,
    kind: VcGroupKind,
) -> &'a mut VcGroup {
    document
        .functions
        .iter_mut()
        .find(|function| function.function_id == function_id)
        .and_then(|function| function.groups.iter_mut().find(|group| group.kind == kind))
        .unwrap_or_else(|| panic!("missing {function_id} {} group", kind.as_str()))
}

fn declaration_hash(declaration: &GroupedTheoremDeclaration) -> String {
    let bytes = serde_json::to_vec(declaration).expect("declaration serializes");
    let strict = parse_strict_json(
        &bytes,
        StrictJsonLimits::new(268_435_456, 268_435_456, 768, 1_048_576),
    )
    .expect("declaration parses strictly");
    let canonical = canonical_json_bytes(&strict).expect("declaration canonicalizes");
    hash_hex(&hash_with_domain(
        CertificateHashDomain::Declaration,
        &canonical,
    ))
}

fn policy_binding_matches(
    skeleton: &ValidatedVcCertificateSkeleton,
    member_id: &str,
    group_id: &str,
    declaration_name: &str,
    declaration_hash_value: &str,
) -> bool {
    if validate_policy_member_binding(skeleton, member_id, group_id, declaration_name).is_err() {
        return false;
    }
    skeleton
        .skeleton()
        .theorem_declarations
        .iter()
        .find(|declaration| declaration.name == declaration_name)
        .is_some_and(|declaration| declaration_hash(declaration) == declaration_hash_value)
}

fn declaration_closure(
    skeleton: &ValidatedVcCertificateSkeleton,
    roots: BTreeSet<String>,
) -> BTreeSet<String> {
    let declarations = skeleton
        .skeleton()
        .theorem_declarations
        .iter()
        .map(|declaration| (declaration.name.as_str(), declaration))
        .collect::<BTreeMap<_, _>>();
    let mut pending = roots.into_iter().collect::<Vec<_>>();
    let mut visited = BTreeSet::new();
    while let Some(name) = pending.pop() {
        if !visited.insert(name.clone()) {
            continue;
        }
        let declaration = declarations
            .get(name.as_str())
            .unwrap_or_else(|| panic!("missing checked declaration {name}"));
        for dependency in &declaration.dependencies {
            if !visited.contains(dependency) {
                pending.push(dependency.clone());
            }
        }
    }
    visited
}

fn program_function<'a>(program: &'a ProgramVcModule, id: &str) -> &'a ProgramVcFunction {
    program
        .functions
        .iter()
        .find(|function| function.function_id == id)
        .unwrap_or_else(|| panic!("missing program VC function {id}"))
}

fn members(
    function: &ProgramVcFunction,
    kind: ProgramVcMemberKind,
) -> Vec<&mpk_vc::ProgramVcMember> {
    function
        .members
        .iter()
        .filter(|member| member.kind == kind)
        .collect()
}

fn forall_depth(term: &MpkExprTerm) -> usize {
    match term {
        MpkExprTerm::Forall { body, .. } => 1 + forall_depth(body),
        MpkExprTerm::Apply { args, .. } => args.iter().map(forall_depth).max().unwrap_or(0),
        MpkExprTerm::Convert { value, .. } => forall_depth(value),
        _ => 0,
    }
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

fn diamond_module() -> VirModule {
    import_successor_rust_vir_projection(FRONTEND_VIR)
}

fn repeated_module() -> VirModule {
    let leaf = leaf_function();
    let right = caller_function(RIGHT_ID, "c_right", LEAF_ID, &leaf.contracts.contract_hash);
    let selected = repeated_function(&right.contracts.contract_hash);
    module_with(vec![leaf, right, selected])
}

fn leaf_function() -> VirFunction {
    function_with_blocks(
        LEAF_ID,
        "a_leaf",
        vec![u8_binding("arg0")],
        vec![VirBlock {
            label: "bb0".to_owned(),
            parameters: Vec::new(),
            instructions: Vec::new(),
            terminator: VirTerminator::Return {
                values: vec![variable("arg0")],
            },
        }],
        Vec::new(),
    )
}

fn caller_function(
    id: &str,
    name: &str,
    callee: &str,
    callee_hash: &LowercaseSha256,
) -> VirFunction {
    function_with_blocks(
        id,
        name,
        vec![u8_binding("arg0")],
        vec![VirBlock {
            label: "bb0".to_owned(),
            parameters: Vec::new(),
            instructions: vec![call("t0", callee, "arg0", callee_hash)],
            terminator: VirTerminator::Return {
                values: vec![variable("t0")],
            },
        }],
        vec![VirFeature::CallStatic],
    )
}

fn repeated_function(right_hash: &LowercaseSha256) -> VirFunction {
    function_with_blocks(
        REPEATED_ID,
        "z_repeated",
        vec![u8_binding("arg0")],
        vec![VirBlock {
            label: "bb0".to_owned(),
            parameters: Vec::new(),
            instructions: vec![
                call("t0", RIGHT_ID, "arg0", right_hash),
                call("t1", RIGHT_ID, "t0", right_hash),
                VirInstruction::BinOp {
                    id: "t2".to_owned(),
                    op: VirBinaryOperator::BvUdiv,
                    r#type: u8_type(),
                    lhs: variable("arg0"),
                    rhs: variable("t1"),
                    safety_checks: vec![VirSafetyCheck::DivisorNonzero {}],
                },
            ],
            terminator: VirTerminator::Return {
                values: vec![variable("t2")],
            },
        }],
        vec![VirFeature::CallStatic],
    )
}

fn function_with_blocks(
    id: &str,
    name: &str,
    params: Vec<VirBinding>,
    blocks: Vec<VirBlock>,
    features_used: Vec<VirFeature>,
) -> VirFunction {
    VirFunction {
        id: id.to_owned(),
        unit_id: UNIT_ID.to_owned(),
        name: name.to_owned(),
        params: params.clone(),
        results: vec![u8_binding("result0")],
        locals: Vec::new(),
        blocks,
        contracts: identity_contract(id, &params),
        features_used,
    }
}

fn identity_contract(function_id: &str, params: &[VirBinding]) -> VirContract {
    let value = VirContractExpr::Variable(VirVariableRef {
        var: "arg0".to_owned(),
    });
    let mut contract = VirContract {
        unit_id: UNIT_ID.to_owned(),
        function_id: function_id.to_owned(),
        semantic_profile: SemanticProfile::RustCheckedV0,
        semantic_parameters: parameters(),
        requires: vec![equal(value.clone(), value.clone())],
        ensures: vec![equal(
            VirContractExpr::Result(VirResultRef { result: 0 }),
            value,
        )],
        modifies: Vec::new(),
        panic: VirPanicPolicy::Forbidden,
        termination: VirTermination::Total,
        loops: Vec::new(),
        contract_hash: zero_hash(),
    };
    assert_eq!(contract.requires.len(), 1);
    assert!(!params.is_empty());
    contract.contract_hash = contract_hash(&contract).expect("contract hash");
    contract
}

fn equal(lhs: VirContractExpr, rhs: VirContractExpr) -> VirContractExpr {
    VirContractExpr::Binary(VirContractBinaryExpr {
        op: VirBinaryOperator::Eq,
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
    })
}

fn call(
    id: &str,
    function: &str,
    argument: &str,
    contract_hash: &LowercaseSha256,
) -> VirInstruction {
    VirInstruction::CallStatic {
        id: id.to_owned(),
        r#type: u8_type(),
        function: function.to_owned(),
        contract_hash: contract_hash.clone(),
        args: vec![variable(argument)],
        safety_checks: Vec::new(),
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
            name: UNIT_ID.to_owned(),
            type_decls: Vec::new(),
            const_decls: Vec::new(),
            functions,
        }],
        vir_hash: zero_hash(),
    };
    module.vir_hash = vir_hash(&module).expect("VIR hash");
    module
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

fn variable(id: &str) -> VirValue {
    VirValue::Variable(VirVariableRef { var: id.to_owned() })
}

fn u8_binding(id: &str) -> VirBinding {
    VirBinding {
        id: id.to_owned(),
        r#type: u8_type(),
    }
}

fn u8_type() -> VirType {
    VirType::Bv {
        width: BitVectorWidth::Bits8,
        signed: false,
    }
}
