use mpk_vc::go_practical_ordinary::*;
use mpk_vc::go_practical_type_keys::*;
use serde_json::{json, Value};
use std::io::Write;
use std::path::{Path, PathBuf};

fn corpus(name: &str) -> Value {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/go-practical/t02-w01/verification");
    serde_json::from_slice(&std::fs::read(root.join(name).join("corpus.json")).unwrap()).unwrap()
}
fn error_corpus(name: &str) -> Value {
    if name == "source-string-indices" {
        corpus("source-string-indices")
    } else if name == "heterogeneous-payloads" {
        let path=Path::new(env!("CARGO_MANIFEST_DIR")).join("../../develop/migrations/go-practical/t02-w01/verification/source-error-representation/corpus-heterogeneous.json");
        serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
    } else {
        corpus("error-dispatch")
    }
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
    let c = error_corpus(name);
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
#[path = "support/go_word_division_dependencies.rs"]
mod dependencies;

fn check(name: &str) {
    let selection = dispatch(name);
    let c = generate_go_practical_source_string_index_catalogue(selection.domain()).unwrap();
    assert_eq!(
        c.strings().source_types(),
        selection.domain().types().cloned().collect::<Vec<_>>()
    );
    if name == "source-string-indices" {
        assert_eq!(c.index_types().len(), 14);
        assert_eq!(c.strings().source_string_type_ids().len(), 4);
    }
    let out = std::env::var_os("MPK_GO_SOURCE_STRING_INDEX_OUTPUT").map(PathBuf::from);
    for string in c.strings().source_string_type_ids() {
        let prior = c.strings().generate_row(string).unwrap();
        let before = dependencies::declarations(
            &mpk_cert::decode_canonical_certificate(prior.certificate_bytes()).unwrap(),
        );
        for (index, width, signed) in c.index_types() {
            let row = c.generate_row(string, index).unwrap();
            assert_eq!(row.index_width(), *width);
            assert_eq!(row.is_signed_index(), *signed);
            let decoded = mpk_cert::decode_canonical_certificate(row.certificate_bytes()).unwrap();
            let after = dependencies::declarations(&decoded);
            for (n, v) in &before {
                assert_eq!(
                    after.get(n),
                    Some(v),
                    "all prior source string identities: {n}"
                );
            }
            assert_eq!(
                c.import_row(row.certificate_bytes(), string, index)
                    .unwrap(),
                row
            );
            assert_eq!(
                c.import_row(row.certificate_bytes(), string, "wrong-index"),
                Err(GoOrdinaryError::Linkage)
            );
            if let Some((other, _, _)) = c.index_types().iter().find(|r| r.0 != *index) {
                assert_eq!(
                    c.import_row(row.certificate_bytes(), string, other),
                    Err(GoOrdinaryError::Linkage)
                );
            }
            let file = format!("{name}-{string}-{index}");
            if let Some(root) = &out {
                write(root, &format!("{file}.mpcert"), row.certificate_bytes());
                write(root,&format!("{file}.sources.json"),&serde_json::to_vec_pretty(&json!({"unit":name,"module":row.module_name(),"string_module":row.prior_string_row().module_name(),"source_layout_module":row.prior_string_row().source_layout_module_name(),"source_string_type_id":string,"source_index_type_id":index,"width":width,"signed":signed,"source_byte_type_id":row.source_byte_type_id(),"cases":c.strings().case_type_ids(),"types":c.strings().source_types(),"underlying":c.strings().source_types().iter().filter_map(|t|c.strings().underlying_type_id(t.id()).map(|u|json!([t.id(),u]))).collect::<Vec<_>>(),"declarations":decoded.declarations.len(),"terms":decoded.term_table.len(),"bytes":row.certificate_bytes().len(),"all_prior_source_string_identities_unchanged":true,"source_expression_or_context_authority":false})).unwrap());
            }
            let start = std::time::Instant::now();
            let r = mpk_kernel::verify_certificate_bytes_json_output(row.certificate_bytes());
            println!("{file}: {:?} {}", start.elapsed(), r.accepted);
            if let Some(root) = &out {
                write(
                    root,
                    &format!("{file}.rust.json"),
                    format!("{}\n", r.json).as_bytes(),
                );
            }
            assert!(r.accepted, "{}", r.json);
            let report: Value = serde_json::from_str(&r.json).unwrap();
            assert_eq!(report["axiom_count"], 0);
        }
    }
}
#[test]
fn source_string_indices_heterogeneous() {
    check("heterogeneous-payloads")
}
#[test]
fn source_string_indices_large() {
    check("seventy-cases")
}
#[test]
fn source_string_indices_nominal() {
    check("source-string-indices")
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

fn apply(c: &mut mpk_cert::encode::Certificate, f: u32, args: Vec<u32>) -> u32 {
    node(
        c,
        mpk_cert::encode::TermNode::App {
            function: f,
            arguments: args,
        },
    )
}
fn changed_lambda(c: &mut mpk_cert::encode::Certificate, n: u32, body: u32) -> u32 {
    use mpk_cert::encode::TermNode;
    match c.term_table[n as usize].clone() {
        TermNode::Lam { ty, body: old } => {
            let body = changed_lambda(c, old, body);
            node(c, TermNode::Lam { ty, body })
        }
        _ => body,
    }
}
fn control(
    name: &str,
    catalogue: &GoSourceStringIndexCatalogue,
    row: &GoSourceStringIndexRow,
    c: mpk_cert::encode::Certificate,
    accepted: bool,
) {
    let bytes = encoded(c);
    let string = row.prior_string_row().source_string_type_id();
    let index = row.source_index_type_id();
    assert_eq!(
        catalogue.import_row(&bytes, string, index),
        Err(GoOrdinaryError::Linkage)
    );
    let out = std::env::var_os("MPK_GO_SOURCE_STRING_INDEX_OUTPUT").map(PathBuf::from);
    let stem = format!("control-{name}");
    if let Some(root) = &out {
        write(root, &format!("{stem}.mpcert"), &bytes);
    }
    let r = mpk_kernel::verify_certificate_bytes_json_output(&bytes);
    if let Some(root) = &out {
        write(
            root,
            &format!("{stem}.rust.json"),
            format!("{}\n", r.json).as_bytes(),
        );
    }
    assert_eq!(r.accepted, accepted, "{name}: {}", r.json);
    let r: Value = serde_json::from_str(&r.json).unwrap();
    if accepted {
        assert_eq!(r["axiom_count"], 0)
    } else {
        assert_eq!(r["error_code"], "KERNEL_CORE_CHECK");
        let detail: Value = serde_json::from_str(r["error_detail"].as_str().unwrap()).unwrap();
        assert_eq!(detail["code"], "CORE_TYPE_MISMATCH");
    }
}
#[test]
fn source_string_indices_controls() {
    use mpk_cert::encode::{DeclarationKind, TermNode};
    let selection = dispatch("source-string-indices");
    let catalogue =
        generate_go_practical_source_string_index_catalogue(selection.domain()).unwrap();
    let string = &catalogue.strings().source_string_type_ids()[0];
    let signed = &catalogue
        .index_types()
        .iter()
        .find(|r| r.1 == 16 && r.2)
        .unwrap()
        .0;
    let row = catalogue.generate_row(string, signed).unwrap();
    let original = mpk_cert::decode_canonical_certificate(row.certificate_bytes()).unwrap();
    let mut c = original.clone();
    let n = format!("{}.Negative", row.module_name());
    let DeclarationKind::Def { value, .. } = c.declarations[member(&c, &n)].kind else {
        panic!()
    };
    let no = constant(&mut c, "Std.Bool.false");
    let value = changed_lambda(&mut c, value, no);
    define(&mut c, &n, value);
    control("erased-sign", &catalogue, &row, c, false);
    let index64 = &catalogue
        .index_types()
        .iter()
        .find(|r| r.1 == 64 && !r.2)
        .unwrap()
        .0;
    let row64 = catalogue.generate_row(string, index64).unwrap();
    let mut c = mpk_cert::decode_canonical_certificate(row64.certificate_bytes()).unwrap();
    let n = format!("{}.To64", row64.module_name());
    let DeclarationKind::Def { value, .. } = c.declarations[member(&c, &n)].kind else {
        panic!()
    };
    let TermNode::Lam { ty, .. } = c.term_table[value as usize] else {
        panic!()
    };
    let b = constant(&mut c, "Std.Bool");
    let no = constant(&mut c, "Std.Bool.false");
    let v = node(&mut c, TermNode::Var(6));
    let a2 = node(&mut c, TermNode::Var(2));
    let a1 = node(&mut c, TermNode::Var(1));
    let a0 = node(&mut c, TermNode::Var(0));
    let to_word = constant(&mut c, &format!("{}.ToWord", row64.module_name()));
    let f = apply(&mut c, to_word, vec![v]);
    let mut body = apply(&mut c, f, vec![no, no, no, a2, a1, a0]);
    for _ in 0..6 {
        body = node(&mut c, TermNode::Lam { ty: b, body });
    }
    let body = node(&mut c, TermNode::Lam { ty, body });
    define(&mut c, &n, body);
    control("repeated-low-byte", &catalogue, &row64, c, false);
    let n = format!("{}.Read", row.module_name());
    let mut c = original.clone();
    let DeclarationKind::Def { value, .. } = c.declarations[member(&c, &n)].kind else {
        panic!()
    };
    let zero = constant(
        &mut c,
        &format!(
            "{}.T{}.Zero",
            row.prior_string_row().source_layout_module_name(),
            row.source_byte_type_id()
        ),
    );
    let value = changed_lambda(&mut c, value, zero);
    define(&mut c, &n, value);
    control("constant-read", &catalogue, &row, c.clone(), false);
    let law = member(&c, &format!("{}.ReadWithinSlot", row.module_name()));
    let DeclarationKind::Theorem { ty, proof } = c.declarations[law].kind else {
        panic!()
    };
    fn rhs(c: &mut mpk_cert::encode::Certificate, t: u32, z: u32) -> u32 {
        match c.term_table[t as usize].clone() {
            TermNode::Pi { ty, body } => {
                let body = rhs(c, body, z);
                node(c, TermNode::Pi { ty, body })
            }
            TermNode::App {
                function,
                mut arguments,
            } => {
                assert_eq!(arguments.len(), 3);
                arguments[2] = z;
                node(
                    c,
                    TermNode::App {
                        function,
                        arguments,
                    },
                )
            }
            _ => panic!(),
        }
    }
    let word = constant(
        &mut c,
        &format!(
            "{}.T{}.Carrier",
            row.prior_string_row().source_layout_module_name(),
            row.source_byte_type_id()
        ),
    );
    let refl = constant(&mut c, "Std.Eq.refl");
    let refl = apply(&mut c, refl, vec![word, zero]);
    let proof = changed_lambda(&mut c, proof, refl);
    let ty = rhs(&mut c, ty, zero);
    c.declarations[law].kind = DeclarationKind::Theorem { ty, proof };
    if let Some(root) = std::env::var_os("MPK_GO_SOURCE_STRING_INDEX_OUTPUT").map(PathBuf::from) {
        write(&root,"control-coherent-read.sources.json",&serde_json::to_vec_pretty(&json!({"unit":"source-string-indices","module":row.module_name(),"string_module":row.prior_string_row().module_name(),"source_layout_module":row.prior_string_row().source_layout_module_name(),"source_string_type_id":string,"source_index_type_id":signed,"width":row.index_width(),"signed":row.is_signed_index(),"source_byte_type_id":row.source_byte_type_id(),"cases":catalogue.strings().case_type_ids(),"types":catalogue.strings().source_types(),"underlying":catalogue.strings().source_types().iter().filter_map(|t|catalogue.strings().underlying_type_id(t.id()).map(|u|json!([t.id(),u]))).collect::<Vec<_>>()})).unwrap());
    }
    control("coherent-read", &catalogue, &row, c, true);
}
