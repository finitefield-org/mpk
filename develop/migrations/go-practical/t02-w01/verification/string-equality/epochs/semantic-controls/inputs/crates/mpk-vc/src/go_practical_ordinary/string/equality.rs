//! Actual bit/byte/prefix equality. Fixed Bool-depth induction supplies all
//! observations, not operand enumeration or equality of functions by fiat.
use super::*;

const SELECTORS: [&str; 8] = ["q0", "q1", "q2", "q3", "q4", "q5", "q6", "q7"];
fn ec(name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("Mpk.Go.Practical.String.Equality.{name}"), args)
}
fn norm(s: Expr) -> Expr {
    call("Mpk.Go.Practical.String.Normalization.Value", vec![s])
}
fn strong(s: Expr, t: Expr) -> Expr {
    call(
        "Mpk.Go.Practical.String.Normalization.Equivalent",
        vec![s, t],
    )
}
fn all(d: usize, f: Expr) -> Expr {
    ec(&format!("D{d}.All"), vec![f])
}
fn flags(d: usize) -> Expr {
    cube(&SELECTORS[..d], boolean(), false)
}
fn truth(b: Expr) -> Expr {
    eq(boolean(), b, bit(true))
}
fn all_true(d: usize, f: Expr) -> Expr {
    cube(
        &SELECTORS[..d],
        truth(app(f, address_bits(&SELECTORS[..d]))),
        false,
    )
}
fn bool_left(name: &str, e: Equality, b: Expr) -> Equality {
    let (left, right) = (
        bc(name, vec![e.left.clone(), b.clone()]),
        bc(name, vec![e.right.clone(), b.clone()]),
    );
    e.mapped(
        lam("_flag", boolean(), bc(name, vec![var("_flag"), b])),
        left,
        right,
        boolean(),
    )
}
fn bool_right(name: &str, a: Expr, e: Equality) -> Equality {
    let (left, right) = (
        bc(name, vec![a.clone(), e.left.clone()]),
        bc(name, vec![a.clone(), e.right.clone()]),
    );
    e.mapped(
        lam("_flag", boolean(), bc(name, vec![a, var("_flag")])),
        left,
        right,
        boolean(),
    )
}
fn less_right(i: Expr, e: Equality) -> Equality {
    let (left, right) = (
        less(i.clone(), e.left.clone()),
        less(i.clone(), e.right.clone()),
    );
    e.mapped(lam("_n", nat(), less(i, var("_n"))), left, right, boolean())
}
fn byte_same(x: Expr, y: Expr) -> Expr {
    let bits = address_bits(&BITS[..3]);
    cube(
        &BITS[..3],
        eq(boolean(), app(x, bits.clone()), app(y, bits)),
        false,
    )
}
fn byte_equal(x: Expr, y: Expr) -> Expr {
    ec("ByteEqual", vec![x, y])
}
fn byte_flags(x: Expr, y: Expr) -> Expr {
    let bits = address_bits(&SELECTORS[..3]);
    cube(
        &SELECTORS[..3],
        bc("Equal", vec![app(x, bits.clone()), app(y, bits)]),
        true,
    )
}
fn slot(s: Expr, address: Vec<Expr>) -> Expr {
    app(data(s), address)
}
fn inside(s: Expr, address: Vec<Expr>) -> Expr {
    truth(less(index(address), size(s)))
}
fn content(s: Expr, t: Expr) -> Expr {
    let addr = address_bits(&SLOTS);
    cube(
        &SLOTS,
        pi(
            "_inside",
            inside(s.clone(), addr.clone()),
            byte_same(slot(s, addr.clone()), slot(t, addr)),
        ),
        false,
    )
}
fn same(s: Expr, t: Expr) -> Expr {
    ec("Equivalent", vec![s, t])
}
fn same_length(s: Expr, t: Expr, e: Expr) -> Equality {
    Equality::natural(
        size(s.clone()),
        size(t.clone()),
        ec("EquivalentLength", vec![s, t, e]),
    )
}
fn content_at(s: Expr, t: Expr, e: Expr, address: Vec<Expr>, bound: Expr) -> Expr {
    let mut args = address;
    args.push(bound);
    app(ec("EquivalentContents", vec![s, t, e]), args)
}
fn byte_reflexive(x: Expr) -> Expr {
    let bits = address_bits(&BITS[..3]);
    cube(&BITS[..3], refl(boolean(), app(x, bits)), true)
}
fn byte_from_strong(x: Expr, y: Expr, e: Expr) -> Expr {
    ec("ByteFromEquality", vec![x, y, e])
}
fn string_flags(s: Expr, t: Expr) -> Expr {
    let addr = address_bits(&SELECTORS);
    cube(
        &SELECTORS,
        call(
            "Std.Bool.cases",
            vec![
                lam("_case", boolean(), boolean()),
                bit(true),
                byte_equal(slot(s.clone(), addr.clone()), slot(t, addr.clone())),
                less(index(addr), size(s)),
            ],
        ),
        true,
    )
}
fn string_equal(s: Expr, t: Expr) -> Expr {
    ec("Equal", vec![s, t])
}
fn not_equal(s: Expr, t: Expr) -> Expr {
    ec("NotEqual", vec![s, t])
}

struct EqualityProofs<'a>(&'a mut Builder);
impl EqualityProofs<'_> {
    fn emit(
        &mut self,
        name: &str,
        args: &[Binder],
        ty: Expr,
        body: Expr,
        theorem: bool,
    ) -> Result<()> {
        emit(
            self.0,
            &format!("Mpk.Go.Practical.String.Equality.{name}"),
            args,
            ty,
            body,
            theorem,
        )
    }
    fn theorem(&mut self, name: &str, args: &[Binder], e: Equality) -> Result<()> {
        self.emit(name, args, eq(e.ty, e.left, e.right), e.proof, true)
    }
    fn boolean(&mut self) -> Result<()> {
        let (a, b) = (var("a"), var("b"));
        for name in ["AndLeft", "AndRight", "BitReflect"] {
            let statement = |a: Expr, b: Expr| {
                let result = if name == "AndLeft" {
                    truth(a.clone())
                } else if name == "AndRight" {
                    truth(b.clone())
                } else {
                    eq(boolean(), a.clone(), b.clone())
                };
                pi(
                    "_h",
                    truth(bc(
                        if name == "BitReflect" { "Equal" } else { "And" },
                        vec![a, b],
                    )),
                    result,
                )
            };
            let leaf = |a: bool, b: bool| {
                let p = truth(bc(
                    if name == "BitReflect" { "Equal" } else { "And" },
                    vec![bit(a), bit(b)],
                ));
                let proof = if name == "AndLeft" && a || name == "AndRight" && b {
                    refl(boolean(), bit(true))
                } else if name != "BitReflect" || !a {
                    var("_h")
                } else if a == b {
                    refl(boolean(), bit(a))
                } else {
                    call(
                        "Std.Eq.symm",
                        vec![boolean(), bit(false), bit(true), var("_h")],
                    )
                };
                let proof = if name == "BitReflect" && a == b {
                    refl(boolean(), bit(a))
                } else {
                    proof
                };
                lam("_h", p, proof)
            };
            let branch = |a: bool| {
                lam(
                    "b",
                    boolean(),
                    call(
                        "Std.Bool.cases",
                        vec![
                            lam("b", boolean(), statement(bit(a), var("b"))),
                            leaf(a, false),
                            leaf(a, true),
                            b.clone(),
                        ],
                    ),
                )
            };
            self.emit(
                name,
                &[("a", boolean()), ("b", boolean())],
                statement(a.clone(), b.clone()),
                app(
                    call(
                        "Std.Bool.cases",
                        vec![
                            lam(
                                "a",
                                boolean(),
                                pi("b", boolean(), statement(var("a"), var("b"))),
                            ),
                            branch(false),
                            branch(true),
                            a.clone(),
                        ],
                    ),
                    vec![b.clone()],
                ),
                true,
            )?;
        }
        let eq_a = Equality::boolean(a.clone(), bit(true), var("left"));
        let eq_b = Equality::boolean(b.clone(), bit(true), var("right"));
        self.theorem(
            "AndComplete",
            &[
                ("a", boolean()),
                ("b", boolean()),
                ("left", truth(a.clone())),
                ("right", truth(b.clone())),
            ],
            bool_left("And", eq_a, b.clone())
                .then(bool_right("And", bit(true), eq_b))?
                .then(Equality::boolean(
                    bc("And", vec![bit(true), bit(true)]),
                    bit(true),
                    refl(boolean(), bit(true)),
                ))?,
        )?;
        let e = Equality::boolean(a.clone(), b.clone(), var("same"));
        self.theorem(
            "BitComplete",
            &[
                ("a", boolean()),
                ("b", boolean()),
                ("same", eq(boolean(), a.clone(), b.clone())),
            ],
            bool_right("Equal", a.clone(), e.reverse()).then(Equality::boolean(
                bc("Equal", vec![a.clone(), a.clone()]),
                bit(true),
                bc("EqualReflexive", vec![a.clone()]),
            ))?,
        )?;
        let property = |a: Expr, b: Expr| {
            pi(
                "_ab",
                pi("_h", truth(a.clone()), truth(b.clone())),
                pi(
                    "_ba",
                    pi("_h", truth(b.clone()), truth(a.clone())),
                    eq(boolean(), a, b),
                ),
            )
        };
        let leaf = |a: bool, b: bool| {
            let proof = if a == b {
                refl(boolean(), bit(a))
            } else if !a {
                app(var("_ba"), vec![refl(boolean(), bit(true))])
            } else {
                call(
                    "Std.Eq.symm",
                    vec![
                        boolean(),
                        bit(false),
                        bit(true),
                        app(var("_ab"), vec![refl(boolean(), bit(true))]),
                    ],
                )
            };
            lam(
                "_ab",
                pi("_h", truth(bit(a)), truth(bit(b))),
                lam("_ba", pi("_h", truth(bit(b)), truth(bit(a))), proof),
            )
        };
        let branch = |a: bool| {
            lam(
                "b",
                boolean(),
                call(
                    "Std.Bool.cases",
                    vec![
                        lam("b", boolean(), property(bit(a), var("b"))),
                        leaf(a, false),
                        leaf(a, true),
                        var("b"),
                    ],
                ),
            )
        };
        self.emit(
            "TruthIff",
            &[("a", boolean()), ("b", boolean())],
            property(a.clone(), b.clone()),
            app(
                call(
                    "Std.Bool.cases",
                    vec![
                        lam(
                            "a",
                            boolean(),
                            pi("b", boolean(), property(var("a"), var("b"))),
                        ),
                        branch(false),
                        branch(true),
                        a,
                    ],
                ),
                vec![b],
            ),
            true,
        )
    }
    fn fold(&mut self, d: usize) -> Result<()> {
        let f = var("f");
        let h = var("h");
        let ty = flags(d);
        let low = if d == 0 {
            f.clone()
        } else {
            all(d - 1, app(f.clone(), vec![bit(false)]))
        };
        let high = if d == 0 {
            f.clone()
        } else {
            all(d - 1, app(f.clone(), vec![bit(true)]))
        };
        let body = if d == 0 {
            f.clone()
        } else {
            bc("And", vec![low.clone(), high.clone()])
        };
        self.emit(
            &format!("D{d}.All"),
            &[("f", ty.clone())],
            boolean(),
            body.clone(),
            false,
        )?;
        self.theorem(
            &format!("D{d}.Step"),
            &[("f", ty.clone())],
            Equality::boolean(
                all(d, f.clone()),
                body.clone(),
                refl(boolean(), body.clone()),
            ),
        )?;
        let reflect = if d == 0 {
            h.clone()
        } else {
            let pair_truth = Equality::boolean(
                all(d, f.clone()),
                body.clone(),
                ec(&format!("D{d}.Step"), vec![f.clone()]),
            )
            .reverse()
            .then(Equality::boolean(all(d, f.clone()), bit(true), h.clone()))?;
            let branch = |yes| {
                ec(
                    &format!("D{}.Reflect", d - 1),
                    vec![
                        app(f.clone(), vec![bit(yes)]),
                        ec(
                            if yes { "AndRight" } else { "AndLeft" },
                            vec![low.clone(), high.clone(), pair_truth.proof.clone()],
                        ),
                    ],
                )
            };
            lam(
                SELECTORS[0],
                boolean(),
                call(
                    "Std.Bool.cases",
                    vec![
                        lam(
                            SELECTORS[0],
                            boolean(),
                            cube(
                                &SELECTORS[1..d],
                                truth(app(f.clone(), address_bits(&SELECTORS[..d]))),
                                false,
                            ),
                        ),
                        branch(false),
                        branch(true),
                        var(SELECTORS[0]),
                    ],
                ),
            )
        };
        self.emit(
            &format!("D{d}.Reflect"),
            &[("f", ty.clone()), ("h", truth(all(d, f.clone())))],
            all_true(d, f.clone()),
            reflect,
            true,
        )?;
        let complete = if d == 0 {
            h
        } else {
            let a = ec(
                &format!("D{}.Complete", d - 1),
                vec![
                    app(f.clone(), vec![bit(false)]),
                    app(h.clone(), vec![bit(false)]),
                ],
            );
            let b = ec(
                &format!("D{}.Complete", d - 1),
                vec![app(f.clone(), vec![bit(true)]), app(h, vec![bit(true)])],
            );
            Equality::boolean(
                all(d, f.clone()),
                body,
                ec(&format!("D{d}.Step"), vec![f.clone()]),
            )
            .then(Equality::boolean(
                bc("And", vec![low.clone(), high.clone()]),
                bit(true),
                ec("AndComplete", vec![low, high, a, b]),
            ))?
            .proof
        };
        self.emit(
            &format!("D{d}.Complete"),
            &[("f", ty), ("h", all_true(d, f.clone()))],
            truth(all(d, f)),
            complete,
            true,
        )
    }
    fn bytes(&mut self) -> Result<()> {
        let (x, y, z) = (var("x"), var("y"), var("z"));
        let args = [("x", byte()), ("y", byte())];
        let f = byte_flags(x.clone(), y.clone());
        self.emit("ByteEqual", &args, boolean(), all(3, f.clone()), false)?;
        self.theorem(
            "ByteStep",
            &args,
            Equality::boolean(
                byte_equal(x.clone(), y.clone()),
                all(3, f.clone()),
                refl(boolean(), all(3, f.clone())),
            ),
        )?;
        let bits = address_bits(&BITS[..3]);
        let a = app(x.clone(), bits.clone());
        let b = app(y.clone(), bits.clone());
        let fold_truth = Equality::boolean(
            byte_equal(x.clone(), y.clone()),
            all(3, f.clone()),
            ec("ByteStep", vec![x.clone(), y.clone()]),
        )
        .reverse()
        .then(Equality::boolean(
            byte_equal(x.clone(), y.clone()),
            bit(true),
            var("h"),
        ))?;
        let at = app(
            ec("D3.Reflect", vec![f.clone(), fold_truth.proof]),
            bits.clone(),
        );
        self.emit(
            "ByteReflect",
            &[
                args[0].clone(),
                args[1].clone(),
                ("h", truth(byte_equal(x.clone(), y.clone()))),
            ],
            byte_same(x.clone(), y.clone()),
            cube(
                &BITS[..3],
                ec("BitReflect", vec![a.clone(), b.clone(), at]),
                true,
            ),
            true,
        )?;
        let pointwise = cube(
            &BITS[..3],
            ec(
                "BitComplete",
                vec![a.clone(), b.clone(), app(var("h"), bits.clone())],
            ),
            true,
        );
        let proved = ec("D3.Complete", vec![f.clone(), pointwise]);
        self.theorem(
            "ByteComplete",
            &[
                args[0].clone(),
                args[1].clone(),
                ("h", byte_same(x.clone(), y.clone())),
            ],
            Equality::boolean(
                byte_equal(x.clone(), y.clone()),
                all(3, f.clone()),
                ec("ByteStep", vec![x.clone(), y.clone()]),
            )
            .then(Equality::boolean(all(3, f), bit(true), proved))?,
        )?;
        self.emit(
            "ByteSameRefl",
            &[args[0].clone()],
            byte_same(x.clone(), x.clone()),
            byte_reflexive(x.clone()),
            true,
        )?;
        self.emit(
            "ByteSameSym",
            &[
                args[0].clone(),
                args[1].clone(),
                ("h", byte_same(x.clone(), y.clone())),
            ],
            byte_same(y.clone(), x.clone()),
            cube(
                &BITS[..3],
                Equality::boolean(a.clone(), b.clone(), app(var("h"), bits.clone()))
                    .reverse()
                    .proof,
                true,
            ),
            true,
        )?;
        self.emit(
            "ByteSameTrans",
            &[
                args[0].clone(),
                args[1].clone(),
                ("z", byte()),
                ("h", byte_same(x.clone(), y.clone())),
                ("k", byte_same(y.clone(), z.clone())),
            ],
            byte_same(x.clone(), z.clone()),
            cube(
                &BITS[..3],
                Equality::boolean(a.clone(), b.clone(), app(var("h"), bits.clone()))
                    .then(Equality::boolean(
                        b,
                        app(z, bits.clone()),
                        app(var("k"), bits.clone()),
                    ))?
                    .proof,
                true,
            ),
            true,
        )?;
        self.emit(
            "ByteFromEquality",
            &[
                args[0].clone(),
                args[1].clone(),
                ("h", eq(byte(), x.clone(), y.clone())),
            ],
            byte_same(x.clone(), y.clone()),
            cube(
                &BITS[..3],
                Equality {
                    ty: byte(),
                    left: x.clone(),
                    right: y.clone(),
                    proof: var("h"),
                }
                .mapped(
                    lam("_byte", byte(), app(var("_byte"), bits.clone())),
                    app(x, bits.clone()),
                    app(y, bits),
                    boolean(),
                )
                .proof,
                true,
            ),
            true,
        )
    }
    fn equivalence(&mut self) -> Result<()> {
        let (s, t, u, e, f) = (var("s"), var("t"), var("u"), var("e"), var("f"));
        let pair = [("s", string_type()), ("t", string_type())];
        self.emit(
            "Equivalent",
            &pair,
            Expr::Sort0,
            product(
                eq(nat(), size(s.clone()), size(t.clone())),
                content(s.clone(), t.clone()),
            ),
            false,
        )?;
        let mut args = pair.to_vec();
        args.push(("e", same(s.clone(), t.clone())));
        self.emit(
            "EquivalentLength",
            &args,
            eq(nat(), size(s.clone()), size(t.clone())),
            pc(
                "Left",
                vec![
                    eq(nat(), size(s.clone()), size(t.clone())),
                    content(s.clone(), t.clone()),
                    e.clone(),
                ],
            ),
            true,
        )?;
        self.emit(
            "EquivalentContents",
            &args,
            content(s.clone(), t.clone()),
            pc(
                "Right",
                vec![
                    eq(nat(), size(s.clone()), size(t.clone())),
                    content(s.clone(), t.clone()),
                    e.clone(),
                ],
            ),
            true,
        )?;
        let addr = address_bits(&SLOTS);
        let refl_content = cube(
            &SLOTS,
            lam(
                "_inside",
                inside(s.clone(), addr.clone()),
                ec("ByteSameRefl", vec![slot(s.clone(), addr.clone())]),
            ),
            true,
        );
        self.emit(
            "EquivalentRefl",
            &[pair[0].clone()],
            same(s.clone(), s.clone()),
            pc(
                "Make",
                vec![
                    eq(nat(), size(s.clone()), size(s.clone())),
                    content(s.clone(), s.clone()),
                    refl(nat(), size(s.clone())),
                    refl_content,
                ],
            ),
            true,
        )?;
        let len = same_length(s.clone(), t.clone(), e.clone());
        let inside_s = less_right(index(addr.clone()), len.clone()).then(Equality::boolean(
            less(index(addr.clone()), size(t.clone())),
            bit(true),
            var("_inside"),
        ))?;
        let symmetry = cube(
            &SLOTS,
            lam(
                "_inside",
                inside(t.clone(), addr.clone()),
                ec(
                    "ByteSameSym",
                    vec![
                        slot(s.clone(), addr.clone()),
                        slot(t.clone(), addr.clone()),
                        content_at(
                            s.clone(),
                            t.clone(),
                            e.clone(),
                            addr.clone(),
                            inside_s.proof,
                        ),
                    ],
                ),
            ),
            true,
        );
        self.emit(
            "EquivalentSym",
            &args,
            same(t.clone(), s.clone()),
            pc(
                "Make",
                vec![
                    eq(nat(), size(t.clone()), size(s.clone())),
                    content(t.clone(), s.clone()),
                    len.clone().reverse().proof,
                    symmetry,
                ],
            ),
            true,
        )?;
        let inside_t =
            less_right(index(addr.clone()), len.clone().reverse()).then(Equality::boolean(
                less(index(addr.clone()), size(s.clone())),
                bit(true),
                var("_inside"),
            ))?;
        let joined = cube(
            &SLOTS,
            lam(
                "_inside",
                inside(s.clone(), addr.clone()),
                ec(
                    "ByteSameTrans",
                    vec![
                        slot(s.clone(), addr.clone()),
                        slot(t.clone(), addr.clone()),
                        slot(u.clone(), addr.clone()),
                        content_at(
                            s.clone(),
                            t.clone(),
                            e.clone(),
                            addr.clone(),
                            var("_inside"),
                        ),
                        content_at(
                            t.clone(),
                            u.clone(),
                            f.clone(),
                            addr.clone(),
                            inside_t.proof,
                        ),
                    ],
                ),
            ),
            true,
        );
        self.emit(
            "EquivalentTrans",
            &[
                pair[0].clone(),
                pair[1].clone(),
                ("u", string_type()),
                args[2].clone(),
                ("f", same(t.clone(), u.clone())),
            ],
            same(s.clone(), u.clone()),
            pc(
                "Make",
                vec![
                    eq(nat(), size(s.clone()), size(u.clone())),
                    content(s.clone(), u.clone()),
                    len.then(same_length(t.clone(), u.clone(), f))?.proof,
                    joined,
                ],
            ),
            true,
        )?;
        let mut slot_args = vec![s.clone(), t.clone(), e.clone()];
        slot_args.extend(addr.clone());
        slot_args.push(var("_inside"));
        let length = call(
            "Mpk.Go.Practical.String.Normalization.EquivalentLength",
            vec![s.clone(), t.clone(), e.clone()],
        );
        let contents = cube(
            &SLOTS,
            lam(
                "_inside",
                inside(s.clone(), addr.clone()),
                byte_from_strong(
                    slot(s.clone(), addr.clone()),
                    slot(t.clone(), addr),
                    call(
                        "Mpk.Go.Practical.String.Normalization.EquivalentSlot",
                        slot_args,
                    ),
                ),
            ),
            true,
        );
        self.emit(
            "FromStrongEquivalent",
            &[
                pair[0].clone(),
                pair[1].clone(),
                ("e", strong(s.clone(), t.clone())),
            ],
            same(s.clone(), t.clone()),
            pc(
                "Make",
                vec![
                    eq(nat(), size(s.clone()), size(t.clone())),
                    content(s, t),
                    length,
                    contents,
                ],
            ),
            true,
        )?;
        Ok(())
    }
    fn string_operations(&mut self) -> Result<()> {
        let (s, t) = (var("s"), var("t"));
        let args = [("s", string_type()), ("t", string_type())];
        let prefix = all(8, string_flags(s.clone(), t.clone()));
        let sizes = equal(size(s.clone()), size(t.clone()));
        let body = bc("And", vec![prefix.clone(), sizes.clone()]);
        self.emit("Equal", &args, boolean(), body.clone(), false)?;
        self.theorem(
            "EqualStep",
            &args,
            Equality::boolean(
                string_equal(s.clone(), t.clone()),
                body.clone(),
                refl(boolean(), body.clone()),
            ),
        )?;
        self.emit(
            "NotEqual",
            &args,
            boolean(),
            bc("Not", vec![string_equal(s.clone(), t.clone())]),
            false,
        )?;
        self.theorem(
            "NotEqualStep",
            &args,
            Equality::boolean(
                not_equal(s.clone(), t.clone()),
                bc("Not", vec![string_equal(s.clone(), t.clone())]),
                refl(
                    boolean(),
                    bc("Not", vec![string_equal(s.clone(), t.clone())]),
                ),
            ),
        )?;
        let equality_truth = Equality::boolean(
            string_equal(s.clone(), t.clone()),
            body.clone(),
            ec("EqualStep", vec![s.clone(), t.clone()]),
        )
        .reverse()
        .then(Equality::boolean(
            string_equal(s.clone(), t.clone()),
            bit(true),
            var("h"),
        ))?;
        let full_length = ec(
            "AndRight",
            vec![prefix.clone(), sizes.clone(), equality_truth.proof.clone()],
        );
        let length = nc(
            "EqualReflect",
            vec![size(s.clone()), size(t.clone()), full_length],
        );
        let all_truth = ec(
            "AndLeft",
            vec![prefix.clone(), sizes.clone(), equality_truth.proof],
        );
        let addr = address_bits(&SLOTS);
        let flag = less(index(addr.clone()), size(s.clone()));
        let beq = byte_equal(slot(s.clone(), addr.clone()), slot(t.clone(), addr.clone()));
        let guarded = call(
            "Std.Bool.cases",
            vec![
                lam("_case", boolean(), boolean()),
                bit(true),
                beq.clone(),
                flag.clone(),
            ],
        );
        let observation = Equality::boolean(
            guarded.clone(),
            beq.clone(),
            sc(
                "ChoiceTrue",
                vec![
                    boolean(),
                    flag.clone(),
                    beq.clone(),
                    bit(true),
                    var("_inside"),
                ],
            ),
        )
        .reverse()
        .then(Equality::boolean(
            guarded,
            bit(true),
            app(
                ec(
                    "D8.Reflect",
                    vec![string_flags(s.clone(), t.clone()), all_truth],
                ),
                addr.clone(),
            ),
        ))?;
        let content_proof = cube(
            &SLOTS,
            lam(
                "_inside",
                inside(s.clone(), addr.clone()),
                ec(
                    "ByteReflect",
                    vec![
                        slot(s.clone(), addr.clone()),
                        slot(t.clone(), addr.clone()),
                        observation.proof,
                    ],
                ),
            ),
            true,
        );
        self.emit(
            "EqualReflect",
            &[
                args[0].clone(),
                args[1].clone(),
                ("h", truth(string_equal(s.clone(), t.clone()))),
            ],
            same(s.clone(), t.clone()),
            pc(
                "Make",
                vec![
                    eq(nat(), size(s.clone()), size(t.clone())),
                    content(s.clone(), t.clone()),
                    length,
                    content_proof,
                ],
            ),
            true,
        )?;
        let e = var("e");
        let length = nc(
            "EqualComplete",
            vec![
                size(s.clone()),
                size(t.clone()),
                same_length(s.clone(), t.clone(), e.clone()).proof,
            ],
        );
        let goal = |b: Expr| {
            truth(call(
                "Std.Bool.cases",
                vec![
                    lam("_case", boolean(), boolean()),
                    bit(true),
                    beq.clone(),
                    b,
                ],
            ))
        };
        let otherwise = lam(
            "_branch",
            eq(boolean(), flag.clone(), bit(false)),
            refl(boolean(), bit(true)),
        );
        let yes = lam(
            "_branch",
            truth(flag.clone()),
            ec(
                "ByteComplete",
                vec![
                    slot(s.clone(), addr.clone()),
                    slot(t.clone(), addr.clone()),
                    content_at(
                        s.clone(),
                        t.clone(),
                        e.clone(),
                        addr.clone(),
                        var("_branch"),
                    ),
                ],
            ),
        );
        let point = app(
            call(
                "Std.Bool.cases",
                vec![
                    lam(
                        "_flag",
                        boolean(),
                        pi(
                            "_branch",
                            eq(boolean(), flag.clone(), var("_flag")),
                            goal(var("_flag")),
                        ),
                    ),
                    otherwise,
                    yes,
                    flag.clone(),
                ],
            ),
            vec![refl(boolean(), flag)],
        );
        let all = ec(
            "D8.Complete",
            vec![
                string_flags(s.clone(), t.clone()),
                cube(&SLOTS, point, true),
            ],
        );
        self.theorem(
            "EqualComplete",
            &[
                args[0].clone(),
                args[1].clone(),
                ("e", same(s.clone(), t.clone())),
            ],
            Equality::boolean(
                string_equal(s.clone(), t.clone()),
                body,
                ec("EqualStep", vec![s.clone(), t.clone()]),
            )
            .then(Equality::boolean(
                bc("And", vec![prefix.clone(), sizes.clone()]),
                bit(true),
                ec(
                    "AndComplete",
                    vec![prefix.clone(), sizes.clone(), all, length],
                ),
            ))?,
        )?;
        let zero = bool_right(
            "And",
            prefix.clone(),
            Equality::boolean(sizes.clone(), bit(false), var("different")),
        )
        .then(Equality::boolean(
            bc("And", vec![prefix, bit(false)]),
            bit(false),
            refl(boolean(), bit(false)),
        ))?;
        self.theorem(
            "DifferentLength",
            &[
                args[0].clone(),
                args[1].clone(),
                ("different", eq(boolean(), sizes, bit(false))),
            ],
            Equality::boolean(
                string_equal(s.clone(), t.clone()),
                zero.left.clone(),
                ec("EqualStep", vec![s, t]),
            )
            .then(zero)?,
        )
    }
    fn behavior(&mut self) -> Result<()> {
        let (s, t, u) = (var("s"), var("t"), var("u"));
        let pair = [("s", string_type()), ("t", string_type())];
        self.theorem(
            "EqualRefl",
            &[pair[0].clone()],
            Equality::boolean(
                string_equal(s.clone(), s.clone()),
                bit(true),
                ec(
                    "EqualComplete",
                    vec![s.clone(), s.clone(), ec("EquivalentRefl", vec![s.clone()])],
                ),
            ),
        )?;
        let forward = lam(
            "_h",
            truth(string_equal(s.clone(), t.clone())),
            ec(
                "EqualComplete",
                vec![
                    t.clone(),
                    s.clone(),
                    ec(
                        "EquivalentSym",
                        vec![
                            s.clone(),
                            t.clone(),
                            ec("EqualReflect", vec![s.clone(), t.clone(), var("_h")]),
                        ],
                    ),
                ],
            ),
        );
        let backward = lam(
            "_h",
            truth(string_equal(t.clone(), s.clone())),
            ec(
                "EqualComplete",
                vec![
                    s.clone(),
                    t.clone(),
                    ec(
                        "EquivalentSym",
                        vec![
                            t.clone(),
                            s.clone(),
                            ec("EqualReflect", vec![t.clone(), s.clone(), var("_h")]),
                        ],
                    ),
                ],
            ),
        );
        self.theorem(
            "EqualSym",
            &pair,
            Equality::boolean(
                string_equal(s.clone(), t.clone()),
                string_equal(t.clone(), s.clone()),
                ec(
                    "TruthIff",
                    vec![
                        string_equal(s.clone(), t.clone()),
                        string_equal(t.clone(), s.clone()),
                        forward,
                        backward,
                    ],
                ),
            ),
        )?;
        self.theorem(
            "EqualTrans",
            &[
                pair[0].clone(),
                pair[1].clone(),
                ("u", string_type()),
                ("left", truth(string_equal(s.clone(), t.clone()))),
                ("right", truth(string_equal(t.clone(), u.clone()))),
            ],
            Equality::boolean(
                string_equal(s.clone(), u.clone()),
                bit(true),
                ec(
                    "EqualComplete",
                    vec![
                        s.clone(),
                        u.clone(),
                        ec(
                            "EquivalentTrans",
                            vec![
                                s.clone(),
                                t.clone(),
                                u,
                                ec("EqualReflect", vec![s.clone(), t.clone(), var("left")]),
                                ec("EqualReflect", vec![t.clone(), var("u"), var("right")]),
                            ],
                        ),
                    ],
                ),
            ),
        )?;
        let (s2, t2, ss, tt) = (var("s2"), var("t2"), var("ss"), var("tt"));
        let equivalent_back = ec(
            "EquivalentTrans",
            vec![
                s2.clone(),
                s.clone(),
                t.clone(),
                ec("EquivalentSym", vec![s.clone(), s2.clone(), ss.clone()]),
                ec("EqualReflect", vec![s.clone(), t.clone(), var("_h")]),
            ],
        );
        let equivalent_forward = ec(
            "EquivalentTrans",
            vec![
                s.clone(),
                s2.clone(),
                t2.clone(),
                ss.clone(),
                ec("EqualReflect", vec![s2.clone(), t2.clone(), var("_h")]),
            ],
        );
        let forward = lam(
            "_h",
            truth(string_equal(s.clone(), t.clone())),
            ec(
                "EqualComplete",
                vec![
                    s2.clone(),
                    t2.clone(),
                    ec(
                        "EquivalentTrans",
                        vec![
                            s2.clone(),
                            t.clone(),
                            t2.clone(),
                            equivalent_back,
                            tt.clone(),
                        ],
                    ),
                ],
            ),
        );
        let backward = lam(
            "_h",
            truth(string_equal(s2.clone(), t2.clone())),
            ec(
                "EqualComplete",
                vec![
                    s.clone(),
                    t.clone(),
                    ec(
                        "EquivalentTrans",
                        vec![
                            s.clone(),
                            t2.clone(),
                            t.clone(),
                            equivalent_forward,
                            ec("EquivalentSym", vec![t.clone(), t2.clone(), tt.clone()]),
                        ],
                    ),
                ],
            ),
        );
        let args = [
            pair[0].clone(),
            ("s2", string_type()),
            pair[1].clone(),
            ("t2", string_type()),
            ("ss", same(s.clone(), s2.clone())),
            ("tt", same(t.clone(), t2.clone())),
        ];
        self.theorem(
            "EqualRespects",
            &args,
            Equality::boolean(
                string_equal(s.clone(), t.clone()),
                string_equal(s2.clone(), t2.clone()),
                ec(
                    "TruthIff",
                    vec![
                        string_equal(s.clone(), t.clone()),
                        string_equal(s2.clone(), t2.clone()),
                        forward,
                        backward,
                    ],
                ),
            ),
        )?;
        let equality = Equality::boolean(
            string_equal(s.clone(), t.clone()),
            string_equal(s2.clone(), t2.clone()),
            ec(
                "EqualRespects",
                vec![s.clone(), s2.clone(), t.clone(), t2.clone(), ss, tt],
            ),
        );
        self.theorem(
            "NotEqualRespects",
            &args,
            equality.mapped(
                lam("_flag", boolean(), bc("Not", vec![var("_flag")])),
                not_equal(s.clone(), t.clone()),
                not_equal(s2, t2),
                boolean(),
            ),
        )?;
        self.emit(
            "NormalizationEquivalent",
            &[pair[0].clone()],
            same(norm(s.clone()), s.clone()),
            ec(
                "FromStrongEquivalent",
                vec![
                    norm(s.clone()),
                    s.clone(),
                    call(
                        "Mpk.Go.Practical.String.Normalization.ResultEquivalent",
                        vec![s.clone()],
                    ),
                ],
            ),
            true,
        )?;
        self.theorem(
            "EqualNormalized",
            &pair,
            Equality::boolean(
                string_equal(norm(s.clone()), norm(t.clone())),
                string_equal(s.clone(), t.clone()),
                ec(
                    "EqualRespects",
                    vec![
                        norm(s.clone()),
                        s.clone(),
                        norm(t.clone()),
                        t.clone(),
                        ec("NormalizationEquivalent", vec![s.clone()]),
                        ec("NormalizationEquivalent", vec![t.clone()]),
                    ],
                ),
            ),
        )?;
        self.theorem(
            "NotEqualNormalized",
            &pair,
            Equality::boolean(
                not_equal(norm(s.clone()), norm(t.clone())),
                not_equal(s.clone(), t.clone()),
                ec(
                    "NotEqualRespects",
                    vec![
                        norm(s.clone()),
                        s.clone(),
                        norm(t.clone()),
                        t.clone(),
                        ec("NormalizationEquivalent", vec![s.clone()]),
                        ec("NormalizationEquivalent", vec![t.clone()]),
                    ],
                ),
            ),
        )
    }
}
pub(super) fn append(builder: &mut Builder) -> Result<()> {
    let mut e = EqualityProofs(builder);
    e.boolean()?;
    for d in 0..=8 {
        e.fold(d)?;
    }
    e.bytes()?;
    e.equivalence()?;
    e.string_operations()?;
    e.behavior()
}
