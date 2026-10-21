//! Full-width substring arithmetic and the frozen complete output carrier.
//! Signed guards precede access; bytes outside the result are canonical zero.
use super::*;

fn sl(name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("Mpk.Go.Practical.String.Slice.{name}"), args)
}
fn ord(name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("Mpk.Go.Practical.Word.Order.{name}"), args)
}
fn wc(name: &str, args: Vec<Expr>) -> Expr {
    word::wc(64, name, args)
}
fn modulus() -> Expr {
    numeric("Weight6", vec![])
}
fn truth(flag: Expr, yes: bool) -> Expr {
    eq(boolean(), flag, bit(yes))
}
fn sign(x: Expr) -> Result<Expr> {
    word::at(x, 64, 63)
}
fn nl(e: Equality, b: Expr) -> Equality {
    let (l, r) = (
        less(e.left.clone(), b.clone()),
        less(e.right.clone(), b.clone()),
    );
    e.mapped(lam("_n", nat(), less(var("_n"), b)), l, r, boolean())
}
fn nr(a: Expr, e: Equality) -> Equality {
    let (l, r) = (
        less(a.clone(), e.left.clone()),
        less(a.clone(), e.right.clone()),
    );
    e.mapped(lam("_n", nat(), less(a, var("_n"))), l, r, boolean())
}
fn not(e: Equality) -> Equality {
    let (l, r) = (
        bc("Not", vec![e.left.clone()]),
        bc("Not", vec![e.right.clone()]),
    );
    e.mapped(
        lam("_b", boolean(), bc("Not", vec![var("_b")])),
        l,
        r,
        boolean(),
    )
}
fn bool_left(name: &str, e: Equality, b: Expr) -> Equality {
    let (l, r) = (
        bc(name, vec![e.left.clone(), b.clone()]),
        bc(name, vec![e.right.clone(), b.clone()]),
    );
    e.mapped(
        lam("_b", boolean(), bc(name, vec![var("_b"), b])),
        l,
        r,
        boolean(),
    )
}
fn bool_right(name: &str, a: Expr, e: Equality) -> Equality {
    let (l, r) = (
        bc(name, vec![a.clone(), e.left.clone()]),
        bc(name, vec![a.clone(), e.right.clone()]),
    );
    e.mapped(
        lam("_b", boolean(), bc(name, vec![a, var("_b")])),
        l,
        r,
        boolean(),
    )
}
fn bit_value(e: Equality) -> Equality {
    let (l, r) = (
        numeric("Bit", vec![e.left.clone()]),
        numeric("Bit", vec![e.right.clone()]),
    );
    e.mapped(
        lam("_b", boolean(), numeric("Bit", vec![var("_b")])),
        l,
        r,
        nat(),
    )
}
fn decide(flag: Expr, target: Expr, no: Expr, yes: Expr) -> Expr {
    app(
        call(
            "Std.Bool.cases",
            vec![
                lam(
                    "_tag",
                    boolean(),
                    pi("_branch", eq(boolean(), flag.clone(), var("_tag")), target),
                ),
                lam("_branch", truth(flag.clone(), false), no),
                lam("_branch", truth(flag.clone(), true), yes),
                flag.clone(),
            ],
        ),
        vec![refl(boolean(), flag)],
    )
}

struct Slice<'a>(&'a mut Builder);
impl Slice<'_> {
    fn emit(
        &mut self,
        name: &str,
        args: &[Binder],
        ty: Expr,
        body: Expr,
        proof: bool,
    ) -> Result<()> {
        emit(
            self.0,
            &format!("Mpk.Go.Practical.String.Slice.{name}"),
            args,
            ty,
            body,
            proof,
        )
    }
    fn theorem(&mut self, name: &str, args: &[Binder], e: Equality) -> Result<()> {
        self.emit(name, args, eq(e.ty, e.left, e.right), e.proof, true)
    }
    fn sub_exact(&mut self) -> Result<()> {
        let (x, y) = (var("x"), var("y"));
        let a = value(x.clone());
        let b = value(y.clone());
        let r = value(wc("Sub", vec![x.clone(), y.clone()]));
        let c = wc(
            "Carry64",
            vec![x.clone(), wc("Not", vec![y.clone()]), bit(true)],
        );
        let carry = Equality::boolean(
            bc("Not", vec![c.clone()]),
            less(a.clone(), b.clone()),
            ord("W64.UnsignedLessNatural", vec![x.clone(), y.clone()]),
        )
        .then(Equality::boolean(
            less(a.clone(), b.clone()),
            bit(false),
            var("ordered"),
        ))?;
        let carry = Equality::boolean(
            bc("Not", vec![bc("Not", vec![c.clone()])]),
            c.clone(),
            bc("DoubleNot", vec![c.clone()]),
        )
        .reverse()
        .then(not(carry))?
        .then(Equality::boolean(
            bc("Not", vec![bit(false)]),
            bit(true),
            refl(boolean(), bit(true)),
        ))?;
        let unit = bit_value(carry).then(Equality::natural(
            numeric("Bit", vec![bit(true)]),
            one(),
            refl(nat(), one()),
        ))?;
        let weighted =
            super::super::polynomial::mul_right(modulus(), unit).then(Equality::natural(
                mul(modulus(), one()),
                modulus(),
                nc("MulOne", vec![modulus()]),
            ))?;
        let sum = add(r.clone(), b.clone());
        let conserve = Equality::natural(
            add(a.clone(), modulus()),
            add(sum.clone(), mul(modulus(), numeric("Bit", vec![c]))),
            ord("W64.SubtractionConservation", vec![x.clone(), y.clone()]),
        )
        .then(weighted.on_right(sum.clone()))?;
        let cancellation = Equality::natural(
            a.clone(),
            sum.clone(),
            nc(
                "AddCancel",
                vec![a.clone(), sum.clone(), modulus(), conserve.proof],
            ),
        );
        let e = cancellation
            .mapped(
                lam("_n", nat(), sub(var("_n"), b.clone())),
                sub(a.clone(), b.clone()),
                sub(sum, b.clone()),
                nat(),
            )
            .then(Equality::natural(
                sub(add(r.clone(), b.clone()), b.clone()),
                r.clone(),
                nc("SubAddRight", vec![r, b.clone()]),
            ))?
            .reverse();
        self.theorem(
            "SubValue",
            &[
                ("x", word64()),
                ("y", word64()),
                ("ordered", truth(less(a, b), false)),
            ],
            e,
        )
    }
}
pub(super) fn append(builder: &mut Builder) -> Result<()> {
    Slice(builder).sub_exact()
}
