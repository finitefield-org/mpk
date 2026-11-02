use mpk_cert::encode::{Certificate, DeclarationKind, LevelNode, TermNode};
use mpk_vc::go_practical_ordinary::*;
use mpk_vc::go_practical_type_keys::*;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
#[path = "support/go_word_division_dependencies.rs"]
mod dependencies;
#[path = "support/go_source_entry_inputs.rs"]
mod inputs;

fn check(name: &str, bytes: &[u8], accepted: bool) {
    let result = mpk_kernel::verify_certificate_bytes_json_output(bytes);
    inputs::write(&format!("{name}.mpcert"), bytes);
    inputs::write(
        &format!("{name}.rust.json"),
        format!("{}\n", result.json).as_bytes(),
    );
    assert_eq!(result.accepted, accepted, "{name}: {}", result.json);
    if !accepted {
        let r: Value = serde_json::from_str(&result.json).unwrap();
        assert_eq!(r["error_code"], "KERNEL_CORE_CHECK");
        let detail: Value = serde_json::from_str(r["error_detail"].as_str().unwrap()).unwrap();
        assert_eq!(detail["code"], "CORE_TYPE_MISMATCH");
    }
}
fn metadata(unit: &GoSourceErrorSelection, row: &GoSourceEntryRow) -> Value {
    let l = row.layout();
    let r = l.function();
    let n = &unit
        .selection()
        .source(r.source_object_id())
        .unwrap()
        .nodes()[r.node()];
    json!({"function":[r.source_object_id(),r.node()],"name":inputs::function_name(unit,r),"start":n.start(),"end":n.end(),"module":row.module_name(),"source_module":row.source_layout_module_name(),"source_binding":row.source_binding_sha256(),"cases":row.case_type_ids(),
        "inputs":l.inputs().iter().map(|s|json!([s.type_id(),s.binding_index()])).collect::<Vec<_>>(),"results":l.results().iter().map(|s|json!([s.type_id(),s.binding_index()])).collect::<Vec<_>>(),
        "bindings":l.bindings().iter().map(|b|json!({"name":b.binding().name(),"source":b.binding().source_object_id(),"declaration":b.binding().declaration_node(),"name_node":b.binding().name_node(),"type":b.type_id(),"origin":match b.origin(){GoSourceEntryBindingOrigin::Input(i)=>json!(["input",i]),GoSourceEntryBindingOrigin::NamedResult(i)=>json!(["named_result",i]),GoSourceEntryBindingOrigin::Local=>json!(["local"])}})).collect::<Vec<_>>()})
}
fn generate(name: &str, unit: &GoSourceErrorSelection) {
    let catalogue = generate_go_practical_source_entry_catalogue(unit.domain()).unwrap();
    let expected = unit
        .selection()
        .declarations()
        .iter()
        .filter(|d| {
            unit.selection()
                .source(d.locator().source_object_id())
                .unwrap()
                .nodes()[d.locator().node()]
            .kind()
                == "FuncDecl"
                && d.roles().any(|role| {
                    matches!(
                        role,
                        GoSourceSelectionRole::SelectedRoot | GoSourceSelectionRole::BodyDependency
                    )
                })
        })
        .map(|d| d.locator())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        catalogue
            .layouts()
            .iter()
            .map(|l| l.function())
            .collect::<std::collections::BTreeSet<_>>(),
        expected
    );
    let mut complete = std::collections::BTreeMap::new();
    let mut rows = vec![];
    for (i, layout) in catalogue.layouts().iter().enumerate() {
        let row = catalogue.generate_row(layout.function()).unwrap();
        assert_eq!(
            catalogue
                .import_row(row.certificate_bytes(), layout.function())
                .unwrap(),
            row
        );
        let decoded = mpk_cert::decode_canonical_certificate(row.certificate_bytes()).unwrap();
        for (n, v) in dependencies::declarations(&decoded) {
            if let Some(old) = complete.insert(n, v.clone()) {
                assert_eq!(old, v);
            }
        }
        inputs::write(
            &format!("rows/{name}-{i:03}.mpcert"),
            row.certificate_bytes(),
        );
        let meta = metadata(unit, &row);
        inputs::write(
            &format!("rows/{name}-{i:03}.entry.json"),
            &serde_json::to_vec_pretty(&meta).unwrap(),
        );
        rows.push(meta);
    }
    let assemblies = catalogue.generate_assemblies().unwrap();
    let mut combined = std::collections::BTreeMap::new();
    let mut functions = vec![];
    let mut parts = vec![];
    for (i, assembly) in assemblies.parts().iter().enumerate() {
        assert_eq!(
            assembly.case_type_ids().len(),
            unit.domain().error_cases().len()
        );
        let decoded = mpk_cert::decode_canonical_certificate(assembly.certificate_bytes()).unwrap();
        for (n, v) in dependencies::declarations(&decoded) {
            if let Some(old) = combined.insert(n, v.clone()) {
                assert_eq!(old, v);
            }
        }
        functions.extend(assembly.function_modules().iter().map(|(f, _)| f.clone()));
        let file = format!("{name}-part-{i:03}");
        parts.push(json!({"file":format!("{file}.mpcert"),"functions":assembly.function_modules().iter().map(|(f,m)|json!([f.source_object_id(),f.node(),m])).collect::<Vec<_>>(),"declarations":decoded.declarations.len(),"terms":decoded.term_table.len(),"bytes":assembly.certificate_bytes().len(),"raw_sha256":format!("{:x}",Sha256::digest(assembly.certificate_bytes()))}));
        check(&file, assembly.certificate_bytes(), true);
    }
    assert_eq!(
        combined, complete,
        "every full original declaration remains across the complete parts"
    );
    assert_eq!(
        functions,
        catalogue
            .layouts()
            .iter()
            .map(|l| l.function().clone())
            .collect::<Vec<_>>(),
        "every original function appears exactly once in order"
    );
    for (i, snapshot) in assemblies.packing_snapshots().iter().enumerate() {
        inputs::write(
            &format!("packing/{name}-{i:03}.incomplete.mpcert"),
            snapshot,
        );
    }
    if assemblies.parts().len() == 1 {
        let single = catalogue.generate_assembly().unwrap();
        assert_eq!(
            catalogue
                .import_assembly(single.certificate_bytes())
                .unwrap(),
            single
        );
        let decoded = mpk_cert::decode_canonical_certificate(single.certificate_bytes()).unwrap();
        assert_eq!(dependencies::declarations(&decoded), complete);
    }
    let max_inputs = catalogue
        .layouts()
        .iter()
        .map(|l| l.inputs().len())
        .max()
        .unwrap();
    let max_results = catalogue
        .layouts()
        .iter()
        .map(|l| l.results().len())
        .max()
        .unwrap();
    if name != "supplement" {
        assert_eq!(max_inputs, 65);
        assert_eq!(max_results, 16);
    }
    let inventory = json!({"unit":name,"rows":rows,"source_types":unit.domain().types().collect::<Vec<_>>(),"underlying":unit.domain().types().filter_map(|t|unit.domain().underlying(t.id()).map(|u|json!([t.id(),u]))).collect::<Vec<_>>(),"cases":unit.domain().error_cases().iter().map(|c|c.value_type_id()).collect::<Vec<_>>(),"max_inputs":max_inputs,"max_results":max_results,"all_selected_bodies":catalogue.layouts().len(),"complete_named_declarations":combined.len(),"parts":parts,"incomplete_packing_states":assemblies.packing_snapshots().len(),"single_program_certificate_claim":false});
    inputs::write(
        &format!("{name}-inventory.json"),
        &serde_json::to_vec_pretty(&inventory).unwrap(),
    );
}

#[test]
fn source_entries_complete_pure_and_wide_assemblies() {
    for name in ["pure", "wide"] {
        let unit = inputs::original(name);
        generate(name, &unit);
    }
}

#[test]
fn original_blank_named_and_shadowed_entries_match_native_execution() {
    let unit = inputs::supplement();
    let layouts = derive_go_source_entry_layouts(unit.domain()).unwrap();
    let find = |name: &str| {
        layouts
            .iter()
            .find(|r| inputs::function_name(&unit, r.function()) == name)
            .unwrap()
    };
    let named = find("Named");
    assert_eq!(named.inputs().len(), 2);
    assert_eq!(named.results().len(), 2);
    assert_eq!(named.inputs()[0].binding_index(), None);
    assert_eq!(named.results()[1].binding_index(), None);
    assert_eq!(
        named
            .bindings()
            .iter()
            .map(|b| b.binding().name())
            .collect::<Vec<_>>(),
        ["input", "out"]
    );
    let hidden = find("Hidden");
    assert_eq!(hidden.inputs().len(), 2);
    assert!(hidden.bindings().is_empty());
    let shadow = find("Shadow");
    let shadowed = shadow
        .bindings()
        .iter()
        .filter(|b| b.binding().name() == "x")
        .collect::<Vec<_>>();
    assert_eq!(shadowed.len(), 2);
    assert!(shadowed[0].exists_at_entry());
    assert!(!shadowed[1].exists_at_entry());
    assert_ne!(
        shadowed[0].binding().declaration_node(),
        shadowed[1].binding().declaration_node()
    );
    assert!(find("ReadZero")
        .bindings()
        .iter()
        .any(|b| b.binding().name() == "local" && !b.exists_at_entry()));
    let word = |t: &str, n| GoSourceRuntimeValue::new(t, GoSourceRuntimeData::Word(n));
    let run = |entry: &GoSourceEntryLayout, args| match execute_go_source(
        unit.domain(),
        entry.function(),
        args,
        1000,
    )
    .unwrap()
    .status
    {
        GoSourceExecutionStatus::Returned(v) => v,
        other => panic!("{other:?}"),
    };
    let named_result = run(
        named,
        vec![
            word(named.inputs()[0].type_id(), 99),
            word(named.inputs()[1].type_id(), 41),
        ],
    );
    assert_eq!(named_result[0].data(), &GoSourceRuntimeData::Word(41));
    assert_eq!(named_result[1].data(), &GoSourceRuntimeData::Bool(false));
    let zero = find("ReadZero");
    let zero_result = run(zero, vec![word(zero.inputs()[0].type_id(), 8)]);
    let GoSourceRuntimeData::Aggregate(array) = zero_result[0].data() else {
        panic!("full array result")
    };
    assert_eq!(array[0].data(), &GoSourceRuntimeData::Word(8));
    assert_eq!(array[1].data(), &GoSourceRuntimeData::Word(0));
    assert_eq!(zero_result[1].data(), &GoSourceRuntimeData::Bytes(vec![]));
    assert_eq!(zero_result[2].data(), &GoSourceRuntimeData::Error(None));
    let receiver = unit
        .domain()
        .underlying(hidden.inputs()[0].type_id())
        .unwrap();
    let GoSourceTypeKey::Struct { fields, .. } =
        unit.domain().types().find(|t| t.id() == receiver).unwrap()
    else {
        panic!("receiver struct")
    };
    let receiver = GoSourceRuntimeValue::new(
        hidden.inputs()[0].type_id(),
        GoSourceRuntimeData::Aggregate(vec![word(&fields[0].type_id, 13)]),
    );
    let hidden_result = run(
        hidden,
        vec![receiver, word(hidden.inputs()[1].type_id(), 45)],
    );
    assert_eq!(hidden_result[0].data(), &GoSourceRuntimeData::Word(7));
    assert_eq!(hidden_result[1].data(), &GoSourceRuntimeData::Bool(true));
    assert_eq!(
        run(shadow, vec![word(shadow.inputs()[0].type_id(), 5)])[0].data(),
        &GoSourceRuntimeData::Word(6)
    );
    let native: Value = serde_json::from_slice(
        &std::fs::read(inputs::evidence().join("source-entries/native/observations.json")).unwrap(),
    )
    .unwrap();
    let observed = json!({"named":[41,false],"zero":[8,0,0,true],"hidden":[7,true],"shadow":[6]});
    assert_eq!(
        native["values"], observed,
        "actual untouched original Go main"
    );
    inputs::write("supplement-observations.json",&serde_json::to_vec_pretty(&json!({"status":"passed","actual_native_values":native,"observed":observed,"original_functions":4,"unused_original_bodies_retained":true})).unwrap());
    // Unselected original main/channel/callback/map bodies stay in the package.
    let source = named.function().source_object_id();
    let parsed = unit.selection().source(source).unwrap();
    let inactive = parsed
        .nodes()
        .iter()
        .enumerate()
        .find(|(i, n)| {
            n.kind() == "FuncDecl"
                && inputs::function_name(&unit, &GoSourceSelectionRef::new(source, *i))
                    == "UnusedCallback"
        })
        .unwrap()
        .0;
    assert_eq!(
        derive_go_source_entry_layout(unit.domain(), &GoSourceSelectionRef::new(source, inactive)),
        Err(GoScalarTypingError::Locator)
    );
    generate("supplement", &unit);
}

fn push(c: &mut Certificate, t: TermNode) -> u32 {
    let i = c.term_table.len() as u32;
    c.term_table.push(t);
    i
}
fn constant(c: &mut Certificate, n: &str) -> u32 {
    let g = c
        .declarations
        .iter()
        .position(|d| c.name_table[d.name as usize] == n)
        .unwrap() as u32;
    push(
        c,
        TermNode::Const {
            global: g,
            levels: vec![],
        },
    )
}
fn encoded(mut c: Certificate) -> Vec<u8> {
    let old = c.name_table.clone();
    c.name_table.sort();
    for d in &mut c.declarations {
        d.name = c.name_table.binary_search(&old[d.name as usize]).unwrap() as u32;
    }
    for l in &mut c.level_table {
        if let LevelNode::Param(n) = l {
            *n = c.name_table.binary_search(&old[*n as usize]).unwrap() as u32;
        }
    }
    c.export_block = mpk_cert::build_export_block(&c).unwrap();
    c.axiom_report = mpk_cert::build_axiom_report(&c).unwrap();
    c.hashes.export_hash = mpk_cert::export_block_hash(&c.export_block);
    c.hashes.axiom_report_hash = mpk_cert::axiom_report_hash_for_report(&c.axiom_report);
    mpk_cert::encode_certificate_bounded(&c, 2097152).unwrap()
}
fn replace_rhs(c: &mut Certificate, t: u32, right: u32) -> u32 {
    match c.term_table[t as usize].clone() {
        TermNode::Pi { ty, body } => {
            let body = replace_rhs(c, body, right);
            push(c, TermNode::Pi { ty, body })
        }
        TermNode::App {
            function,
            mut arguments,
        } => {
            assert_eq!(arguments.len(), 3);
            arguments[2] = right;
            push(
                c,
                TermNode::App {
                    function,
                    arguments,
                },
            )
        }
        _ => panic!("actual equality type"),
    }
}
fn rhs(c: &Certificate, mut ty: u32) -> u32 {
    loop {
        match &c.term_table[ty as usize] {
            TermNode::Pi { body, .. } => ty = *body,
            TermNode::App { arguments, .. } => {
                assert_eq!(arguments.len(), 3);
                return arguments[2];
            }
            _ => panic!("original equality"),
        }
    }
}
#[test]
fn source_entry_controls_reject_premature_locals_and_crossed_arguments() {
    let unit = inputs::original("pure");
    let catalogue = generate_go_practical_source_entry_catalogue(unit.domain()).unwrap();
    let find = |name| {
        catalogue
            .layouts()
            .iter()
            .find(|l| inputs::function_name(&unit, l.function()) == name)
            .unwrap()
    };
    let local = find("Loop");
    let row = catalogue.generate_row(local.function()).unwrap();
    let index = local
        .bindings()
        .iter()
        .position(|b| !b.exists_at_entry())
        .unwrap();
    let mut c = mpk_cert::decode_canonical_certificate(row.certificate_bytes()).unwrap();
    let global = c
        .declarations
        .iter()
        .position(|d| {
            c.name_table[d.name as usize] == format!("{}.EntryPresence{index}", row.module_name())
        })
        .unwrap();
    let DeclarationKind::Theorem { ty, proof } = c.declarations[global].kind else {
        panic!("actual presence theorem")
    };
    let yes = constant(&mut c, "Std.Bool.true");
    let ty = replace_rhs(&mut c, ty, yes);
    c.declarations[global].kind = DeclarationKind::Theorem { ty, proof };
    let bad = encoded(c);
    check("premature-local", &bad, false);
    assert_eq!(
        catalogue.import_row(&bad, local.function()),
        Err(GoOrdinaryError::Linkage)
    );
    let wide = find("Wide");
    let wide_row = catalogue.generate_row(wide.function()).unwrap();
    assert_eq!(
        catalogue.import_row(row.certificate_bytes(), wide.function()),
        Err(GoOrdinaryError::Linkage)
    );
    let mut c = mpk_cert::decode_canonical_certificate(wide_row.certificate_bytes()).unwrap();
    let global = c
        .declarations
        .iter()
        .position(|d| {
            c.name_table[d.name as usize]
                == format!("{}.InitializedBinding64", wide_row.module_name())
        })
        .unwrap();
    let DeclarationKind::Theorem { ty, proof } = c.declarations[global].kind else {
        panic!("actual input theorem")
    };
    // Retain the same malformed control under this recipe's declaration order.
    // The original exact state is also retained in the prior failed epoch.
    let mut forward = c.clone();
    let f = constant(
        &mut forward,
        &format!("{}.Argument0", wide_row.module_name()),
    );
    let argument = push(&mut forward, TermNode::Var(0));
    let wrong = push(
        &mut forward,
        TermNode::App {
            function: f,
            arguments: vec![argument],
        },
    );
    let changed = replace_rhs(&mut forward, ty, wrong);
    forward.declarations[global].kind = DeclarationKind::Theorem { ty: changed, proof };
    forward.export_block = mpk_cert::build_export_block(&forward).unwrap();
    let error = mpk_cert::build_axiom_report(&forward).unwrap_err();
    assert!(format!("{error:?}").contains("references non-previous declaration"));
    inputs::write(
        "failed-candidates/forward-reference.candidate.mpcert",
        &mpk_cert::encode_certificate(&forward),
    );
    inputs::write("failed-candidates/forward-reference.error.json",&serde_json::to_vec_pretty(&json!({"status":"reproduced-construction-failure-with-current-declaration-order","error":format!("{error:?}"),"ordinary_kernel_candidate":false,"complete_pre_error_state_retained":true})).unwrap());
    let earlier = c
        .declarations
        .iter()
        .find(|d| {
            c.name_table[d.name as usize]
                == format!("{}.InitializedBinding0", wide_row.module_name())
        })
        .unwrap();
    let DeclarationKind::Theorem { ty: earlier_ty, .. } = earlier.kind else {
        panic!("earlier input law")
    };
    let wrong = rhs(&c, earlier_ty);
    let ty = replace_rhs(&mut c, ty, wrong);
    c.declarations[global].kind = DeclarationKind::Theorem { ty, proof };
    check("crossed-input", &encoded(c), false);
    let mut c = mpk_cert::decode_canonical_certificate(row.certificate_bytes()).unwrap();
    let b = constant(&mut c, "Std.Bool");
    let yes = constant(&mut c, "Std.Bool.true");
    let eq = constant(&mut c, "Std.Eq");
    let r = constant(&mut c, "Std.Eq.refl");
    let ty = push(
        &mut c,
        TermNode::App {
            function: eq,
            arguments: vec![b, yes, yes],
        },
    );
    let proof = push(
        &mut c,
        TermNode::App {
            function: r,
            arguments: vec![b, yes],
        },
    );
    let name = c.name_table.len() as u32;
    c.name_table.push("EntryTest.UnrelatedTrue".into());
    c.declarations.push(mpk_cert::encode::Declaration {
        name,
        kind: DeclarationKind::Theorem { ty, proof },
    });
    let extra = encoded(c);
    check("extra-mathematics", &extra, true);
    assert_eq!(
        catalogue.import_row(&extra, local.function()),
        Err(GoOrdinaryError::Linkage)
    );
}
