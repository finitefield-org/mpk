use mpk_cert::encode::{Certificate, Declaration, DeclarationKind, TermNode};
use mpk_vc::go_practical_ordinary::{
    generate_go_practical_finite_vector_foundation, generate_go_practical_sequence_foundation,
    import_go_practical_finite_vector_foundation, GoOrdinaryError,
};

#[path = "support/go_word_division_dependencies.rs"]
mod dependencies;

fn inspect(name: &str, bytes: &[u8], accepted: bool) {
    let result = mpk_kernel::verify_certificate_bytes_json_output(bytes);
    if let Some(path) = std::env::var_os("MPK_GO_FINITE_VECTOR_FOUNDATION_OUTPUT") {
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
    if accepted && !result.accepted {
        let certificate = mpk_cert::decode_canonical_certificate(bytes).unwrap();
        for end in 1..=certificate.declarations.len() {
            let mut prefix = certificate.clone();
            prefix.declarations.truncate(end);
            if let Err(error) = mpk_kernel::check_declarations(&prefix) {
                let declaration = &certificate.declarations[end - 1];
                panic!(
                    "{name}: first declaration type-check failure {}: {error:?}; full verdict {}",
                    certificate.name_table[declaration.name as usize], result.json
                );
            }
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
        generate_go_practical_finite_vector_foundation()
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
fn finite_vectors_preserve_all_sequence_prerequisites_and_slot_depths() {
    let foundation = generate_go_practical_finite_vector_foundation().unwrap();
    let bytes = foundation.certificate_bytes();
    inspect("finite-vector-foundation", bytes, true);
    let certificate = baseline();
    assert!(certificate.declarations.len() <= 4096);
    assert!(certificate.term_table.len() <= 131_072);
    assert!(bytes.len() <= 2_097_152);
    assert_eq!(certificate.axiom_report.summary.total_axiom_count, 0);
    assert!(certificate.imports.is_empty());
    let all = dependencies::declarations(&certificate);
    let previous = generate_go_practical_sequence_foundation().unwrap();
    let previous = mpk_cert::decode_canonical_certificate(previous.certificate_bytes()).unwrap();
    for (name, value) in dependencies::declarations(&previous) {
        assert_eq!(all.get(&name), Some(&value), "{name}");
    }
    for d in 0..=8 {
        for name in [
            "Type",
            "Capacity",
            "Index",
            "Lookup",
            "All",
            "Fits",
            "Embed",
            "IndexPaddedBound",
            "IndexBound",
            "LookupIndex",
            "LookupValid",
            "EmbedValid",
        ] {
            assert!(all.contains_key(&format!("Mpk.Go.Practical.FiniteVector.D{d}.{name}")));
        }
        let declaration = certificate
            .declarations
            .iter()
            .find(|item| {
                certificate.name_table[item.name as usize]
                    == format!("Mpk.Go.Practical.FiniteVector.D{d}.Type")
            })
            .unwrap();
        let DeclarationKind::Def { value, .. } = declaration.kind else {
            panic!("type definition")
        };
        let TermNode::Lam {
            body: mut value, ..
        } = certificate.term_table[value as usize]
        else {
            panic!("element parameter")
        };
        for _ in 0..d {
            let TermNode::Pi { ty, body } = certificate.term_table[value as usize] else {
                panic!("missing slot bit")
            };
            let TermNode::Const { global, .. } = certificate.term_table[ty as usize] else {
                panic!("Bool slot address")
            };
            assert_eq!(
                certificate.name_table[certificate.declarations[global as usize].name as usize],
                "Std.Bool"
            );
            value = body;
        }
        assert_eq!(
            certificate.term_table[value as usize],
            TermNode::Var(d as u32)
        );
    }
    println!(
        "finite vectors: declarations={} terms={} bytes={} zero axioms; depths=0..8 slots=1..256",
        certificate.declarations.len(),
        certificate.term_table.len(),
        bytes.len()
    );
}

#[test]
fn exact_finite_vector_ownership_is_separate_from_core_acceptance() {
    let foundation = generate_go_practical_finite_vector_foundation().unwrap();
    assert_eq!(
        foundation,
        import_go_practical_finite_vector_foundation(foundation.certificate_bytes()).unwrap()
    );
    assert_eq!(
        foundation,
        generate_go_practical_finite_vector_foundation().unwrap()
    );
    let mut c = baseline();
    let ty = constant(&mut c, "Std.Bool");
    let value = constant(&mut c, "Std.Bool.false");
    let name = c.name_table.len() as u32;
    c.name_table
        .push("Mpk.Go.Practical.FiniteVector.HonestExtra".into());
    c.declarations.push(Declaration {
        name,
        kind: DeclarationKind::Def {
            ty,
            value,
            reducibility: mpk_cert::encode::DefinitionReducibility::Reducible,
        },
    });
    let different = encode(c);
    inspect("finite-vector-honest-extra", &different, true);
    assert_eq!(
        import_go_practical_finite_vector_foundation(&different),
        Err(GoOrdinaryError::Linkage)
    );
    let mut trailing = foundation.certificate_bytes().to_vec();
    trailing.push(0);
    assert_eq!(
        import_go_practical_finite_vector_foundation(&trailing),
        Err(GoOrdinaryError::Linkage)
    );
    assert_eq!(
        import_go_practical_finite_vector_foundation(&vec![0; 2_097_153]),
        Err(GoOrdinaryError::Limit)
    );
}

fn changed_operation(c: &mut Certificate, node: u32, operation: &str, d: usize) -> u32 {
    match c.term_table[node as usize].clone() {
        TermNode::Lam { ty, body } => {
            let body = changed_operation(c, body, operation, d);
            term(c, TermNode::Lam { ty, body })
        }
        _ => match operation {
            "Capacity" => constant(
                c,
                &format!("Mpk.Go.Practical.FiniteVector.D{}.Capacity", d - 1),
            ),
            "Index" if d == 0 => {
                let zero = constant(c, "Std.Nat.zero");
                call(c, "Std.Nat.succ", vec![zero])
            }
            "Index" => {
                let tail = (0..d - 1)
                    .rev()
                    .map(|i| term(c, TermNode::Var(i as u32)))
                    .collect();
                call(
                    c,
                    &format!("Mpk.Go.Practical.FiniteVector.D{}.Index", d - 1),
                    tail,
                )
            }
            "Lookup" => {
                let a = term(c, TermNode::Var(2));
                let data = term(c, TermNode::Var(1));
                let no = constant(c, "Std.Bool.false");
                let data = term(
                    c,
                    TermNode::App {
                        function: data,
                        arguments: vec![no],
                    },
                );
                let i = term(c, TermNode::Var(0));
                call(
                    c,
                    &format!("Mpk.Go.Practical.FiniteVector.D{}.Lookup", d - 1),
                    vec![a, data, i],
                )
            }
            "Embed" => {
                let a = term(c, TermNode::Var(2));
                let n = constant(c, "Std.Nat.zero");
                let a_in = term(c, TermNode::Var(3));
                let data_in = term(c, TermNode::Var(1));
                let i = term(c, TermNode::Var(0));
                let read = call(
                    c,
                    &format!("Mpk.Go.Practical.FiniteVector.D{d}.Lookup"),
                    vec![a_in, data_in, i],
                );
                let natural = constant(c, "Std.Nat");
                let function = term(
                    c,
                    TermNode::Lam {
                        ty: natural,
                        body: read,
                    },
                );
                call(c, "Mpk.Go.Practical.Sequence.Make", vec![a, n, function])
            }
            _ => unreachable!(),
        },
    }
}

#[test]
fn finite_vector_laws_reject_false_claims_erased_bounds_and_wrong_operations() {
    let original = baseline();
    let names = original
        .declarations
        .iter()
        .filter(|d| {
            matches!(d.kind, DeclarationKind::Theorem { .. })
                && original.name_table[d.name as usize]
                    .starts_with("Mpk.Go.Practical.FiniteVector.")
        })
        .map(|d| original.name_table[d.name as usize].clone())
        .collect::<Vec<_>>();
    assert_eq!(names.len(), 99);
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
        let short = name.strip_prefix("Mpk.Go.Practical.FiniteVector.").unwrap();
        inspect(&format!("false-{short}"), &encode(c), false);
    }
    for (d, name) in [
        (0, "Index"),
        (1, "Index"),
        (6, "Index"),
        (8, "Index"),
        (1, "Lookup"),
        (6, "Lookup"),
        (8, "Lookup"),
        (0, "Embed"),
        (6, "Embed"),
        (8, "Embed"),
        (8, "Capacity"),
    ] {
        let mut c = original.clone();
        let index = c
            .declarations
            .iter()
            .position(|item| {
                c.name_table[item.name as usize]
                    == format!("Mpk.Go.Practical.FiniteVector.D{d}.{name}")
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
        let value = changed_operation(&mut c, value, name, d);
        c.declarations[index].kind = DeclarationKind::Def {
            ty,
            value,
            reducibility,
        };
        let mut definitions = c.clone();
        definitions.declarations.truncate(index + 1);
        assert_eq!(
            mpk_kernel::check_declarations(&definitions)
                .unwrap()
                .declaration_count,
            index + 1
        );
        println!(
            "type-correct changed operation: D{d}.{name}; declarations={}",
            index + 1
        );
        inspect(&format!("body-D{d}.{name}"), &encode(c), false);
    }
    for d in 0..=8 {
        let mut premises = vec![
            ("LookupValid", 3),
            ("EmbedValid", 4),
            ("EmbedValid", 5),
            ("EmbedIndexValid", 3 + d),
        ];
        if d > 0 {
            premises.push(("LookupValid", 5));
        }
        for (name, premise) in premises {
            let mut c = original.clone();
            let index = c
                .declarations
                .iter()
                .position(|item| {
                    c.name_table[item.name as usize]
                        == format!("Mpk.Go.Practical.FiniteVector.D{d}.{name}")
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
            inspect(&format!("premise-D{d}.{name}-{premise}"), &encode(c), false);
        }
    }
}
