// Read-only comparison of common definition DAGs, independent of term IDs.
use mpk_cert::encode::{Certificate, DeclarationKind, TermNode};
use std::collections::{BTreeMap, BTreeSet};
fn read(path: &str) -> Certificate {
    let s=std::fs::read_to_string(path).unwrap(); let s=s.trim();
    let bytes=(0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i+2],16).unwrap()).collect::<Vec<_>>();
    mpk_cert::decode_canonical_certificate(&bytes).unwrap()
}
fn terms(c: &Certificate, intern: &mut BTreeMap<String,usize>) -> BTreeMap<String,(usize,usize)> {
    let mut ids=vec![];
    for node in &c.term_table {
        let key=match node {
            TermNode::Sort(l)=>format!("sort:{l}"),
            TermNode::Var(v)=>format!("var:{v}"),
            TermNode::Const{global,levels}=>format!("const:{:?}:{levels:?}",c.name_table[c.declarations[*global as usize].name as usize]),
            TermNode::App{function,arguments}=>format!("app:{}:{:?}",ids[*function as usize],arguments.iter().map(|a|ids[*a as usize]).collect::<Vec<_>>()),
            TermNode::Lam{ty,body}=>format!("lam:{}:{}",ids[*ty as usize],ids[*body as usize]),
            TermNode::Pi{ty,body}=>format!("pi:{}:{}",ids[*ty as usize],ids[*body as usize]),
            TermNode::Let{ty,value,body}=>format!("let:{}:{}:{}",ids[*ty as usize],ids[*value as usize],ids[*body as usize]),
        };
        let next=intern.len(); let id=*intern.entry(key).or_insert(next); ids.push(id);
    }
    c.declarations.iter().filter_map(|d|match d.kind {
        DeclarationKind::Def{ty,value,..}=>Some((c.name_table[d.name as usize].clone(),(ids[ty as usize],ids[value as usize]))),
        _=>None
    }).collect()
}
fn main() {
    let args=std::env::args().skip(1).collect::<Vec<_>>();
    let old=read(&args[0]); let new=read(&args[1]);
    assert_eq!(old.level_table,new.level_table);
    let allowed=args[2..].iter().cloned().collect::<BTreeSet<_>>();
    let mut intern=BTreeMap::new(); let a=terms(&old,&mut intern); let b=terms(&new,&mut intern);
    let mut changed=BTreeSet::new();
    for (name,value) in &a { assert!(b.contains_key(name),"removed definition {name}"); if b[name] != *value { changed.insert(name.clone()); } }
    assert_eq!(changed,allowed,"only selected freeze relation bodies may change");
    println!("{}: {} common definitions unchanged; {} selected freeze definitions changed",args[1],a.len()-changed.len(),changed.len());
}
