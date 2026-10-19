use mpk_vc::go_practical_ordinary::generate_go_practical_word_signed_magnitude_foundation;
use sha2::{Digest, Sha256};
#[path = "go_word_eval.rs"]
#[allow(
    dead_code,
    reason = "Only the actual Boolean interpreter and original query schema are used; the old full operator oracle is unchanged."
)]
mod word_eval;

#[test]
fn actual_negation_magnitude_carries_signs_and_guards_match_original_inputs() {
    let raw = include_bytes!(
        "../../../../develop/migrations/go-practical/t01-w04/word-foundation/queries.json"
    );
    let all: Vec<word_eval::Query> = serde_json::from_slice(raw).unwrap();
    let queries = all
        .into_iter()
        .filter(|q| q.operation == "Neg")
        .collect::<Vec<_>>();
    assert_eq!(queries.len(), 32);
    let f = generate_go_practical_word_signed_magnitude_foundation().unwrap();
    let c = mpk_cert::decode_canonical_certificate(f.certificate_bytes()).unwrap();
    let (mut rows, mut steps, mut maximum, mut stages, mut minima, mut zeros, mut exact) =
        (vec![], 0, 0, 0, 0, 0, 0);
    for q in queries {
        let x = u64::from_str_radix(&q.x_hex, 16).unwrap();
        let modulus = 1_u128 << q.width;
        let mask = (modulus - 1) as u64;
        let mut model = word_eval::Interpreter::new(&c);
        let xx = model.word(q.width, x);
        let tr = model.invoke("Std.Bool.true", &[]);
        let p = format!("Mpk.Go.Practical.Word.W{}.", q.width);
        let zero = model.invoke(&(p.clone() + "Zero"), &[]);
        let nx = model.invoke(&(p.clone() + "Not"), &[xx]);
        let n = model.invoke(&(p.clone() + "Neg"), &[xx]);
        let a = model.invoke(&(p.clone() + "AbsBits"), &[xx]);
        let (notx, neg, abs) = (
            model.bits(q.width, nx),
            model.bits(q.width, n),
            model.bits(q.width, a),
        );
        assert_eq!(notx, (!x) & mask);
        assert_eq!(neg, x.wrapping_neg() & mask);
        let mut observations = vec![];
        for i in 0..q.width {
            let input = model.invoke(&(p.clone() + &format!("Carry{i}")), &[nx, zero, tr]);
            let output = model.invoke(&(p.clone() + &format!("Carry{}", i + 1)), &[nx, zero, tr]);
            let (cin, cout) = (model.boolean(input), model.boolean(output));
            let bit = ((neg >> i) & 1) != 0;
            let aa = ((notx >> i) & 1) != 0;
            assert_eq!(
                u8::from(aa) + u8::from(cin),
                u8::from(bit) + 2 * u8::from(cout)
            );
            observations.push(serde_json::json!({"index":i,"complement_bit":aa,"carry_in":cin,"result_bit":bit,"carry_out":cout}));
            stages += 1;
        }
        let co = model.invoke(&(p.clone() + &format!("Carry{}", q.width)), &[nx, zero, tr]);
        let co = model.boolean(co);
        let mut observed = vec![];
        for word in [xx, n, a] {
            let z = model.invoke(&(p.clone() + "IsZero"), &[word]);
            let d = model.invoke(&(p.clone() + "DivideDefined"), &[word]);
            observed.push((model.boolean(z), model.boolean(d)));
        }
        let sx = x >> (q.width - 1) != 0;
        let sn = neg >> (q.width - 1) != 0;
        let signed_x = i128::from(x) - if sx { modulus as i128 } else { 0 };
        let signed_neg = i128::from(neg) - if sn { modulus as i128 } else { 0 };
        let cp = u8::from(sx) + u8::from(sn);
        let cn = u8::from(observed[0].1);
        let correction = i128::from(cp) - i128::from(cn);
        let guard = cp == cn;
        assert_eq!(
            u128::from(x) + u128::from(neg) + modulus * u128::from(co),
            modulus
        );
        assert_eq!(co, x == 0);
        assert_eq!(observed, vec![(x == 0, x != 0); 3]);
        assert_eq!(abs, if sx { neg } else { x });
        assert_eq!(i128::from(abs), signed_x.abs());
        assert_eq!(-signed_x, signed_neg + (modulus as i128) * correction);
        assert_eq!(guard, -signed_x == signed_neg);
        if x as u128 == modulus / 2 {
            assert!(!guard && correction == 1);
            minima += 1;
        }
        zeros += usize::from(x == 0);
        exact += usize::from(guard);
        rows.push(serde_json::json!({"id":q.id,"neg_hex":format!("{:0w$x}",neg,w=usize::from(q.width)/4),"abs_hex":format!("{:0w$x}",abs,w=usize::from(q.width)/4),"signed_x":signed_x.to_string(),"go_signed_negation":signed_neg.to_string(),"exact_negation":(-signed_x).to_string(),"absolute_value":signed_x.abs().to_string(),"carry":co,"input_zero":observed[0].0,"negation_zero":observed[1].0,"magnitude_zero":observed[2].0,"input_defined":observed[0].1,"negation_defined":observed[1].1,"magnitude_defined":observed[2].1,"correction_positive":cp,"correction_negative":cn,"signed_correction":correction.to_string(),"no_overflow":guard,"stages":observations}));
        steps += model.steps();
        maximum = maximum.max(model.steps());
    }
    assert_eq!(minima, 4);
    assert_eq!(zeros, 4);
    assert_eq!(stages, 960);
    assert_eq!(exact, 28);
    if let Some(out) = std::env::var_os("MPK_GO_WORD_SIGNED_MAGNITUDE_MODEL_OUTPUT") {
        use std::io::Write;
        let out = std::path::PathBuf::from(out);
        std::fs::create_dir_all(&out).unwrap();
        let metrics = serde_json::json!({"selected_original_Neg_queries":32,"actual_full_adder_stages":stages,"minimum_negation_wrap_cases":minima,"zero_cases":zeros,"exact_negations":exact,"total_AST_steps":steps,"maximum_query_AST_steps":maximum,"original_frozen_query_sha256":format!("{:x}",Sha256::digest(raw)),"actual_Boolean_certificate_terms_executed":true,"Nat_correction_guard_or_theorem_bodies_executed":false,"scope":"Execute actual Neg, AbsBits, every complement-plus-one carry, IsZero and DivideDefined with only Bool primitives. Host derives actual sign bits, both correction parts and mathematical exact-result predicate. Universal ordinary Nat/Int laws check separately in both cores; no large unary Nat or theorem execution and no source/program or Linux-release claim."});
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
    println!("signed magnitude queries=32 stages={stages} minimum_wrap={minima} zero={zeros} exact={exact} AST_steps={steps} max={maximum}");
}
