//! Untrusted closed ordinary-term emitter, using the retained private core proposal.
include!(
    "../../support/builder.rs"
);

use mpk_cert::encode::DefinitionReducibility;
use serde_json::{json, Value};
use std::{env, fs};

fn fields(value: &Value, expected: &[&str]) {
    let object = value.as_object().expect("closed expression object");
    assert_eq!(object.len(), expected.len());
    assert!(expected.iter().all(|key| object.contains_key(*key)));
}

fn compile(value: &Value, scope: &mut Vec<String>, builder: &mut Builder) -> u32 {
    match value["kind"].as_str().unwrap() {
        "sort0" => {
            fields(value, &["kind"]);
            builder.sort
        }
        "const" => {
            fields(value, &["kind", "name"]);
            let name = value["name"].as_str().unwrap();
            if name == EQ || name == REFL {
                call(builder, name, vec![]).unwrap()
            } else if name == "Std.Bool.true" || name == "Std.Bool.false" {
                bit(builder, name == "Std.Bool.true").unwrap()
            } else {
                builder.constant(name).unwrap()
            }
        }
        "var" => {
            fields(value, &["kind", "name"]);
            let index = scope
                .iter()
                .rev()
                .position(|s| s == value["name"].as_str().unwrap());
            builder
                .var(index.expect("no free variable") as u32)
                .unwrap()
        }
        "app" => {
            fields(value, &["kind", "function", "arguments"]);
            let f = compile(&value["function"], scope, builder);
            let args = value["arguments"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| compile(v, scope, builder))
                .collect();
            builder.app(f, args).unwrap()
        }
        "pi" | "lam" => {
            fields(value, &["kind", "name", "domain", "body"]);
            let domain = compile(&value["domain"], scope, builder);
            scope.push(value["name"].as_str().unwrap().to_owned());
            let body = compile(&value["body"], scope, builder);
            scope.pop();
            if value["kind"] == "pi" {
                builder.pi(domain, body).unwrap()
            } else {
                builder.lam(domain, body).unwrap()
            }
        }
        _ => panic!("closed ordinary AST; no axiom/opcode extension"),
    }
}

fn main() {
    let args: Vec<_> = env::args().collect();
    if args[1] == "verify" {
        let raw = fs::read(&args[2]).unwrap();
        let output = mpk_kernel::verify_certificate_bytes_json_output(&raw);
        println!("{}", output.json);
        if !output.accepted {
            std::process::exit(1);
        }
        return;
    }
    assert_eq!(args.len(), 5, "emit SEED AST OUTPUT");
    assert_eq!(args[1], "emit");
    let seed = mpk_cert::decode_canonical_certificate(&fs::read(&args[2]).unwrap()).unwrap();
    assert_eq!(seed.axiom_report.summary.total_axiom_count, 0);
    assert!(
        seed.imports.is_empty()
            && seed.proof_node_table.is_empty()
            && seed.theory_certificates.is_empty()
    );
    let plan: Value = serde_json::from_slice(&fs::read(&args[3]).unwrap()).unwrap();
    fields(&plan, &["module", "declarations"]);
    let mut builder = Builder::from(seed);
    builder.c.module = plan["module"].as_str().unwrap().to_owned();
    let mut added = Vec::new();
    for declaration in plan["declarations"].as_array().unwrap() {
        fields(declaration, &["name", "kind", "ty", "value"]);
        let name = declaration["name"].as_str().unwrap().to_owned();
        assert!(!builder.globals.contains_key(&name));
        let ty = compile(&declaration["ty"], &mut Vec::new(), &mut builder);
        let value = compile(&declaration["value"], &mut Vec::new(), &mut builder);
        let global = builder.c.declarations.len() as u32;
        let name_index = builder.c.name_table.len() as u32;
        builder.c.name_table.push(name.clone());
        let kind = match declaration["kind"].as_str().unwrap() {
            "definition" => DeclarationKind::Def {
                ty,
                value,
                reducibility: DefinitionReducibility::Reducible,
            },
            "theorem" => DeclarationKind::Theorem { ty, proof: value },
            _ => panic!("only ordinary definitions/theorems"),
        };
        builder.c.declarations.push(Declaration {
            name: name_index,
            kind,
        });
        builder.globals.insert(name.clone(), global);
        added.push(name);
    }
    let terms = builder.c.term_table.len();
    let declarations = builder.c.declarations.len();
    let raw = builder.finish();
    fs::write(&args[4], &raw).unwrap();
    println!(
        "{}",
        json!({"bytes":raw.len(),"terms":terms,"declarations":declarations,"added":added})
    );
}
