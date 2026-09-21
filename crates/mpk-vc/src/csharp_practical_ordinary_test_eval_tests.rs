use super::test_eval::*;
use super::Builder;
use mpk_cert::{decode_canonical_certificate, encode::TermNode};
use std::rc::Rc;

#[test]
fn ordinary_core_dense_normalization_preserves_addresses_and_demand() {
    let mut b = Builder::new().unwrap();
    let cube = b.cube(10).unwrap();
    let x = b.var(0).unwrap();
    let body = b.lam(cube, x).unwrap();
    let ty = b.pi(cube, cube).unwrap();
    b.define("Test.DenseIdentity", ty, body).unwrap();
    let c = decode_canonical_certificate(&b.finish().unwrap()).unwrap();
    for count in [0, 1, 16, 17, 1024] {
        let mut bits = vec![false; 1024];
        for index in 0..count {
            bits[1023 - index] = true;
        }
        let normalized = dense_cube(bits.clone());
        assert_eq!(matches!(normalized, V::SharedCube(..)), count > 16);
        let passed = run(&c, "Test.DenseIdentity", vec![V::Cube(bits.clone())]);
        let dense = V::SharedCube(Rc::new(bits.clone()), 0, 1);
        for (address, expected) in bits.iter().enumerate() {
            for mut value in [normalized.clone(), passed.clone(), dense.clone()] {
                for selector in 0..10 {
                    value = apply(&c, value, V::Bit(address & (1 << selector) != 0));
                }
                assert_eq!(bit(value), *expected, "count {count}, address {address}");
            }
        }
        // Also exercise normalization when a raw table is applied directly.
        let mut value = V::Cube(bits);
        for _ in 0..10 {
            value = apply(&c, value, V::Bit(true));
        }
        assert_eq!(bit(value), count != 0);
    }
    for length in [0, 1, 512, 1023, 1025] {
        assert!(matches!(dense_cube(vec![false; length]), V::SharedCube(..)));
    }
    let demanded = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        apply(
            &c,
            V::Cube(vec![false; 1024]),
            thunk(u32::MAX, Rc::new(Env::Empty)),
        )
    }));
    assert!(
        demanded.is_err(),
        "normalization skipped a demanded selector"
    );
}

#[test]
fn ordinary_core_scalar_observations_survive_released_selector_closures() {
    for depth in [5, 14, 18] {
        let mut b = Builder::new().unwrap();
        let rec = b.constant("Std.Bool.rec").unwrap();
        let not = b.constant("Std.Bool.not").unwrap();
        let mut body = b.var(0).unwrap();
        for i in 1..depth {
            let input = b.var(i).unwrap();
            let inverse = b.app(not, vec![body]).unwrap();
            body = b.app(rec, vec![body, inverse, input]).unwrap();
        }
        for _ in 0..512 {
            body = b.app(not, vec![body]).unwrap();
        }
        let body = b.wrap_selectors(depth, body).unwrap();
        let ty = b.cube(depth).unwrap();
        b.define("Test.ObservedWord", ty, body).unwrap();
        let c = decode_canonical_certificate(&b.finish().unwrap()).unwrap();
        let root = run(&c, "Test.ObservedWord", vec![]);
        let mut totals = [0u64; 2];
        for total in &mut totals {
            for address in 0u32..32 {
                let address = address | if depth > 5 { 1 << (depth - 1) } else { 0 };
                let mut value = root.clone();
                let mut first_memo = None;
                for selector in 0..depth {
                    let (next, steps) =
                        apply_counted(&c, value, V::Bit(address & (1 << selector) != 0));
                    *total += steps;
                    if selector == 0 {
                        let V::Lambda(_, _, memo) = &next else {
                            panic!()
                        };
                        first_memo = Some(Rc::downgrade(memo));
                    }
                    value = next;
                }
                assert_eq!(bit(value), address.count_ones() % 2 != 0);
                assert!(
                    first_memo.unwrap().upgrade().is_none(),
                    "scalar cache retained a selector closure"
                );
            }
        }
        assert!(
            totals[1] < totals[0] / 4,
            "cold/warm transitions: {totals:?}"
        );
    }
}

#[test]
fn ordinary_core_sparse_uniform_views_preserve_every_address() {
    let c = decode_canonical_certificate(&Builder::new().unwrap().finish().unwrap()).unwrap();
    for depth in 0..=7 {
        let length = 1usize << depth;
        for ones in [
            std::collections::BTreeSet::new(),
            [0].into_iter().collect(),
            [length - 1].into_iter().collect(),
            (0..length).filter(|n| n % 3 == 0).collect(),
        ] {
            let sparse = sparse_cube(depth, ones.clone());
            for index in 0..length {
                let mut value = sparse.clone();
                for selector in 0..depth {
                    value = apply(&c, value, V::Bit(index & (1 << selector) != 0));
                }
                assert_eq!(
                    bit(value),
                    ones.contains(&index),
                    "C{depth} address {index}"
                );
            }
        }
        for value in [false, true] {
            if depth == 0 {
                continue;
            }
            let mut cube = V::UniformCube(value, depth);
            for selector in 0..depth {
                cube = apply(&c, cube, V::Bit(selector % 2 == 0));
            }
            assert_eq!(bit(cube), value);
        }
    }
    // Empty views with different physical offsets/backings must have the same
    // exact constant representation, while a last-address mutation survives.
    let input = sparse_cube(33, [1usize, (1usize << 33) - 1].into_iter().collect());
    let empty = apply(&c, input.clone(), V::Bit(false));
    assert!(matches!(empty, V::UniformCube(false, 32)));
    let mut last = input;
    for _ in 0..33 {
        last = apply(&c, last, V::Bit(true));
    }
    assert!(bit(last));
}

#[test]
fn ordinary_core_demanded_uniform_arguments_reuse_complete_results() {
    let mut b = Builder::new().unwrap();
    let no = b.constant("Std.Bool.false").unwrap();
    let not = b.constant("Std.Bool.not").unwrap();
    let or = b.constant("Std.Bool.or").unwrap();
    let x = b.var(0).unwrap();
    let head = b.app(x, vec![no; 8]).unwrap();
    let mut expensive = no;
    for _ in 0..512 {
        expensive = b.app(not, vec![expensive]).unwrap();
    }
    let body = b.app(or, vec![head, expensive]).unwrap();
    let input = b.cube(8).unwrap();
    let body = b.lam(input, body).unwrap();
    let ty = b.pi(input, b.boolean).unwrap();
    b.define("Test.DemandThenWork", ty, body).unwrap();
    // A different suspension/backing per call projects the even half of C9.
    let x = b.var(0).unwrap();
    let project = b.app(x, vec![no]).unwrap();
    let c = decode_canonical_certificate(&b.finish().unwrap()).unwrap();
    let f = run(&c, "Test.DemandThenWork", vec![]);
    let (first, cold) = apply_counted(&c, f.clone(), sparse_cube(8, Default::default()));
    assert!(!bit(first));
    for _ in 0..8 {
        let backing = Rc::new([1usize, 511].into_iter().collect());
        let weak = Rc::downgrade(&backing);
        let argument = thunk(
            project,
            Rc::new(Env::Bind(
                V::SparseCube(backing, 0, 1, 512),
                Rc::new(Env::Empty).into(),
            )),
        );
        let (same, warm) = apply_counted(&c, f.clone(), argument);
        assert!(!bit(same));
        assert!(
            warm < cold / 8,
            "demanded argument recomputed: cold={cold}, warm={warm}"
        );
        assert!(
            weak.upgrade().is_none(),
            "empty view retained its old backing"
        );
    }
    // A changed demanded bit must miss the constant-cube memo and return true.
    let changed = thunk(
        project,
        Rc::new(Env::Bind(
            sparse_cube(9, [0usize].into_iter().collect()),
            Rc::new(Env::Empty).into(),
        )),
    );
    assert!(bit(apply(&c, f, changed)));
}

#[test]
fn ordinary_core_uniform_views_do_not_force_unused_arguments() {
    let mut b = Builder::new().unwrap();
    let no = b.constant("Std.Bool.false").unwrap();
    let ty = b.cube(8).unwrap();
    let body = b.lam(ty, no).unwrap();
    let ty = b.pi(ty, b.boolean).unwrap();
    b.define("Test.IgnoreUniform", ty, body).unwrap();
    let c = decode_canonical_certificate(&b.finish().unwrap()).unwrap();
    let f = run(&c, "Test.IgnoreUniform", vec![]);
    assert!(!bit(apply(
        &c,
        f.clone(),
        sparse_cube(8, Default::default())
    )));
    assert!(!bit(apply(&c, f, thunk(u32::MAX, Rc::new(Env::Empty)))));
    // A demanded selector retains the original strict selector behavior.
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        apply(
            &c,
            V::UniformCube(false, 8),
            thunk(u32::MAX, Rc::new(Env::Empty)),
        )
    }));
    assert!(result.is_err());
}

#[test]
fn ordinary_core_closed_function_memos_survive_value_cache_eviction() {
    let mut b = Builder::new().unwrap();
    let no = b.constant("Std.Bool.false").unwrap();
    let yes = b.constant("Std.Bool.true").unwrap();
    let not = b.constant("Std.Bool.not").unwrap();
    let and = b.constant("Std.Bool.and").unwrap();
    let input = b.cube(8).unwrap();
    let x = b.var(0).unwrap();
    let mut work = b.app(x, vec![no; 8]).unwrap();
    for _ in 0..513 {
        work = b.app(not, vec![work]).unwrap();
    }
    let body = b.lam(input, work).unwrap();
    let ty = b.pi(input, b.boolean).unwrap();
    b.define("Test.EvictedFunction", ty, body).unwrap();
    let function = b.constant("Test.EvictedFunction").unwrap();
    let mut noise = vec![];
    for i in 0..80 {
        let name = format!("Test.MemoNoise{i}");
        b.define(&name, b.boolean, yes).unwrap();
        noise.push(b.constant(&name).unwrap());
    }
    let mut terms = vec![];
    for _ in 0..8 {
        terms.push(b.app(function, vec![x]).unwrap());
        terms.extend(noise.iter().copied());
    }
    let mut body = yes;
    for term in terms.into_iter().rev() {
        body = b.app(and, vec![term, body]).unwrap();
    }
    let body = b.lam(input, body).unwrap();
    b.define("Test.ReuseEvictedFunction", ty, body).unwrap();
    let c = decode_canonical_certificate(&b.finish().unwrap()).unwrap();
    let function = run(&c, "Test.EvictedFunction", vec![]);
    let (value, cold) = apply_counted(&c, function, sparse_cube(8, Default::default()));
    assert!(bit(value));
    let program = run(&c, "Test.ReuseEvictedFunction", vec![]);
    let (value, shared) = apply_counted(&c, program, sparse_cube(8, Default::default()));
    assert!(bit(value));
    assert!(
        shared < cold * 4,
        "closed function memo lost: cold={cold}, repeated={shared}"
    );
    // The table is local to one evaluation/certificate, including identical
    // declaration numbers with a changed function body in a later certificate.
    let mut changed = c.clone();
    let decl = changed
        .declarations
        .iter_mut()
        .find(|d| changed.name_table[d.name as usize] == "Test.EvictedFunction")
        .unwrap();
    let mpk_cert::encode::DeclarationKind::Def { value, .. } = &mut decl.kind else {
        panic!()
    };
    let TermNode::Lam { body, .. } = c.term_table[*value as usize] else {
        panic!()
    };
    let TermNode::App { ref arguments, .. } = c.term_table[body as usize] else {
        panic!()
    };
    // Remove one outer negation: 512 negations of the false input return false.
    let replacement = arguments[0];
    let TermNode::Lam { body, .. } = &mut changed.term_table[*value as usize] else {
        panic!()
    };
    *body = replacement;
    let program = run(&changed, "Test.ReuseEvictedFunction", vec![]);
    assert!(!bit(apply(
        &changed,
        program,
        sparse_cube(8, Default::default())
    )));
}

#[test]
fn ordinary_core_unused_binders_share_closures_and_release_arguments() {
    let mut b = Builder::new().unwrap();
    let outer = b.var(2).unwrap();
    let leaf = b.lam(b.boolean, outer).unwrap();
    let input = b.cube(20).unwrap();
    let middle = b.lam(input, leaf).unwrap();
    let body = b.lam(b.boolean, middle).unwrap();
    let output = b.cube(1).unwrap();
    let ty = b.pi(input, output).unwrap();
    let ty = b.pi(b.boolean, ty).unwrap();
    b.define("Test.IgnoreCubeCaptureOuter", ty, body).unwrap();
    let c = decode_canonical_certificate(&b.finish().unwrap()).unwrap();
    for outer in [false, true] {
        let f = run(&c, "Test.IgnoreCubeCaptureOuter", vec![V::Bit(outer)]);
        let env = Rc::new(Env::Bind(
            V::Cube(vec![true; 4096]),
            Rc::new(Env::Empty).into(),
        ));
        let weak = Rc::downgrade(&env);
        let a = apply(&c, f.clone(), thunk(u32::MAX, env));
        assert!(
            weak.upgrade().is_none(),
            "unused input environment was retained"
        );
        let b = apply(&c, f, sparse_cube(20, [1, 1 << 19].into_iter().collect()));
        let (V::Lambda(_, _, a_memo), V::Lambda(_, _, b_memo)) = (&a, &b) else {
            panic!()
        };
        assert!(
            Rc::ptr_eq(a_memo, b_memo),
            "unused input rebuilt the same closure"
        );
        for selector in [false, true] {
            assert_eq!(bit(apply(&c, a.clone(), V::Bit(selector))), outer);
            assert_eq!(bit(apply(&c, b.clone(), V::Bit(selector))), outer);
        }
    }
}

#[test]
fn ordinary_core_usage_analysis_preserves_lets_and_deep_captures() {
    let mut b = Builder::new().unwrap();
    let x = b.var(0).unwrap();
    let y = b.var(1).unwrap();
    let body = b.lam(b.boolean, y).unwrap();
    let body = b
        .term(TermNode::Let {
            ty: b.boolean,
            value: x,
            body,
        })
        .unwrap();
    let body = b.lam(b.boolean, body).unwrap();
    let ty = b.cube(2).unwrap();
    b.define("Test.LetCapture", ty, body).unwrap();
    // Values above the compact usage mask must conservatively remain live.
    const DEPTH: u32 = 140;
    let mut body = b.var(DEPTH).unwrap();
    for _ in 0..=DEPTH {
        body = b.lam(b.boolean, body).unwrap();
    }
    let ty = b.cube(DEPTH + 1).unwrap();
    b.define("Test.DeepCapture", ty, body).unwrap();
    let c = decode_canonical_certificate(&b.finish().unwrap()).unwrap();
    let f = run(&c, "Test.LetCapture", vec![]);
    // The first result returns a closure without demanding x. It must not
    // be cached as independent: invoking that result would demand poison.
    let unused_yet = apply(&c, f.clone(), thunk(u32::MAX, Rc::new(Env::Empty)));
    drop(unused_yet);
    for x in [false, true] {
        let inner = apply(&c, f.clone(), V::Bit(x));
        assert_eq!(bit(apply(&c, inner, V::Bit(!x))), x);
        let mut deep = run(&c, "Test.DeepCapture", vec![V::Bit(x)]);
        for _ in 0..DEPTH {
            deep = apply(&c, deep, V::Bit(!x));
        }
        assert_eq!(bit(deep), x);
    }
}

#[test]
fn ordinary_core_usage_cache_matches_small_boolean_terms() {
    #[derive(Clone)]
    enum Expression {
        Bool(bool),
        Var(u32),
        Not(Box<Self>),
        And(Box<Self>, Box<Self>),
        If(Box<Self>, Box<Self>, Box<Self>),
        Let(Box<Self>, Box<Self>),
    }
    impl Expression {
        fn expected(&self, env: &[bool]) -> bool {
            match self {
                Self::Bool(b) => *b,
                Self::Var(i) => env[*i as usize],
                Self::Not(a) => !a.expected(env),
                Self::And(a, b) => a.expected(env) && b.expected(env),
                Self::If(c, a, b) => {
                    if c.expected(env) {
                        a.expected(env)
                    } else {
                        b.expected(env)
                    }
                }
                Self::Let(a, b) => b.expected(
                    &std::iter::once(a.expected(env))
                        .chain(env.iter().copied())
                        .collect::<Vec<_>>(),
                ),
            }
        }
        fn lower(&self, b: &mut Builder) -> u32 {
            match self {
                Self::Bool(on) => b
                    .constant(if *on {
                        "Std.Bool.true"
                    } else {
                        "Std.Bool.false"
                    })
                    .unwrap(),
                Self::Var(i) => b.var(*i).unwrap(),
                Self::Not(a) => {
                    let a = a.lower(b);
                    let head = b.constant("Std.Bool.not").unwrap();
                    b.app(head, vec![a]).unwrap()
                }
                Self::And(a, c) => {
                    let a = a.lower(b);
                    let c = c.lower(b);
                    let head = b.constant("Std.Bool.and").unwrap();
                    b.app(head, vec![a, c]).unwrap()
                }
                Self::If(c, yes, no) => {
                    let c = c.lower(b);
                    let yes = yes.lower(b);
                    let no = no.lower(b);
                    let rec = b.constant("Std.Bool.rec").unwrap();
                    b.app(rec, vec![no, yes, c]).unwrap()
                }
                Self::Let(a, body) => {
                    let value = a.lower(b);
                    let body = body.lower(b);
                    b.term(TermNode::Let {
                        ty: b.boolean,
                        value,
                        body,
                    })
                    .unwrap()
                }
            }
        }
    }
    use Expression::*;
    let atoms = [Bool(false), Bool(true), Var(0), Var(1)];
    let mut expressions = atoms.to_vec();
    for a in &atoms {
        expressions.push(Not(Box::new(a.clone())));
        expressions.push(Let(
            Box::new(a.clone()),
            Box::new(And(Box::new(Var(0)), Box::new(Var(2)))),
        ));
        for b in &atoms {
            expressions.push(And(Box::new(a.clone()), Box::new(b.clone())));
            for c in &atoms {
                expressions.push(If(
                    Box::new(a.clone()),
                    Box::new(b.clone()),
                    Box::new(c.clone()),
                ));
            }
        }
    }
    assert_eq!(expressions.len(), 92);
    let mut b = Builder::new().unwrap();
    for (i, expression) in expressions.iter().enumerate() {
        let value = expression.lower(&mut b);
        let value = b.lam(b.boolean, value).unwrap();
        let value = b.lam(b.boolean, value).unwrap();
        let ty = b.cube(2).unwrap();
        b.define(&format!("Test.Boolean{i}"), ty, value).unwrap();
    }
    let yes = b.constant("Std.Bool.true").unwrap();
    let no = b.constant("Std.Bool.false").unwrap();
    let c = decode_canonical_certificate(&b.finish().unwrap()).unwrap();
    let mut observations = 0;
    for (i, expression) in expressions.iter().enumerate() {
        let f = run(&c, &format!("Test.Boolean{i}"), vec![]);
        for delayed in [false, true] {
            for x in [false, true] {
                for y in [false, true] {
                    let arg = |value| {
                        if delayed {
                            thunk(if value { yes } else { no }, Rc::new(Env::Empty))
                        } else {
                            V::Bit(value)
                        }
                    };
                    let inner = apply(&c, f.clone(), arg(x));
                    assert_eq!(
                        bit(apply(&c, inner, arg(y))),
                        expression.expected(&[y, x]),
                        "expression {i}: {x}/{y}, delayed={delayed}"
                    );
                    observations += 1;
                }
            }
        }
    }
    assert_eq!(observations, 736);
    eprintln!("unused-argument cache: 92 Boolean/let terms, 736 eager/delayed observations");
}

#[test]
fn ordinary_core_closed_definition_cache_is_bounded_and_local() {
    let mut b = Builder::new().unwrap();
    let yes = b.constant("Std.Bool.true").unwrap();
    let no = b.constant("Std.Bool.false").unwrap();
    let not = b.constant("Std.Bool.not").unwrap();
    let and = b.constant("Std.Bool.and").unwrap();
    let mut expensive = yes;
    for _ in 0..128 {
        expensive = b.app(not, vec![expensive]).unwrap();
    }
    b.define("Test.ClosedWork", b.boolean, expensive).unwrap();
    let closed = b.constant("Test.ClosedWork").unwrap();
    let mut repeated = yes;
    for _ in 0..128 {
        repeated = b.app(and, vec![closed, repeated]).unwrap();
    }
    b.define("Test.RepeatedClosedWork", b.boolean, repeated)
        .unwrap();
    // More than the cache capacity: all demands must still compute correctly,
    // including demands for entries that were evicted in the meantime.
    let mut evictions = closed;
    for i in 0..160 {
        let value = b.app(not, vec![no]).unwrap();
        let name = format!("Test.Closed{i}");
        b.define(&name, b.boolean, value).unwrap();
        let value = b.constant(&name).unwrap();
        evictions = b.app(and, vec![value, evictions]).unwrap();
    }
    b.define("Test.Evictions", b.boolean, evictions).unwrap();
    let c = decode_canonical_certificate(&b.finish().unwrap()).unwrap();
    let (value, steps) = run_counted(&c, "Test.RepeatedClosedWork");
    assert!(bit(value));
    assert!(steps < 30_000, "closed definition recomputed: {steps}");
    assert!(bit(run(&c, "Test.Evictions", vec![])));
    // A second certificate has the same global index/name but a different body.
    // No result from the first evaluation may be reused in this context.
    let mut changed = c.clone();
    let declaration = changed
        .declarations
        .iter_mut()
        .find(|d| changed.name_table[d.name as usize] == "Test.ClosedWork")
        .unwrap();
    let mpk_cert::encode::DeclarationKind::Def { value, .. } = &mut declaration.kind else {
        panic!()
    };
    *value = no;
    assert!(!bit(run(&changed, "Test.RepeatedClosedWork", vec![])));
    assert!(bit(run(&c, "Test.RepeatedClosedWork", vec![])));
    eprintln!("bounded closed-definition cache: repeated work {steps} transitions; 160-entry eviction and cross-certificate cases passed");
}

#[test]
fn ordinary_core_shared_function_arguments_reuse_results_without_retention() {
    let mut b = Builder::new().unwrap();
    let yes = b.constant("Std.Bool.true").unwrap();
    let not = b.constant("Std.Bool.not").unwrap();
    let x = b.var(0).unwrap();
    let mut body = b.app(x, vec![yes]).unwrap();
    for _ in 0..128 {
        body = b.app(not, vec![body]).unwrap();
    }
    let input = b.cube(1).unwrap();
    let body = b.lam(input, body).unwrap();
    let ty = b.pi(input, b.boolean).unwrap();
    b.define("Test.ConsumeFunction", ty, body).unwrap();
    let outer = b.var(1).unwrap();
    let body = b.lam(b.boolean, outer).unwrap();
    let body = b.lam(b.boolean, body).unwrap();
    let ty = b.cube(2).unwrap();
    b.define("Test.ConstantFunction", ty, body).unwrap();
    let c = decode_canonical_certificate(&b.finish().unwrap()).unwrap();
    let f = run(&c, "Test.ConsumeFunction", vec![]);
    for expected in [true, false, true] {
        let g = run(&c, "Test.ConstantFunction", vec![V::Bit(expected)]);
        let V::Lambda(_, _, memo) = &g else { panic!() };
        let weak = Rc::downgrade(memo);
        let (first, initial_steps) = apply_counted(&c, f.clone(), g.clone());
        assert_eq!(bit(first), expected);
        let (second, shared_steps) = apply_counted(&c, f.clone(), g.clone());
        assert_eq!(bit(second), expected);
        assert!(initial_steps > 1000, "missing expensive fixture");
        assert!(
            shared_steps < 10,
            "shared function recomputed: {shared_steps}"
        );
        let delayed = thunk(
            x,
            EnvRef::new(Env::Bind(g.clone(), EnvRef::new(Env::Empty))),
        );
        let V::Thunk(delayed_memo) = &delayed else {
            panic!()
        };
        let delayed_weak = Rc::downgrade(delayed_memo);
        assert_eq!(bit(apply(&c, f.clone(), delayed.clone())), expected);
        let (again, delayed_steps) = apply_counted(&c, f.clone(), delayed.clone());
        assert_eq!(bit(again), expected);
        assert!(delayed_steps < 10, "shared suspension recomputed");
        drop(delayed);
        assert!(
            delayed_weak.upgrade().is_none(),
            "cache retained suspension"
        );
        drop(g);
        assert!(weak.upgrade().is_none(), "cache retained its argument");
    }
    // Offset/stride participate in identity: two views of the same storage
    // are different function arguments, including sparse storage.
    let bits = Rc::new(vec![false, true, false, true]);
    let sparse = Rc::new(
        [1usize, 3]
            .into_iter()
            .collect::<std::collections::BTreeSet<_>>(),
    );
    for offset in [0, 1, 0, 1] {
        for g in [
            V::SharedCube(bits.clone(), offset, 2),
            V::SparseCube(sparse.clone(), offset, 2, 4),
        ] {
            assert_eq!(bit(apply(&c, f.clone(), g)), offset == 1);
        }
    }
    let bits = Rc::new((0..64).map(|i| i % 3 == 0).collect::<Vec<_>>());
    for offset in 0..16 {
        assert_eq!(
            bit(apply(
                &c,
                f.clone(),
                V::SharedCube(bits.clone(), offset, 32)
            )),
            (offset + 32) % 3 == 0
        );
    }
    let (retained, hit_steps) = apply_counted(&c, f.clone(), V::SharedCube(bits.clone(), 15, 32));
    assert!(!bit(retained));
    assert!(hit_steps < 10);
    let (evicted, miss_steps) = apply_counted(&c, f, V::SharedCube(bits, 0, 32));
    assert!(!bit(evicted));
    assert!(miss_steps > 1000, "identity entries exceeded their bound");
}

#[test]
fn ordinary_core_function_result_cache_does_not_create_capture_cycles() {
    let mut b = Builder::new().unwrap();
    let input = b.cube(1).unwrap();
    let function_type = b.pi(input, input).unwrap();
    let x = b.var(0).unwrap();
    let identity = b.lam(input, x).unwrap();
    b.define("Test.FunctionIdentity", function_type, identity)
        .unwrap();
    let outer = b.var(1).unwrap();
    let constant = b.lam(b.boolean, outer).unwrap();
    let function = b.var(1).unwrap();
    let body = b.app(function, vec![constant]).unwrap();
    let no = b.constant("Std.Bool.false").unwrap();
    let body = b.app(body, vec![no]).unwrap();
    let body = b.lam(b.boolean, body).unwrap();
    let body = b.lam(function_type, body).unwrap();
    let ty = b.pi(function_type, input).unwrap();
    b.define("Test.CaptureFunction", ty, body).unwrap();
    let c = decode_canonical_certificate(&b.finish().unwrap()).unwrap();
    let f = run(&c, "Test.FunctionIdentity", vec![]);
    let g = run(&c, "Test.CaptureFunction", vec![f.clone()]);
    let V::Lambda(_, _, fm) = &f else { panic!() };
    let fw = Rc::downgrade(fm);
    let V::Lambda(_, _, gm) = &g else { panic!() };
    let gw = Rc::downgrade(gm);
    // g captures f, so a strong cached result edge f -> g would form a cycle.
    let result = apply(&c, f.clone(), g.clone());
    let (again, steps) = apply_counted(&c, f.clone(), g.clone());
    assert!(steps < 10);
    assert!(bit(apply(&c, result.clone(), V::Bit(true))));
    assert!(!bit(apply(&c, again.clone(), V::Bit(false))));
    drop(result);
    drop(again);
    drop(g);
    drop(f);
    assert!(gw.upgrade().is_none(), "cache retained returned closure");
    assert!(fw.upgrade().is_none(), "cache introduced a capture cycle");
}

#[test]
fn ordinary_core_boolean_and_unused_result_caches_release_closures() {
    let mut b = Builder::new().unwrap();
    let captured = b.var(1).unwrap();
    let no = b.constant("Std.Bool.false").unwrap();
    for (name, result) in [("Test.CapturedResult", captured), ("Test.UnusedResult", no)] {
        let inner = b.lam(b.boolean, result).unwrap();
        let outer = b.lam(b.boolean, inner).unwrap();
        let ty = b.cube(2).unwrap();
        b.define(name, ty, outer).unwrap();
    }
    let c = decode_canonical_certificate(&b.finish().unwrap()).unwrap();
    for unused in [false, true] {
        let f = run(
            &c,
            if unused {
                "Test.UnusedResult"
            } else {
                "Test.CapturedResult"
            },
            vec![],
        );
        for input in [false, true] {
            // Unused poison must remain unforced even after cache expiration.
            let argument = || {
                if unused {
                    thunk(u32::MAX, Rc::new(Env::Empty))
                } else {
                    V::Bit(input)
                }
            };
            let first = apply(&c, f.clone(), argument());
            let V::Lambda(_, _, memo) = &first else {
                panic!()
            };
            let weak = Rc::downgrade(memo);
            let again = apply(&c, f.clone(), argument());
            let V::Lambda(_, _, again_memo) = &again else {
                panic!()
            };
            assert!(Rc::ptr_eq(memo, again_memo), "live result was not shared");
            assert_eq!(
                bit(apply(&c, first.clone(), V::Bit(false))),
                !unused && input
            );
            drop(first);
            assert!(weak.upgrade().is_some(), "live alias lost its result");
            drop(again);
            // Keep the source function alive: its cache must not own the result.
            // The old strong Boolean/independent caches fail this assertion.
            assert!(
                weak.upgrade().is_none(),
                "cache retained an unused result closure"
            );
            let rebuilt = apply(&c, f.clone(), argument());
            assert_eq!(bit(apply(&c, rebuilt, V::Bit(true))), !unused && input);
        }
    }
}
