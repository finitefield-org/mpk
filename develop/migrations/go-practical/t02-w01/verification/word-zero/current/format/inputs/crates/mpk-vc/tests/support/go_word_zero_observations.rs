use mpk_vc::go_practical_ordinary::generate_go_practical_word_zero_foundation;
use sha2::{Digest, Sha256};
#[path = "go_word_eval.rs"]
#[allow(
    dead_code,
    reason = "Only the unchanged Bool interpreter and full original query schema are used."
)]
mod word_eval;
#[test]
fn actual_direct_zero_bits_and_source_guards_match_original_words() {
    let raw = include_bytes!(
        "../../../../develop/migrations/go-practical/t01-w04/word-foundation/queries.json"
    );
    let all: Vec<word_eval::Query> = serde_json::from_slice(raw).unwrap();
    let queries = all
        .into_iter()
        .filter(|q| q.operation == "Neg")
        .collect::<Vec<_>>();
    assert_eq!(queries.len(), 32);
    let f = generate_go_practical_word_zero_foundation().unwrap();
    let c = mpk_cert::decode_canonical_certificate(f.certificate_bytes()).unwrap();
    let (mut rows, mut steps, mut max, mut zeros) = (vec![], 0, 0, 0);
    for q in queries {
        let x = u64::from_str_radix(&q.x_hex, 16).unwrap();
        let mut model = word_eval::Interpreter::new(&c);
        let xx = model.word(q.width, x);
        let word = model.bits(q.width, xx);
        assert_eq!(word, x);
        let p = format!("Mpk.Go.Practical.Word.W{}.", q.width);
        let z = model.invoke(&(p.clone() + "IsZero"), &[xx]);
        let df = model.invoke(&(p + "DivideDefined"), &[xx]);
        let zb = model.invoke(
            &format!(
                "Mpk.Go.Practical.Word.Zero.ZeroBits{}",
                q.width.trailing_zeros()
            ),
            &[xx],
        );
        let (z, df, zb) = (model.boolean(z), model.boolean(df), model.boolean(zb));
        assert_eq!((z, df, zb), (x == 0, x != 0, x == 0));
        let sign = x >> (q.width - 1) != 0;
        if z {
            assert!(!sign);
            zeros += 1;
        }
        rows.push(serde_json::json!({"kind":"zero","id":q.id,"word_hex":format!("{:0w$x}",word,w=usize::from(q.width)/4),"is_zero":z,"divide_defined":df,"zero_bits":zb,"source_sign":sign}));
        steps += model.steps();
        max = max.max(model.steps());
    }
    assert_eq!(zeros, 4);
    if let Some(out) = std::env::var_os("MPK_GO_WORD_ZERO_MODEL_OUTPUT") {
        use std::io::Write;
        let out = std::path::PathBuf::from(out);
        std::fs::create_dir_all(&out).unwrap();
        let metrics = serde_json::json!({"original_queries":32,"actual_word_bits":960,"actual_zero_predicates":96,"zero_cases":4,"total_AST_steps":steps,"maximum_query_AST_steps":max,"original_frozen_query_sha256":format!("{:x}",Sha256::digest(raw)),"Nat_numeric_zero_or_theorem_bodies_executed":false,"scope":"Actual decoded IsZero/DivideDefined and new recursive ZeroBits functions execute with only Bool primitives. Full input words remain original; Nat zero/numeric/positive-product/theorem meanings check separately in both cores, not via arithmetic callbacks or giant unary execution."});
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
    println!("direct zero original_queries=32 predicates=96 bits=960 zeros={zeros} AST_steps={steps} max={max}");
}
