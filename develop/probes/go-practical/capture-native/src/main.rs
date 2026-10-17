//! Private observer of unchanged public retained source-map/manifest validators.
use mpk_vc::semantic_profile_registry::{
    canonical_registry_transport, validate_semantic_profile_registry, RegistryRevision,
};
use mpk_vc::source_manifest::ReleaseRegistryIdentity;
use mpk_vc::source_map::SyntheticPermission;
use mpk_vc::successor_source_artifacts::{
    import_successor_source_manifest_json, import_successor_source_map_json,
    import_successor_vir_json, successor_contract_hash_value, successor_source_manifest_hash_value,
    successor_source_map_hash_value, successor_vir_hash_value, SuccessorSourceManifestStage,
    SuccessorSourceManifestValidationContext, SuccessorSourceMapValidationContext,
};
use mpk_vc::{CapturedInput, InputKind};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::io::{self, Read};

fn hex(raw: &str) -> Result<Vec<u8>, String> {
    if !raw.is_ascii() || raw.len() % 2 != 0 {
        return Err("hex length".to_owned());
    }
    (0..raw.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&raw[i..i + 2], 16).map_err(|_| "hex digit".to_owned()))
        .collect()
}

fn vir_value(q: &Value) -> Result<Value, String> {
    if q.get("original").is_some() {
        return Ok(q["original"]["vir"].clone());
    }
    let mut units = q["units"].clone();
    let mut hashes = BTreeMap::new();
    for u in units.as_array_mut().ok_or("units")? {
        for f in u["functions"].as_array_mut().ok_or("functions")? {
            let c = f["contracts"].as_object_mut().ok_or("contract")?;
            c.insert("semantic_context".into(), q["context"].clone());
            c.insert("contract_hash".into(), json!("0".repeat(64)));
            let h = successor_contract_hash_value(&f["contracts"]).map_err(|e| e.to_string())?;
            f["contracts"]["contract_hash"] = json!(h.as_str());
            hashes.insert(
                f["id"].as_str().ok_or("function id")?.to_owned(),
                h.as_str().to_owned(),
            );
        }
    }
    for u in units.as_array_mut().unwrap() {
        for f in u["functions"].as_array_mut().unwrap() {
            for b in f["blocks"].as_array_mut().ok_or("blocks")? {
                for op in b["instructions"].as_array_mut().ok_or("instructions")? {
                    if op["kind"] == "CallStatic" {
                        op["contract_hash"] = json!(hashes
                            .get(op["function"].as_str().ok_or("callee")?)
                            .ok_or("unknown callee")?);
                    }
                }
            }
        }
    }
    let mut v = json!({"schema":"mpk.vir.v1","semantic_context":q["context"],"units":units,"vir_hash":"0".repeat(64)});
    v["vir_hash"] = json!(successor_vir_hash_value(&v)
        .map_err(|e| e.to_string())?
        .as_str());
    Ok(v)
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
    let mut out = Vec::new();
    for q in input["queries"].as_array().unwrap() {
        let result = (|| -> Result<Value, String> {
            let bytes = q["captured_inputs"]
                .as_array()
                .ok_or("captured inputs")?
                .iter()
                .map(|r| hex(r["raw_hex"].as_str().ok_or("raw input")?))
                .collect::<Result<Vec<_>, _>>()?;
            let captured = q["captured_inputs"]
                .as_array()
                .unwrap()
                .iter()
                .zip(&bytes)
                .map(|(r, b)| {
                    let kind: InputKind =
                        serde_json::from_value(r["kind"].clone()).map_err(|e| e.to_string())?;
                    Ok(CapturedInput {
                        kind,
                        normalized_path: r["normalized_path"].as_str().ok_or("input path")?,
                        bytes: b,
                    })
                })
                .collect::<Result<Vec<_>, String>>()?;
            let vir =
                import_successor_vir_json(&serde_json::to_vec(&vir_value(q)?).unwrap(), &registry)
                    .map_err(|e| e.to_string())?;
            let permissions = q["synthetic_permissions"]
                .as_array()
                .ok_or("synthetic permissions")?
                .iter()
                .map(|p| {
                    Ok(SyntheticPermission {
                        reference: serde_json::from_value(p["reference"].clone())
                            .map_err(|e| e.to_string())?,
                        reason: p["reason"].as_str().ok_or("synthetic reason")?.to_owned(),
                    })
                })
                .collect::<Result<Vec<_>, String>>()?;
            let mut map = if q.get("original").is_some() {
                q["original"]["source_map"].clone()
            } else {
                json!({"schema":"mpk.source_map.v1","semantic_context":q["context"],"source_ir_schema":"mpk.vir.v1","source_ir_hash":vir.hash().as_str(),"entries":q["entries"],"source_map_hash":"0".repeat(64)})
            };
            if q.get("original").is_none() {
                map["source_map_hash"] = json!(successor_source_map_hash_value(&map)
                    .map_err(|e| e.to_string())?
                    .as_str());
            }
            let map = import_successor_source_map_json(
                &serde_json::to_vec(&map).unwrap(),
                SuccessorSourceMapValidationContext {
                    registry: &registry,
                    vir: &vir,
                    captured_inputs: &captured,
                    synthetic_permissions: &permissions,
                },
            )
            .map_err(|e| e.to_string())?;
            if q["stage"] == "map" {
                return Ok(
                    json!({"accepted":true,"source_map":serde_json::to_value(map.map()).unwrap()}),
                );
            }
            if q["stage"] != "manifest" {
                return Err("closed observer stage".to_owned());
            }
            let release: ReleaseRegistryIdentity =
                serde_json::from_value(q["expected_release_registry"].clone())
                    .map_err(|e| e.to_string())?;
            let mut manifest = if q.get("original").is_some() {
                q["original"]["manifest"].clone()
            } else {
                q["manifest"].clone()
            };
            if q.get("original").is_none() {
                manifest["schema"] = json!("mpk.source_manifest.v1");
                manifest["semantic_context"] = q["context"].clone();
                manifest["vir_hash"] = json!(vir.hash().as_str());
                manifest["source_map_hash"] = json!(map.hash().as_str());
                manifest["source_manifest_hash"] = json!("0".repeat(64));
                manifest["source_manifest_hash"] =
                    json!(successor_source_manifest_hash_value(&manifest)
                        .map_err(|e| e.to_string())?
                        .as_str());
            }
            let m = import_successor_source_manifest_json(
                &serde_json::to_vec(&manifest).unwrap(),
                SuccessorSourceManifestStage::Frontend,
                SuccessorSourceManifestValidationContext {
                    registry: &registry,
                    vir: &vir,
                    source_map: &map,
                    captured_inputs: &captured,
                    expected_release_registry: &release,
                },
            )
            .map_err(|e| e.to_string())?;
            Ok(
                json!({"accepted":true,"source_map":serde_json::to_value(map.map()).unwrap(),"manifest":serde_json::to_value(m.manifest()).unwrap()}),
            )
        })();
        out.push(result.unwrap_or_else(|e| json!({"accepted":false,"error":e})));
    }
    println!("{}", serde_json::to_string(&out).unwrap());
}
