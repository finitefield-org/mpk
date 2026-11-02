use mpk_vc::go_practical_ordinary::generate_go_practical_word_conversions;
use sha2::{Digest, Sha256};
#[path = "go_word_eval.rs"]
#[allow(
    dead_code,
    reason = "Only unchanged Bool circuit evaluation and the full original query schema execute."
)]
mod word_eval;
#[test]
fn actual_all_conversion_words_and_count_guards_match_full_original_inputs() {
    let raw = include_bytes!(
        "../../../../develop/migrations/go-practical/t01-w04/word-foundation/queries.json"
    );
    let queries: Vec<word_eval::Query> = serde_json::from_slice(raw).unwrap();
    let queries = queries
        .into_iter()
        .filter(|q| q.operation.contains("To"))
        .collect::<Vec<_>>();
    assert_eq!(queries.len(), 256);
    let f = generate_go_practical_word_conversions().unwrap();
    let c = mpk_cert::decode_canonical_certificate(f.certificate_bytes()).unwrap();
    let (
        mut rows,
        mut steps,
        mut maximum,
        mut faults,
        mut narrow,
        mut extend,
        mut exact_s,
        mut exact_u,
    ) = (vec![], 0, 0, 0, 0, 0, 0, 0);
    for q in queries {
        let x = u64::from_str_radix(&q.x_hex, 16).unwrap();
        let (w, t) = (q.width, q.target_width);
        let sg = q.operation.starts_with("SignedTo");
        let mut m = word_eval::Interpreter::new(&c);
        let xx = m.word(w, x);
        let y = m.invoke(
            &format!("Mpk.Go.Practical.Word.W{w}.{}", q.operation),
            &[xx],
        );
        let r = m.bits(t, y);
        let sx = m.invoke(&format!("Mpk.Go.Practical.Word.Numeric.W{w}.Sign"), &[xx]);
        let sx = m.boolean(sx);
        let sy = m.invoke(&format!("Mpk.Go.Practical.Word.Numeric.W{t}.Sign"), &[y]);
        let sy = m.boolean(sy);
        let ms = 1_i128 << w;
        let mt = 1_i128 << t;
        let source = if sg {
            i128::from(x) - if sx { ms } else { 0 }
        } else {
            i128::from(x)
        };
        let target_u = i128::from(r);
        let target_s = target_u - if sy { mt } else { 0 };
        let discard = if w >= t {
            x.checked_shr(u32::from(t)).unwrap_or(0)
        } else {
            0
        };
        let extension = if w < t { Some(mt - ms) } else { None };
        if w >= t {
            narrow += 1;
            assert_eq!(i128::from(x), target_u + mt * i128::from(discard));
        } else {
            extend += 1;
            assert_eq!(
                target_u,
                i128::from(x) + if sg && sx { extension.unwrap() } else { 0 }
            );
        }
        let cn = if sg && sx {
            if w >= t {
                1_i128 << (w - t)
            } else {
                1
            }
        } else {
            0
        };
        let cu = i128::from(discard) - cn;
        let cs = cu + i128::from(sy);
        assert_eq!(source, target_u + mt * cu);
        assert_eq!(source, target_s + mt * cs);
        assert!(target_u >= 0 && target_u < mt);
        let exs = sg && w <= t;
        let exu = !sg && w <= t;
        if exs {
            exact_s += 1;
            assert_eq!(source, target_s);
        }
        if exu {
            exact_u += 1;
            assert_eq!(source, target_u);
        }
        let count = m.invoke(
            &format!(
                "Mpk.Go.Practical.Word.W{w}.{}To64",
                if sg { "Signed" } else { "Unsigned" }
            ),
            &[xx],
        );
        let count_word = m.bits(64, count);
        let defined = if sg {
            let g = m.invoke("Mpk.Go.Practical.Word.W64.SignedCountDefined", &[count]);
            m.boolean(g)
        } else {
            true
        };
        assert_eq!(defined, !sg || !sx);
        if defined {
            assert_eq!(count_word, x);
        } else {
            faults += 1;
        }
        rows.push(serde_json::json!({"id":q.id,"operation":q.operation,"result_hex":format!("{:0d$x}",r,d=usize::from(t)/4),"source_value":source.to_string(),"target_unsigned":target_u.to_string(),"target_signed":target_s.to_string(),"source_modulus":ms.to_string(),"target_modulus":mt.to_string(),"discarded":discard.to_string(),"extension_mask":extension.map(|n|n.to_string()),"target_sign":sy,"source_sign":sx,"correction_unsigned":cu.to_string(),"correction_signed":cs.to_string(),"unsigned_refinement":true,"signed_refinement":true,"exact_unsigned":exu,"exact_signed":exs,"count_word_hex":format!("{count_word:016x}"),"count_defined":defined,"native_count_panicked":!defined}));
        steps += m.steps();
        maximum = maximum.max(m.steps());
    }
    assert_eq!(
        (faults, narrow, extend, exact_s, exact_u),
        (64, 160, 96, 80, 80)
    );
    if let Some(out) = std::env::var_os("MPK_GO_WORD_CONVERSIONS_MODEL_OUTPUT") {
        use std::io::Write;
        let out = std::path::PathBuf::from(out);
        std::fs::create_dir_all(&out).unwrap();
        let metrics = serde_json::json!({"original_conversion_queries":256,"actual_source_conversion_results":256,"truncating_or_identity_rows":narrow,"extending_rows":extend,"signed_exact_type_cases":exact_s,"unsigned_exact_type_cases":exact_u,"actual_count_normalizations":256,"defined_counts":256-faults,"actual_false_signed_count_guards":faults,"total_AST_steps":steps,"maximum_query_AST_steps":maximum,"original_frozen_query_sha256":format!("{:x}",Sha256::digest(raw)),"Nat_Int_masks_ratios_quotients_or_theorem_bodies_executed":false,"universal_conversion_laws_supplied_by_certificate_not_finite_model":true,"scope":"Every original complete conversion row executes the actual decoded source conversion, source/result sign circuits, actual signed/unsigned To64 and actual signed-count Bool guard. Host derives numeric quotient/mask/correction counterparts from the observed full words; no Nat/Int/quotient/mask/ratio/theorem body executes. Native Go separately probes real negative-count panic. The finite model does not execute variable shifts or supply their universal proof. Exactness flags identify type-level theorem applicability, not an iff classification of exactness for each finite input. Full source/program/profile/application binding remains due."});
        for (name, mut bytes) in [
            ("model.json", serde_json::to_vec(&rows).unwrap()),
            ("metrics.json", serde_json::to_vec_pretty(&metrics).unwrap()),
            ("certificate.mpcert", f.certificate_bytes().to_vec()),
        ] {
            if name != "certificate.mpcert" {
                bytes.push(b'\n');
            }
            let mut p = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(out.join(name))
                .unwrap();
            p.write_all(&bytes).unwrap();
        }
    }
    println!("conversion_rows=256 count_faults={faults} exact_signed={exact_s} exact_unsigned={exact_u} AST_steps={steps} max={maximum}");
}
