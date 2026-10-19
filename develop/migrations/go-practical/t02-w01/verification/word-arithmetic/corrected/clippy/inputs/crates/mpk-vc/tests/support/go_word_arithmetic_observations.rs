use mpk_vc::go_practical_ordinary::generate_go_practical_word_arithmetic_foundation;
use sha2::{Digest, Sha256};

#[path = "go_word_eval.rs"]
#[allow(
    dead_code,
    reason = "Only the shared Boolean interpreter and frozen query schema are needed; the separate full operator oracle remains covered by go_practical_word_foundation."
)]
mod word_eval;

#[test]
fn actual_product_and_every_fixed_shift_stage_match_exact_integer_arithmetic() {
    let raw = include_bytes!(
        "../../../../develop/migrations/go-practical/t01-w04/word-foundation/queries.json"
    );
    let all: Vec<word_eval::Query> = serde_json::from_slice(raw).unwrap();
    let queries = all
        .into_iter()
        .filter(|q| q.operation == "Mul")
        .collect::<Vec<_>>();
    assert_eq!(queries.len(), 48);
    let f = generate_go_practical_word_arithmetic_foundation().unwrap();
    let c = mpk_cert::decode_canonical_certificate(f.certificate_bytes()).unwrap();
    let mut rows = vec![];
    let mut total_steps = 0;
    let mut maximum_steps = 0;
    let mut stages = 0;
    let mut actual_carries = 0;
    for q in queries {
        let (x, y) = (
            u64::from_str_radix(&q.x_hex, 16).unwrap(),
            u64::from_str_radix(&q.y_hex, 16).unwrap(),
        );
        let modulus = 1_u128 << q.width;
        assert!(u128::from(x) < modulus && u128::from(y) < modulus);
        let mut model = word_eval::Interpreter::new(&c);
        let xx = model.word(q.width, x);
        let yy = model.word(q.width, y);
        let false_ = model.invoke("Std.Bool.false", &[]);
        let true_ = model.invoke("Std.Bool.true", &[]);
        let prefix = format!("Mpk.Go.Practical.Word.W{}.", q.width);
        let zero = model.invoke(&(prefix.clone() + "Zero"), &[]);
        let mut acc = model.invoke(&(prefix.clone() + "Mul0"), &[xx, yy]);
        assert_eq!(model.bits(q.width, acc), 0);
        let mut product_discard = 0_u128;
        let mut shift_discard = 0_u128;
        let mut previous_left = 0_u64;
        let mut observations = vec![];
        for i in 0..q.width {
            let left = model.invoke(&(prefix.clone() + &format!("Left{i}")), &[xx]);
            let left_bits = model.bits(q.width, left);
            if i > 0 {
                shift_discard = 2 * shift_discard + u128::from(previous_left >> (q.width - 1));
            }
            previous_left = left_bits;
            assert_eq!(
                u128::from(left_bits) + modulus * shift_discard,
                (1_u128 << i) * u128::from(x),
                "{} shift{i}",
                q.id
            );
            let flag = (y >> i) & 1 != 0;
            let selected = model.invoke(
                &(prefix.clone() + "Select"),
                &[if flag { true_ } else { false_ }, left, zero],
            );
            let carry = model.invoke(
                &(prefix.clone() + &format!("Carry{}", q.width)),
                &[acc, selected, false_],
            );
            let carry = model.boolean(carry);
            acc = model.invoke(&(prefix.clone() + &format!("Mul{}", i + 1)), &[xx, yy]);
            let acc_bits = model.bits(q.width, acc);
            product_discard += u128::from(flag) * shift_discard + u128::from(carry);
            let digit_prefix = if i == 63 {
                y
            } else {
                y & ((1_u64 << (i + 1)) - 1)
            };
            assert_eq!(
                u128::from(acc_bits) + modulus * product_discard,
                u128::from(x) * u128::from(digit_prefix),
                "{} product stage{i}",
                q.id
            );
            actual_carries += usize::from(carry);
            stages += 1;
            observations.push(serde_json::json!({"index":i,"left_hex":format!("{:0width$x}",left_bits,width=usize::from(q.width)/4),"shift_discard":shift_discard.to_string(),"carry":carry,"acc_hex":format!("{:0width$x}",acc_bits,width=usize::from(q.width)/4)}));
        }
        let product = model.invoke(&(prefix + "Mul"), &[xx, yy]);
        let bits = model.bits(q.width, product);
        let exact = u128::from(x) * u128::from(y);
        assert_eq!(u128::from(bits), exact % modulus, "{}", q.id);
        assert_eq!(product_discard, exact / modulus, "{}", q.id);
        let signed = i128::from(bits)
            - if u128::from(bits) >= modulus / 2 {
                modulus as i128
            } else {
                0
            };
        rows.push(serde_json::json!({"id":q.id,"word_hex":format!("{:0width$x}",bits,width=usize::from(q.width)/4),"go_signed_result":signed.to_string(),"unsigned_product":exact.to_string(),"discarded_quotient":product_discard.to_string(),"stages":observations}));
        total_steps += model.steps();
        maximum_steps = maximum_steps.max(model.steps());
    }
    assert_eq!(stages, 1440);
    assert!(actual_carries > 0);
    if let Some(out) = std::env::var_os("MPK_GO_WORD_ARITHMETIC_MODEL_OUTPUT") {
        use std::io::Write;
        let out = std::path::PathBuf::from(out);
        std::fs::create_dir_all(&out).unwrap();
        let metrics = serde_json::json!({"selected_original_Mul_queries":48,"actual_fixed_shift_and_product_stages":stages,"observed_true_carries":actual_carries,"total_AST_steps":total_steps,"maximum_query_AST_steps":maximum_steps,"original_frozen_query_sha256":format!("{:x}",Sha256::digest(raw)),"actual_Boolean_certificate_terms_executed":true,"Nat_witness_declarations_executed":false,"witness_observation_scope":"Scalar recurrence values derived using actual fixed-left bits and actual full-adder carry bits; universal Nat witness declarations are checked independently in both cores. No Boolean-model arithmetic callback or source-program claim."});
        for (name, mut bytes) in [
            ("model.json", serde_json::to_vec(&rows).unwrap()),
            ("metrics.json", serde_json::to_vec_pretty(&metrics).unwrap()),
            ("certificate.mpcert", f.certificate_bytes().to_vec()),
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
    println!("product queries=48 stages={stages} true_carries={actual_carries} AST_steps={total_steps} max={maximum_steps}");
}
