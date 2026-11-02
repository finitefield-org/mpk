#[path = "../src/go_practical_checked_entries.rs"]
#[allow(dead_code)]
mod checked_entries;
#[path = "../../mpk-vc/tests/support/go_source_entry_inputs.rs"]
#[allow(dead_code)]
mod inputs;

use mpk_vc::go_practical_ordinary::generate_go_practical_source_entry_catalogue;
use mpk_vc::go_practical_type_keys::{import_go_source_selection_inputs, GoTypeSourceBuffer};
use serde_json::Value;

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
}
