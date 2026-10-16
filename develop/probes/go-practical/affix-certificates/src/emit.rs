//! Ordinary source-shaped claims with explicit, nonvacuous mutation pairs.
use super::*;
fn families(n: usize) -> Vec<&'static str> {
    let mut f = vec![
        "range",
        "length",
        "binding",
        "canonical",
        "empty",
        "matched-length",
    ];
    if n > 0 {
        f.extend(["longer", "matched-index", "matched-read", "coverage"]);
    }
    f
}
fn wrong(n: usize, f: &str) -> &'static str {
    match f {
        "length" => "extra-byte",
        "binding" if n > 0 => "one-bit-comparison",
        "canonical" if n >= 2 => "prefix-suffix-confusion",
        "empty" => "false-empty",
        "matched-length" if n > 0 => "erased-match",
        "longer" => "erased-length-guard",
        "matched-index" => "erased-index-guard",
        "matched-read" => "flipped-byte-equality",
        "coverage" => "flipped-word",
        _ => "flipped-result",
    }
}
fn one_bit(suf: bool, d: usize) -> Expr {
    let (_, n) = words_affix();
    let bits = addresses(d, "__affix_wrong_");
    let i = index_word(&bits);
    let s = read_byte(
        mask(n.value.clone(), raw_affix(suf, v("source"))),
        i.clone(),
    );
    let t = read_byte(mask(n.value, v("pattern")), i);
    let body = bool_eq(
        apply(s, vec![literal(false); BYTE]),
        apply(t, vec![literal(false); BYTE]),
    );
    and(
        guard(),
        fold(d, false, cube_lambda(d, "__affix_wrong_", body)),
    )
}
fn proof_call(suf: bool, d: usize, f: &str) -> Typed {
    if f == "range" {
        return range_call(suf);
    }
    if f == "length" {
        return length_call(suf);
    }
    if f == "binding" {
        return binding_call(suf, d);
    }
    let source = v("source");
    let pattern = v("pattern");
    let (l, n) = words_affix();
    let mut a = vec![
        v("source_length"),
        source.clone(),
        v("pattern_length"),
        pattern.clone(),
        v("source_nonnegative"),
        v("pattern_nonnegative"),
    ];
    let (left, right, tag) = match f {
        "canonical" => (
            relation(
                suf,
                d,
                mask(l.value, source.clone()),
                mask(n.value, pattern.clone()),
            ),
            relation(suf, d, source, pattern),
            "Canonical",
        ),
        "empty" => (
            subst_expr(
                relation(suf, d, source, pattern),
                &[("pattern_length", literal_word(WORD, 0))],
            ),
            literal(true),
            "Empty",
        ),
        "longer" => {
            a.push(v("longer"));
            (relation(suf, d, source, pattern), literal(false), "Longer")
        }
        "matched-length" => {
            a.push(v("matched"));
            (guard(), literal(true), "MatchedLength")
        }
        "matched-index" | "matched-read" => {
            a.extend([
                v("matched"),
                v("index"),
                v("index_guard"),
                v("pattern_bound"),
            ]);
            let mapped = if suf {
                sum(
                    WORD,
                    start(suf).value,
                    canon(WORD, v("index")),
                    literal(false),
                )
            } else {
                v("index")
            };
            (
                if f == "matched-index" {
                    active(mapped, l.value)
                } else {
                    word_eq(
                        BYTE,
                        read_byte(source, mapped),
                        read_byte(pattern, v("index")),
                    )
                },
                literal(true),
                if f == "matched-index" {
                    "MatchedIndex"
                } else {
                    "MatchedRead"
                },
            )
        }
        _ => panic!("affix family"),
    };
    typed(
        bool_ty(),
        left,
        right,
        af(&format!("{}{tag}{d}", label(suf)), a),
    )
}
pub(super) fn generate(base: Certificate, output: &Path, count: usize) {
    let depth = max_depth(count);
    let finite_s = finite_lift(depth, v("backing"));
    let finite_t = finite_lift(depth, v("pattern_backing"));
    let bound = literal_word(WORD, count as u64);
    let mut cases = vec![];
    for suf in [false, true] {
        for f in families(count) {
            for mutation in 0..2 {
                let mut p = Producer {
                    builder: Builder::from(base.clone()),
                    fresh: 0,
                };
                let (l, n) = words_affix();
                let mut a = vec![
                    ("source_length".into(), word_ty()),
                    ("backing".into(), cube_type(depth + BYTE)),
                    ("pattern_length".into(), word_ty()),
                    ("pattern_backing".into(), cube_type(depth + BYTE)),
                    (
                        "source_bound".into(),
                        eq(numeric::le(l.value.clone(), bound.clone()), literal(true)),
                    ),
                    (
                        "pattern_bound".into(),
                        eq(numeric::le(n.value.clone(), bound.clone()), literal(true)),
                    ),
                ];
                if f == "range" || f == "length" {
                    a.push(("guard".into(), eq(guard(), literal(true))));
                }
                if f == "longer" {
                    a.push((
                        "longer".into(),
                        eq(
                            word_less(WORD, l.value.clone(), n.value.clone()),
                            literal(true),
                        ),
                    ));
                }
                if f.starts_with("matched-") {
                    a.push((
                        "source_success".into(),
                        eq(
                            subst_expr(
                                algorithm(suf, depth, v("source"), v("pattern")),
                                &[("source", finite_s.clone()), ("pattern", finite_t.clone())],
                            ),
                            literal(true),
                        ),
                    ));
                }
                if f == "matched-index" || f == "matched-read" || f == "coverage" {
                    a.push(("index".into(), word_ty()));
                    a.push((
                        "index_guard".into(),
                        eq(active(v("index"), n.value.clone()), literal(true)),
                    ));
                }
                let mut q = if f == "coverage" {
                    direct_coverage(&mut p, count, v("index"))
                } else {
                    proof_call(suf, depth, f)
                };
                if f.starts_with("matched-") {
                    q = subst_t(q, &[("matched", source_matched(suf, depth).proof)]);
                }
                let nn_l = numeric::nonnegative(
                    l.value,
                    bound.clone(),
                    v("source_bound"),
                    refl(literal(false)),
                );
                let nn_p = numeric::nonnegative(
                    n.value,
                    bound.clone(),
                    v("pattern_bound"),
                    refl(literal(false)),
                );
                q = subst_t(
                    q,
                    &[
                        ("source_nonnegative", nn_l.proof),
                        ("pattern_nonnegative", nn_p.proof),
                        ("source", finite_s.clone()),
                        ("pattern", finite_t.clone()),
                    ],
                );
                let variant = if mutation == 0 {
                    "valid"
                } else {
                    wrong(count, f)
                };
                if mutation != 0 {
                    match variant {
                        "extra-byte" => {
                            q.right = sum(WORD, q.right, literal_word(WORD, 1), literal(false))
                        }
                        "one-bit-comparison" => {
                            q.right = subst_expr(
                                one_bit(suf, depth),
                                &[("source", finite_s.clone()), ("pattern", finite_t.clone())],
                            )
                        }
                        "prefix-suffix-confusion" => {
                            q.right = relation(!suf, depth, finite_s.clone(), finite_t.clone())
                        }
                        "erased-match" => {
                            for (key, ty) in &mut a {
                                if key == "source_success" {
                                    *ty = eq(literal(true), literal(true));
                                }
                            }
                        }
                        "erased-index-guard" => {
                            for (key, ty) in &mut a {
                                if key == "index_guard" {
                                    *ty = eq(literal(true), literal(true));
                                }
                            }
                        }
                        "erased-length-guard" => {
                            let (_, n) = words_affix();
                            q.right = observer(
                                depth,
                                false,
                                n.value.clone(),
                                raw_affix(suf, finite_s.clone()),
                                n.value,
                                finite_t.clone(),
                            );
                        }
                        "flipped-word" => {
                            q.right = cube_lambda(
                                WORD,
                                "__affix_bad_word_",
                                not(apply(q.right, addresses(WORD, "__affix_bad_word_"))),
                            )
                        }
                        _ => q.right = not(q.right),
                    }
                }
                let claim = format!("StringAffixClaim{}{f}", label(suf)).replace('-', "");
                let goal = eq_ty(q.ty, q.left, q.right);
                let (goal, proof) = fold_binders(&a, goal, q.proof);
                p.declare(&claim, goal.clone(), proof, true);
                let id = format!("affix-{count}-{}-{f}-{variant}", label(suf).to_lowercase());
                let terms = p.builder.c.term_table.len();
                let declarations = p.builder.c.declarations.len();
                let bytes = p.builder.finish();
                fs::write(output.join(format!("{id}.mpcert")), &bytes).unwrap();
                cases.push(json!({"id":id,"kind":label(suf).to_lowercase(),"family":f,"count":count,"capacity":count,"padding_capacity":1usize<<depth,"address_depth":depth,"source_word_bits":64,"length_bits":64,"payload_bits":8,"quantified_storage_bits":16usize<<depth,"mutation":mutation,"expected":if mutation==0{"accepted"}else{"rejected"},"declarations":declarations,"terms":terms,"bytes":bytes.len(),"theorem":name(&claim),"scope":"private signed length/slice/equality correspondence, computed prefix/suffix guards, complete byte observations and input-tail preservation; exact source identities do not activate a production adapter or profile","goal_debug":format!("{goal:?}")}));
            }
        }
    }
    fs::write(
        output.join("index.json"),
        serde_json::to_vec_pretty(
            &json!({"schema":"mpk.go_practical.affix_pilot.v0","cases":cases}),
        )
        .unwrap(),
    )
    .unwrap();
}
