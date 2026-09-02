use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use mpk_vc::semantic_profile::{SemanticParameters, SemanticProfile, SourceLanguage};
use mpk_vc::semantic_profile_registry::{validate_semantic_profile_registry, RegistryRevision};
use mpk_vc::successor_source_artifacts::{import_successor_vir_json, ValidatedSuccessorVir};
use mpk_vc::{
    canonical_vir_json, contract_hash, import_vir_json, vir_hash, LowercaseSha256, VirContract,
    VirFunction, VirInstruction, VirModule, VirUnit, VIR_SCHEMA_VERSION,
};

#[allow(dead_code)]
pub fn import_successor_rust_vir_projection(input: &[u8]) -> VirModule {
    let projection = import_successor_vir_projection(input);
    assert_eq!(projection.source_language, SourceLanguage::Rust);
    assert_eq!(projection.semantic_profile, SemanticProfile::RustCheckedV0);
    projection
}

pub fn import_successor_vir_projection(input: &[u8]) -> VirModule {
    let canonical = input.strip_suffix(b"\n").unwrap_or(input);
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let registry_bytes = fs::read(root.join("release/bundles/semantic-profile-registry.json"))
        .expect("semantic-profile registry");
    let registry = validate_semantic_profile_registry(&registry_bytes, RegistryRevision::Revision3)
        .expect("revision-3 semantic-profile registry");
    let successor = import_successor_vir_json(canonical, &registry)
        .expect("successor VIR passes independent validation");
    project_successor_vir(&successor)
}

fn project_successor_vir(successor: &ValidatedSuccessorVir) -> VirModule {
    let context = successor.module().semantic_context();
    let source_language: SourceLanguage = serde_json::from_value(serde_json::Value::String(
        context.source_language().to_owned(),
    ))
    .expect("VC-core source language");
    let semantic_profile: SemanticProfile = serde_json::from_value(serde_json::Value::String(
        context.semantic_profile().to_owned(),
    ))
    .expect("VC-core semantic profile");
    let parameters: SemanticParameters =
        serde_json::from_value(context.semantic_parameters().value().clone())
            .expect("typed VC-core semantic parameters");
    assert_eq!(source_language, semantic_profile.source_language());
    assert_eq!(semantic_profile, parameters.profile());
    let mut units: Vec<VirUnit> = successor
        .module()
        .units()
        .iter()
        .map(|unit| VirUnit {
            id: unit.id().to_owned(),
            name: unit.name().to_owned(),
            type_decls: unit.type_decls().to_vec(),
            const_decls: unit.const_decls().to_vec(),
            functions: unit
                .functions()
                .iter()
                .map(|function| {
                    let successor_contract = function.contracts();
                    let mut contract = VirContract {
                        unit_id: successor_contract.unit_id().to_owned(),
                        function_id: successor_contract.function_id().to_owned(),
                        semantic_profile,
                        semantic_parameters: parameters.clone(),
                        requires: successor_contract.requires().to_vec(),
                        ensures: successor_contract.ensures().to_vec(),
                        modifies: successor_contract.modifies().to_vec(),
                        panic: successor_contract.panic(),
                        termination: successor_contract.termination(),
                        loops: successor_contract.loops().to_vec(),
                        contract_hash: zero_hash(),
                    };
                    contract.contract_hash =
                        contract_hash(&contract).expect("projected contract hash");
                    VirFunction {
                        id: function.id().to_owned(),
                        unit_id: function.unit_id().to_owned(),
                        name: function.name().to_owned(),
                        params: function.params().to_vec(),
                        results: function.results().to_vec(),
                        locals: function.locals().to_vec(),
                        blocks: function.blocks().to_vec(),
                        contracts: contract,
                        features_used: function.features_used().to_vec(),
                    }
                })
                .collect(),
        })
        .collect();
    let active_contract_hashes = units
        .iter()
        .flat_map(|unit| &unit.functions)
        .map(|function| {
            (
                function.id.clone(),
                function.contracts.contract_hash.clone(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    for function in units.iter_mut().flat_map(|unit| &mut unit.functions) {
        for instruction in function
            .blocks
            .iter_mut()
            .flat_map(|block| &mut block.instructions)
        {
            if let VirInstruction::CallStatic {
                function,
                contract_hash,
                ..
            } = instruction
            {
                *contract_hash = active_contract_hashes
                    .get(function)
                    .cloned()
                    .expect("static callee belongs to the validated successor VIR");
            }
        }
    }
    let mut projection = VirModule {
        schema: VIR_SCHEMA_VERSION.to_owned(),
        source_language,
        semantic_profile,
        semantic_parameters: parameters,
        units,
        vir_hash: zero_hash(),
    };
    projection.vir_hash = vir_hash(&projection).expect("projected VIR hash");
    let canonical = canonical_vir_json(&projection).expect("canonical projected VIR");
    import_vir_json(&canonical).expect("projected VIR passes the VC-core boundary")
}

fn zero_hash() -> LowercaseSha256 {
    LowercaseSha256::new("0".repeat(64)).expect("zero hash")
}
