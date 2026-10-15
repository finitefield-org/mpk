//! Compositional unsigned word value conservation using ordinary functions.
//! Only the three flags of the local full adder are split into cases.
//! Width induction is performed by the untrusted producer, producing checked
//! theorem applications, not by enumerating complete operand bit patterns.

use super::*;
use mpk_cert::encode::DefinitionReducibility;

mod balanced;
pub(super) fn generate_order(root: &Path, output: &Path, depth: usize, trace: bool) {
    balanced::generate_order(root, output, depth, trace);
}
pub(super) fn generate_balanced(root: &Path, output: &Path, depth: usize, trace: bool) {
    balanced::generate(root, output, depth, trace);
}
pub(super) fn generate_signed(root: &Path, output: &Path, depth: usize, trace: bool) {
    balanced::generate_signed(root, output, depth, trace);
}
pub(super) fn generate_fold(root: &Path, output: &Path, count: usize, trace: bool) {
    balanced::generate_fold(root, output, count, trace);
}
pub(super) fn generate_signed_fold(root: &Path, output: &Path, count: usize, trace: bool) {
    balanced::generate_signed_fold(root, output, count, trace);
}

const PREFIX: &str = "Probe.GoPractical.Word";
fn name(suffix: &str) -> String {
    format!("{PREFIX}.{suffix}")
}
fn nat() -> Expr {
    c("Std.Nat")
}
fn function() -> Expr {
    forall("__arg", nat(), nat())
}
fn eqf(left: Expr, right: Expr) -> Expr {
    apply(c(EQ), vec![function(), left, right])
}
fn cmp(left: Expr, right: Expr) -> Expr {
    apply(c(&name("Compose")), vec![left, right])
}
fn square(f: Expr) -> Expr {
    apply(c(&name("Square")), vec![f])
}
fn unit() -> Expr {
    c(&name("Identity"))
}
fn conditional(f: Expr, bit: Expr) -> Expr {
    apply(c(&name("Maybe")), vec![f, bit])
}
fn word(width: usize, f: Expr, bits: &[Expr]) -> Expr {
    assert_eq!(bits.len(), width);
    let mut args = vec![f];
    args.extend_from_slice(bits);
    apply(c(&name(&format!("Value{width}"))), args)
}
fn power(width: usize, f: Expr) -> Expr {
    apply(c(&name(&format!("Power{width}"))), vec![f])
}
fn commute(f: Expr, g: Expr) -> Expr {
    eqf(cmp(f.clone(), g.clone()), cmp(g, f))
}
fn reflf(value: Expr) -> Expr {
    apply(c(REFL), vec![function(), value])
}

#[derive(Clone)]
struct Equality {
    left: Expr,
    right: Expr,
    proof: Expr,
}
fn reflexive(value: Expr) -> Equality {
    Equality {
        left: value.clone(),
        right: value.clone(),
        proof: reflf(value),
    }
}
fn reverse(e: Equality) -> Equality {
    Equality {
        left: e.right.clone(),
        right: e.left.clone(),
        proof: apply(c("Std.Eq.symm"), vec![function(), e.left, e.right, e.proof]),
    }
}
fn transit(p: Equality, q: Equality) -> Equality {
    assert_eq!(p.right, q.left, "untrusted equality chain endpoints differ");
    Equality {
        left: p.left.clone(),
        right: q.right.clone(),
        proof: apply(
            c("Std.Eq.trans"),
            vec![function(), p.left, p.right, q.right, p.proof, q.proof],
        ),
    }
}
fn instance(suffix: &str, args: Vec<Expr>, left: Expr, right: Expr) -> Equality {
    Equality {
        left,
        right,
        proof: apply(c(&name(suffix)), args),
    }
}
fn fold_binders(binders: &[(String, Expr)], mut ty: Expr, mut value: Expr) -> (Expr, Expr) {
    for (n, t) in binders.iter().rev() {
        ty = forall(n, t.clone(), ty);
        value = lambda(n, t.clone(), value);
    }
    (ty, value)
}
fn bits(prefix: &str, width: usize) -> Vec<Expr> {
    (0..width).map(|i| v(&format!("{prefix}{i}"))).collect()
}
fn bit_binders(prefix: &str, width: usize) -> Vec<(String, Expr)> {
    (0..width)
        .map(|i| (format!("{prefix}{i}"), bool_ty()))
        .collect()
}
fn adder(x: &[Expr], y: &[Expr], incoming: Expr) -> (Vec<Expr>, Expr) {
    let mut carry = incoming;
    let mut result = vec![];
    for (x, y) in x.iter().zip(y) {
        let xy = xor(x.clone(), y.clone());
        result.push(xor(xy.clone(), carry.clone()));
        carry = or(and(x.clone(), y.clone()), and(carry, xy));
    }
    (result, carry)
}

struct Producer {
    builder: Builder,
    fresh: usize,
}
impl Producer {
    fn declare(&mut self, suffix: &str, ty: Expr, value: Expr, theorem: bool) {
        let ty = compile(&ty, &mut vec![], &mut self.builder);
        let value = compile(&value, &mut vec![], &mut self.builder);
        let global = self.builder.c.declarations.len() as u32;
        let n = self.builder.c.name_table.len() as u32;
        let declaration_name = name(suffix);
        assert!(!self.builder.globals.contains_key(&declaration_name));
        self.builder.c.name_table.push(declaration_name.clone());
        self.builder.c.declarations.push(Declaration {
            name: n,
            kind: if theorem {
                DeclarationKind::Theorem { ty, proof: value }
            } else {
                DeclarationKind::Def {
                    ty,
                    value,
                    reducibility: DefinitionReducibility::Reducible,
                }
            },
        });
        self.builder.globals.insert(declaration_name, global);
    }
    fn theorem(&mut self, suffix: &str, binders: &[(String, Expr)], e: Equality) {
        let (ty, proof) = fold_binders(binders, eqf(e.left, e.right), e.proof);
        self.declare(suffix, ty, proof, true);
    }
    fn rewrite(&mut self, whole: Expr, e: Equality) -> Equality {
        if e.left == e.right {
            return reflexive(whole);
        }
        let right = substitute(&whole, &e.left, &e.right);
        assert_ne!(whole, right, "rewrite pattern absent: {:?}", e.left);
        let n = format!("__word_rewrite_{}", self.fresh);
        self.fresh += 1;
        let context = lambda(&n, function(), substitute(&whole, &e.left, &v(&n)));
        Equality {
            left: whole,
            right,
            proof: apply(
                c("Std.Eq.congr"),
                vec![function(), function(), context, e.left, e.right, e.proof],
            ),
        }
    }
    fn change(&mut self, p: Equality, e: Equality) -> Equality {
        let q = self.rewrite(p.right.clone(), e);
        transit(p, q)
    }
    fn assoc(a: Expr, b: Expr, d: Expr) -> Equality {
        instance(
            "Associative",
            vec![a.clone(), b.clone(), d.clone()],
            cmp(cmp(a.clone(), b.clone()), d.clone()),
            cmp(a, cmp(b, d)),
        )
    }
    fn comp_parts(e: &Expr) -> Option<(Expr, Expr)> {
        if let Expr::Apply(head, args) = e {
            if **head == c(&name("Compose")) && args.len() == 2 {
                return Some((args[0].clone(), args[1].clone()));
            }
        }
        None
    }
    fn normalize_monoid(&mut self, e: Expr) -> Equality {
        let Some((a, b)) = Self::comp_parts(&e) else {
            return reflexive(e);
        };
        let pa = self.normalize_monoid(a);
        let a = pa.right.clone();
        let mut p = Self::on_left(pa, b.clone());
        let pb = self.normalize_monoid(b);
        p = transit(p, Self::on_right(a, pb));
        loop {
            let (a, b) = Self::comp_parts(&p.right).unwrap();
            let Some((x, y)) = Self::comp_parts(&a) else {
                break;
            };
            p = transit(p, Self::assoc(x.clone(), y.clone(), b.clone()));
            let tail = self.normalize_monoid(cmp(y, b));
            p = transit(p, Self::on_right(x, tail));
        }
        p
    }
    fn monoid(&mut self, left: Expr, right: Expr) -> Equality {
        let p = self.normalize_monoid(left);
        let q = self.normalize_monoid(right);
        assert_eq!(p.right, q.right, "associative normal forms differ");
        transit(p, reverse(q))
    }
    fn product(items: &[Expr]) -> Expr {
        assert!(!items.is_empty());
        items[..items.len() - 1]
            .iter()
            .rev()
            .fold(items.last().unwrap().clone(), |tail, x| {
                cmp(x.clone(), tail)
            })
    }
    fn swap(&mut self, p: Equality, items: &mut [Expr], index: usize, e: Equality) -> Equality {
        assert_eq!(e.left, cmp(items[index].clone(), items[index + 1].clone()));
        let mut grouped = items[..index].to_vec();
        grouped.push(e.left.clone());
        grouped.extend_from_slice(&items[index + 2..]);
        let target = Self::product(&grouped);
        let q = self.monoid(p.right.clone(), target);
        let p = transit(p, q);
        let p = self.change(p, e);
        items.swap(index, index + 1);
        let q = self.monoid(p.right.clone(), Self::product(items));
        transit(p, q)
    }
    fn square_unfold(f: Expr) -> Equality {
        instance(
            "SquareDefinition",
            vec![f.clone()],
            square(f.clone()),
            cmp(f.clone(), f),
        )
    }
    fn square_commute(f: Expr, g: Expr, h: Equality) -> Equality {
        assert_eq!(h.left, cmp(f.clone(), g.clone()));
        instance(
            "SquareCommutes",
            vec![f.clone(), g.clone(), h.proof],
            cmp(square(f.clone()), g.clone()),
            cmp(g, square(f)),
        )
    }
    fn maybe_left(f: Expr, g: Expr, b: Expr, h: Equality) -> Equality {
        assert_eq!(h.left, cmp(f.clone(), g.clone()));
        instance(
            "MaybeCommutes",
            vec![f.clone(), g.clone(), b.clone(), h.proof],
            cmp(conditional(f.clone(), b.clone()), g.clone()),
            cmp(g, conditional(f, b)),
        )
    }
    fn maybe_right(f: Expr, g: Expr, b: Expr, h: Equality) -> Equality {
        reverse(Self::maybe_left(g, f, b, reverse(h)))
    }
    fn on_left(e: Equality, b: Expr) -> Equality {
        let context = lambda(
            "__compose_hole",
            function(),
            cmp(v("__compose_hole"), b.clone()),
        );
        Equality {
            left: cmp(e.left.clone(), b.clone()),
            right: cmp(e.right.clone(), b),
            proof: apply(
                c("Std.Eq.congr"),
                vec![function(), function(), context, e.left, e.right, e.proof],
            ),
        }
    }
    fn on_right(a: Expr, e: Equality) -> Equality {
        let context = lambda(
            "__compose_hole",
            function(),
            cmp(a.clone(), v("__compose_hole")),
        );
        Equality {
            left: cmp(a.clone(), e.left.clone()),
            right: cmp(a, e.right.clone()),
            proof: apply(
                c("Std.Eq.congr"),
                vec![function(), function(), context, e.left, e.right, e.proof],
            ),
        }
    }
    fn word_unfold(width: usize, f: Expr, bits: &[Expr]) -> Equality {
        let left = word(width, f.clone(), bits);
        let mut args = vec![f.clone()];
        args.extend_from_slice(bits);
        let right = if width == 0 {
            unit()
        } else {
            cmp(
                conditional(f.clone(), bits[0].clone()),
                word(width - 1, square(f), &bits[1..]),
            )
        };
        instance(&format!("ValueDefinition{width}"), args, left, right)
    }
    fn word_commute(width: usize, f: Expr, g: Expr, h: Equality, bits: &[Expr]) -> Equality {
        let value = word(width, f.clone(), bits);
        let mut args = vec![f, g.clone(), h.proof];
        args.extend_from_slice(bits);
        instance(
            &format!("ValueCommutes{width}"),
            args,
            cmp(value.clone(), g.clone()),
            cmp(g, value),
        )
    }
    fn power_unfold(width: usize, f: Expr) -> Equality {
        let left = power(width, f.clone());
        let right = if width == 0 {
            f.clone()
        } else {
            power(width - 1, square(f.clone()))
        };
        instance(&format!("PowerDefinition{width}"), vec![f], left, right)
    }
    fn addition(width: usize, f: Expr, x: &[Expr], y: &[Expr], carry: Expr) -> Equality {
        let (sum, out) = adder(x, y, carry.clone());
        let left = cmp(
            word(width, f.clone(), x),
            cmp(
                word(width, f.clone(), y),
                conditional(f.clone(), carry.clone()),
            ),
        );
        let right = cmp(
            word(width, f.clone(), &sum),
            conditional(power(width, f.clone()), out),
        );
        let mut args = vec![f];
        args.extend_from_slice(x);
        args.extend_from_slice(y);
        args.push(carry);
        instance(&format!("Addition{width}"), args, left, right)
    }
    fn primitive_library(&mut self) {
        self.declare("Identity", function(), lambda("n", nat(), v("n")), false);
        let comp_binders = vec![("a".into(), function()), ("b".into(), function())];
        let comp_value = lambda("n", nat(), apply(v("a"), vec![apply(v("b"), vec![v("n")])]));
        let (ty, value) = fold_binders(&comp_binders, function(), comp_value);
        self.declare("Compose", ty, value, false);
        let f_binders = vec![("f".into(), function())];
        let (ty, value) = fold_binders(&f_binders, function(), cmp(v("f"), v("f")));
        self.declare("Square", ty, value, false);
        let maybe_binders = vec![("f".into(), function()), ("b".into(), bool_ty())];
        let (ty, value) = fold_binders(
            &maybe_binders,
            function(),
            choose_t(function(), v("b"), v("f"), unit()),
        );
        self.declare("Maybe", ty, value, false);
        let abg = vec![
            ("a".into(), function()),
            ("b".into(), function()),
            ("g".into(), function()),
        ];
        let left = cmp(cmp(v("a"), v("b")), v("g"));
        let right = cmp(v("a"), cmp(v("b"), v("g")));
        self.theorem(
            "Associative",
            &abg,
            Equality {
                left,
                right: right.clone(),
                proof: reflf(right),
            },
        );
        let left = square(v("f"));
        let right = cmp(v("f"), v("f"));
        self.theorem(
            "SquareDefinition",
            &f_binders,
            Equality {
                left,
                right: right.clone(),
                proof: reflf(right),
            },
        );

        let mut binders = abg.clone();
        binders.extend([
            ("ha".into(), commute(v("a"), v("g"))),
            ("hb".into(), commute(v("b"), v("g"))),
        ]);
        let mut items = vec![v("a"), v("b"), v("g")];
        let left = cmp(cmp(v("a"), v("b")), v("g"));
        let mut p = self.monoid(left, Self::product(&items));
        let hb = Equality {
            left: cmp(v("b"), v("g")),
            right: cmp(v("g"), v("b")),
            proof: v("hb"),
        };
        p = self.swap(p, &mut items, 1, hb);
        let ha = Equality {
            left: cmp(v("a"), v("g")),
            right: cmp(v("g"), v("a")),
            proof: v("ha"),
        };
        p = self.swap(p, &mut items, 0, ha);
        self.theorem("ProductCommutes", &binders, p);

        let fg = vec![
            ("f".into(), function()),
            ("g".into(), function()),
            ("h".into(), commute(v("f"), v("g"))),
        ];
        let mut p = self.rewrite(cmp(square(v("f")), v("g")), Self::square_unfold(v("f")));
        let q = instance(
            "ProductCommutes",
            vec![v("f"), v("f"), v("g"), v("h"), v("h")],
            cmp(cmp(v("f"), v("f")), v("g")),
            cmp(v("g"), cmp(v("f"), v("f"))),
        );
        p = transit(p, q);
        p = self.change(p, reverse(Self::square_unfold(v("f"))));
        self.theorem("SquareCommutes", &fg, p);

        let mut maybe_fg = fg.clone();
        // The public lemma argument order is f, g, bit, commutation witness.
        maybe_fg.insert(2, ("bit".into(), bool_ty()));
        let left = cmp(conditional(v("f"), v("bit")), v("g"));
        let right = cmp(v("g"), conditional(v("f"), v("bit")));
        let goal = eqf(left.clone(), right.clone());
        let motive = lambda(
            "__selected_bit",
            bool_ty(),
            substitute(&goal, &v("bit"), &v("__selected_bit")),
        );
        let no_right = substitute(&right, &v("bit"), &literal(false));
        let proof = apply(
            c("Std.Bool.cases"),
            vec![motive, reflf(no_right), v("h"), v("bit")],
        );
        self.theorem("MaybeCommutes", &maybe_fg, Equality { left, right, proof });

        let local_binders = vec![
            ("f".into(), function()),
            ("x".into(), bool_ty()),
            ("y".into(), bool_ty()),
            ("incoming".into(), bool_ty()),
        ];
        let (sum, carry) = adder(&[v("x")], &[v("y")], v("incoming"));
        let left = cmp(
            conditional(v("f"), v("x")),
            cmp(
                conditional(v("f"), v("y")),
                conditional(v("f"), v("incoming")),
            ),
        );
        let right = cmp(
            conditional(v("f"), sum[0].clone()),
            conditional(square(v("f")), carry),
        );
        let proof = self.prove_local(
            &eqf(left.clone(), right.clone()),
            &[v("x"), v("y"), v("incoming")],
        );
        self.theorem("FullAdder", &local_binders, Equality { left, right, proof });
    }
    fn normalize_local(&mut self, expr: Expr) -> Equality {
        if let Some((a, b)) = Self::comp_parts(&expr) {
            let pa = self.normalize_local(a);
            let a = pa.right.clone();
            let p = Self::on_left(pa, b.clone());
            let pb = self.normalize_local(b);
            return transit(p, Self::on_right(a, pb));
        }
        let Expr::Apply(head, args) = &expr else {
            return reflexive(expr);
        };
        if **head == c("Std.Bool.cases") && args.len() == 4 {
            let (condition, proof) = normalize_ground_boolean(&args[3], &mut self.fresh);
            let selected = if condition == literal(false) {
                args[1].clone()
            } else {
                assert_eq!(condition, literal(true));
                args[2].clone()
            };
            let n = format!("__local_function_condition_{}", self.fresh);
            self.fresh += 1;
            let mut context_args = args.clone();
            context_args[3] = v(&n);
            let context = lambda(&n, bool_ty(), apply((**head).clone(), context_args));
            let mut rewritten = args.clone();
            rewritten[3] = condition.clone();
            let middle = apply((**head).clone(), rewritten);
            let e = Equality {
                left: expr.clone(),
                right: middle.clone(),
                proof: apply(
                    c("Std.Eq.congr"),
                    vec![
                        bool_ty(),
                        function(),
                        context,
                        args[3].clone(),
                        condition,
                        proof,
                    ],
                ),
            };
            let e = transit(
                e,
                Equality {
                    left: middle,
                    right: selected.clone(),
                    proof: reflf(selected.clone()),
                },
            );
            let tail = self.normalize_local(selected);
            return transit(e, tail);
        }
        if **head != c(&name("Maybe")) {
            return reflexive(expr);
        }
        let (bit, proof) = normalize_ground_boolean(&args[1], &mut self.fresh);
        let selected = if bit == literal(false) {
            unit()
        } else {
            assert_eq!(bit, literal(true));
            args[0].clone()
        };
        let context = lambda(
            "__local_bit",
            bool_ty(),
            conditional(args[0].clone(), v("__local_bit")),
        );
        let middle = conditional(args[0].clone(), bit.clone());
        let p = Equality {
            left: expr.clone(),
            right: middle.clone(),
            proof: apply(
                c("Std.Eq.congr"),
                vec![bool_ty(), function(), context, args[1].clone(), bit, proof],
            ),
        };
        transit(
            p,
            Equality {
                left: middle,
                right: selected.clone(),
                proof: reflf(selected),
            },
        )
    }
    fn prove_local(&mut self, goal: &Expr, majors: &[Expr]) -> Expr {
        if let Some((major, rest)) = majors.split_first() {
            let n = format!("__local_case_{}", self.fresh);
            self.fresh += 1;
            return apply(
                c("Std.Bool.cases"),
                vec![
                    lambda(&n, bool_ty(), substitute(goal, major, &v(&n))),
                    self.prove_local(&substitute(goal, major, &literal(false)), rest),
                    self.prove_local(&substitute(goal, major, &literal(true)), rest),
                    major.clone(),
                ],
            );
        }
        let Expr::Apply(_, args) = goal else {
            panic!("expected equality");
        };
        let p = self.normalize_local(args[1].clone());
        let q = self.normalize_local(args[2].clone());
        // The short local branch has at most three applications of arbitrary f.
        // Its associativity/identity computation is checked by core conversion.
        let bridge = Equality {
            left: p.right.clone(),
            right: q.right.clone(),
            proof: reflf(q.right.clone()),
        };
        transit(transit(p, bridge), reverse(q)).proof
    }
    fn definitions(&mut self, width: usize) {
        let mut binders = vec![("f".into(), function())];
        binders.extend(bit_binders("b", width));
        let bs = bits("b", width);
        let body = if width == 0 {
            unit()
        } else {
            cmp(
                conditional(v("f"), bs[0].clone()),
                word(width - 1, square(v("f")), &bs[1..]),
            )
        };
        let (ty, value) = fold_binders(&binders, function(), body.clone());
        self.declare(&format!("Value{width}"), ty, value, false);
        let mut e = Self::word_unfold(width, v("f"), &bs);
        e.proof = reflf(body);
        self.theorem(&format!("ValueDefinition{width}"), &binders, e);
        let f_binders = vec![("f".into(), function())];
        let body = if width == 0 {
            v("f")
        } else {
            power(width - 1, square(v("f")))
        };
        let (ty, value) = fold_binders(&f_binders, function(), body.clone());
        self.declare(&format!("Power{width}"), ty, value, false);
        let mut e = Self::power_unfold(width, v("f"));
        e.proof = reflf(body);
        self.theorem(&format!("PowerDefinition{width}"), &f_binders, e);
    }
    fn commuting_word(&mut self, width: usize) {
        let bs = bits("b", width);
        let mut binders = vec![
            ("f".into(), function()),
            ("g".into(), function()),
            ("h".into(), commute(v("f"), v("g"))),
        ];
        binders.extend(bit_binders("b", width));
        let target = Self::word_commute(
            width,
            v("f"),
            v("g"),
            Equality {
                left: cmp(v("f"), v("g")),
                right: cmp(v("g"), v("f")),
                proof: v("h"),
            },
            &bs,
        );
        let proof = if width == 0 {
            Equality {
                left: target.left,
                right: target.right.clone(),
                proof: reflf(target.right),
            }
        } else {
            let a = conditional(v("f"), bs[0].clone());
            let tail = word(width - 1, square(v("f")), &bs[1..]);
            let h = Equality {
                left: cmp(v("f"), v("g")),
                right: cmp(v("g"), v("f")),
                proof: v("h"),
            };
            let ha = Self::maybe_left(v("f"), v("g"), bs[0].clone(), h.clone());
            let hs = Self::square_commute(v("f"), v("g"), h);
            let ht = Self::word_commute(width - 1, square(v("f")), v("g"), hs, &bs[1..]);
            let mut p = self.rewrite(target.left, Self::word_unfold(width, v("f"), &bs));
            let q = instance(
                "ProductCommutes",
                vec![a.clone(), tail.clone(), v("g"), ha.proof, ht.proof],
                cmp(cmp(a.clone(), tail.clone()), v("g")),
                cmp(v("g"), cmp(a, tail)),
            );
            p = transit(p, q);
            self.change(p, reverse(Self::word_unfold(width, v("f"), &bs)))
        };
        self.theorem(&format!("ValueCommutes{width}"), &binders, proof);
    }
    fn adding_word(&mut self, width: usize) {
        let x = bits("x", width);
        let y = bits("y", width);
        let f = v("f");
        let incoming = v("incoming");
        let mut binders = vec![("f".into(), function())];
        binders.extend(bit_binders("x", width));
        binders.extend(bit_binders("y", width));
        binders.push(("incoming".into(), bool_ty()));
        let target = Self::addition(width, f.clone(), &x, &y, incoming.clone());
        if width == 0 {
            self.theorem(
                "Addition0",
                &binders,
                Equality {
                    left: target.left,
                    right: target.right.clone(),
                    proof: reflf(target.right),
                },
            );
            return;
        }
        let sq = square(f.clone());
        let a = conditional(f.clone(), x[0].clone());
        let b = conditional(f.clone(), y[0].clone());
        let c = conditional(f.clone(), incoming.clone());
        let xt = word(width - 1, sq.clone(), &x[1..]);
        let yt = word(width - 1, sq.clone(), &y[1..]);
        let ((s, k), (all_sum, all_carry)) = {
            let (s, k) = adder(&x[..1], &y[..1], incoming.clone());
            ((s[0].clone(), k), adder(&x, &y, incoming.clone()))
        };
        let hs = Self::square_commute(f.clone(), f.clone(), reflexive(cmp(f.clone(), f.clone())));
        let hx = Self::word_commute(width - 1, sq.clone(), f.clone(), hs.clone(), &x[1..]);
        let hy = Self::word_commute(width - 1, sq.clone(), f.clone(), hs, &y[1..]);
        let mut p = self.rewrite(target.left.clone(), Self::word_unfold(width, f.clone(), &x));
        p = self.change(p, Self::word_unfold(width, f.clone(), &y));
        let mut items = vec![a.clone(), xt.clone(), b.clone(), yt.clone(), c.clone()];
        let q = self.monoid(p.right.clone(), Self::product(&items));
        p = transit(p, q);
        p = self.swap(
            p,
            &mut items,
            1,
            Self::maybe_right(xt.clone(), f.clone(), y[0].clone(), hx.clone()),
        );
        p = self.swap(
            p,
            &mut items,
            3,
            Self::maybe_right(yt.clone(), f.clone(), incoming.clone(), hy),
        );
        p = self.swap(
            p,
            &mut items,
            2,
            Self::maybe_right(xt.clone(), f.clone(), incoming.clone(), hx),
        );
        let grouped = cmp(
            cmp(a.clone(), cmp(b.clone(), c.clone())),
            cmp(xt.clone(), yt.clone()),
        );
        let q = self.monoid(p.right.clone(), grouped);
        p = transit(p, q);
        let sf = conditional(f.clone(), s);
        let kf = conditional(sq.clone(), k.clone());
        let local = instance(
            "FullAdder",
            vec![f.clone(), x[0].clone(), y[0].clone(), incoming],
            cmp(a, cmp(b, c)),
            cmp(sf.clone(), kf.clone()),
        );
        p = self.change(p, local);
        let mut items = vec![sf.clone(), kf.clone(), xt.clone(), yt.clone()];
        let q = self.monoid(p.right.clone(), Self::product(&items));
        p = transit(p, q);
        let hxx = Self::word_commute(
            width - 1,
            sq.clone(),
            sq.clone(),
            reflexive(cmp(sq.clone(), sq.clone())),
            &x[1..],
        );
        let hyy = Self::word_commute(
            width - 1,
            sq.clone(),
            sq.clone(),
            reflexive(cmp(sq.clone(), sq.clone())),
            &y[1..],
        );
        p = self.swap(
            p,
            &mut items,
            1,
            reverse(Self::maybe_right(xt, sq.clone(), k.clone(), hxx)),
        );
        p = self.swap(
            p,
            &mut items,
            2,
            reverse(Self::maybe_right(yt, sq.clone(), k.clone(), hyy)),
        );
        let ih = Self::addition(width - 1, sq.clone(), &x[1..], &y[1..], k.clone());
        assert_eq!(p.right, cmp(sf.clone(), ih.left.clone()));
        p = self.change(p, ih);
        let (tail_sum, tail_carry) = adder(&x[1..], &y[1..], k);
        assert_eq!(tail_carry, all_carry);
        let z = word(width - 1, sq.clone(), &tail_sum);
        let final_old = conditional(power(width - 1, sq), all_carry);
        let q = self.monoid(p.right.clone(), cmp(cmp(sf, z), final_old));
        p = transit(p, q);
        p = self.change(p, reverse(Self::word_unfold(width, f.clone(), &all_sum)));
        p = self.change(p, reverse(Self::power_unfold(width, f)));
        assert_eq!(p.right, target.right);
        self.theorem(&format!("Addition{width}"), &binders, p);
    }
    fn nat_claim(&mut self, width: usize, mutation: usize) -> Expr {
        let x = bits("x", width);
        let y = bits("y", width);
        let incoming = v("incoming");
        let e = Self::addition(width, c("Std.Nat.succ"), &x, &y, incoming);
        let mut right = e.right.clone();
        if mutation == 1 {
            let (sum, _) = adder(&x, &y, v("incoming"));
            right = word(width, c("Std.Nat.succ"), &sum);
        } else if mutation == 2 {
            let (mut sum, carry) = adder(&x, &y, v("incoming"));
            sum[0] = not(sum[0].clone());
            right = cmp(
                word(width, c("Std.Nat.succ"), &sum),
                conditional(power(width, c("Std.Nat.succ")), carry),
            );
        }
        let initial = v("initial");
        let goal = apply(
            c(EQ),
            vec![
                nat(),
                apply(e.left.clone(), vec![initial.clone()]),
                apply(right, vec![initial.clone()]),
            ],
        );
        let context = lambda(
            "__numeric_function",
            function(),
            apply(v("__numeric_function"), vec![initial]),
        );
        let proof = apply(
            c("Std.Eq.congr"),
            vec![function(), nat(), context, e.left, e.right, e.proof],
        );
        let mut binders = vec![("initial".into(), nat())];
        binders.extend(bit_binders("x", width));
        binders.extend(bit_binders("y", width));
        binders.push(("incoming".into(), bool_ty()));
        let (ty, proof) = fold_binders(&binders, goal, proof);
        self.declare("Claim", ty.clone(), proof, true);
        ty
    }
}

pub(super) fn generate(root: &Path, output: &Path, width: usize) {
    assert!((1..=72).contains(&width));
    fs::create_dir_all(output).unwrap();
    let mut producer = Producer {
        builder: Builder::from(natural::foundation(root)),
        fresh: 0,
    };
    producer.builder.c.module = format!("{PREFIX}.Width{width}");
    producer.primitive_library();
    fs::write(
        output.join("library.mpcert"),
        Builder::from(producer.builder.c.clone()).finish(),
    )
    .unwrap();
    for n in 0..=width {
        producer.definitions(n);
        producer.commuting_word(n);
        producer.adding_word(n);
        if width <= 2 {
            fs::write(
                output.join(format!("prefix-{n}.mpcert")),
                Builder::from(producer.builder.c.clone()).finish(),
            )
            .unwrap();
        }
    }
    let base = producer.builder.c;
    let mut index = vec![];
    for mutation in 0..3 {
        let id = match mutation {
            0 => format!("word-{width}-nat-conservation"),
            1 => format!("word-{width}-nat-conservation-wrong"),
            _ => format!("word-{width}-low-bit-wrong"),
        };
        let mut producer = Producer {
            builder: Builder::from(base.clone()),
            fresh: 0,
        };
        let goal = producer.nat_claim(width, mutation);
        let terms = producer.builder.c.term_table.len();
        let declarations = producer.builder.c.declarations.len();
        let bytes = producer.builder.finish();
        fs::write(output.join(format!("{id}.mpcert")), &bytes).unwrap();
        index.push(json!({"id":id,"width":width,"expected":if mutation == 0 {"accepted"} else {"rejected"},
            "mutation":match mutation {0=>"none",1=>"dropped_final_carry",_=>"flipped_low_sum_bit"},
            "bytes":bytes.len(),"terms":terms,"declarations":declarations,"theorem":name("Claim"),
            "goal_debug":format!("{goal:?}"),"operand_binders":width*2,"initial_natural_quantified":true,
            "scope":"exact unsigned word addition with final carry in Nat; signed interpretation and prefix sums remain open"}));
    }
    fs::write(
        output.join("index.json"),
        serde_json::to_vec_pretty(&json!({
            "schema":"mpk.go_practical.word_conservation.v0","cases":index,
        }))
        .unwrap(),
    )
    .unwrap();
}
