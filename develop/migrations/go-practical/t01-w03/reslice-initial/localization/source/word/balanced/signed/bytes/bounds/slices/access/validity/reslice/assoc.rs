//! Three-operand associativity preserves actual carry counts in six states.
use super::*;

fn flags() -> Vec<Expr> { ["xy_in","left_in","yz_in","right_in"].into_iter().map(v).collect() }
fn relation(s: &[Expr]) -> Expr {
    choose(bool_eq(s[0].clone(),s[1].clone()),
           and(bool_eq(s[2].clone(),s[0].clone()),bool_eq(s[3].clone(),s[0].clone())),
           not(bool_eq(s[2].clone(),s[3].clone())))
}
fn operands(d: usize) -> Vec<(String,Expr)> {
    let mut a=["x","y","z"].into_iter().map(|n|(n.into(),cube_type(d))).collect::<Vec<_>>();
    a.extend(["xy_in","left_in","yz_in","right_in"].into_iter().map(|n|(n.into(),bool_ty())));
    a
}
fn args(x: Expr,y: Expr,z: Expr,s: &[Expr]) -> Vec<Expr> {
    let mut a=vec![x,y,z];a.extend_from_slice(s);a
}
fn left(d: usize,x: Expr,y: Expr,z: Expr,s: &[Expr]) -> Expr { sum(d,sum(d,x,y,s[0].clone()),z,s[1].clone()) }
fn right(d: usize,x: Expr,y: Expr,z: Expr,s: &[Expr]) -> Expr { sum(d,x,sum(d,y,z,s[2].clone()),s[3].clone()) }
fn outputs(d: usize,x: Expr,y: Expr,z: Expr,s: &[Expr]) -> Vec<Expr> {
    vec![carry(d,x.clone(),y.clone(),s[0].clone()),
         carry(d,sum(d,x.clone(),y.clone(),s[0].clone()),z.clone(),s[1].clone()),
         carry(d,y.clone(),z.clone(),s[2].clone()),
         carry(d,x,sum(d,y,z,s[2].clone()),s[3].clone())]
}
fn local_add(x: Expr,y: Expr,b: Expr) -> (Expr,Expr) {
    let (s,c)=adder(&[x],&[y],b);(s[0].clone(),c)
}
fn local_outputs(x: Expr,y: Expr,z: Expr,s: &[Expr]) -> (Expr,Expr,Vec<Expr>) {
    let (xy,a)=local_add(x.clone(),y.clone(),s[0].clone());
    let (l,b)=local_add(xy,z.clone(),s[1].clone());
    let (yz,c)=local_add(y,z,s[2].clone());
    let (r,d)=local_add(x,yz,s[3].clone());
    (l,r,vec![a,b,c,d])
}
pub(super) fn reassociate(d: usize,x: Expr,y: Expr,z: Expr,s: &[Expr],h: Expr) -> Typed {
    let mut a=args(x.clone(),y.clone(),z.clone(),s);a.push(h);
    typed(cube_type(d),left(d,x.clone(),y.clone(),z.clone(),s),right(d,x,y,z,s),rn("AssocSum",d,a))
}
fn conserved(d: usize,x: Expr,y: Expr,z: Expr,s: &[Expr],h: Expr) -> Typed {
    let mut a=args(x.clone(),y.clone(),z.clone(),s);a.push(h);
    typed(bool_ty(),relation(&outputs(d,x,y,z,s)),literal(true),rn("AssocCarries",d,a))
}
pub(super) fn no_wrap(d: usize,x: Expr,y: Expr,z: Expr,hinner: Expr,houter: Expr) -> Typed {
    let s=vec![literal(false);4];let out=outputs(d,x.clone(),y.clone(),z.clone(),&s);
    typed(bool_ty(),and(not(out[0].clone()),not(out[1].clone())),literal(true),rn("AssocNoWrap",d,vec![x,y,z,hinner,houter]))
}
fn primitives(p: &mut Producer) {
    let x=v("x");let y=v("y");let z=v("z");let s=flags();
    let (l,r,out)=local_outputs(x,y,z,&s);
    bool_local(p,"SliceAccessValidityResliceLocalAssocSum",&["x","y","z","xy_in","left_in","yz_in","right_in"],choose(relation(&s),l,r.clone()),r);
    bool_local(p,"SliceAccessValidityResliceLocalAssocCarries",&["x","y","z","xy_in","left_in","yz_in","right_in"],choose(relation(&s),relation(&out),literal(true)),literal(true));
    let condition=and(relation(&s),and(not(s[2].clone()),not(s[3].clone())));
    bool_local(p,"SliceAccessValidityResliceLocalAssocFinish",&["xy_in","left_in","yz_in","right_in"],choose(condition,and(not(s[0].clone()),not(s[1].clone())),literal(true)),literal(true));
    let q=normalize(relation(&vec![literal(false);4]),p);
    theorem(p,"SliceAccessValidityResliceAssocStart",&[],q);
}
fn sum_view(p: &mut Producer,d: usize,x: Expr,y: Expr,z: Expr,s: &[Expr],is_right: bool) -> Typed {
    let whole=if is_right {right(d,x.clone(),y.clone(),z.clone(),s)} else {left(d,x.clone(),y.clone(),z.clone(),s)};
    let mut q=reflexive_t(cube_type(d),whole);
    if is_right {
        q=change(p,q,sum_def(d,x.clone(),sum(d,y.clone(),z.clone(),s[2].clone()),s[3].clone()));
        if d==0 {q=change(p,q,sum_def(d,y,z,s[2].clone()));}
        else {for high in [false,true] {q=maybe(p,q,sum_part(d,y.clone(),z.clone(),s[2].clone(),high));}}
    } else {
        q=change(p,q,sum_def(d,sum(d,x.clone(),y.clone(),s[0].clone()),z,s[1].clone()));
        if d==0 {q=change(p,q,sum_def(d,x,y,s[0].clone()));}
        else {for high in [false,true] {q=maybe(p,q,sum_part(d,x.clone(),y.clone(),s[0].clone(),high));}}
    }
    q
}
fn sum_proof(p: &mut Producer,d: usize) -> Typed {
    let (x,y,z)=(v("x"),v("y"),v("z"));let s=flags();
    let mut q=sum_view(p,d,x.clone(),y.clone(),z.clone(),&s,false);
    if d==0 {
        let (l,r,_)=local_outputs(x.clone(),y.clone(),z.clone(),&s);
        let proof=rc("LocalAssocSum",args(x.clone(),y.clone(),z.clone(),&s));
        let a=under(p,relation(&s),l,r,v("balanced"),proof);
        q=next(q,a);
    } else {
        let (xl,yl,zl)=(project(x.clone(),false),project(y.clone(),false),project(z.clone(),false));
        let mid=outputs(d-1,xl.clone(),yl.clone(),zl.clone(),&s);
        let lo=reassociate(d-1,xl.clone(),yl.clone(),zl.clone(),&s,v("balanced"));
        let hm=conserved(d-1,xl,yl,zl,&s,v("balanced"));
        q=change(p,q,lo);
        q=change(p,q,reassociate(d-1,project(x.clone(),true),project(y.clone(),true),project(z.clone(),true),&mid,hm.proof));
    }
    next(q,backwards(sum_view(p,d,x,y,z,&s,true)))
}
fn carry_view(p: &mut Producer,d: usize,x: Expr,y: Expr,z: Expr,s: &[Expr]) -> Typed {
    let mut q=reflexive_t(bool_ty(),relation(&outputs(d,x.clone(),y.clone(),z.clone(),s)));
    for c in [carry_def(d,x.clone(),y.clone(),s[0].clone()),
              carry_def(d,sum(d,x.clone(),y.clone(),s[0].clone()),z.clone(),s[1].clone()),
              carry_def(d,y.clone(),z.clone(),s[2].clone()),
              carry_def(d,x.clone(),sum(d,y.clone(),z.clone(),s[2].clone()),s[3].clone())] {q=maybe(p,q,c);}
    if d==0 {
        q=maybe(p,q,sum_def(d,x,y.clone(),s[0].clone()));
        q=maybe(p,q,sum_def(d,y,z,s[2].clone()));
    } else {
        for high in [false,true] {
            q=maybe(p,q,sum_part(d,x.clone(),y.clone(),s[0].clone(),high));
            q=maybe(p,q,sum_part(d,y.clone(),z.clone(),s[2].clone(),high));
        }
    }
    q
}
fn carry_proof(p: &mut Producer,d: usize) -> Typed {
    let (x,y,z)=(v("x"),v("y"),v("z"));let s=flags();
    let q=carry_view(p,d,x.clone(),y.clone(),z.clone(),&s);
    let tail=if d==0 {
        let (_,_,out)=local_outputs(x.clone(),y.clone(),z.clone(),&s);
        under(p,relation(&s),relation(&out),literal(true),v("balanced"),rc("LocalAssocCarries",args(x,y,z,&s)))
    } else {
        let (xl,yl,zl)=(project(x.clone(),false),project(y.clone(),false),project(z.clone(),false));
        let mid=outputs(d-1,xl.clone(),yl.clone(),zl.clone(),&s);
        let h=conserved(d-1,xl,yl,zl,&s,v("balanced"));
        conserved(d-1,project(x,true),project(y,true),project(z,true),&mid,h.proof)
    };
    next(q,tail)
}
fn no_wrap_proof(p: &mut Producer,d: usize) -> Typed {
    let (x,y,z)=(v("x"),v("y"),v("z"));let s=vec![literal(false);4];
    let out=outputs(d,x.clone(),y.clone(),z.clone(),&s);
    let condition=and(relation(&out),and(not(out[2].clone()),not(out[3].clone())));
    let mut q=reflexive_t(bool_ty(),condition.clone());
    q=change(p,q,conserved(d,x,y,z,&s,rc("AssocStart",vec![])));
    q=change(p,q,typed(bool_ty(),out[2].clone(),literal(false),v("inner_no_wrap")));
    q=change(p,q,typed(bool_ty(),out[3].clone(),literal(false),v("outer_no_wrap")));
    let n=normalize(q.right.clone(),p);q=next(q,n);
    under(p,condition,and(not(out[0].clone()),not(out[1].clone())),literal(true),q.proof,rc("LocalAssocFinish",out))
}
pub(super) fn install(p: &mut Producer,trace: Option<&Path>) {
    primitives(p);
    for d in 0..=WORD {
        let mut a=operands(d);a.push(("balanced".into(),eq(relation(&flags()),literal(true))));
        let q=carry_proof(p,d);
        theorem(p,&format!("SliceAccessValidityResliceAssocCarries{d}"),&a,q);
        let q=sum_proof(p,d);
        theorem(p,&format!("SliceAccessValidityResliceAssocSum{d}"),&a,q);
        let (x,y,z)=(v("x"),v("y"),v("z"));
        let mut a=operands(d)[..3].to_vec();
        a.push(("inner_no_wrap".into(),eq(carry(d,y.clone(),z.clone(),literal(false)),literal(false))));
        a.push(("outer_no_wrap".into(),eq(carry(d,x,sum(d,y,z,literal(false)),literal(false)),literal(false))));
        let q=no_wrap_proof(p,d);
        theorem(p,&format!("SliceAccessValidityResliceAssocNoWrap{d}"),&a,q);
        save_trace(p,trace,&format!("reslice-assoc-{d}"));
    }
}
