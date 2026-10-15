//! Exact local value conservation in genuine Std.Nat, not just a carry flag.
//! This is one compositional ingredient, not the complete 64-bit sum bridge.

use super::*;

fn nat() -> Expr {
    c("Std.Nat")
}
fn equal(left: Expr, right: Expr) -> Expr {
    apply(c(EQ), vec![nat(), left, right])
}
fn reflexive(value: Expr) -> Expr {
    apply(c(REFL), vec![nat(), value])
}
fn trans(left: Expr, middle: Expr, right: Expr, p: Expr, q: Expr) -> Expr {
    apply(c("Std.Eq.trans"), vec![nat(), left, middle, right, p, q])
}
fn congr(domain: Expr, function: Expr, left: Expr, right: Expr, proof: Expr) -> Expr {
    apply(
        c("Std.Eq.congr"),
        vec![domain, nat(), function, left, right, proof],
    )
}

fn normalize(expr: &Expr, fresh: &mut usize) -> (Expr, Expr) {
    let beta = weak_beta(expr);
    let beta_proof = reflexive(beta.clone());
    let Expr::Apply(ref head, ref args) = beta else {
        return (beta, beta_proof);
    };
    let (normal, proof) = if **head == c("Std.Bool.cases") && args.len() == 4 {
        let (condition, condition_proof) = normalize_ground_boolean(&args[3], fresh);
        let branch = if condition == literal(false) {
            1
        } else if condition == literal(true) {
            2
        } else {
            panic!("natural pilot has an uneliminated condition");
        };
        let name = format!("__nat_condition_{}", *fresh);
        *fresh += 1;
        let mut context = args.clone();
        context[3] = v(&name);
        let mut rewritten_args = args.clone();
        rewritten_args[3] = condition.clone();
        let rewritten = apply(c("Std.Bool.cases"), rewritten_args);
        let rewrite = congr(
            bool_ty(),
            lambda(&name, bool_ty(), apply(c("Std.Bool.cases"), context)),
            args[3].clone(),
            condition,
            condition_proof,
        );
        let selected = args[branch].clone();
        let (normal, tail) = normalize(&selected, fresh);
        let select = reflexive(selected.clone());
        let tail = trans(rewritten.clone(), selected, normal.clone(), select, tail);
        (
            normal.clone(),
            trans(beta.clone(), rewritten, normal, rewrite, tail),
        )
    } else if args.len() == 1 {
        let (argument, proof) = normalize(&args[0], fresh);
        let normal = apply((**head).clone(), vec![argument.clone()]);
        let proof = congr(nat(), (**head).clone(), args[0].clone(), argument, proof);
        (normal, proof)
    } else {
        panic!("unexpected natural expression: {beta:?}");
    };
    if &beta == expr {
        (normal, proof)
    } else {
        (
            normal.clone(),
            trans(expr.clone(), beta, normal, beta_proof, proof),
        )
    }
}

fn prove(goal: &Expr, majors: &[Expr], fresh: &mut usize) -> Expr {
    if let Some((major, rest)) = majors.split_first() {
        let name = format!("__nat_case_{}", *fresh);
        *fresh += 1;
        return apply(
            c("Std.Bool.cases"),
            vec![
                lambda(&name, bool_ty(), substitute(goal, major, &v(&name))),
                prove(&substitute(goal, major, &literal(false)), rest, fresh),
                prove(&substitute(goal, major, &literal(true)), rest, fresh),
                major.clone(),
            ],
        );
    }
    let Expr::Apply(_, args) = goal else {
        panic!("expected natural equality");
    };
    let (left, p) = normalize(&args[1], fresh);
    let (right, q) = normalize(&args[2], fresh);
    if left != right {
        // Well-formed candidate for a deliberately false claim. The checkers,
        // not this producer's comparison, decide whether the proof is valid.
        return reflexive(args[2].clone());
    }
    let reverse = apply(c("Std.Eq.symm"), vec![nat(), args[2].clone(), right, q]);
    trans(args[1].clone(), left, args[2].clone(), p, reverse)
}

fn decode_hex(path: &Path) -> Certificate {
    let hex = fs::read_to_string(path).unwrap();
    let digits: String = hex.chars().filter(|c| !c.is_whitespace()).collect();
    let raw = (0..digits.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&digits[i..i + 2], 16).unwrap())
        .collect::<Vec<_>>();
    mpk_cert::decode_canonical_certificate(&raw).unwrap()
}

pub(super) fn foundation(root: &Path) -> Certificate {
    let mut result = decode_hex(&root.join("fixtures/core-bool-cases/right-identity.hex"));
    let source = decode_hex(&root.join("proofs/std/nat/std-nat.hex"));
    for input in [&result, &source] {
        assert!(input.imports.is_empty());
        assert!(input.proof_node_table.is_empty());
        assert!(input.theory_certificates.is_empty());
        assert_eq!(input.axiom_report.summary.total_axiom_count, 0);
    }
    let n = result.name_table.len() as u32;
    let l = result.level_table.len() as u32;
    let t = result.term_table.len() as u32;
    let g = result.declarations.len() as u32;
    result.name_table.extend(source.name_table);
    result
        .level_table
        .extend(source.level_table.into_iter().map(|level| match level {
            LevelNode::Zero => LevelNode::Zero,
            LevelNode::Succ(a) => LevelNode::Succ(a + l),
            LevelNode::Max(a, b) => LevelNode::Max(a + l, b + l),
            LevelNode::Param(a) => LevelNode::Param(a + n),
        }));
    result
        .term_table
        .extend(source.term_table.into_iter().map(|term| match term {
            TermNode::Sort(a) => TermNode::Sort(a + l),
            TermNode::Var(a) => TermNode::Var(a),
            TermNode::Const { global, levels } => TermNode::Const {
                global: global + g,
                levels: levels.into_iter().map(|a| a + l).collect(),
            },
            TermNode::App {
                function,
                arguments,
            } => TermNode::App {
                function: function + t,
                arguments: arguments.into_iter().map(|a| a + t).collect(),
            },
            TermNode::Lam { ty, body } => TermNode::Lam {
                ty: ty + t,
                body: body + t,
            },
            TermNode::Pi { ty, body } => TermNode::Pi {
                ty: ty + t,
                body: body + t,
            },
            TermNode::Let { ty, value, body } => TermNode::Let {
                ty: ty + t,
                value: value + t,
                body: body + t,
            },
        }));
    result.declarations.extend(
        source
            .declarations
            .into_iter()
            .map(|declaration| Declaration {
                name: declaration.name + n,
                kind: match declaration.kind {
                    DeclarationKind::Inductive { ty } => DeclarationKind::Inductive { ty: ty + t },
                    DeclarationKind::Constructor {
                        ty,
                        inductive,
                        generated,
                    } => DeclarationKind::Constructor {
                        ty: ty + t,
                        inductive: inductive + g,
                        generated,
                    },
                    DeclarationKind::Recursor {
                        ty,
                        inductive,
                        generated,
                    } => DeclarationKind::Recursor {
                        ty: ty + t,
                        inductive: inductive + g,
                        generated,
                    },
                    _ => panic!("Nat foundation changed; audit the input"),
                },
            }),
    );
    result
}

pub(super) fn generate(root: &Path, output: &Path, selection: Option<&String>) {
    fs::create_dir_all(output).unwrap();
    let base = foundation(root);
    let mut index = vec![];
    for abstract_step in [false, true] {
        for wrong in [false, true] {
            let kind = if abstract_step { "step" } else { "nat" };
            let id = format!(
                "full-adder-{kind}-conservation{}",
                if wrong { "-wrong" } else { "" }
            );
            if selection.is_some_and(|selected| selected != &id) {
                continue;
            }
            let step = if abstract_step {
                v("step")
            } else {
                c("Std.Nat.succ")
            };
            let once = |n| apply(step.clone(), vec![n]);
            let maybe = |flag, n: Expr| choose_t(nat(), flag, once(n.clone()), n);
            let sum = xor(xor(v("x"), v("y")), v("incoming"));
            let carry = or(and(v("x"), v("y")), and(v("incoming"), xor(v("x"), v("y"))));
            let left = maybe(v("x"), maybe(v("y"), maybe(v("incoming"), v("initial"))));
            let carried = choose_t(nat(), carry, once(once(v("initial"))), v("initial"));
            let right = maybe(sum, if wrong { v("initial") } else { carried });
            let mut ty = equal(left, right);
            let mut proof = prove(&ty, &[v("x"), v("y"), v("incoming")], &mut 0);
            let mut binders = vec![("initial", nat())];
            if abstract_step {
                binders.push(("step", forall("__nat_arg", nat(), nat())));
            }
            binders.extend([("x", bool_ty()), ("y", bool_ty()), ("incoming", bool_ty())]);
            for (name, domain) in binders.iter().rev() {
                ty = forall(name, domain.clone(), ty);
                proof = lambda(name, domain.clone(), proof);
            }
            let mut builder = Builder::from(base.clone());
            let module = format!("Probe.GoPractical.{}", id.replace('-', "_"));
            builder.c.module.clone_from(&module);
            let theorem = format!("{module}.Claim");
            let type_id = compile(&ty, &mut vec![], &mut builder);
            let proof_id = compile(&proof, &mut vec![], &mut builder);
            let name = builder.c.name_table.len() as u32;
            builder.c.name_table.push(theorem.clone());
            builder.c.declarations.push(Declaration {
                name,
                kind: DeclarationKind::Theorem {
                    ty: type_id,
                    proof: proof_id,
                },
            });
            let terms = builder.c.term_table.len();
            let bytes = builder.finish();
            fs::write(output.join(format!("{id}.mpcert")), &bytes).unwrap();
            index.push(json!({"id":id,"expected":if wrong {"rejected"} else {"accepted"},
                "theorem":theorem,"terms":terms,"bytes":bytes.len(),"declarations":23,
                "quantified_binders":binders.len(),"goal_debug":format!("{ty:?}"),
                "scope":"exact one-bit Nat value/step conservation; full signed 64-bit mathematical sum bridge remains open"}));
        }
    }
    assert!(!index.is_empty(), "unknown natural pilot case");
    fs::write(
        output.join("index.json"),
        serde_json::to_vec_pretty(&json!({
            "schema":"mpk.go_practical.natural_pilot.v0", "cases":index,
        }))
        .unwrap(),
    )
    .unwrap();
}
