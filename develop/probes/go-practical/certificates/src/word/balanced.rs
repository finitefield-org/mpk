//! Balanced Bool-cube word arithmetic. Depth d has exactly 2^d payload bits.
//! Every full operand is universally quantified; no payload enumeration.

use super::*;
mod signed;
pub(super) fn generate_fold(root: &Path, output: &Path, count: usize, trace: bool) {
    signed::generate_fold(root, output, count, trace);
}
pub(super) fn generate_signed_fold(root: &Path, output: &Path, count: usize, trace: bool) {
    signed::generate_signed_fold(root, output, count, trace);
}

fn tree(suffix: &str, depth: usize, args: Vec<Expr>) -> Expr {
    apply(c(&name(&format!("Tree{suffix}{depth}"))), args)
}
fn value(d: usize, f: Expr, x: Expr) -> Expr {
    tree("Value", d, vec![f, x])
}
fn weight(d: usize, f: Expr) -> Expr {
    tree("Weight", d, vec![f])
}
fn sum(d: usize, x: Expr, y: Expr, carry: Expr) -> Expr {
    tree("Sum", d, vec![x, y, carry])
}
fn carry(d: usize, x: Expr, y: Expr, incoming: Expr) -> Expr {
    tree("Carry", d, vec![x, y, incoming])
}
fn project(x: Expr, high: bool) -> Expr {
    apply(x, vec![literal(high)])
}
fn flip(d: usize, x: Expr) -> Expr {
    tree("Flip", d, vec![x])
}

fn value_body(d: usize, f: Expr, x: Expr) -> Expr {
    if d == 0 {
        conditional(f, x)
    } else {
        cmp(
            value(d - 1, f.clone(), project(x.clone(), false)),
            value(d - 1, weight(d - 1, f), project(x, true)),
        )
    }
}
fn weight_body(d: usize, f: Expr) -> Expr {
    if d == 0 {
        square(f)
    } else {
        weight(d - 1, weight(d - 1, f))
    }
}
fn half_results(d: usize, x: Expr, y: Expr, c: Expr) -> (Expr, Expr, Expr) {
    assert!(d > 0);
    let low = carry(
        d - 1,
        project(x.clone(), false),
        project(y.clone(), false),
        c.clone(),
    );
    let sl = sum(
        d - 1,
        project(x.clone(), false),
        project(y.clone(), false),
        c,
    );
    let sh = sum(
        d - 1,
        project(x.clone(), true),
        project(y.clone(), true),
        low.clone(),
    );
    let high = carry(d - 1, project(x, true), project(y, true), low);
    (sl, sh, high)
}
fn sum_value_body(d: usize, f: Expr, x: Expr, y: Expr, c: Expr) -> Expr {
    if d == 0 {
        let (s, _) = adder(&[x], &[y], c);
        conditional(f, s[0].clone())
    } else {
        let (sl, sh, _) = half_results(d, x, y, c);
        cmp(
            value(d - 1, f.clone(), sl),
            value(d - 1, weight(d - 1, f), sh),
        )
    }
}
fn add_equation(d: usize, f: Expr, x: Expr, y: Expr, c: Expr) -> (Expr, Expr) {
    (
        cmp(
            value(d, f.clone(), x.clone()),
            cmp(
                value(d, f.clone(), y.clone()),
                conditional(f.clone(), c.clone()),
            ),
        ),
        cmp(
            value(d, f.clone(), sum(d, x.clone(), y.clone(), c.clone())),
            conditional(weight(d, f), carry(d, x, y, c)),
        ),
    )
}
fn word_unfold(d: usize, f: Expr, x: Expr) -> Equality {
    instance(
        &format!("TreeValueDefinition{d}"),
        vec![f.clone(), x.clone()],
        value(d, f.clone(), x.clone()),
        value_body(d, f, x),
    )
}
fn weight_unfold(d: usize, f: Expr) -> Equality {
    instance(
        &format!("TreeWeightDefinition{d}"),
        vec![f.clone()],
        weight(d, f.clone()),
        weight_body(d, f),
    )
}
fn sum_value_unfold(d: usize, f: Expr, x: Expr, y: Expr, c: Expr) -> Equality {
    instance(
        &format!("TreeSumValueDefinition{d}"),
        vec![f.clone(), x.clone(), y.clone(), c.clone()],
        value(d, f.clone(), sum(d, x.clone(), y.clone(), c.clone())),
        sum_value_body(d, f, x, y, c),
    )
}
fn weight_commutes(d: usize, f: Expr, g: Expr, h: Equality) -> Equality {
    instance(
        &format!("TreeWeightCommutes{d}"),
        vec![f.clone(), g.clone(), h.proof],
        cmp(weight(d, f.clone()), g.clone()),
        cmp(g, weight(d, f)),
    )
}
fn value_commutes(d: usize, f: Expr, g: Expr, h: Equality, x: Expr) -> Equality {
    instance(
        &format!("TreeValueCommutes{d}"),
        vec![f.clone(), g.clone(), h.proof, x.clone()],
        cmp(value(d, f.clone(), x.clone()), g.clone()),
        cmp(g, value(d, f, x)),
    )
}
fn addition(d: usize, f: Expr, x: Expr, y: Expr, c: Expr) -> Equality {
    let (left, right) = add_equation(d, f.clone(), x.clone(), y.clone(), c.clone());
    instance(&format!("TreeAddition{d}"), vec![f, x, y, c], left, right)
}
fn function_definition(
    p: &mut Producer,
    suffix: &str,
    binders: &[(String, Expr)],
    ty: Expr,
    body: Expr,
) {
    let (ty, value) = fold_binders(binders, ty, body);
    p.declare(suffix, ty, value, false);
}
fn computed(e: Equality) -> Equality {
    Equality {
        left: e.left,
        right: e.right.clone(),
        proof: reflf(e.right),
    }
}
fn save_trace(p: &Producer, trace: Option<&Path>, label: &str) {
    if let Some(output) = trace {
        fs::write(
            output.join(format!("stage-{label}.mpcert")),
            Builder::from(p.builder.c.clone()).finish(),
        )
        .unwrap();
    }
}
fn definitions(p: &mut Producer, d: usize, trace: Option<&Path>) {
    let operands = vec![
        ("x".into(), cube_type(d)),
        ("y".into(), cube_type(d)),
        ("incoming".into(), bool_ty()),
    ];
    let (sum_body, carry_body) = if d == 0 {
        let (s, c) = adder(&[v("x")], &[v("y")], v("incoming"));
        (s[0].clone(), c)
    } else {
        let (sl, sh, c) = half_results(d, v("x"), v("y"), v("incoming"));
        (
            lambda(
                "__half",
                bool_ty(),
                choose_t(cube_type(d - 1), v("__half"), sh, sl),
            ),
            c,
        )
    };
    function_definition(p, &format!("TreeSum{d}"), &operands, cube_type(d), sum_body);
    function_definition(
        p,
        &format!("TreeCarry{d}"),
        &operands,
        bool_ty(),
        carry_body.clone(),
    );
    let (ty, proof) = fold_binders(
        &operands,
        eq(carry(d, v("x"), v("y"), v("incoming")), carry_body.clone()),
        refl(carry_body),
    );
    p.declare(&format!("TreeCarryDefinition{d}"), ty, proof, true);
    let unary = vec![("x".into(), cube_type(d))];
    let flipped = if d == 0 {
        not(v("x"))
    } else {
        lambda(
            "__half",
            bool_ty(),
            choose_t(
                cube_type(d - 1),
                v("__half"),
                project(v("x"), true),
                flip(d - 1, project(v("x"), false)),
            ),
        )
    };
    function_definition(p, &format!("TreeFlip{d}"), &unary, cube_type(d), flipped);
    let vf = vec![("f".into(), function()), ("x".into(), cube_type(d))];
    function_definition(
        p,
        &format!("TreeValue{d}"),
        &vf,
        function(),
        value_body(d, v("f"), v("x")),
    );
    let root = vec![("f".into(), function())];
    function_definition(
        p,
        &format!("TreeWeight{d}"),
        &root,
        function(),
        weight_body(d, v("f")),
    );
    p.theorem(
        &format!("TreeValueDefinition{d}"),
        &vf,
        computed(word_unfold(d, v("f"), v("x"))),
    );
    p.theorem(
        &format!("TreeWeightDefinition{d}"),
        &root,
        computed(weight_unfold(d, v("f"))),
    );
    save_trace(p, trace, &format!("{d}-before-sum-value"));
    let sum_value_proof = if d == 0 {
        computed(sum_value_unfold(d, v("f"), v("x"), v("y"), v("incoming")))
    } else {
        let (sl, sh, _) = half_results(d, v("x"), v("y"), v("incoming"));
        let sum_cube = sum(d, v("x"), v("y"), v("incoming"));
        let mut lifted = vec![];
        for (high, selected) in [(false, sl), (true, sh)] {
            let suffix = format!("TreeSumProjection{d}{}", if high { "High" } else { "Low" });
            let left = project(sum_cube.clone(), high);
            let ty = apply(
                c(EQ),
                vec![cube_type(d - 1), left.clone(), selected.clone()],
            );
            let proof = apply(c(REFL), vec![cube_type(d - 1), selected.clone()]);
            let (ty, proof) = fold_binders(&operands, ty, proof);
            p.declare(&suffix, ty, proof, true);
            save_trace(p, trace, &format!("{d}-sum-projection-{high}"));
            let proof = apply(c(&name(&suffix)), vec![v("x"), v("y"), v("incoming")]);
            let root = if high { weight(d - 1, v("f")) } else { v("f") };
            let context = lambda(
                "__projected_word",
                cube_type(d - 1),
                value(d - 1, root.clone(), v("__projected_word")),
            );
            lifted.push(Equality {
                left: value(d - 1, root.clone(), left.clone()),
                right: value(d - 1, root, selected.clone()),
                proof: apply(
                    c("Std.Eq.congr"),
                    vec![cube_type(d - 1), function(), context, left, selected, proof],
                ),
            });
        }
        let mut e = word_unfold(d, v("f"), sum_cube);
        for projection in lifted {
            e = p.change(e, projection);
        }
        e
    };
    let mut fs = vec![("f".into(), function())];
    fs.extend(operands);
    p.theorem(&format!("TreeSumValueDefinition{d}"), &fs, sum_value_proof);
}

fn commutation(p: &mut Producer, d: usize, trace: Option<&Path>) {
    let fg = vec![
        ("f".into(), function()),
        ("g".into(), function()),
        ("h".into(), commute(v("f"), v("g"))),
    ];
    let h = Equality {
        left: cmp(v("f"), v("g")),
        right: cmp(v("g"), v("f")),
        proof: v("h"),
    };
    let left = cmp(weight(d, v("f")), v("g"));
    let mut e = p.rewrite(left, weight_unfold(d, v("f")));
    let q = if d == 0 {
        Producer::square_commute(v("f"), v("g"), h.clone())
    } else {
        let first = weight_commutes(d - 1, v("f"), v("g"), h.clone());
        weight_commutes(d - 1, weight(d - 1, v("f")), v("g"), first)
    };
    e = transit(e, q);
    e = p.change(e, reverse(weight_unfold(d, v("f"))));
    p.theorem(&format!("TreeWeightCommutes{d}"), &fg, e);
    save_trace(p, trace, &format!("{d}-weight-commutes"));
    let mut binders = fg;
    binders.push(("x".into(), cube_type(d)));
    let mut e = p.rewrite(
        cmp(value(d, v("f"), v("x")), v("g")),
        word_unfold(d, v("f"), v("x")),
    );
    let q = if d == 0 {
        Producer::maybe_left(v("f"), v("g"), v("x"), h)
    } else {
        let low = value(d - 1, v("f"), project(v("x"), false));
        let high = value(d - 1, weight(d - 1, v("f")), project(v("x"), true));
        let lo = value_commutes(d - 1, v("f"), v("g"), h.clone(), project(v("x"), false));
        let hw = weight_commutes(d - 1, v("f"), v("g"), h);
        let hi = value_commutes(
            d - 1,
            weight(d - 1, v("f")),
            v("g"),
            hw,
            project(v("x"), true),
        );
        instance(
            "ProductCommutes",
            vec![low.clone(), high.clone(), v("g"), lo.proof, hi.proof],
            cmp(cmp(low.clone(), high.clone()), v("g")),
            cmp(v("g"), cmp(low, high)),
        )
    };
    e = transit(e, q);
    e = p.change(e, reverse(word_unfold(d, v("f"), v("x"))));
    p.theorem(&format!("TreeValueCommutes{d}"), &binders, e);
}

fn adding(p: &mut Producer, d: usize) {
    let f = v("f");
    let x = v("x");
    let y = v("y");
    let incoming = v("incoming");
    let binders = vec![
        ("f".into(), function()),
        ("x".into(), cube_type(d)),
        ("y".into(), cube_type(d)),
        ("incoming".into(), bool_ty()),
    ];
    let (left, right) = add_equation(d, f.clone(), x.clone(), y.clone(), incoming.clone());
    if d == 0 {
        let proof = apply(c(&name("FullAdder")), vec![f, x, y, incoming]);
        p.theorem("TreeAddition0", &binders, Equality { left, right, proof });
        return;
    }
    let f = v("f");
    let x = v("x");
    let y = v("y");
    let incoming = v("incoming");
    let hf = weight(d - 1, f.clone());
    let xl = project(x.clone(), false);
    let xh = project(x.clone(), true);
    let yl = project(y.clone(), false);
    let yh = project(y.clone(), true);
    let a = value(d - 1, f.clone(), xl.clone());
    let b = value(d - 1, f.clone(), yl.clone());
    let xt = value(d - 1, hf.clone(), xh.clone());
    let yt = value(d - 1, hf.clone(), yh.clone());
    let cf = conditional(f.clone(), incoming.clone());
    let hw = weight_commutes(
        d - 1,
        f.clone(),
        f.clone(),
        reflexive(cmp(f.clone(), f.clone())),
    );
    let hx = value_commutes(d - 1, hf.clone(), f.clone(), hw.clone(), xh.clone());
    let hy = value_commutes(d - 1, hf.clone(), f.clone(), hw, yh.clone());
    let mut e = p.rewrite(left, word_unfold(d, f.clone(), x.clone()));
    e = p.change(e, word_unfold(d, f.clone(), y.clone()));
    let mut items = vec![a.clone(), xt.clone(), b.clone(), yt.clone(), cf.clone()];
    let norm = p.monoid(e.right.clone(), Producer::product(&items));
    e = transit(e, norm);
    let hxb = reverse(value_commutes(
        d - 1,
        f.clone(),
        xt.clone(),
        reverse(hx.clone()),
        yl.clone(),
    ));
    e = p.swap(e, &mut items, 1, hxb);
    e = p.swap(
        e,
        &mut items,
        3,
        Producer::maybe_right(yt.clone(), f.clone(), incoming.clone(), hy),
    );
    e = p.swap(
        e,
        &mut items,
        2,
        Producer::maybe_right(xt.clone(), f.clone(), incoming.clone(), hx),
    );
    let local = addition(d - 1, f.clone(), xl.clone(), yl.clone(), incoming.clone());
    let grouped = cmp(local.left.clone(), cmp(xt.clone(), yt.clone()));
    let norm = p.monoid(e.right.clone(), grouped);
    e = transit(e, norm);
    e = p.change(e, local);
    let low_carry = carry(d - 1, xl.clone(), yl.clone(), incoming.clone());
    let sl = value(d - 1, f.clone(), sum(d - 1, xl, yl, incoming.clone()));
    let kc = conditional(hf.clone(), low_carry.clone());
    let mut items = vec![sl.clone(), kc, xt.clone(), yt.clone()];
    let norm = p.monoid(e.right.clone(), Producer::product(&items));
    e = transit(e, norm);
    let hxx = value_commutes(
        d - 1,
        hf.clone(),
        hf.clone(),
        reflexive(cmp(hf.clone(), hf.clone())),
        xh.clone(),
    );
    let hyy = value_commutes(
        d - 1,
        hf.clone(),
        hf.clone(),
        reflexive(cmp(hf.clone(), hf.clone())),
        yh.clone(),
    );
    e = p.swap(
        e,
        &mut items,
        1,
        reverse(Producer::maybe_right(
            xt,
            hf.clone(),
            low_carry.clone(),
            hxx,
        )),
    );
    e = p.swap(
        e,
        &mut items,
        2,
        reverse(Producer::maybe_right(
            yt,
            hf.clone(),
            low_carry.clone(),
            hyy,
        )),
    );
    let high = addition(d - 1, hf.clone(), xh.clone(), yh.clone(), low_carry.clone());
    assert_eq!(e.right, cmp(sl.clone(), high.left.clone()));
    e = p.change(e, high);
    let sh = value(
        d - 1,
        hf.clone(),
        sum(d - 1, xh.clone(), yh.clone(), low_carry.clone()),
    );
    let final_carry = carry(d - 1, xh, yh, low_carry);
    let tail = conditional(weight(d - 1, hf), final_carry);
    let norm = p.monoid(e.right.clone(), cmp(cmp(sl, sh), tail));
    e = transit(e, norm);
    e = p.change(
        e,
        reverse(sum_value_unfold(
            d,
            f.clone(),
            x.clone(),
            y.clone(),
            incoming,
        )),
    );
    e = p.change(e, reverse(weight_unfold(d, f)));
    let body = carry(
        d - 1,
        project(x.clone(), true),
        project(y.clone(), true),
        carry(
            d - 1,
            project(x.clone(), false),
            project(y.clone(), false),
            v("incoming"),
        ),
    );
    let whole = carry(d, x.clone(), y.clone(), v("incoming"));
    let forward = apply(
        c(&name(&format!("TreeCarryDefinition{d}"))),
        vec![x, y, v("incoming")],
    );
    let reversed = apply(
        c("Std.Eq.symm"),
        vec![bool_ty(), whole.clone(), body.clone(), forward],
    );
    let ctx = lambda(
        "__final_carry",
        bool_ty(),
        conditional(weight(d, v("f")), v("__final_carry")),
    );
    let lifted = Equality {
        left: conditional(weight(d, v("f")), body.clone()),
        right: conditional(weight(d, v("f")), whole.clone()),
        proof: apply(
            c("Std.Eq.congr"),
            vec![bool_ty(), function(), ctx, body, whole, reversed],
        ),
    };
    e = p.change(e, lifted);
    assert_eq!(e.right, right);
    p.theorem(&format!("TreeAddition{d}"), &binders, e);
}

fn claim(p: &mut Producer, d: usize, mutation: usize) -> Expr {
    let f = c("Std.Nat.succ");
    let x = v("x");
    let y = v("y");
    let incoming = v("incoming");
    let e = addition(d, f.clone(), x.clone(), y.clone(), incoming.clone());
    let result = sum(d, x.clone(), y.clone(), incoming.clone());
    let right = match mutation {
        0 => e.right.clone(),
        1 => value(d, f, result),
        _ => cmp(
            value(d, f.clone(), flip(d, result)),
            conditional(weight(d, f), carry(d, x, y, incoming)),
        ),
    };
    let goal = apply(
        c(EQ),
        vec![
            nat(),
            apply(e.left.clone(), vec![v("initial")]),
            apply(right, vec![v("initial")]),
        ],
    );
    let ctx = lambda(
        "__evaluated_function",
        function(),
        apply(v("__evaluated_function"), vec![v("initial")]),
    );
    let proof = apply(
        c("Std.Eq.congr"),
        vec![function(), nat(), ctx, e.left, e.right, e.proof],
    );
    let binders = vec![
        ("initial".into(), nat()),
        ("x".into(), cube_type(d)),
        ("y".into(), cube_type(d)),
        ("incoming".into(), bool_ty()),
    ];
    let (ty, proof) = fold_binders(&binders, goal, proof);
    p.declare("TreeClaim", ty.clone(), proof, true);
    ty
}

pub(super) fn generate(root: &Path, output: &Path, depth: usize, trace: bool) {
    generate_kind(root, output, depth, trace, false);
}
pub(super) fn generate_signed(root: &Path, output: &Path, depth: usize, trace: bool) {
    generate_kind(root, output, depth, trace, true);
}
fn generate_kind(root: &Path, output: &Path, depth: usize, trace: bool, signed: bool) {
    assert!(depth <= 7, "private tree probe supports up to 128 bits");
    fs::create_dir_all(output).unwrap();
    let mut p = Producer {
        builder: Builder::from(natural::foundation(root)),
        fresh: 0,
    };
    p.builder.c.module = format!(
        "{PREFIX}.{}TreeDepth{depth}",
        if signed { "Signed" } else { "" }
    );
    p.primitive_library();
    for d in 0..=depth {
        definitions(&mut p, d, trace.then_some(output));
        if trace {
            fs::write(
                output.join(format!("prefix-{d}-definitions.mpcert")),
                Builder::from(p.builder.c.clone()).finish(),
            )
            .unwrap();
        }
        commutation(&mut p, d, trace.then_some(output));
        if trace {
            fs::write(
                output.join(format!("prefix-{d}-commutation.mpcert")),
                Builder::from(p.builder.c.clone()).finish(),
            )
            .unwrap();
        }
        adding(&mut p, d);
        if depth <= 1 {
            fs::write(
                output.join(format!("prefix-{d}.mpcert")),
                Builder::from(p.builder.c.clone()).finish(),
            )
            .unwrap();
        }
    }
    if signed {
        signed::primitives(&mut p);
        for d in 0..=depth {
            signed::build(&mut p, d, trace.then_some(output));
        }
    }
    let base = p.builder.c;
    let mut index = vec![];
    for mutation in 0..3 {
        let id = format!(
            "{}tree-{}-{}",
            if signed { "signed-" } else { "" },
            1usize << depth,
            match mutation {
                0 => "nat-conservation",
                1 =>
                    if signed {
                        "dropped-result-sign"
                    } else {
                        "dropped-carry"
                    },
                _ =>
                    if signed {
                        "unrelated-overflow-premise"
                    } else {
                        "low-bit-wrong"
                    },
            }
        );
        let mut p = Producer {
            builder: Builder::from(base.clone()),
            fresh: 0,
        };
        let ty = if signed {
            signed::claim(&mut p, depth, mutation)
        } else {
            claim(&mut p, depth, mutation)
        };
        let terms = p.builder.c.term_table.len();
        let declarations = p.builder.c.declarations.len();
        let data = p.builder.finish();
        fs::write(output.join(format!("{id}.mpcert")), &data).unwrap();
        index.push(json!({"id":id,"depth":depth,"width":1usize<<depth,"mutation":mutation,
            "kind":if signed {"signed-no-overflow-cross-sum"} else {"unsigned-final-carry"},
            "required_premise":if signed {if mutation==2 {"unrelated true equality (deliberate mutation)"} else {"actual sign-stage incoming/outgoing carry agreement"}} else {"none"},
            "incoming_carry":if signed {"literal zero"} else {"universally quantified Bool"},
            "expected":if mutation==0 {"accepted"} else {"rejected"},"declarations":declarations,"terms":terms,"bytes":data.len(),
            "operand_carrier":"Bool-addressed cube","operand_binders":2,"quantified_operand_bits":2usize<<depth,
            "initial_natural_quantified":true,"theorem":name(if signed {"SignedTreeClaim"} else {"TreeClaim"}),"goal_debug":format!("{ty:?}"),
            "scope":if signed {"conditional two's-complement cross-sum equality in Nat; prefix sums and Go bindings remain open"} else {"exact unsigned Nat value with final carry; prefix sums and Go bindings remain open"}}));
    }
    fs::write(
        output.join("index.json"),
        serde_json::to_vec_pretty(
            &json!({"schema":"mpk.go_practical.tree_word_pilot.v0","cases":index}),
        )
        .unwrap(),
    )
    .unwrap();
}
