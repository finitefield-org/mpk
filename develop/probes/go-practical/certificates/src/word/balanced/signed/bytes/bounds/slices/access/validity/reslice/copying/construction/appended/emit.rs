//! Genuine source/model mutation pairs, without empty-domain false controls.
use super::*;

fn one_cell() -> Expr {
    cube_lambda(ELEMENT, "__nonzero_cell_", literal(true))
}
fn families(n: usize) -> Vec<(&'static str, &'static str)> {
    let mut a = vec![
        ("make", "validity"),
        ("make", "nil"),
        ("append", "validity"),
        ("append", "length"),
        ("append", "nil"),
        ("composed", "validity"),
        ("composed", "nil"),
    ];
    if n > 0 {
        a.extend([
            ("make", "zero"),
            ("make", "other-frame"),
            ("make", "fresh-frame"),
            ("append", "capacity"),
            ("append", "index"),
            ("append", "alias-index"),
            ("append", "read"),
            ("append", "prefix-frame"),
            ("append", "alias-read"),
            ("append", "tail-frame"),
            ("append", "other-frame"),
            ("append", "physical-frame"),
            ("composed", "read"),
            ("composed", "borrowed-frame"),
        ]);
    }
    if n >= 8 {
        a.push(("append", "snapshot"));
    }
    a
}
fn wrong(label: &str, k: &str, n: usize) -> &'static str {
    match (label, k) {
        ("make", "validity") => "erased-make-guard",
        ("make", "nil") => "nil-result",
        ("make", "zero") => "nonzero-initialization",
        ("make", "fresh-frame") => "erased-freshness",
        ("append", "validity") => {
            if n == 0 {
                "false-empty-validity"
            } else {
                "erased-capacity-guard"
            }
        }
        ("append", "length") => "extra-element",
        ("append", "nil") => "flipped-nil",
        ("append", "capacity") => "zero-capacity",
        ("append", "index" | "alias-index") => "original-length-index",
        ("composed", "validity") => {
            if n == 0 {
                "false-empty-validity"
            } else {
                "erased-capacity-guard"
            }
        }
        ("composed", "nil") => "nil-result",
        ("append", "snapshot") => "poststate-source",
        ("append", "alias-read") => "stale-alias",
        _ => "zeroed-cell",
    }
}
fn replace_assumption(a: &mut [(String, Expr)], from: Expr) {
    for (_, ty) in a {
        *ty = substitute(ty, &from, &eq(literal(true), literal(true)));
    }
}
pub(super) fn generate(base: Certificate, output: &Path, count: usize) {
    let depth = capacity_depth(count);
    let h = finite_heap(depth, v("backing"));
    let mut cases = vec![];
    for (kind, family) in families(count) {
        for mutation in 0..2 {
            let mut p = Producer {
                builder: Builder::from(base.clone()),
                fresh: 0,
            };
            let mut a = vec![("backing".into(), cube_type(1 + depth + ELEMENT))];
            a.extend(if kind == "composed" {
                made::composed::data(family)
            } else if kind == "make" {
                match family {
                    "zero" => made::read_data().into_iter().skip(1).collect(),
                    "other-frame" => made::other_data().into_iter().skip(1).collect(),
                    "fresh-frame" => made::fresh_data().into_iter().skip(1).collect(),
                    _ => made::data(),
                }
            } else {
                match family {
                    "index" | "alias-index" | "read" | "alias-read" | "snapshot" => indices::data(),
                    "prefix-frame" => reads::prefix_data(),
                    "tail-frame" => reads::tail_data(),
                    "other-frame" => reads::other_data(),
                    "physical-frame" => reads::physical_data(),
                    _ => data(),
                }
            });
            let bound = if kind != "append" {
                canon(WORD, v("capacity"))
            } else {
                v("storage_length")
            };
            a.push((
                "storage_bound".into(),
                eq(
                    not(word_less(WORD, literal_word(WORD, count as u64), bound)),
                    literal(true),
                ),
            ));
            let d = v("destination");
            let s = v("source");
            let mut q = if kind == "composed" {
                match family {
                    "validity" => made::composed::result(),
                    "nil" => made::composed::nil_result(),
                    "read" => made::composed::read_result(h.clone()),
                    _ => made::composed::borrowed(h.clone()),
                }
            } else if kind == "make" {
                match family {
                    "validity" => made::validity(),
                    "nil" => ctor_projection(
                        vec![
                            literal(false),
                            v("fresh_region"),
                            literal_word(WORD, 0),
                            canon(WORD, v("length")),
                            canon(WORD, v("capacity")),
                        ],
                        0,
                    ),
                    "zero" => made::zero_read(h.clone()),
                    "other-frame" => made::other(h.clone()),
                    _ => made::fresh_frame(h.clone()),
                }
            } else {
                match family {
                    "validity" => validity::result(),
                    "length" => projection(d.clone(), s.clone(), 3),
                    "nil" => projection(d.clone(), s.clone(), 0),
                    "capacity" => projection(d.clone(), s.clone(), 4),
                    "index" | "alias-index" => indices::index(family == "alias-index"),
                    "read" | "snapshot" => reads::appended_read(h.clone(), false),
                    "prefix-frame" => reads::prefix(h.clone()),
                    "alias-read" => reads::appended_read(h.clone(), true),
                    "tail-frame" => reads::tail(h.clone()),
                    "other-frame" => reads::other(h.clone()),
                    _ => reads::physical_frame(h.clone()),
                }
            };
            let label = if mutation == 0 {
                "valid"
            } else {
                match (kind, family) {
                    ("composed", "validity") => {
                        if count == 0 {
                            q.right = literal(false)
                        } else {
                            replace_assumption(
                                &mut a,
                                eq(
                                    guard(
                                        made::view(v("fresh_region"), v("length"), v("capacity")),
                                        s.clone(),
                                    ),
                                    literal(true),
                                ),
                            )
                        }
                    }
                    ("composed", "nil") => q.right = literal(true),
                    ("make", "validity") => replace_assumption(
                        &mut a,
                        eq(made::guard(v("length"), v("capacity")), literal(true)),
                    ),
                    ("make", "nil") => q.right = literal(true),
                    ("make", "zero") => q.right = one_cell(),
                    ("make", "fresh-frame") => replace_assumption(
                        &mut a,
                        eq(
                            apply(v("live_mask"), vec![v("fresh_region")]),
                            literal(false),
                        ),
                    ),
                    ("append", "validity") => {
                        if count == 0 {
                            q.right = literal(false)
                        } else {
                            replace_assumption(
                                &mut a,
                                eq(guard(d.clone(), s.clone()), literal(true)),
                            )
                        }
                    }
                    ("append", "length") => {
                        q.right = sum(
                            WORD,
                            total(d.clone(), s.clone()).value,
                            literal_word(WORD, 1),
                            literal(false),
                        )
                    }
                    ("append", "nil") => q.right = not(nil(d.clone())),
                    ("append", "capacity") => q.right = literal_word(WORD, 0),
                    ("append", "index" | "alias-index") => {
                        q.left = defined(d.clone(), indices::mapped().value)
                    }
                    ("append", "alias-read") => {
                        q.right = read(h.clone(), full(d.clone()), indices::mapped().value)
                    }
                    ("append", "snapshot") => {
                        q.right = read(
                            updated(h.clone(), d.clone(), s.clone()),
                            s.clone(),
                            v("index"),
                        )
                    }
                    _ => q.right = cell_zero(),
                };
                wrong(kind, family, count)
            };
            let suffix = format!("SliceConstructionClaim{kind}{}", family.replace('-', ""));
            let ty = eq_ty(q.ty, q.left, q.right);
            let (ty, proof) = fold_binders(&a, ty, q.proof);
            p.declare(&suffix, ty.clone(), proof, true);
            let id = format!("construct-{count}-{kind}-{family}-{label}");
            let terms = p.builder.c.term_table.len();
            let declarations = p.builder.c.declarations.len();
            let bytes = p.builder.finish();
            fs::write(output.join(format!("{id}.mpcert")), &bytes).unwrap();
            cases.push(json!({"id":id,"kind":kind,"family":family,"count":count,"capacity":count,"padding_capacity":1usize<<depth,"address_depth":depth,"source_word_bits":64,"extent_intermediate_bits":65,"descriptor_bits":512,"element_bits":128,"logical_region_count":2,"source_pointer_bits_admitted":false,"quantified_storage_bits":128usize<<(depth+1),"mutation":mutation,"expected":if mutation==0 {"accepted"} else {"rejected"},"declarations":declarations,"terms":terms,"bytes":bytes.len(),"theorem":name(&suffix),"scope":"ordinary signed make guards, initialized capacity and computed live-region separation; capacity-preserving append with actual result/alias guards, pre-state reads and absolute write frames; production capture/ownership/remaining operations stay pending","goal_debug":format!("{ty:?}")}));
        }
    }
    fs::write(
        output.join("index.json"),
        serde_json::to_vec_pretty(
            &json!({"schema":"mpk.go_practical.construction_pilot.v0","cases":cases}),
        )
        .unwrap(),
    )
    .unwrap();
}
