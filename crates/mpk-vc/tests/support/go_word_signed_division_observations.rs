use mpk_vc::go_practical_ordinary::generate_go_practical_word_signed_division_prerequisites;
use sha2::{Digest, Sha256};
#[path = "go_word_eval.rs"]
#[allow(
    dead_code,
    reason = "Only the unchanged Bool interpreter and full original query schema are used."
)]
mod word_eval;
#[test]
fn actual_signed_div_rem_and_scoped_zero_match_original_go_inputs() {
    let raw = include_bytes!(
        "../../../../develop/migrations/go-practical/t01-w04/word-foundation/queries.json"
    );
    let all: Vec<word_eval::Query> = serde_json::from_slice(raw).unwrap();
    let queries = all
        .into_iter()
        .filter(|q| matches!(q.operation.as_str(), "SignedDiv" | "SignedRem"))
        .collect::<Vec<_>>();
    assert_eq!(queries.len(), 96);
    let f = generate_go_practical_word_signed_division_prerequisites().unwrap();
    let c = mpk_cert::decode_canonical_certificate(f.certificate_bytes()).unwrap();
    let (mut rows, mut steps, mut max, mut undefined, mut exceptional) = (vec![], 0, 0, 0, 0);
    for q in queries {
        let x = u64::from_str_radix(&q.x_hex, 16).unwrap();
        let y = u64::from_str_radix(&q.y_hex, 16).unwrap();
        let m = 1_i128 << q.width;
        let half = m / 2;
        let signed = |u: u64| i128::from(u) - if i128::from(u) >= half { m } else { 0 };
        let mut model = word_eval::Interpreter::new(&c);
        let xx = model.word(q.width, x);
        let yy = model.word(q.width, y);
        let p = format!("Mpk.Go.Practical.Word.W{}.", q.width);
        let mut predicates = vec![];
        for word in [xx, yy] {
            let z = model.invoke(&(p.clone() + "IsZero"), &[word]);
            let d = model.invoke(&(p.clone() + "DivideDefined"), &[word]);
            let b = model.invoke(
                &format!(
                    "Mpk.Go.Practical.Word.SignedDivision.Prerequisite.Zero.ZeroBits{}",
                    q.width.trailing_zeros()
                ),
                &[word],
            );
            predicates.push((model.boolean(z), model.boolean(d), model.boolean(b)));
        }
        assert_eq!(
            predicates,
            vec![(x == 0, x != 0, x == 0), (y == 0, y != 0, y == 0)]
        );
        let ay = model.invoke(&(p.clone() + "AbsBits"), &[yy]);
        let az = model.invoke(&(p.clone() + "IsZero"), &[ay]);
        let az = model.boolean(az);
        assert_eq!(az, y == 0);
        let defined = predicates[1].1;
        let (value_hex, result) = if defined {
            let output = model.invoke(&(p + &q.operation), &[xx, yy]);
            let bits = model.bits(q.width, output);
            let expected = if q.operation == "SignedDiv" {
                signed(x) / signed(y)
            } else {
                signed(x) % signed(y)
            };
            assert_eq!(i128::from(bits), expected.rem_euclid(m));
            if signed(x) == -half && signed(y) == -1 {
                if q.operation == "SignedDiv" {
                    assert_eq!(signed(bits), -half);
                } else {
                    assert_eq!(bits, 0);
                }
                exceptional += 1;
            }
            (
                Some(format!("{:0w$x}", bits, w = usize::from(q.width) / 4)),
                Some(signed(bits).to_string()),
            )
        } else {
            undefined += 1;
            (None, None)
        };
        rows.push(serde_json::json!({"kind":"arithmetic","id":q.id,"operation":q.operation,"defined":defined,"panicked":!defined,"value_hex":value_hex,"go_signed_result":result,"x_zero":predicates[0].0,"y_zero":predicates[1].0,"x_defined":predicates[0].1,"y_defined":predicates[1].1,"x_zero_bits":predicates[0].2,"y_zero_bits":predicates[1].2,"x_sign":signed(x)<0,"y_sign":signed(y)<0,"divisor_magnitude_zero":az}));
        steps += model.steps();
        max = max.max(model.steps());
    }
    assert_eq!(undefined, 8);
    assert_eq!(exceptional, 8);
    if let Some(out) = std::env::var_os("MPK_GO_WORD_SIGNED_DIVISION_MODEL_OUTPUT") {
        use std::io::Write;
        let out = std::path::PathBuf::from(out);
        std::fs::create_dir_all(&out).unwrap();
        let metrics = serde_json::json!({"original_signed_queries":96,"defined_results":88,"source_zero_divisor_faults":undefined,"minimum_minus_one_observations":exceptional,"scoped_zero_predicates":192,"total_AST_steps":steps,"maximum_query_AST_steps":max,"original_frozen_query_sha256":format!("{:x}",Sha256::digest(raw)),"Nat_integer_or_theorem_bodies_executed":false,"universal_signed_division_theorem_claimed":false,"scope":"Actual decoded SignedDiv/SignedRem/AbsBits/source guards and fresh scoped ZeroBits execute with Bool-only primitives. Undefined source rows record the actual false divisor guard and compare to native Go panic without evaluating a word result. Minimum/-one wraps normally. Finite behavior does not prove full signed arithmetic/source binding; this component owns complete unsigned/magnitude/half/zero prerequisites only."});
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
    println!("signed original_queries=96 defined=88 source_faults={undefined} minimum_minus_one={exceptional} AST_steps={steps} max={max}");
}
