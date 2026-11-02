//! Ordinary finite histories over an arbitrary typed transition relation.
//!
//! Prefix validity checks every edge. Safety quantifies over all lengths and
//! histories; a terminating run additionally needs its normal-completion
//! witness. These library laws do not define Go steps, prove source/IR
//! correspondence, or authorize a caller-supplied transition as Go semantics.
use super::syntax::*;
use super::{natural_builder, product, Builder, GoOrdinaryError, Result, MAX_BYTES};

const PREFIX: &str = "Mpk.Go.Practical.ExecutionHistory";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoExecutionHistoryFoundation {
    certificate: Vec<u8>,
}
impl GoExecutionHistoryFoundation {
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
}

pub fn generate_go_practical_execution_history_foundation() -> Result<GoExecutionHistoryFoundation>
{
    let mut builder = natural_builder()?;
    product::append(&mut builder)?;
    Histories(&mut builder).append()?;
    builder.certificate.module = "Mpk.Go.Practical.ExecutionHistoryFoundation".into();
    Ok(GoExecutionHistoryFoundation {
        certificate: builder.finish()?,
    })
}

/// Regenerate all definitions and proofs. A valid unrelated theorem or a
/// rehashed candidate is not this exact foundation component.
pub fn import_go_practical_execution_history_foundation(
    candidate: &[u8],
) -> Result<GoExecutionHistoryFoundation> {
    if candidate.len() > MAX_BYTES {
        return Err(GoOrdinaryError::Limit);
    }
    let expected = generate_go_practical_execution_history_foundation()?;
    if candidate != expected.certificate_bytes() {
        return Err(GoOrdinaryError::Linkage);
    }
    Ok(expected)
}

fn ec(name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("{PREFIX}.{name}"), args)
}
fn pc(name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("Mpk.Go.Practical.Product.{name}"), args)
}
fn history(a: Expr) -> Expr {
    pi("_index", nat(), a)
}
fn predicate(a: Expr) -> Expr {
    pi("_state", a, Expr::Sort0)
}
fn relation(a: Expr) -> Expr {
    pi("_from", a.clone(), pi("_to", a, Expr::Sort0))
}
fn at(h: Expr, n: Expr) -> Expr {
    app(h, vec![n])
}
fn holds(p: Expr, x: Expr) -> Expr {
    app(p, vec![x])
}
fn edge(r: Expr, x: Expr, y: Expr) -> Expr {
    app(r, vec![x, y])
}
fn below(k: Expr, n: Expr) -> Expr {
    eq(boolean(), less(k, n), bit(true))
}
fn starts(a: Expr, h: Expr, initial: Expr) -> Expr {
    eq(a, at(h, zero()), initial)
}
// Keep function types explicit at neutral proof binders. The checked aliases
// below are definitionally equal; none is an opaque reachability predicate.
fn path(r: Expr, h: Expr, n: Expr) -> Expr {
    pi(
        "_k",
        nat(),
        pi(
            "_below",
            below(var("_k"), n),
            edge(r, at(h.clone(), var("_k")), at(h, succ(var("_k")))),
        ),
    )
}
fn safe(a: Expr, r: Expr, initial: Expr, p: Expr) -> Expr {
    pi(
        "n",
        nat(),
        pi(
            "h",
            history(a.clone()),
            pi(
                "start",
                starts(a, var("h"), initial),
                pi(
                    "path",
                    path(r, var("h"), var("n")),
                    holds(p, at(var("h"), var("n"))),
                ),
            ),
        ),
    )
}
fn normal_post(a: Expr, r: Expr, initial: Expr, normal: Expr, post: Expr) -> Expr {
    pi(
        "n",
        nat(),
        pi(
            "h",
            history(a.clone()),
            pi(
                "start",
                starts(a, var("h"), initial),
                pi(
                    "path",
                    path(r, var("h"), var("n")),
                    pi(
                        "normal",
                        holds(normal, at(var("h"), var("n"))),
                        holds(post, at(var("h"), var("n"))),
                    ),
                ),
            ),
        ),
    )
}
fn pair(a: Expr, b: Expr) -> Expr {
    pc("Type", vec![a, b])
}
fn make(a: Expr, b: Expr, x: Expr, y: Expr) -> Expr {
    pc("Make", vec![a, b, x, y])
}
fn left(a: Expr, b: Expr, p: Expr) -> Expr {
    pc("Left", vec![a, b, p])
}
fn right(a: Expr, b: Expr, p: Expr) -> Expr {
    pc("Right", vec![a, b, p])
}
fn normal_run(a: Expr, r: Expr, initial: Expr, normal: Expr, h: Expr, n: Expr) -> Expr {
    pair(
        starts(a, h.clone(), initial),
        pair(path(r, h.clone(), n.clone()), holds(normal, at(h, n))),
    )
}
fn tail(a: Expr, h: Expr) -> Expr {
    ec("Tail", vec![a, h])
}
fn prepend(a: Expr, initial: Expr, h: Expr) -> Expr {
    ec("Prepend", vec![a, initial, h])
}
fn lambdas(args: &[Binder], mut body: Expr) -> Expr {
    for (name, ty) in args.iter().rev() {
        body = lam(name, ty.clone(), body);
    }
    body
}
fn basics() -> Vec<Binder> {
    vec![("A", Expr::Sort0), ("R", relation(var("A")))]
}
fn preserves(a: Expr, r: Expr, p: Expr) -> Expr {
    pi(
        "x",
        a.clone(),
        pi(
            "y",
            a,
            pi(
                "edge",
                edge(r, var("x"), var("y")),
                pi("valid", holds(p.clone(), var("x")), holds(p, var("y"))),
            ),
        ),
    )
}

struct Histories<'a>(&'a mut Builder);
impl Histories<'_> {
    fn emit(&mut self, name: &str, args: &[Binder], ty: Expr, body: Expr, law: bool) -> Result<()> {
        emit(self.0, &format!("{PREFIX}.{name}"), args, ty, body, law)
    }
    fn definitions(&mut self) -> Result<()> {
        let (a, r, h, n, p) = (var("A"), var("R"), var("h"), var("n"), var("P"));
        self.emit(
            "History",
            &[("A", Expr::Sort0)],
            Expr::Sort0,
            history(a.clone()),
            false,
        )?;
        let mut args = basics();
        args.extend([("h", history(a.clone())), ("n", nat())]);
        self.emit(
            "Path",
            &args,
            Expr::Sort0,
            path(r.clone(), h.clone(), n.clone()),
            false,
        )?;
        self.emit(
            "Starts",
            &[
                ("A", Expr::Sort0),
                ("h", history(a.clone())),
                ("initial", a.clone()),
            ],
            Expr::Sort0,
            starts(a.clone(), h.clone(), var("initial")),
            false,
        )?;
        let mut args = basics();
        args.extend([("initial", a.clone()), ("P", predicate(a.clone()))]);
        self.emit(
            "SafePrefixes",
            &args,
            Expr::Sort0,
            safe(a.clone(), r.clone(), var("initial"), p.clone()),
            false,
        )?;
        let mut args = basics();
        args.extend([
            ("initial", a.clone()),
            ("Normal", predicate(a.clone())),
            ("Post", predicate(a.clone())),
        ]);
        let post = normal_post(
            a.clone(),
            r.clone(),
            var("initial"),
            var("Normal"),
            var("Post"),
        );
        self.emit("NormalPost", &args, Expr::Sort0, post.clone(), false)?;
        args.push(("Safe", predicate(a.clone())));
        self.emit(
            "PartialCorrectness",
            &args,
            Expr::Sort0,
            pair(
                safe(a.clone(), r.clone(), var("initial"), var("Safe")),
                post,
            ),
            false,
        )?;
        args.pop();
        args.pop(); // NormalRun has a normal predicate, not a postcondition.
        args.extend([("h", history(a.clone())), ("n", nat())]);
        self.emit(
            "NormalRun",
            &args,
            Expr::Sort0,
            normal_run(a.clone(), r, var("initial"), var("Normal"), h.clone(), n),
            false,
        )?;
        self.emit(
            "Tail",
            &[("A", Expr::Sort0), ("h", history(a.clone()))],
            history(a.clone()),
            lam("_k", nat(), at(h.clone(), succ(var("_k")))),
            false,
        )?;
        self.emit(
            "Prepend",
            &[
                ("A", Expr::Sort0),
                ("initial", a.clone()),
                ("h", history(a.clone())),
            ],
            history(a.clone()),
            lam(
                "_k",
                nat(),
                induct(
                    "_position",
                    a.clone(),
                    var("initial"),
                    lam(
                        "_position",
                        nat(),
                        lam("_ignored", a.clone(), at(h.clone(), var("_position"))),
                    ),
                    var("_k"),
                ),
            ),
            false,
        )?;
        self.emit(
            "Map",
            &[
                ("A", Expr::Sort0),
                ("B", Expr::Sort0),
                ("f", pi("_x", a.clone(), var("B"))),
                ("h", history(a)),
            ],
            history(var("B")),
            lam("_k", nat(), app(var("f"), vec![at(h, var("_k"))])),
            false,
        )
    }
    fn path_laws(&mut self) -> Result<()> {
        let (a, r, h, n) = (var("A"), var("R"), var("h"), var("n"));
        let mut args = basics();
        args.push(("h", history(a.clone())));
        let less_zero = Equality::boolean(
            less(var("_k"), zero()),
            bit(false),
            nc("LessZero", vec![var("_k")]),
        )
        .reverse()
        .then(Equality::boolean(
            less(var("_k"), zero()),
            bit(true),
            var("_below"),
        ))?;
        self.emit(
            "PathZero",
            &args,
            path(r.clone(), h.clone(), zero()),
            lam(
                "_k",
                nat(),
                lam(
                    "_below",
                    below(var("_k"), zero()),
                    call(
                        "Mpk.Go.Practical.Bool.FalseElim",
                        vec![
                            edge(
                                r.clone(),
                                at(h.clone(), var("_k")),
                                at(h.clone(), succ(var("_k"))),
                            ),
                            less_zero.proof,
                        ],
                    ),
                ),
            ),
            true,
        )?;
        args.extend([
            ("n", nat()),
            ("path", path(r.clone(), h.clone(), succ(n.clone()))),
        ]);
        self.emit(
            "PathHead",
            &args,
            edge(r.clone(), at(h.clone(), zero()), at(h.clone(), one())),
            app(
                var("path"),
                vec![zero(), nc("LessZeroSucc", vec![n.clone()])],
            ),
            true,
        )?;
        let lifted = Equality::boolean(
            less(succ(var("_k")), succ(n.clone())),
            less(var("_k"), n.clone()),
            nc("LessSuccBoth", vec![var("_k"), n.clone()]),
        )
        .then(Equality::boolean(
            less(var("_k"), n.clone()),
            bit(true),
            var("_below"),
        ))?;
        self.emit(
            "PathTail",
            &args,
            path(r.clone(), tail(a.clone(), h.clone()), n.clone()),
            lam(
                "_k",
                nat(),
                lam(
                    "_below",
                    below(var("_k"), n.clone()),
                    app(var("path"), vec![succ(var("_k")), lifted.proof]),
                ),
            ),
            true,
        )?;
        args.pop();
        args.extend([
            ("initial", a.clone()),
            (
                "first",
                edge(r.clone(), var("initial"), at(h.clone(), zero())),
            ),
            ("path", path(r.clone(), h.clone(), n.clone())),
        ]);
        let joined = prepend(a.clone(), var("initial"), h.clone());
        let target = |k: Expr| {
            pi(
                "_below",
                below(k.clone(), succ(n.clone())),
                edge(
                    r.clone(),
                    at(joined.clone(), k.clone()),
                    at(joined.clone(), succ(k)),
                ),
            )
        };
        let reduced = Equality::boolean(
            less(succ(var("_k")), succ(n.clone())),
            less(var("_k"), n.clone()),
            nc("LessSuccBoth", vec![var("_k"), n.clone()]),
        )
        .reverse()
        .then(Equality::boolean(
            less(succ(var("_k")), succ(n.clone())),
            bit(true),
            var("_below"),
        ))?;
        self.emit(
            "PathPrepend",
            &args,
            path(r.clone(), joined.clone(), succ(n.clone())),
            lam(
                "_index",
                nat(),
                induct(
                    "_k",
                    target(var("_k")),
                    lam("_below", below(zero(), succ(n.clone())), var("first")),
                    lam(
                        "_k",
                        nat(),
                        lam(
                            "_ignored",
                            target(var("_k")),
                            lam(
                                "_below",
                                below(succ(var("_k")), succ(n)),
                                app(var("path"), vec![var("_k"), reduced.proof]),
                            ),
                        ),
                    ),
                    var("_index"),
                ),
            ),
            true,
        )
    }
    fn invariant_laws(&mut self) -> Result<()> {
        let (a, r, p, h, n) = (var("A"), var("R"), var("P"), var("h"), var("n"));
        let mut args = basics();
        args.extend([
            ("P", predicate(a.clone())),
            ("preserve", preserves(a.clone(), r.clone(), p.clone())),
            ("n", nat()),
        ]);
        let trace_args = |n: Expr| {
            vec![
                ("h", history(a.clone())),
                ("initial_valid", holds(p.clone(), at(h.clone(), zero()))),
                ("path", path(r.clone(), h.clone(), n)),
            ]
        };
        let property = |n: Expr| {
            let mut ty = holds(p.clone(), at(h.clone(), n.clone()));
            for (name, domain) in trace_args(n).into_iter().rev() {
                ty = pi(name, domain, ty);
            }
            ty
        };
        let next_valid = app(
            var("preserve"),
            vec![
                at(h.clone(), zero()),
                at(h.clone(), one()),
                ec(
                    "PathHead",
                    vec![a.clone(), r.clone(), h.clone(), var("_n"), var("path")],
                ),
                var("initial_valid"),
            ],
        );
        self.emit(
            "InvariantHistory",
            &args,
            property(n.clone()),
            induct(
                "_n",
                property(var("_n")),
                lambdas(&trace_args(zero()), var("initial_valid")),
                lam(
                    "_n",
                    nat(),
                    lam(
                        "_prior",
                        property(var("_n")),
                        lambdas(
                            &trace_args(succ(var("_n"))),
                            app(
                                var("_prior"),
                                vec![
                                    tail(a.clone(), h.clone()),
                                    next_valid,
                                    ec(
                                        "PathTail",
                                        vec![
                                            a.clone(),
                                            r.clone(),
                                            h.clone(),
                                            var("_n"),
                                            var("path"),
                                        ],
                                    ),
                                ],
                            ),
                        ),
                    ),
                ),
                n,
            ),
            true,
        )?;
        let mut args = basics();
        args.extend([
            ("P", predicate(a.clone())),
            ("initial", a.clone()),
            ("initial_valid", holds(p.clone(), var("initial"))),
            ("preserve", preserves(a.clone(), r.clone(), p.clone())),
        ]);
        let start = starts(a.clone(), h.clone(), var("initial"));
        let initial_valid = call(
            "Std.Eq.rewrite",
            vec![
                a.clone(),
                var("initial"),
                at(h.clone(), zero()),
                p.clone(),
                call(
                    "Std.Eq.symm",
                    vec![
                        a.clone(),
                        at(h.clone(), zero()),
                        var("initial"),
                        var("start"),
                    ],
                ),
                var("initial_valid"),
            ],
        );
        self.emit(
            "InvariantPrefixes",
            &args,
            safe(a.clone(), r.clone(), var("initial"), p.clone()),
            lambdas(
                &[
                    ("n", nat()),
                    ("h", history(a.clone())),
                    ("start", start),
                    ("path", path(r.clone(), h.clone(), var("n"))),
                ],
                ec(
                    "InvariantHistory",
                    vec![
                        a.clone(),
                        r.clone(),
                        p.clone(),
                        var("preserve"),
                        var("n"),
                        h.clone(),
                        initial_valid,
                        var("path"),
                    ],
                ),
            ),
            true,
        )?;
        let mut args = basics();
        args.extend([
            ("initial", a.clone()),
            ("P", predicate(a.clone())),
            ("Q", predicate(a.clone())),
            (
                "safe",
                safe(a.clone(), r.clone(), var("initial"), p.clone()),
            ),
            (
                "implies",
                pi(
                    "x",
                    a.clone(),
                    pi(
                        "proof",
                        holds(p.clone(), var("x")),
                        holds(var("Q"), var("x")),
                    ),
                ),
            ),
        ]);
        self.emit(
            "WeakenSafety",
            &args,
            safe(a.clone(), r.clone(), var("initial"), var("Q")),
            lambdas(
                &[
                    ("n", nat()),
                    ("h", history(a.clone())),
                    ("start", starts(a.clone(), h.clone(), var("initial"))),
                    ("path", path(r.clone(), h.clone(), var("n"))),
                ],
                app(
                    var("implies"),
                    vec![
                        at(h.clone(), var("n")),
                        app(var("safe"), vec![var("n"), h, var("start"), var("path")]),
                    ],
                ),
            ),
            true,
        )
    }
    fn mapping_laws(&mut self) -> Result<()> {
        let (a, b, r, s, h, f, n) = (
            var("A"),
            var("B"),
            var("R"),
            var("S"),
            var("h"),
            var("f"),
            var("n"),
        );
        let mapped = ec("Map", vec![a.clone(), b.clone(), f.clone(), h.clone()]);
        let mut args = vec![
            ("A", Expr::Sort0),
            ("B", Expr::Sort0),
            ("R", relation(a.clone())),
            ("S", relation(b.clone())),
            ("f", pi("x", a.clone(), b.clone())),
            (
                "respects",
                pi(
                    "x",
                    a.clone(),
                    pi(
                        "y",
                        a.clone(),
                        pi(
                            "edge",
                            edge(r.clone(), var("x"), var("y")),
                            edge(
                                s.clone(),
                                app(f.clone(), vec![var("x")]),
                                app(f.clone(), vec![var("y")]),
                            ),
                        ),
                    ),
                ),
            ),
        ];
        args.extend([
            ("h", history(a.clone())),
            ("n", nat()),
            ("path", path(r.clone(), h.clone(), n.clone())),
        ]);
        self.emit(
            "MapPath",
            &args,
            path(s, mapped.clone(), n.clone()),
            lam(
                "_k",
                nat(),
                lam(
                    "_below",
                    below(var("_k"), n),
                    app(
                        var("respects"),
                        vec![
                            at(h.clone(), var("_k")),
                            at(h.clone(), succ(var("_k"))),
                            app(var("path"), vec![var("_k"), var("_below")]),
                        ],
                    ),
                ),
            ),
            true,
        )?;
        self.emit(
            "MapStarts",
            &[
                ("A", Expr::Sort0),
                ("B", Expr::Sort0),
                ("f", pi("x", a.clone(), b.clone())),
                ("h", history(a.clone())),
                ("initial", a.clone()),
                ("start", starts(a.clone(), h.clone(), var("initial"))),
            ],
            starts(b.clone(), mapped, app(f.clone(), vec![var("initial")])),
            call(
                "Std.Eq.congr",
                vec![a, b, f, at(h, zero()), var("initial"), var("start")],
            ),
            true,
        )
    }
    fn completion_laws(&mut self) -> Result<()> {
        let (a, r, h, n) = (var("A"), var("R"), var("h"), var("n"));
        let mut args = basics();
        args.extend([
            ("initial", a.clone()),
            ("Normal", predicate(a.clone())),
            ("h", history(a.clone())),
            ("n", nat()),
        ]);
        let start = starts(a.clone(), h.clone(), var("initial"));
        let path_ty = path(r.clone(), h.clone(), n.clone());
        let normal = holds(var("Normal"), at(h.clone(), n.clone()));
        let rest = pair(path_ty.clone(), normal.clone());
        args.push(("run", pair(start.clone(), rest.clone())));
        let start_proof = left(start.clone(), rest.clone(), var("run"));
        let rest_proof = right(start.clone(), rest.clone(), var("run"));
        self.emit("RunStarts", &args, start, start_proof, true)?;
        self.emit(
            "RunPath",
            &args,
            path_ty.clone(),
            left(path_ty.clone(), normal.clone(), rest_proof.clone()),
            true,
        )?;
        self.emit(
            "RunNormal",
            &args,
            normal.clone(),
            right(path_ty, normal, rest_proof),
            true,
        )?;

        let mut args = basics();
        args.extend([
            ("initial", a.clone()),
            ("Safe", predicate(a.clone())),
            ("Normal", predicate(a.clone())),
            ("Post", predicate(a.clone())),
        ]);
        let safety = safe(a.clone(), r.clone(), var("initial"), var("Safe"));
        let post = normal_post(
            a.clone(),
            r.clone(),
            var("initial"),
            var("Normal"),
            var("Post"),
        );
        args.push(("partial", pair(safety.clone(), post.clone())));
        self.emit(
            "PartialSafety",
            &args,
            safety.clone(),
            left(safety.clone(), post.clone(), var("partial")),
            true,
        )?;
        self.emit(
            "PartialPost",
            &args,
            post.clone(),
            right(safety.clone(), post.clone(), var("partial")),
            true,
        )?;
        args.extend([
            ("h", history(a.clone())),
            ("n", nat()),
            (
                "run",
                normal_run(
                    a.clone(),
                    r.clone(),
                    var("initial"),
                    var("Normal"),
                    h.clone(),
                    n.clone(),
                ),
            ),
        ]);
        let run_args = |name: &str| {
            ec(
                name,
                vec![
                    a.clone(),
                    r.clone(),
                    var("initial"),
                    var("Normal"),
                    h.clone(),
                    n.clone(),
                    var("run"),
                ],
            )
        };
        let mut execution = vec![
            n.clone(),
            h.clone(),
            run_args("RunStarts"),
            run_args("RunPath"),
        ];
        let safety_result = app(
            left(safety.clone(), post.clone(), var("partial")),
            execution.clone(),
        );
        execution.push(run_args("RunNormal"));
        self.emit(
            "ObserveNormalRun",
            &args,
            pair(
                holds(var("Safe"), at(h.clone(), n.clone())),
                holds(var("Post"), at(h.clone(), n.clone())),
            ),
            make(
                holds(var("Safe"), at(h.clone(), n.clone())),
                holds(var("Post"), at(h, n)),
                safety_result,
                app(right(safety, post, var("partial")), execution),
            ),
            true,
        )
    }
    fn partial_from_invariant(&mut self) -> Result<()> {
        let (a, r, inv, h, n) = (var("A"), var("R"), var("Inv"), var("h"), var("n"));
        let mut args = basics();
        args.extend([
            ("initial", a.clone()),
            ("Inv", predicate(a.clone())),
            ("Safe", predicate(a.clone())),
            ("Normal", predicate(a.clone())),
            ("Post", predicate(a.clone())),
            ("initial_valid", holds(inv.clone(), var("initial"))),
            ("preserve", preserves(a.clone(), r.clone(), inv.clone())),
            (
                "safe_implies",
                pi(
                    "x",
                    a.clone(),
                    pi(
                        "valid",
                        holds(inv.clone(), var("x")),
                        holds(var("Safe"), var("x")),
                    ),
                ),
            ),
            (
                "post_implies",
                pi(
                    "x",
                    a.clone(),
                    pi(
                        "valid",
                        holds(inv.clone(), var("x")),
                        pi(
                            "normal",
                            holds(var("Normal"), var("x")),
                            holds(var("Post"), var("x")),
                        ),
                    ),
                ),
            ),
        ]);
        let inv_prefixes = ec(
            "InvariantPrefixes",
            vec![
                a.clone(),
                r.clone(),
                inv.clone(),
                var("initial"),
                var("initial_valid"),
                var("preserve"),
            ],
        );
        let safety = safe(a.clone(), r.clone(), var("initial"), var("Safe"));
        let post = normal_post(
            a.clone(),
            r.clone(),
            var("initial"),
            var("Normal"),
            var("Post"),
        );
        let post_proof = lambdas(
            &[
                ("n", nat()),
                ("h", history(a.clone())),
                ("start", starts(a.clone(), h.clone(), var("initial"))),
                ("path", path(r.clone(), h.clone(), n.clone())),
                ("normal", holds(var("Normal"), at(h.clone(), n.clone()))),
            ],
            app(
                var("post_implies"),
                vec![
                    at(h.clone(), n),
                    app(
                        inv_prefixes.clone(),
                        vec![var("n"), h, var("start"), var("path")],
                    ),
                    var("normal"),
                ],
            ),
        );
        self.emit(
            "PartialFromInvariant",
            &args,
            pair(safety.clone(), post.clone()),
            make(
                safety,
                post,
                ec(
                    "WeakenSafety",
                    vec![
                        a,
                        r,
                        var("initial"),
                        inv,
                        var("Safe"),
                        inv_prefixes,
                        var("safe_implies"),
                    ],
                ),
                post_proof,
            ),
            true,
        )
    }
    fn safety_at_entry(&mut self) -> Result<()> {
        let (a, r, p, initial) = (var("A"), var("R"), var("P"), var("initial"));
        let mut args = basics();
        args.extend([
            ("initial", a.clone()),
            ("P", predicate(a.clone())),
            (
                "safe",
                safe(a.clone(), r.clone(), initial.clone(), p.clone()),
            ),
        ]);
        let h = lam("_k", nat(), initial.clone());
        self.emit(
            "SafetyAtEntry",
            &args,
            holds(p, initial.clone()),
            app(
                var("safe"),
                vec![
                    zero(),
                    h.clone(),
                    refl(a.clone(), initial),
                    ec("PathZero", vec![a, r, h]),
                ],
            ),
            true,
        )
    }
    fn append(&mut self) -> Result<()> {
        self.definitions()?;
        self.path_laws()?;
        self.invariant_laws()?;
        self.mapping_laws()?;
        self.completion_laws()?;
        self.partial_from_invariant()?;
        self.safety_at_entry()
    }
}
