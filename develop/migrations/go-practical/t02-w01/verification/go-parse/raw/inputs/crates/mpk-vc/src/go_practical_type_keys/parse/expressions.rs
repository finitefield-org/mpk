use super::*;
impl Parser{
 pub(super) fn expression(&mut self)->Result<Id>{self.binary(None,1)}
 pub(super) fn exprs(&mut self)->Result<Vec<Id>>{let mut xs=vec![self.expression()?];while self.eat(",").is_some(){xs.push(self.expression()?)}Ok(xs)}
 fn precedence(&self)->u8{match self.kind(){"||"=>1,"&&"=>2,"=="|"!="|"<"|"<="|">"|">="=>3,"+"|"-"|"|"|"^"=>4,"*"|"/"|"%"|"<<"|">>"|"&"|"&^"=>5,_=>0}}
 pub(super) fn binary(&mut self,initial:Option<Id>,min:u8)->Result<Id>{
  let depth=self.nesting;let result=self.binary_inner(initial,min);self.nesting=depth;result
 }
 fn binary_inner(&mut self,initial:Option<Id>,min:u8)->Result<Id>{
  let mut x=if let Some(x)=initial{x}else{self.unary()?};
  loop{self.step_nesting()?;if self.precedence()<min{break}
   let p=self.precedence();let op=self.kind().to_string();self.bump();let y=self.binary(None,p+1)?;let n=self.node("BinaryExpr",self.s(x),self.e(y))?;self.edge(n,"X",x);self.edge(n,"Y",y);self.attr(n,"Op",op);x=n;
  }Ok(x)
 }
 fn unary(&mut self)->Result<Id>{self.nested(|p|p.unary_inner())}
 fn unary_inner(&mut self)->Result<Id>{
  if matches!(self.kind(),"+"|"-"|"!"|"^"|"&"|"~"|"*"|"<-"){
   let start=self.pos();let op=self.kind().to_string();self.bump();let x=self.unary()?;
   if op=="<-"&&self.nk(x)=="ChanType"{
    let mut at=x;let mut arrow=start;loop{let dir=self.nodes[at].attrs["Dir"].clone();if dir=="2"{return self.err()}let value=self.child(at,"Value").unwrap();let next_arrow=if dir=="1"{self.lexed.tokens().iter().find(|t|t.start()>=self.s(at)+4&&t.end()<=self.s(value)&&t.kind()=="<-").map(|t|t.start())}else{None};self.nodes[at].start=arrow;self.attr(at,"Dir",2);if dir!="1"{break}at=value;arrow=next_arrow.ok_or(GoParseError{offset:start,kind:GoParseErrorKind::Syntax})?;if self.nk(at)!="ChanType"{return self.err()}}
    return Ok(x)
   }
   let n=self.node(if op=="*"{"StarExpr"}else{"UnaryExpr"},start,self.e(x))?;self.edge(n,"X",x);if op!="*"{self.attr(n,"Op",op)}return Ok(n)
  }
  let x=self.operand()?;self.primary(x)
 }
 fn operand(&mut self)->Result<Id>{
  match self.kind(){
   "IDENT"=>self.ident(),"INT"|"FLOAT"|"IMAG"|"CHAR"|"STRING"=>self.literal(),
   "("=>{let start=self.bump();self.level+=1;let x=self.expression()?;self.level-=1;let end=self.close(")")?;let n=self.node("ParenExpr",start,end)?;self.edge(n,"X",x);Ok(n)},
   "func"=>{let t=self.function_type(true)?;if self.is("{"){self.level+=1;let body=self.block()?;self.level-=1;let n=self.node("FuncLit",self.s(t),self.e(body))?;self.edge(n,"Type",t);self.edge(n,"Body",body);Ok(n)}else{Ok(t)}},
   _=>self.typ(),
  }
 }
 pub(super) fn primary(&mut self,x:Id)->Result<Id>{let depth=self.nesting;let result=self.primary_inner(x);self.nesting=depth;result}
 fn primary_inner(&mut self,mut x:Id)->Result<Id>{
  loop{self.step_nesting()?;match self.kind(){
   "."=>{self.bump();if self.eat("(").is_some(){let typ=if self.eat("type").is_some(){None}else{Some(self.typ()?)};let end=self.close(")")?;let n=self.node("TypeAssertExpr",self.s(x),end)?;self.edge(n,"X",x);self.optional(n,"Type",typ);x=n}else{let sel=self.ident()?;let n=self.node("SelectorExpr",self.s(x),self.e(sel))?;self.edge(n,"X",x);self.edge(n,"Sel",sel);x=n}},
   "["=>{x=self.index(x)?},
   "("=>{self.bump();self.level+=1;let mut args=vec![];let mut ellipsis=false;if !self.is(")"){
    loop{args.push(self.expression()?);if self.eat("...").is_some(){ellipsis=true;if self.eat(",").is_some(){}break}if self.eat(",").is_none()||self.is(")"){break}}
   }self.level-=1;let end=self.close(")")?;let n=self.node("CallExpr",self.s(x),end)?;self.edge(n,"Fun",x);self.list(n,"Args",args);self.attr(n,"Ellipsis",ellipsis);x=n},
   "{"=>{
    let mut unparen=x;while self.nk(unparen)=="ParenExpr"{unparen=self.child(unparen,"X").unwrap()}
    match self.nk(unparen){"Ident"|"SelectorExpr"|"IndexExpr"|"IndexListExpr"=>if self.level<0{return Ok(x)},"ArrayType"|"StructType"|"MapType"=>{},_=>return Ok(x)}
    if unparen!=x{return self.err()}x=self.composite(Some(x))?;
   },_=>return Ok(x),
  }}
 }
 pub(super) fn index(&mut self,x:Id)->Result<Id>{
  self.expect("[")?;self.level+=1;
  let low=if self.is(":"){None}else{Some(self.expression()?)};
  if self.eat(":").is_some(){
   let high=if matches!(self.kind(),":"|"]"){None}else{Some(self.expression()?)};
   let slice3=self.eat(":").is_some();let max=if slice3{if high.is_none()||self.is("]"){return self.err()}Some(self.expression()?)}else{None};let end=self.close("]")?;self.level-=1;
   let n=self.node("SliceExpr",self.s(x),end)?;self.edge(n,"X",x);self.optional(n,"Low",low);self.optional(n,"High",high);self.optional(n,"Max",max);self.attr(n,"Slice3",slice3);Ok(n)
  }else{
   let mut indices=vec![low.ok_or(GoParseError{offset:self.pos(),kind:GoParseErrorKind::Syntax})?];while self.eat(",").is_some(){if self.is("]"){break}indices.push(self.expression()?)}let end=self.close("]")?;self.level-=1;self.pack_index(x,indices,end)
  }
 }
 pub(super) fn pack_index(&mut self,x:Id,indices:Vec<Id>,end:usize)->Result<Id>{
  if indices.is_empty(){return self.err()}let single=indices.len()==1;let n=self.node(if single{"IndexExpr"}else{"IndexListExpr"},self.s(x),end)?;self.edge(n,"X",x);self.list(n,if single{"Index"}else{"Indices"},indices);Ok(n)
 }
 fn element(&mut self)->Result<Id>{if self.is("{"){self.composite(None)}else{self.expression()}}
 fn composite(&mut self,typ:Option<Id>)->Result<Id>{
  let start=self.expect("{")?;self.level+=1;let mut elts=vec![];
  while !self.is("}"){
   let mut x=self.element()?;if self.eat(":").is_some(){let value=self.element()?;let n=self.node("KeyValueExpr",self.s(x),self.e(value))?;self.edge(n,"Key",x);self.edge(n,"Value",value);x=n}elts.push(x);
   if self.eat(",").is_none(){break}
  }let end=self.close("}")?;self.level-=1;let n=self.node("CompositeLit",typ.map_or(start,|t|self.s(t)),end)?;self.optional(n,"Type",typ);self.list(n,"Elts",elts);self.attr(n,"Incomplete",false);Ok(n)
 }
}
