//! Closed source-call result/state interfaces. The ordinary laws couple one
//! completion's fields; actual callee execution, safety, effects and source/IR
//! correspondence still require their own proofs. No callee is trusted here.
use super::syntax::*;
use super::{error_layout,source_error_layout,Builder,GoOrdinaryError,Result,MAX_BYTES};
use crate::go_practical_type_keys::{derive_go_source_call_sites,GoSourceCallSite,GoSourceErrorSelection,GoSourceSelectionRef};
use serde_json::{json,Value};
use sha2::{Digest,Sha256};

pub struct GoSourceInvocationCatalogue {sites:Vec<GoSourceCallSite>,builder:Builder,source:String,cases:Vec<String>,original:Vec<u8>,scope:Value}
#[derive(Clone,Debug,Eq,PartialEq)]
pub struct GoSourceInvocationRow {site:GoSourceCallSite,module:String,source:String,binding:String,cases:Vec<String>,original:Vec<u8>,certificate:Vec<u8>}
impl GoSourceInvocationRow {
 pub fn site(&self)->&GoSourceCallSite{&self.site}
 pub fn module_name(&self)->&str{&self.module}
 pub fn source_layout_module_name(&self)->&str{&self.source}
 pub fn source_binding_sha256(&self)->&str{&self.binding}
 pub fn case_type_ids(&self)->&[String]{&self.cases}
 pub fn prior_certificate_bytes(&self)->&[u8]{&self.original}
 pub fn certificate_bytes(&self)->&[u8]{&self.certificate}
}
fn site_value(s:&GoSourceCallSite)->Value{
 json!({"caller":[s.caller().source_object_id(),s.caller().node()],"node":s.node(),"start":s.start_byte(),"end":s.end_byte(),"inputs":s.input_type_ids(),"results":s.result_type_ids(),"arguments":s.arguments().iter().map(|a|json!({"expression":a.expression_node(),"receiver":a.is_receiver(),"values":a.values().iter().map(|v|json!([v.projection(),v.parameter_position(),v.source_type_id(),v.parameter_type_id(),v.is_untyped_nil()])).collect::<Vec<_>>()})).collect::<Vec<_>>(),"targets":s.targets().iter().map(|t|json!([t.dynamic_type_id(),t.callee().source_object_id(),t.callee().node(),t.receiver_type_id(),t.receiver_path()])).collect::<Vec<_>>(),"error_dispatch":s.is_error_dispatch()})
}
/// The original checked source closure determines every site, complete signature
/// and possible target. State remains universally quantified: these equations
/// cannot invent a source state model or discharge a callee's proof demands.
pub fn generate_go_practical_source_invocation_catalogue(unit:&GoSourceErrorSelection)->Result<GoSourceInvocationCatalogue>{
 let domain=unit.domain();let sites=derive_go_source_call_sites(unit).map_err(|_|GoOrdinaryError::Linkage)?;
 let selection=unit.selection();let ids=selection.packages().iter().flat_map(|p|p.source_objects.iter()).collect::<std::collections::BTreeSet<_>>();
 let sources=ids.into_iter().map(|id|selection.source(id).map(|p|json!([p.source().object(),p.source().bytes()])).ok_or(GoOrdinaryError::Linkage)).collect::<Result<Vec<_>>>()?;
 let scope=json!({"sources":sources,"packages":selection.packages(),"roots":selection.roots().iter().map(|r|json!([r.source_object_id(),r.node()])).collect::<Vec<_>>(),"types":domain.types().collect::<Vec<_>>(),"underlying":domain.types().filter_map(|k|domain.underlying(k.id()).map(|u|json!([k.id(),u]))).collect::<Vec<_>>(),"cases":domain.error_cases().iter().map(|c|c.value_type_id()).collect::<Vec<_>>(),"sites":sites.iter().map(site_value).collect::<Vec<_>>()});
 let (builder,source,cases)=source_error_layout::source_type_builder(domain)?;let original=builder.clone().finish()?;
 Ok(GoSourceInvocationCatalogue{sites,builder,source,cases,original,scope})
}
impl GoSourceInvocationCatalogue {
 pub fn sites(&self)->&[GoSourceCallSite]{&self.sites}
 pub fn generate_row(&self,caller:&GoSourceSelectionRef,node:usize)->Result<GoSourceInvocationRow>{
  let site=self.sites.iter().find(|s|s.caller()==caller&&s.node()==node).ok_or(GoOrdinaryError::Linkage)?;
  let mut h=Sha256::new();h.update(b"MPK-PRIVATE-GO-SOURCE-INVOCATION-1\0");h.update(serde_json::to_vec(&(&self.scope,site_value(site))).map_err(|_|GoOrdinaryError::Linkage)?);let binding=format!("{:x}",h.finalize());let module=format!("Mpk.Go.Practical.SourceInvocation.S{binding}");
  let mut builder=self.builder.clone();Invocation{builder:&mut builder,module:&module,source:&self.source,site}.append()?;builder.certificate.module=module.clone();
  Ok(GoSourceInvocationRow{site:site.clone(),module,source:self.source.clone(),binding,cases:self.cases.clone(),original:self.original.clone(),certificate:builder.finish()?})
 }
 pub fn import_row(&self,bytes:&[u8],caller:&GoSourceSelectionRef,node:usize)->Result<GoSourceInvocationRow>{
  if bytes.len()>MAX_BYTES{return Err(GoOrdinaryError::Limit);}let expected=self.generate_row(caller,node)?;if bytes!=expected.certificate_bytes(){return Err(GoOrdinaryError::Linkage);}Ok(expected)
 }
}
struct Invocation<'a>{builder:&'a mut Builder,module:&'a str,source:&'a str,site:&'a GoSourceCallSite}
impl Invocation<'_>{
 fn own(&self,n:&str,args:Vec<Expr>)->Expr{call(&format!("{}.{n}",self.module),args)}
 fn carrier(&self,id:&str)->Expr{constant(&format!("{}.T{id}.Carrier",self.source))}
 fn emit(&mut self,n:&str,args:&[Binder],ty:Expr,body:Expr,law:bool)->Result<()>{emit(self.builder,&format!("{}.{n}",self.module),args,ty,body,law)}
 fn append(&mut self)->Result<()>{
  let input_types=self.site.input_type_ids().iter().map(|id|self.carrier(id)).collect::<Vec<_>>();let result_types=self.site.result_type_ids().iter().map(|id|self.carrier(id)).collect::<Vec<_>>();
  let args_type=error_layout::product(&input_types);let result_type=error_layout::product(&result_types);let state=var("State");
  self.emit("Arguments",&[],Expr::Sort0,args_type.clone(),false)?;self.emit("Results",&[],Expr::Sort0,result_type.clone(),false)?;
  let completion_fields=vec![result_type.clone(),state.clone()];let completion=error_layout::product(&completion_fields);
  self.emit("Completion",&[("State",Expr::Sort0)],Expr::Sort0,completion.clone(),false)?;
  // Complete evaluated inputs, starting state, complete result product and
  // ending state share one unbounded logical invocation identity.
  let fields=vec![nat(),args_type.clone(),state.clone(),result_type.clone(),state.clone()];let joint=error_layout::product(&fields);
  self.emit("Joint",&[("State",Expr::Sort0)],Expr::Sort0,joint.clone(),false)?;
  let make_args=vec![("State",Expr::Sort0),("id",nat()),("arguments",args_type.clone()),("before",state.clone()),("results",result_type.clone()),("post",state.clone())];
  let values=vec![var("id"),var("arguments"),var("before"),var("results"),var("post")];let made=self.own("Make",vec![state.clone(),var("id"),var("arguments"),var("before"),var("results"),var("post")]);
  self.emit("Make",&make_args,joint.clone(),error_layout::make(&fields,&values),false)?;
  let names=["Identifier","CapturedArguments","Before","Returned","Post"];
  for (i,name) in names.iter().enumerate(){
   self.emit(name,&[("State",Expr::Sort0),("joint",joint.clone())],fields[i].clone(),error_layout::get(&fields,var("joint"),i),false)?;
   self.emit(&format!("{name}FromMake"),&make_args,eq(fields[i].clone(),self.own(name,vec![state.clone(),made.clone()]),values[i].clone()),refl(fields[i].clone(),values[i].clone()),true)?;
  }
  let capture_args=vec![("State",Expr::Sort0),("id",nat()),("arguments",args_type.clone()),("before",state.clone()),("completion",completion.clone())];
  let result=error_layout::get(&completion_fields,var("completion"),0);let post=error_layout::get(&completion_fields,var("completion"),1);
  let capture_body=self.own("Make",vec![state.clone(),var("id"),var("arguments"),var("before"),result.clone(),post.clone()]);
  self.emit("CaptureCompletion",&capture_args,joint.clone(),capture_body,false)?;
  let captured=self.own("CaptureCompletion",vec![state.clone(),var("id"),var("arguments"),var("before"),var("completion")]);
  let expected=[var("id"),var("arguments"),var("before"),result.clone(),post.clone()];
  for(i,name)in names.iter().enumerate(){self.emit(&format!("{name}FromCompletion"),&capture_args,eq(fields[i].clone(),self.own(name,vec![state.clone(),captured.clone()]),expected[i].clone()),refl(fields[i].clone(),expected[i].clone()),true)?;}
  let callable=pi("_arguments",args_type.clone(),pi("_before",state.clone(),completion.clone()));
  let call_args=vec![("State",Expr::Sort0),("invoke",callable),("id",nat()),("arguments",args_type.clone()),("before",state.clone())];
  let actual=app(var("invoke"),vec![var("arguments"),var("before")]);
  // One completion binder, one application occurrence. This is an ordinary
  // data equation, not a proof that a Go source call has executed once.
  let captured_once=app(lam("_completion",completion.clone(),self.own("CaptureCompletion",vec![state.clone(),var("id"),var("arguments"),var("before"),var("_completion")])),vec![actual.clone()]);
  self.emit("CaptureCall",&call_args,joint.clone(),captured_once,false)?;
  let call=self.own("CaptureCall",vec![state.clone(),var("invoke"),var("id"),var("arguments"),var("before")]);
  let expected=[var("id"),var("arguments"),var("before"),error_layout::get(&completion_fields,actual.clone(),0),error_layout::get(&completion_fields,actual,1)];
  for(i,name)in names.iter().enumerate(){self.emit(&format!("{name}FromCall"),&call_args,eq(fields[i].clone(),self.own(name,vec![state.clone(),call.clone()]),expected[i].clone()),refl(fields[i].clone(),expected[i].clone()),true)?;}
  // Every result slot projects from the same joint Returned product.
  for(i,ty)in result_types.iter().enumerate(){
   let suffix=format!("Result{i}");
   self.emit(&suffix,&[("State",Expr::Sort0),("joint",joint.clone())],ty.clone(),error_layout::get(&result_types,self.own("Returned",vec![state.clone(),var("joint")]),i),false)?;
   self.emit(&format!("{suffix}FromMake"),&make_args,eq(ty.clone(),self.own(&suffix,vec![state.clone(),made.clone()]),error_layout::get(&result_types,var("results"),i)),refl(ty.clone(),error_layout::get(&result_types,var("results"),i)),true)?;
   self.emit(&format!("{suffix}FromCompletion"),&capture_args,eq(ty.clone(),self.own(&suffix,vec![state.clone(),captured.clone()]),error_layout::get(&result_types,result.clone(),i)),refl(ty.clone(),error_layout::get(&result_types,result.clone(),i)),true)?;
  }
  // Even void calls retain their source argument groups and post-state. Their
  // zero-result source product has a fixed constructor, not an arbitrary flag.
  let pack_names=(0..result_types.len()).map(|i|format!("_result{i}")).collect::<Vec<_>>();
  let pack_values=pack_names.iter().map(|n|var(n)).collect::<Vec<_>>();
  let mut pack_type=result_type.clone();let mut pack_body=error_layout::make(&result_types,&pack_values);
  for (name,ty) in pack_names.iter().zip(&result_types).rev(){pack_type=pi(name,ty.clone(),pack_type);pack_body=lam(name,ty.clone(),pack_body);}
  self.emit("PackResults",&[],pack_type,pack_body,false)?;
  if result_types.is_empty(){self.emit("VoidResults",&[],eq(boolean(),self.own("PackResults",vec![]),bit(false)),refl(boolean(),bit(false)),true)?;}
  for(i,ty)in result_types.iter().enumerate(){
   let mut goal=eq(ty.clone(),error_layout::get(&result_types,self.own("PackResults",pack_values.clone()),i),pack_values[i].clone());let mut proof=refl(ty.clone(),pack_values[i].clone());
   for(name,ty)in pack_names.iter().zip(&result_types).rev(){goal=pi(name,ty.clone(),goal);proof=lam(name,ty.clone(),proof);}
   self.emit(&format!("PackedResult{i}"),&[],goal,proof,true)?;
  }
  Ok(())
 }
}
