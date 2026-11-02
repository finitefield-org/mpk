//! Damage actual conclusions, premise domains and well-typed runtime bodies.
use super::*;
fn witness_domain(c: &Certificate, mut node: u32) -> bool {
    while let TermNode::Pi { body, .. } = c.term_table[node as usize] {
        node = body;
    }
    let TermNode::App { function, .. } = c.term_table[node as usize] else {
        return false;
    };
    match c.term_table[function as usize] {
        TermNode::Var(_) => true,
        TermNode::Const { global, .. } => {
            let n = &c.name_table[c.declarations[global as usize].name as usize];
            n == "Std.Eq"
                || n.starts_with("Mpk.Go.Practical.")
                    && [".Shape", ".Valid", ".Related"]
                        .iter()
                        .any(|s| n.ends_with(s))
        }
        _ => false,
    }
}
pub(super) fn laws(base: &Certificate) {
    let (mut laws, mut witnesses) = (0, 0);
    for (at, d) in base.declarations.iter().enumerate() {
        let Some(name) =
            base.name_table[d.name as usize].strip_prefix("Mpk.Go.Practical.Storage.Lens.")
        else {
            continue;
        };
        let DeclarationKind::Theorem { ty, proof } = d.kind else {
            continue;
        };
        let mut c = base.clone();
        let changed = bad_result(&mut c, ty);
        c.declarations[at].kind = DeclarationKind::Theorem { ty: changed, proof };
        inspect(&format!("false-{name}"), &encode(c), false);
        laws += 1;
        let (mut cursor, mut binder) = (ty, 0);
        while let TermNode::Pi { ty: domain, body } = base.term_table[cursor as usize] {
            if witness_domain(base, domain) {
                let mut c = base.clone();
                let b = sym(&mut c, "Std.Bool");
                let changed = erase(&mut c, ty, binder, b);
                c.declarations[at].kind = DeclarationKind::Theorem { ty: changed, proof };
                inspect(&format!("premise-{name}-{binder}"), &encode(c), false);
                witnesses += 1;
            }
            cursor = body;
            binder += 1;
        }
    }
    assert_eq!(laws, 181);
    assert!(witnesses > 100);
    retain("control-inventory","json",format!("{}\n",serde_json::json!({"altered_conclusions":laws,"actual_typed_witness_domains":witnesses,"typed_wrong_definitions":16,"zero_axiom_acceptances":2,"complete_candidates":laws+witnesses+18})).as_bytes());
    println!("lens conclusions={laws} actual typed witness domains={witnesses}");
}
fn shift(c: &mut Certificate, node: u32, amount: u32, depth: u32) -> u32 {
    let n = match c.term_table[node as usize].clone() {
        TermNode::Var(i) if i >= depth => TermNode::Var(i + amount),
        TermNode::App {
            function,
            arguments,
        } => {
            let function = shift(c, function, amount, depth);
            let arguments = arguments
                .into_iter()
                .map(|n| shift(c, n, amount, depth))
                .collect();
            TermNode::App {
                function,
                arguments,
            }
        }
        TermNode::Lam { ty, body } => {
            let ty = shift(c, ty, amount, depth);
            let body = shift(c, body, amount, depth + 1);
            TermNode::Lam { ty, body }
        }
        TermNode::Pi { ty, body } => {
            let ty = shift(c, ty, amount, depth);
            let body = shift(c, body, amount, depth + 1);
            TermNode::Pi { ty, body }
        }
        TermNode::Let { ty, value, body } => {
            let ty = shift(c, ty, amount, depth);
            let value = shift(c, value, amount, depth);
            let body = shift(c, body, amount, depth + 1);
            TermNode::Let { ty, value, body }
        }
        _ => return node,
    };
    term(c, n)
}
fn no_op(c: &mut Certificate, root: u32, leaf: u32) -> u32 {
    let leaf = shift(c, leaf, 1, 0);
    let body = term(c, TermNode::Var(1));
    let body = term(c, TermNode::Lam { ty: leaf, body });
    term(c, TermNode::Lam { ty: root, body })
}
fn change_body(c: &mut Certificate, value: u32, arity: usize, name: &str) -> u32 {
    if arity > 0 {
        let TermNode::Lam { ty, body } = c.term_table[value as usize] else {
            panic!("explicit parameter {name}")
        };
        let body = change_body(c, body, arity - 1, name);
        return term(c, TermNode::Lam { ty, body });
    }
    if name == "Put" {
        return term(c, TermNode::Var(1));
    }
    if name == "Get" {
        let TermNode::App {
            function: getter,
            arguments: read_args,
        } = c.term_table[value as usize].clone()
        else {
            unreachable!()
        };
        assert_eq!(read_args.len(), 1);
        let TermNode::App {
            arguments: mut projection,
            ..
        } = c.term_table[getter as usize].clone()
        else {
            unreachable!()
        };
        assert_eq!(projection.len(), 3);
        let putter = call(
            c,
            "Mpk.Go.Practical.Product.Right",
            std::mem::take(&mut projection),
        );
        let updated = term(
            c,
            TermNode::App {
                function: putter,
                arguments: vec![read_args[0], value],
            },
        );
        return term(
            c,
            TermNode::App {
                function: getter,
                arguments: vec![updated],
            },
        );
    }
    let TermNode::App {
        function,
        mut arguments,
    } = c.term_table[value as usize].clone()
    else {
        unreachable!()
    };
    assert_eq!(arguments.len(), 4);
    let (root, leaf) = if name == "Make" {
        (term(c, TermNode::Var(3)), term(c, TermNode::Var(2)))
    } else {
        (arguments[0], arguments[1])
    };
    arguments[3] = no_op(c, root, leaf);
    term(
        c,
        TermNode::App {
            function,
            arguments,
        },
    )
}
pub(super) fn definitions(base: &Certificate) {
    let mut count = 0;
    for (at, d) in base.declarations.iter().enumerate() {
        let Some(name) =
            base.name_table[d.name as usize].strip_prefix("Mpk.Go.Practical.Storage.Lens.")
        else {
            continue;
        };
        let DeclarationKind::Def {
            ty,
            value,
            reducibility,
        } = d.kind
        else {
            continue;
        };
        if name == "Type" {
            continue;
        }
        let arity = match name {
            "Make" | "Get" => 4,
            "Put" | "Compose" => 5,
            "Identity" => 1,
            "LeftField" | "RightField" => 2,
            _ if name.starts_with("ArrayD") && name.ends_with("Element") => 2,
            _ => panic!("uncovered {name}"),
        };
        let mut c = base.clone();
        let value = change_body(&mut c, value, arity, name);
        c.declarations[at].kind = DeclarationKind::Def {
            ty,
            value,
            reducibility,
        };
        let raw = encode(c.clone());
        let label = format!("definition-{name}");
        retain(&label, "mpcert", &raw);
        // Isolate actual definition typing before inspecting the damaged laws.
        let mut prefix = c.clone();
        prefix.declarations.truncate(at + 1);
        mpk_kernel::check_declarations(&prefix).unwrap();
        retain(&label,"typing.json",format!("{}\n",serde_json::json!({"actual_definition_type_checked":true,"prefix_declarations":at+1,"complete_candidate_saved_before_check":true})).as_bytes());
        observations::countermodel(&c, name, retain);
        inspect_saved(&label, &raw, false);
        count += 1;
    }
    assert_eq!(count, 16);
}
