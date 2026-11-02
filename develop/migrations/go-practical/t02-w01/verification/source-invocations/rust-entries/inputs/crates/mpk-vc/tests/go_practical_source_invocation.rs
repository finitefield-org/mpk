use mpk_vc::go_practical_ordinary::*;
use mpk_vc::go_practical_type_keys::*;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
#[path = "support/go_word_division_dependencies.rs"]
mod dependencies;
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/go-practical/t02-w01/verification/source-invocations")
}
fn unhex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
fn selection(name: &str) -> GoSourceErrorSelection {
    let c: Value =
        serde_json::from_slice(&std::fs::read(root().join("corpus.json")).unwrap()).unwrap();
    let c = c["units"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["name"] == name)
        .unwrap();
    let input = &c["input"];
    let sources = input["sources"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| {
            GoTypeSourceBuffer::import(
                &serde_json::to_vec(&s["object"]).unwrap(),
                &unhex(s["raw_hex"].as_str().unwrap()),
            )
            .unwrap()
        })
        .collect::<Vec<_>>();
    let keys =
        import_go_type_key_components(&unhex(input["components_hex"].as_str().unwrap()), &sources)
            .unwrap();
    let scopes = build_go_source_scopes(&keys).unwrap();
    let locate = |r: &Value, kind: &str| {
        let source = r["source"].as_str().unwrap();
        let node = scopes
            .source(source)
            .unwrap()
            .nodes()
            .iter()
            .position(|n| {
                n.kind() == kind
                    && n.start() == r["start"].as_u64().unwrap() as usize
                    && n.end() == r["end"].as_u64().unwrap() as usize
            })
            .unwrap();
        GoSourceSelectionRef::new(source, node)
    };
    let roots = c["roots"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| locate(r, "FuncDecl"))
        .collect::<Vec<_>>();
    let cases = c["cases"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| locate(r, "TypeSpec"))
        .collect::<Vec<_>>();
    let entries = entry_conditions(&scopes, &roots, &cases);
    derive_go_source_error_selection(&keys, &roots, &entries).unwrap_or_else(|e| {
        if let GoSourceErrorSelectionError::Selection(e) = &e {
            let parsed = scopes.source(e.source_object_id()).unwrap();
            let n = &parsed.nodes()[e.node()];
            eprintln!(
                "actual source {} {}..{} {}",
                n.kind(),
                n.start(),
                n.end(),
                String::from_utf8_lossy(&parsed.source().bytes()[n.start()..n.end()])
            );
        }
        panic!("{e:?}")
    })
}
fn entry_conditions(
    scopes: &GoSourceScopes,
    roots: &[GoSourceSelectionRef],
    cases: &[GoSourceSelectionRef],
) -> Vec<GoErrorEntryConstraint> {
    let mut entries = vec![];
    for root in roots {
        let signature =
            derive_go_source_signature(scopes, root.source_object_id(), root.node()).unwrap();
        let graph = signature.types();
        for (input, slot) in signature
            .slots()
            .iter()
            .filter(|s| s.role() != GoSourceSignatureRole::Result)
            .enumerate()
        {
            let mut work = vec![(slot.type_id().to_string(), vec![])];
            while let Some((id, path)) = work.pop() {
                let key = graph.types().find(|k| k.id() == id).unwrap();
                match key {
                    GoSourceTypeKey::Nominal { .. } => {
                        work.push((graph.underlying(&id).unwrap().to_string(), path))
                    }
                    GoSourceTypeKey::Builtin { name, .. } if name == "error" => entries.push(
                        GoErrorEntryConstraint::new(root.clone(), input, path, cases.to_vec()),
                    ),
                    GoSourceTypeKey::Array { element, .. }
                    | GoSourceTypeKey::Slice { element, .. } => {
                        let mut p = path;
                        p.push(GoErrorEntryStep::Elements);
                        work.push((element.clone(), p));
                    }
                    GoSourceTypeKey::Struct { fields, .. } => {
                        for (i, f) in fields.iter().enumerate() {
                            let mut p = path.clone();
                            p.push(GoErrorEntryStep::Field(i));
                            work.push((f.type_id.clone(), p));
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    entries
}
fn write(name: &str, raw: &[u8]) {
    if let Some(out) = std::env::var_os("MPK_GO_SOURCE_INVOCATION_OUTPUT") {
        use std::io::Write;
        let out = PathBuf::from(out);
        std::fs::create_dir_all(&out).unwrap();
        std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(out.join(name))
            .unwrap()
            .write_all(raw)
            .unwrap();
    }
}
fn span(s: &GoSourceErrorSelection, r: &GoSourceSelectionRef) -> Value {
    let node = &s.selection().source(r.source_object_id()).unwrap().nodes()[r.node()];
    json!([r.source_object_id(), node.start(), node.end()])
}
fn site(s: &GoSourceErrorSelection, c: &GoSourceCallSite) -> Value {
    let parsed = s.selection().source(c.caller().source_object_id()).unwrap();
    json!({"caller":span(s,c.caller()),"caller_node":c.caller().node(),"source":c.caller().source_object_id(),"node":c.node(),"start":c.start_byte(),"end":c.end_byte(),"inputs":c.input_type_ids(),"results":c.result_type_ids(),"dispatch":c.is_error_dispatch(),"arguments":c.arguments().iter().map(|a|{let node=&parsed.nodes()[a.expression_node()];json!({"node":a.expression_node(),"start":node.start(),"end":node.end(),"receiver":a.is_receiver(),"values":a.values().iter().map(|v|json!({"projection":v.projection(),"parameter":v.parameter_position(),"source_type":v.source_type_id(),"parameter_type":v.parameter_type_id(),"nil":v.is_untyped_nil()})).collect::<Vec<_>>()})}).collect::<Vec<_>>(),"targets":c.targets().iter().map(|t|json!({"dynamic_type":t.dynamic_type_id(),"callee":span(s,t.callee()),"receiver_type":t.receiver_type_id(),"receiver_path":t.receiver_path()})).collect::<Vec<_>>()})
}
fn metadata(name: &str, s: &GoSourceErrorSelection, row: &GoSourceInvocationRow) -> Value {
    let cert = mpk_cert::decode_canonical_certificate(row.certificate_bytes()).unwrap();
    json!({"unit":name,"module":row.module_name(),"source_module":row.source_layout_module_name(),"binding":row.source_binding_sha256(),"site":site(s,row.site()),"cases":row.case_type_ids(),"types":s.domain().types().collect::<Vec<_>>(),"underlying":s.domain().types().filter_map(|k|s.domain().underlying(k.id()).map(|u|json!([k.id(),u]))).collect::<Vec<_>>(),"declarations":cert.declarations.len(),"terms":cert.term_table.len(),"bytes":row.certificate_bytes().len(),"all_original_source_types_and_U_retained":true,"source_execution_callee_or_context_proof":false})
}
fn check(name: &str) {
    let s = selection(name);
    let c = generate_go_practical_source_invocation_catalogue(&s).unwrap();
    let expected = if name == "wide" { 70 } else { 0 };
    assert_eq!(s.domain().error_cases().len(), expected);
    assert!(c.sites().iter().any(|c| c.input_type_ids().len() == 65));
    assert!(c.sites().iter().any(|c| c.result_type_ids().len() == 16));
    assert!(c.sites().iter().any(|c| c.result_type_ids().is_empty()));
    assert!(c
        .sites()
        .iter()
        .any(|c| c.arguments().len() == 1 && c.arguments()[0].values().len() == 16));
    assert!(c
        .sites()
        .iter()
        .any(|c| c.arguments().iter().any(|g| g.is_receiver())));
    assert!(c.sites().iter().any(|c| c
        .arguments()
        .iter()
        .any(|g| g.values().iter().any(|v| v.is_untyped_nil()))));
    for c in c.sites().iter().filter(|c| c.is_error_dispatch()) {
        assert_eq!(c.targets().len(), expected);
        assert_eq!(
            c.targets()
                .iter()
                .map(|t| t.dynamic_type_id().unwrap())
                .collect::<Vec<_>>(),
            s.domain()
                .error_cases()
                .iter()
                .map(|e| e.value_type_id())
                .collect::<Vec<_>>()
        );
    }
    for (i, site) in c.sites().iter().enumerate() {
        let row = c.generate_row(site.caller(), site.node()).unwrap();
        let prior = mpk_cert::decode_canonical_certificate(row.prior_certificate_bytes()).unwrap();
        let cert = mpk_cert::decode_canonical_certificate(row.certificate_bytes()).unwrap();
        let before = dependencies::declarations(&prior);
        let after = dependencies::declarations(&cert);
        for (n, d) in before {
            assert_eq!(after.get(&n), Some(&d), "complete original identity {n}");
        }
        assert_eq!(
            c.import_row(row.certificate_bytes(), site.caller(), site.node())
                .unwrap(),
            row
        );
        let file = format!("{name}-{i:03}");
        write(&format!("{file}.mpcert"), row.certificate_bytes());
        write(
            &format!("{file}.sources.json"),
            &serde_json::to_vec_pretty(&metadata(name, &s, &row)).unwrap(),
        );
        let result = mpk_kernel::verify_certificate_bytes_json_output(row.certificate_bytes());
        write(
            &format!("{file}.rust.json"),
            format!("{}\n", result.json).as_bytes(),
        );
        assert!(result.accepted, "{file}: {}", result.json);
        let report: Value = serde_json::from_str(&result.json).unwrap();
        assert_eq!(report["axiom_count"], 0);
    }
    write(&format!("{name}-inventory.json"),&serde_json::to_vec_pretty(&json!({"sites":c.sites().iter().map(|c|site(&s,c)).collect::<Vec<_>>(),"cases":s.domain().error_cases().iter().map(|e|e.value_type_id()).collect::<Vec<_>>()})).unwrap());
}
#[test]
fn source_invocations_pure() {
    check("pure")
}
#[test]
fn source_invocations_wide() {
    check("wide")
}

fn encoded(mut c: mpk_cert::encode::Certificate) -> Vec<u8> {
    use mpk_cert::encode::LevelNode;
    let old = c.name_table.clone();
    c.name_table.sort();
    for d in &mut c.declarations {
        d.name = c.name_table.binary_search(&old[d.name as usize]).unwrap() as u32;
    }
    for level in &mut c.level_table {
        if let LevelNode::Param(name) = level {
            *name = c.name_table.binary_search(&old[*name as usize]).unwrap() as u32;
        }
    }
    c.export_block = mpk_cert::build_export_block(&c).unwrap();
    c.axiom_report = mpk_cert::build_axiom_report(&c).unwrap();
    c.hashes.export_hash = mpk_cert::export_block_hash(&c.export_block);
    c.hashes.axiom_report_hash = mpk_cert::axiom_report_hash_for_report(&c.axiom_report);
    assert!(c.imports.is_empty() && c.declarations.len() <= 4096 && c.term_table.len() <= 131072);
    assert_eq!(c.axiom_report.summary.total_axiom_count, 0);
    mpk_cert::encode_certificate_bounded(&c, 2097152).unwrap()
}
fn push(c: &mut mpk_cert::encode::Certificate, t: mpk_cert::encode::TermNode) -> u32 {
    let n = c.term_table.len() as u32;
    c.term_table.push(t);
    n
}
fn constant(c: &mut mpk_cert::encode::Certificate, name: &str) -> u32 {
    let id = c
        .declarations
        .iter()
        .position(|d| c.name_table[d.name as usize] == name)
        .unwrap();
    push(
        c,
        mpk_cert::encode::TermNode::Const {
            global: id as u32,
            levels: vec![],
        },
    )
}

#[test]
fn source_invocations_controls() {
    use mpk_cert::encode::{Declaration, DeclarationKind, TermNode};
    let s = selection("pure");
    let c = generate_go_practical_source_invocation_catalogue(&s).unwrap();
    let a = c
        .sites()
        .iter()
        .find(|a| {
            c.sites().iter().any(|b| {
                a.node() != b.node()
                    && a.targets() == b.targets()
                    && a.result_type_ids() == b.result_type_ids()
                    && a.input_type_ids() == b.input_type_ids()
            })
        })
        .unwrap();
    let b = c
        .sites()
        .iter()
        .find(|b| {
            a.node() != b.node()
                && a.targets() == b.targets()
                && a.result_type_ids() == b.result_type_ids()
                && a.input_type_ids() == b.input_type_ids()
        })
        .unwrap();
    let row = c.generate_row(a.caller(), a.node()).unwrap();
    assert_eq!(
        c.import_row(row.certificate_bytes(), b.caller(), b.node()),
        Err(GoOrdinaryError::Linkage)
    );
    let original = mpk_cert::decode_canonical_certificate(row.certificate_bytes()).unwrap();
    for name in ["stale-post", "extra-theorem"] {
        let mut candidate = original.clone();
        let boolean = constant(&mut candidate, "Std.Bool");
        let no = constant(&mut candidate, "Std.Bool.false");
        let eq = constant(&mut candidate, "Std.Eq");
        let truth = push(
            &mut candidate,
            TermNode::App {
                function: eq,
                arguments: vec![boolean, no, no],
            },
        );
        if name == "extra-theorem" {
            let refl = constant(&mut candidate, "Std.Eq.refl");
            let proof = push(
                &mut candidate,
                TermNode::App {
                    function: refl,
                    arguments: vec![boolean, no],
                },
            );
            let name = candidate.name_table.len() as u32;
            candidate
                .name_table
                .push(format!("{}.CallerExtra", row.module_name()));
            candidate.declarations.push(Declaration {
                name,
                kind: DeclarationKind::Theorem { ty: truth, proof },
            });
        } else {
            let before = constant(&mut candidate, &format!("{}.Before", row.module_name()));
            let id = candidate
                .declarations
                .iter()
                .position(|d| {
                    candidate.name_table[d.name as usize] == format!("{}.Post", row.module_name())
                })
                .unwrap();
            let DeclarationKind::Def {
                ty, reducibility, ..
            } = candidate.declarations[id].kind
            else {
                panic!()
            };
            candidate.declarations[id].kind = DeclarationKind::Def {
                ty,
                value: before,
                reducibility,
            };
        }
        let bytes = encoded(candidate);
        assert_eq!(
            c.import_row(&bytes, a.caller(), a.node()),
            Err(GoOrdinaryError::Linkage)
        );
        write(&format!("control-{name}.mpcert"), &bytes);
        let result = mpk_kernel::verify_certificate_bytes_json_output(&bytes);
        write(
            &format!("control-{name}.rust.json"),
            format!("{}\n", result.json).as_bytes(),
        );
        assert_eq!(result.accepted, name == "extra-theorem", "{}", result.json);
        if !result.accepted {
            let report: Value = serde_json::from_str(&result.json).unwrap();
            assert_eq!(report["error_code"], "KERNEL_CORE_CHECK");
            let detail: Value =
                serde_json::from_str(report["error_detail"].as_str().unwrap()).unwrap();
            assert_eq!(detail["code"], "CORE_TYPE_MISMATCH");
        }
    }
}
