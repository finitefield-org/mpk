//! Unchanged public retained frontend process validator and canonical hash observer.
use mpk_cli::frontend_protocol::FrontendProcessFacts;
use mpk_cli::successor_frontend_protocol::{
    validate_successor_frontend_process, SuccessorFrontendProtocolRequest,
    SUCCESSOR_FRONTEND_SCHEMA, SUCCESSOR_FRONTEND_STDERR_BYTES_MAX,
    SUCCESSOR_FRONTEND_STDOUT_BYTES_MAX,
};
use mpk_vc::semantic_profile_registry::{
    canonical_registry_transport, validate_registry_selection_envelope,
    validate_registry_semantic_context, validate_semantic_profile_registry, RegistryRevision,
    ValidatedSemanticProfileRegistry,
};
use mpk_vc::successor_source_artifacts::{
    successor_contract_hash_value, successor_source_manifest_hash_value,
    successor_source_map_hash_value, successor_vir_hash_value,
};
use mpk_vc::{
    canonical_json_bytes_bounded, parse_strict_json, CapturedInput, InputKind,
    ReleaseRegistryIdentity, StrictJsonLimits, SyntheticPermission,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    io::{self, Read},
};

fn canonical(v: &Value) -> Result<Vec<u8>, String> {
    let raw = serde_json::to_vec(v).map_err(|e| e.to_string())?;
    let strict = parse_strict_json(&raw, StrictJsonLimits::new(1 << 28, 1 << 28, 768, 1 << 20))
        .map_err(|e| e.to_string())?;
    canonical_json_bytes_bounded(&strict, 1 << 28).map_err(|e| e.to_string())
}
fn hex(raw: &[u8]) -> String {
    raw.iter().map(|b| format!("{b:02x}")).collect()
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
fn projection(q: &Value) -> Result<Value, String> {
    let mut units = q["units"].clone();
    let mut hashes = BTreeMap::new();
    for u in units.as_array_mut().ok_or("units")? {
        for f in u["functions"].as_array_mut().ok_or("functions")? {
            f["contracts"]["semantic_context"] = q["context"].clone();
            f["contracts"]["contract_hash"] = json!("0".repeat(64));
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
    let mut vir = json!({"schema":"mpk.vir.v1","semantic_context":q["context"],"units":units,"vir_hash":"0".repeat(64)});
    vir["vir_hash"] = json!(successor_vir_hash_value(&vir)
        .map_err(|e| e.to_string())?
        .as_str());
    let mut map = json!({"schema":"mpk.source_map.v1","semantic_context":q["context"],"source_ir_schema":"mpk.vir.v1","source_ir_hash":vir["vir_hash"],"entries":q["entries"],"source_map_hash":"0".repeat(64)});
    map["source_map_hash"] = json!(successor_source_map_hash_value(&map)
        .map_err(|e| e.to_string())?
        .as_str());
    let mut manifest = q["manifest"].clone();
    manifest["schema"] = json!("mpk.source_manifest.v1");
    manifest["semantic_context"] = q["context"].clone();
    manifest["vir_hash"] = vir["vir_hash"].clone();
    manifest["source_map_hash"] = map["source_map_hash"].clone();
    manifest["source_manifest_hash"] = json!("0".repeat(64));
    manifest["source_manifest_hash"] = json!(successor_source_manifest_hash_value(&manifest)
        .map_err(|e| e.to_string())?
        .as_str());
    Ok(
        json!({"schema":SUCCESSOR_FRONTEND_SCHEMA,"status":"ir-lowered","phase":"emission","semantic_context":q["context"],"selection":q["selection"],"ir":{"schema":"mpk.vir.v1","sha256":vir["vir_hash"],"value":vir},"source_map":map,"source_manifest":manifest,"rejected_features":[],"diagnostics":[]}),
    )
}
fn process(q: &Value, registry: &ValidatedSemanticProfileRegistry) -> Result<Value, String> {
    if q["mode"] == "policy" {
        let context = validate_registry_semantic_context(registry, &q["context"]);
        let selection_ok = context.as_ref().is_ok_and(|c| {
            q.get("selection")
                .is_none_or(|s| validate_registry_selection_envelope(registry, c, s).is_ok())
        });
        return Ok(json!({"context_accepted":context.is_ok(),"selection_accepted":selection_ok}));
    }
    let context =
        validate_registry_semantic_context(registry, &q["context"]).map_err(|e| e.to_string())?;
    let selection = validate_registry_selection_envelope(registry, &context, &q["selection"])
        .map_err(|e| e.to_string())?;
    let release: ReleaseRegistryIdentity =
        serde_json::from_value(q["expected_release_registry"].clone())
            .map_err(|e| e.to_string())?;
    let buffers = q["captured_inputs"]
        .as_array()
        .ok_or("captured")?
        .iter()
        .map(|x| unhex(x["raw_hex"].as_str().ok_or("raw")?))
        .collect::<Result<Vec<_>, String>>()?;
    let captured = q["captured_inputs"]
        .as_array()
        .unwrap()
        .iter()
        .zip(&buffers)
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
        .map(|r| {
            Ok(SyntheticPermission {
                reference: serde_json::from_value(r["reference"].clone())
                    .map_err(|e| e.to_string())?,
                reason: r["reason"].as_str().ok_or("reason")?.to_owned(),
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let stdout = if q["mode"] == "project" {
        let mut b = canonical(&projection(q)?)?;
        b.push(b'\n');
        b
    } else if let Some(n) = q.get("padding_bytes") {
        let n = n.as_u64().ok_or("padding")? as usize;
        if n > SUCCESSOR_FRONTEND_STDOUT_BYTES_MAX + 1 {
            return Err("padding bound".into());
        }
        vec![0; n]
    } else {
        unhex(q["stdout_hex"].as_str().ok_or("stdout")?)?
    };
    let exit = if q["mode"] == "project" {
        Some(0)
    } else {
        q["exit_code"].as_i64().map(|x| x as i32)
    };
    let result = validate_successor_frontend_process(
        SuccessorFrontendProtocolRequest {
            registry,
            semantic_context: &context,
            selection: &selection,
            release_registry: &release,
            captured_inputs: &captured,
            synthetic_permissions: &permissions,
        },
        FrontendProcessFacts {
            exit_code: exit,
            signaled: q["signaled"].as_bool().unwrap_or(false),
            stdout: &stdout,
            stderr_observed_bytes: q["stderr_observed_bytes"].as_u64().unwrap_or(0) as usize,
        },
    );
    match result {
        Ok(v) => Ok(
            json!({"accepted":true,"status":v.status(),"phase":v.phase(),"canonical_hex":hex(v.canonical_bytes()),"value":serde_json::from_slice::<Value>(v.canonical_bytes()).map_err(|e|e.to_string())?}),
        ),
        Err(e) => Ok(json!({"accepted":false,"error_code":e.code().as_str()})),
    }
}
fn hash_observation(q: &Value) -> Value {
    let result = (|| -> Result<Value, String> {
        let domain = q["domain"].as_str().ok_or("domain")?;
        let (prefix, version) = domain.rsplit_once('-').ok_or("domain grammar")?;
        let (major, minor) = version.split_once('.').ok_or("domain version")?;
        if !prefix.starts_with(|c: char| c.is_ascii_uppercase())
            || !prefix
                .bytes()
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == b'-')
            || major.is_empty()
            || minor.is_empty()
            || !major
                .bytes()
                .chain(minor.bytes())
                .all(|c| c.is_ascii_digit())
        {
            return Err("domain grammar".into());
        }
        let raw = unhex(q["raw_json_hex"].as_str().ok_or("raw")?)?;
        let strict = parse_strict_json(&raw, StrictJsonLimits::new(1 << 28, 1 << 28, 768, 1 << 20))
            .map_err(|e| e.to_string())?;
        let b = canonical_json_bytes_bounded(&strict, 1 << 28).map_err(|e| e.to_string())?;
        let mut h = Sha256::new();
        h.update(domain.as_bytes());
        h.update([0]);
        h.update(&b);
        Ok(
            json!({"id":q["id"],"accepted":true,"canonical_hex":hex(&b),"digest":hex(&h.finalize())}),
        )
    })();
    result.unwrap_or_else(|_| json!({"id":q["id"],"accepted":false}))
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
    let out=input["queries"].as_array().unwrap().iter().map(|q|{
        if q.get("raw_json_hex").is_some(){return hash_observation(q)}
        if q["mode"]=="limits"{return json!({"stdout_bytes":SUCCESSOR_FRONTEND_STDOUT_BYTES_MAX,"stderr_bytes":SUCCESSOR_FRONTEND_STDERR_BYTES_MAX})}
        process(q,&registry).unwrap_or_else(|e|json!({"accepted":false,"setup_error":e}))
    }).collect::<Vec<_>>();
    println!("{}", serde_json::to_string(&out).unwrap());
}
