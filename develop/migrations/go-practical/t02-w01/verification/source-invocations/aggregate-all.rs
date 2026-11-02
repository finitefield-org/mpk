// Untrusted, exact rebundling of every complete original ordinary declaration.
include!("/Users/kazuyoshitoshiya/ff/mpk/crates/mpk-vc/tests/go_practical_source_invocation.rs");
use mpk_cert::encode::{Certificate, Declaration, DeclarationKind, LevelNode, TermNode};
use std::collections::{BTreeMap, HashMap};

fn intern<T: std::fmt::Debug>(table: &mut Vec<T>, index: &mut HashMap<String,u32>, node: T) -> u32 {
    let key=format!("{node:?}");
    if let Some(&i)=index.get(&key){return i;}
    let id=table.len() as u32;table.push(node);index.insert(key,id);id
}
fn all_bytes(mut c: Certificate) -> Vec<u8> {
    let old=c.name_table.clone();c.name_table.sort();
    for d in &mut c.declarations{d.name=c.name_table.binary_search(&old[d.name as usize]).unwrap() as u32;}
    for level in &mut c.level_table{if let LevelNode::Param(n)=level{*n=c.name_table.binary_search(&old[*n as usize]).unwrap() as u32;}}
    c.export_block=mpk_cert::build_export_block(&c).unwrap();c.axiom_report=mpk_cert::build_axiom_report(&c).unwrap();
    c.hashes.export_hash=mpk_cert::export_block_hash(&c.export_block);c.hashes.axiom_report_hash=mpk_cert::axiom_report_hash_for_report(&c.axiom_report);
    assert!(c.imports.is_empty() && c.proof_node_table.is_empty() && c.theory_certificates.is_empty());
    assert_eq!(c.axiom_report.summary.total_axiom_count,0);
    mpk_cert::encode_certificate(&c)
}
fn main(){
    for unit in ["pure","wide"]{
        let source=root().join("generation-linked/certificates");let inventory:Value=serde_json::from_slice(&std::fs::read(source.join(format!("{unit}-inventory.json"))).unwrap()).unwrap();
        let mut out:Option<Certificate>=None;let mut expected=BTreeMap::new();let mut components=vec![];
        for i in 0..inventory["sites"].as_array().unwrap().len(){
            let key=format!("{unit}-{i:03}");let bytes=std::fs::read(source.join(format!("{key}.mpcert"))).unwrap();let c=mpk_cert::decode_canonical_certificate(&bytes).unwrap();
            assert!(c.imports.is_empty() && c.proof_node_table.is_empty() && c.theory_certificates.is_empty());
            for (name,value) in dependencies::declarations(&c){if let Some(prior)=expected.insert(name,value.clone()){assert_eq!(prior,value);}}
            components.push(json!({"name":key,"module":c.module,"bytes":bytes.len(),"sha256":format!("{:x}",sha2::Sha256::digest(&bytes))}));
            if out.is_none(){let mut empty=c.clone();empty.name_table.clear();empty.level_table.clear();empty.term_table.clear();empty.declarations.clear();out=Some(empty);}
            let target=out.as_mut().unwrap();let mut names=target.name_table.iter().enumerate().map(|(i,n)|(n.clone(),i as u32)).collect::<BTreeMap<_,_>>();
            let nm=c.name_table.iter().map(|name|{if let Some(&id)=names.get(name){id}else{let id=target.name_table.len() as u32;target.name_table.push(name.clone());names.insert(name.clone(),id);id}}).collect::<Vec<_>>();
            let mut globals=target.declarations.iter().enumerate().map(|(i,d)|(target.name_table[d.name as usize].clone(),i as u32)).collect::<BTreeMap<_,_>>();
            let mut added=0;let gm=c.declarations.iter().map(|d|{let name=&c.name_table[d.name as usize];if let Some(&id)=globals.get(name){id}else{let id=target.declarations.len() as u32+added;added+=1;globals.insert(name.clone(),id);id}}).collect::<Vec<_>>();
            let mut li=target.level_table.iter().enumerate().map(|(i,n)|(format!("{n:?}"),i as u32)).collect::<HashMap<_,_>>();let mut lm=vec![];
            for l in &c.level_table{let n=match l{LevelNode::Zero=>LevelNode::Zero,LevelNode::Succ(a)=>LevelNode::Succ(lm[*a as usize]),LevelNode::Max(a,b)=>LevelNode::Max(lm[*a as usize],lm[*b as usize]),LevelNode::Param(n)=>LevelNode::Param(nm[*n as usize])};lm.push(intern(&mut target.level_table,&mut li,n));}
            let mut ti=target.term_table.iter().enumerate().map(|(i,n)|(format!("{n:?}"),i as u32)).collect::<HashMap<_,_>>();let mut tm=vec![];
            for t in &c.term_table{let n=match t{
                TermNode::Sort(l)=>TermNode::Sort(lm[*l as usize]),TermNode::Var(i)=>TermNode::Var(*i),
                TermNode::Const{global,levels}=>TermNode::Const{global:gm[*global as usize],levels:levels.iter().map(|i|lm[*i as usize]).collect()},
                TermNode::App{function,arguments}=>TermNode::App{function:tm[*function as usize],arguments:arguments.iter().map(|i|tm[*i as usize]).collect()},
                TermNode::Lam{ty,body}=>TermNode::Lam{ty:tm[*ty as usize],body:tm[*body as usize]},TermNode::Pi{ty,body}=>TermNode::Pi{ty:tm[*ty as usize],body:tm[*body as usize]},
                TermNode::Let{ty,value,body}=>TermNode::Let{ty:tm[*ty as usize],value:tm[*value as usize],body:tm[*body as usize]},
            };tm.push(intern(&mut target.term_table,&mut ti,n));}
            for (old,d) in c.declarations.iter().enumerate(){if gm[old]<(target.declarations.len() as u32){continue;}
                assert_eq!(gm[old],target.declarations.len() as u32);
                let kind=match &d.kind{
                    DeclarationKind::Def{ty,value,reducibility}=>DeclarationKind::Def{ty:tm[*ty as usize],value:tm[*value as usize],reducibility:*reducibility},
                    DeclarationKind::Theorem{ty,proof}=>DeclarationKind::Theorem{ty:tm[*ty as usize],proof:tm[*proof as usize]},
                    DeclarationKind::Inductive{ty}=>DeclarationKind::Inductive{ty:tm[*ty as usize]},
                    DeclarationKind::Constructor{ty,inductive,generated}=>DeclarationKind::Constructor{ty:tm[*ty as usize],inductive:gm[*inductive as usize],generated:*generated},
                    DeclarationKind::Recursor{ty,inductive,generated}=>DeclarationKind::Recursor{ty:tm[*ty as usize],inductive:gm[*inductive as usize],generated:*generated},
                    _=>panic!("nonordinary original declaration"),
                };target.declarations.push(Declaration{name:nm[d.name as usize],kind});
            }
        }
        let mut c=out.unwrap();assert_eq!(dependencies::declarations(&c),expected);
        use sha2::Digest;let binding=format!("{:x}",sha2::Sha256::digest(serde_json::to_vec(&components).unwrap()));c.module=format!("Mpk.Go.Practical.SourceInvocationCatalogue.S{binding}");
        let declarations=c.declarations.len();let terms=c.term_table.len();let raw=all_bytes(c);write(&format!("{unit}-all.mpcert"),&raw);
        let meta=json!({"unit":unit,"components":components,"declarations":declarations,"terms":terms,"bytes":raw.len(),"complete_named_original_declaration_union_equal":true,"all_source_types_and_U_retained":true,"new_mathematical_axioms":false,"kernel_checked":false});
        write(&format!("{unit}-all.sources.json"),&serde_json::to_vec_pretty(&meta).unwrap());println!("{meta}");
        assert!(declarations<=4096 && terms<=131072 && raw.len()<=2097152,"complete original union exceeds unchanged core caps; entire candidate retained");
    }
}
