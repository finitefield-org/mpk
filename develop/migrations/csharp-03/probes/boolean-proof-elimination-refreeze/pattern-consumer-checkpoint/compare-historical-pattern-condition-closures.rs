use serde_json::{json,Value};
use std::{collections::BTreeMap,fs,path::Path};
fn declaration_bodies(c: &mpk_cert::encode::Certificate) -> BTreeMap<String, Value> {
    use mpk_cert::encode::{DeclarationKind as D, LevelNode as L, TermNode as T};
    use sha2::{Digest, Sha256};
    let hash =
        |value: Value| -> Vec<u8> { Sha256::digest(serde_json::to_vec(&value).unwrap()).to_vec() };
    let global = |id: u32| c.name_table[c.declarations[id as usize].name as usize].clone();
    let mut levels: Vec<Vec<u8>> = vec![];
    for level in &c.level_table {
        levels.push(hash(match level {
            L::Zero => json!(["zero"]),
            L::Succ(a) => json!(["succ", levels[*a as usize]]),
            L::Max(a, b) => json!(["max", levels[*a as usize], levels[*b as usize]]),
            L::Param(n) => json!(["param", c.name_table[*n as usize]]),
        }));
    }
    let mut terms: Vec<Vec<u8>> = vec![];
    for term in &c.term_table {
        terms.push(hash(match term {
            T::Sort(l) => json!(["sort", levels[*l as usize]]),
            T::Var(v) => json!(["var", v]),
            T::Const {
                global: g,
                levels: ls,
            } => json!([
                "const",
                global(*g),
                ls.iter().map(|l| &levels[*l as usize]).collect::<Vec<_>>()
            ]),
            T::App {
                function,
                arguments,
            } => json!([
                "app",
                terms[*function as usize],
                arguments
                    .iter()
                    .map(|a| &terms[*a as usize])
                    .collect::<Vec<_>>()
            ]),
            T::Lam { ty, body } => json!(["lam", terms[*ty as usize], terms[*body as usize]]),
            T::Pi { ty, body } => json!(["pi", terms[*ty as usize], terms[*body as usize]]),
            T::Let { ty, value, body } => json!([
                "let",
                terms[*ty as usize],
                terms[*value as usize],
                terms[*body as usize]
            ]),
        }));
    }
    c.declarations
        .iter()
        .map(|d| {
            let body = match &d.kind {
                D::Axiom { ty } => json!(["axiom", terms[*ty as usize]]),
                D::Def {
                    ty,
                    value,
                    reducibility,
                } => json!([
                    "def",
                    terms[*ty as usize],
                    terms[*value as usize],
                    format!("{reducibility:?}")
                ]),
                D::Theorem { ty, proof } => {
                    json!(["theorem", terms[*ty as usize], terms[*proof as usize]])
                }
                D::Inductive { ty } => json!(["inductive", terms[*ty as usize]]),
                D::Constructor {
                    ty,
                    inductive,
                    generated,
                } => json!([
                    "constructor",
                    terms[*ty as usize],
                    global(*inductive),
                    generated
                ]),
                D::Recursor {
                    ty,
                    inductive,
                    generated,
                } => json!([
                    "recursor",
                    terms[*ty as usize],
                    global(*inductive),
                    generated
                ]),
                D::TheoryPrimitive { ty } => json!(["theory", terms[*ty as usize]]),
            };
            (c.name_table[d.name as usize].clone(), body)
        })
        .collect()
}

fn main(){
 let args=std::env::args().collect::<Vec<_>>();assert_eq!(args.len(),4);let mapping:BTreeMap<String,String>=serde_json::from_slice(&fs::read(&args[1]).unwrap()).unwrap();let root=Path::new(&args[2]);let fresh=Path::new(&args[3]);let mut rows=vec![];
 for entry in fs::read_dir(root).unwrap(){let path=entry.unwrap().path();if path.extension().and_then(|x|x.to_str())!=Some("hex"){continue;}let name=path.file_name().unwrap();let decode=|p:&Path|{let text=fs::read_to_string(p).unwrap();let data=text.trim().as_bytes().chunks_exact(2).map(|p|u8::from_str_radix(std::str::from_utf8(p).unwrap(),16).unwrap()).collect::<Vec<_>>();mpk_cert::decode_canonical_certificate(&data).unwrap()};let mut old=decode(&path);let current=decode(&fresh.join(name));assert!(old.imports.is_empty() && old.proof_node_table.is_empty() && old.theory_certificates.is_empty() && old.axiom_report.entries.is_empty());
  for name in &mut old.name_table {for (a,c) in &mapping {if name.contains(a){*name=name.replace(a,c);}}}
  let prior=declaration_bodies(&old);let actual=declaration_bodies(&current);for (name,body) in &prior {assert_eq!(actual.get(name),Some(body),"{}: historical typed definition {}",path.display(),name);}
  rows.push(json!({"source_case":path.file_stem().unwrap().to_str().unwrap(),"all_original_typed_definition_bodies_preserved_after_actual_identity_name_rebinding":true,"original_declarations":old.declarations.len(),"current_declarations":current.declarations.len()}));
 }
 assert_eq!(rows.len(),18);println!("{}",serde_json::to_string_pretty(&json!({"status":"passed_all_18_complete_historical_condition_definition_closures_after_actual_name_rebinding","sources":rows,"application_proof_ids_pending":987})).unwrap());
}
