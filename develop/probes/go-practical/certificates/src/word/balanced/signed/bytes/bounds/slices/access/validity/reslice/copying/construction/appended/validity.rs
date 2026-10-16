//! Result validity from actual computed capacity bounds, including nil.
use super::*;

pub(super) fn result() -> Typed {
    typed(
        bool_ty(),
        valid(
            view(v("destination"), v("source")),
            v("storage_length"),
            v("live_mask"),
        ),
        literal(true),
        nc("AppendValid", arguments()),
    )
}
fn nil_proof(p: &mut Producer, mut q: Typed) -> Typed {
    let d = v("destination");
    let s = v("source");
    let n = v("storage_length");
    let live = v("live_mask");
    let old = selected_valid(
        p,
        d.clone(),
        n,
        live,
        v("destination_valid"),
        true,
        v("__nil"),
    );
    let fs = facts(p, &nil_clauses(d.clone()), old.proof);
    let raw = capacity(d.clone());
    let z = literal_word(WORD, 0);
    let capz = canon_congr(
        WORD,
        raw.clone(),
        z.clone(),
        equal_bits(WORD, raw, z, fs[1].proof.clone()),
    );
    let capz = next(capz, arith::canonical_zero(WORD));
    let le = guard_facts(p)[1].clone();
    let le = numeric::change_left(p, le, capz);
    let t = total(d, s);
    let zbits = numeric::to_zero(t.value.clone(), le.proof);
    let teq = numeric::canonical_equal(&t, &numeric::zero(), zbits.proof);
    q = maybe(p, q, teq);
    q = maybe(p, q, refl_equal(WORD, literal_word(WORD, 0)));
    q = maybe(p, q, fs[1].clone());
    finish(p, q)
}
fn nonnil_proof(p: &mut Producer, mut q: Typed) -> Typed {
    let d = v("destination");
    let n = v("storage_length");
    let live = v("live_mask");
    let fs = nonnil_facts(p, d.clone(), n, live, v("destination_valid"), v("__nil"));
    let nn = nonnegative(p);
    let le = guard_facts(p)[1].clone();
    let raw = capacity(d);
    let t = total(v("destination"), v("source"));
    let eq = canon_equal(WORD, raw.clone());
    let tr = substitute_equal(
        WORD,
        false,
        false,
        raw.clone(),
        canon(WORD, raw),
        t.value,
        eq.proof,
    );
    let le = numeric::change_left(p, le, backwards(tr));
    for f in [
        fs[0].clone(),
        fs[1].clone(),
        nn,
        fs[3].clone(),
        fs[4].clone(),
        le,
        fs[6].clone(),
        fs[7].clone(),
    ] {
        q = maybe(p, q, f);
    }
    finish(p, q)
}
fn proof(p: &mut Producer) -> Typed {
    let (d, s) = (v("destination"), v("source"));
    let n = v("storage_length");
    let live = v("live_mask");
    let mut q = valid_def(view(d.clone(), s.clone()), n, live);
    for f in 0..5 {
        q = maybe(p, q, projection(d.clone(), s.clone(), f));
    }
    let yes = select(p, q.clone(), nil(d.clone()), true, v("__nil"));
    let yes = nil_proof(p, yes);
    let no = select(p, q.clone(), nil(d.clone()), false, v("__nil"));
    let no = nonnil_proof(p, no);
    assert_eq!(yes.right, literal(true));
    assert_eq!(no.right, literal(true));
    typed(
        bool_ty(),
        q.left.clone(),
        literal(true),
        cases(
            nil(d),
            "__nil",
            eq(q.left, literal(true)),
            no.proof,
            yes.proof,
        ),
    )
}
pub(super) fn install(p: &mut Producer, trace: Option<&Path>) {
    let q = proof(p);
    theorem(
        p,
        "SliceAccessValidityResliceBulkConstructionAppendValid",
        &data(),
        q,
    );
    save_trace(p, trace, "construct-append-valid");
}
