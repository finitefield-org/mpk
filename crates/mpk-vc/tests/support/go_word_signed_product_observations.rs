use mpk_vc::go_practical_ordinary::generate_go_practical_word_signed_product_foundation;
use sha2::{Digest, Sha256};

#[path = "go_word_eval.rs"]
#[allow(
    dead_code,
    reason = "Only the shared Boolean interpreter and frozen query schema are needed; the separate full operator oracle remains covered by go_practical_word_foundation."
)]
mod word_eval;

#[test]
fn actual_signed_product_correction_and_exact_guard_match_all_frozen_inputs() {
    let raw = include_bytes!(
        "../../../../develop/migrations/go-practical/t01-w04/word-foundation/queries.json"
    );
    let all: Vec<word_eval::Query> = serde_json::from_slice(raw).unwrap();
    let queries = all
        .into_iter()
        .filter(|q| q.operation == "Mul")
        .collect::<Vec<_>>();
    assert_eq!(queries.len(), 48);
    let f = generate_go_practical_word_signed_product_foundation().unwrap();
    let c = mpk_cert::decode_canonical_certificate(f.certificate_bytes()).unwrap();
    let mut rows = vec![];
    let mut total_steps = 0;
    let mut maximum_steps = 0;
    let mut stages = 0;
    let mut actual_carries = 0;
    let mut exact_products = 0;
    let mut minimum_minus_one = 0;
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
        let sx = x >> (q.width - 1) != 0;
        let sy = y >> (q.width - 1) != 0;
        let sr = bits >> (q.width - 1) != 0;
        let signed_x = i128::from(x) - if sx { modulus as i128 } else { 0 };
        let signed_y = i128::from(y) - if sy { modulus as i128 } else { 0 };
        let signed_product = signed_x * signed_y;
        let positive = product_discard + u128::from(sr) + modulus * u128::from(sx && sy);
        let negative = u128::from(x) * u128::from(sy) + u128::from(y) * u128::from(sx);
        let correction = positive as i128 - negative as i128;
        let no_overflow = positive == negative;
        assert_eq!(signed_product, signed + (modulus as i128) * correction);
        assert_eq!(no_overflow, signed_product == signed);
        exact_products += usize::from(no_overflow);
        if u128::from(x) == modulus / 2 && u128::from(y) == modulus - 1 {
            assert!(!no_overflow);
            assert_eq!(correction, 1);
            minimum_minus_one += 1;
        }
        rows.push(serde_json::json!({"id":q.id,"word_hex":format!("{:0width$x}",bits,width=usize::from(q.width)/4),"go_signed_result":signed.to_string(),"unsigned_product":exact.to_string(),"discarded_quotient":product_discard.to_string(),"signed_x":signed_x.to_string(),"signed_y":signed_y.to_string(),"signed_product":signed_product.to_string(),"correction_positive":positive.to_string(),"correction_negative":negative.to_string(),"signed_correction":correction.to_string(),"no_overflow":no_overflow,"stages":observations}));
        total_steps += model.steps();
        maximum_steps = maximum_steps.max(model.steps());
    }
    assert_eq!(stages, 1440);
    assert!(actual_carries > 0);
    assert_eq!(minimum_minus_one, 4);
    if let Some(out) = std::env::var_os("MPK_GO_WORD_SIGNED_PRODUCT_MODEL_OUTPUT") {
        use std::io::Write;
        let out = std::path::PathBuf::from(out);
        std::fs::create_dir_all(&out).unwrap();
        let metrics = serde_json::json!({"selected_original_Mul_queries":48,"actual_fixed_shift_and_product_stages":stages,"observed_true_carries":actual_carries,"exact_signed_products":exact_products,"overflowed_signed_products":48-exact_products,"minimum_times_minus_one_cases":minimum_minus_one,"total_AST_steps":total_steps,"maximum_query_AST_steps":maximum_steps,"original_frozen_query_sha256":format!("{:x}",Sha256::digest(raw)),"actual_Boolean_certificate_terms_executed":true,"Nat_witness_correction_guard_or_theorem_bodies_executed":false,"scope":"Actual Left_i/Mul_i/full-adder carries and result/sign bits execute with Bool-only primitives. Host derives unsigned discard recurrence and both natural correction parts, signed correction and exact-product predicate. Universal actual Nat correction/guard equivalence terms check separately in both cores; no arithmetic callback, source-program or unary Nat execution."});
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
    println!("signed product queries=48 stages={stages} exact={exact_products} overflow={} minimum_minus_one=4 true_carries={actual_carries} AST_steps={total_steps} max={maximum_steps}",48-exact_products);
}
