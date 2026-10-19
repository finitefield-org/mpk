use mpk_vc::go_practical_ordinary::generate_go_practical_word_signed_division_bounds;
use sha2::{Digest, Sha256};
#[path = "go_word_eval.rs"]
#[allow(
    dead_code,
    reason = "Only the unchanged Bool interpreter and original full query schema are used."
)]
mod word_eval;

#[test]
fn actual_source_quotient_remainder_bounds_and_minimum_conditions_match_original_inputs() {
    let raw = include_bytes!(
        "../../../../develop/migrations/go-practical/t01-w04/word-foundation/queries.json"
    );
    let all: Vec<word_eval::Query> = serde_json::from_slice(raw).unwrap();
    let queries = all
        .into_iter()
        .filter(|q| matches!(q.operation.as_str(), "SignedDiv" | "SignedRem"))
        .collect::<Vec<_>>();
    assert_eq!(queries.len(), 96);
    let f = generate_go_practical_word_signed_division_bounds().unwrap();
    let c = mpk_cert::decode_canonical_certificate(f.certificate_bytes()).unwrap();
    let (mut rows, mut steps, mut maximum, mut faults, mut minima, mut units, mut exact) =
        (vec![], 0, 0, 0, 0, 0, 0);
    for query in queries {
        let x = u64::from_str_radix(&query.x_hex, 16).unwrap();
        let y = u64::from_str_radix(&query.y_hex, 16).unwrap();
        let mut model = word_eval::Interpreter::new(&c);
        let xx = model.word(query.width, x);
        let yy = model.word(query.width, y);
        let prefix = format!("Mpk.Go.Practical.Word.W{}.", query.width);
        let top = format!(
            "Mpk.Go.Practical.Word.SignedDivision.Prerequisite.Bounds.Top{}",
            query.width.trailing_zeros()
        );
        let guard = model.invoke(&(prefix.clone() + "DivideDefined"), &[yy]);
        let defined = model.boolean(guard);
        assert_eq!(defined, y != 0);
        let mut row = serde_json::json!({"id":query.id,"operation":query.operation,"defined":defined,"panicked":!defined});
        for key in [
            "abs_x_hex",
            "abs_y_hex",
            "q_amount_hex",
            "r_amount_hex",
            "quotient_hex",
            "remainder_hex",
            "signed_x",
            "signed_y",
            "signed_quotient",
            "signed_remainder",
            "half",
            "modulus",
            "input_gap_x",
            "input_gap_y",
            "quotient_gap",
            "quotient_bounded",
            "remainder_strict_half",
            "negative_unit",
            "quotient_overflow",
            "minimum_minus_one",
            "quotient_no_overflow",
            "quotient_correction",
            "remainder_correction",
            "corrected_conservation",
            "exact_conservation",
            "minimum_results",
        ] {
            row[key] = serde_json::Value::Null;
        }
        if !defined {
            faults += 1;
        } else {
            let sx = model.invoke(&top, &[xx]);
            let sy = model.invoke(&top, &[yy]);
            let qflag = model.invoke("Mpk.Go.Practical.Bool.Xor", &[sx, sy]);
            let ax = model.invoke(&(prefix.clone() + "AbsBits"), &[xx]);
            let ay = model.invoke(&(prefix.clone() + "AbsBits"), &[yy]);
            let qa = model.invoke(&(prefix.clone() + "UnsignedDiv"), &[ax, ay]);
            let ra = model.invoke(&(prefix.clone() + "UnsignedRem"), &[ax, ay]);
            let sq = model.invoke(&(prefix.clone() + "SignedDiv"), &[xx, yy]);
            let sr = model.invoke(&(prefix.clone() + "SignedRem"), &[xx, yy]);
            let nq = model.invoke(&(prefix.clone() + "Neg"), &[qa]);
            let nr = model.invoke(&(prefix.clone() + "Neg"), &[ra]);
            let qb = model.bits(query.width, qa);
            let rb = model.bits(query.width, ra);
            let sqb = model.bits(query.width, sq);
            let srb = model.bits(query.width, sr);
            let axb = model.bits(query.width, ax);
            let ayb = model.bits(query.width, ay);
            let qt = model.invoke(&top, &[qa]);
            let nqt = model.invoke(&top, &[nq]);
            let rt = model.invoke(&top, &[ra]);
            let nrt = model.invoke(&top, &[nr]);
            let dq = model.invoke(&(prefix.clone() + "DivideDefined"), &[qa]);
            let dr = model.invoke(&(prefix + "DivideDefined"), &[ra]);
            let qflag = model.boolean(qflag);
            let rflag = model.boolean(sx);
            let qc = i128::from(model.boolean(if qflag { nqt } else { qt }))
                - i128::from(qflag && model.boolean(dq));
            let rc = i128::from(model.boolean(if rflag { nrt } else { rt }))
                - i128::from(rflag && model.boolean(dr));
            let modulus = 1_i128 << query.width;
            let half = modulus / 2;
            let signed = |v: u64| i128::from(v) - if i128::from(v) >= half { modulus } else { 0 };
            let (sxv, syv, sqv, srv) = (signed(x), signed(y), signed(sqb), signed(srb));
            let qbounded = i128::from(qb) <= half;
            let rbounded = i128::from(rb) < half;
            assert!(qbounded && rbounded);
            assert!(i128::from(axb) <= half && i128::from(ayb) <= half && ayb > 0);
            let gx = half - i128::from(axb);
            let gy = half - i128::from(ayb);
            let gap = (i128::from(ayb) - 1) * i128::from(qb) + i128::from(rb) + gx;
            assert_eq!(i128::from(qb) + gap, half);
            assert!(gap >= 0);
            let negative_unit = model.boolean(sy) && ayb == 1;
            assert_eq!(negative_unit, syv == -1);
            let overflow = !qflag && i128::from(qb) == half;
            let minimum = i128::from(x) == half && negative_unit;
            assert_eq!(overflow, minimum);
            assert_eq!(qc, i128::from(minimum));
            assert_eq!(rc, 0);
            let corrected = sxv == syv * (sqv + modulus * qc) + srv;
            let is_exact = sxv == syv * sqv + srv;
            assert!(corrected);
            assert_eq!(is_exact, !minimum);
            let minimum_results = minimum && i128::from(sqb) == half && srb == 0;
            assert_eq!(minimum_results, minimum);
            minima += usize::from(minimum);
            units += usize::from(negative_unit);
            exact += usize::from(is_exact);
            let format = |v: u64| format!("{:0w$x}", v, w = usize::from(query.width) / 4);
            row["abs_x_hex"] = format(axb).into();
            row["abs_y_hex"] = format(ayb).into();
            row["q_amount_hex"] = format(qb).into();
            row["r_amount_hex"] = format(rb).into();
            row["quotient_hex"] = format(sqb).into();
            row["remainder_hex"] = format(srb).into();
            for (key, value) in [
                ("signed_x", sxv),
                ("signed_y", syv),
                ("signed_quotient", sqv),
                ("signed_remainder", srv),
                ("half", half),
                ("modulus", modulus),
                ("input_gap_x", gx),
                ("input_gap_y", gy),
                ("quotient_gap", gap),
            ] {
                row[key] = value.to_string().into();
            }
            for (key, value) in [
                ("quotient_bounded", qbounded),
                ("remainder_strict_half", rbounded),
                ("negative_unit", negative_unit),
                ("quotient_overflow", overflow),
                ("minimum_minus_one", minimum),
                ("quotient_no_overflow", !minimum),
                ("corrected_conservation", corrected),
                ("exact_conservation", is_exact),
                ("minimum_results", minimum_results),
            ] {
                row[key] = value.into();
            }
            row["quotient_correction"] = serde_json::json!(qc);
            row["remainder_correction"] = serde_json::json!(rc);
        }
        steps += model.steps();
        maximum = maximum.max(model.steps());
        rows.push(row);
    }
    assert_eq!(faults, 8);
    assert_eq!(minima, 8);
    assert_eq!(exact, 80);
    if let Some(out) = std::env::var_os("MPK_GO_WORD_DIVISION_BOUNDS_MODEL_OUTPUT") {
        use std::io::Write;
        let out = std::path::PathBuf::from(out);
        std::fs::create_dir_all(&out).unwrap();
        let metrics = serde_json::json!({"original_signed_queries":96,"defined_pairs":88,"source_zero_divisor_faults":faults,"minimum_minus_one_pairs":minima,"negative_unit_divisors":units,"exact_source_reconstructions":exact,"corrected_source_reconstructions":88,"actual_source_operator_results":176,"total_AST_steps":steps,"maximum_query_AST_steps":maximum,"original_frozen_query_sha256":format!("{:x}",Sha256::digest(raw)),"Nat_gap_half_predicate_Int_or_theorem_bodies_executed":false,"universal_classification_supplied_by_certificate_not_finite_model":true,"scope":"Both actual signed source outputs and actual Abs/unsigned magnitude/Neg/Top/divisor Bool circuits execute for each defined original full row. Host derives numeric half/gaps and Nat-guard/correction counterparts from those actual full word observations; no Nat/Int/predicate/theorem body or arithmetic callback is executed. Undefined source rows retain actual false guard and compare to real native faults without observing results. Universal classification is proved separately by both ordinary cores; no whole program/application/profile release claim."});
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
    println!("division bounds original_queries=96 defined=88 faults={faults} minimum_pairs={minima} exact={exact} units={units} source_results=176 AST_steps={steps} max={maximum}");
}
