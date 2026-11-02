use mpk_vc::go_practical_ordinary::*;
use mpk_vc::go_practical_type_keys::*;
use serde_json::Value;
use std::io::Write;
use std::path::{Path, PathBuf};

fn corpus(name: &str) -> Value {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/go-practical/t02-w01/verification");
    serde_json::from_slice(&std::fs::read(root.join(name).join("corpus.json")).unwrap()).unwrap()
}
fn unhex(value: &Value) -> Vec<u8> {
    value
        .as_str()
        .unwrap()
        .as_bytes()
        .chunks_exact(2)
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect()
}
fn keys(corpus: &Value) -> GoTypeKeyComponents {
    let input = &corpus["input"];
    let sources = input["sources"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| {
            GoTypeSourceBuffer::import(
                &serde_json::to_vec(&s["object"]).unwrap(),
                &unhex(&s["raw_hex"]),
            )
            .unwrap()
        })
        .collect::<Vec<_>>();
    import_go_type_key_components(&unhex(&input["components_hex"]), &sources).unwrap()
}
fn locate(keys: &GoTypeKeyComponents, row: &Value, kind: &str) -> GoSourceSelectionRef {
    let source = row["source"].as_str().unwrap();
    let parsed = keys.source_declarations().source(source).unwrap();
    let node = parsed
        .nodes()
        .iter()
        .position(|n| {
            n.kind() == kind
                && n.start() == row["start"].as_u64().unwrap() as usize
                && n.end() == row["end"].as_u64().unwrap() as usize
        })
        .unwrap();
    GoSourceSelectionRef::new(source, node)
}
fn dispatch(name: &str) -> GoSourceErrorSelection {
    let c = corpus("error-dispatch");
    let keys = keys(&c);
    let row = c["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["input"]["name"] == name)
        .unwrap();
    let input = &row["input"];
    let cases = input["entry_cases"]
        .as_array()
        .map(|cs| {
            cs.iter()
                .map(|n| {
                    locate(
                        &keys,
                        &c["declarations"][format!("example.com/policy.{}", n.as_str().unwrap())],
                        "TypeSpec",
                    )
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let mut roots = vec![];
    let mut entries = vec![];
    for row in input["roots"].as_array().unwrap() {
        let r = locate(&keys, &row["ref"], "FuncDecl");
        let name = row["name"].as_str().unwrap();
        roots.push(r.clone());
        if matches!(
            name,
            "Entry" | "Expr" | "Paren" | "Guarded" | "Branch" | "Imported" | "OtherRoot"
        ) {
            entries.push(GoErrorEntryConstraint::new(
                r.clone(),
                0,
                vec![],
                if name == "OtherRoot" {
                    vec![]
                } else {
                    cases.clone()
                },
            ));
        }
        if name == "Nested" {
            entries.push(GoErrorEntryConstraint::new(
                r.clone(),
                0,
                vec![GoErrorEntryStep::Field(0)],
                cases.clone(),
            ));
            entries.push(GoErrorEntryConstraint::new(
                r,
                0,
                vec![GoErrorEntryStep::Field(1), GoErrorEntryStep::Elements],
                vec![],
            ));
        }
    }
    derive_go_source_error_selection(&keys, &roots, &entries).unwrap()
}
fn write(root: &Path, name: &str, raw: &[u8]) {
    std::fs::create_dir_all(root).unwrap();
    std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(root.join(name))
        .unwrap()
        .write_all(raw)
        .unwrap();
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
fn node(c: &mut mpk_cert::encode::Certificate, n: mpk_cert::encode::TermNode) -> u32 {
    let id = c.term_table.len() as u32;
    c.term_table.push(n);
    id
}
fn constant(c: &mut mpk_cert::encode::Certificate, name: &str) -> u32 {
    let global = c
        .declarations
        .iter()
        .position(|d| c.name_table[d.name as usize] == name)
        .unwrap() as u32;
    node(
        c,
        mpk_cert::encode::TermNode::Const {
            global,
            levels: vec![],
        },
    )
}
fn candidate(name: &str, bytes: &[u8], accepted: bool) {
    let out = std::env::var_os("MPK_GO_SOURCE_ERROR_DISPATCH_OUTPUT").map(PathBuf::from);
    if let Some(root) = &out {
        write(root, &format!("{name}.mpcert"), bytes);
    }
    let r = mpk_kernel::verify_certificate_bytes_json_output(bytes);
    if let Some(root) = out {
        write(
            &root,
            &format!("{name}.rust.json"),
            format!("{}\n", r.json).as_bytes(),
        );
    }
    assert_eq!(r.accepted, accepted, "{name}: {}", r.json);
    let report: Value = serde_json::from_str(&r.json).unwrap();
    if accepted {
        assert_eq!(report["axiom_count"], 0);
    } else {
        assert_eq!(report["error_code"], "KERNEL_CORE_CHECK");
        let detail: Value = serde_json::from_str(report["error_detail"].as_str().unwrap()).unwrap();
        assert_eq!(detail["code"], "CORE_TYPE_MISMATCH");
    }
}
fn member(c: &mpk_cert::encode::Certificate, name: &str) -> usize {
    c.declarations
        .iter()
        .position(|d| c.name_table[d.name as usize] == name)
        .unwrap()
}
fn define(c: &mut mpk_cert::encode::Certificate, name: &str, value: u32) {
    use mpk_cert::encode::{DeclarationKind, DefinitionReducibility};
    let i = member(c, name);
    let DeclarationKind::Def { ty, .. } = c.declarations[i].kind else {
        panic!()
    };
    c.declarations[i].kind = DeclarationKind::Def {
        ty,
        value,
        reducibility: DefinitionReducibility::Reducible,
    };
}
fn target_name(t: &GoErrorImplementation) -> String {
    let m = t.method();
    format!(
        "Target.C{}.S{}.N{}",
        t.value_type_id(),
        m.declaration_source_object_id().unwrap(),
        m.declaration_node().unwrap()
    )
}
fn reject(
    name: &str,
    catalogue: &GoSourceErrorDispatchCatalogue,
    row: &GoSourceErrorDispatchRow,
    c: mpk_cert::encode::Certificate,
) {
    let bytes = encoded(c);
    assert_eq!(
        catalogue.import_row(&bytes, row.call(), row.boxed_source_type_id()),
        Err(GoOrdinaryError::Linkage)
    );
    candidate(name, &bytes, false);
}
fn constant_selector(c: &mut mpk_cert::encode::Certificate, name: &str, result: u32) {
    use mpk_cert::encode::{DeclarationKind, TermNode};
    let i = member(c, name);
    let DeclarationKind::Def { value, .. } = c.declarations[i].kind else {
        panic!()
    };
    let TermNode::Lam { ty, .. } = c.term_table[value as usize].clone() else {
        panic!()
    };
    let value = node(c, TermNode::Lam { ty, body: result });
    define(c, name, value);
}
#[test]
fn original_selector_controls_and_target_ordinal_ownership() {
    use mpk_cert::encode::{DeclarationKind, TermNode};
    let selected = dispatch("entry");
    let catalogue = generate_go_practical_source_error_dispatch_catalogue(&selected).unwrap();
    let site = &catalogue.calls()[0];
    let id = &catalogue.case_tags()[0].0;
    let row = catalogue.generate_row(site, Some(id)).unwrap();
    let original=Path::new(env!("CARGO_MANIFEST_DIR")).join("../../develop/migrations/go-practical/t02-w01/verification/source-error-selector/entries/certificates").join(format!("entry-call0-{id}.mpcert"));
    assert_eq!(row.certificate_bytes(), std::fs::read(&original).unwrap());
    let fresh = || mpk_cert::decode_canonical_certificate(row.certificate_bytes()).unwrap();
    let module = row.module_name();
    let mut c = fresh();
    let result = constant(&mut c, &format!("{module}.NoTarget"));
    let i = member(&c, &format!("{module}.SelectedOfBox"));
    let DeclarationKind::Theorem { ty, proof } = c.declarations[i].kind else {
        panic!()
    };
    let TermNode::Pi { ty: domain, body } = c.term_table[ty as usize].clone() else {
        panic!()
    };
    let TermNode::App {
        function,
        mut arguments,
    } = c.term_table[body as usize].clone()
    else {
        panic!()
    };
    arguments[2] = result;
    let body = node(
        &mut c,
        TermNode::App {
            function,
            arguments,
        },
    );
    let ty = node(&mut c, TermNode::Pi { ty: domain, body });
    c.declarations[i].kind = DeclarationKind::Theorem { ty, proof };
    reject("wrong-box-no-target-claim", &catalogue, &row, c);
    let mut c = fresh();
    let no = constant(&mut c, &format!("{module}.NoTarget"));
    constant_selector(&mut c, &format!("{module}.Select"), no);
    reject("erased-selector", &catalogue, &row, c);
    let mut c = fresh();
    let no = constant(&mut c, &format!("{module}.NoTarget"));
    constant_selector(&mut c, &format!("{module}.SelectFrom0"), no);
    reject("erased-first-branch", &catalogue, &row, c);
    let mut c = fresh();
    let first = constant(
        &mut c,
        &format!("{module}.{}", target_name(&catalogue.targets()[0])),
    );
    constant_selector(&mut c, &format!("{module}.SelectFrom0"), first);
    reject("forced-first-branch", &catalogue, &row, c);
    let nil = catalogue.generate_row(site, None).unwrap();
    let mut c = mpk_cert::decode_canonical_certificate(nil.certificate_bytes()).unwrap();
    let i = member(&c, &format!("{}.SelectedOfNil", nil.module_name()));
    let DeclarationKind::Theorem { ty, proof } = c.declarations[i].kind else {
        panic!()
    };
    let TermNode::App {
        function,
        mut arguments,
    } = c.term_table[ty as usize].clone()
    else {
        panic!()
    };
    arguments[2] = constant(
        &mut c,
        &format!(
            "{}.{}",
            nil.module_name(),
            target_name(&catalogue.targets()[0])
        ),
    );
    let ty = node(
        &mut c,
        TermNode::App {
            function,
            arguments,
        },
    );
    c.declarations[i].kind = DeclarationKind::Theorem { ty, proof };
    reject("nil-selects-concrete-target", &catalogue, &nil, c);
    // Named target equations remain coherent if a named ordinal is changed.
    // Exact compiler-derived table construction must reject that alternative.
    let mut c = fresh();
    let zero = constant(&mut c, "Std.Nat.zero");
    let succ = constant(&mut c, "Std.Nat.succ");
    let one = node(
        &mut c,
        TermNode::App {
            function: succ,
            arguments: vec![zero],
        },
    );
    let replacement = node(
        &mut c,
        TermNode::App {
            function: succ,
            arguments: vec![one],
        },
    );
    define(
        &mut c,
        &format!("{module}.{}", target_name(&catalogue.targets()[0])),
        replacement,
    );
    let bytes = encoded(c);
    assert_eq!(
        catalogue.import_row(&bytes, site, Some(id)),
        Err(GoOrdinaryError::Linkage)
    );
    candidate("wrong-ordinal-is-only-math", &bytes, true);
    if let Some(root) = std::env::var_os("MPK_GO_SOURCE_ERROR_DISPATCH_OUTPUT") {
        let meta = original.with_extension("sources.json");
        write(
            &PathBuf::from(root),
            "wrong-ordinal-is-only-math.sources.json",
            &std::fs::read(meta).unwrap(),
        );
    }
}
#[test]
fn last_original_large_selector_keeps_its_actual_target() {
    use mpk_cert::encode::{DeclarationKind, TermNode};
    let selected = dispatch("seventy-cases");
    let catalogue = generate_go_practical_source_error_dispatch_catalogue(&selected).unwrap();
    let call = &catalogue.calls()[0];
    let id = &catalogue.case_tags()[69].0;
    let row = catalogue.generate_row(call, Some(id)).unwrap();
    let original=Path::new(env!("CARGO_MANIFEST_DIR")).join("../../develop/migrations/go-practical/t02-w01/verification/source-error-selector/first-large/certificates").join(format!("seventy-cases-call0-{id}.mpcert"));
    assert_eq!(row.certificate_bytes(), std::fs::read(original).unwrap());
    let mut c = mpk_cert::decode_canonical_certificate(row.certificate_bytes()).unwrap();
    let result = constant(
        &mut c,
        &format!(
            "{}.{}",
            row.module_name(),
            target_name(&catalogue.targets()[0])
        ),
    );
    let i = member(&c, &format!("{}.SelectedOfBox", row.module_name()));
    let DeclarationKind::Theorem { ty, proof } = c.declarations[i].kind else {
        panic!()
    };
    let TermNode::Pi { ty: domain, body } = c.term_table[ty as usize].clone() else {
        panic!()
    };
    let TermNode::App {
        function,
        mut arguments,
    } = c.term_table[body as usize].clone()
    else {
        panic!()
    };
    arguments[2] = result;
    let body = node(
        &mut c,
        TermNode::App {
            function,
            arguments,
        },
    );
    let ty = node(&mut c, TermNode::Pi { ty: domain, body });
    c.declarations[i].kind = DeclarationKind::Theorem { ty, proof };
    reject("last-type-selects-first-target-claim", &catalogue, &row, c);
}
