//! Untrusted ordinary Boolean computation and pointwise word proof recipes.
//! Each computed-major rewrite is an actual Eq.congr/trans application.
use super::syntax::*;
use super::word::{at, choose, depth, wc, word_type, Words, ADDRESSES};
use super::{GoOrdinaryError, Result};
use std::collections::BTreeMap;

fn free(node: &Expr, name: &str) -> bool {
    match node {
        Expr::Var(n) => n == name,
        Expr::Apply(f, args) => free(f, name) || args.iter().any(|a| free(a, name)),
        Expr::Lambda(n, ty, body) | Expr::Pi(n, ty, body) => {
            free(ty, name) || (n != name && free(body, name))
        }
        _ => false,
    }
}
fn occurs(node: &Expr, name: &str) -> bool {
    match node {
        Expr::Var(n) => n == name,
        Expr::Apply(f, args) => occurs(f, name) || args.iter().any(|a| occurs(a, name)),
        Expr::Lambda(n, ty, body) | Expr::Pi(n, ty, body) => {
            n == name || occurs(ty, name) || occurs(body, name)
        }
        _ => false,
    }
}

pub(super) struct Evaluation<'a> {
    definitions: &'a BTreeMap<String, Expr>,
    fresh: usize,
    steps: usize,
}
impl<'a> Evaluation<'a> {
    pub fn new(definitions: &'a BTreeMap<String, Expr>) -> Self {
        Self {
            definitions,
            fresh: 0,
            steps: 0,
        }
    }
    fn tick(&mut self) -> Result<()> {
        self.steps += 1;
        if self.steps > 2_000_000 {
            Err(GoOrdinaryError::Limit)
        } else {
            Ok(())
        }
    }
    fn name(&mut self, avoid: &[&Expr]) -> Result<String> {
        loop {
            self.tick()?;
            let name = format!("_word_evaluation{}", self.fresh);
            self.fresh += 1;
            if !avoid.iter().any(|e| occurs(e, &name)) {
                return Ok(name);
            }
        }
    }
    fn substitute(&mut self, node: Expr, name: &str, value: &Expr) -> Result<Expr> {
        self.tick()?;
        let is_lambda = matches!(&node, Expr::Lambda(..));
        Ok(match node {
            Expr::Var(n) if n == name => value.clone(),
            Expr::Apply(f, args) => app(
                self.substitute(*f, name, value)?,
                args.into_iter()
                    .map(|a| self.substitute(a, name, value))
                    .collect::<Result<Vec<_>>>()?,
            ),
            Expr::Lambda(n, ty, body) | Expr::Pi(n, ty, body) => {
                let ty = self.substitute(*ty, name, value)?;
                let mut n = n;
                let mut body = *body;
                if n != name {
                    if free(value, &n) {
                        let fresh = self.name(&[&body, value])?;
                        body = self.substitute(body, &n, &var(&fresh))?;
                        n = fresh;
                    }
                    body = self.substitute(body, name, value)?;
                }
                if is_lambda {
                    lam(&n, ty, body)
                } else {
                    pi(&n, ty, body)
                }
            }
            other => other,
        })
    }
    fn spine(mut node: Expr) -> (Expr, Vec<Expr>) {
        let mut tail = vec![];
        while let Expr::Apply(f, mut args) = node {
            args.extend(tail);
            tail = args;
            node = *f;
        }
        (node, tail)
    }
    fn recursor(head: &Expr) -> Option<(usize, usize)> {
        if *head == constant("Std.Bool.rec") {
            Some((2, 0))
        } else if *head == constant("Std.Bool.cases") {
            Some((3, 1))
        } else {
            None
        }
    }
    fn head(&mut self, mut node: Expr) -> Result<Expr> {
        loop {
            self.tick()?;
            let (head, args) = Self::spine(node);
            if let Expr::Constant(name) = &head {
                if let Some(body) = self.definitions.get(name) {
                    node = app(body.clone(), args);
                    continue;
                }
            }
            if let Expr::Lambda(name, _, body) = &head {
                if !args.is_empty() {
                    node = app(
                        self.substitute(*body.clone(), name, &args[0])?,
                        args[1..].to_vec(),
                    );
                    continue;
                }
            }
            if let Some((major, offset)) = Self::recursor(&head) {
                if args.len() > major && (args[major] == bit(false) || args[major] == bit(true)) {
                    let index = offset + usize::from(args[major] == bit(true));
                    node = app(args[index].clone(), args[major + 1..].to_vec());
                    continue;
                }
            }
            return Ok(app(head, args));
        }
    }
    fn normalize(&mut self, node: Expr) -> Result<Equality> {
        self.tick()?;
        let h = self.head(node.clone())?;
        let (head, args) = Self::spine(h.clone());
        if let Some((major, _)) = Self::recursor(&head) {
            if args.len() > major {
                let original = args[major].clone();
                let e = self.normalize(original.clone())?;
                if e.right == bit(false) || e.right == bit(true) {
                    let name = self.name(&[&h])?;
                    let replace = |value: Expr| {
                        let mut args = args.clone();
                        args[major] = value;
                        app(head.clone(), args)
                    };
                    let changed = replace(e.right.clone());
                    let first = e.mapped(
                        lam(&name, boolean(), replace(var(&name))),
                        node,
                        changed.clone(),
                        boolean(),
                    );
                    return first.then(self.normalize(changed)?);
                }
            }
        }
        Ok(Equality::boolean(node, h.clone(), refl(boolean(), h)))
    }
    pub fn equation(&mut self, left: Expr, right: Expr) -> Result<Equality> {
        let a = self.normalize(left)?;
        let b = self.normalize(right)?;
        a.then(b.reverse())
    }
}

fn boolean_cases<F>(
    names: &[&'static str],
    environment: &BTreeMap<&'static str, Expr>,
    proposition: &F,
    leaf: &mut impl FnMut(&BTreeMap<&'static str, Expr>) -> Result<Expr>,
) -> Result<Expr>
where
    F: Fn(&BTreeMap<&'static str, Expr>) -> Expr,
{
    let Some((&name, tail)) = names.split_first() else {
        return leaf(environment);
    };
    let mut scope = environment.clone();
    for &n in names {
        scope.insert(n, var(n));
    }
    let mut result = proposition(&scope);
    for n in tail.iter().rev() {
        result = pi(n, boolean(), result);
    }
    let motive = lam(name, boolean(), result);
    let mut no = environment.clone();
    no.insert(name, bit(false));
    let no = boolean_cases(tail, &no, proposition, leaf)?;
    let mut yes = environment.clone();
    yes.insert(name, bit(true));
    let yes = boolean_cases(tail, &yes, proposition, leaf)?;
    Ok(lam(
        name,
        boolean(),
        call("Std.Bool.cases", vec![motive, no, yes, var(name)]),
    ))
}

pub(super) fn boolean_laws(words: &mut Words<'_>) -> Result<()> {
    for name in ["SumAssociative", "CarryMajority", "CarryZero", "SumZero"] {
        let names: &[&'static str] = if matches!(name, "CarryZero" | "SumZero") {
            &["a"]
        } else {
            &["a", "b", "c"]
        };
        let endpoints = |e: &BTreeMap<&'static str, Expr>| {
            let a = e["a"].clone();
            let b = e.get("b").cloned().unwrap_or_else(|| bit(false));
            let c = e.get("c").cloned().unwrap_or_else(|| bit(false));
            match name {
                "SumAssociative" => (
                    bc("Sum", vec![a.clone(), b.clone(), c.clone()]),
                    bc("Xor", vec![a, bc("Xor", vec![b, c])]),
                ),
                "CarryMajority" => (
                    bc("Carry", vec![a.clone(), b.clone(), c.clone()]),
                    choose(
                        a,
                        bc("Or", vec![b.clone(), c.clone()]),
                        bc("And", vec![b, c]),
                        boolean(),
                    ),
                ),
                "CarryZero" => (bc("Carry", vec![a, b, c]), bit(false)),
                _ => (bc("Sum", vec![a.clone(), b, c]), a),
            }
        };
        let proposition = |e: &BTreeMap<&'static str, Expr>| {
            let (a, b) = endpoints(e);
            eq(boolean(), a, b)
        };
        let mut evaluator = Evaluation::new(&words.definitions);
        let mut leaf = |e: &BTreeMap<&'static str, Expr>| {
            let (a, b) = endpoints(e);
            Ok(evaluator.equation(a, b)?.proof)
        };
        let proof = boolean_cases(names, &BTreeMap::new(), &proposition, &mut leaf)?;
        let scope = names.iter().map(|&name| (name, var(name))).collect();
        let mut ty = proposition(&scope);
        for name in names.iter().rev() {
            ty = pi(name, boolean(), ty);
        }
        words.emit(
            &format!("Mpk.Go.Practical.Bool.{name}"),
            &[],
            ty,
            proof,
            true,
        )?;
    }
    Ok(())
}

pub(super) fn word_laws(words: &mut Words<'_>, width: usize) -> Result<()> {
    let ty = word_type(width)?;
    let x = var("x");
    let zero = wc(width, "Zero", vec![]);
    for i in 0..=width {
        let current = wc(
            width,
            &format!("Carry{i}"),
            vec![x.clone(), zero.clone(), bit(false)],
        );
        let proof = if i == 0 {
            refl(boolean(), bit(false))
        } else {
            let previous = wc(
                width,
                &format!("Carry{}", i - 1),
                vec![x.clone(), zero.clone(), bit(false)],
            );
            let a = at(x.clone(), width, i - 1)?;
            let z = at(zero.clone(), width, i - 1)?;
            let middle = bc("Carry", vec![a.clone(), z.clone(), bit(false)]);
            let prior = Equality::boolean(
                previous,
                bit(false),
                wc(width, &format!("CarryZero{}", i - 1), vec![x.clone()]),
            );
            let first = prior.mapped(
                lam("_carry", boolean(), bc("Carry", vec![a, z, var("_carry")])),
                current.clone(),
                middle.clone(),
                boolean(),
            );
            let tail = Evaluation::new(&words.definitions).equation(middle, bit(false))?;
            first.then(tail)?.proof
        };
        words.local(
            width,
            &format!("CarryZero{i}"),
            &[("x", ty.clone())],
            eq(boolean(), current, bit(false)),
            proof,
            true,
        )?;
    }
    let names = &ADDRESSES[..depth(width)?];
    for name in ["AddZeroAt", "XorZeroAt", "AndOnesAt"] {
        let endpoints = |e: &BTreeMap<&'static str, Expr>| {
            let av = names.iter().map(|n| e[*n].clone()).collect::<Vec<_>>();
            let op = match name {
                "AddZeroAt" => "Add",
                "XorZeroAt" => "Xor",
                _ => "And",
            };
            let other = if name == "AndOnesAt" {
                wc(width, "Ones", vec![])
            } else {
                zero.clone()
            };
            (
                app(wc(width, op, vec![x.clone(), other]), av.clone()),
                app(x.clone(), av),
            )
        };
        let proposition = |e: &BTreeMap<&'static str, Expr>| {
            let (a, b) = endpoints(e);
            eq(boolean(), a, b)
        };
        let mut evaluator = Evaluation::new(&words.definitions);
        let mut leaf = |e: &BTreeMap<&'static str, Expr>| -> Result<Expr> {
            let (left, right) = endpoints(e);
            if name != "AddZeroAt" {
                return Ok(evaluator.equation(left, right)?.proof);
            }
            let index = names.iter().enumerate().fold(0, |n, (i, name)| {
                n + if e[*name] == bit(true) {
                    1 << (names.len() - 1 - i)
                } else {
                    0
                }
            });
            let a = at(x.clone(), width, index)?;
            let z = at(zero.clone(), width, index)?;
            let carry = wc(
                width,
                &format!("Carry{index}"),
                vec![x.clone(), zero.clone(), bit(false)],
            );
            let middle = bc("Sum", vec![a.clone(), z.clone(), bit(false)]);
            let h = Equality::boolean(
                carry,
                bit(false),
                wc(width, &format!("CarryZero{index}"), vec![x.clone()]),
            );
            let first = h.mapped(
                lam("_carry", boolean(), bc("Sum", vec![a, z, var("_carry")])),
                left,
                middle.clone(),
                boolean(),
            );
            Ok(first.then(evaluator.equation(middle, right)?)?.proof)
        };
        let proof = boolean_cases(names, &BTreeMap::new(), &proposition, &mut leaf)?;
        let scope = names.iter().map(|&name| (name, var(name))).collect();
        let mut result = proposition(&scope);
        for n in names.iter().rev() {
            result = pi(n, boolean(), result);
        }
        words.local(width, name, &[("x", ty.clone())], result, proof, true)?;
    }
    let av = names.iter().map(|n| var(n)).collect::<Vec<_>>();
    let value = app(x.clone(), av.clone());
    let left = app(
        wc(width, "Not", vec![wc(width, "Not", vec![x.clone()])]),
        av,
    );
    let mut args = vec![("x", ty)];
    args.extend(names.iter().map(|&n| (n, boolean())));
    words.local(
        width,
        "NotNotAt",
        &args,
        eq(boolean(), left, value.clone()),
        bc("DoubleNot", vec![value]),
        true,
    )
}
