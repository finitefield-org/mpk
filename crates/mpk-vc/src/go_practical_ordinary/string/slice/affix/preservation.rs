//! Logical content equivalence preserves actual whole-string affix predicates.
use super::matching::{
    at_bit, byte_value, content, matches, relation, relation_bound, relation_content,
};
use super::*;
fn same(s: Expr, t: Expr) -> Expr {
    ec("Equivalent", vec![s, t])
}
fn sizes(s: Expr, t: Expr, e: Expr) -> Equality {
    Equality::natural(
        size(s.clone()),
        size(t.clone()),
        ec("EquivalentLength", vec![s, t, e]),
    )
}
fn norm(s: Expr) -> Expr {
    call("Mpk.Go.Practical.String.Normalization.Value", vec![s])
}
impl Affix<'_> {
    pub(super) fn preservation(&mut self) -> Result<()> {
        self.equal_lengths()?;
        self.relation_transfer()?;
        self.match_transfer()
    }
    fn equal_lengths(&mut self) -> Result<()> {
        let (s, u, p, q, k) = (var("s"), var("u"), var("p"), var("q"), var("k"));
        let args = [
            ("s", string_type()),
            ("u", string_type()),
            ("p", string_type()),
            ("q", string_type()),
            ("k", boolean()),
            ("se", same(s.clone(), u.clone())),
            ("pe", same(p.clone(), q.clone())),
        ];
        let es = sizes(s.clone(), u.clone(), var("se"));
        let ep = sizes(p.clone(), q.clone(), var("pe"));
        self.theorem(
            "LengthTestRespects",
            &args,
            nl(es.clone(), size(p.clone())).then(nr(size(u.clone()), ep.clone()))?,
        )?;
        let mut bound_args = args.to_vec();
        bound_args.push((
            "bound",
            truth(less(size(s.clone()), size(p.clone())), false),
        ));
        self.theorem(
            "BoundTransfer",
            &bound_args,
            Equality::boolean(
                less(size(s.clone()), size(p.clone())),
                less(size(u.clone()), size(q.clone())),
                af(
                    "LengthTestRespects",
                    vec![
                        s.clone(),
                        u.clone(),
                        p.clone(),
                        q.clone(),
                        k.clone(),
                        var("se"),
                        var("pe"),
                    ],
                ),
            )
            .reverse()
            .then(Equality::boolean(
                less(size(s.clone()), size(p.clone())),
                bit(false),
                var("bound"),
            ))?,
        )?;
        let diff = es
            .clone()
            .mapped(
                lam("_n", nat(), sub(var("_n"), size(p.clone()))),
                sub(size(s.clone()), size(p.clone())),
                sub(size(u.clone()), size(p.clone())),
                nat(),
            )
            .then(ep.mapped(
                lam("_n", nat(), sub(size(u.clone()), var("_n"))),
                sub(size(u.clone()), size(p.clone())),
                sub(size(u.clone()), size(q.clone())),
                nat(),
            ))?;
        let from = word::choose(
            k.clone(),
            sub(size(s.clone()), size(p.clone())),
            zero(),
            nat(),
        );
        let to = word::choose(
            k.clone(),
            sub(size(u.clone()), size(q.clone())),
            zero(),
            nat(),
        );
        self.theorem(
            "OffsetNatRespects",
            &args,
            Equality::natural(
                offset_nat(s.clone(), p.clone(), k.clone()),
                from.clone(),
                af("OffsetNatStep", vec![s.clone(), p.clone(), k.clone()]),
            )
            .then(diff.mapped(
                lam(
                    "_n",
                    nat(),
                    word::choose(k.clone(), var("_n"), zero(), nat()),
                ),
                from,
                to.clone(),
                nat(),
            ))?
            .then(
                Equality::natural(
                    offset_nat(u.clone(), q.clone(), k.clone()),
                    to,
                    af("OffsetNatStep", vec![u.clone(), q.clone(), k.clone()]),
                )
                .reverse(),
            )?,
        )?;
        let args = [
            ("s", string_type()),
            ("u", string_type()),
            ("e", same(s.clone(), u.clone())),
            ("upper", upper(s.clone())),
        ];
        self.theorem(
            "UpperTransfer",
            &args,
            nr(capacity(), sizes(s.clone(), u.clone(), var("e")).reverse()).then(
                Equality::boolean(less(capacity(), size(s)), bit(false), var("upper")),
            )?,
        )
    }
    fn relation_transfer(&mut self) -> Result<()> {
        let (s, u, p, q, k) = (var("s"), var("u"), var("p"), var("q"), var("k"));
        let args = [
            ("s", string_type()),
            ("u", string_type()),
            ("p", string_type()),
            ("q", string_type()),
            ("k", boolean()),
            ("se", same(s.clone(), u.clone())),
            ("pe", same(p.clone(), q.clone())),
            ("sup", upper(s.clone())),
            ("pup", upper(p.clone())),
            ("r", relation(s.clone(), p.clone(), k.clone())),
        ];
        let old_bound = relation_bound(s.clone(), p.clone(), k.clone(), var("r"));
        let new_bound = af(
            "BoundTransfer",
            vec![
                s.clone(),
                u.clone(),
                p.clone(),
                q.clone(),
                k.clone(),
                var("se"),
                var("pe"),
                old_bound.clone(),
            ],
        );
        let i = var("_i");
        let bits = address_bits(&BITS[..3]);
        let old_inside = nr(value(i.clone()), sizes(p.clone(), q.clone(), var("pe"))).then(
            Equality::boolean(
                less(value(i.clone()), size(q.clone())),
                bit(true),
                var("_within"),
            ),
        )?;
        let a = source(s.clone(), p.clone(), k.clone(), i.clone());
        let b = source(u.clone(), q.clone(), k.clone(), i.clone());
        let actual = vec![
            s.clone(),
            u.clone(),
            p.clone(),
            q.clone(),
            k.clone(),
            var("se"),
            var("pe"),
        ];
        let offset = Equality::natural(
            offset_nat(s.clone(), p.clone(), k.clone()),
            offset_nat(u.clone(), q.clone(), k.clone()),
            af("OffsetNatRespects", actual.clone()),
        );
        let values = index_value(
            s.clone(),
            p.clone(),
            k.clone(),
            old_bound.clone(),
            i.clone(),
            old_inside.proof.clone(),
        )
        .then(offset.on_left(value(i.clone())))?
        .then(
            index_value(
                u.clone(),
                q.clone(),
                k.clone(),
                new_bound.clone(),
                i.clone(),
                var("_within"),
            )
            .reverse(),
        )?;
        let source_bits = Equality::boolean(
            app(observed(s.clone(), a.clone()), bits.clone()),
            app(observed(u.clone(), a.clone()), bits.clone()),
            app(
                sl(
                    "ByteSameAt",
                    vec![
                        s.clone(),
                        u.clone(),
                        var("se"),
                        a.clone(),
                        var("sup"),
                        af(
                            "IndexWithin",
                            vec![
                                s.clone(),
                                p.clone(),
                                k.clone(),
                                old_bound,
                                i.clone(),
                                old_inside.proof.clone(),
                            ],
                        ),
                    ],
                ),
                bits.clone(),
            ),
        )
        .then(at_bit(byte_value(u.clone(), a, b, values), bits.clone()))?;
        let pattern_bits = Equality::boolean(
            app(observed(p.clone(), i.clone()), bits.clone()),
            app(observed(q.clone(), i.clone()), bits.clone()),
            app(
                sl(
                    "ByteSameAt",
                    vec![
                        p.clone(),
                        q.clone(),
                        var("pe"),
                        i.clone(),
                        var("pup"),
                        old_inside.proof.clone(),
                    ],
                ),
                bits.clone(),
            ),
        );
        let original = Equality::boolean(
            app(
                observed(
                    s.clone(),
                    source(s.clone(), p.clone(), k.clone(), i.clone()),
                ),
                bits.clone(),
            ),
            app(observed(p.clone(), i.clone()), bits.clone()),
            app(
                relation_content(
                    s.clone(),
                    p.clone(),
                    k.clone(),
                    var("r"),
                    i.clone(),
                    old_inside.proof,
                ),
                bits.clone(),
            ),
        );
        let points = lam(
            "_i",
            word64(),
            lam(
                "_within",
                within(q.clone(), i),
                cube(
                    &BITS[..3],
                    source_bits
                        .reverse()
                        .then(original)?
                        .then(pattern_bits)?
                        .proof,
                    true,
                ),
            ),
        );
        self.emit(
            "RelationTransfer",
            &args,
            relation(u.clone(), q.clone(), k.clone()),
            pc(
                "Make",
                vec![
                    truth(less(size(u.clone()), size(q.clone())), false),
                    content(u, q, k),
                    new_bound,
                    points,
                ],
            ),
            true,
        )
    }
    fn match_transfer(&mut self) -> Result<()> {
        let (s, u, p, q, k) = (var("s"), var("u"), var("p"), var("q"), var("k"));
        let base = vec![
            ("s", string_type()),
            ("u", string_type()),
            ("p", string_type()),
            ("q", string_type()),
            ("k", boolean()),
            ("se", same(s.clone(), u.clone())),
            ("pe", same(p.clone(), q.clone())),
            ("sup", upper(s.clone())),
            ("pup", upper(p.clone())),
        ];
        let mut args = base.clone();
        args.push(("h", truth(matches(s.clone(), p.clone(), k.clone()), true)));
        self.theorem(
            "MatchTransfer",
            &args,
            Equality::boolean(
                matches(u.clone(), q.clone(), k.clone()),
                bit(true),
                af(
                    "Complete",
                    vec![
                        u.clone(),
                        q.clone(),
                        k.clone(),
                        af(
                            "RelationTransfer",
                            vec![
                                s.clone(),
                                u.clone(),
                                p.clone(),
                                q.clone(),
                                k.clone(),
                                var("se"),
                                var("pe"),
                                var("sup"),
                                var("pup"),
                                af(
                                    "Reflect",
                                    vec![s.clone(), p.clone(), k.clone(), var("pup"), var("h")],
                                ),
                            ],
                        ),
                    ],
                ),
            ),
        )?;
        let forward = lam(
            "_h",
            truth(matches(s.clone(), p.clone(), k.clone()), true),
            af(
                "MatchTransfer",
                vec![
                    s.clone(),
                    u.clone(),
                    p.clone(),
                    q.clone(),
                    k.clone(),
                    var("se"),
                    var("pe"),
                    var("sup"),
                    var("pup"),
                    var("_h"),
                ],
            ),
        );
        let backward = lam(
            "_h",
            truth(matches(u.clone(), q.clone(), k.clone()), true),
            af(
                "MatchTransfer",
                vec![
                    u.clone(),
                    s.clone(),
                    q.clone(),
                    p.clone(),
                    k.clone(),
                    ec("EquivalentSym", vec![s.clone(), u.clone(), var("se")]),
                    ec("EquivalentSym", vec![p.clone(), q.clone(), var("pe")]),
                    af(
                        "UpperTransfer",
                        vec![s.clone(), u.clone(), var("se"), var("sup")],
                    ),
                    af(
                        "UpperTransfer",
                        vec![p.clone(), q.clone(), var("pe"), var("pup")],
                    ),
                    var("_h"),
                ],
            ),
        );
        self.theorem(
            "MatchRespects",
            &base,
            Equality::boolean(
                matches(s.clone(), p.clone(), k.clone()),
                matches(u.clone(), q.clone(), k.clone()),
                ec(
                    "TruthIff",
                    vec![
                        matches(s.clone(), p.clone(), k.clone()),
                        matches(u, q, k.clone()),
                        forward,
                        backward,
                    ],
                ),
            ),
        )?;
        let args = [
            ("s", string_type()),
            ("p", string_type()),
            ("k", boolean()),
            ("sup", upper(s.clone())),
            ("pup", upper(p.clone())),
        ];
        self.theorem(
            "Normalization",
            &args,
            Equality::boolean(
                matches(norm(s.clone()), norm(p.clone()), k.clone()),
                matches(s.clone(), p.clone(), k.clone()),
                af(
                    "MatchRespects",
                    vec![
                        norm(s.clone()),
                        s.clone(),
                        norm(p.clone()),
                        p.clone(),
                        k.clone(),
                        ec("NormalizationEquivalent", vec![s.clone()]),
                        ec("NormalizationEquivalent", vec![p.clone()]),
                        call(
                            "Mpk.Go.Practical.String.Normalization.Upper",
                            vec![s, var("sup")],
                        ),
                        call(
                            "Mpk.Go.Practical.String.Normalization.Upper",
                            vec![p, var("pup")],
                        ),
                    ],
                ),
            ),
        )
    }
}
