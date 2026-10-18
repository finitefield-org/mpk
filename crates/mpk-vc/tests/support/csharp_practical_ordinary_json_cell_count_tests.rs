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

fn numeric_capacity_changes(
    old: &mpk_cert::encode::Certificate,
    new: &mpk_cert::encode::Certificate,
    expected: &BTreeSet<String>,
) -> Result<BTreeSet<String>, String> {
    use super::super::super::super::structural_equivalence_tests::{
        same_definition_bodies, same_definition_closure, same_definition_types,
    };
    let names = |c: &mpk_cert::encode::Certificate| {
        c.declarations
            .iter()
            .map(|d| c.name_table[d.name as usize].clone())
            .collect::<BTreeSet<_>>()
    };
    let roots = names(old);
    if roots != names(new) {
        return Err("historical numeric-capacity declaration names changed".into());
    }
    same_definition_types(old, new, &roots)?;
    if same_definition_closure(old, new, &roots).is_ok() {
        return Err("numeric-capacity correction did not change any definition".into());
    }
    let changed = roots
        .iter()
        .filter(|n| same_definition_bodies(old, new, &BTreeSet::from([(*n).clone()])).is_err())
        .cloned()
        .collect::<BTreeSet<_>>();
    if &changed != expected {
        return Err(format!(
            "unexpected numeric-capacity body changes: {changed:?}"
        ));
    }
    Ok(roots)
}

#[test]
fn csharp_03_t06_w09_json_numeric_capacity_definition_changes() {
    use mpk_cert::encode::DeclarationKind;
    use sha2::{Digest, Sha256};
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let manifest = fs::read(root.join(
        "develop/migrations/go-practical/t01-w05/retained-csharp/json-numeric-capacity-epochs/manifest.json",
    ))
    .unwrap();
    assert_eq!(
        format!("{:x}", Sha256::digest(&manifest)),
        "2b8516c2810ec76d2a5b5684c2ee6893b1119b93c253959f90e41c9639e69549"
    );
    let manifest: Value = serde_json::from_slice(&manifest).unwrap();
    let pin = |record: &Value| {
        let path = root.join(record["path"].as_str().unwrap());
        let bytes = fs::read(&path).unwrap();
        assert_eq!(bytes.len() as u64, record["bytes"].as_u64().unwrap());
        assert_eq!(
            format!("{:x}", Sha256::digest(&bytes)),
            record["sha256"].as_str().unwrap()
        );
        path
    };
    for receipt in manifest["supporting_retained_receipts"].as_array().unwrap() {
        pin(receipt);
    }
    let cases = manifest["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 2);
    assert_eq!(
        cases
            .iter()
            .map(|case| (
                case["family"].as_str().unwrap(),
                case["file"].as_str().unwrap()
            ))
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([
            ("json-sequences", "source-arrays.hex"),
            ("json-sums", "all-sums.hex"),
        ])
    );
    // The emitter hex-encodes the operation name in its shared IntegerText
    // namespace. Keep the original exact six-block expectation.
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
        blocks
            .into_iter()
            .map(move |block| format!("Mpk.CSharp.Ordinary.IntegerText.O{encoded}.Block.B{block}"))
    })
    .collect::<BTreeSet<_>>();
    let mut rows = vec![];
    for case in cases {
        let family = case["family"].as_str().unwrap();
        let file = case["file"].as_str().unwrap();
        let old = decode(&pin(&case["before_cell_count"]));
        // Compare the original complete correction epoch. Current certificates
        // also contain a later DomainCount repair and cannot claim identical
        // historical syntax outside these six blocks.
        let new = decode(&pin(&case["after_cell_count"]));
        validate_csharp_practical_certificate_structure(&old).unwrap();
        validate_csharp_practical_certificate_structure(&new).unwrap();
        let roots = numeric_capacity_changes(&old, &new, &expected).unwrap();
        assert_eq!(
            roots.len() as u64,
            case["declarations_before_and_after_cell_count"]
                .as_u64()
                .unwrap()
        );
        let unreviewed = new
            .declarations
            .iter()
            .position(|d| {
                !expected.contains(&new.name_table[d.name as usize])
                    && matches!(d.kind, DeclarationKind::Def { ty, value, .. } if ty != value)
            })
            .unwrap();
        for change_type in [false, true] {
            let mut forged = new.clone();
            let DeclarationKind::Def { ty, value, .. } = &mut forged.declarations[unreviewed].kind
            else {
                unreachable!()
            };
            if change_type {
                *ty = *value;
            } else {
                *value = *ty;
            }
            assert!(numeric_capacity_changes(&old, &forged, &expected).is_err());
        }
        let mut missing = new.clone();
        missing.declarations.pop().unwrap();
        assert!(numeric_capacity_changes(&old, &missing, &expected).is_err());
        // These independently sealed current pins remain owned by the actual
        // json_sequence_source/json_sum_source generation/import/mutation tests.
        let current = decode(&pin(&case["current"]));
        validate_csharp_practical_certificate_structure(&current).unwrap();
        eprintln!(
            "Numeric capacity historical {family}: all{} names/types unchanged;exact six bodies changed;three mutations rejected;current pin separate",
            roots.len()
        );
        rows.push(json!({"family":family,"file":file,"declarations":roots.len(),"changed_definitions":expected,"old_term_count":old.term_table.len(),"new_term_count":new.term_table.len(),"scope":"historical_cell_count_correction_only","mutations_rejected":3,"current_source_proved":false}));
    }
    if let Some(path) = std::env::var_os("MPK_W09_JSON_CELL_DIFF_OUT") {
        fs::write(path, serde_json::to_vec_pretty(&rows).unwrap()).unwrap();
    }
}
