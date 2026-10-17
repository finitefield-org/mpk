//! Public retained policy/API observation. No profile registration or release.
use mpk_api::successor_api::{
    SuccessorApiService, SuccessorFrontendArtifactStore, SuccessorFrontendArtifacts,
};
use mpk_cert::hash_hex;
use mpk_cli::frontend_protocol::FrontendProcessFacts;
use mpk_cli::program_certificate::ProgramCertificateOutcome;
use mpk_cli::successor_frontend_protocol::{
    validate_successor_frontend_process, SuccessorFrontendProtocolRequest,
};
use mpk_cli::successor_policy::{
    generate_successor_policy_scan, import_successor_policy_scan_json, run_successor_policy,
    PolicyVerificationOptions, SuccessorPolicyScanSource, SuccessorPolicySource,
};
use mpk_vc::semantic_profile_registry::{
    canonical_registry_transport, validate_registry_selection_envelope,
    validate_registry_semantic_context, validate_semantic_profile_registry, RegistryRevision,
    ValidatedSemanticProfileRegistry,
};
use mpk_vc::successor_vc::{emit_successor_vc_skeleton, generate_successor_vc, SuccessorVcSource};
use mpk_vc::{
    canonical_json_bytes_bounded, parse_strict_json, CapturedInput, InputKind,
    ReleaseRegistryIdentity, StrictJsonLimits, SyntheticPermission,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::io::{self, Read};

fn canonical(v: &Value) -> Result<Vec<u8>, String> {
    let raw = serde_json::to_vec(v).map_err(|e| e.to_string())?;
    let strict = parse_strict_json(&raw, StrictJsonLimits::new(1 << 28, 1 << 28, 768, 1 << 20))
        .map_err(|e| e.to_string())?;
    canonical_json_bytes_bounded(&strict, 1 << 28).map_err(|e| e.to_string())
}
fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}
fn chunks(b: &[u8]) -> Vec<String> {
    b.chunks(32768).map(hex).collect()
}
fn unhex(s: &str) -> Result<Vec<u8>, String> {
    if !s.is_ascii() || s.len() % 2 != 0 {
        return Err("hex shape".into());
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(|_| "hex digit".into()))
        .collect()
}
fn decoded(raw: &[u8]) -> Result<Value, String> {
    serde_json::from_slice(raw).map_err(|e| e.to_string())
}
fn observe(q: &Value, registry: &ValidatedSemanticProfileRegistry) -> Result<Value, String> {
    if q["mode"] == "hash" {
        let attempt = (|| -> Result<Value, String> {
            let raw = unhex(q["raw_json_hex"].as_str().ok_or("hash input")?)?;
            let strict =
                parse_strict_json(&raw, StrictJsonLimits::new(1 << 28, 1 << 28, 768, 1 << 20))
                    .map_err(|e| e.to_string())?;
            let raw = canonical_json_bytes_bounded(&strict, 1 << 28).map_err(|e| e.to_string())?;
            let mut v = json!({"id":q["id"],"accepted":true,"canonical_hex":hex(&raw),"canonical_sha256":mpk_vc::sha256_raw_file_bytes(&raw).to_hex()});
            if let Some(d) = q.get("domain") {
                let d = d.as_str().ok_or("domain")?;
                let (prefix, version) = d.rsplit_once('-').ok_or("domain")?;
                let (major, minor) = version.split_once('.').ok_or("version")?;
                if !prefix
                    .as_bytes()
                    .first()
                    .is_some_and(u8::is_ascii_uppercase)
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
                let mut hash = Sha256::new();
                hash.update(d.as_bytes());
                hash.update([0]);
                hash.update(&raw);
                v["digest"] = json!(hex(&hash.finalize()));
            }
            Ok(v)
        })();
        return Ok(attempt.unwrap_or_else(|_| json!({"id":q["id"],"accepted":false})));
    }
    if q["mode"] == "certificate" {
        let mut raw = Vec::new();
        for s in q["byte_chunks"].as_array().ok_or("certificate chunks")? {
            raw.extend(unhex(s.as_str().ok_or("certificate chunk")?)?)
        }
        if let Some(manifest) = q.get("source_manifest") {
            let mut certificate = mpk_cert::decode_canonical_certificate(&raw)
                .map_err(|e| format!("certificate decode:{e:?}"))?;
            certificate.source_manifest = Some(mpk_cert::encode::SourceManifest {
                payload: canonical(manifest)?,
            });
            // Certificate v0 stores a zero certificate-hash field; the checkers
            // hash the complete resulting bytes,including the source payload.
            raw = mpk_cert::encode::encode_certificate(&certificate);
        }
        let rust = match mpk_kernel::verify_certificate_bytes(&raw) {
            Ok(v) => {
                json!({"accepted":true,"certificate_hash":hash_hex(&v.certificate_hash),"export_hash":hash_hex(&v.export_hash),"axiom_report_hash":hash_hex(&v.axiom_report_hash),"axiom_count":v.axiom_count,"declarations":v.declaration_count})
            }
            Err(e) => json!({"accepted":false,"kind":format!("{:?}",e.kind()),"detail":e.detail()}),
        };
        let reference = match mpk_cli::reference_checker::execute_reference_checker(&raw) {
            Ok(v) => {
                json!({"exit_code":v.status_code(),"stdout":decoded(v.stdout())?,"stderr_hex":hex(v.stderr())})
            }
            Err(e) => json!({"execution_error":e.to_string()}),
        };
        let source = mpk_cert::decode_canonical_certificate(&raw)
            .ok()
            .and_then(|c| c.source_manifest)
            .map(|m| hex(&m.payload));
        return Ok(
            json!({"byte_chunks":chunks(&raw),"embedded_source_manifest_hex":source,"rust":rust,"reference":reference}),
        );
    }
    let context = validate_registry_semantic_context(registry, &q["frontend"]["semantic_context"])
        .map_err(|e| e.to_string())?;
    let selection =
        validate_registry_selection_envelope(registry, &context, &q["frontend"]["selection"])
            .map_err(|e| e.to_string())?;
    let release: ReleaseRegistryIdentity =
        serde_json::from_value(q["expected_release_registry"].clone())
            .map_err(|e| e.to_string())?;
    let bytes = q["captured_inputs"]
        .as_array()
        .ok_or("captured")?
        .iter()
        .map(|x| unhex(x["raw_hex"].as_str().ok_or("raw")?))
        .collect::<Result<Vec<_>, _>>()?;
    let captured = q["captured_inputs"]
        .as_array()
        .unwrap()
        .iter()
        .zip(&bytes)
        .map(|(x, b)| {
            Ok(CapturedInput {
                kind: serde_json::from_value::<InputKind>(x["kind"].clone())
                    .map_err(|e| e.to_string())?,
                normalized_path: x["normalized_path"].as_str().ok_or("path")?,
                bytes: b,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let permissions = q["synthetic_permissions"]
        .as_array()
        .ok_or("permissions")?
        .iter()
        .map(|x| {
            Ok(SyntheticPermission {
                reference: serde_json::from_value(x["reference"].clone())
                    .map_err(|e| e.to_string())?,
                reason: x["reason"].as_str().ok_or("reason")?.to_owned(),
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let mut stdout = canonical(&q["frontend"])?;
    stdout.push(b'\n');
    let frontend = validate_successor_frontend_process(
        SuccessorFrontendProtocolRequest {
            registry,
            semantic_context: &context,
            selection: &selection,
            release_registry: &release,
            captured_inputs: &captured,
            synthetic_permissions: &permissions,
        },
        FrontendProcessFacts {
            exit_code: Some(0),
            signaled: false,
            stdout: &stdout,
            stderr_observed_bytes: 0,
        },
    )
    .map_err(|e| format!("frontend:{}", e.code().as_str()))?;
    let a = frontend.artifacts().ok_or("frontend artifacts")?;
    if q["mode"] == "api" {
        let store = SuccessorFrontendArtifactStore::from_frontend_successes(
            registry,
            [SuccessorFrontendArtifacts {
                vir: a.vir().clone(),
                source_map: a.source_map().clone(),
                source_manifest: a.source_manifest().clone(),
                vc_profile_contract: q["vc_contract"].clone(),
            }],
        )
        .map_err(|e| e.to_string())?;
        let mut service =
            SuccessorApiService::new(registry.clone(), store).map_err(|e| e.to_string())?;
        let mut session = Value::Null;
        let mut target = Value::Null;
        let mut rows = Vec::new();
        for op in q["requests"].as_array().ok_or("requests")? {
            let mut request = op["value"].clone();
            if request["session_id"] == "@session" {
                request["session_id"] = session.clone()
            }
            if request["target_id"] == "@target" {
                request["target_id"] = target.clone()
            }
            let raw = if let Some(s) = op.get("raw_hex") {
                unhex(s.as_str().ok_or("request raw")?)?
            } else {
                let mut b = canonical(&request)?;
                b.push(b'\n');
                b
            };
            let before = service.mutation_count();
            let name = op["operation"].as_str().ok_or("operation")?;
            let result = match name {
                "session/start" => service.handle_start_session(&raw),
                "vir/import" => service.handle_vir_import(&raw),
                "vc/generate" => service.handle_vc_generate(&raw),
                "vc/list" => service.handle_vc_list(&raw),
                "vc/start-proof" => service.handle_vc_start_proof(&raw),
                "vc/attach-candidate" => service.handle_vc_attach_candidate(&raw),
                "vc/check-candidate" => service.handle_vc_check_candidate(&raw),
                _ => return Err("unknown harness operation".into()),
            };
            let mut row = match result {
                Ok(b) => {
                    let v = decoded(&b)?;
                    if name == "session/start" {
                        session = v["session_id"].clone()
                    }
                    if name == "vc/start-proof" {
                        target = v["target_id"].clone()
                    }
                    json!({"accepted":true,"canonical_hex":hex(&b),"value":v})
                }
                Err(e) => {
                    json!({"accepted":false,"error_code":e.code.as_str(),"phase":e.phase().as_str()})
                }
            };
            row["mutation_before"] = json!(before);
            row["mutation_after"] = json!(service.mutation_count());
            row["request_hex"] = json!(hex(&raw));
            row["operation"] = json!(name);
            if let Some(id) = session.as_str() {
                let sid = mpk_api::SessionId(id.to_owned());
                row["source_state"] = json!(service.source_state(&sid).map(|s| s.label()));
            }
            rows.push(row);
        }
        return Ok(json!({"accepted":true,"operations":rows,"mutations":service.mutation_count()}));
    }
    let scan_source = SuccessorPolicyScanSource {
        registry,
        vir: a.vir(),
        source_map: a.source_map(),
        frontend_manifest: a.source_manifest(),
        policy_contract: &q["policy_contract"],
        captured_inputs: &captured,
    };
    let scan = generate_successor_policy_scan(scan_source)
        .map_err(|e| format!("scan:{}:{}", e.phase().as_str(), e.code().as_str()))?;
    if q["mode"] == "scan" {
        if let Some(value) = q.get("candidate") {
            let raw = if let Some(h) = q.get("candidate_hex") {
                unhex(h.as_str().ok_or("scan hex")?)?
            } else {
                canonical(value)?
            };
            return Ok(match import_successor_policy_scan_json(&raw, scan_source) {
                Ok(v) => json!({"accepted":true,"scan":decoded(v.canonical_bytes())?}),
                Err(e) => {
                    json!({"accepted":false,"error_code":e.code().as_str(),"phase":e.phase().as_str()})
                }
            });
        }
        return Ok(json!({"accepted":true,"scan":decoded(scan.canonical_bytes())?}));
    }
    let source = SuccessorVcSource {
        registry,
        vir: a.vir(),
        manifest: a.source_manifest(),
        profile_contract: &q["vc_contract"],
    };
    let vc = generate_successor_vc(source).map_err(|e| e.to_string())?;
    let skeleton = emit_successor_vc_skeleton(&vc, source).map_err(|e| e.to_string())?;
    let policy_source = SuccessorPolicySource {
        registry,
        vir: a.vir(),
        source_map: a.source_map(),
        frontend_manifest: a.source_manifest(),
        vc: &vc,
        skeleton: &skeleton,
        policy_contract: &q["policy_contract"],
        evidence_contract: &q["evidence_contract"],
        captured_inputs: &captured,
    };
    let run = match run_successor_policy(
        policy_source,
        PolicyVerificationOptions {
            strict: true,
            update_fixtures: false,
        },
    ) {
        Ok(run) => run,
        Err(e) => {
            return Ok(
                json!({"accepted":false,"error_code":e.code().as_str(),"phase":e.phase().as_str(),"detail":e.to_string()}),
            )
        }
    };
    if let Some(v) = q.get("candidate") {
        return Ok(match run.import_evidence_json(&canonical(v)?) {
            Ok(v) => json!({"accepted":true,"evidence":decoded(v.canonical_bytes())?}),
            Err(e) => {
                json!({"accepted":false,"error_code":e.code().as_str(),"phase":e.phase().as_str()})
            }
        });
    }
    let outcome = match run.program_certificate() {
        ProgramCertificateOutcome::Pending {
            generated_declarations,
            missing_member_ids,
        } => {
            json!({"kind":"pending","declarations":generated_declarations.len(),"missing_member_ids":missing_member_ids})
        }
        ProgramCertificateOutcome::Candidate(v) => {
            json!({"kind":"checked","byte_chunks":chunks(&v.bytes),"rust":{"certificate_hash":hash_hex(&v.rust_report.certificate_hash),"export_hash":hash_hex(&v.rust_report.export_hash),"axiom_report_hash":hash_hex(&v.rust_report.axiom_report_hash),"axiom_count":v.rust_report.axiom_count},"reference":{"certificate_hash":v.reference_report.certificate_hash,"export_hash":v.reference_report.export_hash,"axiom_report_hash":v.reference_report.axiom_report_hash,"axiom_count":v.reference_report.axiom_count}})
        }
        ProgramCertificateOutcome::Unaccepted(v) => {
            json!({"kind":"unaccepted","byte_chunks":chunks(&v.bytes),"rust_verdict":v.rust_verdict.as_str(),"reference_verdict":v.reference_verdict.as_str(),"detail":v.failure_detail})
        }
    };
    let mut imports = Vec::new();
    if let Some(tests) = q.get("imports") {
        for t in tests.as_array().ok_or("policy import tests")? {
            let raw = if let Some(h) = t.get("raw_hex") {
                unhex(h.as_str().ok_or("policy import hex")?)?
            } else {
                canonical(&t["value"])?
            };
            let accepted = match t["stage"].as_str().ok_or("policy import stage")? {
                "scan" => run.import_scan_json(&raw).map(|_| ()),
                "evidence" => run.import_evidence_json(&raw).map(|_| ()),
                _ => return Err("policy import stage".into()),
            };
            imports.push(match accepted{Ok(())=>json!({"id":t["id"],"accepted":true}),Err(e)=>json!({"id":t["id"],"accepted":false,"error_code":e.code().as_str(),"phase":e.phase().as_str()})});
        }
    }
    Ok(
        json!({"accepted":true,"scan":decoded(run.scan().canonical_bytes())?,"evidence":decoded(run.evidence().canonical_bytes())?,"outcome":outcome,"imports":imports,
        "vc":serde_json::to_value(vc.document()).map_err(|e|e.to_string())?,"skeleton":serde_json::to_value(skeleton.skeleton()).map_err(|e|e.to_string())?,"certificate_manifest":decoded(run.certificate_manifest().canonical_bytes())?}),
    )
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
    let rows = input["queries"]
        .as_array()
        .unwrap()
        .iter()
        .map(|q| {
            observe(q, &registry).unwrap_or_else(|e| json!({"accepted":false,"setup_error":e}))
        })
        .collect::<Vec<_>>();
    println!("{}", serde_json::to_string(&rows).unwrap());
}
