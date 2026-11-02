//! Actual source int result carrier for dynamic bounded string length.
//! Representation/signed-value laws do not establish original CallExpr
//! evaluation, operand-once execution or canonical capture/context authority.
use super::syntax::*;
use super::{composition,word,Builder,GoOrdinaryError,Result,MAX_BYTES};
use super::{generate_go_practical_source_string_catalogue,generate_go_practical_word_signed_bounds_foundation,generate_go_practical_word_shift_foundation,GoSourceStringCatalogue,GoSourceStringRow};
use crate::go_practical_type_keys::{GoSourceTypeKey,GoSourceValueDomain};
mod math;
const MATH:&str="Mpk.Go.Practical.String.GoLength";
fn own_math(n:&str,args:Vec<Expr>)->Expr{call(&format!("{MATH}.{n}"),args)}
fn num(n:&str,args:Vec<Expr>)->Expr{call(&format!("Mpk.Go.Practical.Word.Numeric.{n}"),args)}
fn int(n:&str,args:Vec<Expr>)->Expr{call(&format!("Mpk.Go.Practical.Int.{n}"),args)}
fn half(d:usize)->Expr{call(&format!("Mpk.Go.Practical.Word.SignedBounds.Half{d}"),vec![])}
fn power(n:usize)->Expr{num("PowerTwo",vec![numeral(n)])}
fn numeral(n:usize)->Expr{(0..n).fold(zero(),|n,_|succ(n))}
fn wt()->Expr{word::word_type(64).expect("frozen Go int width")}
fn integer_type()->Expr{pi("_part",boolean(),nat())}
fn capacity()->Expr{call("Mpk.Go.Practical.FiniteVector.D8.Capacity",vec![])}
fn gl(n:&str,args:Vec<Expr>)->Expr{call(&format!("Mpk.Go.Practical.String.{n}"),args)}
fn generic_word(s:Expr)->Expr{gl("Length",vec![s])}
fn value(w:Expr)->Expr{num("Value6",vec![w])}
fn signed(w:Expr)->Expr{num("W64.Signed",vec![w])}
fn bit63(w:Expr)->Expr{word::at(w,64,63).expect("actual top bit")}

#[derive(Clone)]
pub struct GoSourceStringLengthCatalogue{strings:GoSourceStringCatalogue,result:Option<String>,math:[mpk_cert::encode::Certificate;2]}
#[derive(Clone,Debug,Eq,PartialEq)]
pub struct GoSourceStringLengthRow{prior:GoSourceStringRow,result:String,module:String,certificate:Vec<u8>}
impl GoSourceStringLengthRow{
    pub fn prior_string_row(&self)->&GoSourceStringRow{&self.prior}
    pub fn source_result_type_id(&self)->&str{&self.result}
    pub fn module_name(&self)->&str{&self.module}
    pub fn certificate_bytes(&self)->&[u8]{&self.certificate}
}
impl GoSourceStringLengthCatalogue{
    pub fn strings(&self)->&GoSourceStringCatalogue{&self.strings}
    pub fn source_int_type_id(&self)->Option<&str>{self.result.as_deref()}
    pub fn generate_row(&self,string:&str)->Result<GoSourceStringLengthRow>{
        let result=self.result.as_deref().ok_or(GoOrdinaryError::Linkage)?;
        let prior=self.strings.generate_row(string)?;
        let source=mpk_cert::decode_canonical_certificate(prior.certificate_bytes()).map_err(|_|GoOrdinaryError::Foundation)?;
        let mut builder=composition::source_string_length_prerequisites(&[source,self.math[0].clone(),self.math[1].clone()])?;
        math::append(&mut builder)?;
        let module=format!("{}.GoLength.T{result}",prior.module_name());
        Length{builder:&mut builder,prior:&prior,result,module:&module}.append()?;
        builder.certificate.module=module.clone();
        Ok(GoSourceStringLengthRow{prior,result:result.into(),module,certificate:builder.finish()?})
    }
    pub fn import_row(&self,bytes:&[u8],string:&str)->Result<GoSourceStringLengthRow>{
        if bytes.len()>MAX_BYTES{return Err(GoOrdinaryError::Limit)}
        let expected=self.generate_row(string)?;if bytes!=expected.certificate_bytes(){return Err(GoOrdinaryError::Linkage)}Ok(expected)
    }
}
/// No source int or byte type is fabricated. The dynamic result is the actual
/// builtin int key of the selected source domain, never a named integer alias.
pub fn generate_go_practical_source_string_length_catalogue(domain:&GoSourceValueDomain)->Result<GoSourceStringLengthCatalogue>{
    let strings=generate_go_practical_source_string_catalogue(domain)?;
    let result=strings.source_types().iter().find_map(|k|matches!(k,GoSourceTypeKey::Builtin{name,width:Some(64),signed:Some(true),..} if name=="int").then(||k.id().to_string()));
    let bounds=generate_go_practical_word_signed_bounds_foundation()?;let shifts=generate_go_practical_word_shift_foundation()?;
    let decode=|raw:&[u8]|mpk_cert::decode_canonical_certificate(raw).map_err(|_|GoOrdinaryError::Foundation);
    Ok(GoSourceStringLengthCatalogue{strings,result,math:[decode(bounds.certificate_bytes())?,decode(shifts.certificate_bytes())?]})
}
struct Length<'a>{builder:&'a mut Builder,prior:&'a GoSourceStringRow,result:&'a str,module:&'a str}
impl Length<'_>{
    fn source(&self,id:&str,n:&str)->Expr{constant(&format!("{}.T{id}.{n}",self.prior.source_layout_module_name()))}
    fn st(&self)->Expr{self.source(self.prior.source_string_type_id(),"Carrier")}
    fn rt(&self)->Expr{self.source(self.result,"Carrier")}
    fn own(&self,n:&str,args:Vec<Expr>)->Expr{call(&format!("{}.{n}",self.module),args)}
    fn string(&self,n:&str,args:Vec<Expr>)->Expr{call(&format!("{}.{n}",self.prior.module_name()),args)}
    fn emit(&mut self,n:&str,args:&[Binder],ty:Expr,body:Expr,law:bool)->Result<()>{emit(self.builder,&format!("{}.{n}",self.module),args,ty,body,law)}
    fn append(&mut self)->Result<()>{
        let s=var("s");let sa=[("s",self.st())];let raw=generic_word(self.string("ToMath",vec![s.clone()]));
        self.emit("ToWord",&[("n",self.rt())],wt(),var("n"),false)?;
        self.emit("FromWord",&[("w",wt())],self.rt(),var("w"),false)?;
        self.emit("ToFromWord",&[("w",wt())],eq(wt(),self.own("ToWord",vec![self.own("FromWord",vec![var("w")])]),var("w")),refl(wt(),var("w")),true)?;
        self.emit("FromToWord",&[("n",self.rt())],eq(self.rt(),self.own("FromWord",vec![self.own("ToWord",vec![var("n")])]),var("n")),refl(self.rt(),var("n")),true)?;
        self.emit("Len",&sa,self.rt(),self.own("FromWord",vec![self.string("Length",vec![s.clone()])]),false)?;
        let len=self.own("Len",vec![s.clone()]);let lw=self.own("ToWord",vec![len.clone()]);
        self.emit("LenDefinition",&sa,eq(self.rt(),len.clone(),self.own("FromWord",vec![raw.clone()])),refl(self.rt(),self.own("FromWord",vec![raw.clone()])),true)?;
        self.emit("LenWord",&sa,eq(wt(),lw.clone(),raw.clone()),self.own("ToFromWord",vec![raw.clone()]),true)?;
        self.emit("Natural",&sa,nat(),value(lw.clone()),false)?;
        let size=self.string("Size",vec![s.clone()]);
        let decoded=Equality::natural(value(raw.clone()),size.clone(),refl(nat(),value(raw.clone())));
        let e=Equality{ty:wt(),left:lw.clone(),right:raw.clone(),proof:self.own("LenWord",vec![s.clone()])}.mapped(lam("_word",wt(),value(var("_word"))),self.own("Natural",vec![s.clone()]),value(raw.clone()),nat()).then(decoded.clone())?;
        self.emit("LenNatural",&sa,eq(e.ty,e.left,e.right),e.proof,true)?;
        self.emit("AsInteger",&sa,integer_type(),signed(lw.clone()),false)?;
        self.emit("AsIntegerDefinition",&sa,eq(integer_type(),self.own("AsInteger",vec![s.clone()]),signed(lw.clone())),refl(integer_type(),signed(lw.clone())),true)?;
        let fits=[("s",self.st()),("fits",self.string("LengthFits",vec![s.clone()]))];
        let clear=own_math("BoundedSign",vec![raw.clone(),var("fits")]);
        let e=Equality{ty:wt(),left:lw.clone(),right:raw.clone(),proof:self.own("LenWord",vec![s.clone()])}.mapped(lam("_word",wt(),bit63(var("_word"))),bit63(lw.clone()),bit63(raw.clone()),boolean()).then(Equality::boolean(bit63(raw.clone()),bit(false),clear.clone()))?;
        self.emit("FitsNonnegative",&fits,eq(e.ty,e.left,e.right),e.proof,true)?;
        let e=Equality{ty:wt(),left:lw.clone(),right:raw.clone(),proof:self.own("LenWord",vec![s.clone()])}.mapped(lam("_word",wt(),signed(var("_word"))),self.own("AsInteger",vec![s.clone()]),signed(raw.clone()),integer_type())
            .then(Equality{ty:integer_type(),left:signed(raw.clone()),right:int("FromNat",vec![value(raw.clone())]),proof:own_math("SignedAsNat",vec![raw,clear])})?
            .then(decoded.mapped(lam("_natural",nat(),int("FromNat",vec![var("_natural")])),int("FromNat",vec![value(generic_word(self.string("ToMath",vec![s.clone()]))) ]),int("FromNat",vec![size]),integer_type()))?;
        self.emit("FitsInteger",&fits,eq(e.ty,e.left,e.right),e.proof,true)?;
        let valid=[("s",self.st()),("valid",self.string("Valid",vec![s.clone()]))];
        for (name,ty) in [("ValidNonnegative",eq(boolean(),bit63(lw),bit(false))),("ValidInteger",eq(integer_type(),self.own("AsInteger",vec![s.clone()]),int("FromNat",vec![self.string("Size",vec![s.clone()])])))]{
            self.emit(name,&valid,ty,self.own(if name=="ValidInteger"{"FitsInteger"}else{"FitsNonnegative"},vec![s.clone(),self.string("ValidUpper",vec![s.clone(),var("valid")])]),true)?;
        }
        let z=self.source(self.prior.source_string_type_id(),"Zero");let zi=self.source(self.result,"Zero");
        self.emit("ZeroWordToSource",&[],eq(self.rt(),self.own("FromWord",vec![gl("Length",vec![gl("Zero",vec![])])]),zi.clone()),refl(self.rt(),zi.clone()),true)?;
        let a=Equality{ty:self.rt(),left:self.own("Len",vec![z.clone()]),right:self.own("FromWord",vec![generic_word(self.string("ToMath",vec![z.clone()]))]),proof:self.own("LenDefinition",vec![z.clone()])};
        let b=Equality{ty:wt(),left:self.string("Length",vec![z.clone()]),right:gl("Length",vec![gl("Zero",vec![])]),proof:self.string("ZeroLength",vec![])}.mapped(lam("_word",wt(),self.own("FromWord",vec![var("_word")])),a.right.clone(),self.own("FromWord",vec![gl("Length",vec![gl("Zero",vec![])])]),self.rt());
        let e=a.then(b)?.then(Equality{ty:self.rt(),left:self.own("FromWord",vec![gl("Length",vec![gl("Zero",vec![])])]),right:zi,proof:self.own("ZeroWordToSource",vec![])})?;
        self.emit("LenZero",&[],eq(e.ty,e.left,e.right),e.proof,true)
    }
}
