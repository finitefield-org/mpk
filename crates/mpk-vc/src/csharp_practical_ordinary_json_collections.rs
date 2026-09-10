//! Ordered Map/Set JSON: complete storage, strict canonical order and cell roles.
use super::super::integer_format::circuit_with_block_bits;
use super::super::temporal::literal;
use super::*;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryJsonCollectionDefinition {
    pub sequence: OrdinaryJsonSequenceDefinition,
    pub template_id: String,
    pub key_type_id: String,
    pub key_compare_definition: String,
    pub order_definition: String,
    pub join_definition: String,
}
fn index_helpers(b: &mut Builder) -> R<(String, String)> {
    let previous = "Mpk.CSharp.Ordinary.JsonCollections.Previous";
    let nonempty = "Mpk.CSharp.Ordinary.JsonCollections.Nonempty";
    if !b.globals.contains_key(previous) {
        let mut c = Circuit::new(&[32]);
        let n = c.inputs[0].clone();
        let out = c.sub(&n, &literal(1, 32)).0;
        let f = circuit_with_block_bits(b, previous, c, out, 7)?;
        let n = b.var(0)?;
        let body = call(b, &f, vec![n])?;
        define(b, previous, &[5], 5, body)?;
        let mut c = Circuit::new(&[32]);
        let n = c.inputs[0].clone();
        let out = c.lt(&literal(0, 32), &n, false);
        let f = circuit_with_block_bits(b, nonempty, c, vec![out], 7)?;
        let n = b.var(0)?;
        let body = call(b, &f, vec![n])?;
        define(b, nonempty, &[5], 0, body)?;
    }
    Ok((previous.into(), nonempty.into()))
}
#[allow(clippy::too_many_arguments)]
fn order(
    b: &mut Builder,
    id: &str,
    depth: u32,
    element_depth: u32,
    key_depth: u32,
    compare: &str,
    map: bool,
) -> R<String> {
    let base = format!(
        "Mpk.CSharp.Ordinary.JsonCollections.T{}",
        id.as_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    );
    let length = format!("{base}.Length");
    let read = format!("{base}.Read");
    let key = format!("{base}.Key");
    let source = b.var(5)?;
    let mut args = vec![truth(b, false)?; (depth - 5) as usize];
    args.extend(b.selectors(5)?);
    let body = b.app(source, args)?;
    let body = b.wrap_selectors(5, body)?;
    define(b, &length, &[depth], 5, body)?;
    let source = b.var(element_depth + 1)?;
    let index = b.var(element_depth)?;
    let mut args = vec![truth(b, true)?];
    args.extend(vec![
        truth(b, false)?;
        (depth - 1 - 12 - element_depth) as usize
    ]);
    for i in 0..12 {
        args.push(core_read(b, index, i, 5)?);
    }
    args.extend(b.selectors(element_depth)?);
    let body = b.app(source, args)?;
    let body = b.wrap_selectors(element_depth, body)?;
    define(b, &read, &[depth, 5], element_depth, body)?;
    if map {
        let source = b.var(key_depth)?;
        let mut args = vec![truth(b, false)?; (element_depth - key_depth) as usize];
        args.extend(b.selectors(key_depth)?);
        let body = b.app(source, args)?;
        let body = b.wrap_selectors(key_depth, body)?;
        define(b, &key, &[element_depth], key_depth, body)?;
    }
    let (previous, nonempty) = index_helpers(b)?;
    let source = b.var(1)?;
    let mut right = b.var(0)?;
    let n = call(b, &length, vec![source])?;
    let index = call(b, &previous, vec![n])?;
    let mut left = call(b, &read, vec![source, index])?;
    if map {
        left = call(b, &key, vec![left])?;
        right = call(b, &key, vec![right])?;
    }
    let compared = call(b, compare, vec![left, right])?;
    let less = core_read(b, compared, 31, 5)?;
    let has_previous = call(b, &nonempty, vec![n])?;
    let yes = truth(b, true)?;
    let body = core_mux(b, has_previous, less, yes)?;
    let name = format!("{base}.Increasing");
    define(b, &name, &[depth, element_depth], 0, body)?;
    Ok(name)
}
impl Products<'_> {
    pub(super) fn collection(&mut self, id: &str) -> R<Option<Child>> {
        let metadata = self
            .vir
            .data_closed()
            .metadata
            .get(id)
            .ok_or(OrdinaryCarrierError::Linkage)?
            .clone();
        let map = match metadata.template_id.as_str() {
            "mpk.csharp.semantic.ordered_map.v1" => true,
            "mpk.csharp.semantic.ordered_set.v1" => false,
            _ => return Err(OrdinaryCarrierError::Shape),
        };
        if metadata.argument_ids.len() != if map { 2 } else { 1 } {
            return Err(OrdinaryCarrierError::Shape);
        }
        let carrier = (*self.carriers.get(id).ok_or(OrdinaryCarrierError::Shape)?).clone();
        let OrdinaryShape::Sequence {
            capacity: 4096,
            element,
        } = &carrier.shape
        else {
            return Err(OrdinaryCarrierError::Shape);
        };
        let key_id = &metadata.argument_ids[0];
        let element_id = if map {
            self.vir
                .data_closed()
                .metadata
                .iter()
                .find(|(_, m)| {
                    m.template_id == "mpk.csharp.semantic.ordered_entry.v1"
                        && m.argument_ids == metadata.argument_ids
                })
                .map(|(id, _)| id.clone())
                .ok_or(OrdinaryCarrierError::Linkage)?
        } else {
            key_id.clone()
        };
        let child_carrier = (*self
            .carriers
            .get(element_id.as_str())
            .ok_or(OrdinaryCarrierError::Shape)?)
        .clone();
        if map {
            if element.as_ref() != &child_carrier.shape {
                return Err(OrdinaryCarrierError::Shape);
            }
            let OrdinaryShape::Product { fields } = element.as_ref() else {
                return Err(OrdinaryCarrierError::Shape);
            };
            if fields.len() != 2
                || fields[0].id != "key"
                || fields[1].id != "value"
                || fields[0].shape
                    != (OrdinaryShape::Reference {
                        type_id: key_id.clone(),
                    })
                || fields[1].shape
                    != (OrdinaryShape::Reference {
                        type_id: metadata.argument_ids[1].clone(),
                    })
            {
                return Err(OrdinaryCarrierError::Shape);
            }
        } else if element.as_ref()
            != &(OrdinaryShape::Reference {
                type_id: key_id.clone(),
            })
        {
            return Err(OrdinaryCarrierError::Shape);
        }
        let Some(child) = self.ty(&element_id)? else {
            self.active.remove(id);
            self.deferred.insert(id.into());
            return Ok(None);
        };
        let relation = self
            .key_relations
            .iter()
            .find(|r| r.carrier.type_id == *key_id)
            .ok_or(OrdinaryCarrierError::Linkage)?;
        let compare = relation
            .compare_definition
            .clone()
            .ok_or(OrdinaryCarrierError::Shape)?;
        if carrier.depth != 1 + 5.max(12 + child_carrier.depth) {
            return Err(OrdinaryCarrierError::Shape);
        }
        let order = order(
            self.b,
            id,
            carrier.depth,
            child_carrier.depth,
            relation.carrier.depth,
            &compare,
            map,
        )?;
        let join = if map {
            self.container_join_definition()?
        } else {
            self.grammar.child_definition.clone()
        };
        let (node, sequence) = self.emit_sequence(
            id,
            &carrier,
            element_id,
            child,
            child_carrier.depth,
            Some(&order),
            Some(&join),
        )?;
        self.collections.insert(
            id.into(),
            OrdinaryJsonCollectionDefinition {
                sequence,
                template_id: metadata.template_id,
                key_type_id: key_id.clone(),
                key_compare_definition: compare,
                order_definition: order,
                join_definition: join,
            },
        );
        self.nodes.insert(id.into(), node.clone());
        self.active.remove(id);
        Ok(Some(node))
    }
}
