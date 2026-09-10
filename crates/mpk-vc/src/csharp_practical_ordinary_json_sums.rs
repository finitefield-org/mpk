//! Exact tagged semantic values; tags are syntax, payloads are semantic cells.
use super::super::integer_format::circuit_with_block_bits;
use super::super::temporal::literal;
use super::sequences::{mux, zero};
use super::*;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryJsonSumArm {
    pub arm_id: String,
    pub tag: u32,
    pub tag_literal: OrdinaryJsonSyntaxLiteral,
    pub parse_definition: String,
    pub payload_type_id: Option<String>,
    pub payload_parse_definition: Option<String>,
    pub payload_maximum: Option<u32>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryJsonSumDefinition {
    pub carrier: OrdinaryCarrier,
    pub template_id: String,
    pub arms: Vec<OrdinaryJsonSumArm>,
    pub parse_definition: String,
    pub packet_depth: u32,
    pub header_definition: String,
    pub value_definition: String,
}

pub(super) fn arm_names(template: &str) -> Option<&'static [&'static str]> {
    Some(match template {
        "mpk.csharp.semantic.option.v1" => &["none", "some"],
        "mpk.csharp.semantic.lookup.v1" => &["missing_key", "found"],
        "mpk.csharp.semantic.result.v1" => &["ok", "error"],
        "mpk.csharp.semantic.validation.v1" => &["valid", "invalid"],
        "mpk.csharp.semantic.boundary_field.v1" => &["missing", "null", "value"],
        _ => return None,
    })
}

/// Validation.invalid retains a sequence cell and requires 1..=256 errors.
/// The sequence parser independently checks its full 4096-slot value bound.
fn validation_payload(b: &mut Builder, value: u32, depth: u32) -> R<u32> {
    let name = "Mpk.CSharp.Ordinary.JsonSums.ValidationErrors";
    if !b.globals.contains_key(name) {
        let mut c = Circuit::new(&[32]);
        let n = c.inputs[0].clone();
        let positive = c.lt(&literal(0, 32), &n, false);
        let bounded = c.lt(&n, &literal(257, 32), false);
        let good = c.and(positive, bounded);
        let helper = circuit_with_block_bits(b, name, c, vec![good], 7)?;
        let n = b.var(0)?;
        let body = call(b, &helper, vec![n])?;
        define(b, name, &[5], 0, body)?;
    }
    let projection = format!("Mpk.CSharp.Ordinary.JsonSums.SequenceLength.D{depth}");
    if !b.globals.contains_key(&projection) {
        let source = b.var(5)?;
        let mut args = vec![truth(b, false)?; (depth - 5) as usize];
        args.extend(b.selectors(5)?);
        let body = b.app(source, args)?;
        let body = b.wrap_selectors(5, body)?;
        define(b, &projection, &[depth], 5, body)?;
    }
    let length = call(b, &projection, vec![value])?;
    call(b, name, vec![length])
}

impl Products<'_> {
    pub(super) fn sum(&mut self, id: &str) -> R<Option<Child>> {
        let template = self
            .vir
            .data_closed()
            .entries()
            .iter()
            .find(|e| e["instance_id"] == id)
            .and_then(|e| e["template_id"].as_str())
            .ok_or(OrdinaryCarrierError::Linkage)?
            .to_owned();
        let names = arm_names(&template).ok_or(OrdinaryCarrierError::Shape)?;
        let carrier = (*self.carriers.get(id).ok_or(OrdinaryCarrierError::Shape)?).clone();
        let OrdinaryShape::Sum { arms } = &carrier.shape else {
            return Err(OrdinaryCarrierError::Shape);
        };
        if arms.iter().map(|a| a.id.as_str()).ne(names.iter().copied()) {
            return Err(OrdinaryCarrierError::Linkage);
        }
        let mut payloads = vec![];
        for (ordinal, arm) in arms.iter().enumerate() {
            let required = !matches!(arm.id.as_str(), "none" | "missing_key" | "missing" | "null");
            if arm.tag != ordinal as u32 || arm.fields.len() != usize::from(required) {
                return Err(OrdinaryCarrierError::Shape);
            }
            let mut payload = None;
            if required {
                let validation =
                    template == "mpk.csharp.semantic.validation.v1" && arm.id == "invalid";
                let (shape, maximum) = match &arm.fields[0].shape {
                    OrdinaryShape::RoleBound {
                        maximum: 256,
                        value,
                    } if validation => (value.as_ref(), Some(256)),
                    OrdinaryShape::Reference { .. } if !validation => (&arm.fields[0].shape, None),
                    _ => return Err(OrdinaryCarrierError::Shape),
                };
                let OrdinaryShape::Reference { type_id } = shape else {
                    return Err(OrdinaryCarrierError::Shape);
                };
                let Some(child) = self.ty(type_id)? else {
                    self.active.remove(id);
                    self.deferred.insert(id.into());
                    return Ok(None);
                };
                if validation && !self.sequences.contains_key(type_id) {
                    return Err(OrdinaryCarrierError::Shape);
                }
                payload = Some((type_id.clone(), child, maximum));
            }
            payloads.push(payload);
        }
        let storage = super::super::super::structural::emit(self.b, &carrier, &self.carriers)?
            .ok_or(OrdinaryCarrierError::Shape)?;
        let OrdinaryStructuralOperations::Sum {
            arms: constructors, ..
        } = storage.operations
        else {
            return Err(OrdinaryCarrierError::Shape);
        };
        let field = |name: &str| -> R<String> {
            self.syntax
                .fields()
                .iter()
                .find(|f| f.owner == id && f.group == "semantic_field_names" && f.field_id == name)
                .map(|f| f.match_definition.clone())
                .ok_or(OrdinaryCarrierError::Linkage)
        };
        let tag_field = field("tag")?;
        let payload_field = field("payload")?;
        let base = format!(
            "Mpk.CSharp.Ordinary.JsonSums.T{}",
            id.as_bytes()
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
        );
        let packet_depth = carrier.depth.max(7) + 1;
        let mut definitions = vec![];
        for ((arm, payload), constructor) in arms.iter().zip(payloads).zip(constructors) {
            if constructor.arm_id != arm.id || constructor.tag != arm.tag {
                return Err(OrdinaryCarrierError::Linkage);
            }
            let bytes = format!("\"{}\"", arm.id).into_bytes();
            let tag_literal = if let Some(l) = self.arm_literals.get(&bytes) {
                l.clone()
            } else {
                let l = super::super::json_syntax::emit_literal(
                    self.b,
                    self.fragments,
                    self.syntax_helpers.0,
                    self.syntax_helpers.1,
                    bytes.clone(),
                    self.chunks,
                )?;
                self.arm_literals.insert(bytes, l.clone());
                l
            };
            let mut scope = Scope::default();
            let doc = scope.outer(self.b, 3)?;
            let start = scope.outer(self.b, 2)?;
            let depth = scope.outer(self.b, 0)?;
            let length = call(self.b, &self.document.length_definition, vec![doc])?;
            // Even an arm without a payload has the tag string as a JSON child.
            let children = truth(self.b, true)?;
            let begin = call(
                self.b,
                &self.grammar.begin_definition,
                vec![length, start, depth, children],
            )?;
            let mut head = scope.bind(self.b, 7, begin)?;
            for matcher in [
                self.punctuation(b'{')?,
                tag_field.clone(),
                tag_literal.match_definition.clone(),
            ] {
                head = syntax_step(self.b, &mut scope, head, &matcher, self.grammar)?;
            }
            let mut payload_value = None;
            if let Some((type_id, child, maximum)) = &payload {
                for matcher in [self.punctuation(b',')?, payload_field.clone()] {
                    head = syntax_step(self.b, &mut scope, head, &matcher, self.grammar)?;
                }
                let doc = scope.outer(self.b, 3)?;
                let h = scope.bound(self.b, head)?;
                let cursor = call(self.b, &self.grammar.cursor_definition, vec![h])?;
                let ending = word(self.b, 3, 3)?;
                let mut args = vec![doc, cursor, ending];
                if child.compound {
                    let depth = scope.outer(self.b, 0)?;
                    args.push(call(
                        self.b,
                        &self.grammar.child_depth_definition,
                        vec![depth],
                    )?);
                }
                let parsed = call(self.b, &child.parse, args)?;
                let parsed = scope.bind(self.b, child.packet_depth, parsed)?;
                let h = scope.bound(self.b, head)?;
                let packet = scope.bound(self.b, parsed)?;
                let child_head = call(self.b, &child.header, vec![packet])?;
                let joined = call(self.b, &self.grammar.child_definition, vec![h, child_head])?;
                head = scope.bind(self.b, 7, joined)?;
                let packet = scope.bound(self.b, parsed)?;
                let value = call(self.b, &child.value, vec![packet])?;
                let depth = self
                    .carriers
                    .get(type_id.as_str())
                    .ok_or(OrdinaryCarrierError::Shape)?
                    .depth;
                let value = scope.bind(self.b, depth, value)?;
                payload_value = Some(value);
                if maximum.is_some() {
                    let value = scope.bound(self.b, value)?;
                    let good = validation_payload(self.b, value, depth)?;
                    let h = scope.bound(self.b, head)?;
                    let no = zero(self.b, 7)?;
                    let h = mux(self.b, 7, good, h, no)?;
                    head = scope.bind(self.b, 7, h)?;
                }
            }
            head = syntax_step(
                self.b,
                &mut scope,
                head,
                &self.punctuation(b'}')?,
                self.grammar,
            )?;
            let doc = scope.outer(self.b, 3)?;
            let h = scope.bound(self.b, head)?;
            let ending = scope.outer(self.b, 1)?;
            let h = call(
                self.b,
                &self.grammar.finish_definition,
                vec![doc, h, ending],
            )?;
            let args = payload_value
                .map(|v| scope.bound(self.b, v))
                .transpose()?
                .into_iter()
                .collect();
            let value = call(self.b, &constructor.make_definition, args)?;
            let assemble = super::super::json_grammar::assemble(self.b, carrier.depth)?;
            let body = call(self.b, &assemble, vec![h, value])?;
            let body = scope.finish(self.b, body)?;
            let parse = format!("{base}.A{}.Parse", arm.tag);
            define(self.b, &parse, &[24, 5, 3, 5], packet_depth, body)?;
            definitions.push(OrdinaryJsonSumArm {
                arm_id: arm.id.clone(),
                tag: arm.tag,
                tag_literal,
                parse_definition: parse,
                payload_type_id: payload.as_ref().map(|p| p.0.clone()),
                payload_parse_definition: payload.as_ref().map(|p| p.1.parse.clone()),
                payload_maximum: payload.and_then(|p| p.2),
            });
        }
        let mut scope = Scope::default();
        let initial = zero(self.b, packet_depth)?;
        let mut result = scope.bind(self.b, packet_depth, initial)?;
        for arm in &definitions {
            let args = (0..4)
                .rev()
                .map(|i| scope.outer(self.b, i))
                .collect::<R<Vec<_>>>()?;
            let candidate = call(self.b, &arm.parse_definition, args)?;
            let candidate = scope.bind(self.b, packet_depth, candidate)?;
            let candidate = scope.bound(self.b, candidate)?;
            let good = core_read(self.b, candidate, 0, packet_depth)?;
            let previous = scope.bound(self.b, result)?;
            let selected = mux(self.b, packet_depth, good, candidate, previous)?;
            result = scope.bind(self.b, packet_depth, selected)?;
        }
        let body = scope.bound(self.b, result)?;
        let body = scope.finish(self.b, body)?;
        let parse = format!("{base}.Parse");
        define(self.b, &parse, &[24, 5, 3, 5], packet_depth, body)?;
        let header = super::super::json_values::projection(self.b, packet_depth, 7, false)?;
        let value =
            super::super::json_values::projection(self.b, packet_depth, carrier.depth, true)?;
        let node = Child {
            parse: parse.clone(),
            header: header.clone(),
            value: value.clone(),
            packet_depth,
            compound: true,
        };
        self.sums.insert(
            id.into(),
            OrdinaryJsonSumDefinition {
                carrier,
                template_id: template,
                arms: definitions,
                parse_definition: parse,
                packet_depth,
                header_definition: header,
                value_definition: value,
            },
        );
        self.nodes.insert(id.into(), node.clone());
        self.active.remove(id);
        Ok(Some(node))
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::super::test_eval::{bit, run, sparse_cube};
    use super::*;

    #[test]
    fn json_validation_payload_checks_full_sequence_length_role() {
        let mut b = Builder::new().unwrap();
        for depth in [13, 18, 32] {
            let value = b.var(0).unwrap();
            let body = validation_payload(&mut b, value, depth).unwrap();
            define(
                &mut b,
                &format!("Test.Validation.D{depth}"),
                &[depth],
                0,
                body,
            )
            .unwrap();
        }
        let cert = decode_canonical_certificate(&b.finish().unwrap()).unwrap();
        crate::csharp_practical_vc_model::validate_csharp_practical_certificate_structure(&cert)
            .unwrap();
        for depth in [13, 18, 32] {
            for n in [
                0_u32,
                1,
                255,
                256,
                257,
                4096,
                4097,
                1 << 16,
                1 << 31,
                u32::MAX,
            ] {
                let ones = (0..32)
                    .filter(|i| n & (1 << i) != 0)
                    .map(|i| (i as usize) << (depth - 5))
                    .collect();
                let result = run(
                    &cert,
                    &format!("Test.Validation.D{depth}"),
                    vec![sparse_cube(depth, ones)],
                );
                assert_eq!(bit(result), (1..=256).contains(&n), "C{depth} length {n}");
            }
        }
    }
}
