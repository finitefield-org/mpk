use super::emit;
use mpk_cert::encode::{
    Certificate, Declaration, DeclarationKind, DefinitionReducibility, LevelNode, TermNode,
};
use mpk_vc::go_practical_ordinary::{
    generate_go_practical_word_order_foundation,
    import_go_practical_word_order_foundation, GoOrdinaryError,
};

fn baseline() -> Certificate {
    mpk_cert::decode_canonical_certificate(
        generate_go_practical_word_order_foundation()
            .unwrap()
            .certificate_bytes(),
    )
    .unwrap()
}
fn global(c: &Certificate, name: &str) -> usize {
    c.declarations
        .iter()
        .position(|d| c.name_table[d.name as usize] == name)
        .unwrap()
}
fn term(c: &mut Certificate, node: TermNode) -> u32 {
    let id = c.term_table.len() as u32;
    c.term_table.push(node);
    id
}
fn constant(c: &mut Certificate, name: &str) -> u32 {
    let global = global(c, name) as u32;
    term(
        c,
        TermNode::Const {
            global,
            levels: vec![],
        },
    )
}
fn call(c: &mut Certificate, name: &str, arguments: Vec<u32>) -> u32 {
    let function = constant(c, name);
    term(
        c,
        TermNode::App {
            function,
            arguments,
        },
    )
}
fn encode(mut c: Certificate) -> Vec<u8> {
    let old = c.name_table.clone();
    c.name_table.sort();
    for declaration in &mut c.declarations {
        declaration.name = c
            .name_table
            .binary_search(&old[declaration.name as usize])
            .unwrap() as u32;
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
fn reject(name: &str, c: Certificate) {
    let bytes = encode(c);
    assert_eq!(
        import_go_practical_word_order_foundation(&bytes),
        Err(GoOrdinaryError::Linkage)
    );
    emit(name, &bytes, false);
}

pub(super) fn honest_different_component() -> Vec<u8> {
    let mut c = baseline();
    let ty = constant(&mut c, "Std.Bool");
    let value = constant(&mut c, "Std.Bool.true");
    let name = c.name_table.len() as u32;
    c.name_table
        .push("Mpk.Go.Practical.Test.ExtraOrder".into());
    c.declarations.push(Declaration {
        name,
        kind: DeclarationKind::Def {
            ty,
            value,
            reducibility: DefinitionReducibility::Reducible,
        },
    });
    encode(c)
}

fn replace_domain(c:&mut Certificate,id:u32,index:usize,domain:u32)->u32 {
    let TermNode::Pi{ty,body}=c.term_table[id as usize].clone()else{panic!("theorem binder");};
    if index==0 {term(c,TermNode::Pi{ty:domain,body})}
    else {let body=replace_domain(c,body,index-1,domain);term(c,TermNode::Pi{ty,body})}
}
fn conclusion(c:&mut Certificate,id:u32,mutation:&str)->u32 {
    match c.term_table[id as usize].clone() {
        TermNode::Pi{ty,body}=>{let body=conclusion(c,body,mutation);term(c,TermNode::Pi{ty,body})},
        TermNode::App{function,mut arguments}=>{
            assert_eq!(arguments.len(),3);
            match mutation {
                "FalseBound"=>{
                    let TermNode::App{function,mut arguments:less_args}=c.term_table[arguments[1] as usize].clone()else{panic!("natural less");};
                    assert_eq!(less_args.len(),2);less_args[1]=less_args[0];arguments[1]=term(c,TermNode::App{function,arguments:less_args});
                },
                "FalseLess"=>arguments[2]=constant(c,"Std.Bool.false"),
                "FalseEqual"|"FalseTotal"=>arguments[2]=constant(c,"Std.Bool.true"),
                "FalseZero"=>{let zero=constant(c,"Std.Nat.zero");arguments[2]=call(c,"Std.Nat.succ",vec![zero]);},
                "WrongOppositeCarry"=>{
                    let TermNode::App{arguments:flags,..}=c.term_table[arguments[1] as usize].clone()else{panic!("carry conjunction");};
                    arguments[1]=call(c,"Mpk.Go.Practical.Bool.Or",flags);
                },
                _=>panic!("isolated conclusion control"),
            }
            term(c,TermNode::App{function,arguments})
        },
        _=>panic!("ordinary equality"),
    }
}

#[test]
fn false_complement_total_order_carry_and_ignored_bound_reject() {
    for mutation in ["WrongNotLeaf","FalseTotal","WrongOppositeCarry","IgnoredBound"] {
        let mut c=baseline();let name=match mutation{"WrongNotLeaf"=>"NotCube0","FalseTotal"=>"NaturalTotalEquality","WrongOppositeCarry"=>"OppositeCarry",_=>"SubtractionOrder"};
        let index=global(&c,&format!("Mpk.Go.Practical.Word.Order.{name}"));
        match c.declarations[index].kind.clone() {
            DeclarationKind::Def{ty,reducibility,..}=>{
                let boolean=constant(&mut c,"Std.Bool");let body=term(&mut c,TermNode::Var(0));let value=term(&mut c,TermNode::Lam{ty:boolean,body});
                c.declarations[index].kind=DeclarationKind::Def{ty,value,reducibility};
            },
            DeclarationKind::Theorem{ty,proof}=>{
                let ty=if mutation=="IgnoredBound" {
                    let bt=constant(&mut c,"Std.Bool");let t=constant(&mut c,"Std.Bool.true");let unrelated=call(&mut c,"Std.Eq",vec![bt,t,t]);replace_domain(&mut c,ty,5,unrelated)
                }else{conclusion(&mut c,ty,mutation)};
                c.declarations[index].kind=DeclarationKind::Theorem{ty,proof};
            },
            _=>panic!("ordinary member"),
        }
        reject(mutation,c);
    }
}

#[test]
fn false_actual_bounds_subtraction_comparisons_zero_and_divisor_guards_reject() {
    for width in [8,16,32,64] {
        for mutation in ["FalseBound","WrongSub","FalseLess","FalseEqual","FalseZero","FakeDivisorGuard"] {
            let mut c=baseline();let suffix=match mutation{"FalseBound"=>"ValueBound","WrongSub"=>"","FalseLess"=>"UnsignedLessNatural","FalseEqual"=>"EqualNatural","FalseZero"=>"ZeroValue",_=>"DefinedDivisorPositive"};
            let name=if mutation=="WrongSub"{format!("Mpk.Go.Practical.Word.W{width}.Sub")}else{format!("Mpk.Go.Practical.Word.Order.W{width}.{suffix}")};
            let index=global(&c,&name);
            match c.declarations[index].kind.clone() {
                DeclarationKind::Def{ty,value,reducibility}=>{
                    let TermNode::Lam{ty:xt,body}=c.term_table[value as usize]else{panic!("x");};
                    let TermNode::Lam{ty:yt,..}=c.term_table[body as usize]else{panic!("y");};
                    let x=term(&mut c,TermNode::Var(1));let y=term(&mut c,TermNode::Var(0));let body=call(&mut c,&format!("Mpk.Go.Practical.Word.W{width}.Add"),vec![x,y]);
                    let body=term(&mut c,TermNode::Lam{ty:yt,body});let value=term(&mut c,TermNode::Lam{ty:xt,body});
                    c.declarations[index].kind=DeclarationKind::Def{ty,value,reducibility};
                },
                DeclarationKind::Theorem{ty,proof}=>{
                    let ty=if mutation=="FakeDivisorGuard" {
                        let bt=constant(&mut c,"Std.Bool");let t=constant(&mut c,"Std.Bool.true");let unrelated=call(&mut c,"Std.Eq",vec![bt,t,t]);replace_domain(&mut c,ty,1,unrelated)
                    }else{conclusion(&mut c,ty,mutation)};
                    c.declarations[index].kind=DeclarationKind::Theorem{ty,proof};
                },
                _=>panic!("ordinary member"),
            }
            reject(&format!("{mutation}-{width}"),c);
        }
    }
}
