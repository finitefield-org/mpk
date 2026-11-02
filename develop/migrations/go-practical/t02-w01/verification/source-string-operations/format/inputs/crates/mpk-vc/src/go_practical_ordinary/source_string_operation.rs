//! Actual-source len/index instances with one coupled operand product.
//! Captured operands still need their own source execution and callee proofs.
use super::syntax::*;
use super::{composition,error_layout,word,Builder,GoOrdinaryError,Result,MAX_BYTES};
use super::{generate_go_practical_source_string_length_catalogue,generate_go_practical_source_string_index_catalogue,GoSourceStringLengthCatalogue,GoSourceStringIndexCatalogue};
use crate::go_practical_type_keys::{derive_go_source_string_operations,GoSourceStringOperation,GoSourceStringOperationKind,GoSourceSelectionRef,GoSourceValueDomain};
use serde_json::{json,Value};
use sha2::{Digest,Sha256};

pub struct GoSourceStringOperationCatalogue {
    sites:Vec<GoSourceStringOperation>,
    lengths:GoSourceStringLengthCatalogue,
    indices:GoSourceStringIndexCatalogue,
    scope:Value,
}
#[derive(Clone,Debug,Eq,PartialEq)]
pub struct GoSourceStringOperationRow {
    site:GoSourceStringOperation,
    module:String,
    prior:String,
    source:String,
    binding:String,
    original:Vec<u8>,
    certificate:Vec<u8>,
}
impl GoSourceStringOperationRow {
    pub fn site(&self)->&GoSourceStringOperation {&self.site}
    pub fn module_name(&self)->&str {&self.module}
    pub fn prior_module_name(&self)->&str {&self.prior}
    pub fn source_layout_module_name(&self)->&str {&self.source}
    pub fn source_binding_sha256(&self)->&str {&self.binding}
    pub fn prior_certificate_bytes(&self)->&[u8] {&self.original}
    pub fn certificate_bytes(&self)->&[u8] {&self.certificate}
}
fn site_value(s:&GoSourceStringOperation)->Value {
    json!({"caller":[s.caller().source_object_id(),s.caller().node()],"node":s.node(),"start":s.start_byte(),"end":s.end_byte(),"kind":match s.kind(){GoSourceStringOperationKind::Length=>json!("string_length"),GoSourceStringOperationKind::Index=>json!("string_index"),GoSourceStringOperationKind::ConstantLength(n)=>json!(["constant_string_length",n])},"operands":s.operand_nodes(),"operand_types":s.operand_type_ids(),"result":s.result_type_id(),"calls":s.operand_calls().iter().map(|c|json!([c.caller().source_object_id(),c.caller().node(),c.call_node(),c.callee().source_object_id(),c.callee().node()])).collect::<Vec<_>>()})
}
/// The original selected bodies determine all sites and type-specialization
/// roots. Constant lengths remain listed but do not acquire a runtime recipe.
/// No native observation, caller opcode/type/U or custom adapter is accepted.
pub fn generate_go_practical_source_string_operation_catalogue(domain:&GoSourceValueDomain)->Result<GoSourceStringOperationCatalogue> {
    let sites=derive_go_source_string_operations(domain).map_err(|_|GoOrdinaryError::Linkage)?;
    let selection=domain.selection();
    let source_ids=selection.packages().iter().flat_map(|p|p.source_objects.iter()).collect::<std::collections::BTreeSet<_>>();
    let originals=source_ids.into_iter().map(|id|selection.source(id).map(|p|json!([p.source().object(),p.source().bytes()])).ok_or(GoOrdinaryError::Linkage)).collect::<Result<Vec<_>>>()?;
    let scope=json!({"sources":originals,"packages":selection.packages(),"roots":selection.roots().iter().map(|r|json!([r.source_object_id(),r.node()])).collect::<Vec<_>>(),"types":domain.types().collect::<Vec<_>>(),"underlying":domain.types().filter_map(|k|domain.underlying(k.id()).map(|u|json!([k.id(),u]))).collect::<Vec<_>>(),"cases":domain.error_cases().iter().map(|c|c.value_type_id()).collect::<Vec<_>>(),"sites":sites.iter().map(site_value).collect::<Vec<_>>()});
    Ok(GoSourceStringOperationCatalogue {sites,lengths:generate_go_practical_source_string_length_catalogue(domain)?,indices:generate_go_practical_source_string_index_catalogue(domain)?,scope})
}
impl GoSourceStringOperationCatalogue {
    pub fn sites(&self)->&[GoSourceStringOperation] {&self.sites}
    pub fn generate_row(&self,caller:&GoSourceSelectionRef,node:usize)->Result<GoSourceStringOperationRow> {
        let site=self.sites.iter().find(|s|s.caller()==caller && s.node()==node).ok_or(GoOrdinaryError::Linkage)?;
        let string=site.operand_type_ids().first().and_then(Option::as_deref).ok_or(GoOrdinaryError::Foundation)?;
        let (bytes,prior,source,result)=match site.kind() {
            GoSourceStringOperationKind::Length=>{
                let r=self.lengths.generate_row(string)?;
                (r.certificate_bytes().to_vec(),r.module_name().to_string(),r.prior_string_row().source_layout_module_name().to_string(),r.source_result_type_id().to_string())
            }
            GoSourceStringOperationKind::Index=>{
                let index=site.operand_type_ids().get(1).and_then(Option::as_deref).ok_or(GoOrdinaryError::Linkage)?;
                let r=self.indices.generate_row(string,index)?;
                (r.certificate_bytes().to_vec(),r.module_name().to_string(),r.prior_string_row().source_layout_module_name().to_string(),r.source_byte_type_id().to_string())
            }
            GoSourceStringOperationKind::ConstantLength(_)=>return Err(GoOrdinaryError::Foundation),
        };
        if site.result_type_id()!=Some(result.as_str()) {return Err(GoOrdinaryError::Linkage)}
        let mut hash=Sha256::new();hash.update(b"MPK-PRIVATE-GO-SOURCE-STRING-OPERATION-1\0");
        hash.update(serde_json::to_vec(&(&self.scope,site_value(site))).map_err(|_|GoOrdinaryError::Linkage)?);
        let binding=format!("{:x}",hash.finalize());let module=format!("Mpk.Go.Practical.SourceStringOperation.S{binding}");
        let original=mpk_cert::decode_canonical_certificate(&bytes).map_err(|_|GoOrdinaryError::Foundation)?;
        let mut builder=composition::extend_complete_source_row(original)?;
        Operation {builder:&mut builder,site,module:&module,prior:&prior,source:&source,result:&result}.append()?;
        builder.certificate.module=module.clone();
        Ok(GoSourceStringOperationRow {site:site.clone(),module,prior,source,binding,original:bytes,certificate:builder.finish()?})
    }
    pub fn import_row(&self,bytes:&[u8],caller:&GoSourceSelectionRef,node:usize)->Result<GoSourceStringOperationRow> {
        if bytes.len()>MAX_BYTES {return Err(GoOrdinaryError::Limit)}
        let expected=self.generate_row(caller,node)?;
        if bytes!=expected.certificate_bytes() {return Err(GoOrdinaryError::Linkage)}
        Ok(expected)
    }
}
struct Operation<'a> {builder:&'a mut Builder,site:&'a GoSourceStringOperation,module:&'a str,prior:&'a str,source:&'a str,result:&'a str}
impl Operation<'_> {
    fn own(&self,n:&str,args:Vec<Expr>)->Expr {call(&format!("{}.{n}",self.module),args)}
    fn prior(&self,n:&str,args:Vec<Expr>)->Expr {call(&format!("{}.{n}",self.prior),args)}
    fn carrier(&self,id:&str)->Expr {constant(&format!("{}.T{id}.Carrier",self.source))}
    fn emit(&mut self,n:&str,args:&[Binder],ty:Expr,body:Expr,law:bool)->Result<()> {emit(self.builder,&format!("{}.{n}",self.module),args,ty,body,law)}
    fn append(&mut self)->Result<()> {
        let types=self.site.operand_type_ids().iter().map(|i|i.as_deref().map(|i|self.carrier(i)).ok_or(GoOrdinaryError::Linkage)).collect::<Result<Vec<_>>>()?;
        let output=self.carrier(self.result);let input=error_layout::product(&types);
        let names=["s","i"];let values=(0..types.len()).map(|i|var(names[i])).collect::<Vec<_>>();
        let args=types.iter().enumerate().map(|(i,t)|(names[i],t.clone())).collect::<Vec<_>>();
        self.emit("Operands",&[],Expr::Sort0,input.clone(),false)?;
        self.emit("Capture",&args,input.clone(),error_layout::make(&types,&values),false)?;
        let captured=self.own("Capture",values.clone());
        let p=var("p");let pa=[("p",input.clone())];let mut operands=vec![];
        for i in 0..types.len() {
            let name=format!("Operand{i}");let v=error_layout::get(&types,p.clone(),i);
            self.emit(&name,&pa,types[i].clone(),v,false)?;
            self.emit(&format!("CaptureOperand{i}"),&args,eq(types[i].clone(),self.own(&name,vec![captured.clone()]),values[i].clone()),refl(types[i].clone(),values[i].clone()),true)?;
            operands.push(self.own(&name,vec![p.clone()]));
        }
        let length=self.site.kind()==&GoSourceStringOperationKind::Length;
        let action=if length {"Len"} else {"Read"};let predicate=if length {
            call(&format!("{}.Valid",self.prior.rsplit_once(".GoLength.").ok_or(GoOrdinaryError::Linkage)?.0),operands.clone())
        } else {self.prior("Defined",operands.clone())};
        self.emit("Result",&pa,output.clone(),self.prior(action,operands.clone()),false)?;
        self.emit("Domain",&pa,Expr::Sort0,predicate.clone(),false)?;
        self.emit("CaptureResult",&args,eq(output.clone(),self.own("Result",vec![captured.clone()]),self.prior(action,values.clone())),refl(output,self.prior(action,values.clone())),true)?;
        let mut guarded=pa.to_vec();guarded.push(("domain",self.own("Domain",vec![p.clone()])));
        self.emit("DomainWitness",&guarded,predicate.clone(),var("domain"),true)?;
        let mut witnessed=pa.to_vec();witnessed.push(("witness",predicate));
        self.emit("DomainOfWitness",&witnessed,self.own("Domain",vec![p.clone()]),var("witness"),true)?;
        let word=self.own("Result",vec![p.clone()]);
        let width=if length {64} else {8};let raw=word::word_type(width)?;
        let value=if length {self.prior("ToWord",vec![word.clone()])} else {word.clone()};
        self.emit("ResultWord",&pa,raw.clone(),value,false)?;
        self.emit("ResultWordDefinition",&pa,eq(raw.clone(),self.own("ResultWord",vec![p.clone()]),if length {self.prior("ToWord",vec![self.prior("Len",operands.clone())])}else{self.prior("Read",operands.clone())}),refl(raw,self.own("ResultWord",vec![p.clone()])),true)?;
        if length {
            let strings=self.prior.rsplit_once(".GoLength.").ok_or(GoOrdinaryError::Linkage)?.0;
            let intty=pi("_part",boolean(),nat());
            self.emit("DomainInteger",&guarded,eq(intty,self.prior("AsInteger",operands.clone()),call("Mpk.Go.Practical.Int.FromNat",vec![call(&format!("{strings}.Size"),operands.clone())])),self.prior("ValidInteger",vec![operands[0].clone(),var("domain")]),true)?;
        } else {
            let i=operands[1].clone();let s=operands[0].clone();
            self.emit("DomainNonnegative",&guarded,self.prior("Nonnegative",vec![i.clone()]),self.prior("DefinedNonnegative",vec![s.clone(),i.clone(),var("domain")]),true)?;
            let strings=self.prior.rsplit_once(".Index.").ok_or(GoOrdinaryError::Linkage)?.0;
            self.emit("DomainWithin",&guarded,eq(boolean(),less(self.prior("Natural",vec![i.clone()]),call(&format!("{strings}.Size"),vec![s.clone()])),bit(true)),self.prior("DefinedWithin",vec![s.clone(),i.clone(),var("domain")]),true)?;
            self.emit("DomainInteger",&guarded,call("Mpk.Go.Practical.Int.Equivalent",vec![self.prior("Integer",vec![i.clone()]),call("Mpk.Go.Practical.Int.FromNat",vec![self.prior("Natural",vec![i.clone()])])]),self.prior("DefinedInteger",vec![s,i,var("domain")]),true)?;
        }
        Ok(())
    }
}
