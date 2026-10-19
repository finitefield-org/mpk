use mpk_cert::encode::{
    Certificate, Declaration, DeclarationKind, DefinitionReducibility, LevelNode, TermNode,
};
use mpk_vc::go_practical_ordinary::{
    generate_go_practical_integer_foundation, generate_go_practical_natural_foundation,
    import_go_practical_integer_foundation, GoOrdinaryError,
};

fn emit(name: &str, bytes: &[u8], accepted: bool) {
    let output = mpk_kernel::verify_certificate_bytes_json_output(bytes);
    if let Some(root) = std::env::var_os("MPK_GO_INTEGER_FOUNDATION_TEST_OUTPUT") {
        use std::io::Write;
        let root = std::path::PathBuf::from(root);
        std::fs::create_dir_all(&root).unwrap();
        for (extension, data) in [
            ("mpcert", bytes.to_vec()),
            ("rust.json", format!("{}\n", output.json).into_bytes()),
        ] {
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(root.join(format!("{name}.{extension}")))
                .unwrap();
            file.write_all(&data).unwrap();
        }
    }
    assert_eq!(output.accepted, accepted, "{name}: {}", output.json);
    if !accepted {
        let report: serde_json::Value = serde_json::from_str(&output.json).unwrap();
        assert_eq!(report["error_code"], "KERNEL_CORE_CHECK", "{name}");
        let detail: serde_json::Value =
            serde_json::from_str(report["error_detail"].as_str().unwrap()).unwrap();
        assert_eq!(detail["code"], "CORE_TYPE_MISMATCH", "{name}");
    }
}
fn global(c: &Certificate, name: &str) -> u32 {
    c.declarations
        .iter()
        .position(|d| c.name_table[d.name as usize] == name)
        .unwrap() as u32
}
fn term(c: &mut Certificate, node: TermNode) -> u32 {
    let id = c.term_table.len() as u32;
    c.term_table.push(node);
    id
}
fn constant(c: &mut Certificate, name: &str) -> u32 {
    let global = global(c, name);
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
fn lam(c: &mut Certificate, ty: u32, body: u32) -> u32 {
    term(c, TermNode::Lam { ty, body })
}
fn natural(c: &mut Certificate, n: u32) -> u32 {
    let mut v = constant(c, "Std.Nat.zero");
    for _ in 0..n {
        v = call(c, "Std.Nat.succ", vec![v]);
    }
    v
}
fn integer(c: &mut Certificate) -> u32 {
    let boolean = constant(c, "Std.Bool");
    let nat = constant(c, "Std.Nat");
    term(
        c,
        TermNode::Pi {
            ty: boolean,
            body: nat,
        },
    )
}
fn declare(c: &mut Certificate, name: &str, kind: DeclarationKind) {
    let id = c.name_table.len() as u32;
    c.name_table.push(name.into());
    c.declarations.push(Declaration { name: id, kind });
}
fn encode(mut c: Certificate) -> Vec<u8> {
    let previous = c.name_table.clone();
    c.name_table.sort();
    for declaration in &mut c.declarations {
        declaration.name = c
            .name_table
            .binary_search(&previous[declaration.name as usize])
            .unwrap() as u32;
    }
    for level in &mut c.level_table {
        if let LevelNode::Param(name) = level {
            *name = c
                .name_table
                .binary_search(&previous[*name as usize])
                .unwrap() as u32;
        }
    }
    c.export_block = mpk_cert::build_export_block(&c).unwrap();
    c.axiom_report = mpk_cert::build_axiom_report(&c).unwrap();
    assert_eq!(c.axiom_report.summary.total_axiom_count, 0);
    c.hashes.export_hash = mpk_cert::export_block_hash(&c.export_block);
    c.hashes.axiom_report_hash = mpk_cert::axiom_report_hash_for_report(&c.axiom_report);
    mpk_cert::encode_certificate_bounded(&c, 2_097_152).unwrap()
}
fn baseline() -> Certificate {
    mpk_cert::decode_canonical_certificate(
        generate_go_practical_integer_foundation()
            .unwrap()
            .certificate_bytes(),
    )
    .unwrap()
}

#[test]
fn actual_balanced_integer_foundation_checks_all_ring_equivalence_and_observation_laws() {
    let foundation = generate_go_practical_integer_foundation().unwrap();
    let c = mpk_cert::decode_canonical_certificate(foundation.certificate_bytes()).unwrap();
    assert!(
        c.imports.is_empty()
            && c.source_manifest.is_none()
            && c.theory_certificates.is_empty()
            && c.proof_node_table.is_empty()
    );
    assert_eq!(c.axiom_report.summary.total_axiom_count, 0);
    for name in [
        "Make",
        "Equivalent",
        "Reflexive",
        "Symmetric",
        "Transitive",
        "AddAssoc",
        "MulAssoc",
        "MulAddLeft",
        "MulAddRight",
        "AddInverse",
        "RepresentationShift",
        "AddRespects",
        "SubRespects",
        "NegateRespects",
        "MulRespects",
        "EqualFromEquivalent",
        "EqualReflect",
        "EqualRespects",
        "LessRespects",
        "LessEqualRespects",
        "FromNatInjective",
    ] {
        global(&c, &format!("Mpk.Go.Practical.Int.{name}"));
    }
    emit("integer-foundation", foundation.certificate_bytes(), true);
    println!(
        "bytes={} terms={} declarations={}",
        foundation.certificate_bytes().len(),
        c.term_table.len(),
        c.declarations.len()
    );
}

#[test]
fn exact_integer_import_is_deterministic_and_preserves_the_complete_natural_basis() {
    let first = generate_go_practical_integer_foundation().unwrap();
    assert_eq!(first, generate_go_practical_integer_foundation().unwrap());
    assert_eq!(
        first,
        import_go_practical_integer_foundation(first.certificate_bytes()).unwrap()
    );
    let mut changed = first.certificate_bytes().to_vec();
    changed.push(0);
    assert_eq!(
        import_go_practical_integer_foundation(&changed),
        Err(GoOrdinaryError::Linkage)
    );
    assert_eq!(
        import_go_practical_integer_foundation(&vec![0; 2_097_153]),
        Err(GoOrdinaryError::Limit)
    );
    let retained = generate_go_practical_natural_foundation().unwrap();
    let output = mpk_kernel::verify_certificate_bytes_json_output(retained.certificate_bytes());
    assert!(
        output.accepted
            && output
                .json
                .contains("ca12c603ca781b1ea72ac3c65830bc40dcb205111957714621a62f987e2ae4a5")
    );
    emit("retained-natural-basis", retained.certificate_bytes(), true);
    let mut c = baseline();
    let nat = constant(&mut c, "Std.Nat");
    let value = natural(&mut c, 0);
    declare(
        &mut c,
        "Mpk.Go.Practical.Test.Extra",
        DeclarationKind::Def {
            ty: nat,
            value,
            reducibility: DefinitionReducibility::Reducible,
        },
    );
    let changed = encode(c);
    emit("honest-different-integer-component", &changed, true);
    assert_eq!(
        import_go_practical_integer_foundation(&changed),
        Err(GoOrdinaryError::Linkage)
    );
}

#[test]
fn redundant_representations_are_equivalent_without_assuming_function_equality() {
    let mut c = baseline();
    let boolean = constant(&mut c, "Std.Bool");
    for (i, (p, n, k)) in [(2, 0, 3), (0, 4, 3), (0, 0, 6)].into_iter().enumerate() {
        let p = natural(&mut c, p);
        let n = natural(&mut c, n);
        let k = natural(&mut c, k);
        let left = call(&mut c, "Mpk.Go.Practical.Int.Make", vec![p, n]);
        let pk = call(&mut c, "Mpk.Go.Practical.Nat.Add", vec![p, k]);
        let nk = call(&mut c, "Mpk.Go.Practical.Nat.Add", vec![n, k]);
        let right = call(&mut c, "Mpk.Go.Practical.Int.Make", vec![pk, nk]);
        let ty = call(&mut c, "Mpk.Go.Practical.Int.Equivalent", vec![left, right]);
        let proof = call(
            &mut c,
            "Mpk.Go.Practical.Int.RepresentationShift",
            vec![p, n, k],
        );
        declare(
            &mut c,
            &format!("Mpk.Go.Practical.Test.Representation{i}"),
            DeclarationKind::Theorem { ty, proof },
        );
        let value = call(&mut c, "Mpk.Go.Practical.Int.Equal", vec![left, right]);
        let yes = constant(&mut c, "Std.Bool.true");
        let ty = call(&mut c, "Std.Eq", vec![boolean, value, yes]);
        let proof = call(
            &mut c,
            "Mpk.Go.Practical.Int.EqualFromEquivalent",
            vec![left, right, proof],
        );
        declare(
            &mut c,
            &format!("Mpk.Go.Practical.Test.Observation{i}"),
            DeclarationKind::Theorem { ty, proof },
        );
    }
    let correct = encode(c);
    emit("redundant-representations", &correct, true);
    let mut c = baseline();
    let p = natural(&mut c, 5);
    let n = natural(&mut c, 3);
    let q = natural(&mut c, 2);
    let zero = natural(&mut c, 0);
    let left = call(&mut c, "Mpk.Go.Practical.Int.Make", vec![p, n]);
    let right = call(&mut c, "Mpk.Go.Practical.Int.Make", vec![q, zero]);
    let int = integer(&mut c);
    let ty = call(&mut c, "Std.Eq", vec![int, left, right]);
    let proof = call(&mut c, "Std.Eq.refl", vec![int, left]);
    declare(
        &mut c,
        "Mpk.Go.Practical.Test.FalseFunctionEquality",
        DeclarationKind::Theorem { ty, proof },
    );
    emit("false-function-equality", &encode(c), false);
}

#[test]
fn reencoded_wrong_representations_operations_and_ignored_premises_fail_by_type_mismatch() {
    let original = baseline();
    for mutation in [
        "make",
        "negative-projection",
        "equivalent-function-equality",
        "positive-only-equal",
        "incomplete-multiplication",
        "transitive-premise",
        "addition-premise",
        "equal-respects-premise",
        "less-equal-respects-premise",
        "nat-injection-premise",
    ] {
        let mut c = original.clone();
        let nat = constant(&mut c, "Std.Nat");
        let boolean = constant(&mut c, "Std.Bool");
        let int = integer(&mut c);
        let (name, value, is_definition) = match mutation {
            "make" => {
                let p = term(&mut c, TermNode::Var(2));
                let value = lam(&mut c, boolean, p);
                let value = lam(&mut c, nat, value);
                let value = lam(&mut c, nat, value);
                ("Make", value, true)
            }
            "negative-projection" => {
                let x = term(&mut c, TermNode::Var(0));
                let no = constant(&mut c, "Std.Bool.false");
                let value = term(
                    &mut c,
                    TermNode::App {
                        function: x,
                        arguments: vec![no],
                    },
                );
                ("Negative", lam(&mut c, int, value), true)
            }
            "equivalent-function-equality" => {
                let x = term(&mut c, TermNode::Var(1));
                let y = term(&mut c, TermNode::Var(0));
                let value = call(&mut c, "Std.Eq", vec![int, x, y]);
                let value = lam(&mut c, int, value);
                ("Equivalent", lam(&mut c, int, value), true)
            }
            "positive-only-equal" | "incomplete-multiplication" => {
                let x = term(&mut c, TermNode::Var(1));
                let y = term(&mut c, TermNode::Var(0));
                let px = call(&mut c, "Mpk.Go.Practical.Int.Positive", vec![x]);
                let py = call(&mut c, "Mpk.Go.Practical.Int.Positive", vec![y]);
                let (name, value) = if mutation == "positive-only-equal" {
                    (
                        "Equal",
                        call(&mut c, "Mpk.Go.Practical.Nat.Equal", vec![px, py]),
                    )
                } else {
                    let nx = call(&mut c, "Mpk.Go.Practical.Int.Negative", vec![x]);
                    let ny = call(&mut c, "Mpk.Go.Practical.Int.Negative", vec![y]);
                    let positive = call(&mut c, "Mpk.Go.Practical.Nat.Mul", vec![px, py]);
                    let pny = call(&mut c, "Mpk.Go.Practical.Nat.Mul", vec![px, ny]);
                    let npy = call(&mut c, "Mpk.Go.Practical.Nat.Mul", vec![nx, py]);
                    let negative = call(&mut c, "Mpk.Go.Practical.Nat.Add", vec![pny, npy]);
                    (
                        "Mul",
                        call(
                            &mut c,
                            "Mpk.Go.Practical.Int.Make",
                            vec![positive, negative],
                        ),
                    )
                };
                let value = lam(&mut c, int, value);
                (name, lam(&mut c, int, value), true)
            }
            _ => {
                let name = match mutation {
                    "transitive-premise" => "Transitive",
                    "addition-premise" => "AddRespects",
                    "equal-respects-premise" => "EqualRespects",
                    "less-equal-respects-premise" => "LessEqualRespects",
                    _ => "FromNatInjective",
                };
                let index = global(&c, &format!("Mpk.Go.Practical.Int.{name}")) as usize;
                let DeclarationKind::Theorem { ty, .. } = c.declarations[index].kind else {
                    panic!();
                };
                let mut cursor = ty;
                let mut binders = vec![];
                while let TermNode::Pi { ty, body } = c.term_table[cursor as usize] {
                    binders.push(ty);
                    cursor = body;
                }
                let value = if matches!(mutation, "transitive-premise" | "addition-premise") {
                    term(&mut c, TermNode::Var(1))
                } else if mutation == "nat-injection-premise" {
                    let a = term(&mut c, TermNode::Var(2));
                    call(&mut c, "Std.Eq.refl", vec![nat, a])
                } else {
                    let yes = constant(&mut c, "Std.Bool.true");
                    call(&mut c, "Std.Eq.refl", vec![boolean, yes])
                };
                let mut value = value;
                for ty in binders.into_iter().rev() {
                    value = lam(&mut c, ty, value);
                }
                (name, value, false)
            }
        };
        let index = global(&c, &format!("Mpk.Go.Practical.Int.{name}")) as usize;
        match &mut c.declarations[index].kind {
            DeclarationKind::Def { value: v, .. } if is_definition => *v = value,
            DeclarationKind::Theorem { proof, .. } if !is_definition => *proof = value,
            _ => panic!(),
        }
        let bytes = encode(c);
        assert_eq!(
            import_go_practical_integer_foundation(&bytes),
            Err(GoOrdinaryError::Linkage)
        );
        emit(mutation, &bytes, false);
    }
}
