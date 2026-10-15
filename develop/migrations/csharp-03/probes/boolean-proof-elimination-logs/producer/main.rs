include!("builder.rs");
use std::{fs, path::Path};
fn theorem(b: &mut Builder, name: &str, ty: u32, proof: u32) {
    let n = b.c.name_table.len() as u32;
    b.c.name_table.push(name.into());
    b.globals.insert(name.into(), b.c.declarations.len() as u32);
    b.c.declarations.push(Declaration {
        name: n,
        kind: DeclarationKind::Theorem { ty, proof },
    });
}
fn goal(b: &mut Builder, x: u32) -> u32 {
    let yes = bit(b, true).unwrap();
    let left = call(b, "Std.Bool.and", vec![x, yes]).unwrap();
    call(b, EQ, vec![b.boolean, left, x]).unwrap()
}
fn install(b: &mut Builder, case: &str) -> (String, u32) {
    let boolean = b.boolean;
    let no = bit(b, false).unwrap();
    let yes = bit(b, true).unwrap();
    let mut sort = b.sort;
    if case == "wrong-motive-universe" {
        let zero =
            b.c.level_table
                .iter()
                .position(|x| matches!(x, LevelNode::Zero))
                .unwrap() as u32;
        let upper = b.c.level_table.len() as u32;
        b.c.level_table.push(LevelNode::Succ(zero));
        sort = b.term(TermNode::Sort(upper)).unwrap();
    }
    if case == "extra-constructor" {
        let n = b.c.name_table.len() as u32;
        b.c.name_table.push("Std.Bool.extra".into());
        b.c.declarations.push(Declaration {
            name: n,
            kind: DeclarationKind::Constructor {
                ty: boolean,
                inductive: b.globals["Std.Bool"],
                generated: true,
            },
        });
    }
    let motive = b.pi(boolean, sort).unwrap();
    let p = b.var(0).unwrap();
    let no_ty = b.app(p, vec![no]).unwrap();
    let p = b.var(1).unwrap();
    let yes_ty = b.app(p, vec![yes]).unwrap();
    let p = b.var(3).unwrap();
    let x = b.var(0).unwrap();
    let result = b.app(p, vec![x]).unwrap();
    let major = b.pi(boolean, result).unwrap();
    let yes_to_major = b.pi(yes_ty, major).unwrap();
    let no_to_yes = b.pi(no_ty, yes_to_major).unwrap();
    let mut ty = b.pi(motive, no_to_yes).unwrap();
    if case == "wrong-interface" {
        let inner = b.pi(boolean, boolean).unwrap();
        let middle = b.pi(boolean, inner).unwrap();
        ty = b.pi(boolean, middle).unwrap();
    }
    let name = if case == "renamed" {
        "Std.Bool.casez"
    } else {
        "Std.Bool.cases"
    }
    .to_owned();
    let n = b.c.name_table.len() as u32;
    b.c.name_table.push(name.clone());
    b.globals
        .insert(name.clone(), b.c.declarations.len() as u32);
    b.c.declarations.push(Declaration {
        name: n,
        kind: DeclarationKind::Recursor {
            ty,
            inductive: b.globals["Std.Bool"],
            generated: case != "nongenerated",
        },
    });
    (name, ty)
}
fn main() {
    let a = std::env::args().collect::<Vec<_>>();
    let original = mpk_cert::decode_canonical_certificate(&fs::read(&a[1]).unwrap()).unwrap();
    let out = Path::new(&a[2]);
    fs::create_dir_all(out).unwrap();
    for case in [
        "right-identity",
        "constructor-false",
        "constructor-true",
        "open-motive",
        "wrong-branch",
        "wrong-motive-universe",
        "wrong-interface",
        "nongenerated",
        "extra-constructor",
        "renamed",
        "wrong-case-levels",
        "wrong-major-levels",
        "extra-constructor-after-cases",
        "conjunction-left",
        "conjunction-right",
        "wrong-conjunction-left",
        "wrong-conjunction-right",
        "wrong-case-levels-open",
        "wrong-major-levels-open",
    ] {
        let mut b = Builder::from(original.clone());
        b.c.module = "Probe.Bool.Cases".into();
        let (name, case_ty) = install(&mut b, case);
        let boolean = b.boolean;
        let no = bit(&mut b, false).unwrap();
        let yes = bit(&mut b, true).unwrap();
        if case == "extra-constructor-after-cases" {
            let n = b.c.name_table.len() as u32;
            b.c.name_table.push("Std.Bool.extra".into());
            b.c.declarations.push(Declaration {
                name: n,
                kind: DeclarationKind::Constructor {
                    ty: boolean,
                    inductive: b.globals["Std.Bool"],
                    generated: true,
                },
            });
        }
        if case == "wrong-major-levels-open" {
            let global = b.globals["Std.Bool.false"];
            let major = b
                .term(TermNode::Const {
                    global,
                    levels: vec![0],
                })
                .unwrap();
            let ty = call(&mut b, EQ, vec![boolean, major, major]).unwrap();
            let proof = call(&mut b, REFL, vec![boolean, major]).unwrap();
            theorem(&mut b, "Probe.Bool.ConstructorUniverseArgument", ty, proof);
        } else if case.contains("conjunction") {
            let bool_eq = |b: &mut Builder, left: u32, right: u32| {
                call(b, EQ, vec![boolean, left, right]).unwrap()
            };
            let refl_true = call(&mut b, REFL, vec![boolean, yes]).unwrap();
            let p = b.var(1).unwrap();
            let q = b.var(0).unwrap();
            let pq = call(&mut b, "Std.Bool.and", vec![p, q]).unwrap();
            let premise = bool_eq(&mut b, pq, yes);
            let shifted = b.var(if case.ends_with("left") { 2 } else { 1 }).unwrap();
            let conclusion = bool_eq(&mut b, shifted, yes);
            let implication = b.pi(premise, conclusion).unwrap();
            let qs = b.pi(boolean, implication).unwrap();
            let theorem_ty = b.pi(boolean, qs).unwrap();
            let branch_motive = {
                let p = b.var(1).unwrap();
                let q = b.var(0).unwrap();
                let pq = call(&mut b, "Std.Bool.and", vec![p, q]).unwrap();
                let premise = bool_eq(&mut b, pq, yes);
                let shifted = b.var(if case.ends_with("left") { 2 } else { 1 }).unwrap();
                let conclusion = bool_eq(&mut b, shifted, yes);
                let implication = b.pi(premise, conclusion).unwrap();
                let qs = b.pi(boolean, implication).unwrap();
                b.lam(boolean, qs).unwrap()
            };
            let false_true = bool_eq(&mut b, no, yes);
            let false_branch = if case.ends_with("left") {
                let id = b.var(0).unwrap();
                let id = b.lam(false_true, id).unwrap();
                b.lam(boolean, id).unwrap()
            } else {
                let x = b.var(1).unwrap();
                let conclusion = bool_eq(&mut b, x, yes);
                let ty = b.pi(false_true, conclusion).unwrap();
                let motive = b.lam(boolean, ty).unwrap();
                let id = b.var(0).unwrap();
                let no_branch = b.lam(false_true, id).unwrap();
                let yes_branch = b.lam(false_true, refl_true).unwrap();
                let q = b.var(0).unwrap();
                let split = call(&mut b, &name, vec![motive, no_branch, yes_branch, q]).unwrap();
                b.lam(boolean, split).unwrap()
            };
            let true_branch = {
                let q = b.var(0).unwrap();
                let premise = bool_eq(&mut b, q, yes);
                let body = if case.ends_with("left") {
                    refl_true
                } else {
                    b.var(0).unwrap()
                };
                let implication = b.lam(premise, body).unwrap();
                b.lam(boolean, implication).unwrap()
            };
            let p = b.var(0).unwrap();
            let branches = if case.starts_with("wrong-") {
                vec![branch_motive, true_branch, false_branch, p]
            } else {
                vec![branch_motive, false_branch, true_branch, p]
            };
            let proof = call(&mut b, &name, branches).unwrap();
            let proof = b.lam(boolean, proof).unwrap();
            theorem(
                &mut b,
                "Probe.Bool.ConjunctionProjection",
                theorem_ty,
                proof,
            );
        } else if case == "open-motive"
            || case == "extra-constructor-after-cases"
            || case == "wrong-case-levels-open"
            || case == "wrong-major-levels-open"
        {
            let sort = b.sort;
            let motive_ty = b.pi(boolean, sort).unwrap();
            let p = b.var(0).unwrap();
            let no_ty = b.app(p, vec![no]).unwrap();
            let p = b.var(1).unwrap();
            let yes_ty = b.app(p, vec![yes]).unwrap();
            let p = b.var(3).unwrap();
            let no_proof = b.var(2).unwrap();
            let yes_proof = b.var(1).unwrap();
            let x = if case == "wrong-major-levels-open" {
                let global = b.globals["Std.Bool.false"];
                b.term(TermNode::Const {
                    global,
                    levels: vec![0],
                })
                .unwrap()
            } else {
                b.var(0).unwrap()
            };
            let f = if case == "wrong-case-levels-open" {
                let global = b.globals[&name];
                b.term(TermNode::Const {
                    global,
                    levels: vec![0],
                })
                .unwrap()
            } else {
                b.constant(&name).unwrap()
            };
            let mut proof = b.app(f, vec![p, no_proof, yes_proof, x]).unwrap();
            for ty in [motive_ty, no_ty, yes_ty, boolean].into_iter().rev() {
                proof = b.lam(ty, proof).unwrap();
            }
            theorem(&mut b, "Probe.Bool.OpenMotive", case_ty, proof);
        } else if [
            "wrong-interface",
            "wrong-motive-universe",
            "nongenerated",
            "extra-constructor",
        ]
        .contains(&case)
        {
        } else {
            let variable = b.var(0).unwrap();
            let g = goal(&mut b, variable);
            let motive = b.lam(boolean, g).unwrap();
            let no_proof = call(&mut b, REFL, vec![boolean, no]).unwrap();
            let yes_proof = call(&mut b, REFL, vec![boolean, yes]).unwrap();
            if case == "right-identity" || case == "wrong-branch" {
                let args = if case == "wrong-branch" {
                    vec![motive, yes_proof, no_proof, variable]
                } else {
                    vec![motive, no_proof, yes_proof, variable]
                };
                let proof = call(&mut b, &name, args).unwrap();
                let proof = b.lam(boolean, proof).unwrap();
                let ty = b.pi(boolean, g).unwrap();
                theorem(&mut b, "Probe.Bool.RightIdentity", ty, proof);
            } else {
                let selected = case == "constructor-true";
                let mut major = if selected { yes } else { no };
                if case == "wrong-major-levels" {
                    let global = b.globals["Std.Bool.false"];
                    major = b
                        .term(TermNode::Const {
                            global,
                            levels: vec![0],
                        })
                        .unwrap();
                }
                let chosen_proof = if selected { yes_proof } else { no_proof };
                let mut function = b.constant(&name).unwrap();
                if case == "wrong-case-levels" {
                    let global = b.globals[&name];
                    function = b
                        .term(TermNode::Const {
                            global,
                            levels: vec![0],
                        })
                        .unwrap();
                }
                let actual = b
                    .app(function, vec![motive, no_proof, yes_proof, major])
                    .unwrap();
                let proposition = goal(&mut b, if selected { yes } else { no });
                let ty = call(&mut b, EQ, vec![proposition, actual, chosen_proof]).unwrap();
                let proof = call(&mut b, REFL, vec![proposition, chosen_proof]).unwrap();
                theorem(&mut b, "Probe.Bool.ConstructorEquation", ty, proof);
            }
        }
        let bytes = b.finish();
        let hex = bytes.iter().map(|x| format!("{x:02x}")).collect::<String>() + "\n";
        fs::write(out.join(format!("{case}.hex")), hex).unwrap();
        println!("{case} {} bytes", bytes.len());
    }
}
