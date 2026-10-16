//! Full-width byte substring and concatenation ingredients, with actual guards.
use super::*;
mod emit;

fn sc(s: &str, args: Vec<Expr>) -> Expr {
    rc(&format!("String{s}"), args)
}
fn suffix(s: &str) -> String {
    format!("SliceAccessValidityResliceString{s}")
}
fn words(names: &[&str]) -> Vec<(String, Expr)> {
    names.iter().map(|s| ((*s).into(), word_ty())).collect()
}
fn sub_words() -> Vec<numeric::Word> {
    ["length", "lower", "upper"]
        .into_iter()
        .map(|s| numeric::source(v(s)))
        .collect()
}
fn sub_length() -> numeric::Word {
    let w = sub_words();
    numeric::difference(w[2].value.clone(), w[1].value.clone())
}
fn sub_checks() -> Vec<Expr> {
    let w = sub_words();
    vec![
        not(sign_bit(WORD, w[1].value.clone())),
        model::signed_le(w[1].value.clone(), w[2].value.clone()),
        model::signed_le(w[2].value.clone(), w[0].value.clone()),
    ]
}
fn sub_guard() -> Expr {
    sc(
        "SubstringGuard",
        ["length", "lower", "upper"].into_iter().map(v).collect(),
    )
}
fn sub_args(index: bool) -> Vec<(String, Expr)> {
    let mut a = words(&["length", "lower", "upper"]);
    a.push(("range_guard".into(), eq(sub_guard(), literal(true))));
    if index {
        a.push(("index".into(), word_ty()));
        a.push((
            "index_guard".into(),
            eq(active(v("index"), sub_length().value), literal(true)),
        ));
    }
    a
}
fn call_sub(s: &str, index: bool) -> Typed {
    let mut a = vec![v("length"), v("lower"), v("upper"), v("range_guard")];
    if index {
        a.extend([v("index"), v("index_guard")]);
    }
    let w = sub_words();
    let i = numeric::source(v("index"));
    let mapped = numeric::addition(w[1].value.clone(), i.value);
    let left = match s {
        "LengthNonnegative" => sign_bit(WORD, sub_length().value),
        "LengthBound" => numeric::le(sub_length().value, w[0].value.clone()),
        "Index" => active(mapped.value, w[0].value.clone()),
        _ => panic!("substring fact"),
    };
    typed(
        bool_ty(),
        left,
        literal(s != "LengthNonnegative"),
        sc(&format!("Substring{s}"), a),
    )
}
fn sub_raw(s: Expr, lo: Expr) -> Expr {
    sc("SubstringRaw", vec![s, lo])
}
fn sub_store(s: Expr) -> Expr {
    sc("Substring", vec![s, v("lower"), v("upper")])
}
fn store_definition(sub: bool, s: Expr, t: Expr) -> Typed {
    let left = if sub {
        sub_store(s.clone())
    } else {
        cat_store(s.clone(), t.clone())
    };
    let right = if sub {
        mask(sub_length().value, sub_raw(s.clone(), v("lower")))
    } else {
        mask(cat_length().value, cat_raw(s.clone(), t.clone()))
    };
    let mut a = if sub {
        vec![s, v("lower"), v("upper")]
    } else {
        vec![v("left_length"), v("right_length"), s, t]
    };
    typed(
        store_ty(),
        left,
        right,
        sc(
            if sub {
                "SubstringDefinition"
            } else {
                "ConcatDefinition"
            },
            std::mem::take(&mut a),
        ),
    )
}
fn active_facts(p: &mut Producer, i: Expr, l: Expr, h: Expr) -> (Typed, Typed) {
    let n = not(sign_bit(WORD, canon(WORD, i.clone())));
    let lt = word_less(WORD, canon(WORD, i.clone()), l);
    let nn = conjunction(p, n.clone(), lt.clone(), h.clone(), false);
    (
        not_true(p, sign_bit(WORD, canon(WORD, i)), nn.proof),
        conjunction(p, n, lt, h, true),
    )
}
fn active_word(p: &mut Producer, i: numeric::Word, l: Expr, nn: Typed, lt: Typed) -> Typed {
    let mut q = reflexive_t(bool_ty(), active(i.value.clone(), l));
    q = change(
        p,
        q,
        typed(word_ty(), canon(WORD, i.value.clone()), i.value, i.normal),
    );
    q = change(p, q, nn);
    q = change(p, q, lt);
    let n = normalize(q.right.clone(), p);
    next(q, n)
}
fn derive_sub(p: &mut Producer) -> (Typed, Typed, Typed) {
    let w = sub_words();
    let checks = sub_checks();
    let h = typed(bool_ty(), both(&checks), literal(true), v("range_guard"));
    let fs = facts(p, &checks, h.proof);
    let mut nn = not_true(p, sign_bit(WORD, w[1].value.clone()), fs[0].proof.clone());
    let mut ordered = vec![];
    let mut high_nn = None;
    for (j, (x, y)) in [(w[1].clone(), w[2].clone()), (w[2].clone(), w[0].clone())]
        .into_iter()
        .enumerate()
    {
        let sx = sign_bit(WORD, x.value);
        let sy = sign_bit(WORD, y.value.clone());
        let reverse = word_less(
            WORD,
            y.value,
            [w[1].value.clone(), w[2].value.clone()][j].clone(),
        );
        let condition = and(not(sx.clone()), checks[j + 1].clone());
        let mut q = reflexive_t(bool_ty(), condition.clone());
        q = change(p, q, fs[j + 1].clone());
        q = change(p, q, nn);
        let n = normalize(q.right.clone(), p);
        q = next(q, n);
        let result = and(not(sy.clone()), not(reverse.clone()));
        let q = under(
            p,
            condition,
            result,
            literal(true),
            q.proof,
            rc("SignedForward", vec![sx, sy.clone(), reverse.clone()]),
        );
        let n = conjunction(
            p,
            not(sy.clone()),
            not(reverse.clone()),
            q.proof.clone(),
            false,
        );
        nn = not_true(p, sy.clone(), n.proof);
        if j == 0 {
            high_nn = Some(nn.clone());
        }
        ordered.push(conjunction(p, not(sy), not(reverse), q.proof, true));
    }
    let n = sub_length();
    let nonnegative = numeric::difference_nonnegative(
        w[2].clone(),
        w[1].clone(),
        ordered[0].proof.clone(),
        high_nn.unwrap().proof,
    );
    let hc = numeric::restore(
        w[2].clone(),
        w[1].clone(),
        ordered[0].proof.clone(),
        false,
        true,
    );
    let lower = numeric::sum_lower(n.clone(), w[1].clone(), hc.proof);
    let lower = numeric::change_left(
        p,
        lower,
        numeric::restore(
            w[2].clone(),
            w[1].clone(),
            ordered[0].proof.clone(),
            false,
            false,
        ),
    );
    let bound = numeric::transitive_le(
        n.value.clone(),
        w[2].value.clone(),
        w[0].value.clone(),
        lower.proof,
        ordered[1].proof.clone(),
    );
    let i = numeric::source(v("index"));
    let (_, ilt) = active_facts(p, v("index"), n.value.clone(), v("index_guard"));
    let hc = numeric::restore(
        w[2].clone(),
        w[1].clone(),
        ordered[0].proof.clone(),
        true,
        true,
    );
    let h = an(
        "Monotone",
        WORD,
        vec![
            w[1].value.clone(),
            i.value.clone(),
            n.value.clone(),
            ilt.proof,
            hc.proof,
        ],
    );
    let ci = carry(WORD, w[1].value.clone(), i.value.clone(), literal(false));
    let mapped = numeric::addition(w[1].value.clone(), i.value);
    let lt = word_less(
        WORD,
        mapped.value.clone(),
        sum(WORD, w[1].value.clone(), n.value, literal(false)),
    );
    let lt = conjunction(p, not(ci), lt, h, true);
    let lt = numeric::change_left(
        p,
        lt,
        numeric::restore(
            w[2].clone(),
            w[1].clone(),
            ordered[0].proof.clone(),
            true,
            false,
        ),
    );
    let rev = not_true(
        p,
        word_less(WORD, w[0].value.clone(), w[2].value.clone()),
        ordered[1].proof.clone(),
    );
    let lt = mixed_transitive(
        WORD,
        mapped.value.clone(),
        w[2].value.clone(),
        w[0].value.clone(),
        lt.proof,
        rev.proof,
    );
    let nn = numeric::nonnegative(
        mapped.value.clone(),
        w[0].value.clone(),
        numeric::less_le(mapped.value.clone(), w[0].value.clone(), lt.proof.clone()).proof,
        nn.proof,
    );
    let index = active_word(p, mapped, w[0].value.clone(), nn, lt);
    (nonnegative, bound, index)
}
fn cat_lengths() -> Vec<numeric::Word> {
    ["left_length", "right_length"]
        .into_iter()
        .map(|s| numeric::source(v(s)))
        .collect()
}
fn cat_length() -> numeric::Word {
    let w = cat_lengths();
    numeric::addition(w[0].value.clone(), w[1].value.clone())
}
fn cat_args(index: Option<bool>) -> Vec<(String, Expr)> {
    let w = cat_lengths();
    let mut a = words(&["left_length", "right_length"]);
    for (j, s) in ["left_nonnegative", "right_nonnegative"]
        .into_iter()
        .enumerate()
    {
        a.push((
            s.into(),
            eq(sign_bit(WORD, w[j].value.clone()), literal(false)),
        ));
    }
    if let Some(right) = index {
        a.push(("index".into(), word_ty()));
        a.push((
            "index_guard".into(),
            eq(
                active(v("index"), w[usize::from(right)].value.clone()),
                literal(true),
            ),
        ));
        // A concatenation may be a valid Go value yet exceed the selected bound.
        // The theorem uses a semantic nonnegative result, later derived from B.
        a.push((
            "result_nonnegative".into(),
            eq(sign_bit(WORD, cat_length().value), literal(false)),
        ));
    }
    a
}
fn cat_raw(s: Expr, t: Expr) -> Expr {
    sc("ConcatRaw", vec![v("left_length"), s, t])
}
fn cat_store(s: Expr, t: Expr) -> Expr {
    sc("Concat", vec![v("left_length"), v("right_length"), s, t])
}
fn no_carry() -> Typed {
    let w = cat_lengths();
    typed(
        bool_ty(),
        carry(WORD, w[0].value.clone(), w[1].value.clone(), literal(false)),
        literal(false),
        sc(
            "ConcatNoCarry",
            vec![
                v("left_length"),
                v("right_length"),
                v("left_nonnegative"),
                v("right_nonnegative"),
            ],
        ),
    )
}
fn no_carry_proof(p: &mut Producer) -> Typed {
    let w = cat_lengths();
    let mut x = w[0].value.clone();
    let mut y = w[1].value.clone();
    let mut b = literal(false);
    let mut q = reflexive_t(bool_ty(), carry(WORD, x.clone(), y.clone(), b.clone()));
    for d in (0..=WORD).rev() {
        q = change(p, q, carry_def(d, x.clone(), y.clone(), b.clone()));
        if d > 0 {
            b = carry(
                d - 1,
                project(x.clone(), false),
                project(y.clone(), false),
                b,
            );
            x = project(x, true);
            y = project(y, true);
        }
    }
    for (j, s) in ["left_nonnegative", "right_nonnegative"]
        .into_iter()
        .enumerate()
    {
        q = change(
            p,
            q,
            typed(
                bool_ty(),
                [x.clone(), y.clone()][j].clone(),
                literal(false),
                v(s),
            ),
        );
    }
    let n = normalize(q.right.clone(), p);
    next(q, n)
}
fn cat_index(p: &mut Producer, right: bool) -> Typed {
    let w = cat_lengths();
    let total = cat_length();
    let i = numeric::source(v("index"));
    let (_, lt) = active_facts(
        p,
        v("index"),
        w[usize::from(right)].value.clone(),
        v("index_guard"),
    );
    let (mapped, lt) = if right {
        let h = an(
            "Monotone",
            WORD,
            vec![
                w[0].value.clone(),
                i.value.clone(),
                w[1].value.clone(),
                lt.proof,
                no_carry().proof,
            ],
        );
        let ci = carry(WORD, w[0].value.clone(), i.value.clone(), literal(false));
        let m = numeric::addition(w[0].value.clone(), i.value.clone());
        let lt = conjunction(
            p,
            not(ci),
            word_less(WORD, m.value.clone(), total.value.clone()),
            h,
            true,
        );
        (m, lt)
    } else {
        let h = numeric::sum_lower(w[0].clone(), w[1].clone(), no_carry().proof);
        let no = not_true(
            p,
            word_less(WORD, total.value.clone(), w[0].value.clone()),
            h.proof,
        );
        let lt = mixed_transitive(
            WORD,
            i.value.clone(),
            w[0].value.clone(),
            total.value.clone(),
            lt.proof,
            no.proof,
        );
        (i, lt)
    };
    let nn = numeric::nonnegative(
        mapped.value.clone(),
        total.value.clone(),
        numeric::less_le(mapped.value.clone(), total.value.clone(), lt.proof.clone()).proof,
        v("result_nonnegative"),
    );
    active_word(p, mapped, total.value, nn, lt)
}
fn cat_index_call(right: bool) -> Typed {
    let w = cat_lengths();
    let i = numeric::source(v("index"));
    let m = if right {
        numeric::addition(w[0].value.clone(), i.value).value
    } else {
        canon(WORD, v("index"))
    };
    typed(
        bool_ty(),
        active(m, cat_length().value),
        literal(true),
        sc(
            if right {
                "ConcatRightIndex"
            } else {
                "ConcatLeftIndex"
            },
            vec![
                v("left_length"),
                v("right_length"),
                v("left_nonnegative"),
                v("right_nonnegative"),
                v("index"),
                v("index_guard"),
                v("result_nonnegative"),
            ],
        ),
    )
}
fn read_store(p: &mut Producer, sub: bool, s: Expr, t: Expr, i: Expr, hg: Expr) -> Typed {
    let def = store_definition(sub, s.clone(), t.clone());
    let left = read_byte(def.left.clone(), i.clone());
    let mut q = reflexive_t(byte(), left);
    q = change(p, q, def);
    let (len, raw) = if sub {
        (sub_length().value, sub_raw(s, v("lower")))
    } else {
        (cat_length().value, cat_raw(s, t))
    };
    change(p, q, index_observation(len, raw, i, hg))
}
fn sub_read(p: &mut Producer, s: Expr) -> Typed {
    let mut q = read_store(p, true, s.clone(), s.clone(), v("index"), v("index_guard"));
    let w = sub_words();
    let mapped = numeric::addition(w[1].value.clone(), canon(WORD, v("index")));
    let right = read_byte(s.clone(), mapped.value);
    let raw = typed(
        byte(),
        read_byte(sub_raw(s.clone(), v("lower")), v("index")),
        right.clone(),
        refl_t(byte(), right),
    );
    q = next(q, raw);
    q
}
fn refl_t(ty: Expr, x: Expr) -> Expr {
    apply(c(REFL), vec![ty, x])
}
fn cat_read(p: &mut Producer, right: bool, s: Expr, t: Expr) -> Typed {
    let w = cat_lengths();
    let i = numeric::source(v("index"));
    let m = if right {
        numeric::addition(w[0].value.clone(), i.value.clone())
    } else {
        i.clone()
    };
    let q = cat_index_call(right);
    let mut q = read_store(p, false, s.clone(), t.clone(), m.value.clone(), q.proof);
    let ci = canon(WORD, m.value.clone());
    let cond = word_less(WORD, ci.clone(), w[0].value.clone());
    let body = choose_t(
        byte(),
        cond.clone(),
        read_byte(s.clone(), ci.clone()),
        read_byte(t.clone(), arith::difference(WORD, ci, w[0].value.clone())),
    );
    let raw = typed(
        byte(),
        read_byte(cat_raw(s.clone(), t.clone()), m.value.clone()),
        body.clone(),
        refl_t(byte(), body),
    );
    q = next(q, raw);
    q = change(
        p,
        q,
        typed(
            word_ty(),
            canon(WORD, m.value.clone()),
            m.value.clone(),
            m.normal.clone(),
        ),
    );
    let (_, lt) = active_facts(
        p,
        v("index"),
        w[usize::from(right)].value.clone(),
        v("index_guard"),
    );
    if right {
        // (left+i)-left=i over every source bit. SumLower chooses the right arm.
        let h = numeric::sum_lower(w[0].clone(), i.clone(), {
            let h = an(
                "Monotone",
                WORD,
                vec![
                    w[0].value.clone(),
                    i.value.clone(),
                    w[1].value.clone(),
                    lt.proof,
                    no_carry().proof,
                ],
            );
            let out = carry(WORD, w[0].value.clone(), i.value.clone(), literal(false));
            let hs = conjunction(
                p,
                not(out.clone()),
                word_less(WORD, m.value.clone(), cat_length().value),
                h,
                false,
            );
            not_true(p, out, hs.proof).proof
        });
        let h = not_true(
            p,
            word_less(WORD, m.value.clone(), w[0].value.clone()),
            h.proof,
        );
        q = select(p, q, h.left.clone(), false, h.proof);
        q = change(p, q, copying::cancel_sum(w[0].clone(), i.clone()));
    } else {
        q = select(p, q, lt.left.clone(), true, lt.proof);
    }
    q = maybe(p, q, canon_idem(WORD, v("index")));
    q
}
fn covered(count: usize, length: Expr, index: Expr, bound: Expr, guard: Expr) -> Typed {
    typed(
        word_ty(),
        canon(WORD, index.clone()),
        canon(WORD, string_reconstruction(count, index.clone())),
        sc("Coverage", vec![length, index, bound, guard]),
    )
}
fn install_coverage(p: &mut Producer, count: usize) {
    let l = v("source_length");
    let i = v("source_index");
    let ci = canon(WORD, i.clone());
    let b = literal_word(WORD, count as u64);
    let padding = 1usize << max_depth(count);
    let (_, lt) = active_facts(p, i.clone(), l.clone(), v("source_guard"));
    let no = not_true(p, word_less(WORD, b.clone(), l.clone()), v("source_bound"));
    let mut lt = mixed_transitive(WORD, ci.clone(), l.clone(), b.clone(), lt.proof, no.proof);
    if padding != count {
        let no = super::super::embedding::less_literal(p, WORD, padding as u64, count as u64);
        lt = mixed_transitive(
            WORD,
            ci,
            b.clone(),
            literal_word(WORD, padding as u64),
            lt.proof,
            no.proof,
        );
    }
    let q = typed(
        word_ty(),
        canon(WORD, i.clone()),
        canon(WORD, string_reconstruction(count, i.clone())),
        apply(c(&name("IndexUnsignedPrefix")), vec![i.clone(), lt.proof]),
    );
    let a = vec![
        ("source_length".into(), word_ty()),
        ("source_index".into(), word_ty()),
        (
            "source_bound".into(),
            eq(numeric::le(l.clone(), b), literal(true)),
        ),
        ("source_guard".into(), eq(active(i, l), literal(true))),
    ];
    theorem(p, &suffix("Coverage"), &a, q);
}
pub(super) fn install(p: &mut Producer, count: usize, trace: Option<&Path>) {
    function_definition(
        p,
        &suffix("SubstringGuard"),
        &words(&["length", "lower", "upper"]),
        bool_ty(),
        both(&sub_checks()),
    );
    let s = v("source");
    let t = v("right_source");
    function_definition(
        p,
        &suffix("SubstringRaw"),
        &[("source".into(), store_ty()), ("lower".into(), word_ty())],
        store_ty(),
        lambda(
            "__substring_index",
            word_ty(),
            read_byte(
                s.clone(),
                sum(
                    WORD,
                    canon(WORD, v("lower")),
                    v("__substring_index"),
                    literal(false),
                ),
            ),
        ),
    );
    function_definition(
        p,
        &suffix("Substring"),
        &[
            ("source".into(), store_ty()),
            ("lower".into(), word_ty()),
            ("upper".into(), word_ty()),
        ],
        store_ty(),
        mask(sub_length().value, sub_raw(s.clone(), v("lower"))),
    );
    function_definition(
        p,
        &suffix("ConcatRaw"),
        &[
            ("left_length".into(), word_ty()),
            ("source".into(), store_ty()),
            ("right_source".into(), store_ty()),
        ],
        store_ty(),
        lambda(
            "__concat_index",
            word_ty(),
            choose_t(
                byte(),
                word_less(WORD, v("__concat_index"), canon(WORD, v("left_length"))),
                read_byte(s.clone(), v("__concat_index")),
                read_byte(
                    t.clone(),
                    arith::difference(WORD, v("__concat_index"), canon(WORD, v("left_length"))),
                ),
            ),
        ),
    );
    let mut a = words(&["left_length", "right_length"]);
    a.extend([
        ("source".into(), store_ty()),
        ("right_source".into(), store_ty()),
    ]);
    function_definition(
        p,
        &suffix("Concat"),
        &a,
        store_ty(),
        mask(cat_length().value, cat_raw(s.clone(), t.clone())),
    );
    for sub in [true, false] {
        let mut a = if sub {
            vec![
                ("source".into(), store_ty()),
                ("lower".into(), word_ty()),
                ("upper".into(), word_ty()),
            ]
        } else {
            a.clone()
        };
        let mut q = store_definition(sub, s.clone(), t.clone());
        q.proof = refl_t(store_ty(), q.right.clone());
        theorem(
            p,
            &suffix(if sub {
                "SubstringDefinition"
            } else {
                "ConcatDefinition"
            }),
            &std::mem::take(&mut a),
            q,
        );
    }
    let (nn, bound, index) = derive_sub(p);
    for (s, q, indexed) in [
        ("LengthNonnegative", nn, false),
        ("LengthBound", bound, false),
        ("Index", index, true),
    ] {
        theorem(p, &suffix(&format!("Substring{s}")), &sub_args(indexed), q);
    }
    let q = no_carry_proof(p);
    theorem(p, &suffix("ConcatNoCarry"), &cat_args(None), q);
    for right in [false, true] {
        let q = cat_index(p, right);
        theorem(
            p,
            &suffix(if right {
                "ConcatRightIndex"
            } else {
                "ConcatLeftIndex"
            }),
            &cat_args(Some(right)),
            q,
        );
    }
    let mut a = sub_args(true);
    a.push(("source".into(), store_ty()));
    let q = sub_read(p, s.clone());
    theorem(p, &suffix("SubstringRead"), &a, q);
    let w = sub_words();
    let m = numeric::addition(w[1].value.clone(), canon(WORD, v("index")));
    let mut q = sub_read(p, mask(w[0].value.clone(), s.clone()));
    q = next(
        q,
        index_observation(
            w[0].value.clone(),
            s.clone(),
            m.value,
            call_sub("Index", true).proof,
        ),
    );
    q = next(q, backwards(sub_read(p, s.clone())));
    theorem(p, &suffix("SubstringCanonical"), &a, q);
    for right in [false, true] {
        let mut a = cat_args(Some(right));
        a.extend([
            ("source".into(), store_ty()),
            ("right_source".into(), store_ty()),
        ]);
        let q = cat_read(p, right, s.clone(), t.clone());
        theorem(
            p,
            &suffix(if right {
                "ConcatRightRead"
            } else {
                "ConcatLeftRead"
            }),
            &a,
            q,
        );
        let w = cat_lengths();
        let mut q = cat_read(
            p,
            right,
            mask(w[0].value.clone(), s.clone()),
            mask(w[1].value.clone(), t.clone()),
        );
        q = next(
            q,
            index_observation(
                w[usize::from(right)].value.clone(),
                if right { t.clone() } else { s.clone() },
                v("index"),
                v("index_guard"),
            ),
        );
        q = next(q, backwards(cat_read(p, right, s.clone(), t.clone())));
        theorem(
            p,
            &suffix(if right {
                "ConcatRightCanonical"
            } else {
                "ConcatLeftCanonical"
            }),
            &a,
            q,
        );
    }
    install_coverage(p, count);
    save_trace(p, trace, "string-operations");
}
pub(super) fn emit(base: Certificate, output: &Path, count: usize) {
    emit::generate(base, output, count);
}
