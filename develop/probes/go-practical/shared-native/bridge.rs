// Appended only in a private mirror. All called production functions are unchanged.
pub fn go_practical_shared_probe(source: &ValidatedSuccessorVir) -> Result<(Value, Value), String> {
    let (profile, checks) = match source.module().semantic_context().semantic_profile() {
        "mpk.go.fixed.v0" => (
            CompiledSemanticProfile::GoFixedV0,
            CompiledRequiredCheckProfile::GoFixedV0,
        ),
        "mpk.rust.checked.v0" => (
            CompiledSemanticProfile::RustCheckedV0,
            CompiledRequiredCheckProfile::RustCheckedV0,
        ),
        _ => return Err("unknown retained profile".to_owned()),
    };
    let projection = project_successor_vir(source, profile).map_err(|e| e.to_string())?;
    validate_projection(&projection, profile).map_err(|e| e.to_string())?;
    let generated = generate_program_vcs_from_profiled_projection(&projection, checks)
        .map_err(|e| e.to_string())?;
    let functions = generated
        .module
        .functions
        .iter()
        .map(|f| {
            let members = f.members.iter().map(|m| serde_json::json!({
            "id":m.id,"function_id":m.function_id,"kind":m.kind.as_str(),
            "local_binders":m.local_binders,"assumptions":m.assumptions,"conclusion":m.conclusion,
            "group_id":m.group_id,"safety_evidence":format!("{:?}",m.safety_evidence)
        })).collect::<Vec<_>>();
            serde_json::json!({"function_id":f.function_id,"requires":f.requires,"members":members,
            "direct_callees":f.direct_callees,"contract_dependencies":f.contract_dependencies,
            "panic_free_dependencies":f.panic_free_dependencies})
        })
        .collect::<Vec<_>>();
    Ok((
        serde_json::to_value(&projection).map_err(|e| e.to_string())?,
        serde_json::json!({"functions":functions,"safety_origins":format!("{:?}",generated.safety_origins)}),
    ))
}
