//! Canonical-index byte observations and unobservable string tails.
//! A total backing function is an ordinary proof interface, not Go admission.
use super::*;
mod bounds;
pub(super) fn generate_stringops(root: &Path, output: &Path, capacity: usize, trace: bool) {
    bounds::generate_stringops(root, output, capacity, trace);
}
pub(super) fn generate_construction(root: &Path, output: &Path, capacity: usize, trace: bool) {
    bounds::generate_construction(root, output, capacity, trace);
}
pub(super) fn generate_bulk_copy(root: &Path, output: &Path, capacity: usize, trace: bool) {
    bounds::generate_bulk_copy(root, output, capacity, trace);
}
pub(super) fn generate_reslice(root: &Path, output: &Path, capacity: usize, trace: bool) {
    bounds::generate_reslice(root, output, capacity, trace);
}
pub(super) fn generate_access(root: &Path, output: &Path, capacity: usize, trace: bool) {
    bounds::generate_access(root, output, capacity, trace);
}
pub(super) fn generate_slices(root: &Path, output: &Path, capacity: usize, trace: bool) {
    bounds::generate_slices(root, output, capacity, trace);
}
pub(super) fn generate_bounds(root: &Path, output: &Path, capacity: usize, trace: bool) {
    bounds::generate(root, output, capacity, trace);
}

const WORD: usize = 6;
const BYTE: usize = 3;
fn byte() -> Expr {
    cube_type(BYTE)
}
fn word_ty() -> Expr {
    cube_type(WORD)
}
fn store_ty() -> Expr {
    forall("__byte_index", word_ty(), byte())
}
fn bool_fn() -> Expr {
    forall("__byte_continuation", bool_ty(), bool_ty())
}
fn addressed(d: usize, leaf: Expr) -> Expr {
    (0..d).rev().fold(leaf, |ty, i| {
        forall(&format!("__byte_address_type_{i}"), bool_ty(), ty)
    })
}
fn eq_ty(ty: Expr, a: Expr, b: Expr) -> Expr {
    apply(c(EQ), vec![ty, a, b])
}
fn reflexive_t(ty: Expr, x: Expr) -> Typed {
    typed(
        ty.clone(),
        x.clone(),
        x.clone(),
        apply(c(REFL), vec![ty, x]),
    )
}
fn canon(d: usize, x: Expr) -> Expr {
    tree("ByteCanonical", d, vec![x])
}
fn join(d: usize, lo: Expr, hi: Expr) -> Expr {
    lambda(
        "__byte_half",
        bool_ty(),
        choose_t(cube_type(d - 1), v("__byte_half"), hi, lo),
    )
}
fn canon_body(d: usize, x: Expr) -> Expr {
    if d == 0 {
        x
    } else {
        join(
            d,
            canon(d - 1, project(x.clone(), false)),
            canon(d - 1, project(x, true)),
        )
    }
}
fn canon_def(d: usize, x: Expr) -> Typed {
    typed(
        cube_type(d),
        canon(d, x.clone()),
        canon_body(d, x.clone()),
        tree("ByteCanonicalDefinition", d, vec![x]),
    )
}
fn canon_project(d: usize, x: Expr, high: bool) -> Typed {
    typed(
        cube_type(d - 1),
        project(canon(d, x.clone()), high),
        canon(d - 1, project(x.clone(), high)),
        tree(
            if high {
                "ByteCanonicalHigh"
            } else {
                "ByteCanonicalLow"
            },
            d,
            vec![x],
        ),
    )
}
fn canon_idem(d: usize, x: Expr) -> Typed {
    typed(
        cube_type(d),
        canon(d, canon(d, x.clone())),
        canon(d, x.clone()),
        tree("ByteCanonicalIdempotent", d, vec![x]),
    )
}
fn observations(d: usize, x: Expr, y: Expr) -> Expr {
    let indices = addresses(d, "__byte_bit_");
    (0..d).rev().fold(
        eq(apply(x, indices.clone()), apply(y, indices)),
        |body, i| forall(&format!("__byte_bit_{i}"), bool_ty(), body),
    )
}
fn child_observation(d: usize, h: Expr, high: bool) -> Expr {
    let rest = addresses(d - 1, "__byte_child_bit_");
    let mut args = vec![literal(high)];
    args.extend(rest);
    cube_lambda(d - 1, "__byte_child_bit_", apply(h, args))
}
fn canon_congr(d: usize, x: Expr, y: Expr, h: Expr) -> Typed {
    typed(
        cube_type(d),
        canon(d, x.clone()),
        canon(d, y.clone()),
        tree("ByteCanonicalCongruence", d, vec![x, y, h]),
    )
}
fn canon_bits(d: usize, x: Expr) -> Expr {
    tree("ByteCanonicalBits", d, vec![x])
}
fn bit_address(n: usize) -> Vec<Expr> {
    (0..WORD)
        .rev()
        .map(|j| literal((n >> j) & 1 != 0))
        .collect()
}
fn low_bits(i: Expr, d: usize) -> Vec<Expr> {
    (0..d)
        .rev()
        .map(|j| apply(i.clone(), bit_address(j)))
        .collect()
}
fn word_eq(d: usize, x: Expr, y: Expr) -> Expr {
    tree("ByteWordEqual", d, vec![x, y])
}
fn word_less(d: usize, x: Expr, y: Expr) -> Expr {
    tree("ByteWordLess", d, vec![x, y])
}
fn index_word(bits: &[Expr]) -> Expr {
    let query = addresses(WORD, "__byte_source_bit_");
    let body = bits
        .iter()
        .rev()
        .enumerate()
        .rev()
        .fold(literal(false), |tail, (i, b)| {
            let match_bit = query.iter().enumerate().fold(literal(true), |a, (j, q)| {
                and(
                    a,
                    bool_eq(q.clone(), literal((i >> (WORD - 1 - j)) & 1 != 0)),
                )
            });
            choose(match_bit, b.clone(), tail)
        });
    cube_lambda(WORD, "__byte_source_bit_", body)
}
fn constant_word(n: usize) -> Expr {
    let bits = (0..9)
        .rev()
        .map(|i| literal((n >> i) & 1 != 0))
        .collect::<Vec<_>>();
    index_word(&bits)
}
fn zero_byte() -> Expr {
    cube_lambda(BYTE, "__byte_zero_", literal(false))
}
fn raw_active(i: Expr, l: Expr) -> Expr {
    and(
        not(apply(i.clone(), vec![literal(true); WORD])),
        word_less(WORD, i, l),
    )
}
fn active(i: Expr, l: Expr) -> Expr {
    raw_active(canon(WORD, i), l)
}
fn read_byte(s: Expr, i: Expr) -> Expr {
    apply(s, vec![canon(WORD, i)])
}
fn mask(l: Expr, s: Expr) -> Expr {
    apply(c(&name("ByteTailMask")), vec![l, s])
}
fn masked_body(l: Expr, s: Expr, i: Expr) -> Expr {
    choose_t(byte(), active(i.clone(), l), read_byte(s, i), zero_byte())
}
fn mask_read(l: Expr, s: Expr, i: Expr) -> Typed {
    typed(
        byte(),
        read_byte(mask(l.clone(), s.clone()), i.clone()),
        masked_body(l.clone(), s.clone(), i.clone()),
        apply(c(&name("ByteTailMaskRead")), vec![l, s, i]),
    )
}
fn remask(l: Expr, s: Expr, i: Expr) -> Typed {
    typed(
        byte(),
        read_byte(mask(l.clone(), mask(l.clone(), s.clone())), i.clone()),
        read_byte(mask(l.clone(), s.clone()), i.clone()),
        apply(c(&name("ByteTailIdempotentRead")), vec![l, s, i]),
    )
}
fn index_observation(l: Expr, s: Expr, i: Expr, h: Expr) -> Typed {
    typed(
        byte(),
        read_byte(mask(l.clone(), s.clone()), i.clone()),
        read_byte(s.clone(), i.clone()),
        apply(c(&name("ByteTailDefinedRead")), vec![l, s, i, h]),
    )
}
fn select(p: &mut Producer, e: Typed, b: Expr, truth: bool, h: Expr) -> Typed {
    let e = change(p, e, typed(bool_ty(), b, literal(truth), h));
    let Expr::Apply(_, args) = &e.right else {
        panic!("byte selection absent")
    };
    let right = args[if truth { 2 } else { 1 }].clone();
    let tail = typed(
        e.ty.clone(),
        e.right.clone(),
        right.clone(),
        apply(c(REFL), vec![e.ty.clone(), right]),
    );
    next(e, tail)
}
fn word_library(p: &mut Producer, d: usize) {
    let x = v("x");
    let y = v("y");
    let unary = vec![("x".into(), cube_type(d))];
    let binary = vec![("x".into(), cube_type(d)), ("y".into(), cube_type(d))];
    function_definition(
        p,
        &format!("TreeByteCanonical{d}"),
        &unary,
        cube_type(d),
        canon_body(d, x.clone()),
    );
    let mut e = canon_def(d, x.clone());
    e.proof = apply(c(REFL), vec![e.ty.clone(), e.right.clone()]);
    theorem(p, &format!("TreeByteCanonicalDefinition{d}"), &unary, e);
    if d > 0 {
        for high in [false, true] {
            let mut e = canon_project(d, x.clone(), high);
            e.proof = apply(c(REFL), vec![e.ty.clone(), e.right.clone()]);
            theorem(
                p,
                &format!(
                    "TreeByteCanonical{}{}",
                    if high { "High" } else { "Low" },
                    d
                ),
                &unary,
                e,
            );
        }
    }
    let mut e = canon_def(d, canon(d, x.clone()));
    if d > 0 {
        for high in [false, true] {
            e = change(p, e, canon_project(d, x.clone(), high));
            e = change(p, e, canon_idem(d - 1, project(x.clone(), high)));
        }
    }
    if d > 0 {
        e = next(e, backwards(canon_def(d, x.clone())));
    }
    theorem(p, &format!("TreeByteCanonicalIdempotent{d}"), &unary, e);
    let mut args = binary.clone();
    args.push(("same_bits".into(), observations(d, x.clone(), y.clone())));
    let e = if d == 0 {
        typed(
            bool_ty(),
            canon(0, x.clone()),
            canon(0, y.clone()),
            v("same_bits"),
        )
    } else {
        let mut e = canon_def(d, x.clone());
        for high in [false, true] {
            e = change(
                p,
                e,
                canon_congr(
                    d - 1,
                    project(x.clone(), high),
                    project(y.clone(), high),
                    child_observation(d, v("same_bits"), high),
                ),
            );
        }
        next(e, backwards(canon_def(d, y.clone())))
    };
    theorem(p, &format!("TreeByteCanonicalCongruence{d}"), &args, e);
    let proof = if d == 0 {
        refl(x.clone())
    } else {
        let rest = (1..d)
            .map(|j| v(&format!("__byte_bit_{j}")))
            .collect::<Vec<_>>();
        let mut branches = vec![];
        for high in [false, true] {
            let q = canon_project(d, x.clone(), high);
            let left = apply(q.left.clone(), rest.clone());
            let right = apply(q.right.clone(), rest.clone());
            let q = mapped(
                q,
                bool_ty(),
                lambda(
                    "__byte_bit_view",
                    cube_type(d - 1),
                    apply(v("__byte_bit_view"), rest.clone()),
                ),
                left,
                right,
            );
            let tail = typed(
                bool_ty(),
                q.right.clone(),
                apply(project(x.clone(), high), rest.clone()),
                apply(canon_bits(d - 1, project(x.clone(), high)), rest.clone()),
            );
            let e = next(q, tail);
            branches.push((1..d).rev().fold(e.proof, |body, j| {
                lambda(&format!("__byte_bit_{j}"), bool_ty(), body)
            }));
        }
        let mut indices = vec![v("__byte_first_bit")];
        indices.extend(rest);
        let mut goal = eq(
            apply(canon(d, x.clone()), indices.clone()),
            apply(x.clone(), indices),
        );
        for j in (1..d).rev() {
            goal = forall(&format!("__byte_bit_{j}"), bool_ty(), goal);
        }
        lambda(
            "__byte_bit_0",
            bool_ty(),
            apply(
                c("Std.Bool.cases"),
                vec![
                    lambda("__byte_first_bit", bool_ty(), goal),
                    branches[0].clone(),
                    branches[1].clone(),
                    v("__byte_bit_0"),
                ],
            ),
        )
    };
    let (ty, proof) = fold_binders(
        &unary,
        observations(d, canon(d, x.clone()), x.clone()),
        proof,
    );
    p.declare(&format!("TreeByteCanonicalBits{d}"), ty, proof, true);
    let eq_body = if d == 0 {
        bool_eq(x.clone(), y.clone())
    } else {
        and(
            word_eq(d - 1, project(x.clone(), false), project(y.clone(), false)),
            word_eq(d - 1, project(x.clone(), true), project(y.clone(), true)),
        )
    };
    let less_body = if d == 0 {
        and(not(x.clone()), y.clone())
    } else {
        let xh = project(x.clone(), true);
        let yh = project(y.clone(), true);
        choose(
            word_eq(d - 1, xh.clone(), yh.clone()),
            word_less(d - 1, project(x.clone(), false), project(y.clone(), false)),
            word_less(d - 1, xh, yh),
        )
    };
    function_definition(
        p,
        &format!("TreeByteWordEqual{d}"),
        &binary,
        bool_ty(),
        eq_body,
    );
    function_definition(
        p,
        &format!("TreeByteWordLess{d}"),
        &binary,
        bool_ty(),
        less_body,
    );
}
fn primitives(p: &mut Producer) {
    let l = v("length");
    let s = v("store");
    let i = v("index");
    let args = vec![("length".into(), word_ty()), ("store".into(), store_ty())];
    function_definition(
        p,
        "ByteTailMask",
        &args,
        store_ty(),
        lambda(
            "__byte_mask_index",
            word_ty(),
            choose_t(
                byte(),
                raw_active(v("__byte_mask_index"), l.clone()),
                read_byte(s.clone(), v("__byte_mask_index")),
                zero_byte(),
            ),
        ),
    );
    let mut args = args;
    args.push(("index".into(), word_ty()));
    let raw = choose_t(
        byte(),
        active(i.clone(), l.clone()),
        read_byte(s.clone(), canon(WORD, i.clone())),
        zero_byte(),
    );
    let e = typed(
        byte(),
        read_byte(mask(l.clone(), s.clone()), i.clone()),
        raw.clone(),
        apply(c(REFL), vec![byte(), raw]),
    );
    let e = change(p, e, canon_idem(WORD, i.clone()));
    theorem(p, "ByteTailMaskRead", &args, e);
    let t = byte();
    let b = v("condition");
    let a = v("a");
    let z = v("z");
    let left = choose_t(
        t.clone(),
        b.clone(),
        choose_t(t.clone(), b.clone(), a.clone(), z.clone()),
        z.clone(),
    );
    let right = choose_t(t.clone(), b.clone(), a.clone(), z.clone());
    let goal = eq_ty(t.clone(), left.clone(), right.clone());
    let proof = apply(
        c("Std.Bool.cases"),
        vec![
            lambda(
                "__byte_case",
                bool_ty(),
                substitute(&goal, &b, &v("__byte_case")),
            ),
            apply(c(REFL), vec![t.clone(), z]),
            apply(c(REFL), vec![t.clone(), a]),
            b,
        ],
    );
    theorem(
        p,
        "ByteTailSelectIdempotent",
        &[
            ("condition".into(), bool_ty()),
            ("a".into(), t.clone()),
            ("z".into(), t.clone()),
        ],
        typed(t, left, right, proof),
    );
    let mut e = mask_read(l.clone(), mask(l.clone(), s.clone()), i.clone());
    e = change(p, e, mask_read(l.clone(), s.clone(), i.clone()));
    let selected = typed(
        byte(),
        e.right.clone(),
        masked_body(l.clone(), s.clone(), i.clone()),
        apply(
            c(&name("ByteTailSelectIdempotent")),
            vec![
                active(i.clone(), l.clone()),
                read_byte(s.clone(), i.clone()),
                zero_byte(),
            ],
        ),
    );
    let e = next(
        next(e, selected),
        backwards(mask_read(l.clone(), s.clone(), i.clone())),
    );
    theorem(p, "ByteTailIdempotentRead", &args, e);
    let mut safe = args.clone();
    safe.push((
        "defined".into(),
        eq(active(i.clone(), l.clone()), literal(true)),
    ));
    let e = select(
        p,
        mask_read(l.clone(), s.clone(), i.clone()),
        active(i.clone(), l),
        true,
        v("defined"),
    );
    theorem(p, "ByteTailDefinedRead", &safe, e);
    let ext_args = vec![
        ("store".into(), store_ty()),
        ("x".into(), word_ty()),
        ("y".into(), word_ty()),
        ("same_bits".into(), observations(WORD, v("x"), v("y"))),
    ];
    let q = canon_congr(WORD, v("x"), v("y"), v("same_bits"));
    let e = mapped(
        q,
        byte(),
        v("store"),
        read_byte(v("store"), v("x")),
        read_byte(v("store"), v("y")),
    );
    theorem(p, "ByteReadValueExtensional", &ext_args, e);
}
fn fold_name(d: usize, lex: bool, suffix: &str) -> String {
    format!("Byte{}Fold{suffix}{d}", if lex { "Lex" } else { "All" })
}
fn fold(d: usize, lex: bool, m: Expr) -> Expr {
    apply(c(&name(&fold_name(d, lex, ""))), vec![m])
}
fn leaf_ty(lex: bool) -> Expr {
    if lex {
        bool_fn()
    } else {
        bool_ty()
    }
}
fn fold_body(d: usize, lex: bool, m: Expr) -> Expr {
    if d == 0 {
        return m;
    }
    let low = fold(d - 1, lex, project(m.clone(), false));
    let high = fold(d - 1, lex, project(m, true));
    if lex {
        lambda(
            "__byte_continuation",
            bool_ty(),
            apply(low, vec![apply(high, vec![v("__byte_continuation")])]),
        )
    } else {
        and(low, high)
    }
}
fn fold_definition(d: usize, lex: bool, m: Expr) -> Typed {
    typed(
        leaf_ty(lex),
        fold(d, lex, m.clone()),
        fold_body(d, lex, m.clone()),
        apply(c(&name(&fold_name(d, lex, "Definition"))), vec![m]),
    )
}
fn fold_congr(d: usize, lex: bool, a: Expr, b: Expr, h: Expr) -> Typed {
    typed(
        leaf_ty(lex),
        fold(d, lex, a.clone()),
        fold(d, lex, b.clone()),
        apply(c(&name(&fold_name(d, lex, "Congruence"))), vec![a, b, h]),
    )
}
fn fold_library(p: &mut Producer, d: usize, lex: bool) {
    let a = v("a");
    let b = v("b");
    let ty = addressed(d, leaf_ty(lex));
    let args = vec![("a".into(), ty.clone())];
    function_definition(
        p,
        &fold_name(d, lex, ""),
        &args,
        leaf_ty(lex),
        fold_body(d, lex, a.clone()),
    );
    let mut e = fold_definition(d, lex, a.clone());
    e.proof = apply(c(REFL), vec![e.ty.clone(), e.right.clone()]);
    theorem(p, &fold_name(d, lex, "Definition"), &args, e);
    let indices = addresses(d, "__byte_bit_");
    let relation = (0..d).rev().fold(
        eq_ty(
            leaf_ty(lex),
            apply(a.clone(), indices.clone()),
            apply(b.clone(), indices),
        ),
        |body, i| forall(&format!("__byte_bit_{i}"), bool_ty(), body),
    );
    let args = vec![
        ("a".into(), ty.clone()),
        ("b".into(), ty),
        ("same".into(), relation),
    ];
    let e = if d == 0 {
        typed(
            leaf_ty(lex),
            fold(d, lex, a.clone()),
            fold(d, lex, b.clone()),
            v("same"),
        )
    } else {
        let mut e = fold_definition(d, lex, a.clone());
        for high in [false, true] {
            e = change(
                p,
                e,
                fold_congr(
                    d - 1,
                    lex,
                    project(a.clone(), high),
                    project(b.clone(), high),
                    child_observation(d, v("same"), high),
                ),
            );
        }
        next(e, backwards(fold_definition(d, lex, b.clone())))
    };
    theorem(p, &fold_name(d, lex, "Congruence"), &args, e);
}
fn equal_observer(d: usize, l: Expr, s: Expr, r: Expr, t: Expr) -> Expr {
    let bits = addresses(d, "__byte_compare_index_");
    let i = index_word(&bits);
    let cells = cube_lambda(
        d,
        "__byte_compare_index_",
        word_eq(
            BYTE,
            read_byte(mask(l.clone(), s), i.clone()),
            read_byte(mask(r.clone(), t), i),
        ),
    );
    and(word_eq(WORD, l, r), fold(d, false, cells))
}
fn lex_cells(d: usize, l: Expr, s: Expr, r: Expr, t: Expr) -> Expr {
    let bits = addresses(d, "__byte_compare_index_");
    let i = index_word(&bits);
    let x = read_byte(mask(l.clone(), s), i.clone());
    let y = read_byte(mask(r.clone(), t), i.clone());
    let k = v("__byte_continuation");
    let body = choose(
        active(i.clone(), l),
        choose(
            active(i.clone(), r.clone()),
            choose(
                word_eq(BYTE, x.clone(), y.clone()),
                k.clone(),
                word_less(BYTE, x, y),
            ),
            literal(false),
        ),
        choose(active(i, r), literal(true), k),
    );
    cube_lambda(
        d,
        "__byte_compare_index_",
        lambda("__byte_continuation", bool_ty(), body),
    )
}
fn less_observer(d: usize, l: Expr, s: Expr, r: Expr, t: Expr) -> Expr {
    apply(
        fold(d, true, lex_cells(d, l, s, r, t)),
        vec![literal(false)],
    )
}
fn observer(d: usize, lex: bool, l: Expr, s: Expr, r: Expr, t: Expr) -> Expr {
    apply(
        c(&name(&format!(
            "ByteString{}{}",
            if lex { "Less" } else { "Equal" },
            d
        ))),
        vec![l, s, r, t],
    )
}
fn observer_body(d: usize, lex: bool, l: Expr, s: Expr, r: Expr, t: Expr) -> Expr {
    if lex {
        less_observer(d, l, s, r, t)
    } else {
        equal_observer(d, l, s, r, t)
    }
}
fn remask_observer(p: &mut Producer, d: usize, lex: bool) -> Typed {
    let l = v("left_length");
    let r = v("right_length");
    let s = v("left_store");
    let t = v("right_store");
    let ms = mask(l.clone(), s.clone());
    let mt = mask(r.clone(), t.clone());
    let left = observer(d, lex, l.clone(), ms.clone(), r.clone(), mt.clone());
    let right = observer(d, lex, l.clone(), s.clone(), r.clone(), t.clone());
    let body = observer_body(d, lex, l.clone(), ms.clone(), r.clone(), mt.clone());
    let mut e = typed(bool_ty(), left, body.clone(), refl(body));
    let idx = addresses(d, "__byte_relation_index_");
    let i = index_word(&idx);
    let cell_ty = leaf_ty(lex);
    let ca = if lex {
        lex_cells(d, l.clone(), ms, r.clone(), mt)
    } else {
        let value = word_eq(
            BYTE,
            read_byte(mask(l.clone(), ms), i.clone()),
            read_byte(mask(r.clone(), mt), i.clone()),
        );
        cube_lambda(d, "__byte_relation_index_", value)
    };
    let cb = if lex {
        lex_cells(d, l.clone(), s.clone(), r.clone(), t.clone())
    } else {
        let value = word_eq(
            BYTE,
            read_byte(mask(l.clone(), s.clone()), i.clone()),
            read_byte(mask(r.clone(), t.clone()), i.clone()),
        );
        cube_lambda(d, "__byte_relation_index_", value)
    };
    // Open the finite address telescope, then use full Byte equalities. No
    // equality of backing functions or function extensionality is assumed.
    let opened = weak_beta(&apply(ca.clone(), idx.clone()));
    let mut q = reflexive_t(cell_ty.clone(), opened);
    q = change(p, q, remask(l.clone(), s.clone(), i.clone()));
    q = change(p, q, remask(r.clone(), t.clone(), i.clone()));
    let expected = weak_beta(&apply(cb.clone(), idx));
    assert_eq!(q.right, expected);
    let h = (0..d).rev().fold(q.proof, |body, j| {
        lambda(&format!("__byte_relation_index_{j}"), bool_ty(), body)
    });
    let f = fold_congr(d, lex, ca, cb, h);
    // The two telescopes differ only in bound names. Preserve exact endpoints
    // used by the observer while retaining the checked congruence witness.
    let a_cells = if lex {
        lex_cells(
            d,
            l.clone(),
            mask(l.clone(), s.clone()),
            r.clone(),
            mask(r.clone(), t.clone()),
        )
    } else {
        let bits = addresses(d, "__byte_compare_index_");
        let i = index_word(&bits);
        cube_lambda(
            d,
            "__byte_compare_index_",
            word_eq(
                BYTE,
                read_byte(mask(l.clone(), mask(l.clone(), s.clone())), i.clone()),
                read_byte(mask(r.clone(), mask(r.clone(), t.clone())), i),
            ),
        )
    };
    let b_cells = if lex {
        lex_cells(d, l.clone(), s.clone(), r.clone(), t.clone())
    } else {
        let bits = addresses(d, "__byte_compare_index_");
        let i = index_word(&bits);
        cube_lambda(
            d,
            "__byte_compare_index_",
            word_eq(
                BYTE,
                read_byte(mask(l.clone(), s.clone()), i.clone()),
                read_byte(mask(r.clone(), t.clone()), i),
            ),
        )
    };
    let f = typed(
        cell_ty,
        fold(d, lex, a_cells),
        fold(d, lex, b_cells),
        f.proof,
    );
    e = change(p, e, f);
    let tail = typed(
        bool_ty(),
        right.clone(),
        observer_body(d, lex, l, s, r, t),
        refl(e.right.clone()),
    );
    next(e, backwards(tail))
}
fn finite_lift(d: usize, s: Expr) -> Expr {
    apply(c(&name(&format!("ByteFiniteBacking{d}"))), vec![s])
}
fn finite_read(d: usize, s: Expr, i: Expr) -> Typed {
    typed(
        byte(),
        read_byte(finite_lift(d, s.clone()), i.clone()),
        apply(s.clone(), low_bits(i.clone(), d)),
        apply(c(&name(&format!("ByteFiniteBackingRead{d}"))), vec![s, i]),
    )
}
fn finite_library(p: &mut Producer, d: usize) {
    let s = v("finite");
    let i = v("index");
    let args = vec![("finite".into(), cube_type(d + BYTE))];
    function_definition(
        p,
        &format!("ByteFiniteBacking{d}"),
        &args,
        store_ty(),
        lambda(
            "__finite_index",
            word_ty(),
            apply(s.clone(), low_bits(v("__finite_index"), d)),
        ),
    );
    let right = apply(s.clone(), low_bits(canon(WORD, i.clone()), d));
    let mut e = typed(
        byte(),
        read_byte(finite_lift(d, s.clone()), i.clone()),
        right.clone(),
        apply(c(REFL), vec![byte(), right]),
    );
    for j in 0..d {
        let indices = bit_address(j);
        let q = typed(
            bool_ty(),
            apply(canon(WORD, i.clone()), indices.clone()),
            apply(i.clone(), indices.clone()),
            apply(canon_bits(WORD, i.clone()), indices),
        );
        e = change(p, e, q);
    }
    let mut args = args;
    args.push(("index".into(), word_ty()));
    theorem(p, &format!("ByteFiniteBackingRead{d}"), &args, e);
}
pub(super) fn install_string_primitives(p: &mut Producer, depth: usize) {
    primitives(p);
    finite_library(p, depth);
}
pub(super) fn generate(root: &Path, output: &Path, capacity: usize, trace: bool) {
    assert!([0, 1, 32, 256].contains(&capacity));
    fs::create_dir_all(output).unwrap();
    let d = if capacity <= 1 {
        0
    } else {
        capacity.ilog2() as usize
    };
    let mut p = Producer {
        builder: Builder::from(natural::foundation(root)),
        fresh: 0,
    };
    p.builder.c.module = format!("{PREFIX}.ByteTailCapacity{capacity}");
    for n in 0..=WORD {
        word_library(&mut p, n);
    }
    primitives(&mut p);
    save_trace(&p, trace.then_some(output), "byte-tail-primitives");
    finite_library(&mut p, d);
    for n in 0..=d {
        for lex in [false, true] {
            fold_library(&mut p, n, lex);
        }
    }
    let args = vec![
        ("left_length".into(), word_ty()),
        ("left_store".into(), store_ty()),
        ("right_length".into(), word_ty()),
        ("right_store".into(), store_ty()),
    ];
    for lex in [false, true] {
        function_definition(
            &mut p,
            &format!("ByteString{}{}", if lex { "Less" } else { "Equal" }, d),
            &args,
            bool_ty(),
            observer_body(
                d,
                lex,
                v("left_length"),
                v("left_store"),
                v("right_length"),
                v("right_store"),
            ),
        );
        let e = remask_observer(&mut p, d, lex);
        theorem(
            &mut p,
            &format!("ByteStringTail{}{}", if lex { "Less" } else { "Equal" }, d),
            &args,
            e,
        );
    }
    save_trace(&p, trace.then_some(output), "byte-tail-observers");
    emit(p.builder.c, output, capacity, d);
}
fn bound(l: Expr, capacity: usize) -> Expr {
    eq(
        not(word_less(WORD, constant_word(capacity), l)),
        literal(true),
    )
}
fn emit(base: Certificate, output: &Path, capacity: usize, d: usize) {
    let mut cases = vec![];
    for family in [
        "canonical-byte",
        "defined-index",
        "index-extensional",
        "finite-source-read",
        "equality",
        "lexicographic",
    ] {
        if capacity == 0 && family == "defined-index" {
            continue;
        }
        for mutation in 0..2 {
            let mut p = Producer {
                builder: Builder::from(base.clone()),
                fresh: 0,
            };
            let l = v("length");
            let s = finite_lift(d, v("store"));
            let i = v("index");
            let mut args = vec![
                ("length".into(), word_ty()),
                ("store".into(), cube_type(d + BYTE)),
                ("length_bound".into(), bound(l.clone(), capacity)),
            ];
            let mut e = match family {
                "canonical-byte" => {
                    args.push(("index".into(), word_ty()));
                    remask(l.clone(), s.clone(), i.clone())
                }
                "defined-index" => {
                    args.push(("index".into(), word_ty()));
                    args.push((
                        "defined".into(),
                        if mutation == 0 {
                            eq(active(i.clone(), l.clone()), literal(true))
                        } else {
                            eq(literal(true), literal(true))
                        },
                    ));
                    index_observation(l.clone(), s.clone(), i.clone(), v("defined"))
                }
                "index-extensional" => {
                    args.push(("index".into(), word_ty()));
                    args.push(("other_index".into(), word_ty()));
                    let h = if mutation == 0 {
                        observations(WORD, i.clone(), v("other_index"))
                    } else {
                        observations(WORD, i.clone(), i.clone())
                    };
                    args.push(("same_bits".into(), h));
                    typed(
                        byte(),
                        read_byte(s.clone(), i.clone()),
                        read_byte(s.clone(), v("other_index")),
                        apply(
                            c(&name("ByteReadValueExtensional")),
                            vec![s.clone(), i.clone(), v("other_index"), v("same_bits")],
                        ),
                    )
                }
                "finite-source-read" => {
                    args.push(("index".into(), word_ty()));
                    finite_read(d, v("store"), i.clone())
                }
                _ => {
                    args.push(("right_length".into(), word_ty()));
                    args.push(("right_store".into(), cube_type(d + BYTE)));
                    args.push(("right_bound".into(), bound(v("right_length"), capacity)));
                    let lex = family == "lexicographic";
                    let t = finite_lift(d, v("right_store"));
                    typed(
                        bool_ty(),
                        observer(
                            d,
                            lex,
                            l.clone(),
                            mask(l.clone(), s.clone()),
                            v("right_length"),
                            mask(v("right_length"), t.clone()),
                        ),
                        observer(d, lex, l.clone(), s.clone(), v("right_length"), t.clone()),
                        apply(
                            c(&name(&format!(
                                "ByteStringTail{}{}",
                                if lex { "Less" } else { "Equal" },
                                d
                            ))),
                            vec![l.clone(), s.clone(), v("right_length"), t],
                        ),
                    )
                }
            };
            if mutation == 1 {
                if family == "canonical-byte" {
                    let bits = addresses(BYTE, "__byte_negative_");
                    e.right =
                        cube_lambda(BYTE, "__byte_negative_", not(apply(e.right.clone(), bits)));
                } else if family == "finite-source-read" {
                    args.push(("replacement".into(), byte()));
                    e.right = v("replacement");
                } else if family == "equality" || family == "lexicographic" {
                    e.right = not(e.right);
                }
            }
            let single_cell_control =
                family == "index-extensional" && capacity <= 1 && mutation == 1;
            if single_cell_control {
                e.proof = apply(c(REFL), vec![byte(), v("store")]);
            }
            let suffix = format!("ByteTailClaim{}", family.replace('-', ""));
            let ty = eq_ty(e.ty.clone(), e.left, e.right);
            let (ty, proof) = fold_binders(&args, ty, e.proof);
            p.declare(&suffix, ty.clone(), proof, true);
            let id = format!(
                "byte-tail-{capacity}-{family}-{}",
                if single_cell_control {
                    "single-cell-control"
                } else if mutation == 0 {
                    "valid"
                } else {
                    "altered"
                }
            );
            let terms = p.builder.c.term_table.len();
            let declarations = p.builder.c.declarations.len();
            let bytes = p.builder.finish();
            fs::write(output.join(format!("{id}.mpcert")), &bytes).unwrap();
            cases.push(json!({"id":id,"family":family,"count":capacity,"capacity":capacity,"address_depth":d,"source_word_bits":64,"payload_bits":8,"full_index_bits":64,"full_length_bits":64,"quantified_storage_bits":8usize<<d,"two_stores":family=="equality" || family=="lexicographic","backing":"complete finite Bool-addressed bytes lifted to canonical 64-bit index values; no payload enumeration","mutation":mutation,"expected":if mutation==0 || single_cell_control {"accepted"} else {"rejected"},"declarations":declarations,"terms":terms,"bytes":bytes.len(),"theorem":name(&suffix),"scope":"private string byte observations and tail unobservability; complete Go source/carrier/operation binding and remaining byte laws stay pending","goal_debug":format!("{ty:?}")}));
        }
    }
    fs::write(
        output.join("index.json"),
        serde_json::to_vec_pretty(
            &json!({"schema":"mpk.go_practical.byte_tail_pilot.v0","cases":cases}),
        )
        .unwrap(),
    )
    .unwrap();
}
