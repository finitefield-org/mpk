#[path = "../src/go_practical_checked_entries.rs"]
#[allow(dead_code)]
mod checked_entries;
#[path = "../src/go_practical_checked_invocations.rs"]
#[allow(dead_code)]
mod checked_invocations;
#[path = "../../mpk-vc/tests/support/go_source_entry_inputs.rs"]
#[allow(dead_code)]
mod inputs;

use mpk_vc::go_practical_context::{GoPrivateCandidateRegistry, GoPrivateCandidateRequest};
use mpk_vc::go_practical_ordinary::generate_go_practical_source_entry_catalogue;
use mpk_vc::go_practical_type_keys::{
    bind_go_practical_source_selection, import_go_source_selection_inputs, GoTypeSourceBuffer,
};
use mpk_vc::{
    canonical_json_bytes, hash_canonical_json, parse_strict_json, HashDomain, StrictJsonLimits,
};
use serde_json::{json, Value};

fn candidate_request() -> GoPrivateCandidateRequest {
    let limits = StrictJsonLimits::new(268_435_456, 268_435_456, 64, 16_777_216);
    let strict =
        |value: &Value| parse_strict_json(&serde_json::to_vec(value).unwrap(), limits).unwrap();
    let mut roots = vec![
        json!({"package":"example.com/policy","function":"Root","receiver":null}),
        json!({"package":"example.com/policy","function":"Work","receiver":{"package":"example.com/policy","type_name":"Count"}}),
        json!({"package":"example.com/policy","function":"Work","receiver":{"package":"example.com/policy","type_name":"Other"}}),
    ];
    roots.sort_by_cached_key(|root| canonical_json_bytes(&strict(root)).unwrap());
    let mut wire: Value = serde_json::from_slice(include_bytes!(
        "../../mpk-vc/tests/fixtures/go-practical-candidate-request.json"
    ))
    .unwrap();
    wire["selection"]["value"]["roots"] = json!(roots);
    wire.as_object_mut().unwrap().remove("request_sha256");
    wire["request_sha256"] = json!(hash_canonical_json(
        HashDomain::new("MPK-VALIDATED-SEMANTIC-REQUEST-3.0"),
        &strict(&wire)
    )
    .unwrap()
    .to_hex());
    GoPrivateCandidateRegistry::new()
        .unwrap()
        .import_request(&strict(&wire))
        .unwrap()
}

fn unhex(value: &str) -> Vec<u8> {
    assert_eq!(value.len() % 2, 0);
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

fn buffers(packet: &Value, field: &str) -> Vec<GoTypeSourceBuffer> {
    packet[field]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            GoTypeSourceBuffer::import(
                &serde_json::to_vec(&row["object"]).unwrap(),
                &unhex(row["raw_hex"].as_str().unwrap()),
            )
            .unwrap()
        })
        .collect()
}

fn check_original(name: &str, functions: usize, parts: usize, cases: usize) {
    let unit = inputs::original(name);
    assert_eq!(unit.domain().error_cases().len(), cases);
    let catalogue = generate_go_practical_source_entry_catalogue(unit.domain()).unwrap();
    assert_eq!(catalogue.layouts().len(), functions);
    let checked = checked_entries::check_go_entry_catalogue(&catalogue).unwrap();
    assert_eq!(checked.assemblies().parts().len(), parts);
    assert_eq!(checked.reports().len(), parts);
    assert_eq!(
        checked
            .assemblies()
            .parts()
            .iter()
            .map(|part| part.function_modules().len())
            .sum::<usize>(),
        functions
    );
    for (index, (part, report)) in checked
        .assemblies()
        .parts()
        .iter()
        .zip(checked.reports())
        .enumerate()
    {
        assert_eq!(part.case_type_ids().len(), cases);
        assert_eq!(report.axiom_count, 0);
        assert!(part.certificate_bytes().len() <= 2_097_152);
        inputs::write(
            &format!("checked-{name}-{index}.mpcert"),
            part.certificate_bytes(),
        );
        inputs::write(
            &format!("checked-{name}-{index}.rust.json"),
            mpk_kernel::render_verification_report_json(report).as_bytes(),
        );
        eprintln!(
            "{name} part {index}: {} functions, {} declarations, {} bytes",
            part.function_modules().len(),
            report.declaration_count,
            part.certificate_bytes().len()
        );
    }
}

#[test]
fn checks_every_original_pure_entry() {
    check_original("pure", 47, 1, 0);
}

#[test]
fn checks_every_original_wide_entry_across_complete_parts() {
    check_original("wide", 117, 2, 70);
}

#[test]
fn matches_contract_roots_and_transitive_bodies_without_crossing_sources() {
    let packet: Value = serde_json::from_slice(
        &std::fs::read(inputs::evidence().join(
            "selection-inputs/native/native-originals/selection-owned/selection-inputs.json",
        ))
        .unwrap(),
    )
    .unwrap();
    let inputs = import_go_source_selection_inputs(
        &unhex(packet["selection_hex"].as_str().unwrap()),
        &unhex(packet["envelope_hex"].as_str().unwrap()),
        &unhex(packet["components_hex"].as_str().unwrap()),
        &buffers(&packet, "sources"),
        &buffers(&packet, "overlays"),
    )
    .unwrap();
    assert_eq!(inputs.roots().len(), 3);
    assert_eq!(inputs.contract_inputs().overlays().len(), 3);
    let module = inputs.derive_contracts().unwrap();
    assert_eq!(module.functions().len(), 4);
    let catalogue = generate_go_practical_source_entry_catalogue(module.value_domain()).unwrap();
    assert_eq!(catalogue.layouts().len(), 4);
    assert!(checked_entries::catalogue_covers_contracts(
        &module, &catalogue
    ));

    // This is the separate, complete original entry project, not a modified
    // or synthetic subset of either source owner.
    let other = inputs::supplement();
    let other_catalogue = generate_go_practical_source_entry_catalogue(other.domain()).unwrap();
    assert_eq!(other_catalogue.layouts().len(), 4);
    assert!(!checked_entries::catalogue_covers_contracts(
        &module,
        &other_catalogue
    ));

    let request = candidate_request();
    let invocations = bind_go_practical_source_selection(&request, &inputs)
        .unwrap()
        .derive_contracts()
        .unwrap()
        .derive_invocation_catalogue()
        .unwrap();
    let checked = checked_invocations::check_go_invocations(invocations).unwrap();
    assert_eq!(
        checked.invocations().contracts().module().functions().len(),
        4
    );
    assert_eq!(checked.checked().receipts().len(), 1);
    let receipt = &checked.checked().receipts()[0];
    assert_eq!(
        receipt.site(),
        &checked.invocations().catalogue().sites()[0]
    );
    assert_eq!(receipt.declarations(), 2091);
    assert_eq!(
        receipt.module(),
        "Mpk.Go.Practical.SourceInvocation.S30b86b1c8615d7f04152796f61663db38dc1c9b52bd876ba4e8b5edf1d0d4ced"
    );
    assert_eq!(
        mpk_cert::hash_hex(receipt.export_hash()),
        "00e2de41631c0d33c3e868561187a21f754ef003144bc49d3de39de57ed108c7"
    );
    assert_eq!(
        mpk_cert::hash_hex(receipt.axiom_report_hash()),
        "0ebc281c3a8d37e2d1a9ce033773e2865f96a13186a6364cb3446204c6a990d5"
    );
    assert_eq!(
        mpk_cert::hash_hex(receipt.certificate_hash()),
        "9838a820776a5d5a62f79d589127531704f0bd6b99c2931b3fa16ba4ffd60f9f"
    );
    let reconstructed = checked.reconstruct(0).unwrap();
    assert_eq!(reconstructed.site(), receipt.site());
    assert_eq!(
        mpk_cert::certificate_hash(reconstructed.certificate_bytes()),
        *receipt.certificate_hash()
    );
    assert!(matches!(
        checked.reconstruct(1),
        Err(checked_invocations::GoCheckedInvocationError::Coverage { site: 1 })
    ));
}
