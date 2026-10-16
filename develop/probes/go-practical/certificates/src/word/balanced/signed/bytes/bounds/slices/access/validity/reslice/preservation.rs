//! Computed source guards preserve the complete descriptor-validity predicate.
use super::*;
use numeric::Word;

fn scalar_normal(w: &Word) -> Typed {
    typed(
        word_ty(),
        canon(WORD, w.value.clone()),
        w.value.clone(),
        w.normal.clone(),
    )
}
pub(super) fn data(three: bool) -> Vec<(String, Expr)> {
    let mut a = model::data(three);
    a.extend([
        ("storage_length".into(), word_ty()),
        ("live_mask".into(), cube_type(1)),
        (
            "old_valid".into(),
            eq(
                valid(v("descriptor"), v("storage_length"), v("live_mask")),
                literal(true),
            ),
        ),
        (
            "range_guard".into(),
            eq(
                model::guard(three, v("descriptor"), v("lower"), v("upper"), v("maximum")),
                literal(true),
            ),
        ),
    ]);
    a
}
pub(super) fn validity(
    three: bool,
    indices: [Expr; 4],
    heap_args: [Expr; 2],
    proofs: [Expr; 2],
) -> Typed {
    let [d, l, h, m] = indices;
    let [n, live] = heap_args;
    let view = model::view(three, d.clone(), l.clone(), h.clone(), m.clone());
    let mut args = model::arguments(three, d, l, h, m);
    args.extend([
        n.clone(),
        live.clone(),
        proofs[0].clone(),
        proofs[1].clone(),
    ]);
    typed(
        bool_ty(),
        valid(view, n, live),
        literal(true),
        rc(if three { "ThreeValid" } else { "TwoValid" }, args),
    )
}
fn high_cap(g: &model::GuardFacts) -> Typed {
    numeric::transitive_le(
        g.high.value.clone(),
        g.maximum.value.clone(),
        g.capacity.value.clone(),
        g.ordered[1].proof.clone(),
        g.ordered[2].proof.clone(),
    )
}
fn nil_zero(x: &Word, h: Typed) -> Typed {
    let q = numeric::to_zero(x.value.clone(), h.proof);
    numeric::canonical_equal(x, &numeric::zero(), q.proof)
}
fn nil_proof(p: &mut Producer, three: bool, mut e: Typed) -> Typed {
    let (d, l, h, m) = (v("descriptor"), v("lower"), v("upper"), v("maximum"));
    let n = v("storage_length");
    let live = v("live_mask");
    let g = model::guard_facts(three, d.clone(), l, h, m, v("range_guard"));
    let selected = selected_valid(p, d.clone(), n, live, v("old_valid"), true, v("__nil"));
    let fs = facts(p, &nil_clauses(d.clone()), selected.proof);
    let z = literal_word(WORD, 0);
    let capraw = capacity(d);
    let cap_zero = canon_congr(
        WORD,
        capraw.clone(),
        z.clone(),
        equal_bits(WORD, capraw, z, fs[1].proof.clone()),
    );
    let cap_zero = next(cap_zero, arith::canonical_zero(WORD));
    let hm = if three {
        let le = numeric::change_left(p, g.ordered[2].clone(), cap_zero.clone());
        nil_zero(&g.maximum, le)
    } else {
        cap_zero.clone()
    };
    let high = numeric::change_left(p, high_cap(&g), cap_zero);
    let high = nil_zero(&g.high, high);
    let low_high = numeric::change_left(p, g.ordered[0].clone(), high.clone());
    let low = nil_zero(&g.low, low_high);
    let dl = numeric::difference(g.high.value.clone(), g.low.value.clone());
    let dk = numeric::difference(g.maximum.value.clone(), g.low.value.clone());
    for (word, top) in [(dl, high), (dk, hm)] {
        let mut q = reflexive_t(word_ty(), word.value);
        q = change(p, q, top);
        q = change(p, q, low.clone());
        q = next(q, numeric::difference_zero());
        e = maybe(p, e, q);
    }
    e = change(p, e, refl_equal(WORD, literal_word(WORD, 0)));
    let q = normalize(e.right.clone(), p);
    next(e, q)
}
fn nonnil_proof(p: &mut Producer, three: bool, mut e: Typed) -> Typed {
    let (d, l, h, m) = (v("descriptor"), v("lower"), v("upper"), v("maximum"));
    let (n, live) = (v("storage_length"), v("live_mask"));
    let g = model::guard_facts(three, d.clone(), l, h, m, v("range_guard"));
    let fs = nonnil_facts(p, d.clone(), n.clone(), live, v("old_valid"), v("__nil"));
    let off = numeric::source(offset(d.clone()));
    let new_off = numeric::addition(off.value.clone(), g.low.value.clone());
    let new_cap = numeric::difference(g.maximum.value.clone(), g.low.value.clone());
    let cap_end = numeric::addition(off.value.clone(), g.capacity.value.clone());
    let max_end = numeric::addition(off.value.clone(), g.maximum.value.clone());
    let hc = not_true(
        p,
        carry(
            WORD,
            off.value.clone(),
            g.capacity.value.clone(),
            literal(false),
        ),
        fs[6].proof.clone(),
    );
    let max_carry = numeric::bounded_add(
        off.clone(),
        g.maximum.clone(),
        g.capacity.clone(),
        g.ordered[2].proof.clone(),
        hc.proof.clone(),
        true,
    );
    let max_le = numeric::bounded_add(
        off.clone(),
        g.maximum.clone(),
        g.capacity.clone(),
        g.ordered[2].proof.clone(),
        hc.proof,
        false,
    );
    let old_end = numeric::change_left(
        p,
        fs[7].clone(),
        address_def(offset(d.clone()), capacity(d)),
    );
    let max_n = numeric::transitive_le(
        max_end.value.clone(),
        cap_end.value,
        n.clone(),
        max_le.proof,
        old_end.proof,
    );
    let low_max = numeric::transitive_le(
        g.low.value.clone(),
        g.high.value.clone(),
        g.maximum.value.clone(),
        g.ordered[0].proof.clone(),
        g.ordered[1].proof.clone(),
    );
    let restore = numeric::restore(
        g.maximum.clone(),
        g.low.clone(),
        low_max.proof.clone(),
        true,
        false,
    );
    let restore_carry = numeric::restore(
        g.maximum.clone(),
        g.low.clone(),
        low_max.proof.clone(),
        true,
        true,
    );
    let mut outer = reflexive_t(
        bool_ty(),
        carry(
            WORD,
            off.value.clone(),
            sum(
                WORD,
                g.low.value.clone(),
                new_cap.value.clone(),
                literal(false),
            ),
            literal(false),
        ),
    );
    outer = change(p, outer, restore.clone());
    outer = next(outer, max_carry);
    let no = assoc::no_wrap(
        WORD,
        off.value.clone(),
        g.low.value.clone(),
        new_cap.value.clone(),
        restore_carry.proof,
        outer.proof,
    );
    let first = not(carry(
        WORD,
        off.value.clone(),
        g.low.value.clone(),
        literal(false),
    ));
    let total_carry = carry(
        WORD,
        new_off.value.clone(),
        new_cap.value.clone(),
        literal(false),
    );
    let total_no = conjunction(p, first, not(total_carry.clone()), no.proof, true);
    let total_no = not_true(p, total_carry, total_no.proof);
    let flags = vec![literal(false); 4];
    let mut total = assoc::reassociate(
        WORD,
        off.value.clone(),
        g.low.value.clone(),
        new_cap.value.clone(),
        &flags,
        rc("AssocStart", vec![]),
    );
    total = change(p, total, restore);
    let total_n = numeric::change_left(p, max_n.clone(), backwards(total.clone()));
    let lower = numeric::sum_lower(new_off.clone(), new_cap.clone(), total_no.proof.clone());
    let new_off_n = numeric::transitive_le(
        new_off.value.clone(),
        sum(
            WORD,
            new_off.value.clone(),
            new_cap.value.clone(),
            literal(false),
        ),
        n.clone(),
        lower.proof,
        total_n.proof,
    );
    let nn = not_true(p, sign_bit(WORD, n.clone()), fs[4].proof.clone());
    let off_nn = numeric::nonnegative(new_off.value.clone(), n, new_off_n.proof, nn.proof);
    let len_nn = numeric::difference_nonnegative(
        g.high.clone(),
        g.low.clone(),
        g.ordered[0].proof.clone(),
        g.nonnegative[1].proof.clone(),
    );
    let cap_nn = numeric::difference_nonnegative(
        g.maximum.clone(),
        g.low.clone(),
        low_max.proof.clone(),
        g.nonnegative[2].proof.clone(),
    );
    let order = numeric::difference_order(
        g.high.clone(),
        g.low.clone(),
        g.maximum.clone(),
        [
            g.ordered[0].proof.clone(),
            low_max.proof,
            g.ordered[1].proof.clone(),
        ],
    );
    e = change(
        p,
        e,
        address_def(new_off.value.clone(), new_cap.value.clone()),
    );
    e = change(p, e, scalar_normal(&new_off));
    e = change(p, e, scalar_normal(&new_cap));
    e = change(p, e, total_no);
    e = change(p, e, total);
    e = change(p, e, max_n);
    for q in [fs[0].clone(), fs[4].clone(), off_nn, len_nn, cap_nn, order] {
        e = maybe(p, e, q);
    }
    let q = normalize(e.right.clone(), p);
    next(e, q)
}
fn proof(p: &mut Producer, three: bool) -> Typed {
    let (d, l, h, m) = (v("descriptor"), v("lower"), v("upper"), v("maximum"));
    let view = model::view(three, d.clone(), l.clone(), h.clone(), m.clone());
    let mut e = valid_def(view.clone(), v("storage_length"), v("live_mask"));
    for f in 0..5 {
        e = change(
            p,
            e,
            model::projection(three, f, d.clone(), l.clone(), h.clone(), m.clone()),
        );
    }
    let mut proofs = vec![];
    for is_nil in [false, true] {
        let e = select(p, e.clone(), nil(d.clone()), is_nil, v("__nil"));
        let q = if is_nil {
            nil_proof(p, three, e)
        } else {
            nonnil_proof(p, three, e)
        };
        assert_eq!(q.right, literal(true));
        proofs.push(q.proof);
    }
    typed(
        bool_ty(),
        valid(view, v("storage_length"), v("live_mask")),
        literal(true),
        cases(
            nil(d),
            "__nil",
            eq(
                valid(
                    model::view(three, v("descriptor"), v("lower"), v("upper"), v("maximum")),
                    v("storage_length"),
                    v("live_mask"),
                ),
                literal(true),
            ),
            proofs[0].clone(),
            proofs[1].clone(),
        ),
    )
}
pub(super) fn install(p: &mut Producer, trace: Option<&Path>) {
    for three in [false, true] {
        let q = proof(p, three);
        theorem(
            p,
            if three {
                "SliceAccessValidityResliceThreeValid"
            } else {
                "SliceAccessValidityResliceTwoValid"
            },
            &data(three),
            q,
        );
        save_trace(
            p,
            trace,
            if three {
                "reslice-three-valid"
            } else {
                "reslice-two-valid"
            },
        );
    }
}
