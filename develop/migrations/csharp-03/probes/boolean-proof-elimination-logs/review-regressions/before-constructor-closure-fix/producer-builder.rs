use mpk_cert::encode::{Certificate, Declaration, DeclarationKind, DefinitionReducibility, LevelNode, TermNode};
use std::collections::{BTreeMap,BTreeSet};
const PREFIX:&str="Mpk.CSharp.Ordinary";
const EQ:&str="Std.Eq";
const REFL:&str="Std.Eq.refl";
#[derive(Debug)] enum OrdinaryCarrierError { Linkage, Limit }
type R<T> = Result<T,OrdinaryCarrierError>;
struct Builder { c:Certificate, terms:BTreeMap<String,u32>, globals:BTreeMap<String,u32>, boolean:u32, sort:u32 }
impl Builder {
 fn from(c:Certificate)->Self {
  let terms=c.term_table.iter().enumerate().map(|(i,t)|(format!("{t:?}"),i as u32)).collect();
  let globals=c.declarations.iter().enumerate().map(|(i,d)|(c.name_table[d.name as usize].clone(),i as u32)).collect::<BTreeMap<_,_>>();
  let DeclarationKind::Inductive{ty:sort}=c.declarations[globals["Std.Bool"]as usize].kind else {panic!("missing Boolean type")};
  let mut b=Self{c,terms,globals,boolean:0,sort};b.boolean=b.constant("Std.Bool").unwrap();b
 }
 fn term(&mut self,n:TermNode)->R<u32>{
  let k=format!("{n:?}");if let Some(&i)=self.terms.get(&k){return Ok(i)}
  if self.c.term_table.len()>=300000{return Err(OrdinaryCarrierError::Limit)}
  let i=self.c.term_table.len()as u32;self.c.term_table.push(n);self.terms.insert(k,i);Ok(i)
 }
 fn constant(&mut self,name:&str)->R<u32>{let global=*self.globals.get(name).ok_or(OrdinaryCarrierError::Linkage)?;self.term(TermNode::Const{global,levels:vec![]})}
 fn app(&mut self,mut function:u32,mut arguments:Vec<u32>)->R<u32>{
  if arguments.is_empty(){return Ok(function)}
  if let TermNode::App{function:f,arguments:a}=&self.c.term_table[function as usize]{let mut all=a.clone();all.append(&mut arguments);function=*f;arguments=all;}
  self.term(TermNode::App{function,arguments})
 }
 fn var(&mut self,i:u32)->R<u32>{self.term(TermNode::Var(i))}
 fn lam(&mut self,ty:u32,body:u32)->R<u32>{self.term(TermNode::Lam{ty,body})}
 fn pi(&mut self,ty:u32,body:u32)->R<u32>{self.term(TermNode::Pi{ty,body})}
 fn cube(&mut self,depth:u32)->R<u32>{let mut t=self.boolean;for _ in 0..depth{t=self.pi(self.boolean,t)?}Ok(t)}
 fn wrap_selectors(&mut self,depth:u32,mut body:u32)->R<u32>{for _ in 0..depth{body=self.lam(self.boolean,body)?}Ok(body)}
 fn define(&mut self,name:&str,ty:u32,value:u32)->R<()>{
  if self.globals.contains_key(name){return Err(OrdinaryCarrierError::Linkage)}
  let id=self.c.name_table.len()as u32;self.c.name_table.push(name.into());self.globals.insert(name.into(),self.c.declarations.len()as u32);
  self.c.declarations.push(Declaration{name:id,kind:DeclarationKind::Def{ty,value,reducibility:DefinitionReducibility::Reducible}});Ok(())
 }
 fn finish(mut self)->Vec<u8>{
  let old=self.c.name_table.clone();self.c.name_table.sort();self.c.name_table.dedup();
  for d in &mut self.c.declarations{d.name=self.c.name_table.binary_search(&old[d.name as usize]).unwrap()as u32}
  for l in &mut self.c.level_table{if let LevelNode::Param(n)=l{*n=self.c.name_table.binary_search(&old[*n as usize]).unwrap()as u32}}
  self.c.export_block=mpk_cert::build_export_block(&self.c).unwrap();self.c.axiom_report=mpk_cert::build_axiom_report(&self.c).unwrap();
  self.c.hashes.export_hash=mpk_cert::export_block_hash(&self.c.export_block);self.c.hashes.axiom_report_hash=mpk_cert::axiom_report_hash_for_report(&self.c.axiom_report);
  mpk_cert::encode::encode_certificate(&self.c)
 }
}
fn call(b:&mut Builder,name:&str,args:Vec<u32>)->R<u32>{let f=b.constant(name)?;b.app(f,args)}
fn bit(b:&mut Builder,yes:bool)->R<u32>{b.constant(if yes{"Std.Bool.true"}else{"Std.Bool.false"})}
