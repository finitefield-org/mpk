//! Private ordinary Nat order/strong-induction proofs, including type-level recursion.
use super::*;
const MODULE: &str = "Probe.GoPractical.NatTermination";
fn named(s: &str) -> String {
    format!("{MODULE}.{s}")
}
fn app(s: &str, a: Vec<Expr>) -> Expr {
    apply(c(&named(s)), a)
}
fn ty0() -> Expr {
    c("__Sort0")
}
fn nt() -> Expr {
    c("Std.Nat")
}
fn zero() -> Expr {
    c("Std.Nat.zero")
}
fn succ(n: Expr) -> Expr {
    apply(c("Std.Nat.succ"), vec![n])
}
fn neq(a: Expr, b: Expr) -> Expr {
    apply(c(EQ), vec![nt(), a, b])
}
fn nrfl(n: Expr) -> Expr {
    apply(c(REFL), vec![nt(), n])
}
fn induct(p: Expr, z: Expr, s: Expr, n: Expr) -> Expr {
    apply(c("Std.Nat.induct"), vec![p, z, s, n])
}
fn rec_type(z: Expr, s: Expr, n: Expr) -> Expr {
    apply(c("Std.Bool.recType"), vec![z, s, n])
}
fn less(m: Expr, n: Expr) -> Expr {
    app("Less", vec![m, n])
}
fn lt(m: Expr, n: Expr) -> Expr {
    eq(less(m, n), literal(true))
}
fn history(p: Expr, n: Expr) -> Expr {
    app("History", vec![p, n])
}
fn fold(a: &[(String, Expr)], r: Expr, b: Expr) -> (Expr, Expr) {
    fold_binders(a, r, b)
}
fn wrapped(a: &[(String, Expr)], b: Expr) -> Expr {
    a.iter().rev().fold(b, |r, (n, t)| lambda(n, t.clone(), r))
}
fn result(a: &[(String, Expr)], b: Expr) -> Expr {
    a.iter().rev().fold(b, |r, (n, t)| forall(n, t.clone(), r))
}
struct Library {
    b: Builder,
}
impl Library {
    fn declare(&mut self, s: &str, a: &[(String, Expr)], r: Expr, b: Expr, theorem: bool) {
        let (ty, body) = fold(a, r, b);
        let ty = compile(&ty, &mut vec![], &mut self.b);
        let body = compile(&body, &mut vec![], &mut self.b);
        let global = self.b.c.declarations.len() as u32;
        let n = self.b.c.name_table.len() as u32;
        let name = named(s);
        assert!(!self.b.globals.contains_key(&name));
        self.b.c.name_table.push(name.clone());
        self.b.c.declarations.push(Declaration {
            name: n,
            kind: if theorem {
                DeclarationKind::Theorem { ty, proof: body }
            } else {
                DeclarationKind::Def {
                    ty,
                    value: body,
                    reducibility: DefinitionReducibility::Reducible,
                }
            },
        });
        self.b.globals.insert(name, global);
    }
}
fn interfaces(b: &mut Builder) {
    let nat = b.constant("Std.Nat").unwrap();
    let z = b.constant("Std.Nat.zero").unwrap();
    let s = b.constant("Std.Nat.succ").unwrap();
    let p_ty = b.pi(nat, b.sort).unwrap();
    let p = b.var(0).unwrap();
    let z_ty = b.app(p, vec![z]).unwrap();
    let p = b.var(2).unwrap();
    let n = b.var(0).unwrap();
    let ih = b.app(p, vec![n]).unwrap();
    let p = b.var(3).unwrap();
    let n = b.var(1).unwrap();
    let sn = b.app(s, vec![n]).unwrap();
    let out = b.app(p, vec![sn]).unwrap();
    let step = b.pi(ih, out).unwrap();
    let step = b.pi(nat, step).unwrap();
    let p = b.var(3).unwrap();
    let n = b.var(0).unwrap();
    let out = b.app(p, vec![n]).unwrap();
    let out = b.pi(nat, out).unwrap();
    let out = b.pi(step, out).unwrap();
    let out = b.pi(z_ty, out).unwrap();
    let induct = b.pi(p_ty, out).unwrap();
    let boolean = b.constant("Std.Bool").unwrap();
    let out = b.pi(boolean, b.sort).unwrap();
    let out = b.pi(b.sort, out).unwrap();
    let rec = b.pi(b.sort, out).unwrap();
    for (name, ty, family) in [
        ("Std.Nat.induct", induct, b.globals["Std.Nat"]),
        ("Std.Bool.recType", rec, b.globals["Std.Bool"]),
    ] {
        let global = b.c.declarations.len() as u32;
        let n = b.c.name_table.len() as u32;
        b.c.name_table.push(name.into());
        b.c.declarations.push(Declaration {
            name: n,
            kind: DeclarationKind::Recursor {
                ty,
                inductive: family,
                generated: true,
            },
        });
        b.globals.insert(name.into(), global);
    }
}
fn split_args(a: Expr, n: Expr, m: Expr) -> Vec<(String, Expr)> {
    vec![
        (
            "kl".into(),
            forall("_lt", lt(m.clone(), n.clone()), a.clone()),
        ),
        (
            "ke".into(),
            forall("_eq", neq(m.clone(), n.clone()), a.clone()),
        ),
        ("h".into(), lt(m, succ(n))),
    ]
}
fn split_at(a: Expr, n: Expr, m: Expr) -> Expr {
    result(&split_args(a.clone(), n, m), a)
}
fn split_proof() -> Expr {
    let a = v("A");
    let outer_motive = lambda(
        "on",
        nt(),
        forall("om", nt(), split_at(a.clone(), v("on"), v("om"))),
    );
    let base_motive = lambda("bm", nt(), split_at(a.clone(), zero(), v("bm")));
    let base_zero = wrapped(
        &split_args(a.clone(), zero(), zero()),
        apply(v("ke"), vec![nrfl(zero())]),
    );
    let base_succ = lambda(
        "bm",
        nt(),
        lambda(
            "bih",
            split_at(a.clone(), zero(), v("bm")),
            wrapped(
                &split_args(a.clone(), zero(), succ(v("bm"))),
                app("FalseElim", vec![a.clone(), v("h")]),
            ),
        ),
    );
    let outer_base = induct(base_motive, base_zero, base_succ, v("m"));
    let outer_base = lambda("m", nt(), outer_base);
    let step_motive = lambda("sm", nt(), split_at(a.clone(), succ(v("on")), v("sm")));
    let step_zero = wrapped(
        &split_args(a.clone(), succ(v("on")), zero()),
        apply(v("kl"), vec![refl(literal(true))]),
    );
    let congr = apply(
        c("Std.Eq.congr"),
        vec![nt(), nt(), c("Std.Nat.succ"), v("sm"), v("on"), v("eq")],
    );
    let equal_handler = lambda("eq", neq(v("sm"), v("on")), apply(v("ke"), vec![congr]));
    let step_body = apply(v("oih"), vec![v("sm"), v("kl"), equal_handler, v("h")]);
    let step_succ = lambda(
        "sm",
        nt(),
        lambda(
            "sih",
            split_at(a.clone(), succ(v("on")), v("sm")),
            wrapped(
                &split_args(a.clone(), succ(v("on")), succ(v("sm"))),
                step_body,
            ),
        ),
    );
    let inner = induct(step_motive, step_zero, step_succ, v("m"));
    let outer_step = lambda(
        "on",
        nt(),
        lambda(
            "oih",
            forall("om", nt(), split_at(a.clone(), v("on"), v("om"))),
            lambda("m", nt(), inner),
        ),
    );
    induct(outer_motive, outer_base, outer_step, v("n"))
}
fn strong_proof() -> Expr {
    let p = v("P");
    let init = lambda(
        "hm",
        nt(),
        lambda(
            "hh",
            lt(v("hm"), zero()),
            app("FalseElim", vec![apply(p.clone(), vec![v("hm")]), v("hh")]),
        ),
    );
    let pn = apply(v("step"), vec![v("hn"), v("hih")]);
    let reverse = apply(c("Std.Eq.symm"), vec![nt(), v("hm"), v("hn"), v("heq")]);
    let rewrite = apply(
        c("Std.Eq.rewrite"),
        vec![nt(), v("hn"), v("hm"), p.clone(), reverse, pn],
    );
    let equal = lambda("heq", neq(v("hm"), v("hn")), rewrite);
    let lower = lambda(
        "hlt",
        lt(v("hm"), v("hn")),
        apply(v("hih"), vec![v("hm"), v("hlt")]),
    );
    let split = app(
        "Split",
        vec![
            apply(p.clone(), vec![v("hm")]),
            v("hn"),
            v("hm"),
            lower,
            equal,
            v("hh"),
        ],
    );
    let step = lambda(
        "hn",
        nt(),
        lambda(
            "hih",
            history(p.clone(), v("hn")),
            lambda("hm", nt(), lambda("hh", lt(v("hm"), succ(v("hn"))), split)),
        ),
    );
    let motive = lambda("hn", nt(), history(p, v("hn")));
    let all = induct(motive, init, step, v("n"));
    apply(v("step"), vec![v("n"), all])
}
fn strong_args(weaken: bool) -> Vec<(String, Expr)> {
    let p = v("P");
    let value = if weaken {
        eq(literal(true), literal(true))
    } else {
        apply(p.clone(), vec![v("k")])
    };
    vec![
        ("P".into(), forall("_n", nt(), ty0())),
        (
            "step".into(),
            forall("k", nt(), forall("_hist", history(p, v("k")), value)),
        ),
        ("n".into(), nt()),
    ]
}
fn library(root: &Path) -> Certificate {
    let mut l = Library {
        b: Builder::from(natural::foundation(root)),
    };
    interfaces(&mut l.b);
    l.b.c.module = MODULE.into();
    l.declare(
        "Pred",
        &[("n".into(), nt())],
        nt(),
        apply(
            c("Std.Nat.rec"),
            vec![
                zero(),
                lambda("k", nt(), lambda("acc", nt(), v("k"))),
                v("n"),
            ],
        ),
        false,
    );
    l.declare(
        "Add",
        &[("a".into(), nt()), ("b".into(), nt())],
        nt(),
        apply(
            c("Std.Nat.rec"),
            vec![
                v("a"),
                lambda("k", nt(), lambda("acc", nt(), succ(v("acc")))),
                v("b"),
            ],
        ),
        false,
    );
    l.declare(
        "Sub",
        &[("a".into(), nt()), ("b".into(), nt())],
        nt(),
        apply(
            c("Std.Nat.rec"),
            vec![
                v("a"),
                lambda("k", nt(), lambda("acc", nt(), app("Pred", vec![v("acc")]))),
                v("b"),
            ],
        ),
        false,
    );
    let fty = forall("_m", nt(), bool_ty());
    let inner = induct(
        lambda("_m", nt(), bool_ty()),
        literal(true),
        lambda(
            "k",
            nt(),
            lambda("_prev", bool_ty(), apply(v("ih"), vec![v("k")])),
        ),
        v("m"),
    );
    let f = induct(
        lambda("_n", nt(), fty.clone()),
        lambda("_m", nt(), literal(false)),
        lambda("k", nt(), lambda("ih", fty, lambda("m", nt(), inner))),
        v("n"),
    );
    l.declare(
        "Less",
        &[("m".into(), nt()), ("n".into(), nt())],
        bool_ty(),
        apply(f, vec![v("m")]),
        false,
    );
    l.declare(
        "BoolNat",
        &[("b".into(), bool_ty())],
        nt(),
        choose_t(nt(), v("b"), succ(zero()), zero()),
        false,
    );
    let ts = rec_type(v("a"), v("b"), v("q"));
    l.declare(
        "TypeSelect",
        &[
            ("q".into(), bool_ty()),
            ("a".into(), ty0()),
            ("b".into(), ty0()),
        ],
        ty0(),
        ts,
        false,
    );
    let motive = lambda(
        "q",
        bool_ty(),
        app("TypeSelect", vec![v("q"), bool_ty(), v("A")]),
    );
    let proof = apply(
        c("Std.Eq.rewrite"),
        vec![
            bool_ty(),
            literal(false),
            literal(true),
            motive,
            v("h"),
            literal(true),
        ],
    );
    l.declare(
        "FalseElim",
        &[
            ("A".into(), ty0()),
            ("h".into(), eq(literal(false), literal(true))),
        ],
        v("A"),
        proof,
        true,
    );
    let h = forall(
        "m",
        nt(),
        forall("_lt", lt(v("m"), v("n")), apply(v("P"), vec![v("m")])),
    );
    l.declare(
        "History",
        &[("P".into(), forall("_n", nt(), ty0())), ("n".into(), nt())],
        ty0(),
        h,
        false,
    );
    let mut a = vec![("A".into(), ty0()), ("n".into(), nt()), ("m".into(), nt())];
    a.extend(split_args(v("A"), v("n"), v("m")));
    l.declare(
        "Split",
        &a,
        v("A"),
        apply(split_proof(), vec![v("m"), v("kl"), v("ke"), v("h")]),
        true,
    );
    l.declare(
        "Strong",
        &strong_args(false),
        apply(v("P"), vec![v("n")]),
        strong_proof(),
        true,
    );
    l.b.c
}
pub(super) fn generate(root: &Path, output: &Path) {
    fs::create_dir_all(output).unwrap();
    let base = library(root);
    let mut cases = vec![];
    for family in [
        "less-zero",
        "less-zero-successor",
        "less-successor-cancel",
        "less-irreflexive",
        "strong-induction",
        "type-select-zero",
        "type-select-successor",
        "false-elimination",
    ] {
        for altered in [false, true] {
            let mut l = Library {
                b: Builder::from(base.clone()),
            };
            let (a, goal, proof) = match family {
                "less-zero" => (
                    vec![("m".into(), nt())],
                    eq(less(v("m"), zero()), literal(altered)),
                    refl(literal(false)),
                ),
                "less-zero-successor" => (
                    vec![("n".into(), nt())],
                    eq(less(zero(), succ(v("n"))), literal(!altered)),
                    refl(literal(true)),
                ),
                "less-successor-cancel" => {
                    let x = less(v("m"), v("n"));
                    let target = if altered { not(x.clone()) } else { x.clone() };
                    (
                        vec![("m".into(), nt()), ("n".into(), nt())],
                        eq(less(succ(v("m")), succ(v("n"))), target),
                        refl(x),
                    )
                }
                "less-irreflexive" => {
                    let motive = lambda("k", nt(), eq(less(v("k"), v("k")), literal(false)));
                    let step = lambda(
                        "k",
                        nt(),
                        lambda("ih", eq(less(v("k"), v("k")), literal(false)), v("ih")),
                    );
                    (
                        vec![("n".into(), nt())],
                        eq(less(v("n"), v("n")), literal(altered)),
                        induct(motive, refl(literal(false)), step, v("n")),
                    )
                }
                "strong-induction" => (
                    strong_args(altered),
                    apply(v("P"), vec![v("n")]),
                    app("Strong", vec![v("P"), v("step"), v("n")]),
                ),
                "type-select-zero" => (
                    vec![("A".into(), ty0()), ("x".into(), v("A"))],
                    app("TypeSelect", vec![literal(altered), v("A"), bool_ty()]),
                    v("x"),
                ),
                "type-select-successor" => (
                    vec![],
                    app("TypeSelect", vec![literal(!altered), bool_ty(), nt()]),
                    zero(),
                ),
                _ => (
                    vec![
                        ("A".into(), ty0()),
                        ("h".into(), eq(literal(altered), literal(true))),
                    ],
                    v("A"),
                    app("FalseElim", vec![v("A"), v("h")]),
                ),
            };
            l.declare("Claim", &a, goal.clone(), proof, true);
            let declarations = l.b.c.declarations.len();
            let terms = l.b.c.term_table.len();
            let raw = l.b.finish();
            let id = format!("{family}-{}", if altered { "altered" } else { "valid" });
            fs::write(output.join(format!("{id}.mpcert")), &raw).unwrap();
            cases.push(json!({"id":id,"family":family,"altered":altered,"expected":if altered {"rejected"}else{"accepted"},
                "bytes":raw.len(),"terms":terms,"declarations":declarations,"theorem":named("Claim"),
                "scope":"ordinary arbitrary-Nat/Sort0 type and strong-induction schema; no source application or completed loop variant proof",
                "goal_debug":format!("{goal:?}")}));
        }
    }
    fs::write(
        output.join("index.json"),
        serde_json::to_vec_pretty(&json!({
        "schema":"mpk.go_practical.nat_termination_pilot.v0","cases":cases}))
        .unwrap(),
    )
    .unwrap();
}
