//! W08: original transition matrices, pending counterexamples and strict imports.
use super::*;
pub(super) fn nodes(p: &TransitionVcProgram) -> usize {
    p.sequents()
        .iter()
        .flat_map(|s| s.assumptions.iter().chain(&s.goals))
        .map(ContractTerm::nodes)
        .sum::<usize>()
        + p.paths().iter().map(|p| p.guard.nodes()).sum::<usize>()
}
fn key(t: &ContractTerm) -> String {
    serde_json::to_string(t).unwrap()
}
fn head(t: &ContractTerm) -> (&str, Vec<&ContractTerm>) {
    let mut f = t;
    let mut args = vec![];
    while let ContractTerm::App {
        function, argument, ..
    } = f
    {
        args.push(argument.as_ref());
        f = function;
    }
    args.reverse();
    let ContractTerm::Const { name, .. } = f else {
        panic!("no head")
    };
    (name, args)
}
fn atoms(t: &ContractTerm, out: &mut BTreeSet<String>) {
    let (n, a) = head(t);
    match n {
        "Mpk.CSharp.Bool.true" | "Mpk.CSharp.Bool.false" => (),
        "Mpk.CSharp.Bool.And" | "Mpk.CSharp.Bool.Or" | "Mpk.CSharp.Bool.Not" => {
            for t in a {
                atoms(t, out)
            }
        }
        _ => {
            out.insert(key(t));
        }
    }
}
fn check(p: &TransitionVcProgram) {
    let mut defs = BTreeMap::new();
    fn visit(t: &ContractTerm, defs: &mut BTreeMap<String, String>) {
        match t {
            ContractTerm::Const { name, type_id } => {
                if let Some(old) = defs.insert(name.clone(), type_id.clone()) {
                    assert_eq!(old, *type_id, "{name}");
                }
            }
            ContractTerm::App {
                function, argument, ..
            } => {
                visit(function, defs);
                visit(argument, defs)
            }
            ContractTerm::Lam { body, .. } => visit(body, defs),
            ContractTerm::Let { value, body, .. } => {
                visit(value, defs);
                visit(body, defs)
            }
            _ => (),
        }
    }
    for s in p.sequents() {
        for t in s.assumptions.iter().chain(&s.goals) {
            super::construction::assert_typed(
                t,
                &s.subjects
                    .iter()
                    .map(|s| s.type_id.clone())
                    .collect::<Vec<_>>(),
            );
            visit(t, &mut defs);
        }
    }
    assert_eq!(
        defs.keys().collect::<Vec<_>>(),
        p.definition_names().iter().collect::<Vec<_>>()
    );
    let mut aa = BTreeSet::new();
    for path in p.paths() {
        atoms(&path.guard, &mut aa)
    }
    let aa = aa.into_iter().collect::<Vec<_>>();
    assert!(aa.len() <= 12);
    // Cache truth tables for atoms once; serializing large snapshot terms on
    // every valuation obscures this small independent Boolean check.
    enum Circuit {
        Constant(bool),
        Atom(usize),
        Not(Box<Circuit>),
        And(Box<Circuit>, Box<Circuit>),
        Or(Box<Circuit>, Box<Circuit>),
    }
    fn compile(t: &ContractTerm, aa: &[String]) -> Circuit {
        let (n, a) = head(t);
        match n {
            "Mpk.CSharp.Bool.true" => Circuit::Constant(true),
            "Mpk.CSharp.Bool.false" => Circuit::Constant(false),
            "Mpk.CSharp.Bool.Not" => Circuit::Not(Box::new(compile(a[0], aa))),
            "Mpk.CSharp.Bool.And" => {
                Circuit::And(Box::new(compile(a[0], aa)), Box::new(compile(a[1], aa)))
            }
            "Mpk.CSharp.Bool.Or" => {
                Circuit::Or(Box::new(compile(a[0], aa)), Box::new(compile(a[1], aa)))
            }
            _ => Circuit::Atom(aa.iter().position(|a| a == &key(t)).unwrap()),
        }
    }
    fn run(c: &Circuit, m: usize) -> bool {
        match c {
            Circuit::Constant(b) => *b,
            Circuit::Atom(i) => m & (1 << i) != 0,
            Circuit::Not(a) => !run(a, m),
            Circuit::And(a, b) => run(a, m) && run(b, m),
            Circuit::Or(a, b) => run(a, m) || run(b, m),
        }
    }
    let circuits = p
        .paths()
        .iter()
        .map(|p| compile(&p.guard, &aa))
        .collect::<Vec<_>>();
    for mask in 0..(1 << aa.len()) {
        let selected = circuits
            .iter()
            .enumerate()
            .filter(|(_, c)| run(c, mask))
            .map(|(i, _)| &p.paths()[i])
            .collect::<Vec<_>>();
        assert_eq!(selected.len(), 1);
        if let Some(found) = aa.iter().position(|a| a.contains(".RetainedKeyPresent.")) {
            if mask & (1 << found) != 0 {
                let same = aa
                    .iter()
                    .enumerate()
                    .filter(|(_, a)| a.contains(".SourceEqual."))
                    .all(|(i, _)| mask & (1 << i) != 0);
                assert_eq!(
                    selected[0].path,
                    if same {
                        "replay"
                    } else {
                        "error.idempotency_conflict"
                    }
                );
                continue;
            }
        }
        let value = |needle: &str| {
            aa.iter()
                .position(|a| a.contains(needle))
                .map(|i| mask & (1 << i) != 0)
        };
        let version_equal = aa
            .iter()
            .position(|a| {
                a.contains("Mpk.CSharp.Binding.Equal.mpk.csharp.value.u64.v1")
                    && !a.contains("Mpk.CSharp.U64.Max")
            })
            .unwrap();
        let expected_fixed = if mask & (1 << version_equal) == 0 {
            Some("error.version_conflict")
        } else if value(".HistoryCapacity4096.") == Some(true) {
            Some("error.history_capacity")
        } else if value("Mpk.CSharp.U64.Max") == Some(true) {
            Some("error.version_exhausted")
        } else {
            None
        };
        if let Some(expected) = expected_fixed {
            assert_eq!(selected[0].path, expected);
        } else {
            assert!(![
                "error.version_conflict",
                "error.version_exhausted",
                "replay"
            ]
            .contains(&selected[0].path.as_str()));
        }
    }
}
#[test]
fn csharp_03_t06_w08_original_source_programs_and_mutations() {
    let b = b();
    let mut goldens = vec![];
    for family in ["transition", "idempotency", "transition-vc"] {
        let requests = read(&format!("{family}/requests.json"));
        let responses = read(&format!("{family}/responses.json"));
        for req in requests
            .as_array()
            .unwrap()
            .iter()
            .filter(|r| r["expected_attachment"] == true)
        {
            let row = responses
                .as_array()
                .unwrap()
                .iter()
                .find(|r| r["id"] == req["id"])
                .unwrap();
            let (ctx, captures) = support::replay_context(&b, req);
            let source = ValidatedDataSource::import_captured_facts(
                &b,
                &ctx,
                &captures,
                &serde_json::to_vec(&row["facts"]).unwrap(),
            )
            .unwrap();
            let e = emit_data_phase(&b, &ctx, &captures, &source).unwrap();
            let src = PracticalVcSource {
                artifact_context: &ctx,
                captured_inputs: &captures,
                vir: e.vir(),
            };
            let p = if req["case"] == "valid" {
                let vc = generate_csharp_practical_vc(src)
                    .unwrap_or_else(|e| panic!("{family}/{} {e:?}", req["case"]));
                let p = vc.transition_vcs();
                let global = vc
                    .obligation_groups()
                    .iter()
                    .find(|g| g.id() == "vc.group.0800.global")
                    .unwrap();
                assert!(global
                    .subject_ids()
                    .contains(&format!("transition_program:{}", p.hash())));
                for sequent in p.sequents() {
                    assert!(global.subject_ids().contains(&sequent.id));
                }
                for g in vc.obligation_groups() {
                    if g.id().starts_with("vc.group.0800.function.") {
                        assert!(g.dependencies().contains(&global.id().to_owned()));
                    } else if matches!(
                        g.proof_owner(),
                        LaterProofOwner::ConstructionAndTypeInvariants
                            | LaterProofOwner::DataAndCollections
                            | LaterProofOwner::LoopSwitchAndPatterns
                            | LaterProofOwner::ExceptionalControl
                            | LaterProofOwner::BindingsAndSpecialization
                            | LaterProofOwner::BoundaryRoundTrip
                    ) {
                        assert!(global.dependencies().contains(&g.id().to_owned()));
                    }
                }
                assert!(vc.resource_reservation().ordinary_term_nodes_minimum() >= nodes(p) as u64);
                p.clone()
            } else {
                generate_csharp_practical_transition_vcs(src)
                    .unwrap_or_else(|e| panic!("{family}/{} {e:?}", req["case"]))
            };
            assert_eq!(p.contracts().len(), 1);
            if family != "idempotency" {
                source_counterexamples(
                    req["case"].as_str().unwrap(),
                    &source,
                    e.vir(),
                    &p,
                    row["runs"].as_array().unwrap(),
                );
            } else {
                snapshot_counterexamples(
                    req["case"].as_str().unwrap(),
                    &p,
                    row["runs"].as_array().unwrap(),
                );
            }
            check(&p);
            assert_eq!(
                import_csharp_practical_transition_vcs(&p.canonical_bytes(), src).unwrap(),
                p
            );
            assert_eq!(
                p.paths().first().unwrap().path,
                if family == "idempotency" {
                    "replay"
                } else {
                    "error.version_conflict"
                }
            );
            if req["case"] == "valid" {
                let m: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
                for field in [
                    "contracts",
                    "paths",
                    "sequents",
                    "source_functions",
                    "definition_names",
                ] {
                    let mut changed = m.clone();
                    changed[field].as_array_mut().unwrap().pop();
                    assert!(
                        import_csharp_practical_transition_vcs(
                            &super::construction::edited_bytes(&p.canonical_bytes(), &changed),
                            src
                        )
                        .is_err(),
                        "{field}"
                    );
                }
            }
            let m: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
            if family == "idempotency" {
                let plan = e.transitions()[0].idempotency().unwrap();
                for s in plan.snapshot_obligations() {
                    for n in s.nodes() {
                        assert_eq!(
                            p.snapshots()
                                .iter()
                                .find(|x| x.type_id == n.recipe().type_id)
                                .unwrap()
                                .member_ids,
                            n.member_ids()
                        );
                    }
                }
                let mut changed = m.clone();
                let index = p
                    .snapshots()
                    .iter()
                    .position(|n| !n.member_ids.is_empty())
                    .unwrap();
                changed["snapshots"][index]["member_ids"]
                    .as_array_mut()
                    .unwrap()
                    .pop();
                assert!(import_csharp_practical_transition_vcs(
                    &super::construction::edited_bytes(&p.canonical_bytes(), &changed),
                    src
                )
                .is_err());
                assert!(p
                    .sequents()
                    .iter()
                    .any(|s| s.kind == "snapshot_helper_equivalence"));
            }
            goldens.push(json!({"family":family,"case":req["case"],"id":req["id"],"transition_sha256":p.hash(),"sequents":p.sequents().len(),"paths":p.paths().len(),"snapshot_nodes":p.snapshots().len()}));
        }
    }
    assert_eq!(goldens.len(), 11);
    if let Ok(path) = std::env::var("MPK_T06_W08_GOLDEN_OUT") {
        fs::write(path, serde_json::to_vec_pretty(&goldens).unwrap()).unwrap();
    } else {
        assert_eq!(json!(goldens), read("transition-vc/goldens.json"));
    }
}

// Test-only interpretation of the ordinary expression fragment used by the
// retained CLR fixtures. It is never used by production or proof acceptance.
fn eval(
    t: &ContractTerm,
    vars: &[Value],
    defs: &BTreeMap<String, ContractDefinition>,
    result: &Value,
    apply: &str,
) -> Value {
    if let ContractTerm::Var { index, .. } = t {
        return vars[*index].clone();
    }
    let (n, args) = head(t);
    let a = args
        .iter()
        .map(|t| eval(t, vars, defs, result, apply))
        .collect::<Vec<_>>();
    if n == apply {
        return result.clone();
    }
    if n.starts_with("binding.project.") || n.contains(".SuccessPayload.") {
        return a[0].clone();
    }
    for (prefix, field) in [
        (".Transition.State.", "state"),
        (".Transition.Events.", "events"),
        (".Transition.Response.", "response"),
    ] {
        if n.contains(prefix) {
            return a[0][field].clone();
        }
    }
    if let Some(id) = n.strip_prefix("Mpk.CSharp.Transition.Member.") {
        return a[0][id].clone();
    }
    if n.starts_with("Mpk.CSharp.Binding.Equal.") {
        return json!(a[0] == a[1]);
    }
    if n == "Mpk.CSharp.U64.CheckedAddOne" {
        return json!(a[0].as_u64().unwrap() + 1);
    }
    let d = defs
        .get(n)
        .unwrap_or_else(|| panic!("uninterpreted fixture term {n}"));
    let params: Value = serde_json::from_str(&d.parameters).unwrap();
    match d.tag.as_str() {
        "literal" => {
            if params["value"].is_boolean() {
                params["value"].clone()
            } else {
                json!(params["value"].as_str().unwrap().parse::<i64>().unwrap())
            }
        }
        "field" => a[0][params["member_id"].as_str().unwrap()].clone(),
        "sequence_length" => json!(a[0].as_array().unwrap().len()),
        "sequence_index" => a[0][a[1].as_u64().unwrap() as usize].clone(),
        "binary" => match params["operation_id"].as_str().unwrap() {
            "boolean.and" => json!(a[0].as_bool().unwrap() && a[1].as_bool().unwrap()),
            "integer.i32.equal.checked" | "integer.i64.equal.checked" => json!(a[0] == a[1]),
            "integer.i32.greater_equal.checked" => {
                json!(a[0].as_i64().unwrap() >= a[1].as_i64().unwrap())
            }
            "integer.i32.less.checked" => json!(a[0].as_i64().unwrap() < a[1].as_i64().unwrap()),
            x => panic!("{x}"),
        },
        x => panic!("{x}"),
    }
}
fn source_counterexamples(
    case: &str,
    source: &ValidatedDataSource,
    vir: &mpk_vc::csharp_practical_vir_validation::ValidatedPracticalVir,
    p: &TransitionVcProgram,
    rows: &[Value],
) {
    let doc: Value = serde_json::from_str(&p.contracts()[0]).unwrap();
    let apply = doc["selected_callable_id"].as_str().unwrap();
    let obj = |name: &str, fields: &[(&str, i64)]| {
        let t = source
            .source_types()
            .as_object()
            .unwrap()
            .values()
            .find(|t| t["identity"]["name"] == name)
            .unwrap();
        let mut o = serde_json::Map::new();
        for (name, value) in fields {
            let m = t["members"]
                .as_array()
                .unwrap()
                .iter()
                .find(|m| m["name"] == *name)
                .unwrap();
            o.insert(m["id"].as_str().unwrap().into(), json!(value));
        }
        Value::Object(o)
    };
    let defs = vir
        .contract_expressions()
        .iter()
        .flat_map(|e| e.definitions().iter().map(|d| (d.name.clone(), d.clone())))
        .collect();
    let goals = &p
        .sequents()
        .iter()
        .find(|s| s.kind == "successful_result" && s.id.ends_with(".new_success"))
        .unwrap()
        .goals;
    let mut failures = BTreeSet::new();
    for trace in rows.chunks_exact(11) {
        let values = trace
            .iter()
            .map(|r| r["value"].as_i64().unwrap())
            .collect::<Vec<_>>();
        if values[0] != 0 {
            continue;
        }
        let a = trace[0]["a"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_i64().unwrap())
            .collect::<Vec<_>>();
        let vars = vec![
            obj("State", &[("Version", a[0]), ("Balance", a[3])]),
            obj("Command", &[("Expected", a[1]), ("Amount", a[2])]),
            obj("Context", &[("Effective", a[4])]),
        ];
        let output = json!({"state":obj("State",&[("Version",values[2]),("Balance",values[3])]),"response":obj("Response",&[("Balance",values[7])]),"events":[obj("Event",&[("Amount",values[5]),("Effective",values[6])]),obj("Event",&[("Amount",values[10]),("Effective",values[6])])]});
        for (index, kind) in [
            (6, "invariant"),
            (7, "version"),
            (9, "event"),
            (11, "response"),
        ] {
            if eval(&goals[index], &vars, &defs, &output, apply) != true {
                failures.insert(kind.to_owned());
            }
        }
    }
    let expected = match case {
        "mutant_invariant" => vec!["invariant"],
        "mutant_version" => vec!["version"],
        "mutant_event" => vec!["event"],
        "mutant_response" => vec!["response"],
        _ => vec![],
    };
    assert_eq!(
        failures.iter().map(String::as_str).collect::<Vec<_>>(),
        expected,
        "{case}"
    );
}
fn snapshot_counterexamples(case: &str, p: &TransitionVcProgram, rows: &[Value]) {
    let doc: Value = serde_json::from_str(&p.contracts()[0]).unwrap();
    let helper = doc["idempotency"]["equality_callable_id"].as_str().unwrap();
    let command = doc["command_type_id"].as_str().unwrap();
    let goal = &p
        .sequents()
        .iter()
        .find(|s| s.kind == "snapshot_helper_equivalence")
        .unwrap()
        .goals[0];
    fn compare(t: &ContractTerm, helper: &str, ct: &str, h: bool, c: bool, x: bool) -> bool {
        let (name, args) = head(t);
        if name == helper {
            return h;
        }
        if name.contains(".SourceEqual.") {
            return if name.ends_with(ct) { c } else { x };
        }
        let a = compare(args[0], helper, ct, h, c, x);
        let b = compare(args[1], helper, ct, h, c, x);
        if name == "Mpk.CSharp.Bool.And" {
            a && b
        } else {
            assert!(name.starts_with("Mpk.CSharp.Binding.Equal."));
            a == b
        }
    }
    let mut failures = 0;
    for trace in rows.chunks_exact(13) {
        let a = trace[0]["a"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_i64().unwrap())
            .collect::<Vec<_>>();
        if !(a[5] >= a[9] && a[5] < a[9] + a[8]) {
            continue;
        }
        let source_helper = trace[0]["value"] == 0;
        let same_command = a[1] == a[10] && a[2] == a[11] && a[6] == a[13] && a[7] == a[14];
        if !compare(
            goal,
            helper,
            command,
            source_helper,
            same_command,
            a[4] == a[12],
        ) {
            failures += 1;
        }
    }
    assert_eq!(failures > 0, case == "omitted_context_body");
}
