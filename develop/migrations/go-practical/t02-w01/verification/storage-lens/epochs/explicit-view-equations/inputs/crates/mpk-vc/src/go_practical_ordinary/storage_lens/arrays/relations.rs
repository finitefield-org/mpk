//! Root relations are pointwise element proofs, never equality of functions.
use super::*;

impl Lenses<'_> {
    pub(super) fn array_relations(&mut self, d: usize) -> Result<()> {
        let (a, r, n, i) = (var("A"), var("R"), var("n"), var("i"));
        let root_ty = cube(d, a.clone(), false);
        let base = [("A", Expr::Sort0), ("R", relation(a.clone())), ("n", nat())];
        let at = address(d);
        let observation = |s: Expr, t: Expr, h: Expr| {
            let mut args = vec![a.clone(), a.clone(), r.clone(), n.clone(), s, t, h];
            args.extend(at.clone());
            args.push(var("_in"));
            ac(d, "RelatedSlot", args)
        };
        let args = [
            base.to_vec(),
            vec![
                ("refl", reflexive(a.clone(), r.clone())),
                ("s", root_ty.clone()),
            ],
        ]
        .concat();
        self.emit(
            &name(d, "RelationReflexive"),
            &args,
            related(d, a.clone(), r.clone(), n.clone(), var("s"), var("s")),
            points(
                d,
                n.clone(),
                app(var("refl"), vec![app(var("s"), at.clone())]),
            ),
            true,
        )?;
        let symmetric = pi(
            "_x",
            a.clone(),
            pi(
                "_y",
                a.clone(),
                pi(
                    "_xy",
                    holds(r.clone(), var("_x"), var("_y")),
                    holds(r.clone(), var("_y"), var("_x")),
                ),
            ),
        );
        let args = [
            base.to_vec(),
            vec![
                ("sym", symmetric),
                ("s", root_ty.clone()),
                ("t", root_ty.clone()),
                (
                    "h",
                    related(d, a.clone(), r.clone(), n.clone(), var("s"), var("t")),
                ),
            ],
        ]
        .concat();
        self.emit(
            &name(d, "RelationSymmetric"),
            &args,
            related(d, a.clone(), r.clone(), n.clone(), var("t"), var("s")),
            points(
                d,
                n.clone(),
                app(
                    var("sym"),
                    vec![
                        app(var("s"), at.clone()),
                        app(var("t"), at.clone()),
                        observation(var("s"), var("t"), var("h")),
                    ],
                ),
            ),
            true,
        )?;
        let args = [
            base.to_vec(),
            vec![
                ("trans", transitive(a.clone(), r.clone())),
                ("s", root_ty.clone()),
                ("t", root_ty.clone()),
                ("u", root_ty.clone()),
                (
                    "st",
                    related(d, a.clone(), r.clone(), n.clone(), var("s"), var("t")),
                ),
                (
                    "tu",
                    related(d, a.clone(), r.clone(), n.clone(), var("t"), var("u")),
                ),
            ],
        ]
        .concat();
        self.emit(
            &name(d, "RelationTransitive"),
            &args,
            related(d, a.clone(), r.clone(), n.clone(), var("s"), var("u")),
            points(
                d,
                n.clone(),
                app(
                    var("trans"),
                    vec![
                        app(var("s"), at.clone()),
                        app(var("t"), at.clone()),
                        app(var("u"), at.clone()),
                        observation(var("s"), var("t"), var("st")),
                        observation(var("t"), var("u"), var("tu")),
                    ],
                ),
            ),
            true,
        )?;
        let view = element(d, a.clone(), i.clone());
        let root_rel = root_relation(d, a.clone(), r.clone(), n.clone());
        let args = [
            base.to_vec(),
            vec![
                ("i", word()),
                ("shape", ac(d, "Shape", vec![n.clone()])),
                ("within", inside(value(i.clone()), n.clone())),
            ],
        ]
        .concat();
        let proof=ac(d,"RelatedRead",vec![a.clone(),a.clone(),r.clone(),n.clone(),var("_s"),var("_t"),var("_h"),i.clone(),var("shape"),var("within")]);
        let proof=transfer(a.clone(),r.clone(),get_equation(d,a.clone(),i.clone(),var("_s")),get_equation(d,a.clone(),i.clone(),var("_t")),proof);
        self.emit(&name(d,"ElementGetRespect"),&args,get_respect(root_ty.clone(),a.clone(),root_rel.clone(),r.clone(),view.clone()),
            lam("_s",root_ty.clone(),lam("_t",root_ty.clone(),lam("_h",related(d,a.clone(),r.clone(),n.clone(),var("_s"),var("_t")),proof))),true)?;
        let args=[base.to_vec(),vec![("i",word()),("refl",reflexive(a.clone(),r.clone()))]].concat();
        let proof=ac(d,"RelatedSet",vec![a.clone(),a.clone(),r.clone(),n.clone(),var("_s"),var("_s"),
            lc(&name(d,"RelationReflexive"),vec![a.clone(),r.clone(),n,var("refl"),var("_s")]),i.clone(),var("_x"),var("_y"),var("_h")]);
        let proof=transfer(root_ty.clone(),root_rel.clone(),put_equation(d,a.clone(),i.clone(),var("_s"),var("_x")),put_equation(d,a.clone(),i,var("_s"),var("_y")),proof);
        self.emit(&name(d,"ElementPutRespect"),&args,put_respect(root_ty.clone(),a.clone(),root_rel,r.clone(),view),
            lam("_s",root_ty,lam("_x",a.clone(),lam("_y",a.clone(),lam("_h",holds(r,var("_x"),var("_y")),proof)))),true)
    }
}
