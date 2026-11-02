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
fn check(name: &str, domain: &GoSourceValueDomain) {
    let artifact = generate_go_practical_source_error_layout_foundation(domain).unwrap();
    assert_eq!(
        artifact.case_type_ids(),
        domain
            .error_cases()
            .iter()
            .map(|c| c.value_type_id().to_string())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        artifact,
        import_go_practical_source_error_layout_foundation(artifact.certificate_bytes(), domain)
            .unwrap()
    );
    let certificate = mpk_cert::decode_canonical_certificate(artifact.certificate_bytes()).unwrap();
    assert!(certificate.imports.is_empty());
    assert_eq!(certificate.axiom_report.summary.total_axiom_count, 0);
    assert_eq!(certificate.module, artifact.module_name());
    let prefix = format!("{}.T", artifact.module_name());
    assert_eq!(
        certificate
            .declarations
            .iter()
            .filter(|d| certificate.name_table[d.name as usize].starts_with(&prefix))
            .count(),
        3 * domain.types().count()
    );
    let out = std::env::var_os("MPK_GO_SOURCE_ERROR_LAYOUTS_OUTPUT").map(PathBuf::from);
    if let Some(root) = &out {
        write(
            root,
            &format!("{name}.mpcert"),
            artifact.certificate_bytes(),
        );
        let source = json!({"module":artifact.module_name(),"source_error_type":artifact.source_error_type_id(),"cases":artifact.case_type_ids(),"types":domain.types().collect::<Vec<_>>(),"underlying":domain.types().filter_map(|t|domain.underlying(t.id()).map(|u|json!([t.id(),u]))).collect::<Vec<_>>()});
        write(
            root,
            &format!("{name}.sources.json"),
            &serde_json::to_vec_pretty(&source).unwrap(),
        );
    }
    let started = std::time::Instant::now();
    let result = mpk_kernel::verify_certificate_bytes_json_output(artifact.certificate_bytes());
    println!("{name}: {:?}", started.elapsed());
    if let Some(root) = out {
        write(
            &root,
            &format!("{name}.rust.json"),
            format!("{}\n", result.json).as_bytes(),
        );
    }
    assert!(result.accepted, "{name}: {}", result.json);
}

#[test]
fn source_derived_empty_single_and_dispatch_grown_universes() {
    for name in ["nil-only", "boxing-only", "entry"] {
        check(name, dispatch(name).domain());
    }
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
#[test]
fn source_roots_keep_plain_root_string_array_and_zero_size_dependencies() {
    check("all-root-shapes", &root_domain());
}
#[test]
fn source_large_universe_keeps_every_case() {
    let selected = dispatch("seventy-cases");
    assert_eq!(selected.domain().error_cases().len(), 70);
    check("source-N70", selected.domain());
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
fn replace_field(
    c: &mut mpk_cert::encode::Certificate,
    value: u32,
    len: usize,
    index: usize,
    new: u32,
) -> u32 {
    use mpk_cert::encode::TermNode;
    if len == 1 {
        assert_eq!(index, 0);
        return new;
    }
    let TermNode::Lam { ty, body } = c.term_table[value as usize].clone() else {
        panic!("actual frozen product constructor")
    };
    let TermNode::App {
        function,
        mut arguments,
    } = c.term_table[body as usize].clone()
    else {
        panic!()
    };
    assert_eq!(arguments.len(), 4);
    let middle = len / 2;
    if index < middle {
        arguments[1] = replace_field(c, arguments[1], middle, index, new);
    } else {
        arguments[2] = replace_field(c, arguments[2], len - middle, index - middle, new);
    }
    let body = node(
        c,
        TermNode::App {
            function,
            arguments,
        },
    );
    node(c, TermNode::Lam { ty, body })
}
fn candidate(name: &str, bytes: &[u8], accepted: bool) {
    let out = std::env::var_os("MPK_GO_SOURCE_ERROR_LAYOUTS_OUTPUT").map(PathBuf::from);
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
#[test]
fn altered_actual_source_zeros_fail_their_retained_recipe_witnesses() {
    use mpk_cert::encode::{DeclarationKind, DefinitionReducibility, TermNode};
    let domain = root_domain();
    let foundation = generate_go_practical_source_error_layout_foundation(&domain).unwrap();
    candidate(
        "current-source-baseline",
        foundation.certificate_bytes(),
        true,
    );
    let types = domain.types().collect::<Vec<_>>();
    for label in [
        "bool-true",
        "word-all-ones",
        "empty-struct-true",
        "empty-array-true",
        "slice-not-nil",
        "string-not-empty",
        "error-tag-one",
        "array-padding-nonzero",
    ] {
        let key = types
            .iter()
            .find(|t| match (label, t) {
                ("bool-true", GoSourceTypeKey::Builtin { name, .. }) => name == "bool",
                ("word-all-ones", GoSourceTypeKey::Builtin { name, .. }) => name == "uint8",
                ("empty-struct-true", GoSourceTypeKey::Struct { fields, .. }) => fields.is_empty(),
                ("empty-array-true", GoSourceTypeKey::Array { length, .. }) => *length == 0,
                ("slice-not-nil", GoSourceTypeKey::Slice { .. }) => true,
                ("string-not-empty", GoSourceTypeKey::Builtin { name, .. }) => name == "string",
                ("error-tag-one", GoSourceTypeKey::Builtin { name, .. }) => name == "error",
                ("array-padding-nonzero", GoSourceTypeKey::Array { length, .. }) => *length == 31,
                _ => false,
            })
            .unwrap();
        let mut c = mpk_cert::decode_canonical_certificate(foundation.certificate_bytes()).unwrap();
        let member = format!("{}.T{}.Zero", foundation.module_name(), key.id());
        let id = c
            .declarations
            .iter()
            .position(|d| c.name_table[d.name as usize] == member)
            .unwrap();
        let DeclarationKind::Def { ty, value, .. } = c.declarations[id].kind else {
            panic!()
        };
        let replacement = match label {
            "bool-true" | "empty-struct-true" | "empty-array-true" => {
                constant(&mut c, "Std.Bool.true")
            }
            "word-all-ones" => {
                let mut v = constant(&mut c, "Std.Bool.true");
                let boolean = constant(&mut c, "Std.Bool");
                for _ in 0..3 {
                    v = node(
                        &mut c,
                        TermNode::Lam {
                            ty: boolean,
                            body: v,
                        },
                    );
                }
                v
            }
            "slice-not-nil" => {
                let v = constant(&mut c, "Std.Bool.false");
                replace_field(&mut c, value, 6, 0, v)
            }
            "string-not-empty" => {
                let v = constant(&mut c, "Mpk.Go.Practical.Word.W64.One");
                replace_field(&mut c, value, 2, 0, v)
            }
            "error-tag-one" => {
                let v = constant(&mut c, "Mpk.Go.Practical.Word.W64.One");
                replace_field(&mut c, value, domain.error_cases().len() + 1, 0, v)
            }
            _ => {
                let GoSourceTypeKey::Array { element, .. } = key else {
                    panic!()
                };
                assert!(
                    matches!(types.iter().find(|t|t.id()==element).unwrap(),GoSourceTypeKey::Builtin{name,..}if name=="string")
                );
                let string_name = format!("{}.T{element}.Zero", foundation.module_name());
                let string_id = c
                    .declarations
                    .iter()
                    .position(|d| c.name_table[d.name as usize] == string_name)
                    .unwrap();
                let DeclarationKind::Def {
                    value: string_value,
                    ..
                } = c.declarations[string_id].kind
                else {
                    panic!()
                };
                let one = constant(&mut c, "Mpk.Go.Practical.Word.W64.One");
                let nonzero = replace_field(&mut c, string_value, 2, 0, one);
                let boolean = constant(&mut c, "Std.Bool");
                let mut condition = constant(&mut c, "Std.Bool.true");
                let and = constant(&mut c, "Mpk.Go.Practical.Bool.And");
                for i in 0..5 {
                    let address = node(&mut c, TermNode::Var(i));
                    condition = node(
                        &mut c,
                        TermNode::App {
                            function: and,
                            arguments: vec![condition, address],
                        },
                    );
                }
                let string_ty = constant(
                    &mut c,
                    &format!("{}.T{element}.Carrier", foundation.module_name()),
                );
                let motive = node(
                    &mut c,
                    TermNode::Lam {
                        ty: boolean,
                        body: string_ty,
                    },
                );
                let zero = constant(&mut c, &string_name);
                let cases = constant(&mut c, "Std.Bool.cases");
                let mut v = node(
                    &mut c,
                    TermNode::App {
                        function: cases,
                        arguments: vec![motive, zero, nonzero, condition],
                    },
                );
                for _ in 0..5 {
                    v = node(
                        &mut c,
                        TermNode::Lam {
                            ty: boolean,
                            body: v,
                        },
                    );
                }
                v
            }
        };
        c.declarations[id].kind = DeclarationKind::Def {
            ty,
            value: replacement,
            reducibility: DefinitionReducibility::Reducible,
        };
        let bytes = encoded(c);
        assert_eq!(
            import_go_practical_source_error_layout_foundation(&bytes, &domain),
            Err(GoOrdinaryError::Linkage)
        );
        candidate(label, &bytes, false);
    }
}
#[test]
fn same_case_count_different_nominal_source_universe_fails_exact_import() {
    let first = dispatch("boxing-only");
    let other = dispatch("global-boxing-only");
    let a = generate_go_practical_source_error_layout_foundation(first.domain()).unwrap();
    let b = generate_go_practical_source_error_layout_foundation(other.domain()).unwrap();
    assert_eq!(a.case_type_ids().len(), b.case_type_ids().len());
    assert_ne!(a.case_type_ids(), b.case_type_ids());
    assert_eq!(
        import_go_practical_source_error_layout_foundation(a.certificate_bytes(), other.domain()),
        Err(GoOrdinaryError::Linkage)
    );
    candidate("same-shape-other-source", b.certificate_bytes(), true);
}
#[test]
fn current_producer_matches_every_original_source_error_stream() {
    use sha2::{Digest, Sha256};
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/go-practical/t02-w01/verification/source-error-layouts");
    let mut rows = vec![];
    for name in [
        "nil-only",
        "boxing-only",
        "entry",
        "all-root-shapes",
        "source-N70",
    ] {
        let domain = if name == "all-root-shapes" {
            root_domain()
        } else {
            dispatch(if name == "source-N70" {
                "seventy-cases"
            } else {
                name
            })
            .domain()
            .clone()
        };
        let epoch = if name == "source-N70" {
            "first-large"
        } else {
            "first"
        };
        let path = root
            .join(epoch)
            .join("certificates")
            .join(format!("{name}.mpcert"));
        let original = std::fs::read(&path).unwrap();
        let fresh = generate_go_practical_source_error_layout_foundation(&domain).unwrap();
        assert_eq!(fresh.certificate_bytes(), original, "{name} complete bytes");
        let report: Value =
            serde_json::from_slice(&std::fs::read(path.with_extension("rust.json")).unwrap())
                .unwrap();
        assert_eq!(report["verdict"], "accepted");
        assert_eq!(report["axiom_count"], 0);
        rows.push(json!({"name":name,"bytes":original.len(),"raw_sha256":format!("{:x}",Sha256::digest(&original)),"complete_current_original_bytes_literal_equal":true}));
    }
    if let Some(path) = std::env::var_os("MPK_GO_SOURCE_ERROR_REPLAY_OUTPUT") {
        std::fs::write(path, serde_json::to_vec_pretty(&rows).unwrap()).unwrap();
    }
}
