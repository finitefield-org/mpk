use mpk_vc::go_practical_type_keys::*;
use mpk_vc::{
    canonical_json_bytes, hash_canonical_json, parse_strict_json, HashDomain, StrictJsonLimits,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/go-practical/t02-w01/verification")
}
fn unhex(raw: &str) -> Vec<u8> {
    (0..raw.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&raw[i..i + 2], 16).unwrap())
        .collect()
}
fn packets() -> Vec<Value> {
    serde_json::from_slice::<Value>(
        &std::fs::read(root().join("source-inputs/corpus.json")).unwrap(),
    )
    .unwrap()["units"]
        .as_array()
        .unwrap()
        .clone()
}
fn original(name: &str) -> Value {
    serde_json::from_slice::<Value>(
        &std::fs::read(root().join("source-invocations/corpus.json")).unwrap(),
    )
    .unwrap()["units"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["name"] == name)
        .unwrap()
        .clone()
}
fn sources(p: &Value) -> Vec<GoTypeSourceBuffer> {
    p["input"]["sources"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| {
            GoTypeSourceBuffer::import(
                &serde_json::to_vec(&s["object"]).unwrap(),
                &unhex(s["raw_hex"].as_str().unwrap()),
            )
            .unwrap()
        })
        .collect()
}
fn entries(
    scopes: &GoSourceScopes,
    roots: &[GoSourceSelectionRef],
    cases: &[GoSourceSelectionRef],
) -> Vec<GoErrorEntryConstraint> {
    let mut entries = vec![];
    for root in roots {
        let signature =
            derive_go_source_signature(scopes, root.source_object_id(), root.node()).unwrap();
        let graph = signature.types();
        for (input, slot) in signature
            .slots()
            .iter()
            .filter(|s| s.role() != GoSourceSignatureRole::Result)
            .enumerate()
        {
            let mut work = vec![(slot.type_id().to_string(), vec![])];
            while let Some((id, path)) = work.pop() {
                match graph.types().find(|t| t.id() == id).unwrap() {
                    GoSourceTypeKey::Nominal { .. } => {
                        work.push((graph.underlying(&id).unwrap().to_string(), path))
                    }
                    GoSourceTypeKey::Builtin { name, .. } if name == "error" => entries.push(
                        GoErrorEntryConstraint::new(root.clone(), input, path, cases.to_vec()),
                    ),
                    GoSourceTypeKey::Array { element, .. }
                    | GoSourceTypeKey::Slice { element, .. } => {
                        let mut p = path;
                        p.push(GoErrorEntryStep::Elements);
                        work.push((element.clone(), p));
                    }
                    GoSourceTypeKey::Struct { fields, .. } => {
                        for (i, f) in fields.iter().enumerate() {
                            let mut p = path.clone();
                            p.push(GoErrorEntryStep::Field(i));
                            work.push((f.type_id.clone(), p));
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    entries
}
#[test]
fn source_inputs_owned_handoff_preserves_every_original_call_proof() {
    let mut checked = vec![];
    for p in packets() {
        let name = p["name"].as_str().unwrap();
        let prior = original(name);
        assert_eq!(
            p["input"], prior["input"],
            "complete original producer bytes"
        );
        let buffers = sources(&p);
        let raw = unhex(p["input"]["components_hex"].as_str().unwrap());
        let input = import_go_source_inputs(&raw, &buffers).unwrap();
        assert_eq!(input.components().canonical_bytes(), raw);
        let scopes = input.source_scopes().unwrap();
        let locate = |r: &Value, kind: &str| {
            let source = r["source"].as_str().unwrap();
            let node = scopes
                .source(source)
                .unwrap()
                .nodes()
                .iter()
                .position(|n| {
                    n.kind() == kind
                        && n.start() == r["start"].as_u64().unwrap() as usize
                        && n.end() == r["end"].as_u64().unwrap() as usize
                })
                .unwrap();
            GoSourceSelectionRef::new(source, node)
        };
        let roots = prior["roots"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| locate(r, "FuncDecl"))
            .collect::<Vec<_>>();
        let cases = prior["cases"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| locate(r, "TypeSpec"))
            .collect::<Vec<_>>();
        let unit = derive_go_source_error_selection(
            input.components(),
            &roots,
            &entries(&scopes, &roots, &cases),
        )
        .unwrap();
        assert_eq!(
            unit.domain().error_cases().len(),
            if name == "wide" { 70 } else { 0 }
        );
        assert_eq!(unit.selection().packages().len(), 3);
        let c =
            mpk_vc::go_practical_ordinary::generate_go_practical_source_invocation_catalogue(&unit)
                .unwrap();
        for (i, site) in c.sites().iter().enumerate() {
            let row = c.generate_row(site.caller(), site.node()).unwrap();
            let path = root().join(format!(
                "source-invocations/generation-linked/certificates/{name}-{i:03}.mpcert"
            ));
            assert_eq!(
                row.certificate_bytes(),
                std::fs::read(&path).unwrap(),
                "every whole prior proof byte {name}-{i:03}"
            );
            checked.push(json!({"unit":name,"site":i,"inputs":site.input_type_ids().len(),"results":site.result_type_ids().len(),"error_cases":unit.domain().error_cases().len(),"source_types":unit.domain().types().count(),"original_certificate":path,"bytes":row.certificate_bytes().len(),"raw_sha256":format!("{:x}",Sha256::digest(row.certificate_bytes())),"whole_original_proof_bytes_equal":true}));
        }
    }
    assert_eq!(checked.len(), 32);
    if let Some(out) = std::env::var_os("MPK_GO_SOURCE_INPUTS_OUTPUT") {
        let out = PathBuf::from(out);
        std::fs::create_dir_all(&out).unwrap();
        std::fs::write(out.join("original-proof-replay.json"),serde_json::to_vec_pretty(&json!({"status":"passed","complete_original_call_proofs":checked,"source_input_producer":"owned_original_packages","kernel_verification_repeated":false,"individual_wrapper_kernel_acceptance_claimed":false,"source_program_or_context_proof":false})).unwrap()).unwrap();
    }
}
fn canonical(v: &Value) -> Vec<u8> {
    canonical_json_bytes(
        &parse_strict_json(
            &serde_json::to_vec(v).unwrap(),
            StrictJsonLimits::new(268435456, 268435456, 64, 16777216),
        )
        .unwrap(),
    )
    .unwrap()
}
fn seal(v: &mut Value, domain: &'static str, field: &str) {
    v.as_object_mut().unwrap().remove(field);
    let bytes = canonical(v);
    let value = parse_strict_json(
        &bytes,
        StrictJsonLimits::new(268435456, 268435456, 64, 16777216),
    )
    .unwrap();
    v[field] = json!(hash_canonical_json(HashDomain::new(domain), &value)
        .unwrap()
        .to_hex());
}
fn reseal(v: &mut Value) {
    seal(v, "MPK-GO-TYPE-COMPONENTS-2.0", "content_sha256")
}
#[test]
fn source_inputs_reject_coherent_native_facts_and_incomplete_package_roots() {
    let p = packets().remove(0);
    let buffers = sources(&p);
    let original: Value =
        serde_json::from_slice(&unhex(p["input"]["components_hex"].as_str().unwrap())).unwrap();
    let mut extra = original.clone();
    let mut word = json!({"schema":"mpk.go.source_type.v2","kind":"builtin","name":"uint16","width":16,"signed":false});
    seal(&mut word, "MPK-GO-SOURCE-TYPE-2.0", "id");
    let id = word["id"].clone();
    extra["types"].as_array_mut().unwrap().push(word);
    extra["types"]
        .as_array_mut()
        .unwrap()
        .sort_by(|a, b| a["id"].as_str().cmp(&b["id"].as_str()));
    extra["type_roots"].as_array_mut().unwrap().push(id);
    extra["type_roots"]
        .as_array_mut()
        .unwrap()
        .sort_by(|a, b| a.as_str().cmp(&b.as_str()));
    reseal(&mut extra);
    assert!(
        import_go_type_key_components(&canonical(&extra), &buffers).is_ok(),
        "coherent extra facts are generic data"
    );
    assert_eq!(
        import_go_source_inputs(&canonical(&extra), &buffers),
        Err(GoTypeKeyError::Shape)
    );
    let mut subset = original;
    subset["package_roots"].as_array_mut().unwrap().pop();
    reseal(&mut subset);
    assert!(import_go_type_key_components(&canonical(&subset), &buffers).is_ok());
    assert_eq!(
        import_go_source_inputs(&canonical(&subset), &buffers),
        Err(GoTypeKeyError::Shape)
    );
}
#[test]
fn source_inputs_reject_missing_or_changed_original_bytes() {
    for p in packets() {
        let raw = unhex(p["input"]["components_hex"].as_str().unwrap());
        let mut buffers = sources(&p);
        let keep = buffers.pop().unwrap();
        assert!(import_go_source_inputs(&raw, &buffers).is_err());
        let mut changed = keep.bytes().to_vec();
        changed.push(b'\n');
        let o = keep.object();
        buffers.push(GoTypeSourceBuffer::new(&o.capability, &o.path, &o.kind, &changed).unwrap());
        assert!(import_go_source_inputs(&raw, &buffers).is_err());
    }
}

#[test]
fn source_inputs_keep_a_selected_original_replacement_module() {
    let p: Value =
        serde_json::from_slice(&std::fs::read(root().join("source-inputs/corpus.json")).unwrap())
            .unwrap();
    let p = &p["replacement"];
    let buffers = sources(p);
    let raw = unhex(p["input"]["components_hex"].as_str().unwrap());
    let input = import_go_source_inputs(&raw, &buffers).unwrap();
    let wire: Value = serde_json::from_slice(&raw).unwrap();
    assert_eq!(wire["packages"].as_array().unwrap().len(), 1);
    assert_ne!(
        wire["packages"][0]["module_source_object_id"],
        p["main_source_object_id"]
    );
    assert_eq!(
        wire["packages"][0]["module_source_object_id"],
        p["requested_module_source_object_id"]
    );
    assert!(
        buffers.iter().any(
            |b| b.object().path == "api/unused.go" && b.bytes().windows(3).any(|w| w == b"<-c")
        ),
        "unselected unsupported body must remain intact"
    );
    let scopes = input.source_scopes().unwrap();
    let r = &p["root"];
    let source = r["source"].as_str().unwrap();
    let node = scopes
        .source(source)
        .unwrap()
        .nodes()
        .iter()
        .position(|n| {
            n.kind() == "FuncDecl"
                && n.start() == r["start"].as_u64().unwrap() as usize
                && n.end() == r["end"].as_u64().unwrap() as usize
        })
        .unwrap();
    let selection = derive_go_source_selection(
        input.components(),
        &[GoSourceSelectionRef::new(source, node)],
    )
    .unwrap();
    assert_eq!(selection.roots().len(), 1);
    let signature = derive_go_source_signature(&scopes, source, node).unwrap();
    assert!(
        signature
            .types()
            .types()
            .any(|k| matches!(k,GoSourceTypeKey::Builtin{name,..}if name=="uint16")),
        "one logical seed is not a source type cap"
    );
}
