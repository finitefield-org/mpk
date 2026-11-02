//! Restoration and overwrite prove actual full-index slot cases.
use super::*;

fn set_slot(d: usize, a: Expr, s: Expr, i: Expr, x: Expr, yes: bool) -> Equality {
    let at = address(d);
    let mut args = vec![a.clone(), s.clone(), i.clone(), x.clone()];
    args.extend(at.clone()); args.push(var("_case"));
    Equality {
        ty: a.clone(), left: app(set(d, a, s.clone(), i, x.clone()), at.clone()),
        right: if yes { x } else { app(s, at) },
        proof: ac(d, if yes { "SetSameSlot" } else { "SetOtherSlot" }, args),
    }
}
impl Lenses<'_> {
    pub(super) fn array_restoration(&mut self, d: usize) -> Result<()> {
        let (a, i) = (var("A"), var("i"));
        let root_ty = cube(d, a.clone(), false);
        let at = address(d);
        let (s, x, y) = (var("_s"), var("_x"), var("_y"));
        let args = [("A", Expr::Sort0), ("s", root_ty.clone()), ("i", word())];
        let lookup = fc(d, "Lookup", vec![a.clone(), var("s"), value(i.clone())]);
        let body = lam("_word", word(), fc(d, "Lookup", vec![a.clone(), var("s"), value(var("_word"))]));
        let e = Equality {
            ty: function(word(), a.clone()),
            left: call(&format!("Mpk.Go.Practical.Word.Index.D{d}.Read64"), vec![a.clone(), var("s")]),
            right: body,
            proof: call(&format!("Mpk.Go.Practical.Word.Index.D{d}.Read64Definition"), vec![a.clone(), var("s")]),
        }.mapped(lam("_reader", function(word(), a.clone()), app(var("_reader"), vec![i.clone()])),
            read(d, a.clone(), var("s"), i.clone()), lookup, a.clone());
        self.theorem(&name(d, "ReadNatural"), &args, e)?;
        let flag = equal(fc(d, "Index", at.clone()), value(i.clone()));
        let mut args = args.to_vec(); args.extend(BITS[..d].iter().map(|b| (*b, boolean())));
        args.push(("same", eq(boolean(), flag.clone(), bit(true))));
        let index_eq = Equality::natural(fc(d, "Index", at.clone()), value(i.clone()),
            nc("EqualReflect", vec![fc(d, "Index", at.clone()), value(i.clone()), var("same")])).reverse();
        let eq_lookup = index_eq.mapped(lam("_index", nat(), fc(d, "Lookup", vec![a.clone(), var("s"), var("_index")])),
            fc(d, "Lookup", vec![a.clone(), var("s"), value(i.clone())]),
            fc(d, "Lookup", vec![a.clone(), var("s"), fc(d, "Index", at.clone())]), a.clone());
        let mut lookup_args = vec![a.clone(), var("s")]; lookup_args.extend(at.clone());
        let e = Equality {
            ty: a.clone(), left: read(d, a.clone(), var("s"), i.clone()),
            right: fc(d, "Lookup", vec![a.clone(), var("s"), value(i.clone())]),
            proof: lc(&name(d, "ReadNatural"), vec![a.clone(), var("s"), i.clone()]),
        }.then(eq_lookup)?.then(Equality {
            ty: a.clone(), left: fc(d, "Lookup", vec![a.clone(), var("s"), fc(d, "Index", at.clone())]),
            right: app(var("s"), at.clone()), proof: fc(d, "LookupIndex", lookup_args),
        })?;
        self.theorem(&name(d, "ReadMatchedSlot"), &args, e)?;
        let (r, n) = (var("R"), var("n"));
        let args = [("A", Expr::Sort0), ("R", relation(a.clone())), ("n", nat()), ("i", word()), ("refl", reflexive(a.clone(), r.clone()))];
        let view = element(d, a.clone(), i.clone());
        let root_rel = root_relation(d, a.clone(), r.clone(), n.clone());
        let current = read(d, a.clone(), s.clone(), i.clone());
        let old_slot = app(s.clone(), at.clone());
        let mut matched = vec![a.clone(), s.clone(), i.clone()]; matched.extend(at.clone()); matched.push(var("_case"));
        let yes = set_slot(d, a.clone(), s.clone(), i.clone(), current.clone(), true).then(Equality {
            ty: a.clone(), left: current.clone(), right: old_slot.clone(), proof: lc(&name(d, "ReadMatchedSlot"), matched),
        })?;
        let unchanged = Equality { ty: a.clone(), left: old_slot.clone(), right: old_slot.clone(), proof: refl(a.clone(), old_slot.clone()) };
        let restored = set(d, a.clone(), s.clone(), i.clone(), current.clone());
        let proof = |e| transfer(a.clone(), r.clone(), e, unchanged.clone(), app(var("refl"), vec![old_slot.clone()]));
        let body = points(d, n.clone(), decide(flag.clone(), holds(r.clone(), app(restored, at.clone()), old_slot.clone()),
            proof(set_slot(d, a.clone(), s.clone(), i.clone(), current, false)), proof(yes)));
        self.emit(&name(d, "ElementPutGet"), &args, put_get(root_ty.clone(), a.clone(), root_rel.clone(), view.clone()), lam("_s", root_ty.clone(), body), true)?;
        let once_x = set(d, a.clone(), s.clone(), i.clone(), x.clone());
        let twice = set(d, a.clone(), once_x.clone(), i.clone(), y.clone());
        let once_y = set(d, a.clone(), s.clone(), i.clone(), y.clone());
        let branch = |yes| -> Result<Expr> {
            let mut left = set_slot(d, a.clone(), once_x.clone(), i.clone(), y.clone(), yes);
            if !yes { left = left.then(set_slot(d, a.clone(), s.clone(), i.clone(), x.clone(), false))?; }
            let right = set_slot(d, a.clone(), s.clone(), i.clone(), y.clone(), yes);
            let at_value = if yes { y.clone() } else { old_slot.clone() };
            Ok(transfer(a.clone(), r.clone(), left, right, app(var("refl"), vec![at_value])))
        };
        let body = points(d, n, decide(flag, holds(r.clone(), app(twice, at.clone()), app(once_y, at)), branch(false)?, branch(true)?));
        self.emit(&name(d, "ElementPutPut"), &args, put_put(root_ty.clone(), a.clone(), root_rel, view),
            lam("_s", root_ty, lam("_x", a.clone(), lam("_y", a, body))), true)
    }
}
