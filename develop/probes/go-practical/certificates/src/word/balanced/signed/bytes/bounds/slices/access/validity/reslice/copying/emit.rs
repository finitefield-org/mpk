//! Full conditional copy candidates and genuine mutation counterexamples.
use super::*;

pub(super) fn families(n: usize) -> Vec<&'static str> {
    let mut a = vec!["count"];
    if n > 0 {
        a.extend([
            "destination-guard",
            "source-guard",
            "copied-read",
            "frame",
            "other-frame",
        ]);
    }
    if n >= 8 {
        a.push("snapshot");
    }
    a
}
pub(super) fn generate(base: Certificate, output: &Path, count: usize) {
    let depth = capacity_depth(count);
    let h = finite_heap(depth, v("backing"));
    let mut cases = vec![];
    for family in families(count) {
        for mutation in 0..2 {
            let mut p = Producer {
                builder: Builder::from(base.clone()),
                fresh: 0,
            };
            let mut a = vec![("backing".into(), cube_type(1 + depth + ELEMENT))];
            a.extend(match family {
                "count" => counts::data(),
                "frame" => store::frame_data(),
                "other-frame" => {
                    let mut a = counts::data();
                    a.push(("index".into(), word_ty()));
                    a
                }
                _ => counts::access_data(),
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
                "count" => counts::metadata(),
                "destination-guard" | "source-guard" => counts::access(
                    family == "destination-guard",
                    v("destination"),
                    v("source"),
                    [v("storage_length"), v("live_mask")],
                    [v("destination_valid"), v("source_valid")],
                    v("index"),
                    v("copied_guard"),
                ),
                "frame" => store::frame(h.clone()),
                "other-frame" => store::other(h.clone(), v("destination"), v("source"), v("index")),
                _ => store::copied_read(h.clone()),
            };
            let label = if mutation == 0 {
                "valid"
            } else {
                match family {
                    "count" => {
                        q.left = substitute(
                            &q.left,
                            &counts::count(v("destination"), v("source")),
                            &literal_word(WORD, 1),
                        );
                        "count-one"
                    }
                    "destination-guard" | "source-guard" => {
                        let target = eq(
                            active(v("index"), counts::count(v("destination"), v("source"))),
                            literal(true),
                        );
                        for (_, ty) in &mut a {
                            *ty = substitute(ty, &target, &eq(literal(true), literal(true)));
                        }
                        "erased-copy-guard"
                    }
                    "snapshot" => {
                        q.right = read(
                            store::bulk(h.clone(), v("destination"), v("source")),
                            v("source"),
                            v("index"),
                        );
                        "poststate-source"
                    }
                    _ => {
                        q.right = cell_zero();
                        "zeroed-cell"
                    }
                }
            };
            let suffix = format!("SliceBulkCopyClaim{}", family.replace('-', ""));
            let ty = eq_ty(q.ty, q.left, q.right);
            let (ty, proof) = fold_binders(&a, ty, q.proof);
            p.declare(&suffix, ty.clone(), proof, true);
            let id = format!("copy-{count}-{family}-{label}");
            let terms = p.builder.c.term_table.len();
            let declarations = p.builder.c.declarations.len();
            let bytes = p.builder.finish();
            fs::write(output.join(format!("{id}.mpcert")), &bytes).unwrap();
            cases.push(json!({"id":id,"family":family,"count":count,"capacity":count,"padding_capacity":1usize<<depth,"address_depth":depth,"source_word_bits":64,"extent_intermediate_bits":65,"descriptor_bits":512,"element_bits":128,"logical_region_count":2,"source_pointer_bits_admitted":false,"quantified_storage_bits":128usize<<(depth+1),"mutation":mutation,"expected":if mutation==0 {"accepted"} else {"rejected"},"declarations":declarations,"terms":terms,"bytes":bytes.len(),"theorem":name(&suffix),"scope":"ordinary min-length copy with computed source/destination guards, immutable pre-state reads and whole-cell frames; production capture/ownership/remaining operations stay pending","goal_debug":format!("{ty:?}")}));
        }
    }
    fs::write(
        output.join("index.json"),
        serde_json::to_vec_pretty(
            &json!({"schema":"mpk.go_practical.copy_pilot.v0","cases":cases}),
        )
        .unwrap(),
    )
    .unwrap();
}
