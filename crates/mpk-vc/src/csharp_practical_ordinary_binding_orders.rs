//! W06 canonical order and key uniqueness. Representation domains remain separate.
use super::*;

fn index_word(b: &mut Builder, bits: u32) -> R<u32> {
    let mut body = bit(b, false)?;
    for i in 0..bits {
        let at = equal_address(b, 5, 0, 5, i)?;
        let value = b.var(5 + bits - 1 - i)?;
        body = mux(b, at, value, body)?;
    }
    b.wrap_selectors(5, body)
}
fn emit_order(a: &mut BindingAssembly<'_>, id: &str) -> R<()> {
    let symbol = format!("Mpk.CSharp.Binding.CanonicalOrder.{id}");
    if a.predicates.contains_key(&symbol) {
        return Ok(());
    }
    let r = &mut a.r;
    let target = r
        .carriers
        .get(id)
        .ok_or(OrdinaryCarrierError::Linkage)?
        .clone();
    let template = r.template(id).ok_or(OrdinaryCarrierError::Linkage)?;
    let map = match template.as_str() {
        "mpk.csharp.semantic.ordered_map.v1" => true,
        "mpk.csharp.semantic.ordered_set.v1" => false,
        _ => return Err(OrdinaryCarrierError::Shape),
    };
    let args = r
        .vir
        .data_closed()
        .metadata
        .get(id)
        .ok_or(OrdinaryCarrierError::Linkage)?
        .argument_ids
        .clone();
    if args.len() != if map { 2 } else { 1 } {
        return Err(OrdinaryCarrierError::Shape);
    }
    let OrdinaryShape::Sequence { capacity, element } = &target.shape else {
        return Err(OrdinaryCarrierError::Shape);
    };
    if *capacity != 4096 {
        return Err(OrdinaryCarrierError::Shape);
    }
    let key_id = &args[0];
    let refs = r.carriers.iter().map(|(id, c)| (id.as_str(), c)).collect();
    let element_depth = shape_depth(element, &refs)?;
    let key = r.ty(key_id)?;
    let compare = key.compare.ok_or(OrdinaryCarrierError::Shape)?;
    let key_get = if map {
        let OrdinaryShape::Product { fields } = element.as_ref() else {
            return Err(OrdinaryCarrierError::Shape);
        };
        if fields.len() != 2
            || fields[0].id != "key"
            || fields[1].id != "value"
            || reference(&fields[0].shape)? != key_id
            || reference(&fields[1].shape)? != args[1]
            || element_depth <= key.depth
        {
            return Err(OrdinaryCarrierError::Shape);
        }
        Some(r.getter(
            &name(&format!("canonical-key.{id}")),
            element_depth,
            key.depth,
            &vec![false; (element_depth - key.depth) as usize],
        )?)
    } else {
        if reference(element)? != key_id || element_depth != key.depth {
            return Err(OrdinaryCarrierError::Shape);
        }
        None
    };
    let indices = address_bits(*capacity);
    if target.depth != 1 + 5.max(indices + element_depth) {
        return Err(OrdinaryCarrierError::Shape);
    }
    let get_length = r.getter(
        &name(&format!("canonical-length.{id}")),
        target.depth,
        5,
        &vec![false; (target.depth - 5) as usize],
    )?;
    let read = name(&format!("canonical-read.{id}"));
    let source = r.b.var(1)?;
    let index = r.b.var(0)?;
    let mut address = vec![bit(&mut r.b, true)?];
    address.extend(vec![
        bit(&mut r.b, false)?;
        (target.depth - 1 - indices - element_depth) as usize
    ]);
    for i in 0..indices {
        address.push(ordered_fold::read_bit(&mut r.b, index, i)?);
    }
    let body = r.b.app(source, address)?;
    define(&mut r.b, &read, &[target.depth, 5], element_depth, body)?;
    let fold = aggregate_fold::emit_fold(&mut r.b, indices)?;
    let source = r.b.var(indices)?;
    let index = index_word(&mut r.b, indices)?;
    let next = ordered_fold::helper(&mut r.b, "Add1", vec![index])?;
    let length = call(&mut r.b, &get_length, vec![source])?;
    let within = ordered_fold::helper(&mut r.b, "Less", vec![next, length])?;
    let mut left = call(&mut r.b, &read, vec![source, index])?;
    let mut right = call(&mut r.b, &read, vec![source, next])?;
    if let Some(get) = key_get {
        left = call(&mut r.b, &get, vec![left])?;
        right = call(&mut r.b, &get, vec![right])?;
    }
    let order = call(&mut r.b, &compare, vec![left, right])?;
    // Canonical comparators return the signed word -1, 0 or 1.
    let increasing = ordered_fold::read_bit(&mut r.b, order, 31)?;
    let yes = bit(&mut r.b, true)?;
    let body = mux(&mut r.b, within, increasing, yes)?;
    let predicate = r.b.wrap_selectors(indices, body)?;
    let source = r.b.var(0)?;
    let length = call(&mut r.b, &get_length, vec![source])?;
    let limit = ordered_fold::word(&mut r.b, *capacity + 1)?;
    let bounded = ordered_fold::helper(&mut r.b, "Less", vec![length, limit])?;
    let ordered = call(&mut r.b, &fold.all_definition, vec![predicate, length])?;
    let body = and(&mut r.b, bounded, ordered)?;
    let definition = name(&symbol);
    define(&mut r.b, &definition, &[target.depth], 0, body)?;
    insert(&mut a.predicates, symbol, definition, vec![id.into()])
}
pub fn generate_csharp_practical_ordinary_binding_orders(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryBindingRelationProgram> {
    let construction = generate_construction_vcs(vir).map_err(|_| OrdinaryCarrierError::Linkage)?;
    let vc = generate_binding_vcs(vir, &construction).map_err(|_| OrdinaryCarrierError::Linkage)?;
    let layouts = generate_csharp_practical_ordinary_carriers(vir)?;
    let mut a = assemble(vir, &vc, &layouts)?;
    guards::emit_guards(&mut a, &vc)?;
    for symbol in vc.definition_names() {
        if let Some(id) = symbol.strip_prefix("Mpk.CSharp.Binding.CanonicalOrder.") {
            emit_order(&mut a, id)?;
        }
    }
    finish(vir, &vc, a, "mpk.csharp.ordinary_binding_orders.v1")
}
pub fn import_csharp_practical_ordinary_binding_orders(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryBindingRelationProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let p = generate_csharp_practical_ordinary_binding_orders(vir)?;
    if input != p.canonical_bytes() || certificate != p.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(p)
}
