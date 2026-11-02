use mpk_vc::go_practical_ordinary::generate_go_practical_word_signed_bounds_foundation;
use sha2::{Digest, Sha256};
#[path = "go_word_eval.rs"]
#[allow(
    dead_code,
    reason = "Only the existing actual Boolean interpreter and original query schema are used; the unchanged full operator oracle is not repeated."
)]
mod word_eval;

// Host scalar counterparts of the new balanced Nat definitions. The independent
// Go and Python observations use direct masks/remainders and absolute values.
// No Nat function is a primitive of the actual certificate interpreter.
fn parts(d: u32, bits: u64) -> (u128, u128, u128) {
    if d == 0 {
        return (1, 0, 0);
    }
    let child_width = 1_u32 << (d - 1);
    let weight = 1_u128 << child_width;
    let lo = u128::from(bits) & (weight - 1);
    let (half, low, complement) = parts(d - 1, bits >> child_width);
    (
        weight * half,
        lo + weight * low,
        weight - 1 - lo + weight * complement,
    )
}
#[test]
fn actual_top_bits_magnitudes_and_computed_half_gaps_match_original_inputs() {
    let raw = include_bytes!(
        "../../../../develop/migrations/go-practical/t01-w04/word-foundation/queries.json"
    );
    let all: Vec<word_eval::Query> = serde_json::from_slice(raw).unwrap();
    let queries = all
        .into_iter()
        .filter(|q| q.operation == "Neg")
        .collect::<Vec<_>>();
    assert_eq!(queries.len(), 32);
    let f = generate_go_practical_word_signed_bounds_foundation().unwrap();
    let c = mpk_cert::decode_canonical_certificate(f.certificate_bytes()).unwrap();
    let (mut rows, mut steps, mut maximum, mut minima, mut zeros, mut nonnegative) =
        (vec![], 0, 0, 0, 0, 0);
    for q in queries {
        let x = u64::from_str_radix(&q.x_hex, 16).unwrap();
        let modulus = 1_u128 << q.width;
        let d = q.width.trailing_zeros();
        let mut model = word_eval::Interpreter::new(&c);
        let xx = model.word(q.width, x);
        let p = format!("Mpk.Go.Practical.Word.W{}.", q.width);
        let n = model.invoke(&(p.clone() + "Neg"), &[xx]);
        let a = model.invoke(&(p + "AbsBits"), &[xx]);
        let (word, neg, abs) = (
            model.bits(q.width, xx),
            model.bits(q.width, n),
            model.bits(q.width, a),
        );
        assert_eq!(word, x);
        let mut signs = vec![];
        for value in [xx, n, a] {
            let top = model.invoke(
                &format!("Mpk.Go.Practical.Word.SignedBounds.Top{d}"),
                &[value],
            );
            signs.push(model.boolean(top));
        }
        assert_eq!(
            signs,
            vec![
                x >> (q.width - 1) != 0,
                neg >> (q.width - 1) != 0,
                abs >> (q.width - 1) != 0
            ]
        );
        let (half, low, complement) = parts(d, word);
        assert_eq!(2 * half, modulus);
        assert_eq!(low + complement + 1, half);
        assert_eq!(u128::from(word), low + half * u128::from(signs[0]));
        let gap = if signs[0] { low } else { complement + 1 };
        let signed_x = i128::from(word) - if signs[0] { modulus as i128 } else { 0 };
        assert_eq!(u128::from(abs) + gap, half);
        assert!(u128::from(abs) <= half);
        assert_eq!(u128::from(abs) == half, u128::from(word) == half);
        assert_eq!(signs[2], u128::from(word) == half);
        assert_eq!(u128::from(word) < half, !signs[0]);
        if !signs[0] {
            assert!(u128::from(abs) < half);
            nonnegative += 1;
        }
        if u128::from(word) == half {
            assert!(signs[0] && signs[2] && gap == 0);
            minima += 1;
        }
        zeros += usize::from(word == 0);
        rows.push(serde_json::json!({"id":q.id,"word_hex":format!("{:0w$x}",word,w=usize::from(q.width)/4),"neg_hex":format!("{:0w$x}",neg,w=usize::from(q.width)/4),"abs_hex":format!("{:0w$x}",abs,w=usize::from(q.width)/4),"signed_x":signed_x.to_string(),"half":half.to_string(),"modulus":modulus.to_string(),"low":low.to_string(),"lower_complement":complement.to_string(),"gap":gap.to_string(),"source_sign":signs[0],"negation_sign":signs[1],"magnitude_sign":signs[2],"unsigned_below_half":u128::from(word)<half,"magnitude_half_bound":u128::from(abs)<=half,"magnitude_equals_half":u128::from(abs)==half,"input_is_minimum":u128::from(word)==half}));
        steps += model.steps();
        maximum = maximum.max(model.steps());
    }
    assert_eq!(minima, 4);
    assert_eq!(zeros, 4);
    if let Some(out) = std::env::var_os("MPK_GO_WORD_SIGNED_BOUNDS_MODEL_OUTPUT") {
        use std::io::Write;
        let out = std::path::PathBuf::from(out);
        std::fs::create_dir_all(&out).unwrap();
        let metrics = serde_json::json!({"selected_original_queries":32,"actual_word_observations":96,"actual_Top_definition_applications":96,"actual_word_bit_observations":2880,"minimum_magnitude_cases":minima,"zero_cases":zeros,"nonnegative_cases":nonnegative,"total_AST_steps":steps,"maximum_query_AST_steps":maximum,"original_frozen_query_sha256":format!("{:x}",Sha256::digest(raw)),"actual_Boolean_terms_executed":true,"Nat_half_low_complement_gap_guard_or_theorem_bodies_executed":false,"scope":"Actual decoded source/Neg/AbsBits word bits and new Top_d functions execute with unchanged Bool-only primitives. Host recurrences derive scalar half/low/complement/gap counterparts. Independent native Go uses typed direct masks and math-big absolute values; independent Python uses remainder/subtraction and half-minus-absolute gap. Universal Nat definitions/bounds/minimum/threshold laws check separately in both cores. No huge unary Nat, theorem execution, source/application proof or Linux release acceptance."});
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
    println!("signed bounds queries=32 actual_words=96 actual_Top_calls=96 bits=2880 minimum={minima} zero={zeros} nonnegative={nonnegative} AST_steps={steps} max={maximum}");
}
