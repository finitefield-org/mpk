use mpk_cert::encode::{Certificate, DeclarationKind};
use mpk_vc::go_practical_ordinary::{
    generate_go_practical_string_normalization_foundation,
    generate_go_practical_string_validity_foundation,
    import_go_practical_string_normalization_foundation, GoOrdinaryError,
};
#[path = "support/go_word_division_dependencies.rs"]
mod dependencies;
fn retain(name: &str, extension: &str, raw: &[u8]) {
    if let Some(root) = std::env::var_os("MPK_GO_STRING_NORMALIZATION_OUTPUT") {
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

fn foundation() -> &'static mpk_vc::go_practical_ordinary::GoStringNormalizationFoundation {
    static VALUE: std::sync::OnceLock<
        mpk_vc::go_practical_ordinary::GoStringNormalizationFoundation,
    > = std::sync::OnceLock::new();
    VALUE.get_or_init(|| generate_go_practical_string_normalization_foundation().unwrap())
}
fn baseline() -> Certificate {
    mpk_cert::decode_canonical_certificate(foundation().certificate_bytes()).unwrap()
}
#[test]
fn generic_normalization_preserves_all1109_original_identities() {
    let f = foundation();
    inspect("normalization-foundation", f.certificate_bytes(), true);
    assert_eq!(
        *f,
        import_go_practical_string_normalization_foundation(f.certificate_bytes()).unwrap()
    );
    let c = baseline();
    let all = dependencies::declarations(&c);
    let old = generate_go_practical_string_validity_foundation().unwrap();
    let golden=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../develop/migrations/go-practical/t02-w01/verification/string-validity/epochs/native-controls/selected/candidates/validity-foundation.mpcert");
    assert_eq!(old.certificate_bytes(), std::fs::read(golden).unwrap());
    let old = mpk_cert::decode_canonical_certificate(old.certificate_bytes()).unwrap();
    assert_eq!(old.declarations.len(), 1109);
    for (name, value) in dependencies::declarations(&old) {
        assert_eq!(all.get(&name), Some(&value), "{name}");
    }
    assert_eq!(c.declarations.len(), 1143);
    println!("string normalization: declarations={} terms={} bytes={} all1109 old identities exact;6 definitions28 laws",c.declarations.len(),c.term_table.len(),f.certificate_bytes().len());
}
#[test]
fn exact_ownership_is_distinct_from_core_accepted_extensions() {
    let mut c = baseline();
    let ty = sym(&mut c, "Std.Bool");
    let value = flag(&mut c, true);
    let name = c.name_table.len() as u32;
    c.name_table
        .push("Mpk.Go.Practical.String.Normalization.HonestExtra".into());
    c.declarations.push(Declaration {
        name,
        kind: DeclarationKind::Def {
            ty,
            value,
            reducibility: mpk_cert::encode::DefinitionReducibility::Reducible,
        },
    });
    let raw = encode(c);
    inspect("normalization-honest-extra", &raw, true);
    assert_eq!(
        import_go_practical_string_normalization_foundation(&raw),
        Err(GoOrdinaryError::Linkage)
    );
    assert_eq!(
        import_go_practical_string_normalization_foundation(&vec![0; 2097153]),
        Err(GoOrdinaryError::Limit)
    );
}
#[test]
fn universal_normalization_laws_require_real_conclusions_and_witnesses() {
    let base = baseline();
    let names = base
        .declarations
        .iter()
        .filter_map(|d| {
            let name = &base.name_table[d.name as usize];
            (name.starts_with("Mpk.Go.Practical.String.Normalization.")
                && matches!(d.kind, DeclarationKind::Theorem { .. }))
            .then_some(name.clone())
        })
        .collect::<Vec<_>>();
    assert_eq!(names.len(), 28);
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
    let conditions = [
        ("EquivalentLength", 2),
        ("EquivalentContents", 2),
        ("EquivalentSlot", 2),
        ("EquivalentSlot", 11),
        ("EquivalentSym", 2),
        ("EquivalentTrans", 3),
        ("EquivalentTrans", 4),
        ("EquivalentUpper", 2),
        ("EquivalentUpper", 3),
        ("EquivalentWithin", 2),
        ("EquivalentWithin", 4),
        ("EquivalentRead", 2),
        ("EquivalentRead", 4),
        ("EquivalentRead", 5),
        ("Inside", 9),
        ("Outside", 9),
        ("Upper", 1),
        ("Valid", 1),
        ("Read", 2),
        ("Read", 3),
        ("SequenceRead", 2),
        ("SequenceRead", 3),
        ("StableSlot", 1),
        ("EmptySlot", 1),
        ("FullSlot", 1),
        ("EquivalentNormalized", 2),
        ("InsideBit", 9),
        ("OutsideBit", 9),
    ];
    assert_eq!(conditions.len(), 28);
    for (name, position) in conditions {
        let mut c = base.clone();
        let full = format!("Mpk.Go.Practical.String.Normalization.{name}");
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
    println!("normalization controls:28 altered actual conclusions,28 erased real witnesses");
}

fn replace_leaf(
    c: &mut Certificate,
    at: u32,
    depth: usize,
    change: &mut impl FnMut(&mut Certificate, u32) -> u32,
) -> u32 {
    if depth == 0 {
        return change(c, at);
    }
    let TermNode::Lam { ty, body } = c.term_table[at as usize].clone() else {
        panic!("complete input/address lambdas")
    };
    let body = replace_leaf(c, body, depth - 1, change);
    term(c, TermNode::Lam { ty, body })
}

#[test]
fn wrong_normalizers_are_type_correct_before_their_real_laws_reject() {
    let mut checks = vec![];
    for mode in [
        "keep-tail",
        "clear-prefix",
        "reverse-guard",
        "equal-boundary",
        "identity",
        "zero-length",
        "zero-value",
    ] {
        let mut c = baseline();
        let definition = if matches!(
            mode,
            "keep-tail" | "clear-prefix" | "reverse-guard" | "equal-boundary"
        ) {
            "Data"
        } else {
            "Value"
        };
        let name = format!("Mpk.Go.Practical.String.Normalization.{definition}");
        let at = c
            .declarations
            .iter()
            .position(|d| c.name_table[d.name as usize] == name)
            .unwrap();
        let DeclarationKind::Def {
            ty,
            value,
            reducibility,
        } = c.declarations[at].kind
        else {
            unreachable!()
        };
        let value = match mode {
            "keep-tail" => replace_leaf(&mut c, value, 1, &mut |c, _| {
                let s = term(c, TermNode::Var(0));
                call(c, "Mpk.Go.Practical.String.Data", vec![s])
            }),
            "clear-prefix" => replace_leaf(&mut c, value, 1, &mut |c, _| {
                let boolean = sym(c, "Std.Bool");
                let mut body = flag(c, false);
                for _ in 0..11 {
                    body = term(c, TermNode::Lam { ty: boolean, body });
                }
                body
            }),
            "reverse-guard" | "equal-boundary" => replace_leaf(&mut c, value, 9, &mut |c, leaf| {
                let TermNode::App {
                    function,
                    mut arguments,
                } = c.term_table[leaf as usize].clone()
                else {
                    panic!("actual byte choice")
                };
                assert_eq!(arguments.len(), 4);
                if mode == "reverse-guard" {
                    arguments.swap(1, 2);
                } else {
                    let TermNode::App {
                        arguments: indices, ..
                    } = c.term_table[arguments[3] as usize].clone()
                    else {
                        panic!("actual full-size comparison")
                    };
                    assert_eq!(indices.len(), 2);
                    arguments[3] = call(c, "Mpk.Go.Practical.Nat.Equal", indices);
                }
                term(
                    c,
                    TermNode::App {
                        function,
                        arguments,
                    },
                )
            }),
            "identity" => replace_leaf(&mut c, value, 1, &mut |c, _| term(c, TermNode::Var(0))),
            "zero-length" => replace_leaf(&mut c, value, 1, &mut |c, leaf| {
                let TermNode::App {
                    function,
                    mut arguments,
                } = c.term_table[leaf as usize].clone()
                else {
                    panic!("actual constructor")
                };
                assert_eq!(arguments.len(), 2);
                let boolean = sym(c, "Std.Bool");
                let mut body = flag(c, false);
                for _ in 0..6 {
                    body = term(c, TermNode::Lam { ty: boolean, body });
                }
                arguments[0] = body;
                term(
                    c,
                    TermNode::App {
                        function,
                        arguments,
                    },
                )
            }),
            "zero-value" => replace_leaf(&mut c, value, 1, &mut |c, _| {
                sym(c, "Mpk.Go.Practical.String.Zero")
            }),
            _ => unreachable!(),
        };
        c.declarations[at].kind = DeclarationKind::Def {
            ty,
            value,
            reducibility,
        };
        let raw = encode(c.clone());
        let label = format!("operation-{mode}");
        retain(&label, "mpcert", &raw);
        let mut prefix = c;
        let end = prefix
            .declarations
            .iter()
            .position(|d| {
                prefix.name_table[d.name as usize]
                    == "Mpk.Go.Practical.String.Normalization.EquivalentLength"
            })
            .unwrap();
        prefix.declarations.truncate(end);
        let typed = mpk_kernel::check_declarations(&prefix).is_ok();
        assert!(typed, "wrong operation prefix: {mode}");
        checks.push(serde_json::json!({"case":mode,"actual_definition_prefix_typechecks":typed,"full_certificate_acceptance_claimed":false}));
        inspect_saved(&label, &raw, false);
    }
    retain(
        "definition-controls",
        "prefix.json",
        format!("{}\n", serde_json::to_string_pretty(&checks).unwrap()).as_bytes(),
    );
}

#[path = "support/go_string_normalization_eval.rs"]
mod normalization_eval;

#[test]
fn actual_definitions_preserve_all_bits_and_clear_arbitrary_nonzero_tails() {
    let c = baseline();
    let mut cases = 0;
    let mut total_steps = 0;
    for n in [
        0, 1, 2, 3, 7, 8, 15, 16, 31, 32, 63, 64, 127, 128, 129, 254, 255, 256,
    ] {
        for pattern in 0..3 {
            let raw: [u8; 256] = std::array::from_fn(|i| match pattern {
                0 => 255,
                1 => i as u8,
                _ => (i * 37 + 129) as u8,
            });
            let label = format!("raw-{n}-{pattern}");
            let input = serde_json::json!({"full_length_u64":n,"all256_raw_bytes":raw.to_vec(),"source_or_nominal_authority":false});
            retain(
                &label,
                "input.json",
                format!("{}\n", serde_json::to_string_pretty(&input).unwrap()).as_bytes(),
            );
            let mut model = normalization_eval::Interpreter::new(&c);
            let value = model.raw(n as u64, raw);
            let normalized = model.invoke("Mpk.Go.Practical.String.Normalization.Value", &[value]);
            let twice = model.invoke("Mpk.Go.Practical.String.Normalization.Value", &[normalized]);
            let lengths = [
                model.length(value),
                model.length(normalized),
                model.length(twice),
            ];
            let actual_raw = model.bytes(value);
            let actual_normal = model.bytes(normalized);
            let actual_twice = model.bytes(twice);
            let expected: [u8; 256] = std::array::from_fn(|i| if i < n { raw[i] } else { 0 });
            let steps = model.steps();
            total_steps += steps;
            let output = serde_json::json!({"lengths":lengths,"raw":actual_raw.to_vec(),"normalized":actual_normal.to_vec(),"twice":actual_twice.to_vec(),"expected":expected.to_vec(),"actual_AST_steps":steps,"arithmetic_or_string_definition_callbacks":false,"theorem_bodies_executed":false,"core_primitives":"Bool constructors/rec/cases/recType, Nat zero/succ/rec/induct; full original input carrier only","universal_core_proof_claimed_from_finite_model":false});
            retain(
                &label,
                "observation.json",
                format!("{}\n", serde_json::to_string_pretty(&output).unwrap()).as_bytes(),
            );
            assert_eq!(lengths, [n as u64; 3]);
            assert_eq!(actual_raw, raw);
            assert_eq!(actual_normal, expected);
            assert_eq!(actual_twice, expected);
            cases += 1;
        }
    }
    assert_eq!(cases, 54);
    println!("actual normalization definition observations:54 complete raw carriers,all64 length and2048 data bits,dirty tails,empty/full/binary boundaries,idempotence;{total_steps} AST steps;universal proofs checked separately");
}
