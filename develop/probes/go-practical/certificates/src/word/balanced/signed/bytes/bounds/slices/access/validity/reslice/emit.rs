//! Complete conditional operation candidates, with actual guard erasures.
use super::*;

fn vars() -> [Expr; 4] {
    [v("descriptor"), v("lower"), v("upper"), v("maximum")]
}
fn changed(ty: &Expr, pattern: &Expr) -> Expr {
    substitute(ty, pattern, &eq(literal(true), literal(true)))
}
pub(super) fn generate(base: Certificate, output: &Path, count: usize) {
    let depth = capacity_depth(count);
    let h = finite_heap(depth, v("backing"));
    let mut cases = vec![];
    for three in [false, true] {
        for family in ["validity", "capacity", "mapped-guard", "read"] {
            if count == 0 && family != "validity" {
                continue;
            }
            for mutation in 0..2 {
                let mut p = Producer {
                    builder: Builder::from(base.clone()),
                    fresh: 0,
                };
                let [d, l, hi, m] = vars();
                let g = model::guard_facts(
                    three,
                    d.clone(),
                    l.clone(),
                    hi.clone(),
                    m.clone(),
                    v("range_guard"),
                );
                let mut a = vec![("backing".into(), cube_type(1 + depth + ELEMENT))];
                a.extend(if matches!(family, "mapped-guard" | "read") {
                    readable::data(three)
                } else {
                    preservation::data(three)
                });
                a.push((
                    "storage_bound".into(),
                    eq(
                        not(word_less(
                            WORD,
                            literal_word(WORD, count as u64),
                            v("storage_length"),
                        )),
                        literal(true),
                    ),
                ));
                let mut q = match family {
                    "validity" => preservation::validity(
                        three,
                        vars(),
                        [v("storage_length"), v("live_mask")],
                        [v("old_valid"), v("range_guard")],
                    ),
                    "capacity" => {
                        model::projection(three, 4, d.clone(), l.clone(), hi.clone(), m.clone())
                    }
                    "mapped-guard" => readable::lift(three),
                    _ => readable::read_law(three, h.clone()),
                };
                if mutation == 1 {
                    match family {
                        "validity" => {
                            let target = eq(
                                model::guard(three, d.clone(), l.clone(), hi.clone(), m.clone()),
                                literal(true),
                            );
                            for (_, ty) in &mut a {
                                *ty = changed(ty, &target);
                            }
                        }
                        "capacity" => {
                            let maximum = if three {
                                g.capacity.value.clone()
                            } else {
                                g.high.value.clone()
                            };
                            q.right = numeric::difference(maximum, g.low.value.clone()).value;
                        }
                        "mapped-guard" => {
                            q.left = defined(
                                d.clone(),
                                numeric::addition(g.low.value.clone(), canon(WORD, v("index")))
                                    .value,
                            );
                        }
                        _ => q.right = cell_zero(),
                    }
                }
                let kind = if three { "three" } else { "two" };
                let label = if mutation == 0 {
                    "valid"
                } else {
                    match family {
                        "validity" => "erased-range-guard",
                        "capacity" => "wrong-capacity",
                        "mapped-guard" => "original-length-guard",
                        _ => "zeroed-read",
                    }
                };
                let suffix = format!("SliceResliceClaim{kind}{}", family.replace('-', ""));
                let ty = eq_ty(q.ty.clone(), q.left, q.right);
                let (ty, proof) = fold_binders(&a, ty, q.proof);
                p.declare(&suffix, ty.clone(), proof, true);
                let id = format!("reslice-{count}-{kind}-{family}-{label}");
                let terms = p.builder.c.term_table.len();
                let declarations = p.builder.c.declarations.len();
                let bytes = p.builder.finish();
                fs::write(output.join(format!("{id}.mpcert")), &bytes).unwrap();
                cases.push(json!({"id":id,"kind":kind,"family":family,"count":count,"capacity":count,"padding_capacity":1usize<<depth,"address_depth":depth,"source_word_bits":64,"extent_intermediate_bits":65,"descriptor_bits":512,"element_bits":128,"logical_region_count":2,"source_pointer_bits_admitted":false,"quantified_storage_bits":128usize<<(depth+1),"mutation":mutation,"expected":if mutation==0 {"accepted"} else {"rejected"},"declarations":declarations,"terms":terms,"bytes":bytes.len(),"theorem":name(&suffix),"scope":"ordinary actual signed two/three-index guards preserve computed validity, capacity and full-index/backing reads; production Go capture/ownership/remaining operations stay pending","goal_debug":format!("{ty:?}")}));
            }
        }
    }
    fs::write(
        output.join("index.json"),
        serde_json::to_vec_pretty(
            &json!({"schema":"mpk.go_practical.reslice_pilot.v0","cases":cases}),
        )
        .unwrap(),
    )
    .unwrap();
}
