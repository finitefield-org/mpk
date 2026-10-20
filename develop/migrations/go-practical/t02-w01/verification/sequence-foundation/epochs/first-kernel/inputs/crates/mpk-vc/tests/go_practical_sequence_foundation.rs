use mpk_cert::encode::{Certificate, Declaration, DeclarationKind, TermNode};
use mpk_vc::go_practical_ordinary::{
    generate_go_practical_natural_foundation, generate_go_practical_product_foundation,
    generate_go_practical_sequence_foundation, import_go_practical_sequence_foundation,
    GoOrdinaryError,
};

#[path = "support/go_word_division_dependencies.rs"]
mod dependencies;

fn inspect(name: &str, bytes: &[u8], accepted: bool) {
    let result = mpk_kernel::verify_certificate_bytes_json_output(bytes);
    if let Some(path) = std::env::var_os("MPK_GO_SEQUENCE_FOUNDATION_OUTPUT") {
        let root = std::path::PathBuf::from(path);
        std::fs::create_dir_all(&root).unwrap();
        for (extension, raw) in [
            ("mpcert", bytes.to_vec()),
            ("rust.json", format!("{}\n", result.json).into_bytes()),
        ] {
            use std::io::Write;
            let mut out = std::fs::OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(root.join(format!("{name}.{extension}")))
                .unwrap();
            out.write_all(&raw).unwrap();
        }
    }
    assert_eq!(result.accepted, accepted, "{name}: {}", result.json);
    if !accepted {
        let output: serde_json::Value = serde_json::from_str(&result.json).unwrap();
        assert_eq!(output["error_code"], "KERNEL_CORE_CHECK", "{name}");
        let detail: serde_json::Value =
            serde_json::from_str(output["error_detail"].as_str().unwrap()).unwrap();
        assert_eq!(detail["code"], "CORE_TYPE_MISMATCH", "{name}");
    }
}

fn baseline() -> Certificate {
    mpk_cert::decode_canonical_certificate(
        generate_go_practical_sequence_foundation()
            .unwrap()
            .certificate_bytes(),
    )
    .unwrap()
}

fn encode(mut certificate: Certificate) -> Vec<u8> {
    let old = certificate.name_table.clone();
    certificate.name_table.sort();
    for declaration in &mut certificate.declarations {
        declaration.name = certificate
            .name_table
            .binary_search(&old[declaration.name as usize])
            .unwrap() as u32;
    }
    for level in &mut certificate.level_table {
        if let mpk_cert::encode::LevelNode::Param(name) = level {
            *name = certificate
                .name_table
                .binary_search(&old[*name as usize])
                .unwrap() as u32;
        }
    }
    assert!(certificate.declarations.len() <= 4096);
    assert!(certificate.term_table.len() <= 131_072);
    certificate.export_block = mpk_cert::build_export_block(&certificate).unwrap();
    certificate.axiom_report = mpk_cert::build_axiom_report(&certificate).unwrap();
    assert_eq!(certificate.axiom_report.summary.total_axiom_count, 0);
    certificate.hashes.export_hash = mpk_cert::export_block_hash(&certificate.export_block);
    certificate.hashes.axiom_report_hash =
        mpk_cert::axiom_report_hash_for_report(&certificate.axiom_report);
    mpk_cert::encode_certificate_bounded(&certificate, 2_097_152).unwrap()
}

fn term(c: &mut Certificate, node: TermNode) -> u32 {
    let id = c.term_table.len() as u32;
    c.term_table.push(node);
    id
}
fn constant(c: &mut Certificate, name: &str) -> u32 {
    let global = c
        .declarations
        .iter()
        .position(|d| c.name_table[d.name as usize] == name)
        .unwrap() as u32;
    term(
        c,
        TermNode::Const {
            global,
            levels: vec![],
        },
    )
}
fn call(c: &mut Certificate, name: &str, arguments: Vec<u32>) -> u32 {
    let function = constant(c, name);
    term(
        c,
        TermNode::App {
            function,
            arguments,
        },
    )
}
fn false_claim(c: &mut Certificate) -> u32 {
    let ty = constant(c, "Std.Bool");
    let no = constant(c, "Std.Bool.false");
    let yes = constant(c, "Std.Bool.true");
    call(c, "Std.Eq", vec![ty, no, yes])
}
fn replace_result(c: &mut Certificate, ty: u32, result: u32) -> u32 {
    match c.term_table[ty as usize].clone() {
        TermNode::Pi { ty, body } => {
            let body = replace_result(c, body, result);
            term(c, TermNode::Pi { ty, body })
        }
        _ => result,
    }
}

fn replace_domain(c: &mut Certificate, ty: u32, at: usize, domain: u32) -> u32 {
    let TermNode::Pi { ty: old, body } = c.term_table[ty as usize].clone() else {
        panic!("missing premise")
    };
    if at == 0 {
        term(c, TermNode::Pi { ty: domain, body })
    } else {
        let body = replace_domain(c, body, at - 1, domain);
        term(c, TermNode::Pi { ty: old, body })
    }
}

#[test]
fn actual_indexed_sequence_component_preserves_all_prerequisites() {
    let foundation = generate_go_practical_sequence_foundation().unwrap();
    let bytes = foundation.certificate_bytes();
    inspect("sequence-foundation", bytes, true);
    let certificate = baseline();
    assert!(certificate.declarations.len() <= 4096);
    assert!(certificate.term_table.len() <= 131_072);
    assert!(bytes.len() <= 2_097_152);
    assert_eq!(certificate.axiom_report.summary.total_axiom_count, 0);
    assert!(certificate.imports.is_empty());
    let all = dependencies::declarations(&certificate);
    for previous in [
        generate_go_practical_natural_foundation()
            .unwrap()
            .certificate_bytes()
            .to_vec(),
        generate_go_practical_product_foundation()
            .unwrap()
            .certificate_bytes()
            .to_vec(),
    ] {
        let old = mpk_cert::decode_canonical_certificate(&previous).unwrap();
        for (name, value) in dependencies::declarations(&old) {
            assert_eq!(all.get(&name), Some(&value), "{name}");
        }
    }
    println!(
        "sequences: declarations={} terms={} bytes={} zero axioms",
        certificate.declarations.len(),
        certificate.term_table.len(),
        bytes.len()
    );
}

#[test]
fn exact_sequence_ownership_is_separate_from_core_acceptance() {
    let foundation = generate_go_practical_sequence_foundation().unwrap();
    assert_eq!(
        foundation,
        import_go_practical_sequence_foundation(foundation.certificate_bytes()).unwrap()
    );
    assert_eq!(
        foundation,
        generate_go_practical_sequence_foundation().unwrap()
    );
    let mut c = baseline();
    let ty = constant(&mut c, "Std.Bool");
    let value = constant(&mut c, "Std.Bool.false");
    let name = c.name_table.len() as u32;
    c.name_table
        .push("Mpk.Go.Practical.Sequence.HonestExtra".into());
    c.declarations.push(Declaration {
        name,
        kind: DeclarationKind::Def {
            ty,
            value,
            reducibility: mpk_cert::encode::DefinitionReducibility::Reducible,
        },
    });
    let different = encode(c);
    inspect("sequence-honest-extra", &different, true);
    assert_eq!(
        import_go_practical_sequence_foundation(&different),
        Err(GoOrdinaryError::Linkage)
    );
    let mut trailing = foundation.certificate_bytes().to_vec();
    trailing.push(0);
    assert_eq!(
        import_go_practical_sequence_foundation(&trailing),
        Err(GoOrdinaryError::Linkage)
    );
    assert_eq!(
        import_go_practical_sequence_foundation(&vec![0; 2_097_153]),
        Err(GoOrdinaryError::Limit)
    );
}

#[test]
fn sequence_laws_reject_false_claims_and_changed_operations() {
    let original = baseline();
    let names = original
        .declarations
        .iter()
        .filter(|d| {
            matches!(d.kind, DeclarationKind::Theorem { .. })
                && original.name_table[d.name as usize].starts_with("Mpk.Go.Practical.Sequence.")
        })
        .map(|d| original.name_table[d.name as usize].clone())
        .collect::<Vec<_>>();
    assert!(names.len() >= 25);
    for name in names {
        let mut c = original.clone();
        let index = c
            .declarations
            .iter()
            .position(|d| c.name_table[d.name as usize] == name)
            .unwrap();
        let DeclarationKind::Theorem { ty, proof } = c.declarations[index].kind.clone() else {
            unreachable!()
        };
        let bad = false_claim(&mut c);
        let ty = replace_result(&mut c, ty, bad);
        c.declarations[index].kind = DeclarationKind::Theorem { ty, proof };
        inspect(
            &format!("false-{}", name.rsplit('.').next().unwrap()),
            &encode(c),
            false,
        );
    }
    for name in ["Set", "Map", "Offset", "Append"] {
        let mut c = original.clone();
        let index = c
            .declarations
            .iter()
            .position(|d| {
                c.name_table[d.name as usize] == format!("Mpk.Go.Practical.Sequence.{name}")
            })
            .unwrap();
        let DeclarationKind::Def {
            ty,
            value,
            reducibility,
        } = c.declarations[index].kind.clone()
        else {
            unreachable!()
        };
        // Keep the actual sequence result type. Each mutation changes the
        // mathematical operation: ignore an update, freeze mapping at index
        // zero, discard the offset, or discard the appended operand.
        fn bad_value(c: &mut Certificate, node: u32, operation: &str) -> u32 {
            match c.term_table[node as usize].clone() {
                TermNode::Lam { ty, body } => {
                    let body = bad_value(c, body, operation);
                    term(c, TermNode::Lam { ty, body })
                }
                _ => match operation {
                    "Set" => term(c, TermNode::Var(2)),
                    "Append" => term(c, TermNode::Var(1)),
                    "Map" | "Offset" => {
                        let natural = constant(c, "Std.Nat");
                        let a = term(c, TermNode::Var(4));
                        let source = term(c, TermNode::Var(if operation == "Map" { 1 } else { 3 }));
                        let index = if operation == "Map" {
                            constant(c, "Std.Nat.zero")
                        } else {
                            term(c, TermNode::Var(0))
                        };
                        let read = call(c, "Mpk.Go.Practical.Sequence.At", vec![a, source, index]);
                        let body = if operation == "Map" {
                            let f = term(c, TermNode::Var(2));
                            term(
                                c,
                                TermNode::App {
                                    function: f,
                                    arguments: vec![read],
                                },
                            )
                        } else {
                            read
                        };
                        let data = term(c, TermNode::Lam { ty: natural, body });
                        let target = term(c, TermNode::Var(if operation == "Map" { 2 } else { 3 }));
                        let count = if operation == "Map" {
                            let a = term(c, TermNode::Var(3));
                            let source = term(c, TermNode::Var(0));
                            call(c, "Mpk.Go.Practical.Sequence.Length", vec![a, source])
                        } else {
                            term(c, TermNode::Var(0))
                        };
                        call(
                            c,
                            "Mpk.Go.Practical.Sequence.Make",
                            vec![target, count, data],
                        )
                    }
                    _ => unreachable!(),
                },
            }
        }
        let value = bad_value(&mut c, value, name);
        c.declarations[index].kind = DeclarationKind::Def {
            ty,
            value,
            reducibility,
        };
        inspect(&format!("body-{name}"), &encode(c), false);
    }
    for (name, premise) in [
        ("ChoiceTrue", 4),
        ("ChoiceFalse", 4),
        ("SetOther", 5),
        ("ConstantValid", 4),
        ("MapValid", 5),
        ("MapValid", 7),
        ("SetValid", 5),
        ("SetValid", 6),
        ("OffsetValid", 5),
        ("OffsetValid", 6),
        ("PrefixSymmetric", 4),
        ("PrefixTransitive", 5),
        ("PrefixTransitive", 6),
        ("PrefixRestrict", 5),
        ("PrefixRestrict", 6),
        ("MapRespects", 6),
    ] {
        let mut c = original.clone();
        let index = c
            .declarations
            .iter()
            .position(|d| {
                c.name_table[d.name as usize] == format!("Mpk.Go.Practical.Sequence.{name}")
            })
            .unwrap();
        let DeclarationKind::Theorem { ty, proof } = c.declarations[index].kind.clone() else {
            unreachable!()
        };
        let boolean = constant(&mut c, "Std.Bool");
        let yes = constant(&mut c, "Std.Bool.true");
        let unrelated = call(&mut c, "Std.Eq", vec![boolean, yes, yes]);
        let ty = replace_domain(&mut c, ty, premise, unrelated);
        c.declarations[index].kind = DeclarationKind::Theorem { ty, proof };
        inspect(&format!("premise-{name}-{premise}"), &encode(c), false);
    }
}
