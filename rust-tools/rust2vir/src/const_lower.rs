use super::hir_check::HirCheckCode;
use super::type_lower::{canonical_item_id, contract_type, vir_type};
use rust2vir_internal::contract::ContractType;
use rust2vir_internal::json::JsonValue;
use rustc_ast::ast::{LitKind, UnOp};
use rustc_hir as hir;
use rustc_hir::def::DefKind;
use rustc_middle::mir;
use rustc_middle::ty::TyCtxt;
use rustc_span::def_id::LocalDefId;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HirConstant {
    pub def_id: LocalDefId,
    pub id: String,
    pub name: String,
    pub ty: ContractType,
    pub value: PrimitiveConstantValue,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PrimitiveConstantValue {
    Bool(bool),
    Integer {
        value: String,
        width: u8,
        signed: bool,
    },
}

pub(super) fn lower_constant(
    tcx: TyCtxt<'_>,
    def_id: LocalDefId,
    name: &str,
    body_id: hir::BodyId,
) -> Result<HirConstant, HirCheckCode> {
    let ty = contract_type(tcx, def_id, tcx.type_of(def_id).instantiate_identity())?;
    if ty.is_aggregate() {
        return Err(HirCheckCode::Type);
    }
    let value = tcx.hir_body(body_id).value;
    let value = match value.kind {
        hir::ExprKind::Lit(literal) => literal_value(literal.node.clone(), &ty)?,
        hir::ExprKind::Unary(UnOp::Neg, inner) => {
            let hir::ExprKind::Lit(literal) = inner.kind else {
                return Err(HirCheckCode::Purity);
            };
            let LitKind::Int(magnitude, _) = literal.node else {
                return Err(HirCheckCode::Purity);
            };
            let ContractType::BitVector {
                width,
                signed: true,
            } = ty
            else {
                return Err(HirCheckCode::Purity);
            };
            integer_value(
                if magnitude == 0 {
                    "0".to_owned()
                } else {
                    format!("-{magnitude}")
                },
                width,
                true,
            )
        }
        _ => return Err(HirCheckCode::Purity),
    };
    Ok(HirConstant {
        def_id,
        id: canonical_item_id(tcx, def_id),
        name: name.to_owned(),
        ty,
        value,
    })
}

fn literal_value(
    literal: LitKind,
    ty: &ContractType,
) -> Result<PrimitiveConstantValue, HirCheckCode> {
    match (literal, ty) {
        (LitKind::Bool(value), ContractType::Bool) => Ok(PrimitiveConstantValue::Bool(value)),
        (LitKind::Int(value, _), ContractType::BitVector { width, signed }) => {
            Ok(integer_value(value.to_string(), *width, *signed))
        }
        _ => Err(HirCheckCode::Type),
    }
}

fn integer_value(value: String, width: u8, signed: bool) -> PrimitiveConstantValue {
    PrimitiveConstantValue::Integer {
        value,
        width,
        signed,
    }
}

fn value_json(value: &PrimitiveConstantValue) -> JsonValue {
    match value {
        PrimitiveConstantValue::Bool(value) => JsonValue::Object(BTreeMap::from([(
            "bool".to_owned(),
            JsonValue::Bool(*value),
        )])),
        PrimitiveConstantValue::Integer {
            value,
            width,
            signed,
        } => JsonValue::Object(BTreeMap::from([(
            "int".to_owned(),
            JsonValue::Object(BTreeMap::from([
                ("value".to_owned(), JsonValue::String(value.clone())),
                ("width".to_owned(), JsonValue::Number(width.to_string())),
                ("signed".to_owned(), JsonValue::Bool(*signed)),
            ])),
        )])),
    }
}

pub(super) fn declarations(constants: &[HirConstant]) -> Result<Vec<JsonValue>, HirCheckCode> {
    let mut ordered = constants.iter().collect::<Vec<_>>();
    ordered.sort_by(|left, right| left.id.cmp(&right.id));
    let mut ids = BTreeSet::new();
    ordered
        .into_iter()
        .map(|constant| {
            if !ids.insert(constant.id.as_str()) {
                return Err(HirCheckCode::Identifier);
            }
            Ok(JsonValue::Object(BTreeMap::from([
                ("id".to_owned(), JsonValue::String(constant.id.clone())),
                ("name".to_owned(), JsonValue::String(constant.name.clone())),
                ("type".to_owned(), vir_type(&constant.ty)),
                ("value".to_owned(), value_json(&constant.value)),
            ])))
        })
        .collect()
}

pub(super) fn reference<'tcx>(
    tcx: TyCtxt<'tcx>,
    constant: &mir::Const<'tcx>,
) -> Option<(String, ContractType)> {
    let mir::Const::Unevaluated(unevaluated, _) = constant else {
        return None;
    };
    let def_id = unevaluated.def.as_local()?;
    if !unevaluated.args.is_empty()
        || unevaluated.promoted.is_some()
        || !matches!(tcx.def_kind(def_id), DefKind::Const)
    {
        return None;
    }
    let ty = contract_type(tcx, def_id, tcx.type_of(def_id).instantiate_identity()).ok()?;
    (!ty.is_aggregate()).then(|| (canonical_item_id(tcx, def_id), ty))
}
