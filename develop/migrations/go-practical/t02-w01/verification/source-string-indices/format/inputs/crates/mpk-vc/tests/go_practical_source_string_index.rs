use mpk_vc::go_practical_ordinary::*;
use mpk_vc::go_practical_type_keys::*;
use serde_json::{json, Value};
use std::io::Write;
use std::path::{Path, PathBuf};

fn corpus(name: &str) -> Value {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/go-practical/t02-w01/verification");
    serde_json::from_slice(&std::fs::read(root.join(name).join("corpus.json")).unwrap()).unwrap()
}
fn error_corpus(name: &str) -> Value {
    if name == "source-string-indices" {
        corpus("source-string-indices")
    } else if name == "heterogeneous-payloads" {
        let path=Path::new(env!("CARGO_MANIFEST_DIR")).join("../../develop/migrations/go-practical/t02-w01/verification/source-error-representation/corpus-heterogeneous.json");
        serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
    } else {
        corpus("error-dispatch")
    }
}
fn unhex(value: &Value) -> Vec<u8> {
    value
        .as_str()
        .unwrap()
        .as_bytes()
        .chunks_exact(2)
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect()
}
fn keys(corpus: &Value) -> GoTypeKeyComponents {
    let input = &corpus["input"];
    let sources = input["sources"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| {
            GoTypeSourceBuffer::import(
                &serde_json::to_vec(&s["object"]).unwrap(),
                &unhex(&s["raw_hex"]),
            )
            .unwrap()
        })
        .collect::<Vec<_>>();
    import_go_type_key_components(&unhex(&input["components_hex"]), &sources).unwrap()
}
fn locate(keys: &GoTypeKeyComponents, row: &Value, kind: &str) -> GoSourceSelectionRef {
    let source = row["source"].as_str().unwrap();
    let parsed = keys.source_declarations().source(source).unwrap();
    let node = parsed
        .nodes()
        .iter()
        .position(|n| {
            n.kind() == kind
                && n.start() == row["start"].as_u64().unwrap() as usize
                && n.end() == row["end"].as_u64().unwrap() as usize
        })
        .unwrap();
    GoSourceSelectionRef::new(source, node)
}
fn dispatch(name: &str) -> GoSourceErrorSelection {
    let c = error_corpus(name);
    let keys = keys(&c);
    let row = c["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["input"]["name"] == name)
        .unwrap();
    let input = &row["input"];
    let cases = input["entry_cases"]
        .as_array()
        .map(|cs| {
            cs.iter()
                .map(|n| {
                    locate(
                        &keys,
                        &c["declarations"][format!("example.com/policy.{}", n.as_str().unwrap())],
                        "TypeSpec",
                    )
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let mut roots = vec![];
    let mut entries = vec![];
    for row in input["roots"].as_array().unwrap() {
        let r = locate(&keys, &row["ref"], "FuncDecl");
        let name = row["name"].as_str().unwrap();
        roots.push(r.clone());
        if matches!(
            name,
            "Entry" | "Expr" | "Paren" | "Guarded" | "Branch" | "Imported" | "OtherRoot"
        ) {
            entries.push(GoErrorEntryConstraint::new(
                r.clone(),
                0,
                vec![],
                if name == "OtherRoot" {
                    vec![]
                } else {
                    cases.clone()
                },
            ));
        }
        if name == "Nested" {
            entries.push(GoErrorEntryConstraint::new(
                r.clone(),
                0,
                vec![GoErrorEntryStep::Field(0)],
                cases.clone(),
            ));
            entries.push(GoErrorEntryConstraint::new(
                r,
                0,
                vec![GoErrorEntryStep::Field(1), GoErrorEntryStep::Elements],
                vec![],
            ));
        }
    }
    derive_go_source_error_selection(&keys, &roots, &entries).unwrap()
}
fn write(root: &Path, name: &str, raw: &[u8]) {
    std::fs::create_dir_all(root).unwrap();
    std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(root.join(name))
        .unwrap()
        .write_all(raw)
        .unwrap();
}
#[path = "support/go_word_division_dependencies.rs"]
mod dependencies;

fn check(name:&str){
 let selection=dispatch(name);let c=generate_go_practical_source_string_index_catalogue(selection.domain()).unwrap();
 assert_eq!(c.strings().source_types(),selection.domain().types().cloned().collect::<Vec<_>>());
 if name=="source-string-indices" {assert_eq!(c.index_types().len(),14);assert_eq!(c.strings().source_string_type_ids().len(),4);}
 let out=std::env::var_os("MPK_GO_SOURCE_STRING_INDEX_OUTPUT").map(PathBuf::from);
 for string in c.strings().source_string_type_ids(){
  let prior=c.strings().generate_row(string).unwrap();let before=dependencies::declarations(&mpk_cert::decode_canonical_certificate(prior.certificate_bytes()).unwrap());
  for (index,width,signed) in c.index_types(){
   let row=c.generate_row(string,index).unwrap();assert_eq!(row.index_width(),*width);assert_eq!(row.is_signed_index(),*signed);
   let decoded=mpk_cert::decode_canonical_certificate(row.certificate_bytes()).unwrap();let after=dependencies::declarations(&decoded);
   for (n,v) in &before {assert_eq!(after.get(n),Some(v),"all prior source string identities: {n}");}
   assert_eq!(c.import_row(row.certificate_bytes(),string,index).unwrap(),row);
   assert_eq!(c.import_row(row.certificate_bytes(),string,"wrong-index"),Err(GoOrdinaryError::Linkage));
   if let Some((other,_,_))=c.index_types().iter().find(|r|r.0!=*index){assert_eq!(c.import_row(row.certificate_bytes(),string,other),Err(GoOrdinaryError::Linkage));}
   let file=format!("{name}-{string}-{index}");
   if let Some(root)=&out{
    write(root,&format!("{file}.mpcert"),row.certificate_bytes());
    write(root,&format!("{file}.sources.json"),&serde_json::to_vec_pretty(&json!({"unit":name,"module":row.module_name(),"string_module":row.prior_string_row().module_name(),"source_layout_module":row.prior_string_row().source_layout_module_name(),"source_string_type_id":string,"source_index_type_id":index,"width":width,"signed":signed,"source_byte_type_id":row.source_byte_type_id(),"cases":c.strings().case_type_ids(),"types":c.strings().source_types(),"underlying":c.strings().source_types().iter().filter_map(|t|c.strings().underlying_type_id(t.id()).map(|u|json!([t.id(),u]))).collect::<Vec<_>>(),"declarations":decoded.declarations.len(),"terms":decoded.term_table.len(),"bytes":row.certificate_bytes().len(),"all_prior_source_string_identities_unchanged":true,"source_expression_or_context_authority":false})).unwrap());
   }
   let start=std::time::Instant::now();let r=mpk_kernel::verify_certificate_bytes_json_output(row.certificate_bytes());println!("{file}: {:?} {}",start.elapsed(),r.accepted);
   if let Some(root)=&out{write(root,&format!("{file}.rust.json"),format!("{}\n",r.json).as_bytes());}
   assert!(r.accepted,"{}",r.json);let report:Value=serde_json::from_str(&r.json).unwrap();assert_eq!(report["axiom_count"],0);
  }
 }
}
#[test] fn source_string_indices_heterogeneous(){check("heterogeneous-payloads")}
#[test] fn source_string_indices_large(){check("seventy-cases")}
#[test] fn source_string_indices_nominal(){check("source-string-indices")}
