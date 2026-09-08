//! T06-W03 data sequents. A semantic definition is a pending ordinary
//! definition obligation, never an axiom or an evaluation receipt. The closed
//! foundation equations and structural recipes travel with each specialization.
use super::*;
use crate::csharp_practical_vir_validation::{PracticalConstructionAction, ValidatedPracticalVir};

const BOOL: &str = "mpk.csharp.value.bool.v1";
const MAX_NODES: usize = 262_144;
const MAX_DECLARATIONS: usize = 8_192;
const MAX_BYTES: usize = 16 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DataVcError {
    Contract,
    Limit,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DataDefinitionFamily {
    Foundation,
    Structural,
    SourceValue,
    IntegerBoolean,
    String,
    Codec,
    FloatingDecimal,
    NullableOutcome,
    CalendarTimeGuidMoney,
    SequenceOwnership,
}
/// A monomorphic definition requirement. `signature` fixes the full error
/// precedence, including checks whose edge composition belongs to W05/W07.
/// No body is obtained by executing the host semantic model.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DataSemanticDefinition {
    pub id: String,
    pub family: DataDefinitionFamily,
    pub signature: ClosedOperationSignature,
    /// Frozen instantiated equation, where supplied by the foundation.
    pub foundation_equation: Option<Value>,
    /// Finite carrier/structural DAG (stored declaration/argument order).
    pub structural_recipes: Value,
    pub carrier_definitions: Vec<Value>,
    pub relation_name: String,
    pub failure_names: Vec<String>,
    pub failure_result_names: Vec<Option<String>>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DataCheckVc {
    pub id: String,
    pub check: RequiredCheck,
    /// Previous checks have all passed. This guard contains no result variable.
    pub prefix_guard: ContractTerm,
    pub failure_predicate: ContractTerm,
    pub failure_guard: ContractTerm,
    /// Only static checks require failure to be impossible. Runtime failures
    /// retain their edge/tag relation; they are not incorrectly ruled out.
    pub static_goal: Option<ContractTerm>,
    pub exceptional_successor: Option<ExceptionalSuccessor>,
    pub exception_value: Option<MonomorphicValue>,
    /// Tagged failures use the same result SSA carrier as the normal CFG edge.
    pub tagged_result_goal: Option<ContractTerm>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DataOperationVc {
    pub id: String,
    pub function_id: String,
    pub node_id: String,
    pub definition_id: String,
    /// Free variable index i denotes subjects[i], including the final result.
    pub subjects: Vec<TypedValueRef>,
    pub normal_successor_id: String,
    pub checks: Vec<DataCheckVc>,
    pub success_guard: ContractTerm,
    pub success_relation: ContractTerm,
    pub success_goal: ContractTerm,
}
/// Evidence reuses the independent ownership replay already required by VIR
/// import. It does not discharge user invariants or semantic operation goals.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DataOwnershipVc {
    pub id: String,
    pub function_id: String,
    pub node_id: String,
    pub before: Vec<SequenceConstructionState>,
    pub actions: Vec<PracticalConstructionAction>,
    pub after: Vec<SequenceConstructionState>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DataContractVc {
    pub id: String,
    pub attachment_sha256: String,
    pub subjects: Vec<(String, String)>,
    pub definedness: ContractTerm,
}
fn definedness(
    t: &ContractTerm,
    definitions: &[ContractDefinition],
) -> Result<ContractTerm, DataVcError> {
    match t {
        ContractTerm::Var { .. } => return Ok(boolean(true)),
        ContractTerm::Let { value, body, .. } => {
            return Ok(and(
                definedness(value, definitions)?,
                ContractTerm::Let {
                    value: value.clone(),
                    body: Box::new(definedness(body, definitions)?),
                    type_id: BOOL.into(),
                },
            ))
        }
        ContractTerm::Lam {
            parameter_type,
            body,
            ..
        } => {
            return Ok(ContractTerm::Lam {
                parameter_type: parameter_type.clone(),
                body: Box::new(definedness(body, definitions)?),
                type_id: format!("({parameter_type}->{BOOL})"),
            })
        }
        _ => {}
    }
    let mut head = t;
    let mut args = vec![];
    while let ContractTerm::App {
        function, argument, ..
    } = head
    {
        args.push(argument.as_ref().clone());
        head = function;
    }
    args.reverse();
    let ContractTerm::Const { name, .. } = head else {
        return Err(DataVcError::Contract);
    };
    let d = definitions
        .iter()
        .find(|d| &d.name == name)
        .ok_or(DataVcError::Contract)?;
    if args.len() != d.argument_types.len() {
        return Err(DataVcError::Contract);
    }
    if d.tag == "conditional" {
        return Ok(all(&[
            definedness(&args[0], definitions)?,
            implies(args[0].clone(), definedness(&args[1], definitions)?),
            implies(not(args[0].clone()), definedness(&args[2], definitions)?),
        ]));
    }
    if matches!(d.tag.as_str(), "bounded_forall" | "bounded_exists") {
        let bound = apply(
            &format!("Mpk.CSharp.Data.ContractBound.{}", d.name),
            args[..2].to_vec(),
            BOOL,
        );
        let body = definedness(&args[2], definitions)?;
        return Ok(all(&[
            definedness(&args[0], definitions)?,
            definedness(&args[1], definitions)?,
            bound,
            apply(
                &format!("Mpk.CSharp.Data.ContractForall.{}", d.name),
                vec![args[0].clone(), args[1].clone(), body],
                BOOL,
            ),
        ]));
    }
    let mut terms = args
        .iter()
        .map(|a| definedness(a, definitions))
        .collect::<Result<Vec<_>, _>>()?;
    // Contract expressions are total mathematical terms: exceptions cannot
    // escape them. Tagged parse/error results remain values, however.
    terms.extend(
        d.ordered_checks
            .iter()
            .filter(|c| {
                matches!(
                    c.tag,
                    RequiredCheckTag::StaticObligation | RequiredCheckTag::Exception
                )
            })
            .map(|c| {
                not(apply(
                    &format!("Mpk.CSharp.Data.ContractFails.{}.{}", d.name, c.id),
                    args.clone(),
                    BOOL,
                ))
            }),
    );
    if matches!(
        d.tag.as_str(),
        "sequence_index" | "tagged_payload" | "exception_payload" | "codec_format"
    ) {
        terms.push(apply(
            &format!("Mpk.CSharp.Data.ContractDefined.{}", d.name),
            args,
            BOOL,
        ));
    }
    Ok(all(&terms))
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DataVcProgram {
    source_ir_sha256: String,
    closed_set_sha256: String,
    foundation_sha256: String,
    definitions: Vec<DataSemanticDefinition>,
    operations: Vec<DataOperationVc>,
    ownership: Vec<DataOwnershipVc>,
    /// W01 expressions retain their complete attachment, bound terms and check
    /// definitions. Their use-site control composition remains with that owner.
    contract_expressions: Vec<VerifiedContractExpression>,
    contracts: Vec<DataContractVc>,
    definition_names: Vec<String>,
    #[serde(skip)]
    node_count: usize,
}
impl DataVcProgram {
    pub fn definition_names(&self) -> &[String] {
        &self.definition_names
    }
    pub fn definitions(&self) -> &[DataSemanticDefinition] {
        &self.definitions
    }
    pub fn operations(&self) -> &[DataOperationVc] {
        &self.operations
    }
    pub fn contracts(&self) -> &[DataContractVc] {
        &self.contracts
    }
    pub fn ownership(&self) -> &[DataOwnershipVc] {
        &self.ownership
    }
    pub fn contract_expressions(&self) -> &[VerifiedContractExpression] {
        &self.contract_expressions
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("typed data VCs")
    }
    pub fn hash(&self) -> String {
        digest(&self.canonical_bytes())
    }
    pub(crate) fn nodes(&self) -> usize {
        self.node_count
    }
    pub(crate) fn declarations(&self) -> usize {
        self.definition_names.len()
            + self
                .operations
                .iter()
                .map(|o| 1 + o.checks.len())
                .sum::<usize>()
            + self.ownership.len()
            + self.contracts.len()
    }
    pub(crate) fn binder_depth(&self) -> usize {
        self.operations
            .iter()
            .map(|o| o.subjects.len())
            .max()
            .unwrap_or(0)
    }
}
fn digest(bytes: &[u8]) -> String {
    crate::hash::hash_domain_separated_raw(HashDomain::new("MPK-CSHARP-DATA-VC-1.0"), bytes)
        .expect("typed VC hash")
        .to_hex()
}
fn boolean(v: bool) -> ContractTerm {
    ContractTerm::Const {
        name: format!("Mpk.CSharp.Bool.{v}"),
        type_id: BOOL.into(),
    }
}
fn apply(name: &str, args: Vec<ContractTerm>, result: &str) -> ContractTerm {
    let function_type = |args: &[ContractTerm]| {
        args.iter()
            .rev()
            .fold(result.to_owned(), |t, a| format!("({}->{t})", a.type_id()))
    };
    let mut term = ContractTerm::Const {
        name: name.into(),
        type_id: function_type(&args),
    };
    for (i, arg) in args.iter().enumerate() {
        term = ContractTerm::App {
            function: Box::new(term),
            argument: Box::new(arg.clone()),
            type_id: function_type(&args[i + 1..]),
        };
    }
    term
}
fn not(t: ContractTerm) -> ContractTerm {
    apply("Mpk.CSharp.Bool.Not", vec![t], BOOL)
}
fn and(a: ContractTerm, b: ContractTerm) -> ContractTerm {
    apply("Mpk.CSharp.Bool.And", vec![a, b], BOOL)
}
fn implies(a: ContractTerm, b: ContractTerm) -> ContractTerm {
    apply("Mpk.CSharp.Bool.Or", vec![not(a), b], BOOL)
}
fn all(terms: &[ContractTerm]) -> ContractTerm {
    match terms.len() {
        0 => boolean(true),
        1 => terms[0].clone(),
        n => and(all(&terms[..n / 2]), all(&terms[n / 2..])),
    }
}
fn variables(values: &[TypedValueRef]) -> Vec<ContractTerm> {
    values
        .iter()
        .enumerate()
        .map(|(index, v)| ContractTerm::Var {
            index,
            type_id: v.type_id.clone(),
        })
        .collect()
}
fn family(s: &ClosedOperationSignature) -> Option<DataDefinitionFamily> {
    use DataDefinitionFamily as F;
    Some(match s.tag {
        ClosedOperationTag::Foundation => F::Foundation,
        ClosedOperationTag::FieldRead | ClosedOperationTag::ValueConstruct => F::SourceValue,
        ClosedOperationTag::StructuralEqual | ClosedOperationTag::CanonicalCompare => F::Structural,
        ClosedOperationTag::BoundaryParse | ClosedOperationTag::BoundaryFormat => F::Codec,
        ClosedOperationTag::Data if s.id.starts_with("object.") => return None,
        ClosedOperationTag::Data => {
            let id = s.id.as_str();
            if id.starts_with("string.") {
                F::String
            } else if ["floating.", "decimal.", "numeric.conversion."]
                .iter()
                .any(|p| id.starts_with(p))
            {
                F::FloatingDecimal
            } else if ["boolean.", "integer."].iter().any(|p| id.starts_with(p)) {
                F::IntegerBoolean
            } else if [
                "date.",
                "time.",
                "duration.",
                "guid.",
                "instant.",
                "day_of_week.",
                "money.",
            ]
            .iter()
            .any(|p| id.starts_with(p))
            {
                F::CalendarTimeGuidMoney
            } else if ["construction.", "array.", "sequence."]
                .iter()
                .any(|p| id.starts_with(p))
            {
                F::SequenceOwnership
            } else {
                F::NullableOutcome
            }
        }
        _ => return None,
    })
}
/// Concrete finite semantic carrier parameters. These are definition inputs,
/// not facts about a program execution. Numeric precision and collection bounds
/// are fixed here; the ordinary assembler cannot substitute host/BCL behavior.
fn carrier(vir: &ValidatedPracticalVir, id: &str) -> Result<Value, DataVcError> {
    let (b, roots, _) = vir.construction_context();
    if let Some(entry) = vir
        .data_closed()
        .entries()
        .iter()
        .find(|e| e["instance_id"] == id)
    {
        return Ok(entry["type_definition"].clone());
    }
    if let Some(source) = roots.source_types.get(id) {
        let members = source.members.iter().map(|m| Ok(json!({"id":m.id,"ordinal":m.ordinal,"type_id":closed_type_id(b,&m.ty).map_err(|_|DataVcError::Contract)?}))).collect::<Result<Vec<Value>, DataVcError>>()?;
        return Ok(
            json!({"id":id,"representation":"stored_product_or_declared_enum","members":members,"enum_values":source.enum_values,"enum_underlying":source.enum_underlying}),
        );
    }
    let token = id
        .strip_prefix("mpk.csharp.value.")
        .and_then(|s| s.strip_suffix(".v1"))
        .ok_or(DataVcError::Contract)?;
    let body = match token {
        "bool" => json!({"representation":"bool","bits":1}),
        "i8" | "u8" | "i16" | "u16" | "i32" | "u32" | "i64" | "u64" => {
            json!({"representation":"little_endian_bits","bits":token[1..].parse::<u32>().map_err(|_|DataVcError::Contract)?,"signed":token.starts_with('i'),"signed_decode":"unsigned - sign_bit * 2^width","unchecked":"modulo 2^width","division":"truncate_toward_zero","shift_mask":"width-1"})
        }
        "char" => json!({"representation":"utf16_code_unit","bits":16}),
        "f32" | "f64" => {
            let (width, exponent, fraction, bias) = if token == "f32" {
                (32, 8, 23, 127)
            } else {
                (64, 11, 52, 1023)
            };
            json!({"representation":"ieee_bits","bits":width,"exponent_bits":exponent,"fraction_bits":fraction,"bias":bias,"subnormal":"(-1)^s * fraction * 2^(1-bias-fraction_bits)","normal":"(-1)^s * (2^fraction_bits+fraction) * 2^(exponent-bias-fraction_bits)","nan":"exponent=2^exponent_bits-1 and fraction!=0","infinity":"exponent=2^exponent_bits-1 and fraction=0","rounding":"nearest_ties_to_even","zero_sign_retained":true,"nan_payload_retained_by_bit_codec":true,"structural_equality":"ieee_equal","total_order":false,"bounded_natural_words":72,"natural_word_bits":32})
        }
        "decimal" => {
            json!({"representation":"sign_coefficient_scale","coefficient_bits":96,"scale_bits":8,"scale_maximum":28,"numeric_value":"(-1)^sign * coefficient / 10^scale","numeric_equality":"aligned_exact_integer_coefficients","representation_equality":"sign_and_coefficient_and_scale","arithmetic_rounding":"nearest_ties_to_even","bounded_natural_words":72,"natural_word_bits":32})
        }
        "string" => {
            json!({"representation":"u32_length_and_utf16_code_units","length_maximum":STRING_VALUE_LENGTH_MAX,"unit_bits":16,"comparison":"first_different_unit_then_length","null_representation":"option_string","null_first":true,"concat_null":"empty","inactive_cells":"zero"})
        }
        "date" => {
            json!({"representation":"unsigned_day_number","bits":32,"minimum":0,"maximum":3_652_058,"epoch":"0001-01-01","leap_year":"y%4=0 and (y%100!=0 or y%400=0)","days_before_year":"365*(y-1)+(y-1)/4-(y-1)/100+(y-1)/400"})
        }
        "time" => {
            json!({"representation":"unsigned_ticks","bits":64,"minimum":0,"maximum":863_999_999_999_u64,"ticks_per_second":10_000_000})
        }
        "duration" => {
            json!({"representation":"signed_ticks","bits":64,"ticks_per_second":10_000_000,"components":"truncate_toward_zero"})
        }
        "instant" => {
            json!({"representation":"signed_unix_milliseconds","bits":64,"ticks_per_millisecond":10_000,"conversion_checks":["precision","range"]})
        }
        "guid" => {
            json!({"representation":"unsigned_n_fields","field_widths":[32,16,16,8,8,8,8,8,8,8,8],"comparison":"lexicographic_unsigned_fields","generation":"none"})
        }
        "day_of_week" => json!({"representation":"closed_tag","minimum":0,"maximum":6,"sunday":0}),
        "unit" | "parse_error" | "exception" => {
            return b
                .non_template_definitions()
                .iter()
                .find(|d| d["id"] == id)
                .cloned()
                .ok_or(DataVcError::Contract)
        }
        _ => return Err(DataVcError::Contract),
    };
    Ok(json!({"id":id,"definition":body}))
}
fn definition(
    vir: &ValidatedPracticalVir,
    s: &ClosedOperationSignature,
) -> Result<DataSemanticDefinition, DataVcError> {
    let (b, roots, _) = vir.construction_context();
    let closed = vir.data_closed();
    let hash = digest(&serde_json::to_vec(s).map_err(|_| DataVcError::Contract)?);
    let mut recipes = BTreeMap::new();
    for ty in s
        .argument_type_ids
        .iter()
        .chain(std::iter::once(&s.normal_result_type_id))
    {
        // Private construction/exception carriers have no structural comparison
        // recipe. Their complete signature and validated ownership remain bound.
        if roots.source_types.contains_key(ty)
            || closed
                .metadata
                .get(ty)
                .is_some_and(|m| template_name(&m.template_id) != Some("sequence_construction"))
            || PRIMITIVES
                .iter()
                .any(|p| ty == &format!("mpk.csharp.value.{p}.v1"))
        {
            let p = generate_structural_program(b, roots, closed, ty)
                .map_err(|_| DataVcError::Contract)?;
            for (id, r) in p.recipes() {
                recipes.insert(id.clone(), json!({"type_id":r.type_id,"rule":r.rule,"children":r.children,"total":r.total}));
            }
        }
    }
    let mut carrier_ids = recipes.keys().cloned().collect::<BTreeSet<_>>();
    carrier_ids.extend(s.argument_type_ids.iter().cloned());
    carrier_ids.insert(s.normal_result_type_id.clone());
    let carrier_definitions = carrier_ids
        .iter()
        .map(|id| carrier(vir, id))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(DataSemanticDefinition {
        id: format!("data.definition.{hash}"),
        family: family(s).ok_or(DataVcError::Contract)?,
        signature: s.clone(),
        foundation_equation: foundation_operation_definition(closed, &s.id).cloned(),
        structural_recipes: serde_json::to_value(recipes).map_err(|_| DataVcError::Contract)?,
        carrier_definitions,
        relation_name: format!("Mpk.CSharp.Data.Relation.{hash}"),
        failure_names: s
            .ordered_checks
            .iter()
            .map(|c| format!("Mpk.CSharp.Data.Fails.{hash}.{}", c.id))
            .collect(),
        failure_result_names: s
            .ordered_checks
            .iter()
            .map(|c| {
                matches!(
                    c.tag,
                    RequiredCheckTag::ParseError | RequiredCheckTag::ErrorOutcome
                )
                .then(|| format!("Mpk.CSharp.Data.FailureResult.{hash}.{}", c.id))
            })
            .collect(),
    })
}
fn operation(
    function_id: &str,
    node_id: &str,
    invocation: &OperationInvocation,
    exceptions: &[(String, MonomorphicValue)],
    d: &DataSemanticDefinition,
) -> Result<DataOperationVc, DataVcError> {
    let mut subjects = invocation.operands.clone();
    subjects.push(invocation.result.clone());
    if subjects.len() > 256 {
        return Err(DataVcError::Limit);
    }
    let operands = variables(&invocation.operands);
    let values = variables(&subjects);
    let mut passed = Vec::new();
    let mut checks = Vec::new();
    for (i, check) in d.signature.ordered_checks.iter().enumerate() {
        // Even an output-bound check is evaluated from the operands (the
        // computed mathematical length), before constructing a bounded result.
        let failure_predicate = apply(&d.failure_names[i], operands.clone(), BOOL);
        let prefix_guard = all(&passed);
        let failure_guard = and(prefix_guard.clone(), failure_predicate.clone());
        let exceptional_successor = invocation
            .exceptional_successors
            .iter()
            .find(|e| e.check_id == check.id)
            .cloned();
        if (check.tag == RequiredCheckTag::Exception) != exceptional_successor.is_some() {
            return Err(DataVcError::Contract);
        }
        let exception_value = exceptions
            .iter()
            .find(|(id, _)| id == &check.id)
            .map(|(_, v)| v.clone());
        let tagged_result_goal = d.failure_result_names[i]
            .as_ref()
            .map(|name| implies(failure_guard.clone(), apply(name, values.clone(), BOOL)));
        checks.push(DataCheckVc {
            id: format!("data.check.{function_id}.{node_id}.{i:04}"),
            check: check.clone(),
            prefix_guard: prefix_guard.clone(),
            failure_predicate: failure_predicate.clone(),
            failure_guard,
            static_goal: (check.tag == RequiredCheckTag::StaticObligation)
                .then(|| implies(prefix_guard, not(failure_predicate.clone()))),
            exceptional_successor,
            exception_value,
            tagged_result_goal,
        });
        passed.push(not(failure_predicate));
    }
    let success_guard = all(&passed);
    let success_relation = apply(&d.relation_name, values, BOOL);
    let success_goal = implies(success_guard.clone(), success_relation.clone());
    Ok(DataOperationVc {
        id: format!("data.operation.{function_id}.{node_id}"),
        function_id: function_id.into(),
        node_id: node_id.into(),
        definition_id: d.id.clone(),
        subjects,
        normal_successor_id: invocation.normal_successor_id.clone(),
        checks,
        success_guard,
        success_relation,
        success_goal,
    })
}
fn names(t: &ContractTerm, output: &mut BTreeSet<String>) {
    match t {
        ContractTerm::Const { name, .. } => {
            if !name.starts_with("contract.def.") {
                output.insert(name.clone());
            }
        }
        ContractTerm::App {
            function, argument, ..
        } => {
            names(function, output);
            names(argument, output);
        }
        ContractTerm::Let { value, body, .. } => {
            names(value, output);
            names(body, output);
        }
        ContractTerm::Lam { body, .. } => names(body, output),
        _ => {}
    }
}
fn operation_nodes(o: &DataOperationVc) -> usize {
    o.success_guard.nodes()
        + o.success_relation.nodes()
        + o.success_goal.nodes()
        + o.checks
            .iter()
            .map(|c| {
                c.prefix_guard.nodes()
                    + c.failure_predicate.nodes()
                    + c.failure_guard.nodes()
                    + c.static_goal.as_ref().map_or(0, ContractTerm::nodes)
                    + c.tagged_result_goal.as_ref().map_or(0, ContractTerm::nodes)
            })
            .sum::<usize>()
}
pub(crate) fn generate_data_vcs(vir: &ValidatedPracticalVir) -> Result<DataVcProgram, DataVcError> {
    let mut p = DataVcProgram {
        source_ir_sha256: vir.hash().into(),
        closed_set_sha256: vir.data_closed().closed_set_sha256().into(),
        foundation_sha256: vir.construction_context().0.content_sha256().into(),
        definitions: vec![],
        operations: vec![],
        ownership: vec![],
        contract_expressions: vir.contract_expressions().to_vec(),
        contracts: vec![],
        definition_names: vec![],
        node_count: 0,
    };
    for expression in vir.contract_expressions() {
        let term = definedness(expression.term(), expression.definitions())?;
        p.node_count += term.nodes();
        if p.node_count > MAX_NODES {
            return Err(DataVcError::Limit);
        }
        p.contracts.push(DataContractVc {
            id: format!("data.contract.{}", expression.attachment_sha256()),
            attachment_sha256: expression.attachment_sha256().into(),
            subjects: expression.subjects().to_vec(),
            definedness: term,
        });
    }
    let signatures = vir
        .operation_signatures()
        .iter()
        .map(|s| (s.id.as_str(), s))
        .collect::<BTreeMap<_, _>>();
    let mut definitions = BTreeMap::new();
    let mut generated_bytes = 0usize;
    for f in vir.functions() {
        for block in &f.blocks {
            if !block.construction_actions.is_empty()
                || !block.ownership_in.is_empty()
                || !block.ownership_out.is_empty()
            {
                p.ownership.push(DataOwnershipVc {
                    id: format!("data.ownership.{}.{}", f.id, block.node.id),
                    function_id: f.id.clone(),
                    node_id: block.node.id.clone(),
                    before: block.ownership_in.clone(),
                    actions: block.construction_actions.clone(),
                    after: block.ownership_out.clone(),
                });
            }
            if let Some(inv) = &block.invocation {
                let signature = *signatures
                    .get(inv.operation_id.as_str())
                    .ok_or(DataVcError::Contract)?;
                if family(signature).is_none() {
                    continue;
                }
                if !definitions.contains_key(&inv.operation_id) {
                    let d = definition(vir, signature)?;
                    generated_bytes += serde_json::to_vec(&d)
                        .map_err(|_| DataVcError::Contract)?
                        .len();
                    if generated_bytes > MAX_BYTES {
                        return Err(DataVcError::Limit);
                    }
                    definitions.insert(inv.operation_id.clone(), d);
                }
                let exceptions = block
                    .exception_values
                    .iter()
                    .map(|e| (e.check_id.clone(), e.value.clone()))
                    .collect::<Vec<_>>();
                let o = operation(
                    &f.id,
                    &block.node.id,
                    inv,
                    &exceptions,
                    &definitions[&inv.operation_id],
                )?;
                p.node_count = p
                    .node_count
                    .checked_add(operation_nodes(&o))
                    .ok_or(DataVcError::Limit)?;
                if p.node_count > MAX_NODES {
                    return Err(DataVcError::Limit);
                }
                generated_bytes += serde_json::to_vec(&o)
                    .map_err(|_| DataVcError::Contract)?
                    .len();
                if generated_bytes > MAX_BYTES {
                    return Err(DataVcError::Limit);
                }
                p.operations.push(o);
            }
            if definitions.len() + p.operations.len() + p.ownership.len() > MAX_DECLARATIONS {
                return Err(DataVcError::Limit);
            }
        }
    }
    p.definitions = definitions.into_values().collect();
    p.definitions.sort_by(|a, b| a.id.cmp(&b.id));
    p.operations.sort_by(|a, b| a.id.cmp(&b.id));
    p.ownership.sort_by(|a, b| a.id.cmp(&b.id));
    let mut definition_names = BTreeSet::new();
    for o in &p.operations {
        names(&o.success_goal, &mut definition_names);
        for c in &o.checks {
            names(&c.failure_guard, &mut definition_names);
            for goal in c.static_goal.iter().chain(c.tagged_result_goal.iter()) {
                names(goal, &mut definition_names);
            }
        }
    }
    for c in &p.contracts {
        names(&c.definedness, &mut definition_names);
    }
    p.definition_names = definition_names.into_iter().collect();
    if p.declarations() > MAX_DECLARATIONS || p.canonical_bytes().len() > MAX_BYTES {
        return Err(DataVcError::Limit);
    }
    Ok(p)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn d(name: &str, tag: &str, args: &[&str], checks: Vec<RequiredCheck>) -> ContractDefinition {
        ContractDefinition {
            name: name.into(),
            tag: tag.into(),
            parameters: "{}".into(),
            argument_types: args.iter().map(|s| (*s).into()).collect(),
            result_type: BOOL.into(),
            ordered_checks: checks,
        }
    }
    fn eval(t: &ContractTerm, values: &BTreeMap<String, bool>) -> bool {
        let mut head = t;
        let mut args = vec![];
        while let ContractTerm::App {
            function, argument, ..
        } = head
        {
            args.push(argument.as_ref());
            head = function;
        }
        args.reverse();
        let ContractTerm::Const { name, .. } = head else {
            panic!("test boolean term")
        };
        match name.as_str() {
            "Mpk.CSharp.Bool.true" => true,
            "Mpk.CSharp.Bool.false" => false,
            "Mpk.CSharp.Bool.And" => eval(args[0], values) && eval(args[1], values),
            "Mpk.CSharp.Bool.Or" => eval(args[0], values) || eval(args[1], values),
            "Mpk.CSharp.Bool.Not" => !eval(args[0], values),
            _ => values[name],
        }
    }
    #[test]
    fn data_vc_contract_definedness_respects_conditional_and_lexical_binders() {
        let check = RequiredCheck {
            id: "exception.division_by_zero".into(),
            tag: RequiredCheckTag::Exception,
            failure_type_id: Some("System.DivideByZeroException".into()),
        };
        let defs = vec![
            d("condition", "literal", &[], vec![]),
            d("value", "literal", &[], vec![]),
            d("divide", "binary", &[BOOL, BOOL], vec![check]),
            d("choose", "conditional", &[BOOL, BOOL, BOOL], vec![]),
        ];
        let value = apply("value", vec![], BOOL);
        let term = apply(
            "choose",
            vec![
                apply("condition", vec![], BOOL),
                apply("divide", vec![value.clone(), value.clone()], BOOL),
                value.clone(),
            ],
            BOOL,
        );
        let goal = definedness(&term, &defs).unwrap();
        let mut values = BTreeMap::from([
            ("condition".into(), false),
            (
                "Mpk.CSharp.Data.ContractFails.divide.exception.division_by_zero".into(),
                true,
            ),
        ]);
        assert!(eval(&goal, &values));
        values.insert("condition".into(), true);
        assert!(!eval(&goal, &values));
        values.insert(
            "Mpk.CSharp.Data.ContractFails.divide.exception.division_by_zero".into(),
            false,
        );
        assert!(eval(&goal, &values));
        let body = ContractTerm::Lam {
            parameter_type: BOOL.into(),
            body: Box::new(ContractTerm::Let {
                value: Box::new(value),
                body: Box::new(ContractTerm::Var {
                    index: 1,
                    type_id: BOOL.into(),
                }),
                type_id: BOOL.into(),
            }),
            type_id: format!("({BOOL}->{BOOL})"),
        };
        let lowered = definedness(&body, &defs).unwrap();
        assert_eq!(lowered.type_id(), body.type_id());
        assert!(matches!(lowered, ContractTerm::Lam { .. }));
    }
    #[test]
    fn data_vc_rejects_binder_limit_before_generating_operation_terms() {
        let signature = ClosedOperationSignature {
            id: "unit".into(),
            tag: ClosedOperationTag::Data,
            argument_type_ids: vec![BOOL.into(); 256],
            normal_result_type_id: BOOL.into(),
            ordered_checks: vec![],
        };
        let def = DataSemanticDefinition {
            id: "d".into(),
            family: DataDefinitionFamily::IntegerBoolean,
            signature,
            foundation_equation: None,
            structural_recipes: json!({}),
            carrier_definitions: vec![],
            relation_name: "r".into(),
            failure_names: vec![],
            failure_result_names: vec![],
        };
        let inv = OperationInvocation {
            operation_id: "unit".into(),
            operands: (0..256)
                .map(|i| TypedValueRef {
                    id: format!("v{i}"),
                    type_id: BOOL.into(),
                })
                .collect(),
            result: TypedValueRef {
                id: "result".into(),
                type_id: BOOL.into(),
            },
            ordered_check_ids: vec![],
            normal_successor_id: "next".into(),
            exceptional_successors: vec![],
        };
        assert_eq!(
            operation("f", "n", &inv, &[], &def),
            Err(DataVcError::Limit)
        );
    }
}
