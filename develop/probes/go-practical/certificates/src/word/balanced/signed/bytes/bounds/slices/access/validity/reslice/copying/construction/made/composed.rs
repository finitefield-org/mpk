//! Preallocate then append: reuse the checked operators and preserve old inputs.
use super::*;

fn destination() -> Expr {
    view(v("fresh_region"), v("length"), v("capacity"))
}
fn new_live() -> Expr {
    alive(v("live_mask"), v("fresh_region"))
}
fn first_heap(h: Expr) -> Expr {
    initialized(h, v("fresh_region"), v("capacity"))
}
fn result_heap(h: Expr) -> Expr {
    appended::updated(first_heap(h), destination(), v("source"))
}
fn base_data() -> Vec<(String, Expr)> {
    let mut a = super::data();
    a.extend([
        ("source".into(), descriptor()),
        (
            "source_valid".into(),
            eq(
                valid(v("source"), canon(WORD, v("capacity")), v("live_mask")),
                literal(true),
            ),
        ),
        (
            "no_reallocation".into(),
            eq(appended::guard(destination(), v("source")), literal(true)),
        ),
    ]);
    a
}
fn base_args() -> Vec<Expr> {
    base_data().iter().map(|(n, _)| v(n)).collect()
}
fn make_args() -> Vec<Expr> {
    super::data().iter().map(|(n, _)| v(n)).collect()
}
fn source_valid() -> Typed {
    typed(
        bool_ty(),
        valid(v("source"), canon(WORD, v("capacity")), new_live()),
        literal(true),
        nc("MakeSourceValid", base_args()),
    )
}
fn source_valid_proof(p: &mut Producer) -> Typed {
    let d = v("source");
    let n = canon(WORD, v("capacity"));
    let mut q = valid_def(d.clone(), n.clone(), new_live());
    let yes = select(p, q.clone(), nil(d.clone()), true, v("__nil"));
    let old = selected_valid(
        p,
        d.clone(),
        n.clone(),
        v("live_mask"),
        v("source_valid"),
        true,
        v("__nil"),
    );
    let yes = next(yes, old);
    q = select(p, q, nil(d.clone()), false, v("__nil"));
    let fs = nonnil_facts(
        p,
        d.clone(),
        n,
        v("live_mask"),
        v("source_valid"),
        v("__nil"),
    );
    q = change(
        p,
        q,
        alive_def(v("live_mask"), v("fresh_region"), region(d.clone())),
    );
    for f in fs {
        q = maybe(p, q, f);
    }
    let no = finish(p, q);
    assert_eq!(no.right, literal(true));
    typed(
        bool_ty(),
        yes.left.clone(),
        literal(true),
        cases(
            nil(d),
            "__nil",
            eq(yes.left, literal(true)),
            no.proof,
            yes.proof,
        ),
    )
}
fn indexed_data() -> Vec<(String, Expr)> {
    let mut a = base_data();
    a.extend([
        ("index".into(), word_ty()),
        (
            "source_index_guard".into(),
            eq(defined(v("source"), v("index")), literal(true)),
        ),
    ]);
    a
}
fn source_read(h: Expr) -> Typed {
    let mut a = vec![h.clone()];
    a.extend(indexed_data().iter().map(|(n, _)| v(n)));
    typed(
        element(),
        read(first_heap(h.clone()), v("source"), v("index")),
        read(h, v("source"), v("index")),
        nc("MakeSourceRead", a),
    )
}
fn fresh_at(h: Expr, r: Expr, i: Expr, h_live: Expr) -> Typed {
    let mut a = vec![h.clone()];
    a.extend(make_args());
    a.extend([i.clone(), r.clone(), h_live]);
    typed(
        element(),
        read_heap(first_heap(h.clone()), r.clone(), i.clone()),
        read_heap(h, r, i),
        nc("MakeFreshFrame", a),
    )
}
fn source_read_proof(p: &mut Producer) -> Typed {
    let s = v("source");
    let i = v("index");
    let (hnil, _) = index_facts(p, s.clone(), i.clone(), v("source_index_guard"));
    let fs = nonnil_facts(
        p,
        s.clone(),
        canon(WORD, v("capacity")),
        v("live_mask"),
        v("source_valid"),
        hnil.proof,
    );
    let mut q = read_definition(first_heap(v("heap")), s.clone(), i.clone());
    q = change(
        p,
        q,
        fresh_at(
            v("heap"),
            region(s.clone()),
            address(offset(s.clone()), i.clone()),
            fs[0].proof.clone(),
        ),
    );
    next(q, backwards(read_definition(v("heap"), s, i)))
}
fn append_arguments() -> Vec<Expr> {
    vec![
        destination(),
        v("source"),
        canon(WORD, v("capacity")),
        new_live(),
        validity().proof,
        source_valid().proof,
        v("no_reallocation"),
    ]
}
pub(in super::super) fn result() -> Typed {
    typed(
        bool_ty(),
        valid(
            appended::view(destination(), v("source")),
            canon(WORD, v("capacity")),
            new_live(),
        ),
        literal(true),
        nc("MakeAppendValid", base_args()),
    )
}
fn result_proof() -> Typed {
    typed(
        bool_ty(),
        result().left,
        literal(true),
        nc("AppendValid", append_arguments()),
    )
}
pub(in super::super) fn data(family: &str) -> Vec<(String, Expr)> {
    match family {
        "read" => indexed_data(),
        "borrowed-frame" => {
            let mut a = base_data();
            a.extend([
                ("index".into(), word_ty()),
                ("query_region".into(), bool_ty()),
                (
                    "borrowed_live".into(),
                    eq(
                        apply(v("live_mask"), vec![v("query_region")]),
                        literal(true),
                    ),
                ),
                (
                    "physical_index_guard".into(),
                    eq(
                        active(v("index"), canon(WORD, v("capacity"))),
                        literal(true),
                    ),
                ),
            ]);
            a
        }
        _ => base_data(),
    }
}
pub(in super::super) fn read_result(h: Expr) -> Typed {
    let mut a = vec![h.clone()];
    a.extend(indexed_data().iter().map(|(n, _)| v(n)));
    typed(
        element(),
        read(
            result_heap(h.clone()),
            appended::view(destination(), v("source")),
            numeric::addition(canon(WORD, length(destination())), canon(WORD, v("index"))).value,
        ),
        read(h, v("source"), v("index")),
        nc("MakeAppendRead", a),
    )
}
fn read_proof() -> Typed {
    let h = first_heap(v("heap"));
    let mut a = vec![h.clone()];
    a.extend(append_arguments());
    a.extend([v("index"), v("source_index_guard")]);
    let q = typed(
        element(),
        read_result(v("heap")).left,
        read(h, v("source"), v("index")),
        nc("AppendCopiedRead", a),
    );
    next(q, source_read(v("heap")))
}
pub(in super::super) fn nil_result() -> Typed {
    let d = destination();
    let s = v("source");
    next(
        appended::projection(d.clone(), s, 0),
        ctor_projection(
            vec![
                literal(false),
                v("fresh_region"),
                literal_word(WORD, 0),
                canon(WORD, v("length")),
                canon(WORD, v("capacity")),
            ],
            0,
        ),
    )
}
pub(in super::super) fn borrowed(h: Expr) -> Typed {
    let mut a = vec![h.clone()];
    a.extend(data("borrowed-frame").iter().map(|(n, _)| v(n)));
    typed(
        element(),
        read_heap(result_heap(h.clone()), v("query_region"), v("index")),
        read_heap(h, v("query_region"), v("index")),
        nc("MakeAppendBorrowedFrame", a),
    )
}
fn borrowed_proof(p: &mut Producer) -> Typed {
    let (h, d, s, r, i) = (
        v("heap"),
        destination(),
        v("source"),
        v("query_region"),
        v("index"),
    );
    let first = first_heap(h.clone());
    let qleft = read_heap(
        appended::updated(first.clone(), d.clone(), s.clone()),
        r.clone(),
        i.clone(),
    );
    // Reify the already checked physical interval definition at this allocation.
    let start = numeric::addition(
        canon(WORD, offset(d.clone())),
        canon(WORD, length(d.clone())),
    )
    .value;
    let stop = numeric::addition(
        canon(WORD, offset(d.clone())),
        appended::total(d.clone(), s.clone()).value,
    )
    .value;
    let key = canon(WORD, i.clone());
    let pred = and(
        word_eq(0, r.clone(), region(d.clone())),
        and(
            numeric::le(start.clone(), key.clone()),
            word_less(WORD, key.clone(), stop),
        ),
    );
    let rhs = choose_t(
        element(),
        pred.clone(),
        read(
            first.clone(),
            s.clone(),
            arith::difference(WORD, key, start),
        ),
        read_heap(first.clone(), r.clone(), i.clone()),
    );
    let mut q = typed(
        element(),
        qleft,
        rhs,
        nc(
            "AppendCellDefinition",
            vec![first, d.clone(), s, r.clone(), i.clone()],
        ),
    );
    let vals = vec![
        literal(false),
        v("fresh_region"),
        literal_word(WORD, 0),
        canon(WORD, v("length")),
        canon(WORD, v("capacity")),
    ];
    q = change(p, q, ctor_projection(vals, 1));
    let hn = apart(r.clone(), v("borrowed_live"));
    q = change(p, q, hn);
    let Expr::Apply(_, a) = &q.right else {
        panic!()
    };
    let n = normalize(a[3].clone(), p);
    q = change(p, q, n);
    let Expr::Apply(_, a) = &q.right else {
        panic!()
    };
    let target = a[1].clone();
    let left = q.right.clone();
    q = next(
        q,
        typed(
            element(),
            left,
            target.clone(),
            apply(c(REFL), vec![element(), target]),
        ),
    );
    next(q, fresh_at(h, r, i, v("borrowed_live")))
}
pub(super) fn install(p: &mut Producer, trace: Option<&Path>) {
    let q = source_valid_proof(p);
    theorem(
        p,
        "SliceAccessValidityResliceBulkConstructionMakeSourceValid",
        &base_data(),
        q,
    );
    let mut a = vec![("heap".into(), heap())];
    a.extend(indexed_data());
    let q = source_read_proof(p);
    theorem(
        p,
        "SliceAccessValidityResliceBulkConstructionMakeSourceRead",
        &a,
        q,
    );
    let q = result_proof();
    theorem(
        p,
        "SliceAccessValidityResliceBulkConstructionMakeAppendValid",
        &base_data(),
        q,
    );
    let q = read_proof();
    theorem(
        p,
        "SliceAccessValidityResliceBulkConstructionMakeAppendRead",
        &a,
        q,
    );
    save_trace(p, trace, "construct-composed-read");
    let mut a = vec![("heap".into(), heap())];
    a.extend(data("borrowed-frame"));
    let q = borrowed_proof(p);
    theorem(
        p,
        "SliceAccessValidityResliceBulkConstructionMakeAppendBorrowedFrame",
        &a,
        q,
    );
    save_trace(p, trace, "construct-composed-borrow");
}
