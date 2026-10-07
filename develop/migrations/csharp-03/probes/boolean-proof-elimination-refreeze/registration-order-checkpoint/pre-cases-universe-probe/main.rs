include!("builder.rs");
use mpk_cert::encode::DefinitionReducibility;
use std::{fs,path::Path};
fn main() {
 let args=std::env::args().collect::<Vec<_>>();
 let text=fs::read_to_string(&args[1]).unwrap();
 let text=text.trim();
 let bytes=(0..text.len()).step_by(2).map(|i|u8::from_str_radix(&text[i..i+2],16).unwrap()).collect::<Vec<_>>();
 let original=mpk_cert::decode_canonical_certificate(&bytes).unwrap();
 let out=Path::new(&args[2]);fs::create_dir_all(out).unwrap();
 for before in [true,false] {
  let mut b=Builder::from(original.clone());
  let cases=b.globals["Std.Bool.cases"];
  let insert=if before {cases as usize} else {b.c.declarations.len()};
  for term in &mut b.c.term_table {
   if let TermNode::Const{global,..}=term {if *global>=insert as u32 {*global+=1;}}
  }
  for decl in &mut b.c.declarations {
   match &mut decl.kind {
    DeclarationKind::Constructor{inductive,..}|DeclarationKind::Recursor{inductive,..}=>if *inductive>=insert as u32 {*inductive+=1;},
    _=>{}
   }
  }
  let false_global=b.globals["Std.Bool.false"];
  let name=b.c.name_table.len() as u32;b.c.name_table.push("Probe.Bool.PreCasesUniverseValue".into());
  let value=b.c.term_table.len() as u32;
  b.c.term_table.push(TermNode::Const{global:false_global,levels:vec![0]});
  b.c.declarations.insert(insert,Declaration{name,kind:DeclarationKind::Def{ty:b.boolean,value,reducibility:DefinitionReducibility::Reducible}});
  let b=Builder::from(b.c);
  let data=b.finish();
  let label=if before {"before-cases-universe-argument"}else{"after-cases-universe-argument"};
  fs::write(out.join(format!("{label}.mpcert")),&data).unwrap();
  println!("{label}: {} bytes",data.len());
 }
}
