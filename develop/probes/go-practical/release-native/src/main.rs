//! Release metadata and compact canonical-hash observations,not installation.
use mpk_vc::release_bundle_v1::{
    validate_successor_release_registry, SuccessorReleaseSelectionRequest,
};
use mpk_vc::semantic_profile_registry::{validate_semantic_profile_registry, RegistryRevision};
use mpk_vc::{canonical_json_bytes_bounded, parse_strict_json, StrictJsonLimits};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::io::{self, Read};

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}
fn sha(b: &[u8]) -> String {
    hex(&Sha256::digest(b))
}
fn decode(s: &str) -> Result<Vec<u8>, String> {
    if !s.is_ascii() || s.len() % 2 != 0 || s.len() > 65536 {
        return Err("bounded chunk".into());
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(|_| "hex".into()))
        .collect()
}
fn canonical(v: &Value) -> Result<Vec<u8>, String> {
    let b = serde_json::to_vec(v).map_err(|e| e.to_string())?;
    let v = parse_strict_json(&b, StrictJsonLimits::new(1 << 26, 1 << 26, 256, 1 << 20))
        .map_err(|e| e.to_string())?;
    canonical_json_bytes_bounded(&v, 1 << 26).map_err(|e| e.to_string())
}
fn legacy() -> Result<Value, String> {
    let raw = include_bytes!("../../../../../release/bundles/bundle-registry.json");
    let semantic = validate_semantic_profile_registry(
        include_bytes!("../../../../../release/bundles/semantic-profile-registry.json"),
        RegistryRevision::Revision4,
    )
    .map_err(|e| e.to_string())?;
    let registry =
        validate_successor_release_registry(raw, &semantic).map_err(|e| e.to_string())?;
    let mut rows = Vec::new();
    for t in &registry.registry().tuples {
        let r = registry
            .resolve(
                &semantic,
                SuccessorReleaseSelectionRequest {
                    semantic_context: &t.semantic_context,
                    frontend_bundle_id: &t.frontend_bundle_id,
                    toolchain_bundle_id: &t.toolchain_bundle_id,
                },
            )
            .map_err(|e| e.to_string())?;
        rows.push(json!({"semantic_context":t.semantic_context,"frontend_id":r.frontend.bundle_id,"toolchain_id":r.toolchain.bundle_id,"frontend_raw_sha256":sha(&canonical(&serde_json::to_value(r.frontend).map_err(|e|e.to_string())?)?),"toolchain_raw_sha256":sha(&canonical(&serde_json::to_value(r.toolchain).map_err(|e|e.to_string())?)?)}));
    }
    let original: Value = serde_json::from_slice(raw).map_err(|e| e.to_string())?;
    let mut controls = Vec::new();
    for name in ["reserved_registry_v2", "unknown_field", "wrong_hash"] {
        let mut v = original.clone();
        match name {
            "reserved_registry_v2" => v["schema"] = json!("mpk.release.bundle_registry.v2"),
            "unknown_field" => v["accepted"] = json!(true),
            _ => v["registry_sha256"] = json!("0".repeat(64)),
        };
        let mut b = canonical(&v)?;
        b.push(b'\n');
        let result = validate_successor_release_registry(&b, &semantic);
        controls.push(match result{Ok(_)=>json!({"id":name,"accepted":true}),Err(e)=>json!({"id":name,"accepted":false,"phase":format!("{:?}",e.phase()),"code":format!("{:?}",e.code())})});
    }
    Ok(
        json!({"accepted":true,"source_transport_sha256":sha(raw),"registry_sha256":registry.registry().registry_sha256,"frontends":registry.registry().frontend_bundles.len(),"toolchains":registry.registry().toolchain_bundles.len(),"tuples":rows,"controls":controls}),
    )
}
fn observe(q: &Value) -> Result<Value, String> {
    if q["mode"] == "legacy" {
        return legacy();
    }
    if q["mode"] != "hash" {
        return Err("observer mode".into());
    }
    let result = (|| -> Result<Value, String> {
        let mut raw = Vec::new();
        for c in q["chunks"].as_array().ok_or("chunks")? {
            let b = decode(c.as_str().ok_or("chunk")?)?;
            if raw.len() + b.len() > 1 << 26 {
                return Err("bytes".into());
            }
            raw.extend(b);
        }
        let strict = parse_strict_json(&raw, StrictJsonLimits::new(1 << 26, 1 << 26, 256, 1 << 20))
            .map_err(|e| e.to_string())?;
        let raw = canonical_json_bytes_bounded(&strict, 1 << 26).map_err(|e| e.to_string())?;
        let mut out =
            json!({"id":q["id"],"accepted":true,"canonical_sha256":sha(&raw),"bytes":raw.len()});
        if let Some(d) = q.get("domain") {
            let d = d.as_str().ok_or("domain")?;
            let (prefix, version) = d.rsplit_once('-').ok_or("domain")?;
            let (a, b) = version.split_once('.').ok_or("domain")?;
            if !prefix
                .as_bytes()
                .first()
                .is_some_and(u8::is_ascii_uppercase)
                || !prefix
                    .bytes()
                    .all(|x| x.is_ascii_uppercase() || x.is_ascii_digit() || x == b'-')
                || a.is_empty()
                || b.is_empty()
                || !a.bytes().chain(b.bytes()).all(|x| x.is_ascii_digit())
            {
                return Err("domain".into());
            }
            let mut h = Sha256::new();
            h.update(d.as_bytes());
            h.update([0]);
            h.update(&raw);
            out["digest"] = json!(hex(&h.finalize()));
        }
        Ok(out)
    })();
    Ok(result.unwrap_or_else(|_| json!({"id":q["id"],"accepted":false})))
}
fn main() {
    let mut raw = String::new();
    io::stdin().read_to_string(&mut raw).unwrap();
    let v: Value = serde_json::from_str(&raw).unwrap();
    let out: Vec<Value> = v
        .as_array()
        .unwrap()
        .iter()
        .map(|q| observe(q).unwrap_or_else(|e| json!({"setup_error":e})))
        .collect();
    println!("{}", serde_json::to_string(&out).unwrap());
}
