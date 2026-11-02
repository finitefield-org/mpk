use mpk_vc::go_practical_ordinary::*;
use mpk_vc::go_practical_type_keys::*;
use serde_json::{json, Value};
use std::io::Write;
use std::path::{Path, PathBuf};

#[path = "../../mpk-cli/src/go_practical_checked_strings.rs"]
#[allow(dead_code)]
mod checked_strings;

fn corpus(name: &str) -> Value {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/go-practical/t02-w01/verification");
    serde_json::from_slice(&std::fs::read(root.join(name).join("corpus.json")).unwrap()).unwrap()
}
fn error_corpus(name: &str) -> Value {
    if name == "source-strings" {
        corpus("source-strings")
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
    let domain = selection.domain();
    let catalogue = generate_go_practical_source_string_catalogue(domain).unwrap();
    assert_eq!(
        catalogue.source_types(),
        domain.types().cloned().collect::<Vec<_>>()
    );
    assert_eq!(
        catalogue.case_type_ids(),
        domain
            .error_cases()
            .iter()
            .map(|c| c.value_type_id().to_string())
            .collect::<Vec<_>>()
    );
    if name == "source-strings" {
        assert_eq!(catalogue.source_string_type_ids().len(), 4);
    }
    let prior =
        mpk_cert::decode_canonical_certificate(catalogue.prior_layout_certificate_bytes()).unwrap();
    let generic =
        mpk_cert::decode_canonical_certificate(catalogue.generic_string_certificate_bytes())
            .unwrap();
    let before = dependencies::declarations(&prior);
    let generic_before = dependencies::declarations(&generic);
    let out = std::env::var_os("MPK_GO_SOURCE_STRING_OUTPUT").map(PathBuf::from);
    assert_eq!(
        catalogue.generate_row("not-a-source-string").unwrap_err(),
        GoOrdinaryError::Linkage
    );
    for id in catalogue.source_string_type_ids() {
        let row = catalogue.generate_row(id).unwrap();
        let decoded = mpk_cert::decode_canonical_certificate(row.certificate_bytes()).unwrap();
        let after = dependencies::declarations(&decoded);
        for (n, v) in before
            .iter()
            .filter(|(n, _)| n.starts_with(&(row.source_layout_module_name().to_string() + ".")))
        {
            assert_eq!(
                after.get(n),
                Some(v),
                "complete original source type/zero/proof: {n}"
            );
        }
        for (n, v) in generic_before
            .iter()
            .filter(|(n, _)| n.starts_with("Mpk.Go.Practical.String."))
        {
            assert_eq!(
                after.get(n),
                Some(v),
                "unchanged checked generic string identity: {n}"
            );
        }
        assert!(decoded.imports.is_empty());
        assert!(decoded.theory_certificates.is_empty());
        assert_eq!(decoded.axiom_report.summary.total_axiom_count, 0);
        assert_eq!(
            catalogue.import_row(row.certificate_bytes(), id).unwrap(),
            row
        );
        assert_eq!(
            catalogue.import_row(row.certificate_bytes(), "not-a-source-string"),
            Err(GoOrdinaryError::Linkage)
        );
        if let Some(other) = catalogue.source_string_type_ids().iter().find(|s| *s != id) {
            assert_eq!(
                catalogue.import_row(row.certificate_bytes(), other),
                Err(GoOrdinaryError::Linkage)
            );
        }
        let file = format!("{name}-{id}");
        if let Some(root) = &out {
            write(root, &format!("{file}.mpcert"), row.certificate_bytes());
            write(root,&format!("{file}.sources.json"),&serde_json::to_vec_pretty(&json!({"unit":name,"module":row.module_name(),"source_layout_module":row.source_layout_module_name(),"source_string_type_id":id,"cases":catalogue.case_type_ids(),"types":catalogue.source_types(),"underlying":catalogue.source_types().iter().filter_map(|t|catalogue.underlying_type_id(t.id()).map(|u|json!([t.id(),u]))).collect::<Vec<_>>(),"all_original_source_and_generic_string_identities_unchanged":true,"declarations":decoded.declarations.len(),"terms":decoded.term_table.len(),"bytes":row.certificate_bytes().len(),"source_expression_or_context_authority":false})).unwrap());
        }
        let start = std::time::Instant::now();
        let result = mpk_kernel::verify_certificate_bytes_json_output(row.certificate_bytes());
        println!("{file}: {:?} {}", start.elapsed(), result.accepted);
        if let Some(root) = &out {
            write(
                root,
                &format!("{file}.rust.json"),
                format!("{}\n", result.json).as_bytes(),
            );
        }
        assert!(result.accepted, "{}", result.json);
        let report: Value = serde_json::from_str(&result.json).unwrap();
        assert_eq!(report["axiom_count"], 0);
    }
}
#[test]
fn source_strings_empty_and_entry() {
    for name in ["nil-only", "entry"] {
        check(name);
    }
}
#[test]
fn source_strings_heterogeneous() {
    check("heterogeneous-payloads");
}
#[test]
fn source_strings_large() {
    check("seventy-cases");
}
#[test]
fn source_strings_nominal() {
    check("source-strings");
}

fn check_original_checked_strings(name: &str, cases: usize, strings: usize) {
    let selection = dispatch(name);
    let domain = selection.domain();
    assert_eq!(domain.error_cases().len(), cases);
    let catalogue = generate_go_practical_source_string_catalogue(domain).unwrap();
    let checked = checked_strings::check_go_string_catalogue(&catalogue, domain).unwrap();
    assert_eq!(checked.receipts().len(), strings);
    for (index, (receipt, id)) in checked
        .receipts()
        .iter()
        .zip(catalogue.source_string_type_ids())
        .enumerate()
    {
        assert_eq!(receipt.source_type_id(), id);
        let row = checked.reconstruct(&catalogue, index).unwrap();
        assert_eq!(row.module_name(), receipt.module());
        assert_eq!(
            mpk_cert::certificate_hash(row.certificate_bytes()),
            *receipt.certificate_hash()
        );
        if let Some(output) = std::env::var_os("MPK_GO_CHECKED_STRING_OUTPUT") {
            let output = PathBuf::from(output);
            write(
                &output,
                &format!("{name}-{id}.mpcert"),
                row.certificate_bytes(),
            );
            write(
                &output,
                &format!("{name}-{id}.rust.json"),
                &serde_json::to_vec(&json!({
                    "verdict": "accepted", "module": receipt.module(),
                    "declaration_count": receipt.declarations(), "axiom_count": 0,
                    "hashes": {"export": mpk_cert::hash_hex(receipt.export_hash()),
                        "axiom_report": mpk_cert::hash_hex(receipt.axiom_report_hash()),
                        "certificate": mpk_cert::hash_hex(receipt.certificate_hash())}
                }))
                .unwrap(),
            );
        }
    }
    assert!(matches!(
        checked.reconstruct(&catalogue, strings),
        Err(checked_strings::GoCheckedStringError::Coverage { .. })
    ));
    eprintln!("{name}: checked {strings} original strings and {cases} error cases");
}

#[test]
fn checked_source_strings_small_and_nominal() {
    for (name, cases, strings) in [
        ("nil-only", 0, 1),
        ("entry", 3, 1),
        ("heterogeneous-payloads", 6, 1),
        ("source-strings", 4, 4),
    ] {
        check_original_checked_strings(name, cases, strings);
    }
    let first = dispatch("nil-only");
    let other = dispatch("entry");
    let first_catalogue = generate_go_practical_source_string_catalogue(first.domain()).unwrap();
    let other_catalogue = generate_go_practical_source_string_catalogue(other.domain()).unwrap();
    let checked =
        checked_strings::check_go_string_catalogue(&first_catalogue, first.domain()).unwrap();
    assert!(matches!(
        checked_strings::check_go_string_catalogue(&first_catalogue, other.domain()),
        Err(checked_strings::GoCheckedStringError::Coverage { .. })
    ));
    assert!(checked.reconstruct(&other_catalogue, 0).is_err());
}

#[test]
fn checked_source_strings_complete_error_universe() {
    check_original_checked_strings("seventy-cases", 70, 1);
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

fn apply(c: &mut mpk_cert::encode::Certificate, function: u32, arguments: Vec<u32>) -> u32 {
    node(
        c,
        mpk_cert::encode::TermNode::App {
            function,
            arguments,
        },
    )
}
fn source_string_control(
    name: &str,
    catalogue: &GoSourceStringCatalogue,
    id: &str,
    c: mpk_cert::encode::Certificate,
    accepted: bool,
) {
    let bytes = encoded(c);
    assert_eq!(
        catalogue.import_row(&bytes, id),
        Err(GoOrdinaryError::Linkage)
    );
    let out = std::env::var_os("MPK_GO_SOURCE_STRING_OUTPUT").map(PathBuf::from);
    let stem = format!("control-{name}");
    if let Some(root) = &out {
        write(root, &format!("{stem}.mpcert"), &bytes);
    }
    let start = std::time::Instant::now();
    let r = mpk_kernel::verify_certificate_bytes_json_output(&bytes);
    println!("{stem}: {:?} {}", start.elapsed(), r.accepted);
    if let Some(root) = &out {
        write(
            root,
            &format!("{stem}.rust.json"),
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
#[test]
fn source_strings_controls() {
    use mpk_cert::encode::{DeclarationKind, TermNode};
    let selection = dispatch("source-strings");
    let catalogue = generate_go_practical_source_string_catalogue(selection.domain()).unwrap();
    let id = &catalogue.source_string_type_ids()[0];
    let row = catalogue.generate_row(id).unwrap();
    let original = mpk_cert::decode_canonical_certificate(row.certificate_bytes()).unwrap();
    let module = row.module_name();
    for name in ["Length", "Data"] {
        let mut c = original.clone();
        let n = format!("{module}.{name}");
        let DeclarationKind::Def { value, .. } = c.declarations[member(&c, &n)].kind else {
            panic!()
        };
        let TermNode::Lam { ty, .. } = c.term_table[value as usize] else {
            panic!()
        };
        let projection = constant(&mut c, &format!("Mpk.Go.Practical.String.{name}"));
        let zero = constant(&mut c, "Mpk.Go.Practical.String.Zero");
        let body = apply(&mut c, projection, vec![zero]);
        let lambda = node(&mut c, TermNode::Lam { ty, body });
        define(&mut c, &n, lambda);
        source_string_control(
            &format!("constant-{}", name.to_lowercase()),
            &catalogue,
            id,
            c,
            false,
        );
    }
    let mut c = original.clone();
    let DeclarationKind::Def { value, .. } =
        c.declarations[member(&c, &format!("{module}.LengthFits"))].kind
    else {
        panic!()
    };
    define(&mut c, &format!("{module}.Valid"), value);
    source_string_control("missing-padding-validity", &catalogue, id, c, false);
    let mut c = original.clone();
    let n = format!("{module}.ToSource");
    let DeclarationKind::Def { value, .. } = c.declarations[member(&c, &n)].kind else {
        panic!()
    };
    let TermNode::Lam { ty, .. } = c.term_table[value as usize] else {
        panic!()
    };
    let body = constant(
        &mut c,
        &format!("{}.T{id}.Zero", row.source_layout_module_name()),
    );
    let value = node(&mut c, TermNode::Lam { ty, body });
    define(&mut c, &n, value);
    source_string_control("constant-to-source", &catalogue, id, c, false);
    // A coherent false source adapter can still describe true mathematics.
    // Exact import and the independent original-source oracle must reject it.
    let mut c = original.clone();
    let n = format!("{module}.Length");
    let DeclarationKind::Def { value, .. } = c.declarations[member(&c, &n)].kind else {
        panic!()
    };
    let TermNode::Lam { ty, .. } = c.term_table[value as usize] else {
        panic!()
    };
    let projection = constant(&mut c, "Mpk.Go.Practical.String.Length");
    let zero = constant(&mut c, "Mpk.Go.Practical.String.Zero");
    let body = apply(&mut c, projection, vec![zero]);
    let value = node(&mut c, TermNode::Lam { ty, body });
    define(&mut c, &n, value);
    let theorem = member(&c, &format!("{module}.MakeLength"));
    let DeclarationKind::Theorem { ty, proof } = c.declarations[theorem].kind else {
        panic!()
    };
    fn weaken_type(c: &mut mpk_cert::encode::Certificate, t: u32, zero: u32) -> u32 {
        match c.term_table[t as usize].clone() {
            TermNode::Pi { ty, body } => {
                let body = weaken_type(c, body, zero);
                node(c, TermNode::Pi { ty, body })
            }
            TermNode::App {
                function,
                mut arguments,
            } => {
                assert_eq!(arguments.len(), 3);
                arguments[2] = zero;
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
    fn weaken_proof(c: &mut mpk_cert::encode::Certificate, t: u32, zero: u32) -> u32 {
        match c.term_table[t as usize].clone() {
            TermNode::Lam { ty, body } => {
                let body = weaken_proof(c, body, zero);
                node(c, TermNode::Lam { ty, body })
            }
            _ => {
                let refl = constant(c, "Std.Eq.refl");
                let word = constant(c, "Mpk.Go.Practical.String.LengthType");
                apply(c, refl, vec![word, zero])
            }
        }
    }
    let ty = weaken_type(&mut c, ty, body);
    let proof = weaken_proof(&mut c, proof, body);
    c.declarations[theorem].kind = DeclarationKind::Theorem { ty, proof };
    let bytes = encoded(c.clone());
    if let Some(root) = std::env::var_os("MPK_GO_SOURCE_STRING_OUTPUT").map(PathBuf::from) {
        let original_meta = json!({"unit":"source-strings","module":row.module_name(),"source_layout_module":row.source_layout_module_name(),"source_string_type_id":id,"cases":catalogue.case_type_ids(),"types":catalogue.source_types(),"underlying":catalogue.source_types().iter().filter_map(|t|catalogue.underlying_type_id(t.id()).map(|u|json!([t.id(),u]))).collect::<Vec<_>>()});
        write(
            &root,
            "control-coherent-length.sources.json",
            &serde_json::to_vec_pretty(&original_meta).unwrap(),
        );
    }
    assert_eq!(
        catalogue.import_row(&bytes, id),
        Err(GoOrdinaryError::Linkage)
    );
    source_string_control("coherent-length", &catalogue, id, c, true);
}
