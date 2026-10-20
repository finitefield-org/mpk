use mpk_cert::encode::{Certificate, DeclarationKind};
use mpk_vc::go_practical_ordinary::{
    generate_go_practical_string_foundation, generate_go_practical_string_validity_foundation,
    generate_go_practical_string_value, generate_go_practical_valid_string_value,
    import_go_practical_string_validity_foundation, import_go_practical_valid_string_value,
    GoOrdinaryError, GoStringValueError,
};
#[path = "support/go_word_division_dependencies.rs"]
mod dependencies;

fn retain(name: &str, extension: &str, raw: &[u8]) {
    if let Some(root) = std::env::var_os("MPK_GO_STRING_VALIDITY_OUTPUT") {
        use std::io::Write;
        let root = std::path::PathBuf::from(root);
        std::fs::create_dir_all(&root).unwrap();
        std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(root.join(format!("{name}.{extension}")))
            .unwrap()
            .write_all(raw)
            .unwrap();
    }
}
fn inspect(name: &str, bytes: &[u8], accepted: bool) {
    retain(name, "mpcert", bytes);
    inspect_saved(name, bytes, accepted);
}
fn inspect_saved(name: &str, bytes: &[u8], accepted: bool) {
    let result = mpk_kernel::verify_certificate_bytes_json_output(bytes);
    retain(name, "rust.json", format!("{}\n", result.json).as_bytes());
    if accepted && !result.accepted {
        let c = mpk_cert::decode_canonical_certificate(bytes).unwrap();
        let (mut lo, mut hi) = (1, c.declarations.len());
        while lo < hi {
            let end = (lo + hi) / 2;
            let mut prefix = c.clone();
            prefix.declarations.truncate(end);
            if mpk_kernel::check_declarations(&prefix).is_ok() {
                lo = end + 1;
            } else {
                hi = end;
            }
        }
        panic!(
            "{name}: first declaration type-check failure {}: {}",
            c.name_table[c.declarations[lo - 1].name as usize],
            result.json
        );
    }
    assert_eq!(result.accepted, accepted, "{name}: {}", result.json);
    if accepted {
        let v: serde_json::Value = serde_json::from_str(&result.json).unwrap();
        assert_eq!(v["axiom_count"], 0);
        assert!(v["axiom_report"]["entries"].as_array().unwrap().is_empty());
    } else {
        let v: serde_json::Value = serde_json::from_str(&result.json).unwrap();
        let d: serde_json::Value =
            serde_json::from_str(v["error_detail"].as_str().unwrap()).unwrap();
        assert_eq!(d["code"], "CORE_TYPE_MISMATCH");
    }
}

#[test]
fn binary_boundaries_preserve_the_entire_length_and_zero_tail() {
    for n in [1, 7, 8, 15, 16, 31, 32, 63, 64, 127, 128, 129, 254, 255] {
        let bytes = (0..n).map(|i| (i * 37 + 129) as u8).collect::<Vec<_>>();
        let value = generate_go_practical_valid_string_value(&bytes).unwrap();
        inspect(&format!("boundary-{n}"), value.certificate_bytes(), true);
        assert_eq!(value.native_bytes(), bytes);
    }
}

use mpk_cert::encode::{Declaration, TermNode};
fn term(c: &mut Certificate, node: TermNode) -> u32 {
    let at = c.term_table.len() as u32;
    c.term_table.push(node);
    at
}
fn sym(c: &mut Certificate, name: &str) -> u32 {
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
    let function = sym(c, name);
    term(
        c,
        TermNode::App {
            function,
            arguments,
        },
    )
}
fn flag(c: &mut Certificate, yes: bool) -> u32 {
    sym(
        c,
        if yes {
            "Std.Bool.true"
        } else {
            "Std.Bool.false"
        },
    )
}
fn encode(mut c: Certificate) -> Vec<u8> {
    let old = c.name_table.clone();
    c.name_table.sort();
    for d in &mut c.declarations {
        d.name = c.name_table.binary_search(&old[d.name as usize]).unwrap() as u32;
    }
    for l in &mut c.level_table {
        if let mpk_cert::encode::LevelNode::Param(name) = l {
            *name = c.name_table.binary_search(&old[*name as usize]).unwrap() as u32;
        }
    }
    assert!(c.declarations.len() <= 4096 && c.term_table.len() <= 131072);
    c.export_block = mpk_cert::build_export_block(&c).unwrap();
    c.axiom_report = mpk_cert::build_axiom_report(&c).unwrap();
    assert_eq!(c.axiom_report.summary.total_axiom_count, 0);
    c.hashes.export_hash = mpk_cert::export_block_hash(&c.export_block);
    c.hashes.axiom_report_hash = mpk_cert::axiom_report_hash_for_report(&c.axiom_report);
    mpk_cert::encode_certificate_bounded(&c, 2097152).unwrap()
}
fn bad_result(c: &mut Certificate, ty: u32) -> u32 {
    match c.term_table[ty as usize].clone() {
        TermNode::Pi { ty, body } => {
            let body = bad_result(c, body);
            term(c, TermNode::Pi { ty, body })
        }
        TermNode::App {
            function,
            mut arguments,
        } => {
            if let TermNode::Const { global, .. } = c.term_table[function as usize] {
                if c.name_table[c.declarations[global as usize].name as usize] == "Std.Eq"
                    && arguments.len() == 3
                {
                    if let TermNode::Const { global, .. } = c.term_table[arguments[0] as usize] {
                        if c.name_table[c.declarations[global as usize].name as usize] == "Std.Bool"
                        {
                            arguments[2] = call(c, "Mpk.Go.Practical.Bool.Not", vec![arguments[2]]);
                            return term(
                                c,
                                TermNode::App {
                                    function,
                                    arguments,
                                },
                            );
                        }
                    }
                }
            }
            let boolean = sym(c, "Std.Bool");
            let no = flag(c, false);
            let yes = flag(c, true);
            call(c, "Std.Eq", vec![boolean, no, yes])
        }
        _ => {
            let boolean = sym(c, "Std.Bool");
            let no = flag(c, false);
            let yes = flag(c, true);
            call(c, "Std.Eq", vec![boolean, no, yes])
        }
    }
}
fn erase(c: &mut Certificate, ty: u32, at: usize, domain: u32) -> u32 {
    let TermNode::Pi { ty: old, body } = c.term_table[ty as usize].clone() else {
        panic!("missing actual witness")
    };
    if at == 0 {
        term(c, TermNode::Pi { ty: domain, body })
    } else {
        let body = erase(c, body, at - 1, domain);
        term(c, TermNode::Pi { ty: old, body })
    }
}

#[test]
fn universal_comparisons_and_full_size_require_their_actual_witnesses() {
    let f = generate_go_practical_string_validity_foundation().unwrap();
    let base = decode(f.certificate_bytes());
    let names = base
        .declarations
        .iter()
        .filter_map(|d| {
            let name = &base.name_table[d.name as usize];
            (name.starts_with("Mpk.Go.Practical.String.Validity.")
                && matches!(d.kind, DeclarationKind::Theorem { .. }))
            .then_some(name.clone())
        })
        .collect::<Vec<_>>();
    assert_eq!(names.len(), 56);
    for name in &names {
        let mut c = base.clone();
        let at = c
            .declarations
            .iter()
            .position(|d| c.name_table[d.name as usize] == *name)
            .unwrap();
        let DeclarationKind::Theorem { ty, proof } = c.declarations[at].kind else {
            unreachable!()
        };
        let ty = bad_result(&mut c, ty);
        c.declarations[at].kind = DeclarationKind::Theorem { ty, proof };
        inspect(
            &format!("false-{}", name.rsplit('.').next().unwrap()),
            &encode(c),
            false,
        );
    }
    let mut conditions = vec![];
    for d in 1..=8 {
        for name in ["LessLow", "LessHigh"] {
            conditions.push((format!("{name}{d}"), 2 * (d - 1)));
        }
    }
    for name in [
        "SizeIndex",
        "ClearUpper",
        "FullUpper",
        "FullPadding",
        "FullValid",
    ] {
        conditions.push((name.into(), 1));
    }
    assert_eq!(conditions.len(), 21);
    for (name, position) in conditions {
        let mut c = base.clone();
        let full = format!("Mpk.Go.Practical.String.Validity.{name}");
        let at = c
            .declarations
            .iter()
            .position(|d| c.name_table[d.name as usize] == full)
            .unwrap();
        let DeclarationKind::Theorem { ty, proof } = c.declarations[at].kind else {
            unreachable!()
        };
        let boolean = sym(&mut c, "Std.Bool");
        let ty = erase(&mut c, ty, position, boolean);
        c.declarations[at].kind = DeclarationKind::Theorem { ty, proof };
        inspect(&format!("premise-{name}-{position}"), &encode(c), false);
    }
    println!(
        "whole string validity controls:56 actual altered conclusions,21 erased real witnesses"
    );
}

#[test]
fn exact_whole_validity_ownership_is_separate_from_core_acceptance() {
    let f = generate_go_practical_string_validity_foundation().unwrap();
    let mut c = decode(f.certificate_bytes());
    let ty = sym(&mut c, "Std.Bool");
    let value = flag(&mut c, false);
    let name = c.name_table.len() as u32;
    c.name_table
        .push("Mpk.Go.Practical.String.Validity.HonestExtra".into());
    c.declarations.push(Declaration {
        name,
        kind: DeclarationKind::Def {
            ty,
            value,
            reducibility: mpk_cert::encode::DefinitionReducibility::Reducible,
        },
    });
    let bytes = encode(c);
    inspect("validity-honest-extra", &bytes, true);
    assert_eq!(
        import_go_practical_string_validity_foundation(&bytes),
        Err(GoOrdinaryError::Linkage)
    );
    let value = generate_go_practical_valid_string_value(b"abc").unwrap();
    let mut c = decode(value.certificate_bytes());
    let ty = sym(&mut c, "Std.Bool");
    let value_node = flag(&mut c, true);
    let name = c.name_table.len() as u32;
    c.name_table
        .push("Mpk.Go.Practical.String.Validity.ValueHonestExtra".into());
    c.declarations.push(Declaration {
        name,
        kind: DeclarationKind::Def {
            ty,
            value: value_node,
            reducibility: mpk_cert::encode::DefinitionReducibility::Reducible,
        },
    });
    let bytes = encode(c);
    inspect("valid-value-honest-extra", &bytes, true);
    assert_eq!(
        import_go_practical_valid_string_value(&bytes, b"abc"),
        Err(GoStringValueError::Foundation(GoOrdinaryError::Linkage))
    );
    assert_eq!(
        import_go_practical_valid_string_value(value.certificate_bytes(), b"abd"),
        Err(GoStringValueError::Foundation(GoOrdinaryError::Linkage))
    );
}

// Remove the old slot-bit conclusions and their unused term DAG, so a padding
// control cannot be rejected merely by a previously asserted byte observation.
fn without_slot_observations(mut c: Certificate) -> Certificate {
    fn clone_term(
        old: &[TermNode],
        out: &mut Vec<TermNode>,
        ids: &mut std::collections::BTreeMap<u32, u32>,
        globals: &[Option<u32>],
        at: u32,
    ) -> u32 {
        if let Some(&id) = ids.get(&at) {
            return id;
        }
        let node = match old[at as usize].clone() {
            TermNode::Const { global, levels } => TermNode::Const {
                global: globals[global as usize]
                    .expect("retained proof depends on removed byte assertion"),
                levels,
            },
            TermNode::App {
                function,
                arguments,
            } => {
                let function = clone_term(old, out, ids, globals, function);
                let arguments = arguments
                    .into_iter()
                    .map(|a| clone_term(old, out, ids, globals, a))
                    .collect();
                TermNode::App {
                    function,
                    arguments,
                }
            }
            TermNode::Lam { ty, body } => {
                let ty = clone_term(old, out, ids, globals, ty);
                let body = clone_term(old, out, ids, globals, body);
                TermNode::Lam { ty, body }
            }
            TermNode::Pi { ty, body } => {
                let ty = clone_term(old, out, ids, globals, ty);
                let body = clone_term(old, out, ids, globals, body);
                TermNode::Pi { ty, body }
            }
            TermNode::Let { ty, value, body } => {
                let ty = clone_term(old, out, ids, globals, ty);
                let value = clone_term(old, out, ids, globals, value);
                let body = clone_term(old, out, ids, globals, body);
                TermNode::Let { ty, value, body }
            }
            other => other,
        };
        let id = out.len() as u32;
        out.push(node);
        ids.insert(at, id);
        id
    }
    let mut globals = vec![];
    let mut count = 0;
    for d in &c.declarations {
        if c.name_table[d.name as usize].starts_with("Mpk.Go.Practical.String.Value.Slot") {
            globals.push(None);
        } else {
            globals.push(Some(count));
            count += 1;
        }
    }
    let old = c.term_table.clone();
    let mut table = vec![];
    let mut ids = std::collections::BTreeMap::new();
    let mut declarations = vec![];
    for (i, mut d) in c.declarations.into_iter().enumerate() {
        if globals[i].is_none() {
            continue;
        }
        d.kind = match d.kind {
            DeclarationKind::Def {
                ty,
                value,
                reducibility,
            } => DeclarationKind::Def {
                ty: clone_term(&old, &mut table, &mut ids, &globals, ty),
                value: clone_term(&old, &mut table, &mut ids, &globals, value),
                reducibility,
            },
            DeclarationKind::Theorem { ty, proof } => DeclarationKind::Theorem {
                ty: clone_term(&old, &mut table, &mut ids, &globals, ty),
                proof: clone_term(&old, &mut table, &mut ids, &globals, proof),
            },
            DeclarationKind::Inductive { ty } => DeclarationKind::Inductive {
                ty: clone_term(&old, &mut table, &mut ids, &globals, ty),
            },
            DeclarationKind::Constructor {
                ty,
                inductive,
                generated,
            } => DeclarationKind::Constructor {
                ty: clone_term(&old, &mut table, &mut ids, &globals, ty),
                inductive: globals[inductive as usize].unwrap(),
                generated,
            },
            DeclarationKind::Recursor {
                ty,
                inductive,
                generated,
            } => DeclarationKind::Recursor {
                ty: clone_term(&old, &mut table, &mut ids, &globals, ty),
                inductive: globals[inductive as usize].unwrap(),
                generated,
            },
            _ => panic!("unexpected source/axiom declaration"),
        };
        declarations.push(d);
    }
    c.declarations = declarations;
    c.term_table = table;
    c
}
fn word_literal(c: &mut Certificate, number: u64) -> u32 {
    fn tree(c: &mut Certificate, start: usize, end: usize, depth: usize, number: u64) -> u32 {
        if start + 1 == end {
            return flag(c, number & (1 << start) != 0);
        }
        let mid = (start + end) / 2;
        let lower = tree(c, start, mid, depth + 1, number);
        let upper = tree(c, mid, end, depth + 1, number);
        let boolean = sym(c, "Std.Bool");
        let motive = term(
            c,
            TermNode::Lam {
                ty: boolean,
                body: boolean,
            },
        );
        let selector = term(c, TermNode::Var((5 - depth) as u32));
        call(c, "Std.Bool.cases", vec![motive, lower, upper, selector])
    }
    let boolean = sym(c, "Std.Bool");
    let mut body = tree(c, 0, 64, 0, number);
    for _ in 0..6 {
        body = term(c, TermNode::Lam { ty: boolean, body });
    }
    body
}

#[test]
fn malformed_whole_carriers_fail_after_old_byte_observations_are_removed() {
    let mut checks = vec![];
    for (label, n, new_length) in [
        ("padding-empty", 0, None),
        ("padding-partial", 3, None),
        ("length257", 3, Some(257u64)),
        ("length-high32", 3, Some((1u64 << 32) + 3)),
        ("length-high63", 3, Some((1u64 << 63) + 3)),
    ] {
        let value = generate_go_practical_valid_string_value(&vec![0; n]).unwrap();
        let mut c = decode(value.certificate_bytes());
        let at = c
            .declarations
            .iter()
            .position(|d| c.name_table[d.name as usize] == "Mpk.Go.Practical.String.Value")
            .unwrap();
        let DeclarationKind::Def {
            ty,
            value,
            reducibility,
        } = c.declarations[at].kind.clone()
        else {
            unreachable!()
        };
        let TermNode::App {
            function,
            mut arguments,
        } = c.term_table[value as usize].clone()
        else {
            unreachable!()
        };
        if let Some(length) = new_length {
            arguments[0] = word_literal(&mut c, length);
            for position in 0..64 {
                let full = format!("Mpk.Go.Practical.String.Value.LengthBit{position}");
                let at = c
                    .declarations
                    .iter()
                    .position(|d| c.name_table[d.name as usize] == full)
                    .unwrap();
                let DeclarationKind::Theorem { ty, .. } = c.declarations[at].kind else {
                    unreachable!()
                };
                let TermNode::App {
                    function,
                    mut arguments,
                } = c.term_table[ty as usize].clone()
                else {
                    unreachable!()
                };
                let actual = flag(&mut c, length & (1 << position) != 0);
                arguments[2] = actual;
                let ty = term(
                    &mut c,
                    TermNode::App {
                        function,
                        arguments,
                    },
                );
                let boolean = sym(&mut c, "Std.Bool");
                let proof = call(&mut c, "Std.Eq.refl", vec![boolean, actual]);
                c.declarations[at].kind = DeclarationKind::Theorem { ty, proof };
            }
        } else {
            let boolean = sym(&mut c, "Std.Bool");
            let mut body = flag(&mut c, true);
            for _ in 0..11 {
                body = term(&mut c, TermNode::Lam { ty: boolean, body });
            }
            arguments[1] = body;
        }
        let value = term(
            &mut c,
            TermNode::App {
                function,
                arguments,
            },
        );
        c.declarations[at].kind = DeclarationKind::Def {
            ty,
            value,
            reducibility,
        };
        let c = without_slot_observations(c);
        let bytes = encode(c.clone());
        let name = format!("carrier-{label}");
        retain(&name, "mpcert", &bytes);
        let first_new = c
            .declarations
            .iter()
            .position(|d| {
                c.name_table[d.name as usize] == "Mpk.Go.Practical.String.Validity.IndexZero0"
            })
            .unwrap();
        let mut prefix = c;
        prefix.declarations.truncate(first_new);
        let typed = mpk_kernel::check_declarations(&prefix).is_ok();
        assert!(
            typed,
            "actual length/byte carrier prefix not type-correct: {label}"
        );
        checks.push(serde_json::json!({"case":label,"all2048_slot_observation_proofs_removed":true,"actual_carrier_and64_length_bit_prefix_typechecks":typed,"full_certificate_acceptance_claimed":false}));
        inspect_saved(&name, &bytes, false);
    }
    retain(
        "carrier-controls",
        "prefix.json",
        format!("{}\n", serde_json::to_string_pretty(&checks).unwrap()).as_bytes(),
    );
}
fn decode(bytes: &[u8]) -> Certificate {
    mpk_cert::decode_canonical_certificate(bytes).unwrap()
}

#[test]
fn generic_validity_preserves_every_original_string_identity() {
    let new = generate_go_practical_string_validity_foundation().unwrap();
    inspect("validity-foundation", new.certificate_bytes(), true);
    assert_eq!(
        new,
        import_go_practical_string_validity_foundation(new.certificate_bytes()).unwrap()
    );
    let c = decode(new.certificate_bytes());
    let all = dependencies::declarations(&c);
    let old = generate_go_practical_string_foundation().unwrap();
    let golden=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../develop/migrations/go-practical/t02-w01/verification/string-foundation/epochs/native-controls/selected/candidates/string-foundation.mpcert");
    assert_eq!(old.certificate_bytes(), std::fs::read(golden).unwrap());
    for (name, value) in dependencies::declarations(&decode(old.certificate_bytes())) {
        assert_eq!(all.get(&name), Some(&value), "{name}");
    }
    println!(
        "string validity: declarations={} terms={} bytes={} all1053 old identities exact",
        c.declarations.len(),
        c.term_table.len(),
        new.certificate_bytes().len()
    );
}

#[test]
fn literal_empty_partial_and_full_capacity_have_actual_validity_witnesses() {
    for (name, bytes) in [
        ("empty", vec![]),
        ("partial", vec![0, 255, 128]),
        ("full", (0..=255u8).collect::<Vec<_>>()),
    ] {
        let value = generate_go_practical_valid_string_value(&bytes).unwrap();
        inspect(&format!("valid-{name}"), value.certificate_bytes(), true);
        assert_eq!(value.native_bytes(), bytes);
        let c = decode(value.certificate_bytes());
        let old = generate_go_practical_string_value(&bytes).unwrap();
        let all = dependencies::declarations(&c);
        for (name, term) in dependencies::declarations(&decode(old.certificate_bytes())) {
            assert_eq!(all.get(&name), Some(&term), "{name}");
        }
        let proof = c
            .declarations
            .iter()
            .find(|d| c.name_table[d.name as usize] == "Mpk.Go.Practical.String.Value.Valid")
            .unwrap();
        assert!(matches!(proof.kind, DeclarationKind::Theorem { .. }));
        assert_eq!(
            value,
            import_go_practical_valid_string_value(value.certificate_bytes(), &bytes).unwrap()
        );
        println!(
            "{name} valid: length={} declarations={} terms={} bytes={}",
            bytes.len(),
            c.declarations.len(),
            c.term_table.len(),
            value.certificate_bytes().len()
        );
    }
    assert_eq!(
        generate_go_practical_valid_string_value(&[0; 257]),
        Err(GoStringValueError::Domain { bytes: 257 })
    );
}
fn base64(value: &serde_json::Value) -> Vec<u8> {
    if value.is_null() {
        return vec![];
    }
    let text = value.as_str().unwrap().as_bytes();
    assert_eq!(text.len() % 4, 0);
    let mut output = Vec::new();
    let digit = |b| match b {
        b'A'..=b'Z' => (b - b'A') as u32,
        b'a'..=b'z' => (b - b'a' + 26) as u32,
        b'0'..=b'9' => (b - b'0' + 52) as u32,
        b'+' => 62,
        b'/' => 63,
        _ => panic!("invalid original base64"),
    };
    for (index, chunk) in text.chunks_exact(4).enumerate() {
        let padding = if chunk[2] == b'=' {
            assert_eq!(chunk[3], b'=');
            2
        } else if chunk[3] == b'=' {
            1
        } else {
            0
        };
        if padding > 0 {
            assert_eq!((index + 1) * 4, text.len());
        }
        let n = (digit(chunk[0]) << 18)
            | (digit(chunk[1]) << 12)
            | if padding == 2 {
                0
            } else {
                digit(chunk[2]) << 6
            }
            | if padding > 0 { 0 } else { digit(chunk[3]) };
        output.push((n >> 16) as u8);
        if padding < 2 {
            output.push((n >> 8) as u8);
        }
        if padding == 0 {
            output.push(n as u8);
        }
    }
    output
}

#[test]
fn actual_owned_go_constants_have_whole_validity_without_replacing_their_bits() {
    use sha2::{Digest, Sha256};
    let root=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../develop/migrations/go-practical/t02-w01/verification/string-constants/input-review.json");
    let receipt: serde_json::Value = serde_json::from_slice(&std::fs::read(root).unwrap()).unwrap();
    let mut total = 0;
    let mut rejected = 0;
    for row in receipt["original_evidence_files"].as_array().unwrap() {
        let path = std::path::Path::new(row["path"].as_str().unwrap());
        let name = path.file_name().unwrap().to_str().unwrap();
        let full = path.to_str().unwrap();
        if !name.ends_with("-native.json")
            || !(full.contains("-complete-outputs-selected-evidence/")
                || full.contains("-complete-outputs-replay/evidence/"))
        {
            continue;
        }
        let raw = std::fs::read(path).unwrap();
        assert_eq!(
            format!("{:x}", Sha256::digest(&raw)),
            row["sha256"].as_str().unwrap()
        );
        assert_eq!(raw.len() as u64, row["bytes"].as_u64().unwrap());
        let name = format!("native-{}", path.file_stem().unwrap().to_str().unwrap());
        retain(&name, "native.json", &raw);
        let original: serde_json::Value = serde_json::from_slice(&raw).unwrap();
        let bytes = base64(&original["native_bytes"]);
        let source = base64(&original["original_source"]);
        let expression = base64(&original["original_expression"]);
        assert_eq!(
            format!("{:x}", Sha256::digest(&source)),
            original["source_object"]["raw_sha256"].as_str().unwrap()
        );
        assert_eq!(
            source.len() as u64,
            original["source_object"]["bytes"].as_u64().unwrap()
        );
        let start = original["span"]["start"].as_u64().unwrap() as usize;
        let end = original["span"]["end"].as_u64().unwrap() as usize;
        assert_eq!(&source[start..end], expression);
        assert_eq!(
            original["span"]["source_id"],
            original["source_object"]["id"]
        );
        assert_eq!(original["source_object"]["kind"], "go_file");
        assert_eq!(
            original["ordinary_source_evaluation_or_nominal_identity_proof"],
            false
        );
        assert_eq!(original["registered_context_or_program_proof"], false);
        total += 1;
        if bytes.len() > 256 {
            assert_eq!(bytes.len(), 257);
            assert_eq!(
                generate_go_practical_valid_string_value(&bytes),
                Err(GoStringValueError::Domain { bytes: 257 })
            );
            retain(
                &name,
                "domain.json",
                b"{\"bytes\":257,\"modeled_domain_error\":true,\"semantic_falsity\":false}\n",
            );
            rejected += 1;
            continue;
        }
        let value = generate_go_practical_valid_string_value(&bytes).unwrap();
        assert_eq!(value.native_bytes(), bytes);
        inspect(&name, value.certificate_bytes(), true);
        let c = mpk_cert::decode_canonical_certificate(value.certificate_bytes()).unwrap();
        let mut bits = 0;
        for d in &c.declarations {
            let name = &c.name_table[d.name as usize];
            let Some(suffix) = name.strip_prefix("Mpk.Go.Practical.String.Value.") else {
                continue;
            };
            if !suffix.starts_with("LengthBit") && !suffix.starts_with("Slot") {
                continue;
            }
            let expected = if let Some(position) = suffix.strip_prefix("LengthBit") {
                ((bytes.len() as u64) >> position.parse::<usize>().unwrap()) & 1 != 0
            } else {
                let (slot, bit) = suffix
                    .strip_prefix("Slot")
                    .unwrap()
                    .split_once("Bit")
                    .unwrap();
                (bytes
                    .get(slot.parse::<usize>().unwrap())
                    .copied()
                    .unwrap_or(0)
                    >> bit.parse::<usize>().unwrap())
                    & 1
                    != 0
            };
            let DeclarationKind::Theorem { ty, .. } = d.kind else {
                panic!("bit was not an actual ordinary theorem")
            };
            let mpk_cert::encode::TermNode::App { arguments, .. } = &c.term_table[ty as usize]
            else {
                panic!("missing exact bit equality")
            };
            assert_eq!(arguments.len(), 3);
            let mpk_cert::encode::TermNode::Const { global, .. } =
                c.term_table[arguments[2] as usize]
            else {
                panic!("missing atomic original bit")
            };
            assert_eq!(
                c.name_table[c.declarations[global as usize].name as usize],
                if expected {
                    "Std.Bool.true"
                } else {
                    "Std.Bool.false"
                }
            );
            bits += 1;
        }
        assert_eq!(bits, 2112);
        assert!(c.source_manifest.is_none() && c.imports.is_empty());
    }
    assert_eq!(total, 19);
    assert_eq!(rejected, 1);
    println!("actual original Go data:19 complete source observations,18 full2112-bit ordinary certificates,one distinct257-byte domain error;whole carrier validity proved; no source evaluation or nominal/context proof");
}
