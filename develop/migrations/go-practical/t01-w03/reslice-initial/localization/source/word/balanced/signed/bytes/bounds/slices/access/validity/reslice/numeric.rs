//! Non-strict unsigned bounds over complete source words and computed sums.
use super::*;

#[derive(Clone)]
pub(super) struct Word {
    pub value: Expr,
    pub normal: Expr,
}
pub(super) fn source(x: Expr) -> Word {
    Word {value:canon(WORD,x.clone()),normal:canon_idem(WORD,x).proof}
}
pub(super) fn zero() -> Word {
    Word {value:literal_word(WORD,0),normal:arith::canonical_zero(WORD).proof}
}
pub(super) fn addition(x: Expr,y: Expr) -> Word {
    Word {value:sum(WORD,x.clone(),y.clone(),literal(false)),normal:arith::canonical_sum(WORD,x,y,literal(false)).proof}
}
pub(super) fn difference(x: Expr,y: Expr) -> Word {
    Word {value:arith::difference(WORD,x.clone(),y.clone()),normal:arith::canonical_sum(WORD,x,arith::bit_not(WORD,y),literal(true)).proof}
}
pub(super) fn le(x: Expr,y: Expr) -> Expr {not(word_less(WORD,y,x))}
fn call(s: &str,args: Vec<Expr>) -> Expr {rc(&format!("Number{s}"),args)}
fn word_args(names: &[&str]) -> Vec<(String,Expr)> {names.iter().map(|n|((*n).into(),word_ty())).collect()}
fn normal(w: &Word) -> Typed {typed(word_ty(),canon(WORD,w.value.clone()),w.value.clone(),w.normal.clone())}
pub(super) fn change_left(p: &mut Producer,e: Typed,q: Typed) -> Typed {
    let bridge=reflexive_t(e.ty.clone(),e.left.clone());
    let bridge=change(p,bridge,q);
    next(backwards(bridge),e)
}
fn negate_false(p: &mut Producer,e: Typed) -> Typed {
    assert_eq!(e.right,literal(false));
    let mut q=reflexive_t(bool_ty(),not(e.left.clone()));q=change(p,q,e);
    let n=normalize(q.right.clone(),p);next(q,n)
}
pub(super) fn reflexive_le(p: &mut Producer,x: Expr) -> Typed {
    negate_false(p,equal_not_less(WORD,x.clone(),x.clone(),refl_equal(WORD,x).proof))
}
pub(super) fn canonical_equal(x: &Word,y: &Word,h: Expr) -> Typed {
    let q=canon_congr(WORD,x.value.clone(),y.value.clone(),equal_bits(WORD,x.value.clone(),y.value.clone(),h));
    next(next(backwards(normal(x)),q),normal(y))
}
fn negative_cases(major: Expr,key: &str,yes: Expr) -> Typed {
    typed(bool_ty(),major.clone(),literal(false),cases(major.clone(),key,eq(major,literal(false)),v(key),yes))
}
fn strict(p: &mut Producer,x: Expr,y: Expr,hle: Expr,hne: Expr) -> Typed {
    let reverse=word_less(WORD,y.clone(),x.clone());
    let no=not_true(p,reverse,hle);
    let e=trichotomy(WORD,x.clone(),y.clone());
    let mut b=reflexive_t(bool_ty(),e.left.clone());
    b=change(p,b,no);
    b=change(p,b,typed(bool_ty(),word_eq(WORD,x,y),literal(false),hne));
    let n=normalize(b.right.clone(),p);b=next(b,n);
    next(backwards(b),e)
}
pub(super) fn transitive_le(x: Expr,y: Expr,z: Expr,hxy: Expr,hyz: Expr) -> Typed {
    typed(bool_ty(),le(x.clone(),z.clone()),literal(true),call("Transitive",vec![x,y,z,hxy,hyz]))
}
pub(super) fn nonnegative(x: Expr,y: Expr,hxy: Expr,hy: Expr) -> Typed {
    typed(bool_ty(),sign_bit(WORD,x.clone()),literal(false),call("Nonnegative",vec![x,y,hxy,hy]))
}
pub(super) fn less_le(x: Expr,y: Expr,hxy: Expr) -> Typed {
    typed(bool_ty(),le(x.clone(),y.clone()),literal(true),call("Strict",vec![x,y,hxy]))
}
pub(super) fn to_zero(x: Expr,h: Expr) -> Typed {
    typed(bool_ty(),word_eq(WORD,x.clone(),literal_word(WORD,0)),literal(true),call("Zero",vec![x,h]))
}
fn less_le_proof(p: &mut Producer) -> Typed {
    let (x,y)=(v("x"),v("y"));let reverse=word_less(WORD,y.clone(),x.clone());
    let bad=transitive(WORD,x.clone(),y,x.clone(),v("less"),v("__reverse"));
    let no=equal_not_less(WORD,x.clone(),x.clone(),refl_equal(WORD,x).proof);
    let bad=impossible(bool_ty(),reverse.clone(),literal(false),contradiction(bad,no.proof));
    negate_false(p,negative_cases(reverse,"__reverse",bad.proof))
}
fn transitive_proof(p: &mut Producer) -> Typed {
    let (x,y,z)=(v("x"),v("y"),v("z"));let reverse=word_less(WORD,z.clone(),x.clone());
    let xy=not_true(p,word_less(WORD,y.clone(),x.clone()),v("first"));
    let yz=not_true(p,word_less(WORD,z.clone(),y.clone()),v("second"));
    let bad=mixed_transitive(WORD,z,x,y,v("__reverse"),xy.proof);
    let bad=impossible(bool_ty(),reverse.clone(),literal(false),contradiction(bad,yz.proof));
    negate_false(p,negative_cases(reverse,"__reverse",bad.proof))
}
fn nonnegative_proof(p: &mut Producer) -> Typed {
    let (x,y)=(v("x"),v("y"));let sx=sign_bit(WORD,x.clone());
    let no=not_true(p,word_less(WORD,y.clone(),x.clone()),v("ordered"));
    let bad=sign_less(WORD,y,x,v("right_nonnegative"),v("__negative"));
    let bad=impossible(bool_ty(),sx.clone(),literal(false),contradiction(bad,no.proof));
    negative_cases(sx,"__negative",bad.proof)
}
fn zero_proof(p: &mut Producer) -> Typed {
    let x=v("x");let z=literal_word(WORD,0);
    let e=trichotomy(WORD,x.clone(),z.clone());let mut q=reflexive_t(bool_ty(),e.left.clone());
    q=change(p,q,less_zero(WORD,x.clone()));
    let h=not_true(p,word_less(WORD,z,x),v("zero_bound"));
    q=change(p,q,h);
    let n=normalize(q.right.clone(),p);q=next(q,n);
    next(backwards(q),e)
}
pub(super) fn bounded_add(o: Word,i: Word,k: Word,hle: Expr,hcarry: Expr,carry_result: bool) -> Typed {
    let si=sum(WORD,o.value.clone(),i.value.clone(),literal(false));
    let sk=sum(WORD,o.value.clone(),k.value.clone(),literal(false));
    typed(bool_ty(),if carry_result {carry(WORD,o.value.clone(),i.value.clone(),literal(false))} else {le(si,sk)},
          literal(!carry_result),call(if carry_result {"BoundCarry"} else {"BoundOrder"},vec![o.value,i.value,k.value,i.normal,k.normal,hle,hcarry]))
}
pub(super) fn sum_lower(x: Word,y: Word,hcarry: Expr) -> Typed {
    typed(bool_ty(),le(x.value.clone(),sum(WORD,x.value.clone(),y.value.clone(),literal(false))),literal(true),call("SumLower",vec![x.value,y.value,x.normal,y.normal,hcarry]))
}
fn sum_lower_proof(p: &mut Producer) -> Typed {
    let x=Word{value:v("x"),normal:v("x_canonical")};let y=Word{value:v("y"),normal:v("y_canonical")};
    let hz=negate_false(p,less_zero(WORD,y.value.clone()));
    let q=bounded_add(x.clone(),zero(),y,hz.proof,v("no_wrap"),false);
    let q=change_left(p,q,arith::zero_sum(WORD,x.value.clone()));
    change_left(p,q,normal(&x))
}
fn sub_carry(a: Expr,b: Expr,h: Expr) -> Typed {
    typed(bool_ty(),carry(WORD,a.clone(),arith::bit_not(WORD,b.clone()),literal(true)),literal(true),call("SubCarry",vec![a,b,h]))
}
fn sub_carry_proof(p: &mut Producer) -> Typed {
    let (a,b)=(v("upper"),v("lower"));let mut q=arith::borrow(WORD,a.clone(),b.clone(),literal(true));
    let n=normalize(not(literal(true)),p);q=change(p,q,n);
    q=change(p,q,scan_false(WORD,a.clone(),b.clone()));
    next(q,typed(bool_ty(),le(b,a),literal(true),v("ordered")))
}
pub(super) fn restore(a: Word,b: Word,h: Expr,commuted: bool,is_carry: bool) -> Typed {
    let delta=arith::difference(WORD,a.value.clone(),b.value.clone());
    let (x,y)=if commuted {(b.value.clone(),delta)} else {(delta,b.value.clone())};
    typed(if is_carry {bool_ty()} else {word_ty()},
          if is_carry {carry(WORD,x,y,literal(false))} else {sum(WORD,x,y,literal(false))},
          if is_carry {literal(false)} else {a.value.clone()},
          call(match (commuted,is_carry) {(false,false)=>"RestoreSum",(false,true)=>"RestoreCarry",(true,false)=>"RestoreLeftSum",(true,true)=>"RestoreLeftCarry"},vec![a.value,b.value,a.normal,h]))
}
fn restore_proof(p: &mut Producer,commuted: bool,is_carry: bool) -> Typed {
    let a=Word{value:v("upper"),normal:v("upper_canonical")};let b=Word{value:v("lower"),normal:v("lower_canonical")};
    if commuted {
        let q=arith::commute(WORD,b.value.clone(),arith::difference(WORD,a.value.clone(),b.value.clone()),literal(false),is_carry);
        return next(q,restore(a,b,v("ordered"),false,is_carry));
    }
    let mut q=arith::inverse(WORD,a.value.clone(),b.value.clone(),literal(true),is_carry);
    q.left=if is_carry {carry(WORD,arith::difference(WORD,a.value.clone(),b.value.clone()),b.value.clone(),literal(false))} else {sum(WORD,arith::difference(WORD,a.value.clone(),b.value.clone()),b.value.clone(),literal(false))};
    if is_carry {
        q=change(p,q,sub_carry(a.value,b.value,v("ordered")));
        let n=normalize(q.right.clone(),p);next(q,n)
    } else {next(q,normal(&a))}
}
pub(super) fn difference_nonnegative(a: Word,b: Word,h: Expr,hna: Expr) -> Typed {
    typed(bool_ty(),sign_bit(WORD,arith::difference(WORD,a.value.clone(),b.value.clone())),literal(false),call("DifferenceNonnegative",vec![a.value,b.value,a.normal,b.normal,h,hna]))
}
fn difference_nonnegative_proof(p: &mut Producer) -> Typed {
    let a=Word{value:v("upper"),normal:v("upper_canonical")};let b=Word{value:v("lower"),normal:v("lower_canonical")};
    let delta=difference(a.value.clone(),b.value.clone());
    let hc=restore(a.clone(),b.clone(),v("ordered"),false,true);
    let q=sum_lower(delta.clone(),b.clone(),hc.proof);
    let q=change_left(p,q,restore(a.clone(),b,v("ordered"),false,false));
    nonnegative(delta.value,a.value,q.proof,v("upper_nonnegative"))
}
pub(super) fn difference_order(a: Word,b: Word,c: Word,hs: [Expr;3]) -> Typed {
    typed(bool_ty(),le(arith::difference(WORD,a.value.clone(),b.value.clone()),arith::difference(WORD,c.value.clone(),b.value.clone())),literal(true),call("DifferenceOrder",vec![a.value,b.value,c.value,a.normal,c.normal,hs[0].clone(),hs[1].clone(),hs[2].clone()]))
}
fn difference_order_proof(p: &mut Producer) -> Typed {
    let a=Word{value:v("upper"),normal:v("upper_canonical")};let b=Word{value:v("lower"),normal:v("lower_canonical")};let c=Word{value:v("maximum"),normal:v("maximum_canonical")};
    let da=arith::difference(WORD,a.value.clone(),b.value.clone());let dc=arith::difference(WORD,c.value.clone(),b.value.clone());
    let reversed=word_less(WORD,dc.clone(),da.clone());
    let hc=restore(a.clone(),b.clone(),v("lower_upper"),true,true);
    let h=an("Monotone",WORD,vec![b.value.clone(),dc.clone(),da.clone(),v("__reversed"),hc.proof]);
    let q=conjunction(p,not(carry(WORD,b.value.clone(),dc,literal(false))),word_less(WORD,sum(WORD,b.value.clone(),arith::difference(WORD,c.value.clone(),b.value.clone()),literal(false)),sum(WORD,b.value.clone(),da,literal(false))),h,true);
    let q=change_left(p,q,restore(a.clone(),b.clone(),v("lower_upper"),true,false));
    let q=change_left(p,q,restore(c.clone(),b,v("lower_maximum"),true,false));
    let no=not_true(p,word_less(WORD,c.value,a.value),v("upper_maximum"));
    let bad=impossible(bool_ty(),reversed.clone(),literal(false),contradiction(q,no.proof));
    negate_false(p,negative_cases(reversed,"__reversed",bad.proof))
}
pub(super) fn difference_zero() -> Typed {
    let z=literal_word(WORD,0);
    typed(word_ty(),arith::difference(WORD,z.clone(),z.clone()),z,call("DifferenceZero",vec![]))
}
fn difference_zero_proof(p: &mut Producer) -> Typed {
    let z=zero();let d=difference(z.value.clone(),z.value.clone());let h=reflexive_le(p,z.value.clone());
    let q=next(arith::zero_sum(WORD,d.value.clone()),normal(&d));
    next(backwards(q),restore(z.clone(),z,h.proof,false,false))
}
fn bound_proof(p: &mut Producer,carry_result: bool) -> Typed {
    let o=v("offset");let i=Word{value:v("index"),normal:v("index_canonical")};let k=Word{value:v("capacity"),normal:v("capacity_canonical")};
    let flag=word_eq(WORD,i.value.clone(),k.value.clone());
    let si=sum(WORD,o.clone(),i.value.clone(),literal(false));let sk=sum(WORD,o.clone(),k.value.clone(),literal(false));
    let target=if carry_result {carry(WORD,o.clone(),i.value.clone(),literal(false))} else {le(si,sk.clone())};
    let goal=eq(target.clone(),literal(!carry_result));let mut branches=vec![];
    for same in [false,true] {
        let q=if same {
            let h=canonical_equal(&i,&k,v("__same"));let mut q=reflexive_t(bool_ty(),target.clone());
            q=change(p,q,h);
            if carry_result {next(q,typed(bool_ty(),carry(WORD,o.clone(),k.value.clone(),literal(false)),literal(false),v("extent_no_wrap")))}
            else {q=change(p,q,equal_not_less(WORD,sk.clone(),sk.clone(),refl_equal(WORD,sk.clone()).proof));let n=normalize(q.right.clone(),p);next(q,n)}
        } else {
            let lt=strict(p,i.value.clone(),k.value.clone(),v("ordered"),v("__same"));
            let ci=carry(WORD,o.clone(),i.value.clone(),literal(false));
            let ls=word_less(WORD,sum(WORD,o.clone(),i.value.clone(),literal(false)),sk.clone());
            let h=an("Monotone",WORD,vec![o.clone(),i.value.clone(),k.value.clone(),lt.proof,v("extent_no_wrap")]);
            let q=conjunction(p,not(ci.clone()),ls.clone(),h,!carry_result);
            if carry_result {not_true(p,ci,q.proof)} else {less_le(sum(WORD,o.clone(),i.value.clone(),literal(false)),sk.clone(),q.proof)}
        };
        branches.push(q.proof);
    }
    typed(bool_ty(),target,literal(!carry_result),cases(flag,"__same",goal,branches[0].clone(),branches[1].clone()))
}
pub(super) fn install(p: &mut Producer,trace: Option<&Path>) {
    let (x,y,z)=(v("x"),v("y"),v("z"));
    let mut a=word_args(&["x","y"]);a.push(("less".into(),eq(word_less(WORD,x.clone(),y.clone()),literal(true))));
    let q=less_le_proof(p);theorem(p,"SliceAccessValidityResliceNumberStrict",&a,q);
    let mut a=word_args(&["x","y","z"]);a.extend([("first".into(),eq(le(x.clone(),y.clone()),literal(true))),("second".into(),eq(le(y.clone(),z),literal(true)))]);
    let q=transitive_proof(p);theorem(p,"SliceAccessValidityResliceNumberTransitive",&a,q);
    let mut a=word_args(&["x","y"]);a.extend([("ordered".into(),eq(le(x.clone(),y.clone()),literal(true))),("right_nonnegative".into(),eq(sign_bit(WORD,y),literal(false)))]);
    let q=nonnegative_proof(p);theorem(p,"SliceAccessValidityResliceNumberNonnegative",&a,q);
    let mut a=word_args(&["x"]);a.push(("zero_bound".into(),eq(le(x,literal_word(WORD,0)),literal(true))));
    let q=zero_proof(p);theorem(p,"SliceAccessValidityResliceNumberZero",&a,q);
    for carry_result in [false,true] {
        let (o,i,k)=(v("offset"),v("index"),v("capacity"));
        let mut a=word_args(&["offset","index","capacity"]);
        a.extend([("index_canonical".into(),eq_ty(word_ty(),canon(WORD,i.clone()),i.clone())),("capacity_canonical".into(),eq_ty(word_ty(),canon(WORD,k.clone()),k.clone())),("ordered".into(),eq(le(i,k.clone()),literal(true))),("extent_no_wrap".into(),eq(carry(WORD,o,k,literal(false)),literal(false)))]);
        let q=bound_proof(p,carry_result);theorem(p,if carry_result {"SliceAccessValidityResliceNumberBoundCarry"} else {"SliceAccessValidityResliceNumberBoundOrder"},&a,q);
    }
    let (x,y)=(v("x"),v("y"));
    let mut a=word_args(&["x","y"]);a.extend([("x_canonical".into(),eq_ty(word_ty(),canon(WORD,x.clone()),x.clone())),("y_canonical".into(),eq_ty(word_ty(),canon(WORD,y.clone()),y.clone())),("no_wrap".into(),eq(carry(WORD,x,y,literal(false)),literal(false)))]);
    let q=sum_lower_proof(p);theorem(p,"SliceAccessValidityResliceNumberSumLower",&a,q);
    save_trace(p,trace,"reslice-sum-lower");
    let (upper,lower)=(v("upper"),v("lower"));let mut a=word_args(&["upper","lower"]);a.push(("ordered".into(),eq(le(lower.clone(),upper.clone()),literal(true))));
    let q=sub_carry_proof(p);theorem(p,"SliceAccessValidityResliceNumberSubCarry",&a,q);
    save_trace(p,trace,"reslice-sub-carry");
    for commuted in [false,true] {for is_carry in [false,true] {
        let mut a=word_args(&["upper","lower"]);a.extend([("upper_canonical".into(),eq_ty(word_ty(),canon(WORD,upper.clone()),upper.clone())),("ordered".into(),eq(le(lower.clone(),upper.clone()),literal(true)))]);
        let q=restore_proof(p,commuted,is_carry);
        theorem(p,match (commuted,is_carry) {(false,false)=>"SliceAccessValidityResliceNumberRestoreSum",(false,true)=>"SliceAccessValidityResliceNumberRestoreCarry",(true,false)=>"SliceAccessValidityResliceNumberRestoreLeftSum",(true,true)=>"SliceAccessValidityResliceNumberRestoreLeftCarry"},&a,q);
        save_trace(p,trace,&format!("reslice-restore-{commuted}-{is_carry}"));
    }}
    let mut a=word_args(&["upper","lower"]);a.extend([("upper_canonical".into(),eq_ty(word_ty(),canon(WORD,upper.clone()),upper.clone())),("lower_canonical".into(),eq_ty(word_ty(),canon(WORD,lower.clone()),lower.clone())),("ordered".into(),eq(le(lower.clone(),upper.clone()),literal(true))),("upper_nonnegative".into(),eq(sign_bit(WORD,upper.clone()),literal(false)))]);
    let q=difference_nonnegative_proof(p);theorem(p,"SliceAccessValidityResliceNumberDifferenceNonnegative",&a,q);
    save_trace(p,trace,"reslice-difference-nonnegative");
    let maximum=v("maximum");let mut a=word_args(&["upper","lower","maximum"]);a.extend([("upper_canonical".into(),eq_ty(word_ty(),canon(WORD,upper.clone()),upper.clone())),("maximum_canonical".into(),eq_ty(word_ty(),canon(WORD,maximum.clone()),maximum.clone())),("lower_upper".into(),eq(le(lower.clone(),upper.clone()),literal(true))),("lower_maximum".into(),eq(le(lower,maximum.clone()),literal(true))),("upper_maximum".into(),eq(le(upper,maximum),literal(true)))]);
    let q=difference_order_proof(p);theorem(p,"SliceAccessValidityResliceNumberDifferenceOrder",&a,q);
    save_trace(p,trace,"reslice-difference-order");
    let q=difference_zero_proof(p);theorem(p,"SliceAccessValidityResliceNumberDifferenceZero",&[],q);
    save_trace(p,trace,"reslice-numeric");
}
