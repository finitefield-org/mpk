//! Ordinary descriptor/keyed-store algebra. Absolute bounds/ownership stay due.
//! Logical region names are two Bool constructors, not source pointer bits.
use super::*;
mod access;
pub(super) fn generate_bulk_copy(root: &Path, output: &Path, capacity: usize, trace: bool) {
    access::generate_bulk_copy(root, output, capacity, trace);
}
pub(super) fn generate_reslice(root: &Path, output: &Path, capacity: usize, trace: bool) {
    access::generate_reslice(root, output, capacity, trace);
}
pub(super) fn generate_access(root: &Path, output: &Path, capacity: usize, trace: bool) {
    access::generate(root, output, capacity, trace);
}

const ELEMENT: usize = 7;
const FIELDS: usize = 3;
const CAPACITIES: [usize; 7] = [0, 1, 8, 32, 127, 128, 129];
fn element() -> Expr {
    cube_type(ELEMENT)
}
fn descriptor() -> Expr {
    cube_type(FIELDS + WORD)
}
fn heap() -> Expr {
    forall(
        "__slice_region_type",
        bool_ty(),
        forall("__slice_index_type", word_ty(), element()),
    )
}
fn cell_zero() -> Expr {
    cube_lambda(ELEMENT, "__slice_zero_", literal(false))
}
fn tag_word(d: usize, flag: Expr) -> Expr {
    if d == 0 {
        flag
    } else {
        join(d, tag_word(d - 1, flag), literal_word(d - 1, 0))
    }
}
fn at(suffix: &str, args: Vec<Expr>) -> Expr {
    apply(c(&name(&format!("Slice{suffix}"))), args)
}
fn field(d: Expr, n: usize) -> Expr {
    apply(
        d,
        (0..FIELDS)
            .rev()
            .map(|j| literal((n >> j) & 1 != 0))
            .collect(),
    )
}
fn nil(d: Expr) -> Expr {
    apply(field(d, 0), bit_address(0))
}
fn region(d: Expr) -> Expr {
    apply(field(d, 1), bit_address(0))
}
fn offset(d: Expr) -> Expr {
    field(d, 2)
}
fn length(d: Expr) -> Expr {
    field(d, 3)
}
fn capacity(d: Expr) -> Expr {
    field(d, 4)
}
fn ctor(n: Expr, r: Expr, o: Expr, l: Expr, k: Expr) -> Expr {
    at("Descriptor", vec![n, r, o, l, k])
}
fn copy(d: Expr) -> Expr {
    at("Copy", vec![d])
}
fn full(d: Expr) -> Expr {
    at("Full", vec![d])
}
fn address(o: Expr, i: Expr) -> Expr {
    at("Address", vec![o, i])
}
fn read_heap(h: Expr, r: Expr, i: Expr) -> Expr {
    apply(h, vec![r, canon(WORD, i)])
}
fn updated(h: Expr, r: Expr, i: Expr, value: Expr) -> Expr {
    at("HeapUpdate", vec![h, r, i, value])
}
fn snapshot(h: Expr, r: Expr) -> Expr {
    at("ArraySnapshot", vec![h, r])
}
fn snapshot_definition(h: Expr, r: Expr, i: Expr) -> Typed {
    let target = choose_t(
        element(),
        word_eq(0, not(r.clone()), not(r.clone())),
        read_heap(h.clone(), r.clone(), canon(WORD, i.clone())),
        read_heap(h.clone(), not(r.clone()), i.clone()),
    );
    typed(
        element(),
        read_heap(snapshot(h.clone(), r.clone()), not(r.clone()), i.clone()),
        target,
        at("ArraySnapshotReadDefinition", vec![h, r, i]),
    )
}
fn snapshot_read(h: Expr, r: Expr, i: Expr) -> Typed {
    typed(
        element(),
        read_heap(snapshot(h.clone(), r.clone()), not(r.clone()), i.clone()),
        read_heap(h.clone(), r.clone(), i.clone()),
        at("ArraySnapshotRead", vec![h, r, i]),
    )
}
fn read(h: Expr, d: Expr, i: Expr) -> Expr {
    at("Read", vec![h, d, i])
}
fn write(h: Expr, d: Expr, i: Expr, value: Expr) -> Expr {
    at("Write", vec![h, d, i, value])
}
fn signed_guard(l: Expr, i: Expr) -> Expr {
    let i = canon(WORD, i);
    let si = sign_bit(WORD, i.clone());
    and(
        not(si.clone()),
        choose(
            bool_eq(si.clone(), sign_bit(WORD, l.clone())),
            word_less(WORD, i, l),
            si,
        ),
    )
}
fn defined(d: Expr, i: Expr) -> Expr {
    at("Defined", vec![d, i])
}
fn match_key(r: Expr, key: Expr, wr: Expr, wkey: Expr) -> Expr {
    and(word_eq(0, r, wr), word_eq(WORD, key, wkey))
}
fn ctor_args() -> Vec<(String, Expr)> {
    vec![
        ("nil".into(), bool_ty()),
        ("region".into(), bool_ty()),
        ("offset".into(), word_ty()),
        ("length".into(), word_ty()),
        ("capacity".into(), word_ty()),
    ]
}
fn ctor_vars() -> Vec<Expr> {
    ctor_args().iter().map(|(n, _)| v(n)).collect()
}
fn accessor(d: Expr, f: usize) -> Expr {
    match f {
        0 => nil(d),
        1 => region(d),
        2 => offset(d),
        3 => length(d),
        _ => capacity(d),
    }
}
fn field_type(f: usize) -> Expr {
    if f < 2 {
        bool_ty()
    } else {
        word_ty()
    }
}
fn field_name(f: usize) -> &'static str {
    ["Nil", "Region", "Offset", "Length", "Capacity"][f]
}
fn tree_fields(d: usize, start: usize, fields: &[Expr]) -> Expr {
    if d == 0 {
        fields[start].clone()
    } else {
        let n = format!("__slice_field_{}", FIELDS - d);
        lambda(
            &n,
            bool_ty(),
            choose_t(
                addressed(d - 1, word_ty()),
                v(&n),
                tree_fields(d - 1, start + (1 << (d - 1)), fields),
                tree_fields(d - 1, start, fields),
            ),
        )
    }
}
fn ctor_projection(args: Vec<Expr>, f: usize) -> Typed {
    typed(
        field_type(f),
        accessor(at("Descriptor", args.clone()), f),
        args[f].clone(),
        at(&format!("Descriptor{}", field_name(f)), args),
    )
}
fn view_projection(d: Expr, f: usize, is_full: bool) -> Typed {
    let target = accessor(d.clone(), if is_full && f == 3 { 4 } else { f });
    typed(
        field_type(f),
        accessor(
            if is_full {
                full(d.clone())
            } else {
                copy(d.clone())
            },
            f,
        ),
        target,
        at(
            &format!("{}{}", if is_full { "Full" } else { "Copy" }, field_name(f)),
            vec![d],
        ),
    )
}
fn metadata(d: Expr, is_full: bool) -> Expr {
    (0..5).rev().fold(literal(true), |tail, f| {
        let a = accessor(
            if is_full {
                full(d.clone())
            } else {
                copy(d.clone())
            },
            f,
        );
        let b = accessor(d.clone(), if is_full && f == 3 { 4 } else { f });
        and(word_eq(if f < 2 { 0 } else { WORD }, a, b), tail)
    })
}
fn metadata_proof(p: &mut Producer, d: Expr, is_full: bool) -> Typed {
    let mut e = reflexive_t(bool_ty(), metadata(d.clone(), is_full));
    for f in 0..5 {
        e = change(p, e, view_projection(d.clone(), f, is_full));
        e = change(
            p,
            e,
            refl_equal(
                if f < 2 { 0 } else { WORD },
                accessor(d.clone(), if is_full && f == 3 { 4 } else { f }),
            ),
        );
    }
    let q = normalize(e.right.clone(), p);
    next(e, q)
}
fn read_view(h: Expr, args: Vec<Expr>, i: Expr) -> Typed {
    let target = read_heap(
        h.clone(),
        args[1].clone(),
        address(args[2].clone(), i.clone()),
    );
    let mut call = vec![h.clone()];
    call.extend(args.clone());
    call.push(i.clone());
    typed(
        element(),
        read(h, at("Descriptor", args), i),
        target.clone(),
        at("CtorRead", call),
    )
}
fn write_view(h: Expr, args: Vec<Expr>, i: Expr, value: Expr) -> Typed {
    let target = updated(
        h.clone(),
        args[1].clone(),
        address(args[2].clone(), i.clone()),
        value.clone(),
    );
    let mut call = vec![h.clone()];
    call.extend(args.clone());
    call.extend([i.clone(), value.clone()]);
    typed(
        heap(),
        write(h, at("Descriptor", args), i, value),
        target.clone(),
        at("CtorWrite", call),
    )
}
fn updated_read(h: Expr, r: Expr, i: Expr, value: Expr, rr: Expr, ri: Expr) -> Typed {
    let target = choose_t(
        element(),
        match_key(
            rr.clone(),
            canon(WORD, ri.clone()),
            r.clone(),
            canon(WORD, i.clone()),
        ),
        value.clone(),
        read_heap(h.clone(), rr.clone(), ri.clone()),
    );
    typed(
        element(),
        read_heap(
            updated(h.clone(), r.clone(), i.clone(), value.clone()),
            rr.clone(),
            ri.clone(),
        ),
        target,
        at("UpdatedRead", vec![h, r, i, value, rr, ri]),
    )
}
fn equal_match_proof(p: &mut Producer, r: Expr, i: Expr) -> Typed {
    let mut e = reflexive_t(
        bool_ty(),
        match_key(
            r.clone(),
            canon(WORD, i.clone()),
            r.clone(),
            canon(WORD, i.clone()),
        ),
    );
    e = change(p, e, refl_equal(0, r));
    e = change(p, e, refl_equal(WORD, canon(WORD, i)));
    let q = normalize(e.right.clone(), p);
    next(e, q)
}
fn different_match_proof(p: &mut Producer, r: Expr, wi: Expr, ri: Expr) -> Typed {
    let mut e = reflexive_t(
        bool_ty(),
        match_key(not(r.clone()), canon(WORD, ri), r.clone(), canon(WORD, wi)),
    );
    let q = typed(
        bool_ty(),
        word_eq(0, not(r.clone()), r.clone()),
        literal(false),
        at("RegionApart", vec![r]),
    );
    e = change(p, e, q);
    let q = normalize(e.right.clone(), p);
    next(e, q)
}
fn equal_match(r: Expr, i: Expr) -> Typed {
    typed(
        bool_ty(),
        match_key(
            r.clone(),
            canon(WORD, i.clone()),
            r.clone(),
            canon(WORD, i.clone()),
        ),
        literal(true),
        at("EqualMatch", vec![r, i]),
    )
}
fn different_match(r: Expr, wi: Expr, ri: Expr) -> Typed {
    typed(
        bool_ty(),
        match_key(
            not(r.clone()),
            canon(WORD, ri.clone()),
            r.clone(),
            canon(WORD, wi.clone()),
        ),
        literal(false),
        at("DifferentMatch", vec![r, wi, ri]),
    )
}
fn read_definition(h: Expr, d: Expr, i: Expr) -> Typed {
    typed(
        element(),
        read(h.clone(), d.clone(), i.clone()),
        read_heap(
            h.clone(),
            region(d.clone()),
            address(offset(d.clone()), i.clone()),
        ),
        at("ReadDefinition", vec![h, d, i]),
    )
}
fn write_definition(h: Expr, d: Expr, i: Expr, value: Expr) -> Typed {
    typed(
        heap(),
        write(h.clone(), d.clone(), i.clone(), value.clone()),
        updated(
            h.clone(),
            region(d.clone()),
            address(offset(d.clone()), i.clone()),
            value.clone(),
        ),
        at("WriteDefinition", vec![h, d, i, value]),
    )
}
fn view(r: Expr, o: Expr, l: Expr, k: Expr) -> Expr {
    ctor(literal(false), r, o, l, k)
}
fn view_args(r: Expr, o: Expr, l: Expr, k: Expr) -> Vec<Expr> {
    vec![literal(false), r, o, l, k]
}
fn alias_args() -> Vec<(String, Expr)> {
    vec![
        ("heap".into(), heap()),
        ("region".into(), bool_ty()),
        ("offset".into(), word_ty()),
        ("index".into(), word_ty()),
        ("value".into(), element()),
        ("length_a".into(), word_ty()),
        ("capacity_a".into(), word_ty()),
        ("length_b".into(), word_ty()),
        ("capacity_b".into(), word_ty()),
    ]
}
fn alias_views() -> (Vec<Expr>, Vec<Expr>) {
    (
        view_args(v("region"), v("offset"), v("length_a"), v("capacity_a")),
        view_args(v("region"), v("offset"), v("length_b"), v("capacity_b")),
    )
}
fn alias_result() -> (Expr, Expr) {
    let (a, b) = alias_views();
    (
        read(
            write(v("heap"), at("Descriptor", a), v("index"), v("value")),
            at("Descriptor", b),
            v("index"),
        ),
        v("value"),
    )
}
fn alias_proof(p: &mut Producer) -> Typed {
    let (a, b) = alias_views();
    let post = write(
        v("heap"),
        at("Descriptor", a.clone()),
        v("index"),
        v("value"),
    );
    let mut e = read_view(post, b, v("index"));
    e = change(p, e, write_view(v("heap"), a, v("index"), v("value")));
    let addr = address(v("offset"), v("index"));
    e = next(
        e,
        updated_read(
            v("heap"),
            v("region"),
            addr.clone(),
            v("value"),
            v("region"),
            addr.clone(),
        ),
    );
    let h = equal_match(v("region"), addr);
    select(p, e, h.left, true, h.proof)
}
fn frame_args() -> Vec<(String, Expr)> {
    vec![
        ("heap".into(), heap()),
        ("region".into(), bool_ty()),
        ("offset".into(), word_ty()),
        ("index".into(), word_ty()),
        ("value".into(), element()),
        ("other_offset".into(), word_ty()),
        ("other_index".into(), word_ty()),
        ("length".into(), word_ty()),
        ("capacity".into(), word_ty()),
    ]
}
fn frame_views() -> (Vec<Expr>, Vec<Expr>) {
    (
        view_args(v("region"), v("offset"), v("length"), v("capacity")),
        view_args(
            not(v("region")),
            v("other_offset"),
            v("length"),
            v("capacity"),
        ),
    )
}
fn frame_result() -> (Expr, Expr) {
    let (a, b) = frame_views();
    (
        read(
            write(v("heap"), at("Descriptor", a), v("index"), v("value")),
            at("Descriptor", b.clone()),
            v("other_index"),
        ),
        read(v("heap"), at("Descriptor", b), v("other_index")),
    )
}
fn frame_proof(p: &mut Producer) -> Typed {
    let (a, b) = frame_views();
    let post = write(
        v("heap"),
        at("Descriptor", a.clone()),
        v("index"),
        v("value"),
    );
    let mut e = read_view(post, b.clone(), v("other_index"));
    e = change(p, e, write_view(v("heap"), a, v("index"), v("value")));
    let wi = address(v("offset"), v("index"));
    let ri = address(v("other_offset"), v("other_index"));
    e = next(
        e,
        updated_read(
            v("heap"),
            v("region"),
            wi.clone(),
            v("value"),
            not(v("region")),
            ri.clone(),
        ),
    );
    let h = different_match(v("region"), wi, ri);
    e = select(p, e, h.left, false, h.proof);
    next(e, backwards(read_view(v("heap"), b, v("other_index"))))
}
fn array_result() -> (Expr, Expr) {
    let (a, b) = frame_views();
    let old = view(v("region"), v("other_offset"), v("length"), v("capacity"));
    (
        read(
            write(
                snapshot(v("heap"), v("region")),
                at("Descriptor", a),
                v("index"),
                v("value"),
            ),
            at("Descriptor", b),
            v("other_index"),
        ),
        read(v("heap"), old, v("other_index")),
    )
}
fn array_proof() -> Typed {
    let ah = snapshot(v("heap"), v("region"));
    let (l, r) = frame_result();
    let e = typed(
        element(),
        substitute(&l, &v("heap"), &ah),
        substitute(&r, &v("heap"), &ah),
        at("OtherRegionFrame", call_args(&frame_args(), ah.clone())),
    );
    let (_, b) = frame_views();
    let e = next(e, read_view(ah, b, v("other_index")));
    let e = next(
        e,
        snapshot_read(
            v("heap"),
            v("region"),
            address(v("other_offset"), v("other_index")),
        ),
    );
    next(
        e,
        backwards(read_view(
            v("heap"),
            view_args(v("region"), v("other_offset"), v("length"), v("capacity")),
            v("other_index"),
        )),
    )
}
fn nil_empty(r: Expr, k: Expr) -> Expr {
    let z = literal_word(WORD, 0);
    let a = ctor(literal(true), r.clone(), z.clone(), z.clone(), z.clone());
    let b = view(r, z.clone(), z, k);
    and(
        word_eq(WORD, length(a.clone()), length(b.clone())),
        not(word_eq(0, nil(a), nil(b))),
    )
}
fn nil_empty_proof(p: &mut Producer) -> Typed {
    let z = literal_word(WORD, 0);
    let a = vec![literal(true), v("region"), z.clone(), z.clone(), z.clone()];
    let b = view_args(v("region"), z.clone(), z.clone(), v("capacity"));
    let mut e = reflexive_t(bool_ty(), nil_empty(v("region"), v("capacity")));
    for args in [a, b] {
        for f in [0, 3] {
            e = change(p, e, ctor_projection(args.clone(), f));
        }
    }
    e = change(p, e, refl_equal(WORD, z));
    let q = definition(true, 0, literal(true), literal(false));
    e = change(p, e, q);
    let q = normalize(e.right.clone(), p);
    next(e, q)
}
fn definitions(p: &mut Producer, trace: Option<&Path>) {
    // Actual modular source-width offset addition; no uninterpreted address map.
    for d in 0..=WORD {
        let args = vec![
            ("x".into(), cube_type(d)),
            ("y".into(), cube_type(d)),
            ("incoming".into(), bool_ty()),
        ];
        let (s, carry_body) = if d == 0 {
            let (s, c) = adder(&[v("x")], &[v("y")], v("incoming"));
            (s[0].clone(), c)
        } else {
            let (lo, hi, c) = half_results(d, v("x"), v("y"), v("incoming"));
            (join(d, lo, hi), c)
        };
        function_definition(p, &format!("TreeSum{d}"), &args, cube_type(d), s);
        function_definition(p, &format!("TreeCarry{d}"), &args, bool_ty(), carry_body);
    }
    function_definition(
        p,
        "SliceAddress",
        &[("offset".into(), word_ty()), ("index".into(), word_ty())],
        word_ty(),
        sum(
            WORD,
            canon(WORD, v("offset")),
            canon(WORD, v("index")),
            literal(false),
        ),
    );
    let fields = vec![
        tag_word(WORD, v("nil")),
        tag_word(WORD, v("region")),
        v("offset"),
        v("length"),
        v("capacity"),
        literal_word(WORD, 0),
        literal_word(WORD, 0),
        literal_word(WORD, 0),
    ];
    function_definition(
        p,
        "SliceDescriptor",
        &ctor_args(),
        descriptor(),
        tree_fields(FIELDS, 0, &fields),
    );
    let cv = ctor_vars();
    for f in 0..5 {
        let mut e = ctor_projection(cv.clone(), f);
        e.proof = apply(c(REFL), vec![e.ty.clone(), e.right.clone()]);
        theorem(
            p,
            &format!("SliceDescriptor{}", field_name(f)),
            &ctor_args(),
            e,
        );
    }
    save_trace(p, trace, "slice-constructor");
    let d = v("descriptor");
    let da = vec![("descriptor".into(), descriptor())];
    for is_full in [false, true] {
        let label = if is_full { "Full" } else { "Copy" };
        let fields = (0..5)
            .map(|f| accessor(d.clone(), if is_full && f == 3 { 4 } else { f }))
            .collect();
        function_definition(
            p,
            &format!("Slice{label}"),
            &da,
            descriptor(),
            at("Descriptor", fields),
        );
        for f in 0..5 {
            let mut e = view_projection(d.clone(), f, is_full);
            e.proof = apply(c(REFL), vec![e.ty.clone(), e.right.clone()]);
            theorem(p, &format!("Slice{label}{}", field_name(f)), &da, e);
        }
        let e = metadata_proof(p, d.clone(), is_full);
        theorem(p, &format!("Slice{label}Metadata"), &da, e);
    }
    save_trace(p, trace, "slice-views");
    let h = v("heap");
    let r = v("region");
    let i = v("index");
    let value = v("value");
    let ua = vec![
        ("heap".into(), heap()),
        ("region".into(), bool_ty()),
        ("index".into(), word_ty()),
        ("value".into(), element()),
    ];
    let body = lambda(
        "__slice_update_region",
        bool_ty(),
        lambda(
            "__slice_update_index",
            word_ty(),
            choose_t(
                element(),
                match_key(
                    v("__slice_update_region"),
                    v("__slice_update_index"),
                    r.clone(),
                    canon(WORD, i.clone()),
                ),
                value.clone(),
                apply(
                    h.clone(),
                    vec![v("__slice_update_region"), v("__slice_update_index")],
                ),
            ),
        ),
    );
    function_definition(p, "SliceHeapUpdate", &ua, heap(), body);
    let mut ura = ua;
    ura.extend([
        ("read_region".into(), bool_ty()),
        ("read_index".into(), word_ty()),
    ]);
    let mut e = updated_read(
        h.clone(),
        r.clone(),
        i.clone(),
        value.clone(),
        v("read_region"),
        v("read_index"),
    );
    e.proof = apply(c(REFL), vec![e.ty.clone(), e.right.clone()]);
    theorem(p, "SliceUpdatedRead", &ura, e);
    let rw = vec![
        ("heap".into(), heap()),
        ("descriptor".into(), descriptor()),
        ("index".into(), word_ty()),
    ];
    function_definition(
        p,
        "SliceRead",
        &rw,
        element(),
        read_heap(
            h.clone(),
            region(d.clone()),
            address(offset(d.clone()), i.clone()),
        ),
    );
    let mut ww = rw.clone();
    ww.push(("value".into(), element()));
    function_definition(
        p,
        "SliceWrite",
        &ww,
        heap(),
        updated(
            h.clone(),
            region(d.clone()),
            address(offset(d.clone()), i.clone()),
            value,
        ),
    );
    function_definition(
        p,
        "SliceDefined",
        &[
            ("descriptor".into(), descriptor()),
            ("index".into(), word_ty()),
        ],
        bool_ty(),
        and(
            not(nil(d.clone())),
            signed_guard(length(d.clone()), i.clone()),
        ),
    );
    let mut e = read_definition(h.clone(), d.clone(), i.clone());
    e.proof = apply(c(REFL), vec![e.ty.clone(), e.right.clone()]);
    theorem(p, "SliceReadDefinition", &rw, e);
    let mut e = write_definition(h.clone(), d.clone(), i.clone(), v("value"));
    e.proof = apply(c(REFL), vec![e.ty.clone(), e.right.clone()]);
    theorem(p, "SliceWriteDefinition", &ww, e);
    let mut ra = vec![("heap".into(), heap())];
    ra.extend(ctor_args());
    ra.push(("index".into(), word_ty()));
    let cv = ctor_vars();
    let mut e = read_definition(h.clone(), at("Descriptor", cv.clone()), i.clone());
    for f in [1, 2] {
        e = change(p, e, ctor_projection(cv.clone(), f));
    }
    theorem(p, "SliceCtorRead", &ra, e);
    let mut wa = ra;
    wa.push(("value".into(), element()));
    let mut e = write_definition(
        h.clone(),
        at("Descriptor", cv.clone()),
        i.clone(),
        v("value"),
    );
    for f in [1, 2] {
        e = change(p, e, ctor_projection(cv.clone(), f));
    }
    theorem(p, "SliceCtorWrite", &wa, e);
    save_trace(p, trace, "slice-ctor-read-write");
    let no = word_eq(0, not(r.clone()), r.clone());
    let mut e = definition(true, 0, not(r.clone()), r.clone());
    let proof = prove_with_checked_normalization(
        &eq(e.right.clone(), literal(false)),
        std::slice::from_ref(&r),
        &mut p.fresh,
    );
    e = next(
        e,
        typed(
            bool_ty(),
            bool_eq(not(r.clone()), r.clone()),
            literal(false),
            proof,
        ),
    );
    assert_eq!(e.left, no);
    theorem(p, "SliceRegionApart", &[("region".into(), bool_ty())], e);
    let e = equal_match_proof(p, r.clone(), i.clone());
    theorem(
        p,
        "SliceEqualMatch",
        &[("region".into(), bool_ty()), ("index".into(), word_ty())],
        e,
    );
    let e = different_match_proof(p, r.clone(), i.clone(), v("read_index"));
    theorem(
        p,
        "SliceDifferentMatch",
        &[
            ("region".into(), bool_ty()),
            ("index".into(), word_ty()),
            ("read_index".into(), word_ty()),
        ],
        e,
    );
    save_trace(p, trace, "slice-store");
    for is_full in [false, true] {
        let left = read(
            h.clone(),
            if is_full {
                full(d.clone())
            } else {
                copy(d.clone())
            },
            i.clone(),
        );
        let view = if is_full {
            full(d.clone())
        } else {
            copy(d.clone())
        };
        let mut e = read_definition(h.clone(), view, i.clone());
        for f in [1, 2] {
            e = change(p, e, view_projection(d.clone(), f, is_full));
        }
        e = next(
            e,
            backwards(read_definition(h.clone(), d.clone(), i.clone())),
        );
        assert_eq!(e.left, left);
        theorem(
            p,
            if is_full {
                "SliceReadFull"
            } else {
                "SliceReadCopy"
            },
            &rw,
            e,
        );
    }
    save_trace(p, trace, "slice-read-views");
    let e = alias_proof(p);
    theorem(p, "SliceAliasWriteRead", &alias_args(), e);
    save_trace(p, trace, "slice-alias");
    let e = frame_proof(p);
    theorem(p, "SliceOtherRegionFrame", &frame_args(), e);
    save_trace(p, trace, "slice-frame");
    let sa = vec![("heap".into(), heap()), ("region".into(), bool_ty())];
    let body = lambda(
        "__slice_snapshot_region",
        bool_ty(),
        lambda(
            "__slice_snapshot_index",
            word_ty(),
            choose_t(
                element(),
                word_eq(0, v("__slice_snapshot_region"), not(r.clone())),
                read_heap(h.clone(), r.clone(), v("__slice_snapshot_index")),
                apply(
                    h.clone(),
                    vec![v("__slice_snapshot_region"), v("__slice_snapshot_index")],
                ),
            ),
        ),
    );
    function_definition(p, "SliceArraySnapshot", &sa, heap(), body);
    let mut sa = sa;
    sa.push(("index".into(), word_ty()));
    let mut e = snapshot_definition(h.clone(), r.clone(), i.clone());
    e.proof = apply(c(REFL), vec![e.ty.clone(), e.right.clone()]);
    theorem(p, "SliceArraySnapshotReadDefinition", &sa, e);
    let e = snapshot_definition(h.clone(), r.clone(), i.clone());
    let yes = refl_equal(0, not(r.clone()));
    let e = select(p, e, yes.left, true, yes.proof);
    let e = change(p, e, canon_idem(WORD, i.clone()));
    theorem(p, "SliceArraySnapshotRead", &sa, e);
    let e = array_proof();
    let (l, r) = array_result();
    assert_eq!(e.left, l);
    assert_eq!(e.right, r);
    theorem(p, "SliceArrayValueCopy", &frame_args(), e);
    save_trace(p, trace, "slice-array-copy");
    let e = nil_empty_proof(p);
    theorem(
        p,
        "SliceNilEmpty",
        &[("region".into(), bool_ty()), ("capacity".into(), word_ty())],
        e,
    );
    save_trace(p, trace, "slice-root");
}
fn finite_heap(d: usize, storage: Expr) -> Expr {
    let mut args = vec![v("__slice_lift_region")];
    args.extend(low_bits(v("__slice_lift_index"), d));
    lambda(
        "__slice_lift_region",
        bool_ty(),
        lambda("__slice_lift_index", word_ty(), apply(storage, args)),
    )
}
fn call_args(args: &[(String, Expr)], backing: Expr) -> Vec<Expr> {
    args.iter()
        .map(|(n, _)| if n == "heap" { backing.clone() } else { v(n) })
        .collect()
}
fn emit(base: Certificate, output: &Path, count: usize) {
    let d = capacity_depth(count);
    let h = finite_heap(d, v("backing"));
    let mut cases = vec![];
    for family in [
        "descriptor-copy",
        "full-reslice",
        "nil-empty",
        "same-key-alias",
        "other-region-frame",
        "array-value-copy",
        "tail-exposure",
    ] {
        if count == 0
            && matches!(
                family,
                "same-key-alias" | "other-region-frame" | "array-value-copy" | "tail-exposure"
            )
        {
            continue;
        }
        for mutation in 0..2 {
            let mut p = Producer {
                builder: Builder::from(base.clone()),
                fresh: 0,
            };
            let mut args = vec![("backing".into(), cube_type(1 + d + ELEMENT))];
            let mut e = match family {
                "descriptor-copy" | "full-reslice" => {
                    args.push(("descriptor".into(), descriptor()));
                    let is_full = family == "full-reslice";
                    typed(
                        bool_ty(),
                        metadata(v("descriptor"), is_full),
                        literal(true),
                        at(
                            if is_full {
                                "FullMetadata"
                            } else {
                                "CopyMetadata"
                            },
                            vec![v("descriptor")],
                        ),
                    )
                }
                "nil-empty" => {
                    args.push(("region".into(), bool_ty()));
                    typed(
                        bool_ty(),
                        nil_empty(v("region"), literal_word(WORD, count as u64)),
                        literal(true),
                        at(
                            "NilEmpty",
                            vec![v("region"), literal_word(WORD, count as u64)],
                        ),
                    )
                }
                "same-key-alias" => {
                    let aa = alias_args();
                    args.extend(aa.iter().skip(1).cloned());
                    let (a, b) = alias_views();
                    for (label, desc) in [
                        ("write_defined", at("Descriptor", a)),
                        ("read_defined", at("Descriptor", b)),
                    ] {
                        args.push((label.into(), eq(defined(desc, v("index")), literal(true))));
                    }
                    let (l, r) = alias_result();
                    typed(
                        element(),
                        substitute(&l, &v("heap"), &h),
                        r,
                        at("AliasWriteRead", call_args(&aa, h.clone())),
                    )
                }
                "other-region-frame" | "array-value-copy" => {
                    let aa = frame_args();
                    args.extend(aa.iter().skip(1).cloned());
                    let (a, b) = frame_views();
                    for (label, desc, index) in [
                        ("write_defined", at("Descriptor", a), v("index")),
                        ("read_defined", at("Descriptor", b), v("other_index")),
                    ] {
                        args.push((label.into(), eq(defined(desc, index), literal(true))));
                    }
                    let (l, r) = if family == "array-value-copy" {
                        array_result()
                    } else {
                        frame_result()
                    };
                    typed(
                        element(),
                        substitute(&l, &v("heap"), &h),
                        substitute(&r, &v("heap"), &h),
                        at(
                            if family == "array-value-copy" {
                                "ArrayValueCopy"
                            } else {
                                "OtherRegionFrame"
                            },
                            call_args(&aa, h.clone()),
                        ),
                    )
                }
                _ => {
                    args.extend([
                        ("region".into(), bool_ty()),
                        ("offset".into(), word_ty()),
                        ("index".into(), word_ty()),
                    ]);
                    let cv = view_args(
                        v("region"),
                        v("offset"),
                        literal_word(WORD, 0),
                        literal_word(WORD, count as u64),
                    );
                    let desc = at("Descriptor", cv.clone());
                    args.push((
                        "read_defined".into(),
                        eq(defined(full(desc.clone()), v("index")), literal(true)),
                    ));
                    let a = typed(
                        element(),
                        read(h.clone(), full(desc.clone()), v("index")),
                        read(h.clone(), desc.clone(), v("index")),
                        at("ReadFull", vec![h.clone(), desc, v("index")]),
                    );
                    let b = read_view(h.clone(), cv, v("index"));
                    next(a, b)
                }
            };
            if mutation != 0 {
                match family {
                    "descriptor-copy" | "full-reslice" | "nil-empty" => e.right = literal(false),
                    "same-key-alias" => {
                        e.left = substitute(
                            &e.left,
                            &at("Descriptor", alias_views().1),
                            &view(
                                not(v("region")),
                                v("offset"),
                                v("length_b"),
                                v("capacity_b"),
                            ),
                        );
                    }
                    "other-region-frame" | "array-value-copy" => {
                        let old = at("Descriptor", frame_views().1);
                        let wrong =
                            view(v("region"), v("other_offset"), v("length"), v("capacity"));
                        e.left = substitute(&e.left, &old, &wrong);
                        e.right = substitute(&e.right, &old, &wrong);
                    }
                    _ => e.right = cell_zero(),
                }
            }
            let suffix = format!("SliceClaim{}", family.replace('-', ""));
            let ty = eq_ty(e.ty.clone(), e.left, e.right);
            let (ty, proof) = fold_binders(&args, ty, e.proof);
            p.declare(&suffix, ty.clone(), proof, true);
            let label = if mutation == 0 {
                "valid"
            } else {
                match family {
                    "same-key-alias" => "fresh-id-per-view",
                    "other-region-frame" => "erased-region-separation",
                    "array-value-copy" => "shared-copy-region",
                    "tail-exposure" => "zeroed-tail",
                    _ => "altered-result",
                }
            };
            let id = format!("slice-store-{count}-{family}-{label}");
            let terms = p.builder.c.term_table.len();
            let declarations = p.builder.c.declarations.len();
            let bytes = p.builder.finish();
            fs::write(output.join(format!("{id}.mpcert")), &bytes).unwrap();
            cases.push(json!({"id":id,"family":family,"count":count,"capacity":count,"mutation":mutation,"expected":if mutation==0 {"accepted"} else {"rejected"},"source_word_bits":64,"descriptor_bits":512,"element_bits":128,"logical_region_count":2,"source_pointer_bits_admitted":false,"address_depth":d,"quantified_storage_bits":128usize<<(d+1),"declarations":declarations,"terms":terms,"bytes":bytes.len(),"theorem":name(&suffix),"scope":"ordinary descriptor/keyed-store and array-snapshot algebra over complete finite backing; absolute-bound/validity/ownership and production Go operation binding remain pending","goal_debug":format!("{ty:?}")}));
        }
    }
    fs::write(
        output.join("index.json"),
        serde_json::to_vec_pretty(
            &json!({"schema":"mpk.go_practical.slice_store_pilot.v0","cases":cases}),
        )
        .unwrap(),
    )
    .unwrap();
}
fn capacity_depth(count: usize) -> usize {
    if count <= 1 {
        0
    } else {
        usize::BITS as usize - (count - 1).leading_zeros() as usize
    }
}
pub(super) fn generate(root: &Path, output: &Path, count: usize, trace: bool) {
    assert!(CAPACITIES.contains(&count));
    fs::create_dir_all(output).unwrap();
    let mut p = Producer {
        builder: Builder::from(natural::foundation(root)),
        fresh: 0,
    };
    p.builder.c.module = format!("{PREFIX}.SliceStoreCapacity{count}");
    primitives(&mut p);
    for d in 0..=WORD {
        word_library(&mut p, d);
        build(&mut p, d, trace.then_some(output));
    }
    definitions(&mut p, trace.then_some(output));
    if trace {
        fs::write(
            output.join("library.mpcert"),
            Builder::from(p.builder.c.clone()).finish(),
        )
        .unwrap();
    }
    emit(p.builder.c, output, count);
}
