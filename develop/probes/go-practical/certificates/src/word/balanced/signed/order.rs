//! Actual modular subtraction/order/carry composition, not arithmetic axioms.
//! Four interacting bit carry chains preserve a six-state local invariant.
use super::*;
mod path;

fn named(base: &str, d: usize, args: Vec<Expr>) -> Expr {
    tree(&format!("Order{base}"), d, args)
}
fn complement(d: usize, x: Expr) -> Expr {
    named("Not", d, vec![x])
}
fn complement_body(d: usize, x: Expr) -> Expr {
    if d == 0 {
        not(x)
    } else {
        lambda(
            "__not_half",
            bool_ty(),
            choose_t(
                cube_type(d - 1),
                v("__not_half"),
                complement(d - 1, project(x.clone(), true)),
                complement(d - 1, project(x, false)),
            ),
        )
    }
}
fn complementary_projection(d: usize, x: Expr, high: bool) -> Typed {
    typed(
        cube_type(d - 1),
        project(complement(d, x.clone()), high),
        complement(d - 1, project(x.clone(), high)),
        apply(
            c(&name(&format!(
                "TreeOrderNotProjection{d}{}",
                if high { "High" } else { "Low" }
            ))),
            vec![x],
        ),
    )
}
fn sum_projection(d: usize, x: Expr, y: Expr, incoming: Expr, high: bool) -> Typed {
    let (sl, sh, _) = half_results(d, x.clone(), y.clone(), incoming.clone());
    typed(
        cube_type(d - 1),
        project(sum(d, x.clone(), y.clone(), incoming.clone()), high),
        if high { sh } else { sl },
        apply(
            c(&name(&format!(
                "TreeSumProjection{d}{}",
                if high { "High" } else { "Low" }
            ))),
            vec![x, y, incoming],
        ),
    )
}
fn carry_definition(d: usize, x: Expr, y: Expr, incoming: Expr) -> Typed {
    let body = if d == 0 {
        local(x.clone(), y.clone(), incoming.clone()).1
    } else {
        half_results(d, x.clone(), y.clone(), incoming.clone()).2
    };
    typed(
        bool_ty(),
        carry(d, x.clone(), y.clone(), incoming.clone()),
        body,
        apply(
            c(&name(&format!("TreeCarryDefinition{d}"))),
            vec![x, y, incoming],
        ),
    )
}
fn relation(d: Expr, e: Expr, r: Expr, t: Expr) -> Expr {
    // d+e+r=t+1: when d=e the remaining carry is its opposite;
    // otherwise t=r. Only six of the sixteen carry states are admitted.
    choose(
        bool_eq(d.clone(), e),
        and(bool_eq(t.clone(), d.clone()), bool_eq(r.clone(), not(d))),
        bool_eq(t, r),
    )
}
fn local_chains(l: Expr, a: Expr, x: Expr, ins: &[Expr]) -> Vec<Expr> {
    let (d, dc) = local(l.clone(), not(a.clone()), ins[0].clone());
    let (_, ec) = local(d, not(x.clone()), ins[1].clone());
    let (r, rc) = local(a, x, ins[2].clone());
    let (_, tc) = local(l, not(r), ins[3].clone());
    vec![dc, ec, rc, tc]
}
fn chains(d: usize, l: Expr, a: Expr, x: Expr, ins: &[Expr]) -> Vec<Expr> {
    let rem = sum(d, l.clone(), complement(d, a.clone()), ins[0].clone());
    let result = sum(d, a.clone(), x.clone(), ins[2].clone());
    vec![
        carry(d, l.clone(), complement(d, a.clone()), ins[0].clone()),
        carry(d, rem, complement(d, x.clone()), ins[1].clone()),
        carry(d, a, x, ins[2].clone()),
        carry(d, l, complement(d, result), ins[3].clone()),
    ]
}
fn related(flags: &[Expr]) -> Expr {
    relation(
        flags[0].clone(),
        flags[1].clone(),
        flags[2].clone(),
        flags[3].clone(),
    )
}
fn bit_args() -> Vec<(String, Expr)> {
    ["di", "ei", "ri", "ti"]
        .into_iter()
        .map(|n| (n.into(), bool_ty()))
        .collect()
}
fn in_bits() -> Vec<Expr> {
    ["di", "ei", "ri", "ti"].into_iter().map(v).collect()
}
fn operands(d: usize) -> Vec<(String, Expr)> {
    let mut args = ["limit", "acc", "item"]
        .into_iter()
        .map(|n| (n.into(), cube_type(d)))
        .collect::<Vec<_>>();
    args.extend(bit_args());
    args
}
fn call_args(l: Expr, a: Expr, x: Expr, ins: &[Expr]) -> Vec<Expr> {
    let mut args = vec![l, a, x];
    args.extend_from_slice(ins);
    args
}
fn conserved(d: usize, l: Expr, a: Expr, x: Expr, ins: &[Expr], h: Expr) -> Typed {
    let out = related(&chains(d, l.clone(), a.clone(), x.clone(), ins));
    let mut args = call_args(l, a, x, ins);
    args.push(h);
    typed(
        bool_ty(),
        out,
        literal(true),
        apply(c(&name(&format!("TreeOrderConservation{d}"))), args),
    )
}
fn normalized(expr: Expr, p: &mut Producer) -> Typed {
    let (normal, proof) = normalize_ground_boolean(&expr, &mut p.fresh);
    typed(bool_ty(), expr, normal, proof)
}
fn under(
    p: &mut Producer,
    condition: Expr,
    result: Expr,
    target: Expr,
    h: Expr,
    proof: Expr,
) -> Typed {
    let original = choose(condition.clone(), result.clone(), target.clone());
    let q = typed(bool_ty(), condition, literal(true), h);
    let selection = replace(p, bool_ty(), original.clone(), q);
    let tail = typed(
        bool_ty(),
        selection.right.clone(),
        result.clone(),
        refl(result.clone()),
    );
    let selection = next(selection, tail);
    next(
        backwards(selection),
        typed(bool_ty(), original, target, proof),
    )
}
fn primitives(p: &mut Producer) {
    let l = v("limit");
    let a = v("acc");
    let x = v("item");
    let ins = in_bits();
    let condition = related(&ins);
    let result = related(&local_chains(l.clone(), a.clone(), x.clone(), &ins));
    let guarded = choose(condition.clone(), result.clone(), literal(true));
    let mut args = vec![
        ("limit".into(), bool_ty()),
        ("acc".into(), bool_ty()),
        ("item".into(), bool_ty()),
    ];
    args.extend(bit_args());
    let proof = prove_with_checked_normalization(
        &eq(guarded.clone(), literal(true)),
        &call_args(l, a, x, &ins),
        &mut p.fresh,
    );
    theorem(
        p,
        "OrderLocalPreservation",
        &args,
        typed(bool_ty(), guarded, literal(true), proof),
    );
    let mut applied = call_args(v("limit"), v("acc"), v("item"), &ins);
    let proof = apply(
        c(&name("OrderLocalPreservation")),
        std::mem::take(&mut applied),
    );
    let e = under(p, condition.clone(), result, literal(true), v("h"), proof);
    args.push(("h".into(), eq(condition, literal(true))));
    theorem(p, "OrderLocalStep", &args, e);
}
fn definitions(p: &mut Producer, d: usize) {
    let x = v("x");
    let args = vec![("x".into(), cube_type(d))];
    function_definition(
        p,
        &format!("TreeOrderNot{d}"),
        &args,
        cube_type(d),
        complement_body(d, x.clone()),
    );
    if d > 0 {
        for high in [false, true] {
            let mut e = complementary_projection(d, x.clone(), high);
            e.proof = apply(c(REFL), vec![cube_type(d - 1), e.right.clone()]);
            theorem(
                p,
                &format!(
                    "TreeOrderNotProjection{d}{}",
                    if high { "High" } else { "Low" }
                ),
                &args,
                e,
            );
        }
    }
}
fn binding(p: &mut Producer, d: usize, l: Expr, a: Expr, x: Expr, ins: &[Expr]) -> Typed {
    let whole = related(&chains(d, l.clone(), a.clone(), x.clone(), ins));
    let raw = chains(d, l.clone(), a.clone(), x.clone(), ins);
    if d == 0 {
        let target = related(&local_chains(l, a, x, ins));
        return typed(bool_ty(), whole, target.clone(), refl(target));
    }
    let rem = sum(d, l.clone(), complement(d, a.clone()), ins[0].clone());
    let result = sum(d, a.clone(), x.clone(), ins[2].clone());
    let mut flags = vec![
        carry_definition(d, l.clone(), complement(d, a.clone()), ins[0].clone()),
        carry_definition(d, rem.clone(), complement(d, x.clone()), ins[1].clone()),
        carry_definition(d, a.clone(), x.clone(), ins[2].clone()),
        carry_definition(d, l.clone(), complement(d, result.clone()), ins[3].clone()),
    ];
    for (i, flag) in flags.iter_mut().enumerate() {
        assert_eq!(flag.left, raw[i]);
        for high in [false, true] {
            if i == 1 {
                *flag = change(
                    p,
                    flag.clone(),
                    sum_projection(d, l.clone(), complement(d, a.clone()), ins[0].clone(), high),
                );
            }
            if i == 3 {
                *flag = change(
                    p,
                    flag.clone(),
                    complementary_projection(d, result.clone(), high),
                );
                *flag = change(
                    p,
                    flag.clone(),
                    sum_projection(d, a.clone(), x.clone(), ins[2].clone(), high),
                );
            }
        }
        for high in [false, true] {
            if i == 0 || i == 1 {
                *flag = change(
                    p,
                    flag.clone(),
                    complementary_projection(d, a.clone(), high),
                );
            }
            if i == 1 {
                *flag = change(
                    p,
                    flag.clone(),
                    complementary_projection(d, x.clone(), high),
                );
            }
        }
    }
    let mut e = typed(bool_ty(), whole.clone(), whole.clone(), refl(whole));
    for flag in flags {
        e = change(p, e, flag);
    }
    let mid = chains(
        d - 1,
        project(l.clone(), false),
        project(a.clone(), false),
        project(x.clone(), false),
        ins,
    );
    let expected = related(&chains(
        d - 1,
        project(l, true),
        project(a, true),
        project(x, true),
        &mid,
    ));
    assert_eq!(e.right, expected);
    e
}
fn build(p: &mut Producer, d: usize, trace: Option<&Path>) {
    definitions(p, d);
    let l = v("limit");
    let a = v("acc");
    let x = v("item");
    let ins = in_bits();
    let args = operands(d);
    let condition = related(&ins);
    let e = binding(p, d, l.clone(), a.clone(), x.clone(), &ins);
    theorem(p, &format!("TreeOrderNextBinding{d}"), &args, e.clone());
    let tail = if d == 0 {
        let mut actual = call_args(l, a, x, &ins);
        actual.push(v("h"));
        typed(
            bool_ty(),
            e.right.clone(),
            literal(true),
            apply(c(&name("OrderLocalStep")), actual),
        )
    } else {
        let mid = chains(
            d - 1,
            project(l.clone(), false),
            project(a.clone(), false),
            project(x.clone(), false),
            &ins,
        );
        let lo = conserved(
            d - 1,
            project(l.clone(), false),
            project(a.clone(), false),
            project(x.clone(), false),
            &ins,
            v("h"),
        );
        conserved(
            d - 1,
            project(l, true),
            project(a, true),
            project(x, true),
            &mid,
            lo.proof,
        )
    };
    let e = next(e, tail);
    let mut args = args;
    args.push(("h".into(), eq(condition, literal(true))));
    theorem(p, &format!("TreeOrderConservation{d}"), &args, e);
    save_trace(p, trace, &format!("order-{d}-conservation"));
}
pub(super) fn generate(root: &Path, output: &Path, depth: usize, trace: bool) {
    assert!(depth <= 7);
    fs::create_dir_all(output).unwrap();
    let mut p = Producer {
        builder: Builder::from(natural::foundation(root)),
        fresh: 0,
    };
    p.builder.c.module = format!("{PREFIX}.OrderDepth{depth}");
    p.primitive_library();
    super::primitives(&mut p);
    primitives(&mut p);
    path::primitives(&mut p);
    for d in 0..=depth {
        super::super::definitions(&mut p, d, None);
        commutation(&mut p, d, None);
        adding(&mut p, d);
        super::build(&mut p, d, None);
        build(&mut p, d, trace.then_some(output));
        path::build(&mut p, d, trace.then_some(output));
    }
    let base = p.builder.c;
    let mut cases = vec![];
    for mutation in 0..2 {
        let mut p = Producer {
            builder: Builder::from(base.clone()),
            fresh: 0,
        };
        let args = operands(depth);
        let ins = in_bits();
        let condition = if mutation == 1 {
            literal(true)
        } else {
            related(&ins)
        };
        let mut args = args;
        args.push(("h".into(), eq(condition, literal(true))));
        let e = conserved(depth, v("limit"), v("acc"), v("item"), &ins, v("h"));
        theorem(&mut p, "OrderClaim", &args, e);
        let id = format!(
            "order-{}-{}",
            1usize << depth,
            if mutation == 0 {
                "carry-conservation"
            } else {
                "unrelated-carry-premise"
            }
        );
        let terms = p.builder.c.term_table.len();
        let declarations = p.builder.c.declarations.len();
        let bytes = p.builder.finish();
        fs::write(output.join(format!("{id}.mpcert")), &bytes).unwrap();
        cases.push(json!({"id":id,"family":"carry","depth":depth,"width":1usize<<depth,"mutation":mutation,
            "expected":if mutation==0 {"accepted"} else {"rejected"},"terms":terms,"declarations":declarations,"bytes":bytes.len(),"theorem":name("OrderClaim"),
            "operand_binders":3,"quantified_operand_bits":3usize<<depth,"initial_natural_quantified":false,
            "premises":"four arbitrary incoming carries satisfying d + e + r = t + 1; unrelated true replacement is a deliberate mutation",
            "scope":"private unsigned modular add/subtract carry relation; Go frontend/source binding remains pending"}));
    }
    path::emit(&base, output, depth, &mut cases);
    fs::write(
        output.join("index.json"),
        serde_json::to_vec_pretty(
            &json!({"schema":"mpk.go_practical.order_pilot.v0","cases":cases}),
        )
        .unwrap(),
    )
    .unwrap();
}
