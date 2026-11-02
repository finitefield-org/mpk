use mpk_cert::encode::{Certificate, Declaration, DeclarationKind, TermNode};
use mpk_vc::go_practical_ordinary::{
    generate_go_practical_finite_vector_foundation, generate_go_practical_word_index_foundation,
    generate_go_practical_word_shift_foundation, import_go_practical_word_index_foundation,
    GoOrdinaryError,
};
#[path = "support/go_word_division_dependencies.rs"]
mod dependencies;

fn inspect(name: &str, bytes: &[u8], accepted: bool) {
    let result = mpk_kernel::verify_certificate_bytes_json_output(bytes);
    if let Some(path) = std::env::var_os("MPK_GO_WORD_INDEX_FOUNDATION_OUTPUT") {
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
        let (mut lower, mut upper) = (1, certificate.declarations.len());
        while lower < upper {
            let end = (lower + upper) / 2;
            let mut prefix = certificate.clone();
            prefix.declarations.truncate(end);
            if mpk_kernel::check_declarations(&prefix).is_ok() {
                lower = end + 1;
            } else {
                upper = end;
            }
        }
        let declaration = &certificate.declarations[lower - 1];
        panic!(
            "{name}: first declaration type-check failure {}: full verdict {}",
            certificate.name_table[declaration.name as usize], result.json
        );
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

fn foundation() -> &'static mpk_vc::go_practical_ordinary::GoWordIndexFoundation {
    static VALUE: std::sync::OnceLock<mpk_vc::go_practical_ordinary::GoWordIndexFoundation> =
        std::sync::OnceLock::new();
    VALUE.get_or_init(|| generate_go_practical_word_index_foundation().unwrap())
}
fn baseline() -> Certificate {
    mpk_cert::decode_canonical_certificate(foundation().certificate_bytes()).unwrap()
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
        TermNode::App {
            function,
            mut arguments,
        } => {
            let TermNode::Const { global, .. } = c.term_table[function as usize] else {
                return result;
            };
            let name = &c.name_table[c.declarations[global as usize].name as usize];
            if name == "Std.Eq" {
                let TermNode::Const { global, .. } = c.term_table[arguments[0] as usize] else {
                    return result;
                };
                if c.name_table[c.declarations[global as usize].name as usize] == "Std.Bool" {
                    // Reverse the actual range claim, retaining its entire
                    // computed left side. A costly numerical normalization
                    // must not stand in for rejection of the wrong truth value.
                    let old = arguments[2];
                    let no = constant(c, "Std.Bool.false");
                    let yes = constant(c, "Std.Bool.true");
                    arguments[2] = if c.term_table[old as usize] == c.term_table[yes as usize] {
                        no
                    } else if c.term_table[old as usize] == c.term_table[no as usize] {
                        yes
                    } else {
                        call(c, "Mpk.Go.Practical.Bool.Not", vec![old])
                    };
                    return term(
                        c,
                        TermNode::App {
                            function,
                            arguments,
                        },
                    );
                }
            } else if name == "Mpk.Go.Practical.Sequence.IndexValid" {
                let length = call(
                    c,
                    "Mpk.Go.Practical.Sequence.Length",
                    arguments[..2].to_vec(),
                );
                let less = call(c, "Mpk.Go.Practical.Nat.Less", vec![arguments[2], length]);
                let boolean = constant(c, "Std.Bool");
                let no = constant(c, "Std.Bool.false");
                return call(c, "Std.Eq", vec![boolean, less, no]);
            }
            result
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
fn complete_full64_index_component_preserves_original_inputs() {
    let foundation = foundation();
    inspect(
        "word-index-foundation",
        foundation.certificate_bytes(),
        true,
    );
    let c = mpk_cert::decode_canonical_certificate(foundation.certificate_bytes()).unwrap();
    let complete = dependencies::declarations(&c);
    let first=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../develop/migrations/go-practical/t02-w01/verification/word-index-foundation/epochs/initial/selected/candidates/word-index-foundation.mpcert");
    let first = mpk_cert::decode_canonical_certificate(&std::fs::read(first).unwrap()).unwrap();
    for (name, value) in dependencies::declarations(&first) {
        assert_eq!(complete.get(&name), Some(&value), "{name}");
    }

    let finite = generate_go_practical_finite_vector_foundation().unwrap();
    let finite = mpk_cert::decode_canonical_certificate(finite.certificate_bytes()).unwrap();
    for (name, value) in dependencies::declarations(&finite) {
        assert_eq!(complete.get(&name), Some(&value), "{name}");
    }
    let shift = generate_go_practical_word_shift_foundation().unwrap();
    let original=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../develop/migrations/go-practical/t02-w01/verification/word-shift-count-bounds/final/rust-tests/certificates/word-shift-counts.mpcert");
    assert!(
        shift.certificate_bytes() == std::fs::read(original).unwrap(),
        "the complete original shift certificate changed"
    );
    let shift = mpk_cert::decode_canonical_certificate(shift.certificate_bytes()).unwrap();
    let old = dependencies::declarations(&shift);
    let mut retained = 0;
    for (name, value) in &complete {
        if let Some(previous) = old.get(name) {
            assert_eq!(previous, value, "{name}");
            retained += 1;
        }
    }
    assert!(retained > 100);
    for d in 0..=8 {
        for name in ["BelowCapacity", "BoundedIndex", "BoundedRead", "FullLength"] {
            assert!(complete.contains_key(&format!("Mpk.Go.Practical.Word.Index.D{d}.{name}")));
        }
    }
    assert_eq!(c.axiom_report.summary.total_axiom_count, 0);
    assert!(c.imports.is_empty());
    println!(
        "full64 indices: declarations={} terms={} bytes={} old shift identities={} zero axioms",
        c.declarations.len(),
        c.term_table.len(),
        foundation.certificate_bytes().len(),
        retained
    );
}

#[test]
fn exact_full64_index_ownership_is_separate_from_core_acceptance() {
    let value = foundation();
    assert_eq!(
        value,
        &import_go_practical_word_index_foundation(value.certificate_bytes()).unwrap()
    );
    assert_eq!(
        value,
        &generate_go_practical_word_index_foundation().unwrap()
    );
    let mut c = baseline();
    let ty = constant(&mut c, "Std.Bool");
    let body = constant(&mut c, "Std.Bool.false");
    let name = c.name_table.len() as u32;
    c.name_table
        .push("Mpk.Go.Practical.Word.Index.HonestExtra".into());
    c.declarations.push(Declaration {
        name,
        kind: DeclarationKind::Def {
            ty,
            value: body,
            reducibility: mpk_cert::encode::DefinitionReducibility::Reducible,
        },
    });
    let bytes = encode(c);
    inspect("word-index-honest-extra", &bytes, true);
    assert_eq!(
        import_go_practical_word_index_foundation(&bytes),
        Err(GoOrdinaryError::Linkage)
    );
    let mut trailing = value.certificate_bytes().to_vec();
    trailing.push(0);
    assert_eq!(
        import_go_practical_word_index_foundation(&trailing),
        Err(GoOrdinaryError::Linkage)
    );
    assert_eq!(
        import_go_practical_word_index_foundation(&vec![0; 2_097_153]),
        Err(GoOrdinaryError::Limit)
    );
}

fn wrong_operation(c: &mut Certificate, node: u32, name: &str, d: usize) -> u32 {
    match c.term_table[node as usize].clone() {
        TermNode::Lam { ty, body } => {
            let body = wrong_operation(c, body, name, d);
            term(c, TermNode::Lam { ty, body })
        }
        _ => match name {
            "Value6" => {
                let word = term(c, TermNode::Var(0));
                let no = constant(c, "Std.Bool.false");
                let low = term(
                    c,
                    TermNode::App {
                        function: word,
                        arguments: vec![no],
                    },
                );
                call(c, "Mpk.Go.Practical.Word.Numeric.Value5", vec![low])
            }
            "CountHigh" => constant(c, "Std.Bool.false"),
            "Digits8" => {
                let word = term(c, TermNode::Var(0));
                call(c, "Mpk.Go.Practical.Word.Numeric.W64.Digits7", vec![word])
            }
            "Read64" | "Sequence64" => {
                let a = term(c, TermNode::Var(2));
                let data = term(c, TermNode::Var(1));
                let index = if name == "Sequence64" && d == 8 {
                    let word = term(c, TermNode::Var(0));
                    call(c, "Mpk.Go.Practical.Word.Numeric.W64.Digits8", vec![word])
                } else {
                    constant(c, "Std.Nat.zero")
                };
                call(
                    c,
                    &format!(
                        "Mpk.Go.Practical.FiniteVector.D{d}.{}",
                        if name == "Read64" { "Lookup" } else { "Embed" }
                    ),
                    if name == "Read64" {
                        vec![a, data, index]
                    } else {
                        vec![a, index, data]
                    },
                )
            }
            _ => unreachable!(),
        },
    }
}

#[test]
fn word_index_laws_reject_forged_claims_erased_bounds_and_wrong_operations() {
    let original = baseline();
    let names = original
        .declarations
        .iter()
        .filter(|d| {
            matches!(d.kind, DeclarationKind::Theorem { .. })
                && original.name_table[d.name as usize].starts_with("Mpk.Go.Practical.Word.Index.")
        })
        .map(|d| original.name_table[d.name as usize].clone())
        .collect::<Vec<_>>();
    assert_eq!(names.len(), 166);
    let mut conclusions = names;
    for d in [0, 1, 2, 7, 8] {
        for name in ["CountBalance", "CountHighZero"] {
            conclusions.push(format!("Mpk.Go.Practical.Word.Shifts.{name}{d}"));
        }
    }
    for d in 7..=8 {
        for name in ["CountLowBalance", "CountLowBound"] {
            conclusions.push(format!("Mpk.Go.Practical.Word.Shifts.{name}{d}"));
        }
    }
    assert_eq!(conclusions.len(), 180);
    for name in conclusions {
        let mut c = original.clone();
        let at = c
            .declarations
            .iter()
            .position(|d| c.name_table[d.name as usize] == name)
            .unwrap();
        let DeclarationKind::Theorem { ty, proof } = c.declarations[at].kind.clone() else {
            unreachable!()
        };
        let false_claim = false_claim(&mut c);
        let ty = replace_result(&mut c, ty, false_claim);
        c.declarations[at].kind = DeclarationKind::Theorem { ty, proof };
        let short = name.strip_prefix("Mpk.Go.Practical.Word.").unwrap();
        inspect(&format!("false-{short}"), &encode(c), false);
    }
    let mut premises = vec![
        ("NotTrueReflect".into(), 1),
        ("UpperBound".into(), 3),
        ("UpperBound".into(), 4),
    ];
    for d in 0..=8 {
        for (name, at) in [
            ("SmallValue", 1),
            ("LargeValue", 1),
            ("HighClear", 1),
            ("BoundedValue", 1),
            ("BoundedIndex", 1),
            ("BoundedRead", 3),
            ("FullLengthFits", 1),
            ("WithinLengthValid", 4),
            ("ReadWithinLength", 4),
            ("ReadWithinLength", 5),
            ("SequenceWithinSlot", 4),
            ("SequenceWithinSlot", 5),
            ("FullSequenceValid", 4),
            ("FullSequenceValid", 5),
        ] {
            premises.push((format!("D{d}.{name}"), at));
        }
    }
    assert_eq!(premises.len(), 129);
    for (name, premise) in premises {
        let mut c = original.clone();
        let at = c
            .declarations
            .iter()
            .position(|d| {
                c.name_table[d.name as usize] == format!("Mpk.Go.Practical.Word.Index.{name}")
            })
            .unwrap();
        let DeclarationKind::Theorem { ty, proof } = c.declarations[at].kind.clone() else {
            unreachable!()
        };
        let boolean = constant(&mut c, "Std.Bool");
        let yes = constant(&mut c, "Std.Bool.true");
        let unrelated = call(&mut c, "Std.Eq", vec![boolean, yes, yes]);
        let ty = replace_domain(&mut c, ty, premise, unrelated);
        c.declarations[at].kind = DeclarationKind::Theorem { ty, proof };
        inspect(&format!("premise-{name}-{premise}"), &encode(c), false);
    }
    for (d, operation, name) in [
        (6, "Value6", "Mpk.Go.Practical.Word.Numeric.Value6".into()),
        (
            8,
            "Digits8",
            "Mpk.Go.Practical.Word.Numeric.W64.Digits8".into(),
        ),
        (
            0,
            "CountHigh",
            "Mpk.Go.Practical.Word.Shifts.CountHigh0".into(),
        ),
        (
            3,
            "CountHigh",
            "Mpk.Go.Practical.Word.Shifts.CountHigh3".into(),
        ),
        (
            8,
            "CountHigh",
            "Mpk.Go.Practical.Word.Shifts.CountHigh8".into(),
        ),
        (3, "Read64", "Mpk.Go.Practical.Word.Index.D3.Read64".into()),
        (8, "Read64", "Mpk.Go.Practical.Word.Index.D8.Read64".into()),
        (
            0,
            "Sequence64",
            "Mpk.Go.Practical.Word.Index.D0.Sequence64".into(),
        ),
        (
            3,
            "Sequence64",
            "Mpk.Go.Practical.Word.Index.D3.Sequence64".into(),
        ),
        (
            8,
            "Sequence64",
            "Mpk.Go.Practical.Word.Index.D8.Sequence64".into(),
        ),
    ] as [(usize, &str, String); 10]
    {
        let mut c = original.clone();
        let at = c
            .declarations
            .iter()
            .position(|item| c.name_table[item.name as usize] == name)
            .unwrap();
        let DeclarationKind::Def {
            ty,
            value,
            reducibility,
        } = c.declarations[at].kind.clone()
        else {
            unreachable!()
        };
        let value = wrong_operation(&mut c, value, operation, d);
        c.declarations[at].kind = DeclarationKind::Def {
            ty,
            value,
            reducibility,
        };
        let mut definitions = c.clone();
        definitions.declarations.truncate(at + 1);
        assert_eq!(
            mpk_kernel::check_declarations(&definitions)
                .unwrap()
                .declaration_count,
            at + 1
        );
        println!(
            "type-correct changed operation: {name}; declarations={}",
            at + 1
        );
        inspect(&format!("body-D{d}.{operation}"), &encode(c), false);
    }
}

fn literal_word(c: &mut Certificate, value: u64) -> u32 {
    let boolean = constant(c, "Std.Bool");
    let no = constant(c, "Std.Bool.false");
    let yes = constant(c, "Std.Bool.true");
    fn tree(
        c: &mut Certificate,
        values: &[bool],
        depth: usize,
        boolean: u32,
        no: u32,
        yes: u32,
    ) -> u32 {
        if values.iter().all(|v| *v == values[0]) {
            return if values[0] { yes } else { no };
        }
        let mid = values.len() / 2;
        let lower = tree(c, &values[..mid], depth + 1, boolean, no, yes);
        let upper = tree(c, &values[mid..], depth + 1, boolean, no, yes);
        let selector = term(c, TermNode::Var((5 - depth) as u32));
        let motive = term(
            c,
            TermNode::Lam {
                ty: boolean,
                body: boolean,
            },
        );
        call(c, "Std.Bool.cases", vec![motive, lower, upper, selector])
    }
    let bits = (0..64)
        .map(|i| value & (1u64 << i) != 0)
        .collect::<Vec<_>>();
    let mut body = tree(c, &bits, 0, boolean, no, yes);
    for _ in 0..6 {
        body = term(c, TermNode::Lam { ty: boolean, body });
    }
    body
}

#[test]
fn actual_full64_guard_boundaries_keep_high32_and_high63() {
    let original = baseline();
    let mut cases = vec![];
    for d in 0..=8 {
        cases.push((d, (1u64 << d) - 1, true, "maximum"));
        cases.push((d, 1u64 << d, false, "boundary"));
    }
    for d in [3, 6, 8] {
        cases.push((d, 1u64 << 32, false, "high32"));
    }
    for d in [0, 3, 6, 8] {
        cases.push((d, 1u64 << 63, false, "high63"));
    }
    assert_eq!(cases.len(), 25);
    for (d, value, accepted, label) in cases {
        let mut c = original.clone();
        let word = literal_word(&mut c, value);
        let natural = call(&mut c, "Mpk.Go.Practical.Word.Numeric.Value6", vec![word]);
        let capacity = constant(
            &mut c,
            &format!("Mpk.Go.Practical.FiniteVector.D{d}.Capacity"),
        );
        let less = call(&mut c, "Mpk.Go.Practical.Nat.Less", vec![natural, capacity]);
        let flag = call(
            &mut c,
            &format!("Mpk.Go.Practical.Word.Shifts.CountHigh{d}"),
            vec![word],
        );
        let not = call(&mut c, "Mpk.Go.Practical.Bool.Not", vec![flag]);
        let boolean = constant(&mut c, "Std.Bool");
        let yes = constant(&mut c, "Std.Bool.true");
        let no = constant(&mut c, "Std.Bool.false");
        let inferred = if accepted { yes } else { no };
        let below = call(
            &mut c,
            &format!("Mpk.Go.Practical.Word.Index.D{d}.BelowCapacity"),
            vec![word],
        );
        let literal = call(&mut c, "Std.Eq.refl", vec![boolean, inferred]);
        let proof = call(
            &mut c,
            "Std.Eq.trans",
            vec![boolean, less, not, inferred, below, literal],
        );
        let ty = call(&mut c, "Std.Eq", vec![boolean, less, yes]);
        let name = c.name_table.len() as u32;
        c.name_table.push(format!(
            "Mpk.Go.Practical.Word.Index.TestGuard.D{d}.{label}"
        ));
        c.declarations.push(Declaration {
            name,
            kind: DeclarationKind::Theorem { ty, proof },
        });
        inspect(&format!("guard-D{d}.{label}"), &encode(c), accepted);
    }
}
