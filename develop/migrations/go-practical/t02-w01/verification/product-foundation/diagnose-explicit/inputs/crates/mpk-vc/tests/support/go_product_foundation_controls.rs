use super::inspect;
use mpk_cert::encode::{
    Certificate, Declaration, DeclarationKind, DefinitionReducibility, LevelNode, TermNode,
};
use mpk_vc::go_practical_ordinary::generate_go_practical_product_foundation;

pub(super) fn baseline() -> Certificate {
    static ORIGINAL: std::sync::OnceLock<Certificate> = std::sync::OnceLock::new();
    ORIGINAL
        .get_or_init(|| {
            mpk_cert::decode_canonical_certificate(
                generate_go_practical_product_foundation()
                    .unwrap()
                    .certificate_bytes(),
            )
            .unwrap()
        })
        .clone()
}
pub(super) fn global(c: &Certificate, name: &str) -> usize {
    c.declarations
        .iter()
        .position(|d| c.name_table[d.name as usize] == name)
        .unwrap()
}
pub(super) fn term(c: &mut Certificate, node: TermNode) -> u32 {
    let id = c.term_table.len() as u32;
    c.term_table.push(node);
    id
}
pub(super) fn constant(c: &mut Certificate, name: &str) -> u32 {
    let global = global(c, name) as u32;
    term(
        c,
        TermNode::Const {
            global,
            levels: vec![],
        },
    )
}
pub(super) fn call(c: &mut Certificate, name: &str, arguments: Vec<u32>) -> u32 {
    let function = constant(c, name);
    term(
        c,
        TermNode::App {
            function,
            arguments,
        },
    )
}
pub(super) fn encode(mut c: Certificate) -> Vec<u8> {
    let old = c.name_table.clone();
    c.name_table.sort();
    for d in &mut c.declarations {
        d.name = c.name_table.binary_search(&old[d.name as usize]).unwrap() as u32;
    }
    for level in &mut c.level_table {
        if let LevelNode::Param(name) = level {
            *name = c.name_table.binary_search(&old[*name as usize]).unwrap() as u32;
        }
    }
    c.export_block = mpk_cert::build_export_block(&c).unwrap();
    c.axiom_report = mpk_cert::build_axiom_report(&c).unwrap();
    assert_eq!(c.axiom_report.summary.total_axiom_count, 0);
    c.hashes.export_hash = mpk_cert::export_block_hash(&c.export_block);
    c.hashes.axiom_report_hash = mpk_cert::axiom_report_hash_for_report(&c.axiom_report);
    mpk_cert::encode_certificate_bounded(&c, 2_097_152).unwrap()
}
fn replace_domain(c: &mut Certificate, id: u32, index: usize, domain: u32) -> u32 {
    let TermNode::Pi { ty, body } = c.term_table[id as usize].clone() else {
        panic!("binder")
    };
    if index == 0 {
        term(c, TermNode::Pi { ty: domain, body })
    } else {
        let body = replace_domain(c, body, index - 1, domain);
        term(c, TermNode::Pi { ty, body })
    }
}
fn result(c: &mut Certificate, id: u32, changed: u32) -> u32 {
    match c.term_table[id as usize].clone() {
        TermNode::Pi {ty,body} => {
            let body=result(c,body,changed);
            term(c,TermNode::Pi{ty,body})
        }
        _ => changed,
    }
}
fn boolean_claim(c: &mut Certificate, left: bool, right: bool) -> u32 {
    let ty=constant(c,"Std.Bool");
    let left=constant(c,if left {"Std.Bool.true"} else {"Std.Bool.false"});
    let right=constant(c,if right {"Std.Bool.true"} else {"Std.Bool.false"});
    call(c,"Std.Eq",vec![ty,left,right])
}
fn claim(label: &str, name: &str, premise: Option<usize>) {
    let mut c=baseline();
    let index=global(&c,&format!("Mpk.Go.Practical.Product.{name}"));
    let DeclarationKind::Theorem{ty,proof}=c.declarations[index].kind.clone() else {panic!("theorem")};
    let ty=if let Some(index)=premise {
        let unrelated=boolean_claim(&mut c,true,true);
        replace_domain(&mut c,ty,index,unrelated)
    } else {
        let false_claim=boolean_claim(&mut c,false,true);
        result(&mut c,ty,false_claim)
    };
    c.declarations[index].kind=DeclarationKind::Theorem{ty,proof};
    inspect(label,&encode(c),false);
}
fn body(c: &mut Certificate,id:u32, replacement:&mut impl FnMut(&mut Certificate)->u32)->u32 {
    match c.term_table[id as usize].clone() {
        TermNode::Lam{ty,body:old}=>{
            let body=body(c,old,replacement);
            term(c,TermNode::Lam{ty,body})
        }
        _=>replacement(c),
    }
}
fn variable(c:&mut Certificate,index:u32)->u32{term(c,TermNode::Var(index))}
fn source(label:&str,name:&str, mut replacement:impl FnMut(&mut Certificate)->u32){
    let mut c=baseline();
    let index=global(&c,&format!("Mpk.Go.Practical.Product.{name}"));
    let DeclarationKind::Def{ty,value,reducibility}=c.declarations[index].kind.clone() else {panic!("definition")};
    let value=body(&mut c,value,&mut replacement);
    c.declarations[index].kind=DeclarationKind::Def{ty,value,reducibility};
    inspect(label,&encode(c),false);
}
pub(super) fn honest_different_component()->Vec<u8>{
    let mut c=baseline();
    let ty=constant(&mut c,"Std.Bool");let value=constant(&mut c,"Std.Bool.true");
    let name=c.name_table.len() as u32;
    c.name_table.push("Mpk.Go.Practical.Test.ExtraProduct".into());
    c.declarations.push(Declaration{name,kind:DeclarationKind::Def{ty,value,reducibility:DefinitionReducibility::Reducible}});
    encode(c)
}
pub(super) fn semantic_controls(){
    for name in ["LeftOfMake", "RightOfMake", "PointwiseReflexive", "PointwiseOfProjections", "PointwiseLeft", "PointwiseRight", "PointwiseSymmetric", "PointwiseTransitive", "Reconstruct", "LeftOfSetLeft", "RightOfSetLeft", "LeftOfSetRight", "RightOfSetRight", "OverwriteLeft", "OverwriteRight", "CommuteUpdates", "LeftOfMap", "RightOfMap", "MapIdentity", "MapPointwise", "MapComposition", "ValidMake", "ValidLeft", "ValidRight", "SetLeftValid", "SetRightValid", "MapValid", "UnitZeroValid", "UnitValidUnique"] {
        claim(&format!("False{name}"),name,None);
    }
    for (label,name,premise) in [
        ("FakePointwiseLeft","PointwiseLeft",4),
        ("FakePointwiseRight","PointwiseRight",4),
        ("FakePointwiseTransitivePQ","PointwiseTransitive",5),
        ("FakePointwiseTransitiveQR","PointwiseTransitive",6),
        ("FakeValidMakeX","ValidMake",6),
        ("FakeValidMakeY","ValidMake",7),
        ("FakeSetLeftInputValid","SetLeftValid",5),
        ("FakeSetLeftValueValid","SetLeftValid",7),
        ("FakeSetRightInputValid","SetRightValid",5),
        ("FakeSetRightValueValid","SetRightValid",7),
        ("FakeMapPointwise","MapPointwise",8),
        ("FakeMapValidInput","MapValid",13),
        ("FakeMapValidPreserveF","MapValid",10),
        ("FakeMapValidPreserveG","MapValid",11),
        ("FakeUnitValidUnique","UnitValidUnique",1),
    ] {claim(label,name,Some(premise));}
    source("IgnoreSetLeft","SetLeft",|c|variable(c,1));
    source("IgnoreSetRight","SetRight",|c|variable(c,1));
    source("UnitZeroTrue","UnitZero",|c|constant(c,"Std.Bool.true"));
    source("UnitValidOnlyTrue","UnitValid",|c|{
        let ty=constant(c,"Std.Bool");let unit=variable(c,0);let yes=constant(c,"Std.Bool.true");
        call(c,"Std.Eq",vec![ty,unit,yes])
    });
    source("PointwiseLeftOnly","Pointwise",|c|{
        let boolean=constant(c,"Std.Bool");let no=constant(c,"Std.Bool.false");
        let a=variable(c,4);let p=variable(c,2);let q=variable(c,1);
        let left=term(c,TermNode::App{function:p,arguments:vec![no]});
        let right=term(c,TermNode::App{function:q,arguments:vec![no]});
        let body=call(c,"Std.Eq",vec![a,left,right]);
        term(c,TermNode::Pi{ty:boolean,body})
    });
    source("ValidLeftOnly","Valid",|c|{
        let a=variable(c,4);let b=variable(c,3);let predicate=variable(c,2);let p=variable(c,0);
        let left=call(c,"Mpk.Go.Practical.Product.Left",vec![a,b,p]);
        term(c,TermNode::App{function:predicate,arguments:vec![left]})
    });
}
