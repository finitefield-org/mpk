//! Untrusted proof construction for Nat polynomials. Normalization never
//! supplies a fact: every identity/swap/distribution is an ordinary checked law.
use super::syntax::*;
use super::{Builder, GoOrdinaryError, Result};

pub(super) fn split(expression: &Expr, operation: &str) -> Option<(Expr, Expr)> {
    let Expr::Apply(function, arguments) = expression else {
        return None;
    };
    if **function != constant(&format!("Mpk.Go.Practical.Nat.{operation}")) {
        return None;
    }
    let [a, b] = arguments.as_slice() else {
        return None;
    };
    Some((a.clone(), b.clone()))
}

pub(super) fn mul_left(e: Equality, b: Expr) -> Equality {
    let left = mul(e.left.clone(), b.clone());
    let right = mul(e.right.clone(), b.clone());
    e.mapped(
        lam("_mapped", nat(), mul(var("_mapped"), b)),
        left,
        right,
        nat(),
    )
}
pub(super) fn mul_right(a: Expr, e: Equality) -> Equality {
    let left = mul(a.clone(), e.left.clone());
    let right = mul(a.clone(), e.right.clone());
    e.mapped(
        lam("_mapped", nat(), mul(a, var("_mapped"))),
        left,
        right,
        nat(),
    )
}

pub(super) fn checkpoint(
    builder: &mut Builder,
    args: &[Binder],
    serial: &mut usize,
    e: Equality,
) -> Result<Equality> {
    if e.left == e.right {
        return Ok(e);
    }
    let name = format!("Mpk.Go.Practical.Int.CheckedStep{serial:04}");
    *serial += 1;
    emit(
        builder,
        &name,
        args,
        eq(e.ty.clone(), e.left.clone(), e.right.clone()),
        e.proof,
        true,
    )?;
    let proof = call(&name, args.iter().map(|(name, _)| var(name)).collect());
    Ok(Equality {
        ty: e.ty,
        left: e.left,
        right: e.right,
        proof,
    })
}

#[derive(Clone, Copy)]
enum Monoid {
    Sum,
    Product,
}

impl Monoid {
    fn name(self) -> &'static str {
        match self {
            Self::Sum => "Add",
            Self::Product => "Mul",
        }
    }
    fn identity(self) -> Expr {
        match self {
            Self::Sum => zero(),
            Self::Product => one(),
        }
    }
    fn combine(self, a: Expr, b: Expr) -> Expr {
        nc(self.name(), vec![a, b])
    }
    fn left(self, e: Equality, b: Expr) -> Equality {
        match self {
            Self::Sum => e.on_left(b),
            Self::Product => mul_left(e, b),
        }
    }
    fn right(self, a: Expr, e: Equality) -> Equality {
        match self {
            Self::Sum => e.on_right(a),
            Self::Product => mul_right(a, e),
        }
    }
    fn assoc(self, a: Expr, b: Expr, c: Expr) -> Equality {
        let left = self.combine(self.combine(a.clone(), b.clone()), c.clone());
        let right = self.combine(a.clone(), self.combine(b.clone(), c.clone()));
        Equality::natural(
            left,
            right,
            nc(&format!("{}Assoc", self.name()), vec![a, b, c]),
        )
    }
    fn comm(self, a: Expr, b: Expr) -> Equality {
        let left = self.combine(a.clone(), b.clone());
        let right = self.combine(b.clone(), a.clone());
        Equality::natural(left, right, nc(&format!("{}Comm", self.name()), vec![a, b]))
    }
    fn join(self, a: Expr, b: Expr) -> Result<Equality> {
        let left = self.combine(a.clone(), b.clone());
        if matches!(self, Self::Product) {
            if a == zero() {
                return Ok(Equality::natural(left, zero(), nc("ZeroMul", vec![b])));
            }
            if b == zero() {
                return Ok(Equality::natural(left, zero(), nc("MulZero", vec![a])));
            }
        }
        if a == self.identity() {
            let law = match self {
                Self::Sum => "ZeroAdd",
                Self::Product => "OneMul",
            };
            return Ok(Equality::natural(left, b.clone(), nc(law, vec![b])));
        }
        if b == self.identity() {
            let law = match self {
                Self::Sum => "AddZero",
                Self::Product => "MulOne",
            };
            return Ok(Equality::natural(left, a.clone(), nc(law, vec![a])));
        }
        if let Some((x, y)) = split(&a, self.name()) {
            return self
                .assoc(x.clone(), y.clone(), b.clone())
                .then(self.right(x, self.join(y, b)?));
        }
        Ok(Equality::reflexive(left))
    }
    fn associate(self, node: Expr) -> Result<Equality> {
        let Some((a, b)) = split(&node, self.name()) else {
            return Ok(Equality::reflexive(node));
        };
        let p = self.associate(a)?;
        let q = self.associate(b.clone())?;
        let pa = p.right.clone();
        let qb = q.right.clone();
        self.left(p, b)
            .then(self.right(pa.clone(), q))?
            .then(self.join(pa, qb)?)
    }
    fn atoms(self, node: Expr) -> Vec<Expr> {
        if node == self.identity() {
            return vec![];
        }
        if let Some((a, b)) = split(&node, self.name()) {
            let mut atoms = vec![a];
            atoms.extend(self.atoms(b));
            atoms
        } else {
            vec![node]
        }
    }
    fn value(self, atoms: &[Expr]) -> Expr {
        match atoms {
            [] => self.identity(),
            [a] => a.clone(),
            [a, tail @ ..] => self.combine(a.clone(), self.value(tail)),
        }
    }
    fn swap(self, atoms: &[Expr], index: usize) -> Result<Equality> {
        if index + 1 >= atoms.len() {
            return Err(GoOrdinaryError::Linkage);
        }
        if index > 0 {
            return Ok(self.right(atoms[0].clone(), self.swap(&atoms[1..], index - 1)?));
        }
        let (a, b) = (atoms[0].clone(), atoms[1].clone());
        if atoms.len() == 2 {
            return Ok(self.comm(a, b));
        }
        let tail = self.value(&atoms[2..]);
        self.assoc(a.clone(), b.clone(), tail.clone())
            .reverse()
            .then(self.left(self.comm(a.clone(), b.clone()), tail.clone()))?
            .then(self.assoc(b, a, tail))
    }
}

struct Polynomial<'a> {
    builder: &'a mut Builder,
    args: &'a [Binder],
    serial: &'a mut usize,
}

impl Polynomial<'_> {
    fn save(&mut self, e: Equality) -> Result<Equality> {
        checkpoint(self.builder, self.args, self.serial, e)
    }
    fn monoid(&mut self, kind: Monoid, node: Expr) -> Result<Equality> {
        let mut e = kind.associate(node)?;
        let mut atoms = kind.atoms(e.right.clone());
        for end in (1..atoms.len()).rev() {
            for index in 0..end {
                // This fixed syntax order is only a deterministic proof-search
                // strategy. The core checks the actual commutativity proof.
                if atoms[index] > atoms[index + 1] {
                    e = self.save(e.then(kind.swap(&atoms, index)?)?)?;
                    atoms.swap(index, index + 1);
                }
            }
        }
        Ok(e)
    }
    fn distribute(&mut self, a: Expr, b: Expr) -> Result<Equality> {
        if let Some((x, y)) = split(&a, "Add") {
            let e = Equality::natural(
                mul(a, b.clone()),
                add(mul(x.clone(), b.clone()), mul(y.clone(), b.clone())),
                nc("MulAddLeft", vec![x.clone(), y.clone(), b.clone()]),
            );
            let p = self.distribute(x, b.clone())?;
            let q = self.distribute(y.clone(), b.clone())?;
            let pa = p.right.clone();
            let e = e.then(p.on_left(mul(y, b)))?.then(q.on_right(pa))?;
            let tail = self.monoid(Monoid::Sum, e.right.clone())?;
            return self.save(e.then(tail)?);
        }
        if let Some((x, y)) = split(&b, "Add") {
            let e = Equality::natural(
                mul(a.clone(), b),
                add(mul(a.clone(), x.clone()), mul(a.clone(), y.clone())),
                nc("MulAddRight", vec![a.clone(), x.clone(), y.clone()]),
            );
            let p = self.distribute(a.clone(), x)?;
            let q = self.distribute(a.clone(), y.clone())?;
            let pa = p.right.clone();
            let e = e.then(p.on_left(mul(a, y)))?.then(q.on_right(pa))?;
            let tail = self.monoid(Monoid::Sum, e.right.clone())?;
            return self.save(e.then(tail)?);
        }
        self.monoid(Monoid::Product, mul(a, b))
    }
    fn normalize(&mut self, node: Expr) -> Result<Equality> {
        for kind in [Monoid::Sum, Monoid::Product] {
            if let Some((a, b)) = split(&node, kind.name()) {
                let p = self.normalize(a)?;
                let q = self.normalize(b.clone())?;
                let pa = p.right.clone();
                let qb = q.right.clone();
                let e = kind.left(p, b).then(kind.right(pa.clone(), q))?;
                let tail = match kind {
                    Monoid::Sum => self.monoid(kind, e.right.clone())?,
                    Monoid::Product => self.distribute(pa, qb)?,
                };
                return self.save(e.then(tail)?);
            }
        }
        Ok(Equality::reflexive(node))
    }
}

pub(super) fn prove(
    builder: &mut Builder,
    args: &[Binder],
    serial: &mut usize,
    left: Expr,
    right: Expr,
) -> Result<Equality> {
    let mut normalizer = Polynomial {
        builder,
        args,
        serial,
    };
    let a = normalizer.normalize(left)?;
    let b = normalizer.normalize(right)?;
    // A failed search produces no claim. Even a matching normal form yields
    // only the explicit ordinary proof composed here and checked downstream.
    let e = a.then(b.reverse())?;
    normalizer.save(e)
}
