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

mod natural;
mod word;

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

fn equality_trans(left: Expr, middle: Expr, right: Expr, first: Expr, second: Expr) -> Expr {
    apply(
        c("Std.Eq.trans"),
        vec![bool_ty(), left, middle, right, first, second],
    )
}

fn weak_beta(expr: &Expr) -> Expr {
    let mut current = expr.clone();
    loop {
        let Expr::Apply(head, mut args) = current else {
            return current;
        };
        match *head {
            Expr::Apply(inner, mut prefix) => {
                prefix.append(&mut args);
                current = apply(*inner, prefix);
            }
            Expr::Lambda(name, _, body) if !args.is_empty() => {
                let first = args.remove(0);
                current = apply(substitute(&body, &v(&name), &first), args);
            }
            head => return apply(head, args),
        }
    }
}

// Current conversion only fires iota on a constructor-headed major. Supply
// checked congruence/transitivity rather than assuming that nested majors
// normalize automatically or changing either checker to accept our tactic.
fn normalize_ground_boolean(expr: &Expr, fresh: &mut usize) -> (Expr, Expr) {
    let beta = weak_beta(expr);
    let beta_proof = refl(beta.clone());
    let Expr::Apply(ref head, ref arguments) = beta else {
        return (beta, beta_proof);
    };
    if **head != c("Std.Bool.cases") || arguments.len() < 4 {
        return (beta, beta_proof);
    }
    let original_major = arguments[3].clone();
    let (major, major_proof) = normalize_ground_boolean(&original_major, fresh);
    let branch = if major == literal(false) {
        1
    } else if major == literal(true) {
        2
    } else {
        return (beta, beta_proof);
    };
    let name = format!("__normalize_major_{}", *fresh);
    *fresh += 1;
    let mut context_args = arguments.clone();
    context_args[3] = v(&name);
    let context = lambda(&name, bool_ty(), apply(c("Std.Bool.cases"), context_args));
    let mut rewritten_args = arguments.clone();
    rewritten_args[3] = major.clone();
    let rewritten = apply(c("Std.Bool.cases"), rewritten_args);
    let congruence = apply(
        c("Std.Eq.congr"),
        vec![
            bool_ty(),
            bool_ty(),
            context,
            original_major,
            major,
            major_proof,
        ],
    );
    let selected = apply(arguments[branch].clone(), arguments[4..].to_vec());
    let selected_proof = refl(selected.clone());
    let (normal, remainder) = normalize_ground_boolean(&selected, fresh);
    let tail = equality_trans(
        rewritten.clone(),
        selected,
        normal.clone(),
        selected_proof,
        remainder,
    );
    let proof = equality_trans(beta.clone(), rewritten, normal.clone(), congruence, tail);
    if &beta == expr {
        (normal, proof)
    } else {
        (
            normal.clone(),
            equality_trans(expr.clone(), beta, normal, beta_proof, proof),
        )
    }
}

fn prove_with_checked_normalization(goal: &Expr, majors: &[Expr], fresh: &mut usize) -> Expr {
    let Expr::Apply(head, args) = goal else {
        panic!("expected equality goal")
    };
    assert_eq!(**head, c(EQ));
    assert_eq!(args.len(), 3);
    if args[1] == args[2] {
        return refl(args[2].clone());
    }
    if majors.is_empty() {
        let (left, left_proof) = normalize_ground_boolean(&args[1], fresh);
        let (right, right_proof) = normalize_ground_boolean(&args[2], fresh);
        if left != right {
            // Keep a well-formed false-claim candidate for the negative tests.
            return refl(args[2].clone());
        }
        let reversed = apply(
            c("Std.Eq.symm"),
            vec![bool_ty(), args[2].clone(), right, right_proof],
        );
        return equality_trans(args[1].clone(), left, args[2].clone(), left_proof, reversed);
    }
    let name = format!("__exact_case_{}", *fresh);
    *fresh += 1;
    let motive = lambda(&name, bool_ty(), substitute(goal, &majors[0], &v(&name)));
    let no = substitute(goal, &majors[0], &literal(false));
    let yes = substitute(goal, &majors[0], &literal(true));
    apply(
        c("Std.Bool.cases"),
        vec![
            motive,
            prove_with_checked_normalization(&no, &majors[1..], fresh),
            prove_with_checked_normalization(&yes, &majors[1..], fresh),
            majors[0].clone(),
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

fn write_cell(
    store: Expr,
    address: &[Expr],
    value: Expr,
    payload_address_bits: usize,
    wrong: bool,
) -> Expr {
    let cells = addresses(address.len(), "__cell_");
    let payload = addresses(payload_address_bits, "__value_bit_");
    let equal = cells
        .iter()
        .cloned()
        .zip(address.iter().cloned())
        .fold(literal(true), |p, (x, y)| and(p, bool_eq(x, y)));
    let mut old_address = cells.clone();
    old_address.extend(payload.clone());
    let old = read(store, &old_address);
    let new_value = if wrong {
        old.clone()
    } else {
        read(value, &payload)
    };
    cube_lambda(
        address.len(),
        "__cell_",
        cube_lambda(
            payload_address_bits,
            "__value_bit_",
            choose(equal, new_value, old),
        ),
    )
}

fn cell_candidates(kind: &str, capacity: usize, payload_address_bits: usize) -> Vec<Candidate> {
    if capacity == 0 {
        let index = addresses(6, "i");
        let zero_word = cube_lambda(6, "__zero_length_", literal(false));
        let mut out = vec![Candidate {
            id: format!("{kind}-capacity-0-length-zero"),
            description: format!(
                "The empty {kind} constructor has a zero 64-bit length word, at every bit address"
            ),
            expected: "accepted",
            binders: address_binders(6),
            goal: eq(read(zero_word, &index), literal(false)),
            majors: vec![],
        }];
        if kind == "slice" {
            out.push(Candidate{
                id:"slice-nonnil-empty-tag".into(),
                description:"A non-nil empty slice retains a false nil tag for every inactive payload; a separate validity/source binding is still required".into(),
                expected:"accepted",binders:vec![("padding".into(),cube_type(3))],
                goal:eq(tag(tagged(literal(false),v("padding"),3),3),literal(false)),majors:vec![],
            });
        }
        return out;
    }
    assert!(capacity.is_power_of_two());
    let cell_bits = capacity.trailing_zeros() as usize;
    let cell = addresses(cell_bits, "i");
    let payload = addresses(payload_address_bits, "j");
    let mut query = cell.clone();
    query.extend(payload.clone());
    let mut binders = vec![
        ("store".into(), cube_type(cell_bits + payload_address_bits)),
        ("value".into(), cube_type(payload_address_bits)),
    ];
    binders.extend(address_binders(cell_bits));
    binders.extend((0..payload_address_bits).map(|i| (format!("j{i}"), bool_ty())));
    [false,true].into_iter().map(|wrong|{
        let updated=write_cell(v("store"),&cell,v("value"),payload_address_bits,wrong);
        Candidate{
            id:format!("{kind}-capacity-{capacity}-cell-write{}",if wrong{"-wrong"}else{""}),
            description:format!("A {}-bit cell update is visible at every payload bit in a {capacity}-cell {kind} store; all complete stores and cell values are quantified",1usize<<payload_address_bits),
            expected:if wrong{"rejected"}else{"accepted"},binders:binders.clone(),
            goal:eq(read(updated,&query),read(v("value"),&payload)),majors:cell.clone(),
        }
    }).collect()
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
    for capacity in [0, 1, 32, 256] {
        out.extend(cell_candidates("string", capacity, 3));
    }
    for capacity in [0, 1, 8, 32, 128] {
        out.extend(cell_candidates("slice", capacity, 6));
    }
    out
}

fn describe_term(c: &Certificate, id: u32, scope: &mut Vec<String>) -> String {
    match &c.term_table[id as usize] {
        TermNode::Sort(level) => format!("Sort({level})"),
        TermNode::Var(index) => scope
            .iter()
            .rev()
            .nth(*index as usize)
            .cloned()
            .unwrap_or_else(|| format!("free{index}")),
        TermNode::Const { global, .. } => {
            c.name_table[c.declarations[*global as usize].name as usize].clone()
        }
        TermNode::App {
            function,
            arguments,
        } => format!(
            "({} {})",
            describe_term(c, *function, scope),
            arguments
                .iter()
                .map(|a| describe_term(c, *a, scope))
                .collect::<Vec<_>>()
                .join(" ")
        ),
        TermNode::Lam { ty, body } | TermNode::Pi { ty, body } => {
            let domain = describe_term(c, *ty, scope);
            let name = format!("x{}", scope.len());
            scope.push(name.clone());
            let body = describe_term(c, *body, scope);
            scope.pop();
            format!(
                "({} {name}:{domain}. {body})",
                if matches!(c.term_table[id as usize], TermNode::Lam { .. }) {
                    "lambda"
                } else {
                    "forall"
                }
            )
        }
        TermNode::Let { ty, value, body } => format!(
            "(let {} {} {})",
            describe_term(c, *ty, scope),
            describe_term(c, *value, scope),
            describe_term(c, *body, scope)
        ),
    }
}

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    if matches!(
        args.get(1).map(String::as_str),
        Some("signed-generate" | "signed-trace")
    ) {
        assert_eq!(args.len(), 5, "signed-generate REPO_ROOT OUTPUT DEPTH");
        word::generate_signed(
            Path::new(&args[2]),
            Path::new(&args[3]),
            args[4].parse().unwrap(),
            args[1] == "signed-trace",
        );
        return;
    }
    if matches!(
        args.get(1).map(String::as_str),
        Some("tree-generate" | "tree-trace")
    ) {
        assert_eq!(args.len(), 5, "tree-generate REPO_ROOT OUTPUT DEPTH");
        word::generate_balanced(
            Path::new(&args[2]),
            Path::new(&args[3]),
            args[4].parse().unwrap(),
            args[1] == "tree-trace",
        );
        return;
    }
    if args.get(1).map(String::as_str) == Some("word-generate") {
        assert_eq!(args.len(), 5, "word-generate REPO_ROOT OUTPUT WIDTH");
        word::generate(
            Path::new(&args[2]),
            Path::new(&args[3]),
            args[4].parse().unwrap(),
        );
        return;
    }
    if args.get(1).map(String::as_str) == Some("natural-generate") {
        assert!(
            (4..=5).contains(&args.len()),
            "natural-generate REPO_ROOT OUTPUT [CASE_ID]"
        );
        natural::generate(Path::new(&args[2]), Path::new(&args[3]), args.get(4));
        return;
    }
    if args.get(1).map(String::as_str) == Some("describe") {
        let certificate =
            mpk_cert::decode_canonical_certificate(&fs::read(&args[2]).unwrap()).unwrap();
        for decl in &certificate.declarations {
            let name = &certificate.name_table[decl.name as usize];
            if name == &args[3] {
                let ty = match decl.kind {
                    DeclarationKind::Theorem { ty, .. }
                    | DeclarationKind::Def { ty, .. }
                    | DeclarationKind::Inductive { ty }
                    | DeclarationKind::Constructor { ty, .. }
                    | DeclarationKind::Recursor { ty, .. }
                    | DeclarationKind::Axiom { ty }
                    | DeclarationKind::TheoryPrimitive { ty } => ty,
                };
                println!("{name}: {}", describe_term(&certificate, ty, &mut vec![]));
            }
        }
        return;
    }
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
    let selection = args.get(5);
    let all = candidates(address_bits);
    if let Some(selection) = selection {
        assert!(all.iter().any(|c| &c.id == selection), "unknown pilot case");
    }
    for candidate in all
        .into_iter()
        .filter(|candidate| selection.is_none_or(|id| &candidate.id == id))
    {
        let mut b = Builder::from(base.clone());
        assert!(matches!(b.c.term_table[b.sort as usize], TermNode::Sort(_)));
        let yes = bit(&mut b, true).unwrap();
        assert_eq!(call(&mut b, "Std.Bool.true", vec![]).unwrap(), yes);
        let component = candidate.id.replace('-', "_");
        b.c.module = format!("Probe.GoPractical.{component}");
        let mut fresh = 0;
        let checked_normalization = candidate.id.starts_with("nominal-tag")
            || candidate.id == "closed-error-outcome"
            || candidate.id == "full-adder-carry-bit"
            || candidate.id.starts_with("backing-read-after-write")
            || candidate.id.starts_with("string-")
            || candidate.id.starts_with("slice-");
        let mut proof = if checked_normalization {
            prove_with_checked_normalization(&candidate.goal, &candidate.majors, &mut fresh)
        } else {
            prove_boolean_equality(&candidate.goal, &candidate.majors, &mut fresh)
        };
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
            "quantified_binders":candidate.binders.len(),"pilot_address_bits":address_bits,
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
