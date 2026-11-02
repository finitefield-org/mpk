use super::emit;
use mpk_cert::encode::{
    Certificate, Declaration, DeclarationKind, DefinitionReducibility, LevelNode, TermNode,
};
use mpk_vc::go_practical_ordinary::{
    generate_go_practical_word_signed_division_refinement,
    import_go_practical_word_signed_division_refinement, GoOrdinaryError,
};

fn baseline() -> Certificate {
    static ORIGINAL: std::sync::OnceLock<Certificate> = std::sync::OnceLock::new();
    ORIGINAL
        .get_or_init(|| {
            mpk_cert::decode_canonical_certificate(
                generate_go_practical_word_signed_division_refinement()
                    .unwrap()
                    .certificate_bytes(),
            )
            .unwrap()
        })
        .clone()
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
        import_go_practical_word_signed_division_refinement(&bytes),
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
        .push("Mpk.Go.Practical.Test.ExtraSignedDivisionRefinement".into());
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

fn replace_domain(c: &mut Certificate, id: u32, index: usize, domain: u32) -> u32 {
    let TermNode::Pi { ty, body } = c.term_table[id as usize].clone() else {
        panic!("binder");
    };
    if index == 0 {
        term(c, TermNode::Pi { ty: domain, body })
    } else {
        let body = replace_domain(c, body, index - 1, domain);
        term(c, TermNode::Pi { ty, body })
    }
}
fn unrelated(c: &mut Certificate) -> u32 {
    let bt = constant(c, "Std.Bool");
    let t = constant(c, "Std.Bool.true");
    call(c, "Std.Eq", vec![bt, t, t])
}
fn conclusion(c: &mut Certificate, id: u32, value: &str) -> u32 {
    match c.term_table[id as usize].clone() {
        TermNode::Pi { ty, body } => {
            let body = conclusion(c, body, value);
            term(c, TermNode::Pi { ty, body })
        }
        TermNode::App {
            function,
            mut arguments,
        } => {
            assert_eq!(arguments.len(), 3);
            arguments[2] = if value == "nat-one" {
                let zero = constant(c, "Std.Nat.zero");
                call(c, "Std.Nat.succ", vec![zero])
            } else {
                constant(c, value)
            };
            term(
                c,
                TermNode::App {
                    function,
                    arguments,
                },
            )
        }
        _ => panic!("equality"),
    }
}
fn constant_body(c: &mut Certificate, id: u32, value: &str) -> u32 {
    match c.term_table[id as usize].clone() {
        TermNode::Lam { ty, body } => {
            let body = constant_body(c, body, value);
            term(c, TermNode::Lam { ty, body })
        }
        _ => constant(c, value),
    }
}

fn false_relation(c: &mut Certificate, id: u32) -> u32 {
    match c.term_table[id as usize].clone() {
        TermNode::Pi { ty, body } => {
            let body = false_relation(c, body);
            term(c, TermNode::Pi { ty, body })
        }
        TermNode::App {
            function,
            mut arguments,
        } => {
            assert_eq!(arguments.len(), 2);
            let zero = constant(c, "Std.Nat.zero");
            arguments[1] = call(c, "Mpk.Go.Practical.Int.FromNat", vec![zero]);
            term(
                c,
                TermNode::App {
                    function,
                    arguments,
                },
            )
        }
        _ => panic!("ordinary balanced-Int relation"),
    }
}
fn replace_constant(c: &mut Certificate, id: u32, old: usize, new: usize) -> u32 {
    let node = match c.term_table[id as usize].clone() {
        TermNode::Const { global, levels } if global as usize == old => TermNode::Const {
            global: new as u32,
            levels,
        },
        TermNode::App {
            function,
            arguments,
        } => {
            let function = replace_constant(c, function, old, new);
            let arguments = arguments
                .into_iter()
                .map(|i| replace_constant(c, i, old, new))
                .collect();
            TermNode::App {
                function,
                arguments,
            }
        }
        TermNode::Pi { ty, body } => {
            let ty = replace_constant(c, ty, old, new);
            let body = replace_constant(c, body, old, new);
            TermNode::Pi { ty, body }
        }
        TermNode::Lam { ty, body } => {
            let ty = replace_constant(c, ty, old, new);
            let body = replace_constant(c, body, old, new);
            TermNode::Lam { ty, body }
        }
        TermNode::Let { ty, value, body } => {
            let ty = replace_constant(c, ty, old, new);
            let value = replace_constant(c, value, old, new);
            let body = replace_constant(c, body, old, new);
            TermNode::Let { ty, value, body }
        }
        _ => return id,
    };
    term(c, node)
}
fn correction_part(c: &mut Certificate, id: u32, negative: bool) -> u32 {
    match c.term_table[id as usize].clone() {
        TermNode::Lam { ty, body } => {
            let body = correction_part(c, body, negative);
            term(c, TermNode::Lam { ty, body })
        }
        TermNode::App {
            function,
            mut arguments,
        } => {
            assert_eq!(arguments.len(), 2);
            arguments[usize::from(negative)] = constant(c, "Std.Nat.zero");
            term(
                c,
                TermNode::App {
                    function,
                    arguments,
                },
            )
        }
        _ => panic!("computed correction Make"),
    }
}

#[test]
fn false_signed_nat_scalar_conservation_signs_and_premises_reject() {
    for label in [
        "FalseSignedNatProjection",
        "FalseScalarConservation",
        "FakeScalarBalance",
        "FalseScalarSign",
    ] {
        let mut c = baseline();
        let name = if label == "FalseSignedNatProjection" {
            "SignedNatPositive"
        } else {
            "ScalarSignedDivision"
        };
        let i = global(&c, &format!("Mpk.Go.Practical.Word.SignedDivision.{name}"));
        let DeclarationKind::Theorem { ty, proof } = c.declarations[i].kind.clone() else {
            panic!("ordinary theorem")
        };
        let ty = match label {
            "FalseSignedNatProjection" => conclusion(&mut c, ty, "Std.Nat.zero"),
            "FalseScalarConservation" => false_relation(&mut c, ty),
            "FakeScalarBalance" => {
                let domain = unrelated(&mut c);
                replace_domain(&mut c, ty, 4, domain)
            }
            "FalseScalarSign" => {
                let old = global(&c, "Mpk.Go.Practical.Bool.Xor");
                let new = global(&c, "Mpk.Go.Practical.Bool.And");
                replace_constant(&mut c, ty, old, new)
            }
            _ => unreachable!(),
        };
        c.declarations[i].kind = DeclarationKind::Theorem { ty, proof };
        reject(label, c);
    }
}

#[test]
fn false_actual_selection_corrections_guards_source_results_and_reconstruction_reject() {
    for w in [8, 16, 32, 64] {
        for label in [
            "FalseCorrectionPositive",
            "FalseCorrectionNegative",
            "FalseSelectionRefinement",
            "FalseMagnitudeConservation",
            "FakeMagnitudeGuard",
            "FalseMagnitudeRemainderBound",
            "FalseQuotientRefinement",
            "FalseRemainderRefinement",
            "FalseMathematicalConservation",
            "FakeMathematicalGuard",
            "FalseSourceConservation",
            "FakeSourceGuard",
            "ChangedSignedDivSource",
            "ChangedSignedRemSource",
        ] {
            let mut c = baseline();
            if label.starts_with("Changed") {
                let name = if label == "ChangedSignedDivSource" {
                    "SignedDiv"
                } else {
                    "SignedRem"
                };
                let i = global(&c, &format!("Mpk.Go.Practical.Word.W{w}.{name}"));
                let DeclarationKind::Def {
                    ty,
                    value,
                    reducibility,
                } = c.declarations[i].kind.clone()
                else {
                    panic!("source definition")
                };
                let value =
                    constant_body(&mut c, value, &format!("Mpk.Go.Practical.Word.W{w}.Zero"));
                c.declarations[i].kind = DeclarationKind::Def {
                    ty,
                    value,
                    reducibility,
                };
            } else if label.starts_with("FalseCorrection") {
                let i = global(
                    &c,
                    &format!("Mpk.Go.Practical.Word.SignedDivision.W{w}.SelectionCorrection"),
                );
                let DeclarationKind::Def {
                    ty,
                    value,
                    reducibility,
                } = c.declarations[i].kind.clone()
                else {
                    panic!("correction definition")
                };
                let value = correction_part(&mut c, value, label == "FalseCorrectionNegative");
                c.declarations[i].kind = DeclarationKind::Def {
                    ty,
                    value,
                    reducibility,
                };
            } else {
                let name = match label {
                    "FalseSelectionRefinement" => "SignedSelectionRefinement",
                    "FalseMagnitudeConservation" | "FakeMagnitudeGuard" => {
                        "MagnitudeDivisionConservation"
                    }
                    "FalseMagnitudeRemainderBound" => "MagnitudeRemainderBound",
                    "FalseQuotientRefinement" => "QuotientRefinement",
                    "FalseRemainderRefinement" => "RemainderRefinement",
                    "FalseMathematicalConservation" | "FakeMathematicalGuard" => {
                        "MathematicalDivisionConservation"
                    }
                    "FalseSourceConservation" | "FakeSourceGuard" => "SignedDivisionConservation",
                    _ => unreachable!(),
                };
                let i = global(
                    &c,
                    &format!("Mpk.Go.Practical.Word.SignedDivision.W{w}.{name}"),
                );
                let DeclarationKind::Theorem { ty, proof } = c.declarations[i].kind.clone() else {
                    panic!("actual theorem")
                };
                let ty = if label.starts_with("Fake") {
                    let domain = unrelated(&mut c);
                    replace_domain(&mut c, ty, 2, domain)
                } else if label == "FalseMagnitudeConservation" {
                    conclusion(&mut c, ty, "Std.Nat.zero")
                } else if label == "FalseMagnitudeRemainderBound" {
                    conclusion(&mut c, ty, "Std.Bool.false")
                } else {
                    false_relation(&mut c, ty)
                };
                c.declarations[i].kind = DeclarationKind::Theorem { ty, proof };
            }
            reject(&format!("{label}-{w}"), c);
        }
    }
}
