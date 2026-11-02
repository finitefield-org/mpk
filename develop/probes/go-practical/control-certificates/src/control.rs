//! Ordinary induction for arbitrary Nat counts and explicit abrupt-flow edges.
use super::*;
fn frame() -> Expr {
    cube_type(7)
}
fn unary(output: Expr) -> Expr {
    forall("__frame_argument", frame(), output)
}
fn eq_frame(a: Expr, b: Expr) -> Expr {
    apply(c(EQ), vec![frame(), a, b])
}
fn rfl(x: Expr) -> Expr {
    apply(c(REFL), vec![frame(), x])
}
fn congr(ty: Expr, f: Expr, x: Expr, y: Expr, h: Expr) -> Expr {
    apply(c("Std.Eq.congr"), vec![ty, frame(), f, x, y, h])
}
fn trans(a: Expr, b: Expr, d: Expr, p: Expr, q: Expr) -> Expr {
    apply(c("Std.Eq.trans"), vec![frame(), a, b, d, p, q])
}
fn iterate(next: Expr, n: Expr, initial: Expr) -> Expr {
    apply(c(&name("LoopIterate")), vec![next, n, initial])
}
fn natrec(motive: Expr, base: Expr, step: Expr, n: Expr) -> Expr {
    apply(c("Std.Nat.induct"), vec![motive, base, step, n])
}
fn induction_interface(b: &mut Builder) {
    let natural = b.constant("Std.Nat").unwrap();
    let zero = b.constant("Std.Nat.zero").unwrap();
    let successor = b.constant("Std.Nat.succ").unwrap();
    let motive_ty = b.pi(natural, b.sort).unwrap();
    let p = b.var(0).unwrap();
    let zero_ty = b.app(p, vec![zero]).unwrap();
    let p = b.var(2).unwrap();
    let n = b.var(0).unwrap();
    let ih = b.app(p, vec![n]).unwrap();
    let p = b.var(3).unwrap();
    let n = b.var(1).unwrap();
    let sn = b.app(successor, vec![n]).unwrap();
    let sn = b.app(p, vec![sn]).unwrap();
    let step = b.pi(ih, sn).unwrap();
    let step = b.pi(natural, step).unwrap();
    let p = b.var(3).unwrap();
    let n = b.var(0).unwrap();
    let result = b.app(p, vec![n]).unwrap();
    let result = b.pi(natural, result).unwrap();
    let result = b.pi(step, result).unwrap();
    let result = b.pi(zero_ty, result).unwrap();
    let ty = b.pi(motive_ty, result).unwrap();
    let global = b.c.declarations.len() as u32;
    let name = b.c.name_table.len() as u32;
    b.c.name_table.push("Std.Nat.induct".into());
    b.c.declarations.push(Declaration {
        name,
        kind: DeclarationKind::Recursor {
            ty,
            inductive: b.globals["Std.Nat"],
            generated: true,
        },
    });
    b.globals.insert("Std.Nat.induct".into(), global);
}
fn interface_candidates(base: &Certificate, output: &Path) {
    let original = Builder::from(base.clone());
    let nat = original.globals["Std.Nat"];
    let zero = original.globals["Std.Nat.zero"];
    let succ = original.globals["Std.Nat.succ"];
    let induct = original.globals["Std.Nat.induct"];
    let mut cases = vec![];
    for variant in [
        "legacy-no-induct",
        "wrong-interface",
        "nongenerated",
        "wrong-zero-name",
        "wrong-succ-type",
        "nongenerated-zero",
        "extra-constructor-before",
        "extra-constructor-after",
        "higher-universe-family",
        "induct-levels-partial",
        "family-levels",
        "constructor-levels",
        "prior-opaque-levels",
        "prior-theorem-levels",
        "unused-term-levels",
    ] {
        let mut b = Builder::from(base.clone());
        let mut good = false;
        match variant {
            "legacy-no-induct" => {
                b.c.declarations.truncate(induct as usize);
                good = true;
            }
            "wrong-interface" => {
                let ty = b.constant("Std.Nat").unwrap();
                if let DeclarationKind::Recursor {
                    ty: ref mut old, ..
                } = b.c.declarations[induct as usize].kind
                {
                    *old = ty;
                }
            }
            "nongenerated" => {
                if let DeclarationKind::Recursor {
                    ref mut generated, ..
                } = b.c.declarations[induct as usize].kind
                {
                    *generated = false;
                }
            }
            "wrong-zero-name" => {
                let id = b.c.name_table.len() as u32;
                b.c.name_table.push("Std.Nat.renamed".into());
                b.c.declarations[zero as usize].name = id;
            }
            "wrong-succ-type" => {
                let ty = b.constant("Std.Nat").unwrap();
                if let DeclarationKind::Constructor {
                    ty: ref mut old, ..
                } = b.c.declarations[succ as usize].kind
                {
                    *old = ty;
                }
            }
            "nongenerated-zero" => {
                if let DeclarationKind::Constructor {
                    ref mut generated, ..
                } = b.c.declarations[zero as usize].kind
                {
                    *generated = false;
                }
            }
            "extra-constructor-before" | "extra-constructor-after" => {
                b.c.declarations.truncate(induct as usize + 1);
                let ty = b.constant("Std.Nat").unwrap();
                let id = b.c.name_table.len() as u32;
                b.c.name_table.push("Std.Nat.extra".into());
                let extra = Declaration {
                    name: id,
                    kind: DeclarationKind::Constructor {
                        ty,
                        inductive: nat,
                        generated: true,
                    },
                };
                if variant.ends_with("before") {
                    let d = b.c.declarations.pop().unwrap();
                    b.c.declarations.extend([extra, d]);
                } else {
                    b.c.declarations.push(extra);
                }
            }
            "higher-universe-family" => {
                let level = b.c.level_table.len() as u32;
                b.c.level_table.push(LevelNode::Succ(0));
                let ty = b.term(TermNode::Sort(level)).unwrap();
                b.c.declarations[nat as usize].kind = DeclarationKind::Inductive { ty };
            }
            "unused-term-levels" => {
                b.term(TermNode::Const {
                    global: zero,
                    levels: vec![0],
                })
                .unwrap();
                good = true;
            }
            _ => {
                // Keep old declaration indexes valid. The unused term table may
                // contain arbitrary nodes; only actual checked uses are rejected.
                b.c.declarations.truncate(induct as usize + 1);
                let interface = b.c.declarations.pop().unwrap();
                let before = variant.starts_with("prior-");
                if !before {
                    b.c.declarations.push(interface.clone());
                }
                let native_ty = b.constant("Std.Nat").unwrap();
                let (ty, value) = if variant == "induct-levels-partial" {
                    let DeclarationKind::Recursor { ty, .. } = interface.kind else {
                        unreachable!()
                    };
                    (
                        ty,
                        b.term(TermNode::Const {
                            global: induct,
                            levels: vec![0],
                        })
                        .unwrap(),
                    )
                } else {
                    let ty = if variant == "family-levels" {
                        b.term(TermNode::Const {
                            global: nat,
                            levels: vec![0],
                        })
                        .unwrap()
                    } else {
                        native_ty
                    };
                    (
                        ty,
                        b.term(TermNode::Const {
                            global: zero,
                            levels: vec![0],
                        })
                        .unwrap(),
                    )
                };
                let id = b.c.name_table.len() as u32;
                b.c.name_table
                    .push("Probe.GoPractical.Word.InterfaceUse".into());
                let kind = if variant == "prior-theorem-levels" {
                    DeclarationKind::Theorem { ty, proof: value }
                } else {
                    DeclarationKind::Def {
                        ty,
                        value,
                        reducibility: mpk_cert::encode::DefinitionReducibility::Opaque,
                    }
                };
                b.c.declarations.push(Declaration { name: id, kind });
                if before {
                    b.c.declarations.push(interface);
                }
            }
        }
        let terms = b.c.term_table.len();
        let declarations = b.c.declarations.len();
        let bytes = b.finish();
        let id = format!("induction-interface-{variant}");
        fs::write(output.join(format!("{id}.mpcert")), &bytes).unwrap();
        cases.push(json!({"id":id,"variant":variant,"expected":if good {"accepted"} else {"rejected"},"declarations":declarations,"terms":terms,"bytes":bytes.len(),"scope":"private proposed Nat.induct interface validation; malformed declaration rejection is distinct from semantic false-claim rejection"}));
    }
    fs::write(
        output.join("interfaces.json"),
        serde_json::to_vec_pretty(
            &json!({"schema":"mpk.go_practical.induction_interface_pilot.v0","cases":cases}),
        )
        .unwrap(),
    )
    .unwrap();
}
fn halted_step(halted: Expr, next: Expr, s: Expr) -> Expr {
    apply(c(&name("LoopHaltedStep")), vec![halted, next, s])
}
fn preservation(s: Expr) -> Expr {
    eq(apply(v("invariant"), vec![s]), literal(true))
}
fn invariant_arguments() -> Vec<(String, Expr)> {
    let s = v("__preservation_state");
    vec![
        ("next".into(), unary(frame())),
        ("invariant".into(), unary(bool_ty())),
        ("initial".into(), frame()),
        ("entry_proof".into(), preservation(v("initial"))),
        (
            "body_proof".into(),
            forall(
                "__preservation_state",
                frame(),
                forall(
                    "__preservation_input",
                    preservation(s.clone()),
                    preservation(apply(v("next"), vec![s])),
                ),
            ),
        ),
        ("n".into(), c("Std.Nat")),
    ]
}
fn invariant_proof() -> Expr {
    let at = |n| iterate(v("next"), n, v("initial"));
    let motive = lambda(
        "__induction_n",
        c("Std.Nat"),
        preservation(at(v("__induction_n"))),
    );
    let step = lambda(
        "__induction_n",
        c("Std.Nat"),
        lambda(
            "__induction_h",
            preservation(at(v("__induction_n"))),
            apply(
                v("body_proof"),
                vec![at(v("__induction_n")), v("__induction_h")],
            ),
        ),
    );
    natrec(motive, v("entry_proof"), step, v("n"))
}
fn stopped_proof() -> Expr {
    let s = v("state");
    let test = apply(v("halted"), vec![s.clone()]);
    congr(
        bool_ty(),
        lambda(
            "__halted_test",
            bool_ty(),
            choose_t(
                frame(),
                v("__halted_test"),
                s.clone(),
                apply(v("next"), vec![s]),
            ),
        ),
        test,
        literal(true),
        v("stopped"),
    )
}
fn absorb_arguments() -> Vec<(String, Expr)> {
    vec![
        ("halted".into(), unary(bool_ty())),
        ("next".into(), unary(frame())),
        ("state".into(), frame()),
        (
            "stopped".into(),
            eq(apply(v("halted"), vec![v("state")]), literal(true)),
        ),
        ("n".into(), c("Std.Nat")),
    ]
}
fn absorb_proof() -> Expr {
    let s = v("state");
    let f = lambda(
        "__halted_state",
        frame(),
        halted_step(v("halted"), v("next"), v("__halted_state")),
    );
    let at = |n| iterate(f.clone(), n, s.clone());
    let k = v("__absorb_n");
    let previous = at(k.clone());
    let first = halted_step(v("halted"), v("next"), previous.clone());
    let middle = halted_step(v("halted"), v("next"), s.clone());
    let h = congr(frame(), f.clone(), previous, s.clone(), v("__absorb_h"));
    let step = lambda(
        "__absorb_n",
        c("Std.Nat"),
        lambda(
            "__absorb_h",
            eq_frame(at(k), s.clone()),
            trans(first, middle, s.clone(), h, stopped_proof()),
        ),
    );
    natrec(
        lambda(
            "__absorb_n",
            c("Std.Nat"),
            eq_frame(at(v("__absorb_n")), s.clone()),
        ),
        rfl(s),
        step,
        v("n"),
    )
}
fn control_body() -> Expr {
    choose_t(
        frame(),
        v("returned"),
        v("return_frame"),
        choose_t(
            frame(),
            v("broken"),
            v("break_frame"),
            choose_t(
                frame(),
                v("continued"),
                apply(v("post"), vec![v("body_frame")]),
                apply(v("post"), vec![apply(v("rest"), vec![v("body_frame")])]),
            ),
        ),
    )
}
fn edge_arguments() -> Vec<(String, Expr)> {
    vec![
        ("returned".into(), bool_ty()),
        ("broken".into(), bool_ty()),
        ("continued".into(), bool_ty()),
        ("body_frame".into(), frame()),
        ("return_frame".into(), frame()),
        ("break_frame".into(), frame()),
        ("rest".into(), unary(frame())),
        ("post".into(), unary(frame())),
    ]
}
fn edge_proof(flags: &[(&str, bool)], target: Expr) -> Expr {
    let original = control_body();
    let mut current = original.clone();
    let mut proof = rfl(original.clone());
    for (key, value) in flags {
        let replacement = literal(*value);
        let next = substitute(&current, &v(key), &replacement);
        let q = congr(
            bool_ty(),
            lambda(
                "__edge_flag",
                bool_ty(),
                substitute(&current, &v(key), &v("__edge_flag")),
            ),
            v(key),
            replacement,
            v(&format!("{key}_guard")),
        );
        proof = trans(original.clone(), current, next.clone(), proof, q);
        current = next;
    }
    trans(original, current, target.clone(), proof, rfl(target))
}
pub(super) fn generate(root: &Path, output: &Path) {
    fs::create_dir_all(output).unwrap();
    let mut p = Producer {
        builder: Builder::from(natural::foundation(root)),
        fresh: 0,
    };
    induction_interface(&mut p.builder);
    p.builder.c.module = "Probe.GoPractical.ControlInduction".into();
    let a = vec![
        ("next".into(), unary(frame())),
        ("n".into(), c("Std.Nat")),
        ("initial".into(), frame()),
    ];
    let value = natrec(
        lambda("__iteration_n", c("Std.Nat"), frame()),
        v("initial"),
        lambda(
            "__iteration_n",
            c("Std.Nat"),
            lambda(
                "__iteration_frame",
                frame(),
                apply(v("next"), vec![v("__iteration_frame")]),
            ),
        ),
        v("n"),
    );
    let (ty, value) = fold_binders(&a, frame(), value);
    p.declare("LoopIterate", ty, value, false);
    let a = vec![
        ("halted".into(), unary(bool_ty())),
        ("next".into(), unary(frame())),
        ("state".into(), frame()),
    ];
    let value = choose_t(
        frame(),
        apply(v("halted"), vec![v("state")]),
        v("state"),
        apply(v("next"), vec![v("state")]),
    );
    let (ty, value) = fold_binders(&a, frame(), value);
    p.declare("LoopHaltedStep", ty, value, false);
    let base = p.builder.c;
    interface_candidates(&base, output);
    let mut cases = vec![];
    for family in [
        "invariant-entry",
        "invariant-step",
        "absorbing-stop",
        "continue-post",
        "break-exit",
        "return-precedence",
    ] {
        for mutation in 0..2 {
            let mut p = Producer {
                builder: Builder::from(base.clone()),
                fresh: 0,
            };
            let (mut a, mut goal, proof) = if family.starts_with("invariant-") {
                (
                    invariant_arguments(),
                    preservation(iterate(v("next"), v("n"), v("initial"))),
                    invariant_proof(),
                )
            } else if family == "absorbing-stop" {
                let f = lambda(
                    "__halted_state",
                    frame(),
                    halted_step(v("halted"), v("next"), v("__halted_state")),
                );
                (
                    absorb_arguments(),
                    eq_frame(iterate(f, v("n"), v("state")), v("state")),
                    absorb_proof(),
                )
            } else {
                let (flags, target) = match family {
                    "continue-post" => (
                        vec![("returned", false), ("broken", false), ("continued", true)],
                        apply(v("post"), vec![v("body_frame")]),
                    ),
                    "break-exit" => (
                        vec![("returned", false), ("broken", true)],
                        v("break_frame"),
                    ),
                    _ => (vec![("returned", true)], v("return_frame")),
                };
                let mut a = edge_arguments();
                for (key, value) in &flags {
                    a.push((format!("{key}_guard"), eq(v(key), literal(*value))));
                }
                (
                    a,
                    eq_frame(control_body(), target.clone()),
                    edge_proof(&flags, target),
                )
            };
            if mutation == 1 {
                if family == "invariant-entry" {
                    a.iter_mut().find(|(n, _)| n == "entry_proof").unwrap().1 =
                        eq(literal(true), literal(true));
                } else if family == "invariant-step" {
                    a.iter_mut().find(|(n, _)| n == "body_proof").unwrap().1 = forall(
                        "__preservation_state",
                        frame(),
                        forall(
                            "__preservation_input",
                            preservation(v("__preservation_state")),
                            eq(literal(true), literal(true)),
                        ),
                    );
                } else if family == "absorbing-stop" {
                    a.iter_mut().find(|(n, _)| n == "stopped").unwrap().1 =
                        eq(literal(true), literal(true));
                } else if let Expr::Apply(_, ref mut args) = goal {
                    args[2] = if family == "continue-post" {
                        apply(v("post"), vec![apply(v("rest"), vec![v("body_frame")])])
                    } else if family == "break-exit" {
                        apply(v("post"), vec![v("body_frame")])
                    } else {
                        v("break_frame")
                    };
                }
            }
            let claim = format!("ControlClaim{}", family.replace('-', ""));
            let (goal, proof) = fold_binders(&a, goal, proof);
            p.declare(&claim, goal.clone(), proof, true);
            let id = format!(
                "control-{family}-{}",
                if mutation == 0 { "valid" } else { "altered" }
            );
            let terms = p.builder.c.term_table.len();
            let declarations = p.builder.c.declarations.len();
            let bytes = p.builder.finish();
            fs::write(output.join(format!("{id}.mpcert")), &bytes).unwrap();
            cases.push(json!({"id":id,"family":family,"mutation":mutation,"expected":if mutation==0 {"accepted"} else {"rejected"},"frame_bits":128,"iteration_domain":if family.starts_with("invariant-") || family=="absorbing-stop" {"arbitrary genuine Std.Nat; no finite unrolling"} else {"single source-shaped abrupt edge; no iteration-count premise"},"declarations":declarations,"terms":terms,"bytes":bytes.len(),"theorem":name(&claim),"scope":"ordinary loop/control proof schema; entry and body preservation are proof obligations, never caller entry assumptions; production source and variant bindings remain due in T06","goal_debug":format!("{goal:?}")}));
        }
    }
    fs::write(
        output.join("index.json"),
        serde_json::to_vec_pretty(
            &json!({"schema":"mpk.go_practical.control_pilot.v0","cases":cases}),
        )
        .unwrap(),
    )
    .unwrap();
}
