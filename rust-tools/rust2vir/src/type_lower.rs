use super::hir_check::HirCheckCode;
use rust2vir_internal::contract::ContractType;
use rust2vir_internal::json::JsonValue;
use rustc_middle::ty::{self, IntTy, Ty, TyCtxt, UintTy};
use rustc_span::def_id::{LocalDefId, LOCAL_CRATE};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HirStructField {
    pub name: String,
    pub ty: ContractType,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HirStructDecl {
    pub def_id: LocalDefId,
    pub id: String,
    pub name: String,
    pub fields: Vec<HirStructField>,
}

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

pub(super) fn struct_declaration(
    tcx: TyCtxt<'_>,
    def_id: LocalDefId,
    name: &str,
) -> Result<HirStructDecl, HirCheckCode> {
    let ty = tcx.type_of(def_id).instantiate_identity();
    let ty::Adt(adt, arguments) = ty.kind() else {
        return Err(HirCheckCode::Type);
    };
    if !adt.is_struct() || !adt.did().is_local() || !arguments.is_empty() {
        return Err(HirCheckCode::Type);
    }
    let fields = adt
        .non_enum_variant()
        .fields
        .iter()
        .map(|field| {
            Ok(HirStructField {
                name: field.name.as_str().to_owned(),
                ty: contract_type(tcx, def_id, field.ty(tcx, arguments))?,
            })
        })
        .collect::<Result<Vec<_>, HirCheckCode>>()?;
    Ok(HirStructDecl {
        def_id,
        id: canonical_item_id(tcx, def_id),
        name: name.to_owned(),
        fields,
    })
}

pub(super) fn collect_struct_def_ids<'tcx>(
    tcx: TyCtxt<'tcx>,
    ty: Ty<'tcx>,
    output: &mut HashSet<LocalDefId>,
) {
    match ty.kind() {
        ty::Array(element, _) => collect_struct_def_ids(tcx, *element, output),
        ty::Adt(adt, arguments)
            if adt.is_struct() && adt.did().is_local() && arguments.is_empty() =>
        {
            let Some(def_id) = adt.did().as_local() else {
                return;
            };
            if output.insert(def_id) {
                for field in &adt.non_enum_variant().fields {
                    collect_struct_def_ids(tcx, field.ty(tcx, arguments), output);
                }
            }
        }
        _ => {}
    }
}

pub(super) fn selected_struct_declarations(
    all: &HashMap<LocalDefId, HirStructDecl>,
    selected: &HashSet<LocalDefId>,
) -> Result<Vec<HirStructDecl>, HirCheckCode> {
    let by_id = selected
        .iter()
        .map(|def_id| {
            all.get(def_id)
                .cloned()
                .map(|declaration| (declaration.id.clone(), declaration))
                .ok_or(HirCheckCode::Type)
        })
        .collect::<Result<BTreeMap<_, _>, _>>()?;
    let mut remaining = BTreeMap::<String, usize>::new();
    let mut dependents = BTreeMap::<String, Vec<String>>::new();
    let mut ready = BTreeSet::new();
    for (id, declaration) in &by_id {
        let mut dependencies = BTreeSet::new();
        for field in &declaration.fields {
            collect_contract_struct_ids(&field.ty, &mut dependencies);
        }
        if dependencies
            .iter()
            .any(|dependency| !by_id.contains_key(dependency))
        {
            return Err(HirCheckCode::Type);
        }
        remaining.insert(id.clone(), dependencies.len());
        if dependencies.is_empty() {
            ready.insert(id.clone());
        }
        for dependency in dependencies {
            dependents.entry(dependency).or_default().push(id.clone());
        }
    }
    let mut ordered = Vec::with_capacity(by_id.len());
    while let Some(id) = ready.pop_first() {
        ordered.push(by_id[&id].clone());
        for dependent in dependents.get(&id).into_iter().flatten() {
            let count = remaining.get_mut(dependent).ok_or(HirCheckCode::Type)?;
            *count = count.checked_sub(1).ok_or(HirCheckCode::Type)?;
            if *count == 0 {
                ready.insert(dependent.clone());
            }
        }
    }
    if ordered.len() == by_id.len() {
        Ok(ordered)
    } else {
        Err(HirCheckCode::Type)
    }
}

pub(super) fn struct_declarations(declarations: &[HirStructDecl]) -> Vec<JsonValue> {
    declarations
        .iter()
        .map(|declaration| {
            JsonValue::Object(BTreeMap::from([
                ("id".to_owned(), JsonValue::String(declaration.id.clone())),
                (
                    "name".to_owned(),
                    JsonValue::String(declaration.name.clone()),
                ),
                (
                    "fields".to_owned(),
                    JsonValue::Array(
                        declaration
                            .fields
                            .iter()
                            .map(|field| {
                                JsonValue::Object(BTreeMap::from([
                                    ("name".to_owned(), JsonValue::String(field.name.clone())),
                                    ("type".to_owned(), vir_type(&field.ty)),
                                ]))
                            })
                            .collect(),
                    ),
                ),
            ]))
        })
        .collect()
}

fn collect_contract_struct_ids(ty: &ContractType, output: &mut BTreeSet<String>) {
    match ty {
        ContractType::Array { element, .. } => collect_contract_struct_ids(element, output),
        ContractType::Struct { id } => {
            output.insert(id.clone());
        }
        ContractType::Bool | ContractType::BitVector { .. } => {}
    }
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
