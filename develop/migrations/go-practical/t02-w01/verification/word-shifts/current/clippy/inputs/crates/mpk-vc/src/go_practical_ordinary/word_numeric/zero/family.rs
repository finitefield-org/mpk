//! Actual Word.IsZero/DivideDefined bind directly to the balanced zero proof.
use super::*;
fn literal_equal(w: usize, d: usize, offset: usize, x: Expr) -> Result<Expr> {
    if d == 0 {
        return Ok(bc(
            "Equal",
            vec![at(x, w, offset)?, at(wc(w, "Zero", vec![]), w, offset)?],
        ));
    }
    Ok(bc(
        "And",
        vec![
            literal_equal(w, d - 1, offset, x.clone())?,
            literal_equal(w, d - 1, offset + (1 << (d - 1)), x)?,
        ],
    ))
}
fn literal_fold(w: usize, d: usize, offset: usize, x: Expr) -> Result<Equality> {
    if d == 0 {
        let (a, z) = (at(x, w, offset)?, at(wc(w, "Zero", vec![]), w, offset)?);
        let e = bool_right(
            "Equal",
            a.clone(),
            Equality::boolean(z, bit(false), refl(boolean(), bit(false))),
        )
        .then(Equality::boolean(
            bc("Equal", vec![a.clone(), bit(false)]),
            bc("Not", vec![a.clone()]),
            refl(boolean(), bc("Not", vec![a])),
        ))?;
        return Ok(e);
    }
    let h = d - 1;
    let l = literal_fold(w, h, offset, x.clone())?;
    let r = literal_fold(w, h, offset + (1 << h), x)?;
    bool_left("And", l.clone(), r.left.clone()).then(bool_right("And", l.right, r))
}
impl Zero<'_> {
    pub(super) fn family(&mut self, w: usize) -> Result<()> {
        let d = depth(w)?;
        let x = var("x");
        let args = [("x", word_type(w)?)];
        let actual = wc(w, "IsZero", vec![x.clone()]);
        let tree = literal_equal(w, d, 0, x.clone())?;
        let start = Equality::boolean(actual.clone(), tree.clone(), refl(boolean(), tree));
        let e = start.then(literal_fold(w, d, 0, x.clone())?)?;
        let e = e.then(
            Equality::boolean(
                bits(d, x.clone()),
                literal(d, x.clone()),
                zc(&format!("ZeroBitsLiteral{d}"), vec![x.clone()]),
            )
            .reverse(),
        )?;
        self.theorem(&format!("W{w}.IsZeroBits"), &args, e.clone())?;
        let v = value(d, x.clone());
        let natural = e.clone().then(Equality::boolean(
            bits(d, x.clone()),
            equal(v.clone(), zero()),
            zc(&format!("ZeroBitsNatural{d}"), vec![x.clone()]),
        ))?;
        self.theorem(&format!("W{w}.IsZeroNatural"), &args, natural.clone())?;
        let positive = bool_map("Not", natural).then(Equality::boolean(
            bc("Not", vec![equal(v.clone(), zero())]),
            less(zero(), v.clone()),
            zc("NaturalNonzeroPositive", vec![v.clone()]),
        ))?;
        let defined = wc(w, "DivideDefined", vec![x.clone()]);
        let positive = Equality {
            left: defined.clone(),
            ..positive
        };
        self.theorem(
            &format!("W{w}.DivideDefinedNatural"),
            &args,
            positive.clone(),
        )?;
        self.theorem(
            &format!("W{w}.DefinedDivisorPositive"),
            &[
                ("x", word_type(w)?),
                ("defined", eq(boolean(), defined.clone(), bit(true))),
            ],
            positive
                .reverse()
                .then(Equality::boolean(defined, bit(true), var("defined")))?,
        )?;
        let zero = e
            .reverse()
            .then(Equality::boolean(actual.clone(), bit(true), var("zero")))?;
        self.theorem(
            &format!("W{w}.ZeroSign"),
            &[
                ("x", word_type(w)?),
                ("zero", eq(boolean(), actual, bit(true))),
            ],
            Equality::boolean(
                top(d, x.clone()),
                bit(false),
                zc(&format!("ZeroBitsSign{d}"), vec![x, zero.proof]),
            ),
        )
    }
}
