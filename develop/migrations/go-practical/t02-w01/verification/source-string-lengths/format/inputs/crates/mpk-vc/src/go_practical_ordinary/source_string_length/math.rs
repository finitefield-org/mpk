//! Unbounded Nat order and balanced 2^63 threshold; no length truncation.
use super::*;
use super::super::polynomial::{mul_left,mul_right};
fn sh(n:&str,args:Vec<Expr>)->Expr{call(&format!("Mpk.Go.Practical.Word.Shifts.{n}"),args)}
fn idx(n:&str,args:Vec<Expr>)->Expr{call(&format!("Mpk.Go.Practical.Word.Index.{n}"),args)}
fn sb(n:&str,args:Vec<Expr>)->Expr{call(&format!("Mpk.Go.Practical.Word.SignedBounds.{n}"),args)}
fn weight(d:usize)->Expr{num(&format!("Weight{d}"),vec![])}
fn nr(a:Expr,e:Equality)->Equality{e.clone().mapped(lam("_n",nat(),less(a.clone(),var("_n"))),less(a.clone(),e.left),less(a,e.right),boolean())}
fn decide(flag:Expr,target:Expr,no:Expr,yes:Expr)->Expr{app(call("Std.Bool.cases",vec![lam("_tag",boolean(),pi("_branch",eq(boolean(),flag.clone(),var("_tag")),target)),lam("_branch",eq(boolean(),flag.clone(),bit(false)),no),lam("_branch",eq(boolean(),flag.clone(),bit(true)),yes),flag.clone()]),vec![refl(boolean(),flag)])}
fn add_numerals(a:usize,b:usize)->Result<Equality>{
    if b==0{return Ok(Equality::natural(add(numeral(a),zero()),numeral(a),nc("AddZero",vec![numeral(a)])))}
    Equality::natural(add(numeral(a),numeral(b)),succ(add(numeral(a),numeral(b-1))),nc("AddSucc",vec![numeral(a),numeral(b-1)])).then(add_numerals(a,b-1)?.on_succ())
}
fn map_power(e:Equality)->Equality{e.clone().mapped(lam("_n",nat(),num("PowerTwo",vec![var("_n")])),num("PowerTwo",vec![e.left]),num("PowerTwo",vec![e.right]),nat())}
struct Math<'a>{b:&'a mut Builder,serial:usize}
impl Math<'_>{
    fn theorem(&mut self,n:&str,args:&[Binder],e:Equality)->Result<()>{emit(self.b,&format!("{MATH}.{n}"),args,eq(e.ty,e.left,e.right),e.proof,true)}
    fn polynomial(&mut self,left:Expr,right:Expr)->Result<Equality>{super::super::polynomial::prove_named(self.b,&[],&mut self.serial,MATH,left,right)}
    fn append(&mut self)->Result<()>{
        let (a,b,c)=(var("a"),var("b"),var("c"));
        let flag=less(a.clone(),c.clone());let goal=eq(boolean(),flag.clone(),bit(true));
        let contradiction=Equality::boolean(less(b.clone(),a.clone()),bit(false),var("upper")).reverse().then(Equality::boolean(less(b.clone(),a.clone()),bit(true),idx("UpperBound",vec![a.clone(),c.clone(),b.clone(),var("_branch"),var("strict") ])))?;
        self.theorem("AtMostBelow",&[("a",nat()),("b",nat()),("c",nat()),("upper",eq(boolean(),less(b.clone(),a.clone()),bit(false))),("strict",eq(boolean(),less(b.clone(),c.clone()),bit(true)))],Equality::boolean(flag.clone(),bit(true),decide(flag,goal.clone(),bc("FalseElim",vec![goal,contradiction.proof]),var("_branch"))))?;
        let tail=sub(b.clone(),one());
        let e=Equality::natural(b.clone(),add(one(),tail.clone()),sh("CountPositiveDecomposition",vec![b.clone(),var("positive")]))
            .then(Equality::natural(add(one(),tail.clone()),add(tail.clone(),one()),nc("AddComm",vec![one(),tail.clone()])))?
            .then(Equality::natural(add(tail.clone(),one()),succ(tail.clone()),nc("AddOne",vec![tail.clone()])))?;
        let e=nr(a.clone(),e.on_right(a.clone())).then(Equality::boolean(less(a.clone(),add(a.clone(),succ(tail.clone()))),bit(true),call("Mpk.Go.Practical.Word.Order.LessPositiveExtension",vec![a.clone(),tail])))?;
        self.theorem("PositiveAdd",&[("a",nat()),("b",nat()),("positive",eq(boolean(),less(zero(),b),bit(true)))],e)?;
        for d in 0..=6{
            let e=if d==0{
                Equality::natural(half(0),one(),sb("HalfDefinition0",vec![])).then(Equality::natural(power(0),one(),num("PowerTwoZero",vec![])).reverse())?
            }else{
                let h=d-1;let p=1<<h;let prev=p-1;
                Equality::natural(half(d),mul(weight(h),half(h)),sb(&format!("HalfDefinition{d}"),vec![]))
                    .then(mul_left(Equality::natural(power(p),weight(h),num(&format!("PowerTwoWeight{h}"),vec![])).reverse(),half(h)))?
                    .then(mul_right(power(p),Equality::natural(half(h),power(prev),own_math(&format!("HalfPower{h}"),vec![]))))?
                    .then(Equality::natural(num("PowerTwo",vec![add(numeral(p),numeral(prev))]),mul(power(p),power(prev)),num("PowerTwoAdd",vec![numeral(p),numeral(prev)])).reverse())?
                    .then(map_power(add_numerals(p,prev)?))?
            };
            self.theorem(&format!("HalfPower{d}"),&[],e)?;
        }
        let p=power(8);let minus=sh("PowerMinusOne",vec![numeral(54)]);let twice=add(one(),one());let rest=mul(mul(twice.clone(),p.clone()),minus.clone());let gap=add(p.clone(),rest.clone());
        let split=map_power(add_numerals(8,55)?).reverse()
            .then(Equality::natural(num("PowerTwo",vec![add(numeral(8),numeral(55))]),mul(p.clone(),power(55)),num("PowerTwoAdd",vec![numeral(8),numeral(55)])))?
            .then(mul_right(p.clone(),Equality::natural(power(55),mul(twice.clone(),power(54)),num("PowerTwoSucc",vec![numeral(54)]))))?
            .then(mul_right(p.clone(),mul_right(twice,Equality::natural(add(minus.clone(),one()),power(54),sh("PowerMinusBalance",vec![numeral(54)])).reverse())))?;
        let e=split.then(self.polynomial(mul(p.clone(),mul(add(one(),one()),add(minus,one()))),add(p.clone(),gap.clone()))?)?;
        self.theorem("Power63Gap",&[],e.clone())?;
        let positive=idx("UpperBound",vec![gap.clone(),p.clone(),zero(),sh("CountNoLessExtension",vec![p.clone(),rest]),sh("PowerPositive",vec![numeral(8)])]);
        let lt=nr(p.clone(),e.reverse()).reverse().then(Equality::boolean(less(p.clone(),add(p.clone(),gap.clone())),bit(true),own_math("PositiveAdd",vec![p.clone(),gap,positive])))?;
        self.theorem("Power8LessPower63",&[],lt.clone())?;
        let cap=Equality::natural(power(8),capacity(),call("Mpk.Go.Practical.Word.Index.D8.CapacityPower",vec![])).reverse();
        let bound=cap.clone().mapped(lam("_n",nat(),less(var("_n"),half(6))),less(capacity(),half(6)),less(p.clone(),half(6)),boolean())
            .then(nr(p,Equality::natural(half(6),power(63),own_math("HalfPower6",vec![]))))?.then(lt)?;
        self.theorem("CapacityBelowHalf",&[],bound)?;
        let w=var("word");let u=value(w.clone());let top=bit63(w.clone());
        let below=own_math("AtMostBelow",vec![u.clone(),capacity(),half(6),var("upper"),own_math("CapacityBelowHalf",vec![])]);
        let not_top=Equality::boolean(less(u.clone(),half(6)),bc("Not",vec![top.clone()]),sb("W64.UnsignedHalfLess",vec![w.clone()])).reverse().then(Equality::boolean(less(u,half(6)),bit(true),below))?;
        self.theorem("BoundedSign",&[("word",wt()),("upper",eq(boolean(),less(capacity(),value(w.clone())),bit(false)))],Equality::boolean(top.clone(),bit(false),idx("NotTrueReflect",vec![top,not_top.proof])))?;
        let raw=int("Make",vec![value(w.clone()),mul(weight(6),num("Bit",vec![bit63(w.clone())]))]);
        let bit_eq=Equality::boolean(bit63(w.clone()),bit(false),var("clear")).mapped(lam("_sign",boolean(),num("Bit",vec![var("_sign")])),num("Bit",vec![bit63(w.clone())]),num("Bit",vec![bit(false)]),nat()).then(Equality::natural(num("Bit",vec![bit(false)]),zero(),refl(nat(),zero())))?;
        let negative=mul_right(weight(6),bit_eq).then(Equality::natural(mul(weight(6),zero()),zero(),nc("MulZero",vec![weight(6)])))?;
        let e=Equality{ty:integer_type(),left:signed(w.clone()),right:raw.clone(),proof:refl(integer_type(),raw)}
            .then(negative.mapped(lam("_negative",nat(),int("Make",vec![value(w.clone()),var("_negative")])),int("Make",vec![value(w.clone()),mul(weight(6),num("Bit",vec![bit63(w.clone())]))]),int("Make",vec![value(w.clone()),zero()]),integer_type()))?
            .then(Equality{ty:integer_type(),left:int("Make",vec![value(w.clone()),zero()]),right:int("FromNat",vec![value(w.clone())]),proof:refl(integer_type(),int("FromNat",vec![value(w.clone())]))})?;
        self.theorem("SignedAsNat",&[("word",wt()),("clear",eq(boolean(),bit63(w),bit(false)))],e)
    }
}
pub(super) fn append(builder:&mut Builder)->Result<()>{Math{b:builder,serial:0}.append()}
