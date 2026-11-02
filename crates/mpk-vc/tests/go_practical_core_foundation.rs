use mpk_cert::encode::{
    Certificate, Declaration, DeclarationKind, DefinitionReducibility, LevelNode, TermNode,
};
use mpk_vc::go_practical_ordinary::{
    generate_go_practical_boolean_foundation, generate_go_practical_core_foundation,
    import_go_practical_core_foundation, GoOrdinaryError,
};

fn emit(name: &str, bytes: &[u8], accepted: bool) {
    let checked = mpk_kernel::verify_certificate_bytes_json_output(bytes);
    assert_eq!(checked.accepted, accepted, "{name}: {}", checked.json);
    if let Some(root) = std::env::var_os("MPK_GO_CORE_FOUNDATION_TEST_OUTPUT") {
        use std::io::Write;
        let root = std::path::PathBuf::from(root);
        std::fs::create_dir_all(&root).unwrap();
        for (extension, data) in [
            ("mpcert", bytes.to_vec()),
            ("rust.json", format!("{}\n", checked.json).into_bytes()),
        ] {
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(root.join(format!("{name}.{extension}")))
                .unwrap();
            file.write_all(&data).unwrap();
        }
    }
}

fn global(c: &Certificate, name: &str) -> u32 {
    c.declarations
        .iter()
        .position(|d| c.name_table[d.name as usize] == name)
        .unwrap() as u32
}

fn term(c: &mut Certificate, node: TermNode) -> u32 {
    let index = c.term_table.len() as u32;
    c.term_table.push(node);
    index
}

fn constant(c: &mut Certificate, name: &str, levels: Vec<u32>) -> u32 {
    let global = global(c, name);
    term(c, TermNode::Const { global, levels })
}

fn call(c: &mut Certificate, name: &str, arguments: Vec<u32>) -> u32 {
    let function = constant(c, name, vec![]);
    term(
        c,
        TermNode::App {
            function,
            arguments,
        },
    )
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

fn insert(c: &mut Certificate, index: usize, name: &str, kind: DeclarationKind) {
    for node in &mut c.term_table {
        if let TermNode::Const { global, .. } = node {
            if *global >= index as u32 {
                *global += 1;
            }
        }
    }
    for declaration in &mut c.declarations {
        match &mut declaration.kind {
            DeclarationKind::Constructor { inductive, .. }
            | DeclarationKind::Recursor { inductive, .. }
                if *inductive >= index as u32 =>
            {
                *inductive += 1
            }
            _ => {}
        }
    }
    let name_id = c.name_table.len() as u32;
    c.name_table.push(name.into());
    c.declarations.insert(
        index,
        Declaration {
            name: name_id,
            kind,
        },
    );
}

#[test]
fn actual_core_foundation_proves_unbounded_identity_and_conditional_false_elimination() {
    let foundation = generate_go_practical_core_foundation().unwrap();
    let c = mpk_cert::decode_canonical_certificate(foundation.certificate_bytes()).unwrap();
    assert!(
        c.imports.is_empty()
            && c.proof_node_table.is_empty()
            && c.theory_certificates.is_empty()
            && c.source_manifest.is_none()
    );
    assert_eq!(c.axiom_report.summary.total_axiom_count, 0);
    for name in [
        "Std.Nat.induct",
        "Std.Bool.recType",
        "Mpk.Go.Practical.Nat.Identity",
        "Mpk.Go.Practical.Nat.IdentityLaw",
        "Mpk.Go.Practical.Bool.FalseElim",
    ] {
        global(&c, name);
    }
    emit("core-foundation", foundation.certificate_bytes(), true);
    println!(
        "bytes={} terms={} declarations={}",
        foundation.certificate_bytes().len(),
        c.term_table.len(),
        c.declarations.len()
    );
}

#[test]
fn exact_core_import_regenerates_complete_bytes_and_preserves_existing_boolean_basis() {
    let first = generate_go_practical_core_foundation().unwrap();
    assert_eq!(first, generate_go_practical_core_foundation().unwrap());
    assert_eq!(
        first,
        import_go_practical_core_foundation(first.certificate_bytes()).unwrap()
    );
    let mut changed = first.certificate_bytes().to_vec();
    changed.push(0);
    assert_eq!(
        import_go_practical_core_foundation(&changed),
        Err(GoOrdinaryError::Linkage)
    );
    let boolean = generate_go_practical_boolean_foundation().unwrap();
    emit("retained-boolean-basis", boolean.certificate_bytes(), true);
    let output = mpk_kernel::verify_certificate_bytes_json_output(boolean.certificate_bytes());
    assert!(output
        .json
        .contains("f3ee9e0c4d0286bb1cce42a4d2146b2098da5b818ffe350cd5d612c3653690fd"));
}

#[test]
fn malformed_interfaces_and_unproved_false_claims_are_rejected_without_axioms() {
    let original = generate_go_practical_core_foundation().unwrap();
    let baseline = mpk_cert::decode_canonical_certificate(original.certificate_bytes()).unwrap();
    for mutation in [
        "nat-type",
        "bool-type",
        "nat-flag",
        "bool-flag",
        "nat-alias",
        "bool-alias",
        "nat-extra-before",
        "nat-extra-after",
        "nat-prior-opaque",
        "nat-prior-type",
        "nat-levels",
        "bool-levels",
        "nat-order",
        "false-equality",
        "false-nat-law",
    ] {
        let mut c = baseline.clone();
        let nat = constant(&mut c, "Std.Nat", vec![]);
        match mutation {
            "nat-type" | "bool-type" => {
                let name = if mutation == "nat-type" {
                    "Std.Nat.induct"
                } else {
                    "Std.Bool.recType"
                };
                let index = global(&c, name) as usize;
                if let DeclarationKind::Recursor { ty, .. } = &mut c.declarations[index].kind {
                    *ty = nat;
                } else {
                    panic!();
                }
            }
            "nat-flag" | "bool-flag" => {
                let name = if mutation == "nat-flag" {
                    "Std.Nat.induct"
                } else {
                    "Std.Bool.recType"
                };
                let index = global(&c, name) as usize;
                if let DeclarationKind::Recursor { generated, .. } = &mut c.declarations[index].kind
                {
                    *generated = false;
                } else {
                    panic!();
                }
            }
            "nat-alias" | "bool-alias" => {
                let name = if mutation == "nat-alias" {
                    "Std.Nat.induct"
                } else {
                    "Std.Bool.recType"
                };
                let index = global(&c, name) as usize;
                c.name_table[c.declarations[index].name as usize] = if mutation == "nat-alias" {
                    "Other.Nat.induct"
                } else {
                    "Other.Bool.recType"
                }
                .into();
            }
            "nat-extra-before" | "nat-extra-after" => {
                let family = global(&c, "Std.Nat");
                let index = if mutation == "nat-extra-before" {
                    global(&c, "Std.Nat.induct") as usize
                } else {
                    c.declarations.len()
                };
                insert(
                    &mut c,
                    index,
                    "Mpk.Go.Practical.Test.ExtraNat",
                    DeclarationKind::Constructor {
                        ty: nat,
                        inductive: family,
                        generated: true,
                    },
                );
            }
            "nat-prior-opaque" | "nat-prior-type" => {
                let value = constant(&mut c, "Std.Nat.zero", vec![0]);
                let ty = if mutation == "nat-prior-type" {
                    constant(&mut c, "Std.Nat", vec![0])
                } else {
                    nat
                };
                let index = global(&c, "Std.Nat.induct") as usize;
                insert(
                    &mut c,
                    index,
                    "Mpk.Go.Practical.Test.PriorNat",
                    DeclarationKind::Def {
                        ty,
                        value,
                        reducibility: DefinitionReducibility::Opaque,
                    },
                );
            }
            "nat-levels" | "bool-levels" => {
                let id = global(
                    &c,
                    if mutation == "nat-levels" {
                        "Std.Nat.induct"
                    } else {
                        "Std.Bool.recType"
                    },
                );
                for node in &mut c.term_table {
                    if let TermNode::Const { global, levels } = node {
                        if *global == id {
                            *levels = vec![0];
                        }
                    }
                }
            }
            "nat-order" => {
                let a = c.declarations[global(&c, "Std.Nat.zero") as usize].name as usize;
                let b = c.declarations[global(&c, "Std.Nat.succ") as usize].name as usize;
                c.name_table.swap(a, b);
            }
            "false-equality" => {
                let boolean = constant(&mut c, "Std.Bool", vec![]);
                let no = constant(&mut c, "Std.Bool.false", vec![]);
                let yes = constant(&mut c, "Std.Bool.true", vec![]);
                let ty = call(&mut c, "Std.Eq", vec![boolean, no, yes]);
                let proof = call(&mut c, "Std.Eq.refl", vec![boolean, no]);
                let index = global(&c, "Mpk.Go.Practical.Bool.FalseElim") as usize;
                c.declarations[index].kind = DeclarationKind::Theorem { ty, proof };
            }
            "false-nat-law" => {
                let n = term(&mut c, TermNode::Var(0));
                let next = call(&mut c, "Std.Nat.succ", vec![n]);
                let property = call(&mut c, "Std.Eq", vec![nat, next, n]);
                let ty = term(
                    &mut c,
                    TermNode::Pi {
                        ty: nat,
                        body: property,
                    },
                );
                let refl = call(&mut c, "Std.Eq.refl", vec![nat, n]);
                let proof = term(
                    &mut c,
                    TermNode::Lam {
                        ty: nat,
                        body: refl,
                    },
                );
                let index = global(&c, "Mpk.Go.Practical.Nat.IdentityLaw") as usize;
                c.declarations[index].kind = DeclarationKind::Theorem { ty, proof };
            }
            _ => unreachable!(),
        }
        let bytes = encode(c);
        assert_eq!(
            import_go_practical_core_foundation(&bytes),
            Err(GoOrdinaryError::Linkage)
        );
        emit(mutation, &bytes, false);
    }
}

#[test]
fn unused_monomorphic_interface_nodes_do_not_create_actual_uses() {
    let foundation = generate_go_practical_core_foundation().unwrap();
    let mut c = mpk_cert::decode_canonical_certificate(foundation.certificate_bytes()).unwrap();
    constant(&mut c, "Std.Nat.zero", vec![0]);
    constant(&mut c, "Std.Bool.recType", vec![0]);
    let bytes = encode(c);
    assert_eq!(
        import_go_practical_core_foundation(&bytes),
        Err(GoOrdinaryError::Linkage)
    );
    emit("unused-interface-nodes", &bytes, true);
}
