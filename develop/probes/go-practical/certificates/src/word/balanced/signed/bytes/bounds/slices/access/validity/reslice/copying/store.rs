//! Explicit immutable pre-state source heap, including overlapping ranges.
use super::*;

pub(super) fn bulk(h: Expr, d: Expr, s: Expr) -> Expr {
    bc("Heap", vec![h, d, s])
}
fn relative(d: Expr, q: Expr) -> Expr {
    arith::difference(WORD, q, canon(WORD, offset(d)))
}
fn member(d: Expr, s: Expr, r: Expr, q: Expr) -> Expr {
    and(
        word_eq(0, r, region(d.clone())),
        word_less(WORD, relative(d.clone(), q), counts::count(d, s)),
    )
}
fn cell_body(h: Expr, d: Expr, s: Expr, r: Expr, q: Expr) -> Expr {
    choose_t(
        element(),
        member(d.clone(), s.clone(), r.clone(), q.clone()),
        read(h.clone(), s, relative(d, q.clone())),
        apply(h, vec![r, q]),
    )
}
fn data() -> Vec<(String, Expr)> {
    vec![
        ("heap".into(), heap()),
        ("destination".into(), descriptor()),
        ("source".into(), descriptor()),
    ]
}
fn cell_def(h: Expr, d: Expr, s: Expr, r: Expr, i: Expr) -> Typed {
    let mut a = vec![h.clone(), d.clone(), s.clone(), r.clone(), i.clone()];
    typed(
        element(),
        read_heap(bulk(h.clone(), d.clone(), s.clone()), r.clone(), i.clone()),
        cell_body(h, d, s, r, canon(WORD, i)),
        bc("CellDefinition", std::mem::take(&mut a)),
    )
}
pub(super) fn read_canonical(h: Expr, d: Expr, i: Expr) -> Typed {
    typed(
        element(),
        read(h.clone(), d.clone(), canon(WORD, i.clone())),
        read(h.clone(), d.clone(), i.clone()),
        bc("CanonicalRead", vec![h, d, i]),
    )
}
fn read_canonical_proof(p: &mut Producer) -> Typed {
    let (h, d, i) = (v("heap"), v("destination"), v("index"));
    let mut q = read_definition(h.clone(), d.clone(), canon(WORD, i.clone()));
    q = change(p, q, address_def(offset(d.clone()), canon(WORD, i.clone())));
    q = change(p, q, canon_idem(WORD, i.clone()));
    let mut rhs = read_definition(h, d.clone(), i.clone());
    rhs = change(p, rhs, address_def(offset(d), i));
    next(q, backwards(rhs))
}
fn read_result(d: Expr, s: Expr) -> Expr {
    word_less(WORD, canon(WORD, v("index")), counts::count(d, s))
}
pub(super) fn read_law(h: Expr, d: Expr, s: Expr, i: Expr) -> Typed {
    typed(
        element(),
        read(bulk(h.clone(), d.clone(), s.clone()), d.clone(), i.clone()),
        choose_t(
            element(),
            word_less(
                WORD,
                canon(WORD, i.clone()),
                counts::count(d.clone(), s.clone()),
            ),
            read(h.clone(), s.clone(), i.clone()),
            read(h.clone(), d.clone(), i.clone()),
        ),
        bc("DestinationRead", vec![h, d, s, i]),
    )
}
fn read_proof(p: &mut Producer) -> Typed {
    let (h, d, s, i) = (v("heap"), v("destination"), v("source"), v("index"));
    let o = numeric::source(offset(d.clone()));
    let j = numeric::source(i.clone());
    let physical = sum(WORD, o.value.clone(), j.value.clone(), literal(false));
    let mut q = read_definition(bulk(h.clone(), d.clone(), s.clone()), d.clone(), i.clone());
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
        arith::canonical_sum(WORD, o.value.clone(), j.value.clone(), literal(false)),
    );
    q = change(p, q, shift::undo(o.clone(), j.clone()));
    q = change(p, q, refl_equal(0, region(d.clone())));
    let b = and(literal(true), read_result(d.clone(), s.clone()));
    let n = normalize(b, p);
    q = change(p, q, n);
    q = change(p, q, read_canonical(h.clone(), s, i.clone()));
    let mut old = read_definition(h, d.clone(), i.clone());
    old = change(p, old, address_def(offset(d), i));
    old = change(
        p,
        old,
        arith::canonical_sum(WORD, o.value, j.value, literal(false)),
    );
    assert_eq!(
        old.right,
        apply(v("heap"), vec![region(v("destination")), physical])
    );
    change(p, q, backwards(old))
}
pub(super) fn copied_read(h: Expr) -> Typed {
    let mut a = vec![h.clone()];
    a.extend(counts::access_data().iter().map(|(n, _)| v(n)));
    typed(
        element(),
        read(
            bulk(h.clone(), v("destination"), v("source")),
            v("destination"),
            v("index"),
        ),
        read(h, v("source"), v("index")),
        bc("CopiedRead", a),
    )
}
pub(super) fn frame_data() -> Vec<(String, Expr)> {
    let mut a = counts::access_data();
    a.pop();
    a.push((
        "destination_guard".into(),
        eq(defined(v("destination"), v("index")), literal(true)),
    ));
    a.push((
        "outside_copy".into(),
        eq(read_result(v("destination"), v("source")), literal(false)),
    ));
    a
}
pub(super) fn frame(h: Expr) -> Typed {
    let mut a = vec![h.clone()];
    a.extend(frame_data().iter().map(|(n, _)| v(n)));
    typed(
        element(),
        read(
            bulk(h.clone(), v("destination"), v("source")),
            v("destination"),
            v("index"),
        ),
        read(h, v("destination"), v("index")),
        bc("DestinationFrame", a),
    )
}
pub(super) fn other(h: Expr, d: Expr, s: Expr, i: Expr) -> Typed {
    typed(
        element(),
        read_heap(
            bulk(h.clone(), d.clone(), s.clone()),
            not(region(d.clone())),
            i.clone(),
        ),
        read_heap(h.clone(), not(region(d.clone())), i.clone()),
        bc("OtherRegionFrame", vec![h, d, s, i]),
    )
}
fn other_proof(p: &mut Producer) -> Typed {
    let (h, d, s, i) = (v("heap"), v("destination"), v("source"), v("index"));
    let mut q = cell_def(h, d.clone(), s, not(region(d.clone())), i);
    q = change(
        p,
        q,
        typed(
            bool_ty(),
            word_eq(0, not(region(d.clone())), region(d.clone())),
            literal(false),
            at("RegionApart", vec![region(d)]),
        ),
    );
    let Expr::Apply(_, args) = &q.right else {
        panic!()
    };
    let m = args[3].clone();
    let n = normalize(m, p);
    let q = change(p, q, n);
    let Expr::Apply(_, args) = &q.right else {
        panic!()
    };
    let rhs = args[1].clone();
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
    let body = lambda(
        "__bulk_region",
        bool_ty(),
        lambda(
            "__bulk_key",
            word_ty(),
            cell_body(
                v("heap"),
                v("destination"),
                v("source"),
                v("__bulk_region"),
                v("__bulk_key"),
            ),
        ),
    );
    function_definition(
        p,
        "SliceAccessValidityResliceBulkHeap",
        &data(),
        heap(),
        body,
    );
    let mut a = data();
    a.extend([("region".into(), bool_ty()), ("key".into(), word_ty())]);
    let mut q = cell_def(
        v("heap"),
        v("destination"),
        v("source"),
        v("region"),
        v("key"),
    );
    q.proof = apply(c(REFL), vec![element(), q.right.clone()]);
    theorem(p, "SliceAccessValidityResliceBulkCellDefinition", &a, q);
    let mut a = data();
    a.pop();
    a.push(("index".into(), word_ty()));
    let q = read_canonical_proof(p);
    theorem(p, "SliceAccessValidityResliceBulkCanonicalRead", &a, q);
    let mut a = data();
    a.push(("index".into(), word_ty()));
    let q = read_proof(p);
    theorem(p, "SliceAccessValidityResliceBulkDestinationRead", &a, q);
    save_trace(p, trace, "copy-store");
    let mut a = vec![("heap".into(), heap())];
    a.extend(counts::access_data());
    let mut q = read_law(v("heap"), v("destination"), v("source"), v("index"));
    let lt = conjunction(
        p,
        not(sign_bit(WORD, canon(WORD, v("index")))),
        read_result(v("destination"), v("source")),
        v("copied_guard"),
        true,
    );
    q = select(p, q, lt.left.clone(), true, lt.proof);
    theorem(p, "SliceAccessValidityResliceBulkCopiedRead", &a, q);
    let mut a = vec![("heap".into(), heap())];
    a.extend(frame_data());
    let q = read_law(v("heap"), v("destination"), v("source"), v("index"));
    let q = select(
        p,
        q,
        read_result(v("destination"), v("source")),
        false,
        v("outside_copy"),
    );
    theorem(p, "SliceAccessValidityResliceBulkDestinationFrame", &a, q);
    let mut a = data();
    a.push(("index".into(), word_ty()));
    let q = other_proof(p);
    theorem(p, "SliceAccessValidityResliceBulkOtherRegionFrame", &a, q);
    save_trace(p, trace, "copy-frame");
}
