use mpk_vc::go_practical_type_keys::{
    build_go_source_scopes,evaluate_go_constant,evaluate_go_constant_binding,
    import_go_type_key_components,GoConstantError,GoEvaluatedConstant,GoSourceScopes,
    GoTypeSourceBuffer,
};
use serde_json::{json,Value};
use std::path::PathBuf;
fn root()->PathBuf{PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../develop/migrations/go-practical/t02-w01/verification/source-constants")}
fn bytes(value:&Value)->Vec<u8>{let s=value.as_str().unwrap().as_bytes();assert_eq!(s.len()%2,0);s.chunks_exact(2).map(|b|u8::from_str_radix(std::str::from_utf8(b).unwrap(),16).unwrap()).collect()}
fn retain(name:&str,value:&Value){
    if let Some(out)=std::env::var_os("MPK_GO_SOURCE_CONSTANTS_RUST_OUTPUT"){
        use std::io::Write;let out=PathBuf::from(out);std::fs::create_dir_all(&out).unwrap();
        let mut raw=serde_json::to_vec(value).unwrap();raw.push(b'\n');
        std::fs::OpenOptions::new().create_new(true).write(true).open(out.join(name)).unwrap().write_all(&raw).unwrap();
    }
}
fn scopes(input:&Value)->std::result::Result<GoSourceScopes,String>{
    let sources=input["sources"].as_array().unwrap().iter().map(|v|GoTypeSourceBuffer::import(&serde_json::to_vec(&v["object"]).unwrap(),&bytes(&v["raw_hex"]))).collect::<std::result::Result<Vec<_>,_>>().map_err(|e|format!("{e:?}"))?;
    let keys=import_go_type_key_components(&bytes(&input["components_hex"]),&sources).map_err(|e|format!("{e:?}"))?;
    build_go_source_scopes(&keys).map_err(|e|format!("{e:?}"))
}
fn output(value:&GoEvaluatedConstant)->Value{
    json!({"accepted":true,"kind":value.kind().as_str(),"integer_decimal":value.integer_decimal(),"boolean":value.boolean(),"bytes_hex":value.bytes().map(|raw|raw.iter().map(|b|format!("{b:02x}")).collect::<String>()),"type_key":value.scalar_type().map(|t|t.source_type_key()),"underlying_builtin":value.scalar_type().map(|t|t.underlying_builtin()),"source_typing_or_context_or_program_authority":false})
}
#[test]
fn exact_original_go_scalar_constants(){
    let corpus:Value=serde_json::from_slice(&std::fs::read(root().join("corpus.json")).unwrap()).unwrap();
    let cases=corpus.as_array().unwrap();assert!(cases.len()>5000);
    let(mut accepted,mut rejected,mut outside)=(0,0,0);
    for(n,row)in cases.iter().enumerate(){
        let input=&row["input"];retain(&format!("case-{n:05}-input.json"),input);
        let graph=scopes(input);
        let result=graph.as_ref().map_err(Clone::clone).and_then(|graph|{
            let binding=graph.bindings().iter().find(|b|b.source_object_id()==input["query_source_object_id"].as_str().unwrap()&&b.name()=="Result").ok_or_else(||"missing binding".to_string())?;
            evaluate_go_constant_binding(graph,binding).map_err(|e|format!("{e:?}"))
        });
        let actual=match &result{Ok(v)=>output(v),Err(e)=>json!({"accepted":false,"error":e,"source_typing_or_context_or_program_authority":false})};
        retain(&format!("case-{n:05}-actual.json"),&actual);
        if input["outside_scalar_reader"]==true{
            assert!(row["native"]["accepted"]==true);assert_eq!(result.unwrap_err(),"Unsupported","outside reader {n}: {}",input["label"]);outside+=1;continue;
        }
        assert_eq!(actual["accepted"],row["native"]["accepted"],"actual acceptance {n}: {}: {actual}",input["label"]);
        if actual["accepted"]==true{assert_eq!(actual,row["native"],"actual scalar value/type {n}: {}",input["label"]);accepted+=1}else{rejected+=1}
    }
    println!("exact original scalar constants {accepted} accepted {rejected} rejected {outside} explicitly outside reader");
}
#[test]
fn original_constant_ownership_and_expression_locator(){
    let corpus:Value=serde_json::from_slice(&std::fs::read(root().join("corpus.json")).unwrap()).unwrap();
    let row=corpus.as_array().unwrap().iter().find(|r|r["input"]["label"]=="edge-\"\\xff\"+\"\\x00\"").unwrap();
    retain("ownership-original-input.json",&row["input"]);
    let graph=scopes(&row["input"]).unwrap();let b=graph.bindings().iter().find(|b|b.name()=="Result").unwrap();
    let value=evaluate_go_constant_binding(&graph,b).unwrap();
    let original=graph.source(b.source_object_id()).unwrap();let spec=&original.nodes()[b.declaration_node()];
    let expr=spec.edges().iter().find(|e|e.role()=="Values").unwrap().targets()[0];
    assert_eq!(evaluate_go_constant(&graph,b.source_object_id(),expr).unwrap(),value);
    assert_eq!(evaluate_go_constant(&graph,"missing",0),Err(GoConstantError::Locator));
    assert_eq!(evaluate_go_constant(&graph,b.source_object_id(),usize::MAX),Err(GoConstantError::Locator));
    drop(graph);drop(corpus);assert_eq!(value.bytes(),Some(b"\xff\0".as_slice()));
    retain("ownership-actual.json",&output(&value));
}
