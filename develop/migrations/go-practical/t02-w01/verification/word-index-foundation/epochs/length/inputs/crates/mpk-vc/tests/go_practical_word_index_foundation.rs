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

fn baseline() -> Certificate {
    mpk_cert::decode_canonical_certificate(
        generate_go_practical_word_index_foundation()
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
                    let yes = constant(c, "Std.Bool.true");
                    assert_eq!(
                        c.term_table[arguments[2] as usize],
                        c.term_table[yes as usize]
                    );
                    arguments[2] = constant(c, "Std.Bool.false");
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
    let foundation = generate_go_practical_word_index_foundation().unwrap();
    inspect(
        "word-index-foundation",
        foundation.certificate_bytes(),
        true,
    );
    let c = mpk_cert::decode_canonical_certificate(foundation.certificate_bytes()).unwrap();
    let complete = dependencies::declarations(&c);
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
