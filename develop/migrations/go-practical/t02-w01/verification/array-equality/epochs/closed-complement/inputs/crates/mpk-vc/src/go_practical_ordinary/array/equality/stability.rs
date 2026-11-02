//! Actual comparator congruence and input normalization preserve both operands.
use super::*;
fn cmp_congruence(a: Expr, c: Expr, x: Equality, y: Equality) -> Result<Equality> {
    let first = x.clone().mapped(
        lam(
            "_x",
            a.clone(),
            compared(c.clone(), var("_x"), y.left.clone()),
        ),
        compared(c.clone(), x.left, y.left.clone()),
        compared(c.clone(), x.right.clone(), y.left.clone()),
        boolean(),
    );
    first.then(y.clone().mapped(
        lam("_y", a, compared(c.clone(), x.right.clone(), var("_y"))),
        compared(c.clone(), x.right.clone(), y.left),
        compared(c, x.right, y.right),
        boolean(),
    ))
}
fn pointwise(d: usize, body: Expr) -> Expr {
    cube(
        d,
        lam("_in", inside(idx(d, address(d)), var("n")), body),
        true,
    )
}
fn pointwise_arguments(f: Expr, g: Expr, u: Expr, v: Expr, h: Expr) -> Vec<Expr> {
    let mut args = value_args(f, g);
    args.extend([u, v, h]);
    args
}
impl Comparisons<'_> {
    pub(super) fn stability(&mut self, d: usize) -> Result<()> {
        let (a, f, g) = (var("A"), var("f"), var("g"));
        let at = address(d);
        let mut args = actual(d);
        args.extend([("zf", a.clone()), ("zg", a.clone())]);
        let norm = |data: Expr, z: Expr| ac(d, "Normalize", vec![a.clone(), z, var("n"), data]);
        let norm_eq = |data: Expr, z: Expr| {
            let mut args = vec![a.clone(), z.clone(), var("n"), data.clone()];
            args.extend(at.clone());
            args.push(var("_in"));
            Equality {
                ty: a.clone(),
                left: slot(norm(data.clone(), z), at.clone()),
                right: slot(data, at.clone()),
                proof: ac(d, "NormalizeInside", args),
            }
        };
        let f_norm = norm(f.clone(), var("zf"));
        let g_norm = norm(g.clone(), var("zg"));
        let same = cmp_congruence(
            a.clone(),
            var("cmp"),
            norm_eq(f.clone(), var("zf")),
            norm_eq(g.clone(), var("zg")),
        )?;
        self.theorem(
            d,
            "Normalize",
            &args,
            Equality::boolean(
                equal_by(d, f_norm.clone(), g_norm.clone()),
                equal_by(d, f.clone(), g.clone()),
                qc(
                    d,
                    "Pointwise",
                    pointwise_arguments(
                        f_norm,
                        g_norm,
                        f.clone(),
                        g.clone(),
                        pointwise(d, same.proof),
                    ),
                ),
            ),
        )?;
        let copy = |data: Expr| ac(d, "Copy", vec![a.clone(), data]);
        let copy_eq = |data: Expr| {
            let x = copy(data.clone());
            Equality {
                ty: cube(d, a.clone(), false),
                left: x.clone(),
                right: data.clone(),
                proof: ac(d, "CopyIdentity", vec![a.clone(), data.clone()]),
            }
            .mapped(
                lam("_f", cube(d, a.clone(), false), slot(var("_f"), at.clone())),
                slot(x, at.clone()),
                slot(data, at.clone()),
                a.clone(),
            )
        };
        let same = cmp_congruence(
            a.clone(),
            var("cmp"),
            copy_eq(f.clone()),
            copy_eq(g.clone()),
        )?;
        self.theorem(
            d,
            "Copy",
            &actual(d),
            Equality::boolean(
                equal_by(d, copy(f.clone()), copy(g.clone())),
                equal_by(d, f.clone(), g.clone()),
                qc(
                    d,
                    "Pointwise",
                    pointwise_arguments(
                        copy(f.clone()),
                        copy(g.clone()),
                        f.clone(),
                        g.clone(),
                        pointwise(d, same.proof),
                    ),
                ),
            ),
        )?;
        self.related_stability(d)
    }
    fn related_stability(&mut self, d: usize) -> Result<()> {
        let (a, p, s, f, g, u, v) = (
            var("A"),
            var("P"),
            var("S"),
            var("f"),
            var("g"),
            var("u"),
            var("v"),
        );
        let mut args = vec![
            ("A", Expr::Sort0),
            ("P", pred(a.clone())),
            ("S", relation(a.clone())),
        ];
        args.extend(actual(d).into_iter().skip(1));
        args.extend([
            ("u", cube(d, a.clone(), false)),
            ("v", cube(d, a.clone(), false)),
            ("fp", elements(d, p.clone(), f.clone())),
            ("gp", elements(d, p.clone(), g.clone())),
            ("up", elements(d, p.clone(), u.clone())),
            ("vp", elements(d, p.clone(), v.clone())),
            ("fu", related(d, s.clone(), f.clone(), u.clone())),
            ("gv", related(d, s.clone(), g.clone(), v.clone())),
        ]);
        let (x, xx, y, yy) = (var("_x"), var("_xx"), var("_y"), var("_yy"));
        let bindings = [
            ("_x", a.clone()),
            ("_xx", a.clone()),
            ("_y", a.clone()),
            ("_yy", a.clone()),
            ("_px", app(p.clone(), vec![x.clone()])),
            ("_pxx", app(p.clone(), vec![xx.clone()])),
            ("_py", app(p.clone(), vec![y.clone()])),
            ("_pyy", app(p, vec![yy.clone()])),
            ("_xr", app(s.clone(), vec![x.clone(), xx.clone()])),
            ("_yr", app(s.clone(), vec![y.clone(), yy.clone()])),
        ];
        let law = bindings.into_iter().rev().fold(
            eq(
                boolean(),
                compared(var("cmp"), x, y),
                compared(var("cmp"), xx, yy),
            ),
            |body, (name, ty)| pi(name, ty, body),
        );
        args.push(("preserve", law));
        let at = address(d);
        let witness = |f: Expr, g: Expr, h| {
            let mut args = vec![a.clone(), a.clone(), s.clone(), var("n"), f, g, var(h)];
            args.extend(at.clone());
            args.push(var("_in"));
            ac(d, "RelatedSlot", args)
        };
        let same = app(
            var("preserve"),
            vec![
                slot(f.clone(), at.clone()),
                slot(u.clone(), at.clone()),
                slot(g.clone(), at.clone()),
                slot(v.clone(), at.clone()),
                slot(var("fp"), at.clone()),
                slot(var("up"), at.clone()),
                slot(var("gp"), at.clone()),
                slot(var("vp"), at.clone()),
                witness(f.clone(), u.clone(), "fu"),
                witness(g.clone(), v.clone(), "gv"),
            ],
        );
        self.theorem(
            d,
            "RelatedRespects",
            &args,
            Equality::boolean(
                equal_by(d, f.clone(), g.clone()),
                equal_by(d, u.clone(), v.clone()),
                qc(
                    d,
                    "Pointwise",
                    pointwise_arguments(f, g, u, v, pointwise(d, same)),
                ),
            ),
        )
    }
}
