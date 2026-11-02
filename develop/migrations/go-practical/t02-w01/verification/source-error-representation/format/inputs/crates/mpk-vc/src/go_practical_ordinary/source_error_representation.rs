//! Concrete case-tag and every inactive-payload-zero representation conditions.
//! Active payload validity, source execution and canonical context authority
//! remain separate. Equalities are on explicit observations, with no funext.
use super::syntax::*;
use super::{
    error_layout, source_error_receiver, word, Builder, GoOrdinaryError,
    GoSourceErrorDispatchCatalogue, GoSourceErrorDispatchRow, GoSourceErrorReceiverRow,
    Result, MAX_BYTES,
};
use crate::go_practical_type_keys::GoSourceErrorCall;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoSourceErrorRepresentationRow {
    prior: GoSourceErrorReceiverRow,
    module: String,
    certificate: Vec<u8>,
}
impl GoSourceErrorRepresentationRow {
    pub fn prior_receiver_row(&self) -> &GoSourceErrorReceiverRow { &self.prior }
    pub fn module_name(&self) -> &str { &self.module }
    pub fn certificate_bytes(&self) -> &[u8] { &self.certificate }
}
fn member(module: &str, name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("{module}.{name}"), args)
}
struct Representation<'a> {
    builder: &'a mut Builder,
    prior: &'a GoSourceErrorReceiverRow,
    module: &'a str,
    types: Vec<Expr>,
}
impl Representation<'_> {
    fn dispatch(&self) -> &GoSourceErrorDispatchRow { self.prior.prior_dispatch_row() }
    fn own(&self,n: &str,args: Vec<Expr>) -> Expr { member(self.module,n,args) }
    fn receiver(&self,n: &str,args: Vec<Expr>) -> Expr { member(self.prior.module_name(),n,args) }
    fn operation(&self,n: &str,args: Vec<Expr>) -> Expr { member(self.dispatch().operations_module_name(),n,args) }
    fn carrier(&self) -> Expr { error_layout::product(&self.types) }
    fn zero(&self,i: usize) -> Expr {
        constant(&format!("{}.T{}.Zero",self.dispatch().source_layout_module_name(),self.dispatch().case_tags()[i].0))
    }
    fn inactive(&self) -> Vec<usize> {
        self.dispatch().case_tags().iter().enumerate()
            .filter_map(|(i,(id,_))|(self.dispatch().boxed_source_type_id()!=Some(id)).then_some(i)).collect()
    }
    fn field(&self,i: usize,e: Expr) -> Expr { error_layout::get(&self.types,e,i+1) }
    fn clauses(&self,e: Expr) -> Vec<Expr> {
        std::iter::once(self.receiver("ReceiverTag",vec![e.clone()]))
            .chain(self.inactive().into_iter().map(|i| eq(self.types[i+1].clone(),self.field(i,e.clone()),self.zero(i)))).collect()
    }
    fn projected(&self,e: Expr,h: Expr,index: usize) -> Expr { error_layout::get(&self.clauses(e),h,index) }
    fn args(&self) -> Vec<Binder> {
        vec![("e",self.carrier()),("represented",self.own("RepresentedCase",vec![var("e")]))]
    }
    fn emit(&mut self,n: &str,args: &[Binder],ty: Expr,body: Expr,law: bool) -> Result<()> {
        emit(self.builder,&format!("{}.{n}",self.module),args,ty,body,law)
    }
    fn predicate_and_projections(&mut self) -> Result<()> {
        self.emit("RepresentedCase",&[("e",self.carrier())],Expr::Sort0,error_layout::product(&self.clauses(var("e"))),false)?;
        self.emit("RepresentedTag",&self.args(),self.receiver("ReceiverTag",vec![var("e")]),self.projected(var("e"),var("represented"),0),true)?;
        for (position,i) in self.inactive().into_iter().enumerate() {
            let id=&self.dispatch().case_tags()[i].0;
            self.emit(&format!("InactiveZero.C{id}"),&self.args(),eq(self.types[i+1].clone(),self.field(i,var("e")),self.zero(i)),self.projected(var("e"),var("represented"),position+1),true)?;
        }
        Ok(())
    }
    fn constructed(&mut self) -> Result<()> {
        let boxed=self.dispatch().boxed_source_type_id();
        let args=boxed.map(|id|vec![("v",constant(&format!("{}.T{id}.Carrier",self.dispatch().source_layout_module_name())))])
            .unwrap_or_default();
        let value=boxed.map(|id|member(&format!("{}.C{id}",self.dispatch().boxing_module_name()),"Box",vec![var("v")]))
            .unwrap_or_else(||self.operation("Nil",vec![]));
        let actual_args=if boxed.is_some(){vec![var("v")]}else{vec![]};
        let mut witnesses=vec![self.receiver("ConstructedReceiverTag",actual_args.clone())];
        for i in self.inactive() {
            let name=format!("ConstructedInactiveZero.C{}",self.dispatch().case_tags()[i].0);
            self.emit(&name,&args,eq(self.types[i+1].clone(),self.field(i,value.clone()),self.zero(i)),refl(self.types[i+1].clone(),self.zero(i)),true)?;
            witnesses.push(self.own(&name,actual_args.clone()));
        }
        self.emit("ConstructedRepresentation",&args,self.own("RepresentedCase",vec![value.clone()]),error_layout::make(&self.clauses(value.clone()),&witnesses),true)?;
        let copied=self.operation("Copy",vec![value.clone()]);
        self.emit("CopiedConstructionRepresentation",&args,self.own("RepresentedCase",vec![copied]),self.own("CopyRepresentation",vec![value.clone(),self.own("ConstructedRepresentation",actual_args.clone())]),true)?;
        for (name,goal,observation,result_ty,result) in [
            ("ConstructedSelectionFromRepresentation","SelectedFromRepresentation","Select",nat(),self.output()),
            ("CopiedConstructionSelectionFromRepresentation","CopySelectedFromRepresentation","CopySelect",nat(),self.output()),
        ] {
            let observed=if observation=="Select"{member(self.dispatch().module_name(),"Select",vec![value.clone()])}else{member(self.dispatch().module_name(),"Select",vec![self.operation("Copy",vec![value.clone()])])};
            self.emit(name,&args,eq(result_ty,observed,result),self.own(goal,vec![value.clone(),self.own("ConstructedRepresentation",actual_args.clone())]),true)?;
        }
        Ok(())
    }
    fn copying(&mut self) -> Result<()> {
        let args=self.args();let e=var("e");let copied=self.operation("Copy",vec![e.clone()]);
        let tag=self.own("RepresentedTag",vec![e.clone(),var("represented")]);
        let mut witnesses=vec![self.receiver("CopyReceiverTag",vec![e.clone(),tag])];
        for i in self.inactive() {
            let id=&self.dispatch().case_tags()[i].0;
            let proof=Equality{ty:self.types[i+1].clone(),left:self.field(i,copied.clone()),right:self.field(i,e.clone()),proof:self.operation(&format!("CopyPayload{i}"),vec![e.clone()])}
                .then(Equality{ty:self.types[i+1].clone(),left:self.field(i,e.clone()),right:self.zero(i),proof:self.own(&format!("InactiveZero.C{id}"),vec![e.clone(),var("represented")])})?;
            let name=format!("CopyInactiveZero.C{id}");
            self.emit(&name,&args,eq(self.types[i+1].clone(),self.field(i,copied.clone()),self.zero(i)),proof.proof,true)?;
            witnesses.push(self.own(&name,vec![e.clone(),var("represented")]));
        }
        self.emit("CopyRepresentation",&args,self.own("RepresentedCase",vec![copied.clone()]),error_layout::make(&self.clauses(copied),&witnesses),true)
    }
    fn output(&self) -> Expr {
        let row=self.dispatch();
        row.boxed_source_type_id().and_then(|id|row.targets().iter().find(|t|t.value_type_id()==id))
            .map(|t|{let m=t.method();member(row.module_name(),&format!("Target.C{}.S{}.N{}",t.value_type_id(),m.declaration_source_object_id().expect("checked method"),m.declaration_node().expect("checked method")),vec![])})
            .unwrap_or_else(||member(row.module_name(),"NoTarget",vec![]))
    }
    fn observations(&mut self) -> Result<()> {
        let args=self.args();let e=var("e");let tag=self.own("RepresentedTag",vec![e.clone(),var("represented")]);
        let boxing=self.dispatch().boxing_module_name().to_string();
        let dispatch=self.dispatch().module_name().to_string();
        let is_nil=self.dispatch().boxed_source_type_id().is_none();
        for (name,goal,observed,ty,result) in [
            ("SelectedFromRepresentation","SelectedByTag",member(&dispatch,"Select",vec![e.clone()]),nat(),self.output()),
            ("IsNilFromRepresentation","IsNilByTag",member(&boxing,"IsNil",vec![e.clone()]),boolean(),bit(is_nil)),
            ("IsNonNilFromRepresentation","IsNonNilByTag",member(&boxing,"IsNonNil",vec![e.clone()]),boolean(),bit(!is_nil)),
        ] {self.emit(name,&args,eq(ty,observed,result),self.receiver(goal,vec![e.clone(),tag.clone()]),true)?;}
        let copied=self.operation("Copy",vec![e.clone()]);let represented=self.own("CopyRepresentation",vec![e.clone(),var("represented")]);
        for (name,goal,observed,ty,result) in [
            ("CopySelectedFromRepresentation","SelectedFromRepresentation",member(&dispatch,"Select",vec![copied.clone()]),nat(),self.output()),
            ("CopyNonNilFromRepresentation","IsNonNilFromRepresentation",member(&boxing,"IsNonNil",vec![copied.clone()]),boolean(),bit(!is_nil)),
        ] {self.emit(name,&args,eq(ty,observed,result),self.own(goal,vec![copied.clone(),represented.clone()]),true)?;}
        Ok(())
    }
}
impl GoSourceErrorDispatchCatalogue {
    /// Closed representation equations for the exact source case and every
    /// inactive canonical zero. Active-payload validity is still separate.
    pub fn generate_representation_row(&self,call: &GoSourceErrorCall,boxed_type: Option<&str>) -> Result<GoSourceErrorRepresentationRow> {
        let (mut builder,prior)=self.receiver_row_builder(call,boxed_type)?;
        let prior=source_error_receiver::complete_row(builder.clone(),prior)?;
        let dispatch=prior.prior_dispatch_row();
        let types=std::iter::once(word::word_type(64)?).chain(dispatch.case_tags().iter().map(|(id,_)|constant(&format!("{}.T{id}.Carrier",dispatch.source_layout_module_name())))).collect();
        let module=format!("{}.Representation",prior.module_name());
        let mut generator=Representation{builder:&mut builder,prior:&prior,module:&module,types};
        generator.predicate_and_projections()?;
        generator.copying()?;
        generator.observations()?;
        generator.constructed()?;
        builder.certificate.module=module.clone();
        Ok(GoSourceErrorRepresentationRow{prior,module,certificate:builder.finish()?})
    }
    pub fn import_representation_row(&self,candidate: &[u8],call: &GoSourceErrorCall,boxed_type: Option<&str>) -> Result<GoSourceErrorRepresentationRow> {
        if candidate.len()>MAX_BYTES{return Err(GoOrdinaryError::Limit)}
        let expected=self.generate_representation_row(call,boxed_type)?;
        if candidate!=expected.certificate_bytes(){return Err(GoOrdinaryError::Linkage)}
        Ok(expected)
    }
}
