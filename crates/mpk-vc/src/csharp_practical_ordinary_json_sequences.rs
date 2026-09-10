//! Concrete bounded-sequence JSON parsing, using existing typed child packets.
use super::super::integer_format::circuit_with_block_bits;
use super::super::temporal::literal;
use super::*;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryJsonSequenceDefinition {
    pub carrier: OrdinaryCarrier,
    pub element_type_id: String,
    pub capacity: u32,
    pub parse_definition: String,
    pub packet_depth: u32,
    pub header_definition: String,
    pub value_definition: String,
    pub child_parse_definition: String,
    pub pipeline_definition: String,
}

fn cube(b: &mut Builder, depth: u32) -> R<()> {
    if !b
        .globals
        .contains_key(&format!("{PREFIX}.Cube.D{depth}.Mux"))
    {
        b.helpers(depth)?;
    }
    Ok(())
}
pub(super) fn zero(b: &mut Builder, depth: u32) -> R<u32> {
    cube(b, depth)?;
    b.constant(&format!("{PREFIX}.Cube.D{depth}.Zero"))
}
pub(super) fn mux(b: &mut Builder, depth: u32, selected: u32, yes: u32, no: u32) -> R<u32> {
    cube(b, depth)?;
    call(
        b,
        &format!("{PREFIX}.Cube.D{depth}.Mux"),
        vec![selected, yes, no],
    )
}
fn helpers(b: &mut Builder) -> R<(String, String, String)> {
    let base = "Mpk.CSharp.Ordinary.JsonSequences";
    let increment = format!("{base}.Increment");
    let nonempty = format!("{base}.Nonempty");
    let active = format!("{base}.ActiveHeader");
    if !b.globals.contains_key(&increment) {
        let mut c = Circuit::new(&[32]);
        let n = c.inputs[0].clone();
        let out = c.add(&n, &literal(1, 32), F).0;
        let f = circuit_with_block_bits(b, &increment, c, out, 7)?;
        let n = b.var(0)?;
        let body = call(b, &f, vec![n])?;
        define(b, &increment, &[5], 5, body)?;
        let mut c = Circuit::new(&[32]);
        let n = c.inputs[0].clone();
        let empty = c.equal(&n, &literal(0, 32));
        let out = c.not(empty);
        let f = circuit_with_block_bits(b, &nonempty, c, vec![out], 7)?;
        let n = b.var(0)?;
        let body = call(b, &f, vec![n])?;
        define(b, &nonempty, &[5], 0, body)?;
        let mut c = Circuit::new(&[128, 8]);
        let head = c.inputs[0].clone();
        let next = c.inputs[1].clone();
        let closed = c.equal(&next, &literal(b']' as u128, 8));
        let open = c.not(closed);
        let available = c.not(head[1]);
        let valid = c.and(head[0], available);
        let out = c.and(valid, open);
        let f = circuit_with_block_bits(b, &active, c, vec![out], 7)?;
        let h = b.var(1)?;
        let next = b.var(0)?;
        let body = call(b, &f, vec![h, next])?;
        define(b, &active, &[7, 3], 0, body)?;
    }
    Ok((increment, nonempty, active))
}

/// Complete finite guarded composition. Group occurrences and pipeline
/// occurrences are both charged through Builder's original accounting.
fn pipeline(b: &mut Builder, depth: u32, capacity: u32) -> R<String> {
    if !capacity.is_power_of_two() || !(8..=16_384).contains(&capacity) {
        return Err(OrdinaryCarrierError::Shape);
    }
    let base = format!("Mpk.CSharp.Ordinary.JsonSequences.Fold.D{depth}");
    let name = format!("{base}.N{capacity}");
    if b.globals.contains_key(&name) {
        return Ok(name);
    }
    cube(b, depth)?;
    let state_ty = b.cube(depth)?;
    let transformer = b.pi(state_ty, state_ty)?;
    let predicate = b.pi(state_ty, b.boolean)?;
    let composition = format!("{base}.Compose");
    let group = format!("{base}.StepEight");
    if !b.globals.contains_key(&composition) {
        // active, f, g, state; bind f(state) before observing or continuing it.
        let f = b.var(2)?;
        let state = b.var(0)?;
        let first = b.app(f, vec![state])?;
        let active = b.var(4)?;
        let g = b.var(2)?;
        let current = b.var(0)?;
        let keep_going = b.app(active, vec![current])?;
        let next = b.app(g, vec![current])?;
        let body = mux(b, depth, keep_going, next, current)?;
        let mut body = b.term(TermNode::Let {
            ty: state_ty,
            value: first,
            body,
        })?;
        for ty in [state_ty, transformer, transformer, predicate] {
            body = b.lam(ty, body)?;
        }
        let ty = b.pi(transformer, transformer)?;
        let ty = b.pi(transformer, ty)?;
        let ty = b.pi(predicate, ty)?;
        b.define(&composition, ty, body)?;
        let active = b.var(1)?;
        let step = b.var(0)?;
        let compose = call(b, &composition, vec![active])?;
        let body = b.compose_term(depth, compose, &[step; 8])?;
        let body = b.lam(transformer, body)?;
        let body = b.lam(predicate, body)?;
        let ty = b.pi(transformer, transformer)?;
        let ty = b.pi(predicate, ty)?;
        b.define(&group, ty, body)?;
    }
    let active = b.var(2)?;
    let step = b.var(1)?;
    let state = b.var(0)?;
    let grouped = call(b, &group, vec![active, step])?;
    let compose = call(b, &composition, vec![active])?;
    let fold = b.compose_term(depth, compose, &vec![grouped; (capacity / 8) as usize])?;
    let next = b.app(fold, vec![state])?;
    let keep_going = b.app(active, vec![state])?;
    let body = mux(b, depth, keep_going, next, state)?;
    let body = b.lam(state_ty, body)?;
    let body = b.lam(transformer, body)?;
    let body = b.lam(predicate, body)?;
    let ty = b.pi(transformer, transformer)?;
    let ty = b.pi(predicate, ty)?;
    b.define(&name, ty, body)?;
    Ok(name)
}

fn storage(
    b: &mut Builder,
    capacity: u32,
    element_depth: u32,
    increment: &str,
) -> R<(String, String)> {
    let indices = address_bits(capacity);
    let padded = 5.max(indices + element_depth);
    let depth = padded + 1;
    let base = format!("Mpk.CSharp.Ordinary.JsonSequences.Storage.N{capacity}.D{element_depth}");
    let length = format!("{base}.Length");
    let append = format!("{base}.Append");
    if b.globals.contains_key(&length) {
        return Ok((length, append));
    }
    let source = b.var(5)?;
    let mut args = vec![truth(b, false)?; (1 + padded - 5) as usize];
    args.extend(b.selectors(5)?);
    let body = b.app(source, args)?;
    let body = b.wrap_selectors(5, body)?;
    define(b, &length, &[depth], 5, body)?;

    // source, index, element, followed by the complete sequence selectors.
    let source = b.var(depth + 2)?;
    let index = b.var(depth + 1)?;
    let element = b.var(depth)?;
    let new_length = call(b, increment, vec![index])?;
    let selectors = (0..5).map(|i| b.var(4 - i)).collect::<R<Vec<_>>>()?;
    let mut length_leaf = b.app(new_length, selectors)?;
    for i in 0..padded - 5 {
        let selected = b.var(padded - 1 - i)?;
        let no = truth(b, false)?;
        length_leaf = core_mux(b, selected, no, length_leaf)?;
    }
    let mut matches = truth(b, true)?;
    for i in 0..32 {
        let actual = core_read(b, index, i as usize, 5)?;
        let expected = if i < indices {
            b.var(padded - 1 - i)?
        } else {
            truth(b, false)?
        };
        let not_expected = call(b, "Std.Bool.not", vec![expected])?;
        let eq = core_mux(b, actual, expected, not_expected)?;
        let no = truth(b, false)?;
        matches = core_mux(b, matches, eq, no)?;
    }
    let selectors = (0..element_depth)
        .map(|i| b.var(padded - 1 - indices - i))
        .collect::<R<Vec<_>>>()?;
    let child_leaf = b.app(element, selectors)?;
    let selectors = b.selectors(depth)?;
    let old_leaf = b.app(source, selectors)?;
    let data_leaf = core_mux(b, matches, child_leaf, old_leaf)?;
    let role = b.var(padded)?;
    let body = core_mux(b, role, data_leaf, length_leaf)?;
    let body = b.wrap_selectors(depth, body)?;
    define(b, &append, &[depth, 5, element_depth], depth, body)?;
    Ok((length, append))
}

fn array_child(b: &mut Builder, child: &Child) -> R<String> {
    let name = format!("{}.ArrayElement", child.parse);
    if b.globals.contains_key(&name) {
        return Ok(name);
    }
    let mut scope = Scope::default();
    let args = |b: &mut Builder, scope: &Scope, ending| -> R<Vec<u32>> {
        let doc = scope.outer(b, 2)?;
        let start = scope.outer(b, 1)?;
        let mut args = vec![doc, start, word(b, ending, 3)?];
        if child.compound {
            args.push(scope.outer(b, 0)?);
        }
        Ok(args)
    };
    let input = args(b, &scope, 2)?;
    let first = call(b, &child.parse, input)?;
    let first = scope.bind(b, child.packet_depth, first)?;
    let first_value = scope.bound(b, first)?;
    let head = call(b, &child.header, vec![first_value])?;
    let valid = core_read(b, head, 0, 7)?;
    let input = args(b, &scope, 1)?;
    let second = call(b, &child.parse, input)?;
    let body = mux(b, child.packet_depth, valid, first_value, second)?;
    let body = scope.finish(b, body)?;
    define(b, &name, &[24, 5, 5], child.packet_depth, body)?;
    Ok(name)
}

impl Products<'_> {
    pub(super) fn sequence(&mut self, id: &str) -> R<Option<Child>> {
        let carrier = (*self.carriers.get(id).ok_or(OrdinaryCarrierError::Shape)?).clone();
        let OrdinaryShape::Sequence { capacity, element } = &carrier.shape else {
            return Err(OrdinaryCarrierError::Shape);
        };
        let OrdinaryShape::Reference { type_id } = element.as_ref() else {
            return Err(OrdinaryCarrierError::Shape);
        };
        if *capacity != 4096 {
            return Err(OrdinaryCarrierError::Linkage);
        }
        let element_id = type_id.clone();
        let Some(child) = self.ty(&element_id)? else {
            self.active.remove(id);
            self.deferred.insert(id.into());
            return Ok(None);
        };
        let child_depth = self
            .carriers
            .get(element_id.as_str())
            .ok_or(OrdinaryCarrierError::Shape)?
            .depth;
        let (node, definition) =
            self.emit_sequence(id, &carrier, element_id, child, child_depth, None, None)?;
        self.sequences.insert(id.into(), definition);
        self.nodes.insert(id.into(), node.clone());
        self.active.remove(id);
        Ok(Some(node))
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn emit_sequence(
        &mut self,
        id: &str,
        carrier: &OrdinaryCarrier,
        element_id: String,
        child: Child,
        child_depth: u32,
        order: Option<&str>,
        join: Option<&str>,
    ) -> R<(Child, OrdinaryJsonSequenceDefinition)> {
        let OrdinaryShape::Sequence { capacity, .. } = &carrier.shape else {
            return Err(OrdinaryCarrierError::Shape);
        };
        if carrier.depth != 1 + 5.max(address_bits(*capacity) + child_depth) {
            return Err(OrdinaryCarrierError::Shape);
        }
        let packet_depth = carrier.depth.max(7) + 1;
        let (increment, nonempty, active_header) = helpers(self.b)?;
        let (length, append) = storage(self.b, *capacity, child_depth, &increment)?;
        let pipeline_definition = pipeline(self.b, packet_depth, *capacity)?;
        let child_parse = array_child(self.b, &child)?;
        let header = super::super::json_values::projection(self.b, packet_depth, 7, false)?;
        let value =
            super::super::json_values::projection(self.b, packet_depth, carrier.depth, true)?;
        let assemble = super::super::json_grammar::assemble(self.b, carrier.depth)?;
        let hex = id
            .as_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>();
        let base = format!("Mpk.CSharp.Ordinary.JsonSequences.T{hex}");
        let active_name = format!("{base}.Active");
        let step_name = format!("{base}.Step");
        let parse_name = format!("{base}.Parse");

        let doc = self.b.var(1)?;
        let state = self.b.var(0)?;
        let head = call(self.b, &header, vec![state])?;
        let cursor = call(self.b, &self.grammar.cursor_definition, vec![head])?;
        let next = call(
            self.b,
            &self.document.read_byte_definition,
            vec![doc, cursor],
        )?;
        let body = call(self.b, &active_header, vec![head, next])?;
        define(self.b, &active_name, &[24, packet_depth], 0, body)?;

        // Step(doc, parent_depth, state): leave the delimiter after each child
        // unconsumed. The next step consumes a comma; Finish consumes ']'.
        let mut s = Scope::default();
        let state = s.outer(self.b, 0)?;
        let head = call(self.b, &header, vec![state])?;
        let mut head = s.bind(self.b, 7, head)?;
        let state = s.outer(self.b, 0)?;
        let old = call(self.b, &value, vec![state])?;
        let old = s.bind(self.b, carrier.depth, old)?;
        let old_value = s.bound(self.b, old)?;
        let count = call(self.b, &length, vec![old_value])?;
        let count = s.bind(self.b, 5, count)?;
        let current = s.bound(self.b, head)?;
        let cursor = call(self.b, &self.grammar.cursor_definition, vec![current])?;
        let doc = s.outer(self.b, 2)?;
        let comma = call(self.b, &self.punctuation(b',')?, vec![doc, cursor])?;
        let comma = call(
            self.b,
            &self.grammar.syntax_definition,
            vec![current, comma],
        )?;
        let n = s.bound(self.b, count)?;
        let has_previous = call(self.b, &nonempty, vec![n])?;
        let selected = mux(self.b, 7, has_previous, comma, current)?;
        head = s.bind(self.b, 7, selected)?;
        let current = s.bound(self.b, head)?;
        let start = call(self.b, &self.grammar.cursor_definition, vec![current])?;
        let doc = s.outer(self.b, 2)?;
        let depth = s.outer(self.b, 1)?;
        let depth = call(self.b, &self.grammar.child_depth_definition, vec![depth])?;
        let parsed = call(self.b, &child_parse, vec![doc, start, depth])?;
        let parsed = s.bind(self.b, child.packet_depth, parsed)?;
        let raw = s.bound(self.b, parsed)?;
        let child_head = call(self.b, &child.header, vec![raw])?;
        let current = s.bound(self.b, head)?;
        let joined = call(
            self.b,
            join.unwrap_or(&self.grammar.child_definition),
            vec![current, child_head],
        )?;
        let raw = s.bound(self.b, parsed)?;
        let child_value = call(self.b, &child.value, vec![raw])?;
        let n = s.bound(self.b, count)?;
        let old_value = s.bound(self.b, old)?;
        let joined = if let Some(order) = order {
            let increasing = call(self.b, order, vec![old_value, child_value])?;
            let no = zero(self.b, 7)?;
            mux(self.b, 7, increasing, joined, no)?
        } else {
            joined
        };
        let updated = call(self.b, &append, vec![old_value, n, child_value])?;
        let body = call(self.b, &assemble, vec![joined, updated])?;
        let body = s.finish(self.b, body)?;
        define(
            self.b,
            &step_name,
            &[24, 5, packet_depth],
            packet_depth,
            body,
        )?;

        let mut s = Scope::default();
        let doc = s.outer(self.b, 3)?;
        let start = s.outer(self.b, 2)?;
        let depth = s.outer(self.b, 0)?;
        let doc_length = call(self.b, &self.document.length_definition, vec![doc])?;
        let first = call(self.b, &increment, vec![start])?;
        let next = call(
            self.b,
            &self.document.read_byte_definition,
            vec![doc, first],
        )?;
        let has_children = has_children(self.b, next)?;
        let begin = call(
            self.b,
            &self.grammar.begin_definition,
            vec![doc_length, start, depth, has_children],
        )?;
        let mut head = s.bind(self.b, 7, begin)?;
        head = syntax_step(self.b, &mut s, head, &self.punctuation(b'[')?, self.grammar)?;
        let initial_head = s.bound(self.b, head)?;
        let empty = zero(self.b, carrier.depth)?;
        let initial = call(self.b, &assemble, vec![initial_head, empty])?;
        let doc = s.outer(self.b, 3)?;
        let depth = s.outer(self.b, 0)?;
        let active = call(self.b, &active_name, vec![doc])?;
        let step = call(self.b, &step_name, vec![doc, depth])?;
        let folded = call(self.b, &pipeline_definition, vec![active, step, initial])?;
        let folded = s.bind(self.b, packet_depth, folded)?;
        let state = s.bound(self.b, folded)?;
        let final_head = call(self.b, &header, vec![state])?;
        head = s.bind(self.b, 7, final_head)?;
        head = syntax_step(self.b, &mut s, head, &self.punctuation(b']')?, self.grammar)?;
        let doc = s.outer(self.b, 3)?;
        let ending = s.outer(self.b, 1)?;
        let final_head = s.bound(self.b, head)?;
        let final_head = call(
            self.b,
            &self.grammar.finish_definition,
            vec![doc, final_head, ending],
        )?;
        let state = s.bound(self.b, folded)?;
        let final_value = call(self.b, &value, vec![state])?;
        let body = call(self.b, &assemble, vec![final_head, final_value])?;
        let body = s.finish(self.b, body)?;
        define(self.b, &parse_name, &[24, 5, 3, 5], packet_depth, body)?;
        let node = Child {
            parse: parse_name.clone(),
            header: header.clone(),
            value: value.clone(),
            packet_depth,
            compound: true,
        };
        Ok((
            node,
            OrdinaryJsonSequenceDefinition {
                carrier: carrier.clone(),
                element_type_id: element_id,
                capacity: *capacity,
                parse_definition: parse_name,
                packet_depth,
                header_definition: header,
                value_definition: value,
                child_parse_definition: child_parse,
                pipeline_definition,
            },
        ))
    }
}

fn has_children(b: &mut Builder, byte: u32) -> R<u32> {
    let name = "Mpk.CSharp.Ordinary.JsonSequences.HasChildren";
    if !b.globals.contains_key(name) {
        let mut c = Circuit::new(&[8]);
        let next = c.inputs[0].clone();
        let closed = c.equal(&next, &literal(b']' as u128, 8));
        let open = c.not(closed);
        let f = circuit_with_block_bits(b, name, c, vec![open], 7)?;
        let next = b.var(0)?;
        let body = call(b, &f, vec![next])?;
        define(b, name, &[3], 0, body)?;
    }
    call(b, name, vec![byte])
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn json_sequence_pipeline_accounts_groups_and_reuses_exact_depth() {
        let mut b = Builder::new().unwrap();
        let name = pipeline(&mut b, 14, 4096).unwrap();
        assert_eq!(b.static_transformers, 8 + 512);
        let bytes = b.c.term_table.len();
        assert_eq!(pipeline(&mut b, 14, 4096).unwrap(), name);
        assert_eq!(b.static_transformers, 8 + 512);
        assert_eq!(b.c.term_table.len(), bytes);
        pipeline(&mut b, 14, 16384).unwrap();
        assert_eq!(b.static_transformers, 8 + 512 + 2048);
        let cert = decode_canonical_certificate(&b.finish().unwrap()).unwrap();
        crate::csharp_practical_vc_model::validate_csharp_practical_certificate_structure(&cert)
            .unwrap();
        let mut b = Builder::new().unwrap();
        // Fault-inject the preceding program's cost, never reset a live emitter.
        b.static_transformers = 16384 - (8 + 512);
        pipeline(&mut b, 14, 4096).unwrap();
        assert_eq!(b.static_transformers, 16384);
        assert_eq!(
            pipeline(&mut b, 14, 16384),
            Err(OrdinaryCarrierError::Limit)
        );
    }
}
