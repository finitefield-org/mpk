//! Finite-byte instances; incorrect claims have separate source/model witnesses.
use super::*;

fn families(count: usize) -> Vec<&'static str> {
    let mut f = vec![
        "substring-nonnegative",
        "substring-length",
        "concat-carry",
        "concat-length",
        "concat-nonnegative",
    ];
    if count > 0 {
        f.extend([
            "substring-index",
            "substring-coverage",
            "substring-read",
            "substring-canonical",
            "concat-left-index",
            "concat-right-index",
            "concat-left-coverage",
            "concat-right-coverage",
            "concat-left-read",
            "concat-right-read",
            "concat-left-canonical",
            "concat-right-canonical",
        ]);
    }
    f
}
fn label(count: usize, f: &str) -> &'static str {
    match f {
        "substring-length" => "erased-range-guard",
        "concat-length" if count > 0 => "erased-output-bound",
        "substring-index" | "concat-left-index" | "concat-right-index" => "false-defined-index",
        "substring-read"
        | "concat-left-read"
        | "concat-right-read"
        | "substring-canonical"
        | "concat-left-canonical"
        | "concat-right-canonical" => "flipped-byte",
        f if f.ends_with("coverage") => "flipped-word",
        _ => "flipped-boolean",
    }
}
pub(super) fn generate(base: Certificate, output: &Path, count: usize) {
    let depth = if count <= 1 {
        0
    } else {
        (count - 1).ilog2() as usize + 1
    };
    let s = finite_lift(depth, v("backing"));
    let t = finite_lift(depth, v("right_backing"));
    let b = literal_word(WORD, count as u64);
    let mut cases = vec![];
    for family in families(count) {
        let sub = family.starts_with("substring-");
        let right = family.starts_with("concat-right-");
        let indexed = family.ends_with("coverage")
            || family.ends_with("index")
            || family.ends_with("read")
            || family.ends_with("canonical");
        for mutation in 0..2 {
            let mut p = Producer {
                builder: Builder::from(base.clone()),
                fresh: 0,
            };
            let mut a = vec![("backing".into(), cube_type(depth + BYTE))];
            if !sub {
                a.push(("right_backing".into(), cube_type(depth + BYTE)));
            }
            a.extend(if sub {
                sub_args(indexed)
            } else {
                cat_args(indexed.then_some(right))
            });
            for name in if sub {
                vec!["length"]
            } else {
                vec!["left_length", "right_length"]
            } {
                a.push((
                    format!("{name}_bound"),
                    eq(numeric::le(canon(WORD, v(name)), b.clone()), literal(true)),
                ));
            }
            if !sub {
                a.push((
                    "output_bound".into(),
                    eq(numeric::le(cat_length().value, b.clone()), literal(true)),
                ));
            }
            let mut q = match family {
                "substring-nonnegative" => call_sub("LengthNonnegative", false),
                "substring-length" => {
                    let h = call_sub("LengthBound", false);
                    numeric::transitive_le(
                        sub_length().value,
                        canon(WORD, v("length")),
                        b.clone(),
                        h.proof,
                        v("length_bound"),
                    )
                }
                "substring-index" => call_sub("Index", true),
                "substring-coverage" => {
                    let w = sub_words();
                    let m = numeric::addition(w[1].value.clone(), canon(WORD, v("index")));
                    covered(
                        count,
                        w[0].value.clone(),
                        m.value,
                        v("length_bound"),
                        call_sub("Index", true).proof,
                    )
                }
                "concat-left-coverage" | "concat-right-coverage" => {
                    let w = cat_lengths();
                    let m = if right {
                        numeric::addition(w[0].value.clone(), canon(WORD, v("index"))).value
                    } else {
                        canon(WORD, v("index"))
                    };
                    covered(
                        count,
                        cat_length().value,
                        m,
                        v("output_bound"),
                        cat_index_call(right).proof,
                    )
                }
                "substring-read" => sub_read(&mut p, s.clone()),
                "substring-canonical" => typed(
                    byte(),
                    read_byte(
                        sub_store(mask(canon(WORD, v("length")), s.clone())),
                        v("index"),
                    ),
                    read_byte(sub_store(s.clone()), v("index")),
                    sc(
                        "SubstringCanonical",
                        vec![
                            v("length"),
                            v("lower"),
                            v("upper"),
                            v("range_guard"),
                            v("index"),
                            v("index_guard"),
                            s.clone(),
                        ],
                    ),
                ),
                "concat-carry" => no_carry(),
                "concat-length" => typed(
                    bool_ty(),
                    numeric::le(cat_length().value, b.clone()),
                    literal(true),
                    v("output_bound"),
                ),
                "concat-nonnegative" => numeric::nonnegative(
                    cat_length().value,
                    b.clone(),
                    v("output_bound"),
                    refl(literal(false)),
                ),
                "concat-left-index" | "concat-right-index" => cat_index_call(right),
                "concat-left-read" | "concat-right-read" => {
                    cat_read(&mut p, right, s.clone(), t.clone())
                }
                "concat-left-canonical" | "concat-right-canonical" => {
                    let w = cat_lengths();
                    let i = if right {
                        numeric::addition(w[0].value.clone(), canon(WORD, v("index"))).value
                    } else {
                        canon(WORD, v("index"))
                    };
                    typed(
                        byte(),
                        read_byte(
                            cat_store(
                                mask(w[0].value.clone(), s.clone()),
                                mask(w[1].value.clone(), t.clone()),
                            ),
                            i.clone(),
                        ),
                        read_byte(cat_store(s.clone(), t.clone()), i),
                        sc(
                            if right {
                                "ConcatRightCanonical"
                            } else {
                                "ConcatLeftCanonical"
                            },
                            vec![
                                v("left_length"),
                                v("right_length"),
                                v("left_nonnegative"),
                                v("right_nonnegative"),
                                v("index"),
                                v("index_guard"),
                                v("result_nonnegative"),
                                s.clone(),
                                t.clone(),
                            ],
                        ),
                    )
                }
                _ => panic!("string family"),
            };
            if !sub && indexed {
                let nn = numeric::nonnegative(
                    cat_length().value,
                    b.clone(),
                    v("output_bound"),
                    refl(literal(false)),
                );
                q.proof = substitute(&q.proof, &v("result_nonnegative"), &nn.proof);
                a.retain(|(key, _)| key != "result_nonnegative");
            }
            let mutation_label = if mutation == 0 {
                "valid"
            } else {
                label(count, family)
            };
            if mutation == 1 {
                if mutation_label == "erased-range-guard" {
                    for (key, ty) in &mut a {
                        if key == "range_guard" {
                            *ty = eq(literal(true), literal(true));
                        }
                    }
                } else if mutation_label == "erased-output-bound" {
                    for (key, ty) in &mut a {
                        if key == "output_bound" {
                            *ty = eq(literal(true), literal(true));
                        }
                    }
                } else if mutation_label == "flipped-word" {
                    q.right = cube_lambda(
                        WORD,
                        "__wrong_word_bit_",
                        not(apply(q.right.clone(), addresses(WORD, "__wrong_word_bit_"))),
                    );
                } else if mutation_label == "flipped-byte" {
                    q.right = cube_lambda(
                        BYTE,
                        "__wrong_string_bit_",
                        not(apply(
                            q.right.clone(),
                            addresses(BYTE, "__wrong_string_bit_"),
                        )),
                    );
                } else {
                    q.right = not(q.right);
                }
            }
            let claim = format!("StringOperationClaim{}", family.replace('-', ""));
            let goal = eq_ty(q.ty, q.left, q.right);
            let (goal, proof) = fold_binders(&a, goal, q.proof);
            p.declare(&claim, goal.clone(), proof, true);
            let id = format!("stringop-{count}-{family}-{mutation_label}");
            let terms = p.builder.c.term_table.len();
            let declarations = p.builder.c.declarations.len();
            let bytes = p.builder.finish();
            fs::write(output.join(format!("{id}.mpcert")), &bytes).unwrap();
            cases.push(json!({"id":id,"family":family,"count":count,"capacity":count,"padding_capacity":1usize<<depth,"address_depth":depth,"source_word_bits":64,"length_bits":64,"payload_bits":8,"quantified_storage_bits":(8usize<<depth)*if sub {1} else {2},"mutation":mutation,"expected":if mutation==0 {"accepted"} else {"rejected"},"declarations":declarations,"terms":terms,"bytes":bytes.len(),"theorem":name(&claim),"scope":"ordinary signed substring guards, full-byte reads and input-tail normalization; concat length carry, explicit output bound, source-index lifting and full-byte/tail preservation; production source/effect/operation binding and remaining string laws stay pending","goal_debug":format!("{goal:?}")}));
        }
    }
    fs::write(
        output.join("index.json"),
        serde_json::to_vec_pretty(
            &json!({"schema":"mpk.go_practical.string_operation_pilot.v0","cases":cases}),
        )
        .unwrap(),
    )
    .unwrap();
}
