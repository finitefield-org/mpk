//! Recursive representation domains and saturated logical-cell counts.
//! Source public invariants and application VC proofs remain separate obligations.
use super::*;
#[path = "csharp_practical_ordinary_structural_foundations.rs"]
mod structural_foundations;
pub use structural_foundations::{
    generate_csharp_practical_ordinary_structural_boundary,
    generate_csharp_practical_ordinary_structural_foundations,
    generate_csharp_practical_ordinary_structural_public,
    import_csharp_practical_ordinary_structural_boundary,
    import_csharp_practical_ordinary_structural_foundations,
    import_csharp_practical_ordinary_structural_public, OrdinaryDeferredFoundationInstance,
    OrdinaryStructuralFoundationProgram,
};
#[path = "csharp_practical_ordinary_collection_ops.rs"]
mod collection_ops;
pub(super) use collection_ops::emit_contract_read;
pub use collection_ops::{
    generate_csharp_practical_ordinary_collections, import_csharp_practical_ordinary_collections,
    OrdinaryCollectionDefinition, OrdinaryCollectionFailure, OrdinaryCollectionOperation,
    OrdinaryCollectionProgram,
};
use ordered_fold::{helper as fold_helper, read_bit, word};
const INVALID: u32 = (TOTAL_VALUE_CELLS_MAX + 1) as u32;

#[derive(Clone, Debug)]
struct Count {
    depth: u32,
    definition: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryDomainDefinition {
    pub carrier: OrdinaryCarrier,
    /// Exact logical cells for valid representations, otherwise 65,537.
    pub count_definition: String,
    /// Representation validity only; does not establish source public clauses.
    pub valid_definition: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryDomainProgram {
    schema: String,
    source_ir_sha256: String,
    foundation_sha256: String,
    definitions: Vec<OrdinaryDomainDefinition>,
    static_transformers: usize,
    certificate_sha256: String,
    #[serde(skip)]
    certificate: Vec<u8>,
}
impl OrdinaryDomainProgram {
    pub fn definitions(&self) -> &[OrdinaryDomainDefinition] {
        &self.definitions
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("typed recursive domain program")
    }
}
fn name(id: &str) -> String {
    format!(
        "{PREFIX}.Domain.T{}",
        id.as_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    )
}
fn not(b: &mut Builder, value: u32) -> R<u32> {
    call(b, "Std.Bool.not", vec![value])
}
fn reject_unless(b: &mut Builder, valid: u32, count: u32) -> R<u32> {
    let invalid = word(b, INVALID)?;
    wmux(b, valid, count, invalid)
}
fn valid_count(b: &mut Builder, count: u32) -> R<u32> {
    let limit = word(b, INVALID)?;
    fold_helper(b, "Less", vec![count, limit])
}
fn add(b: &mut Builder, left: u32, right: u32) -> R<u32> {
    let d = super::super::super::scalar_bits::count_addition(b)?;
    call(b, &d, vec![left, right])
}
/// All leaves are false. Each fixed depth shares a closed ordinary definition.
/// Both selector branches are covered; no collection length bounds this check.
fn zero_definition(b: &mut Builder, depth: u32) -> R<String> {
    if depth > 253 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let name = format!("{PREFIX}.Domain.Zero.D{depth}");
    if b.globals.contains_key(&name) {
        return Ok(name);
    }
    let body = if depth == 0 {
        let value = b.var(0)?;
        not(b, value)?
    } else if depth <= 5 {
        // A word is a fixed scalar Boolean expression, just like the existing
        // ordered-fold Empty helper. Do not start a collection state scan for
        // every word inside a large region.
        let source = b.var(0)?;
        let mut valid = bit(b, true)?;
        for i in 0..1 << depth {
            let address = prefix(depth, i)
                .into_iter()
                .map(|v| bit(b, v))
                .collect::<R<Vec<_>>>()?;
            let value = b.app(source, address)?;
            let empty = not(b, value)?;
            valid = and(b, valid, empty)?;
        }
        valid
    } else {
        // This is a fixed truth table, not a counted collection. Partition
        // its full address space by the first selector. Shared closed child
        // definitions let equal subviews reuse ordinary evaluation results
        // without traversing a counted-fold state chain for every empty word.
        // Emit the smaller definition first: the certificate is acyclic.
        let child = zero_definition(b, depth - 1)?;
        let source = b.var(0)?;
        let low = bit(b, false)?;
        let high = bit(b, true)?;
        let low = b.app(source, vec![low])?;
        let high = b.app(source, vec![high])?;
        let low = call(b, &child, vec![low])?;
        let high = call(b, &child, vec![high])?;
        and(b, low, high)?
    };
    define(b, &name, &[depth], 0, body)?;
    Ok(name)
}
/// Check every physical slot at index >= length, without counting those slots.
/// The caller separately enforces the declared capacity bound. A length at or
/// above physical capacity has no inactive slots.
fn zero_tail_definition(b: &mut Builder, indices: u32, child: u32) -> R<String> {
    if indices > 14 || child > 253 - indices {
        return Err(OrdinaryCarrierError::Limit);
    }
    let name = format!("{PREFIX}.Domain.ZeroTail.I{indices}.D{child}");
    if b.globals.contains_key(&name) {
        return Ok(name);
    }
    ordered_fold::auxiliary(b)?;
    let step = zero_tail_step_definition(b, indices, child, 0)?;
    let length = b.var(1)?;
    let source = b.var(0)?;
    let carry = bit(b, false)?;
    let tail = call(b, &step, vec![length, carry, source])?;
    let capacity = word(b, 1 << indices)?;
    let partial = fold_helper(b, "Less", vec![length, capacity])?;
    let yes = bit(b, true)?;
    let body = mux(b, partial, tail, yes)?;
    define(b, &name, &[5, indices + child], 0, body)?;
    Ok(name)
}
/// At offset k, the active count is (original_length >> k) + carry.
/// Keep the original word, reading fixed bit addresses; never build nested
/// shifted/incremented word closures. For the next even/odd branch, the carry
/// is respectively (bit[k] OR carry) / (bit[k] AND carry).
/// The wrapper guarantees original_length < 2^indices before invoking a step.
fn zero_tail_step_definition(b: &mut Builder, indices: u32, child: u32, offset: u32) -> R<String> {
    let name = format!("{PREFIX}.Domain.ZeroTail.I{indices}.D{child}.Step.B{offset}");
    if b.globals.contains_key(&name) {
        return Ok(name);
    }
    let remaining = indices - offset;
    let zero = zero_definition(b, remaining + child)?;
    let next = if remaining > 0 {
        Some(zero_tail_step_definition(b, indices, child, offset + 1)?)
    } else {
        None
    };
    let length = b.var(2)?;
    let carry = b.var(1)?;
    let source = b.var(0)?;
    let all_zero = call(b, &zero, vec![source])?;
    let yes = bit(b, true)?;
    let body = if let Some(next) = next {
        // Because the original high bits are zero, the active subarray is
        // empty iff its remaining length bits and carry are all zero; it is
        // full iff those length bits and carry are all one.
        let mut empty = not(b, carry)?;
        let mut full = carry;
        for i in offset..indices {
            let value = read_bit(b, length, i)?;
            let not_value = not(b, value)?;
            empty = and(b, empty, not_value)?;
            full = and(b, full, value)?;
        }
        let low_bit = read_bit(b, length, offset)?;
        let even_carry = mux(b, low_bit, yes, carry)?;
        let odd_carry = and(b, low_bit, carry)?;
        let low = bit(b, false)?;
        let high = bit(b, true)?;
        let low = b.app(source, vec![low])?;
        let high = b.app(source, vec![high])?;
        let low = call(b, &next, vec![length, even_carry, low])?;
        let high = call(b, &next, vec![length, odd_carry, high])?;
        let split = and(b, low, high)?;
        // Entirely zero subregions satisfy every suffix and need no split.
        let partial = mux(b, all_zero, yes, split)?;
        let nonfull = mux(b, empty, all_zero, partial)?;
        mux(b, full, yes, nonfull)?
    } else {
        // The original quotient is zero after all index bits are removed.
        mux(b, carry, yes, all_zero)?
    };
    define(b, &name, &[5, 0, remaining + child], 0, body)?;
    Ok(name)
}
/// Check a fixed subcube without inspecting any other region.
fn zero_region(b: &mut Builder, source: u32, depth: u32, leading: &[bool]) -> R<u32> {
    let remaining = depth
        .checked_sub(leading.len() as u32)
        .ok_or(OrdinaryCarrierError::Shape)?;
    let zero = zero_definition(b, remaining)?;
    let address = leading.iter().map(|v| bit(b, *v)).collect::<R<Vec<_>>>()?;
    let value = b.app(source, address)?;
    call(b, &zero, vec![value])
}
/// A padded child occupies the all-false leading branch. Check every sibling
/// subcube of that branch, without rechecking the active child's own storage.
fn padding(b: &mut Builder, source: u32, depth: u32, leading: &[bool], child: u32) -> R<u32> {
    let count = depth
        .checked_sub(leading.len() as u32 + child)
        .ok_or(OrdinaryCarrierError::Shape)?;
    let mut valid = bit(b, true)?;
    let mut address = leading.to_vec();
    for _ in 0..count {
        address.push(true);
        let zero = zero_region(b, source, depth, &address)?;
        valid = and(b, valid, zero)?;
        *address.last_mut().unwrap() = false;
    }
    Ok(valid)
}
fn index_word(b: &mut Builder, bits: u32) -> R<u32> {
    let mut body = bit(b, false)?;
    for i in 0..bits {
        let at = equal_address(b, 5, 0, 5, i)?;
        let value = b.var(5 + bits - 1 - i)?;
        body = mux(b, at, value, body)?;
    }
    b.wrap_selectors(5, body)
}
struct Domains<'a> {
    r: Relations<'a>,
    /// None preserves representation-only definitions; Some also filters every
    /// reached source value through all its declared public clauses.
    public_clauses: Option<BTreeMap<String, Vec<String>>>,
    counts: BTreeMap<String, Count>,
    active: BTreeSet<String>,
}
impl Domains<'_> {
    fn type_name(&self, id: &str) -> String {
        let original = name(id);
        if self.public_clauses.is_some() {
            original.replacen(".Domain.T", ".PublicDomain.T", 1)
        } else {
            original
        }
    }
    fn finish_count(&mut self, name: &str, depth: u32, body: u32) -> R<Count> {
        let definition = format!("{name}.Count");
        define(&mut self.r.b, &definition, &[depth], 5, body)?;
        Ok(Count { depth, definition })
    }
    fn scalar(&mut self, carrier: &OrdinaryCarrier, rule: OrdinaryScalarDomainRule) -> R<Count> {
        // Representation and public profiles share each scalar membership
        // definition in one builder. Both are reconstructed from the same VIR.
        let definition = format!(
            "{PREFIX}.ScalarDomain.T{}.Valid",
            carrier
                .type_id
                .as_bytes()
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
        );
        if !self.r.b.globals.contains_key(&definition) {
            scalar_domains::emit_domain(&mut self.r.b, carrier, rule)?;
        }
        let source = self.r.b.var(0)?;
        let valid = call(&mut self.r.b, &definition, vec![source])?;
        let one = word(&mut self.r.b, 1)?;
        let body = reject_unless(&mut self.r.b, valid, one)?;
        self.finish_count(&self.type_name(&carrier.type_id), carrier.depth, body)
    }
    fn ty(&mut self, id: &str) -> R<Count> {
        if let Some(d) = self.counts.get(id) {
            return Ok(d.clone());
        }
        if self.r.internal(id) {
            return Err(OrdinaryCarrierError::Shape);
        }
        if !self.active.insert(id.into()) {
            return Err(OrdinaryCarrierError::Cycle);
        }
        let carrier = self
            .r
            .carriers
            .get(id)
            .ok_or(OrdinaryCarrierError::Linkage)?
            .clone();
        let d = if let Some(rule) = scalar_domains::selected_rule(&carrier, self.r.vir)? {
            self.scalar(&carrier, rule)?
        } else {
            let template = self.r.template(id);
            match template.as_deref() {
                Some("mpk.csharp.semantic.ordered_map.v1") => {
                    let OrdinaryShape::Sequence { capacity, element } = &carrier.shape else {
                        return Err(OrdinaryCarrierError::Shape);
                    };
                    self.sequence(*capacity, element, &self.type_name(id), 1, Some(true))?
                }
                Some("mpk.csharp.semantic.ordered_set.v1") => {
                    let OrdinaryShape::Sequence { capacity, element } = &carrier.shape else {
                        return Err(OrdinaryCarrierError::Shape);
                    };
                    self.sequence(*capacity, element, &self.type_name(id), 1, Some(false))?
                }
                Some("mpk.csharp.semantic.transition.v1") => {
                    let OrdinaryShape::Product { fields } = &carrier.shape else {
                        return Err(OrdinaryCarrierError::Shape);
                    };
                    self.product(fields, &self.type_name(id), 1, true)?
                }
                _ if id == "mpk.csharp.value.exception.v1" => {
                    let OrdinaryShape::Sum { arms } = &carrier.shape else {
                        return Err(OrdinaryCarrierError::Shape);
                    };
                    self.sum(arms, &self.type_name(id), true)?
                }
                _ => self.shape(&carrier.shape, &self.type_name(id))?,
            }
        };
        if d.depth != carrier.depth {
            return Err(OrdinaryCarrierError::Shape);
        }
        let d = if let Some(clauses) = self
            .public_clauses
            .as_ref()
            .and_then(|p| p.get(id))
            .cloned()
        {
            let source = self.r.b.var(0)?;
            let mut valid = bit(&mut self.r.b, true)?;
            for clause in clauses {
                let holds = call(&mut self.r.b, &clause, vec![source])?;
                valid = and(&mut self.r.b, valid, holds)?;
            }
            let cells = call(&mut self.r.b, &d.definition, vec![source])?;
            let cells = reject_unless(&mut self.r.b, valid, cells)?;
            self.finish_count(
                &format!("{}.DeclaredClauses", self.type_name(id)),
                carrier.depth,
                cells,
            )?
        } else {
            d
        };
        self.counts.insert(id.into(), d.clone());
        self.active.remove(id);
        Ok(d)
    }
    fn product(
        &mut self,
        fields: &[OrdinaryField],
        name: &str,
        own: u32,
        transition: bool,
    ) -> R<Count> {
        let mut children = vec![];
        for (i, f) in fields.iter().enumerate() {
            let child_name = format!("{name}.Field.F{i}");
            let child = if transition && f.id == "events" {
                let shape = self.resolve(&f.shape)?;
                let OrdinaryShape::Sequence { capacity, element } = shape else {
                    return Err(OrdinaryCarrierError::Shape);
                };
                self.sequence(capacity, &element, &child_name, 0, None)?
            } else {
                self.shape(&f.shape, &child_name)?
            };
            children.push(child);
        }
        if transition && fields.iter().filter(|f| f.id == "events").count() != 1 {
            return Err(OrdinaryCarrierError::Shape);
        }
        let roles = address_bits(fields.len() as u32);
        let max = children.iter().map(|c| c.depth).max().unwrap_or(0);
        let depth = roles + max;
        let source = self.r.b.var(0)?;
        let mut valid = bit(&mut self.r.b, true)?;
        // Collapse fields whose domain count is identically one, but retain
        // every physical field-padding check. Repeatedly building a scalar
        // addition for each fixed-width field makes large record arrays costly.
        let unit_fields = fields
            .iter()
            .map(|field| self.unconstrained_unit_cell(&field.shape))
            .collect::<R<Vec<_>>>()?;
        let fixed = unit_fields.iter().fold(own, |count, unit| {
            count.saturating_add(u32::from(*unit)).min(INVALID)
        });
        let mut count = word(&mut self.r.b, fixed)?;
        for (i, child) in children.iter().enumerate() {
            let leading = prefix(roles, i as u32);
            let padded = padding(&mut self.r.b, source, depth, &leading, child.depth)?;
            valid = and(&mut self.r.b, valid, padded)?;
            let mut address = leading;
            address.extend(vec![false; (max - child.depth) as usize]);
            let getter =
                self.r
                    .getter(&format!("{name}.Read.F{i}"), depth, child.depth, &address)?;
            if !unit_fields[i] {
                let value = call(&mut self.r.b, &getter, vec![source])?;
                let cells = call(&mut self.r.b, &child.definition, vec![value])?;
                count = add(&mut self.r.b, count, cells)?;
            }
        }
        for i in fields.len() as u32..1 << roles {
            let empty = zero_region(&mut self.r.b, source, depth, &prefix(roles, i))?;
            valid = and(&mut self.r.b, valid, empty)?;
        }
        let count = reject_unless(&mut self.r.b, valid, count)?;
        self.finish_count(name, depth, count)
    }
    fn sum(&mut self, arms: &[OrdinaryArm], name: &str, exception: bool) -> R<Count> {
        let mut children = vec![];
        for arm in arms {
            let own = u32::from(exception && arm.tag >= 9);
            children.push(self.product(
                &arm.fields,
                &format!("{name}.Arm.A{}", arm.tag),
                own,
                false,
            )?);
        }
        let payload = 5.max(children.iter().map(|c| c.depth).max().unwrap_or(0));
        let depth = payload + 1;
        let mut address = vec![false; (depth - 5) as usize];
        let getter = self.r.getter(&format!("{name}.Tag"), depth, 5, &address)?;
        let source = self.r.b.var(0)?;
        let tag = call(&mut self.r.b, &getter, vec![source])?;
        let tag_padding = padding(&mut self.r.b, source, depth, &[false], 5)?;
        let mut count = word(&mut self.r.b, INVALID)?;
        let one = word(&mut self.r.b, 1)?;
        let mut seen = BTreeSet::new();
        for (arm, child) in arms.iter().zip(&children).rev() {
            if !seen.insert(arm.tag) {
                return Err(OrdinaryCarrierError::Shape);
            }
            address = vec![true];
            address.extend(vec![false; (payload - child.depth) as usize]);
            let getter = self.r.getter(
                &format!("{name}.Payload.A{}", arm.tag),
                depth,
                child.depth,
                &address,
            )?;
            let value = call(&mut self.r.b, &getter, vec![source])?;
            let cells = call(&mut self.r.b, &child.definition, vec![value])?;
            let cells = add(&mut self.r.b, one, cells)?;
            let padded = padding(&mut self.r.b, source, depth, &[true], child.depth)?;
            let cells = reject_unless(&mut self.r.b, padded, cells)?;
            let expected = word(&mut self.r.b, arm.tag)?;
            let active = self.r.eq_word(tag, expected)?;
            count = wmux(&mut self.r.b, active, cells, count)?;
        }
        let count = reject_unless(&mut self.r.b, tag_padding, count)?;
        self.finish_count(name, depth, count)
    }
    fn resolve(&self, shape: &OrdinaryShape) -> R<OrdinaryShape> {
        match shape {
            OrdinaryShape::Reference { type_id } => self
                .r
                .carriers
                .get(type_id)
                .map(|c| c.shape.clone())
                .ok_or(OrdinaryCarrierError::Linkage),
            _ => Ok(shape.clone()),
        }
    }
    // Only full-width unconstrained bit scalars have count identically one.
    // Enums, ranges, padding, wrappers and declared public clauses must still
    // evaluate their element domains. Ordering also requires the normal fold.
    fn unconstrained_unit_cell(&self, shape: &OrdinaryShape) -> R<bool> {
        match shape {
            OrdinaryShape::Bits { width } => Ok(width.is_power_of_two() && *width <= 512),
            OrdinaryShape::Reference { type_id } => {
                if self
                    .public_clauses
                    .as_ref()
                    .and_then(|clauses| clauses.get(type_id))
                    .is_some_and(|clauses| !clauses.is_empty())
                {
                    return Ok(false);
                }
                let carrier = self
                    .r
                    .carriers
                    .get(type_id)
                    .ok_or(OrdinaryCarrierError::Linkage)?;
                Ok(
                    matches!(scalar_domains::selected_rule(carrier, self.r.vir)?,
                    Some(OrdinaryScalarDomainRule::Bits { width })
                        if width.is_power_of_two() && width <= 512 && address_bits(width) == carrier.depth),
                )
            }
            _ => Ok(false),
        }
    }
    fn sequence(
        &mut self,
        capacity: u32,
        element: &OrdinaryShape,
        name: &str,
        own: u32,
        ordered: Option<bool>,
    ) -> R<Count> {
        let child = if ordered == Some(true) {
            let OrdinaryShape::Product { fields } = element else {
                return Err(OrdinaryCarrierError::Shape);
            };
            self.product(fields, &format!("{name}.Element"), 0, false)?
        } else {
            self.shape(element, &format!("{name}.Element"))?
        };
        let indices = address_bits(capacity);
        if capacity == 0 || indices > 14 {
            return Err(OrdinaryCarrierError::Shape);
        }
        let array = indices + child.depth;
        let payload = 5.max(array);
        let depth = payload + 1;
        let fold = aggregate_fold::emit_fold(&mut self.r.b, indices)?;
        let mut address = vec![false; (depth - 5) as usize];
        let length = self
            .r
            .getter(&format!("{name}.Length"), depth, 5, &address)?;
        let source = self.r.b.var(1)?;
        let index = self.r.b.var(0)?;
        let mut args = vec![bit(&mut self.r.b, true)?];
        args.extend(vec![bit(&mut self.r.b, false)?; (payload - array) as usize]);
        for i in 0..indices {
            args.push(read_bit(&mut self.r.b, index, i)?);
        }
        // Return the selected child cube directly, rather than recomputing
        // all of its index selectors separately for each of its leaves.
        let body = self.r.b.app(source, args)?;
        let read = format!("{name}.ReadAt");
        define(&mut self.r.b, &read, &[depth, 5], child.depth, body)?;
        let ordering = if let Some(map) = ordered {
            if map {
                let OrdinaryShape::Product { fields } = element else {
                    return Err(OrdinaryCarrierError::Shape);
                };
                if fields.len() != 2 || fields[0].id != "key" || fields[1].id != "value" {
                    return Err(OrdinaryCarrierError::Shape);
                }
                let key = self
                    .r
                    .shape(&fields[0].shape, &format!("{name}.KeyRelation"), None)?;
                let roles = address_bits(fields.len() as u32);
                address = vec![false; (child.depth - key.depth) as usize];
                if child.depth < roles + key.depth {
                    return Err(OrdinaryCarrierError::Shape);
                }
                let getter =
                    self.r
                        .getter(&format!("{name}.Key"), child.depth, key.depth, &address)?;
                Some((
                    key.compare.ok_or(OrdinaryCarrierError::Shape)?,
                    Some(getter),
                ))
            } else {
                let key = self
                    .r
                    .shape(element, &format!("{name}.KeyRelation"), None)?;
                Some((key.compare.ok_or(OrdinaryCarrierError::Shape)?, None))
            }
        } else {
            None
        };
        let zero_tail = zero_tail_definition(&mut self.r.b, indices, child.depth)?;
        // Under the index-selector group, capture the entire source value.
        let source = self.r.b.var(indices)?;
        let index = index_word(&mut self.r.b, indices)?;
        let len = call(&mut self.r.b, &length, vec![source])?;
        let value = call(&mut self.r.b, &read, vec![source, index])?;
        let mut cells = call(&mut self.r.b, &child.definition, vec![value])?;
        if let Some((compare, key)) = ordering {
            let next = fold_helper(&mut self.r.b, "Add1", vec![index])?;
            let within = fold_helper(&mut self.r.b, "Less", vec![next, len])?;
            let next_value = call(&mut self.r.b, &read, vec![source, next])?;
            let (left, right) = if let Some(key) = key {
                (
                    call(&mut self.r.b, &key, vec![value])?,
                    call(&mut self.r.b, &key, vec![next_value])?,
                )
            } else {
                (value, next_value)
            };
            let order = call(&mut self.r.b, &compare, vec![left, right])?;
            let increasing = read_bit(&mut self.r.b, order, 31)?;
            let yes = bit(&mut self.r.b, true)?;
            let valid = mux(&mut self.r.b, within, increasing, yes)?;
            cells = reject_unless(&mut self.r.b, valid, cells)?;
        }
        let predicate = self.r.b.wrap_selectors(indices, cells)?;
        // Sum only active elements. Inactive storage is checked separately,
        // including rounded-up physical slots beyond the declared capacity.
        let source = self.r.b.var(0)?;
        let len = call(&mut self.r.b, &length, vec![source])?;
        let sum = if ordered.is_none() && self.unconstrained_unit_cell(element)? {
            // Exact sum of `length` constant-one counts. Bounds and all
            // physical padding/tail checks below remain mandatory.
            len
        } else {
            call(&mut self.r.b, &fold.sum_definition, vec![predicate, len])?
        };
        let own = word(&mut self.r.b, own)?;
        let count = add(&mut self.r.b, own, sum)?;
        let source = self.r.b.var(0)?;
        let len = call(&mut self.r.b, &length, vec![source])?;
        let limit = word(&mut self.r.b, capacity + 1)?;
        let bound = fold_helper(&mut self.r.b, "Less", vec![len, limit])?;
        let length_padding = padding(&mut self.r.b, source, depth, &[false], 5)?;
        let array_padding = padding(&mut self.r.b, source, depth, &[true], array)?;
        let mut address = vec![bit(&mut self.r.b, true)?];
        address.extend(vec![bit(&mut self.r.b, false)?; (payload - array) as usize]);
        let elements = self.r.b.app(source, address)?;
        let tail = call(&mut self.r.b, &zero_tail, vec![len, elements])?;
        let valid = and(&mut self.r.b, length_padding, array_padding)?;
        let valid = and(&mut self.r.b, valid, tail)?;
        let valid = and(&mut self.r.b, bound, valid)?;
        let count = reject_unless(&mut self.r.b, valid, count)?;
        self.finish_count(name, depth, count)
    }
    fn shape(&mut self, shape: &OrdinaryShape, name: &str) -> R<Count> {
        match shape {
            OrdinaryShape::Bits { width } => {
                let depth = address_bits(*width);
                if depth > 9 {
                    return Err(OrdinaryCarrierError::Shape);
                }
                let source = self.r.b.var(0)?;
                let mut valid = bit(&mut self.r.b, true)?;
                for i in *width..1 << depth {
                    let empty = zero_region(&mut self.r.b, source, depth, &prefix(depth, i))?;
                    valid = and(&mut self.r.b, valid, empty)?;
                }
                let one = word(&mut self.r.b, 1)?;
                let count = reject_unless(&mut self.r.b, valid, one)?;
                self.finish_count(name, depth, count)
            }
            OrdinaryShape::Reference { type_id } if type_id == "mpk.csharp.value.exception.v1" => {
                let depth = self
                    .r
                    .carriers
                    .get(type_id)
                    .ok_or(OrdinaryCarrierError::Linkage)?
                    .depth;
                let count = word(&mut self.r.b, INVALID)?;
                self.finish_count(name, depth, count)
            }
            OrdinaryShape::Reference { type_id } => self.ty(type_id),
            OrdinaryShape::RoleBound { maximum, value } => {
                let child = self.shape(value, &format!("{name}.Value"))?;
                let resolved = self.resolve(value)?;
                let OrdinaryShape::Sequence { capacity, .. } = resolved else {
                    return Err(OrdinaryCarrierError::Shape);
                };
                if *maximum == 0 || *maximum > capacity {
                    return Err(OrdinaryCarrierError::Shape);
                }
                let address = vec![false; (child.depth - 5) as usize];
                let length = self
                    .r
                    .getter(&format!("{name}.Length"), child.depth, 5, &address)?;
                let source = self.r.b.var(0)?;
                let len = call(&mut self.r.b, &length, vec![source])?;
                let limit = word(&mut self.r.b, maximum + 1)?;
                let within = fold_helper(&mut self.r.b, "Less", vec![len, limit])?;
                let empty = fold_helper(&mut self.r.b, "Empty", vec![len])?;
                let nonempty = not(&mut self.r.b, empty)?;
                let valid = and(&mut self.r.b, nonempty, within)?;
                let count = call(&mut self.r.b, &child.definition, vec![source])?;
                let count = reject_unless(&mut self.r.b, valid, count)?;
                self.finish_count(name, child.depth, count)
            }
            OrdinaryShape::Product { fields } => self.product(fields, name, 1, false),
            OrdinaryShape::Sum { arms } => self.sum(arms, name, false),
            OrdinaryShape::Sequence { capacity, element } => {
                self.sequence(*capacity, element, name, 1, None)
            }
            OrdinaryShape::Array { .. } => Err(OrdinaryCarrierError::Shape),
        }
    }
}
pub(super) fn emit_representation_domains<'a>(
    r: Relations<'a>,
    carriers: &[OrdinaryCarrier],
) -> R<(Relations<'a>, Vec<OrdinaryDomainDefinition>)> {
    let mut d = Domains {
        r,
        public_clauses: None,
        counts: BTreeMap::new(),
        active: BTreeSet::new(),
    };
    if !d.r.b.globals.contains_key(&format!("{PREFIX}.Cube.D5.Mux")) {
        d.r.b.helpers(5)?;
    }
    ordered_fold::auxiliary(&mut d.r.b)?;
    let mut definitions = vec![];
    for carrier in carriers {
        if d.r.internal(&carrier.type_id) {
            continue;
        }
        let count = d.ty(&carrier.type_id)?;
        let source = d.r.b.var(0)?;
        let cells = call(&mut d.r.b, &count.definition, vec![source])?;
        let valid = valid_count(&mut d.r.b, cells)?;
        let valid_definition = format!("{}.Valid", name(&carrier.type_id));
        define(&mut d.r.b, &valid_definition, &[carrier.depth], 0, valid)?;
        definitions.push(OrdinaryDomainDefinition {
            carrier: carrier.clone(),
            count_definition: count.definition,
            valid_definition,
        });
    }
    Ok((d.r, definitions))
}
pub fn generate_csharp_practical_ordinary_domains(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryDomainProgram> {
    let layouts = generate_csharp_practical_ordinary_carriers(vir)?;
    let r = Relations {
        vir,
        shared_folds: true,
        observations: false,
        carriers: layouts
            .carriers()
            .iter()
            .map(|c| (c.type_id.clone(), c.clone()))
            .collect(),
        b: Builder::new()?,
        nodes: BTreeMap::new(),
        active: BTreeSet::new(),
        raw: BTreeMap::new(),
        special: BTreeMap::new(),
        storage: StorageCache::default(),
    };
    let (r, definitions) = emit_representation_domains(r, layouts.carriers())?;
    let static_transformers = r.b.static_transformers;
    let certificate = r.b.finish()?;
    let p = OrdinaryDomainProgram {
        schema: "mpk.csharp.ordinary_domains.v1".into(),
        source_ir_sha256: vir.hash().into(),
        foundation_sha256: vir.construction_context().0.content_sha256().into(),
        definitions,
        static_transformers,
        certificate_sha256: mpk_cert::hash_hex(&mpk_cert::certificate_hash(&certificate)),
        certificate,
    };
    if p.canonical_bytes().len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    Ok(p)
}
pub fn import_csharp_practical_ordinary_domains(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryDomainProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let expected = generate_csharp_practical_ordinary_domains(vir)?;
    if input != expected.canonical_bytes() || certificate != expected.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(expected)
}

#[cfg(test)]
mod tests {
    use super::super::super::super::tests::{bit as observed_bit, run, V};
    use super::*;

    fn length_value(length: u32) -> V {
        V::Cube((0..32).map(|i| length & (1 << i) != 0).collect())
    }

    #[test]
    fn recursive_domain_zero_tail_matches_physical_addresses() {
        let mut b = Builder::new().unwrap();
        let names = [0, 2].map(|child| zero_tail_definition(&mut b, 3, child).unwrap());
        let singleton = zero_tail_definition(&mut b, 0, 0).unwrap();
        let cert = mpk_cert::decode_canonical_certificate(&b.finish().unwrap()).unwrap();
        for length in 0..=8 {
            // All eight-slot Boolean arrays, including odd boundaries.
            for mask in 0u32..256 {
                let expected = (length..8).all(|i| mask & (1 << i) == 0);
                let bits = (0..8).map(|i| mask & (1 << i) != 0).collect();
                assert_eq!(
                    observed_bit(run(
                        &cert,
                        &names[0],
                        vec![length_value(length), V::Cube(bits)]
                    )),
                    expected,
                    "length {length}, mask {mask}"
                );
            }
            // Physical addresses carry the index in the low three bits,
            // independently of the higher child-payload selectors.
            for address in 0..32 {
                let mut bits = vec![false; 32];
                bits[address] = true;
                assert_eq!(
                    observed_bit(run(
                        &cert,
                        &names[1],
                        vec![length_value(length), V::Cube(bits)]
                    )),
                    (address & 7) < length as usize,
                    "length {length}, payload address {address}"
                );
            }
        }
        for length in [0, 1] {
            for value in [false, true] {
                assert_eq!(
                    observed_bit(run(
                        &cert,
                        &singleton,
                        vec![length_value(length), V::Bit(value)]
                    )),
                    length == 1 || !value
                );
            }
        }
    }

    #[test]
    fn recursive_domain_sparse_zero_tail_work_is_bounded() {
        use super::super::super::super::test_eval::{apply_counted, sparse_cube};
        let mut b = Builder::new().unwrap();
        let shapes = [(14u32, 4u32), (12, 20)];
        let names =
            shapes.map(|(indices, child)| zero_tail_definition(&mut b, indices, child).unwrap());
        let cert = mpk_cert::decode_canonical_certificate(&b.finish().unwrap()).unwrap();
        for ((indices, child), name) in shapes.into_iter().zip(names) {
            let capacity = 1usize << indices;
            let last_payload = ((1usize << child) - 1) << indices;
            for length in [0usize, 1, 2, 3, capacity / 2, capacity - 1, capacity] {
                let mut active = BTreeSet::new();
                if length > 0 {
                    active.insert(0);
                    active.insert(last_payload | (length - 1));
                }
                let mut cases = vec![("active", active.clone(), true)];
                if length < capacity {
                    let mut first = active.clone();
                    first.insert(last_payload | length);
                    cases.push(("first inactive", first, false));
                    active.insert(last_payload | (capacity - 1));
                    cases.push(("last physical", active, false));
                }
                for (case, bits, expected) in cases {
                    let f = run(&cert, &name, vec![]);
                    let (f, prefix_steps) = apply_counted(&cert, f, length_value(length as u32));
                    let (result, steps) =
                        apply_counted(&cert, f, sparse_cube(indices + child, bits));
                    let steps = steps + prefix_steps;
                    assert_eq!(
                        observed_bit(result),
                        expected,
                        "I{indices} D{child} length {length} {case}"
                    );
                    assert!(
                        steps < 10_000_000,
                        "I{indices} D{child} length {length} {case}: {steps} transitions"
                    );
                    eprintln!("zero tail I{indices} D{child} length {length} {case}: {steps} core transitions");
                }
            }
        }
    }

    #[test]
    fn recursive_domain_sparse_zero_work_is_bounded() {
        use super::super::super::super::test_eval::{apply_counted, sparse_cube};
        let mut b = Builder::new().unwrap();
        b.helpers(5).unwrap();
        // This is the actual predicate used by the first C33 map length-
        // padding region, with all 2^31 addresses.
        let name = zero_definition(&mut b, 31).unwrap();
        let cert = mpk_cert::decode_canonical_certificate(&b.finish().unwrap()).unwrap();
        let f = run(&cert, &name, vec![]);
        let (zero, steps) = apply_counted(&cert, f, sparse_cube(31, BTreeSet::new()));
        assert!(observed_bit(zero));
        assert!(steps < 1_000_000, "C31 zero-region reduction cost: {steps}");
        eprintln!("C31 complete zero-region predicate: {steps} core transitions");
        // A single nonzero leaf, including the last one, must still reject.
        for index in [0usize, 1 << 30, (1usize << 31) - 1] {
            let f = run(&cert, &name, vec![]);
            let (zero, steps) =
                apply_counted(&cert, f, sparse_cube(31, [index].into_iter().collect()));
            assert!(!observed_bit(zero), "C31 nonzero address {index}");
            assert!(
                steps < 1_000_000,
                "C31 nonzero address {index} reduction cost: {steps}"
            );
            eprintln!("C31 nonzero address {index}: {steps} core transitions");
        }
    }

    #[test]
    fn recursive_domain_zero_regions_and_padding() {
        let mut b = Builder::new().unwrap();
        b.helpers(5).unwrap();
        let mut zeros = vec![];
        for depth in 0..=15 {
            zeros.push(zero_definition(&mut b, depth).unwrap());
        }
        let wide = zero_definition(&mut b, 20).unwrap();
        let count = b.static_transformers;
        assert_eq!(count, 0);
        for depth in 0..=15 {
            zero_definition(&mut b, depth).unwrap();
        }
        assert_eq!(b.static_transformers, count);
        // The selected region is the odd branch, with three zero padding
        // selectors before a four-bit child. The even branch is unconstrained.
        let source = b.var(0).unwrap();
        let body = padding(&mut b, source, 6, &[true], 2).unwrap();
        define(&mut b, "Test.Padding", &[6], 0, body).unwrap();
        let cert = mpk_cert::decode_canonical_certificate(&b.finish().unwrap()).unwrap();
        for (depth, zero) in zeros.iter().enumerate().take(9) {
            let value = |bits: Vec<bool>| {
                if depth == 0 {
                    V::Bit(bits[0])
                } else {
                    V::Cube(bits)
                }
            };
            let bits = vec![false; 1 << depth];
            assert!(observed_bit(run(&cert, zero, vec![value(bits.clone())])));
            for i in 0..bits.len() {
                let mut bits = bits.clone();
                bits[i] = true;
                assert!(
                    !observed_bit(run(&cert, zero, vec![value(bits)])),
                    "depth {depth} bit {i}"
                );
            }
        }
        for index in 0..64 {
            let mut bits = vec![false; 64];
            bits[index] = true;
            let allowed = index & 1 == 0 || index & 14 == 0;
            assert_eq!(
                observed_bit(run(&cert, "Test.Padding", vec![V::Cube(bits)])),
                allowed,
                "padding address {index}"
            );
        }
        // Cover high storage selectors on the 15-bit region.
        for index in [0, 1 << 14] {
            let mut bits = vec![false; 1 << 15];
            bits[index] = true;
            assert!(!observed_bit(run(&cert, &zeros[15], vec![V::Cube(bits)])));
        }
        // Binary partitions reach the fixed scalar word without aliasing
        // high address bits or omitting either selector branch.
        for index in [0, 1 << 14, 1 << 19] {
            let mut bits = vec![false; 1 << 20];
            bits[index] = true;
            assert!(!observed_bit(run(&cert, &wide, vec![V::Cube(bits)])));
        }
    }
}

#[path = "csharp_practical_ordinary_public_domains.rs"]
mod public_domains;
pub(super) fn emit_binding_domains<'a>(
    r: Relations<'a>,
    carriers: &[OrdinaryCarrier],
    clauses: &[OrdinarySourceClauseDefinition],
) -> R<(Relations<'a>, Vec<OrdinaryPublicDomainDefinition>)> {
    let mut d = Domains {
        r,
        public_clauses: Some(public_domains::clauses_by_type(clauses)?),
        counts: BTreeMap::new(),
        active: BTreeSet::new(),
    };
    let definitions = public_domains::emit_membership(&mut d, carriers)?;
    Ok((d.r, definitions))
}
pub use public_domains::{
    generate_csharp_practical_ordinary_public_defaults,
    generate_csharp_practical_ordinary_public_domains,
    import_csharp_practical_ordinary_public_defaults,
    import_csharp_practical_ordinary_public_domains, OrdinaryPublicDefaultDefinition,
    OrdinaryPublicDomainDefinition, OrdinaryPublicDomainProgram,
};
