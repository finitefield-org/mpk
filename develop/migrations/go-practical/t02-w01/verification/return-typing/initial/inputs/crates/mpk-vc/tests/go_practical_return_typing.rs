use mpk_vc::go_practical_type_keys::{build_go_source_scopes,check_go_return_statement,import_go_type_key_components,GoReturnTyping,GoSourceScopes,GoTypeSourceBuffer};
use mpk_vc::{canonical_json_bytes,hash_canonical_json,parse_strict_json,HashDomain,StrictJsonLimits};
use serde_json::{json,Value};
use std::path::PathBuf;
fn root()->PathBuf{PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../develop/migrations/go-practical/t02-w01/verification/return-typing")}
fn bytes(value: &Value) -> Vec<u8> {
    let s = value.as_str().unwrap().as_bytes();
    assert_eq!(s.len() % 2, 0);
    s.chunks_exact(2)
        .map(|b| u8::from_str_radix(std::str::from_utf8(b).unwrap(), 16).unwrap())
        .collect()
}
fn retain(name: &str, value: &Value) {
    if let Some(out) = std::env::var_os("MPK_GO_RETURN_TYPING_RUST_OUTPUT") {
        use std::io::Write;
        let out = PathBuf::from(out);
        std::fs::create_dir_all(&out).unwrap();
        let mut raw = serde_json::to_vec(value).unwrap();
        raw.push(b'\n');
        std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(out.join(name))
            .unwrap()
            .write_all(&raw)
            .unwrap();
    }
}
fn scopes(input: &Value) -> std::result::Result<GoSourceScopes, String> {
    let sources = input["sources"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| {
            GoTypeSourceBuffer::import(
                &serde_json::to_vec(&v["object"]).unwrap(),
                &bytes(&v["raw_hex"]),
            )
        })
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| format!("{e:?}"))?;
    let keys = import_go_type_key_components(&bytes(&input["components_hex"]), &sources)
        .map_err(|e| format!("{e:?}"))?;
    build_go_source_scopes(&keys).map_err(|e| format!("{e:?}"))
}

fn output(value:&GoReturnTyping)->Value{
 let source=value.source_object_id();let typing=value.typing();let parsed=typing.source(source).unwrap();let stmt=&parsed.nodes()[value.statement_node()];let sig=value.signature();let fun=&parsed.nodes()[sig.function_node()];let func_type=&parsed.nodes()[sig.type_node()];let body=&parsed.nodes()[sig.body_node().unwrap()];
 let key=|id:&str|typing.types().find(|k|k.id()==id).unwrap().clone();
 let mut calls=vec![];
 for h in typing.call_headers(){
  if h.call_source_object_id()!=source{continue};let call=&parsed.nodes()[h.call_node()];if call.start()<stmt.start()||call.end()>stmt.end(){continue}
  let original=typing.source(h.declaration_source_object_id()).unwrap();let decl=&original.nodes()[h.declaration_node()];let body=h.body_node().map(|n|&original.nodes()[n]);
  calls.push(json!({"start_byte":call.start(),"end_byte":call.end(),"declaration_source":h.declaration_source_object_id(),"declaration_start":decl.start(),"declaration_end":decl.end(),"body_start":body.map(|b|b.start()),"body_end":body.map(|b|b.end())}));
 }
 calls.sort_by_key(|r|(r["start_byte"].as_u64(),r["end_byte"].as_u64()));
 let slots=sig.slots().iter().map(|slot|{let field=&parsed.nodes()[slot.field_node()];let typ=&parsed.nodes()[slot.type_node()];let name=slot.name_node().map(|n|&parsed.nodes()[n]);json!({"role":slot.role().as_str(),"field_start":field.start(),"field_end":field.end(),"type_start":typ.start(),"type_end":typ.end(),"name":slot.name(),"name_start":name.map(|n|n.start()),"name_end":name.map(|n|n.end()),"type_key":key(slot.type_id())})}).collect::<Vec<_>>();
 let bindings=value.result_bindings().iter().map(|b|b.as_ref().map(|b|{let name=&parsed.nodes()[b.name_node()];let decl=&parsed.nodes()[b.declaration_node()];json!({"name":b.name(),"name_start":name.start(),"name_end":name.end(),"declaration_start":decl.start(),"declaration_end":decl.end(),"visible_from":b.visible_from()})})).collect::<Vec<_>>();
 json!({"accepted":true,"body_state_or_program_proof":false,"function":{"kind":fun.kind(),"start_byte":fun.start(),"end_byte":fun.end(),"type_start":func_type.start(),"type_end":func_type.end(),"body_start":body.start(),"body_end":body.end(),"name":sig.name(),"slots":slots},"implicit_results":value.implicit_results(),"bindings":bindings,"values":typing.values().iter().map(|v|json!({"type_key":v.source_type_key(),"addressable":v.addressable(),"constant":v.constant().is_some(),"untyped_nil":v.is_untyped_nil()})).collect::<Vec<_>>(),"calls":calls})
}
#[test]
fn matches_original_go_return_statements(){
 let corpus:Value=serde_json::from_slice(&std::fs::read(root().join("corpus.json")).unwrap()).unwrap();let cases=corpus.as_array().unwrap();assert!(cases.len()>500);
 let(mut accepted,mut rejected,mut outside)=(0,0,0);let mut failures=vec![];
 for(n,row)in cases.iter().enumerate(){
  let input=&row["input"];retain(&format!("case-{n:05}-input.json"),input);
  let actual=scopes(input).and_then(|graph|{let source=input["query_source_object_id"].as_str().unwrap();let parsed=graph.source(source).ok_or("source".to_string())?;let node=parsed.nodes().iter().position(|v|v.kind()=="ReturnStmt"&&v.start()as u64==input["statement_start_byte"].as_u64().unwrap()&&v.end()as u64==input["statement_end_byte"].as_u64().unwrap()).ok_or("return".to_string())?;check_go_return_statement(&graph,source,node).map_err(|e|format!("{e:?}"))});
  let outcome=match &actual{Ok(v)=>output(v),Err(e)=>json!({"accepted":false,"error":e,"body_state_or_program_proof":false})};retain(&format!("case-{n:05}-actual.json"),&outcome);
  if input["outside_return_typing"]==true&&row["native"]["accepted"]==true{if actual.is_ok(){failures.push(format!("outside case {n} {} accepted: {outcome}",input["label"]))};outside+=1}
  else if row["native"]["accepted"]==true{if outcome!=row["native"]{failures.push(format!("case {n} {} expected {} actual {outcome}",input["label"],row["native"]))};accepted+=1}
  else{if actual.is_ok(){failures.push(format!("case {n} {} falsely accepted: {outcome}",input["label"]))};rejected+=1}
 }
 retain("summary.json",&json!({"accepted":accepted,"rejected":rejected,"outside":outside,"failures":failures}));assert!(failures.is_empty(),"{}",failures.join("\n"));
}
fn canonical(v: &Value) -> Vec<u8> {
    canonical_json_bytes(
        &parse_strict_json(
            &serde_json::to_vec(v).unwrap(),
            StrictJsonLimits::new(268_435_456, 268_435_456, 64, 16_777_216),
        )
        .unwrap(),
    )
    .unwrap()
}
fn seal(v: &mut Value, domain: &'static str, field: &str) {
    v.as_object_mut().unwrap().remove(field);
    let b = canonical(v);
    let x = parse_strict_json(
        &b,
        StrictJsonLimits::new(268_435_456, 268_435_456, 64, 16_777_216),
    )
    .unwrap();
    v[field] = json!(hash_canonical_json(HashDomain::new(domain), &x)
        .unwrap()
        .to_hex())
}
fn sort(v: &mut Value, key: &str) {
    v.as_array_mut()
        .unwrap()
        .sort_by(|a, b| a[key].as_str().cmp(&b[key].as_str()))
}

fn control(label: &str, text: &str) -> (Value, GoSourceScopes) {
    let cap = format!("scalar-controls/{label}");
    let file = GoTypeSourceBuffer::new(&cap, "policy.go", "go_file", text.as_bytes()).unwrap();
    let module = GoTypeSourceBuffer::new(
        &cap,
        "go.mod",
        "go_mod",
        b"module example.com/scalar\ngo 1.25.0\n",
    )
    .unwrap();
    let mut p = json!({"schema":"mpk.go.type_package.v2","module_source_object_id":module.object().id,"import_path":"example.com/scalar","package_name":"policy","source_objects":[file.object().id]});
    seal(&mut p, "MPK-GO-TYPE-PACKAGE-2.0", "id");
    let mut t = json!({"schema":"mpk.go.source_type.v2","kind":"builtin","name":"bool","width":null,"signed":null});
    seal(&mut t, "MPK-GO-SOURCE-TYPE-2.0", "id");
    let mut v = json!({"schema":"mpk.go.type_components.v2","source_objects":[file.object(),module.object()],"packages":[p.clone()],"declarations":[],"types":[t.clone()],"underlying":[],"aliases":[],"package_roots":[p["id"]],"type_roots":[t["id"]]});
    sort(&mut v["source_objects"], "id");
    seal(&mut v, "MPK-GO-TYPE-COMPONENTS-2.0", "content_sha256");
    let input = json!({"components_hex":canonical(&v).iter().map(|b|format!("{b:02x}")).collect::<String>(),"sources":[{"object":file.object(),"raw_hex":file.bytes().iter().map(|b|format!("{b:02x}")).collect::<String>()},{"object":module.object(),"raw_hex":module.bytes().iter().map(|b|format!("{b:02x}")).collect::<String>()}]});
    retain(&format!("{label}-input.json"), &input);
    let graph = scopes(&input).unwrap();
    (input, graph)
}


#[test]
fn returns_keep_nearest_function_bindings_and_opaque_joint_calls_without_checking_the_body(){
 let(input,graph)=control("return-owned",r#"package policy
type W uint16
func Pair()(W,error)
func Probe(x W)(r W,e error){
 _ = func()(r string){return "inner"}
 if true{r:="shadow";_=r;return x,nil}
 if false{r:="shadow";_=r;return}
 if false{return Pair()}
 r="unchecked elsewhere"
 return
}
"#);
 let source=input["sources"][0]["object"]["id"].as_str().unwrap().to_string();let parsed=graph.source(&source).unwrap();
 let returns=parsed.nodes().iter().enumerate().filter(|(_,n)|n.kind()=="ReturnStmt").map(|(i,_)|i).collect::<Vec<_>>();assert_eq!(returns.len(),5);
 let typed=returns.iter().map(|&n|check_go_return_statement(&graph,&source,n)).collect::<Vec<_>>();
 let inner=typed[0].as_ref().unwrap();assert_eq!(parsed.nodes()[inner.signature().function_node()].kind(),"FuncLit");assert_eq!(inner.typing().values()[0].scalar_type().unwrap().underlying_builtin(),"string");
 let explicit=typed[1].as_ref().unwrap();assert!(!explicit.implicit_results());assert_eq!(explicit.result_bindings().len(),2);assert_eq!(explicit.typing().values().len(),2);assert!(typed[2].is_err());
 let pair=typed[3].as_ref().unwrap();assert_eq!(pair.typing().call_headers().len(),1);assert!(pair.typing().call_headers()[0].body_node().is_none());assert_eq!(pair.typing().values().len(),2);
 let implicit=typed[4].as_ref().unwrap();assert!(implicit.implicit_results());assert_eq!(implicit.signature().function_node(),explicit.signature().function_node());assert_ne!(implicit.signature().function_node(),inner.signature().function_node());assert!(implicit.typing().call_headers().is_empty());
 assert!(check_go_return_statement(&graph,"missing",returns[0]).is_err());assert!(check_go_return_statement(&graph,&source,usize::MAX).is_err());let not_return=parsed.nodes().iter().position(|n|n.kind()=="AssignStmt").unwrap();assert!(check_go_return_statement(&graph,&source,not_return).is_err());
 retain("owned-return-actual.json",&json!({"inner":output(inner),"explicit":output(explicit),"joint":output(pair),"implicit":output(implicit),"shadowed_naked_rejected":true,"unrelated_assignment_not_typechecked":true,"body_state_or_program_proof":false}));
 let saved=implicit.clone();let last=returns[4];drop(graph);drop(input);assert_eq!(saved.typing().source(&source).unwrap().spelling(last),Some(b"return".as_slice()));
}
#[test]
fn actual_return_context_work_is_flat(){
 let text=format!("package policy\nfunc Probe(n uint64)uint16{{return {}1<<n}}\n","- ".repeat(20000));
 let(input,graph)=control("return-deep",&text);let source=input["sources"][0]["object"]["id"].as_str().unwrap();let parsed=graph.source(source).unwrap();let node=parsed.nodes().iter().position(|n|n.kind()=="ReturnStmt").unwrap();let typed=check_go_return_statement(&graph,source,node).unwrap();assert_eq!(typed.typing().values().len(),1);retain("return-deep-actual.json",&output(&typed));
}
