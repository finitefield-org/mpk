//! Source-derived finite dispatch target selection, separate from callee runs.
//! A zero ordinal denotes no selected target; it grants no nil safety proof.
use super::syntax::*;
use super::{
    error_layout, generate_go_practical_source_error_case_catalogue, word, Builder,
    GoOrdinaryError, GoSourceErrorCaseCatalogue, Result, Sha256, MAX_BYTES,
};
use crate::go_practical_type_keys::{
    GoErrorImplementation, GoSourceErrorCall, GoSourceErrorSelection,
};
use serde_json::json;
use sha2::Digest;

pub struct GoSourceErrorDispatchCatalogue {
    cases: GoSourceErrorCaseCatalogue,
    calls: Vec<GoSourceErrorCall>,
    targets: Vec<GoErrorImplementation>,
    binding: String,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoSourceErrorDispatchRow {
    module: String,
    source_module: String,
    boxing_module: String,
    operations_module: String,
    case_row_module: Option<String>,
    call: GoSourceErrorCall,
    targets: Vec<GoErrorImplementation>,
    cases: Vec<(String, u64)>,
    boxed_type: Option<String>,
    certificate: Vec<u8>,
}
impl GoSourceErrorDispatchRow {
    pub fn module_name(&self) -> &str {
        &self.module
    }
    pub fn source_layout_module_name(&self) -> &str {
        &self.source_module
    }
    pub fn boxing_module_name(&self) -> &str {
        &self.boxing_module
    }
    pub fn operations_module_name(&self) -> &str {
        &self.operations_module
    }
    pub fn case_row_module_name(&self) -> Option<&str> {
        self.case_row_module.as_deref()
    }
    pub fn call(&self) -> &GoSourceErrorCall {
        &self.call
    }
    pub fn targets(&self) -> &[GoErrorImplementation] {
        &self.targets
    }
    pub fn case_tags(&self) -> &[(String, u64)] {
        &self.cases
    }
    pub fn boxed_source_type_id(&self) -> Option<&str> {
        self.boxed_type.as_deref()
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
}
fn member(module: &str, name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("{module}.{name}"), args)
}
fn natural(n: u64) -> Expr {
    (0..n).fold(zero(), |v, _| succ(v))
}
fn target_name(target: &GoErrorImplementation) -> String {
    let method = target.method();
    format!(
        "Target.C{}.S{}.N{}",
        target.value_type_id(),
        method
            .declaration_source_object_id()
            .expect("checked original method"),
        method.declaration_node().expect("checked original method")
    )
}
fn call_binding(call: &GoSourceErrorCall) -> serde_json::Value {
    json!([
        call.caller().source_object_id(),
        call.caller().node(),
        call.call_node(),
        call.receiver_node(),
        call.method_expression()
    ])
}
fn target_binding(target: &GoErrorImplementation) -> serde_json::Value {
    let m = target.method();
    json!([
        target.value_type_id(),
        m.declaration_source_object_id(),
        m.declaration_node(),
        m.body_node(),
        m.name(),
        m.receiver_type_id(),
        m.receiver_path(),
        m.parameter_type_ids(),
        m.result_type_ids()
    ])
}
struct Selector<'a> {
    builder: &'a mut Builder,
    module: &'a str,
    boxing: &'a str,
    operations: &'a str,
    types: Vec<Expr>,
    targets: &'a [GoErrorImplementation],
}
impl Selector<'_> {
    fn own(&self, name: &str, args: Vec<Expr>) -> Expr {
        member(self.module, name, args)
    }
    fn suffix(&self, i: usize, e: Expr) -> Expr {
        self.own(&format!("SelectFrom{i}"), vec![e])
    }
    fn target(&self, i: usize) -> Expr {
        self.own(&target_name(&self.targets[i]), vec![])
    }
    fn matched(&self, i: usize, e: Expr) -> Expr {
        member(
            self.operations,
            "HasCase",
            vec![
                member(
                    &format!("{}.C{}", self.boxing, self.targets[i].value_type_id()),
                    "Tag",
                    vec![],
                ),
                e,
            ],
        )
    }
    fn emit(&mut self, name: &str, args: &[Binder], ty: Expr, body: Expr, law: bool) -> Result<()> {
        emit(
            self.builder,
            &format!("{}.{name}", self.module),
            args,
            ty,
            body,
            law,
        )
    }
    fn definitions(&mut self) -> Result<()> {
        let carrier = error_layout::product(&self.types);
        self.emit("NoTarget", &[], nat(), zero(), false)?;
        for i in 0..self.targets.len() {
            self.emit(
                &target_name(&self.targets[i]),
                &[],
                nat(),
                natural((i as u64) + 1),
                false,
            )?;
        }
        let args = [("e", carrier.clone())];
        self.emit(
            &format!("SelectFrom{}", self.targets.len()),
            &args,
            nat(),
            self.own("NoTarget", vec![]),
            false,
        )?;
        for i in (0..self.targets.len()).rev() {
            let condition = self.matched(i, var("e"));
            let next = self.suffix(i + 1, var("e"));
            self.emit(
                &format!("SelectFrom{i}"),
                &args,
                nat(),
                word::choose(condition, self.target(i), next, nat()),
                false,
            )?;
        }
        self.emit("Select", &args, nat(), self.suffix(0, var("e")), false)?;
        for i in 0..self.targets.len() {
            let condition = self.matched(i, var("e"));
            let next = self.suffix(i + 1, var("e"));
            let target = self.target(i);
            let left = self.suffix(i, var("e"));
            let choice = |b| word::choose(b, target.clone(), next.clone(), nat());
            let yes = Equality::boolean(condition.clone(), bit(true), var("matched")).mapped(
                lam("_match", boolean(), choice(var("_match"))),
                left.clone(),
                choice(bit(true)),
                nat(),
            );
            self.emit(
                &format!("SelectTrue{i}"),
                &[
                    ("e", carrier.clone()),
                    ("matched", eq(boolean(), condition.clone(), bit(true))),
                ],
                eq(nat(), left.clone(), target.clone()),
                yes.proof,
                true,
            )?;
            let no = Equality::boolean(condition.clone(), bit(false), var("not_matched")).mapped(
                lam("_match", boolean(), choice(var("_match"))),
                left.clone(),
                choice(bit(false)),
                nat(),
            );
            let proof = Equality::natural(left.clone(), next.clone(), no.proof)
                .then(Equality::natural(next.clone(), var("result"), var("tail")))?;
            self.emit(
                &format!("SelectFalse{i}"),
                &[
                    ("e", carrier.clone()),
                    ("not_matched", eq(boolean(), condition, bit(false))),
                    ("result", nat()),
                    ("tail", eq(nat(), next, var("result"))),
                ],
                eq(nat(), left, var("result")),
                proof.proof,
                true,
            )?;
        }
        Ok(())
    }
    fn value_laws(&mut self, boxed: Option<(&str, &str)>, source_module: &str) -> Result<()> {
        let args = boxed
            .map(|(id, _)| vec![("v", constant(&format!("{source_module}.T{id}.Carrier")))])
            .unwrap_or_default();
        let value = if let Some((id, _)) = boxed {
            member(&format!("{}.C{id}", self.boxing), "Box", vec![var("v")])
        } else {
            member(self.operations, "Nil", vec![])
        };
        let result = self.own("NoTarget", vec![]);
        self.emit(
            &format!("Suffix{}OfValue", self.targets.len()),
            &args,
            eq(
                nat(),
                self.suffix(self.targets.len(), value.clone()),
                result.clone(),
            ),
            refl(nat(), result),
            true,
        )?;
        for i in (0..self.targets.len()).rev() {
            let id = self.targets[i].value_type_id();
            let is_own = boxed.is_some_and(|(own, _)| own == id);
            let target = if is_own {
                self.target(i)
            } else if let Some((boxed, _)) = boxed {
                let own = self
                    .targets
                    .iter()
                    .position(|t| t.value_type_id() == boxed)
                    .ok_or(GoOrdinaryError::Linkage)?;
                if i < own {
                    self.target(own)
                } else {
                    self.own("NoTarget", vec![])
                }
            } else {
                self.own("NoTarget", vec![])
            };
            let matched = if let Some((_, case_module)) = boxed {
                member(case_module, &format!("Matches.C{id}"), vec![var("v")])
            } else {
                member(self.operations, &format!("C{id}.NilHasNoCase"), vec![])
            };
            let proof = if is_own {
                self.own(&format!("SelectTrue{i}"), vec![value.clone(), matched])
            } else {
                let tail = self.own(
                    &format!("Suffix{}OfValue", i + 1),
                    if boxed.is_some() {
                        vec![var("v")]
                    } else {
                        vec![]
                    },
                );
                self.own(
                    &format!("SelectFalse{i}"),
                    vec![value.clone(), matched, target.clone(), tail],
                )
            };
            self.emit(
                &format!("Suffix{i}OfValue"),
                &args,
                eq(nat(), self.suffix(i, value.clone()), target),
                proof,
                true,
            )?;
        }
        let expected = boxed
            .map(|(id, _)| {
                self.targets
                    .iter()
                    .position(|t| t.value_type_id() == id)
                    .map(|i| self.target(i))
                    .ok_or(GoOrdinaryError::Linkage)
            })
            .transpose()?
            .unwrap_or_else(|| self.own("NoTarget", vec![]));
        self.emit(
            if boxed.is_some() {
                "SelectedOfBox"
            } else {
                "SelectedOfNil"
            },
            &args,
            eq(nat(), self.own("Select", vec![value]), expected),
            self.own(
                "Suffix0OfValue",
                if boxed.is_some() {
                    vec![var("v")]
                } else {
                    vec![]
                },
            ),
            true,
        )
    }
}
impl GoSourceErrorDispatchCatalogue {
    pub fn calls(&self) -> &[GoSourceErrorCall] {
        &self.calls
    }
    pub fn targets(&self) -> &[GoErrorImplementation] {
        &self.targets
    }
    pub fn case_tags(&self) -> &[(String, u64)] {
        self.cases.case_tags()
    }
    pub fn prior_operations_certificate_bytes(&self) -> &[u8] {
        self.cases.prior_operations_certificate_bytes()
    }
    pub fn generate_row(
        &self,
        call: &GoSourceErrorCall,
        boxed_type: Option<&str>,
    ) -> Result<GoSourceErrorDispatchRow> {
        let (builder, row) = self.row_builder(call, boxed_type)?;
        complete_row(builder, row)
    }
    pub(super) fn row_builder(
        &self,
        call: &GoSourceErrorCall,
        boxed_type: Option<&str>,
    ) -> Result<(Builder, GoSourceErrorDispatchRow)> {
        if !self.calls.contains(call) {
            return Err(GoOrdinaryError::Linkage);
        }
        let (mut builder, case_module) = if let Some(id) = boxed_type {
            let (b, row) = self.cases.row_builder(id)?;
            (b, Some(row.module_name().to_string()))
        } else {
            (self.cases.base_builder(), None)
        };
        let bytes = serde_json::to_vec(&(call_binding(call), boxed_type))
            .map_err(|_| GoOrdinaryError::Linkage)?;
        let mut h = Sha256::new();
        h.update(b"MPK-GO-SOURCE-ERROR-DISPATCH-ROW-PREPARATION-1\0");
        h.update(self.binding.as_bytes());
        h.update(bytes);
        let module = format!(
            "{}.Dispatch.S{:x}",
            self.cases.operations_module_name(),
            h.finalize()
        );
        let types = std::iter::once(word::word_type(64)?)
            .chain(self.cases.case_tags().iter().map(|(id, _)| {
                constant(&format!(
                    "{}.T{id}.Carrier",
                    self.cases.source_layout_module_name()
                ))
            }))
            .collect();
        let mut generator = Selector {
            builder: &mut builder,
            module: &module,
            boxing: self.cases.boxing_module_name(),
            operations: self.cases.operations_module_name(),
            types,
            targets: &self.targets,
        };
        generator.definitions()?;
        generator.value_laws(
            boxed_type.zip(case_module.as_deref()),
            self.cases.source_layout_module_name(),
        )?;
        builder.certificate.module = module.clone();
        let row = GoSourceErrorDispatchRow {
            module,
            source_module: self.cases.source_layout_module_name().into(),
            boxing_module: self.cases.boxing_module_name().into(),
            operations_module: self.cases.operations_module_name().into(),
            case_row_module: case_module,
            call: call.clone(),
            targets: self.targets.clone(),
            cases: self.cases.case_tags().to_vec(),
            boxed_type: boxed_type.map(str::to_string),
            certificate: vec![],
        };
        Ok((builder, row))
    }
    pub fn import_row(
        &self,
        candidate: &[u8],
        call: &GoSourceErrorCall,
        boxed_type: Option<&str>,
    ) -> Result<GoSourceErrorDispatchRow> {
        if candidate.len() > MAX_BYTES {
            return Err(GoOrdinaryError::Limit);
        }
        let expected = self.generate_row(call, boxed_type)?;
        if candidate != expected.certificate_bytes() {
            return Err(GoOrdinaryError::Linkage);
        }
        Ok(expected)
    }
}
pub(super) fn complete_row(
    builder: Builder,
    mut row: GoSourceErrorDispatchRow,
) -> Result<GoSourceErrorDispatchRow> {
    row.certificate = builder.finish()?;
    Ok(row)
}
pub fn generate_go_practical_source_error_dispatch_catalogue(
    selection: &GoSourceErrorSelection,
) -> Result<GoSourceErrorDispatchCatalogue> {
    let cases = generate_go_practical_source_error_case_catalogue(selection.domain())?;
    let targets = selection.domain().error_cases().to_vec();
    let calls = selection
        .dispatches()
        .iter()
        .map(|d| {
            if d.targets() != targets {
                return Err(GoOrdinaryError::Linkage);
            }
            Ok(d.call().clone())
        })
        .collect::<Result<Vec<_>>>()?;
    let preimage = serde_json::to_vec(&(
        cases.source_layout_module_name(),
        calls.iter().map(call_binding).collect::<Vec<_>>(),
        targets.iter().map(target_binding).collect::<Vec<_>>(),
    ))
    .map_err(|_| GoOrdinaryError::Linkage)?;
    let mut h = Sha256::new();
    h.update(b"MPK-GO-SOURCE-ERROR-DISPATCH-PREPARATION-1\0");
    h.update(preimage);
    Ok(GoSourceErrorDispatchCatalogue {
        cases,
        calls,
        targets,
        binding: format!("{:x}", h.finalize()),
    })
}
