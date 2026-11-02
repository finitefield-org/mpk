//! Exact frozen balanced [Word64, every payload] layout and checked adapters.
//! Payload types/zeros are parameters, not source or canonical-zero authority.
use super::syntax::*;
use super::{word, Builder, GoOrdinaryError, Result, MAX_DECLARATIONS};
use std::collections::BTreeMap;

fn pc(name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("Mpk.Go.Practical.Product.{name}"), args)
}
fn ec(name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("Mpk.Go.Practical.ErrorValue.{name}"), args)
}
pub(super) fn product(types: &[Expr]) -> Expr {
    match types {
        [] => boolean(),
        [one] => one.clone(),
        _ => {
            let m = types.len() / 2;
            pi(
                "_layout_tag",
                boolean(),
                call(
                    "Std.Bool.recType",
                    vec![
                        product(&types[..m]),
                        product(&types[m..]),
                        var("_layout_tag"),
                    ],
                ),
            )
        }
    }
}
pub(super) fn make(types: &[Expr], values: &[Expr]) -> Expr {
    match values {
        [] => bit(false),
        [one] => one.clone(),
        _ => {
            let m = types.len() / 2;
            let ty = call(
                "Std.Bool.recType",
                vec![
                    product(&types[..m]),
                    product(&types[m..]),
                    var("_layout_tag"),
                ],
            );
            lam(
                "_layout_tag",
                boolean(),
                call(
                    "Std.Bool.cases",
                    vec![
                        lam("_layout_tag", boolean(), ty),
                        make(&types[..m], &values[..m]),
                        make(&types[m..], &values[m..]),
                        var("_layout_tag"),
                    ],
                ),
            )
        }
    }
}
fn get(types: &[Expr], value: Expr, index: usize) -> Expr {
    if types.len() == 1 {
        return value;
    }
    let m = types.len() / 2;
    let v = pc(
        if index < m { "Left" } else { "Right" },
        vec![product(&types[..m]), product(&types[m..]), value],
    );
    if index < m {
        get(&types[..m], v, index)
    } else {
        get(&types[m..], v, index - m)
    }
}
fn zero_word() -> Expr {
    let mut value = bit(false);
    for address in word::ADDRESSES.iter().rev() {
        value = lam(address, boolean(), value);
    }
    value
}

struct Layout<'a> {
    builder: &'a mut Builder,
    prefix: String,
    types: Vec<Expr>,
    parameters: Vec<(String, Expr)>,
}
impl Layout<'_> {
    fn lc(&self, name: &str, tail: Vec<Expr>) -> Expr {
        let mut arguments = self
            .parameters
            .iter()
            .map(|(name, _)| var(name))
            .collect::<Vec<_>>();
        arguments.extend(tail);
        call(&format!("{}.{name}", self.prefix), arguments)
    }
    fn emit(
        &mut self,
        name: &str,
        tail: &[(String, Expr)],
        mut ty: Expr,
        mut body: Expr,
        theorem: bool,
    ) -> Result<()> {
        for (name, domain) in self.parameters.iter().chain(tail).rev() {
            ty = pi(name, domain.clone(), ty);
            body = lam(name, domain.clone(), body);
        }
        emit(
            self.builder,
            &format!("{}.{name}", self.prefix),
            &[],
            ty,
            body,
            theorem,
        )
    }
    fn payload(&self) -> Expr {
        product(&self.types[1..])
    }
    fn carrier(&self) -> Expr {
        product(&self.types)
    }
    fn view(&self) -> Expr {
        ec("Carrier", vec![self.payload()])
    }
    fn field(&self, i: usize, value: Expr) -> Expr {
        self.lc(
            &if i == 0 {
                "Tag".into()
            } else {
                format!("Payload{}", i - 1)
            },
            vec![value],
        )
    }
    fn view_field(&self, i: usize, value: Expr) -> Expr {
        if i == 0 {
            ec("Tag", vec![self.payload(), value])
        } else {
            get(
                &self.types[1..],
                ec("Payload", vec![self.payload(), value]),
                i - 1,
            )
        }
    }
    fn definitions(&mut self) -> Result<()> {
        self.emit("Carrier", &[], Expr::Sort0, self.carrier(), false)?;
        self.emit("PayloadCarrier", &[], Expr::Sort0, self.payload(), false)?;
        self.emit("ZeroTag", &[], word::word_type(64)?, zero_word(), false)?;
        let mut values = vec![("tag".into(), word::word_type(64)?)];
        values.extend(
            self.types[1..]
                .iter()
                .enumerate()
                .map(|(i, t)| (format!("v{i}"), t.clone())),
        );
        let arguments = values.iter().map(|(name, _)| var(name)).collect::<Vec<_>>();
        self.emit(
            "Make",
            &values,
            self.carrier(),
            make(&self.types, &arguments),
            false,
        )?;
        let zeros = self.types[1..]
            .iter()
            .enumerate()
            .map(|(i, t)| (format!("z{i}"), t.clone()))
            .collect::<Vec<_>>();
        let nil_values = std::iter::once(zero_word())
            .chain(zeros.iter().map(|(name, _)| var(name)))
            .collect::<Vec<_>>();
        self.emit(
            "Nil",
            &zeros,
            self.carrier(),
            make(&self.types, &nil_values),
            false,
        )?;
        let input = [("e".into(), self.carrier())];
        for i in 0..self.types.len() {
            self.emit(
                &if i == 0 {
                    "Tag".into()
                } else {
                    format!("Payload{}", i - 1)
                },
                &input,
                self.types[i].clone(),
                get(&self.types, var("e"), i),
                false,
            )?;
        }
        let payload_values = (1..self.types.len())
            .map(|i| get(&self.types, var("e"), i))
            .collect::<Vec<_>>();
        self.emit(
            "ToView",
            &input,
            self.view(),
            ec(
                "Make",
                vec![
                    self.payload(),
                    get(&self.types, var("e"), 0),
                    make(&self.types[1..], &payload_values),
                ],
            ),
            false,
        )?;
        let from_values = (0..self.types.len())
            .map(|i| self.view_field(i, var("v")))
            .collect::<Vec<_>>();
        self.emit(
            "FromView",
            &[("v".into(), self.view())],
            self.carrier(),
            make(&self.types, &from_values),
            false,
        )?;
        self.emit(
            "IsNil",
            &input,
            boolean(),
            word::wc(64, "IsZero", vec![self.field(0, var("e"))]),
            false,
        )?;
        self.emit(
            "HasCase",
            &[
                ("case".into(), word::word_type(64)?),
                ("e".into(), self.carrier()),
            ],
            boolean(),
            word::wc(64, "Equal", vec![var("case"), self.field(0, var("e"))]),
            false,
        )
    }
    fn laws(&mut self) -> Result<()> {
        self.emit(
            "ZeroTagDefinition",
            &[],
            eq(
                word::word_type(64)?,
                self.lc("ZeroTag", vec![]),
                zero_word(),
            ),
            refl(word::word_type(64)?, zero_word()),
            true,
        )?;
        let mut values = vec![("tag".into(), word::word_type(64)?)];
        values.extend(
            self.types[1..]
                .iter()
                .enumerate()
                .map(|(i, t)| (format!("v{i}"), t.clone())),
        );
        let args = values.iter().map(|(name, _)| var(name)).collect::<Vec<_>>();
        let made = self.lc("Make", args.clone());
        let zeros = self.types[1..]
            .iter()
            .enumerate()
            .map(|(i, t)| (format!("z{i}"), t.clone()))
            .collect::<Vec<_>>();
        let nil_args = zeros.iter().map(|(name, _)| var(name)).collect::<Vec<_>>();
        let nil = self.lc("Nil", nil_args);
        let nil_values = std::iter::once(zero_word())
            .chain(zeros.iter().map(|(name, _)| var(name)))
            .collect::<Vec<_>>();
        let nil_recipe = make(&self.types, &nil_values);
        self.emit(
            "NilRecipe",
            &zeros,
            eq(self.carrier(), nil.clone(), nil_recipe.clone()),
            refl(self.carrier(), nil_recipe),
            true,
        )?;
        for i in 0..self.types.len() {
            let name = if i == 0 {
                "Tag".into()
            } else {
                format!("Payload{}", i - 1)
            };
            self.emit(
                &format!("{name}OfMake"),
                &values,
                eq(
                    self.types[i].clone(),
                    self.field(i, made.clone()),
                    args[i].clone(),
                ),
                refl(self.types[i].clone(), args[i].clone()),
                true,
            )?;
            let zero = if i == 0 {
                zero_word()
            } else {
                var(&format!("z{}", i - 1))
            };
            self.emit(
                &format!("{name}OfNil"),
                &zeros,
                eq(
                    self.types[i].clone(),
                    self.field(i, nil.clone()),
                    zero.clone(),
                ),
                refl(self.types[i].clone(), zero),
                true,
            )?;
        }
        let input = [("e".into(), self.carrier())];
        let view_input = [("v".into(), self.view())];
        let to = self.lc("ToView", vec![var("e")]);
        let from = self.lc("FromView", vec![var("v")]);
        let round = self.lc("FromView", vec![to.clone()]);
        let view_round = self.lc("ToView", vec![from.clone()]);
        for i in 0..self.types.len() {
            let suffix = if i == 0 {
                "Tag".into()
            } else {
                format!("Payload{}", i - 1)
            };
            let actual = self.field(i, var("e"));
            let observed = self.view_field(i, var("v"));
            for (name, binders, left, right) in [
                (
                    format!("ToView{suffix}"),
                    input.as_slice(),
                    self.view_field(i, to.clone()),
                    actual.clone(),
                ),
                (
                    format!("FromView{suffix}"),
                    view_input.as_slice(),
                    self.field(i, from.clone()),
                    observed.clone(),
                ),
                (
                    format!("RoundTrip{suffix}"),
                    input.as_slice(),
                    self.field(i, round.clone()),
                    actual,
                ),
                (
                    format!("ViewRoundTrip{suffix}"),
                    view_input.as_slice(),
                    self.view_field(i, view_round.clone()),
                    observed,
                ),
            ] {
                self.emit(
                    &name,
                    binders,
                    eq(self.types[i].clone(), left, right.clone()),
                    refl(self.types[i].clone(), right),
                    true,
                )?;
            }
        }
        if self.types.len() == 1 {
            self.emit(
                "EmptyViewPayload",
                &input,
                eq(
                    boolean(),
                    ec("Payload", vec![self.payload(), to]),
                    bit(false),
                ),
                refl(boolean(), bit(false)),
                true,
            )?;
        }
        Ok(())
    }

    fn zero_tag_law(&mut self, recipes: &BTreeMap<String, Expr>) -> Result<()> {
        fn combine(values: &[Equality]) -> Expr {
            if let [one] = values {
                return one.proof.clone();
            }
            let m = values.len() / 2;
            let left = values[..m]
                .iter()
                .map(|e| e.left.clone())
                .collect::<Vec<_>>();
            let right = values[m..]
                .iter()
                .map(|e| e.left.clone())
                .collect::<Vec<_>>();
            call(
                "Mpk.Go.Practical.Product.Relation.AndTrueIntro",
                vec![
                    word::fold_bits("And", &left, true),
                    word::fold_bits("And", &right, true),
                    combine(&values[..m]),
                    combine(&values[m..]),
                ],
            )
        }
        let mut evaluator = super::word_proofs::Evaluation::new(recipes);
        let leaves = (0..64)
            .map(|i| {
                evaluator.equation(
                    bc(
                        "Equal",
                        vec![
                            word::at(zero_word(), 64, i)?,
                            word::at(word::wc(64, "Zero", vec![]), 64, i)?,
                        ],
                    ),
                    bit(true),
                )
            })
            .collect::<Result<Vec<_>>>()?;
        self.emit(
            "FrozenZeroIsZero",
            &[],
            eq(
                boolean(),
                word::wc(64, "IsZero", vec![zero_word()]),
                bit(true),
            ),
            combine(&leaves),
            true,
        )
    }

    fn comparisons(&mut self) -> Result<()> {
        let input = [("e".into(), self.carrier())];
        let view_input = [("v".into(), self.view())];
        for view in [false, true] {
            let binders = if view {
                view_input.as_slice()
            } else {
                input.as_slice()
            };
            let value = if view { var("v") } else { var("e") };
            let tag = if view {
                self.view_field(0, value.clone())
            } else {
                self.field(0, value.clone())
            };
            let mut args = binders.to_vec();
            let head = if view { "View" } else { "" };
            let nil = if view {
                ec("IsNil", vec![self.payload(), value.clone()])
            } else {
                self.lc("IsNil", vec![value.clone()])
            };
            let computed = word::wc(64, "IsZero", vec![tag.clone()]);
            self.emit(
                &format!("{head}IsNilDefinition"),
                &args,
                eq(boolean(), nil, computed.clone()),
                refl(boolean(), computed),
                true,
            )?;
            args.insert(0, ("case".into(), word::word_type(64)?));
            let matched = if view {
                ec("HasCase", vec![self.payload(), var("case"), value])
            } else {
                self.lc("HasCase", vec![var("case"), value])
            };
            let computed = word::wc(64, "Equal", vec![var("case"), tag]);
            self.emit(
                &format!("{head}HasCaseDefinition"),
                &args,
                eq(boolean(), matched, computed.clone()),
                refl(boolean(), computed),
                true,
            )?;
        }
        for from in [false, true] {
            let value = if from { var("v") } else { var("e") };
            let converted = self.lc(
                if from { "FromView" } else { "ToView" },
                vec![value.clone()],
            );
            let (left_tag, right_tag) = if from {
                (
                    self.field(0, converted.clone()),
                    self.view_field(0, value.clone()),
                )
            } else {
                (
                    self.view_field(0, converted.clone()),
                    self.field(0, value.clone()),
                )
            };
            let tag_eq = Equality {
                ty: word::word_type(64)?,
                left: left_tag.clone(),
                right: right_tag.clone(),
                proof: self.lc(
                    if from { "FromViewTag" } else { "ToViewTag" },
                    vec![value.clone()],
                ),
            };
            let binders = if from {
                view_input.as_slice()
            } else {
                input.as_slice()
            };
            for matched in [false, true] {
                let mut args = binders.to_vec();
                if matched {
                    args.insert(0, ("case".into(), word::word_type(64)?));
                }
                let op = if matched { "HasCase" } else { "IsNil" };
                let observe = |view: bool, e: Expr| {
                    if view {
                        ec(
                            op,
                            if matched {
                                vec![self.payload(), var("case"), e]
                            } else {
                                vec![self.payload(), e]
                            },
                        )
                    } else {
                        self.lc(
                            op,
                            if matched {
                                vec![var("case"), e]
                            } else {
                                vec![e]
                            },
                        )
                    }
                };
                let computed = |t: Expr| {
                    word::wc(
                        64,
                        if matched { "Equal" } else { "IsZero" },
                        if matched {
                            vec![var("case"), t]
                        } else {
                            vec![t]
                        },
                    )
                };
                let parameters = |e: Expr| {
                    if matched {
                        vec![var("case"), e]
                    } else {
                        vec![e]
                    }
                };
                let left = observe(!from, converted.clone());
                let right = observe(from, value.clone());
                let first = Equality::boolean(
                    left.clone(),
                    computed(left_tag.clone()),
                    self.lc(
                        &format!("{}{op}Definition", if from { "" } else { "View" }),
                        parameters(converted.clone()),
                    ),
                );
                let middle = tag_eq.clone().mapped(
                    word::wc(
                        64,
                        if matched { "Equal" } else { "IsZero" },
                        if matched { vec![var("case")] } else { vec![] },
                    ),
                    computed(left_tag.clone()),
                    computed(right_tag.clone()),
                    boolean(),
                );
                let last = Equality::boolean(
                    right.clone(),
                    computed(right_tag.clone()),
                    self.lc(
                        &format!("{}{op}Definition", if from { "View" } else { "" }),
                        parameters(value.clone()),
                    ),
                )
                .reverse();
                self.emit(
                    &format!("{}{op}", if from { "FromView" } else { "ToView" }),
                    &args,
                    eq(boolean(), left, right),
                    first.then(middle)?.then(last)?.proof,
                    true,
                )?;
            }
        }
        let zeros = self.types[1..]
            .iter()
            .enumerate()
            .map(|(i, t)| (format!("z{i}"), t.clone()))
            .collect::<Vec<_>>();
        let nil = self.lc("Nil", zeros.iter().map(|(name, _)| var(name)).collect());
        let tag = self.field(0, nil.clone());
        let first = Equality::boolean(
            self.lc("IsNil", vec![nil.clone()]),
            word::wc(64, "IsZero", vec![tag.clone()]),
            self.lc("IsNilDefinition", vec![nil.clone()]),
        );
        let middle = Equality {
            ty: word::word_type(64)?,
            left: tag.clone(),
            right: zero_word(),
            proof: self.lc(
                "TagOfNil",
                zeros.iter().map(|(name, _)| var(name)).collect(),
            ),
        }
        .mapped(
            word::wc(64, "IsZero", vec![]),
            word::wc(64, "IsZero", vec![tag]),
            word::wc(64, "IsZero", vec![zero_word()]),
            boolean(),
        );
        let last = Equality::boolean(
            word::wc(64, "IsZero", vec![zero_word()]),
            bit(true),
            self.lc("FrozenZeroIsZero", vec![]),
        );
        self.emit(
            "NilIsNil",
            &zeros,
            eq(boolean(), self.lc("IsNil", vec![nil]), bit(true)),
            first.then(middle)?.then(last)?.proof,
            true,
        )
    }
}
pub(super) fn append(
    builder: &mut Builder,
    case_count: usize,
    recipes: &BTreeMap<String, Expr>,
) -> Result<()> {
    // Native declaration accounting only; never prune the case universe.
    let added = case_count
        .checked_mul(7)
        .and_then(|n| n.checked_add(if case_count == 0 { 29 } else { 28 }))
        .ok_or(GoOrdinaryError::Limit)?;
    if builder
        .certificate
        .declarations
        .len()
        .checked_add(added)
        .is_none_or(|n| n > MAX_DECLARATIONS)
    {
        return Err(GoOrdinaryError::Limit);
    }
    let parameters = (0..case_count)
        .map(|i| (format!("T{i}"), Expr::Sort0))
        .collect::<Vec<_>>();
    let types = std::iter::once(word::word_type(64)?)
        .chain(parameters.iter().map(|(name, _)| var(name)))
        .collect();
    let mut layout = Layout {
        builder,
        prefix: format!("Mpk.Go.Practical.ErrorLayout.N{case_count}"),
        types,
        parameters,
    };
    layout.definitions()?;
    layout.laws()?;
    layout.zero_tag_law(recipes)?;
    layout.comparisons()
}
