//! W03 private-array storage relations at original SSA use points.
//! Symbolic ownership failures bind to exact source points and checked flow.
//! Native execution and application proofs remain separate obligations.
use super::*;
use crate::csharp_practical_vir_model::data_vc::{
    DataDefinitionFamily, DataOperationVc, DataOwnershipVc, DataSemanticDefinition,
};
use ordered_fold::{helper, read_bit, word};
use sha2::{Digest, Sha256};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryConstructionDataDefinition {
    pub source: DataSemanticDefinition,
    pub construction: OrdinaryConstructionDefinition,
    pub value_definition: String,
    pub failure_definitions: Vec<Option<String>>,
    pub relation_definition: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryConstructionDataOperation {
    pub source: DataOperationVc,
    /// Original W03 formula role and corresponding closed ordinary definition.
    pub predicates: BTreeMap<String, String>,
    /// Exact W03 formulas without a matching symbolic source ownership proof.
    pub pending_predicates: BTreeMap<String, ContractTerm>,
    pub ownership: Option<OrdinaryConstructionOwnershipUse>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryConstructionOwnershipUse {
    pub function_id: String,
    pub node_id: String,
    pub receiver_id: String,
    pub state_id: String,
    pub source_failure_name: String,
    pub scoped_failure_definition: String,
    pub scoped_failure_theorem: String,
    pub flow_theorem: String,
    pub receiver_theorem: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryConstructionOwnershipRecord {
    pub source: DataOwnershipVc,
    /// Exact local, invocation, terminal and incoming-edge equations.
    pub equation_ids: Vec<String>,
    pub predicate_definition: String,
    pub theorem: String,
    pub flow_theorem: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryConstructionDataProgram {
    schema: String,
    source_ir_sha256: String,
    foundation_sha256: String,
    data_vc_sha256: String,
    definitions: Vec<OrdinaryConstructionDataDefinition>,
    operations: Vec<OrdinaryConstructionDataOperation>,
    /// Other data families are explicitly outstanding, never silently discharged.
    pending_definition_ids: Vec<String>,
    pending_ownership: Vec<DataOwnershipVc>,
    ownership_records: Vec<OrdinaryConstructionOwnershipRecord>,
    symbolic_ownership: Vec<OrdinaryOwnershipFunction>,
    symbolic_ownership_proofs: Vec<OrdinaryOwnershipProof>,
    pending_concrete_ownership_functions: Vec<String>,
    /// Binding a static ownership check does not prove native execution scopes.
    application_scope_pending: bool,
    certificate_sha256: String,
    #[serde(skip)]
    certificate: Vec<u8>,
}
impl OrdinaryConstructionDataProgram {
    pub fn definitions(&self) -> &[OrdinaryConstructionDataDefinition] {
        &self.definitions
    }
    pub fn operations(&self) -> &[OrdinaryConstructionDataOperation] {
        &self.operations
    }
    pub fn pending_definition_ids(&self) -> &[String] {
        &self.pending_definition_ids
    }
    pub fn pending_ownership(&self) -> &[DataOwnershipVc] {
        &self.pending_ownership
    }
    pub fn ownership_records(&self) -> &[OrdinaryConstructionOwnershipRecord] {
        &self.ownership_records
    }
    pub fn symbolic_ownership(&self) -> &[OrdinaryOwnershipFunction] {
        &self.symbolic_ownership
    }
    pub fn symbolic_ownership_proofs(&self) -> &[OrdinaryOwnershipProof] {
        &self.symbolic_ownership_proofs
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("ordinary construction data relations")
    }
}
fn name(kind: &str, identity: &impl Serialize) -> String {
    format!(
        "{PREFIX}.ConstructionData.{kind}.H{:x}",
        Sha256::digest(serde_json::to_vec(identity).expect("typed construction data identity"))
    )
}
pub(super) fn emit(
    c: &mut Clauses<'_>,
    d: &DataSemanticDefinition,
    construction: &OrdinaryConstructionDefinition,
) -> R<OrdinaryConstructionDataDefinition> {
    let vir = c.vir.ok_or(OrdinaryCarrierError::Linkage)?;
    let s = &d.signature;
    validate_closed_operation_signature(vir.construction_context().1, vir.data_closed(), s)
        .map_err(|_| OrdinaryCarrierError::Linkage)?;
    let id = &construction.carrier.type_id;
    let complete = s.id == format!("construction.complete.{id}");
    let (arguments, result, value_definition, failures) = if complete {
        if d.family != DataDefinitionFamily::SequenceOwnership || s.tag != ClosedOperationTag::Data
        {
            return Err(OrdinaryCarrierError::Linkage);
        }
        (
            vec![id.clone()],
            SOURCE_BOOL.to_owned(),
            construction.complete_definition.clone(),
            vec![],
        )
    } else {
        if d.family != DataDefinitionFamily::Foundation || s.tag != ClosedOperationTag::Foundation {
            return Err(OrdinaryCarrierError::Linkage);
        }
        let op = construction
            .operations
            .iter()
            .find(|o| o.operation_id == s.id)
            .ok_or(OrdinaryCarrierError::Linkage)?;
        (
            op.argument_type_ids.clone(),
            op.result_type_id.clone(),
            op.normal_definition.clone(),
            op.failures.clone(),
        )
    };
    if s.argument_type_ids != arguments
        || s.normal_result_type_id != result
        || s.ordered_checks.len() != failures.len()
        || d.failure_names.len() != failures.len()
        || d.failure_result_names.len() != failures.len()
        || d.failure_result_names.iter().any(Option::is_some)
    {
        return Err(OrdinaryCarrierError::Linkage);
    }
    let inputs = s
        .argument_type_ids
        .iter()
        .chain(std::iter::once(&s.normal_result_type_id))
        .map(|id| {
            c.carriers
                .get(id.as_str())
                .map(|c| c.depth)
                .ok_or(OrdinaryCarrierError::Linkage)
        })
        .collect::<R<Vec<_>>>()?;
    let count = s.argument_type_ids.len();
    let args = (0..count)
        .map(|i| c.b.var((count - i) as u32))
        .collect::<R<Vec<_>>>()?;
    let body = if s.id.ends_with(".fill") || s.id.ends_with(".rewrite") {
        update_relation(&mut c.b, construction, inputs[count] - 16)?
    } else if s.id.ends_with(".freeze") {
        freeze_relation(&mut c.b, construction, inputs[count] - 13)?
    } else {
        let computed = call(&mut c.b, &value_definition, args)?;
        let depth = inputs[count];
        let equal = physical_equal(&mut c.b, depth)?;
        let result = c.b.var(0)?;
        let actual = c.b.var(1)?;
        let body = call(&mut c.b, &equal, vec![result, actual])?;
        let ty = c.b.cube(depth)?;
        c.b.term(TermNode::Let {
            ty,
            value: computed,
            body,
        })?
    };
    let relation_definition = name("Relation", &d.id);
    define(&mut c.b, &relation_definition, &inputs, 0, body)?;
    let mut args = s.argument_type_ids.clone();
    args.push(s.normal_result_type_id.clone());
    if c.constants
        .insert(
            d.relation_name.clone(),
            (signature(&args, SOURCE_BOOL), relation_definition.clone()),
        )
        .is_some()
    {
        return Err(OrdinaryCarrierError::Linkage);
    }
    let mut failure_definitions = vec![];
    for (i, (check, failure)) in s.ordered_checks.iter().zip(&failures).enumerate() {
        if check.id != failure.label {
            return Err(OrdinaryCarrierError::Linkage);
        }
        let Some(body_name) = &failure.definition else {
            if check.id != "ownership" || check.tag != RequiredCheckTag::StaticObligation {
                return Err(OrdinaryCarrierError::Linkage);
            }
            failure_definitions.push(None);
            continue;
        };
        let args = failure
            .argument_indices
            .iter()
            .map(|&index| {
                if index >= count {
                    return Err(OrdinaryCarrierError::Linkage);
                }
                c.b.var((count - 1 - index) as u32)
            })
            .collect::<R<Vec<_>>>()?;
        let body = call(&mut c.b, body_name, args)?;
        let core = name("Failure", &(&d.id, i));
        define(&mut c.b, &core, &inputs[..count], 0, body)?;
        if c.constants
            .insert(
                d.failure_names[i].clone(),
                (signature(&s.argument_type_ids, SOURCE_BOOL), core.clone()),
            )
            .is_some()
        {
            return Err(OrdinaryCarrierError::Linkage);
        }
        failure_definitions.push(Some(core));
    }
    Ok(OrdinaryConstructionDataDefinition {
        source: d.clone(),
        construction: construction.clone(),
        value_definition,
        relation_definition,
        failure_definitions,
    })
}

// Exact zero predicates allow equality to share empty subregions while still
// inspecting every physical address represented by those predicates.
fn physical_zero(b: &mut Builder, depth: u32) -> R<String> {
    let name = format!("{PREFIX}.ConstructionData.Zero.D{depth}");
    if b.globals.contains_key(&name) {
        return Ok(name);
    }
    let source = b.var(0)?;
    let body = if depth == 0 {
        let yes = bit(b, true)?;
        let no = bit(b, false)?;
        mux(b, source, no, yes)?
    } else {
        let next = physical_zero(b, depth - 1)?;
        let low = bit(b, false)?;
        let high = bit(b, true)?;
        let low = b.app(source, vec![low])?;
        let high = b.app(source, vec![high])?;
        let low = call(b, &next, vec![low])?;
        let high = call(b, &next, vec![high])?;
        both(b, low, high)?
    };
    define(b, &name, &[depth], 0, body)?;
    Ok(name)
}
pub(super) fn physical_equal(b: &mut Builder, depth: u32) -> R<String> {
    let name = format!("{PREFIX}.ConstructionData.StorageEqual.D{depth}");
    if b.globals.contains_key(&name) {
        return Ok(name);
    }
    let left = b.var(1)?;
    let right = b.var(0)?;
    let body = if depth == 0 {
        let yes = bit(b, true)?;
        let no = bit(b, false)?;
        let different = mux(b, right, no, yes)?;
        mux(b, left, right, different)?
    } else {
        let next = physical_equal(b, depth - 1)?;
        let mut halves = vec![];
        for on in [false, true] {
            let selector = bit(b, on)?;
            let x = b.app(left, vec![selector])?;
            let y = b.app(right, vec![selector])?;
            halves.push(call(b, &next, vec![x, y])?);
        }
        let split = both(b, halves[0], halves[1])?;
        let zero = physical_zero(b, depth)?;
        let x = call(b, &zero, vec![left])?;
        let y = call(b, &zero, vec![right])?;
        mux(b, x, y, split)?
    };
    define(b, &name, &[depth, depth], 0, body)?;
    Ok(name)
}

// Extensional update equality descends only along the modified index.
// Every untouched subtree is still compared in full, and all result padding
// is checked. This avoids recomputing a 32-bit address comparison at each leaf.
fn update_equal(b: &mut Builder, child: u32, remaining: u32) -> R<String> {
    let name = format!("{PREFIX}.ConstructionData.UpdateEqual.D{child}.I{remaining}");
    if b.globals.contains_key(&name) {
        return Ok(name);
    }
    let source = b.var(3)?;
    let index = b.var(2)?;
    let replacement = b.var(1)?;
    let result = b.var(0)?;
    let body = if remaining == 0 {
        let equal = physical_equal(b, child)?;
        call(b, &equal, vec![replacement, result])?
    } else {
        let next = update_equal(b, child, remaining - 1)?;
        let equal = physical_equal(b, child + remaining - 1)?;
        let mut same = vec![];
        let mut changed = vec![];
        for on in [false, true] {
            let selector = bit(b, on)?;
            let x = b.app(source, vec![selector])?;
            let y = b.app(result, vec![selector])?;
            same.push(call(b, &equal, vec![x, y])?);
            changed.push(call(b, &next, vec![x, index, replacement, y])?);
        }
        let low = both(b, changed[0], same[1])?;
        let high = both(b, same[0], changed[1])?;
        let selected = super::super::ordered_fold::read_bit(b, index, 14 - remaining)?;
        mux(b, selected, high, low)?
    };
    define(
        b,
        &name,
        &[child + remaining, 5, child, child + remaining],
        0,
        body,
    )?;
    Ok(name)
}
fn both(b: &mut Builder, left: u32, right: u32) -> R<u32> {
    let no = bit(b, false)?;
    mux(b, left, right, no)
}
fn at(b: &mut Builder, value: u32, address: &[bool]) -> R<u32> {
    let address = address.iter().map(|&v| bit(b, v)).collect::<R<Vec<_>>>()?;
    b.app(value, address)
}
fn zero_region(b: &mut Builder, value: u32, depth: u32, address: &[bool]) -> R<u32> {
    let remaining = depth
        .checked_sub(address.len() as u32)
        .ok_or(OrdinaryCarrierError::Shape)?;
    let zero = physical_zero(b, remaining)?;
    let region = at(b, value, address)?;
    call(b, &zero, vec![region])
}
fn update_array(
    b: &mut Builder,
    child: u32,
    source: u32,
    index: u32,
    replacement: u32,
    result: u32,
) -> R<u32> {
    let update = update_equal(b, child, 14)?;
    let changed = call(b, &update, vec![source, index, replacement, result])?;
    let equal = physical_equal(b, child + 14)?;
    let unchanged = call(b, &equal, vec![source, result])?;
    let mut in_capacity = bit(b, true)?;
    for i in 14..32 {
        let high = super::super::ordered_fold::read_bit(b, index, i)?;
        let no = bit(b, false)?;
        in_capacity = mux(b, high, no, in_capacity)?;
    }
    mux(b, in_capacity, changed, unchanged)
}
fn update_relation(
    b: &mut Builder,
    construction: &OrdinaryConstructionDefinition,
    child: u32,
) -> R<u32> {
    let depth = construction.carrier.depth;
    if depth != child + 16 {
        return Err(OrdinaryCarrierError::Shape);
    }
    let source = b.var(3)?;
    let index = b.var(2)?;
    let replacement = b.var(1)?;
    let result = b.var(0)?;
    let x = call(b, &construction.length_definition, vec![source])?;
    let y = call(b, &construction.length_definition, vec![result])?;
    let equal = physical_equal(b, 5)?;
    let mut body = call(b, &equal, vec![x, y])?;
    let x = at(b, source, &[true, false])?;
    let y = at(b, result, &[true, false])?;
    let cells = update_array(b, child, x, index, replacement, y)?;
    body = both(b, body, cells)?;
    let mut bitmap = vec![false, true];
    bitmap.extend(vec![false; child as usize]);
    let x = at(b, source, &bitmap)?;
    let y = at(b, result, &bitmap)?;
    let yes = bit(b, true)?;
    let bitmap = update_array(b, 0, x, index, yes, y)?;
    body = both(b, body, bitmap)?;
    // Canonical product packing zeros the unused fourth role and padding
    // preceding the length and initialization-bitmap children.
    let zero = zero_region(b, result, depth, &[true, true])?;
    body = both(b, body, zero)?;
    for (role, child_depth) in [(vec![false, false], 5), (vec![false, true], 14)] {
        let mut prefix = role;
        for _ in 0..depth - 2 - child_depth {
            prefix.push(true);
            let zero = zero_region(b, result, depth, &prefix)?;
            body = both(b, body, zero)?;
            *prefix.last_mut().unwrap() = false;
        }
    }
    Ok(body)
}

// Freeze retains the first 4096 cells of the 16384-cell private array.
// Index selectors are low-bit first, so the two removed high bits are fixed
// only after the remaining twelve selectors. Never truncate a result address.
fn frozen_zero(b: &mut Builder, child: u32, remaining: u32) -> R<String> {
    let name = format!("{PREFIX}.ConstructionData.FrozenZero.D{child}.I{remaining}");
    if b.globals.contains_key(&name) {
        return Ok(name);
    }
    let source = b.var(0)?;
    let body = if remaining == 0 {
        let source = at(b, source, &[false, false])?;
        let zero = physical_zero(b, child)?;
        call(b, &zero, vec![source])?
    } else {
        let next = frozen_zero(b, child, remaining - 1)?;
        let x = at(b, source, &[false])?;
        let y = at(b, source, &[true])?;
        let x = call(b, &next, vec![x])?;
        let y = call(b, &next, vec![y])?;
        both(b, x, y)?
    };
    define(b, &name, &[child + remaining + 2], 0, body)?;
    Ok(name)
}
fn frozen_equal(b: &mut Builder, child: u32, remaining: u32) -> R<String> {
    let name = format!("{PREFIX}.ConstructionData.FrozenEqual.D{child}.I{remaining}");
    if b.globals.contains_key(&name) {
        return Ok(name);
    }
    let source = b.var(1)?;
    let result = b.var(0)?;
    let body = if remaining == 0 {
        let source = at(b, source, &[false, false])?;
        let equal = physical_equal(b, child)?;
        call(b, &equal, vec![source, result])?
    } else {
        let next = frozen_equal(b, child, remaining - 1)?;
        let mut halves = vec![];
        for on in [false, true] {
            let x = at(b, source, &[on])?;
            let y = at(b, result, &[on])?;
            halves.push(call(b, &next, vec![x, y])?);
        }
        let split = both(b, halves[0], halves[1])?;
        let source_zero = frozen_zero(b, child, remaining)?;
        let result_zero = physical_zero(b, child + remaining)?;
        let x = call(b, &source_zero, vec![source])?;
        let y = call(b, &result_zero, vec![result])?;
        mux(b, x, y, split)?
    };
    define(
        b,
        &name,
        &[child + remaining + 2, child + remaining],
        0,
        body,
    )?;
    Ok(name)
}
// Exact bounded prefix over the low-bit-first initialization bitmap. At
// offset k the active count is (length >> k) + carry. The even and odd
// children receive (bit[k] OR carry) and (bit[k] AND carry), respectively.
// This checks stored bits directly, without a counted state pipeline.
pub(super) fn initialized_prefix(b: &mut Builder, indices: u32) -> R<String> {
    if indices > 14 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let name = format!("{PREFIX}.ConstructionData.InitializedPrefix.I{indices}");
    if b.globals.contains_key(&name) {
        return Ok(name);
    }
    ordered_fold::auxiliary(b)?;
    let step = initialized_prefix_step(b, indices, 0)?;
    let length = b.var(1)?;
    let bitmap = b.var(0)?;
    let capacity = word(b, 1 << indices)?;
    let too_large = helper(b, "Less", vec![capacity, length])?;
    let partial_count = helper(b, "Less", vec![length, capacity])?;
    let no = bit(b, false)?;
    // For the inclusive full capacity, the quotient after all low index
    // bits is one. Use all low bits plus carry to represent 2^indices exactly.
    let count = word(b, (1 << indices) - 1)?;
    let yes = bit(b, true)?;
    let all = call(b, &step, vec![count, yes, bitmap])?;
    let partial = call(b, &step, vec![length, no, bitmap])?;
    let body = mux(b, partial_count, partial, all)?;
    let body = mux(b, too_large, no, body)?;
    define(b, &name, &[5, indices], 0, body)?;
    Ok(name)
}
fn initialized_prefix_step(b: &mut Builder, indices: u32, offset: u32) -> R<String> {
    let name = format!("{PREFIX}.ConstructionData.InitializedPrefix.I{indices}.B{offset}");
    if b.globals.contains_key(&name) {
        return Ok(name);
    }
    let remaining = indices - offset;
    let next = if remaining > 0 {
        Some(initialized_prefix_step(b, indices, offset + 1)?)
    } else {
        None
    };
    let length = b.var(2)?;
    let carry = b.var(1)?;
    let source = b.var(0)?;
    let yes = bit(b, true)?;
    let body = if let Some(next) = next {
        let mut empty = call(b, "Std.Bool.not", vec![carry])?;
        for i in offset..indices {
            let bit = read_bit(b, length, i)?;
            let off = call(b, "Std.Bool.not", vec![bit])?;
            empty = both(b, empty, off)?;
        }
        let low = read_bit(b, length, offset)?;
        let even = mux(b, low, yes, carry)?;
        let odd = both(b, low, carry)?;
        let x = at(b, source, &[false])?;
        let y = at(b, source, &[true])?;
        let x = call(b, &next, vec![length, even, x])?;
        let y = call(b, &next, vec![length, odd, y])?;
        let split = both(b, x, y)?;
        mux(b, empty, yes, split)?
    } else {
        mux(b, carry, source, yes)?
    };
    define(b, &name, &[5, 0, remaining], 0, body)?;
    Ok(name)
}

pub(super) fn freeze_relation(
    b: &mut Builder,
    construction: &OrdinaryConstructionDefinition,
    child: u32,
) -> R<u32> {
    if construction.carrier.depth != child + 16 {
        return Err(OrdinaryCarrierError::Shape);
    }
    let source = b.var(1)?;
    let result = b.var(0)?;
    let x = call(b, &construction.length_definition, vec![source])?;
    let mut address = vec![false; (child + 13 - 5) as usize];
    let y = at(b, result, &address)?;
    let equal = physical_equal(b, 5)?;
    let mut body = call(b, &equal, vec![x, y])?;
    let x = at(b, source, &[true, false])?;
    let y = at(b, result, &[true])?;
    let equal = frozen_equal(b, child, 12)?;
    let cells = call(b, &equal, vec![x, y])?;
    body = both(b, body, cells)?;
    address.truncate(1);
    for _ in 0..child + 13 - 1 - 5 {
        address.push(true);
        let zero = zero_region(b, result, child + 13, &address)?;
        body = both(b, body, zero)?;
        *address.last_mut().unwrap() = false;
    }
    Ok(body)
}

fn references_pending(term: &ContractTerm, missing: &BTreeSet<String>) -> bool {
    match term {
        ContractTerm::Const { name, .. } => missing.contains(name),
        ContractTerm::Var { .. } => false,
        ContractTerm::App {
            function, argument, ..
        } => references_pending(function, missing) || references_pending(argument, missing),
        ContractTerm::Lam { body, .. } => references_pending(body, missing),
        ContractTerm::Let { value, body, .. } => {
            references_pending(value, missing) || references_pending(body, missing)
        }
    }
}

pub(super) fn ownership_use(
    c: &mut Clauses<'_>,
    o: &DataOperationVc,
    d: &OrdinaryConstructionDataDefinition,
    functions: &[OrdinaryOwnershipFunction],
    proofs: &[OrdinaryOwnershipProof],
) -> R<Option<OrdinaryConstructionOwnershipUse>> {
    let Some(index) = d
        .source
        .signature
        .ordered_checks
        .iter()
        .position(|c| c.id == "ownership")
    else {
        return Ok(None);
    };
    let Some(f) = functions
        .iter()
        .find(|f| f.source.function_id == o.function_id)
    else {
        return Ok(None);
    };
    let point = f
        .points
        .iter()
        .find(|p| p.node_id == o.node_id)
        .ok_or(OrdinaryCarrierError::Linkage)?;
    if o.subjects.first().map(|s| s.id.as_str()) != Some(point.receiver_id.as_str()) {
        return Err(OrdinaryCarrierError::Linkage);
    }
    let proof = proofs
        .iter()
        .find(|p| p.function_id == o.function_id)
        .ok_or(OrdinaryCarrierError::Linkage)?;
    let receiver_theorem = proof
        .point_theorems
        .get(&o.node_id)
        .ok_or(OrdinaryCarrierError::Linkage)?;
    let inputs = d
        .source
        .signature
        .argument_type_ids
        .iter()
        .map(|ty| {
            c.carriers
                .get(ty.as_str())
                .map(|c| c.depth)
                .ok_or(OrdinaryCarrierError::Linkage)
        })
        .collect::<R<Vec<_>>>()?;
    let core = name("OwnershipFailure", &(&o.id, &point.state_id));
    let body = c.b.constant(&point.witness_definition)?;
    define(&mut c.b, &core, &inputs, 0, body)?;
    let args = (0..inputs.len())
        .map(|i| c.b.var((inputs.len() - i - 1) as u32))
        .collect::<R<Vec<_>>>()?;
    let actual = call(&mut c.b, &core, args)?;
    let no = bit(&mut c.b, false)?;
    let boolean = c.b.boolean;
    let mut ty = call(&mut c.b, "Std.Eq", vec![boolean, actual, no])?;
    let flow = c.b.constant(&f.flow_definition)?;
    let yes = bit(&mut c.b, true)?;
    let flow_type = call(&mut c.b, "Std.Eq", vec![boolean, flow, yes])?;
    let flow_proof = c.b.constant(&proof.flow_theorem)?;
    let receiver_proof = c.b.constant(receiver_theorem)?;
    // The flow proof is a checked dependency even though the closed receiver
    // theorem alone has the required result type. Native execution is separate.
    let mut value = c.b.term(TermNode::Let {
        ty: flow_type,
        value: flow_proof,
        body: receiver_proof,
    })?;
    for input in inputs.iter().rev() {
        let binder = c.b.cube(*input)?;
        ty = c.b.pi(binder, ty)?;
        value = c.b.lam(binder, value)?;
    }
    let theorem = format!("{core}.Proof");
    super::super::ownership_proofs::publish_theorem(&mut c.b, &theorem, ty, value)?;
    let source_failure_name = d.source.failure_names[index].clone();
    if c.constants
        .insert(
            source_failure_name.clone(),
            (
                signature(&d.source.signature.argument_type_ids, SOURCE_BOOL),
                core.clone(),
            ),
        )
        .is_some()
    {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(Some(OrdinaryConstructionOwnershipUse {
        function_id: o.function_id.clone(),
        node_id: o.node_id.clone(),
        receiver_id: point.receiver_id.clone(),
        state_id: point.state_id.clone(),
        source_failure_name,
        scoped_failure_definition: core,
        scoped_failure_theorem: theorem,
        flow_theorem: proof.flow_theorem.clone(),
        receiver_theorem: receiver_theorem.clone(),
    }))
}

fn ownership_record(
    b: &mut Builder,
    vir: &ValidatedPracticalVir,
    record: &DataOwnershipVc,
    functions: &[OrdinaryOwnershipFunction],
    proofs: &[OrdinaryOwnershipProof],
) -> R<Option<OrdinaryConstructionOwnershipRecord>> {
    // Symbolic traces deliberately do not interpret the separate concrete
    // borrow/transfer state machine. Preserve those original records pending.
    let Some(f) = functions
        .iter()
        .find(|f| f.source.function_id == record.function_id)
    else {
        return Ok(None);
    };
    if !record.before.is_empty() || !record.after.is_empty() {
        return Ok(None);
    }
    let source = vir
        .functions()
        .iter()
        .find(|f| f.id == record.function_id)
        .ok_or(OrdinaryCarrierError::Linkage)?;
    let block = source
        .blocks
        .iter()
        .find(|b| b.node.id == record.node_id)
        .ok_or(OrdinaryCarrierError::Linkage)?;
    if block.construction_actions != record.actions
        || block.ownership_in != record.before
        || block.ownership_out != record.after
    {
        return Err(OrdinaryCarrierError::Linkage);
    }
    let trace = f
        .source
        .blocks
        .iter()
        .find(|b| b.node_id == record.node_id)
        .ok_or(OrdinaryCarrierError::Linkage)?;
    let mut expected = BTreeSet::from([
        format!("{}.local", record.node_id),
        format!("{}.invocation", record.node_id),
    ]);
    let cleanup = matches!(
        block.node.tag,
        ControlNodeTag::Exit | ControlNodeTag::HandlerEntry | ControlNodeTag::FinallyEntry
    );
    if cleanup || block.node.tag == ControlNodeTag::Return {
        expected.insert(format!("{}.terminal", record.node_id));
    }
    if cleanup {
        expected.insert(format!("{}.cleanup_coverage", record.node_id));
    }
    for edge in trace.incoming.iter().chain(
        f.source
            .backedges
            .iter()
            .filter(|e| e.target_node_id == record.node_id),
    ) {
        let prefix = format!("{}.to.{}", edge.predecessor_node_id, record.node_id);
        for suffix in ["edge", "cleanup", "phi", "join"] {
            expected.insert(format!("{prefix}.{suffix}"));
        }
        let predecessor = source
            .blocks
            .iter()
            .find(|b| b.node.id == edge.predecessor_node_id)
            .ok_or(OrdinaryCarrierError::Linkage)?;
        if cleanup
            && block.node.tag != ControlNodeTag::FinallyEntry
            && predecessor
                .node
                .normal_successor_ids
                .contains(&record.node_id)
        {
            expected.insert(format!("{prefix}.normal_admission"));
        }
    }
    let selected = f
        .equations
        .iter()
        .filter(|e| expected.contains(&e.id))
        .collect::<Vec<_>>();
    if selected.len() != expected.len() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    fn conjunction(b: &mut Builder, leaves: &[u32]) -> R<u32> {
        match leaves {
            [] => bit(b, true),
            [one] => Ok(*one),
            _ => {
                let middle = leaves.len() / 2;
                let a = conjunction(b, &leaves[..middle])?;
                let z = conjunction(b, &leaves[middle..])?;
                both(b, a, z)
            }
        }
    }
    let leaves = selected
        .iter()
        .map(|e| b.constant(&e.witness_definition))
        .collect::<R<Vec<_>>>()?;
    let body = conjunction(b, &leaves)?;
    let predicate_definition = name("OwnershipRecord", &record.id);
    define(b, &predicate_definition, &[], 0, body)?;
    let proof = proofs
        .iter()
        .find(|p| p.function_id == record.function_id)
        .ok_or(OrdinaryCarrierError::Linkage)?;
    let theorem = super::super::ownership_proofs::prove_record(
        b,
        f,
        &proof.flow_theorem,
        &predicate_definition,
    )?;
    Ok(Some(OrdinaryConstructionOwnershipRecord {
        source: record.clone(),
        equation_ids: selected.iter().map(|e| e.id.clone()).collect(),
        predicate_definition,
        theorem,
        flow_theorem: proof.flow_theorem.clone(),
    }))
}

pub fn generate_csharp_practical_ordinary_construction_data(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryConstructionDataProgram> {
    let data = crate::csharp_practical_vir_model::data_vc::generate_data_vcs(vir)
        .map_err(|_| OrdinaryCarrierError::Linkage)?;
    let layouts = generate_csharp_practical_ordinary_carriers(vir)?;
    let (builder, foundations) =
        super::super::construction_ops::emit_definitions(vir, &layouts, Builder::new()?)?;
    let (builder, symbolic_ownership, pending_concrete_ownership_functions) =
        super::super::ownership_flow::emit_definitions(vir, builder)?;
    let (builder, symbolic_ownership_proofs) =
        super::super::ownership_proofs::emit_proofs(builder, &symbolic_ownership)?;
    let available = foundations
        .iter()
        .map(|d| (d.carrier.type_id.as_str(), d))
        .collect::<BTreeMap<_, _>>();
    let mut c = compiler(vir, &layouts, builder, &[])?;
    c.definedness_logic()?;
    let mut definitions = vec![];
    let mut pending = vec![];
    for d in data.definitions() {
        let id = d
            .signature
            .id
            .strip_prefix("construction.complete.")
            .or_else(|| d.signature.id.rsplit_once('.').map(|(id, _)| id))
            .unwrap_or("");
        let Some(operation) = available.get(id).filter(|_| {
            matches!(
                d.family,
                DataDefinitionFamily::Foundation | DataDefinitionFamily::SequenceOwnership
            )
        }) else {
            pending.push(d.id.clone());
            continue;
        };
        definitions.push(emit(&mut c, d, operation)?);
    }

    let missing = definitions
        .iter()
        .flat_map(|d| {
            d.source
                .failure_names
                .iter()
                .zip(&d.failure_definitions)
                .filter(|(_, body)| body.is_none())
                .map(|(name, _)| name.clone())
        })
        .collect::<BTreeSet<_>>();
    let ids = definitions
        .iter()
        .map(|d| d.source.id.as_str())
        .collect::<BTreeSet<_>>();
    let mut operations = vec![];
    for o in data
        .operations()
        .iter()
        .filter(|o| ids.contains(o.definition_id.as_str()))
    {
        let d = definitions
            .iter()
            .find(|d| d.source.id == o.definition_id)
            .ok_or(OrdinaryCarrierError::Linkage)?;
        let ownership = ownership_use(
            &mut c,
            o,
            d,
            &symbolic_ownership,
            &symbolic_ownership_proofs,
        )?;
        let mut point_missing = missing.clone();
        if let Some(binding) = &ownership {
            point_missing.remove(&binding.source_failure_name);
        }
        let mut predicates = BTreeMap::new();
        let mut pending_predicates = BTreeMap::new();
        let mut add = |role: String, term: &ContractTerm| -> R<()> {
            if references_pending(term, &point_missing) {
                if pending_predicates.insert(role, term.clone()).is_some() {
                    return Err(OrdinaryCarrierError::Linkage);
                }
                return Ok(());
            }
            let name = name("Point", &(&o.id, &role));
            integer_data::predicate(&mut c, &name, &o.subjects, term)?;
            if predicates.insert(role, name).is_some() {
                return Err(OrdinaryCarrierError::Linkage);
            }
            Ok(())
        };
        add("success_guard".into(), &o.success_guard)?;
        add("success_relation".into(), &o.success_relation)?;
        add("success_goal".into(), &o.success_goal)?;
        for (i, check) in o.checks.iter().enumerate() {
            add(format!("check.{i}.prefix"), &check.prefix_guard)?;
            add(format!("check.{i}.failed"), &check.failure_predicate)?;
            add(format!("check.{i}.guard"), &check.failure_guard)?;
            if let Some(goal) = &check.static_goal {
                add(format!("check.{i}.static_goal"), goal)?;
            }
            if check.tagged_result_goal.is_some() {
                return Err(OrdinaryCarrierError::Linkage);
            }
        }
        if let Some(binding) = &ownership {
            // This interpretation is valid at this original source use only.
            // The generic operation failure remains unresolved outside it.
            c.constants.remove(&binding.source_failure_name);
        }
        operations.push(OrdinaryConstructionDataOperation {
            source: o.clone(),
            predicates,
            pending_predicates,
            ownership,
        });
    }
    let mut ownership_records = vec![];
    let mut pending_ownership = vec![];
    for record in data.ownership() {
        match ownership_record(
            &mut c.b,
            vir,
            record,
            &symbolic_ownership,
            &symbolic_ownership_proofs,
        )? {
            Some(record) => ownership_records.push(record),
            None => pending_ownership.push(record.clone()),
        }
    }
    let certificate = c.b.finish()?;
    let p = OrdinaryConstructionDataProgram {
        schema: "mpk.csharp.ordinary_construction_data.v1".into(),
        source_ir_sha256: vir.hash().into(),
        foundation_sha256: vir.construction_context().0.content_sha256().into(),
        data_vc_sha256: data.hash(),
        definitions,
        operations,
        pending_definition_ids: pending,
        pending_ownership,
        ownership_records,
        symbolic_ownership,
        symbolic_ownership_proofs,
        pending_concrete_ownership_functions,
        application_scope_pending: true,
        certificate_sha256: mpk_cert::hash_hex(&mpk_cert::certificate_hash(&certificate)),
        certificate,
    };
    if p.canonical_bytes().len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    Ok(p)
}
pub fn import_csharp_practical_ordinary_construction_data(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryConstructionDataProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let p = generate_csharp_practical_ordinary_construction_data(vir)?;
    if input != p.canonical_bytes() || certificate != p.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(p)
}

#[cfg(test)]
mod prefix_tests {
    use super::super::super::super::test_eval::{bit as observed, run, V};
    use super::*;

    #[test]
    fn construction_initialized_prefix_full_capacity_boundaries() {
        let mut b = Builder::new().unwrap();
        let name = initialized_prefix(&mut b, 14).unwrap();
        let c = mpk_cert::decode_canonical_certificate(&b.finish().unwrap()).unwrap();
        for length in [0u32, 1, 4095, 4096, 4097, 16383, 16384, 16385, u32::MAX] {
            let length_value = || V::Cube((0..32).map(|i| length & (1 << i) != 0).collect());
            for missing in [None, Some(0u32), Some(4095), Some(16383)] {
                let bitmap = V::Cube((0..16384).map(|i| Some(i) != missing).collect());
                let expected = length <= 16384 && missing.is_none_or(|i| i >= length);
                assert_eq!(
                    observed(run(&c, &name, vec![length_value(), bitmap])),
                    expected,
                    "length {length}, missing {missing:?}"
                );
            }
        }
    }

    #[test]
    fn construction_initialized_prefix_checks_every_active_bit() {
        let mut b = Builder::new().unwrap();
        let names = [0, 3].map(|depth| initialized_prefix(&mut b, depth).unwrap());
        let bytes = b.finish().unwrap();
        let c = mpk_cert::decode_canonical_certificate(&bytes).unwrap();
        for (depth, name) in [0, 3].into_iter().zip(names) {
            let capacity = 1u32 << depth;
            for mask in 0..(1u32 << capacity) {
                for length in (0..=capacity + 1).chain([u32::MAX]) {
                    let bitmap = if depth == 0 {
                        V::Bit(mask != 0)
                    } else {
                        V::Cube((0..capacity).map(|i| mask & (1 << i) != 0).collect())
                    };
                    let length_value = V::Cube((0..32).map(|i| length & (1 << i) != 0).collect());
                    let expected = length <= capacity && (0..length).all(|i| mask & (1 << i) != 0);
                    assert_eq!(
                        observed(run(&c, &name, vec![length_value, bitmap])),
                        expected,
                        "I{depth} length {length} bitmap {mask}"
                    );
                }
            }
        }
    }
}
