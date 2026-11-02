//! Private named construction syntax. Only compiled-in recipes enter here;
//! certificates contain ordinary de Bruijn terms checked by both verifiers.
use super::{Builder, DeclarationKind, DefinitionReducibility, GoOrdinaryError, Result};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(super) enum Expr {
    Var(&'static str),
    Constant(String),
    Apply(Box<Expr>, Vec<Expr>),
    Lambda(&'static str, Box<Expr>, Box<Expr>),
    Pi(&'static str, Box<Expr>, Box<Expr>),
}

pub(super) type Binder = (&'static str, Expr);

pub(super) fn var(name: &'static str) -> Expr {
    Expr::Var(name)
}
pub(super) fn constant(name: &str) -> Expr {
    Expr::Constant(name.into())
}
pub(super) fn app(function: Expr, arguments: Vec<Expr>) -> Expr {
    if arguments.is_empty() {
        function
    } else {
        Expr::Apply(Box::new(function), arguments)
    }
}
pub(super) fn call(name: &str, arguments: Vec<Expr>) -> Expr {
    app(constant(name), arguments)
}
pub(super) fn nc(name: &str, arguments: Vec<Expr>) -> Expr {
    call(&format!("Mpk.Go.Practical.Nat.{name}"), arguments)
}
pub(super) fn bc(name: &str, arguments: Vec<Expr>) -> Expr {
    call(&format!("Mpk.Go.Practical.Bool.{name}"), arguments)
}
pub(super) fn lam(name: &'static str, ty: Expr, body: Expr) -> Expr {
    Expr::Lambda(name, Box::new(ty), Box::new(body))
}
pub(super) fn pi(name: &'static str, ty: Expr, body: Expr) -> Expr {
    Expr::Pi(name, Box::new(ty), Box::new(body))
}
pub(super) fn nat() -> Expr {
    constant("Std.Nat")
}
pub(super) fn boolean() -> Expr {
    constant("Std.Bool")
}
pub(super) fn bit(value: bool) -> Expr {
    constant(if value {
        "Std.Bool.true"
    } else {
        "Std.Bool.false"
    })
}
pub(super) fn zero() -> Expr {
    constant("Std.Nat.zero")
}
pub(super) fn succ(n: Expr) -> Expr {
    call("Std.Nat.succ", vec![n])
}
pub(super) fn one() -> Expr {
    succ(zero())
}
pub(super) fn add(a: Expr, b: Expr) -> Expr {
    nc("Add", vec![a, b])
}
pub(super) fn mul(a: Expr, b: Expr) -> Expr {
    nc("Mul", vec![a, b])
}
pub(super) fn sub(a: Expr, b: Expr) -> Expr {
    nc("Sub", vec![a, b])
}
pub(super) fn pred(a: Expr) -> Expr {
    nc("Pred", vec![a])
}
pub(super) fn equal(a: Expr, b: Expr) -> Expr {
    nc("Equal", vec![a, b])
}
pub(super) fn less(a: Expr, b: Expr) -> Expr {
    nc("Less", vec![a, b])
}
pub(super) fn eq(ty: Expr, a: Expr, b: Expr) -> Expr {
    call("Std.Eq", vec![ty, a, b])
}
pub(super) fn refl(ty: Expr, value: Expr) -> Expr {
    call("Std.Eq.refl", vec![ty, value])
}
pub(super) fn induct(
    name: &'static str,
    property: Expr,
    base: Expr,
    step: Expr,
    value: Expr,
) -> Expr {
    call(
        "Std.Nat.induct",
        vec![lam(name, nat(), property), base, step, value],
    )
}

fn lower(builder: &mut Builder, expression: &Expr, context: &mut Vec<&'static str>) -> Result<u32> {
    match expression {
        Expr::Constant(name) => builder.constant(name),
        Expr::Var(name) => {
            let index = context
                .iter()
                .rev()
                .position(|n| n == name)
                .ok_or(GoOrdinaryError::Linkage)?;
            builder.var(index as u32)
        }
        Expr::Apply(function, arguments) => {
            let function = lower(builder, function, context)?;
            let arguments = arguments
                .iter()
                .map(|a| lower(builder, a, context))
                .collect::<Result<Vec<_>>>()?;
            builder.app(function, arguments)
        }
        Expr::Lambda(name, ty, body) | Expr::Pi(name, ty, body) => {
            let ty = lower(builder, ty, context)?;
            context.push(name);
            let body = lower(builder, body, context)?;
            context.pop();
            if matches!(expression, Expr::Lambda(..)) {
                builder.lam(ty, body)
            } else {
                builder.pi(ty, body)
            }
        }
    }
}

pub(super) fn emit(
    builder: &mut Builder,
    name: &str,
    args: &[Binder],
    result: Expr,
    body: Expr,
    theorem: bool,
) -> Result<()> {
    let mut ty = result;
    let mut value = body;
    for (name, domain) in args.iter().rev() {
        ty = pi(name, domain.clone(), ty);
        value = lam(name, domain.clone(), value);
    }
    let ty = lower(builder, &ty, &mut vec![])?;
    let value = lower(builder, &value, &mut vec![])?;
    builder.declare(
        name.into(),
        if theorem {
            DeclarationKind::Theorem { ty, proof: value }
        } else {
            DeclarationKind::Def {
                ty,
                value,
                reducibility: DefinitionReducibility::Reducible,
            }
        },
    )
}

pub(super) struct Equality {
    pub ty: Expr,
    pub left: Expr,
    pub right: Expr,
    pub proof: Expr,
}

impl Equality {
    pub fn natural(left: Expr, right: Expr, proof: Expr) -> Self {
        Self {
            ty: nat(),
            left,
            right,
            proof,
        }
    }
    pub fn boolean(left: Expr, right: Expr, proof: Expr) -> Self {
        Self {
            ty: boolean(),
            left,
            right,
            proof,
        }
    }
    pub fn reverse(self) -> Self {
        Self {
            proof: call(
                "Std.Eq.symm",
                vec![
                    self.ty.clone(),
                    self.left.clone(),
                    self.right.clone(),
                    self.proof,
                ],
            ),
            ty: self.ty,
            left: self.right,
            right: self.left,
        }
    }
    pub fn then(self, other: Self) -> Result<Self> {
        if self.ty != other.ty || self.right != other.left {
            return Err(GoOrdinaryError::Linkage);
        }
        Ok(Self {
            proof: call(
                "Std.Eq.trans",
                vec![
                    self.ty.clone(),
                    self.left.clone(),
                    self.right,
                    other.right.clone(),
                    self.proof,
                    other.proof,
                ],
            ),
            ty: self.ty,
            left: self.left,
            right: other.right,
        })
    }
    // Endpoints are explicit recipe syntax. They confer no authority: the
    // ordinary Eq.congr application must typecheck in each independent core.
    pub fn mapped(self, function: Expr, left: Expr, right: Expr, ty: Expr) -> Self {
        Self {
            proof: call(
                "Std.Eq.congr",
                vec![
                    self.ty,
                    ty.clone(),
                    function,
                    self.left,
                    self.right,
                    self.proof,
                ],
            ),
            ty,
            left,
            right,
        }
    }
    pub fn on_succ(self) -> Self {
        let left = succ(self.left.clone());
        let right = succ(self.right.clone());
        self.mapped(constant("Std.Nat.succ"), left, right, nat())
    }
    pub fn on_left(self, b: Expr) -> Self {
        let left = add(self.left.clone(), b.clone());
        let right = add(self.right.clone(), b.clone());
        self.mapped(
            lam("_mapped", nat(), add(var("_mapped"), b)),
            left,
            right,
            nat(),
        )
    }
    pub fn on_right(self, a: Expr) -> Self {
        let left = add(a.clone(), self.left.clone());
        let right = add(a.clone(), self.right.clone());
        self.mapped(
            lam("_mapped", nat(), add(a, var("_mapped"))),
            left,
            right,
            nat(),
        )
    }
}
