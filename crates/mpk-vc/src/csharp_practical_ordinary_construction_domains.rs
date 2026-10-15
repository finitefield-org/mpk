//! Private construction storage validity. Source ownership, origin/version,
//! lifetime and default eligibility remain separate original obligations.
use super::*;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryConstructionStorageDomainDefinition {
    pub carrier: OrdinaryCarrier,
    pub element_type_id: String,
    pub element_public_domain: String,
    pub element_count_definition: String,
    /// Original W06 domain symbol; this is never a public-value capability.
    pub symbol: String,
    pub count_definition: String,
    pub valid_definition: String,
    pub private_storage_only: bool,
    pub ownership_pending: bool,
}

pub(in super::super) fn emit(
    vir: &ValidatedPracticalVir,
    layouts: &OrdinaryCarrierProgram,
    b: Builder,
    public_domains: &[OrdinaryPublicDomainDefinition],
) -> R<(Builder, Vec<OrdinaryConstructionStorageDomainDefinition>)> {
    let mut r = Relations {
        vir,
        shared_folds: true,
        observations: false,
        carriers: layouts
            .carriers()
            .iter()
            .map(|c| (c.type_id.clone(), c.clone()))
            .collect(),
        b,
        nodes: BTreeMap::new(),
        active: BTreeSet::new(),
        raw: BTreeMap::new(),
        special: BTreeMap::new(),
        storage: StorageCache::default(),
    };
    let mut definitions = vec![];
    for entry in vir
        .data_closed()
        .entries()
        .iter()
        .filter(|e| e["template_id"] == "mpk.csharp.semantic.sequence_construction.v1")
    {
        let id = text(entry, "instance_id")?;
        let metadata = &vir.data_closed().metadata[id];
        if metadata.argument_ids.len() != 1 || metadata.dependency_ids.len() != 1 {
            return Err(OrdinaryCarrierError::Linkage);
        }
        let element_id = &metadata.argument_ids[0];
        let element = public_domains
            .iter()
            .find(|d| &d.carrier.type_id == element_id)
            .ok_or(OrdinaryCarrierError::Linkage)?;
        let carrier = r
            .carriers
            .get(id)
            .ok_or(OrdinaryCarrierError::Linkage)?
            .clone();
        let expected = product(vec![
            field("length", bits(32)),
            field(
                "cells",
                OrdinaryShape::Array {
                    capacity: 16384,
                    element: Box::new(reference(element_id)),
                },
            ),
            field(
                "initialized",
                OrdinaryShape::Array {
                    capacity: 16384,
                    element: Box::new(bits(1)),
                },
            ),
        ]);
        if carrier.shape != expected || carrier.depth != 16 + element.carrier.depth {
            return Err(OrdinaryCarrierError::Shape);
        }
        let name = format!(
            "{PREFIX}.ConstructionStorage.T{}",
            id.as_bytes()
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
        );
        let depth = carrier.depth;
        let child = element.carrier.depth;
        let mut getters = vec![];
        for (role, width) in [5, 14 + child, 14].into_iter().enumerate() {
            let mut address = prefix(2, role as u32);
            address.extend(vec![false; (depth - 2 - width) as usize]);
            getters.push(r.getter(&format!("{name}.Field{role}"), depth, width, &address)?);
        }
        // Every initialized element retains its full recursive public clauses
        // and logical-cell count. An uninitialized slot must have zero storage,
        // but need not be an inhabitant of the element's public domain.
        let source = r.b.var(1)?;
        let index = r.b.var(0)?;
        let cells = call(&mut r.b, &getters[1], vec![source])?;
        let bitmap = call(&mut r.b, &getters[2], vec![source])?;
        let address = (0..14)
            .map(|i| read_bit(&mut r.b, index, i))
            .collect::<R<Vec<_>>>()?;
        let value = r.b.app(cells, address.clone())?;
        let initialized = r.b.app(bitmap, address)?;
        let assigned = call(&mut r.b, &element.count_definition, vec![value])?;
        let zero = zero_definition(&mut r.b, child)?;
        let clear = call(&mut r.b, &zero, vec![value])?;
        let one = word(&mut r.b, 1)?;
        let unassigned = reject_unless(&mut r.b, clear, one)?;
        let count = wmux(&mut r.b, initialized, assigned, unassigned)?;
        let at = format!("{name}.CountAt");
        define(&mut r.b, &at, &[depth, 5], 5, count)?;

        let source = r.b.var(14)?;
        let index = index_word(&mut r.b, 14)?;
        let count = call(&mut r.b, &at, vec![source, index])?;
        let predicate = r.b.wrap_selectors(14, count)?;
        let source = r.b.var(0)?;
        let length = call(&mut r.b, &getters[0], vec![source])?;
        let fold = aggregate_fold::emit_fold(&mut r.b, 14)?;
        let count = call(&mut r.b, &fold.sum_definition, vec![predicate, length])?;
        let one = word(&mut r.b, 1)?;
        let count = add(&mut r.b, one, count)?;
        let limit = word(&mut r.b, 16385)?;
        let bound = fold_helper(&mut r.b, "Less", vec![length, limit])?;
        let mut valid = bound;
        for (role, width) in [5, 14 + child, 14].into_iter().enumerate() {
            let padded = padding(&mut r.b, source, depth, &prefix(2, role as u32), width)?;
            valid = and(&mut r.b, valid, padded)?;
        }
        let unused = zero_region(&mut r.b, source, depth, &prefix(2, 3))?;
        valid = and(&mut r.b, valid, unused)?;
        for (role, width) in [(1, child), (2, 0)] {
            let storage = call(&mut r.b, &getters[role], vec![source])?;
            let tail = zero_tail_definition(&mut r.b, 14, width)?;
            let clear = call(&mut r.b, &tail, vec![length, storage])?;
            valid = and(&mut r.b, valid, clear)?;
        }
        let count = reject_unless(&mut r.b, valid, count)?;
        let count_definition = format!("{name}.Count");
        define(&mut r.b, &count_definition, &[depth], 5, count)?;
        let count = call(&mut r.b, &count_definition, vec![source])?;
        let valid = valid_count(&mut r.b, count)?;
        let valid_definition = format!("{name}.Valid");
        define(&mut r.b, &valid_definition, &[depth], 0, valid)?;
        definitions.push(OrdinaryConstructionStorageDomainDefinition {
            carrier,
            element_type_id: element_id.clone(),
            element_public_domain: element.valid_definition.clone(),
            element_count_definition: element.count_definition.clone(),
            symbol: format!("Mpk.CSharp.PublicDomain.{id}"),
            count_definition,
            valid_definition,
            private_storage_only: true,
            ownership_pending: true,
        });
    }
    Ok((r.b, definitions))
}
