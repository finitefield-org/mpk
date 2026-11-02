//! Actual retained VC/skeleton regeneration and certificate-manifest observer.
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
use mpk_vc::successor_vc::{
    emit_successor_vc_skeleton, generate_successor_vc, import_successor_vc_json,
    import_successor_vc_skeleton_json, SuccessorVcSource,
};
use mpk_vc::{
    canonical_json_bytes_bounded, parse_strict_json, CapturedInput, InputKind, StrictJsonLimits,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::io::{self, Read};

fn canonical(v: &Value) -> Result<Vec<u8>, String> {
    let raw = serde_json::to_vec(v).map_err(|e| e.to_string())?;
    let strict = parse_strict_json(&raw, StrictJsonLimits::new(1 << 28, 1 << 28, 768, 1 << 20))
        .map_err(|e| e.to_string())?;
    canonical_json_bytes_bounded(&strict, 1 << 28).map_err(|e| e.to_string())
}
fn hash(v: &Value, domain: &str, field: &str) -> Result<String, String> {
    let mut v = v.clone();
    v.as_object_mut().ok_or("hash object")?.remove(field);
    let mut h = Sha256::new();
    h.update(domain.as_bytes());
    h.update([0]);
    h.update(canonical(&v)?);
    Ok(h.finalize().iter().map(|x| format!("{x:02x}")).collect())
}
fn unhex(raw: &str) -> Result<Vec<u8>, String> {
    if !raw.is_ascii() || raw.len() % 2 != 0 {
        return Err("hex shape".into());
    }
    (0..raw.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&raw[i..i + 2], 16).map_err(|_| "hex digit".into()))
        .collect()
}
fn hash_observation(q: &Value) -> Value {
    let attempt = (|| -> Result<(String, String), String> {
        let domain = q["domain"].as_str().ok_or("domain")?;
        let (prefix, version) = domain.rsplit_once('-').ok_or("domain")?;
        let (major, minor) = version.split_once('.').ok_or("version")?;
        if !prefix
            .as_bytes()
            .first()
            .is_some_and(u8::is_ascii_uppercase)
            || !prefix
                .bytes()
                .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'-')
            || major.is_empty()
            || minor.is_empty()
            || !major.bytes().all(|b| b.is_ascii_digit())
            || !minor.bytes().all(|b| b.is_ascii_digit())
        {
            return Err("domain grammar".into());
        }
        let raw = unhex(q["raw_json_hex"].as_str().ok_or("raw")?)?;
        let v = parse_strict_json(&raw, StrictJsonLimits::new(1 << 28, 1 << 28, 768, 1 << 20))
            .map_err(|e| e.to_string())?;
        let raw = canonical_json_bytes_bounded(&v, 1 << 28).map_err(|e| e.to_string())?;
        let mut h = Sha256::new();
        h.update(domain.as_bytes());
        h.update([0]);
        h.update(&raw);
        Ok((
            raw.iter().map(|b| format!("{b:02x}")).collect(),
            h.finalize().iter().map(|b| format!("{b:02x}")).collect(),
        ))
    })();
    match attempt {
        Ok((c, h)) => json!({"id":q["id"],"accepted":true,"canonical_hex":c,"digest":h}),
        Err(_) => json!({"id":q["id"],"accepted":false}),
    }
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
                f["id"].as_str().ok_or("function")?.to_owned(),
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
                            .ok_or("callee")?);
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
        RegistryRevision::Revision4,
    )
    .unwrap();
    let mut out = Vec::new();
    for q in input["queries"].as_array().unwrap() {
        if q.get("raw_json_hex").is_some() {
            out.push(hash_observation(q));
            continue;
        }
        let attempt = (|| -> Result<Value, String> {
            let bytes = q["captured_inputs"]
                .as_array()
                .ok_or("captured")?
                .iter()
                .map(|r| unhex(r["raw_hex"].as_str().ok_or("raw")?))
                .collect::<Result<Vec<_>, _>>()?;
            let captured = q["captured_inputs"]
                .as_array()
                .unwrap()
                .iter()
                .zip(&bytes)
                .map(|(r, b)| {
                    Ok(CapturedInput {
                        kind: serde_json::from_value::<InputKind>(r["kind"].clone())
                            .map_err(|e| e.to_string())?,
                        normalized_path: r["normalized_path"].as_str().ok_or("path")?,
                        bytes: b,
                    })
                })
                .collect::<Result<Vec<_>, String>>()?;
            let permissions = q["synthetic_permissions"]
                .as_array()
                .ok_or("permissions")?
                .iter()
                .map(|p| {
                    Ok(SyntheticPermission {
                        reference: serde_json::from_value(p["reference"].clone())
                            .map_err(|e| e.to_string())?,
                        reason: p["reason"].as_str().ok_or("reason")?.to_owned(),
                    })
                })
                .collect::<Result<Vec<_>, String>>()?;
            let vir = import_successor_vir_json(&canonical(&vir_value(q)?)?, &registry)
                .map_err(|e| e.to_string())?;
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
                &canonical(&map)?,
                SuccessorSourceMapValidationContext {
                    registry: &registry,
                    vir: &vir,
                    captured_inputs: &captured,
                    synthetic_permissions: &permissions,
                },
            )
            .map_err(|e| e.to_string())?;
            let expected_release: ReleaseRegistryIdentity =
                serde_json::from_value(q["expected_release_registry"].clone())
                    .map_err(|e| e.to_string())?;
            let context = SuccessorSourceManifestValidationContext {
                registry: &registry,
                vir: &vir,
                source_map: &map,
                captured_inputs: &captured,
                expected_release_registry: &expected_release,
            };
            let mut m = if q.get("original").is_some() {
                q["original"]["manifest"].clone()
            } else {
                q["manifest"].clone()
            };
            if q.get("original").is_none() {
                m["schema"] = json!("mpk.source_manifest.v1");
                m["semantic_context"] = q["context"].clone();
                m["vir_hash"] = json!(vir.hash().as_str());
                m["source_map_hash"] = json!(map.hash().as_str());
                m["source_manifest_hash"] = json!("0".repeat(64));
                m["source_manifest_hash"] = json!(successor_source_manifest_hash_value(&m)
                    .map_err(|e| e.to_string())?
                    .as_str());
            }
            let manifest = import_successor_source_manifest_json(
                &canonical(&m)?,
                SuccessorSourceManifestStage::Frontend,
                context,
            )
            .map_err(|e| e.to_string())?;
            let source = SuccessorVcSource {
                registry: &registry,
                vir: &vir,
                manifest: &manifest,
                profile_contract: &q["profile_contract"],
            };
            let generated = generate_successor_vc(source).map_err(|e| e.to_string())?;
            let mut v = serde_json::to_value(generated.document()).map_err(|e| e.to_string())?;
            if let Some(payload) = q.get("vc_payload") {
                v["functions"] = payload["functions"].clone();
                v["verification_limit_profile"] = payload["verification_limit_profile"].clone();
                v["vc_hash"] = json!(hash(&v, "MPK-VC-2.0", "vc_hash")?);
            }
            let vc =
                import_successor_vc_json(&canonical(&v)?, source).map_err(|e| e.to_string())?;
            let skeleton = emit_successor_vc_skeleton(&vc, source).map_err(|e| e.to_string())?;
            let mut s = serde_json::to_value(skeleton.skeleton()).map_err(|e| e.to_string())?;
            if let Some(decls) = q.get("theorem_declarations") {
                s["theorem_declarations"] = decls.clone();
            }
            let skeleton =
                import_successor_vc_skeleton_json(&canonical(&s)?, vc.canonical_bytes(), source)
                    .map_err(|e| e.to_string())?;
            m["vc_hash"] = json!(vc.hash().as_str());
            m["source_manifest_hash"] = json!(successor_source_manifest_hash_value(&m)
                .map_err(|e| e.to_string())?
                .as_str());
            let certificate = import_successor_source_manifest_json(
                &canonical(&m)?,
                SuccessorSourceManifestStage::Certificate,
                context,
            )
            .map_err(|e| e.to_string())?;
            Ok(
                json!({"accepted":true,"vc":serde_json::to_value(vc.document()).unwrap(),"skeleton":serde_json::to_value(skeleton.skeleton()).unwrap(),"certificate_manifest":serde_json::to_value(certificate.manifest()).unwrap()}),
            )
        })();
        out.push(attempt.unwrap_or_else(|e| json!({"accepted":false,"error":e})));
    }
    println!("{}", serde_json::to_string(&out).unwrap());
}
