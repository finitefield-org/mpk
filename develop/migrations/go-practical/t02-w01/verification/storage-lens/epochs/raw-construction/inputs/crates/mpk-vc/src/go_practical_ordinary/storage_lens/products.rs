//! Concrete heterogeneous field views preserve sibling observations.
use super::*;
fn rc(name:&str,args:Vec<Expr>)->Expr{call(&format!("Mpk.Go.Practical.Product.Relation.{name}"),args)}
fn projection(a:Expr,b:Expr,p:Expr,right:bool)->Expr{pc(if right{"Right"}else{"Left"},vec![a,b,p])}
fn field(a:Expr,b:Expr,right:bool)->Expr{lc(if right{"RightField"}else{"LeftField"},vec![a,b])}
fn root_relation(a:Expr,b:Expr,r:Expr,s:Expr)->Expr{let ty=product(a.clone(),b.clone());lam("_p",ty.clone(),lam("_q",ty,rc("Related",vec![a,b,r,s,var("_p"),var("_q")]))) }
fn root_validity(a:Expr,b:Expr,p:Expr,q:Expr)->Expr{lam("_root",product(a.clone(),b.clone()),pc("Valid",vec![a,b,p,q,var("_root")]))}
impl Lenses<'_>{
    pub(super) fn products(&mut self)->Result<()>{
        let(a,b,r,s,p,q)=(var("A"),var("B"),var("R"),var("S"),var("P"),var("Q"));let root_type=product(a.clone(),b.clone());
        let types=[("A",Expr::Sort0),("B",Expr::Sort0)];let root=var("_s");let x=var("_x");let y=var("_y");
        for right in [false,true]{
            let name=if right{"RightField"}else{"LeftField"};let leaf=if right{b.clone()}else{a.clone()};let view=field(a.clone(),b.clone(),right);
            let get_expr=projection(a.clone(),b.clone(),root.clone(),right);
            let put_expr=pc(if right{"SetRight"}else{"SetLeft"},vec![a.clone(),b.clone(),root.clone(),x.clone()]);
            self.emit(name,&types,lens(root_type.clone(),leaf.clone()),make(root_type.clone(),leaf.clone(),lam("_s",root_type.clone(),get_expr.clone()),lam("_s",root_type.clone(),lam("_x",leaf.clone(),put_expr.clone()))),false)?;
            let mut read=types.to_vec();read.push(("s",root_type.clone()));let g=projection(a.clone(),b.clone(),var("s"),right);
            self.theorem(&format!("{name}Get"),&read,Equality{ty:leaf.clone(),left:get(root_type.clone(),leaf.clone(),view.clone(),var("s")),right:g.clone(),proof:refl(leaf.clone(),g)})?;
            let mut write=read;write.push(("x",leaf.clone()));let changed=pc(if right{"SetRight"}else{"SetLeft"},vec![a.clone(),b.clone(),var("s"),var("x")]);
            self.theorem(&format!("{name}Put"),&write,Equality{ty:root_type.clone(),left:put(root_type.clone(),leaf.clone(),view.clone(),var("s"),var("x")),right:changed.clone(),proof:refl(root_type.clone(),changed)})?;
            let h=pc(&format!("{}OfSet{}",if right{"Right"}else{"Left"},if right{"Right"}else{"Left"}),vec![a.clone(),b.clone(),root.clone(),x.clone()]);
            self.emit(&format!("{name}GetPut"),&types,get_put(root_type.clone(),leaf.clone(),view.clone()),lam("_s",root_type.clone(),lam("_x",leaf.clone(),h)),true)?;
            let other=if right{a.clone()}else{b.clone()};let other_view=field(a.clone(),b.clone(),!right);
            let mut args=types.to_vec();args.extend([("s",root_type.clone()),("x",leaf.clone())]);
            self.theorem(&format!("{name}SiblingFrame"),&args,Equality{ty:other.clone(),left:get(root_type.clone(),other.clone(),other_view.clone(),put(root_type.clone(),leaf.clone(),view.clone(),var("s"),var("x"))),right:get(root_type.clone(),other,other_view,var("s")),proof:pc(&format!("{}OfSet{}",if right{"Left"}else{"Right"},if right{"Right"}else{"Left"}),vec![a.clone(),b.clone(),var("s"),var("x")])})?;
            let mut related=types.to_vec();related.extend([("R",relation(a.clone())),("S",relation(b.clone()))]);
            let root_rel=root_relation(a.clone(),b.clone(),r.clone(),s.clone());let child_rel=if right{s.clone()}else{r.clone()};
            let input=rc(if right{"RelatedRight"}else{"RelatedLeft"},vec![a.clone(),b.clone(),r.clone(),s.clone(),root.clone(),var("_t"),var("_h")]);
            self.emit(&format!("{name}GetRespect"),&related,get_respect(root_type.clone(),leaf.clone(),root_rel.clone(),child_rel.clone(),view.clone()),lam("_s",root_type.clone(),lam("_t",root_type.clone(),lam("_h",holds(root_rel.clone(),root.clone(),var("_t")),input))),true)?;
            related.extend([("reflA",pi("_value",a.clone(),holds(r.clone(),var("_value"),var("_value")))),("reflB",pi("_value",b.clone(),holds(s.clone(),var("_value"),var("_value"))))]);
            let source=rc("Reflexive",vec![a.clone(),b.clone(),r.clone(),s.clone(),root.clone(),var("reflA"),var("reflB")]);
            let h=rc(if right{"SetRightRelated"}else{"SetLeftRelated"},vec![a.clone(),b.clone(),r.clone(),s.clone(),root.clone(),root.clone(),source,x.clone(),y.clone(),var("_h")]);
            self.emit(&format!("{name}PutRespect"),&related,put_respect(root_type.clone(),leaf.clone(),root_rel.clone(),child_rel.clone(),view.clone()),lam("_s",root_type.clone(),lam("_x",leaf.clone(),lam("_y",leaf.clone(),lam("_h",holds(child_rel,x.clone(),y.clone()),h)))),true)?;
            let replaced=put(root_type.clone(),leaf.clone(),view.clone(),root.clone(),get(root_type.clone(),leaf.clone(),view.clone(),root.clone()));
            let left=projection(a.clone(),b.clone(),root.clone(),false);let right_value=projection(a.clone(),b.clone(),root.clone(),true);
            let h=rc("RelatedMake",vec![a.clone(),b.clone(),r.clone(),s.clone(),replaced,root.clone(),app(var("reflA"),vec![left]),app(var("reflB"),vec![right_value])]);
            self.emit(&format!("{name}PutGet"),&related,put_get(root_type.clone(),leaf.clone(),root_rel.clone(),view.clone()),lam("_s",root_type.clone(),h),true)?;
            let twice=put(root_type.clone(),leaf.clone(),view.clone(),put(root_type.clone(),leaf.clone(),view.clone(),root.clone(),x.clone()),y.clone());let once=put(root_type.clone(),leaf.clone(),view.clone(),root.clone(),y.clone());
            let left=if right{projection(a.clone(),b.clone(),root.clone(),false)}else{y.clone()};let end=if right{y.clone()}else{projection(a.clone(),b.clone(),root.clone(),true)};
            let h=rc("RelatedMake",vec![a.clone(),b.clone(),r.clone(),s.clone(),twice,once,app(var("reflA"),vec![left]),app(var("reflB"),vec![end])]);
            self.emit(&format!("{name}PutPut"),&related,put_put(root_type.clone(),leaf.clone(),root_rel,view.clone()),lam("_s",root_type.clone(),lam("_x",leaf.clone(),lam("_y",leaf.clone(),h))),true)?;
            let mut valid=types.to_vec();valid.extend([("P",predicate(a.clone())),("Q",predicate(b.clone()))]);let root_pred=root_validity(a.clone(),b.clone(),p.clone(),q.clone());let child_pred=if right{q.clone()}else{p.clone()};
            self.emit(&format!("{name}GetValid"),&valid,get_valid(root_type.clone(),leaf.clone(),root_pred.clone(),child_pred.clone(),view.clone()),lam("_s",root_type.clone(),lam("_valid",app(root_pred.clone(),vec![root.clone()]),pc(if right{"ValidRight"}else{"ValidLeft"},vec![a.clone(),b.clone(),p.clone(),q.clone(),root.clone(),var("_valid")]))),true)?;
            self.emit(&format!("{name}PutValid"),&valid,put_valid(root_type.clone(),leaf.clone(),root_pred.clone(),child_pred.clone(),view),lam("_s",root_type.clone(),lam("_x",leaf.clone(),lam("_valid",app(root_pred,vec![root.clone()]),lam("_value_valid",app(child_pred,vec![x.clone()]),pc(if right{"SetRightValid"}else{"SetLeftValid"},vec![a.clone(),b.clone(),p.clone(),q.clone(),root.clone(),var("_valid"),x.clone(),var("_value_valid")]))))),true)?;
        }
        Ok(())
    }
}
