//! Checked source-width modular identities used by endpoint differences.
use super::*;

pub(super) fn bit_not(d: usize, x: Expr) -> Expr { rn("Not", d, vec![x]) }
pub(super) fn difference(d: usize, x: Expr, y: Expr) -> Expr {
    sum(d, x, bit_not(d, y), literal(true))
}
pub(super) fn canonical_zero(d: usize) -> Typed {
    typed(cube_type(d),canon(d,literal_word(d,0)),literal_word(d,0),rn("CanonicalZero",d,vec![]))
}
fn not_body(d: usize, x: Expr) -> Expr {
    if d == 0 { not(x) } else {
        join(d, bit_not(d - 1, project(x.clone(), false)), bit_not(d - 1, project(x, true)))
    }
}
fn not_def(d: usize, x: Expr) -> Typed {
    typed(cube_type(d), bit_not(d, x.clone()), not_body(d, x.clone()), rn("NotDefinition", d, vec![x]))
}
fn not_part(d: usize, x: Expr, high: bool) -> Typed {
    typed(cube_type(d - 1), project(bit_not(d, x.clone()), high), bit_not(d - 1, project(x.clone(), high)), rn(if high { "NotHigh" } else { "NotLow" }, d, vec![x]))
}
fn binary(d: usize) -> Vec<(String, Expr)> {
    vec![("x".into(),cube_type(d)),("y".into(),cube_type(d)),("incoming".into(),bool_ty())]
}
fn unary(d: usize) -> Vec<(String, Expr)> { vec![("x".into(),cube_type(d))] }
fn local(kind: &str, args: Vec<Expr>) -> Expr { rc(&format!("Local{kind}"), args) }
pub(super) fn canonical_sum(d: usize, x: Expr, y: Expr, b: Expr) -> Typed {
    let s = sum(d,x.clone(),y.clone(),b.clone());
    typed(cube_type(d),canon(d,s.clone()),s,rn("SumCanonical",d,vec![x,y,b]))
}
pub(super) fn zero_sum(d: usize, x: Expr) -> Typed {
    typed(cube_type(d),sum(d,x.clone(),literal_word(d,0),literal(false)),canon(d,x.clone()),rn("ZeroSum",d,vec![x]))
}
fn zero_carry(d: usize, x: Expr) -> Typed {
    typed(bool_ty(),carry(d,x.clone(),literal_word(d,0),literal(false)),literal(false),rn("ZeroCarry",d,vec![x]))
}
pub(super) fn commute(d: usize, x: Expr, y: Expr, b: Expr, is_carry: bool) -> Typed {
    typed(if is_carry {bool_ty()} else {cube_type(d)},
          if is_carry {carry(d,x.clone(),y.clone(),b.clone())} else {sum(d,x.clone(),y.clone(),b.clone())},
          if is_carry {carry(d,y.clone(),x.clone(),b.clone())} else {sum(d,y.clone(),x.clone(),b.clone())},
          rn(if is_carry {"CommuteCarry"} else {"CommuteSum"},d,vec![x,y,b]))
}
pub(super) fn inverse(d: usize, x: Expr, y: Expr, b: Expr, is_carry: bool) -> Typed {
    let delta = sum(d,x.clone(),bit_not(d,y.clone()),b.clone());
    typed(if is_carry {bool_ty()} else {cube_type(d)},
          if is_carry {carry(d,delta,y.clone(),not(b.clone()))} else {sum(d,delta,y.clone(),not(b.clone()))},
          if is_carry {not(carry(d,x.clone(),bit_not(d,y.clone()),b.clone()))} else {canon(d,x.clone())},
          rn(if is_carry {"InverseCarry"} else {"InverseSum"},d,vec![x,y,b]))
}
pub(super) fn borrow(d: usize, x: Expr, y: Expr, b: Expr) -> Typed {
    typed(bool_ty(),carry(d,x.clone(),bit_not(d,y.clone()),b.clone()),not(scan(d,x.clone(),y.clone(),not(b.clone()))),rn("Borrow",d,vec![x,y,b]))
}
fn local_lemmas(p: &mut Producer) {
    bool_local(p,"SliceAccessValidityResliceNotTwice",&["x"],not(not(v("x"))),v("x"));
    let x=v("x"); let y=v("y"); let b=v("incoming");
    let (s,c)=adder(std::slice::from_ref(&x),std::slice::from_ref(&y),b.clone());
    let (ss,cc)=adder(std::slice::from_ref(&y),std::slice::from_ref(&x),b.clone());
    bool_local(p,"SliceAccessValidityResliceLocalCommuteSum",&["x","y","incoming"],s[0].clone(),ss[0].clone());
    bool_local(p,"SliceAccessValidityResliceLocalCommuteCarry",&["x","y","incoming"],c,cc);
    let (delta,dc)=adder(std::slice::from_ref(&x),&[not(y.clone())],b.clone());
    let (back,bc)=adder(&delta,std::slice::from_ref(&y),not(b.clone()));
    bool_local(p,"SliceAccessValidityResliceLocalInverseSum",&["x","y","incoming"],back[0].clone(),x.clone());
    bool_local(p,"SliceAccessValidityResliceLocalInverseCarry",&["x","y","incoming"],bc,not(dc.clone()));
    bool_local(p,"SliceAccessValidityResliceLocalBorrow",&["x","y","incoming"],dc,not(scan_parts(0,x.clone(),y,not(b))));
    let (s,c)=adder(std::slice::from_ref(&x),&[literal(false)],literal(false));
    bool_local(p,"SliceAccessValidityResliceLocalZeroSum",&["x"],s[0].clone(),x);
    bool_local(p,"SliceAccessValidityResliceLocalZeroCarry",&["x"],c,literal(false));
}
fn complement(p: &mut Producer, d: usize) {
    let x=v("x");
    function_definition(p,&format!("SliceAccessValidityResliceNot{d}"),&unary(d),cube_type(d),not_body(d,x.clone()));
    let mut q=not_def(d,x.clone()); q.proof=apply(c(REFL),vec![q.ty.clone(),q.right.clone()]);
    theorem(p,&format!("SliceAccessValidityResliceNotDefinition{d}"),&unary(d),q);
    if d>0 { for high in [false,true] {
        let mut q=not_part(d,x.clone(),high); q.proof=apply(c(REFL),vec![q.ty.clone(),q.right.clone()]);
        theorem(p,&format!("SliceAccessValidityResliceNot{}{d}",if high {"High"} else {"Low"}),&unary(d),q);
    }}
}
fn canonical_proof(p: &mut Producer, d: usize) -> Typed {
    let (x,y,b)=(v("x"),v("y"),v("incoming"));
    let mut q=canon_def(d,sum(d,x.clone(),y.clone(),b.clone()));
    if d==0 { return q; }
    for high in [false,true] { q=change(p,q,sum_part(d,x.clone(),y.clone(),b.clone(),high)); }
    let low=carry(d-1,project(x.clone(),false),project(y.clone(),false),b.clone());
    q=change(p,q,canonical_sum(d-1,project(x.clone(),false),project(y.clone(),false),b.clone()));
    q=change(p,q,canonical_sum(d-1,project(x.clone(),true),project(y.clone(),true),low));
    next(q,backwards(sum_def(d,x,y,b)))
}
fn canonical_zero_proof(p: &mut Producer, d: usize) -> Typed {
    let mut q=canon_def(d,literal_word(d,0));
    if d>0 {
        for high in [false,true] {q=change(p,q,literal_projection(d,0,high));}
        q=change(p,q,canonical_zero(d-1));
    }
    assert_eq!(q.right,literal_word(d,0));
    q
}
fn zero_proof(p: &mut Producer, d: usize, is_carry: bool) -> Typed {
    let x=v("x"); let z=literal_word(d,0); let b=literal(false);
    let mut q=if is_carry {carry_def(d,x.clone(),z,b)} else {sum_def(d,x.clone(),z,b)};
    if d==0 {
        let right=if is_carry {literal(false)} else {x.clone()};
        let tail=typed(bool_ty(),q.right.clone(),right,local(if is_carry {"ZeroCarry"} else {"ZeroSum"},vec![x.clone()]));
        q=next(q,tail);
    } else {
        for high in [false,true] {q=maybe(p,q,literal_projection(d,0,high));}
        q=change(p,q,zero_carry(d-1,project(x.clone(),false)));
        if is_carry {q=next(q,zero_carry(d-1,project(x.clone(),true)));}
        else {
            for high in [false,true] {q=change(p,q,zero_sum(d-1,project(x.clone(),high)));}
        }
    }
    if !is_carry {q=next(q,backwards(canon_def(d,x)));}
    q
}
fn commutative_proof(p: &mut Producer, d: usize, is_carry: bool) -> Typed {
    let (x,y,b)=(v("x"),v("y"),v("incoming"));
    let mut q=if is_carry {carry_def(d,x.clone(),y.clone(),b.clone())} else {sum_def(d,x.clone(),y.clone(),b.clone())};
    if d==0 {
        let right=if is_carry {carry_def(d,y.clone(),x.clone(),b.clone()).right} else {sum_def(d,y.clone(),x.clone(),b.clone()).right};
        let tail=typed(bool_ty(),q.right.clone(),right,local(if is_carry {"CommuteCarry"} else {"CommuteSum"},vec![x.clone(),y.clone(),b.clone()]));
        q=next(q,tail);
    } else {
        let xl=project(x.clone(),false); let yl=project(y.clone(),false);
        q=change(p,q,commute(d-1,xl,yl,b.clone(),true));
        let incoming=carry(d-1,project(y.clone(),false),project(x.clone(),false),b.clone());
        if !is_carry {q=change(p,q,commute(d-1,project(x.clone(),false),project(y.clone(),false),b.clone(),false));}
        q=change(p,q,commute(d-1,project(x.clone(),true),project(y.clone(),true),incoming,is_carry));
    }
    next(q,backwards(if is_carry {carry_def(d,y,x,b)} else {sum_def(d,y,x,b)}))
}
fn inverse_proof(p: &mut Producer, d: usize, is_carry: bool) -> Typed {
    let (x,y,b)=(v("x"),v("y"),v("incoming"));
    let delta=sum(d,x.clone(),bit_not(d,y.clone()),b.clone());
    let mut q=if is_carry {carry_def(d,delta.clone(),y.clone(),not(b.clone()))} else {sum_def(d,delta.clone(),y.clone(),not(b.clone()))};
    if d==0 {
        q=change(p,q,sum_def(d,x.clone(),bit_not(d,y.clone()),b.clone()));
        q=change(p,q,not_def(d,y.clone()));
        let (_,dc)=adder(std::slice::from_ref(&x),&[not(y.clone())],b.clone());
        let right=if is_carry {not(dc)} else {x.clone()};
        let tail=typed(bool_ty(),q.right.clone(),right,local(if is_carry {"InverseCarry"} else {"InverseSum"},vec![x.clone(),y.clone(),b.clone()]));
        q=next(q,tail);
    } else {
        for high in [false,true] {q=maybe(p,q,sum_part(d,x.clone(),bit_not(d,y.clone()),b.clone(),high));}
        for high in [false,true] {q=maybe(p,q,not_part(d,y.clone(),high));}
        q=change(p,q,inverse(d-1,project(x.clone(),false),project(y.clone(),false),b.clone(),true));
        let mid=carry(d-1,project(x.clone(),false),bit_not(d-1,project(y.clone(),false)),b.clone());
        if !is_carry {q=change(p,q,inverse(d-1,project(x.clone(),false),project(y.clone(),false),b.clone(),false));}
        q=change(p,q,inverse(d-1,project(x.clone(),true),project(y.clone(),true),mid,is_carry));
    }
    if is_carry {
        let right=not(carry(d,x.clone(),bit_not(d,y.clone()),b.clone()));
        let mut view=reflexive_t(bool_ty(),right);
        view=change(p,view,carry_def(d,x,bit_not(d,y.clone()),b));
        if d==0 {view=change(p,view,not_def(d,y));}
        else {for high in [false,true] {view=maybe(p,view,not_part(d,y.clone(),high));}}
        next(q,backwards(view))
    } else {next(q,backwards(canon_def(d,x)))}
}
fn borrow_proof(p: &mut Producer, d: usize) -> Typed {
    let (x,y,b)=(v("x"),v("y"),v("incoming"));
    let mut q=carry_def(d,x.clone(),bit_not(d,y.clone()),b.clone());
    if d==0 {
        q=change(p,q,not_def(d,y.clone()));
        let right=not(scan_parts(0,x.clone(),y.clone(),not(b.clone())));
        let tail=typed(bool_ty(),q.right.clone(),right,local("Borrow",vec![x.clone(),y.clone(),b.clone()]));
        q=next(q,tail);
    } else {
        for high in [false,true] {q=maybe(p,q,not_part(d,y.clone(),high));}
        q=change(p,q,borrow(d-1,project(x.clone(),false),project(y.clone(),false),b.clone()));
        let lower=scan(d-1,project(x.clone(),false),project(y.clone(),false),not(b.clone()));
        q=next(q,borrow(d-1,project(x.clone(),true),project(y.clone(),true),not(lower.clone())));
        q=change(p,q,typed(bool_ty(),not(not(lower.clone())),lower.clone(),rc("NotTwice",vec![lower])));
    }
    let mut view=reflexive_t(bool_ty(),not(scan(d,x.clone(),y.clone(),not(b.clone()))));
    view=change(p,view,scan_composition(d,x,y,not(b)));
    next(q,backwards(view))
}
pub(super) fn install(p: &mut Producer, trace: Option<&Path>) {
    local_lemmas(p);
    for d in 0..=WORD {
        complement(p,d);
        let q=canonical_proof(p,d);
        theorem(p,&format!("SliceAccessValidityResliceSumCanonical{d}"),&binary(d),q);
        let q=canonical_zero_proof(p,d);
        theorem(p,&format!("SliceAccessValidityResliceCanonicalZero{d}"),&[],q);
        for is_carry in [false,true] {
            let q=zero_proof(p,d,is_carry);
            theorem(p,&format!("SliceAccessValidityResliceZero{}{d}",if is_carry {"Carry"} else {"Sum"}),&unary(d),q);
            let q=commutative_proof(p,d,is_carry);
            theorem(p,&format!("SliceAccessValidityResliceCommute{}{d}",if is_carry {"Carry"} else {"Sum"}),&binary(d),q);
            let q=inverse_proof(p,d,is_carry);
            theorem(p,&format!("SliceAccessValidityResliceInverse{}{d}",if is_carry {"Carry"} else {"Sum"}),&binary(d),q);
        }
        let q=borrow_proof(p,d);
        theorem(p,&format!("SliceAccessValidityResliceBorrow{d}"),&binary(d),q);
        save_trace(p,trace,&format!("reslice-arith-{d}"));
    }
}
