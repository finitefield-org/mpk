//! Actual semantic storage bound, independent of its power-of-two padding.
use super::*;

fn profile_bound(count: usize, n: Expr) -> Expr {
    not(word_less(WORD, literal_word(WORD, count as u64), n))
}
fn padding(count: usize) -> usize {
    1usize << capacity_depth(count)
}
fn covered_args(count: usize) -> Vec<(String, Expr)> {
    let mut a = conditional_args();
    a.push((
        "storage_bound".into(),
        eq(profile_bound(count, v("storage_length")), literal(true)),
    ));
    a
}
fn present(p: &mut Producer, e: Typed, q: Typed) -> Typed {
    if substitute(&e.right, &q.left, &q.right) == e.right {
        e
    } else {
        change(p, e, q)
    }
}
fn mask(d: usize, n: u64) -> u64 {
    let bits = 1usize << d;
    if bits == 64 {
        n
    } else {
        n & ((1u64 << bits) - 1)
    }
}
fn equal_literal(p: &mut Producer, d: usize, a: u64, b: u64) -> Typed {
    let (a, b) = (mask(d, a), mask(d, b));
    if a == b {
        return refl_equal(d, literal_word(d, a));
    }
    let mut e = definition(true, d, literal_word(d, a), literal_word(d, b));
    if d > 0 {
        for high in [false, true] {
            e = present(p, e, literal_projection(d, a, high));
            e = present(p, e, literal_projection(d, b, high));
        }
        for high in [false, true] {
            let shift = if high { 1usize << (d - 1) } else { 0 };
            let q = equal_literal(p, d - 1, a >> shift, b >> shift);
            e = present(p, e, q);
        }
    }
    let q = normalize(e.right.clone(), p);
    let e = next(e, q);
    assert_eq!(e.right, literal(a == b));
    e
}
pub(super) fn less_literal(p: &mut Producer, d: usize, a: u64, b: u64) -> Typed {
    let (a, b) = (mask(d, a), mask(d, b));
    let x = literal_word(d, a);
    let y = literal_word(d, b);
    if a == b {
        return equal_not_less(d, x.clone(), y, refl_equal(d, x).proof);
    }
    let mut e = definition(false, d, x, y);
    if d > 0 {
        for high in [false, true] {
            e = present(p, e, literal_projection(d, a, high));
            e = present(p, e, literal_projection(d, b, high));
        }
        let half = 1usize << (d - 1);
        let q = equal_literal(p, d - 1, a >> half, b >> half);
        let same = a >> half == b >> half;
        e = select(p, e, q.left, same, q.proof);
        let q = less_literal(
            p,
            d - 1,
            if same { a } else { a >> half },
            if same { b } else { b >> half },
        );
        e = next(e, q);
    } else {
        let q = normalize(e.right.clone(), p);
        e = next(e, q);
    }
    assert_eq!(e.right, literal(a < b));
    e
}
fn reconstructed(count: usize, index: Expr) -> Expr {
    super::super::super::super::coverage::reconstruction(padding(count), index)
}
fn cover_call(count: usize, d: Expr, n: Expr, live: Expr, i: Expr, proofs: [Expr; 3]) -> Typed {
    let a = address(offset(d.clone()), i.clone());
    typed(
        word_ty(),
        canon(WORD, a.clone()),
        canon(WORD, reconstructed(count, a)),
        vc(
            "Coverage",
            vec![
                d,
                n,
                live,
                i,
                proofs[0].clone(),
                proofs[1].clone(),
                proofs[2].clone(),
            ],
        ),
    )
}
fn coverage_proof(p: &mut Producer, count: usize) -> Typed {
    let d = v("descriptor");
    let n = v("storage_length");
    let live = v("live_mask");
    let i = v("index");
    let a = address(offset(d.clone()), i.clone());
    let lt = physical_call(
        2,
        d.clone(),
        n.clone(),
        live.clone(),
        i.clone(),
        v("valid"),
        v("index_guard"),
    );
    let b = literal_word(WORD, count as u64);
    let key = canon(WORD, a.clone());
    let no = not_true(p, word_less(WORD, b.clone(), n.clone()), v("storage_bound"));
    let mut lt = mixed_transitive(WORD, key.clone(), n, b.clone(), lt.proof, no.proof);
    if padding(count) != count {
        let no = less_literal(p, WORD, padding(count) as u64, count as u64);
        lt = mixed_transitive(
            WORD,
            key,
            b,
            literal_word(WORD, padding(count) as u64),
            lt.proof,
            no.proof,
        );
    }
    typed(
        word_ty(),
        canon(WORD, a.clone()),
        canon(WORD, reconstructed(count, a.clone())),
        apply(c(&name("IndexUnsignedPrefix")), vec![a, lt.proof]),
    )
}
fn safety() -> Typed {
    let args = vec![
        v("descriptor"),
        v("storage_length"),
        v("live_mask"),
        v("index"),
        v("valid"),
        v("index_guard"),
    ];
    let d = v("descriptor");
    let a = address(offset(d.clone()), v("index"));
    typed(
        bool_ty(),
        and(
            not(carry(
                WORD,
                canon(WORD, offset(d)),
                canon(WORD, v("index")),
                literal(false),
            )),
            not(sign_bit(WORD, canon(WORD, a))),
        ),
        literal(true),
        vc("Safety", args),
    )
}
fn read_coverage(count: usize, h: Expr) -> Typed {
    let d = v("descriptor");
    let n = v("storage_length");
    let live = v("live_mask");
    let i = v("index");
    let a = address(offset(d.clone()), i.clone());
    let mut args = vec![
        h.clone(),
        d.clone(),
        n,
        live,
        i.clone(),
        v("valid"),
        v("index_guard"),
        v("storage_bound"),
    ];
    typed(
        element(),
        read(h.clone(), d.clone(), i),
        read_heap(h, region(d), reconstructed(count, a)),
        vc("ReadCoverage", std::mem::take(&mut args)),
    )
}
fn nil_guard(d: Expr, i: Expr, hn: Expr) -> Typed {
    typed(
        bool_ty(),
        defined(d.clone(), i.clone()),
        literal(false),
        vc("NilGuard", vec![d, i, hn]),
    )
}
fn empty_guard(d: Expr, n: Expr, live: Expr, i: Expr, hv: Expr, hb: Expr) -> Typed {
    typed(
        bool_ty(),
        defined(d.clone(), i.clone()),
        literal(false),
        vc("EmptyGuard", vec![d, n, live, i, hv, hb]),
    )
}
pub(super) fn install(p: &mut Producer, count: usize, trace: Option<&Path>) {
    let d = v("descriptor");
    let n = v("storage_length");
    let live = v("live_mask");
    let i = v("index");
    let mut e = reflexive_t(bool_ty(), safety().left);
    for j in [1, 3] {
        e = change(
            p,
            e,
            physical_call(
                j,
                d.clone(),
                n.clone(),
                live.clone(),
                i.clone(),
                v("valid"),
                v("index_guard"),
            ),
        );
    }
    let q = normalize(e.right.clone(), p);
    e = next(e, q);
    theorem(p, "SliceAccessValiditySafety", &conditional_args(), e);
    let e = coverage_proof(p, count);
    theorem(p, "SliceAccessValidityCoverage", &covered_args(count), e);
    save_trace(p, trace, "access-coverage");
    let h = v("heap");
    let e = read_definition(h.clone(), d.clone(), i.clone());
    let q = cover_call(
        count,
        d.clone(),
        n.clone(),
        live.clone(),
        i.clone(),
        [v("valid"), v("index_guard"), v("storage_bound")],
    );
    let right = read_heap(
        h.clone(),
        region(d.clone()),
        reconstructed(count, address(offset(d.clone()), i.clone())),
    );
    let q = mapped(
        q,
        element(),
        lambda(
            "__access_key",
            word_ty(),
            apply(h, vec![region(d.clone()), v("__access_key")]),
        ),
        e.right.clone(),
        right,
    );
    let e = next(e, q);
    let mut ra = vec![("heap".into(), heap())];
    ra.extend(covered_args(count));
    theorem(p, "SliceAccessValidityReadCoverage", &ra, e);
    let mut e = index_def(d.clone(), i.clone());
    e = change(
        p,
        e,
        typed(bool_ty(), nil(d.clone()), literal(true), v("is_nil")),
    );
    let q = normalize(e.right.clone(), p);
    e = next(e, q);
    theorem(
        p,
        "SliceAccessValidityNilGuard",
        &[
            ("descriptor".into(), descriptor()),
            ("index".into(), word_ty()),
            ("is_nil".into(), eq(nil(d.clone()), literal(true))),
        ],
        e,
    );
    if count == 0 {
        let guard = defined(d.clone(), i.clone());
        let lt = physical_call(
            2,
            d.clone(),
            n.clone(),
            live.clone(),
            i.clone(),
            v("valid"),
            v("__empty_index"),
        );
        let no = not_true(
            p,
            word_less(WORD, literal_word(WORD, 0), n.clone()),
            v("storage_bound"),
        );
        let key = canon(WORD, address(offset(d.clone()), i.clone()));
        let bad = mixed_transitive(
            WORD,
            key.clone(),
            n.clone(),
            literal_word(WORD, 0),
            lt.proof,
            no.proof,
        );
        let no = less_zero(WORD, key);
        let absurd = impossible(
            bool_ty(),
            guard.clone(),
            literal(false),
            contradiction(bad, no.proof),
        );
        let q = typed(
            bool_ty(),
            guard.clone(),
            literal(false),
            cases(
                guard.clone(),
                "__empty_index",
                eq(guard, literal(false)),
                v("__empty_index"),
                absurd.proof,
            ),
        );
        let mut ea = primary_args();
        ea.extend([
            (
                "valid".into(),
                eq(valid(d.clone(), n.clone(), live.clone()), literal(true)),
            ),
            (
                "storage_bound".into(),
                eq(profile_bound(0, n), literal(true)),
            ),
        ]);
        theorem(p, "SliceAccessValidityEmptyGuard", &ea, q);
    }
    save_trace(p, trace, "access-root");
}
pub(super) fn emit(base: Certificate, output: &Path, count: usize) {
    let depth = capacity_depth(count);
    let h = finite_heap(depth, v("backing"));
    let mut cases = vec![];
    for family in [
        "address-safety",
        "physical-bounds",
        "live-region",
        "full-value-coverage",
        "finite-read",
        "valid-copy",
        "valid-full",
        "nil-unindexable",
        "empty-no-access",
    ] {
        let data = matches!(
            family,
            "address-safety"
                | "physical-bounds"
                | "live-region"
                | "full-value-coverage"
                | "finite-read"
        );
        if count == 0 && data || count > 0 && family == "empty-no-access" {
            continue;
        }
        for mutation in 0..2 {
            let mut p = Producer {
                builder: Builder::from(base.clone()),
                fresh: 0,
            };
            let mut args = vec![("backing".into(), cube_type(1 + depth + ELEMENT))];
            args.extend(primary_args());
            let d = v("descriptor");
            let n = v("storage_length");
            let live = v("live_mask");
            let i = v("index");
            if family != "nil-unindexable" {
                args.push((
                    "valid".into(),
                    if mutation == 1 && matches!(family, "physical-bounds" | "live-region") {
                        eq(literal(true), literal(true))
                    } else {
                        eq(valid(d.clone(), n.clone(), live.clone()), literal(true))
                    },
                ));
            }
            if data {
                args.push((
                    "index_guard".into(),
                    eq(defined(d.clone(), i.clone()), literal(true)),
                ));
            }
            if data || family == "empty-no-access" {
                args.push((
                    "storage_bound".into(),
                    if mutation == 1 && matches!(family, "full-value-coverage" | "empty-no-access")
                    {
                        eq(literal(true), literal(true))
                    } else {
                        eq(profile_bound(count, n.clone()), literal(true))
                    },
                ));
            }
            let mut e = match family {
                "address-safety" => safety(),
                "physical-bounds" => physical_call(
                    2,
                    d.clone(),
                    n.clone(),
                    live.clone(),
                    i.clone(),
                    v("valid"),
                    v("index_guard"),
                ),
                "live-region" => physical_call(
                    0,
                    d.clone(),
                    n.clone(),
                    live.clone(),
                    i.clone(),
                    v("valid"),
                    v("index_guard"),
                ),
                "full-value-coverage" => cover_call(
                    count,
                    d.clone(),
                    n.clone(),
                    live.clone(),
                    i.clone(),
                    [v("valid"), v("index_guard"), v("storage_bound")],
                ),
                "finite-read" => read_coverage(count, h.clone()),
                "valid-copy" | "valid-full" => typed(
                    bool_ty(),
                    valid(
                        if family == "valid-full" {
                            full(d.clone())
                        } else {
                            copy(d.clone())
                        },
                        n.clone(),
                        live.clone(),
                    ),
                    literal(true),
                    vc(
                        if family == "valid-full" {
                            "Full"
                        } else {
                            "Copy"
                        },
                        vec![d.clone(), n.clone(), live.clone(), v("valid")],
                    ),
                ),
                "nil-unindexable" => {
                    let cv = vec![
                        literal(true),
                        region(d.clone()),
                        offset(d.clone()),
                        literal_word(WORD, 0),
                        literal_word(WORD, 0),
                    ];
                    let hn = ctor_projection(cv.clone(), 0);
                    nil_guard(at("Descriptor", cv), i.clone(), hn.proof)
                }
                _ => empty_guard(
                    d.clone(),
                    n.clone(),
                    live.clone(),
                    i.clone(),
                    v("valid"),
                    v("storage_bound"),
                ),
            };
            if mutation == 1 {
                if matches!(family, "address-safety" | "valid-copy" | "valid-full") {
                    e.right = literal(false);
                } else if family == "nil-unindexable" {
                    e.right = literal(true);
                } else if family == "finite-read" {
                    e.right = cell_zero();
                }
            }
            let label = if mutation == 0 {
                "valid"
            } else {
                match family {
                    "physical-bounds" | "live-region" => "erased-validity",
                    "full-value-coverage" | "empty-no-access" => "erased-storage-bound",
                    "finite-read" => "zeroed-read",
                    _ => "altered-result",
                }
            };
            let suffix = format!("SliceAccessClaim{}", family.replace('-', ""));
            let ty = eq_ty(e.ty.clone(), e.left, e.right);
            let (ty, proof) = fold_binders(&args, ty, e.proof);
            p.declare(&suffix, ty.clone(), proof, true);
            let id = format!("slice-access-{count}-{family}-{label}");
            let terms = p.builder.c.term_table.len();
            let declarations = p.builder.c.declarations.len();
            let bytes = p.builder.finish();
            fs::write(output.join(format!("{id}.mpcert")), &bytes).unwrap();
            cases.push(json!({"id":id,"family":family,"count":count,"capacity":count,"padding_capacity":padding(count),"address_depth":depth,"source_word_bits":64,"extent_intermediate_bits":65,"descriptor_bits":512,"element_bits":128,"logical_region_count":2,"source_pointer_bits_admitted":false,"quantified_storage_bits":128usize<<(depth+1),"mutation":mutation,"expected":if mutation==0 {"accepted"} else {"rejected"},"declarations":declarations,"terms":terms,"bytes":bytes.len(),"theorem":name(&suffix),"scope":"ordinary computed descriptor validity and actual index guard discharge absolute full-width address safety/coverage; production Go capture/ownership/operation and remaining laws stay pending","goal_debug":format!("{ty:?}")}));
        }
    }
    fs::write(
        output.join("index.json"),
        serde_json::to_vec_pretty(
            &json!({"schema":"mpk.go_practical.slice_access_pilot.v0","cases":cases}),
        )
        .unwrap(),
    )
    .unwrap();
}
