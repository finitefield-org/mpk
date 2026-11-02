//! Independent scalar expression/initializer typing over owned original syntax.
//! Header argument/result checks do not check callee bodies or invocation state.
use super::*;
mod operators;
use operators::comparison;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GoScalarTypingError {
    Locator, Name, Unsupported, Type, Arity, Cycle,
    Constant(GoConstantError), SourceType(GoSourceTypeError), Signature(GoSourceSignatureError),
}
impl std::fmt::Display for GoScalarTypingError {
    fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{write!(f,"Go scalar typing error: {self:?}")}
}
impl std::error::Error for GoScalarTypingError {}
type Result<T>=std::result::Result<T,GoScalarTypingError>;
#[derive(Clone,Debug,Eq,PartialEq)]
pub struct GoSourceScalarValue {
    typ:GoConstantScalarType,
    constant:Option<GoEvaluatedConstant>,
    addressable:bool,
}
impl GoSourceScalarValue {
    pub fn scalar_type(&self)->&GoConstantScalarType{&self.typ}
    pub fn constant(&self)->Option<&GoEvaluatedConstant>{self.constant.as_ref()}
    pub fn addressable(&self)->bool{self.addressable}
}
#[derive(Clone,Debug,Eq,PartialEq,Ord,PartialOrd)]
pub struct GoScalarCallHeader {
    call_source:String, call_node:usize,
    declaration_source:String, declaration_node:usize, body_node:Option<usize>,
    parameters:Vec<String>, results:Vec<String>,
}
impl GoScalarCallHeader {
    pub fn call_source_object_id(&self)->&str{&self.call_source}
    pub fn call_node(&self)->usize{self.call_node}
    pub fn declaration_source_object_id(&self)->&str{&self.declaration_source}
    pub fn declaration_node(&self)->usize{self.declaration_node}
    pub fn body_node(&self)->Option<usize>{self.body_node}
    pub fn parameter_type_ids(&self)->&[String]{&self.parameters}
    pub fn result_type_ids(&self)->&[String]{&self.results}
}
#[derive(Clone,Debug,Eq,PartialEq)]
pub struct GoScalarTyping {
    scopes:GoSourceScopes,
    values:Vec<GoSourceScalarValue>,
    calls:Vec<GoScalarCallHeader>,
}
impl GoScalarTyping {
    pub fn source(&self,id:&str)->Option<&GoParsedSource>{self.scopes.source(id)}
    pub fn values(&self)->&[GoSourceScalarValue]{&self.values}
    pub fn call_headers(&self)->&[GoScalarCallHeader]{&self.calls}
}
#[derive(Clone,Debug,Eq,PartialEq,Ord,PartialOrd)]
enum Mode { Raw, Value(Option<String>), Fixed(String) }
#[derive(Clone,Debug,Eq,PartialEq,Ord,PartialOrd)]
enum Target { Expression(usize,Mode), Binding(usize), Declaration(usize) }
#[derive(Clone,Debug,Eq,PartialEq,Ord,PartialOrd)]
struct Request { source:String,target:Target }
impl Request {
    fn raw(source:&str,node:usize)->Self{Self{source:source.into(),target:Target::Expression(node,Mode::Raw)}}
    fn value(source:&str,node:usize,typ:Option<String>)->Self{Self{source:source.into(),target:Target::Expression(node,Mode::Value(typ))}}
    fn fixed(source:&str,node:usize,typ:String)->Self{Self{source:source.into(),target:Target::Expression(node,Mode::Fixed(typ))}}
    fn binding(b:&GoSourceNameBinding,index:usize)->Self{Self{source:b.source_object_id().into(),target:Target::Binding(index)}}
    fn declaration(source:&str,node:usize)->Self{Self{source:source.into(),target:Target::Declaration(node)}}
    fn expression(&self)->Result<usize>{match self.target{Target::Expression(n,_)=>Ok(n),_=>Err(GoScalarTypingError::Locator)}}
}
#[derive(Clone,Debug)]
struct Atom { typ:Option<String>,kind:GoConstantKind,value:Option<GoEvaluatedConstant>,addressable:bool }
type Facts=Vec<Atom>;
#[derive(Clone)]
enum Destination { Inferred, Declared(String), Reused(Request) }
enum Plan {
    Forward(Request), Select(Request,usize), Materialize(Request,Option<String>),
    Unary(String,Request), BinaryFirst(String,Request,Request), BinaryReady(String,Request,Request),
    Cast(String,Request), BuiltinFirst(String,Vec<Request>), BuiltinReady(String,Vec<Request>),
    CallFirst(Box<GoScalarCallHeader>,Vec<Request>), CallReady(Box<GoScalarCallHeader>,Vec<Request>),
    DeclarationFirst(Vec<Destination>,Vec<Request>), DeclarationReady(Vec<Destination>,Vec<Request>),
}
impl Plan {
    fn inputs(&self)->Vec<Request>{match self{
        Self::Forward(r)|Self::Select(r,_)|Self::Materialize(r,_)|Self::Unary(_,r)|Self::Cast(_,r)=>vec![r.clone()],
        Self::BinaryFirst(_,a,b)|Self::BinaryReady(_,a,b)=>vec![a.clone(),b.clone()],
        Self::BuiltinFirst(_,rs)|Self::BuiltinReady(_,rs)|Self::CallFirst(_,rs)|Self::CallReady(_,rs)=>rs.clone(),
        Self::DeclarationFirst(dest,rs)|Self::DeclarationReady(dest,rs)=>rs.iter().cloned().chain(dest.iter().filter_map(|d|if let Destination::Reused(r)=d{Some(r.clone())}else{None})).collect(),
    }}
}
enum Step { Done(Facts), More(Plan) }
enum Work { Visit(Request), Finish(Request,Box<Plan>) }
struct Engine<'a> {
    scopes:&'a GoSourceScopes,
    types:BTreeMap<String,GoConstantScalarType>,
    done:BTreeMap<Request,Facts>, active:BTreeSet<Request>, calls:BTreeSet<GoScalarCallHeader>,
}
fn attr<'a>(n:&'a GoSyntaxNode,key:&str)->Option<&'a str>{n.attributes().find_map(|(k,v)|(k==key).then_some(v))}
fn list<'a>(n:&'a GoSyntaxNode,role:&str)->&'a[usize]{n.edges().iter().find(|e|e.role()==role).map_or(&[],|e|e.targets())}
fn child(n:&GoSyntaxNode,role:&str)->Result<usize>{let xs=list(n,role);if xs.len()==1{Ok(xs[0])}else{Err(GoScalarTypingError::Locator)}}
fn kind(typ:&GoConstantScalarType)->GoConstantKind{match typ.underlying_builtin(){"bool"=>GoConstantKind::Bool,"string"=>GoConstantKind::String,_=>GoConstantKind::Integer}}
fn constant_error(error:GoConstantError)->GoScalarTypingError {if error==GoConstantError::Unsupported{GoScalarTypingError::Unsupported}else{GoScalarTypingError::Constant(error)}}
impl<'a> Engine<'a> {
    fn new(scopes:&'a GoSourceScopes)->Self{Self{scopes,types:BTreeMap::new(),done:BTreeMap::new(),active:BTreeSet::new(),calls:BTreeSet::new()}}
    fn node(&self,source:&str,node:usize)->Result<&GoSyntaxNode>{self.scopes.source(source).and_then(|s|s.nodes().get(node)).ok_or(GoScalarTypingError::Locator)}
    fn intern(&mut self,typ:GoConstantScalarType)->Result<String>{let id=typ.source_type_key().id().to_string();if let Some(prior)=self.types.insert(id.clone(),typ.clone()){if prior!=typ{return Err(GoScalarTypingError::Type)}}Ok(id)}
    fn builtin(&mut self,name:&str)->Result<String>{self.intern(GoConstantScalarType::builtin(name).map_err(constant_error)?)}
    fn annotation(&mut self,source:&str,node:usize)->Result<String>{let graph=derive_go_source_type(self.scopes,source,node).map_err(GoScalarTypingError::SourceType)?;self.graph_type(&graph,&graph.roots()[0])}
    fn graph_type(&mut self,graph:&GoDerivedSourceTypes,id:&str)->Result<String>{self.intern(GoConstantScalarType::from_source_graph(graph,id).map_err(constant_error)?)}
    fn atom(&self,id:&str,addressable:bool)->Atom{Atom{typ:Some(id.into()),kind:kind(&self.types[id]),value:None,addressable}}
    fn constant(&mut self,value:GoEvaluatedConstant)->Result<Atom>{let typ=value.scalar_type().cloned().map(|t|self.intern(t)).transpose()?;Ok(Atom{typ,kind:value.kind(),value:Some(value),addressable:false})}
    fn facts(&self,r:&Request)->Result<Facts>{self.done.get(r).cloned().ok_or(GoScalarTypingError::Type)}
    fn one(&self,r:&Request)->Result<Atom>{let v=self.facts(r)?;if v.len()==1{Ok(v[0].clone())}else{Err(GoScalarTypingError::Arity)}}
    fn symbol(&self,source:&str,mut node:usize)->Result<GoNameLookup>{loop{let n=self.node(source,node)?;match n.kind(){
        "ParenExpr"=>node=child(n,"X")?,
        "Ident"=>return self.scopes.lookup_identifier(source,node).map_err(|_|GoScalarTypingError::Name)?.ok_or(GoScalarTypingError::Name),
        "SelectorExpr"=>{let base=child(n,"X")?;if self.node(source,base)?.kind()!="Ident"{return Err(GoScalarTypingError::Unsupported)}let Some(GoNameLookup::Original(package))=self.scopes.lookup_identifier(source,base).map_err(|_|GoScalarTypingError::Name)?else{return Err(GoScalarTypingError::Unsupported)};if package.kind()!=GoSourceNameKind::Import{return Err(GoScalarTypingError::Unsupported)}let name=attr(self.node(source,child(n,"Sel")?)?,"Name").ok_or(GoScalarTypingError::Locator)?;return self.scopes.imported_member(package.import_path().ok_or(GoScalarTypingError::Name)?,name).map_err(|_|GoScalarTypingError::Name)?.map(GoNameLookup::Original).ok_or(GoScalarTypingError::Name)},
        _=>return Err(GoScalarTypingError::Unsupported),
    }}}
    fn binding_index(&self,b:&GoSourceNameBinding)->Result<usize>{self.scopes.bindings().iter().position(|x|x==b).ok_or(GoScalarTypingError::Locator)}
    fn header(&mut self,source:&str,node:usize,b:&GoSourceNameBinding)->Result<GoScalarCallHeader>{
        let signature=derive_go_source_signature(self.scopes,b.source_object_id(),b.declaration_node()).map_err(GoScalarTypingError::Signature)?;
        let mut parameters=vec![];let mut results=vec![];
        for slot in signature.slots(){let typ=self.graph_type(signature.types(),slot.type_id())?;match slot.role(){GoSourceSignatureRole::Parameter=>parameters.push(typ),GoSourceSignatureRole::Result=>results.push(typ),GoSourceSignatureRole::Receiver=>return Err(GoScalarTypingError::Unsupported)}}
        Ok(GoScalarCallHeader{call_source:source.into(),call_node:node,declaration_source:b.source_object_id().into(),declaration_node:b.declaration_node(),body_node:signature.body_node(),parameters,results})
    }
    fn declaration(&mut self,source:&str,node:usize)->Result<Plan>{
        let n=self.node(source,node)?.clone();
        let names=if n.kind()=="ValueSpec"{list(&n,"Names")}else if n.kind()=="AssignStmt"&&attr(&n,"Tok")==Some(":="){list(&n,"Lhs")}else{return Err(GoScalarTypingError::Unsupported)};
        let rhs=if n.kind()=="ValueSpec"{list(&n,"Values")}else{list(&n,"Rhs")};
        let annotation=list(&n,"Type").first().copied().map(|typ|self.annotation(source,typ)).transpose()?;
        let mut targets=vec![];
        for &name in names{
            if self.node(source,name)?.kind()!="Ident"{return Err(GoScalarTypingError::Type)}
            let target=if let Some(t)=&annotation{Destination::Declared(t.clone())}else if n.kind()=="ValueSpec"||attr(self.node(source,name)?,"Name")==Some("_")||self.scopes.bindings().iter().any(|b|b.source_object_id()==source&&b.declaration_node()==node&&b.name_node()==name){Destination::Inferred}else{
                let GoNameLookup::Original(b)=self.symbol(source,name)?else{return Err(GoScalarTypingError::Type)};if b.kind()!=GoSourceNameKind::Variable{return Err(GoScalarTypingError::Type)}Destination::Reused(Request::binding(&b,self.binding_index(&b)?))
            };targets.push(target);
        }
        if rhs.is_empty(){return Err(GoScalarTypingError::Arity)}
        Ok(Plan::DeclarationFirst(targets,rhs.iter().map(|&node|Request::raw(source,node)).collect()))
    }
    fn plan(&mut self,r:&Request)->Result<Step>{
        match &r.target{
            Target::Binding(index)=>{let b=self.scopes.bindings().get(*index).filter(|b|b.source_object_id()==r.source&&b.kind()==GoSourceNameKind::Variable).ok_or(GoScalarTypingError::Locator)?;
                let n=self.node(&r.source,b.declaration_node())?;
                if matches!(n.kind(),"FuncDecl"|"FuncLit"){
                    let declared=derive_go_declared_binding_type(self.scopes,b).map_err(GoScalarTypingError::Signature)?;let typ=self.graph_type(declared.types(),declared.type_id())?;return Ok(Step::Done(vec![self.atom(&typ,true)]));
                }
                let names=if n.kind()=="ValueSpec"{list(n,"Names")}else{list(n,"Lhs")};let ordinal=names.iter().position(|&n|n==b.name_node()).ok_or(GoScalarTypingError::Unsupported)?;
                Ok(Step::More(Plan::Select(Request::declaration(&r.source,b.declaration_node()),ordinal)))
            }
            Target::Declaration(node)=>{
                let n=self.node(&r.source,*node)?;
                if n.kind()=="ValueSpec"&&list(n,"Values").is_empty(){let typ=self.annotation(&r.source,child(n,"Type")?)?;let count=list(self.node(&r.source,*node)?,"Names").len();return Ok(Step::Done((0..count).map(|_|self.atom(&typ,true)).collect()))}
                Ok(Step::More(self.declaration(&r.source,*node)?))
            }
            Target::Expression(node,Mode::Value(expected))=>Ok(Step::More(Plan::Materialize(Request::raw(&r.source,*node),expected.clone()))),
            Target::Expression(node,Mode::Fixed(typ))=>self.fixed(&r.source,*node,typ),
            Target::Expression(node,Mode::Raw)=>{
                let n=self.node(&r.source,*node)?.clone();
                let step=match n.kind(){
                    "BasicLit"=>Step::Done(vec![self.constant(evaluate_go_constant(self.scopes,&r.source,*node).map_err(constant_error)?)?]),
                    "Ident"|"SelectorExpr"=>match self.symbol(&r.source,*node)?{
                        GoNameLookup::Original(b)if b.kind()==GoSourceNameKind::Variable=>Step::More(Plan::Forward(Request::binding(&b,self.binding_index(&b)?))),
                        GoNameLookup::Original(b)if b.kind()==GoSourceNameKind::Constant=>Step::Done(vec![self.constant(evaluate_go_constant_binding(self.scopes,&b).map_err(constant_error)?)?]),
                        GoNameLookup::Predeclared{kind:GoSourceNameKind::BuiltinConstant,..}=>Step::Done(vec![self.constant(evaluate_go_constant(self.scopes,&r.source,*node).map_err(constant_error)?)?]),
                        _=>return Err(GoScalarTypingError::Unsupported),
                    },
                    "ParenExpr"=>Step::More(Plan::Forward(Request::raw(&r.source,child(&n,"X")?))),
                    "UnaryExpr"=>Step::More(Plan::Unary(attr(&n,"Op").ok_or(GoScalarTypingError::Locator)?.into(),Request::raw(&r.source,child(&n,"X")?))),
                    "BinaryExpr"=>Step::More(Plan::BinaryFirst(attr(&n,"Op").ok_or(GoScalarTypingError::Locator)?.into(),Request::raw(&r.source,child(&n,"X")?),Request::raw(&r.source,child(&n,"Y")?))),
                    "CallExpr"=>{
                        if attr(&n,"Ellipsis")==Some("true"){return Err(GoScalarTypingError::Unsupported)}
                        let fun=child(&n,"Fun")?;let args=list(&n,"Args").iter().map(|&arg|Request::raw(&r.source,arg)).collect::<Vec<_>>();
                        match self.symbol(&r.source,fun)?{
                            GoNameLookup::Predeclared{kind:GoSourceNameKind::BuiltinType,..}=>{let target=self.annotation(&r.source,fun)?;if args.len()!=1{return Err(GoScalarTypingError::Arity)}Step::More(Plan::Cast(target,args[0].clone()))},
                            GoNameLookup::Original(b)if b.kind()==GoSourceNameKind::Type=>{let target=self.annotation(&r.source,fun)?;if args.len()!=1{return Err(GoScalarTypingError::Arity)}Step::More(Plan::Cast(target,args[0].clone()))},
                            GoNameLookup::Predeclared{name,kind:GoSourceNameKind::BuiltinFunction}if matches!(name.as_str(),"len"|"min"|"max")=>Step::More(Plan::BuiltinFirst(name,args)),
                            GoNameLookup::Original(b)if b.kind()==GoSourceNameKind::Function=>Step::More(Plan::CallFirst(Box::new(self.header(&r.source,*node,&b)?),args)),
                            _=>return Err(GoScalarTypingError::Unsupported),
                        }
                    }
                    _=>return Err(GoScalarTypingError::Unsupported),
                };Ok(step)
            }
        }
    }
    fn fixed(&mut self,source:&str,node:usize,typ:&str)->Result<Step>{
        let raw=self.one(&Request::raw(source,node))?;
        if raw.typ.is_some()||raw.value.is_some(){return Err(GoScalarTypingError::Type)}
        if !self.compatible_kind(raw.kind,typ){return Err(GoScalarTypingError::Type)}
        let n=self.node(source,node)?.clone();
        Ok(Step::More(match n.kind(){
            "ParenExpr"=>Plan::Forward(Request::value(source,child(&n,"X")?,Some(typ.into()))),
            "UnaryExpr"=>Plan::Unary(attr(&n,"Op").ok_or(GoScalarTypingError::Locator)?.into(),Request::value(source,child(&n,"X")?,Some(typ.into()))),
            "BinaryExpr"=>{let op=attr(&n,"Op").ok_or(GoScalarTypingError::Locator)?;if comparison(op){return Ok(Step::Done(vec![self.atom(typ,false)]))}Plan::BinaryFirst(op.into(),Request::value(source,child(&n,"X")?,Some(typ.into())),if matches!(op,"<<"|">>"){Request::raw(source,child(&n,"Y")?)}else{Request::value(source,child(&n,"Y")?,Some(typ.into()))})},
            "CallExpr"=>{let GoNameLookup::Predeclared{name,kind:GoSourceNameKind::BuiltinFunction}=self.symbol(source,child(&n,"Fun")?)?else{return Err(GoScalarTypingError::Type)};if !matches!(name.as_str(),"min"|"max"){return Err(GoScalarTypingError::Type)}Plan::BuiltinFirst(name,list(&n,"Args").iter().map(|&arg|Request::value(source,arg,Some(typ.into()))).collect())},
            _=>return Err(GoScalarTypingError::Type),
        }))
    }
    fn finish(&mut self,r:&Request,plan:Plan)->Result<Step>{match plan{
        Plan::Forward(child)=>Ok(Step::Done(self.facts(&child)?)),
        Plan::Select(child,index)=>{let facts=self.facts(&child)?;Ok(Step::Done(vec![facts.get(index).ok_or(GoScalarTypingError::Arity)?.clone()]))},
        Plan::Materialize(raw,expected)=>{
            let facts=self.facts(&raw)?;if expected.is_some()&&facts.len()!=1{return Err(GoScalarTypingError::Arity)}
            if facts.len()!=1{if facts.iter().any(|a|a.typ.is_none()){return Err(GoScalarTypingError::Type)}return Ok(Step::Done(facts))}
            let mut a=facts[0].clone();
            let typ=if let Some(expected)=expected{expected}else if let Some(t)=&a.typ{t.clone()}else{self.default_type(a.kind)?};
            if let Some(current)=&a.typ{if current!=&typ{return Err(GoScalarTypingError::Type)}return Ok(Step::Done(vec![a]))}
            if let Some(value)=a.value{let converted=value.convert(self.types[&typ].clone(),false).map_err(constant_error)?;a=self.constant(converted)?;return Ok(Step::Done(vec![a]))}
            Ok(Step::More(Plan::Forward(Request::fixed(&raw.source,raw.expression()?,typ))))
        }
        Plan::Unary(op,arg)=>{let a=self.one(&arg)?;Ok(Step::Done(vec![self.unary(&op,a)?]))},
        Plan::BinaryFirst(op,x,y)=>self.binary_first(op,x,y),
        Plan::BinaryReady(op,x,y)=>{let a=self.one(&x)?;let b=self.one(&y)?;Ok(Step::Done(vec![self.binary(&op,a,b)?]))},
        Plan::Cast(typ,arg)=>{
            let a=self.one(&arg)?;
            if let Some(value)=a.value{return Ok(Step::Done(vec![self.constant(value.convert(self.types[&typ].clone(),true).map_err(constant_error)?)?]))}
            if a.typ.is_none(){return Ok(Step::More(Plan::Forward(Request::value(&arg.source,arg.expression()?,Some(typ)))))}
            let from=&self.types[a.typ.as_ref().ok_or(GoScalarTypingError::Type)?];let to=&self.types[&typ];
            if from.underlying_builtin()!=to.underlying_builtin()&&!(from.integer_shape().is_some()&&(to.integer_shape().is_some()||to.underlying_builtin()=="string")){return Err(GoScalarTypingError::Type)}
            Ok(Step::Done(vec![self.atom(&typ,false)]))
        }
        Plan::BuiltinFirst(name,args)=>self.builtin_first(r,name,args),
        Plan::BuiltinReady(name,args)=>{let values=args.iter().map(|r|self.one(r)).collect::<Result<Vec<_>>>()?;Ok(Step::Done(vec![self.minmax(&name,values)?]))},
        Plan::CallFirst(header,args)=>{
            let facts=self.expand(&args)?;if facts.len()!=header.parameters.len(){return Err(GoScalarTypingError::Arity)}
            if args.len()==1&&facts.len()!=1{self.check_types(&facts,&header.parameters)?;return self.call_result(*header)}
            let next=args.iter().zip(&header.parameters).map(|(arg,typ)|Ok(Request::value(&arg.source,arg.expression()?,Some(typ.clone())))).collect::<Result<Vec<_>>>()?;
            Ok(Step::More(Plan::CallReady(header,next)))
        }
        Plan::CallReady(header,args)=>{let facts=self.expand(&args)?;self.check_types(&facts,&header.parameters)?;self.call_result(*header)},
        Plan::DeclarationFirst(dest,rhs)=>{
            let facts=self.expand(&rhs)?;if facts.len()!=dest.len(){return Err(GoScalarTypingError::Arity)}
            let targets=dest.iter().map(|d|self.destination(d)).collect::<Result<Vec<_>>>()?;
            if rhs.len()==1&&facts.len()!=1{return self.declared(facts,targets)}
            let next=rhs.iter().zip(targets).map(|(arg,typ)|Ok(Request::value(&arg.source,arg.expression()?,typ))).collect::<Result<Vec<_>>>()?;
            Ok(Step::More(Plan::DeclarationReady(dest,next)))
        }
        Plan::DeclarationReady(dest,rhs)=>{let facts=self.expand(&rhs)?;let targets=dest.iter().map(|d|self.destination(d)).collect::<Result<Vec<_>>>()?;self.declared(facts,targets)},
    }}
    fn destination(&self,d:&Destination)->Result<Option<String>>{match d{Destination::Inferred=>Ok(None),Destination::Declared(t)=>Ok(Some(t.clone())),Destination::Reused(r)=>Ok(self.one(r)?.typ)}}
    fn declared(&self,facts:Facts,targets:Vec<Option<String>>)->Result<Step>{if facts.len()!=targets.len(){return Err(GoScalarTypingError::Arity)}let mut result=vec![];for(a,target)in facts.into_iter().zip(targets){let typ=a.typ.ok_or(GoScalarTypingError::Type)?;if target.is_some_and(|t|t!=typ){return Err(GoScalarTypingError::Type)}result.push(self.atom(&typ,true))}Ok(Step::Done(result))}
    fn expand(&self,args:&[Request])->Result<Facts>{let mut result=vec![];for arg in args{let facts=self.facts(arg)?;if facts.is_empty()||(args.len()!=1&&facts.len()!=1){return Err(GoScalarTypingError::Arity)}result.extend(facts)}Ok(result)}
    fn check_types(&self,facts:&[Atom],types:&[String])->Result<()>{if facts.len()!=types.len(){return Err(GoScalarTypingError::Arity)}if facts.iter().zip(types).any(|(a,t)|a.typ.as_ref()!=Some(t)){return Err(GoScalarTypingError::Type)}Ok(())}
    fn call_result(&mut self,header:GoScalarCallHeader)->Result<Step>{let values=header.results.iter().map(|t|self.atom(t,false)).collect();self.calls.insert(header);Ok(Step::Done(values))}
    fn run(mut self,root:Request)->Result<GoScalarTyping>{
        let mut work=vec![Work::Visit(root.clone())];
        while let Some(task)=work.pop(){let(r,step)=match task{
            Work::Visit(r)=>{if self.done.contains_key(&r){continue}if !self.active.insert(r.clone()){return Err(GoScalarTypingError::Cycle)}let step=self.plan(&r)?;(r,step)},
            Work::Finish(r,plan)=>{let step=self.finish(&r,*plan)?;(r,step)},
        };match step{Step::Done(value)=>{self.done.insert(r.clone(),value);self.active.remove(&r);},Step::More(plan)=>{let inputs=plan.inputs();work.push(Work::Finish(r,Box::new(plan)));for input in inputs.into_iter().rev(){work.push(Work::Visit(input))}}}}
        let facts=self.done.remove(&root).ok_or(GoScalarTypingError::Type)?;
        let mut values=vec![];for fact in facts{let id=fact.typ.ok_or(GoScalarTypingError::Type)?;values.push(GoSourceScalarValue{typ:self.types[&id].clone(),constant:fact.value,addressable:fact.addressable})}
        Ok(GoScalarTyping{scopes:self.scopes.clone(),values,calls:self.calls.into_iter().collect()})
    }
}
/// Type a scalar value (or a static call's0..16 scalar results) in default value
/// context. This does not typecheck a containing statement or function body.
pub fn infer_go_scalar_value(scopes:&GoSourceScopes,source:&str,node:usize)->Result<GoScalarTyping>{Engine::new(scopes).run(Request::value(source,node,None))}
/// Check assignment compatibility against one independently derived original
/// scalar type expression. No actual statement relationship is asserted here.
pub fn check_go_scalar_assignment(scopes:&GoSourceScopes,source:&str,node:usize,type_source:&str,type_node:usize)->Result<GoScalarTyping>{let mut engine=Engine::new(scopes);let typ=engine.annotation(type_source,type_node)?;engine.run(Request::value(source,node,Some(typ)))}
/// Derive a real scalar variable's type, checking its complete shared initializer
/// declaration (including reused short-declaration targets) where present.
pub fn infer_go_scalar_binding(scopes:&GoSourceScopes,binding:&GoSourceNameBinding)->Result<GoScalarTyping>{let engine=Engine::new(scopes);let index=engine.binding_index(binding)?;if binding.kind()!=GoSourceNameKind::Variable{return Err(GoScalarTypingError::Type)}engine.run(Request::binding(binding,index))}
