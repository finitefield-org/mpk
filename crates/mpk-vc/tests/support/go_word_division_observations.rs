use mpk_vc::go_practical_ordinary::generate_go_practical_word_division_foundation;
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

#[path = "go_word_eval.rs"]
#[allow(
    dead_code,
    reason = "Only the shared Bool interpreter and frozen query schema are needed; the all-operation oracle remains covered by go_practical_word_foundation."
)]
mod word_eval;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Inputs {
    divisions: Vec<word_eval::Query>,
    steps: Vec<Step>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Step {
    id: String,
    width: u8,
    r_hex: String,
    d_hex: String,
    low: bool,
}

fn observed_step(
    model: &mut word_eval::Interpreter<'_>,
    w: u8,
    inputs: [u32; 5],
    low: bool,
) -> Value {
    let [r, y, s, q, next] = inputs;
    let (rv, dv, sv, nv) = (
        model.bits(w, r),
        model.bits(w, y),
        model.bits(w, s),
        model.bits(w, next),
    );
    assert!(rv < dv);
    let prefix = format!("Mpk.Go.Practical.Word.W{w}.");
    let t = model.invoke(&(prefix.clone() + "Sub"), &[s, y]);
    let tv = model.bits(w, t);
    let ny = model.invoke(&(prefix.clone() + "Not"), &[y]);
    let true_ = model.invoke("Std.Bool.true", &[]);
    let carry = model.invoke(&(prefix + &format!("Carry{w}")), &[s, ny, true_]);
    let carry = model.boolean(carry);
    let high = rv >> (w - 1) != 0;
    let decision = model.boolean(q);
    let modulus = 1_u128 << w;
    assert_eq!(decision, high || carry);
    assert_eq!(
        u128::from(sv) + modulus * u128::from(high),
        2 * u128::from(rv) + u128::from(low)
    );
    assert_eq!(
        u128::from(sv) + modulus,
        u128::from(tv) + u128::from(dv) + modulus * u128::from(carry)
    );
    assert_eq!(nv, if decision { tv } else { sv });
    assert_eq!(
        u128::from(nv) + u128::from(dv) * u128::from(decision),
        2 * u128::from(rv) + u128::from(low)
    );
    assert!(nv < dv);
    json!({"prior_hex":format!("{:0width$x}",rv,width=usize::from(w)/4),"shifted_hex":format!("{:0width$x}",sv,width=usize::from(w)/4),"sub_hex":format!("{:0width$x}",tv,width=usize::from(w)/4),"remainder_hex":format!("{:0width$x}",nv,width=usize::from(w)/4),"low":low,"high":high,"carry":carry,"decision":decision})
}

#[test]
fn actual_source_division_stages_and_full_width_arbitrary_restore_steps_match() {
    let raw=include_bytes!("../../../../develop/migrations/go-practical/t02-w01/verification/word-division/queries.json");
    let input: Inputs = serde_json::from_slice(raw).unwrap();
    assert_eq!(input.divisions.len(), 96);
    assert_eq!(input.steps.len(), 34);
    let f = generate_go_practical_word_division_foundation().unwrap();
    let c = mpk_cert::decode_canonical_certificate(f.certificate_bytes()).unwrap();
    let (mut divisions, mut steps) = (vec![], vec![]);
    let (mut total, mut maximum, mut stages, mut failures, mut overflow) = (0, 0, 0, 0, 0);
    for q in input.divisions {
        let (x, y) = (
            u64::from_str_radix(&q.x_hex, 16).unwrap(),
            u64::from_str_radix(&q.y_hex, 16).unwrap(),
        );
        let w = q.width;
        assert!(matches!(
            q.operation.as_str(),
            "UnsignedDiv" | "UnsignedRem"
        ));
        let mut model = word_eval::Interpreter::new(&c);
        let xx = model.word(w, x);
        let yy = model.word(w, y);
        let prefix = format!("Mpk.Go.Practical.Word.W{w}.");
        let guard = model.invoke(&(prefix.clone() + "DivideDefined"), &[yy]);
        let guard = model.boolean(guard);
        assert_eq!(guard, y != 0);
        let root = model.invoke(&(prefix.clone() + &q.operation), &[xx, yy]);
        let bits = model.bits(w, root);
        if !guard {
            assert_eq!(bits, 0);
            failures += 1;
            divisions.push(json!({"id":q.id,"kind":"runtime_safety_failure","reason":"division_by_zero","guard":false}));
        } else {
            let mut rows = vec![];
            let (mut dividend, mut quotient) = (0_u128, 0_u128);
            for i in 0..w {
                let r = model.invoke(&(prefix.clone() + &format!("Remainder{i}")), &[xx, yy]);
                let s = model.invoke(&(prefix.clone() + &format!("DividendStep{i}")), &[xx, yy]);
                let qb = model.invoke(&(prefix.clone() + &format!("QuotientBit{i}")), &[xx, yy]);
                let rn = model.invoke(
                    &(prefix.clone() + &format!("Remainder{}", i + 1)),
                    &[xx, yy],
                );
                let low = (x >> (w - 1 - i)) & 1 != 0;
                let mut row = observed_step(&mut model, w, [r, yy, s, qb, rn], low);
                dividend = 2 * dividend + u128::from(low);
                quotient = 2 * quotient + u128::from(model.boolean(qb));
                assert_eq!(
                    u128::from(model.bits(w, rn)) + u128::from(y) * quotient,
                    dividend
                );
                row["index"] = json!(i);
                row["dividend_prefix"] = json!(dividend.to_string());
                row["quotient_prefix"] = json!(quotient.to_string());
                rows.push(row);
                stages += 1;
            }
            assert_eq!(dividend, u128::from(x));
            assert_eq!(quotient, u128::from(x / y));
            assert_eq!(
                bits,
                if q.operation == "UnsignedDiv" {
                    x / y
                } else {
                    x % y
                }
            );
            divisions.push(json!({"id":q.id,"kind":"word","word_hex":format!("{:0width$x}",bits,width=usize::from(w)/4),"guard":true,"stages":rows}));
        }
        total += model.steps();
        maximum = maximum.max(model.steps());
    }
    for q in input.steps {
        let (r, d) = (
            u64::from_str_radix(&q.r_hex, 16).unwrap(),
            u64::from_str_radix(&q.d_hex, 16).unwrap(),
        );
        assert!(r < d);
        let w = q.width;
        let mut model = word_eval::Interpreter::new(&c);
        let rr = model.word(w, r);
        let dd = model.word(w, d);
        let low = model.invoke(
            if q.low {
                "Std.Bool.true"
            } else {
                "Std.Bool.false"
            },
            &[],
        );
        let s = model.invoke(&format!("Mpk.Go.Practical.Word.W{w}.PushLow"), &[rr, low]);
        let qb = model.invoke(
            &format!("Mpk.Go.Practical.Word.Division.W{w}.Decision"),
            &[rr, dd, low],
        );
        let rn = model.invoke(
            &format!("Mpk.Go.Practical.Word.Division.W{w}.NextRemainder"),
            &[rr, dd, low],
        );
        let mut row = observed_step(&mut model, w, [rr, dd, s, qb, rn], q.low);
        overflow += usize::from(row["high"].as_bool().unwrap());
        row["id"] = json!(q.id);
        steps.push(row);
        total += model.steps();
        maximum = maximum.max(model.steps());
    }
    assert_eq!(stages, 2640);
    assert_eq!(failures, 8);
    assert_eq!(overflow, 10);
    if let Some(out) = std::env::var_os("MPK_GO_WORD_DIVISION_MODEL_OUTPUT") {
        use std::io::Write;
        let out = std::path::PathBuf::from(out);
        std::fs::create_dir_all(&out).unwrap();
        let metrics = json!({"original_division_queries":96,"actual_source_division_stages":stages,"runtime_domain_failures":failures,"arbitrary_restore_queries":34,"observed_arbitrary_overflow_bits":overflow,"total_AST_steps":total,"maximum_query_AST_steps":maximum,"query_sha256":format!("{:x}",Sha256::digest(raw)),"actual_Boolean_certificate_terms_executed":true,"Nat_prefix_denotation_order_or_theorem_bodies_executed":false,"scope":"Actual retained source Remainder_i/DividendStep_i/QuotientBit_i/UnsignedDiv/UnsignedRem and generic restore alias circuits execute with Bool-only primitives. Host integers derive quotient/dividend prefixes and check conservation/bounds. No arithmetic callback or unary Nat execution; universal genuine Nat proofs check independently in both cores."});
        for (name, mut bytes) in [
            (
                "model.json",
                serde_json::to_vec(&json!({"divisions":divisions,"steps":steps})).unwrap(),
            ),
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
    println!("division queries=96 stages={stages} zero_divisors={failures} arbitrary_steps=34 overflow_bits={overflow} AST_steps={total} max={maximum}");
}
