//! Source-owned call signatures and argument projections. These descriptors
//! select no evaluator, prove no call contract and do not claim operand order.
use super::*;

#[derive(Clone,Debug,Eq,PartialEq)]
pub struct GoSourceCallArgument {
    expression:usize,
    receiver:bool,
    values:Vec<GoSourceCallArgumentValue>,
}
#[derive(Clone,Debug,Eq,PartialEq)]
pub struct GoSourceCallArgumentValue {
    projection:usize,
    parameter:usize,
    source_type:String,
    parameter_type:String,
    nil:bool,
}
impl GoSourceCallArgumentValue {
    pub fn projection(&self)->usize{self.projection}
    pub fn parameter_position(&self)->usize{self.parameter}
    pub fn source_type_id(&self)->&str{&self.source_type}
    pub fn parameter_type_id(&self)->&str{&self.parameter_type}
    pub fn is_untyped_nil(&self)->bool{self.nil}
}
impl GoSourceCallArgument {
    pub fn expression_node(&self)->usize{self.expression}
    pub fn is_receiver(&self)->bool{self.receiver}
    /// All projections belong to this single original expression. An empty
    /// result list never erases the expression's execution/effect dependency.
    pub fn values(&self)->&[GoSourceCallArgumentValue]{&self.values}
}
#[derive(Clone,Debug,Eq,PartialEq)]
pub struct GoSourceCallTarget {
    dynamic_type:Option<String>,
    callee:GoSourceSelectionRef,
    receiver_type:Option<String>,
    receiver_path:Vec<usize>,
}
impl GoSourceCallTarget {
    pub fn dynamic_type_id(&self)->Option<&str>{self.dynamic_type.as_deref()}
    pub fn callee(&self)->&GoSourceSelectionRef{&self.callee}
    pub fn receiver_type_id(&self)->Option<&str>{self.receiver_type.as_deref()}
    pub fn receiver_path(&self)->&[usize]{&self.receiver_path}
}
#[derive(Clone,Debug,Eq,PartialEq)]
pub struct GoSourceCallSite {
    caller:GoSourceSelectionRef,
    node:usize,
    start:usize,
    end:usize,
    inputs:Vec<String>,
    results:Vec<String>,
    arguments:Vec<GoSourceCallArgument>,
    targets:Vec<GoSourceCallTarget>,
    dispatch:bool,
}
impl GoSourceCallSite {
    pub fn caller(&self)->&GoSourceSelectionRef{&self.caller}
    pub fn node(&self)->usize{self.node}
    pub fn start_byte(&self)->usize{self.start}
    pub fn end_byte(&self)->usize{self.end}
    /// Receiver, if any, followed by the actual complete parameter list.
    pub fn input_type_ids(&self)->&[String]{&self.inputs}
    pub fn result_type_ids(&self)->&[String]{&self.results}
    pub fn arguments(&self)->&[GoSourceCallArgument]{&self.arguments}
    pub fn targets(&self)->&[GoSourceCallTarget]{&self.targets}
    /// Empty dispatch still retains its receiver and nil-safety demand.
    pub fn is_error_dispatch(&self)->bool{self.dispatch}
}
fn target(method:&GoSourceMethod,dynamic_type:Option<&str>)->Result<GoSourceCallTarget>{
    Ok(GoSourceCallTarget{dynamic_type:dynamic_type.map(str::to_string),callee:GoSourceSelectionRef::new(method.declaration_source_object_id().ok_or(GoScalarTypingError::Locator)?,method.declaration_node().ok_or(GoScalarTypingError::Locator)?),receiver_type:Some(method.receiver_type_id().into()),receiver_path:method.receiver_path().to_vec()})
}
/// Derive every ordinary and closed-error call in the complete checked body
/// closure. No caller opcode/signature/type/callee list or native facts enter.
pub fn derive_go_source_call_sites(unit:&GoSourceErrorSelection)->Result<Vec<GoSourceCallSite>>{
    let domain=unit.domain();let selection=unit.selection();let scopes=selection.scopes();
    let locators=selection.calls().iter().map(|c|(c.caller().clone(),c.call_node())).chain(selection.error_calls().iter().map(|c|(c.caller().clone(),c.call_node()))).collect::<BTreeSet<_>>();
    let keys=domain.types().map(|t|t.id()).collect::<BTreeSet<_>>();let mut sites=vec![];let mut engine=Engine::new(scopes);
    for (caller,node) in locators {
        if selection.checked_body_types(&caller).is_none(){return Err(GoScalarTypingError::Locator);}
        let source=caller.source_object_id();let n=engine.node(source,node)?.clone();if n.kind()!="CallExpr"{return Err(GoScalarTypingError::Locator);}
        let request=Request::raw(source,node);engine.compute(request.clone())?;let outputs=engine.facts(&request)?;
        let ordinary=selection.calls().iter().find(|c|c.caller()==&caller&&c.call_node()==node);
        let method=engine.method_calls.iter().find(|c|c.call_source_object_id()==source&&c.call_node()==node).cloned();
        let mut expressions=vec![];
        let (inputs,results,targets,dispatch)=if let Some(header)=method {
            let mut inputs=header.parameter_type_ids().to_vec();
            if !header.method_expression(){inputs.insert(0,header.selected_receiver_type_id().into());expressions.push((header.receiver_node(),true));}
            expressions.extend(list(&n,"Args").iter().enumerate().map(|(i,&n)|(n,header.method_expression()&&i==0)));
            let dispatch=header.method().declaration_node().is_none();
            let targets=if dispatch {
                let d=unit.dispatches().iter().find(|d|d.call().caller()==&caller&&d.call().call_node()==node).ok_or(GoScalarTypingError::Locator)?;
                // The unit derives one global U. No per-call result guess may
                // omit a concrete target, even if an entry seems narrower.
                if d.targets()!=domain.error_cases(){return Err(GoScalarTypingError::Type);}
                d.targets().iter().map(|e|target(e.method(),Some(e.value_type_id()))).collect::<Result<Vec<_>>>()?
            }else{
                let t=target(header.method(),None)?;
                if ordinary.is_none_or(|c|c.callee()!=t.callee()){return Err(GoScalarTypingError::Locator);}vec![t]
            };
            (inputs,header.result_type_ids().to_vec(),targets,dispatch)
        }else{
            let c=ordinary.ok_or(GoScalarTypingError::Locator)?;
            let header=engine.calls.iter().find(|h|h.call_source_object_id()==source&&h.call_node()==node).ok_or(GoScalarTypingError::Locator)?;
            if c.callee().source_object_id()!=header.declaration_source_object_id()||c.callee().node()!=header.declaration_node(){return Err(GoScalarTypingError::Locator);}
            expressions.extend(list(&n,"Args").iter().map(|&n|(n,false)));
            (header.parameter_type_ids().to_vec(),header.result_type_ids().to_vec(),vec![GoSourceCallTarget{dynamic_type:None,callee:c.callee().clone(),receiver_type:None,receiver_path:vec![]}],false)
        };
        if outputs.len()!=results.len()||outputs.iter().zip(&results).any(|(a,t)|a.typ.as_ref()!=Some(t)){return Err(GoScalarTypingError::Type);}
        for t in &targets {
            if selection.checked_body_types(t.callee()).is_none(){return Err(GoScalarTypingError::Locator);}
        }
        let mut position=0;let mut arguments=vec![];
        for (expression,receiver) in expressions {
            let r=Request::raw(source,expression);engine.compute(r.clone())?;let facts=engine.facts(&r)?;let mut values=vec![];
            for (projection,mut fact) in facts.into_iter().enumerate(){
                let parameter_type=inputs.get(position).ok_or(GoScalarTypingError::Arity)?.clone();
                if fact.typ.is_none(){let r=Request::value(source,expression,Some(parameter_type.clone()));engine.compute(r.clone())?;fact=engine.one(&r)?;}
                let source_type=fact.typ.ok_or(GoScalarTypingError::Type)?;
                if !keys.contains(source_type.as_str())||!keys.contains(parameter_type.as_str()){return Err(GoScalarTypingError::Type);}
                values.push(GoSourceCallArgumentValue{projection,parameter:position,source_type,parameter_type,nil:fact.nil});position+=1;
            }
            arguments.push(GoSourceCallArgument{expression,receiver,values});
        }
        if position!=inputs.len()||inputs.iter().chain(&results).any(|t|!keys.contains(t.as_str())){return Err(GoScalarTypingError::Arity);}
        sites.push(GoSourceCallSite{caller,node,start:n.start(),end:n.end(),inputs,results,arguments,targets,dispatch});
    }
    Ok(sites)
}
