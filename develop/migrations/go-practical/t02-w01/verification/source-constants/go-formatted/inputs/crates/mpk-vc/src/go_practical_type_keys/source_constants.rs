//! Exact scalar constants derived from owned original syntax and lexical
//! bindings. Candidate type identities are not complete capture/program proofs.
use super::*;
mod evaluate;
mod integer;
use integer::Integer;
pub use evaluate::{evaluate_go_constant,evaluate_go_constant_binding};
#[derive(Clone,Copy,Debug,Eq,PartialEq)]
pub enum GoConstantError {Locator,Scope,NotConstant,Unsupported,Cycle,Type,Precision,Representability,DivideByZero,ShiftCount,Resource}
impl std::fmt::Display for GoConstantError{fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{write!(f,"Go constant error: {self:?}")}}
impl std::error::Error for GoConstantError{}
type Result<T>=std::result::Result<T,GoConstantError>;
#[derive(Clone,Copy,Debug,Eq,PartialEq)]
pub enum GoConstantKind{Integer,Rune,Bool,String}
impl GoConstantKind{pub fn as_str(self)->&'static str{match self{Self::Integer=>"integer",Self::Rune=>"rune",Self::Bool=>"bool",Self::String=>"string"}}}
#[derive(Clone,Debug,Eq,PartialEq)]
pub struct GoConstantScalarType {
    key:GoSourceTypeKey,
    underlying:String,
    declaration:Option<GoTypeDeclarationKey>,
}
impl GoConstantScalarType{
    pub fn source_type_key(&self)->&GoSourceTypeKey{&self.key}
    pub fn underlying_builtin(&self)->&str{&self.underlying}
    pub fn declaration(&self)->Option<&GoTypeDeclarationKey>{self.declaration.as_ref()}
    fn builtin(name:&str)->Result<Self>{
        let canonical=match name{"byte"=>"uint8","rune"=>"int32",n=>n};
        let(width,signed)=match canonical{
            "bool"|"string"=>(None,None),"int8"=>(Some(8),Some(true)),"int16"=>(Some(16),Some(true)),"int32"=>(Some(32),Some(true)),"int64"|"int"=>(Some(64),Some(true)),"uint8"=>(Some(8),Some(false)),"uint16"=>(Some(16),Some(false)),"uint32"=>(Some(32),Some(false)),"uint64"|"uint"=>(Some(64),Some(false)),_=>return Err(GoConstantError::Unsupported)};
        let mut key=GoSourceTypeKey::Builtin{schema:TYPE_SCHEMA.into(),id:String::new(),name:canonical.into(),width,signed};
        let id=identity(&key,TYPE_DOMAIN,"id").map_err(|_|GoConstantError::Type)?;
        if let GoSourceTypeKey::Builtin{id:field,..}=&mut key{*field=id}
        Ok(Self{key,underlying:canonical.into(),declaration:None})
    }
    fn integer_shape(&self)->Option<(u8,bool)>{
        match self.underlying.as_str(){"int8"=>Some((8,true)),"int16"=>Some((16,true)),"int32"=>Some((32,true)),"int64"|"int"=>Some((64,true)),"uint8"=>Some((8,false)),"uint16"=>Some((16,false)),"uint32"=>Some((32,false)),"uint64"|"uint"=>Some((64,false)),_=>None}
    }
}
#[derive(Clone,Debug,Eq,PartialEq)]
enum Value{Integer(Integer),Bool(bool),Bytes(Vec<u8>)}
#[derive(Clone,Debug,Eq,PartialEq)]
pub struct GoEvaluatedConstant{kind:GoConstantKind,value:Value,typ:Option<GoConstantScalarType>}
impl GoEvaluatedConstant{
    pub fn kind(&self)->GoConstantKind{self.kind}
    pub fn scalar_type(&self)->Option<&GoConstantScalarType>{self.typ.as_ref()}
    pub fn integer_decimal(&self)->Option<String>{if let Value::Integer(n)=&self.value{Some(n.decimal())}else{None}}
    pub fn boolean(&self)->Option<bool>{if let Value::Bool(b)=self.value{Some(b)}else{None}}
    pub fn bytes(&self)->Option<&[u8]>{if let Value::Bytes(b)=&self.value{Some(b)}else{None}}
    fn integer(n:Integer,rune:bool)->Self{Self{kind:if rune{GoConstantKind::Rune}else{GoConstantKind::Integer},value:Value::Integer(n),typ:None}}
    fn boolean_value(b:bool)->Self{Self{kind:GoConstantKind::Bool,value:Value::Bool(b),typ:None}}
    fn checked(self)->Result<Self>{
        match (&self.typ,&self.value){
            (None,Value::Integer(n)) if n.bits()>512=>return Err(GoConstantError::Precision),
            (Some(typ),Value::Integer(n))=>{
                let(w,signed)=typ.integer_shape().ok_or(GoConstantError::Type)?;
                let power=Integer::from_u64(1).shl((if signed{w-1}else{w})as usize);
                let valid=if signed{n>=&power.neg()&&n<&power}else{!n.is_negative()&&n<&power};
                if !valid{return Err(GoConstantError::Representability)}
            }
            (Some(typ),Value::Bool(_))if typ.underlying!="bool"=>return Err(GoConstantError::Type),
            (Some(typ),Value::Bytes(_))if typ.underlying!="string"=>return Err(GoConstantError::Type),
            _=>(),
        }
        Ok(self)
    }
    fn convert(mut self,target:GoConstantScalarType,explicit:bool)->Result<Self>{
        if !explicit && self.typ.as_ref().is_some_and(|typ|typ.key.id()!=target.key.id()) {
            return Err(GoConstantError::Type);
        }
        match (&self.value,target.underlying.as_str()){
            (Value::Integer(n),"string") if explicit=>{
                let scalar=n.as_u64().and_then(|n|u32::try_from(n).ok()).and_then(char::from_u32).unwrap_or('\u{fffd}');
                let mut raw=[0;4];self.value=Value::Bytes(scalar.encode_utf8(&mut raw).as_bytes().to_vec());self.kind=GoConstantKind::String;
            }
            (Value::Integer(_),_)if target.integer_shape().is_some()=>self.kind=GoConstantKind::Integer,
            (Value::Bool(_),"bool")|(Value::Bytes(_),"string")=>(),
            _=>return Err(GoConstantError::Type),
        }
        self.typ=Some(target);self.checked()
    }
}
fn merge(mut a:GoEvaluatedConstant,mut b:GoEvaluatedConstant)->Result<(GoEvaluatedConstant,GoEvaluatedConstant)>{
    match(&a.typ,&b.typ){
        (Some(x),Some(y))if x.key.id()!=y.key.id()=>return Err(GoConstantError::Type),
        (Some(x),None)=>b=b.clone().convert(x.clone(),false)?,
        (None,Some(y))=>a=a.clone().convert(y.clone(),false)?,
        _=>(),
    }
    Ok((a,b))
}
fn unary(op:&str,mut a:GoEvaluatedConstant)->Result<GoEvaluatedConstant>{
    a.value=match(op,&a.value){
        ("+",Value::Integer(n))=>Value::Integer(n.clone()),
        ("-",Value::Integer(n))=>Value::Integer(n.neg()),
        ("^",Value::Integer(n))=>Value::Integer(n.complement(a.typ.as_ref().and_then(|t|t.integer_shape()).and_then(|(w,signed)|(!signed).then_some(w)))),
        ("!",Value::Bool(b))=>Value::Bool(!b),
        _=>return Err(GoConstantError::Type),
    };
    a.checked()
}
fn binary(op:&str,a:GoEvaluatedConstant,b:GoEvaluatedConstant)->Result<GoEvaluatedConstant>{
    if matches!(op,"<<"|">>"){
        let(Value::Integer(x),Value::Integer(y))=(&a.value,&b.value)else{return Err(GoConstantError::Type)};
        let count=y.as_u64().filter(|&n|n<=1074).ok_or(GoConstantError::ShiftCount)? as usize;
        let n=if op=="<<"{x.shl(count)}else{x.shr(count)};
        return GoEvaluatedConstant{value:Value::Integer(n),..a}.checked();
    }
    let(a,b)=merge(a,b)?;
    let compare=matches!(op,"=="|"!="|"<"|"<="|">"|">=");
    let order=match(&a.value,&b.value){
        (Value::Integer(x),Value::Integer(y))=>Some(x.cmp(y)),
        (Value::Bytes(x),Value::Bytes(y))=>Some(x.cmp(y)),
        (Value::Bool(x),Value::Bool(y))if matches!(op,"=="|"!=")=>Some(x.cmp(y)),
        _=>None,
    };
    if compare{
        let order=order.ok_or(GoConstantError::Type)?;
        return Ok(GoEvaluatedConstant::boolean_value(match op{"=="=>order.is_eq(),"!="=>!order.is_eq(),"<"=>order.is_lt(),"<="=>!order.is_gt(),">"=>order.is_gt(),">="=>!order.is_lt(),_=>unreachable!()}));
    }
    let value=match(&a.value,&b.value){
        (Value::Integer(x),Value::Integer(y))=>Value::Integer(match op{
            "+"=>x.add(y),"-"=>x.sub(y),"*"=>x.mul(y),"/"=>x.div_rem(y).ok_or(GoConstantError::DivideByZero)?.0,"%"=>x.div_rem(y).ok_or(GoConstantError::DivideByZero)?.1,
            "&"|"|"|"^"|"&^"=>x.bit_op(y,op),_=>return Err(GoConstantError::Type)}),
        (Value::Bool(x),Value::Bool(y))=>Value::Bool(match op{"&&"=>*x&&*y,"||"=>*x||*y,_=>return Err(GoConstantError::Type)}),
        (Value::Bytes(x),Value::Bytes(y))if op=="+"=>{
            let total=x.len().checked_add(y.len()).ok_or(GoConstantError::Resource)?;
            let mut out=Vec::new();out.try_reserve_exact(total).map_err(|_|GoConstantError::Resource)?;out.extend_from_slice(x);out.extend_from_slice(y);Value::Bytes(out)
        }
        _=>return Err(GoConstantError::Type),
    };
    let kind=if a.typ.is_none()&&matches!(value,Value::Integer(_))&&(a.kind==GoConstantKind::Rune||b.kind==GoConstantKind::Rune){GoConstantKind::Rune}else{a.kind};
    GoEvaluatedConstant{kind,value,typ:a.typ}.checked()
}
fn attr<'a>(n:&'a GoSyntaxNode,key:&str)->Option<&'a str>{n.attributes().find_map(|(k,v)|(k==key).then_some(v))}
fn list<'a>(n:&'a GoSyntaxNode,role:&str)->&'a [usize]{n.edges().iter().find(|e|e.role()==role).map_or(&[],|e|e.targets())}
fn child(n:&GoSyntaxNode,role:&str)->Result<usize>{let xs=list(n,role);if xs.len()==1{Ok(xs[0])}else{Err(GoConstantError::Type)}}

// Whole syntax was independently lexed before this decoder runs. Keep string
// byte escapes distinct from Unicode escapes and from rune scalar values.
fn quoted(raw:&[u8])->Result<(Vec<u8>,Option<u32>)>{
    if raw.len()<2{return Err(GoConstantError::Type)}
    let quote=raw[0];if quote==b'`'{return Ok((raw[1..raw.len()-1].iter().copied().filter(|&b|b!=b'\r').collect(),None))}
    if !matches!(quote,b'"'|b'\''){return Err(GoConstantError::Type)}
    let mut at=1;let mut out=vec![];let mut scalars=vec![];
    while at<raw.len()-1{
        if raw[at]!=b'\\'{
            let text=std::str::from_utf8(&raw[at..raw.len()-1]).map_err(|_|GoConstantError::Type)?;
            let c=text.chars().next().ok_or(GoConstantError::Type)?;at+=c.len_utf8();let mut bytes=[0;4];out.extend_from_slice(c.encode_utf8(&mut bytes).as_bytes());scalars.push(c as u32);continue;
        }
        at+=1;let e=*raw.get(at).ok_or(GoConstantError::Type)?;at+=1;
        let direct=match e{b'a'=>Some(7),b'b'=>Some(8),b'f'=>Some(12),b'n'=>Some(10),b'r'=>Some(13),b't'=>Some(9),b'v'=>Some(11),b'\\'=>Some(b'\\'),b'"'=>Some(b'"'),b'\''=>Some(b'\''),_=>None};
        if let Some(v)=direct{out.push(v);scalars.push(v as u32);continue}
        let(base,count,mut value,unicode)=match e{b'0'..=b'7'=>(8,2,(e-b'0')as u32,false),b'x'=>(16,2,0,false),b'u'=>(16,4,0,true),b'U'=>(16,8,0,true),_=>return Err(GoConstantError::Type)};
        for _ in 0..count{let c=*raw.get(at).ok_or(GoConstantError::Type)? as char;at+=1;value=value.checked_mul(base).and_then(|n|n.checked_add(c.to_digit(base)?)).ok_or(GoConstantError::Type)?}
        if unicode{let c=char::from_u32(value).ok_or(GoConstantError::Type)?;let mut buf=[0;4];out.extend_from_slice(c.encode_utf8(&mut buf).as_bytes())}else{out.push(u8::try_from(value).map_err(|_|GoConstantError::Type)?)}
        scalars.push(value);
    }
    let rune=if quote==b'\''{if scalars.len()!=1{return Err(GoConstantError::Type)}Some(scalars[0])}else{None};Ok((out,rune))
}
