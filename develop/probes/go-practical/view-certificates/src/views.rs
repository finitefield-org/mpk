//! Ordinary typed-view coupling on two rows of 64 arbitrary Sort0 cells.
use super::*;
const MODULE: &str = "Probe.GoPractical.TypedViews";
fn value_type() -> Expr {
    v("A")
}
fn cube(n: usize) -> Expr {
    (0..n).rev().fold(value_type(), |t, i| {
        forall(&format!("__t{i}"), bool_ty(), t)
    })
}
fn same(a: Expr, b: Expr) -> Expr {
    apply(c(EQ), vec![value_type(), a, b])
}
fn rfl(a: Expr) -> Expr {
    apply(c(REFL), vec![value_type(), a])
}
fn trans(a: Expr, b: Expr, d: Expr, p: Expr, q: Expr) -> Expr {
    apply(c("Std.Eq.trans"), vec![value_type(), a, b, d, p, q])
}
fn leaf(f: Expr, addr: &[Expr]) -> Expr {
    apply(f, addr.to_vec())
}
fn checked_normalize(expr: &Expr, fresh: &mut usize) -> (Expr, Expr) {
    let beta = weak_beta(expr);
    let reflexive = rfl(beta.clone());
    let Expr::Apply(ref head, ref args) = beta else {
        return (beta, reflexive);
    };
    if **head != c("Std.Bool.cases") || args.len() < 4 {
        return (beta, reflexive);
    }
    let (major, major_proof) = normalize_ground_boolean(&args[3], fresh);
    let branch = if major == literal(false) {
        1
    } else if major == literal(true) {
        2
    } else {
        return (beta, reflexive);
    };
    let name = format!("__typed_major_{}", *fresh);
    *fresh += 1;
    let mut context = args.clone();
    context[3] = v(&name);
    let congr = apply(
        c("Std.Eq.congr"),
        vec![
            bool_ty(),
            value_type(),
            lambda(&name, bool_ty(), apply(c("Std.Bool.cases"), context)),
            args[3].clone(),
            major.clone(),
            major_proof,
        ],
    );
    let mut rewritten_args = args.clone();
    rewritten_args[3] = major;
    let rewritten = apply(c("Std.Bool.cases"), rewritten_args);
    let selected = apply(args[branch].clone(), args[4..].to_vec());
    let (normal, tail) = checked_normalize(&selected, fresh);
    let proof = trans(
        beta.clone(),
        rewritten.clone(),
        normal.clone(),
        congr,
        trans(
            rewritten,
            selected.clone(),
            normal.clone(),
            rfl(selected),
            tail,
        ),
    );
    if &beta == expr {
        (normal, proof)
    } else {
        (
            normal.clone(),
            trans(expr.clone(), beta, normal, reflexive, proof),
        )
    }
}
fn prove(goal: &Expr, majors: &[Expr], fresh: &mut usize) -> Expr {
    if let Some((major, rest)) = majors.split_first() {
        let name = format!("__typed_case_{}", *fresh);
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
        panic!("typed equality");
    };
    let (left, p) = checked_normalize(&args[1], fresh);
    let (right, q) = checked_normalize(&args[2], fresh);
    assert_eq!(
        left, right,
        "intended positive construction must normalize to same ordinary term"
    );
    let reverse = apply(
        c("Std.Eq.symm"),
        vec![value_type(), args[2].clone(), right, q],
    );
    trans(args[1].clone(), left, args[2].clone(), p, reverse)
}
fn index() -> Vec<Expr> {
    addresses(6, "i")
}
fn row_addr(r: Expr) -> Vec<Expr> {
    let mut a = vec![r];
    a.extend(index());
    a
}
fn store(f: Expr, at: &[Expr], value: Expr) -> Expr {
    let q = addresses(at.len(), "__q");
    let guard = q.iter().zip(at).fold(literal(true), |g, (a, b)| {
        and(g, bool_eq(a.clone(), b.clone()))
    });
    cube_lambda(
        at.len(),
        "__q",
        choose_t(value_type(), guard, value, leaf(f, &q)),
    )
}
fn replace_row() -> Expr {
    let q = addresses(7, "__q");
    cube_lambda(
        7,
        "__q",
        choose_t(
            value_type(),
            bool_eq(q[0].clone(), v("r")),
            leaf(v("row"), &q[1..]),
            leaf(v("f"), &q),
        ),
    )
}
pub(super) fn generate(root: &Path, output: &Path) {
    fs::create_dir_all(output).unwrap();
    let mut rows = vec![];
    for family in [
        "child-write-parent-read",
        "parent-replace-child-read",
        "parent-replace-sibling-frame",
        "whole-owner-replace-child-read",
        "distinct-owner-frame",
    ] {
        let mut args = vec![
            ("A".into(), c("__Sort0")),
            (
                "f".into(),
                cube(if family == "distinct-owner-frame" {
                    8
                } else {
                    7
                }),
            ),
            ("r".into(), bool_ty()),
        ];
        args.extend(address_binders(0)); // no value/address table enumeration
        args.extend((0..6).map(|i| (format!("i{i}"), bool_ty())));
        let addr = row_addr(v("r"));
        let (left, right, altered, majors) = match family {
            "child-write-parent-read" => {
                args.push(("value".into(), value_type()));
                (
                    leaf(store(v("f"), &addr, v("value")), &addr),
                    v("value"),
                    leaf(v("f"), &addr),
                    addr.clone(),
                )
            }
            "parent-replace-child-read" | "parent-replace-sibling-frame" => {
                args.push(("row".into(), cube(6)));
                let query = row_addr(if family == "parent-replace-sibling-frame" {
                    not(v("r"))
                } else {
                    v("r")
                });
                let right = if family == "parent-replace-sibling-frame" {
                    leaf(v("f"), &query)
                } else {
                    leaf(v("row"), &index())
                };
                let bad = if family == "parent-replace-sibling-frame" {
                    leaf(v("row"), &index())
                } else {
                    leaf(v("f"), &query)
                };
                (leaf(replace_row(), &query), right, bad, vec![v("r")])
            }
            "whole-owner-replace-child-read" => {
                args.push(("new".into(), cube(7)));
                let replacement =
                    cube_lambda(7, "__whole", leaf(v("new"), &addresses(7, "__whole")));
                (
                    leaf(replacement, &addr),
                    leaf(v("new"), &addr),
                    leaf(v("f"), &addr),
                    vec![],
                )
            }
            _ => {
                args.push(("value".into(), value_type()));
                let mut at = vec![literal(false)];
                at.extend(addr.clone());
                let mut query = vec![literal(true)];
                query.extend(addr.clone());
                (
                    leaf(store(v("f"), &at, v("value")), &query),
                    leaf(v("f"), &query),
                    v("value"),
                    vec![],
                )
            }
        };
        let good = same(left.clone(), right);
        let mut fresh = 0;
        let proof = prove(&good, &majors, &mut fresh);
        for negative in [false, true] {
            let mut b = Builder::from(natural::foundation(root));
            b.c.module = MODULE.into();
            let target = if negative {
                same(left.clone(), altered.clone())
            } else {
                good.clone()
            };
            let (ty, value) = fold_binders(&args, target, proof.clone());
            let ty = compile(&ty, &mut vec![], &mut b);
            let value = compile(&value, &mut vec![], &mut b);
            let name = b.c.name_table.len() as u32;
            b.c.name_table.push(format!("{MODULE}.Claim"));
            b.c.declarations.push(Declaration {
                name,
                kind: DeclarationKind::Theorem { ty, proof: value },
            });
            let terms = b.c.term_table.len();
            let decls = b.c.declarations.len();
            let bytes = b.finish();
            let id = format!("{family}-{}", if negative { "altered" } else { "valid" });
            fs::write(output.join(format!("{id}.mpcert")), &bytes).unwrap();
            rows.push(json!({"id":id,"family":family,"altered":negative,"expected":if negative {"rejected"}else{"accepted"},"bytes":bytes.len(),"terms":terms,"declarations":decls,"theorem":format!("{MODULE}.Claim"),"scope":"ordinary arbitrary Sort0 cell coherence for 2x64 indexed storage and two owner identities; no source/packing/promotion/counter proof","value_enumeration":false,"split_address_flags":majors.len()}));
        }
    }
    fs::write(
        output.join("index.json"),
        serde_json::to_vec_pretty(
            &json!({"schema":"mpk.go_practical.typed_view_pilot.v0","cases":rows}),
        )
        .unwrap(),
    )
    .unwrap();
}
