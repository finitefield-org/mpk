use mpk_cert::encode::{Certificate, Declaration, DeclarationKind, LevelNode, TermNode};
use mpk_vc::go_practical_ordinary::{
    generate_go_practical_execution_history_foundation,
    import_go_practical_execution_history_foundation, GoOrdinaryError,
};

const PREFIX: &str = "Mpk.Go.Practical.ExecutionHistory";

#[derive(Clone)]
enum E {
    C(String),
    V(&'static str),
    App(Box<E>, Vec<E>),
    Lam(&'static str, Box<E>, Box<E>),
    Pi(&'static str, Box<E>, Box<E>),
}
fn c(name: &str) -> E {
    E::C(name.into())
}
fn v(name: &'static str) -> E {
    E::V(name)
}
fn app(f: E, args: Vec<E>) -> E {
    E::App(Box::new(f), args)
}
fn call(name: &str, args: Vec<E>) -> E {
    app(c(name), args)
}
fn ec(name: &str, args: Vec<E>) -> E {
    call(&format!("{PREFIX}.{name}"), args)
}
fn lam(name: &'static str, ty: E, body: E) -> E {
    E::Lam(name, Box::new(ty), Box::new(body))
}
fn pi(name: &'static str, ty: E, body: E) -> E {
    E::Pi(name, Box::new(ty), Box::new(body))
}
fn boolean() -> E {
    c("Std.Bool")
}
fn nat() -> E {
    c("Std.Nat")
}
fn bit(yes: bool) -> E {
    c(if yes {
        "Std.Bool.true"
    } else {
        "Std.Bool.false"
    })
}
fn zero() -> E {
    c("Std.Nat.zero")
}
fn one() -> E {
    call("Std.Nat.succ", vec![zero()])
}
fn eq(ty: E, a: E, b: E) -> E {
    call("Std.Eq", vec![ty, a, b])
}
fn refl(ty: E, a: E) -> E {
    call("Std.Eq.refl", vec![ty, a])
}
fn pair(a: E, b: E) -> E {
    call("Mpk.Go.Practical.Product.Type", vec![a, b])
}
fn make(a: E, b: E, x: E, y: E) -> E {
    call("Mpk.Go.Practical.Product.Make", vec![a, b, x, y])
}

fn push(c: &mut Certificate, node: TermNode) -> u32 {
    let id = c.term_table.len() as u32;
    c.term_table.push(node);
    id
}
fn lower(cert: &mut Certificate, e: E, names: &mut Vec<&'static str>) -> u32 {
    match e {
        E::C(name) => {
            let global = cert
                .declarations
                .iter()
                .position(|d| cert.name_table[d.name as usize] == name)
                .unwrap() as u32;
            push(
                cert,
                TermNode::Const {
                    global,
                    levels: vec![],
                },
            )
        }
        E::V(name) => push(
            cert,
            TermNode::Var(names.iter().rev().position(|n| *n == name).unwrap() as u32),
        ),
        E::App(f, args) => {
            let function = lower(cert, *f, names);
            let arguments = args.into_iter().map(|a| lower(cert, a, names)).collect();
            push(
                cert,
                TermNode::App {
                    function,
                    arguments,
                },
            )
        }
        E::Lam(name, ty, body) => {
            let ty = lower(cert, *ty, names);
            names.push(name);
            let body = lower(cert, *body, names);
            names.pop();
            push(cert, TermNode::Lam { ty, body })
        }
        E::Pi(name, ty, body) => {
            let ty = lower(cert, *ty, names);
            names.push(name);
            let body = lower(cert, *body, names);
            names.pop();
            push(cert, TermNode::Pi { ty, body })
        }
    }
}
fn theorem(cert: &mut Certificate, name: &str, ty: E, proof: E) {
    let ty = lower(cert, ty, &mut vec![]);
    let proof = lower(cert, proof, &mut vec![]);
    let name_id = cert.name_table.len() as u32;
    cert.name_table.push(name.into());
    cert.declarations.push(Declaration {
        name: name_id,
        kind: DeclarationKind::Theorem { ty, proof },
    });
}
fn encode(mut c: Certificate) -> Vec<u8> {
    let old = c.name_table.clone();
    c.name_table.sort();
    for d in &mut c.declarations {
        d.name = c.name_table.binary_search(&old[d.name as usize]).unwrap() as u32;
    }
    for l in &mut c.level_table {
        if let LevelNode::Param(name) = l {
            *name = c.name_table.binary_search(&old[*name as usize]).unwrap() as u32;
        }
    }
    c.export_block = mpk_cert::build_export_block(&c).unwrap();
    c.axiom_report = mpk_cert::build_axiom_report(&c).unwrap();
    assert_eq!(c.axiom_report.summary.total_axiom_count, 0);
    c.hashes.export_hash = mpk_cert::export_block_hash(&c.export_block);
    c.hashes.axiom_report_hash = mpk_cert::axiom_report_hash_for_report(&c.axiom_report);
    assert!(c.declarations.len() <= 4096 && c.term_table.len() <= 131072);
    mpk_cert::encode_certificate_bounded(&c, 2_097_152).unwrap()
}
fn baseline() -> Certificate {
    mpk_cert::decode_canonical_certificate(
        generate_go_practical_execution_history_foundation()
            .unwrap()
            .certificate_bytes(),
    )
    .unwrap()
}
fn check(name: &str, bytes: &[u8], accepted: bool) {
    let result = mpk_kernel::verify_certificate_bytes_json_output(bytes);
    if let Some(path) = std::env::var_os("MPK_GO_EXECUTION_HISTORY_OUTPUT") {
        use std::io::Write;
        let root = std::path::PathBuf::from(path);
        std::fs::create_dir_all(&root).unwrap();
        for (extension, data) in [
            ("mpcert", bytes.to_vec()),
            ("rust.json", format!("{}\n", result.json).into_bytes()),
        ] {
            std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(root.join(format!("{name}.{extension}")))
                .unwrap()
                .write_all(&data)
                .unwrap();
        }
    }
    assert_eq!(result.accepted, accepted, "{name}: {}", result.json);
    if !accepted {
        let r: serde_json::Value = serde_json::from_str(&result.json).unwrap();
        assert_eq!(r["error_code"], "KERNEL_CORE_CHECK");
        let detail: serde_json::Value =
            serde_json::from_str(r["error_detail"].as_str().unwrap()).unwrap();
        assert_eq!(detail["code"], "CORE_TYPE_MISMATCH", "{name}");
    }
}

// A real invariant: identity transitions preserve the state being true.
// Its preservation proof uses both the actual edge equality and prior fact.
fn rel() -> E {
    lam(
        "x",
        boolean(),
        lam("y", boolean(), eq(boolean(), v("y"), v("x"))),
    )
}
fn good() -> E {
    lam("x", boolean(), eq(boolean(), v("x"), bit(true)))
}
fn preserve() -> E {
    lam(
        "x",
        boolean(),
        lam(
            "y",
            boolean(),
            lam(
                "edge",
                eq(boolean(), v("y"), v("x")),
                lam(
                    "valid",
                    eq(boolean(), v("x"), bit(true)),
                    call(
                        "Std.Eq.trans",
                        vec![boolean(), v("y"), v("x"), bit(true), v("edge"), v("valid")],
                    ),
                ),
            ),
        ),
    )
}
fn invariant() -> E {
    ec(
        "InvariantPrefixes",
        vec![
            boolean(),
            rel(),
            good(),
            bit(true),
            refl(boolean(), bit(true)),
            preserve(),
        ],
    )
}
fn partial_type() -> E {
    ec(
        "PartialCorrectness",
        vec![boolean(), rel(), bit(true), good(), good(), good()],
    )
}
fn partial() -> E {
    ec(
        "PartialFromInvariant",
        vec![
            boolean(),
            rel(),
            bit(true),
            good(),
            good(),
            good(),
            good(),
            refl(boolean(), bit(true)),
            preserve(),
            lam(
                "x",
                boolean(),
                lam("valid", eq(boolean(), v("x"), bit(true)), v("valid")),
            ),
            lam(
                "x",
                boolean(),
                lam(
                    "valid",
                    eq(boolean(), v("x"), bit(true)),
                    lam("normal", eq(boolean(), v("x"), bit(true)), v("normal")),
                ),
            ),
        ],
    )
}
fn constant_history(value: bool) -> E {
    lam("k", nat(), bit(value))
}
fn constant_path() -> E {
    lam(
        "k",
        nat(),
        lam(
            "below",
            eq(
                boolean(),
                call("Mpk.Go.Practical.Nat.Less", vec![v("k"), v("n")]),
                bit(true),
            ),
            refl(boolean(), bit(true)),
        ),
    )
}
fn run_type() -> E {
    ec(
        "NormalRun",
        vec![
            boolean(),
            rel(),
            bit(true),
            good(),
            constant_history(true),
            v("n"),
        ],
    )
}
fn run() -> E {
    let start = eq(boolean(), bit(true), bit(true));
    let path = ec(
        "Path",
        vec![boolean(), rel(), constant_history(true), v("n")],
    );
    let normal = start.clone();
    make(
        start.clone(),
        pair(path.clone(), normal.clone()),
        refl(boolean(), bit(true)),
        make(path, normal, constant_path(), refl(boolean(), bit(true))),
    )
}

#[test]
fn whole_ordinary_history_component_is_checked_and_exactly_imported() {
    let generated = generate_go_practical_execution_history_foundation().unwrap();
    assert_eq!(
        generated,
        generate_go_practical_execution_history_foundation().unwrap()
    );
    assert_eq!(
        import_go_practical_execution_history_foundation(generated.certificate_bytes()).unwrap(),
        generated
    );
    assert_eq!(
        import_go_practical_execution_history_foundation(&[0; 2_097_153]),
        Err(GoOrdinaryError::Limit)
    );
    check("foundation", generated.certificate_bytes(), true);
    let mut changed = baseline();
    theorem(
        &mut changed,
        "HistoryTest.UnrelatedTrue",
        eq(boolean(), bit(true), bit(true)),
        refl(boolean(), bit(true)),
    );
    let extra = encode(changed);
    check("extra-mathematics", &extra, true);
    assert_eq!(
        import_go_practical_execution_history_foundation(&extra),
        Err(GoOrdinaryError::Linkage)
    );
    let mut corrupt = generated.certificate_bytes().to_vec();
    let last = corrupt.len() - 1;
    corrupt[last] ^= 1;
    assert_eq!(
        import_go_practical_execution_history_foundation(&corrupt),
        Err(GoOrdinaryError::Linkage)
    );
}

#[test]
fn unbounded_invariant_partial_run_and_mapping_use_actual_edges() {
    let mut cert = baseline();
    theorem(
        &mut cert,
        "HistoryTest.AllFinitePrefixes",
        ec("SafePrefixes", vec![boolean(), rel(), bit(true), good()]),
        invariant(),
    );
    theorem(&mut cert, "HistoryTest.Partial", partial_type(), partial());
    theorem(
        &mut cert,
        "HistoryTest.EveryLengthNormalRun",
        pi("n", nat(), run_type()),
        lam("n", nat(), run()),
    );
    let result = eq(boolean(), bit(true), bit(true));
    theorem(
        &mut cert,
        "HistoryTest.ObserveSameRun",
        pi("n", nat(), pair(result.clone(), result)),
        lam(
            "n",
            nat(),
            ec(
                "ObserveNormalRun",
                vec![
                    boolean(),
                    rel(),
                    bit(true),
                    good(),
                    good(),
                    good(),
                    partial(),
                    constant_history(true),
                    v("n"),
                    run(),
                ],
            ),
        ),
    );
    theorem(
        &mut cert,
        "HistoryTest.Entry",
        eq(boolean(), bit(true), bit(true)),
        ec(
            "SafetyAtEntry",
            vec![boolean(), rel(), bit(true), good(), invariant()],
        ),
    );
    let prefixed = ec(
        "Prepend",
        vec![boolean(), bit(true), constant_history(true)],
    );
    theorem(
        &mut cert,
        "HistoryTest.PrependEveryLength",
        pi(
            "n",
            nat(),
            ec(
                "Path",
                vec![
                    boolean(),
                    rel(),
                    prefixed,
                    call("Std.Nat.succ", vec![v("n")]),
                ],
            ),
        ),
        lam(
            "n",
            nat(),
            ec(
                "PathPrepend",
                vec![
                    boolean(),
                    rel(),
                    constant_history(true),
                    v("n"),
                    bit(true),
                    refl(boolean(), bit(true)),
                    constant_path(),
                ],
            ),
        ),
    );
    let not = c("Mpk.Go.Practical.Bool.Not");
    let mapped = ec(
        "Map",
        vec![boolean(), boolean(), not.clone(), constant_history(true)],
    );
    let respects = lam(
        "x",
        boolean(),
        lam(
            "y",
            boolean(),
            lam(
                "edge",
                eq(boolean(), v("y"), v("x")),
                call(
                    "Std.Eq.congr",
                    vec![boolean(), boolean(), not.clone(), v("y"), v("x"), v("edge")],
                ),
            ),
        ),
    );
    theorem(
        &mut cert,
        "HistoryTest.MapEveryLength",
        pi(
            "n",
            nat(),
            ec("Path", vec![boolean(), rel(), mapped, v("n")]),
        ),
        lam(
            "n",
            nat(),
            ec(
                "MapPath",
                vec![
                    boolean(),
                    boolean(),
                    rel(),
                    rel(),
                    not,
                    respects,
                    constant_history(true),
                    v("n"),
                    constant_path(),
                ],
            ),
        ),
    );
    check("unbounded-models", &encode(cert), true);
}

#[test]
fn invalid_edges_initial_safety_and_missing_normal_completion_reject() {
    // Even the zero-step prefix includes the initial state. Returning true
    // later cannot establish that an initially false state was always safe.
    let mut initial = baseline();
    theorem(
        &mut initial,
        "HistoryTest.UnsafeEntry",
        eq(boolean(), bit(false), bit(true)),
        ec(
            "SafetyAtEntry",
            vec![boolean(), rel(), bit(false), good(), invariant()],
        ),
    );
    check("unsafe-entry", &encode(initial), false);

    // An actual invalid transition is not made reachable by a caller trace.
    let h = lam(
        "k",
        nat(),
        call(
            "Std.Nat.induct",
            vec![
                lam("index", nat(), boolean()),
                bit(true),
                lam(
                    "index",
                    nat(),
                    lam(
                        "prior",
                        boolean(),
                        call("Mpk.Go.Practical.Bool.Not", vec![v("prior")]),
                    ),
                ),
                v("k"),
            ],
        ),
    );
    let mut invalid_edge = baseline();
    let bogus_path = lam(
        "k",
        nat(),
        lam(
            "below",
            eq(
                boolean(),
                call("Mpk.Go.Practical.Nat.Less", vec![v("k"), one()]),
                bit(true),
            ),
            refl(boolean(), bit(true)),
        ),
    );
    theorem(
        &mut invalid_edge,
        "HistoryTest.InvalidEdge",
        ec("Path", vec![boolean(), rel(), h.clone(), one()]),
        bogus_path,
    );
    check("invalid-edge", &encode(invalid_edge), false);

    // This different relation really admits an unsafe intermediate state.
    // Complete valid edge witnesses still do not prove the state is true.
    let total = lam(
        "x",
        boolean(),
        lam("y", boolean(), eq(boolean(), bit(true), bit(true))),
    );
    let mut intermediate = baseline();
    let two = call("Std.Nat.succ", vec![one()]);
    let path_ty = ec("Path", vec![boolean(), total, h.clone(), two.clone()]);
    let path_proof = lam(
        "k",
        nat(),
        lam(
            "below",
            eq(
                boolean(),
                call("Mpk.Go.Practical.Nat.Less", vec![v("k"), two.clone()]),
                bit(true),
            ),
            refl(boolean(), bit(true)),
        ),
    );
    theorem(
        &mut intermediate,
        "HistoryTest.ValidUnsafePath",
        path_ty,
        path_proof,
    );
    let not = c("Mpk.Go.Practical.Bool.Not");
    let first = app(h.clone(), vec![one()]);
    let first_false = call(
        "Std.Eq.trans",
        vec![
            boolean(),
            first.clone(),
            app(not.clone(), vec![bit(true)]),
            bit(false),
            call(
                "Std.Eq.congr",
                vec![
                    boolean(),
                    boolean(),
                    not.clone(),
                    app(h.clone(), vec![zero()]),
                    bit(true),
                    refl(boolean(), bit(true)),
                ],
            ),
            refl(boolean(), bit(false)),
        ],
    );
    theorem(
        &mut intermediate,
        "HistoryTest.UnsafeMiddleIsFalse",
        eq(boolean(), first.clone(), bit(false)),
        first_false.clone(),
    );
    let endpoint_true = call(
        "Std.Eq.trans",
        vec![
            boolean(),
            app(h.clone(), vec![two.clone()]),
            app(not.clone(), vec![bit(false)]),
            bit(true),
            call(
                "Std.Eq.congr",
                vec![boolean(), boolean(), not, first, bit(false), first_false],
            ),
            refl(boolean(), bit(true)),
        ],
    );
    theorem(
        &mut intermediate,
        "HistoryTest.SafeEndpoint",
        eq(boolean(), app(h.clone(), vec![two]), bit(true)),
        endpoint_true,
    );
    check("unsafe-path-witness", &encode(intermediate.clone()), true);
    theorem(
        &mut intermediate,
        "HistoryTest.UnsafeIntermediate",
        app(good(), vec![app(h.clone(), vec![one()])]),
        refl(boolean(), bit(true)),
    );
    check("unsafe-intermediate", &encode(intermediate), false);

    // A normal run must include its own normal-completion witness; the
    // already checked partial-correctness theorem supplies no such witness.
    let mut normal = baseline();
    let trivial = lam("x", boolean(), eq(boolean(), bit(true), bit(true)));
    let trivial_proof = lam(
        "x",
        boolean(),
        lam("valid", eq(boolean(), bit(true), bit(true)), v("valid")),
    );
    let preserve_trivial = lam(
        "x",
        boolean(),
        lam(
            "y",
            boolean(),
            lam(
                "edge",
                eq(boolean(), v("y"), v("x")),
                lam("valid", eq(boolean(), bit(true), bit(true)), v("valid")),
            ),
        ),
    );
    let partial_ty = ec(
        "PartialCorrectness",
        vec![
            boolean(),
            rel(),
            bit(false),
            good(),
            good(),
            trivial.clone(),
        ],
    );
    let partial_proof = ec(
        "PartialFromInvariant",
        vec![
            boolean(),
            rel(),
            bit(false),
            trivial.clone(),
            trivial.clone(),
            good(),
            good(),
            refl(boolean(), bit(true)),
            preserve_trivial,
            trivial_proof,
            lam(
                "x",
                boolean(),
                lam(
                    "valid",
                    eq(boolean(), bit(true), bit(true)),
                    lam("normal", eq(boolean(), v("x"), bit(true)), v("normal")),
                ),
            ),
        ],
    );
    theorem(
        &mut normal,
        "HistoryTest.PartialWithoutNormalRun",
        partial_ty,
        partial_proof,
    );
    let bad = lam("x", boolean(), eq(boolean(), v("x"), bit(false)));
    let preserve_bad = lam(
        "x",
        boolean(),
        lam(
            "y",
            boolean(),
            lam(
                "edge",
                eq(boolean(), v("y"), v("x")),
                lam(
                    "valid",
                    eq(boolean(), v("x"), bit(false)),
                    call(
                        "Std.Eq.trans",
                        vec![boolean(), v("y"), v("x"), bit(false), v("edge"), v("valid")],
                    ),
                ),
            ),
        ),
    );
    let run_for_history = ec(
        "NormalRun",
        vec![boolean(), rel(), bit(false), good(), v("h"), v("n")],
    );
    let run_field = |name| {
        ec(
            name,
            vec![
                boolean(),
                rel(),
                bit(false),
                good(),
                v("h"),
                v("n"),
                v("run"),
            ],
        )
    };
    let end = app(v("h"), vec![v("n")]);
    let bad_at_end = app(
        ec(
            "InvariantPrefixes",
            vec![
                boolean(),
                rel(),
                bad,
                bit(false),
                refl(boolean(), bit(false)),
                preserve_bad,
            ],
        ),
        vec![v("n"), v("h"), run_field("RunStarts"), run_field("RunPath")],
    );
    let no_run = call(
        "Std.Eq.trans",
        vec![
            boolean(),
            bit(false),
            end.clone(),
            bit(true),
            call("Std.Eq.symm", vec![boolean(), end, bit(false), bad_at_end]),
            run_field("RunNormal"),
        ],
    );
    theorem(
        &mut normal,
        "HistoryTest.NoNormalRunAtAnyLength",
        pi(
            "n",
            nat(),
            pi(
                "h",
                pi("k", nat(), boolean()),
                pi(
                    "run",
                    run_for_history.clone(),
                    eq(boolean(), bit(false), bit(true)),
                ),
            ),
        ),
        lam(
            "n",
            nat(),
            lam(
                "h",
                pi("k", nat(), boolean()),
                lam("run", run_for_history, no_run),
            ),
        ),
    );
    check("partial-without-normal", &encode(normal.clone()), true);
    let h = constant_history(false);
    let start = eq(boolean(), bit(false), bit(false));
    let path_ty = ec("Path", vec![boolean(), rel(), h.clone(), zero()]);
    let normal_ty = eq(boolean(), bit(false), bit(true));
    theorem(
        &mut normal,
        "HistoryTest.MissingNormal",
        ec(
            "NormalRun",
            vec![boolean(), rel(), bit(false), good(), h.clone(), zero()],
        ),
        make(
            start,
            pair(path_ty.clone(), normal_ty.clone()),
            refl(boolean(), bit(false)),
            make(
                path_ty,
                normal_ty,
                ec("PathZero", vec![boolean(), rel(), h]),
                refl(boolean(), bit(false)),
            ),
        ),
    );
    check("missing-normal", &encode(normal), false);
}
