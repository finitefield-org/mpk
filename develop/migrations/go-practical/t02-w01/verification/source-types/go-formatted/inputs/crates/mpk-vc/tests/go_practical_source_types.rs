use mpk_vc::go_practical_type_keys::{
    build_go_source_scopes,check_go_source_type_declarations,derive_go_source_type,
    derive_go_source_type_declaration,import_go_type_key_components,GoDerivedSourceTypes,
    GoSourceTypeError,GoTypeKeyComponents,GoTypeSourceBuffer,
};
use mpk_vc::{canonical_json_bytes,hash_canonical_json,parse_strict_json,HashDomain,StrictJsonLimits};
use serde_json::{json,Value};
use std::collections::BTreeMap;
use std::path::{Path,PathBuf};
fn root()->PathBuf{PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../develop/migrations/go-practical/t02-w01/verification/source-types")}
fn canonical(v:&Value)->Vec<u8>{canonical_json_bytes(&parse_strict_json(&serde_json::to_vec(v).unwrap(),StrictJsonLimits::new(268_435_456,268_435_456,64,16_777_216)).unwrap()).unwrap()}
fn seal(v:&mut Value,domain:&'static str,field:&str){v.as_object_mut().unwrap().remove(field);let b=canonical(v);let x=parse_strict_json(&b,StrictJsonLimits::new(268_435_456,268_435_456,64,16_777_216)).unwrap();v[field]=json!(hash_canonical_json(HashDomain::new(domain),&x).unwrap().to_hex())}
fn sort(v:&mut Value,key:&str){v.as_array_mut().unwrap().sort_by(|a,b|a[key].as_str().cmp(&b[key].as_str()))}
fn retain(name:&str,v:&Value){if let Some(out)=std::env::var_os("MPK_GO_SOURCE_TYPES_RUST_OUTPUT"){
    use std::io::Write;let out=PathBuf::from(out);std::fs::create_dir_all(&out).unwrap();let mut raw=canonical(v);raw.push(b'\n');std::fs::OpenOptions::new().create_new(true).write(true).open(out.join(name)).unwrap().write_all(&raw).unwrap();
}}
fn fixture(path:&Path)->(Vec<u8>,Vec<GoTypeSourceBuffer>){
    let mut paths:Vec<_>=std::fs::read_dir(path.join("sources")).unwrap().map(|e|e.unwrap().path()).collect();paths.sort();
    let buffers=paths.into_iter().filter(|p|p.extension().is_some_and(|e|e=="json")).map(|p|GoTypeSourceBuffer::import(&std::fs::read(&p).unwrap(),&std::fs::read(p.with_extension("raw")).unwrap()).unwrap()).collect();
    (std::fs::read(path.join("components.json")).unwrap(),buffers)
}
fn result(v:&GoDerivedSourceTypes)->Value{
    json!({"roots":v.roots(),"types":v.types().collect::<Vec<_>>(),"source_declarations":v.declarations().collect::<Vec<_>>(),"underlying":v.types().filter_map(|t|v.underlying(t.id()).map(|u|json!({"type_id":t.id(),"underlying_type_id":u}))).collect::<Vec<_>>(),"aliases":v.declarations().filter_map(|d|v.alias_target(&d.id).map(|a|json!({"declaration_id":d.id,"target_type_id":a}))).collect::<Vec<_>>(),"source_typing_or_context_or_program_authority":false})
}
#[test]
fn derives_actual_original_go_declaration_graphs(){
    for (group,count)in[("previous",5),("current",7)]{
        for n in 0..count{
            let path=if group=="previous"{root().parent().unwrap().join("type-keys/native-components").join(format!("case-{n:02}"))}else{root().join("native-components").join(format!("case-{n:02}"))};
            let(wire,buffers)=fixture(&path);let label=format!("{group}-{n:02}");
            retain(&format!("{label}-before.json"),&json!({"components":serde_json::from_slice::<Value>(&wire).unwrap(),"sources":buffers.iter().map(|b|json!({"object":b.object(),"whole_original_bytes":b.bytes()})).collect::<Vec<_>>()}));
            let keys=import_go_type_key_components(&wire,&buffers).unwrap();let derived=check_go_source_type_declarations(&keys).unwrap();retain(&format!("{label}-actual.json"),&result(&derived));
            let graph=build_go_source_scopes(&keys).unwrap();
            for d in keys.declarations(){
                let bound=keys.source_declarations().declaration(&d.id).unwrap();
                let one=derive_go_source_type_declaration(&graph,&d.source_object_id,bound.type_spec_node()).unwrap();
                let rhs=derive_go_source_type(&graph,&d.source_object_id,bound.type_node()).unwrap();
                if d.kind=="alias_type"{assert_eq!(one.roots(),rhs.roots())}else{assert_eq!(one.underlying(&one.roots()[0]),Some(rhs.underlying(&rhs.roots()[0]).unwrap_or(&rhs.roots()[0])))}
            }
            drop(graph);drop(keys);drop(buffers);assert!(derived.types().all(|t|!t.id().is_empty()));
        }
    }
}
// Repair the complete type identity DAG and its references. Negative source
// controls must not be rejected merely for stale hashes or broken edges.
fn rekey(v:&mut Value){
    fn node(id:&str,nodes:&BTreeMap<String,Value>,done:&mut BTreeMap<String,Value>)->String{
        if let Some(n)=done.get(id){return n["id"].as_str().unwrap().into()}
        let mut n=nodes[id].clone();match n["kind"].as_str().unwrap(){
            "array"|"slice"=>n["element"]=json!(node(n["element"].as_str().unwrap(),nodes,done)),
            "struct"=>for f in n["fields"].as_array_mut().unwrap(){f["type_id"]=json!(node(f["type_id"].as_str().unwrap(),nodes,done))},_=>(),
        }
        seal(&mut n,"MPK-GO-SOURCE-TYPE-2.0","id");let next=n["id"].as_str().unwrap().to_string();done.insert(id.into(),n);next
    }
    let nodes=v["types"].as_array().unwrap().iter().map(|n|(n["id"].as_str().unwrap().to_string(),n.clone())).collect::<BTreeMap<_,_>>();let mut done=BTreeMap::new();for id in nodes.keys(){node(id,&nodes,&mut done);}
    let ids=done.iter().map(|(id,n)|(id.clone(),n["id"].clone())).collect::<BTreeMap<_,_>>();v["types"]=Value::Array(done.into_values().collect());sort(&mut v["types"],"id");
    for u in v["underlying"].as_array_mut().unwrap(){u["type_id"]=ids[u["type_id"].as_str().unwrap()].clone();u["underlying_type_id"]=ids[u["underlying_type_id"].as_str().unwrap()].clone()};sort(&mut v["underlying"],"type_id");
    for a in v["aliases"].as_array_mut().unwrap(){a["target_type_id"]=ids[a["target_type_id"].as_str().unwrap()].clone()}
    for r in v["type_roots"].as_array_mut().unwrap(){*r=ids[r.as_str().unwrap()].clone()};v["type_roots"].as_array_mut().unwrap().sort_by(|a,b|a.as_str().cmp(&b.as_str()));seal(v,"MPK-GO-TYPE-COMPONENTS-2.0","content_sha256");
}
#[test]
fn rejects_rehashed_source_type_body_forgery(){
    let(wire,buffers)=fixture(&root().parent().unwrap().join("type-keys/native-components/case-00"));let original:Value=serde_json::from_slice(&wire).unwrap();
    for label in ["array-length","array-child","field-order","field-tag","embedded-marker","alias-target","nominal-underlying"]{
        let mut v=original.clone();
        let uint8=v["types"].as_array().unwrap().iter().find(|n|n["kind"]=="builtin"&&n["name"]=="uint8").unwrap()["id"].clone();
        let uint16=v["types"].as_array().unwrap().iter().find(|n|n["kind"]=="builtin"&&n["name"]=="uint16").unwrap()["id"].clone();
        match label{
            "array-length"=>{let t=v["types"].as_array_mut().unwrap().iter_mut().find(|t|t["kind"]=="array"&&t["length"]==256).unwrap();t["length"]=json!(255)},
            "array-child"=>{let t=v["types"].as_array_mut().unwrap().iter_mut().find(|t|t["kind"]=="array"&&t["length"]==256).unwrap();t["element"]=uint16.clone()},
            "field-order"=>{let t=v["types"].as_array_mut().unwrap().iter_mut().find(|t|t["kind"]=="struct"&&t["fields"].as_array().unwrap().len()>3).unwrap();t["fields"].as_array_mut().unwrap().swap(0,1)},
            "field-tag"=>{let f=v["types"].as_array_mut().unwrap().iter_mut().filter_map(|t|t.get_mut("fields").and_then(Value::as_array_mut)).flat_map(|f|f.iter_mut()).find(|f|f["tag_bytes_hex"]=="ff").unwrap();f["tag_bytes_hex"]=json!("fe")},
            "embedded-marker"=>{let f=v["types"].as_array_mut().unwrap().iter_mut().filter_map(|t|t.get_mut("fields").and_then(Value::as_array_mut)).flat_map(|f|f.iter_mut()).find(|f|f["embedded"]==true).unwrap();f["embedded"]=json!(false)},
            "alias-target"=>v["aliases"][0]["target_type_id"]=uint8.clone(),
            "nominal-underlying"=>{let u=v["underlying"].as_array_mut().unwrap().iter_mut().find(|u|u["underlying_type_id"]==uint16).unwrap();u["underlying_type_id"]=uint8},
            _=>unreachable!(),
        }
        rekey(&mut v);retain(&format!("{label}-before.json"),&json!({"components":v,"sources":buffers.iter().map(|b|json!({"object":b.object(),"whole_original_bytes":b.bytes()})).collect::<Vec<_>>()}));
        let keys=import_go_type_key_components(&canonical(&v),&buffers).unwrap();let error=check_go_source_type_declarations(&keys).unwrap_err();retain(&format!("{label}-actual.json"),&json!({"error":format!("{error:?}")}));assert_eq!(error,GoSourceTypeError::Mismatch,"{label}");
    }
}
fn candidate(label:&str,text:&str)->GoTypeKeyComponents{
    let file=GoTypeSourceBuffer::new("root","policy.go","go_file",text.as_bytes()).unwrap();let module=GoTypeSourceBuffer::new("root","go.mod","go_mod",b"module example.com/policy\ngo 1.25.0\n").unwrap();
    let mut p=json!({"schema":"mpk.go.type_package.v2","module_source_object_id":module.object().id,"import_path":"example.com/policy","package_name":"policy","source_objects":[file.object().id]});seal(&mut p,"MPK-GO-TYPE-PACKAGE-2.0","id");
    let mut t=json!({"schema":"mpk.go.source_type.v2","kind":"builtin","name":"bool","width":null,"signed":null});seal(&mut t,"MPK-GO-SOURCE-TYPE-2.0","id");
    let mut v=json!({"schema":"mpk.go.type_components.v2","source_objects":[file.object(),module.object()],"packages":[p.clone()],"declarations":[],"types":[t.clone()],"underlying":[],"aliases":[],"package_roots":[p["id"]],"type_roots":[t["id"]]});sort(&mut v["source_objects"],"id");seal(&mut v,"MPK-GO-TYPE-COMPONENTS-2.0","content_sha256");
    retain(&format!("{label}-before.json"),&json!({"components":v,"sources":[{"object":file.object(),"whole_original_bytes":file.bytes()},{"object":module.object(),"whole_original_bytes":module.bytes()}]}));
    import_go_type_key_components(&canonical(&v),&[file,module]).unwrap()
}
#[test]
fn original_type_geometry_cycles_and_non_type_names_fail(){
    for(label,text,error)in[
        ("array257","package policy\ntype Result [257]byte\n",GoSourceTypeError::Limit),
        ("negative-array","package policy\ntype Result [-1]byte\n",GoSourceTypeError::Limit),
        ("bool-array","package policy\ntype Result [true]byte\n",GoSourceTypeError::Constant),
        ("constant-overflow","package policy\ntype Result [uint8(255)+1]byte\n",GoSourceTypeError::Constant),
        ("named-slice-cycle","package policy\ntype Result []Result\n",GoSourceTypeError::Cycle),
        ("zero-array-cycle","package policy\ntype Result [0]Result\n",GoSourceTypeError::Cycle),
        ("alias-cycle","package policy\ntype Result=Other\ntype Other=Result\n",GoSourceTypeError::Cycle),
        ("named-cycle","package policy\ntype Result Other\ntype Other Result\n",GoSourceTypeError::Cycle),
        ("variable-shadow","package policy\nvar uint16=1\ntype Result uint16\n",GoSourceTypeError::NotType),
        ("selected-generic","package policy\ntype Result[T any] []T\n",GoSourceTypeError::Unsupported),
        ("pointer-outside-domain","package policy\ntype Result *uint16\n",GoSourceTypeError::Unsupported),
        ("named-interface-outside-domain","package policy\ntype Result error\n",GoSourceTypeError::Unsupported),
    ]{
        let keys=candidate(label,text);let graph=build_go_source_scopes(&keys).unwrap();let b=graph.bindings().iter().find(|b|b.name()=="Result").unwrap();
        let actual=derive_go_source_type_declaration(&graph,b.source_object_id(),b.declaration_node()).unwrap_err();retain(&format!("{label}-actual.json"),&json!({"error":format!("{actual:?}")}));assert_eq!(actual,error,"{label}");
    }
    for(depth,pass)in[(16,true),(17,false)]{
        let text=format!("package policy\ntype Result {}byte\n","[0]".repeat(depth));let label=format!("depth-{depth}");let keys=candidate(&label,&text);let graph=build_go_source_scopes(&keys).unwrap();let b=graph.bindings().iter().find(|b|b.name()=="Result").unwrap();let actual=derive_go_source_type_declaration(&graph,b.source_object_id(),b.declaration_node());
        assert_eq!(actual.is_ok(),pass);retain(&format!("{label}-actual.json"),&match actual{Ok(v)=>result(&v),Err(e)=>json!({"error":format!("{e:?}")})});
    }
    for(count,pass)in[(64,true),(65,false)]{
        let fields=(0..count).map(|n|format!("F{n} byte;")).collect::<String>();let text=format!("package policy\ntype Result struct{{{fields}}}\n");let label=format!("fields-{count}");let keys=candidate(&label,&text);let graph=build_go_source_scopes(&keys).unwrap();let b=graph.bindings().iter().find(|b|b.name()=="Result").unwrap();let actual=derive_go_source_type_declaration(&graph,b.source_object_id(),b.declaration_node());
        assert_eq!(actual.is_ok(),pass);retain(&format!("{label}-actual.json"),&match actual{Ok(v)=>result(&v),Err(e)=>json!({"error":format!("{e:?}")})});
    }
}
#[test]
fn anonymous_candidate_roots_gain_no_source_authority(){
    let keys=candidate("anonymous-boundary","package policy\ntype Word uint16\n");let checked=check_go_source_type_declarations(&keys).unwrap();assert!(checked.roots().is_empty());assert_eq!(checked.types().count(),0);
    let graph=build_go_source_scopes(&keys).unwrap();assert_eq!(derive_go_source_type(&graph,"missing",0),Err(GoSourceTypeError::Locator));
    let b=graph.bindings().iter().find(|b|b.name()=="Word").unwrap();let actual=derive_go_source_type_declaration(&graph,b.source_object_id(),b.declaration_node()).unwrap();retain("anonymous-boundary-actual.json",&result(&actual));assert_eq!(actual.roots().len(),1);
}
