use super::hir_check::HirCheckCode;
use rust2vir_internal::contract::ContractType;
use rust2vir_internal::json::JsonValue;
use rustc_middle::ty::{self, IntTy, Ty, TyCtxt, UintTy};
use rustc_span::def_id::{LocalDefId, LOCAL_CRATE};
use std::collections::BTreeMap;

pub(super) fn contract_type<'tcx>(
    tcx: TyCtxt<'tcx>,
    owner: LocalDefId,
    ty: Ty<'tcx>,
) -> Result<ContractType, HirCheckCode> {
    match ty.kind() {
        ty::Bool => Ok(ContractType::Bool),
        ty::Int(integer) => Ok(ContractType::BitVector {
            width: match integer {
                IntTy::I8 => 8,
                IntTy::I16 => 16,
                IntTy::I32 => 32,
                IntTy::I64 => 64,
                IntTy::Isize => u8::try_from(tcx.sess.target.pointer_width).unwrap_or(0),
                _ => return Err(HirCheckCode::Type),
            },
            signed: true,
        }),
        ty::Uint(integer) => Ok(ContractType::BitVector {
            width: match integer {
                UintTy::U8 => 8,
                UintTy::U16 => 16,
                UintTy::U32 => 32,
                UintTy::U64 => 64,
                UintTy::Usize => u8::try_from(tcx.sess.target.pointer_width).unwrap_or(0),
                _ => return Err(HirCheckCode::Type),
            },
            signed: false,
        }),
        ty::Array(element, length) => Ok(ContractType::Array {
            element: Box::new(contract_type(tcx, owner, *element)?),
            length: resolved_array_length(tcx, owner, *length)?,
        }),
        ty::Adt(adt, arguments)
            if adt.is_struct() && adt.did().is_local() && arguments.is_empty() =>
        {
            Ok(ContractType::Struct {
                id: canonical_item_id(tcx, adt.did().as_local().ok_or(HirCheckCode::Type)?),
            })
        }
        _ => Err(HirCheckCode::Type),
    }
}

fn resolved_array_length<'tcx>(
    tcx: TyCtxt<'tcx>,
    owner: LocalDefId,
    length: ty::Const<'tcx>,
) -> Result<u64, HirCheckCode> {
    let typing_env = ty::TypingEnv::post_analysis(tcx, owner);
    length
        .try_to_target_usize(tcx)
        .or_else(|| {
            let ty::ConstKind::Unevaluated(unevaluated) = length.kind() else {
                return None;
            };
            rustc_middle::mir::Const::Unevaluated(
                rustc_middle::mir::UnevaluatedConst::new(unevaluated.def, unevaluated.args),
                tcx.types.usize,
            )
            .try_eval_target_usize(tcx, typing_env)
        })
        .ok_or(HirCheckCode::Type)
}

pub(super) fn canonical_item_id(tcx: TyCtxt<'_>, def_id: LocalDefId) -> String {
    format!(
        "{}::{}",
        tcx.crate_name(LOCAL_CRATE).as_str(),
        tcx.def_path_str(def_id.to_def_id())
    )
}

pub(super) fn vir_type(ty: &ContractType) -> JsonValue {
    match ty {
        ContractType::Bool => JsonValue::Object(BTreeMap::from([(
            "kind".to_owned(),
            JsonValue::String("bool".to_owned()),
        )])),
        ContractType::BitVector { width, signed } => JsonValue::Object(BTreeMap::from([
            ("kind".to_owned(), JsonValue::String("bv".to_owned())),
            ("width".to_owned(), JsonValue::Number(width.to_string())),
            ("signed".to_owned(), JsonValue::Bool(*signed)),
        ])),
        ContractType::Array { element, length } => JsonValue::Object(BTreeMap::from([
            ("kind".to_owned(), JsonValue::String("array".to_owned())),
            ("length".to_owned(), JsonValue::Number(length.to_string())),
            ("element".to_owned(), vir_type(element)),
        ])),
        ContractType::Struct { id } => JsonValue::Object(BTreeMap::from([
            ("kind".to_owned(), JsonValue::String("struct".to_owned())),
            ("id".to_owned(), JsonValue::String(id.clone())),
        ])),
    }
}
