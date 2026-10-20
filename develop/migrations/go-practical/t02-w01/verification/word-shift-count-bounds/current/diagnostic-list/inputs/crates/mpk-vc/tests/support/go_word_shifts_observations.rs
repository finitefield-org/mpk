use mpk_vc::go_practical_ordinary::generate_go_practical_word_shift_foundation;
use sha2::{Digest, Sha256};
#[path = "go_word_eval.rs"]
#[allow(
    dead_code,
    reason = "Only unchanged Bool circuit evaluation and original complete query schema execute."
)]
mod word_eval;
#[test]
fn all_original_shift_rows_match_actual_count_bits_stages_and_fixed_right_circuits() {
    let raw = include_bytes!(
        "../../../../develop/migrations/go-practical/t01-w04/word-foundation/queries.json"
    );
    let qs: Vec<word_eval::Query> = serde_json::from_slice(raw).unwrap();
    let qs = qs
        .into_iter()
        .filter(|q| {
            matches!(
                q.operation.as_str(),
                "ShiftLeft" | "ShiftRight" | "SignedShiftRight"
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(qs.len(), 504);
    let f = generate_go_practical_word_shift_foundation().unwrap();
    let c = mpk_cert::decode_canonical_certificate(f.certificate_bytes()).unwrap();
    let (mut rows, mut steps, mut max_steps, mut faults, mut high_defined, mut fixed) =
        (vec![], 0, 0, 0, 0, 0);
    for q in qs {
        let (w, x, n) = (
            q.width,
            u64::from_str_radix(&q.x_hex, 16).unwrap(),
            u64::from_str_radix(&q.count_hex, 16).unwrap(),
        );
        let d = w.trailing_zeros();
        let mut m = word_eval::Interpreter::new(&c);
        let xx = m.word(w, x);
        let count = m.word(64, n);
        let g = m.invoke(
            &format!("Mpk.Go.Practical.Word.W{w}.SignedCountDefined"),
            &[count],
        );
        let defined = !q.signed_count || m.boolean(g);
        assert_eq!(defined, !q.signed_count || n >> 63 == 0);
        let h = m.invoke(
            &format!("Mpk.Go.Practical.Word.Shifts.CountHigh{d}"),
            &[count],
        );
        let h = m.boolean(h);
        assert_eq!(h, n >= u64::from(w));
        let lo = n % u64::from(w);
        let hi = n / u64::from(w);
        assert_eq!(
            u128::from(n),
            u128::from(lo) + u128::from(w) * u128::from(hi)
        );
        let (result, stages, left) = if defined {
            high_defined += usize::from(h);
            let r = m.invoke(
                &format!("Mpk.Go.Practical.Word.W{w}.{}", q.operation),
                &[xx, count],
            );
            let r = m.bits(w, r);
            let mut stages = vec![];
            for i in 0..=d {
                let s = m.invoke(
                    &format!("Mpk.Go.Practical.Word.W{w}.{}{i}", q.operation),
                    &[xx, count],
                );
                let s = m.bits(w, s);
                stages.push(format!("{s:0a$x}", a = usize::from(w) / 4));
            }
            let full_mask = u64::MAX >> (64 - w);
            for i in 0..d {
                let k = 1 << i;
                for signed in [false, true] {
                    let result = m.invoke(
                        &format!(
                            "Mpk.Go.Practical.Word.W{w}.{}{k}",
                            if signed { "SignedRight" } else { "Right" }
                        ),
                        &[xx],
                    );
                    let result = m.bits(w, result);
                    let sx = i128::from(x) - if x >> (w - 1) != 0 { 1_i128 << w } else { 0 };
                    let expected = if signed {
                        ((sx.div_euclid(1_i128 << k)) as u64) & full_mask
                    } else {
                        x >> k
                    };
                    assert_eq!(result, expected);
                    fixed += 1;
                }
            }
            (
                Some(format!("{r:0a$x}", a = usize::from(w) / 4)),
                Some(stages),
                if q.operation == "ShiftLeft" {
                    Some(format!("{r:0a$x}", a = usize::from(w) / 4))
                } else {
                    None
                },
            )
        } else {
            faults += 1;
            (None, None, None)
        };
        rows.push(serde_json::json!({"id":q.id,"operation":q.operation,"count_defined":defined,"native_count_panicked":!defined,"count_high":h,"low_count":lo.to_string(),"high_value":hi.to_string(),"large_count_excess":if h{Some((n-u64::from(w)).to_string())}else{None},"result_hex":result,"signed_left_result_hex":left,"stage_hex":stages}));
        steps += m.steps();
        max_steps = max_steps.max(m.steps());
    }
    assert_eq!((faults, high_defined), (24, 309));
    if let Some(out) = std::env::var_os("MPK_GO_WORD_SHIFTS_MODEL_OUTPUT") {
        use std::io::Write;
        let out = std::path::PathBuf::from(out);
        std::fs::create_dir_all(&out).unwrap();
        let metrics = serde_json::json!({"original_shift_queries":504,"defined_counts":480,"negative_count_guards":faults,"defined_high_counts":high_defined,"actual_fixed_right_observations":fixed,"total_AST_steps":steps,"maximum_query_AST_steps":max_steps,"original_frozen_query_sha256":format!("{:x}",Sha256::digest(raw)),"Nat_Int_witness_or_theorem_bodies_executed":false,"scope":"Actual source Bool shifts/stages, full64 signed count guards, exact CountHigh trees and real fixed right/ signed-right circuits. Host integers check finite fixed-right values and count decompositions. Universal fixed-right/count arithmetic proofs are checked separately by both cores. Full variable-shift conservation and all application/source/profile binding remain due."});
        for (name, mut bytes) in [
            ("model.json", serde_json::to_vec(&rows).unwrap()),
            ("metrics.json", serde_json::to_vec_pretty(&metrics).unwrap()),
            ("certificate.mpcert", f.certificate_bytes().to_vec()),
        ] {
            if name != "certificate.mpcert" {
                bytes.push(b'\n')
            };
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(out.join(name))
                .unwrap();
            file.write_all(&bytes).unwrap();
        }
    }
}
