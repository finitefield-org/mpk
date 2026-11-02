//! Closed source-derived carrier/zero recipes for the complete error universe.
//! These private source keys are not a canonical capture/context or a proof of
//! arbitrary payload validity, execution, dispatch, or program correspondence.
use super::syntax::*;
use super::{
    error_layout, error_value_builder_with_recipes, word, Builder, GoOrdinaryError, Result,
    MAX_BYTES,
};
use crate::go_practical_type_keys::{GoSourceTypeKey, GoSourceValueDomain};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoSourceErrorLayoutFoundation {
    source_error_type: String,
    cases: Vec<String>,
    module: String,
    certificate: Vec<u8>,
}
impl GoSourceErrorLayoutFoundation {
    pub fn source_error_type_id(&self) -> &str {
        &self.source_error_type
    }
    pub fn case_type_ids(&self) -> &[String] {
        &self.cases
    }
    pub fn module_name(&self) -> &str {
        &self.module
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
}

struct Sources<'a> {
    domain: &'a GoSourceValueDomain,
    keys: BTreeMap<String, &'a GoSourceTypeKey>,
    cases: Vec<String>,
    active: BTreeSet<String>,
    done: BTreeSet<String>,
    order: Vec<String>,
}
impl Sources<'_> {
    fn visit(&mut self, id: &str) -> Result<()> {
        if self.done.contains(id) {
            return Ok(());
        }
        if !self.active.insert(id.into()) {
            return Err(GoOrdinaryError::Linkage);
        }
        let key = *self.keys.get(id).ok_or(GoOrdinaryError::Linkage)?;
        let dependencies = match key {
            GoSourceTypeKey::Nominal { .. } => vec![self
                .domain
                .underlying(id)
                .ok_or(GoOrdinaryError::Linkage)?
                .to_string()],
            GoSourceTypeKey::Array { element, .. } | GoSourceTypeKey::Slice { element, .. } => {
                vec![element.clone()]
            }
            GoSourceTypeKey::Struct { fields, .. } => {
                fields.iter().map(|f| f.type_id.clone()).collect()
            }
            GoSourceTypeKey::Builtin { name, .. } if name == "error" => self.cases.clone(),
            _ => vec![],
        };
        // Even a zero-length array and a nil slice retain their element edge.
        for dependency in dependencies {
            self.visit(&dependency)?;
        }
        self.active.remove(id);
        self.done.insert(id.into());
        self.order.push(id.into());
        Ok(())
    }
}

fn vector(bits: usize, mut ty: Expr, mut value: Expr) -> (Expr, Expr) {
    for i in (0..bits).rev() {
        let name = format!("_source_index{i}");
        ty = pi(&name, boolean(), ty);
        value = lam(&name, boolean(), value);
    }
    (ty, value)
}
fn word_zero(width: usize) -> Result<(Expr, Expr)> {
    Ok(vector(word::depth(width)?, boolean(), bit(false)))
}
fn balanced(parts: &[(Expr, Expr)]) -> (Expr, Expr) {
    let types = parts.iter().map(|p| p.0.clone()).collect::<Vec<_>>();
    let values = parts.iter().map(|p| p.1.clone()).collect::<Vec<_>>();
    (
        error_layout::product(&types),
        error_layout::make(&types, &values),
    )
}
fn member(module: &str, id: &str, name: &str) -> String {
    format!("{module}.T{id}.{name}")
}
fn source_pair(module: &str, id: &str) -> (Expr, Expr) {
    (
        constant(&member(module, id, "Carrier")),
        constant(&member(module, id, "Zero")),
    )
}
fn shape(source: &Sources<'_>, module: &str, id: &str) -> Result<(Expr, Expr)> {
    let key = *source.keys.get(id).ok_or(GoOrdinaryError::Linkage)?;
    Ok(match key {
        GoSourceTypeKey::Nominal { .. } => source_pair(
            module,
            source
                .domain
                .underlying(id)
                .ok_or(GoOrdinaryError::Linkage)?,
        ),
        GoSourceTypeKey::Builtin { name, width, .. } => match name.as_str() {
            "bool" => (boolean(), bit(false)),
            "string" => balanced(&[word_zero(64)?, {
                let (ty, zero) = word_zero(8)?;
                vector(8, ty, zero)
            }]),
            "error" => {
                let mut parts = vec![word_zero(64)?];
                parts.extend(source.cases.iter().map(|id| source_pair(module, id)));
                balanced(&parts)
            }
            "int" | "int8" | "int16" | "int32" | "int64" | "uint" | "uint8" | "uint16"
            | "uint32" | "uint64" => {
                word_zero(usize::from(width.ok_or(GoOrdinaryError::Linkage)?))?
            }
            _ => return Err(GoOrdinaryError::Linkage),
        },
        GoSourceTypeKey::Array {
            length, element, ..
        } => {
            if *length == 0 {
                (boolean(), bit(false))
            } else {
                let (ty, zero) = source_pair(module, element);
                vector(
                    (u64::BITS - (length - 1).leading_zeros()) as usize,
                    ty,
                    zero,
                )
            }
        }
        GoSourceTypeKey::Slice { .. } => {
            let (path_ty, path_zero) = balanced(&[word_zero(8)?, word_zero(64)?]);
            balanced(&[
                (boolean(), bit(true)),
                (nat(), zero()),
                vector(4, path_ty, path_zero),
                word_zero(64)?,
                word_zero(64)?,
                word_zero(64)?,
            ])
        }
        GoSourceTypeKey::Struct { fields, .. } => balanced(
            &fields
                .iter()
                .map(|f| source_pair(module, &f.type_id))
                .collect::<Vec<_>>(),
        ),
    })
}

/// Use the complete source-derived global U, including entry cases. No caller
/// case-count, alternate payload list, or supplied zero recipe is accepted.
/// The result binds private original-source keys, not the future frozen wire.
pub fn generate_go_practical_source_error_layout_foundation(
    domain: &GoSourceValueDomain,
) -> Result<GoSourceErrorLayoutFoundation> {
    let (builder, mut foundation, _) = source_builder(domain)?;
    foundation.certificate = builder.finish()?;
    Ok(foundation)
}

pub(super) fn source_builder(
    domain: &GoSourceValueDomain,
) -> Result<(Builder, GoSourceErrorLayoutFoundation, BTreeMap<String, Expr>)> {
    let cases = domain
        .error_cases()
        .iter()
        .map(|c| c.value_type_id().to_string())
        .collect::<Vec<_>>();
    if cases.windows(2).any(|p| p[0] >= p[1]) {
        return Err(GoOrdinaryError::Linkage);
    }
    let mut source = Sources {
        domain,
        keys: domain.types().map(|k| (k.id().to_string(), k)).collect(),
        cases,
        active: BTreeSet::new(),
        done: BTreeSet::new(),
        order: vec![],
    };
    let mut errors = source
        .keys
        .values()
        .filter(|k| matches!(k, GoSourceTypeKey::Builtin { name, .. } if name == "error"));
    let error = errors
        .next()
        .ok_or(GoOrdinaryError::Linkage)?
        .id()
        .to_string();
    if errors.next().is_some() {
        return Err(GoOrdinaryError::Linkage);
    }
    source.visit(&error)?;
    // Retain every checked value type of this source unit, including element
    // edges hidden by zero lengths and the same-U error fields of aggregates.
    let roots = source.keys.keys().cloned().collect::<Vec<_>>();
    for id in roots {
        source.visit(&id)?;
    }
    let rows = source
        .order
        .iter()
        .map(|id| (source.keys[id], domain.underlying(id)))
        .collect::<Vec<_>>();
    let preimage = serde_json::to_vec(&(&error, &source.cases, &rows))
        .map_err(|_| GoOrdinaryError::Linkage)?;
    let mut h = Sha256::new();
    h.update(b"MPK-GO-SOURCE-ERROR-LAYOUT-PREPARATION-1\0");
    h.update(preimage);
    let module = format!("Mpk.Go.Practical.SourceErrorLayout.S{:x}", h.finalize());
    let (mut builder, recipes) = error_value_builder_with_recipes()?;
    error_layout::append(&mut builder, source.cases.len(), &recipes)?;
    let mut error_shape = None;
    for id in &source.order {
        let (ty, zero) = shape(&source, &module, id)?;
        emit_source(&mut builder, &module, id, ty.clone(), zero)?;
        if id == &error {
            error_shape = Some(ty);
        }
    }
    let ty = error_shape.ok_or(GoOrdinaryError::Linkage)?;
    let mut arguments = source
        .cases
        .iter()
        .map(|id| source_pair(&module, id).0)
        .collect::<Vec<_>>();
    arguments.extend(source.cases.iter().map(|id| source_pair(&module, id).1));
    let nil = call(
        &format!("Mpk.Go.Practical.ErrorLayout.N{}.Nil", source.cases.len()),
        arguments,
    );
    let source_zero = source_pair(&module, &error).1;
    emit(
        &mut builder,
        &format!("{module}.NilFromSourceZeros"),
        &[],
        eq(ty.clone(), nil, source_zero.clone()),
        refl(ty, source_zero),
        true,
    )?;
    builder.certificate.module = module.clone();
    let foundation = GoSourceErrorLayoutFoundation {
        source_error_type: error,
        cases: source.cases,
        module,
        certificate: vec![],
    };
    Ok((builder, foundation, recipes))
}
fn emit_source(builder: &mut Builder, module: &str, id: &str, ty: Expr, zero: Expr) -> Result<()> {
    emit(
        builder,
        &member(module, id, "Carrier"),
        &[],
        Expr::Sort0,
        ty.clone(),
        false,
    )?;
    emit(
        builder,
        &member(module, id, "Zero"),
        &[],
        ty.clone(),
        zero.clone(),
        false,
    )?;
    emit(
        builder,
        &member(module, id, "ZeroRecipe"),
        &[],
        eq(
            ty.clone(),
            constant(&member(module, id, "Zero")),
            zero.clone(),
        ),
        refl(ty, zero),
        true,
    )
}
pub fn import_go_practical_source_error_layout_foundation(
    candidate: &[u8],
    domain: &GoSourceValueDomain,
) -> Result<GoSourceErrorLayoutFoundation> {
    if candidate.len() > MAX_BYTES {
        return Err(GoOrdinaryError::Limit);
    }
    let expected = generate_go_practical_source_error_layout_foundation(domain)?;
    if candidate != expected.certificate_bytes() {
        return Err(GoOrdinaryError::Linkage);
    }
    Ok(expected)
}
