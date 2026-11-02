//! Whole-cell result/source reads, unchanged saved views and complete frames.
use super::*;

fn physical(d: Expr, i: Expr) -> numeric::Word {
    numeric::addition(canon(WORD, offset(d)), canon(WORD, i))
}
fn interval(d: Expr, s: Expr, i: Expr) -> Expr {
    let q = physical(d.clone(), i).value;
    and(
        numeric::le(start(d.clone()).value, q.clone()),
        word_less(WORD, q, end(d, s).value),
    )
}
fn old_read(p: &mut Producer, h: Expr, d: Expr, i: Expr) -> Typed {
    let mut q = read_definition(h, d.clone(), i.clone());
    q = change(p, q, address_def(offset(d.clone()), i.clone()));
    change(
        p,
        q,
        arith::canonical_sum(WORD, canon(WORD, offset(d)), canon(WORD, i), literal(false)),
    )
}
fn view_to_old(p: &mut Producer, h: Expr, d: Expr, s: Expr, i: Expr, alias: bool) -> Typed {
    let target = if alias {
        full(d.clone())
    } else {
        view(d.clone(), s.clone())
    };
    let mut q = read_definition(h.clone(), target, i.clone());
    for f in [1, 2] {
        q = change(
            p,
            q,
            if alias {
                view_projection(d.clone(), f, true)
            } else {
                projection(d.clone(), s.clone(), f)
            },
        );
    }
    next(q, backwards(read_definition(h, d, i)))
}
fn read_law(h: Expr, d: Expr, s: Expr, i: Expr) -> Typed {
    let address = physical(d.clone(), i.clone()).value;
    let rhs = choose_t(
        element(),
        interval(d.clone(), s.clone(), i.clone()),
        read(
            h.clone(),
            s.clone(),
            arith::difference(WORD, address, start(d.clone()).value),
        ),
        read(h.clone(), d.clone(), i.clone()),
    );
    typed(
        element(),
        read(
            updated(h.clone(), d.clone(), s.clone()),
            view(d.clone(), s.clone()),
            i.clone(),
        ),
        rhs,
        nc("AppendRead", vec![h, d, s, i]),
    )
}
fn read_proof(p: &mut Producer) -> Typed {
    let (h, d, s, i) = (v("heap"), v("destination"), v("source"), v("index"));
    let mut q = read_definition(
        updated(h.clone(), d.clone(), s.clone()),
        view(d.clone(), s.clone()),
        i.clone(),
    );
    for f in [1, 2] {
        q = change(p, q, projection(d.clone(), s.clone(), f));
    }
    q = change(
        p,
        q,
        cell_def(
            h.clone(),
            d.clone(),
            s.clone(),
            region(d.clone()),
            address(offset(d.clone()), i.clone()),
        ),
    );
    q = change(p, q, address_def(offset(d.clone()), i.clone()));
    q = change(
        p,
        q,
        arith::canonical_sum(
            WORD,
            canon(WORD, offset(d.clone())),
            canon(WORD, i.clone()),
            literal(false),
        ),
    );
    q = change(p, q, refl_equal(0, region(d.clone())));
    let n = normalize(and(literal(true), interval(d.clone(), s, i.clone())), p);
    q = change(p, q, n);
    let old = old_read(p, h, d, i);
    change(p, q, backwards(old))
}
fn no_wrap(p: &mut Producer, hnil: Expr) -> [Typed; 3] {
    let d = v("destination");
    let s = v("source");
    let fs = nonnil_facts(
        p,
        d.clone(),
        v("storage_length"),
        v("live_mask"),
        v("destination_valid"),
        hnil,
    );
    let off = numeric::source(offset(d.clone()));
    let l = numeric::source(length(d.clone()));
    let src = numeric::source(length(s.clone()));
    let cap = numeric::source(capacity(d.clone()));
    let cap_carry = carry(WORD, off.value.clone(), cap.value.clone(), literal(false));
    let old_no = not_true(p, cap_carry, fs[6].proof.clone());
    let [sum_no, le] = guard_facts(p);
    let outer = numeric::bounded_add(
        off.clone(),
        total(d.clone(), s),
        cap,
        le.proof,
        old_no.proof.clone(),
        true,
    );
    let a = assoc::no_wrap(
        WORD,
        off.value.clone(),
        l.value.clone(),
        src.value.clone(),
        sum_no.proof,
        outer.proof,
    );
    let first = carry(WORD, off.value.clone(), l.value.clone(), literal(false));
    let second = carry(WORD, start(d).value, src.value, literal(false));
    let f = conjunction(
        p,
        not(first.clone()),
        not(second.clone()),
        a.proof.clone(),
        false,
    );
    let s = conjunction(p, not(first.clone()), not(second.clone()), a.proof, true);
    [
        not_true(p, first, f.proof),
        not_true(p, second, s.proof),
        old_no,
    ]
}
pub(super) fn appended_read(h: Expr, alias: bool) -> Typed {
    let mut a = vec![h.clone()];
    a.extend(indices::arguments());
    let d = if alias {
        full(v("destination"))
    } else {
        view(v("destination"), v("source"))
    };
    typed(
        element(),
        read(
            updated(h.clone(), v("destination"), v("source")),
            d,
            indices::mapped().value,
        ),
        read(h, v("source"), v("index")),
        nc(
            if alias {
                "AppendAliasRead"
            } else {
                "AppendCopiedRead"
            },
            a,
        ),
    )
}
fn appended_proof(p: &mut Producer) -> Typed {
    let (h, d, s, i) = (v("heap"), v("destination"), v("source"), v("index"));
    let m = indices::mapped();
    let hi = indices::index(false);
    let (hnil, _) = index_facts(p, view(d.clone(), s.clone()), m.value.clone(), hi.proof);
    let hnil = numeric::change_left(p, hnil, projection(d.clone(), s.clone(), 0));
    let [_, start_no, _] = no_wrap(p, hnil.proof);
    let (_, lt) = indices::source_facts(p);
    let st = start(d.clone());
    let ci = canon(WORD, i.clone());
    let len = canon(WORD, length(s.clone()));
    let mono = an(
        "Monotone",
        WORD,
        vec![
            st.value.clone(),
            ci.clone(),
            len.clone(),
            lt.proof,
            start_no.proof,
        ],
    );
    let phys = sum(WORD, st.value.clone(), ci.clone(), literal(false));
    let stop = sum(WORD, st.value.clone(), len.clone(), literal(false));
    let a = conjunction(
        p,
        not(carry(WORD, st.value.clone(), ci.clone(), literal(false))),
        word_less(WORD, phys.clone(), stop.clone()),
        mono.clone(),
        false,
    );
    let a = not_true(
        p,
        carry(WORD, st.value.clone(), ci.clone(), literal(false)),
        a.proof,
    );
    let b = conjunction(
        p,
        not(a.left.clone()),
        word_less(WORD, phys.clone(), stop),
        mono,
        true,
    );
    let lower = numeric::sum_lower(st.clone(), numeric::source(i.clone()), a.proof);
    let mut q = read_law(h.clone(), d.clone(), s.clone(), m.value.clone());
    q = change(p, q, scalar(&m));
    let zeros = vec![literal(false); 4];
    let off = canon(WORD, offset(d.clone()));
    let dl = canon(WORD, length(d.clone()));
    q = change(
        p,
        q,
        backwards(assoc::reassociate(
            WORD,
            off.clone(),
            dl.clone(),
            ci.clone(),
            &zeros,
            rc("AssocStart", vec![]),
        )),
    );
    q = change(
        p,
        q,
        backwards(assoc::reassociate(
            WORD,
            off,
            dl,
            len,
            &zeros,
            rc("AssocStart", vec![]),
        )),
    );
    for f in [lower, b] {
        q = change(p, q, f);
    }
    let n = normalize(and(literal(true), literal(true)), p);
    q = change(p, q, n);
    let Expr::Apply(_, a) = &q.right else {
        panic!()
    };
    let chosen = a[2].clone();
    let left = q.right.clone();
    q = next(
        q,
        typed(
            element(),
            left,
            chosen.clone(),
            apply(c(REFL), vec![element(), chosen]),
        ),
    );
    q = change(p, q, shift::undo(st, numeric::source(i.clone())));
    change(p, q, store::read_canonical(h, s, i))
}
pub(super) fn prefix_data() -> Vec<(String, Expr)> {
    let mut a = data();
    a.push(("index".into(), word_ty()));
    a.push((
        "destination_index_guard".into(),
        eq(defined(v("destination"), v("index")), literal(true)),
    ));
    a
}
pub(super) fn prefix(h: Expr) -> Typed {
    let mut a = vec![h.clone()];
    a.extend(prefix_data().iter().map(|(n, _)| v(n)));
    typed(
        element(),
        read(
            updated(h.clone(), v("destination"), v("source")),
            v("destination"),
            v("index"),
        ),
        read(h, v("destination"), v("index")),
        nc("AppendPrefixFrame", a),
    )
}
fn prefix_proof(p: &mut Producer) -> Typed {
    let (h, d, s, i) = (v("heap"), v("destination"), v("source"), v("index"));
    let (hnil, _) = index_facts(p, d.clone(), i.clone(), v("destination_index_guard"));
    let [off_len_no, _, _] = no_wrap(p, hnil.proof);
    let (_, lt) = indexed::active_facts(
        p,
        d.clone(),
        v("storage_length"),
        v("live_mask"),
        v("destination_valid"),
        i.clone(),
        v("destination_index_guard"),
    );
    let off = canon(WORD, offset(d.clone()));
    let ci = canon(WORD, i.clone());
    let len = canon(WORD, length(d.clone()));
    let phys = physical(d.clone(), i.clone()).value;
    let mono = an(
        "Monotone",
        WORD,
        vec![off.clone(), ci.clone(), len, lt.proof, off_len_no.proof],
    );
    let lt = conjunction(
        p,
        not(carry(WORD, off, ci, literal(false))),
        word_less(WORD, phys, start(d.clone()).value),
        mono,
        true,
    );
    let bridge = backwards(view_to_old(
        p,
        updated(h.clone(), d.clone(), s.clone()),
        d.clone(),
        s.clone(),
        i.clone(),
        false,
    ));
    let q = next(bridge, read_law(h, d.clone(), s.clone(), i.clone()));
    let cond = interval(d, s, i);
    let mut c = reflexive_t(bool_ty(), cond.clone());
    c = change(p, c, lt);
    let c = finish(p, c);
    select(p, q, cond, false, c.proof)
}
pub(super) fn tail_data() -> Vec<(String, Expr)> {
    let mut a = data();
    a.push(("index".into(), word_ty()));
    a.push((
        "full_index_guard".into(),
        eq(defined(full(v("destination")), v("index")), literal(true)),
    ));
    a.push((
        "outside_append".into(),
        eq(
            numeric::le(
                total(v("destination"), v("source")).value,
                canon(WORD, v("index")),
            ),
            literal(true),
        ),
    ));
    a
}
pub(super) fn tail(h: Expr) -> Typed {
    let mut a = vec![h.clone()];
    a.extend(tail_data().iter().map(|(n, _)| v(n)));
    typed(
        element(),
        read(
            updated(h.clone(), v("destination"), v("source")),
            full(v("destination")),
            v("index"),
        ),
        read(h, full(v("destination")), v("index")),
        nc("AppendTailFrame", a),
    )
}
fn tail_proof(p: &mut Producer) -> Typed {
    let (h, d, s, i) = (v("heap"), v("destination"), v("source"), v("index"));
    let full_valid = indexed::full_valid(
        d.clone(),
        v("storage_length"),
        v("live_mask"),
        v("destination_valid"),
    );
    let (_, lt) = indexed::active_facts(
        p,
        full(d.clone()),
        v("storage_length"),
        v("live_mask"),
        full_valid.proof,
        i.clone(),
        v("full_index_guard"),
    );
    let lt = numeric::change_left(p, lt, view_projection(d.clone(), 3, true));
    let (hnil, _) = index_facts(p, full(d.clone()), i.clone(), v("full_index_guard"));
    let hnil = numeric::change_left(p, hnil, view_projection(d.clone(), 0, true));
    let [_, _, cap_no] = no_wrap(p, hnil.proof);
    let off = numeric::source(offset(d.clone()));
    let ci = numeric::source(i.clone());
    let cap = numeric::source(capacity(d.clone()));
    let mono = an(
        "Monotone",
        WORD,
        vec![
            off.value.clone(),
            ci.value.clone(),
            cap.value,
            lt.proof,
            cap_no.proof,
        ],
    );
    let phys = physical(d.clone(), i.clone()).value;
    let a = conjunction(
        p,
        not(carry(
            WORD,
            off.value.clone(),
            ci.value.clone(),
            literal(false),
        )),
        word_less(
            WORD,
            phys.clone(),
            sum(
                WORD,
                off.value.clone(),
                canon(WORD, capacity(d.clone())),
                literal(false),
            ),
        ),
        mono,
        false,
    );
    let no = not_true(
        p,
        carry(WORD, off.value.clone(), ci.value.clone(), literal(false)),
        a.proof,
    );
    let le = numeric::bounded_add(
        off,
        total(d.clone(), s.clone()),
        ci,
        v("outside_append"),
        no.proof,
        false,
    );
    let lt = not_true(
        p,
        word_less(WORD, phys, end(d.clone(), s.clone()).value),
        le.proof,
    );
    let a = view_to_old(
        p,
        updated(h.clone(), d.clone(), s.clone()),
        d.clone(),
        s.clone(),
        i.clone(),
        true,
    );
    let b = backwards(view_to_old(
        p,
        updated(h.clone(), d.clone(), s.clone()),
        d.clone(),
        s.clone(),
        i.clone(),
        false,
    ));
    let q = next(
        next(a, b),
        read_law(h.clone(), d.clone(), s.clone(), i.clone()),
    );
    let cond = interval(d.clone(), s.clone(), i.clone());
    let mut c = reflexive_t(bool_ty(), cond.clone());
    c = change(p, c, lt);
    let c = finish(p, c);
    let q = select(p, q, cond, false, c.proof);
    next(q, backwards(view_to_old(p, h, d, s, i, true)))
}
pub(super) fn other_data() -> Vec<(String, Expr)> {
    let mut a = data();
    a.push(("index".into(), word_ty()));
    a
}
pub(super) fn other(h: Expr) -> Typed {
    let mut a = vec![h.clone()];
    a.extend(other_data().iter().map(|(n, _)| v(n)));
    typed(
        element(),
        read_heap(
            updated(h.clone(), v("destination"), v("source")),
            not(region(v("destination"))),
            v("index"),
        ),
        read_heap(h, not(region(v("destination"))), v("index")),
        nc("AppendOtherFrame", a),
    )
}
fn other_proof(p: &mut Producer) -> Typed {
    let mut q = cell_def(
        v("heap"),
        v("destination"),
        v("source"),
        not(region(v("destination"))),
        v("index"),
    );
    q = change(
        p,
        q,
        typed(
            bool_ty(),
            word_eq(0, not(region(v("destination"))), region(v("destination"))),
            literal(false),
            at("RegionApart", vec![region(v("destination"))]),
        ),
    );
    let Expr::Apply(_, a) = &q.right else {
        panic!()
    };
    let n = normalize(a[3].clone(), p);
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
pub(super) fn physical_data() -> Vec<(String, Expr)> {
    let mut a = data();
    a.extend([
        ("query_region".into(), bool_ty()),
        ("index".into(), word_ty()),
        (
            "query_live".into(),
            eq(
                apply(v("live_mask"), vec![v("query_region")]),
                literal(true),
            ),
        ),
        (
            "physical_index_guard".into(),
            eq(active(v("index"), v("storage_length")), literal(true)),
        ),
        (
            "outside_write".into(),
            eq(
                member(
                    v("destination"),
                    v("source"),
                    v("query_region"),
                    canon(WORD, v("index")),
                ),
                literal(false),
            ),
        ),
    ]);
    a
}
pub(super) fn physical_frame(h: Expr) -> Typed {
    let mut a = vec![h.clone()];
    a.extend(physical_data().iter().map(|(n, _)| v(n)));
    typed(
        element(),
        read_heap(
            updated(h.clone(), v("destination"), v("source")),
            v("query_region"),
            v("index"),
        ),
        read_heap(h, v("query_region"), v("index")),
        nc("AppendPhysicalFrame", a),
    )
}
fn physical_proof(p: &mut Producer) -> Typed {
    let q = cell_def(
        v("heap"),
        v("destination"),
        v("source"),
        v("query_region"),
        v("index"),
    );
    select(
        p,
        q,
        member(
            v("destination"),
            v("source"),
            v("query_region"),
            canon(WORD, v("index")),
        ),
        false,
        v("outside_write"),
    )
}
pub(super) fn install(p: &mut Producer, trace: Option<&Path>) {
    let mut a = heap_data();
    a.push(("index".into(), word_ty()));
    let q = read_proof(p);
    theorem(
        p,
        "SliceAccessValidityResliceBulkConstructionAppendRead",
        &a,
        q,
    );
    save_trace(p, trace, "construct-append-read-law");
    let mut a = vec![("heap".into(), heap())];
    a.extend(indices::data());
    let q = appended_proof(p);
    theorem(
        p,
        "SliceAccessValidityResliceBulkConstructionAppendCopiedRead",
        &a,
        q,
    );
    save_trace(p, trace, "construct-append-copied-read");
    let h = updated(v("heap"), v("destination"), v("source"));
    let m = indices::mapped().value;
    let left = view_to_old(p, h.clone(), v("destination"), v("source"), m.clone(), true);
    let right = view_to_old(p, h, v("destination"), v("source"), m, false);
    let q = next(
        next(left, backwards(right)),
        appended_read(v("heap"), false),
    );
    theorem(
        p,
        "SliceAccessValidityResliceBulkConstructionAppendAliasRead",
        &a,
        q,
    );
    let mut a = vec![("heap".into(), heap())];
    a.extend(prefix_data());
    let q = prefix_proof(p);
    theorem(
        p,
        "SliceAccessValidityResliceBulkConstructionAppendPrefixFrame",
        &a,
        q,
    );
    save_trace(p, trace, "construct-append-prefix");
    let mut a = vec![("heap".into(), heap())];
    a.extend(tail_data());
    let q = tail_proof(p);
    theorem(
        p,
        "SliceAccessValidityResliceBulkConstructionAppendTailFrame",
        &a,
        q,
    );
    save_trace(p, trace, "construct-append-tail");
    let mut a = vec![("heap".into(), heap())];
    a.extend(other_data());
    let q = other_proof(p);
    theorem(
        p,
        "SliceAccessValidityResliceBulkConstructionAppendOtherFrame",
        &a,
        q,
    );
    save_trace(p, trace, "construct-append-other");
    let mut a = vec![("heap".into(), heap())];
    a.extend(physical_data());
    let q = physical_proof(p);
    theorem(
        p,
        "SliceAccessValidityResliceBulkConstructionAppendPhysicalFrame",
        &a,
        q,
    );
    save_trace(p, trace, "construct-append-physical");
}
