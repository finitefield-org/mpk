//! Representative conditional cases, extended with observation laws below.
use super::*;
pub(super) fn generate(base: Certificate, output: &Path, count: usize) {
    let mut cases = vec![];
    for suf in [false, true] {
        for family in ["range", "length"] {
            for mutation in 0..2 {
                let mut p = Producer {
                    builder: Builder::from(base.clone()),
                    fresh: 0,
                };
                let mut a = base_args();
                a.push(("guard".into(), eq(guard(), literal(true))));
                let mut q = if family == "range" {
                    range_call(suf)
                } else {
                    length_call(suf)
                };
                if mutation != 0 {
                    if family == "range" {
                        q.right = literal(false);
                    } else {
                        q.right = sum(WORD, q.right, literal_word(WORD, 1), literal(false));
                    }
                }
                let claim = format!("StringAffixClaim{}{family}", label(suf));
                let goal = eq_ty(q.ty, q.left, q.right);
                let (goal, proof) = fold_binders(&a, goal, q.proof);
                p.declare(&claim, goal.clone(), proof, true);
                let id = format!(
                    "affix-{count}-{}-{family}-{}",
                    label(suf).to_lowercase(),
                    if mutation == 0 { "valid" } else { "altered" }
                );
                let terms = p.builder.c.term_table.len();
                let declarations = p.builder.c.declarations.len();
                let bytes = p.builder.finish();
                fs::write(output.join(format!("{id}.mpcert")), &bytes).unwrap();
                cases.push(json!({"id":id,"kind":label(suf).to_lowercase(),"family":family,"count":count,"capacity":count,"mutation":mutation,"expected":if mutation==0{"accepted"}else{"rejected"},"declarations":declarations,"terms":terms,"bytes":bytes.len(),"theorem":name(&claim),"goal_debug":format!("{goal:?}")}));
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
