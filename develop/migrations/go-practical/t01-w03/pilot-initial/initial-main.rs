//! Private W03 candidate lemmas. No source profile or production route.
//!
//! The reused file only constructs untrusted ordinary certificate terms.
//! Its old C# status is irrelevant: every new certificate must independently
//! pass both checkers before any feasibility claim can use it.
include!(
    "../../../../migrations/csharp-03/probes/boolean-proof-elimination-logs/producer/builder.rs"
);

use serde_json::{json, Value};
use std::{fs, path::Path};

#[derive(Clone, Debug, Eq, PartialEq)]
enum Expr {
    Constant(String),
    Variable(String),
    Apply(Box<Expr>, Vec<Expr>),
    Lambda(String, Box<Expr>, Box<Expr>),
    Forall(String, Box<Expr>, Box<Expr>),
}

fn c(name: &str) -> Expr {
    Expr::Constant(name.into())
}
fn v(name: &str) -> Expr {
    Expr::Variable(name.into())
}
fn apply(f: Expr, args: Vec<Expr>) -> Expr {
    if args.is_empty() {
        f
    } else {
        Expr::Apply(Box::new(f), args)
    }
}
fn lambda(name: &str, ty: Expr, body: Expr) -> Expr {
    Expr::Lambda(name.into(), Box::new(ty), Box::new(body))
}
fn forall(name: &str, ty: Expr, body: Expr) -> Expr {
    Expr::Forall(name.into(), Box::new(ty), Box::new(body))
}
fn bool_ty() -> Expr {
    c("Std.Bool")
}
fn literal(value: bool) -> Expr {
    c(if value {
        "Std.Bool.true"
    } else {
        "Std.Bool.false"
    })
}
fn eq(left: Expr, right: Expr) -> Expr {
    apply(c(EQ), vec![bool_ty(), left, right])
}
fn refl(value: Expr) -> Expr {
    apply(c(REFL), vec![bool_ty(), value])
}
fn choose_t(ty: Expr, condition: Expr, yes: Expr, no: Expr) -> Expr {
    apply(
        c("Std.Bool.cases"),
        vec![lambda("__type", bool_ty(), ty), no, yes, condition],
    )
}
fn choose(condition: Expr, yes: Expr, no: Expr) -> Expr {
    choose_t(bool_ty(), condition, yes, no)
}
fn not(value: Expr) -> Expr {
    choose(value, literal(false), literal(true))
}
fn and(left: Expr, right: Expr) -> Expr {
    choose(left, right, literal(false))
}
fn or(left: Expr, right: Expr) -> Expr {
    choose(left, literal(true), right)
}
fn xor(left: Expr, right: Expr) -> Expr {
    choose(left, not(right.clone()), right)
}
fn bool_eq(left: Expr, right: Expr) -> Expr {
    choose(left, right.clone(), not(right))
}

fn substitute(expr: &Expr, pattern: &Expr, replacement: &Expr) -> Expr {
    if expr == pattern {
        return replacement.clone();
    }
    match expr {
        Expr::Constant(_) | Expr::Variable(_) => expr.clone(),
        Expr::Apply(f, args) => apply(
            substitute(f, pattern, replacement),
            args.iter()
                .map(|a| substitute(a, pattern, replacement))
                .collect(),
        ),
        Expr::Lambda(name, ty, body) | Expr::Forall(name, ty, body) => {
            let shadowed = pattern == &v(name);
            let new_ty = Box::new(substitute(ty, pattern, replacement));
            let new_body = Box::new(if shadowed {
                (**body).clone()
            } else {
                substitute(body, pattern, replacement)
            });
            if matches!(expr, Expr::Lambda(..)) {
                Expr::Lambda(name.clone(), new_ty, new_body)
            } else {
                Expr::Forall(name.clone(), new_ty, new_body)
            }
        }
    }
}

// Candidate simplification is not trusted. The kernel must still check the
// resulting proof against the original, unsimplified theorem type.
fn simplify(expr: &Expr) -> Expr {
    match expr {
        Expr::Apply(f, args) => {
            let mut f = simplify(f);
            let mut args = args.iter().map(simplify).collect::<Vec<_>>();
            loop {
                if let Expr::Apply(inner, prefix) = f {
                    let mut combined = prefix;
                    combined.append(&mut args);
                    f = *inner;
                    args = combined;
                    continue;
                }
                if let Expr::Lambda(ref name, _, ref body) = f {
                    if !args.is_empty() {
                        let argument = args.remove(0);
                        f = simplify(&substitute(body, &v(name), &argument));
                        continue;
                    }
                }
                if f == c("Std.Bool.cases") && args.len() >= 4 {
                    let selection = if args[3] == literal(false) {
                        Some(1)
                    } else if args[3] == literal(true) {
                        Some(2)
                    } else {
                        None
                    };
                    if let Some(branch) = selection {
                        let selected = args[branch].clone();
                        args.drain(..4);
                        f = simplify(&selected);
                        continue;
                    }
                }
                return apply(f, args);
            }
        }
        Expr::Lambda(name, ty, body) => lambda(name, simplify(ty), simplify(body)),
        Expr::Forall(name, ty, body) => forall(name, simplify(ty), simplify(body)),
        _ => expr.clone(),
    }
}

fn prove_boolean_equality(goal: &Expr, majors: &[Expr], fresh: &mut usize) -> Expr {
    let simplified = simplify(goal);
    let Expr::Apply(ref head, ref args) = simplified else {
        panic!("expected Eq")
    };
    assert_eq!(**head, c(EQ));
    assert_eq!(args.len(), 3);
    if args[1] == args[2] || majors.is_empty() {
        return refl(args[2].clone());
    }
    let name = format!("__case_{}", *fresh);
    *fresh += 1;
    let major = simplify(&majors[0]);
    let motive = lambda(&name, bool_ty(), substitute(&simplified, &major, &v(&name)));
    let no = substitute(&simplified, &major, &literal(false));
    let yes = substitute(&simplified, &major, &literal(true));
    apply(
        c("Std.Bool.cases"),
        vec![
            motive,
            prove_boolean_equality(&no, &majors[1..], fresh),
            prove_boolean_equality(&yes, &majors[1..], fresh),
            major,
        ],
    )
}

fn compile(expr: &Expr, scope: &mut Vec<String>, b: &mut Builder) -> u32 {
    match expr {
        Expr::Constant(name) => b.constant(name).unwrap(),
        Expr::Variable(name) => b
            .var(
                scope
                    .iter()
                    .rev()
                    .position(|n| n == name)
                    .unwrap_or_else(|| panic!("free variable {name}")) as u32,
            )
            .unwrap(),
        Expr::Apply(f, args) => {
            let f = compile(f, scope, b);
            let args = args.iter().map(|a| compile(a, scope, b)).collect();
            b.app(f, args).unwrap()
        }
        Expr::Lambda(name, ty, body) | Expr::Forall(name, ty, body) => {
            let ty = compile(ty, scope, b);
            scope.push(name.clone());
            let body = compile(body, scope, b);
            scope.pop();
            if matches!(expr, Expr::Lambda(..)) {
                b.lam(ty, body).unwrap()
            } else {
                b.pi(ty, body).unwrap()
            }
        }
    }
}

fn cube_type(bits: usize) -> Expr {
    (0..bits).rev().fold(bool_ty(), |body, i| {
        forall(&format!("__type_addr_{i}"), bool_ty(), body)
    })
}
fn addresses(bits: usize, prefix: &str) -> Vec<Expr> {
    (0..bits).map(|i| v(&format!("{prefix}{i}"))).collect()
}
fn cube_lambda(bits: usize, prefix: &str, body: Expr) -> Expr {
    (0..bits).rev().fold(body, |body, i| {
        lambda(&format!("{prefix}{i}"), bool_ty(), body)
    })
}
fn address_binders(bits: usize) -> Vec<(String, Expr)> {
    (0..bits).map(|i| (format!("i{i}"), bool_ty())).collect()
}
fn read(store: Expr, addr: &[Expr]) -> Expr {
    apply(store, addr.to_vec())
}
fn tagged(tag: Expr, payload: Expr, bits: usize) -> Expr {
    let addr = addresses(bits, "__payload_");
    lambda(
        "__tag_field",
        bool_ty(),
        cube_lambda(
            bits,
            "__payload_",
            choose(v("__tag_field"), read(payload, &addr), tag),
        ),
    )
}
fn tag(record: Expr, bits: usize) -> Expr {
    let mut addr = vec![literal(false)];
    addr.extend((0..bits).map(|_| literal(false)));
    read(record, &addr)
}
fn write(store: Expr, addr: &[Expr], value: Expr, wrong: bool) -> Expr {
    let query = addresses(addr.len(), "__write_");
    let equal = query
        .iter()
        .cloned()
        .zip(addr.iter().cloned())
        .fold(literal(true), |p, (x, y)| and(p, bool_eq(x, y)));
    let old = read(store, &query);
    let value = if wrong { old.clone() } else { value };
    cube_lambda(addr.len(), "__write_", choose(equal, value, old))
}

struct Candidate {
    id: String,
    description: String,
    expected: &'static str,
    binders: Vec<(String, Expr)>,
    goal: Expr,
    majors: Vec<Expr>,
}

fn candidates(address_bits: usize) -> Vec<Candidate> {
    let mut out = vec![];
    for wrong in [false, true] {
        let payload = v("payload");
        let actual_tag = if wrong { literal(false) } else { literal(true) };
        let quantity = tagged(actual_tag, payload, 6);
        out.push(Candidate {
            id: format!("nominal-tag{}", if wrong { "-wrong" } else { "" }),
            description: "A quantity-tagged 64-bit value does not satisfy the money-tag predicate, for every payload".into(),
            expected: if wrong { "rejected" } else { "accepted" },
            binders: vec![("payload".into(), cube_type(6))],
            goal: eq(not(tag(quantity, 6)), literal(false)), majors: vec![],
        });
        let addr = addresses(6, "i");
        // The result components are functions, selected by Bool.cases at Sort0.
        let pair = lambda(
            "__field",
            bool_ty(),
            apply(
                c("Std.Bool.cases"),
                vec![
                    lambda("__pair_type", bool_ty(), cube_type(6)),
                    v(if wrong { "right" } else { "left" }),
                    v(if wrong { "left" } else { "right" }),
                    v("__field"),
                ],
            ),
        );
        let first = apply(pair, vec![literal(false)]);
        let mut binders = vec![
            ("left".into(), cube_type(6)),
            ("right".into(), cube_type(6)),
        ];
        binders.extend(address_binders(6));
        out.push(Candidate {
            id: format!(
                "multi-result-projection{}",
                if wrong { "-wrong" } else { "" }
            ),
            description:
                "First projection of a joint pair returns the first 64-bit payload at every address"
                    .into(),
            expected: if wrong { "rejected" } else { "accepted" },
            binders,
            goal: eq(read(first, &addr), read(v("left"), &addr)),
            majors: vec![],
        });
    }
    let error = tagged(v("failed"), v("payload"), 6);
    let is_nil = not(tag(error, 6));
    let error_contract = choose(
        v("failed"),
        bool_eq(is_nil.clone(), literal(false)),
        bool_eq(is_nil, literal(true)),
    );
    out.push(Candidate {
        id: "closed-error-outcome".into(),
        description: "A closed value-error outcome is non-nil on failure and nil on success, for all flags and payloads".into(),
        expected: "accepted", binders: vec![("payload".into(),cube_type(6)),("failed".into(),bool_ty())],
        goal: eq(error_contract,literal(true)), majors: vec![v("failed")],
    });
    let state = cube_type(6);
    let addr = addresses(6, "i");
    let old_bit = read(v("old"), &addr);
    let returned_state = choose_t(state.clone(), v("returned"), v("old"), v("next"));
    let actual = read(returned_state, &addr);
    let mut binders = vec![
        ("old".into(), state.clone()),
        ("next".into(), state.clone()),
        ("returned".into(), bool_ty()),
    ];
    binders.extend(address_binders(6));
    out.push(Candidate {
        id: "return-frame-bit".into(),
        description: "A terminal return preserves every bit of arbitrary 64-bit old state against arbitrary later state updates".into(),
        expected: "accepted", binders,
        goal: eq(choose(v("returned"),bool_eq(actual,old_bit.clone()),literal(true)),literal(true)),
        majors: vec![v("returned"),old_bit],
    });
    let post = v("post");
    let body = v("body");
    let old = v("old");
    let actual = apply(
        post.clone(),
        vec![choose_t(
            state.clone(),
            v("continued"),
            old.clone(),
            apply(body, vec![old.clone()]),
        )],
    );
    let actual = read(actual, &addr);
    let expected = read(apply(post, vec![old]), &addr);
    let transformer = forall("__input_state", state.clone(), state.clone());
    let mut binders = vec![
        ("body".into(), transformer.clone()),
        ("post".into(), transformer),
        ("old".into(), state),
        ("continued".into(), bool_ty()),
    ];
    binders.extend(address_binders(6));
    out.push(Candidate {
        id: "continue-post-bit".into(),
        description: "Continue skips the body but still executes an arbitrary post transformation on a full 64-bit state".into(),
        expected: "accepted",
        binders,
        goal: eq(
            choose(
                v("continued"),
                bool_eq(actual, expected.clone()),
                literal(true),
            ),
            literal(true),
        ),
        majors: vec![v("continued"), expected],
    });
    let xy = xor(v("x"), v("y"));
    let carry = or(and(v("x"), v("y")), and(v("carry"), xy));
    let majority = choose(v("x"), or(v("y"), v("carry")), and(v("y"), v("carry")));
    out.push(Candidate {
        id: "full-adder-carry-bit".into(), description: "Universal three-input carry lemma; a building block, not the full signed-integer/mathematical-sum bridge".into(), expected: "accepted",
        binders: vec![("x".into(),bool_ty()),("y".into(),bool_ty()),("carry".into(),bool_ty())],
        goal: eq(carry,majority), majors: vec![v("x"),v("y"),v("carry")],
    });
    for wrong in [false, true] {
        let addr = addresses(address_bits, "i");
        let updated = write(v("store"), &addr, v("value"), wrong);
        let mut binders = vec![
            ("store".into(), cube_type(address_bits)),
            ("value".into(), bool_ty()),
        ];
        binders.extend(address_binders(address_bits));
        out.push(Candidate {
            id: format!("backing-read-after-write-{address_bits}{}",if wrong {"-wrong"}else{""}),
            description: "Write is visible through an alias of the same backing store, for all stores, values and addresses".into(),
            expected: if wrong {"rejected"}else{"accepted"}, binders,
            goal: eq(read(updated,&addr),v("value")), majors: addr,
        });
    }
    out
}

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    if args.get(1).map(String::as_str) == Some("verify") {
        let data = fs::read(&args[2]).unwrap();
        let output = mpk_kernel::verify_certificate_bytes_json_output(&data);
        println!("{}", output.json);
        if !output.accepted {
            std::process::exit(1);
        }
        return;
    }
    assert!(
        args.len() >= 4 && args[1] == "generate",
        "generate REPO_ROOT OUTPUT [ADDRESS_BITS]"
    );
    let root = Path::new(&args[2]);
    let out = Path::new(&args[3]);
    let address_bits = args.get(4).map(|s| s.parse().unwrap()).unwrap_or(3);
    assert!(address_bits <= 13);
    fs::create_dir_all(out).unwrap();
    let hex = fs::read_to_string(root.join("fixtures/core-bool-cases/right-identity.hex")).unwrap();
    let text: String = hex.chars().filter(|c| !c.is_whitespace()).collect();
    let raw = (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).unwrap())
        .collect::<Vec<_>>();
    let base = mpk_cert::decode_canonical_certificate(&raw).unwrap();
    assert!(
        base.imports.is_empty()
            && base.proof_node_table.is_empty()
            && base.theory_certificates.is_empty()
    );
    assert_eq!(base.axiom_report.summary.total_axiom_count, 0);
    let mut index = vec![];
    for candidate in candidates(address_bits) {
        let mut b = Builder::from(base.clone());
        assert!(matches!(b.c.term_table[b.sort as usize], TermNode::Sort(_)));
        let yes = bit(&mut b, true).unwrap();
        assert_eq!(call(&mut b, "Std.Bool.true", vec![]).unwrap(), yes);
        let component = candidate.id.replace('-', "_");
        b.c.module = format!("Probe.GoPractical.{component}");
        let mut fresh = 0;
        let mut proof = prove_boolean_equality(&candidate.goal, &candidate.majors, &mut fresh);
        let mut ty = candidate.goal.clone();
        for (name, domain) in candidate.binders.iter().rev() {
            ty = forall(name, domain.clone(), ty);
            proof = lambda(name, domain.clone(), proof);
        }
        let ty_id = compile(&ty, &mut vec![], &mut b);
        let proof_id = compile(&proof, &mut vec![], &mut b);
        let name = format!("Probe.GoPractical.{component}.Claim");
        let id = b.c.name_table.len() as u32;
        b.c.name_table.push(name.clone());
        b.c.declarations.push(Declaration {
            name: id,
            kind: DeclarationKind::Theorem {
                ty: ty_id,
                proof: proof_id,
            },
        });
        let terms = b.c.term_table.len();
        let data = b.finish();
        fs::write(out.join(format!("{}.mpcert", candidate.id)), &data).unwrap();
        index.push(json!({"id":candidate.id,"description":candidate.description,"expected":candidate.expected,
            "quantified_binders":candidate.binders.len(),"address_bits":address_bits,
            "terms":terms,"bytes":data.len(),"theorem":name,"goal_debug":format!("{ty:?}"),
            "scope":"ordinary representation lemma; source-binding, complete operations and full arithmetic bridge are not discharged"}));
    }
    let value: Value = json!({"schema":"mpk.go_practical.proof_pilot.v0","cases":index});
    fs::write(
        out.join("index.json"),
        serde_json::to_vec_pretty(&value).unwrap(),
    )
    .unwrap();
}
