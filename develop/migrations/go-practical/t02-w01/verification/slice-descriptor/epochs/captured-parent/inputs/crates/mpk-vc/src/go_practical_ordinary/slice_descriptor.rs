//! Frozen six-field slice metadata, with every path/index bit retained.
//! Header facts do not establish typed storage, permissions or source binding.
use super::syntax::*;
use super::{word, Builder, Result};

mod guards;
mod validity;
const PREFIX: &str = "Mpk.Go.Practical.Slice.Descriptor";
const PATH: [&str; 4] = ["p0", "p1", "p2", "p3"];
const NAMES: [&str; 6] = ["nil", "owner", "path", "offset", "length", "capacity"];
const GETTERS: [&str; 6] = ["NilTag", "Owner", "Path", "Offset", "Length", "Capacity"];
fn dc(name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("{PREFIX}.{name}"), args)
}
fn pc(name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("Mpk.Go.Practical.Product.{name}"), args)
}
fn word_type(width: usize) -> Expr {
    word::word_type(width).expect("frozen descriptor width")
}
fn cube(names: &[&str], mut e: Expr, lambda: bool) -> Expr {
    for name in names.iter().rev() {
        e = if lambda {
            lam(name, boolean(), e)
        } else {
            pi(name, boolean(), e)
        };
    }
    e
}
fn zero_word(width: usize) -> Expr {
    cube(
        &word::ADDRESSES[..word::depth(width).expect("frozen width")],
        bit(false),
        true,
    )
}
fn product(types: &[Expr]) -> Expr {
    if types.len() == 1 {
        return types[0].clone();
    }
    let mid = types.len() / 2;
    pi(
        "_tag",
        boolean(),
        call(
            "Std.Bool.recType",
            vec![product(&types[..mid]), product(&types[mid..]), var("_tag")],
        ),
    )
}
fn make_product(types: &[Expr], values: &[Expr]) -> Expr {
    if types.len() == 1 {
        return values[0].clone();
    }
    let mid = types.len() / 2;
    pc(
        "Make",
        vec![
            product(&types[..mid]),
            product(&types[mid..]),
            make_product(&types[..mid], &values[..mid]),
            make_product(&types[mid..], &values[mid..]),
        ],
    )
}
fn project(types: &[Expr], value: Expr, at: usize) -> Expr {
    if types.len() == 1 {
        return value;
    }
    let mid = types.len() / 2;
    let selected = pc(
        if at < mid { "Left" } else { "Right" },
        vec![product(&types[..mid]), product(&types[mid..]), value],
    );
    if at < mid {
        project(&types[..mid], selected, at)
    } else {
        project(&types[mid..], selected, at - mid)
    }
}
fn path_cell() -> Expr {
    product(&[word_type(8), word_type(64)])
}
fn path_type() -> Expr {
    cube(&PATH, path_cell(), false)
}
fn path_zero() -> Expr {
    dc("PathZero", vec![])
}
fn fields() -> Vec<Expr> {
    vec![
        boolean(),
        nat(),
        path_type(),
        word_type(64),
        word_type(64),
        word_type(64),
    ]
}
fn descriptor() -> Expr {
    product(&fields())
}
fn constructor(values: Vec<Expr>) -> Expr {
    dc("Make", values)
}
fn getter(at: usize, s: Expr) -> Expr {
    dc(GETTERS[at], vec![s])
}
fn arguments() -> Vec<Binder> {
    NAMES.into_iter().zip(fields()).collect()
}
fn values() -> Vec<Expr> {
    NAMES.iter().map(|n| var(n)).collect()
}
fn zero_values(nil: bool) -> Vec<Expr> {
    vec![
        bit(nil),
        zero(),
        path_zero(),
        zero_word(64),
        zero_word(64),
        zero_word(64),
    ]
}
fn same_fields(s: Expr, t: Expr) -> Expr {
    dc("SameFields", vec![s, t])
}
fn field_equalities(s: Expr, t: Expr) -> Vec<Expr> {
    fields()
        .into_iter()
        .enumerate()
        .map(|(i, ty)| eq(ty, getter(i, s.clone()), getter(i, t.clone())))
        .collect()
}
fn value(w: Expr) -> Expr {
    call("Mpk.Go.Practical.Word.Numeric.Value6", vec![w])
}
fn number(at: usize, s: Expr) -> Expr {
    value(getter(at, s))
}
fn truth(e: Expr, yes: bool) -> Expr {
    eq(boolean(), e, bit(yes))
}
struct Descriptors<'a>(&'a mut Builder);
impl Descriptors<'_> {
    fn emit(&mut self, name: &str, args: &[Binder], ty: Expr, body: Expr, law: bool) -> Result<()> {
        emit(self.0, &format!("{PREFIX}.{name}"), args, ty, body, law)
    }
    fn theorem(&mut self, name: &str, args: &[Binder], e: Equality) -> Result<()> {
        self.emit(name, args, eq(e.ty, e.left, e.right), e.proof, true)
    }
    fn definitions(&mut self) -> Result<()> {
        self.emit("PathCell", &[], Expr::Sort0, path_cell(), false)?;
        self.emit("PathType", &[], Expr::Sort0, path_type(), false)?;
        let z = make_product(
            &[word_type(8), word_type(64)],
            &[zero_word(8), zero_word(64)],
        );
        self.emit("PathZero", &[], path_type(), cube(&PATH, z, true), false)?;
        let mut path_args = vec![("path", path_type())];
        path_args.extend(PATH.iter().map(|n| (*n, boolean())));
        let p = app(var("path"), PATH.iter().map(|n| var(n)).collect());
        for (i, name) in ["PathKind", "PathIndex"].iter().enumerate() {
            let ty = if i == 0 { word_type(8) } else { word_type(64) };
            self.emit(
                name,
                &path_args,
                ty.clone(),
                project(&[word_type(8), word_type(64)], p.clone(), i),
                false,
            )?;
            let zero = if i == 0 { zero_word(8) } else { zero_word(64) };
            self.theorem(
                &format!("{name}Zero"),
                &path_args[1..],
                Equality {
                    ty,
                    left: dc(name, {
                        let mut a = vec![path_zero()];
                        a.extend(PATH.iter().map(|n| var(n)));
                        a
                    }),
                    right: zero.clone(),
                    proof: refl(if i == 0 { word_type(8) } else { word_type(64) }, zero),
                },
            )?;
        }
        self.emit("Type", &[], Expr::Sort0, descriptor(), false)?;
        let args = arguments();
        let made = constructor(values());
        self.emit(
            "Make",
            &args,
            descriptor(),
            make_product(&fields(), &values()),
            false,
        )?;
        for (i, ty) in fields().into_iter().enumerate() {
            self.emit(
                GETTERS[i],
                &[("s", descriptor())],
                ty.clone(),
                project(&fields(), var("s"), i),
                false,
            )?;
            self.theorem(
                &format!("{}OfMake", GETTERS[i]),
                &args,
                Equality {
                    ty: ty.clone(),
                    left: getter(i, made.clone()),
                    right: var(NAMES[i]),
                    proof: refl(ty, var(NAMES[i])),
                },
            )?;
        }
        self.emit(
            "Copy",
            &[("s", descriptor())],
            descriptor(),
            var("s"),
            false,
        )?;
        self.theorem(
            "CopyIdentity",
            &[("s", descriptor())],
            Equality {
                ty: descriptor(),
                left: dc("Copy", vec![var("s")]),
                right: var("s"),
                proof: refl(descriptor(), var("s")),
            },
        )?;
        for (name, nil) in [("Nil", true), ("EmptyNonNilRaw", false)] {
            self.emit(
                name,
                &[],
                descriptor(),
                constructor(zero_values(nil)),
                false,
            )?;
            for (i, ty) in fields().into_iter().enumerate() {
                let zero = zero_values(nil)[i].clone();
                self.theorem(
                    &format!("{name}{}", GETTERS[i]),
                    &[],
                    Equality {
                        ty: ty.clone(),
                        left: getter(i, dc(name, vec![])),
                        right: zero.clone(),
                        proof: refl(ty, zero),
                    },
                )?;
            }
        }
        for (i, name) in [(3, "OffsetValue"), (4, "LengthValue"), (5, "CapacityValue")] {
            self.emit(
                name,
                &[("s", descriptor())],
                nat(),
                number(i, var("s")),
                false,
            )?;
            for ctor in ["Nil", "EmptyNonNilRaw"] {
                self.theorem(
                    &format!("{ctor}{name}"),
                    &[],
                    Equality::natural(
                        dc(name, vec![dc(ctor, vec![])]),
                        zero(),
                        call("Mpk.Go.Practical.String.ZeroCubeValue6", vec![]),
                    ),
                )?;
            }
        }
        let with = vec![
            ("s", descriptor()),
            ("offset", word_type(64)),
            ("length", word_type(64)),
            ("capacity", word_type(64)),
        ];
        let replacement = constructor(vec![
            getter(0, var("s")),
            getter(1, var("s")),
            getter(2, var("s")),
            var("offset"),
            var("length"),
            var("capacity"),
        ]);
        self.emit("WithBounds", &with, descriptor(), replacement, false)?;
        let changed = dc(
            "WithBounds",
            vec![var("s"), var("offset"), var("length"), var("capacity")],
        );
        for (i, ty) in fields().into_iter().enumerate() {
            let result = if i < 3 {
                getter(i, var("s"))
            } else {
                var(NAMES[i])
            };
            self.theorem(
                &format!("WithBounds{}", GETTERS[i]),
                &with,
                Equality {
                    ty: ty.clone(),
                    left: getter(i, changed.clone()),
                    right: result.clone(),
                    proof: refl(ty, result),
                },
            )?;
        }
        Ok(())
    }
    fn relations(&mut self) -> Result<()> {
        let (s, t, u) = (var("s"), var("t"), var("u"));
        let pair = [("s", descriptor()), ("t", descriptor())];
        self.emit(
            "SameFields",
            &pair,
            Expr::Sort0,
            product(&field_equalities(s.clone(), t.clone())),
            false,
        )?;
        let mut witness = pair.to_vec();
        witness.push(("h", same_fields(s.clone(), t.clone())));
        for (i, ty) in fields().into_iter().enumerate() {
            self.theorem(
                &format!("SameFields{}", GETTERS[i]),
                &witness,
                Equality {
                    ty,
                    left: getter(i, s.clone()),
                    right: getter(i, t.clone()),
                    proof: project(&field_equalities(s.clone(), t.clone()), var("h"), i),
                },
            )?;
        }
        let proofs = fields()
            .into_iter()
            .enumerate()
            .map(|(i, ty)| refl(ty, getter(i, s.clone())))
            .collect::<Vec<_>>();
        self.emit(
            "SameFieldsReflexive",
            &pair[..1],
            same_fields(s.clone(), s.clone()),
            make_product(&field_equalities(s.clone(), s.clone()), &proofs),
            true,
        )?;
        let proofs = fields()
            .into_iter()
            .enumerate()
            .map(|(i, ty)| {
                call(
                    "Std.Eq.symm",
                    vec![
                        ty,
                        getter(i, s.clone()),
                        getter(i, t.clone()),
                        dc(
                            &format!("SameFields{}", GETTERS[i]),
                            vec![s.clone(), t.clone(), var("h")],
                        ),
                    ],
                )
            })
            .collect::<Vec<_>>();
        self.emit(
            "SameFieldsSymmetric",
            &witness,
            same_fields(t.clone(), s.clone()),
            make_product(&field_equalities(t.clone(), s.clone()), &proofs),
            true,
        )?;
        let three = [
            ("s", descriptor()),
            ("t", descriptor()),
            ("u", descriptor()),
            ("st", same_fields(s.clone(), t.clone())),
            ("tu", same_fields(t.clone(), u.clone())),
        ];
        let proofs = fields()
            .into_iter()
            .enumerate()
            .map(|(i, ty)| {
                call(
                    "Std.Eq.trans",
                    vec![
                        ty,
                        getter(i, s.clone()),
                        getter(i, t.clone()),
                        getter(i, u.clone()),
                        dc(
                            &format!("SameFields{}", GETTERS[i]),
                            vec![s.clone(), t.clone(), var("st")],
                        ),
                        dc(
                            &format!("SameFields{}", GETTERS[i]),
                            vec![t.clone(), u.clone(), var("tu")],
                        ),
                    ],
                )
            })
            .collect::<Vec<_>>();
        self.emit(
            "SameFieldsTransitive",
            &three,
            same_fields(s.clone(), u.clone()),
            make_product(&field_equalities(s.clone(), u.clone()), &proofs),
            true,
        )?;
        let reconstructed = constructor((0..6).map(|i| getter(i, s.clone())).collect());
        self.emit(
            "Reconstruct",
            &pair[..1],
            descriptor(),
            reconstructed.clone(),
            false,
        )?;
        self.emit(
            "ReconstructFields",
            &pair[..1],
            same_fields(dc("Reconstruct", vec![s.clone()]), s.clone()),
            make_product(
                &field_equalities(reconstructed, s.clone()),
                &proofs_for_same_fields(s),
            ),
            true,
        )
    }
}
fn proofs_for_same_fields(s: Expr) -> Vec<Expr> {
    fields()
        .into_iter()
        .enumerate()
        .map(|(i, ty)| refl(ty, getter(i, s.clone())))
        .collect()
}
pub(super) fn append(builder: &mut Builder) -> Result<()> {
    let mut d = Descriptors(builder);
    d.definitions()?;
    d.relations()?;
    d.guards()?;
    d.validity()
}
