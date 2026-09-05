//! W12 private presence/outcome relations. Bindings are candidates with pending
//! universal obligations, never a license to drop source fields or exceptions.
use super::*;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomainError {
    Signature,
    OperandType,
    InactivePayload,
    EmptyInvalid,
    Bound,
    DefaultIneligible,
    Binding,
    ObservationLoss,
    Numeric(NumericError),
}
impl DomainError {
    pub fn exception_type(self) -> Option<&'static str> {
        match self {
            Self::InactivePayload => Some("System.InvalidOperationException"),
            Self::Numeric(e) => e.exception_type(),
            _ => None,
        }
    }
}
fn ty(t: &str) -> String {
    format!("mpk.csharp.value.{t}.v1")
}
fn bool_value(value: bool) -> MonomorphicValue {
    MonomorphicValue::Bool {
        type_id: ty("bool"),
        value,
    }
}
fn arms(role: &str) -> Option<&'static [&'static str]> {
    Some(match role {
        "option" => &["none", "some"],
        "lookup" => &["missing_key", "found"],
        "result" => &["ok", "error"],
        "validation" => &["valid", "invalid"],
        "boundary_field" => &["missing", "null", "value"],
        _ => return None,
    })
}

pub struct OutcomeModel<'a> {
    bundle: &'a ValidatedFoundationBundle,
    roots: &'a ValidatedClosedRootSet,
    closed: &'a ClosedInstanceSet,
    id: String,
    role: String,
    args: Vec<String>,
}
impl<'a> OutcomeModel<'a> {
    pub fn new(
        bundle: &'a ValidatedFoundationBundle,
        roots: &'a ValidatedClosedRootSet,
        closed: &'a ClosedInstanceSet,
        id: &str,
    ) -> Result<Self, DomainError> {
        let m = closed.metadata.get(id).ok_or(DomainError::Signature)?;
        let role = template_name(&m.template_id)
            .filter(|r| arms(r).is_some())
            .ok_or(DomainError::Signature)?;
        Ok(Self {
            bundle,
            roots,
            closed,
            id: id.into(),
            role: role.into(),
            args: m.argument_ids.clone(),
        })
    }
    pub fn type_id(&self) -> &str {
        &self.id
    }
    pub fn role(&self) -> &str {
        &self.role
    }
    pub fn payload_type_ids(&self) -> &[String] {
        &self.args
    }
    fn valid(&self, v: &MonomorphicValue) -> Result<(), DomainError> {
        if v.type_id() != self.id
            || validate_monomorphic_value(self.bundle, self.roots, self.closed, v).is_err()
        {
            Err(DomainError::OperandType)
        } else {
            Ok(())
        }
    }
    pub fn construct(
        &self,
        arm: &str,
        payload: Option<MonomorphicValue>,
    ) -> Result<MonomorphicValue, DomainError> {
        if !arms(&self.role).unwrap().contains(&arm) {
            return Err(DomainError::Signature);
        }
        if self.role == "validation" && arm == "invalid" {
            if let Some(MonomorphicValue::Sequence { elements, .. }) = &payload {
                if elements.is_empty() {
                    return Err(DomainError::EmptyInvalid);
                }
                if elements.len() > VALIDATION_ERRORS_MAX as usize {
                    return Err(DomainError::Bound);
                }
            }
        }
        let result = match self.role.as_str() {
            "option" => MonomorphicValue::Option {
                type_id: self.id.clone(),
                arm: if arm == "none" {
                    OptionArm::None
                } else {
                    OptionArm::Some
                },
                value: payload.map(Box::new),
            },
            "boundary_field" => MonomorphicValue::BoundaryPresence {
                type_id: self.id.clone(),
                arm: match arm {
                    "missing" => BoundaryArm::Missing,
                    "null" => BoundaryArm::Null,
                    _ => BoundaryArm::Value,
                },
                value: payload.map(Box::new),
            },
            _ => MonomorphicValue::TaggedSum {
                type_id: self.id.clone(),
                arm: arm.into(),
                payload: payload.into_iter().collect(),
            },
        };
        self.valid(&result)?;
        Ok(result)
    }
    pub fn arm<'v>(&self, value: &'v MonomorphicValue) -> Result<&'v str, DomainError> {
        self.valid(value)?;
        Ok(match value {
            MonomorphicValue::Option {
                arm: OptionArm::None,
                ..
            } => "none",
            MonomorphicValue::Option { .. } => "some",
            MonomorphicValue::BoundaryPresence {
                arm: BoundaryArm::Missing,
                ..
            } => "missing",
            MonomorphicValue::BoundaryPresence {
                arm: BoundaryArm::Null,
                ..
            } => "null",
            MonomorphicValue::BoundaryPresence { .. } => "value",
            MonomorphicValue::TaggedSum { arm, .. } => arm,
            _ => unreachable!(),
        })
    }
    pub fn read<'v>(
        &self,
        value: &'v MonomorphicValue,
        arm: &str,
    ) -> Result<&'v MonomorphicValue, DomainError> {
        if !arms(&self.role).unwrap().contains(&arm) {
            return Err(DomainError::Signature);
        }
        if self.arm(value)? != arm {
            return Err(DomainError::InactivePayload);
        }
        match value {
            MonomorphicValue::Option {
                value: Some(value), ..
            }
            | MonomorphicValue::BoundaryPresence {
                value: Some(value), ..
            } => Ok(value),
            MonomorphicValue::TaggedSum { payload, .. } => {
                payload.first().ok_or(DomainError::Signature)
            }
            _ => Err(DomainError::Signature),
        }
    }
    /// Fallback is already evaluated, unlike the lazy coalesce callback below.
    /// Its public invariant remains a source obligation even on the some arm.
    pub fn value_or(
        &self,
        value: &MonomorphicValue,
        fallback: MonomorphicValue,
    ) -> Result<MonomorphicValue, DomainError> {
        if self.role != "option" {
            return Err(DomainError::Signature);
        }
        if fallback.type_id() != self.args[0]
            || validate_monomorphic_value(self.bundle, self.roots, self.closed, &fallback).is_err()
        {
            return Err(DomainError::OperandType);
        }
        if self.arm(value)? == "none" {
            Ok(fallback)
        } else {
            Ok(self.read(value, "some")?.clone())
        }
    }
    pub fn value_or_default(
        &self,
        value: &MonomorphicValue,
    ) -> Result<MonomorphicValue, DomainError> {
        if self.role != "option" {
            return Err(DomainError::Signature);
        }
        let fallback = domain_default(self.bundle, self.roots, self.closed, &self.args[0])?;
        self.value_or(value, fallback)
    }
    pub fn coalesce(
        &self,
        value: &MonomorphicValue,
        fallback: impl FnOnce() -> Result<MonomorphicValue, DomainError>,
    ) -> Result<MonomorphicValue, DomainError> {
        if self.role != "option" {
            return Err(DomainError::Signature);
        }
        if self.arm(value)? == "some" {
            Ok(self.read(value, "some")?.clone())
        } else {
            self.value_or(value, fallback()?)
        }
    }
    pub fn exhaustive(&self, selected: &[String]) -> Result<(), DomainError> {
        let set: BTreeSet<_> = selected.iter().map(String::as_str).collect();
        if set.len() != selected.len() || set != arms(&self.role).unwrap().iter().copied().collect()
        {
            Err(DomainError::Signature)
        } else {
            Ok(())
        }
    }
    pub fn append_errors(
        &self,
        left: &MonomorphicValue,
        right: &MonomorphicValue,
    ) -> Result<MonomorphicValue, DomainError> {
        if self.role != "validation" {
            return Err(DomainError::Signature);
        }
        let elements = |v: &MonomorphicValue| -> Result<Vec<MonomorphicValue>, DomainError> {
            match self.read(v, "invalid")? {
                MonomorphicValue::Sequence { elements, .. } => Ok(elements.clone()),
                _ => Err(DomainError::OperandType),
            }
        };
        let mut output = elements(left)?;
        let rhs = elements(right)?;
        if output.len() + rhs.len() > VALIDATION_ERRORS_MAX as usize {
            return Err(DomainError::Bound);
        }
        output.extend(rhs);
        let id = self.closed.metadata[&self.id].dependency_ids[0].clone();
        self.construct(
            "invalid",
            Some(MonomorphicValue::Sequence {
                type_id: id,
                elements: output,
            }),
        )
    }
    /// Exact same-type lift; both concrete operands have already evaluated.
    /// Absence suppresses only the underlying operation, not operand evaluation.
    pub fn lift(
        &self,
        operation: &str,
        operands: &[MonomorphicValue],
        checked: bool,
    ) -> Result<MonomorphicValue, DomainError> {
        if self.role != "option" {
            return Err(DomainError::Signature);
        }
        let token = self.args[0]
            .strip_prefix("mpk.csharp.value.")
            .and_then(|s| s.strip_suffix(".v1"))
            .ok_or(DomainError::Signature)?;
        let comparison = matches!(
            operation,
            "equal" | "not_equal" | "less" | "less_equal" | "greater" | "greater_equal"
        );
        let unary = matches!(operation, "plus" | "negate" | "not");
        let allowed = if token == "bool" {
            matches!(operation, "not" | "and" | "or" | "equal" | "not_equal")
        } else {
            matches!(token, "i32" | "i64" | "f32" | "f64" | "decimal")
                && (comparison
                    || matches!(
                        operation,
                        "plus"
                            | "negate"
                            | "add"
                            | "subtract"
                            | "multiply"
                            | "divide"
                            | "remainder"
                    ))
        };
        if !allowed || operands.len() != if unary { 1 } else { 2 } {
            return Err(DomainError::Signature);
        }
        let values: Vec<_> = operands
            .iter()
            .map(|v| {
                self.valid(v)?;
                Ok(if self.arm(v)? == "none" {
                    None
                } else {
                    Some(self.read(v, "some")?)
                })
            })
            .collect::<Result<_, DomainError>>()?;
        if token == "bool" && matches!(operation, "and" | "or" | "not") {
            let a = values[0].map(|v| match v {
                MonomorphicValue::Bool { value, .. } => *value,
                _ => unreachable!(),
            });
            let b = values.get(1).copied().flatten().map(|v| match v {
                MonomorphicValue::Bool { value, .. } => *value,
                _ => unreachable!(),
            });
            let result = match operation {
                "not" => a.map(|v| !v),
                "and" => {
                    if a == Some(false) || b == Some(false) {
                        Some(false)
                    } else {
                        a.zip(b).map(|(a, b)| a && b)
                    }
                }
                _ => {
                    if a == Some(true) || b == Some(true) {
                        Some(true)
                    } else {
                        a.zip(b).map(|(a, b)| a || b)
                    }
                }
            };
            return self.construct(
                if result.is_some() { "some" } else { "none" },
                result.map(bool_value),
            );
        }
        if values.iter().any(Option::is_none) {
            return if comparison {
                Ok(bool_value(match operation {
                    "equal" => values.iter().all(Option::is_none),
                    "not_equal" => !values.iter().all(Option::is_none),
                    _ => false,
                }))
            } else {
                self.construct("none", None)
            };
        }
        let args: Vec<_> = values.into_iter().map(|v| v.unwrap().clone()).collect();
        let value = if matches!(token, "f32" | "f64" | "decimal") {
            let prefix = match token {
                "f32" => "floating.single",
                "f64" => "floating.double",
                _ => "decimal",
            };
            NumericOperation::new(
                &format!("{prefix}.{operation}"),
                &args
                    .iter()
                    .map(|v| v.type_id().to_owned())
                    .collect::<Vec<_>>(),
                &if comparison {
                    ty("bool")
                } else {
                    self.args[0].clone()
                },
                None,
            )
            .map_err(DomainError::Numeric)?
            .evaluate(self.bundle, self.roots, self.closed, &args)
            .map_err(DomainError::Numeric)?
        } else if comparison {
            let order = relate_monomorphic_values(
                self.bundle,
                self.roots,
                self.closed,
                true,
                &args[0],
                &args[1],
            )
            .map_err(|_| DomainError::OperandType)?;
            bool_value(match operation {
                "equal" => order == Ordering::Equal,
                "not_equal" => order != Ordering::Equal,
                "less" => order == Ordering::Less,
                "less_equal" => order != Ordering::Greater,
                "greater" => order == Ordering::Greater,
                _ => order != Ordering::Less,
            })
        } else {
            let get = |v: &MonomorphicValue| -> i128 {
                let MonomorphicValue::Signed { value, .. } = v else {
                    unreachable!()
                };
                value.parse().unwrap()
            };
            let a = get(&args[0]);
            let b = args.get(1).map(get).unwrap_or(0);
            let width = if token == "i32" { 32 } else { 64 };
            let min = -(1i128 << (width - 1));
            let max = (1i128 << (width - 1)) - 1;
            if matches!(operation, "divide" | "remainder") && b == 0 {
                return Err(DomainError::Numeric(NumericError::DivideByZero));
            }
            if matches!(operation, "divide" | "remainder") && a == min && b == -1 {
                return Err(DomainError::Numeric(NumericError::Overflow));
            }
            let n = match operation {
                "plus" => a,
                "negate" => -a,
                "add" => a + b,
                "subtract" => a - b,
                "multiply" => a * b,
                "divide" => a / b,
                "remainder" => a % b,
                _ => unreachable!(),
            };
            if checked && !(min..=max).contains(&n) {
                return Err(DomainError::Numeric(NumericError::Overflow));
            }
            let n = if width == 32 {
                i128::from(n as i32)
            } else {
                i128::from(n as i64)
            };
            MonomorphicValue::Signed {
                type_id: self.args[0].clone(),
                value: n.to_string(),
            }
        };
        if comparison {
            Ok(value)
        } else {
            self.construct("some", Some(value))
        }
    }
}

/// Recursive CLR defaults; a source declaration's public-default condition is
/// still a pending invariant obligation in its source handoff.
pub fn domain_default(
    b: &ValidatedFoundationBundle,
    r: &ValidatedClosedRootSet,
    c: &ClosedInstanceSet,
    id: &str,
) -> Result<MonomorphicValue, DomainError> {
    let result = if let Some(source) = r.source_types.get(id) {
        if !source.public_default || source.kind == SourceKind::SealedClass {
            return Err(DomainError::DefaultIneligible);
        }
        if source.kind == SourceKind::Enum {
            if !source.enum_values.iter().any(|v| v == "0") {
                return Err(DomainError::DefaultIneligible);
            }
            MonomorphicValue::Enum {
                type_id: id.into(),
                underlying: source.enum_underlying.clone().unwrap(),
                carrier: "0".into(),
            }
        } else {
            let fields = source
                .members
                .iter()
                .map(|m| {
                    Ok(NamedMonomorphicValue {
                        name: m.name.clone(),
                        value: Box::new(domain_default(
                            b,
                            r,
                            c,
                            &closed_type_id(b, &m.ty)
                                .map_err(|_| DomainError::DefaultIneligible)?,
                        )?),
                    })
                })
                .collect::<Result<_, DomainError>>()?;
            MonomorphicValue::Product {
                type_id: id.into(),
                fields,
            }
        }
    } else if let Some(meta) = c.metadata.get(id) {
        match template_name(&meta.template_id) {
            Some("option") => OutcomeModel::new(b, r, c, id)?.construct("none", None)?,
            Some("lookup") => OutcomeModel::new(b, r, c, id)?.construct("missing_key", None)?,
            _ => return Err(DomainError::DefaultIneligible),
        }
    } else {
        let token = id
            .strip_prefix("mpk.csharp.value.")
            .and_then(|s| s.strip_suffix(".v1"))
            .ok_or(DomainError::DefaultIneligible)?;
        match token {
            "bool" => bool_value(false),
            "char" => MonomorphicValue::Char {
                type_id: id.into(),
                utf16: 0,
            },
            "f32" => MonomorphicValue::F32Bits {
                type_id: id.into(),
                bits: "00000000".into(),
            },
            "f64" => MonomorphicValue::F64Bits {
                type_id: id.into(),
                bits: "0000000000000000".into(),
            },
            "decimal" => MonomorphicValue::DecimalBits {
                type_id: id.into(),
                negative: false,
                scale: 0,
                coefficient: "0".into(),
            },
            "date" => MonomorphicValue::Date {
                type_id: id.into(),
                day_number: 0,
            },
            "time" => MonomorphicValue::Time {
                type_id: id.into(),
                ticks: "0".into(),
            },
            "duration" => MonomorphicValue::Duration {
                type_id: id.into(),
                ticks: "0".into(),
            },
            "instant" => MonomorphicValue::Instant {
                type_id: id.into(),
                milliseconds: "0".into(),
            },
            "guid" => MonomorphicValue::Guid {
                type_id: id.into(),
                n: "0".repeat(32),
            },
            "i8" | "i16" | "i32" | "i64" => MonomorphicValue::Signed {
                type_id: id.into(),
                value: "0".into(),
            },
            "u8" | "u16" | "u32" | "u64" => MonomorphicValue::Unsigned {
                type_id: id.into(),
                value: "0".into(),
            },
            _ => return Err(DomainError::DefaultIneligible),
        }
    };
    validate_monomorphic_value(b, r, c, &result).map_err(|_| DomainError::DefaultIneligible)?;
    Ok(result)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutcomeObligation {
    pub source_type_id: String,
    pub semantic_type_id: String,
    pub kind: String,
    pub member_id: String,
    pub discharged: bool,
}
#[derive(Clone, Debug)]
pub struct OutcomeBindingPlan {
    source: String,
    semantic: String,
    role: String,
    members: BTreeMap<String, String>,
    tags: BTreeMap<String, String>,
    obligations: Vec<OutcomeObligation>,
    default_eligible: bool,
    dependencies: BTreeMap<String, std::sync::Arc<OutcomeBindingPlan>>,
}
impl OutcomeBindingPlan {
    /// Consume the content-bound T02 binding view. Callers must validate its
    /// enclosing artifact first. Method signatures are the captured source view,
    /// not a caller claim that commutation or an exception has been proved.
    pub fn new(
        b: &ValidatedFoundationBundle,
        r: &ValidatedClosedRootSet,
        c: &ClosedInstanceSet,
        input: &crate::csharp_practical_source_artifacts::SemanticBindingInput,
        captured: &BTreeMap<String, ClosedOperationSignature>,
    ) -> Result<Self, DomainError> {
        Self::new_with_dependencies(b, r, c, input, captured, &[])
    }
    /// Previously validated application bindings supply typed payload projections.
    /// Their pending universal obligations remain pending in the dependency DAG.
    pub fn new_with_dependencies(
        b: &ValidatedFoundationBundle,
        r: &ValidatedClosedRootSet,
        c: &ClosedInstanceSet,
        input: &crate::csharp_practical_source_artifacts::SemanticBindingInput,
        captured: &BTreeMap<String, ClosedOperationSignature>,
        dependencies: &[Self],
    ) -> Result<Self, DomainError> {
        let dependency_count = dependencies.len();
        let dependencies: BTreeMap<_, _> = dependencies
            .iter()
            .map(|p| (p.source.clone(), std::sync::Arc::new(p.clone())))
            .collect();
        if dependencies.len() != dependency_count
            || dependencies.contains_key(&input.source_type_id)
            || dependencies
                .values()
                .any(|p| !c.metadata.contains_key(&p.semantic))
        {
            return Err(DomainError::Binding);
        }
        let argument_id = |t: &ClosedType| -> Result<String, DomainError> {
            let id = closed_type_id(b, t).map_err(|_| DomainError::Binding)?;
            Ok(dependencies
                .get(&id)
                .map(|p| p.semantic.clone())
                .unwrap_or(id))
        };
        let source = r
            .source_types
            .get(&input.source_type_id)
            .filter(|s| {
                s.kind != SourceKind::Enum && s.source_sha256 == input.source_content_sha256
            })
            .ok_or(DomainError::Binding)?;
        let arm_names = arms(&input.role).ok_or(DomainError::Binding)?;
        let expected_members: Vec<&str> = match input.role.as_str() {
            "result" => vec!["tag", "value", "error"],
            "validation" => vec!["tag", "value", "errors"],
            _ => vec!["tag", "value"],
        };
        let members: BTreeMap<_, _> = input
            .member_map
            .iter()
            .map(|m| (m.role.clone(), m.member_id.clone()))
            .collect();
        if members.len() != input.member_map.len()
            || members.keys().map(String::as_str).collect::<BTreeSet<_>>()
                != expected_members.iter().copied().collect()
            || members.values().collect::<BTreeSet<_>>().len() != members.len()
        {
            return Err(DomainError::Binding);
        }
        let member = |role: &str| {
            source
                .members
                .iter()
                .find(|m| m.id == members[role])
                .ok_or(DomainError::Binding)
        };
        let tag = member("tag")?;
        let ClosedType::Source(enum_id) = &tag.ty else {
            return Err(DomainError::Binding);
        };
        let en = r
            .source_types
            .get(enum_id)
            .filter(|s| s.kind == SourceKind::Enum)
            .ok_or(DomainError::Binding)?;
        let tags: BTreeMap<_, _> = input
            .tag_arms
            .iter()
            .map(|a| (a.semantic_arm.clone(), a.source_tag.clone()))
            .collect();
        if tags.len() != input.tag_arms.len()
            || tags.keys().map(String::as_str).collect::<BTreeSet<_>>()
                != arm_names.iter().copied().collect()
            || tags.values().collect::<BTreeSet<_>>() != en.enum_values.iter().collect()
            || tags.values().collect::<BTreeSet<_>>().len() != tags.len()
        {
            return Err(DomainError::Binding);
        }
        let mut args = vec![argument_id(&member("value")?.ty)?];
        if input.role == "result" {
            args.push(argument_id(&member("error")?.ty)?);
        }
        if input.role == "validation" {
            let ClosedType::Instance {
                template,
                arguments,
            } = &member("errors")?.ty
            else {
                return Err(DomainError::Binding);
            };
            if template != "bounded_sequence" {
                return Err(DomainError::Binding);
            }
            args.push(argument_id(&arguments[0])?);
        }
        if args != input.inferred_argument_ids {
            return Err(DomainError::Binding);
        }
        let semantic = c
            .metadata
            .iter()
            .find(|(_, m)| {
                template_name(&m.template_id) == Some(input.role.as_str()) && m.argument_ids == args
            })
            .map(|(id, _)| id.clone())
            .ok_or(DomainError::Binding)?;
        let default = match input.role.as_str() {
            "option" => "none",
            "lookup" => "missing_key",
            _ => "ineligible",
        };
        if input.default_arm != default
            || if input.role == "validation" {
                input.bounds.len() != 1
                    || input.bounds[0].id != "errors"
                    || input.bounds[0].maximum != VALIDATION_ERRORS_MAX
            } else {
                !input.bounds.is_empty()
            }
        {
            return Err(DomainError::Binding);
        }
        let mut plan = Self {
            source: source.id.clone(),
            semantic,
            role: input.role.clone(),
            members,
            tags,
            obligations: vec![],
            default_eligible: false,
            dependencies,
        };
        for kind in [
            "source_invariant_implies_projection",
            "semantic_invariant_implies_reconstruction",
            "source_round_trip",
            "semantic_round_trip",
            "distinct_arms",
            "public_invariant",
            "identity_unobservable",
        ] {
            plan.obligation(kind, "");
        }
        if default != "ineligible" {
            plan.obligation("actual_default_public_invariant", "");
            plan.default_eligible = source.kind == SourceKind::ReadonlyStruct
                && source.public_default
                && plan.tags[default] == "0"
                && domain_default(b, r, c, &source.id).is_ok();
        }
        for m in &source.members {
            plan.obligation("field_complete_reconstruction", &m.id);
        }
        let entry = c
            .entries()
            .iter()
            .find(|v| v["instance_id"] == plan.semantic)
            .ok_or(DomainError::Binding)?;
        let mut seen = BTreeSet::new();
        for mapping in &input.operation_map {
            if !seen.insert(&mapping.operation) {
                return Err(DomainError::Binding);
            }
            let full = format!("{}.{}", plan.semantic, mapping.operation);
            let definition = entry["operation_definitions"]
                .as_array()
                .unwrap()
                .iter()
                .find(|d| d["id"] == full)
                .ok_or(DomainError::Binding)?;
            let source_call = captured
                .get(&mapping.member_id)
                .filter(|s| s.id == mapping.member_id && s.tag == ClosedOperationTag::SourceCall)
                .ok_or(DomainError::Binding)?;
            validate_closed_operation_signature(r, c, source_call)
                .map_err(|_| DomainError::Binding)?;
            let projection = |id: &String| {
                if id == &source.id {
                    plan.semantic.clone()
                } else if let Some(dependency) = plan.dependencies.get(id) {
                    dependency.semantic.clone()
                } else {
                    id.clone()
                }
            };
            let expected = json_string_array(definition["argument_type_ids"].as_array().unwrap())
                .ok_or(DomainError::Binding)?;
            if source_call
                .argument_type_ids
                .iter()
                .map(&projection)
                .collect::<Vec<_>>()
                != expected
                || projection(&source_call.normal_result_type_id)
                    != definition["normal_result_type_id"].as_str().unwrap()
            {
                return Err(DomainError::Binding);
            }
            for kind in [
                "operation_normal_commutation",
                "operation_error_commutation",
                "operation_exception_commutation",
            ] {
                plan.obligation(kind, &mapping.member_id);
            }
        }
        if plan.obligations.len() > PROJECTION_OBLIGATIONS_PER_BINDING_MAX as usize {
            return Err(DomainError::Bound);
        }
        Ok(plan)
    }
    fn obligation(&mut self, kind: &str, member: &str) {
        self.obligations.push(OutcomeObligation {
            source_type_id: self.source.clone(),
            semantic_type_id: self.semantic.clone(),
            kind: kind.into(),
            member_id: member.into(),
            discharged: false,
        });
    }
    pub fn obligations(&self) -> &[OutcomeObligation] {
        &self.obligations
    }
    pub fn dependencies(&self) -> impl Iterator<Item = &Self> {
        self.dependencies.values().map(std::sync::Arc::as_ref)
    }
    pub fn default_eligible(&self) -> bool {
        self.default_eligible
    }
    pub fn source_type_id(&self) -> &str {
        &self.source
    }
    pub fn semantic_type_id(&self) -> &str {
        &self.semantic
    }
    pub fn project(
        &self,
        b: &ValidatedFoundationBundle,
        r: &ValidatedClosedRootSet,
        c: &ClosedInstanceSet,
        value: &MonomorphicValue,
    ) -> Result<MonomorphicValue, DomainError> {
        validate_monomorphic_value(b, r, c, value).map_err(|_| DomainError::OperandType)?;
        let MonomorphicValue::Product { type_id, fields } = value else {
            return Err(DomainError::OperandType);
        };
        if type_id != &self.source {
            return Err(DomainError::OperandType);
        }
        let source = &r.source_types[&self.source];
        let get = |role: &str| -> Result<&MonomorphicValue, DomainError> {
            let member = source
                .members
                .iter()
                .find(|m| m.id == self.members[role])
                .ok_or(DomainError::Binding)?;
            fields
                .iter()
                .find(|f| f.name == member.name)
                .map(|f| f.value.as_ref())
                .ok_or(DomainError::Binding)
        };
        let MonomorphicValue::Enum { carrier, .. } = get("tag")? else {
            return Err(DomainError::Binding);
        };
        let arm = self
            .tags
            .iter()
            .find(|(_, v)| *v == carrier)
            .map(|(a, _)| a.as_str())
            .ok_or(DomainError::Binding)?;
        let payload = match (self.role.as_str(), arm) {
            ("option", "none")
            | ("lookup", "missing_key")
            | ("boundary_field", "missing" | "null") => None,
            ("result", "error") => Some(get("error")?.clone()),
            ("validation", "invalid") => Some(
                project_bounded_sequence_array(b, r, c, get("errors")?)
                    .map_err(|_| DomainError::OperandType)?,
            ),
            _ => Some(get("value")?.clone()),
        };
        let payload = payload
            .map(|v| {
                if let Some(dependency) = self.dependencies.get(v.type_id()) {
                    dependency.project(b, r, c, &v)
                } else if self.role == "validation" && arm == "invalid" {
                    let MonomorphicValue::Sequence { elements, .. } = v else {
                        return Err(DomainError::OperandType);
                    };
                    let target = &c.metadata[&self.semantic].dependency_ids[0];
                    let elements = elements
                        .into_iter()
                        .map(|v| {
                            if let Some(p) = self.dependencies.get(v.type_id()) {
                                p.project(b, r, c, &v)
                            } else {
                                Ok(v)
                            }
                        })
                        .collect::<Result<_, _>>()?;
                    Ok(MonomorphicValue::Sequence {
                        type_id: target.clone(),
                        elements,
                    })
                } else {
                    Ok(v)
                }
            })
            .transpose()?;
        OutcomeModel::new(b, r, c, &self.semantic)?.construct(arm, payload)
    }
    /// Checks a captured reconstruction candidate over ALL observable storage.
    /// Projection alone never establishes this universal commutation property.
    pub fn check_source_round_trip(
        &self,
        b: &ValidatedFoundationBundle,
        r: &ValidatedClosedRootSet,
        c: &ClosedInstanceSet,
        original: &MonomorphicValue,
        reconstructed: &MonomorphicValue,
    ) -> Result<(), DomainError> {
        self.project(b, r, c, original)?;
        self.project(b, r, c, reconstructed)?;
        if source_observations_equal(original, reconstructed) {
            Ok(())
        } else {
            Err(DomainError::ObservationLoss)
        }
    }
}

pub(super) fn source_observations_equal(
    original: &MonomorphicValue,
    reconstructed: &MonomorphicValue,
) -> bool {
    fn normalize(v: &mut Value) {
        match v {
            Value::Object(fields) => {
                if fields.get("kind").and_then(Value::as_str) == Some("decimal_bits") {
                    let mut coefficient = fields["coefficient"]
                        .as_str()
                        .unwrap()
                        .parse::<u128>()
                        .unwrap();
                    let mut scale = fields["scale"].as_u64().unwrap();
                    while scale > 0 && coefficient.is_multiple_of(10) {
                        coefficient /= 10;
                        scale -= 1;
                    }
                    fields.insert("coefficient".into(), json!(coefficient.to_string()));
                    fields.insert("scale".into(), json!(scale));
                    if coefficient == 0 {
                        fields.insert("negative".into(), json!(false));
                    }
                } else {
                    for value in fields.values_mut() {
                        normalize(value);
                    }
                }
            }
            Value::Array(values) => {
                for value in values {
                    normalize(value);
                }
            }
            _ => {}
        }
    }
    let mut a = serde_json::to_value(original).expect("serializable value");
    let mut z = serde_json::to_value(reconstructed).expect("serializable value");
    normalize(&mut a);
    normalize(&mut z);
    a == z
}
