use mpk_cert::encode::{Certificate, DeclarationKind};
use mpk_vc::go_practical_ordinary::{
    generate_go_practical_slice_descriptor_foundation,
    import_go_practical_slice_descriptor_foundation, GoOrdinaryError,
};
#[path = "support/go_word_division_dependencies.rs"]
mod dependencies;
fn retain(name: &str, extension: &str, raw: &[u8]) {
    if let Some(root) = std::env::var_os("MPK_GO_SLICE_DESCRIPTOR_OUTPUT") {
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

fn foundation() -> &'static mpk_vc::go_practical_ordinary::GoSliceDescriptorFoundation {
    static VALUE: std::sync::OnceLock<mpk_vc::go_practical_ordinary::GoSliceDescriptorFoundation> =
        std::sync::OnceLock::new();
    VALUE.get_or_init(|| generate_go_practical_slice_descriptor_foundation().unwrap())
}
fn baseline() -> Certificate {
    mpk_cert::decode_canonical_certificate(foundation().certificate_bytes()).unwrap()
}
#[test]
fn descriptor_retains_complete_parent() {
    let f = foundation();
    inspect("descriptor-foundation", f.certificate_bytes(), true);
    assert_eq!(
        *f,
        import_go_practical_slice_descriptor_foundation(f.certificate_bytes()).unwrap()
    );
    let old =
        mpk_vc::go_practical_ordinary::generate_go_practical_string_slice_foundation().unwrap();
    let path=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../develop/migrations/go-practical/t02-w01/verification/string-slice/epochs/final-controls/selected/candidates/slice-foundation.mpcert");
    assert_eq!(old.certificate_bytes(), std::fs::read(path).unwrap());
    let prior = mpk_cert::decode_canonical_certificate(old.certificate_bytes()).unwrap();
    let c = baseline();
    let hashes = dependencies::declarations(&c);
    for (name, v) in dependencies::declarations(&prior) {
        assert_eq!(hashes.get(&name), Some(&v), "{name}");
    }
    let fresh = c
        .declarations
        .iter()
        .filter(|d| c.name_table[d.name as usize].starts_with("Mpk.Go.Practical.Slice.Descriptor."))
        .collect::<Vec<_>>();
    let laws = fresh
        .iter()
        .filter(|d| matches!(d.kind, DeclarationKind::Theorem { .. }))
        .count();
    println!("descriptor declarations={} terms={} bytes={} new_definitions={} new_laws={} parent_identities={} full_parent_bytes={}",c.declarations.len(),c.term_table.len(),f.certificate_bytes().len(),fresh.len()-laws,laws,prior.declarations.len(),old.certificate_bytes().len());
}
fn witness_domain(c: &Certificate, mut node: u32) -> bool {
    while let TermNode::Pi { body, .. } = c.term_table[node as usize] {
        node = body;
    }
    let TermNode::App { function, .. } = c.term_table[node as usize] else {
        return false;
    };
    let TermNode::Const { global, .. } = c.term_table[function as usize] else {
        return false;
    };
    let n = &c.name_table[c.declarations[global as usize].name as usize];
    n == "Std.Eq"
        || n.starts_with("Mpk.Go.Practical.Slice.Descriptor.")
            && ["SameFields", "CanonicalNil", "HeaderBounds", "HeaderValid"]
                .iter()
                .any(|s| n.ends_with(s))
}
#[test]
fn exact_ownership_is_distinct_from_core_accepted_extensions() {
    let mut c = baseline();
    let ty = sym(&mut c, "Std.Bool");
    let value = flag(&mut c, true);
    let name = c.name_table.len() as u32;
    c.name_table
        .push("Mpk.Go.Practical.Slice.Descriptor.HonestExtra".into());
    c.declarations.push(Declaration {
        name,
        kind: DeclarationKind::Def {
            ty,
            value,
            reducibility: mpk_cert::encode::DefinitionReducibility::Reducible,
        },
    });
    let raw = encode(c);
    inspect("descriptor-honest-extra", &raw, true);
    assert_eq!(
        import_go_practical_slice_descriptor_foundation(&raw),
        Err(GoOrdinaryError::Linkage)
    );
    assert_eq!(
        import_go_practical_slice_descriptor_foundation(&vec![0; 2097153]),
        Err(GoOrdinaryError::Limit)
    );
}
#[test]
fn descriptor_laws_bind_all_conclusions_and_typed_witnesses() {
    let base = baseline();
    let (mut laws, mut witnesses) = (0, 0);
    for (at, d) in base.declarations.iter().enumerate() {
        let name = &base.name_table[d.name as usize];
        let Some(short) = name.strip_prefix("Mpk.Go.Practical.Slice.Descriptor.") else {
            continue;
        };
        let DeclarationKind::Theorem { ty, proof } = d.kind else {
            continue;
        };
        let mut c = base.clone();
        let changed = bad_result(&mut c, ty);
        c.declarations[at].kind = DeclarationKind::Theorem { ty: changed, proof };
        inspect(&format!("false-{short}"), &encode(c), false);
        laws += 1;
        let (mut cursor, mut binder) = (ty, 0);
        while let TermNode::Pi { ty: domain, body } = base.term_table[cursor as usize] {
            if witness_domain(&base, domain) {
                let mut c = base.clone();
                let b = sym(&mut c, "Std.Bool");
                let changed = erase(&mut c, ty, binder, b);
                c.declarations[at].kind = DeclarationKind::Theorem { ty: changed, proof };
                inspect(&format!("premise-{short}-{binder}"), &encode(c), false);
                witnesses += 1;
            }
            cursor = body;
            binder += 1;
        }
    }
    assert_eq!(laws, 113);
    assert!(witnesses > 30);
    retain("control-inventory","json",format!("{}\n",serde_json::json!({"altered_conclusions":laws,"actual_typed_witness_domains":witnesses,"typed_wrong_definitions":28,"zero_axiom_acceptances":2,"complete_candidates":laws+witnesses+30})).as_bytes());
    println!("descriptor conclusions={laws} actual typed witness domains={witnesses}");
}
fn word_value(c: &mut Certificate, width: usize, yes: bool) -> u32 {
    let b = sym(c, "Std.Bool");
    let mut out = flag(c, yes);
    for _ in 0..width.trailing_zeros() {
        out = term(c, TermNode::Lam { ty: b, body: out });
    }
    out
}
fn word_type(c: &mut Certificate, width: usize) -> u32 {
    let b = sym(c, "Std.Bool");
    let mut out = b;
    for _ in 0..width.trailing_zeros() {
        out = term(c, TermNode::Pi { ty: b, body: out });
    }
    out
}
fn raw_descriptor(c: &mut Certificate, nil: bool) -> u32 {
    let tag = flag(c, nil);
    let owner = sym(c, "Std.Nat.zero");
    let path = sym(c, "Mpk.Go.Practical.Slice.Descriptor.PathZero");
    let z = word_value(c, 64, false);
    call(
        c,
        "Mpk.Go.Practical.Slice.Descriptor.Make",
        vec![tag, owner, path, z, z, z],
    )
}
fn constant_body(c: &mut Certificate, value: u32, replacement: u32, arity: usize) -> u32 {
    if arity == 0 {
        return replacement;
    }
    let TermNode::Lam { ty, body } = c.term_table[value as usize] else {
        panic!("missing explicit parameter");
    };
    let body = constant_body(c, body, replacement, arity - 1);
    term(c, TermNode::Lam { ty, body })
}
fn replace_free(c: &mut Certificate, node: u32, wanted: u32, depth: u32, replacement: u32) -> u32 {
    let n = match c.term_table[node as usize].clone() {
        TermNode::Var(i) if i == wanted + depth => return replacement,
        TermNode::App {
            function,
            arguments,
        } => {
            let function = replace_free(c, function, wanted, depth, replacement);
            let arguments = arguments
                .into_iter()
                .map(|a| replace_free(c, a, wanted, depth, replacement))
                .collect();
            TermNode::App {
                function,
                arguments,
            }
        }
        TermNode::Lam { ty, body } => {
            let ty = replace_free(c, ty, wanted, depth, replacement);
            let body = replace_free(c, body, wanted, depth + 1, replacement);
            TermNode::Lam { ty, body }
        }
        TermNode::Pi { ty, body } => {
            let ty = replace_free(c, ty, wanted, depth, replacement);
            let body = replace_free(c, body, wanted, depth + 1, replacement);
            TermNode::Pi { ty, body }
        }
        TermNode::Let { ty, value, body } => {
            let ty = replace_free(c, ty, wanted, depth, replacement);
            let value = replace_free(c, value, wanted, depth, replacement);
            let body = replace_free(c, body, wanted, depth + 1, replacement);
            TermNode::Let { ty, value, body }
        }
        _ => return node,
    };
    term(c, n)
}
fn ignore_nil(c: &mut Certificate, value: u32, arity: usize) -> u32 {
    if arity == 0 {
        let yes = flag(c, true);
        return replace_free(c, value, 5, 0, yes);
    }
    let TermNode::Lam { ty, body } = c.term_table[value as usize] else {
        unreachable!()
    };
    let body = ignore_nil(c, body, arity - 1);
    term(c, TermNode::Lam { ty, body })
}
#[test]
fn descriptor_definitions_have_actual_countermodels() {
    let mut count = 0;
    let base = baseline();
    for (at, d) in base.declarations.iter().enumerate() {
        let Some(name) =
            base.name_table[d.name as usize].strip_prefix("Mpk.Go.Practical.Slice.Descriptor.")
        else {
            continue;
        };
        let DeclarationKind::Def {
            ty,
            value,
            reducibility,
        } = d.kind
        else {
            continue;
        };
        if [
            "Type",
            "PathCell",
            "PathType",
            "SameFields",
            "CanonicalNil",
            "HeaderBounds",
            "HeaderValid",
        ]
        .contains(&name)
        {
            continue;
        }
        let mut c = base.clone();
        let value = if name == "Make" {
            ignore_nil(&mut c, value, 6)
        } else {
            let (replacement, arity) = match name {
                "Raw8Bit" => (flag(&mut c, true), 4),
                "Raw64Bit" => (flag(&mut c, true), 7),
                "PathMake" => (
                    sym(&mut c, "Mpk.Go.Practical.Slice.Descriptor.PathZero"),
                    16,
                ),
                "PathKind" => (word_value(&mut c, 8, false), 5),
                "PathIndex" => (word_value(&mut c, 64, false), 5),
                "NilTag" => (flag(&mut c, true), 1),
                "Owner" => (sym(&mut c, "Std.Nat.zero"), 1),
                "Path" => (sym(&mut c, "Mpk.Go.Practical.Slice.Descriptor.PathZero"), 1),
                "Offset" | "Length" | "Capacity" => (word_value(&mut c, 64, false), 1),
                "Copy" | "Reconstruct" => (raw_descriptor(&mut c, true), 1),
                "Nil" => (raw_descriptor(&mut c, false), 0),
                "EmptyNonNilRaw" => (raw_descriptor(&mut c, true), 0),
                "OffsetValue" | "LengthValue" | "CapacityValue" => (sym(&mut c, "Std.Nat.zero"), 1),
                "AbsoluteIndex" => (sym(&mut c, "Std.Nat.zero"), 2),
                "WithBounds" => (term(&mut c, TermNode::Var(3)), 4),
                "IndexUnsigned" | "IndexSigned" => (flag(&mut c, true), 2),
                "TwoUnsigned" | "TwoSigned" => (flag(&mut c, true), 3),
                "ThreeUnsigned" | "ThreeSigned" => (flag(&mut c, true), 4),
                "PathZero" => {
                    let a = word_type(&mut c, 8);
                    let b = word_type(&mut c, 64);
                    let k = word_value(&mut c, 8, true);
                    let n = word_value(&mut c, 64, false);
                    let mut body = call(&mut c, "Mpk.Go.Practical.Product.Make", vec![a, b, k, n]);
                    let t = sym(&mut c, "Std.Bool");
                    for _ in 0..4 {
                        body = term(&mut c, TermNode::Lam { ty: t, body });
                    }
                    (body, 0)
                }
                _ => panic!("uncovered runtime definition {name}"),
            };
            constant_body(&mut c, value, replacement, arity)
        };
        c.declarations[at].kind = DeclarationKind::Def {
            ty,
            value,
            reducibility,
        };
        let label = format!("definition-{name}");
        let raw = encode(c);
        retain(&label, "mpcert", &raw);
        observations::countermodel(
            &mpk_cert::decode_canonical_certificate(&raw).unwrap(),
            name,
            retain,
        );
        inspect_saved(&label, &raw, false);
        count += 1;
    }
    assert_eq!(count, 28);
}
#[path = "support/go_string_equality_eval.rs"]
mod actual;
#[path = "support/go_slice_descriptor_observations.rs"]
mod observations;
#[test]
fn observations_preserve_every_descriptor_bit() {
    retain(
        "observed-descriptor-foundation",
        "mpcert",
        foundation().certificate_bytes(),
    );
    observations::run(&baseline(), retain);
}
