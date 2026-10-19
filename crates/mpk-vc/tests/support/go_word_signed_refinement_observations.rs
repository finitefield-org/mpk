use mpk_vc::go_practical_ordinary::generate_go_practical_word_signed_division_refinement;
use sha2::{Digest, Sha256};
#[path = "go_word_eval.rs"]
#[allow(
    dead_code,
    reason = "Use only the unchanged Bool interpreter and original complete query schema."
)]
mod word_eval;

fn top_name(width: u8) -> String {
    format!(
        "Mpk.Go.Practical.Word.SignedDivision.Prerequisite.Bounds.Top{}",
        width.trailing_zeros()
    )
}
fn observed(
    model: &mut word_eval::Interpreter<'_>,
    query: &word_eval::Query,
    kind: &str,
    amount: u32,
    flag: u32,
    result: u32,
) -> serde_json::Value {
    let prefix = format!("Mpk.Go.Practical.Word.W{}.", query.width);
    let negated = model.invoke(&(prefix.clone() + "Neg"), &[amount]);
    let a = model.bits(query.width, amount);
    let n = model.bits(query.width, negated);
    let r = model.bits(query.width, result);
    let at = model.invoke(&top_name(query.width), &[amount]);
    let nt = model.invoke(&top_name(query.width), &[negated]);
    let dd = model.invoke(&(prefix + "DivideDefined"), &[amount]);
    let flag = model.boolean(flag);
    let positive = i64::from(model.boolean(if flag { nt } else { at }));
    let negative = i64::from(flag && model.boolean(dd));
    assert_eq!(model.boolean(dd), a != 0);
    let modulus = 1_i128 << query.width;
    let half = modulus / 2;
    let signed_result = i128::from(r) - if i128::from(r) >= half { modulus } else { 0 };
    let mathematical = if flag { -i128::from(a) } else { i128::from(a) };
    assert_eq!(
        mathematical,
        signed_result + modulus * i128::from(positive - negative)
    );
    let format = |value: u64| format!("{:0w$x}", value, w = usize::from(query.width) / 4);
    serde_json::json!({"kind":kind,"id":query.id,"operation":query.operation,"flag":flag,"defined":true,"panicked":false,"amount_hex":format(a),"negated_amount_hex":format(n),"result_hex":format(r),"mathematical_result":mathematical.to_string(),"go_signed_result":signed_result.to_string(),"correction_positive":positive,"correction_negative":negative,"correction":positive-negative})
}

#[test]
fn actual_sign_selection_and_source_quotient_remainder_corrections_match_original_words() {
    let raw = include_bytes!(
        "../../../../develop/migrations/go-practical/t01-w04/word-foundation/queries.json"
    );
    let all: Vec<word_eval::Query> = serde_json::from_slice(raw).unwrap();
    let queries = all
        .into_iter()
        .filter(|q| matches!(q.operation.as_str(), "Neg" | "SignedDiv" | "SignedRem"))
        .collect::<Vec<_>>();
    assert_eq!(queries.len(), 128);
    let foundation = generate_go_practical_word_signed_division_refinement().unwrap();
    let certificate =
        mpk_cert::decode_canonical_certificate(foundation.certificate_bytes()).unwrap();
    let mut rows = vec![];
    let (mut steps, mut max, mut faults, mut minimum_source_cases) = (0, 0, 0, 0);
    let mut generic_corrections = std::collections::BTreeMap::<i64, u64>::new();
    let mut source_corrections = std::collections::BTreeMap::<i64, u64>::new();
    for query in queries {
        let x = u64::from_str_radix(&query.x_hex, 16).unwrap();
        let y = u64::from_str_radix(&query.y_hex, 16).unwrap();
        let mut model = word_eval::Interpreter::new(&certificate);
        let xx = model.word(query.width, x);
        let yy = model.word(query.width, y);
        let prefix = format!("Mpk.Go.Practical.Word.W{}.", query.width);
        if query.operation == "Neg" {
            let negated = model.invoke(&(prefix.clone() + "Neg"), &[xx]);
            for flag in [false, true] {
                let flag_value = model.invoke(
                    if flag {
                        "Std.Bool.true"
                    } else {
                        "Std.Bool.false"
                    },
                    &[],
                );
                let selected =
                    model.invoke(&(prefix.clone() + "Select"), &[flag_value, negated, xx]);
                let row = observed(&mut model, &query, "selection", xx, flag_value, selected);
                *generic_corrections
                    .entry(row["correction"].as_i64().unwrap())
                    .or_default() += 1;
                rows.push(row);
            }
        } else {
            let sx = model.invoke(&top_name(query.width), &[xx]);
            let sy = model.invoke(&top_name(query.width), &[yy]);
            assert_eq!(model.boolean(sx), x >> (query.width - 1) != 0);
            assert_eq!(model.boolean(sy), y >> (query.width - 1) != 0);
            let flag = if query.operation == "SignedDiv" {
                model.invoke("Mpk.Go.Practical.Bool.Xor", &[sx, sy])
            } else {
                sx
            };
            let guard = model.invoke(&(prefix.clone() + "DivideDefined"), &[yy]);
            let defined = model.boolean(guard);
            assert_eq!(defined, y != 0);
            if defined {
                let ax = model.invoke(&(prefix.clone() + "AbsBits"), &[xx]);
                let ay = model.invoke(&(prefix.clone() + "AbsBits"), &[yy]);
                let operation = if query.operation == "SignedDiv" {
                    "UnsignedDiv"
                } else {
                    "UnsignedRem"
                };
                let amount = model.invoke(&(prefix.clone() + operation), &[ax, ay]);
                let negated = model.invoke(&(prefix.clone() + "Neg"), &[amount]);
                let selected = model.invoke(&(prefix.clone() + "Select"), &[flag, negated, amount]);
                let source = model.invoke(&(prefix + &query.operation), &[xx, yy]);
                assert_eq!(
                    model.bits(query.width, selected),
                    model.bits(query.width, source)
                );
                let row = observed(&mut model, &query, "arithmetic", amount, flag, source);
                let correction = row["correction"].as_i64().unwrap();
                *source_corrections.entry(correction).or_default() += 1;
                let half = 1_u64 << (query.width - 1);
                let minus_one = if query.width == 64 {
                    u64::MAX
                } else {
                    (1_u64 << query.width) - 1
                };
                let minimum = x == half && y == minus_one;
                assert_eq!(correction == 1, minimum && query.operation == "SignedDiv");
                assert!([0, 1].contains(&correction));
                minimum_source_cases += usize::from(minimum);
                rows.push(row);
            } else {
                faults += 1;
                rows.push(serde_json::json!({"kind":"arithmetic","id":query.id,"operation":query.operation,"flag":model.boolean(flag),"defined":false,"panicked":true,"amount_hex":null,"negated_amount_hex":null,"result_hex":null,"mathematical_result":null,"go_signed_result":null,"correction_positive":null,"correction_negative":null,"correction":null}));
            }
        }
        steps += model.steps();
        max = max.max(model.steps());
    }
    assert_eq!(rows.len(), 160);
    assert_eq!(faults, 8);
    assert_eq!(minimum_source_cases, 8);
    assert_eq!(
        generic_corrections.keys().copied().collect::<Vec<_>>(),
        vec![-1, 0, 1]
    );
    assert_eq!(source_corrections, [(0, 84), (1, 4)].into_iter().collect());
    if let Some(out) = std::env::var_os("MPK_GO_WORD_SIGNED_REFINEMENT_MODEL_OUTPUT") {
        use std::io::Write;
        let out = std::path::PathBuf::from(out);
        std::fs::create_dir_all(&out).unwrap();
        let metrics = serde_json::json!({"original_queries":128,"generic_sign_choices":64,"source_operator_queries":96,"observations":160,"defined_source_results":88,"source_zero_divisor_faults":faults,"minimum_minus_one_source_cases":minimum_source_cases,"generic_corrections":generic_corrections,"source_corrections":source_corrections,"total_AST_steps":steps,"maximum_query_AST_steps":max,"original_frozen_query_sha256":format!("{:x}",Sha256::digest(raw)),"Nat_Int_correction_or_theorem_bodies_executed":false,"universal_correction_classification_claimed":false,"scope":"Actual decoded Select/Neg, Abs/unsigned magnitude quotient or remainder/source SignedDiv or SignedRem, Top flags and actual divisor guards execute with the unchanged Bool-only interpreter and its original ceilings. Host derives small balanced correction parts from these actual flags; Nat/Int/correction/theorem bodies do not execute. Every full original row remains exact. Generic -1/0/+1 corrections and finite minimum/-one source behavior are observations; universal zero/minimum correction classification and source/application foundation remain due."});
        for (name, mut bytes) in [
            ("model.json", serde_json::to_vec(&rows).unwrap()),
            ("metrics.json", serde_json::to_vec_pretty(&metrics).unwrap()),
            (
                "certificate.mpcert",
                foundation.certificate_bytes().to_vec(),
            ),
        ] {
            if name != "certificate.mpcert" {
                bytes.push(b'\n');
            }
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(out.join(name))
                .unwrap();
            file.write_all(&bytes).unwrap();
        }
    }
    println!("signed refinement original_queries=128 observations=160 generic=64 source=96 faults={faults} minimum_minus_one={minimum_source_cases} AST_steps={steps} max={max}");
}
