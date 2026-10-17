//! Calls actual unchanged installed-family parameter/selection validators.
use mpk_vc::semantic_profile_registry::{
    canonical_registry_transport, validate_registry_selection_envelope,
    validate_registry_semantic_context, validate_semantic_profile_registry, RegistryRevision,
};
use serde_json::{json, Value};
use std::io::{self, Read};

fn main() {
    let mut raw = String::new();
    io::stdin().read_to_string(&mut raw).unwrap();
    let input: Value = serde_json::from_str(&raw).unwrap();
    let registry = validate_semantic_profile_registry(
        &canonical_registry_transport(&input["registry"]).unwrap(),
        RegistryRevision::Revision3,
    )
    .unwrap();
    let rows = input["queries"]
        .as_array()
        .unwrap()
        .iter()
        .map(|q| {
            let context = validate_registry_semantic_context(&registry, &q["context"]);
            let selected = context.as_ref().is_ok_and(|c| {
                !q["selection"].is_null()
                    && validate_registry_selection_envelope(&registry, c, &q["selection"]).is_ok()
            });
            json!({"context_accepted":context.is_ok(),"selection_accepted":selected})
        })
        .collect::<Vec<_>>();
    println!("{}", serde_json::to_string(&rows).unwrap());
}
