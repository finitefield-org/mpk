//! Share the universal arithmetic proof across all concrete conversion views.
use super::*;
impl Conversions<'_> {
    pub(super) fn scalar_laws(&mut self) -> Result<()> {
        self.narrow_cross()?;
        self.widen_cross()?;
        self.scalar_correction()
    }
    fn narrow_cross(&mut self) -> Result<()> {
        let (a, b, ms, mt, q, r, sx, sy) = (
            var("a"),
            var("b"),
            var("source_modulus"),
            var("target_modulus"),
            var("quotient"),
            var("ratio"),
            var("source_bit"),
            var("target_bit"),
        );
        let mut args = [
            "a",
            "b",
            "source_modulus",
            "target_modulus",
            "quotient",
            "ratio",
            "source_bit",
            "target_bit",
        ]
        .map(|n| (n, nat()))
        .to_vec();
        args.push((
            "division",
            eq(nat(), a.clone(), add(b.clone(), mul(mt.clone(), q.clone()))),
        ));
        args.push(("radix", eq(nat(), mul(mt.clone(), r.clone()), ms.clone())));
        let e = Equality::natural(
            a.clone(),
            add(b.clone(), mul(mt.clone(), q.clone())),
            var("division"),
        )
        .on_left(add(
            mul(mt.clone(), sy.clone()),
            mul(mt.clone(), mul(r.clone(), sx.clone())),
        ));
        let prefix = add(b, mul(mt.clone(), add(q, sy)));
        let p = self.polynomial(
            &args,
            e.right.clone(),
            add(prefix.clone(), mul(mul(mt.clone(), r.clone()), sx.clone())),
        )?;
        let e = e
            .then(p)?
            .then(mul_left(Equality::natural(mul(mt, r), ms, var("radix")), sx).on_right(prefix))?;
        self.theorem("NarrowCross", &args, e)
    }
    fn widen_cross(&mut self) -> Result<()> {
        let (a, b, ms, mt, m, sx, sy) = (
            var("a"),
            var("b"),
            var("source_modulus"),
            var("target_modulus"),
            var("mask"),
            var("source_bit"),
            var("target_bit"),
        );
        let mut args = [
            "a",
            "b",
            "source_modulus",
            "target_modulus",
            "mask",
            "source_bit",
            "target_bit",
        ]
        .map(|n| (n, nat()))
        .to_vec();
        args.push((
            "extension",
            eq(nat(), b.clone(), add(a.clone(), mul(m.clone(), sx.clone()))),
        ));
        args.push(("radix", eq(nat(), add(ms.clone(), m.clone()), mt.clone())));
        let e = Equality::natural(
            b,
            add(a.clone(), mul(m.clone(), sx.clone())),
            var("extension"),
        )
        .on_left(mul(mt.clone(), sy.clone()))
        .on_left(mul(ms.clone(), sx.clone()));
        let p = self.polynomial(
            &args,
            e.right.clone(),
            add(
                a.clone(),
                add(
                    mul(mt.clone(), sy.clone()),
                    mul(add(ms.clone(), m.clone()), sx.clone()),
                ),
            ),
        )?;
        let e = e.then(p)?.then(
            mul_left(Equality::natural(add(ms, m), mt.clone(), var("radix")), sx)
                .on_right(mul(mt, sy))
                .on_right(a),
        )?;
        self.theorem("WidenCross", &args, e.reverse())
    }
    fn scalar_correction(&mut self) -> Result<()> {
        let (a, b, ns, nt, m, cp, cn) = (
            var("a"),
            var("b"),
            var("source_negative"),
            var("target_negative"),
            var("modulus"),
            var("correction_positive"),
            var("correction_negative"),
        );
        let left = int("Make", vec![a.clone(), ns.clone()]);
        let right = int(
            "Add",
            vec![
                int("Make", vec![b.clone(), nt.clone()]),
                int(
                    "Mul",
                    vec![
                        int("FromNat", vec![m.clone()]),
                        int("Make", vec![cp.clone(), cn.clone()]),
                    ],
                ),
            ],
        );
        let (lp, rp) = integer::cross(left.clone(), right.clone())?;
        let lc = add(a, add(nt, mul(m.clone(), cn)));
        let rc = add(add(b, mul(m, cp)), ns);
        let mut args = [
            "a",
            "b",
            "source_negative",
            "target_negative",
            "modulus",
            "correction_positive",
            "correction_negative",
        ]
        .map(|n| (n, nat()))
        .to_vec();
        args.push(("balance", eq(nat(), lc.clone(), rc.clone())));
        let l = self.polynomial(&args, lp.right.clone(), lc.clone())?;
        let r = self.polynomial(&args, rp.right.clone(), rc.clone())?;
        self.relation(
            "ScalarCorrection",
            &args,
            left,
            right,
            l.then(Equality::natural(lc, rc, var("balance")))?
                .then(r.reverse())?,
        )
    }
}
