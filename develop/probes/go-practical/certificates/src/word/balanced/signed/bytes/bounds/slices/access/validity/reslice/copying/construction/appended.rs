//! Exact non-reallocating result, physical write interval and source snapshot.
use super::*;
mod indices;
mod reads;
mod validity;

pub(super) fn total(d: Expr, s: Expr) -> numeric::Word {
    numeric::addition(canon(WORD, length(d)), canon(WORD, length(s)))
}
pub(super) fn guard(d: Expr, s: Expr) -> Expr {
    let l = canon(WORD, length(d.clone()));
    let n = canon(WORD, length(s));
    both(&[
        not(carry(WORD, l.clone(), n.clone(), literal(false))),
        numeric::le(sum(WORD, l, n, literal(false)), canon(WORD, capacity(d))),
    ])
}
pub(super) fn view(d: Expr, s: Expr) -> Expr {
    ctor(
        nil(d.clone()),
        region(d.clone()),
        offset(d.clone()),
        total(d.clone(), s).value,
        capacity(d),
    )
}
pub(super) fn projection(d: Expr, s: Expr, f: usize) -> Typed {
    ctor_projection(
        vec![
            nil(d.clone()),
            region(d.clone()),
            offset(d.clone()),
            total(d.clone(), s).value,
            capacity(d),
        ],
        f,
    )
}
pub(super) fn data() -> Vec<(String, Expr)> {
    let mut a = counts::data();
    a.push((
        "no_reallocation".into(),
        eq(guard(v("destination"), v("source")), literal(true)),
    ));
    a
}
pub(super) fn arguments() -> Vec<Expr> {
    data().iter().map(|(n, _)| v(n)).collect()
}
pub(super) fn guard_facts(p: &mut Producer) -> [Typed; 2] {
    let d = v("destination");
    let s = v("source");
    let l = canon(WORD, length(d.clone()));
    let n = canon(WORD, length(s));
    let raw = carry(WORD, l.clone(), n.clone(), literal(false));
    let le = numeric::le(sum(WORD, l, n, literal(false)), canon(WORD, capacity(d)));
    let f = facts(p, &[not(raw.clone()), le], v("no_reallocation"));
    [not_true(p, raw, f[0].proof.clone()), f[1].clone()]
}
pub(super) fn nonnegative(p: &mut Producer) -> Typed {
    let [_, le] = guard_facts(p);
    let nn = indexed::cap_nn(
        p,
        v("destination"),
        v("storage_length"),
        v("live_mask"),
        v("destination_valid"),
    );
    numeric::nonnegative(
        total(v("destination"), v("source")).value,
        canon(WORD, capacity(v("destination"))),
        le.proof,
        nn.proof,
    )
}
fn start(d: Expr) -> numeric::Word {
    numeric::addition(canon(WORD, offset(d.clone())), canon(WORD, length(d)))
}
fn end(d: Expr, s: Expr) -> numeric::Word {
    numeric::addition(canon(WORD, offset(d.clone())), total(d, s).value)
}
fn member(d: Expr, s: Expr, r: Expr, q: Expr) -> Expr {
    and(
        word_eq(0, r, region(d.clone())),
        and(
            numeric::le(start(d.clone()).value, q.clone()),
            word_less(WORD, q, end(d, s).value),
        ),
    )
}
fn cell_body(h: Expr, d: Expr, s: Expr, r: Expr, q: Expr) -> Expr {
    choose_t(
        element(),
        member(d.clone(), s.clone(), r.clone(), q.clone()),
        read(
            h.clone(),
            s,
            arith::difference(WORD, q.clone(), start(d).value),
        ),
        apply(h, vec![r, q]),
    )
}
pub(super) fn updated(h: Expr, d: Expr, s: Expr) -> Expr {
    nc("AppendHeap", vec![h, d, s])
}
fn cell_def(h: Expr, d: Expr, s: Expr, r: Expr, i: Expr) -> Typed {
    typed(
        element(),
        read_heap(
            updated(h.clone(), d.clone(), s.clone()),
            r.clone(),
            i.clone(),
        ),
        cell_body(
            h.clone(),
            d.clone(),
            s.clone(),
            r.clone(),
            canon(WORD, i.clone()),
        ),
        nc("AppendCellDefinition", vec![h, d, s, r, i]),
    )
}
fn heap_data() -> Vec<(String, Expr)> {
    vec![
        ("heap".into(), heap()),
        ("destination".into(), descriptor()),
        ("source".into(), descriptor()),
    ]
}
pub(super) fn install(p: &mut Producer, trace: Option<&Path>) {
    function_definition(
        p,
        "SliceAccessValidityResliceBulkConstructionAppendHeap",
        &heap_data(),
        heap(),
        lambda(
            "__region",
            bool_ty(),
            lambda(
                "__key",
                word_ty(),
                cell_body(
                    v("heap"),
                    v("destination"),
                    v("source"),
                    v("__region"),
                    v("__key"),
                ),
            ),
        ),
    );
    let mut a = heap_data();
    a.extend([
        ("query_region".into(), bool_ty()),
        ("index".into(), word_ty()),
    ]);
    let mut q = cell_def(
        v("heap"),
        v("destination"),
        v("source"),
        v("query_region"),
        v("index"),
    );
    q.proof = apply(c(REFL), vec![element(), q.right.clone()]);
    theorem(
        p,
        "SliceAccessValidityResliceBulkConstructionAppendCellDefinition",
        &a,
        q,
    );
    validity::install(p, trace);
    indices::install(p, trace);
    reads::install(p, trace);
}

mod emit;
pub(super) fn emit(base: Certificate, output: &Path, count: usize) {
    emit::generate(base, output, count);
}
