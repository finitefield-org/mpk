use super::*;
use core_eval::{bit as observed_bit, run, sparse_cube, V};
use sequence_tests::{number, observe};

#[test]
fn csharp_03_t06_w09_entry_source_request() {
    let bundle = b();
    let id = |name: &str| {
        csharp_practical_declaration_id(&json!({"kind":"type","namespace":"EntryCases","owner":"","name":name,"parameter_type_ids":[],"result_type_id":""})).unwrap()
    };
    let rep = id("Rep");
    let root=csharp_practical_declaration_id(&json!({"kind":"method","namespace":"EntryCases","owner":id("Entry"),"name":"Run","parameter_type_ids":[rep.clone()],"result_type_id":rep})).unwrap();
    let code="namespace EntryCases;public readonly struct Rep{public readonly bool Key;public readonly float Value;}public static class Entry{public static Rep Run(Rep value){return value;}}\n";
    let (plain, captures) = support::context(&bundle, &root, code.as_bytes());
    let member = |role: &str, name: &str, kind: &str| SemanticBindingMember {
        role: role.into(),
        member_id: csharp_practical_stored_member_id(
            &rep,
            name,
            &primitive(kind),
            "readonly_field",
        )
        .unwrap(),
    };
    let binding = SemanticBindingInput {
        source_type_id: rep.clone(),
        source_content_sha256: captures.entries()[0].raw_sha256().into(),
        role: "ordered_entry".into(),
        member_map: vec![
            member("key", "Key", "bool"),
            member("value", "Value", "f32"),
        ],
        inferred_argument_ids: vec![ty("bool"), ty("f32")],
        tag_arms: vec![],
        default_arm: "ineligible".into(),
        bounds: vec![],
        operation_map: vec![],
        enum_arms: BTreeMap::new(),
    };
    let sidecar = build_semantic_bindings(&plain, &captures, vec![binding])
        .unwrap()
        .canonical_bytes()
        .to_vec();
    let (context, captures) =
        support::context_with_sidecar(&bundle, &root, code.as_bytes(), |_| sidecar);
    let requests = json!([{"id":"bool-float-entry","compilation_id":context.compilation_id(),"roots":context.selected_root_ids(),"inputs":captures.entries().iter().map(|e|json!({"kind":if e.kind()==OriginalInputKind::Source{"source"}else{"sidecar"},"path":e.path(),"utf8":std::str::from_utf8(e.bytes()).unwrap()})).collect::<Vec<_>>()}]);
    let bytes = serde_json::to_vec_pretty(&requests).unwrap();
    if let Some(output) = std::env::var_os("MPK_W09_ENTRY_REQUESTS_OUT") {
        fs::write(output, bytes).unwrap();
    } else {
        assert_eq!(fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../develop/migrations/csharp-03/ordinary-foundation/entry-sources/requests.json")).unwrap(),bytes);
    }
}
pub(super) fn sources() -> Vec<(String, Value, Value)> {
    let mut sources = relation_tests::sources()
        .into_iter()
        .chain(domain_sources::sources())
        .collect::<Vec<_>>();
    let requests = read("ordinary-foundation/entry-sources/requests.json");
    let responses = read("ordinary-foundation/entry-sources/responses.json");
    assert_eq!(requests.as_array().unwrap().len(), 1);
    assert_eq!(responses.as_array().unwrap().len(), 1);
    assert_eq!(requests[0]["id"], responses[0]["id"]);
    assert!(responses[0].get("reject").is_none());
    sources.push((
        "bool-float-entry".into(),
        requests[0].clone(),
        responses[0]["facts"].clone(),
    ));
    sources
}
fn input(bits: Vec<bool>) -> V {
    if bits.len() == 1 {
        V::Bit(bits[0])
    } else if bits.len() <= 1024 {
        V::Cube(bits)
    } else {
        sparse_cube(
            bits.len().trailing_zeros(),
            bits.into_iter()
                .enumerate()
                .filter_map(|(i, v)| v.then_some(i))
                .collect(),
        )
    }
}
#[test]
fn csharp_03_t06_w09_entries_original_source_certificates() {
    let bundle = b();
    let mut metrics = vec![];
    let mut non_total = false;
    let output = std::env::var_os("MPK_W09_ENTRIES_OUT").map(std::path::PathBuf::from);
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/entry-operations");
    for (id, row, facts) in sources() {
        let (context, captures) = support::replay_context(&bundle, &row);
        let source = ValidatedDataSource::import_captured_facts(
            &bundle,
            &context,
            &captures,
            &serde_json::to_vec(&facts).unwrap(),
        )
        .unwrap();
        let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
        let p = generate_csharp_practical_ordinary_entries(emitted.vir()).unwrap();
        let expected = emitted
            .closure()
            .closed()
            .entries()
            .iter()
            .filter(|e| e["template_id"] == "mpk.csharp.semantic.ordered_entry.v1")
            .map(|e| e["instance_id"].as_str().unwrap())
            .collect::<BTreeSet<_>>();
        assert_eq!(
            expected,
            p.definitions()
                .iter()
                .map(|d| d.carrier.type_id.as_str())
                .collect()
        );
        if p.definitions().is_empty() {
            continue;
        }
        non_total |= p
            .definitions()
            .iter()
            .any(|d| d.compare_definition.is_none());
        let c = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        validate_csharp_practical_certificate_structure(&c).unwrap();
        let data: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
        assert_eq!(
            import_csharp_practical_ordinary_entries(
                &p.canonical_bytes(),
                p.certificate_bytes(),
                emitted.vir()
            )
            .unwrap(),
            p
        );
        for field in [
            "schema",
            "source_ir_sha256",
            "foundation_sha256",
            "definitions",
            "static_transformers",
            "certificate_sha256",
        ] {
            let mut m = data.clone();
            m[field] = json!("forged");
            assert!(import_csharp_practical_ordinary_entries(
                &serde_json::to_vec(&m).unwrap(),
                p.certificate_bytes(),
                emitted.vir()
            )
            .is_err());
        }
        for field in [
            "key_type_id",
            "value_type_id",
            "key_definition",
            "value_definition",
            "compare_definition",
        ] {
            let mut m = data.clone();
            m["definitions"][0][field] = json!("forged");
            assert!(import_csharp_practical_ordinary_entries(
                &serde_json::to_vec(&m).unwrap(),
                p.certificate_bytes(),
                emitted.vir()
            )
            .is_err());
        }
        let mut changed = p.certificate_bytes().to_vec();
        *changed.last_mut().unwrap() ^= 1;
        assert!(import_csharp_practical_ordinary_entries(
            &p.canonical_bytes(),
            &changed,
            emitted.vir()
        )
        .is_err());
        let file = format!("{id}.hex");
        let hex = p
            .certificate_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
            + "\n";
        if let Some(output) = &output {
            fs::create_dir_all(output).unwrap();
            fs::write(output.join(&file), hex).unwrap();
        } else {
            assert_eq!(fs::read_to_string(fixture.join(&file)).unwrap(), hex);
        }
        metrics.push(json!({"id":id,"file":file,"terms":c.term_table.len(),"declarations":c.declarations.len(),"metadata":data}));
    }
    assert!(metrics.len() >= 6 && non_total);
    eprintln!(
        "entry certificates: {} actual-source contexts; non-total float entry covered",
        metrics.len()
    );
    if let Some(output) = output {
        fs::write(
            output.join("certificates.json"),
            serde_json::to_vec_pretty(&metrics).unwrap(),
        )
        .unwrap();
    } else {
        assert_eq!(
            serde_json::from_slice::<Value>(&fs::read(fixture.join("certificates.json")).unwrap())
                .unwrap(),
            json!(metrics)
        );
    }
}
#[test]
fn csharp_03_t06_w09_entries_original_source_semantics() {
    std::thread::Builder::new().stack_size(64*1024*1024).spawn(||{
        let bundle=b();let mut observations=0;let mut contexts=0;let mut nan=false;
        for (id,row,facts) in sources() {
            let (context,captures)=support::replay_context(&bundle,&row);let source=ValidatedDataSource::import_captured_facts(&bundle,&context,&captures,&serde_json::to_vec(&facts).unwrap()).unwrap();let emitted=emit_data_phase(&bundle,&context,&captures,&source).unwrap();
            let p=generate_csharp_practical_ordinary_entries(emitted.vir()).unwrap();let c=mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();let types=generate_csharp_practical_ordinary_carriers(emitted.vir()).unwrap().carriers().iter().map(|c|(c.type_id.clone(),c.clone())).collect::<BTreeMap<_,_>>();
            for d in p.definitions() {
                contexts+=1;eprintln!("entry semantics {id}: {} / {}",d.key_type_id,d.value_type_id);
                let relation=generate_structural_program(&bundle,emitted.closure().roots(),emitted.closure().closed(),&d.carrier.type_id).unwrap();assert_eq!(relation.is_total(),d.compare_definition.is_some());
                let mut values=vec![];
                // Same key/different value and opposing key/value differences
                // make field completeness and lexicographic priority observable.
                for (k,v) in [(0,0),(1,2),(1,1),(2,1)] {
                    let key=relation_tests::sample(&d.key_type_id,k,&types,&facts,emitted.closure().closed());let value=relation_tests::sample(&d.value_type_id,v,&types,&facts,emitted.closure().closed());
                    let entry=MonomorphicValue::OrderedEntry{type_id:d.carrier.type_id.clone(),key:Box::new(key.clone()),value:Box::new(value.clone())};validate_monomorphic_value(&bundle,emitted.closure().roots(),emitted.closure().closed(),&entry).unwrap();
                    observe(&c,run(&c,&d.make_definition,vec![input(relation_tests::storage(&key,&types)),input(relation_tests::storage(&value,&types))]),&relation_tests::storage(&entry,&types));
                    let encoded=input(relation_tests::storage(&entry,&types));
                    observe(&c,run(&c,&d.key_definition,vec![encoded.clone()]),&relation_tests::storage(&key,&types));observe(&c,run(&c,&d.value_definition,vec![encoded]),&relation_tests::storage(&value,&types));observations+=3;values.push(entry);
                }
                for left in &values{for right in &values{
                    let args=vec![input(relation_tests::storage(left,&types)),input(relation_tests::storage(right,&types))];let expected=relation.structural_equal(left,right).unwrap();
                    assert_eq!(observed_bit(run(&c,&d.equality_definition,args.clone())),expected);observations+=1;
                    if left==right && !expected {nan=true;}
                    if let Some(compare)=&d.compare_definition {let expected=match relation.canonical_compare(left,right).unwrap(){std::cmp::Ordering::Less=>-1,std::cmp::Ordering::Equal=>0,std::cmp::Ordering::Greater=>1};assert_eq!(number(&c,run(&c,compare,args)) as i32,expected);observations+=1;}
                }}
            }
        }
        assert!(contexts>=6 && nan);eprintln!("entry semantics: {contexts} source/instance contexts; {observations} ordinary observations; NaN non-reflexivity covered");
    }).unwrap().join().unwrap();
}
