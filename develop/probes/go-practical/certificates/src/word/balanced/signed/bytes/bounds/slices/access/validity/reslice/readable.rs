//! Checked source index lifting and whole-cell backing-sharing observations.
use super::*;
use numeric::Word;

fn indices() -> [Expr; 4] {
    [v("descriptor"), v("lower"), v("upper"), v("maximum")]
}
fn new_view(three: bool) -> Expr {
    model::view(three, v("descriptor"), v("lower"), v("upper"), v("maximum"))
}
fn mapped(g: &model::GuardFacts, i: Expr) -> Word {
    numeric::addition(g.low.value.clone(), canon(WORD, i))
}
fn scalar_normal(w: &Word) -> Typed {
    typed(
        word_ty(),
        canon(WORD, w.value.clone()),
        w.value.clone(),
        w.normal.clone(),
    )
}
pub(super) fn data(three: bool) -> Vec<(String, Expr)> {
    let mut a = preservation::data(three);
    a.extend([
        ("index".into(), word_ty()),
        (
            "new_index_guard".into(),
            eq(defined(new_view(three), v("index")), literal(true)),
        ),
    ]);
    a
}
fn args(three: bool) -> Vec<Expr> {
    let mut a = model::arguments(three, v("descriptor"), v("lower"), v("upper"), v("maximum"));
    a.extend([
        v("storage_length"),
        v("live_mask"),
        v("old_valid"),
        v("range_guard"),
        v("index"),
        v("new_index_guard"),
    ]);
    a
}
fn lift_proof(p: &mut Producer, three: bool) -> Typed {
    let (d, l, h, m) = (v("descriptor"), v("lower"), v("upper"), v("maximum"));
    let i = v("index");
    let g = model::guard_facts(
        three,
        d.clone(),
        l.clone(),
        h.clone(),
        m.clone(),
        v("range_guard"),
    );
    let view = new_view(three);
    let mapped_word = mapped(&g, i.clone());
    let len = numeric::difference(g.high.value.clone(), g.low.value.clone());
    let (hnil, hguard) = index_facts(p, view, i.clone(), v("new_index_guard"));
    let hnil = numeric::change_left(
        p,
        hnil,
        model::projection(three, 0, d.clone(), l.clone(), h.clone(), m.clone()),
    );
    let hguard = numeric::change_left(p, hguard, model::projection(three, 3, d.clone(), l, h, m));
    let nnlen = numeric::difference_nonnegative(
        g.high.clone(),
        g.low.clone(),
        g.ordered[0].proof.clone(),
        g.nonnegative[1].proof.clone(),
    );
    let hactive = next(
        backwards(signed_nonnegative(
            len.value.clone(),
            i.clone(),
            nnlen.proof,
        )),
        hguard,
    );
    let si = not(sign_bit(WORD, canon(WORD, i.clone())));
    let li = word_less(WORD, canon(WORD, i.clone()), len.value.clone());
    let lt = conjunction(p, si, li, hactive.proof, true);
    let no = numeric::restore(
        g.high.clone(),
        g.low.clone(),
        g.ordered[0].proof.clone(),
        true,
        true,
    );
    let out = an(
        "Monotone",
        WORD,
        vec![
            g.low.value.clone(),
            canon(WORD, i),
            len.value.clone(),
            lt.proof,
            no.proof,
        ],
    );
    let first = not(carry(
        WORD,
        g.low.value.clone(),
        canon(WORD, v("index")),
        literal(false),
    ));
    let mut lt = conjunction(
        p,
        first,
        word_less(
            WORD,
            mapped_word.value.clone(),
            sum(WORD, g.low.value.clone(), len.value, literal(false)),
        ),
        out,
        true,
    );
    lt = numeric::change_left(
        p,
        lt,
        numeric::restore(
            g.high.clone(),
            g.low.clone(),
            g.ordered[0].proof.clone(),
            true,
            false,
        ),
    );
    let high_cap = numeric::transitive_le(
        g.high.value.clone(),
        g.maximum.value.clone(),
        g.capacity.value.clone(),
        g.ordered[1].proof.clone(),
        g.ordered[2].proof.clone(),
    );
    let no = not_true(
        p,
        word_less(WORD, g.capacity.value.clone(), g.high.value.clone()),
        high_cap.proof,
    );
    let lt = mixed_transitive(
        WORD,
        mapped_word.value.clone(),
        g.high.value,
        g.capacity.value.clone(),
        lt.proof,
        no.proof,
    );
    let nn = less_nonnegative(
        mapped_word.value.clone(),
        g.capacity.value.clone(),
        lt.proof.clone(),
        g.nonnegative[3].proof.clone(),
    );
    let nn = numeric::change_left(p, nn, backwards(scalar_normal(&mapped_word)));
    let lt = numeric::change_left(p, lt, backwards(scalar_normal(&mapped_word)));
    let capraw = capacity(d.clone());
    let ce = canon_equal(WORD, capraw.clone());
    let transport = substitute_equal(
        WORD,
        false,
        true,
        capraw.clone(),
        g.capacity.value,
        canon(WORD, mapped_word.value.clone()),
        ce.proof,
    );
    let lt = next(transport, lt);
    let qs = vec![literal(true); WORD];
    let cap_bits = typed(
        bool_ty(),
        sign_bit(WORD, canon(WORD, capraw.clone())),
        sign_bit(WORD, capraw.clone()),
        apply(canon_bits(WORD, capraw), qs),
    );
    let cap_nn = next(backwards(cap_bits), g.nonnegative[3].clone());
    let mut q = index_def(full(d.clone()), mapped_word.value);
    q = change(p, q, view_projection(d.clone(), 0, true));
    q = change(p, q, view_projection(d, 3, true));
    for fact in [hnil, cap_nn, nn, lt] {
        q = maybe(p, q, fact);
    }
    let n = normalize(q.right.clone(), p);
    q = next(q, n);
    assert_eq!(q.right, literal(true));
    q
}
pub(super) fn lift(three: bool) -> Typed {
    let g = model::guard_facts(
        three,
        v("descriptor"),
        v("lower"),
        v("upper"),
        v("maximum"),
        v("range_guard"),
    );
    typed(
        bool_ty(),
        defined(full(v("descriptor")), mapped(&g, v("index")).value),
        literal(true),
        rc(
            if three {
                "ThreeIndexLift"
            } else {
                "TwoIndexLift"
            },
            args(three),
        ),
    )
}
fn view_read(p: &mut Producer, three: bool, heap: Expr) -> Typed {
    let [d, l, h, m] = indices();
    let i = v("index");
    let g = model::guard_facts(
        three,
        d.clone(),
        l.clone(),
        h.clone(),
        m.clone(),
        v("range_guard"),
    );
    let off = numeric::source(offset(d.clone()));
    let new_off = numeric::addition(off.value.clone(), g.low.value.clone());
    let index = canon(WORD, i.clone());
    let old_index = mapped(&g, i.clone());
    let mut q = read_definition(heap.clone(), new_view(three), i);
    for f in [1, 2] {
        q = change(
            p,
            q,
            model::projection(three, f, d.clone(), l.clone(), h.clone(), m.clone()),
        );
    }
    q = change(p, q, address_def(new_off.value.clone(), v("index")));
    q = change(p, q, scalar_normal(&new_off));
    let zero = vec![literal(false); 4];
    q = change(
        p,
        q,
        assoc::reassociate(
            WORD,
            off.value,
            g.low.value,
            index,
            &zero,
            rc("AssocStart", vec![]),
        ),
    );
    let mut right = read_definition(heap, full(d.clone()), old_index.value.clone());
    right = change(p, right, view_projection(d.clone(), 1, true));
    right = change(p, right, view_projection(d.clone(), 2, true));
    right = change(p, right, address_def(offset(d), old_index.value.clone()));
    right = change(p, right, scalar_normal(&old_index));
    next(q, backwards(right))
}
pub(super) fn read_law(three: bool, heap: Expr) -> Typed {
    let g = model::guard_facts(
        three,
        v("descriptor"),
        v("lower"),
        v("upper"),
        v("maximum"),
        v("range_guard"),
    );
    let mut a = vec![heap.clone()];
    a.extend(args(three));
    typed(
        element(),
        read(heap.clone(), new_view(three), v("index")),
        read(heap, full(v("descriptor")), mapped(&g, v("index")).value),
        rc(if three { "ThreeRead" } else { "TwoRead" }, a),
    )
}
pub(super) fn install(p: &mut Producer, trace: Option<&Path>) {
    for three in [false, true] {
        let q = lift_proof(p, three);
        theorem(
            p,
            if three {
                "SliceAccessValidityResliceThreeIndexLift"
            } else {
                "SliceAccessValidityResliceTwoIndexLift"
            },
            &data(three),
            q,
        );
        let mut a = vec![("heap".into(), heap())];
        a.extend(data(three));
        let q = view_read(p, three, v("heap"));
        theorem(
            p,
            if three {
                "SliceAccessValidityResliceThreeRead"
            } else {
                "SliceAccessValidityResliceTwoRead"
            },
            &a,
            q,
        );
        save_trace(
            p,
            trace,
            if three {
                "reslice-three-read"
            } else {
                "reslice-two-read"
            },
        );
    }
}
