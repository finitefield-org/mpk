//! Independent ordinary-term consumer: every heterogeneous arity and field frame.
use super::controls::{baseline, encode, global};
use super::inspect;
use mpk_cert::encode::{Certificate, Declaration, DeclarationKind, LevelNode, TermNode};
use std::collections::BTreeMap;

#[derive(Clone)]
enum E {
    Sort,
    V(String),
    C(String),
    App(Box<E>, Vec<E>),
    Lam(String, Box<E>, Box<E>),
    Pi(String, Box<E>, Box<E>),
}
fn v(name: &str) -> E { E::V(name.into()) }
fn c(name: &str) -> E { E::C(name.into()) }
fn app(f: E, args: Vec<E>) -> E { E::App(Box::new(f),args) }
fn pc(name: &str, args: Vec<E>) -> E { app(c(&format!("Mpk.Go.Practical.Product.{name}")),args) }
fn pi(name:&str,ty:E,body:E)->E { E::Pi(name.into(),Box::new(ty),Box::new(body)) }
fn rec_type(a:E,b:E,tag:E)->E {app(c("Std.Bool.recType"),vec![a,b,tag])}
fn pair(a:E,b:E)->E {pi("_tag",c("Std.Bool"),rec_type(a,b,v("_tag")))}
fn product(parts:&[E])->E {
    match parts {
        []=>pc("Unit",vec![]),
        [one]=>one.clone(),
        _=>{let mid=parts.len()/2;pair(product(&parts[..mid]),product(&parts[mid..]))}
    }
}
fn pack(parts:&[E],values:&[E])->E {
    assert_eq!(parts.len(),values.len());
    match parts {
        []=>pc("UnitZero",vec![]),
        [_]=>values[0].clone(),
        _=>{let mid=parts.len()/2;pc("Make",vec![product(&parts[..mid]),product(&parts[mid..]),pack(&parts[..mid],&values[..mid]),pack(&parts[mid..],&values[mid..])])}
    }
}
fn project(parts:&[E],value:E,index:usize)->E {
    assert!(index<parts.len());
    if parts.len()==1{return value}
    let mid=parts.len()/2;
    let right=index>=mid;
    let value=pc(if right{"Right"}else{"Left"},vec![product(&parts[..mid]),product(&parts[mid..]),value]);
    if right {project(&parts[mid..],value,index-mid)}else{project(&parts[..mid],value,index)}
}
fn replace(parts:&[E],value:E,index:usize,next:E)->E {
    assert!(index<parts.len());
    if parts.len()==1{return next}
    let mid=parts.len()/2;
    let a=product(&parts[..mid]);let b=product(&parts[mid..]);
    let left=pc("Left",vec![a.clone(),b.clone(),value.clone()]);
    let right=pc("Right",vec![a.clone(),b.clone(),value]);
    if index<mid {pc("Make",vec![a,b,replace(&parts[..mid],left,index,next),right])}
    else {pc("Make",vec![a,b,left,replace(&parts[mid..],right,index-mid,next)])}
}
fn equality(ty:E,left:E,right:E)->E {app(c("Std.Eq"),vec![ty,left,right])}
fn reflexivity(ty:E,value:E)->E {app(c("Std.Eq.refl"),vec![ty,value])}
#[derive(Clone,PartialEq,Eq,PartialOrd,Ord)]
enum Key {
    Sort(u32),Var(u32),Const(u32,Vec<u32>),App(u32,Vec<u32>),Lam(u32,u32),Pi(u32,u32),Let(u32,u32,u32),
}
fn key(node:&TermNode)->Key {
    match node {
        TermNode::Sort(i)=>Key::Sort(*i),TermNode::Var(i)=>Key::Var(*i),
        TermNode::Const{global,levels}=>Key::Const(*global,levels.clone()),
        TermNode::App{function,arguments}=>Key::App(*function,arguments.clone()),
        TermNode::Lam{ty,body}=>Key::Lam(*ty,*body),TermNode::Pi{ty,body}=>Key::Pi(*ty,*body),
        TermNode::Let{ty,value,body}=>Key::Let(*ty,*value,*body),
    }
}
struct Consumer { certificate:Certificate, terms:BTreeMap<Key,u32>, zero:u32 }
impl Consumer {
    fn new()->Self {
        let certificate=baseline();
        let terms=certificate.term_table.iter().enumerate().map(|(i,n)|(key(n),i as u32)).collect();
        let zero=certificate.level_table.iter().position(|l|*l==LevelNode::Zero).unwrap() as u32;
        Self{certificate,terms,zero}
    }
    fn term(&mut self,node:TermNode)->u32 {
        let k=key(&node);
        if let Some(index)=self.terms.get(&k){return *index}
        let index=self.certificate.term_table.len() as u32;
        self.certificate.term_table.push(node);self.terms.insert(k,index);index
    }
    fn lower(&mut self,e:&E,scope:&mut Vec<String>)->u32 {
        let node=match e {
            E::Sort=>TermNode::Sort(self.zero),
            E::V(name)=>TermNode::Var(scope.iter().rev().position(|n|n==name).expect("closed consumer scope") as u32),
            E::C(name)=>TermNode::Const{global:global(&self.certificate,name) as u32,levels:vec![]},
            E::App(f,args)=>TermNode::App{function:self.lower(f,scope),arguments:args.iter().map(|a|self.lower(a,scope)).collect()},
            E::Lam(name,ty,body)|E::Pi(name,ty,body)=>{
                let ty=self.lower(ty,scope);scope.push(name.clone());let body=self.lower(body,scope);scope.pop();
                if matches!(e,E::Lam(..)){TermNode::Lam{ty,body}}else{TermNode::Pi{ty,body}}
            }
        };
        self.term(node)
    }
    fn theorem(&mut self,name:&str,args:&[(String,E)],mut ty:E,mut proof:E){
        for (name,domain) in args.iter().rev(){
            ty=pi(name,domain.clone(),ty);proof=E::Lam(name.clone(),Box::new(domain.clone()),Box::new(proof));
        }
        let ty=self.lower(&ty,&mut vec![]);let proof=self.lower(&proof,&mut vec![]);
        let index=self.certificate.name_table.len() as u32;
        self.certificate.name_table.push(format!("Mpk.Go.Practical.Test.Product.{name}"));
        self.certificate.declarations.push(Declaration{name:index,kind:DeclarationKind::Theorem{ty,proof}});
    }
    fn observations(&mut self,name:&str,args:&[(String,E)],types:&[E],actual:&[E],expected:&[E]){
        assert!(!types.is_empty() && types.len()==actual.len() && types.len()==expected.len());
        let claims:Vec<_>=types.iter().zip(actual).zip(expected).map(|((t,a),b)|equality(t.clone(),a.clone(),b.clone())).collect();
        let proofs:Vec<_>=types.iter().zip(expected).map(|(t,b)|reflexivity(t.clone(),b.clone())).collect();
        self.theorem(name,args,product(&claims),pack(&claims,&proofs));
    }
}
pub(super) fn all_balanced_shapes(){
    let mut pack_observations=0usize;let mut reconstructed_observations=0usize;
    let mut updated_fields=0usize;let mut unchanged_fields=0usize;
    for arity in 0..=64 {
        let mut consumer=Consumer::new();
        let mut args:Vec<_>=(0..arity).map(|i|(format!("A{i}"),E::Sort)).collect();
        let types:Vec<_>=(0..arity).map(|i|v(&format!("A{i}"))).collect();
        if arity==0 {
            consumer.theorem("Pack0",&[],pc("UnitValid",vec![pc("UnitZero",vec![])]),pc("UnitZeroValid",vec![]));
            consumer.theorem("Reconstruct0",&[("p".into(),pc("Unit",vec![])),("valid".into(),pc("UnitValid",vec![v("p")]))],
                equality(c("Std.Bool"),v("p"),pc("UnitZero",vec![])),pc("UnitValidUnique",vec![v("p"),v("valid")]));
        } else {
            let mut values=args.clone();
            values.extend(types.iter().enumerate().map(|(i,t)|(format!("x{i}"),t.clone())));
            let expected:Vec<_>=(0..arity).map(|i|v(&format!("x{i}"))).collect();
            let made=pack(&types,&expected);
            let actual:Vec<_>=(0..arity).map(|i|project(&types,made.clone(),i)).collect();
            consumer.observations(&format!("Pack{arity}"),&values,&types,&actual,&expected);
            pack_observations+=arity;
            args.push(("p".into(),product(&types)));
            let expected:Vec<_>=(0..arity).map(|i|project(&types,v("p"),i)).collect();
            let rebuilt=pack(&types,&expected);
            let actual:Vec<_>=(0..arity).map(|i|project(&types,rebuilt.clone(),i)).collect();
            consumer.observations(&format!("Reconstruct{arity}"),&args,&types,&actual,&expected);
            reconstructed_observations+=arity;
            for index in 0..arity {
                let mut update_args=args.clone();update_args.push(("replacement".into(),types[index].clone()));
                let changed=replace(&types,v("p"),index,v("replacement"));
                let actual:Vec<_>=(0..arity).map(|i|project(&types,changed.clone(),i)).collect();
                let mut expected=expected.clone();expected[index]=v("replacement");
                consumer.observations(&format!("Frame{arity}Target{index}"),&update_args,&types,&actual,&expected);
                updated_fields+=1;unchanged_fields+=arity-1;
            }
        }
        let c=&consumer.certificate;
        assert!(c.declarations.len()<=4096 && c.term_table.len()<=131_072);
        let metrics=(c.declarations.len(),c.term_table.len());
        let bytes=encode(consumer.certificate);
        println!("arity={arity} declarations={} terms={} bytes={}",metrics.0,metrics.1,bytes.len());
        inspect(&format!("balanced-shape-{arity:02}"),&bytes,true);
    }
    assert_eq!((pack_observations,reconstructed_observations,updated_fields,unchanged_fields),(2080,2080,2080,87360));
    println!("all65 arities;2080 pack +2080 reconstructed +2080 updated +87360 unchanged field obligations; arbitrary independent Sort0 types; all0..16 result arities and0..64 field counts");
}
