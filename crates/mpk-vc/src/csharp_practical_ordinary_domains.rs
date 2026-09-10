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
/// All leaves are false. Each concrete depth shares the same bounded pipeline.
/// Splitting selector groups also handles padding whose depth exceeds 14.
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
        // every word inside a large region. Wider regions still use the
        // counted concrete state pipeline below.
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
        // Keep a full scalar word at the bottom of each concrete partition.
        // Otherwise depths 6..14 scan one Bool per state step and never use
        // the fixed-word base case, despite having the same scalar storage.
        let group = (depth - 5).min(14);
        let child = zero_definition(b, depth - group)?;
        let fold = aggregate_fold::emit_fold(b, group)?;
        let source = b.var(group)?;
        let selectors = b.selectors(group)?;
        let value = b.app(source, selectors)?;
        let valid = call(b, &child, vec![value])?;
        let predicate = b.wrap_selectors(group, valid)?;
        let count = word(b, 1 << group)?;
        call(b, &fold.all_definition, vec![predicate, count])?
    };
    define(b, &name, &[depth], 0, body)?;
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
        let mut count = word(&mut self.r.b, own)?;
        for (i, child) in children.iter().enumerate() {
            let leading = prefix(roles, i as u32);
            let padded = padding(&mut self.r.b, source, depth, &leading, child.depth)?;
            valid = and(&mut self.r.b, valid, padded)?;
            let mut address = leading;
            address.extend(vec![false; (max - child.depth) as usize]);
            let getter =
                self.r
                    .getter(&format!("{name}.Read.F{i}"), depth, child.depth, &address)?;
            let value = call(&mut self.r.b, &getter, vec![source])?;
            let cells = call(&mut self.r.b, &child.definition, vec![value])?;
            count = add(&mut self.r.b, count, cells)?;
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
        let zero_child = zero_definition(&mut self.r.b, child.depth)?;
        // Under the index-selector group, capture the entire source value.
        let source = self.r.b.var(indices)?;
        let index = index_word(&mut self.r.b, indices)?;
        let len = call(&mut self.r.b, &length, vec![source])?;
        let visible = fold_helper(&mut self.r.b, "Less", vec![index, len])?;
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
        let empty = call(&mut self.r.b, &zero_child, vec![value])?;
        let zero = word(&mut self.r.b, 0)?;
        let tail = reject_unless(&mut self.r.b, empty, zero)?;
        let cells = wmux(&mut self.r.b, visible, cells, tail)?;
        let predicate = self.r.b.wrap_selectors(indices, cells)?;
        let cap = word(&mut self.r.b, 1 << indices)?;
        let sum = call(&mut self.r.b, &fold.sum_definition, vec![predicate, cap])?;
        let own = word(&mut self.r.b, own)?;
        let count = add(&mut self.r.b, own, sum)?;
        let source = self.r.b.var(0)?;
        let len = call(&mut self.r.b, &length, vec![source])?;
        let limit = word(&mut self.r.b, capacity + 1)?;
        let bound = fold_helper(&mut self.r.b, "Less", vec![len, limit])?;
        let length_padding = padding(&mut self.r.b, source, depth, &[false], 5)?;
        let array_padding = padding(&mut self.r.b, source, depth, &[true], array)?;
        let valid = and(&mut self.r.b, length_padding, array_padding)?;
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
pub fn generate_csharp_practical_ordinary_domains(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryDomainProgram> {
    let layouts = generate_csharp_practical_ordinary_carriers(vir)?;
    let mut d = Domains {
        public_clauses: None,
        r: Relations {
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
        },
        counts: BTreeMap::new(),
        active: BTreeSet::new(),
    };
    d.r.b.helpers(5)?;
    ordered_fold::auxiliary(&mut d.r.b)?;
    let mut definitions = vec![];
    for carrier in layouts.carriers() {
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
    let static_transformers = d.r.b.static_transformers;
    let certificate = d.r.b.finish()?;
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
        assert_eq!(count, 8217);
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
        // D20 really uses a 14-selector outer group and a D6 child, whose
        // final partition reaches the fixed scalar word. No index may alias.
        for index in [0, 1 << 14, 1 << 19] {
            let mut bits = vec![false; 1 << 20];
            bits[index] = true;
            assert!(!observed_bit(run(&cert, &wide, vec![V::Cube(bits)])));
        }
    }
}

#[path = "csharp_practical_ordinary_public_domains.rs"]
mod public_domains;
pub use public_domains::{
    generate_csharp_practical_ordinary_public_defaults,
    generate_csharp_practical_ordinary_public_domains,
    import_csharp_practical_ordinary_public_defaults,
    import_csharp_practical_ordinary_public_domains, OrdinaryPublicDefaultDefinition,
    OrdinaryPublicDomainDefinition, OrdinaryPublicDomainProgram,
};
