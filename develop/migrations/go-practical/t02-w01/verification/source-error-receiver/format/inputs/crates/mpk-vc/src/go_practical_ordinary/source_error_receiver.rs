//! Conditional target selection and nil tests for an arbitrary full receiver.
//! The actual tag equality is a checked premise, not assumed source validity,
//! receiver evaluation, method execution, callee effects or context authority.
use super::syntax::*;
use super::{error_layout, source_error_cases, source_error_dispatch, word, Builder,
    GoOrdinaryError, GoSourceErrorDispatchCatalogue, GoSourceErrorDispatchRow, Result, MAX_BYTES};
use crate::go_practical_type_keys::GoSourceErrorCall;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoSourceErrorReceiverRow {
    prior: GoSourceErrorDispatchRow,
    module: String,
    certificate: Vec<u8>,
}
impl GoSourceErrorReceiverRow {
    pub fn prior_dispatch_row(&self) -> &GoSourceErrorDispatchRow {&self.prior}
    pub fn module_name(&self) -> &str {&self.module}
    pub fn certificate_bytes(&self) -> &[u8] {&self.certificate}
}
fn member(module:&str,name:&str,args:Vec<Expr>)->Expr {call(&format!("{module}.{name}"),args)}
fn target_name(prior:&GoSourceErrorDispatchRow,i:usize)->String {
    let t=&prior.targets()[i];let m=t.method();
    format!("Target.C{}.S{}.N{}",t.value_type_id(),m.declaration_source_object_id().expect("checked original method"),m.declaration_node().expect("checked original method"))
}
struct Receiver<'a> {
    builder:&'a mut Builder,
    prior:&'a GoSourceErrorDispatchRow,
    module:&'a str,
    carrier:Expr,
}
impl Receiver<'_> {
    fn own(&self,n:&str,args:Vec<Expr>)->Expr {member(self.module,n,args)}
    fn dispatch(&self,n:&str,args:Vec<Expr>)->Expr {member(self.prior.module_name(),n,args)}
    fn observe(&self,n:&str,e:Expr)->Expr {member(self.prior.boxing_module_name(),n,vec![e])}
    fn tag(&self)->Expr {
        self.prior.boxed_source_type_id().map(|id|member(&format!("{}.C{id}",self.prior.boxing_module_name()),"Tag",vec![]))
            .unwrap_or_else(||member(self.prior.operations_module_name(),"NilTag",vec![]))
    }
    fn matched(&self,i:usize,e:Expr)->Expr {
        let id=&self.prior.case_tags()[i].0;
        member(self.prior.operations_module_name(),"HasCase",vec![member(&format!("{}.C{id}",self.prior.boxing_module_name()),"Tag",vec![]),e])
    }
    fn emit(&mut self,n:&str,args:&[Binder],ty:Expr,body:Expr,law:bool)->Result<()> {
        emit(self.builder,&format!("{}.{n}",self.module),args,ty,body,law)
    }
    fn arguments(&self)->Vec<Binder> {
        vec![("e",self.carrier.clone()),("tag_case",self.own("ReceiverTag",vec![var("e")]))]
    }
    fn output(&self,start:usize)->Expr {
        self.prior.boxed_source_type_id().and_then(|id|self.prior.case_tags().iter().position(|(t,_)|t==id))
            .filter(|i|start<=*i).map(|i|self.dispatch(&target_name(self.prior,i),vec![]))
            .unwrap_or_else(||self.dispatch("NoTarget",vec![]))
    }
    fn predicate(&mut self)->Result<()> {
        self.emit("ReceiverTag",&[("e",self.carrier.clone())],Expr::Sort0,
            eq(word::word_type(64)?,self.observe("Tag",var("e")),self.tag()),false)
    }
    fn matches(&mut self)->Result<()> {
        let args=self.arguments();
        for (i,(id,ordinal)) in self.prior.case_tags().iter().enumerate() {
            let target=member(&format!("{}.C{id}",self.prior.boxing_module_name()),"Tag",vec![]);
            let matched=self.prior.boxed_source_type_id()==Some(id);
            let proof=if matched {
                member(self.prior.boxing_module_name(),"SameTagHasOwnCase",vec![var("e"),self.tag(),var("tag_case")])
            } else {
                let different=if let Some(row)=self.prior.case_row_module_name() {
                    member(row,&format!("TagEqual.C{id}"),vec![])
                }else{
                    let comparison=word::wc(64,"Equal",vec![target.clone(),self.tag()]);
                    self.emit(&format!("NilTagDifferent.C{id}"),&[],eq(boolean(),comparison,bit(false)),
                        source_error_cases::comparison_proof(target.clone(),self.tag(),*ordinal,0)?,true)?;
                    self.own(&format!("NilTagDifferent.C{id}"),vec![])
                };
                member(self.prior.operations_module_name(),"DifferentTagHasNoCase",vec![target,var("e"),self.tag(),var("tag_case"),different])
            };
            self.emit(&format!("MatchByTag.C{id}"),&args,eq(boolean(),self.matched(i,var("e")),bit(matched)),proof,true)?;
        }
        Ok(())
    }
    fn selection(&mut self)->Result<()> {
        let args=self.arguments();let count=self.prior.case_tags().len();
        let suffix=|i,e|self.dispatch(&format!("SelectFrom{i}"),vec![e]);
        let end=self.output(count);
        self.emit(&format!("Suffix{count}ByTag"),&args,eq(nat(),suffix(count,var("e")),end.clone()),refl(nat(),end),true)?;
        for i in (0..count).rev() {
            let matched=self.own(&format!("MatchByTag.C{}",self.prior.case_tags()[i].0),vec![var("e"),var("tag_case")]);
            let own=self.prior.boxed_source_type_id()==Some(self.prior.case_tags()[i].0.as_str());
            let proof=if own {
                self.dispatch(&format!("SelectTrue{i}"),vec![var("e"),matched])
            }else{
                self.dispatch(&format!("SelectFalse{i}"),vec![var("e"),matched,self.output(i),self.own(&format!("Suffix{}ByTag",i+1),vec![var("e"),var("tag_case")])])
            };
            self.emit(&format!("Suffix{i}ByTag"),&args,eq(nat(),self.dispatch(&format!("SelectFrom{i}"),vec![var("e")]),self.output(i)),proof,true)?;
        }
        self.emit("SelectedByTag",&args,eq(nat(),self.dispatch("Select",vec![var("e")]),self.output(0)),self.own("Suffix0ByTag",vec![var("e"),var("tag_case")]),true)
    }
    fn nil_tests(&mut self)->Result<()> {
        let args=self.arguments();let is_nil=self.prior.boxed_source_type_id().is_none();
        let proof=if let Some(id)=self.prior.boxed_source_type_id() {
            member(self.prior.boxing_module_name(),"TagNonzeroIsNotNil",vec![var("e"),self.tag(),var("tag_case"),member(&format!("{}.C{id}",self.prior.boxing_module_name()),"TagNonzero",vec![])])
        }else{
            member(self.prior.operations_module_name(),"FrozenTagIsNil",vec![var("e"),var("tag_case")])
        };
        self.emit("IsNilByTag",&args,eq(boolean(),self.observe("IsNil",var("e")),bit(is_nil)),proof,true)?;
        let proof=member(if is_nil {self.prior.operations_module_name()}else{self.prior.boxing_module_name()},
            if is_nil {"NilIsNotNonNil"}else{"NotNilIsNonNil"},
            vec![var("e"),self.own("IsNilByTag",vec![var("e"),var("tag_case")])]);
        self.emit("IsNonNilByTag",&args,eq(boolean(),self.observe("IsNonNil",var("e")),bit(!is_nil)),proof,true)
    }
    fn copying(&mut self)->Result<()> {
        let args=self.arguments();let copy=member(self.prior.operations_module_name(),"Copy",vec![var("e")]);
        let proof=Equality{
            ty:word::word_type(64)?,left:self.observe("Tag",copy.clone()),right:self.observe("Tag",var("e")),
            proof:member(self.prior.operations_module_name(),"TagOfCopy",vec![var("e")]),
        }.then(Equality{ty:word::word_type(64)?,left:self.observe("Tag",var("e")),right:self.tag(),proof:var("tag_case")})?;
        self.emit("CopyReceiverTag",&args,self.own("ReceiverTag",vec![copy.clone()]),proof.proof,true)?;
        for (name,observation,expected) in [
            ("CopySelectedByTag","Select",self.output(0)),
            ("CopyIsNonNilByTag","IsNonNil",bit(self.prior.boxed_source_type_id().is_some())),
        ] {
            let observed=if observation=="Select" {self.dispatch(observation,vec![copy.clone()])}else{self.observe(observation,copy.clone())};
            let proof=self.own(if observation=="Select" {"SelectedByTag"}else{"IsNonNilByTag"},vec![copy.clone(),self.own("CopyReceiverTag",vec![var("e"),var("tag_case")])]);
            self.emit(name,&args,eq(if observation=="Select" {nat()}else{boolean()},observed,expected),proof,true)?;
        }
        Ok(())
    }
    fn witnesses(&mut self)->Result<()> {
        let boxed=self.prior.boxed_source_type_id();
        let args=boxed.map(|id|vec![("v",constant(&format!("{}.T{id}.Carrier",self.prior.source_layout_module_name())))]).unwrap_or_default();
        let value=boxed.map(|id|member(&format!("{}.C{id}",self.prior.boxing_module_name()),"Box",vec![var("v")]))
            .unwrap_or_else(||member(self.prior.operations_module_name(),"Nil",vec![]));
        let witness=boxed.map(|id|member(&format!("{}.C{id}",self.prior.boxing_module_name()),"TagOfBox",vec![var("v")]))
            .unwrap_or_else(||member(self.prior.operations_module_name(),"TagOfNil",vec![]));
        self.emit("ConstructedReceiverTag",&args,self.own("ReceiverTag",vec![value.clone()]),witness,true)?;
        let witness=self.own("ConstructedReceiverTag",if boxed.is_some(){vec![var("v")]}else{vec![]});
        for (name,goal,op,ty,expected) in [
            ("ConstructedSelection","SelectedByTag","Select",nat(),self.output(0)),
            ("ConstructedNonNil","IsNonNilByTag","IsNonNil",boolean(),bit(boxed.is_some())),
            ("CopiedConstructionSelection","CopySelectedByTag","CopySelect",nat(),self.output(0)),
        ] {
            let observed=if op=="Select" {self.dispatch("Select",vec![value.clone()])}else if op=="CopySelect" {
                self.dispatch("Select",vec![member(self.prior.operations_module_name(),"Copy",vec![value.clone()])])
            }else{self.observe(op,value.clone())};
            self.emit(name,&args,eq(ty,observed,expected),self.own(goal,vec![value.clone(),witness.clone()]),true)?;
        }
        Ok(())
    }
}
impl GoSourceErrorDispatchCatalogue {
    /// A checked tag premise conditions selection/nil tests for the same full
    /// receiver. It grants no caller-supplied source-validity assumption.
    pub fn generate_receiver_row(&self,call:&GoSourceErrorCall,boxed_type:Option<&str>)->Result<GoSourceErrorReceiverRow> {
        let (mut builder,prior)=self.row_builder(call,boxed_type)?;
        let prior=source_error_dispatch::complete_row(builder.clone(),prior)?;
        let types=std::iter::once(word::word_type(64)?).chain(prior.case_tags().iter().map(|(id,_)|constant(&format!("{}.T{id}.Carrier",prior.source_layout_module_name())))).collect::<Vec<_>>();
        let module=format!("{}.Receiver",prior.module_name());
        let mut generator=Receiver{builder:&mut builder,prior:&prior,module:&module,carrier:error_layout::product(&types)};
        generator.predicate()?;generator.matches()?;generator.selection()?;generator.nil_tests()?;generator.copying()?;generator.witnesses()?;
        builder.certificate.module=module.clone();
        Ok(GoSourceErrorReceiverRow{prior,module,certificate:builder.finish()?})
    }
    pub fn import_receiver_row(&self,candidate:&[u8],call:&GoSourceErrorCall,boxed_type:Option<&str>)->Result<GoSourceErrorReceiverRow> {
        if candidate.len()>MAX_BYTES {return Err(GoOrdinaryError::Limit);}
        let expected=self.generate_receiver_row(call,boxed_type)?;
        if candidate!=expected.certificate_bytes() {return Err(GoOrdinaryError::Linkage);}
        Ok(expected)
    }
}
