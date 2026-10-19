use mpk_vc::go_practical_ordinary::generate_go_practical_word_order_foundation;
use sha2::{Digest, Sha256};

#[path = "go_word_eval.rs"]
#[allow(
    dead_code,
    reason = "The shared Bool interpreter, query schema and independent oracle are used here; the separate all-operation observation entry point remains covered by go_practical_word_foundation."
)]
mod word_eval;

#[test]
fn actual_complement_subtraction_carries_order_equality_and_divisor_guard_match() {
    let raw = include_bytes!(
        "../../../../develop/migrations/go-practical/t01-w04/word-foundation/queries.json"
    );
    let all: Vec<word_eval::Query> = serde_json::from_slice(raw).unwrap();
    let queries = all
        .into_iter()
        .filter(|q| {
            matches!(
                q.operation.as_str(),
                "Not" | "Sub" | "Equal" | "NotEqual" | "UnsignedLess" | "UnsignedLessEqual"
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(queries.len(), 272);
    let f = generate_go_practical_word_order_foundation().unwrap();
    let c = mpk_cert::decode_canonical_certificate(f.certificate_bytes()).unwrap();
    let mut rows = vec![];
    let mut laws = vec![];
    let mut total_steps = 0;
    let mut maximum_steps = 0;
    let mut zero_divisors = 0;
    let mut borrows = 0;
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
        let true_ = model.invoke("Std.Bool.true", &[]);
        let prefix = format!("Mpk.Go.Practical.Word.W{}.", q.width);
        let result = model.invoke(
            &(prefix.clone() + &q.operation),
            if q.operation == "Not" {
                &[xx]
            } else {
                &[xx, yy]
            },
        );
        let observation = if matches!(q.operation.as_str(), "Not" | "Sub") {
            let bits = model.bits(q.width, result);
            word_eval::Observation {
                id: q.id.clone(),
                kind: "word",
                value: None,
                value_hex: Some(format!(
                    "{:0width$x}",
                    bits,
                    width = usize::from(q.width) / 4
                )),
                reason: None,
            }
        } else {
            word_eval::Observation {
                id: q.id.clone(),
                kind: "boolean",
                value: Some(model.boolean(result)),
                value_hex: None,
                reason: None,
            }
        };
        assert_eq!(observation, word_eval::oracle(&q));
        rows.push(observation);
        let nx = model.invoke(&(prefix.clone() + "Not"), &[xx]);
        let ny = model.invoke(&(prefix.clone() + "Not"), &[yy]);
        let sub = model.invoke(&(prefix.clone() + "Sub"), &[xx, yy]);
        let (nx_bits, ny_bits, sub_bits) = (
            model.bits(q.width, nx),
            model.bits(q.width, ny),
            model.bits(q.width, sub),
        );
        assert_eq!(u128::from(x) + u128::from(nx_bits) + 1, modulus);
        assert_eq!(u128::from(y) + u128::from(ny_bits) + 1, modulus);
        let carry_name = prefix.clone() + &format!("Carry{}", q.width);
        let carry = model.invoke(&carry_name, &[xx, ny, true_]);
        let reverse = model.invoke(&carry_name, &[yy, nx, true_]);
        let pair = model.invoke("Mpk.Go.Practical.Bool.And", &[carry, reverse]);
        let (carry, reverse, pair) = (
            model.boolean(carry),
            model.boolean(reverse),
            model.boolean(pair),
        );
        assert_eq!(
            u128::from(x) + modulus,
            u128::from(sub_bits) + u128::from(y) + modulus * u128::from(carry),
        );
        assert!(u128::from(sub_bits) < modulus);
        let less = model.invoke(&(prefix.clone() + "UnsignedLess"), &[xx, yy]);
        let le = model.invoke(&(prefix.clone() + "UnsignedLessEqual"), &[xx, yy]);
        let equal = model.invoke(&(prefix.clone() + "Equal"), &[xx, yy]);
        let not_equal = model.invoke(&(prefix.clone() + "NotEqual"), &[xx, yy]);
        let zero = model.invoke(&(prefix.clone() + "IsZero"), &[yy]);
        let defined = model.invoke(&(prefix + "DivideDefined"), &[yy]);
        let (less, le, equal, not_equal, zero, defined) = (
            model.boolean(less),
            model.boolean(le),
            model.boolean(equal),
            model.boolean(not_equal),
            model.boolean(zero),
            model.boolean(defined),
        );
        assert_eq!(less, x < y);
        assert_eq!(less, !carry);
        assert_eq!(le, x <= y);
        assert_eq!(equal, x == y);
        assert_eq!(pair, equal);
        assert_eq!(not_equal, x != y);
        assert_eq!(zero, y == 0);
        assert_eq!(defined, y != 0);
        zero_divisors += usize::from(zero);
        borrows += usize::from(!carry);
        laws.push(serde_json::json!({"id":q.id,"x_complement_hex":format!("{:0width$x}",nx_bits,width=usize::from(q.width)/4),"y_complement_hex":format!("{:0width$x}",ny_bits,width=usize::from(q.width)/4),"sub_hex":format!("{:0width$x}",sub_bits,width=usize::from(q.width)/4),"carry":carry,"reverse_carry":reverse,"carry_pair":pair,"less":less,"less_equal":le,"equal":equal,"not_equal":not_equal,"zero_divisor":zero,"divide_defined":defined}));
        total_steps += model.steps();
        maximum_steps = maximum_steps.max(model.steps());
    }
    assert!(zero_divisors > 0 && borrows > 0);
    if let Some(out) = std::env::var_os("MPK_GO_WORD_ORDER_MODEL_OUTPUT") {
        use std::io::Write;
        let out = std::path::PathBuf::from(out);
        std::fs::create_dir_all(&out).unwrap();
        let metrics = serde_json::json!({"selected_original_queries":272,"word_results":80,"boolean_results":192,"observed_zero_divisors":zero_divisors,"observed_borrows":borrows,"total_AST_steps":total_steps,"maximum_query_AST_steps":maximum_steps,"original_frozen_query_sha256":format!("{:x}",Sha256::digest(raw)),"actual_Boolean_certificate_terms_executed":true,"Nat_denotation_order_or_theorem_bodies_executed":false,"observation_scope":"Actual complement/subtraction/forward and reverse full-adder carries, equality/comparison/zero/divisor-guard Bool circuits. Scalar bounds, conservation, comparisons and nonzero meanings use independent host integers; universal genuine Nat theorem terms are checked separately in both cores. No arithmetic model callback or source-program claim."});
        for (name, mut bytes) in [
            ("model.json", serde_json::to_vec(&rows).unwrap()),
            ("laws.json", serde_json::to_vec(&laws).unwrap()),
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
    println!("order queries=272 zero_divisors={zero_divisors} borrows={borrows} AST_steps={total_steps} max={maximum_steps}");
}
