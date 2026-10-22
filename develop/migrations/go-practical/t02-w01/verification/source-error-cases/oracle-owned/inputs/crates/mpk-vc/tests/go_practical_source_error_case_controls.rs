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
    let roots = input["roots"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| locate(&keys, &r["ref"], "FuncDecl"))
        .collect::<Vec<_>>();
    let entries = if matches!(name, "nil-only" | "entry" | "seventy-cases") {
        vec![GoErrorEntryConstraint::new(
            roots[0].clone(),
            0,
            vec![],
            cases,
        )]
    } else {
        vec![]
    };
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
fn root_domain() -> GoSourceValueDomain {
    let c = corpus("source-domains");
    let keys = keys(&c);
    let roots = ["Plain", "WeightGood", "Depth16", "Empty", "Scalar"]
        .iter()
        .map(|n| locate(&keys, &c["roots"][n], "FuncDecl"))
        .collect::<Vec<_>>();
    let cases = ["Code", "Flag", "Unit", "Ref"]
        .iter()
        .map(|n| {
            locate(
                &keys,
                &c["declarations"][format!("example.com/policy.{n}")],
                "TypeSpec",
            )
        })
        .collect();
    let selection = derive_go_source_selection(&keys, &roots).unwrap();
    let domain = derive_go_source_value_domain(
        &selection,
        &[GoErrorEntryConstraint::new(
            roots[0].clone(),
            0,
            vec![],
            cases,
        )],
    )
    .unwrap();
    assert_eq!(domain.error_cases().len(), 4);
    domain
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
    let out = std::env::var_os("MPK_GO_SOURCE_ERROR_CASES_OUTPUT").map(PathBuf::from);
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
fn change_result(c: &mut mpk_cert::encode::Certificate, name: &str, expected: bool) {
    use mpk_cert::encode::{DeclarationKind, TermNode};
    fn result(c: &mut mpk_cert::encode::Certificate, ty: u32, value: u32) -> u32 {
        if let TermNode::Pi { ty: domain, body } = c.term_table[ty as usize].clone() {
            let body = result(c, body, value);
            return node(c, TermNode::Pi { ty: domain, body });
        }
        let TermNode::App {
            function,
            mut arguments,
        } = c.term_table[ty as usize].clone()
        else {
            panic!()
        };
        assert_eq!(arguments.len(), 3);
        arguments[2] = value;
        node(
            c,
            TermNode::App {
                function,
                arguments,
            },
        )
    }
    let i = member(c, name);
    let DeclarationKind::Theorem { ty, proof } = c.declarations[i].kind else {
        panic!()
    };
    let value = constant(
        c,
        if expected {
            "Std.Bool.true"
        } else {
            "Std.Bool.false"
        },
    );
    let ty = result(c, ty, value);
    c.declarations[i].kind = DeclarationKind::Theorem { ty, proof };
}
fn reject(
    name: &str,
    catalogue: &GoSourceErrorCaseCatalogue,
    row: &GoSourceErrorCaseRow,
    c: mpk_cert::encode::Certificate,
) {
    let bytes = encoded(c);
    assert_eq!(
        catalogue.import_row(&bytes, row.boxed_source_type_id()),
        Err(GoOrdinaryError::Linkage)
    );
    candidate(name, &bytes, false);
}
fn collide(c: &mut mpk_cert::encode::Certificate, boxing: &str, cases: &[(String, u64)]) {
    let value = constant(c, &format!("{boxing}.C{}.Tag", cases[0].0));
    define(c, &format!("{boxing}.C{}.Tag", cases[2].0), value);
}
fn false_proof(c: &mut mpk_cert::encode::Certificate, proof: u32) -> u32 {
    use mpk_cert::encode::TermNode;
    if let TermNode::Lam { ty, body } = c.term_table[proof as usize].clone() {
        let body = false_proof(c, body);
        return node(c, TermNode::Lam { ty, body });
    }
    let function = constant(c, "Std.Eq.refl");
    let boolean = constant(c, "Std.Bool");
    let no = constant(c, "Std.Bool.false");
    node(
        c,
        TermNode::App {
            function,
            arguments: vec![boolean, no],
        },
    )
}
fn heal_constant_false_case_test(c: &mut mpk_cert::encode::Certificate, operations: &str) {
    use mpk_cert::encode::{DeclarationKind, TermNode};
    let i = member(c, &format!("{operations}.HasCase"));
    let DeclarationKind::Def { value, .. } = c.declarations[i].kind else {
        panic!()
    };
    let TermNode::Lam {
        ty: case_type,
        body,
    } = c.term_table[value as usize].clone()
    else {
        panic!()
    };
    let TermNode::Lam { ty: error_type, .. } = c.term_table[body as usize].clone() else {
        panic!()
    };
    let no = constant(c, "Std.Bool.false");
    let body = node(
        c,
        TermNode::Lam {
            ty: error_type,
            body: no,
        },
    );
    let value = node(
        c,
        TermNode::Lam {
            ty: case_type,
            body,
        },
    );
    define(c, &format!("{operations}.HasCase"), value);
    let ids = c
        .declarations
        .iter()
        .enumerate()
        .filter(|(_, d)| {
            let n = &c.name_table[d.name as usize];
            n == &format!("{operations}.HasCaseOfCopy")
                || n == &format!("{operations}.DifferentTagHasNoCase")
                || (n.starts_with(&format!("{operations}.C")) && n.ends_with(".NilHasNoCase"))
        })
        .map(|(i, _)| i)
        .collect::<Vec<_>>();
    for i in ids {
        let DeclarationKind::Theorem { ty, proof } = c.declarations[i].kind else {
            panic!()
        };
        let proof = false_proof(c, proof);
        c.declarations[i].kind = DeclarationKind::Theorem { ty, proof };
    }
}
#[test]
fn complete_case_row_controls_require_distinct_tags_and_real_matches() {
    let domain = root_domain();
    let catalogue = generate_go_practical_source_error_case_catalogue(&domain).unwrap();
    assert_eq!(catalogue.case_tags().len(), 4);
    let id = &catalogue.case_tags()[0].0;
    let other = &catalogue.case_tags()[1].0;
    let row = catalogue.generate_row(id).unwrap();
    let original=Path::new(env!("CARGO_MANIFEST_DIR")).join("../../develop/migrations/go-practical/t02-w01/verification/source-error-cases/first/certificates").join(format!("all-root-shapes-C{id}.mpcert"));
    assert_eq!(row.certificate_bytes(), std::fs::read(original).unwrap());
    for (name, member, expected) in [
        ("wrong-positive-match", format!("Matches.C{other}"), true),
        ("wrong-negative-match", format!("Matches.C{id}"), false),
        (
            "wrong-positive-tag-equality",
            format!("TagEqual.C{other}"),
            true,
        ),
        (
            "wrong-negative-tag-equality",
            format!("TagEqual.C{id}"),
            false,
        ),
    ] {
        let mut c = mpk_cert::decode_canonical_certificate(row.certificate_bytes()).unwrap();
        change_result(&mut c, &format!("{}.{member}", row.module_name()), expected);
        reject(name, &catalogue, &row, c);
    }
    // The earlier nil/copy mathematics allowed a coherent duplicate of tags1/3.
    // The complete row's off-diagonal equations make that mutation inconsistent.
    let mut c =
        mpk_cert::decode_canonical_certificate(catalogue.prior_operations_certificate_bytes())
            .unwrap();
    collide(
        &mut c,
        catalogue.boxing_module_name(),
        catalogue.case_tags(),
    );
    let bytes = encoded(c);
    assert_eq!(
        import_go_practical_source_error_operations_foundation(&bytes, &domain),
        Err(GoOrdinaryError::Linkage)
    );
    candidate("duplicate-tags-prior-math", &bytes, true);
    let mut c = mpk_cert::decode_canonical_certificate(row.certificate_bytes()).unwrap();
    collide(
        &mut c,
        catalogue.boxing_module_name(),
        catalogue.case_tags(),
    );
    reject("duplicate-tags-row", &catalogue, &row, c);
    // Retain the exact earlier coherent constant-false nil/copy alternative.
    // A row must also prove its own constructor matches, so it rejects this heal.
    let mut c =
        mpk_cert::decode_canonical_certificate(catalogue.prior_operations_certificate_bytes())
            .unwrap();
    heal_constant_false_case_test(&mut c, catalogue.operations_module_name());
    let bytes = encoded(c);
    assert_eq!(
        import_go_practical_source_error_operations_foundation(&bytes, &domain),
        Err(GoOrdinaryError::Linkage)
    );
    candidate("constant-false-prior-math", &bytes, true);
    let mut c = mpk_cert::decode_canonical_certificate(row.certificate_bytes()).unwrap();
    heal_constant_false_case_test(&mut c, catalogue.operations_module_name());
    reject("constant-false-row", &catalogue, &row, c);
}
#[test]
fn last_large_case_row_keeps_the_first_target_rejection() {
    let selected = dispatch("seventy-cases");
    let catalogue = generate_go_practical_source_error_case_catalogue(selected.domain()).unwrap();
    assert_eq!(catalogue.case_tags().len(), 70);
    let id = &catalogue.case_tags()[69].0;
    let row = catalogue.generate_row(id).unwrap();
    let original=Path::new(env!("CARGO_MANIFEST_DIR")).join("../../develop/migrations/go-practical/t02-w01/verification/source-error-cases/first-large/certificates").join(format!("source-cases-N70-C{id}.mpcert"));
    assert_eq!(row.certificate_bytes(), std::fs::read(original).unwrap());
    let mut c = mpk_cert::decode_canonical_certificate(row.certificate_bytes()).unwrap();
    change_result(
        &mut c,
        &format!(
            "{}.Matches.C{}",
            row.module_name(),
            catalogue.case_tags()[0].0
        ),
        true,
    );
    reject("last-row-first-target-wrong-positive", &catalogue, &row, c);
}
