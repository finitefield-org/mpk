#[path = "../../mpk-cli/src/go_practical_checked_strings.rs"]
#[allow(dead_code)]
mod checked_strings;
#[path = "support/go_source_entry_inputs.rs"]
#[allow(dead_code)]
mod inputs;

use mpk_vc::go_practical_ordinary::{
    generate_go_practical_error_free_source_value_relations,
    generate_go_practical_source_string_catalogue,
    import_go_practical_error_free_source_value_relations, GoOrdinaryError,
};
use mpk_vc::go_practical_type_keys::{
    derive_go_source_contract_module, import_go_source_selection_inputs,
    import_go_type_key_components, GoSourceContractModule, GoSourceFunctionSelector,
    GoSourceOverlay, GoSourceTypeKey, GoTypeSourceBuffer,
};
use serde_json::{json, Value};
use std::collections::BTreeSet;

fn unhex(value: &Value) -> Vec<u8> {
    value
        .as_str()
        .unwrap()
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

fn original_contracts() -> GoSourceContractModule {
    let packet: Value = serde_json::from_slice(
        &std::fs::read(inputs::evidence().join(
            "selection-inputs/native/native-originals/selection-owned/selection-inputs.json",
        ))
        .unwrap(),
    )
    .unwrap();
    let buffers = |field: &str| {
        packet[field]
            .as_array()
            .unwrap()
            .iter()
            .map(|row| {
                GoTypeSourceBuffer::import(
                    &serde_json::to_vec(&row["object"]).unwrap(),
                    &unhex(&row["raw_hex"]),
                )
                .unwrap()
            })
            .collect::<Vec<_>>()
    };
    let selection = import_go_source_selection_inputs(
        &unhex(&packet["selection_hex"]),
        &unhex(&packet["envelope_hex"]),
        &unhex(&packet["components_hex"]),
        &buffers("sources"),
        &buffers("overlays"),
    )
    .unwrap();
    assert_eq!(selection.roots().len(), 3);
    assert_eq!(selection.contract_inputs().overlays().len(), 3);
    selection.derive_contracts().unwrap()
}

#[test]
fn complete_original_slice_domain_has_checked_relations() {
    let packet: Value = serde_json::from_slice(
        &std::fs::read(inputs::evidence().join("contract-modules/corpus-receivers.json")).unwrap(),
    )
    .unwrap();
    let original_sources = packet["input"]["sources"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            GoTypeSourceBuffer::import(
                &serde_json::to_vec(&row["object"]).unwrap(),
                &unhex(&row["raw_hex"]),
            )
            .unwrap()
        })
        .collect::<Vec<_>>();
    let keys = import_go_type_key_components(
        &unhex(&packet["input"]["components_hex"]),
        &original_sources,
    )
    .unwrap();
    let case = packet["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["name"] == "total-loops")
        .unwrap();
    let roots: Vec<GoSourceFunctionSelector> =
        serde_json::from_value(case["roots"].clone()).unwrap();
    let overlays = case["overlays"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            let original = GoTypeSourceBuffer::import(
                &serde_json::to_vec(&row["object"]).unwrap(),
                &unhex(&row["raw_hex"]),
            )
            .unwrap();
            GoSourceOverlay::import(&original).unwrap()
        })
        .collect::<Vec<_>>();
    let contracts = derive_go_source_contract_module(&keys, &roots, &overlays).unwrap();
    let domain = contracts.value_domain();
    assert!(domain
        .types()
        .any(|key| matches!(key, GoSourceTypeKey::Slice { .. })));
    let relations = generate_go_practical_error_free_source_value_relations(domain).unwrap();
    assert_eq!(
        relations.source_types().iter().collect::<Vec<_>>(),
        domain.types().collect::<Vec<_>>()
    );
    let report = mpk_kernel::verify_certificate_bytes(relations.certificate_bytes()).unwrap();
    assert_eq!(report.axiom_count, 0);
    inputs::write("error-free-slice.mpcert", relations.certificate_bytes());
    inputs::write(
        "error-free-slice.rust.json",
        mpk_kernel::render_verification_report_json(&report).as_bytes(),
    );
    eprintln!(
        "original slice domain types={}, declarations={}, bytes={}",
        relations.source_types().len(),
        report.declaration_count,
        relations.certificate_bytes().len()
    );
}

#[test]
fn complete_original_struct_receiver_domain_has_checked_relations() {
    let packet: Value = serde_json::from_slice(
        &std::fs::read(inputs::evidence().join("contract-expressions/corpus.json")).unwrap(),
    )
    .unwrap();
    let sources = packet["input"]["sources"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            GoTypeSourceBuffer::import(
                &serde_json::to_vec(&row["object"]).unwrap(),
                &unhex(&row["raw_hex"]),
            )
            .unwrap()
        })
        .collect::<Vec<_>>();
    let keys = import_go_type_key_components(&unhex(&packet["input"]["components_hex"]), &sources)
        .unwrap();
    let case = packet["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["native"]["function"]["function"] == "Method")
        .unwrap();
    let selector: GoSourceFunctionSelector =
        serde_json::from_value(case["native"]["function"].clone()).unwrap();
    let original = GoTypeSourceBuffer::import(
        &serde_json::to_vec(&case["object"]).unwrap(),
        &unhex(&case["raw_hex"]),
    )
    .unwrap();
    let overlay = GoSourceOverlay::import(&original).unwrap();
    let contracts = derive_go_source_contract_module(&keys, &[selector], &[overlay]).unwrap();
    let domain = contracts.value_domain();
    assert!(domain
        .types()
        .any(|key| matches!(key, GoSourceTypeKey::Struct { .. })));
    assert!(domain
        .types()
        .any(|key| matches!(key, GoSourceTypeKey::Slice { .. })));
    let relations = generate_go_practical_error_free_source_value_relations(domain).unwrap();
    assert_eq!(
        relations.source_types().iter().collect::<Vec<_>>(),
        domain.types().collect::<Vec<_>>()
    );
    let report = mpk_kernel::verify_certificate_bytes(relations.certificate_bytes()).unwrap();
    assert_eq!(report.axiom_count, 0);
    inputs::write("error-free-struct.mpcert", relations.certificate_bytes());
    inputs::write(
        "error-free-struct.rust.json",
        mpk_kernel::render_verification_report_json(&report).as_bytes(),
    );
    eprintln!(
        "original struct domain types={}, declarations={}, bytes={}",
        relations.source_types().len(),
        report.declaration_count,
        relations.certificate_bytes().len()
    );
}

#[test]
fn contract_only_original_type_is_retained_in_error_free_relations() {
    let packet: Value = serde_json::from_slice(
        &std::fs::read(inputs::evidence().join("contract-expressions/corpus.json")).unwrap(),
    )
    .unwrap();
    let sources = packet["input"]["sources"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            GoTypeSourceBuffer::import(
                &serde_json::to_vec(&row["object"]).unwrap(),
                &unhex(&row["raw_hex"]),
            )
            .unwrap()
        })
        .collect::<Vec<_>>();
    let keys = import_go_type_key_components(&unhex(&packet["input"]["components_hex"]), &sources)
        .unwrap();
    let case = packet["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["native"]["function"]["function"] == "Method")
        .unwrap();
    let selector: GoSourceFunctionSelector =
        serde_json::from_value(case["native"]["function"].clone()).unwrap();
    let mut annotation: Value = serde_json::from_slice(&unhex(&case["raw_hex"])).unwrap();
    let zero = json!({"op":"source.zero","type":"example.com/policy.ContractOnly"});
    annotation["ensures"] = json!([{"op":"eq","args":[zero.clone(),zero]}]);
    let candidate = GoTypeSourceBuffer::new(
        "contract-only-candidate",
        "method.json",
        "overlay",
        &serde_json::to_vec(&annotation).unwrap(),
    )
    .unwrap();
    let overlay = GoSourceOverlay::import(&candidate).unwrap();
    let contracts = derive_go_source_contract_module(&keys, &[selector], &[overlay]).unwrap();
    let finite = contracts
        .finite_selection()
        .domain()
        .types()
        .map(|key| key.id().to_string())
        .collect::<BTreeSet<_>>();
    let domain = contracts.value_domain();
    let extra = domain
        .types()
        .filter(|key| !finite.contains(key.id()))
        .collect::<Vec<_>>();
    assert!(
        !extra.is_empty(),
        "the original annotation-only type was lost"
    );
    let relations = generate_go_practical_error_free_source_value_relations(domain).unwrap();
    for key in extra {
        assert!(relations.source_types().contains(key));
    }
    let report = mpk_kernel::verify_certificate_bytes(relations.certificate_bytes()).unwrap();
    assert_eq!(report.axiom_count, 0);
    inputs::write(
        "error-free-contract-only.mpcert",
        relations.certificate_bytes(),
    );
    inputs::write(
        "error-free-contract-only.rust.json",
        mpk_kernel::render_verification_report_json(&report).as_bytes(),
    );
    eprintln!(
        "contract-only domain types={}, declarations={}, bytes={}",
        relations.source_types().len(),
        report.declaration_count,
        relations.certificate_bytes().len()
    );
}

#[test]
fn complete_original_error_free_domain_has_checked_relations() {
    let contracts = original_contracts();
    assert_eq!(contracts.functions().len(), 4);
    let domain = contracts.value_domain();
    assert!(domain.error_cases().is_empty());
    assert!(!domain
        .types()
        .any(|key| matches!(key, GoSourceTypeKey::Builtin { name, .. } if name == "error")));
    let relations = generate_go_practical_error_free_source_value_relations(domain).unwrap();
    assert_eq!(
        relations.source_types().iter().collect::<Vec<_>>(),
        domain.types().collect::<Vec<_>>()
    );
    for key in domain.types() {
        assert_eq!(
            relations.underlying_type_id(key.id()),
            domain.underlying(key.id())
        );
    }
    let certificate =
        mpk_cert::decode_canonical_certificate(relations.certificate_bytes()).unwrap();
    for key in domain.types() {
        for member in [
            "View",
            "Related",
            "Reflexive",
            "Symmetric",
            "Transitive",
            "OfEquality",
            "ZeroRelated",
        ] {
            let name = format!("{}.T{}.{member}", relations.module_name(), key.id());
            assert!(
                certificate
                    .declarations
                    .iter()
                    .any(|declaration| certificate.name_table[declaration.name as usize] == name),
                "missing complete original relation {name}"
            );
        }
    }
    let report = mpk_kernel::verify_certificate_bytes(relations.certificate_bytes()).unwrap();
    assert_eq!(report.axiom_count, 0);
    let imported = import_go_practical_error_free_source_value_relations(
        relations.certificate_bytes(),
        domain,
    )
    .unwrap();
    assert_eq!(imported.certificate_bytes(), relations.certificate_bytes());
    let mut altered = relations.certificate_bytes().to_vec();
    altered[0] ^= 1;
    assert!(matches!(
        import_go_practical_error_free_source_value_relations(&altered, domain),
        Err(GoOrdinaryError::Linkage)
    ));
    inputs::write("error-free-relations.mpcert", relations.certificate_bytes());
    inputs::write(
        "error-free-relations.rust.json",
        mpk_kernel::render_verification_report_json(&report).as_bytes(),
    );
    eprintln!(
        "original types={}, declarations={}, bytes={}",
        relations.source_types().len(),
        report.declaration_count,
        relations.certificate_bytes().len()
    );
}

#[test]
fn complete_original_domains_check_all_strings() {
    let contracts = original_contracts();
    let empty_domain = contracts.value_domain();
    assert!(empty_domain.error_cases().is_empty());
    let empty = generate_go_practical_source_string_catalogue(empty_domain).unwrap();
    assert!(empty.source_string_type_ids().is_empty());
    assert!(
        checked_strings::check_go_string_catalogue(&empty, empty_domain)
            .unwrap()
            .receipts()
            .is_empty()
    );

    let pure = inputs::original("pure");
    let domain = pure.domain();
    assert!(domain.error_cases().is_empty());
    // The full original selection contains an error-typed signature even
    // though its dynamic error universe is empty.
    assert!(domain
        .types()
        .any(|key| matches!(key, GoSourceTypeKey::Builtin { name, .. } if name == "error")));
    let catalogue = generate_go_practical_source_string_catalogue(domain).unwrap();
    assert!(!catalogue.source_string_type_ids().is_empty());
    let checked = checked_strings::check_go_string_catalogue(&catalogue, domain).unwrap();
    assert_eq!(
        checked.receipts().len(),
        catalogue.source_string_type_ids().len()
    );
    for (index, receipt) in checked.receipts().iter().enumerate() {
        let row = checked.reconstruct(&catalogue, index).unwrap();
        assert_eq!(row.source_string_type_id(), receipt.source_type_id());
        assert_eq!(
            mpk_cert::certificate_hash(row.certificate_bytes()),
            *receipt.certificate_hash()
        );
    }
    assert!(matches!(
        checked_strings::check_go_string_catalogue(&catalogue, empty_domain),
        Err(checked_strings::GoCheckedStringError::Coverage { .. })
    ));
    eprintln!(
        "complete original pure domain: {} strings",
        checked.receipts().len()
    );

    // An additional original root accepts a string without mentioning error.
    // All original package source buffers remain in the imported components.
    let packets: Value = serde_json::from_slice(
        &std::fs::read(inputs::evidence().join("source-inputs/corpus.json")).unwrap(),
    )
    .unwrap();
    let input = &packets["units"]
        .as_array()
        .unwrap()
        .iter()
        .find(|unit| unit["name"] == "pure")
        .unwrap()["input"];
    let sources = input["sources"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            GoTypeSourceBuffer::import(
                &serde_json::to_vec(&row["object"]).unwrap(),
                &unhex(&row["raw_hex"]),
            )
            .unwrap()
        })
        .collect::<Vec<_>>();
    assert_eq!(sources.len(), 4);
    let keys = import_go_type_key_components(&unhex(&input["components_hex"]), &sources).unwrap();
    let roots: Vec<GoSourceFunctionSelector> = serde_json::from_value(json!([
        {"package":"example.com/policy/shadow","function":"Length","receiver":null}
    ]))
    .unwrap();
    let contracts = derive_go_source_contract_module(&keys, &roots, &[]).unwrap();
    let domain = contracts.value_domain();
    assert!(domain.error_cases().is_empty());
    assert!(!domain
        .types()
        .any(|key| matches!(key, GoSourceTypeKey::Builtin { name, .. } if name == "error")));
    let catalogue = generate_go_practical_source_string_catalogue(domain).unwrap();
    assert_eq!(catalogue.source_string_type_ids().len(), 1);
    let checked = checked_strings::check_go_string_catalogue(&catalogue, domain).unwrap();
    assert_eq!(checked.receipts().len(), 1);
    let row = checked.reconstruct(&catalogue, 0).unwrap();
    assert_eq!(
        row.source_string_type_id(),
        checked.receipts()[0].source_type_id()
    );
    let receipt = &checked.receipts()[0];
    inputs::write("error-free-source-string.mpcert", row.certificate_bytes());
    inputs::write(
        "error-free-source-string.rust.json",
        &serde_json::to_vec(&json!({
            "verdict": "accepted", "module": receipt.module(),
            "declaration_count": receipt.declarations(), "axiom_count": 0,
            "hashes": {"export": mpk_cert::hash_hex(receipt.export_hash()),
                "axiom_report": mpk_cert::hash_hex(receipt.axiom_report_hash()),
                "certificate": mpk_cert::hash_hex(receipt.certificate_hash())}
        }))
        .unwrap(),
    );
}

#[test]
fn complete_original_error_domain_is_not_relabelled_as_error_free() {
    for (name, cases) in [("pure", 0), ("wide", 70)] {
        let selected = inputs::original(name);
        assert_eq!(selected.domain().error_cases().len(), cases);
        assert!(selected
            .domain()
            .types()
            .any(|key| matches!(key, GoSourceTypeKey::Builtin { name, .. } if name == "error")));
        assert!(matches!(
            generate_go_practical_error_free_source_value_relations(selected.domain()),
            Err(GoOrdinaryError::Linkage)
        ));
    }
}
