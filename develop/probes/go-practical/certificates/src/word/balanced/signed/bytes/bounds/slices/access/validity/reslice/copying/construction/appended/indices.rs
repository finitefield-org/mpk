//! Source-defined indexes lift to the result and its saved full-capacity alias.
use super::*;

pub(super) fn mapped() -> numeric::Word {
    numeric::addition(
        canon(WORD, length(v("destination"))),
        canon(WORD, v("index")),
    )
}
pub(super) fn data() -> Vec<(String, Expr)> {
    let mut a = super::data();
    a.push(("index".into(), word_ty()));
    a.push((
        "source_index_guard".into(),
        eq(defined(v("source"), v("index")), literal(true)),
    ));
    a
}
pub(super) fn arguments() -> Vec<Expr> {
    data().iter().map(|(n, _)| v(n)).collect()
}
pub(super) fn source_facts(p: &mut Producer) -> (Typed, Typed) {
    indexed::active_facts(
        p,
        v("source"),
        v("storage_length"),
        v("live_mask"),
        v("source_valid"),
        v("index"),
        v("source_index_guard"),
    )
}
pub(super) fn bounds(p: &mut Producer) -> [Typed; 3] {
    let (_, lt) = source_facts(p);
    let [hc, _] = guard_facts(p);
    let l = canon(WORD, length(v("destination")));
    let n = canon(WORD, length(v("source")));
    let ci = canon(WORD, v("index"));
    let q = an(
        "Monotone",
        WORD,
        vec![l.clone(), ci.clone(), n.clone(), lt.proof, hc.proof],
    );
    let no = conjunction(
        p,
        not(carry(WORD, l.clone(), ci.clone(), literal(false))),
        word_less(
            WORD,
            mapped().value,
            total(v("destination"), v("source")).value,
        ),
        q.clone(),
        false,
    );
    let no = not_true(p, carry(WORD, l, ci, literal(false)), no.proof);
    let lt = conjunction(
        p,
        not(no.left.clone()),
        word_less(
            WORD,
            mapped().value,
            total(v("destination"), v("source")).value,
        ),
        q,
        true,
    );
    let nn = nonnegative(p);
    let nn = less_nonnegative(
        mapped().value,
        total(v("destination"), v("source")).value,
        lt.proof.clone(),
        nn.proof,
    );
    [nn, lt, no]
}
fn lift_proof(p: &mut Producer, alias: bool) -> Typed {
    let d = v("destination");
    let s = v("source");
    let m = mapped();
    let [nn, lt, _] = bounds(p);
    let target = if alias {
        full(d.clone())
    } else {
        view(d.clone(), s.clone())
    };
    let hv = if alias {
        indexed::full_valid(
            d.clone(),
            v("storage_length"),
            v("live_mask"),
            v("destination_valid"),
        )
    } else {
        validity::result()
    };
    let lt = if alias {
        let le = guard_facts(p)[1].clone();
        let c = canon(WORD, capacity(d.clone()));
        let t = total(d.clone(), s.clone()).value;
        let no = not_true(p, word_less(WORD, c.clone(), t.clone()), le.proof);
        mixed_transitive(WORD, m.value.clone(), t, c, lt.proof, no.proof)
    } else {
        lt
    };
    let mut active_proof = reflexive_t(
        bool_ty(),
        active(m.value.clone(), canon(WORD, length(target.clone()))),
    );
    if alias {
        active_proof = change(p, active_proof, view_projection(d.clone(), 3, true));
    } else {
        active_proof = change(p, active_proof, projection(d.clone(), s.clone(), 3));
        active_proof = change(p, active_proof, scalar(&total(d, s)));
    }
    active_proof = change(p, active_proof, scalar(&m));
    for f in [nn, lt] {
        active_proof = maybe(p, active_proof, f);
    }
    let active_proof = finish(p, active_proof);
    assert_eq!(active_proof.right, literal(true));
    indexed::lift(
        target,
        [v("storage_length"), v("live_mask")],
        hv.proof,
        m.value,
        active_proof.proof,
    )
}
pub(super) fn index(alias: bool) -> Typed {
    let d = if alias {
        full(v("destination"))
    } else {
        view(v("destination"), v("source"))
    };
    typed(
        bool_ty(),
        defined(d, mapped().value),
        literal(true),
        nc(
            if alias {
                "AppendAliasIndex"
            } else {
                "AppendIndex"
            },
            arguments(),
        ),
    )
}
pub(super) fn install(p: &mut Producer, trace: Option<&Path>) {
    for alias in [false, true] {
        let q = lift_proof(p, alias);
        theorem(
            p,
            if alias {
                "SliceAccessValidityResliceBulkConstructionAppendAliasIndex"
            } else {
                "SliceAccessValidityResliceBulkConstructionAppendIndex"
            },
            &data(),
            q,
        );
    }
    save_trace(p, trace, "construct-append-index");
}
