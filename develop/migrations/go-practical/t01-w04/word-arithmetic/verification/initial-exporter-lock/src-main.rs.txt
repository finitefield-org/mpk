//! Untrusted export of unchanged ordinary word declarations into a new seed.
use mpk_cert::encode::{Certificate, DeclarationKind, DefinitionReducibility, LevelNode, TermNode};
use serde_json::{json, Value};
use std::{collections::BTreeMap, env, fs};

fn name(c: &Certificate, global: u32) -> &str {
    &c.name_table[c.declarations[global as usize].name as usize]
}
fn level(c: &Certificate, index: u32) -> Value {
    match c.level_table[index as usize] {
        LevelNode::Zero => json!(["zero"]),
        LevelNode::Succ(a) => json!(["succ", level(c, a)]),
        LevelNode::Max(a, b) => json!(["max", level(c, a), level(c, b)]),
        LevelNode::Param(a) => json!(["param", c.name_table[a as usize]]),
    }
}
fn signature(c: &Certificate, index: u32) -> Value {
    match &c.term_table[index as usize] {
        TermNode::Sort(a) => json!(["sort", level(c, *a)]),
        TermNode::Var(a) => json!(["var", a]),
        TermNode::Const { global, levels } => json!([
            "const",
            name(c, *global),
            levels.iter().map(|a| level(c, *a)).collect::<Vec<_>>()
        ]),
        TermNode::App {
            function,
            arguments,
        } => json!([
            "app",
            signature(c, *function),
            arguments
                .iter()
                .map(|a| signature(c, *a))
                .collect::<Vec<_>>()
        ]),
        TermNode::Lam { ty, body } => json!(["lam", signature(c, *ty), signature(c, *body)]),
        TermNode::Pi { ty, body } => json!(["pi", signature(c, *ty), signature(c, *body)]),
        TermNode::Let { ty, value, body } => json!([
            "let",
            signature(c, *ty),
            signature(c, *value),
            signature(c, *body)
        ]),
    }
}
fn declaration_signature(c: &Certificate, kind: &DeclarationKind) -> Value {
    match *kind {
        DeclarationKind::Def {
            ty,
            value,
            reducibility,
        } => json!([
            "def",
            format!("{reducibility:?}"),
            signature(c, ty),
            signature(c, value)
        ]),
        DeclarationKind::Theorem { ty, proof } => {
            json!(["theorem", signature(c, ty), signature(c, proof)])
        }
        DeclarationKind::Inductive { ty } => json!(["inductive", signature(c, ty)]),
        DeclarationKind::Constructor {
            ty,
            inductive,
            generated,
        } => json!([
            "constructor",
            signature(c, ty),
            name(c, inductive),
            generated
        ]),
        DeclarationKind::Recursor {
            ty,
            inductive,
            generated,
        } => json!(["recursor", signature(c, ty), name(c, inductive), generated]),
        _ => panic!("no axiom or theory declaration"),
    }
}
fn ordinary(c: &Certificate, index: u32, scope: &mut Vec<String>) -> Value {
    match &c.term_table[index as usize] {
        TermNode::Sort(a) => {
            assert_eq!(c.level_table[*a as usize], LevelNode::Zero);
            json!({"kind":"sort0"})
        }
        TermNode::Var(a) => json!({"kind":"var","name":scope[scope.len()-1-*a as usize]}),
        TermNode::Const { global, levels } => {
            assert!(levels.is_empty(), "no implicit universe instantiation");
            json!({"kind":"const","name":name(c,*global)})
        }
        TermNode::App {
            function,
            arguments,
        } => {
            json!({"kind":"app","function":ordinary(c,*function,scope),"arguments":arguments.iter().map(|a| ordinary(c,*a,scope)).collect::<Vec<_>>()})
        }
        TermNode::Lam { ty, body } | TermNode::Pi { ty, body } => {
            let domain = ordinary(c, *ty, scope);
            let binder = format!("_retained_bound{}", scope.len());
            scope.push(binder.clone());
            let result = ordinary(c, *body, scope);
            scope.pop();
            json!({"kind":if matches!(c.term_table[index as usize],TermNode::Lam{..}) {"lam"} else {"pi"},"name":binder,"domain":domain,"body":result})
        }
        _ => panic!("only unchanged closed ordinary AST arms"),
    }
}
fn load(path: &str) -> Certificate {
    let raw = fs::read(path).unwrap();
    let c = mpk_cert::decode_canonical_certificate(&raw).unwrap();
    assert!(
        c.imports.is_empty() && c.proof_node_table.is_empty() && c.theory_certificates.is_empty()
    );
    assert_eq!(c.axiom_report.summary.total_axiom_count, 0);
    let checked = mpk_kernel::verify_certificate_bytes_json_output(&raw);
    assert!(checked.accepted, "{}", checked.json);
    c
}
fn main() {
    let args: Vec<_> = env::args().collect();
    assert_eq!(args.len(), 5, "export BASE SOURCE OUTPUT");
    assert_eq!(args[1], "export");
    let base = load(&args[2]);
    let source = load(&args[3]);
    let globals: BTreeMap<_, _> = base
        .declarations
        .iter()
        .map(|d| (&base.name_table[d.name as usize], &d.kind))
        .collect();
    let mut common = Vec::new();
    let mut declarations = Vec::new();
    for d in &source.declarations {
        let n = &source.name_table[d.name as usize];
        if let Some(kind) = globals.get(n) {
            assert_eq!(
                declaration_signature(&base, kind),
                declaration_signature(&source, &d.kind),
                "changed shared declaration: {n}"
            );
            common.push(n.clone());
            continue;
        }
        assert!(
            n.starts_with("Probe.GoPractical.Word."),
            "unexpected retained namespace"
        );
        let (kind, ty, value) = match d.kind {
            DeclarationKind::Def {
                ty,
                value,
                reducibility: DefinitionReducibility::Reducible,
            } => ("definition", ty, value),
            DeclarationKind::Theorem { ty, proof } => ("theorem", ty, proof),
            _ => panic!("only ordinary definitions/theorems may be exported"),
        };
        declarations.push(json!({"name":n,"kind":kind,"ty":ordinary(&source,ty,&mut Vec::new()),"value":ordinary(&source,value,&mut Vec::new())}));
    }
    let exported = declarations.len();
    fs::write(&args[4], serde_json::to_vec(&json!({"module":"Probe.GoPractical.WordArithmetic.Retained","declarations":declarations})).unwrap()).unwrap();
    println!(
        "{}",
        json!({"source_declarations":source.declarations.len(),"base_declarations":base.declarations.len(),"common":common,"exported":exported})
    );
}
