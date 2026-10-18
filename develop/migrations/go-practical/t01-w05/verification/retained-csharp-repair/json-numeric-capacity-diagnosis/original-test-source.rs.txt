//! Identify the exact changed definitions before reusing numeric capacity runs.
use super::*;

fn decode(path: &Path) -> mpk_cert::encode::Certificate {
    let hex = fs::read_to_string(path).unwrap();
    let bytes = (0..hex.trim().len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect::<Vec<_>>();
    mpk_cert::decode_canonical_certificate(&bytes).unwrap()
}

#[test]
fn csharp_03_t06_w09_json_numeric_capacity_definition_changes() {
    use super::super::super::super::structural_equivalence_tests::{
        same_definition_bodies, same_definition_closure, same_definition_types,
    };
    let base = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation");
    let mut rows = vec![];
    for (family, file) in [
        ("json-sequences", "source-arrays.hex"),
        ("json-sums", "all-sums.hex"),
    ] {
        let old = decode(&base.join(family).join("previous-cell-count").join(file));
        let new = decode(&base.join(family).join(file));
        validate_csharp_practical_certificate_structure(&old).unwrap();
        validate_csharp_practical_certificate_structure(&new).unwrap();
        let names = |c: &mpk_cert::encode::Certificate| {
            c.declarations
                .iter()
                .map(|d| c.name_table[d.name as usize].clone())
                .collect::<BTreeSet<_>>()
        };
        let roots = names(&old);
        assert_eq!(roots, names(&new));
        same_definition_types(&old, &new, &roots).unwrap();
        assert!(same_definition_closure(&old, &new, &roots).is_err());
        let changed = roots
            .iter()
            .filter(|n| {
                same_definition_bodies(&old, &new, &BTreeSet::from([(*n).clone()])).is_err()
            })
            .cloned()
            .collect::<BTreeSet<_>>();
        // The circuit emitter hex-encodes the operation name below its shared
        // IntegerText namespace. Pin the six actually reviewed blocks, not a
        // broad namespace that could hide changes to cursor or storage logic.
        let expected = [
            ("Child", vec![4, 8, 14, 15]),
            ("Syntax", vec![3]),
            ("FinishPacket", vec![3]),
        ]
        .into_iter()
        .flat_map(|(kind, blocks)| {
            let operation = format!("Mpk.CSharp.Ordinary.JsonGrammar.{kind}");
            let encoded = operation
                .bytes()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>();
            blocks.into_iter().map(move |block| {
                format!("Mpk.CSharp.Ordinary.IntegerText.O{encoded}.Block.B{block}")
            })
        })
        .collect::<BTreeSet<_>>();
        assert_eq!(changed, expected);
        eprintln!(
            "Numeric capacity {family}: all{} names/types unchanged;changed bodies:{changed:?}",
            roots.len()
        );
        rows.push(json!({"family":family,"file":file,"declarations":roots.len(),"changed_definitions":changed,"old_term_count":old.term_table.len(),"new_term_count":new.term_table.len()}));
    }
    if let Some(path) = std::env::var_os("MPK_W09_JSON_CELL_DIFF_OUT") {
        fs::write(path, serde_json::to_vec_pretty(&rows).unwrap()).unwrap();
    }
}
