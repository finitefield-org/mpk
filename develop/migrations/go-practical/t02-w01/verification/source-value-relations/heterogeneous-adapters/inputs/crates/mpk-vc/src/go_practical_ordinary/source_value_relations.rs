//! Closed whole-carrier observations derived from every original source type.
//! These relations preserve padding, inactive fields and complete descriptors.
//! They are neither Go's comparison operator nor source/runtime validity.
use super::syntax::*;
use super::{
    composition, error_layout, source_error_operations, Builder, GoOrdinaryError, Result, MAX_BYTES,
};
use crate::go_practical_type_keys::{GoSourceTypeKey, GoSourceValueDomain};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone)]
enum View {
    Equality(Expr),
    Vector(usize, Box<View>),
    Product(Vec<View>),
    Source(String, Expr),
}
impl View {
    fn carrier(&self) -> Expr {
        match self {
            Self::Equality(t) | Self::Source(_, t) => t.clone(),
            Self::Vector(bits, element) => {
                let mut ty = element.carrier();
                for i in (0..*bits).rev() {
                    ty = pi(&format!("_value_address{i}"), boolean(), ty);
                }
                ty
            }
            Self::Product(parts) => {
                error_layout::product(&parts.iter().map(Self::carrier).collect::<Vec<_>>())
            }
        }
    }
    fn source(module: &str, id: &str, name: &str, args: Vec<Expr>) -> Expr {
        call(&format!("{module}.T{id}.{name}"), args)
    }
    fn indexed(value: Expr, bits: usize, depth: usize) -> Expr {
        app(
            value,
            (0..bits)
                .map(|i| var(&format!("_value_address{depth}_{i}")))
                .collect(),
        )
    }
    fn quantify(bits: usize, depth: usize, mut value: Expr, proof: bool) -> Expr {
        for i in (0..bits).rev() {
            let n = format!("_value_address{depth}_{i}");
            value = if proof {
                lam(&n, boolean(), value)
            } else {
                pi(&n, boolean(), value)
            };
        }
        value
    }
    fn fields(parts: &[Self], value: Expr) -> Vec<Expr> {
        let types = parts.iter().map(Self::carrier).collect::<Vec<_>>();
        (0..parts.len())
            .map(|i| error_layout::get(&types, value.clone(), i))
            .collect()
    }
    fn relation(&self, module: &str, x: Expr, y: Expr) -> Expr {
        self.relation_at(module, x, y, 0)
    }
    fn relation_at(&self, module: &str, x: Expr, y: Expr, depth: usize) -> Expr {
        match self {
            Self::Equality(t) => eq(t.clone(), x, y),
            Self::Source(id, _) => Self::source(module, id, "Related", vec![x, y]),
            Self::Vector(bits, element) => Self::quantify(
                *bits,
                depth,
                element.relation_at(
                    module,
                    Self::indexed(x, *bits, depth),
                    Self::indexed(y, *bits, depth),
                    depth + 1,
                ),
                false,
            ),
            Self::Product(parts) => {
                if parts.is_empty() {
                    return eq(boolean(), x, y);
                }
                let xs = Self::fields(parts, x);
                let ys = Self::fields(parts, y);
                error_layout::product(
                    &parts
                        .iter()
                        .enumerate()
                        .map(|(i, p)| p.relation_at(module, xs[i].clone(), ys[i].clone(), depth))
                        .collect::<Vec<_>>(),
                )
            }
        }
    }
    fn reflexive(&self, module: &str, x: Expr) -> Expr {
        self.reflexive_at(module, x, 0)
    }
    fn reflexive_at(&self, module: &str, x: Expr, depth: usize) -> Expr {
        match self {
            Self::Equality(t) => refl(t.clone(), x),
            Self::Source(id, _) => Self::source(module, id, "Reflexive", vec![x]),
            Self::Vector(bits, element) => Self::quantify(
                *bits,
                depth,
                element.reflexive_at(module, Self::indexed(x, *bits, depth), depth + 1),
                true,
            ),
            Self::Product(parts) => {
                if parts.is_empty() {
                    return refl(boolean(), x);
                }
                let xs = Self::fields(parts, x);
                let types = parts
                    .iter()
                    .enumerate()
                    .map(|(i, p)| p.relation_at(module, xs[i].clone(), xs[i].clone(), depth))
                    .collect::<Vec<_>>();
                let proofs = parts
                    .iter()
                    .enumerate()
                    .map(|(i, p)| p.reflexive_at(module, xs[i].clone(), depth))
                    .collect::<Vec<_>>();
                error_layout::make(&types, &proofs)
            }
        }
    }
    fn symmetric(&self, module: &str, x: Expr, y: Expr, same: Expr) -> Expr {
        self.symmetric_at(module, x, y, same, 0)
    }
    fn symmetric_at(&self, module: &str, x: Expr, y: Expr, same: Expr, depth: usize) -> Expr {
        match self {
            Self::Equality(t) => call("Std.Eq.symm", vec![t.clone(), x, y, same]),
            Self::Source(id, _) => Self::source(module, id, "Symmetric", vec![x, y, same]),
            Self::Vector(bits, element) => Self::quantify(
                *bits,
                depth,
                element.symmetric_at(
                    module,
                    Self::indexed(x, *bits, depth),
                    Self::indexed(y, *bits, depth),
                    Self::indexed(same, *bits, depth),
                    depth + 1,
                ),
                true,
            ),
            Self::Product(parts) => {
                if parts.is_empty() {
                    return call("Std.Eq.symm", vec![boolean(), x, y, same]);
                }
                let xs = Self::fields(parts, x);
                let ys = Self::fields(parts, y);
                let before = parts
                    .iter()
                    .enumerate()
                    .map(|(i, p)| p.relation_at(module, xs[i].clone(), ys[i].clone(), depth))
                    .collect::<Vec<_>>();
                let after = parts
                    .iter()
                    .enumerate()
                    .map(|(i, p)| p.relation_at(module, ys[i].clone(), xs[i].clone(), depth))
                    .collect::<Vec<_>>();
                let proofs = parts
                    .iter()
                    .enumerate()
                    .map(|(i, p)| {
                        p.symmetric_at(
                            module,
                            xs[i].clone(),
                            ys[i].clone(),
                            error_layout::get(&before, same.clone(), i),
                            depth,
                        )
                    })
                    .collect::<Vec<_>>();
                error_layout::make(&after, &proofs)
            }
        }
    }
    fn transitive(&self, module: &str, x: Expr, y: Expr, z: Expr, xy: Expr, yz: Expr) -> Expr {
        self.transitive_at(module, x, y, z, (xy, yz), 0)
    }
    fn transitive_at(
        &self,
        module: &str,
        x: Expr,
        y: Expr,
        z: Expr,
        same: (Expr, Expr),
        depth: usize,
    ) -> Expr {
        let (xy, yz) = same;
        match self {
            Self::Equality(t) => call("Std.Eq.trans", vec![t.clone(), x, y, z, xy, yz]),
            Self::Source(id, _) => Self::source(module, id, "Transitive", vec![x, y, z, xy, yz]),
            Self::Vector(bits, element) => Self::quantify(
                *bits,
                depth,
                element.transitive_at(
                    module,
                    Self::indexed(x, *bits, depth),
                    Self::indexed(y, *bits, depth),
                    Self::indexed(z, *bits, depth),
                    (
                        Self::indexed(xy, *bits, depth),
                        Self::indexed(yz, *bits, depth),
                    ),
                    depth + 1,
                ),
                true,
            ),
            Self::Product(parts) => {
                if parts.is_empty() {
                    return call("Std.Eq.trans", vec![boolean(), x, y, z, xy, yz]);
                }
                let xs = Self::fields(parts, x);
                let ys = Self::fields(parts, y);
                let zs = Self::fields(parts, z);
                let rel = |a: &[Expr], b: &[Expr]| {
                    parts
                        .iter()
                        .enumerate()
                        .map(|(i, p)| p.relation_at(module, a[i].clone(), b[i].clone(), depth))
                        .collect::<Vec<_>>()
                };
                let (a, b, c) = (rel(&xs, &ys), rel(&ys, &zs), rel(&xs, &zs));
                let proofs = parts
                    .iter()
                    .enumerate()
                    .map(|(i, p)| {
                        p.transitive_at(
                            module,
                            xs[i].clone(),
                            ys[i].clone(),
                            zs[i].clone(),
                            (
                                error_layout::get(&a, xy.clone(), i),
                                error_layout::get(&b, yz.clone(), i),
                            ),
                            depth,
                        )
                    })
                    .collect::<Vec<_>>();
                error_layout::make(&c, &proofs)
            }
        }
    }
    fn from_equality(&self, module: &str, x: Expr, y: Expr, same: Expr) -> Expr {
        if let Self::Source(id, _) = self {
            return Self::source(module, id, "OfEquality", vec![x, y, same]);
        }
        call(
            "Std.Eq.rewrite",
            vec![
                self.carrier(),
                x.clone(),
                y,
                lam(
                    "_next_value",
                    self.carrier(),
                    self.relation(module, x.clone(), var("_next_value")),
                ),
                same,
                self.reflexive(module, x),
            ],
        )
    }
}

fn source_carrier(source: &str, id: &str) -> Expr {
    constant(&format!("{source}.T{id}.Carrier"))
}
fn word(width: usize) -> View {
    View::Vector(width.ilog2() as usize, Box::new(View::Equality(boolean())))
}
fn source_view(source: &str, id: &str) -> View {
    View::Source(id.into(), source_carrier(source, id))
}
fn shape(
    source: &str,
    key: &GoSourceTypeKey,
    underlying: &BTreeMap<String, String>,
    cases: &[(String, u64)],
) -> Result<View> {
    Ok(match key {
        GoSourceTypeKey::Nominal { id, .. } => {
            source_view(source, underlying.get(id).ok_or(GoOrdinaryError::Linkage)?)
        }
        GoSourceTypeKey::Builtin { name, width, .. } => match name.as_str() {
            "bool" => View::Equality(boolean()),
            "string" => View::Product(vec![word(64), View::Vector(8, Box::new(word(8)))]),
            "error" => View::Product(
                std::iter::once(word(64))
                    .chain(cases.iter().map(|(id, _)| source_view(source, id)))
                    .collect(),
            ),
            _ => {
                let n = usize::from(width.ok_or(GoOrdinaryError::Linkage)?);
                if ![8, 16, 32, 64].contains(&n) {
                    return Err(GoOrdinaryError::Linkage);
                }
                word(n)
            }
        },
        GoSourceTypeKey::Array {
            length, element, ..
        } => {
            if *length == 0 {
                View::Equality(boolean())
            } else {
                View::Vector(
                    (u64::BITS - (length - 1).leading_zeros()) as usize,
                    Box::new(source_view(source, element)),
                )
            }
        }
        GoSourceTypeKey::Struct { fields, .. } => View::Product(
            fields
                .iter()
                .map(|f| source_view(source, &f.type_id))
                .collect(),
        ),
        GoSourceTypeKey::Slice { .. } => View::Product(vec![
            View::Equality(boolean()),
            View::Equality(nat()),
            View::Vector(4, Box::new(View::Product(vec![word(8), word(64)]))),
            word(64),
            word(64),
            word(64),
        ]),
    })
}
fn type_order(
    keys: &BTreeMap<String, GoSourceTypeKey>,
    underlying: &BTreeMap<String, String>,
    cases: &[(String, u64)],
) -> Result<Vec<String>> {
    fn visit(
        id: &str,
        keys: &BTreeMap<String, GoSourceTypeKey>,
        underlying: &BTreeMap<String, String>,
        cases: &[(String, u64)],
        active: &mut BTreeSet<String>,
        done: &mut BTreeSet<String>,
        order: &mut Vec<String>,
    ) -> Result<()> {
        if done.contains(id) {
            return Ok(());
        }
        if !active.insert(id.into()) {
            return Err(GoOrdinaryError::Linkage);
        }
        let children = match keys.get(id).ok_or(GoOrdinaryError::Linkage)? {
            GoSourceTypeKey::Nominal { .. } => {
                vec![underlying.get(id).ok_or(GoOrdinaryError::Linkage)?.clone()]
            }
            GoSourceTypeKey::Array { element, .. } | GoSourceTypeKey::Slice { element, .. } => {
                vec![element.clone()]
            }
            GoSourceTypeKey::Struct { fields, .. } => {
                fields.iter().map(|f| f.type_id.clone()).collect()
            }
            GoSourceTypeKey::Builtin { name, .. } if name == "error" => {
                cases.iter().map(|(id, _)| id.clone()).collect()
            }
            _ => vec![],
        };
        for child in children {
            visit(&child, keys, underlying, cases, active, done, order)?;
        }
        active.remove(id);
        done.insert(id.into());
        order.push(id.into());
        Ok(())
    }
    let (mut active, mut done, mut order) = (BTreeSet::new(), BTreeSet::new(), vec![]);
    for id in keys.keys() {
        visit(
            id,
            keys,
            underlying,
            cases,
            &mut active,
            &mut done,
            &mut order,
        )?;
    }
    Ok(order)
}
fn append_type(
    builder: &mut Builder,
    source: &str,
    module: &str,
    id: &str,
    view: &View,
) -> Result<()> {
    let carrier = source_carrier(source, id);
    let prefix = format!("{module}.T{id}");
    let own = |n: &str, args| call(&format!("{prefix}.{n}"), args);
    let seen = |x| own("View", vec![x]);
    emit(
        builder,
        &format!("{prefix}.View"),
        &[("x", carrier.clone())],
        view.carrier(),
        var("x"),
        false,
    )?;
    let pair = [("x", carrier.clone()), ("y", carrier.clone())];
    let body = view.relation(module, seen(var("x")), seen(var("y")));
    emit(
        builder,
        &format!("{prefix}.Related"),
        &pair,
        Expr::Sort0,
        body.clone(),
        false,
    )?;
    let mut proof_args = pair.to_vec();
    proof_args.push(("same", own("Related", vec![var("x"), var("y")])));
    emit(
        builder,
        &format!("{prefix}.Observations"),
        &proof_args,
        body,
        var("same"),
        true,
    )?;
    emit(
        builder,
        &format!("{prefix}.Reflexive"),
        &[("x", carrier.clone())],
        own("Related", vec![var("x"), var("x")]),
        view.reflexive(module, seen(var("x"))),
        true,
    )?;
    emit(
        builder,
        &format!("{prefix}.Symmetric"),
        &proof_args,
        own("Related", vec![var("y"), var("x")]),
        view.symmetric(
            module,
            seen(var("x")),
            seen(var("y")),
            own("Observations", vec![var("x"), var("y"), var("same")]),
        ),
        true,
    )?;
    let args = [
        ("x", carrier.clone()),
        ("y", carrier.clone()),
        ("z", carrier.clone()),
        ("xy", own("Related", vec![var("x"), var("y")])),
        ("yz", own("Related", vec![var("y"), var("z")])),
    ];
    emit(
        builder,
        &format!("{prefix}.Transitive"),
        &args,
        own("Related", vec![var("x"), var("z")]),
        view.transitive(
            module,
            seen(var("x")),
            seen(var("y")),
            seen(var("z")),
            own("Observations", vec![var("x"), var("y"), var("xy")]),
            own("Observations", vec![var("y"), var("z"), var("yz")]),
        ),
        true,
    )?;
    let mut eq_args = pair.to_vec();
    eq_args.push(("equal", eq(carrier.clone(), var("x"), var("y"))));
    emit(
        builder,
        &format!("{prefix}.OfEquality"),
        &eq_args,
        own("Related", vec![var("x"), var("y")]),
        call(
            "Std.Eq.rewrite",
            vec![
                carrier.clone(),
                var("x"),
                var("y"),
                lam(
                    "_next_value",
                    carrier,
                    own("Related", vec![var("x"), var("_next_value")]),
                ),
                var("equal"),
                own("Reflexive", vec![var("x")]),
            ],
        ),
        true,
    )?;
    let zero = constant(&format!("{source}.T{id}.Zero"));
    emit(
        builder,
        &format!("{prefix}.ZeroRelated"),
        &[],
        own("Related", vec![zero.clone(), zero.clone()]),
        own("Reflexive", vec![zero]),
        true,
    )
}

#[derive(Clone)]
pub struct GoSourceValueRelationCatalogue {
    builder: Builder,
    source: String,
    boxing: String,
    operations: String,
    module: String,
    error: String,
    cases: Vec<(String, u64)>,
    types: Vec<GoSourceTypeKey>,
    underlying: BTreeMap<String, String>,
    prior_operations: Vec<u8>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoSourceValueRelationRow {
    source: String,
    boxing: String,
    operations: String,
    relation: String,
    module: String,
    error: String,
    cases: Vec<(String, u64)>,
    boxed: Option<String>,
    certificate: Vec<u8>,
}
impl GoSourceValueRelationRow {
    pub fn source_layout_module_name(&self) -> &str {
        &self.source
    }
    pub fn boxing_module_name(&self) -> &str {
        &self.boxing
    }
    pub fn operations_module_name(&self) -> &str {
        &self.operations
    }
    pub fn relation_module_name(&self) -> &str {
        &self.relation
    }
    pub fn module_name(&self) -> &str {
        &self.module
    }
    pub fn source_error_type_id(&self) -> &str {
        &self.error
    }
    pub fn case_tags(&self) -> &[(String, u64)] {
        &self.cases
    }
    pub fn boxed_source_type_id(&self) -> Option<&str> {
        self.boxed.as_deref()
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
}
impl GoSourceValueRelationCatalogue {
    pub fn source_layout_module_name(&self) -> &str {
        &self.source
    }
    pub fn relation_module_name(&self) -> &str {
        &self.module
    }
    pub fn case_tags(&self) -> &[(String, u64)] {
        &self.cases
    }
    pub fn source_types(&self) -> &[GoSourceTypeKey] {
        &self.types
    }
    pub fn underlying_type_id(&self, id: &str) -> Option<&str> {
        self.underlying.get(id).map(String::as_str)
    }
    pub fn prior_operations_certificate_bytes(&self) -> &[u8] {
        &self.prior_operations
    }
    pub fn generate_row(&self, boxed_type: Option<&str>) -> Result<GoSourceValueRelationRow> {
        if boxed_type.is_some_and(|id| !self.cases.iter().any(|(case, _)| case == id)) {
            return Err(GoOrdinaryError::Linkage);
        }
        let mut builder = self.builder.clone();
        let module = format!(
            "{}.{}",
            self.module,
            boxed_type
                .map(|id| format!("C{id}"))
                .unwrap_or_else(|| "Nil".into())
        );
        append_row(&mut builder, self, &module, boxed_type)?;
        builder.certificate.module = module.clone();
        Ok(GoSourceValueRelationRow {
            source: self.source.clone(),
            boxing: self.boxing.clone(),
            operations: self.operations.clone(),
            relation: self.module.clone(),
            module,
            error: self.error.clone(),
            cases: self.cases.clone(),
            boxed: boxed_type.map(String::from),
            certificate: builder.finish()?,
        })
    }
    pub fn import_row(
        &self,
        candidate: &[u8],
        boxed_type: Option<&str>,
    ) -> Result<GoSourceValueRelationRow> {
        if candidate.len() > MAX_BYTES {
            return Err(GoOrdinaryError::Limit);
        }
        let expected = self.generate_row(boxed_type)?;
        if candidate != expected.certificate_bytes() {
            return Err(GoOrdinaryError::Linkage);
        }
        Ok(expected)
    }
}
pub fn generate_go_practical_source_value_relation_catalogue(
    domain: &GoSourceValueDomain,
) -> Result<GoSourceValueRelationCatalogue> {
    let (builder, operations) = source_error_operations::operations_builder(domain)?;
    let prior_operations = builder.clone().finish()?;
    let source = operations.source_layout_module_name().to_string();
    let mut roots = builder
        .certificate
        .declarations
        .iter()
        .filter_map(|d| {
            let name = &builder.certificate.name_table[d.name as usize];
            name.starts_with(&(source.clone() + "."))
                .then(|| name.clone())
        })
        .collect::<Vec<_>>();
    roots.extend(
        [
            "Std.Eq.rewrite",
            "Std.Eq.symm",
            "Std.Eq.trans",
            "Mpk.Go.Practical.Product.Left",
            "Mpk.Go.Practical.Product.Right",
        ]
        .into_iter()
        .map(String::from),
    );
    let mut builder = composition::retain_declarations(builder, &roots)?;
    let module = format!("{}.ValueRelations", operations.module_name());
    let keys = domain
        .types()
        .map(|k| (k.id().to_string(), k.clone()))
        .collect::<BTreeMap<_, _>>();
    let underlying = keys
        .keys()
        .filter_map(|id| domain.underlying(id).map(|u| (id.clone(), u.to_string())))
        .collect::<BTreeMap<_, _>>();
    let cases = operations.case_tags().to_vec();
    for id in type_order(&keys, &underlying, &cases)? {
        append_type(
            &mut builder,
            &source,
            &module,
            &id,
            &shape(&source, &keys[&id], &underlying, &cases)?,
        )?;
    }
    Ok(GoSourceValueRelationCatalogue {
        builder,
        source,
        boxing: operations.boxing_module_name().into(),
        operations: operations.module_name().into(),
        module,
        error: operations.source_error_type_id().into(),
        cases,
        types: keys.into_values().collect(),
        underlying,
        prior_operations,
    })
}

fn append_row(
    builder: &mut Builder,
    c: &GoSourceValueRelationCatalogue,
    module: &str,
    boxed: Option<&str>,
) -> Result<()> {
    let call_member = |m: &str, n: &str, args| call(&format!("{m}.{n}"), args);
    let own = |n: &str, args| call_member(module, n, args);
    let op = |n: &str, args| call_member(&c.operations, n, args);
    let observed = |n: &str, e| call_member(&c.boxing, n, vec![e]);
    let relate = |id: &str, x, y| View::source(&c.module, id, "Related", vec![x, y]);
    let from_eq = |id: &str, x, y, h| View::source(&c.module, id, "OfEquality", vec![x, y, h]);
    let fields = std::iter::once(word(64))
        .chain(c.cases.iter().map(|(id, _)| source_view(&c.source, id)))
        .collect::<Vec<_>>();
    let field_types = fields.iter().map(View::carrier).collect::<Vec<_>>();
    let projection = |e, i| error_layout::get(&field_types, e, i);
    let error_type = source_carrier(&c.source, &c.error);
    let e = var("e");
    let copied = op("Copy", vec![e.clone()]);
    let tag = observed("Tag", e.clone());
    let copy_tag = observed("Tag", copied.clone());
    let tag_proof = word(64).from_equality(
        &c.module,
        copy_tag.clone(),
        tag.clone(),
        op("TagOfCopy", vec![e.clone()]),
    );
    emit(
        builder,
        &format!("{module}.CopyTagRelated"),
        &[("e", error_type.clone())],
        word(64).relation(&c.module, copy_tag, tag),
        tag_proof,
        true,
    )?;
    let mut copy_proofs = vec![own("CopyTagRelated", vec![e.clone()])];
    for (i, (id, _)) in c.cases.iter().enumerate() {
        let left = projection(copied.clone(), i + 1);
        let right = projection(e.clone(), i + 1);
        let proof = from_eq(
            id,
            left.clone(),
            right.clone(),
            op(&format!("CopyPayload{i}"), vec![e.clone()]),
        );
        let n = format!("CopyFieldRelated.C{id}");
        emit(
            builder,
            &format!("{module}.{n}"),
            &[("e", error_type.clone())],
            relate(id, left, right),
            proof,
            true,
        )?;
        copy_proofs.push(own(&n, vec![e.clone()]));
    }
    // Match the exact named observations in the full source relation.
    // Reusing these adapters avoids repeatedly unfolding whole heterogeneous
    // carrier arguments while assembling its balanced proof product.
    let viewed_error = |value| View::source(&c.module, &c.error, "View", vec![value]);
    let lefts = View::fields(&fields, viewed_error(copied.clone()));
    let rights = View::fields(&fields, viewed_error(e.clone()));
    let props = fields
        .iter()
        .enumerate()
        .map(|(i, v)| v.relation(&c.module, lefts[i].clone(), rights[i].clone()))
        .collect::<Vec<_>>();
    emit(
        builder,
        &format!("{module}.CopyRelated"),
        &[("e", error_type.clone())],
        relate(&c.error, copied.clone(), e.clone()),
        error_layout::make(&props, &copy_proofs),
        true,
    )?;
    let copy_again = op("Copy", vec![copied.clone()]);
    emit(
        builder,
        &format!("{module}.CopyTwiceRelated"),
        &[("e", error_type.clone())],
        relate(&c.error, copy_again.clone(), e.clone()),
        View::source(
            &c.module,
            &c.error,
            "Transitive",
            vec![
                copy_again,
                copied.clone(),
                e.clone(),
                own("CopyRelated", vec![copied]),
                own("CopyRelated", vec![e]),
            ],
        ),
        true,
    )?;
    let nil = op("Nil", vec![]);
    emit(
        builder,
        &format!("{module}.NilRelated"),
        &[],
        relate(&c.error, nil.clone(), nil.clone()),
        View::source(&c.module, &c.error, "Reflexive", vec![nil]),
        true,
    )?;
    if let Some(id) = boxed {
        let active = c
            .cases
            .iter()
            .position(|(t, _)| t == id)
            .ok_or(GoOrdinaryError::Linkage)?;
        let payload = source_carrier(&c.source, id);
        let prefix = format!("{}.C{id}", c.boxing);
        let boxed_value = |v| call_member(&prefix, "Box", vec![v]);
        let x = var("x");
        let y = var("y");
        let bx = boxed_value(x.clone());
        let by = boxed_value(y.clone());
        let args = [
            ("x", payload.clone()),
            ("y", payload.clone()),
            ("same", relate(id, x.clone(), y.clone())),
        ];
        let mut proofs = vec![];
        for (i, v) in fields.iter().enumerate() {
            let left = projection(bx.clone(), i);
            let right = projection(by.clone(), i);
            let h = if i == active + 1 {
                let eqx = call_member(&prefix, "ActiveOfBox", vec![x.clone()]);
                let eqy = call_member(&prefix, "ActiveOfBox", vec![y.clone()]);
                let first = call(
                    "Std.Eq.rewrite",
                    vec![
                        payload.clone(),
                        x.clone(),
                        left.clone(),
                        lam(
                            "_next_value",
                            payload.clone(),
                            relate(id, var("_next_value"), y.clone()),
                        ),
                        call(
                            "Std.Eq.symm",
                            vec![payload.clone(), left.clone(), x.clone(), eqx],
                        ),
                        var("same"),
                    ],
                );
                call(
                    "Std.Eq.rewrite",
                    vec![
                        payload.clone(),
                        y.clone(),
                        right.clone(),
                        lam(
                            "_next_value",
                            payload.clone(),
                            relate(id, left.clone(), var("_next_value")),
                        ),
                        call(
                            "Std.Eq.symm",
                            vec![payload.clone(), right.clone(), y.clone(), eqy],
                        ),
                        first,
                    ],
                )
            } else {
                let witness = if i == 0 {
                    call(
                        "Std.Eq.trans",
                        vec![
                            v.carrier(),
                            left.clone(),
                            call_member(&prefix, "Tag", vec![]),
                            right.clone(),
                            call_member(&prefix, "TagOfBox", vec![x.clone()]),
                            call(
                                "Std.Eq.symm",
                                vec![
                                    v.carrier(),
                                    right.clone(),
                                    call_member(&prefix, "Tag", vec![]),
                                    call_member(&prefix, "TagOfBox", vec![y.clone()]),
                                ],
                            ),
                        ],
                    )
                } else {
                    refl(
                        v.carrier(),
                        constant(&format!("{}.T{}.Zero", c.source, c.cases[i - 1].0)),
                    )
                };
                v.from_equality(&c.module, left.clone(), right.clone(), witness)
            };
            let n = format!("BoxFieldRelated{i}");
            emit(
                builder,
                &format!("{module}.{n}"),
                &args,
                v.relation(&c.module, left, right),
                h,
                true,
            )?;
            proofs.push(own(&n, vec![x.clone(), y.clone(), var("same")]));
        }
        let prop_fields = fields
            .iter()
            .enumerate()
            .map(|(i, v)| {
                v.relation(
                    &c.module,
                    projection(viewed_error(bx.clone()), i),
                    projection(viewed_error(by.clone()), i),
                )
            })
            .collect::<Vec<_>>();
        emit(
            builder,
            &format!("{module}.BoxRelated"),
            &args,
            relate(&c.error, bx.clone(), by.clone()),
            error_layout::make(&prop_fields, &proofs),
            true,
        )?;
        let reverse_args = [
            ("x", payload.clone()),
            ("y", payload.clone()),
            ("same", relate(&c.error, bx.clone(), by.clone())),
        ];
        let observations = View::source(
            &c.module,
            &c.error,
            "Observations",
            vec![bx.clone(), by.clone(), var("same")],
        );
        let witness = error_layout::get(&prop_fields, observations, active + 1);
        let first = call(
            "Std.Eq.rewrite",
            vec![
                payload.clone(),
                projection(bx.clone(), active + 1),
                x.clone(),
                lam(
                    "_next_value",
                    payload.clone(),
                    relate(id, var("_next_value"), projection(by.clone(), active + 1)),
                ),
                call_member(&prefix, "ActiveOfBox", vec![x.clone()]),
                witness,
            ],
        );
        let proof = call(
            "Std.Eq.rewrite",
            vec![
                payload.clone(),
                projection(by, active + 1),
                y.clone(),
                lam(
                    "_next_value",
                    payload.clone(),
                    relate(id, x.clone(), var("_next_value")),
                ),
                call_member(&prefix, "ActiveOfBox", vec![y.clone()]),
                first,
            ],
        );
        emit(
            builder,
            &format!("{module}.ActiveRelated"),
            &reverse_args,
            relate(id, x, y),
            proof,
            true,
        )?;
    }
    Ok(())
}
