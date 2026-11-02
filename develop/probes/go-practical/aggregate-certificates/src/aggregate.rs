//! Simultaneous struct fields, full-width descriptor addresses and byte relations.
//! All original stores and every field/padding bit remain arbitrary.
use super::super::super::super::embedding as descriptor_embedding;
use super::*;

fn width(d: usize) -> usize {
    WORD.max(d + BYTE)
}
fn cell_type(d: usize) -> Expr {
    cube_type(2 + width(d))
}
fn store_type(d: usize) -> Expr {
    forall(
        "__aggregate_region",
        bool_ty(),
        forall("__aggregate_address", word_ty(), cell_type(d)),
    )
}
fn field_view(cell: Expr, tag: usize, bits: usize, d: usize) -> Expr {
    let mut address = vec![literal(tag & 2 != 0), literal(tag & 1 != 0)];
    address.extend(vec![literal(false); width(d) - bits]);
    address.extend(addresses(bits, "__aggregate_field_"));
    cube_lambda(bits, "__aggregate_field_", apply(cell, address))
}
fn sku_frame(cell: Expr, d: usize) -> Expr {
    join(
        1 + width(d),
        field_view(cell.clone(), 1, width(d), d),
        field_view(cell, 2, width(d), d),
    )
}
fn all_zero(bits: &[Expr]) -> Expr {
    bits.iter()
        .rev()
        .fold(literal(true), |h, b| and(not(b.clone()), h))
}
fn replace_amount(cell: Expr, replacement: Expr, d: usize) -> Expr {
    fn replace_low(n: usize, old: Expr, value: Expr) -> Expr {
        if n == WORD {
            value
        } else {
            join(
                n,
                replace_low(n - 1, project(old.clone(), false), value),
                project(old, true),
            )
        }
    }
    let lower = project(cell.clone(), false);
    let amount = replace_low(width(d), project(lower.clone(), false), replacement);
    join(
        2 + width(d),
        join(1 + width(d), amount, project(lower, true)),
        project(cell, true),
    )
}
fn flip_first(ty_depth: usize, x: Expr) -> Expr {
    let bits = addresses(ty_depth, "__aggregate_false_");
    let bit = apply(x, bits.clone());
    cube_lambda(
        ty_depth,
        "__aggregate_false_",
        choose(all_zero(&bits), not(bit.clone()), bit),
    )
}
fn raw_cell(h: Expr, d: Expr, index: Expr) -> Expr {
    apply(
        h,
        vec![region(d.clone()), canon(WORD, address(offset(d), index))],
    )
}
fn projected_cell(h: Expr, descriptor: Expr, index: Expr, capacity: usize) -> Expr {
    apply(
        h,
        vec![
            region(descriptor.clone()),
            canon(
                WORD,
                string_reconstruction(capacity, address(offset(descriptor), index)),
            ),
        ],
    )
}
fn literal_le(p: &mut Producer, a: usize, b: usize) -> Typed {
    let q = descriptor_embedding::less_literal(p, WORD, b as u64, a as u64);
    let e = change(p, reflexive_t(bool_ty(), not(q.left.clone())), q);
    let tail = normalize(e.right.clone(), p);
    let e = next(e, tail);
    assert_eq!(e.right, literal(true));
    e
}
fn bound_to(p: &mut Producer, x: Expr, from: usize, to: usize, h: Expr) -> Typed {
    let lo = literal_word(WORD, from as u64);
    let hi = literal_word(WORD, to as u64);
    let q = literal_le(p, from, to);
    numeric::transitive_le(x, lo, hi, h, q.proof)
}
fn cell_equality(p: &mut Producer, byte_bound: usize, slice_bound: usize, d: usize) -> Typed {
    let descriptor = v("descriptor");
    let i = v("line_index");
    let physical = address(offset(descriptor.clone()), i.clone());
    let pad = 1usize << d;
    let hb = bound_to(p, v("storage_length"), slice_bound, pad, v("storage_bound"));
    let q = typed(
        word_ty(),
        canon(WORD, physical.clone()),
        canon(
            WORD,
            string_reconstruction(byte_bound.max(slice_bound), physical),
        ),
        vc(
            "Coverage",
            vec![
                descriptor.clone(),
                v("storage_length"),
                v("live_mask"),
                i.clone(),
                v("valid"),
                v("line_guard"),
                hb.proof,
            ],
        ),
    );
    let actual = raw_cell(v("heap"), descriptor.clone(), i.clone());
    let finite = projected_cell(
        v("heap"),
        descriptor.clone(),
        i,
        byte_bound.max(slice_bound),
    );
    mapped(
        q,
        cell_type(d),
        lambda(
            "__aggregate_key",
            word_ty(),
            apply(v("heap"), vec![region(descriptor), v("__aggregate_key")]),
        ),
        actual,
        finite,
    )
}
fn heap_arguments(d: usize, slice_bound: usize) -> Vec<(String, Expr)> {
    let descriptor = v("descriptor");
    vec![
        ("heap".into(), store_type(d)),
        (
            "descriptor".into(),
            super::super::super::super::super::descriptor(),
        ),
        ("storage_length".into(), word_ty()),
        (
            "live_mask".into(),
            forall("__aggregate_live", bool_ty(), bool_ty()),
        ),
        (
            "valid".into(),
            eq(
                vc(
                    "Predicate",
                    vec![descriptor.clone(), v("storage_length"), v("live_mask")],
                ),
                literal(true),
            ),
        ),
        (
            "storage_bound".into(),
            eq(
                numeric::le(v("storage_length"), literal_word(WORD, slice_bound as u64)),
                literal(true),
            ),
        ),
        ("line_index".into(), word_ty()),
        (
            "line_guard".into(),
            eq(defined(descriptor, v("line_index")), literal(true)),
        ),
    ]
}
fn matched_read(
    p: &mut Producer,
    suf: bool,
    byte_bound: usize,
    slice_bound: usize,
    d: usize,
    cell: Typed,
    a: &mut Vec<(String, Expr)>,
) -> Typed {
    let actual = cell.left.clone();
    let finite = cell.right.clone();
    let source_length = field_view(actual.clone(), 1, WORD, d);
    let source = finite_lift(d, field_view(actual, 2, d + BYTE, d));
    let projected = finite_lift(d, field_view(finite, 2, d + BYTE, d));
    let pattern = finite_lift(d, v("pattern_backing"));
    let pattern_length = v("pattern_length");
    let common_bound = byte_bound.max(slice_bound);
    a.extend([
        ("pattern_length".into(), word_ty()),
        ("pattern_backing".into(), cube_type(d + BYTE)),
        (
            "source_bound".into(),
            eq(
                numeric::le(
                    canon(WORD, source_length.clone()),
                    literal_word(WORD, byte_bound as u64),
                ),
                literal(true),
            ),
        ),
        (
            "pattern_bound".into(),
            eq(
                numeric::le(
                    canon(WORD, pattern_length.clone()),
                    literal_word(WORD, byte_bound as u64),
                ),
                literal(true),
            ),
        ),
    ]);
    let actual_algorithm = subst_expr(
        algorithm(suf, d, v("source"), v("pattern")),
        &[
            ("source_length", source_length.clone()),
            ("source", source.clone()),
            ("pattern", pattern.clone()),
        ],
    );
    a.push(("source_success".into(), eq(actual_algorithm, literal(true))));
    a.push(("index".into(), word_ty()));
    a.push((
        "index_guard".into(),
        eq(
            active(v("index"), canon(WORD, pattern_length.clone())),
            literal(true),
        ),
    ));
    let hn = numeric::nonnegative(
        canon(WORD, source_length.clone()),
        literal_word(WORD, byte_bound as u64),
        v("source_bound"),
        refl(literal(false)),
    );
    let hp = numeric::nonnegative(
        canon(WORD, pattern_length.clone()),
        literal_word(WORD, byte_bound as u64),
        v("pattern_bound"),
        refl(literal(false)),
    );
    let hb = bound_to(
        p,
        canon(WORD, pattern_length),
        byte_bound,
        common_bound,
        v("pattern_bound"),
    );
    let mut q = emit::proof_call(suf, d, "matched-read");
    q = subst_t(q, &[("matched", source_matched(suf, d).proof)]);
    q = subst_t(
        q,
        &[
            ("source_nonnegative", hn.proof),
            ("pattern_nonnegative", hp.proof),
            ("pattern_bound", hb.proof),
            ("source_length", source_length.clone()),
            ("source", source.clone()),
            ("pattern", pattern),
        ],
    );
    // First transport the physical cell to its full-width finite projection.
    // The source-success premise still refers to the actual, unprojected cell.
    let mut f = q.left.clone();
    let projected_read = substitute(&f, &source, &projected);
    f = substitute(
        &f,
        &source,
        &finite_lift(d, field_view(v("__aggregate_cell"), 2, d + BYTE, d)),
    );
    let t = mapped(
        cell,
        bool_ty(),
        lambda("__aggregate_cell", cell_type(d), f),
        q.left.clone(),
        projected_read,
    );
    q = next(backwards(t), q);
    q
}
pub(super) fn generate(base: Certificate, output: &Path, common_bound: usize) {
    let args = std::env::args().collect::<Vec<_>>();
    let byte_bound = args[4].parse::<usize>().unwrap();
    let slice_bound = args[5].parse::<usize>().unwrap();
    assert_eq!(common_bound, byte_bound.max(slice_bound));
    let d = max_depth(common_bound);
    let mut library = Producer {
        builder: Builder::from(base),
        fresh: 0,
    };
    aggregate_validity(
        &mut library,
        1usize << d,
        if args[1] == "aggregate-trace" {
            Some(output)
        } else {
            None
        },
    );
    if args[1] == "aggregate-trace" {
        fs::write(
            output.join("aggregate-library.mpcert"),
            Builder::from(library.builder.c.clone()).finish(),
        )
        .unwrap();
    }
    let base = library.builder.c;
    let mut families = vec!["layout-frame"];
    if slice_bound > 0 {
        families.extend(["full-cell", "amount", "sku-length"]);
        if byte_bound > 0 {
            families.extend(["prefix-read", "suffix-read"]);
        }
    }
    let mut cases = vec![];
    for family in families {
        for mutation in 0..2 {
            let mut p = Producer {
                builder: Builder::from(base.clone()),
                fresh: 0,
            };
            let mut a;
            let mut q;
            if family == "layout-frame" {
                a = vec![
                    ("cell".into(), cell_type(d)),
                    ("replacement".into(), word_ty()),
                ];
                let left = sku_frame(replace_amount(v("cell"), v("replacement"), d), d);
                let right = sku_frame(v("cell"), d);
                q = typed(
                    cube_type(1 + width(d)),
                    left,
                    right.clone(),
                    refl_t(cube_type(1 + width(d)), right),
                );
                assert_eq!(
                    simplify(&q.left),
                    simplify(&q.right),
                    "aggregate frame endpoints"
                );
            } else {
                a = heap_arguments(d, slice_bound);
                q = cell_equality(&mut p, byte_bound, slice_bound, d);
                if family == "amount" || family == "sku-length" {
                    let tag = usize::from(family == "sku-length");
                    q = mapped(
                        q.clone(),
                        word_ty(),
                        lambda(
                            "__aggregate_project",
                            cell_type(d),
                            field_view(v("__aggregate_project"), tag, WORD, d),
                        ),
                        field_view(q.left, tag, WORD, d),
                        field_view(q.right, tag, WORD, d),
                    );
                } else if family.ends_with("-read") {
                    q = matched_read(
                        &mut p,
                        family == "suffix-read",
                        byte_bound,
                        slice_bound,
                        d,
                        q,
                        &mut a,
                    );
                }
            }
            if mutation == 1 {
                q.right = if family.ends_with("-read") {
                    literal(false)
                } else {
                    flip_first(
                        if family == "layout-frame" {
                            1 + width(d)
                        } else if family == "full-cell" {
                            2 + width(d)
                        } else {
                            WORD
                        },
                        q.right,
                    )
                };
            }
            let claim = format!("AggregateClaim{}", family.replace('-', ""));
            let goal = eq_ty(q.ty, q.left, q.right);
            let (goal, proof) = fold_binders(&a, goal, q.proof);
            p.declare(&claim, goal.clone(), proof, true);
            let id = format!(
                "aggregate-{byte_bound}-{slice_bound}-{family}-{}",
                if mutation == 0 { "valid" } else { "altered" }
            );
            let terms = p.builder.c.term_table.len();
            let declarations = p.builder.c.declarations.len();
            let bytes = p.builder.finish();
            fs::write(output.join(format!("{id}.mpcert")), &bytes).unwrap();
            cases.push(json!({"id":id,"family":family,"byte_bound":byte_bound,"slice_bound":slice_bound,"common_bound":common_bound,"address_depth":d,"padding_capacity":1usize<<d,"cell_depth":2+width(d),"cell_bits":1usize<<(2+width(d)),"source_word_bits":64,"payload_bits":8,"heap_regions":2,"heap_address_bits":64,"observed_finite_heap_bits":2usize*(1usize<<d)*(1usize<<(2+width(d))),"mutation":mutation,"expected":if mutation==0 {"accepted"} else {"rejected"},"declarations":declarations,"terms":terms,"bytes":bytes.len(),"theorem":name(&claim),"scope":"private aggregate struct/descriptor/byte composition; complete stores and cell padding quantified; production source and admission remain pending","goal_debug":format!("{goal:?}")}));
        }
    }
    fs::write(
        output.join("index.json"),
        serde_json::to_vec_pretty(
            &json!({"schema":"mpk.go_practical.aggregate_pilot.v0","cases":cases}),
        )
        .unwrap(),
    )
    .unwrap();
}
