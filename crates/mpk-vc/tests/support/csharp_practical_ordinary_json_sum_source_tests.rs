//! Original-source closed sum instances, including a recursively nullable payload.
use super::*;

fn root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/json-sum-sources")
}
pub(super) fn pins() -> std::path::PathBuf {
    std::env::var_os("MPK_W09_JSON_SUMS_OUT")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| root().with_file_name("json-sums"))
}
fn source_id(name: &str) -> String {
    csharp_practical_declaration_id(&json!({"kind":"type","namespace":"Boundary","owner":"","name":name,"parameter_type_ids":[],"result_type_id":""})).unwrap()
}
fn requests() -> Value {
    let code = "namespace Boundary;public enum Tag2{A=0,B=1}public enum Tag3{A=0,B=1,C=2}public readonly struct OptionRep{public readonly Tag2 Tag;public readonly int Value;}public readonly struct LookupRep{public readonly Tag2 Tag;public readonly int Value;}public readonly struct ResultRep{public readonly Tag2 Tag;public readonly int Value;public readonly bool Error;}public readonly struct ValidationRep{public readonly Tag2 Tag;public readonly int Value;public readonly int[] Errors;}public readonly struct FieldRep{public readonly Tag3 Tag;public readonly int Value;}public readonly struct NestedRep{public readonly Tag2 Tag;public readonly int? Value;public readonly bool Error;}public readonly struct Payload{public readonly OptionRep Option;public readonly LookupRep Lookup;public readonly ResultRep Result;public readonly ValidationRep Validation;public readonly FieldRep Field;public readonly NestedRep Nested;}public static class Entry{public static Payload Run(Payload p){return p;}}\n";
    let mut requests = super::super::source_tests::requests_for(&[("all-sums", code)]);
    let bundle = b();
    let (context, captures) = support::replay_context(&bundle, &requests[0]);
    let source_hash = captures
        .entries()
        .iter()
        .find(|e| e.kind() == OriginalInputKind::Source)
        .unwrap()
        .raw_sha256();
    let mut bindings = vec![];
    let optional = instance("option", vec![primitive("i32")]);
    let optional_id = csharp_practical_closed_instance_id(&bundle, &optional).unwrap();
    for (name, role, arms, value_ty, argument, error) in [
        (
            "OptionRep",
            "option",
            vec!["none", "some"],
            primitive("i32"),
            ty("i32"),
            None,
        ),
        (
            "LookupRep",
            "lookup",
            vec!["missing_key", "found"],
            primitive("i32"),
            ty("i32"),
            None,
        ),
        (
            "ResultRep",
            "result",
            vec!["ok", "error"],
            primitive("i32"),
            ty("i32"),
            Some(("error", "Error", primitive("bool"), ty("bool"))),
        ),
        (
            "ValidationRep",
            "validation",
            vec!["valid", "invalid"],
            primitive("i32"),
            ty("i32"),
            Some((
                "errors",
                "Errors",
                instance("bounded_sequence", vec![primitive("i32")]),
                ty("i32"),
            )),
        ),
        (
            "FieldRep",
            "boundary_field",
            vec!["missing", "null", "value"],
            primitive("i32"),
            ty("i32"),
            None,
        ),
        (
            "NestedRep",
            "result",
            vec!["ok", "error"],
            optional,
            optional_id,
            Some(("error", "Error", primitive("bool"), ty("bool"))),
        ),
    ] {
        let id = source_id(name);
        let tag = source_id(if arms.len() == 3 { "Tag3" } else { "Tag2" });
        let mut members = vec![
            ("tag", "Tag", json!({"kind":"source","id":tag})),
            ("value", "Value", value_ty),
        ];
        let mut arguments = vec![argument];
        if let Some((role, name, member_ty, argument)) = error {
            members.push((role, name, member_ty));
            arguments.push(argument);
        }
        bindings.push(SemanticBindingInput {
            source_type_id: id.clone(),
            source_content_sha256: source_hash.into(),
            role: role.into(),
            member_map: members
                .into_iter()
                .map(|(role, name, ty)| SemanticBindingMember {
                    role: role.into(),
                    member_id: csharp_practical_stored_member_id(&id, name, &ty, "readonly_field")
                        .unwrap(),
                })
                .collect(),
            tag_arms: arms
                .iter()
                .enumerate()
                .map(|(i, arm)| SemanticArmMapping {
                    source_tag: i.to_string(),
                    semantic_arm: (*arm).into(),
                })
                .collect(),
            inferred_argument_ids: arguments,
            default_arm: match role {
                "option" => "none",
                "lookup" => "missing_key",
                _ => "ineligible",
            }
            .into(),
            bounds: if role == "validation" {
                vec![SemanticBound {
                    id: "errors".into(),
                    maximum: 256,
                }]
            } else {
                vec![]
            },
            operation_map: vec![],
            enum_arms: BTreeMap::new(),
        });
    }
    let sidecar = build_semantic_bindings(&context, &captures, bindings).unwrap();
    let inputs = requests[0]["inputs"].as_array_mut().unwrap();
    inputs.push(json!({"kind":"sidecar","path":"contracts/data.json","utf8":std::str::from_utf8(sidecar.canonical_bytes()).unwrap()}));
    inputs.sort_by_key(|i| i["path"].as_str().unwrap().to_owned());
    requests
}
#[test]
fn csharp_03_t06_w09_json_sum_requests() {
    let bytes = serde_json::to_vec_pretty(&requests()).unwrap();
    if let Some(path) = std::env::var_os("MPK_W09_JSON_SUM_REQUESTS_OUT") {
        fs::write(path, bytes).unwrap();
    } else {
        assert_eq!(fs::read(root().join("requests.json")).unwrap(), bytes);
    }
}
#[test]
fn csharp_03_t06_w09_json_sum_source() {
    let requests: Value =
        serde_json::from_slice(&fs::read(root().join("requests.json")).unwrap()).unwrap();
    let responses: Value =
        serde_json::from_slice(&fs::read(root().join("responses.json")).unwrap()).unwrap();
    assert_eq!(requests.as_array().unwrap().len(), 1);
    assert_eq!(responses.as_array().unwrap().len(), 1);
    assert_eq!(requests[0]["id"], responses[0]["id"]);
    assert!(responses[0].get("reject").is_none(), "{}", responses[0]);
    let bundle = b();
    let (context, captures) = support::replay_context(&bundle, &requests[0]);
    let source = ValidatedDataSource::import_captured_facts(
        &bundle,
        &context,
        &captures,
        &serde_json::to_vec(&responses[0]["facts"]).unwrap(),
    )
    .unwrap();
    let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
    let vir = emitted.vir();
    let p = generate_csharp_practical_ordinary_json_products(vir).unwrap();
    assert_eq!(p.sums().len(), 6);
    assert_eq!(p.products().len(), 7);
    assert_eq!(p.sequences().len(), 1);
    let roles = p
        .sums()
        .iter()
        .map(|s| s.template_id.as_str())
        .collect::<BTreeSet<_>>();
    assert_eq!(roles.len(), 5);
    let layouts = generate_csharp_practical_ordinary_carriers(vir).unwrap();
    for sum in p.sums() {
        assert_eq!(
            &sum.carrier,
            layouts
                .carriers()
                .iter()
                .find(|c| c.type_id == sum.carrier.type_id)
                .unwrap()
        );
        assert!(!p.deferred_type_ids().contains(&sum.carrier.type_id));
        let entry = emitted
            .closure()
            .closed()
            .entries()
            .iter()
            .find(|e| e["instance_id"] == sum.carrier.type_id)
            .unwrap();
        assert_eq!(entry["template_id"], sum.template_id);
    }
    let validation = p
        .sums()
        .iter()
        .position(|s| s.template_id == "mpk.csharp.semantic.validation.v1")
        .unwrap();
    assert_eq!(p.sums()[validation].arms[1].payload_maximum, Some(256));
    let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
    validate_csharp_practical_certificate_structure(&cert).unwrap();
    assert_eq!(
        import_csharp_practical_ordinary_json_products(
            &p.canonical_bytes(),
            p.certificate_bytes(),
            vir
        )
        .unwrap(),
        p
    );
    for (pointer, forged) in [
        (
            format!("/sums/{validation}/arms/1/payload_maximum"),
            json!(4096),
        ),
        (
            format!("/sums/{validation}/arms/1/payload_type_id"),
            json!("forged"),
        ),
        (format!("/sums/{validation}/arms/1/tag"), json!(0)),
        (
            format!("/sums/{validation}/arms/0/tag_literal/utf8"),
            json!([34, 110, 111, 110, 101, 34]),
        ),
    ] {
        let mut bad: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
        *bad.pointer_mut(&pointer).unwrap() = forged;
        assert!(import_csharp_practical_ordinary_json_products(
            &serde_json::to_vec(&bad).unwrap(),
            p.certificate_bytes(),
            vir
        )
        .is_err());
    }
    let values = generate_csharp_practical_ordinary_json_values(vir).unwrap();
    let syntax = generate_csharp_practical_ordinary_json_syntax(vir).unwrap();
    assert_eq!(p.primitives(), values.definitions());
    for bytes in [values.certificate_bytes(), syntax.certificate_bytes()] {
        let old = mpk_cert::decode_canonical_certificate(bytes).unwrap();
        let names = old
            .declarations
            .iter()
            .map(|d| old.name_table[d.name as usize].clone())
            .collect();
        super::super::super::super::super::structural_equivalence_tests::same_definition_closure(
            &old, &cert, &names,
        )
        .unwrap();
    }
    let row = json!({"id":"all-sums","program":serde_json::from_slice::<Value>(&p.canonical_bytes()).unwrap(),"terms":cert.term_table.len(),"declarations":cert.declarations.len()});
    let bytes = serde_json::to_vec_pretty(&row).unwrap();
    let hex = p
        .certificate_bytes()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>()
        + "\n";
    let out = pins();
    if std::env::var_os("MPK_W09_JSON_SUMS_OUT").is_some() {
        fs::create_dir_all(&out).unwrap();
        fs::write(out.join("certificate.json"), bytes).unwrap();
        fs::write(out.join("all-sums.hex"), hex).unwrap();
    } else {
        assert_eq!(fs::read(out.join("certificate.json")).unwrap(), bytes);
        assert_eq!(fs::read_to_string(out.join("all-sums.hex")).unwrap(), hex);
    }
    eprintln!("JSON sums: six sums,five templates,one sequence,seven source products;{} terms,{} declarations",cert.term_table.len(),cert.declarations.len());
}
