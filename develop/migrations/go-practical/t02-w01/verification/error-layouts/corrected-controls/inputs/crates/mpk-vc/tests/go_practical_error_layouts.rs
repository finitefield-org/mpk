use mpk_cert::encode::{
    Certificate, Declaration, DeclarationKind, DefinitionReducibility, LevelNode, TermNode,
};
use mpk_vc::go_practical_ordinary::{
    generate_go_practical_error_layout_foundation, generate_go_practical_error_value_foundation,
    import_go_practical_error_layout_foundation, GoOrdinaryError,
};
use sha2::{Digest, Sha256};
#[path = "support/go_word_division_dependencies.rs"]
mod dependencies;

fn inspect(name: &str, bytes: &[u8], accepted: bool) {
    let out = std::env::var_os("MPK_GO_ERROR_LAYOUTS_TEST_OUTPUT").map(std::path::PathBuf::from);
    if let Some(root) = &out {
        use std::io::Write;
        std::fs::create_dir_all(root).unwrap();
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(root.join(format!("{name}.mpcert")))
            .unwrap()
            .write_all(bytes)
            .unwrap();
    }
    let started = std::time::Instant::now();
    let result = mpk_kernel::verify_certificate_bytes_json_output(bytes);
    println!("kernel {name}: {:?}", started.elapsed());
    if let Some(root) = out {
        use std::io::Write;
        std::fs::create_dir_all(&root).unwrap();
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(root.join(format!("{name}.rust.json")))
            .unwrap()
            .write_all(format!("{}\n", result.json).as_bytes())
            .unwrap();
    }
    assert_eq!(result.accepted, accepted, "{name}: {}", result.json);
    let report: serde_json::Value = serde_json::from_str(&result.json).unwrap();
    if accepted {
        assert_eq!(report["axiom_count"], 0);
        assert!(report["axiom_report"]["entries"]
            .as_array()
            .unwrap()
            .is_empty());
    } else {
        assert_eq!(report["error_code"], "KERNEL_CORE_CHECK");
        let detail: serde_json::Value =
            serde_json::from_str(report["error_detail"].as_str().unwrap()).unwrap();
        assert_eq!(detail["code"], "CORE_TYPE_MISMATCH");
    }
}
fn baseline(n: usize) -> Certificate {
    mpk_cert::decode_canonical_certificate(
        generate_go_practical_error_layout_foundation(n)
            .unwrap()
            .certificate_bytes(),
    )
    .unwrap()
}
fn global(c: &Certificate, name: &str) -> usize {
    c.declarations
        .iter()
        .position(|d| c.name_table[d.name as usize] == name)
        .unwrap()
}
fn term(c: &mut Certificate, node: TermNode) -> u32 {
    let id = c.term_table.len() as u32;
    c.term_table.push(node);
    id
}
fn constant(c: &mut Certificate, name: &str) -> u32 {
    let global = global(c, name) as u32;
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
fn encode(mut c: Certificate) -> Vec<u8> {
    let old = c.name_table.clone();
    c.name_table.sort();
    for d in &mut c.declarations {
        d.name = c.name_table.binary_search(&old[d.name as usize]).unwrap() as u32;
    }
    for l in &mut c.level_table {
        if let LevelNode::Param(n) = l {
            *n = c.name_table.binary_search(&old[*n as usize]).unwrap() as u32;
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

#[test]
fn actual_three_frozen_layout_endpoints_and_large_case_set() {
    let prior = generate_go_practical_error_value_foundation().unwrap();
    assert_eq!(
        format!("{:x}", Sha256::digest(prior.certificate_bytes())),
        "1b5fd24d70d005c7873921971577b7ba9c13a97c7bd61a7261d2fa8c24b80c10"
    );
    let old = dependencies::declarations(
        &mpk_cert::decode_canonical_certificate(prior.certificate_bytes()).unwrap(),
    );
    assert_eq!(old.len(), 2050);
    for n in [0, 3, 70] {
        let started = std::time::Instant::now();
        let foundation = generate_go_practical_error_layout_foundation(n).unwrap();
        println!("generation N{n}: {:?}", started.elapsed());
        assert_eq!(foundation.case_count(), n);
        inspect(
            &format!("layout-N{n}"),
            foundation.certificate_bytes(),
            true,
        );
        let c = mpk_cert::decode_canonical_certificate(foundation.certificate_bytes()).unwrap();
        assert_frozen_shape(&c, n);
        let new = dependencies::declarations(&c);
        for (name, identity) in &old {
            assert_eq!(new.get(name), Some(identity), "{name}");
        }
        println!(
            "N{n}: declarations={} terms={} bytes={}",
            c.declarations.len(),
            c.term_table.len(),
            foundation.certificate_bytes().len()
        );
    }
}

#[test]
fn all_balanced_layout_sizes_zero_through_seventy_keep_canonical_shapes() {
    for n in 0..=70 {
        let f = generate_go_practical_error_layout_foundation(n).unwrap();
        inspect(&format!("matrix-N{n}"), f.certificate_bytes(), true);
        let c = mpk_cert::decode_canonical_certificate(f.certificate_bytes()).unwrap();
        let expected = 2050 + 7 * n + if n == 0 { 29 } else { 28 };
        assert_eq!(c.declarations.len(), expected);
        assert_frozen_shape(&c, n);
    }
}

fn fingerprint(v: serde_json::Value) -> String {
    format!("{:x}", Sha256::digest(serde_json::to_vec(&v).unwrap()))
}
fn cn(name: &str) -> String {
    fingerprint(serde_json::json!(["const", name, []]))
}
fn vr(index: usize) -> String {
    fingerprint(serde_json::json!(["var", index]))
}
fn apply(f: String, args: Vec<String>) -> String {
    fingerprint(serde_json::json!(["app", f, args]))
}
fn binder(lam: bool, ty: String, body: String) -> String {
    fingerprint(serde_json::json!([
        if lam { "lam" } else { "pi" },
        ty,
        body
    ]))
}
fn word(zero: bool) -> String {
    let mut e = cn(if zero { "Std.Bool.false" } else { "Std.Bool" });
    for _ in 0..6 {
        e = binder(zero, cn("Std.Bool"), e);
    }
    e
}
// Independent translation of frozen carrier_terms.product: no projection or
// adapter implementation is consulted by this structural shape oracle.
fn frozen_type(n: usize, start: usize, len: usize, depth: usize, value_binders: usize) -> String {
    if len == 1 {
        return if start == 0 {
            word(false)
        } else {
            vr(n - 1 - (start - 1) + depth + value_binders)
        };
    }
    let m = len / 2;
    binder(
        false,
        cn("Std.Bool"),
        apply(
            cn("Std.Bool.recType"),
            vec![
                frozen_type(n, start, m, depth + 1, value_binders),
                frozen_type(n, start + m, len - m, depth + 1, value_binders),
                vr(0),
            ],
        ),
    )
}
fn frozen_zero(n: usize, start: usize, len: usize, depth: usize) -> String {
    if len == 1 {
        return if start == 0 {
            word(true)
        } else {
            vr(n - 1 - (start - 1) + depth)
        };
    }
    let m = len / 2;
    let motive = binder(
        true,
        cn("Std.Bool"),
        apply(
            cn("Std.Bool.recType"),
            vec![
                frozen_type(n, start, m, depth + 2, n),
                frozen_type(n, start + m, len - m, depth + 2, n),
                vr(0),
            ],
        ),
    );
    binder(
        true,
        cn("Std.Bool"),
        apply(
            cn("Std.Bool.cases"),
            vec![
                motive,
                frozen_zero(n, start, m, depth + 1),
                frozen_zero(n, start + m, len - m, depth + 1),
                vr(0),
            ],
        ),
    )
}
fn assert_frozen_shape(c: &Certificate, n: usize) {
    let mut levels: Vec<String> = vec![];
    for level in &c.level_table {
        levels.push(fingerprint(match level {
            LevelNode::Zero => serde_json::json!(["zero"]),
            LevelNode::Succ(i) => serde_json::json!(["succ", levels[*i as usize]]),
            LevelNode::Max(a, b) => {
                serde_json::json!(["max", levels[*a as usize], levels[*b as usize]])
            }
            LevelNode::Param(n) => serde_json::json!(["param", c.name_table[*n as usize]]),
        }));
    }
    let mut hashes: Vec<String> = vec![];
    for node in &c.term_table {
        let h = match node {
            TermNode::Sort(i) => fingerprint(serde_json::json!(["sort", levels[*i as usize]])),
            TermNode::Var(i) => vr(*i as usize),
            TermNode::Const { global, levels } => {
                assert!(levels.is_empty());
                cn(&c.name_table[c.declarations[*global as usize].name as usize])
            }
            TermNode::App {
                function,
                arguments,
            } => apply(
                hashes[*function as usize].clone(),
                arguments
                    .iter()
                    .map(|i| hashes[*i as usize].clone())
                    .collect(),
            ),
            TermNode::Pi { ty, body } => binder(
                false,
                hashes[*ty as usize].clone(),
                hashes[*body as usize].clone(),
            ),
            TermNode::Lam { ty, body } => binder(
                true,
                hashes[*ty as usize].clone(),
                hashes[*body as usize].clone(),
            ),
            TermNode::Let { ty, value, body } => fingerprint(serde_json::json!([
                "let",
                hashes[*ty as usize],
                hashes[*value as usize],
                hashes[*body as usize]
            ])),
        };
        hashes.push(h);
    }
    for (name, binders, expected) in [
        ("Carrier", n, frozen_type(n, 0, n + 1, 0, 0)),
        ("Nil", 2 * n, frozen_zero(n, 0, n + 1, 0)),
    ] {
        let d = &c.declarations[global(c, &format!("Mpk.Go.Practical.ErrorLayout.N{n}.{name}"))];
        let DeclarationKind::Def { mut value, .. } = d.kind else {
            panic!()
        };
        for _ in 0..binders {
            let TermNode::Lam { body, .. } = c.term_table[value as usize] else {
                panic!()
            };
            value = body;
        }
        assert_eq!(
            hashes[value as usize], expected,
            "N{n} {name}: exact frozen balanced shape"
        );
    }
}

#[test]
fn exact_layout_ownership_and_unchanged_artifact_resource_boundaries() {
    let f = generate_go_practical_error_layout_foundation(2).unwrap();
    assert_eq!(
        import_go_practical_error_layout_foundation(f.certificate_bytes(), 2).unwrap(),
        f
    );
    assert_eq!(
        import_go_practical_error_layout_foundation(f.certificate_bytes(), 3),
        Err(GoOrdinaryError::Linkage)
    );
    assert_eq!(
        generate_go_practical_error_layout_foundation(usize::MAX),
        Err(GoOrdinaryError::Limit)
    );
    assert_eq!(
        generate_go_practical_error_layout_foundation(4096),
        Err(GoOrdinaryError::Limit)
    );
    assert_eq!(
        import_go_practical_error_layout_foundation(&vec![0; 2097153], 2),
        Err(GoOrdinaryError::Limit)
    );
    let mut trailing = f.certificate_bytes().to_vec();
    trailing.push(0);
    assert_eq!(
        import_go_practical_error_layout_foundation(&trailing, 2),
        Err(GoOrdinaryError::Linkage)
    );
    let mut c = baseline(2);
    let boolean = constant(&mut c, "Std.Bool");
    let yes = constant(&mut c, "Std.Bool.true");
    let ty = call(&mut c, "Std.Eq", vec![boolean, yes, yes]);
    let proof = call(&mut c, "Std.Eq.refl", vec![boolean, yes]);
    let name = c.name_table.len() as u32;
    c.name_table
        .push("Control.ErrorLayout.HonestExtension".into());
    c.declarations.push(Declaration {
        name,
        kind: DeclarationKind::Theorem { ty, proof },
    });
    let raw = encode(c);
    inspect("honest-extension", &raw, true);
    assert_eq!(
        import_go_practical_error_layout_foundation(&raw, 2),
        Err(GoOrdinaryError::Linkage)
    );
}

fn replace_body(
    c: &mut Certificate,
    value: u32,
    replacement: &mut impl FnMut(&mut Certificate) -> u32,
) -> u32 {
    if let TermNode::Lam { ty, body } = c.term_table[value as usize].clone() {
        let body = replace_body(c, body, replacement);
        term(c, TermNode::Lam { ty, body })
    } else {
        replacement(c)
    }
}
fn corruption(
    n: usize,
    label: &str,
    member: &str,
    mut replacement: impl FnMut(&mut Certificate) -> u32,
) {
    let mut c = baseline(n);
    let id = global(&c, &format!("Mpk.Go.Practical.ErrorLayout.N{n}.{member}"));
    let DeclarationKind::Def { ty, value, .. } = c.declarations[id].kind else {
        panic!()
    };
    let value = replace_body(&mut c, value, &mut replacement);
    c.declarations[id].kind = DeclarationKind::Def {
        ty,
        value,
        reducibility: DefinitionReducibility::Reducible,
    };
    inspect(label, &encode(c), false);
}
#[test]
fn reject_wrong_layout_nil_field_order_or_incomplete_adapters() {
    corruption(3, "wrong-tag", "Tag", |c| {
        constant(c, "Mpk.Go.Practical.Word.W64.Zero")
    });
    corruption(3, "wrong-zero-tag", "ZeroTag", |c| {
        constant(c, "Mpk.Go.Practical.Word.W64.One")
    });
    for (n, index) in [(3, 0), (3, 2), (70, 0), (70, 64), (70, 69)] {
        corruption(
            n,
            &format!("dropped-N{n}-payload{index}"),
            &format!("Payload{index}"),
            |c| constant(c, "Std.Bool.false"),
        );
    }
    for n in [0, 3, 70] {
        corruption(n, &format!("dropped-N{n}-from-view-tag"), "FromView", |c| {
            term(c, TermNode::Var(0))
        });
    }
}
