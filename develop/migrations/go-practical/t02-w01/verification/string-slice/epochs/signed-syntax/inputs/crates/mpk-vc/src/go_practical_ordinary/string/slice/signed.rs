//! Guard correspondence with the existing balanced signed64 interpretation.
use super::guards::{and_true, component, conjunction, defined, ec, not_false};
use super::*;

fn int() -> Expr { pi("_part", boolean(), nat()) }
fn ic(name: &str, args: Vec<Expr>) -> Expr { call(&format!("Mpk.Go.Practical.Int.{name}"), args) }
fn signed(x: Expr) -> Expr { numeric("W64.Signed", vec![x]) }
fn from(n: Expr) -> Expr { ic("FromNat", vec![n]) }
fn il(a: Expr, b: Expr) -> Expr { ic("Less", vec![a,b]) }
fn il_left(e: Equality, b: Expr) -> Equality {
    let (l,r)=(il(e.left.clone(),b.clone()),il(e.right.clone(),b.clone()));
    e.mapped(lam("_int",int(),il(var("_int"),b)),l,r,boolean())
}
fn il_right(a: Expr, e: Equality) -> Equality {
    let (l,r)=(il(a.clone(),e.left.clone()),il(a.clone(),e.right.clone()));
    e.mapped(lam("_int",int(),il(a,var("_int"))),l,r,boolean())
}
fn parts(x: Expr, p: Expr, h: Expr) -> Equality {
    Equality {ty:int(), left:signed(x.clone()),right:ic("Make",vec![value(x.clone()),mul(modulus(),numeric("Bit",vec![p.clone()]))]),proof:sl("SignedParts",vec![x,p,h])}
}
fn clear(x: Expr, h: Expr) -> Equality {
    Equality {ty:int(),left:signed(x.clone()),right:from(value(x.clone())),proof:sl("SignedAsNat",vec![x,h])}
}
fn signed_fields(s: Expr,a: Expr,b: Expr) -> Vec<Expr> {
    vec![bc("Not",vec![il(signed(a.clone()),from(zero()))]),bc("Not",vec![il(signed(b.clone()),signed(a))]),bc("Not",vec![il(from(size(s)),signed(b))])]
}
fn bounds(s: Expr,a: Expr,b: Expr) -> Expr { sl("SignedBounds",vec![s,a,b]) }

impl Slice<'_> {
    pub(super) fn signed_bounds(&mut self) -> Result<()> {
        self.signed_values()?;
        self.signed_negative()?;
        self.signed_guard()
    }
    fn signed_values(&mut self) -> Result<()> {
        let (x,p)=(var("x"),var("p"));
        let raw=ic("Make",vec![value(x.clone()),mul(modulus(),numeric("Bit",vec![sign(x.clone())?]))]);
        let e=Equality {ty:int(),left:signed(x.clone()),right:raw.clone(),proof:refl(int(),raw)}
            .then(mul_right(modulus(),bit_value(Equality::boolean(sign(x.clone())?,p.clone(),var("h"))))
                .mapped(lam("_negative",nat(),ic("Make",vec![value(x.clone()),var("_negative")])),
                    ic("Make",vec![value(x.clone()),mul(modulus(),numeric("Bit",vec![sign(x.clone())?]))]),
                    ic("Make",vec![value(x.clone()),mul(modulus(),numeric("Bit",vec![p.clone()]))]),int()))?;
        self.theorem("SignedParts",&[("x",word64()),("p",boolean()),("h",eq(boolean(),sign(x.clone())?,p))],e)?;
        let weighted=mul_right(modulus(),Equality::natural(numeric("Bit",vec![bit(false)]),zero(),refl(nat(),zero())))
            .then(Equality::natural(mul(modulus(),zero()),zero(),nc("MulZero",vec![modulus()])))?;
        let e=parts(x.clone(),bit(false),var("h")).then(weighted.mapped(lam("_n",nat(),ic("Make",vec![value(x.clone()),var("_n")])),ic("Make",vec![value(x.clone()),mul(modulus(),numeric("Bit",vec![bit(false)]))]),ic("Make",vec![value(x.clone()),zero()]),int()))?
            .then(Equality {ty:int(),left:ic("Make",vec![value(x.clone()),zero()]),right:from(value(x.clone())),proof:refl(int(),from(value(x.clone())))})?;
        self.theorem("SignedAsNat",&[("x",word64()),("h",truth(sign(x.clone())?,false))],e)?;
        let (a,b,c,d)=(var("a"),var("b"),var("c"),var("d"));
        let raw=less(add(a.clone(),d.clone()),add(c.clone(),b.clone()));
        self.theorem("LessMake",&[("a",nat()),("b",nat()),("c",nat()),("d",nat())],Equality::boolean(il(ic("Make",vec![a.clone(),b]),ic("Make",vec![c.clone(),d])),raw.clone(),refl(boolean(),raw)))?;
        let e=Equality::boolean(il(from(a.clone()),from(c.clone())),less(add(a.clone(),zero()),add(c.clone(),zero())),sl("LessMake",vec![a.clone(),zero(),c.clone(),zero()]))
            .then(nl(Equality::natural(add(a.clone(),zero()),a.clone(),nc("AddZero",vec![a.clone()])),add(c.clone(),zero())))?
            .then(nr(a.clone(),Equality::natural(add(c.clone(),zero()),c.clone(),nc("AddZero",vec![c.clone()]))))?;
        self.theorem("LessFromNat",&[("a",nat()),("c",nat())],e)?;
        let y=var("y");
        let e=il_left(clear(x.clone(),var("xs")),signed(y.clone()))
            .then(il_right(from(value(x.clone())),clear(y.clone(),var("ys"))))?
            .then(Equality::boolean(il(from(value(x.clone())),from(value(y.clone()))),less(value(x.clone()),value(y.clone())),sl("LessFromNat",vec![value(x.clone()),value(y.clone())])))?;
        self.theorem("SignedLessClear",&[("x",word64()),("y",word64()),("xs",truth(sign(x.clone())?,false)),("ys",truth(sign(y.clone())?,false))],e)?;
        let e=il_right(from(a.clone()),clear(x.clone(),var("xs")))
            .then(Equality::boolean(il(from(a.clone()),from(value(x.clone()))),less(a.clone(),value(x.clone())),sl("LessFromNat",vec![a.clone(),value(x.clone())])))?;
        self.theorem("FromNatSignedLess",&[("a",nat()),("x",word64()),("xs",truth(sign(x.clone())?,false))],e)
    }
    fn signed_negative(&mut self) -> Result<()> {
        let x=var("x");
        let u=value(x.clone());
        let flag=sign(x.clone())?;
        let negative=il(signed(x.clone()),from(zero()));
        let target=eq(boolean(),negative.clone(),flag.clone());
        let mut branches=vec![];
        for yes in [false,true] {
            let cost=mul(modulus(),numeric("Bit",vec![bit(yes)]));
            let weighted=mul_right(modulus(),Equality::natural(numeric("Bit",vec![bit(yes)]),if yes{one()}else{zero()},refl(nat(),if yes{one()}else{zero()})))
                .then(Equality::natural(mul(modulus(),if yes{one()}else{zero()}),if yes{modulus()}else{zero()},nc(if yes{"MulOne"}else{"MulZero"},vec![modulus()])))?;
            let e=il_left(parts(x.clone(),bit(yes),var("_branch")),from(zero()))
                .then(Equality::boolean(il(ic("Make",vec![u.clone(),cost.clone()]),from(zero())),less(add(u.clone(),zero()),add(zero(),cost.clone())),sl("LessMake",vec![u.clone(),cost.clone(),zero(),zero()])))?
                .then(nl(Equality::natural(add(u.clone(),zero()),u.clone(),nc("AddZero",vec![u.clone()])),add(zero(),cost.clone())))?
                .then(nr(u.clone(),Equality::natural(add(zero(),cost.clone()),cost,nc("ZeroAdd",vec![weighted.left.clone()]))))?
                .then(nr(u.clone(),weighted))?
                .then(Equality::boolean(less(u.clone(),if yes{modulus()}else{zero()}),bit(yes),if yes{ord("W64.ValueBound",vec![x.clone()])}else{nc("LessZero",vec![u.clone()])}))?
                .then(Equality::boolean(flag.clone(),bit(yes),var("_branch")).reverse())?;
            branches.push(e.proof);
        }
        self.theorem("SignedNegative",&[("x",word64())],Equality::boolean(negative,flag.clone(),decide(flag,target,branches[0].clone(),branches[1].clone())))?;
        let y=var("y");
        let v=value(y.clone());
        let cost=mul(modulus(),numeric("Bit",vec![bit(true)]));
        let weighted=mul_right(modulus(),Equality::natural(numeric("Bit",vec![bit(true)]),one(),refl(nat(),one())))
            .then(Equality::natural(mul(modulus(),one()),modulus(),nc("MulOne",vec![modulus()])))?;
        let negative=parts(x.clone(),bit(true),var("negative"))
            .then(weighted.mapped(lam("_n",nat(),ic("Make",vec![u.clone(),var("_n")])),ic("Make",vec![u.clone(),cost]),ic("Make",vec![u.clone(),modulus()]),int()))?;
        let upper=nl(Equality::natural(add(v.clone(),modulus()),add(modulus(),v.clone()),nc("AddComm",vec![v.clone(),modulus()])),modulus())
            .then(Equality::boolean(less(add(modulus(),v.clone()),modulus()),bit(false),call("Mpk.Go.Practical.Word.Shifts.CountNoLessExtension",vec![modulus(),v.clone()])))?;
        let e=il_left(negative,signed(y.clone()))
            .then(il_right(ic("Make",vec![u.clone(),modulus()]),clear(y.clone(),var("nonnegative"))))?
            .then(Equality::boolean(il(ic("Make",vec![u.clone(),modulus()]),from(v.clone())),less(add(u.clone(),zero()),add(v.clone(),modulus())),sl("LessMake",vec![u.clone(),modulus(),v.clone(),zero()])))?
            .then(nl(Equality::natural(add(u.clone(),zero()),u.clone(),nc("AddZero",vec![u.clone()])),add(v.clone(),modulus())))?
            .then(Equality::boolean(less(u.clone(),add(v.clone(),modulus())),bit(true),call("Mpk.Go.Practical.Word.Index.UpperBound",vec![add(v,modulus()),modulus(),u,upper.proof,ord("W64.ValueBound",vec![x.clone()])]))))?;
        self.theorem("NegativeBelowNonnegative",&[("x",word64()),("y",word64()),("negative",truth(sign(x)?,true)),("nonnegative",truth(sign(y)?,false))],e)
    }
    fn signed_guard(&mut self) -> Result<()> {
        let (s,a,b)=(var("s"),var("a"),var("b"));
        let base=[("s",string_type()),("a",word64()),("b",word64())];
        let fs=signed_fields(s.clone(),a.clone(),b.clone());
        let body=conjunction(&fs);
        self.emit("SignedBounds",&base,boolean(),body.clone(),false)?;
        self.theorem("SignedBoundsStep",&base,Equality::boolean(bounds(s.clone(),a.clone(),b.clone()),body.clone(),refl(boolean(),body.clone())))?;
        let mut forward=base.to_vec();
        forward.push(("h",truth(defined(s.clone(),a.clone(),b.clone()),true)));
        let low=sl("DefinedLowSign",vec![s.clone(),a.clone(),b.clone(),var("h")]);
        let highsign=sl("DefinedHighSign",vec![s.clone(),a.clone(),b.clone(),var("h")]);
        let negative=Equality::boolean(il(signed(a.clone()),from(zero())),sign(a.clone())?,sl("SignedNegative",vec![a.clone()]))
            .then(Equality::boolean(sign(a.clone())?,bit(false),low.clone()))?;
        let order=Equality::boolean(il(signed(b.clone()),signed(a.clone())),less(value(b.clone()),value(a.clone())),sl("SignedLessClear",vec![b.clone(),a.clone(),highsign.clone(),low]))
            .then(Equality::boolean(less(value(b.clone()),value(a.clone())),bit(false),sl("DefinedOrdered",vec![s.clone(),a.clone(),b.clone(),var("h")])))?;
        let high=Equality::boolean(il(from(size(s.clone())),signed(b.clone())),less(size(s.clone()),value(b.clone())),sl("FromNatSignedLess",vec![size(s.clone()),b.clone(),highsign]))
            .then(Equality::boolean(less(size(s.clone()),value(b.clone())),bit(false),sl("DefinedHigh",vec![s.clone(),a.clone(),b.clone(),var("h")])))?;
        let all=and_true(not_false(high)?,and_true(not_false(order)?,not_false(negative)?)?)?;
        self.theorem("SignedBoundsFromDefined",&forward,Equality::boolean(bounds(s.clone(),a.clone(),b.clone()),body.clone(),sl("SignedBoundsStep",vec![s.clone(),a.clone(),b.clone()])).then(all)?)?;
        let mut backward=base.to_vec();
        backward.push(("h",truth(bounds(s.clone(),a.clone(),b.clone()),true)));
        let all=Equality::boolean(bounds(s.clone(),a.clone(),b.clone()),body,sl("SignedBoundsStep",vec![s.clone(),a.clone(),b.clone()])).reverse()
            .then(Equality::boolean(bounds(s.clone(),a.clone(),b.clone()),bit(true),var("h")))?;
        let claims=fs.iter().enumerate().map(|(i,_)| {
            let args=signed_fields(s.clone(),a.clone(),b.clone());
            let Expr::Apply(_,notargs)=&args[i] else{unreachable!()};
            Equality::boolean(notargs[0].clone(),bit(false),sl("NotTrueFalse",vec![notargs[0].clone(),component(&fs,all.proof.clone(),i)]))
        }).collect::<Vec<_>>();
        let low=Equality::boolean(il(signed(a.clone()),from(zero())),sign(a.clone())?,sl("SignedNegative",vec![a.clone()])).reverse().then(claims[0].clone())?;
        let contradiction=claims[1].clone().reverse().then(Equality::boolean(il(signed(b.clone()),signed(a.clone())),bit(true),sl("NegativeBelowNonnegative",vec![b.clone(),a.clone(),var("_branch"),low.proof.clone()])))?;
        let target=truth(sign(b.clone())?,false);
        let high=decide(sign(b.clone())?,target.clone(),var("_branch"),bc("FalseElim",vec![target,contradiction.proof]));
        let order=Equality::boolean(il(signed(b.clone()),signed(a.clone())),less(value(b.clone()),value(a.clone())),sl("SignedLessClear",vec![b.clone(),a.clone(),high.clone(),low.proof.clone()])).reverse().then(claims[1].clone())?;
        let end=Equality::boolean(il(from(size(s.clone())),signed(b.clone())),less(size(s.clone()),value(b.clone())),sl("FromNatSignedLess",vec![size(s.clone()),b.clone(),high.clone()])).reverse().then(claims[2].clone())?;
        self.theorem("DefinedFromSignedBounds",&backward,Equality::boolean(defined(s.clone(),a.clone(),b.clone()),bit(true),sl("DefinedComplete",vec![s.clone(),a.clone(),b.clone(),low.proof,high,order.proof,end.proof])))?;
        self.theorem("SignedBoundsExact",&base,Equality::boolean(defined(s.clone(),a.clone(),b.clone()),bounds(s.clone(),a.clone(),b.clone()),ec("TruthIff",vec![defined(s.clone(),a.clone(),b.clone()),bounds(s.clone(),a.clone(),b.clone()),lam("_h",truth(defined(s.clone(),a.clone(),b.clone()),true),sl("SignedBoundsFromDefined",vec![s.clone(),a.clone(),b.clone(),var("_h")])),lam("_h",truth(bounds(s.clone(),a.clone(),b.clone()),true),sl("DefinedFromSignedBounds",vec![s,a,b,var("_h")]))])))
    }
}
