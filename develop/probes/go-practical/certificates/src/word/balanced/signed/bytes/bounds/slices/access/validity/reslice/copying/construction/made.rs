//! Make's signed definedness, initialized capacity and fresh-region framing.
use super::*;

fn words() -> [numeric::Word; 2] {
    [numeric::source(v("length")), numeric::source(v("capacity"))]
}
pub(super) fn guard(l: Expr, k: Expr) -> Expr {
    let l = canon(WORD, l);
    let k = canon(WORD, k);
    both(&[
        not(sign_bit(WORD, l.clone())),
        not(sign_bit(WORD, k.clone())),
        numeric::le(l, k),
    ])
}
pub(super) fn view(r: Expr, l: Expr, k: Expr) -> Expr {
    ctor(
        literal(false),
        r,
        literal_word(WORD, 0),
        canon(WORD, l),
        canon(WORD, k),
    )
}
pub(super) fn initialized(h: Expr, r: Expr, k: Expr) -> Expr {
    nc("MakeHeap", vec![h, r, k])
}
pub(super) fn alive(live: Expr, r: Expr) -> Expr {
    nc("MakeLive", vec![live, r])
}
fn alive_def(live: Expr, r: Expr, q: Expr) -> Typed {
    typed(
        bool_ty(),
        apply(alive(live.clone(), r.clone()), vec![q.clone()]),
        choose(
            word_eq(0, q.clone(), r.clone()),
            literal(true),
            apply(live.clone(), vec![q.clone()]),
        ),
        nc("MakeLiveDefinition", vec![live, r, q]),
    )
}
fn cell_body(h: Expr, r: Expr, k: Expr, rr: Expr, q: Expr) -> Expr {
    choose_t(
        element(),
        and(
            word_eq(0, rr.clone(), r),
            word_less(WORD, q.clone(), canon(WORD, k)),
        ),
        cell_zero(),
        apply(h, vec![rr, q]),
    )
}
fn cell_def(h: Expr, r: Expr, k: Expr, rr: Expr, i: Expr) -> Typed {
    typed(
        element(),
        read_heap(
            initialized(h.clone(), r.clone(), k.clone()),
            rr.clone(),
            i.clone(),
        ),
        cell_body(
            h.clone(),
            r.clone(),
            k.clone(),
            rr.clone(),
            canon(WORD, i.clone()),
        ),
        nc("MakeCellDefinition", vec![h, r, k, rr, i]),
    )
}
pub(super) fn data() -> Vec<(String, Expr)> {
    vec![
        ("live_mask".into(), cube_type(1)),
        ("fresh_region".into(), bool_ty()),
        ("length".into(), word_ty()),
        ("capacity".into(), word_ty()),
        (
            "fresh".into(),
            eq(
                apply(v("live_mask"), vec![v("fresh_region")]),
                literal(false),
            ),
        ),
        (
            "make_guard".into(),
            eq(guard(v("length"), v("capacity")), literal(true)),
        ),
    ]
}
fn arguments() -> Vec<Expr> {
    data().iter().map(|(n, _)| v(n)).collect()
}
fn projections(p: &mut Producer, mut q: Typed) -> Typed {
    let xs = vec![
        literal(false),
        v("fresh_region"),
        literal_word(WORD, 0),
        canon(WORD, v("length")),
        canon(WORD, v("capacity")),
    ];
    for f in 0..5 {
        q = maybe(p, q, ctor_projection(xs.clone(), f));
    }
    q
}
fn guard_facts(p: &mut Producer) -> Vec<Typed> {
    let [l, k] = words();
    facts(
        p,
        &[
            not(sign_bit(WORD, l.value.clone())),
            not(sign_bit(WORD, k.value.clone())),
            numeric::le(l.value, k.value),
        ],
        v("make_guard"),
    )
}
pub(super) fn validity() -> Typed {
    typed(
        bool_ty(),
        valid(
            view(v("fresh_region"), v("length"), v("capacity")),
            canon(WORD, v("capacity")),
            alive(v("live_mask"), v("fresh_region")),
        ),
        literal(true),
        nc("MakeValid", arguments()),
    )
}
fn valid_proof(p: &mut Producer) -> Typed {
    let [_, k] = words();
    let fs = guard_facts(p);
    let d = view(v("fresh_region"), v("length"), v("capacity"));
    let q = valid_def(d, k.value.clone(), alive(v("live_mask"), v("fresh_region")));
    let mut q = projections(p, q);
    q = change(
        p,
        q,
        alive_def(v("live_mask"), v("fresh_region"), v("fresh_region")),
    );
    q = change(p, q, refl_equal(0, v("fresh_region")));
    q = change(p, q, arith::canonical_zero(WORD));
    q = change(p, q, canon_idem(WORD, v("capacity")));
    q = change(p, q, zero_left(k.clone(), true));
    q = change(p, q, address_def(literal_word(WORD, 0), k.value.clone()));
    q = maybe(p, q, arith::canonical_zero(WORD));
    q = maybe(p, q, scalar(&k));
    let z = zero_left(k.clone(), false);
    q = change(p, q, z);
    q = change(
        p,
        q,
        equal_not_less(
            WORD,
            k.value.clone(),
            k.value.clone(),
            refl_equal(WORD, k.value).proof,
        ),
    );
    for f in fs {
        q = maybe(p, q, f);
    }
    let q = finish(p, q);
    assert_eq!(q.right, literal(true));
    q
}
pub(super) fn read_data() -> Vec<(String, Expr)> {
    let mut a = vec![("heap".into(), heap())];
    a.extend(data());
    a.push(("index".into(), word_ty()));
    a.push((
        "full_index_guard".into(),
        eq(
            defined(
                full(view(v("fresh_region"), v("length"), v("capacity"))),
                v("index"),
            ),
            literal(true),
        ),
    ));
    a
}
fn capacity_index(p: &mut Producer) -> Typed {
    let d = view(v("fresh_region"), v("length"), v("capacity"));
    let (_, g) = index_facts(p, full(d.clone()), v("index"), v("full_index_guard"));
    let mut g = numeric::change_left(p, g, view_projection(d, 3, true));
    let xs = vec![
        literal(false),
        v("fresh_region"),
        literal_word(WORD, 0),
        canon(WORD, v("length")),
        canon(WORD, v("capacity")),
    ];
    g = numeric::change_left(p, g, ctor_projection(xs, 4));
    let fs = guard_facts(p);
    let nn = not_true(
        p,
        sign_bit(WORD, canon(WORD, v("capacity"))),
        fs[1].proof.clone(),
    );
    let q = next(
        backwards(signed_nonnegative(
            canon(WORD, v("capacity")),
            v("index"),
            nn.proof,
        )),
        g,
    );
    conjunction(
        p,
        not(sign_bit(WORD, canon(WORD, v("index")))),
        word_less(WORD, canon(WORD, v("index")), canon(WORD, v("capacity"))),
        q.proof,
        true,
    )
}
pub(super) fn zero_read(h: Expr) -> Typed {
    let mut a = vec![h.clone()];
    a.extend(arguments());
    a.extend([v("index"), v("full_index_guard")]);
    typed(
        element(),
        read(
            initialized(h, v("fresh_region"), v("capacity")),
            full(view(v("fresh_region"), v("length"), v("capacity"))),
            v("index"),
        ),
        cell_zero(),
        nc("MakeZeroRead", a),
    )
}
fn zero_read_proof(p: &mut Producer) -> Typed {
    let d = view(v("fresh_region"), v("length"), v("capacity"));
    let h = initialized(v("heap"), v("fresh_region"), v("capacity"));
    let mut q = read_definition(h, d.clone(), v("index"));
    q = projections(p, q);
    q = change(p, q, address_def(literal_word(WORD, 0), v("index")));
    q = change(p, q, arith::canonical_zero(WORD));
    let z = zero_left(numeric::source(v("index")), false);
    q = change(p, q, z);
    q = change(
        p,
        q,
        cell_def(
            v("heap"),
            v("fresh_region"),
            v("capacity"),
            v("fresh_region"),
            canon(WORD, v("index")),
        ),
    );
    q = change(p, q, canon_idem(WORD, v("index")));
    q = change(p, q, refl_equal(0, v("fresh_region")));
    let lt = capacity_index(p);
    q = change(p, q, lt);
    let pred = and(literal(true), literal(true));
    let n = normalize(pred, p);
    q = change(p, q, n);
    let left = q.right.clone();
    next(
        q,
        typed(
            element(),
            left,
            cell_zero(),
            apply(c(REFL), vec![element(), cell_zero()]),
        ),
    )
}
pub(super) fn other_data() -> Vec<(String, Expr)> {
    let mut a = vec![("heap".into(), heap())];
    a.extend(data());
    a.push(("index".into(), word_ty()));
    a
}
pub(super) fn other(h: Expr) -> Typed {
    let mut a = vec![h.clone()];
    a.extend(arguments());
    a.push(v("index"));
    typed(
        element(),
        read_heap(
            initialized(h.clone(), v("fresh_region"), v("capacity")),
            not(v("fresh_region")),
            v("index"),
        ),
        read_heap(h, not(v("fresh_region")), v("index")),
        nc("MakeOtherFrame", a),
    )
}
fn other_proof(p: &mut Producer) -> Typed {
    let mut q = cell_def(
        v("heap"),
        v("fresh_region"),
        v("capacity"),
        not(v("fresh_region")),
        v("index"),
    );
    q = change(
        p,
        q,
        typed(
            bool_ty(),
            word_eq(0, not(v("fresh_region")), v("fresh_region")),
            literal(false),
            at("RegionApart", vec![v("fresh_region")]),
        ),
    );
    let Expr::Apply(_, a) = &q.right else {
        panic!()
    };
    let pred = a[3].clone();
    let n = normalize(pred, p);
    q = change(p, q, n);
    let Expr::Apply(_, a) = &q.right else {
        panic!()
    };
    let rhs = a[1].clone();
    let left = q.right.clone();
    next(
        q,
        typed(
            element(),
            left,
            rhs.clone(),
            apply(c(REFL), vec![element(), rhs]),
        ),
    )
}
pub(super) fn fresh_data() -> Vec<(String, Expr)> {
    let mut a = other_data();
    a.extend([
        ("query_region".into(), bool_ty()),
        (
            "borrowed_live".into(),
            eq(
                apply(v("live_mask"), vec![v("query_region")]),
                literal(true),
            ),
        ),
    ]);
    a
}
pub(super) fn fresh_frame(h: Expr) -> Typed {
    let mut a = vec![h.clone()];
    a.extend(arguments());
    a.extend([v("index"), v("query_region"), v("borrowed_live")]);
    typed(
        element(),
        read_heap(
            initialized(h.clone(), v("fresh_region"), v("capacity")),
            v("query_region"),
            v("index"),
        ),
        read_heap(h, v("query_region"), v("index")),
        nc("MakeFreshFrame", a),
    )
}
fn apart(query: Expr, borrowed: Expr) -> Typed {
    let same = word_eq(0, query.clone(), v("fresh_region"));
    let e = typed(
        bool_ty(),
        query.clone(),
        v("fresh_region"),
        equal_bits(0, query.clone(), v("fresh_region"), v("__same")),
    );
    let e = mapped(
        e,
        bool_ty(),
        lambda("__r", bool_ty(), apply(v("live_mask"), vec![v("__r")])),
        apply(v("live_mask"), vec![query.clone()]),
        apply(v("live_mask"), vec![v("fresh_region")]),
    );
    let yes = next(
        backwards(e),
        typed(
            bool_ty(),
            apply(v("live_mask"), vec![query.clone()]),
            literal(true),
            borrowed.clone(),
        ),
    );
    let absurd = impossible(
        bool_ty(),
        same.clone(),
        literal(false),
        contradiction(yes, v("fresh")),
    );
    typed(
        bool_ty(),
        same.clone(),
        literal(false),
        cases(
            same.clone(),
            "__same",
            eq(same, literal(false)),
            v("__same"),
            absurd.proof,
        ),
    )
}
fn fresh_proof(p: &mut Producer) -> Typed {
    let hn = apart(v("query_region"), v("borrowed_live"));
    let mut q = cell_def(
        v("heap"),
        v("fresh_region"),
        v("capacity"),
        v("query_region"),
        v("index"),
    );
    q = change(p, q, hn);
    let Expr::Apply(_, a) = &q.right else {
        panic!()
    };
    let pred = a[3].clone();
    let n = normalize(pred, p);
    q = change(p, q, n);
    let Expr::Apply(_, a) = &q.right else {
        panic!()
    };
    let rhs = a[1].clone();
    let left = q.right.clone();
    next(
        q,
        typed(
            element(),
            left,
            rhs.clone(),
            apply(c(REFL), vec![element(), rhs]),
        ),
    )
}
pub(super) fn install(p: &mut Producer, trace: Option<&Path>) {
    let a = vec![
        ("heap".into(), heap()),
        ("fresh_region".into(), bool_ty()),
        ("capacity".into(), word_ty()),
    ];
    function_definition(
        p,
        "SliceAccessValidityResliceBulkConstructionMakeHeap",
        &a,
        heap(),
        lambda(
            "__region",
            bool_ty(),
            lambda(
                "__key",
                word_ty(),
                cell_body(
                    v("heap"),
                    v("fresh_region"),
                    v("capacity"),
                    v("__region"),
                    v("__key"),
                ),
            ),
        ),
    );
    let mut ac = a;
    ac.extend([
        ("query_region".into(), bool_ty()),
        ("index".into(), word_ty()),
    ]);
    let mut q = cell_def(
        v("heap"),
        v("fresh_region"),
        v("capacity"),
        v("query_region"),
        v("index"),
    );
    q.proof = apply(c(REFL), vec![element(), q.right.clone()]);
    theorem(
        p,
        "SliceAccessValidityResliceBulkConstructionMakeCellDefinition",
        &ac,
        q,
    );
    let a = vec![
        ("live_mask".into(), cube_type(1)),
        ("fresh_region".into(), bool_ty()),
    ];
    function_definition(
        p,
        "SliceAccessValidityResliceBulkConstructionMakeLive",
        &a,
        cube_type(1),
        lambda(
            "__r",
            bool_ty(),
            choose(
                word_eq(0, v("__r"), v("fresh_region")),
                literal(true),
                apply(v("live_mask"), vec![v("__r")]),
            ),
        ),
    );
    let mut ac = a;
    ac.push(("query_region".into(), bool_ty()));
    let mut q = alive_def(v("live_mask"), v("fresh_region"), v("query_region"));
    q.proof = refl(q.right.clone());
    theorem(
        p,
        "SliceAccessValidityResliceBulkConstructionMakeLiveDefinition",
        &ac,
        q,
    );
    let q = valid_proof(p);
    theorem(
        p,
        "SliceAccessValidityResliceBulkConstructionMakeValid",
        &data(),
        q,
    );
    save_trace(p, trace, "construct-make-valid");
    let q = zero_read_proof(p); // fold full-view read separately, without erasing its cap guard
    let d = view(v("fresh_region"), v("length"), v("capacity"));
    let mut q = q;
    let bridge = read_definition(
        initialized(v("heap"), v("fresh_region"), v("capacity")),
        full(d.clone()),
        v("index"),
    );
    let bridge = change(p, bridge, view_projection(d.clone(), 1, true));
    let bridge = change(p, bridge, view_projection(d.clone(), 2, true));
    let rhs = read_definition(
        initialized(v("heap"), v("fresh_region"), v("capacity")),
        d,
        v("index"),
    );
    let bridge = next(bridge, backwards(rhs));
    q = next(bridge, q);
    theorem(
        p,
        "SliceAccessValidityResliceBulkConstructionMakeZeroRead",
        &read_data(),
        q,
    );
    save_trace(p, trace, "construct-make-read");
    let q = other_proof(p);
    theorem(
        p,
        "SliceAccessValidityResliceBulkConstructionMakeOtherFrame",
        &other_data(),
        q,
    );
    let q = fresh_proof(p);
    theorem(
        p,
        "SliceAccessValidityResliceBulkConstructionMakeFreshFrame",
        &fresh_data(),
        q,
    );
    save_trace(p, trace, "construct-make-frame");
}

pub(super) mod composed;
pub(super) fn install_composed(p: &mut Producer, trace: Option<&Path>) {
    composed::install(p, trace);
}
