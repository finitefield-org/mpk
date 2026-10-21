//! Full fixed-array comparisons require actual per-element semantic witnesses.
use super::*;
mod preservation;
mod reflection;
fn qc(d: usize, name: &str, args: Vec<Expr>) -> Expr {
    call(
        &format!("Mpk.Go.Practical.Array.Equality.D{d}.{name}"),
        args,
    )
}
fn ec(d: usize, name: &str, args: Vec<Expr>) -> Expr {
    call(
        &format!("Mpk.Go.Practical.String.Equality.D{d}.{name}"),
        args,
    )
}
fn comparator(a: Expr) -> Expr {
    pi("_x", a.clone(), pi("_y", a, boolean()))
}
fn relation(a: Expr) -> Expr {
    pi("_x", a.clone(), pi("_y", a, Expr::Sort0))
}
fn compared(c: Expr, f: Expr, g: Expr) -> Expr {
    app(c, vec![f, g])
}
fn parameters() -> Vec<Binder> {
    vec![
        ("A", Expr::Sort0),
        ("n", nat()),
        ("f", cube(0, var("A"), false)),
        ("g", cube(0, var("A"), false)),
        ("cmp", comparator(var("A"))),
    ]
}
fn actual(d: usize) -> Vec<Binder> {
    let mut a = parameters();
    a[2].1 = cube(d, var("A"), false);
    a[3].1 = cube(d, var("A"), false);
    a
}
fn value_args(f: Expr, g: Expr) -> Vec<Expr> {
    vec![var("A"), var("n"), f, g, var("cmp")]
}
fn flags(d: usize, f: Expr, g: Expr) -> Expr {
    qc(d, "Flags", value_args(f, g))
}
fn equal_by(d: usize, f: Expr, g: Expr) -> Expr {
    qc(d, "EqualBy", value_args(f, g))
}
fn unequal_by(d: usize, f: Expr, g: Expr) -> Expr {
    qc(d, "NotEqualBy", value_args(f, g))
}
fn related(d: usize, r: Expr, f: Expr, g: Expr) -> Expr {
    ac(d, "Related", vec![var("A"), var("A"), r, var("n"), f, g])
}
fn element_law(a: Expr, p: Expr, r: Expr, c: Expr, reflect: bool) -> Expr {
    let x = var("_x");
    let y = var("_y");
    let premise = if reflect {
        truth(compared(c.clone(), x.clone(), y.clone()), true)
    } else {
        app(r.clone(), vec![x.clone(), y.clone()])
    };
    let result = if reflect {
        app(r, vec![x.clone(), y.clone()])
    } else {
        truth(compared(c, x.clone(), y.clone()), true)
    };
    pi(
        "_x",
        a.clone(),
        pi(
            "_y",
            a,
            pi(
                "_px",
                app(p.clone(), vec![x]),
                pi("_py", app(p, vec![y]), pi("_h", premise, result)),
            ),
        ),
    )
}
struct Comparisons<'a>(&'a mut Builder);
impl Comparisons<'_> {
    fn emit(
        &mut self,
        d: usize,
        name: &str,
        args: &[Binder],
        ty: Expr,
        body: Expr,
        law: bool,
    ) -> Result<()> {
        emit(
            self.0,
            &format!("Mpk.Go.Practical.Array.Equality.D{d}.{name}"),
            args,
            ty,
            body,
            law,
        )
    }
    fn theorem(&mut self, d: usize, name: &str, args: &[Binder], e: Equality) -> Result<()> {
        self.emit(d, name, args, eq(e.ty, e.left, e.right), e.proof, true)
    }
    fn definitions(&mut self, d: usize) -> Result<()> {
        let (f, g) = (var("f"), var("g"));
        let args = actual(d);
        let at = address(d);
        let inside = less(idx(d, at.clone()), var("n"));
        let cmp = compared(
            var("cmp"),
            slot(f.clone(), at.clone()),
            slot(g.clone(), at.clone()),
        );
        let body = choose(boolean(), inside.clone(), cmp.clone(), bit(true));
        self.emit(
            d,
            "Flags",
            &args,
            cube(d, boolean(), false),
            cube(d, body.clone(), true),
            false,
        )?;
        let mut slot_args = args.clone();
        slot_args.extend(BITS[..d].iter().map(|b| (*b, boolean())));
        self.theorem(
            d,
            "FlagsStep",
            &slot_args,
            Equality::boolean(
                slot(flags(d, f.clone(), g.clone()), at.clone()),
                body.clone(),
                refl(boolean(), body.clone()),
            ),
        )?;
        for yes in [false, true] {
            let mut next = slot_args.clone();
            next.push(("within", truth(inside.clone(), yes)));
            self.theorem(
                d,
                if yes { "FlagInside" } else { "FlagOutside" },
                &next,
                Equality::boolean(
                    slot(flags(d, f.clone(), g.clone()), at.clone()),
                    if yes { cmp.clone() } else { bit(true) },
                    seq(
                        if yes { "ChoiceTrue" } else { "ChoiceFalse" },
                        vec![
                            boolean(),
                            inside.clone(),
                            cmp.clone(),
                            bit(true),
                            var("within"),
                        ],
                    ),
                ),
            )?;
        }
        let fold = ec(d, "All", vec![flags(d, f.clone(), g.clone())]);
        self.emit(d, "EqualBy", &args, boolean(), fold.clone(), false)?;
        self.theorem(
            d,
            "EqualStep",
            &args,
            Equality::boolean(
                equal_by(d, f.clone(), g.clone()),
                fold.clone(),
                refl(boolean(), fold),
            ),
        )?;
        let negative = bc("Not", vec![equal_by(d, f.clone(), g.clone())]);
        self.emit(d, "NotEqualBy", &args, boolean(), negative.clone(), false)?;
        self.theorem(
            d,
            "NotEqualStep",
            &args,
            Equality::boolean(
                unequal_by(d, f, g),
                negative.clone(),
                refl(boolean(), negative),
            ),
        )
    }
    fn fold_congruence(&mut self, d: usize) -> Result<()> {
        let (f, g) = (var("f"), var("g"));
        let at = address(d);
        let args = [
            ("f", cube(d, boolean(), false)),
            ("g", cube(d, boolean(), false)),
            (
                "h",
                cube(
                    d,
                    eq(boolean(), slot(f.clone(), at.clone()), slot(g.clone(), at)),
                    false,
                ),
            ),
        ];
        let proof = if d == 0 {
            var("h")
        } else {
            let low_f = slot(f.clone(), vec![bit(false)]);
            let high_f = slot(f.clone(), vec![bit(true)]);
            let low_g = slot(g.clone(), vec![bit(false)]);
            let high_g = slot(g.clone(), vec![bit(true)]);
            let low = Equality::boolean(
                ec(d - 1, "All", vec![low_f.clone()]),
                ec(d - 1, "All", vec![low_g.clone()]),
                qc(
                    d - 1,
                    "FoldCongruence",
                    vec![low_f, low_g, slot(var("h"), vec![bit(false)])],
                ),
            );
            let high = Equality::boolean(
                ec(d - 1, "All", vec![high_f.clone()]),
                ec(d - 1, "All", vec![high_g.clone()]),
                qc(
                    d - 1,
                    "FoldCongruence",
                    vec![high_f, high_g, slot(var("h"), vec![bit(true)])],
                ),
            );
            let start = Equality::boolean(
                ec(d, "All", vec![f.clone()]),
                bc("And", vec![low.left.clone(), high.left.clone()]),
                ec(d, "Step", vec![f.clone()]),
            );
            let first = low.clone().mapped(
                lam(
                    "_b",
                    boolean(),
                    bc("And", vec![var("_b"), high.left.clone()]),
                ),
                bc("And", vec![low.left, high.left.clone()]),
                bc("And", vec![low.right.clone(), high.left.clone()]),
                boolean(),
            );
            let last = high.clone().mapped(
                lam(
                    "_b",
                    boolean(),
                    bc("And", vec![low.right.clone(), var("_b")]),
                ),
                bc("And", vec![low.right.clone(), high.left]),
                bc("And", vec![low.right.clone(), high.right.clone()]),
                boolean(),
            );
            start
                .then(first)?
                .then(last)?
                .then(
                    Equality::boolean(
                        ec(d, "All", vec![g.clone()]),
                        bc("And", vec![low.right, high.right]),
                        ec(d, "Step", vec![g.clone()]),
                    )
                    .reverse(),
                )?
                .proof
        };
        self.theorem(
            d,
            "FoldCongruence",
            &args,
            Equality::boolean(ec(d, "All", vec![f]), ec(d, "All", vec![g]), proof),
        )
    }
}
pub(super) fn append(builder: &mut Builder) -> Result<()> {
    let mut c = Comparisons(builder);
    for d in 0..=8 {
        c.definitions(d)?;
        c.fold_congruence(d)?;
        c.reflection(d)?;
        c.preservation(d)?;
    }
    Ok(())
}
