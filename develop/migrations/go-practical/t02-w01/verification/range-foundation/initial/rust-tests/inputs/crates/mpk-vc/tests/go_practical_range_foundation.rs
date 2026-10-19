use mpk_cert::encode::{
    Certificate, Declaration, DeclarationKind, DefinitionReducibility, LevelNode, TermNode,
};
use mpk_vc::go_practical_ordinary::{
    generate_go_practical_integer_foundation, generate_go_practical_range_foundation,
    import_go_practical_range_foundation, GoOrdinaryError,
};

fn emit(name: &str, bytes: &[u8], accepted: bool) {
    let output = mpk_kernel::verify_certificate_bytes_json_output(bytes);
    if let Some(root) = std::env::var_os("MPK_GO_RANGE_FOUNDATION_TEST_OUTPUT") {
        use std::io::Write;
        let root = std::path::PathBuf::from(root);
        std::fs::create_dir_all(&root).unwrap();
        for (ext, raw) in [
            ("mpcert", bytes.to_vec()),
            ("rust.json", format!("{}\n", output.json).into_bytes()),
        ] {
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(root.join(format!("{name}.{ext}")))
                .unwrap();
            file.write_all(&raw).unwrap();
        }
    }
    assert_eq!(output.accepted, accepted, "{name}: {}", output.json);
    if !accepted {
        let r: serde_json::Value = serde_json::from_str(&output.json).unwrap();
        assert_eq!(r["error_code"], "KERNEL_CORE_CHECK", "{name}");
        let e: serde_json::Value =
            serde_json::from_str(r["error_detail"].as_str().unwrap()).unwrap();
        assert_eq!(e["code"], "CORE_TYPE_MISMATCH", "{name}");
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
fn app(c: &mut Certificate, function: u32, arguments: Vec<u32>) -> u32 {
    term(
        c,
        TermNode::App {
            function,
            arguments,
        },
    )
}
fn call(c: &mut Certificate, name: &str, args: Vec<u32>) -> u32 {
    let f = constant(c, name);
    app(c, f, args)
}
fn rc(c: &mut Certificate, name: &str, args: Vec<u32>) -> u32 {
    call(c, &format!("Mpk.Go.Practical.Range.{name}"), args)
}
fn nc(c: &mut Certificate, name: &str, args: Vec<u32>) -> u32 {
    call(c, &format!("Mpk.Go.Practical.Nat.{name}"), args)
}
fn bc(c: &mut Certificate, name: &str, args: Vec<u32>) -> u32 {
    call(c, &format!("Mpk.Go.Practical.Bool.{name}"), args)
}
fn ic(c: &mut Certificate, name: &str, args: Vec<u32>) -> u32 {
    call(c, &format!("Mpk.Go.Practical.Int.{name}"), args)
}
fn lam(c: &mut Certificate, ty: u32, body: u32) -> u32 {
    term(c, TermNode::Lam { ty, body })
}
fn var(c: &mut Certificate, n: u32) -> u32 {
    term(c, TermNode::Var(n))
}
fn natural(c: &mut Certificate, n: u32) -> u32 {
    let mut v = constant(c, "Std.Nat.zero");
    for _ in 0..n {
        v = call(c, "Std.Nat.succ", vec![v]);
    }
    v
}
fn integer(c: &mut Certificate) -> u32 {
    let ty = constant(c, "Std.Bool");
    let body = constant(c, "Std.Nat");
    term(c, TermNode::Pi { ty, body })
}
fn declare(c: &mut Certificate, name: &str, kind: DeclarationKind) {
    let id = c.name_table.len() as u32;
    c.name_table.push(name.into());
    c.declarations.push(Declaration { name: id, kind });
}
fn equation(c: &mut Certificate, name: &str, e: Equation) {
    let ty = call(c, "Std.Eq", vec![e.ty, e.left, e.right]);
    declare(c, name, DeclarationKind::Theorem { ty, proof: e.proof });
}
fn encode(mut c: Certificate) -> Vec<u8> {
    let previous = c.name_table.clone();
    c.name_table.sort();
    for d in &mut c.declarations {
        d.name = c
            .name_table
            .binary_search(&previous[d.name as usize])
            .unwrap() as u32;
    }
    for l in &mut c.level_table {
        if let LevelNode::Param(name) = l {
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
        generate_go_practical_range_foundation()
            .unwrap()
            .certificate_bytes(),
    )
    .unwrap()
}

#[derive(Clone, Copy)]
struct Equation {
    ty: u32,
    left: u32,
    right: u32,
    proof: u32,
}
impl Equation {
    fn then(self, c: &mut Certificate, next: Self) -> Self {
        assert_eq!(self.ty, next.ty);
        let proof = call(
            c,
            "Std.Eq.trans",
            vec![
                self.ty, self.left, self.right, next.right, self.proof, next.proof,
            ],
        );
        Self {
            ty: self.ty,
            left: self.left,
            right: next.right,
            proof,
        }
    }
    fn mapped(self, c: &mut Certificate, ty: u32, function: u32) -> Self {
        let left = app(c, function, vec![self.left]);
        let right = app(c, function, vec![self.right]);
        let proof = call(
            c,
            "Std.Eq.congr",
            vec![self.ty, ty, function, self.left, self.right, self.proof],
        );
        Self {
            ty,
            left,
            right,
            proof,
        }
    }
}
fn subtraction(c: &mut Certificate, nat: u32, a: u32, b: u32) -> Equation {
    let av = natural(c, a);
    let bv = natural(c, b);
    let left = nc(c, "Sub", vec![av, bv]);
    let right = natural(c, a.saturating_sub(b));
    if b == 0 {
        return Equation {
            ty: nat,
            left,
            right,
            proof: nc(c, "SubZero", vec![av]),
        };
    }
    if a == 0 {
        return Equation {
            ty: nat,
            left,
            right,
            proof: nc(c, "ZeroSub", vec![bv]),
        };
    }
    let ap = natural(c, a - 1);
    let bp = natural(c, b - 1);
    let tail = subtraction(c, nat, a - 1, b - 1);
    Equation {
        ty: nat,
        left,
        right: tail.left,
        proof: nc(c, "SubSuccBoth", vec![ap, bp]),
    }
    .then(c, tail)
}

#[test]
fn actual_unbounded_half_open_folds_and_all_bounded_laws_check_without_axioms() {
    let f = generate_go_practical_range_foundation().unwrap();
    let c = mpk_cert::decode_canonical_certificate(f.certificate_bytes()).unwrap();
    assert!(
        c.imports.is_empty()
            && c.source_manifest.is_none()
            && c.proof_node_table.is_empty()
            && c.theory_certificates.is_empty()
    );
    assert_eq!(c.axiom_report.summary.total_axiom_count, 0);
    for family in ["Forall", "Exists", "NatSum", "IntSum"] {
        for suffix in [
            "",
            "Zero",
            "Step",
            "SameBounds",
            "Empty",
            "Respects",
            "BoundedRespects",
            "IntervalRespects",
        ] {
            global(&c, &format!("Mpk.Go.Practical.Range.{family}{suffix}"));
        }
        global(&c, &format!("Mpk.Go.Practical.Range.Fold{family}"));
    }
    emit("range-foundation", f.certificate_bytes(), true);
    println!(
        "bytes={} terms={} declarations={}",
        f.certificate_bytes().len(),
        c.term_table.len(),
        c.declarations.len()
    );
}

#[test]
fn exact_range_import_preserves_integer_bytes_and_rejects_other_valid_components() {
    let f = generate_go_practical_range_foundation().unwrap();
    assert_eq!(f, generate_go_practical_range_foundation().unwrap());
    assert_eq!(
        f,
        import_go_practical_range_foundation(f.certificate_bytes()).unwrap()
    );
    let mut altered = f.certificate_bytes().to_vec();
    altered.push(0);
    assert_eq!(
        import_go_practical_range_foundation(&altered),
        Err(GoOrdinaryError::Linkage)
    );
    assert_eq!(
        import_go_practical_range_foundation(&vec![0; 2_097_153]),
        Err(GoOrdinaryError::Limit)
    );
    let retained = generate_go_practical_integer_foundation().unwrap();
    let r = mpk_kernel::verify_certificate_bytes_json_output(retained.certificate_bytes());
    assert!(
        r.accepted
            && r.json
                .contains("20336bfcfe4ad0e03c81bf38f35d2af86b5cf669496064387d70d58ae1a152e5")
    );
    emit("retained-integer-basis", retained.certificate_bytes(), true);
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
    let bytes = encode(c);
    emit("honest-different-range-component", &bytes, true);
    assert_eq!(
        import_go_practical_range_foundation(&bytes),
        Err(GoOrdinaryError::Linkage)
    );
}

#[test]
fn concrete_half_open_zero_nonempty_equal_and_reversed_intervals_check() {
    let mut c = baseline();
    let nat = constant(&mut c, "Std.Nat");
    let boolean = constant(&mut c, "Std.Bool");
    let yes = constant(&mut c, "Std.Bool.true");
    let no = constant(&mut c, "Std.Bool.false");
    for (i, (lower, upper)) in [(0, 0), (0, 3), (2, 5), (4, 4), (6, 2)]
        .into_iter()
        .enumerate()
    {
        for (j, (family, ty, value)) in [
            ("Forall", boolean, yes),
            ("Forall", boolean, no),
            ("Exists", boolean, yes),
            ("Exists", boolean, no),
            ("NatSum", nat, natural(&mut c, 1)),
        ]
        .into_iter()
        .enumerate()
        {
            let l = natural(&mut c, lower);
            let u = natural(&mut c, upper);
            let n = upper.saturating_sub(lower);
            let body = lam(&mut c, nat, value);
            let count = subtraction(&mut c, nat, upper, lower);
            let x = var(&mut c, 0);
            let fold = rc(&mut c, &format!("Fold{family}"), vec![l, x, body]);
            let function = lam(&mut c, nat, fold);
            let mapped = count.mapped(&mut c, ty, function);
            let left = rc(&mut c, family, vec![l, u, body]);
            let expected = match family {
                "Forall" if n == 0 => yes,
                "Exists" if n == 0 => no,
                "NatSum" => natural(&mut c, n),
                _ => value,
            };
            let proof = call(&mut c, "Std.Eq.refl", vec![ty, expected]);
            let tail = Equation {
                ty,
                left: mapped.right,
                right: expected,
                proof,
            };
            let e = Equation {
                ty,
                left,
                right: mapped.right,
                proof: mapped.proof,
            }
            .then(&mut c, tail);
            equation(&mut c, &format!("Mpk.Go.Practical.Test.Interval{i}{j}"), e);
        }
    }
    // Integer sum preserves semantic equivalence, including zero over a
    // nonempty prefix, without assuming core equality of representatives.
    let zero = constant(&mut c, "Mpk.Go.Practical.Int.Zero");
    let body = lam(&mut c, nat, zero);
    let lower = natural(&mut c, 2);
    let mut previous = ic(&mut c, "Reflexive", vec![zero]);
    for n in 0..3 {
        let count = natural(&mut c, n);
        let next = natural(&mut c, n + 1);
        let old = rc(&mut c, "FoldIntSum", vec![lower, count, body]);
        let folded = rc(&mut c, "FoldIntSum", vec![lower, next, body]);
        let p = ic(&mut c, "Reflexive", vec![zero]);
        let p = ic(
            &mut c,
            "AddRespects",
            vec![old, zero, zero, zero, previous, p],
        );
        let middle = ic(&mut c, "Add", vec![zero, zero]);
        let q = ic(&mut c, "AddZero", vec![zero]);
        previous = ic(&mut c, "Transitive", vec![folded, middle, zero, p, q]);
    }
    let three = natural(&mut c, 3);
    let folded = rc(&mut c, "FoldIntSum", vec![lower, three, body]);
    let ty = ic(&mut c, "Equivalent", vec![folded, zero]);
    declare(
        &mut c,
        "Mpk.Go.Practical.Test.IntegerSumZero",
        DeclarationKind::Theorem {
            ty,
            proof: previous,
        },
    );
    emit("concrete-intervals", &encode(c), true);
}

#[test]
fn bounded_premises_allow_bodies_that_differ_outside_the_visited_prefix() {
    let mut c = baseline();
    let nat = constant(&mut c, "Std.Nat");
    let boolean = constant(&mut c, "Std.Bool");
    let yes = constant(&mut c, "Std.Bool.true");
    let no = constant(&mut c, "Std.Bool.false");
    let zero = natural(&mut c, 0);
    let one = natural(&mut c, 1);
    let f = lam(&mut c, nat, yes);
    let g = constant(&mut c, "Mpk.Go.Practical.Nat.IsZero");
    let x = var(&mut c, 0);
    let fx = app(&mut c, f, vec![x]);
    let gx = app(&mut c, g, vec![x]);
    let comparator = bc(&mut c, "Equal", vec![fx, gx]);
    let predicate = lam(&mut c, nat, comparator);
    let position = nc(&mut c, "Add", vec![zero, zero]);
    let proof = nc(&mut c, "AddZero", vec![zero]);
    let position = Equation {
        ty: nat,
        left: position,
        right: zero,
        proof,
    };
    let gp = position.mapped(&mut c, boolean, g);
    let proof = call(&mut c, "Std.Eq.refl", vec![boolean, yes]);
    let tail = Equation {
        ty: boolean,
        left: gp.right,
        right: yes,
        proof,
    };
    let gp = gp.then(&mut c, tail);
    let x = var(&mut c, 0);
    let body = bc(&mut c, "Equal", vec![yes, x]);
    let function = lam(&mut c, boolean, body);
    let compared = gp.mapped(&mut c, boolean, function);
    let proof = bc(&mut c, "EqualReflexive", vec![yes]);
    let tail = Equation {
        ty: boolean,
        left: compared.right,
        right: yes,
        proof,
    };
    let compared = compared.then(&mut c, tail);
    let position = nc(&mut c, "Add", vec![zero, zero]);
    let at_position = app(&mut c, predicate, vec![position]);
    let compared = Equation {
        ty: boolean,
        left: at_position,
        right: yes,
        proof: compared.proof,
    };
    let fold0 = rc(&mut c, "FoldForall", vec![zero, zero, predicate]);
    let fold1 = rc(&mut c, "FoldForall", vec![zero, one, predicate]);
    let combined = bc(&mut c, "And", vec![fold0, at_position]);
    let step = rc(&mut c, "ForallStep", vec![zero, zero, predicate]);
    let start = Equation {
        ty: boolean,
        left: fold1,
        right: combined,
        proof: step,
    };
    let base = Equation {
        ty: boolean,
        left: fold0,
        right: yes,
        proof: rc(&mut c, "ForallZero", vec![zero, predicate]),
    };
    let x = var(&mut c, 0);
    let body = bc(&mut c, "And", vec![x, at_position]);
    let function = lam(&mut c, boolean, body);
    let base = base.mapped(&mut c, boolean, function);
    let start = start.then(&mut c, base);
    let x = var(&mut c, 0);
    let body = bc(&mut c, "And", vec![yes, x]);
    let function = lam(&mut c, boolean, body);
    let compared = compared.mapped(&mut c, boolean, function);
    let start = start.then(&mut c, compared);
    let proof = bc(&mut c, "AndTrueRight", vec![yes]);
    let tail = Equation {
        ty: boolean,
        left: start.right,
        right: yes,
        proof,
    };
    let bounded = start.then(&mut c, tail);
    equation(
        &mut c,
        "Mpk.Go.Practical.Test.ActualBoundedPremise",
        Equation {
            ty: boolean,
            left: bounded.left,
            right: yes,
            proof: bounded.proof,
        },
    );
    let length = subtraction(&mut c, nat, 1, 0);
    let x = var(&mut c, 0);
    let fold = rc(&mut c, "FoldForall", vec![zero, x, predicate]);
    let function = lam(&mut c, nat, fold);
    let length = length.mapped(&mut c, boolean, function);
    let interval = length.then(&mut c, bounded);
    for family in ["Forall", "Exists"] {
        let left = rc(&mut c, &format!("Fold{family}"), vec![zero, one, f]);
        let right = rc(&mut c, &format!("Fold{family}"), vec![zero, one, g]);
        let proof = rc(
            &mut c,
            &format!("{family}BoundedRespects"),
            vec![zero, one, f, g, bounded.proof],
        );
        equation(
            &mut c,
            &format!("Mpk.Go.Practical.Test.{family}BoundedOnly"),
            Equation {
                ty: boolean,
                left,
                right,
                proof,
            },
        );
        let left = rc(&mut c, family, vec![zero, one, f]);
        let right = rc(&mut c, family, vec![zero, one, g]);
        let proof = rc(
            &mut c,
            &format!("{family}IntervalRespects"),
            vec![zero, one, f, g, interval.proof],
        );
        equation(
            &mut c,
            &format!("Mpk.Go.Practical.Test.{family}IntervalOnly"),
            Equation {
                ty: boolean,
                left,
                right,
                proof,
            },
        );
    }
    let left = app(&mut c, f, vec![one]);
    let proof = call(&mut c, "Std.Eq.refl", vec![boolean, yes]);
    equation(
        &mut c,
        "Mpk.Go.Practical.Test.OutsideLeftTrue",
        Equation {
            ty: boolean,
            left,
            right: yes,
            proof,
        },
    );
    let left = app(&mut c, g, vec![one]);
    let proof = call(&mut c, "Std.Eq.refl", vec![boolean, no]);
    equation(
        &mut c,
        "Mpk.Go.Practical.Test.OutsideRightFalse",
        Equation {
            ty: boolean,
            left,
            right: no,
            proof,
        },
    );
    emit("bounded-only-premise", &encode(c), true);
}

#[test]
fn reencoded_wrong_folds_bounds_and_unrelated_or_forged_premises_fail_in_core() {
    let original = baseline();
    for mutation in [
        "forall-zero",
        "exists-zero",
        "natsum-body",
        "intsum-body",
        "inclusive-upper",
        "ignored-lower",
        "empty-proof",
        "bounded-natsum-proof",
        "bounded-intsum-function-equality",
        "bool-reflect-proof",
    ] {
        let mut c = original.clone();
        let nat = constant(&mut c, "Std.Nat");
        let boolean = constant(&mut c, "Std.Bool");
        let int = integer(&mut c);
        let name = match mutation {
            "forall-zero" => "Range.FoldForall",
            "exists-zero" => "Range.FoldExists",
            "natsum-body" | "ignored-lower" => "Range.FoldNatSum",
            "intsum-body" => "Range.FoldIntSum",
            "inclusive-upper" => "Range.Forall",
            "empty-proof" => "Range.ExistsEmpty",
            "bounded-natsum-proof" => "Range.NatSumBoundedRespects",
            "bounded-intsum-function-equality" => "Range.IntSumBoundedRespects",
            _ => "Bool.EqualReflect",
        };
        let index = global(&c, &format!("Mpk.Go.Practical.{name}")) as usize;
        let (ty, is_def) = match c.declarations[index].kind {
            DeclarationKind::Def { ty, .. } => (ty, true),
            DeclarationKind::Theorem { ty, .. } => (ty, false),
            _ => panic!(),
        };
        let mut cursor = ty;
        let mut binders = vec![];
        while let TermNode::Pi { ty, body } = c.term_table[cursor as usize] {
            binders.push(ty);
            cursor = body;
        }
        if mutation == "bounded-intsum-function-equality" {
            let lower = var(&mut c, 4);
            let count = var(&mut c, 3);
            let f = var(&mut c, 2);
            let g = var(&mut c, 1);
            let left = rc(&mut c, "FoldIntSum", vec![lower, count, f]);
            let right = rc(&mut c, "FoldIntSum", vec![lower, count, g]);
            let mut altered = call(&mut c, "Std.Eq", vec![int, left, right]);
            for ty in binders.into_iter().rev() {
                altered = term(&mut c, TermNode::Pi { ty, body: altered });
            }
            if let DeclarationKind::Theorem { ty, .. } = &mut c.declarations[index].kind {
                *ty = altered;
            } else {
                panic!();
            }
        } else {
            let body = match mutation {
                "forall-zero" => constant(&mut c, "Std.Bool.false"),
                "exists-zero" => constant(&mut c, "Std.Bool.true"),
                "natsum-body" => natural(&mut c, 0),
                "intsum-body" => constant(&mut c, "Mpk.Go.Practical.Int.Zero"),
                "inclusive-upper" => {
                    let l = var(&mut c, 2);
                    let u = var(&mut c, 1);
                    let f = var(&mut c, 0);
                    let n = nc(&mut c, "Sub", vec![u, l]);
                    let n = call(&mut c, "Std.Nat.succ", vec![n]);
                    rc(&mut c, "FoldForall", vec![l, n, f])
                }
                "ignored-lower" => {
                    let f = var(&mut c, 2);
                    let i = var(&mut c, 1);
                    let acc = var(&mut c, 0);
                    let value = app(&mut c, f, vec![i]);
                    let step = nc(&mut c, "Add", vec![acc, value]);
                    let step = lam(&mut c, nat, step);
                    let step = lam(&mut c, nat, step);
                    let motive = lam(&mut c, nat, nat);
                    let zero = natural(&mut c, 0);
                    let count = var(&mut c, 1);
                    call(&mut c, "Std.Nat.induct", vec![motive, zero, step, count])
                }
                "empty-proof" => {
                    let no = constant(&mut c, "Std.Bool.false");
                    call(&mut c, "Std.Eq.refl", vec![boolean, no])
                }
                "bounded-natsum-proof" => {
                    let l = var(&mut c, 4);
                    let n = var(&mut c, 3);
                    let f = var(&mut c, 2);
                    let value = rc(&mut c, "FoldNatSum", vec![l, n, f]);
                    call(&mut c, "Std.Eq.refl", vec![nat, value])
                }
                _ => {
                    let a = var(&mut c, 2);
                    call(&mut c, "Std.Eq.refl", vec![boolean, a])
                }
            };
            let mut value = body;
            for ty in binders.into_iter().rev() {
                value = lam(&mut c, ty, value);
            }
            match &mut c.declarations[index].kind {
                DeclarationKind::Def { value: v, .. } if is_def => *v = value,
                DeclarationKind::Theorem { proof, .. } if !is_def => *proof = value,
                _ => panic!(),
            };
        }
        let bytes = encode(c);
        assert_eq!(
            import_go_practical_range_foundation(&bytes),
            Err(GoOrdinaryError::Linkage)
        );
        emit(mutation, &bytes, false);
    }
    let mut c = original;
    let nat = constant(&mut c, "Std.Nat");
    let boolean = constant(&mut c, "Std.Bool");
    let yes = constant(&mut c, "Std.Bool.true");
    let no = constant(&mut c, "Std.Bool.false");
    let zero = natural(&mut c, 0);
    let one = natural(&mut c, 1);
    let f = lam(&mut c, nat, yes);
    let g = lam(&mut c, nat, no);
    let left = rc(&mut c, "FoldForall", vec![zero, one, f]);
    let right = rc(&mut c, "FoldForall", vec![zero, one, g]);
    let premise = call(&mut c, "Std.Eq.refl", vec![boolean, yes]);
    let proof = rc(
        &mut c,
        "ForallBoundedRespects",
        vec![zero, one, f, g, premise],
    );
    equation(
        &mut c,
        "Mpk.Go.Practical.Test.ForgedBoundedPremise",
        Equation {
            ty: boolean,
            left,
            right,
            proof,
        },
    );
    emit("forged-bounded-premise", &encode(c), false);
}
