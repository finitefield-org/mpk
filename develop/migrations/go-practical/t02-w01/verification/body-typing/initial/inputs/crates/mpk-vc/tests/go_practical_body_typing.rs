use mpk_vc::go_practical_type_keys::{build_go_source_scopes,check_go_function_body,evaluate_go_constant_declaration,import_go_type_key_components,GoBodyTyping,GoBodyErrorKind,GoSourceScopes,GoTypeSourceBuffer};
use mpk_vc::{canonical_json_bytes,hash_canonical_json,parse_strict_json,HashDomain,StrictJsonLimits};
use serde_json::{json,Value};
use std::path::PathBuf;
fn root()->PathBuf{PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../develop/migrations/go-practical/t02-w01/verification/body-typing")}
fn bytes(value: &Value) -> Vec<u8> {
    let s = value.as_str().unwrap().as_bytes();
    assert_eq!(s.len() % 2, 0);
    s.chunks_exact(2)
        .map(|b| u8::from_str_radix(std::str::from_utf8(b).unwrap(), 16).unwrap())
        .collect()
}
fn retain(name: &str, value: &Value) {
    if let Some(out) = std::env::var_os("MPK_GO_BODY_TYPING_RUST_OUTPUT") {
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

fn output(value:&GoBodyTyping)->Value{
 let typing=value.typing();let sig=value.signature();let source=sig.source_object_id();let parsed=typing.source(source).unwrap();let fun=&parsed.nodes()[sig.function_node()];let func_type=&parsed.nodes()[sig.type_node()];let body=&parsed.nodes()[sig.body_node().unwrap()];
 let key=|id:&str|typing.types().find(|k|k.id()==id).unwrap().clone();
 let slots=sig.slots().iter().map(|slot|{let field=&parsed.nodes()[slot.field_node()];let typ=&parsed.nodes()[slot.type_node()];let name=slot.name_node().map(|n|&parsed.nodes()[n]);json!({"role":slot.role().as_str(),"field_start":field.start(),"field_end":field.end(),"type_start":typ.start(),"type_end":typ.end(),"name":slot.name(),"name_start":name.map(|n|n.start()),"name_end":name.map(|n|n.end()),"type_key":key(slot.type_id())})}).collect::<Vec<_>>();
 let mut statements=value.statements().iter().map(|s|{let n=&parsed.nodes()[s.node()];json!({"start_byte":n.start(),"end_byte":n.end(),"kind":n.kind(),"value_types":s.value_type_ids().iter().map(|t|key(t)).collect::<Vec<_>>(),"terminating":s.syntactically_terminating()})}).collect::<Vec<_>>();
 let mut expressions=value.expressions().iter().map(|s|{let n=&parsed.nodes()[s.node()];json!({"start_byte":n.start(),"end_byte":n.end(),"role":s.role(),"type_key":key(s.type_id()),"untyped_nil":s.is_untyped_nil()})}).collect::<Vec<_>>();
 let mut branches=value.branches().iter().map(|b|{let n=&parsed.nodes()[b.node()];let t=&parsed.nodes()[b.target_node()];json!({"start_byte":n.start(),"end_byte":n.end(),"target_start":t.start(),"target_end":t.end(),"operator":n.attributes().find(|(k,_)|*k=="Tok").unwrap().1})}).collect::<Vec<_>>();
 let mut constants=value.constants().iter().map(|c|{let n=&parsed.nodes()[c.spec_node()];json!({"start_byte":n.start(),"end_byte":n.end(),"values":c.values().iter().map(|c|json!({"kind":c.kind().as_str(),"type_key":c.scalar_type().map(|t|t.source_type_key()),"integer_decimal":c.integer_decimal(),"boolean":c.boolean(),"bytes_hex":c.bytes().map(|b|b.iter().map(|b|format!("{b:02x}")).collect::<String>())})).collect::<Vec<_>>()})}).collect::<Vec<_>>();
 let mut locals=value.local_variables().iter().map(|b|{let n=&parsed.nodes()[b.name_node()];let d=&parsed.nodes()[b.declaration_node()];json!({"name":b.name(),"name_start":n.start(),"name_end":n.end(),"declaration_start":d.start(),"declaration_end":d.end(),"type_key":key(value.local_type_id(b).unwrap())})}).collect::<Vec<_>>();
 let sort=|rows:&mut Vec<Value>,start:&str,end:&str|rows.sort_by_key(|v|(v[start].as_u64(),v[end].as_u64()));sort(&mut statements,"start_byte","end_byte");sort(&mut expressions,"start_byte","end_byte");sort(&mut branches,"start_byte","end_byte");sort(&mut constants,"start_byte","end_byte");sort(&mut locals,"name_start","name_end");
 json!({"accepted":true,"evaluation_state_or_program_proof":false,"function":{"start_byte":fun.start(),"end_byte":fun.end(),"type_start":func_type.start(),"type_end":func_type.end(),"body_start":body.start(),"body_end":body.end(),"name":sig.name(),"slots":slots},"statements":statements,"expressions":expressions,"branches":branches,"constants":constants,"locals":locals,"terminating":value.syntactically_terminating()})
}
#[test]
fn matches_original_go_function_bodies(){
 let corpus:Value=serde_json::from_slice(&std::fs::read(root().join("corpus.json")).unwrap()).unwrap();let cases=corpus.as_array().unwrap();assert!(cases.len()>400);
 let(mut accepted,mut rejected,mut outside)=(0,0,0);let mut failures=vec![];
 for(n,row)in cases.iter().enumerate(){
  let input=&row["input"];retain(&format!("case-{n:05}-input.json"),input);
  let actual=scopes(input).and_then(|graph|{let source=input["query_source_object_id"].as_str().unwrap();let parsed=graph.source(source).ok_or("source".to_string())?;let node=parsed.nodes().iter().position(|v|v.kind()=="FuncDecl"&&v.start()as u64==input["function_start_byte"].as_u64().unwrap()&&v.end()as u64==input["function_end_byte"].as_u64().unwrap()).ok_or("function".to_string())?;check_go_function_body(&graph,source,node).map_err(|e|format!("{e:?}"))});
  let outcome=match &actual{Ok(v)=>output(v),Err(e)=>json!({"accepted":false,"error":e,"evaluation_state_or_program_proof":false})};retain(&format!("case-{n:05}-actual.json"),&outcome);
  if input["outside_body_typing"]==true&&row["native"]["accepted"]==true{if actual.is_ok(){failures.push(format!("outside case {n} {} accepted: {outcome}",input["label"]))};outside+=1}
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
fn body_checks_own_statements_branches_and_constants_but_not_selected_callee_closure(){
 let(input,graph)=control("body-owned",r#"package policy
type W uint16
func Pair(x W)(W,error)
func BadHelper(){_=make(map[int]int)}
func Probe(x W,c bool,s []W)(r W,e error){
 const(C=iota;_;D)
 var a[D]W
 a[0]=x
 for i,v:=range s{switch{case c:r+=v;continue;default:_=i};if v==0{break}}
 if c{return Pair(x)}
 return
}
func Calls(){BadHelper()}
"#);
 let source=input["sources"][0]["object"]["id"].as_str().unwrap().to_string();let parsed=graph.source(&source).unwrap();
 let fun=|name:&str|parsed.nodes().iter().enumerate().find(|(_,n)|n.kind()=="FuncDecl"&&n.edges().iter().find(|e|e.role()=="Name").is_some_and(|e|parsed.spelling(e.targets()[0])==Some(name.as_bytes()))).unwrap().0;
 let probe=check_go_function_body(&graph,&source,fun("Probe")).unwrap();assert!(probe.syntactically_terminating());assert_eq!(probe.branches().len(),2);for b in probe.branches(){assert_eq!(parsed.nodes()[b.target_node()].kind(),"RangeStmt")};assert_eq!(probe.constants().len(),3);assert_eq!(probe.constants()[1].values()[0].integer_decimal().as_deref(),Some("1"));assert_eq!(probe.local_variables().len(),3);assert_eq!(probe.typing().call_headers().len(),1);assert!(probe.typing().call_headers()[0].body_node().is_none());
 let calls=check_go_function_body(&graph,&source,fun("Calls")).unwrap();assert_eq!(calls.typing().call_headers().len(),1);assert!(check_go_function_body(&graph,&source,fun("BadHelper")).is_err());assert!(check_go_function_body(&graph,"missing",0).is_err());assert!(check_go_function_body(&graph,&source,usize::MAX).is_err());
 retain("owned-body-actual.json",&json!({"probe":output(&probe),"calls":output(&calls),"unsupported_callee_body_rejected_separately":true,"callee_closure_or_evaluation_state_program_authority":false}));let saved=probe.clone();drop(graph);drop(input);assert!(!saved.typing().source(&source).unwrap().source().bytes().is_empty());
}
#[test]
fn body_rejects_unreachable_type_errors_unused_locals_and_missing_return_with_original_locations(){
 for(label,text,want)in[("unreachable","package policy\nfunc Probe(){return;var _ uint8=256}","typing"),("unused","package policy\nfunc Probe(){var q uint16;q=1}","unused"),("missing","package policy\nfunc Probe(c bool)uint16{if c{return 1}}","missing"),("branch","package policy\nfunc Probe(){continue}","branch")]{
  let(input,graph)=control(label,text);let source=input["sources"][0]["object"]["id"].as_str().unwrap();let p=graph.source(source).unwrap();let n=p.nodes().iter().position(|n|n.kind()=="FuncDecl").unwrap();let err=check_go_function_body(&graph,source,n).unwrap_err();assert_eq!(err.source_object_id(),source);assert!(p.nodes().get(err.node()).is_some());match want{"typing"=>assert!(matches!(err.kind(),GoBodyErrorKind::Typing(_))),"unused"=>assert_eq!(err.kind(),&GoBodyErrorKind::UnusedVariable),"missing"=>assert_eq!(err.kind(),&GoBodyErrorKind::MissingReturn),_=>assert_eq!(err.kind(),&GoBodyErrorKind::Branch)};retain(&format!("{label}-actual.json"),&json!({"error":format!("{err:?}"),"source":err.source_object_id(),"node":err.node()}));
 }
}
#[test]
fn body_work_and_whole_original_blank_constant_specs_are_flat(){
 let text=format!("package policy\nfunc Probe(x uint16){{if {}(x==x){{}}}}\n","!".repeat(20000));let(input,graph)=control("body-deep",&text);let source=input["sources"][0]["object"]["id"].as_str().unwrap();let p=graph.source(source).unwrap();let n=p.nodes().iter().position(|n|n.kind()=="FuncDecl").unwrap();let value=check_go_function_body(&graph,source,n).unwrap();retain("body-deep-actual.json",&output(&value));
 let(input,graph)=control("blank-constant-spec","package policy\nconst(A uint16=iota;_;B;_,C=iota,iota)\nfunc Probe(){}\n");let source=input["sources"][0]["object"]["id"].as_str().unwrap();let p=graph.source(source).unwrap();let specs=p.nodes().iter().enumerate().filter(|(_,n)|n.kind()=="ValueSpec").map(|(n,_)|n).collect::<Vec<_>>();let values=specs.iter().map(|&n|evaluate_go_constant_declaration(&graph,source,n).unwrap()).collect::<Vec<_>>();assert_eq!(values[1][0].integer_decimal().as_deref(),Some("1"));assert_eq!(values[3].len(),2);assert_eq!(values[3][1].integer_decimal().as_deref(),Some("3"));assert!(evaluate_go_constant_declaration(&graph,source,usize::MAX).is_err());retain("blank-constant-spec-actual.json",&json!({"values":values.iter().map(|v|v.iter().map(|v|v.integer_decimal()).collect::<Vec<_>>()).collect::<Vec<_>>()}));
}
