//! Private bridge to the unchanged retained importer, type validator and WP engine.
use mpk_vc::semantic_profile_registry::{
    canonical_registry_transport, validate_semantic_profile_registry, RegistryRevision,
};
use mpk_vc::successor_source_artifacts::{
    import_successor_vir_json, successor_contract_hash_value, successor_vir_hash_value,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::io::{self, Read};

fn hash(v: &Value) -> String {
    let raw = serde_json::to_vec(v).unwrap();
    let strict = mpk_vc::parse_strict_json(
        &raw,
        mpk_vc::StrictJsonLimits::new(1 << 28, 1 << 28, 256, 1 << 20),
    )
    .unwrap();
    let raw = mpk_vc::canonical_json_bytes(&strict).unwrap();
    Sha256::digest(raw)
        .iter()
        .map(|x| format!("{x:02x}"))
        .collect()
}
fn main() {
    let mut raw = String::new();
    io::stdin().read_to_string(&mut raw).unwrap();
    let input: Value = serde_json::from_str(&raw).unwrap();
    let registry = validate_semantic_profile_registry(
        &canonical_registry_transport(&input["registry"]).unwrap(),
        RegistryRevision::Revision3,
    )
    .unwrap();
    let rows=input["queries"].as_array().unwrap().iter().map(|q| {
        let attempt=(||->Result<Value,String>{
            let vir=if q.get("original_vir").is_some() {
                q["original_vir"].clone()
            } else {
            let mut units=q["units"].clone();let mut hashes=BTreeMap::new();
            for unit in units.as_array_mut().ok_or("units")? {
                for f in unit["functions"].as_array_mut().ok_or("functions")? {
                    let c=f["contracts"].as_object_mut().ok_or("contract")?;
                    c.insert("semantic_context".into(),q["context"].clone());
                    c.insert("contract_hash".into(),json!("0".repeat(64)));
                    let h=successor_contract_hash_value(&f["contracts"]).map_err(|e|e.to_string())?;
                    f["contracts"]["contract_hash"]=json!(h.as_str());
                    hashes.insert(f["id"].as_str().ok_or("function id")?.to_owned(),h.as_str().to_owned());
                }
            }
            for unit in units.as_array_mut().unwrap() {
                for f in unit["functions"].as_array_mut().unwrap() {
                    for b in f["blocks"].as_array_mut().ok_or("blocks")? {
                        for op in b["instructions"].as_array_mut().ok_or("instructions")? {
                            if op["kind"]=="CallStatic" {
                                let name=op["function"].as_str().ok_or("callee")?;
                                op["contract_hash"]=json!(hashes.get(name).ok_or("unknown callee")?);
                            }
                        }
                    }
                }
            }
            let mut vir=json!({"schema":"mpk.vir.v1","semantic_context":q["context"],"units":units,"vir_hash":"0".repeat(64)});
            vir["vir_hash"]=json!(successor_vir_hash_value(&vir).map_err(|e|e.to_string())?.as_str());
            vir
            };
            let parsed=import_successor_vir_json(&serde_json::to_vec(&vir).unwrap(),&registry).map_err(|e|e.to_string())?;
            let (projection,vcs)=mpk_vc::successor_vc::go_practical_shared_probe(&parsed)?;
            Ok(json!({"accepted":true,"projection_sha256":hash(&projection),"vc_sha256":hash(&vcs),
                "projection":projection,"vc_data":vcs}))
        })();
        attempt.unwrap_or_else(|e|json!({"accepted":false,"error":e}))
    }).collect::<Vec<_>>();
    println!("{}", serde_json::to_string(&rows).unwrap());
}
