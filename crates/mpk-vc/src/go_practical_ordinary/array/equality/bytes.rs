//! Closed Word8 instantiation consumes the actual existing eight-bit comparator.
use super::*;
fn byte_type() -> Expr {
    cube(3, boolean(), false)
}
fn scalar(name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("Mpk.Go.Practical.String.Equality.{name}"), args)
}
fn predicate() -> Expr {
    lam("_x", byte_type(), truth(bit(false), false))
}
fn byte_relation() -> Expr {
    lam(
        "_x",
        byte_type(),
        lam(
            "_y",
            byte_type(),
            cube(
                3,
                eq(
                    boolean(),
                    slot(var("_x"), address(3)),
                    slot(var("_y"), address(3)),
                ),
                false,
            ),
        ),
    )
}
fn cmp() -> Expr {
    scalar("ByteEqual", vec![])
}
fn byte_value(d: usize, negative: bool, f: Expr, g: Expr) -> Expr {
    qc(
        d,
        if negative {
            "ByteNotEqual"
        } else {
            "ByteEqual"
        },
        vec![var("n"), f, g],
    )
}
fn dictionary(reflect: bool) -> Expr {
    let x = var("_x");
    let y = var("_y");
    let r = app(byte_relation(), vec![x.clone(), y.clone()]);
    let yes = truth(scalar("ByteEqual", vec![x.clone(), y.clone()]), true);
    lam(
        "_x",
        byte_type(),
        lam(
            "_y",
            byte_type(),
            lam(
                "_px",
                truth(bit(false), false),
                lam(
                    "_py",
                    truth(bit(false), false),
                    lam(
                        "_h",
                        if reflect { yes } else { r },
                        scalar(
                            if reflect {
                                "ByteReflect"
                            } else {
                                "ByteComplete"
                            },
                            vec![x, y, var("_h")],
                        ),
                    ),
                ),
            ),
        ),
    )
}
fn generic_arguments(d: usize, f: Expr, g: Expr, h: Expr, reflect: bool) -> Vec<Expr> {
    vec![
        byte_type(),
        predicate(),
        byte_relation(),
        var("n"),
        f,
        g,
        cmp(),
        cube(d, refl(boolean(), bit(false)), true),
        cube(d, refl(boolean(), bit(false)), true),
        dictionary(reflect),
        h,
    ]
}
impl Comparisons<'_> {
    pub(super) fn bytes(&mut self, d: usize) -> Result<()> {
        let (f, g) = (var("f"), var("g"));
        let args = [
            ("n", nat()),
            ("f", cube(d, byte_type(), false)),
            ("g", cube(d, byte_type(), false)),
        ];
        for negative in [false, true] {
            self.emit(
                d,
                if negative {
                    "ByteNotEqual"
                } else {
                    "ByteEqual"
                },
                &args,
                boolean(),
                qc(
                    d,
                    if negative { "NotEqualBy" } else { "EqualBy" },
                    vec![byte_type(), var("n"), f.clone(), g.clone(), cmp()],
                ),
                false,
            )?;
        }
        self.theorem(
            d,
            "ByteNotEqualStep",
            &args,
            Equality::boolean(
                byte_value(d, true, f.clone(), g.clone()),
                bc("Not", vec![byte_value(d, false, f.clone(), g.clone())]),
                qc(
                    d,
                    "NotEqualStep",
                    vec![byte_type(), var("n"), f.clone(), g.clone(), cmp()],
                ),
            ),
        )?;
        let r = ac(
            d,
            "Related",
            vec![
                byte_type(),
                byte_type(),
                byte_relation(),
                var("n"),
                f.clone(),
                g.clone(),
            ],
        );
        let mut reflect = args.to_vec();
        reflect.push(("h", truth(byte_value(d, false, f.clone(), g.clone()), true)));
        self.emit(
            d,
            "ByteReflect",
            &reflect,
            r.clone(),
            qc(
                d,
                "Reflect",
                generic_arguments(d, f.clone(), g.clone(), var("h"), true),
            ),
            true,
        )?;
        let mut complete = args.to_vec();
        complete.push(("h", r));
        self.theorem(
            d,
            "ByteComplete",
            &complete,
            Equality::boolean(
                byte_value(d, false, f.clone(), g.clone()),
                bit(true),
                qc(
                    d,
                    "Complete",
                    generic_arguments(d, f.clone(), g.clone(), var("h"), false),
                ),
            ),
        )?;
        let mut read_args = reflect;
        read_args.extend([
            ("i", word()),
            ("shape", shape(d, var("n"))),
            ("within", inside(value(var("i")), var("n"))),
        ]);
        let mut supplied = generic_arguments(d, f.clone(), g.clone(), var("h"), true);
        supplied.extend([var("i"), var("shape"), var("within")]);
        self.emit(
            d,
            "ByteReadReflect",
            &read_args,
            app(
                byte_relation(),
                vec![
                    read(d, byte_type(), f.clone(), var("i")),
                    read(d, byte_type(), g, var("i")),
                ],
            ),
            qc(d, "ReadReflect", supplied),
            true,
        )?;
        let element_refl = lam(
            "_x",
            byte_type(),
            lam(
                "_px",
                truth(bit(false), false),
                scalar(
                    "ByteComplete",
                    vec![
                        var("_x"),
                        var("_x"),
                        scalar("ByteSameRefl", vec![var("_x")]),
                    ],
                ),
            ),
        );
        self.theorem(
            d,
            "ByteReflexive",
            &args[..2],
            Equality::boolean(
                byte_value(d, false, f.clone(), f.clone()),
                bit(true),
                qc(
                    d,
                    "Reflexive",
                    vec![
                        byte_type(),
                        predicate(),
                        var("n"),
                        f,
                        cmp(),
                        cube(d, refl(boolean(), bit(false)), true),
                        element_refl,
                    ],
                ),
            ),
        )
    }
}
