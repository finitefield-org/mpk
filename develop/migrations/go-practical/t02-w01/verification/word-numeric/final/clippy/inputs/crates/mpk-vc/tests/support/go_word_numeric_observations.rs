use mpk_vc::go_practical_ordinary::generate_go_practical_word_numeric_foundation;
use serde::Deserialize;
use sha2::{Digest, Sha256};

#[path = "go_word_eval.rs"]
#[allow(
    dead_code,
    reason = "This test uses the shared Boolean interpreter; its separate word-operation oracle is exercised by go_practical_word_foundation."
)]
mod word_eval;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Query {
    id: String,
    width: u8,
    x_hex: String,
    y_hex: String,
    incoming: bool,
    initial_nat: String,
}

#[test]
fn actual_addition_carry_and_overflow_terms_match_frozen_numeric_queries() {
    let raw = include_bytes!(
        "../../../../develop/migrations/go-practical/t01-w04/word-numeric-bridge/queries.json"
    );
    let queries: Vec<Query> = serde_json::from_slice(raw).unwrap();
    assert_eq!(queries.len(), 288);
    let foundation = generate_go_practical_word_numeric_foundation().unwrap();
    let certificate =
        mpk_cert::decode_canonical_certificate(foundation.certificate_bytes()).unwrap();
    let mut ids = std::collections::BTreeSet::new();
    let mut model = vec![];
    let mut total_steps = 0;
    let mut maximum_steps = 0;
    let mut safe = 0;
    for query in queries {
        assert!(ids.insert(query.id.clone()));
        assert!([8, 16, 32, 64].contains(&query.width));
        assert!(query.initial_nat.bytes().all(|c| c.is_ascii_digit()));
        let x = u64::from_str_radix(&query.x_hex, 16).unwrap();
        let y = u64::from_str_radix(&query.y_hex, 16).unwrap();
        let mut interpreter = word_eval::Interpreter::new(&certificate);
        let xx = interpreter.word(query.width, x);
        let yy = interpreter.word(query.width, y);
        let incoming = interpreter.invoke(
            if query.incoming {
                "Std.Bool.true"
            } else {
                "Std.Bool.false"
            },
            &[],
        );
        let args = [xx, yy, incoming];
        let prefix = format!("Mpk.Go.Practical.Word.W{}.", query.width);
        let result = interpreter.invoke(&(prefix.clone() + "AddCarry"), &args);
        let bits = interpreter.bits(query.width, result);
        let out = interpreter.invoke(&(prefix.clone() + &format!("Carry{}", query.width)), &args);
        let out = interpreter.boolean(out);
        let into = interpreter.invoke(&(prefix + &format!("Carry{}", query.width - 1)), &args);
        let into = interpreter.boolean(into);
        let guard = interpreter.invoke(
            &format!("Mpk.Go.Practical.Word.Numeric.W{}.NoOverflow", query.width),
            &args,
        );
        let guard = interpreter.boolean(guard);
        let modulus = 1_u128 << query.width;
        let total = u128::from(x) + u128::from(y) + u128::from(query.incoming);
        let half = modulus / 2;
        let sign_incoming =
            ((u128::from(x) % half) + (u128::from(y) % half) + u128::from(query.incoming)) >= half;
        assert_eq!(u128::from(bits), total % modulus, "{query:?}");
        assert_eq!(out, total >= modulus, "{query:?}");
        assert_eq!(into, sign_incoming, "{query:?}");
        let signed = |v: u64| {
            i128::from(v)
                - if u128::from(v) >= half {
                    modulus as i128
                } else {
                    0
                }
        };
        let signed_sum = signed(x) + signed(y) + i128::from(query.incoming);
        let in_range = -(half as i128) <= signed_sum && signed_sum < half as i128;
        assert_eq!(guard, in_range, "{query:?}");
        assert_eq!(guard, into == out, "{query:?}");
        safe += usize::from(guard);
        let steps = interpreter.steps();
        total_steps += steps;
        maximum_steps = maximum_steps.max(steps);
        model.push(serde_json::json!({"id":query.id,"word_hex":format!("{:0width$x}", bits, width=usize::from(query.width)/4),"carry":out,"sign_incoming":into,"no_overflow":guard}));
    }
    assert_eq!(safe, 210);
    if let Some(root) = std::env::var_os("MPK_GO_WORD_NUMERIC_MODEL_OUTPUT") {
        use std::io::Write;
        let root = std::path::PathBuf::from(root);
        std::fs::create_dir_all(&root).unwrap();
        let metrics = serde_json::json!({"queries":288,"signed_no_overflow":210,"signed_overflow":78,"total_AST_steps":total_steps,"maximum_query_AST_steps":maximum_steps,"query_sha256":format!("{:x}",Sha256::digest(raw)),"primitive_values":"Bool constructors/recursors and original external input-word assignments; no arithmetic callback","Nat_denotation_terms_executed":false,"source_program_proof_claimed":false});
        for (name, bytes) in [
            ("model.json", serde_json::to_vec(&model).unwrap()),
            ("metrics.json", serde_json::to_vec_pretty(&metrics).unwrap()),
            (
                "certificate.mpcert",
                foundation.certificate_bytes().to_vec(),
            ),
        ] {
            let mut bytes = bytes;
            if name != "certificate.mpcert" {
                bytes.push(b'\n');
            }
            let mut output = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(root.join(name))
                .unwrap();
            output.write_all(&bytes).unwrap();
        }
    }
    println!("numeric queries=288 signed_safe=210 overflow=78 AST_steps={total_steps} max={maximum_steps}");
}
