//! Per-field boundary decoding. Envelope admission and source reconstruction
//! remain separate obligations; these definitions never assert AcceptInput.
use super::sequences::{mux, zero};
use super::*;
use crate::csharp_practical_source_artifacts::PracticalJsonValue as J;
use sha2::{Digest, Sha256};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryJsonBoundaryFieldDefinition {
    pub contract_sha256: String,
    pub field_id: String,
    pub source_type_id: String,
    pub semantic_carrier: OrdinaryCarrier,
    pub payload_type_id: String,
    pub name_match_definition: String,
    /// C24 document, C5 start, C3 ending, C5 raw JSON depth -> typed packet.
    /// Null becomes none/null; a supplied payload becomes some/value where
    /// required by the validated contract. Invalid results are entirely zero.
    pub parse_definition: String,
    pub packet_depth: u32,
    pub header_definition: String,
    pub value_definition: String,
    /// None means omission is rejected, including required fields.
    pub missing_value_definition: Option<String>,
    pub missing_cells: Option<u64>,
    pub nullable: bool,
    pub payload_parse_definition: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryJsonBoundaryFieldProgram {
    schema: String,
    parsers: OrdinaryJsonProductProgram,
    fields: Vec<OrdinaryJsonBoundaryFieldDefinition>,
    values: Vec<OrdinaryLiteralDefinition>,
}
impl OrdinaryJsonBoundaryFieldProgram {
    pub fn fields(&self) -> &[OrdinaryJsonBoundaryFieldDefinition] {
        &self.fields
    }
    pub fn values(&self) -> &[OrdinaryLiteralDefinition] {
        &self.values
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        self.parsers.certificate_bytes()
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("ordinary boundary field decoders")
    }
}
pub fn generate_csharp_practical_ordinary_json_boundary_fields(
    emitted: &EmittedDataPhase,
) -> R<OrdinaryJsonBoundaryFieldProgram> {
    let (parsers, fields, values, _, _, _, _) =
        generate_with_fields(emitted.vir(), Some(emitted), EnvelopeMode::None)?;
    let p = OrdinaryJsonBoundaryFieldProgram {
        schema: "mpk.csharp.ordinary_json_boundary_fields.v1".into(),
        parsers,
        fields,
        values,
    };
    if p.canonical_bytes().len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    Ok(p)
}
pub fn import_csharp_practical_ordinary_json_boundary_fields(
    input: &[u8],
    certificate: &[u8],
    emitted: &EmittedDataPhase,
) -> R<OrdinaryJsonBoundaryFieldProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let p = generate_csharp_practical_ordinary_json_boundary_fields(emitted)?;
    if input != p.canonical_bytes() || certificate != p.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(p)
}

pub(super) struct Plan {
    contract: String,
    field: String,
    source: String,
    semantic: String,
    payload: String,
    missing: Option<String>,
    missing_cells: Option<u64>,
    null: Option<String>,
    wrapper: Option<&'static str>,
    codec: Option<(String, Option<u8>, Option<String>, String)>,
}

pub(super) fn codec_carriers(
    plans: &[Plan],
    layouts: &OrdinaryCarrierProgram,
) -> R<Vec<OrdinaryCarrier>> {
    let mut extra = BTreeMap::new();
    for plan in plans {
        let Some((id, _, _, ty)) = &plan.codec else {
            continue;
        };
        if layouts.carriers().iter().any(|c| &c.type_id == ty) {
            continue;
        }
        // A validated raw-instant field is source Int64 but its selected codec
        // is the registered Instant codec. Its parser may need an adapter even
        // when no source/business binding otherwise makes Instant reachable.
        if id != "unix_milliseconds"
            || ty != "mpk.csharp.value.instant.v1"
            || plan.source != "mpk.csharp.value.i64.v1"
            || plan.payload != plan.source
            || plan.wrapper.is_some()
        {
            return Err(OrdinaryCarrierError::Linkage);
        }
        extra.insert(
            ty.clone(),
            OrdinaryCarrier {
                type_id: ty.clone(),
                depth: 6,
                shape: OrdinaryShape::Bits { width: 64 },
            },
        );
    }
    Ok(extra.into_values().collect())
}
fn literal_name(v: &MonomorphicValue) -> String {
    format!(
        "{PREFIX}.BoundaryFieldValue.H{:x}",
        Sha256::digest(serde_json::to_vec(v).unwrap())
    )
}
pub(super) fn prepare(
    emitted: &EmittedDataPhase,
    layouts: &OrdinaryCarrierProgram,
    b: Builder,
) -> R<(Builder, Vec<Plan>, Vec<OrdinaryLiteralDefinition>)> {
    let vir = emitted.vir();
    let boundary = generate_boundary_vcs(vir).map_err(|_| OrdinaryCarrierError::Linkage)?;
    let mut plans = vec![];
    let mut values = BTreeMap::new();
    for contract in emitted.boundaries() {
        let artifact = contract.artifact();
        let doc = std::str::from_utf8(artifact.canonical_bytes())
            .map_err(|_| OrdinaryCarrierError::Linkage)?;
        if !boundary.contracts().iter().any(|c| c == doc) {
            return Err(OrdinaryCarrierError::Linkage);
        }
        let id = artifact
            .value()
            .get("contract_sha256")
            .and_then(J::as_str)
            .ok_or(OrdinaryCarrierError::Linkage)?;
        for field in contract.input_fields() {
            let semantic = emitted
                .closure()
                .projections()
                .get(field.source_type_id())
                .map(String::as_str)
                .unwrap_or(field.source_type_id());
            let wrapper = if field.presence_binding().is_some() {
                Some("value")
            } else if vir
                .data_closed()
                .metadata
                .get(semantic)
                .is_some_and(|m| m.template_id == "mpk.csharp.semantic.option.v1")
            {
                Some("some")
            } else {
                None
            };
            let missing = if field.required() {
                None
            } else {
                match field.missing_rule() {
                    BoundaryMissingRule::Reject => None,
                    BoundaryMissingRule::FrozenDefault(v) => Some(v.clone()),
                    BoundaryMissingRule::ExposeMissing => {
                        Some(MonomorphicValue::BoundaryPresence {
                            type_id: semantic.into(),
                            arm: BoundaryArm::Missing,
                            value: None,
                        })
                    }
                }
            };
            let null = if !field.nullable() {
                None
            } else {
                Some(match wrapper {
                    Some("value") => MonomorphicValue::BoundaryPresence {
                        type_id: semantic.into(),
                        arm: BoundaryArm::Null,
                        value: None,
                    },
                    Some("some") => MonomorphicValue::Option {
                        type_id: semantic.into(),
                        arm: OptionArm::None,
                        value: None,
                    },
                    _ => return Err(OrdinaryCarrierError::Shape),
                })
            };
            let (bundle, roots, _) = vir.construction_context();
            let missing_cells = missing
                .as_ref()
                .map(|v| {
                    validate_value_inner(bundle, roots, vir.data_closed(), v, false)
                        .map_err(|_| OrdinaryCarrierError::Shape)
                })
                .transpose()?;
            let mut intern = |value: Option<MonomorphicValue>| -> R<Option<String>> {
                value
                    .map(|v| {
                        if v.type_id() != semantic {
                            return Err(OrdinaryCarrierError::Shape);
                        }
                        let name = literal_name(&v);
                        values.entry(name.clone()).or_insert(v);
                        Ok(name)
                    })
                    .transpose()
            };
            let missing = intern(missing)?;
            let null = intern(null)?;
            let codec = field.codec().map(|c| {
                let (id, scale, rounding) = c.ordinary_configuration();
                (
                    id.into(),
                    scale,
                    rounding.map(|r| format!("{r:?}")),
                    c.type_id().into(),
                )
            });
            plans.push(Plan {
                contract: id.into(),
                field: field.id().into(),
                source: field.source_type_id().into(),
                semantic: semantic.into(),
                payload: field.payload_type_id().into(),
                missing,
                missing_cells,
                null,
                wrapper,
                codec,
            });
        }
    }
    let (b, values) =
        super::super::super::structural::emit_literal_values(vir, layouts, b, values)?;
    Ok((b, plans, values))
}

/// Preserve complete headers and add only the semantic some/value wrapper.
/// The wrapper consumes no extra raw JSON depth in the supplied document.
fn adjust_header(b: &mut Builder, wrapper: bool) -> R<String> {
    let name = format!("{PREFIX}.JsonBoundaryFields.Header.W{}", u8::from(wrapper));
    if b.globals.contains_key(&name) {
        return Ok(name);
    }
    let mut c = Circuit::new(&[128, 32]);
    let head = c.inputs[0].clone();
    let depth = c.inputs[1].clone();
    let mut good = super::super::json_grammar::header_valid(&mut c, &head);
    let depth_ok = c.lt(&depth, &super::super::temporal::literal(33, 32), false);
    good = c.and(good, depth_ok);
    let (cells, overflow) = c.add(
        &head[34..66],
        &super::super::temporal::literal(u128::from(wrapper), 32),
        F,
    );
    let bounded = c.lt(
        &cells,
        &super::super::temporal::literal(u128::from(TOTAL_VALUE_CELLS_MAX) + 1, 32),
        false,
    );
    good = c.and(good, bounded);
    let no_overflow = c.not(overflow);
    good = c.and(good, no_overflow);
    let mut out = head;
    out[34..66].copy_from_slice(&cells);
    let out = c.select(good, &out, &vec![F; 128]);
    let helper = super::super::integer_format::circuit_with_block_bits(b, &name, c, out, 7)?;
    let head = b.var(1)?;
    let depth = b.var(0)?;
    let body = call(b, &helper, vec![head, depth])?;
    define(b, &name, &[7, 5], 7, body)?;
    Ok(name)
}
impl Products<'_> {
    pub(super) fn boundary_field(
        &mut self,
        plan: Plan,
        primitives: &[OrdinaryJsonValueDefinition],
    ) -> R<OrdinaryJsonBoundaryFieldDefinition> {
        let carrier = (*self
            .carriers
            .get(plan.semantic.as_str())
            .ok_or(OrdinaryCarrierError::Shape)?)
        .clone();
        let payload = (*self
            .carriers
            .get(plan.payload.as_str())
            .ok_or(OrdinaryCarrierError::Shape)?)
        .clone();
        let child = if let Some((id, scale, rounding, ty)) = &plan.codec {
            let candidates = primitives
                .iter()
                .filter(|d| {
                    &d.codec_id == id
                        && &d.scale == scale
                        && &d.rounding == rounding
                        && &d.carrier.type_id == ty
                })
                .collect::<Vec<_>>();
            let [d] = candidates.as_slice() else {
                return Err(OrdinaryCarrierError::Linkage);
            };
            // A raw Unix-millisecond Int64 has the same 64-bit layout as Instant.
            if d.carrier.type_id != plan.payload
                && !(id == "unix_milliseconds"
                    && plan.payload == "mpk.csharp.value.i64.v1"
                    && ty == "mpk.csharp.value.instant.v1")
            {
                return Err(OrdinaryCarrierError::Shape);
            }
            if d.carrier.depth != payload.depth {
                return Err(OrdinaryCarrierError::Shape);
            }
            Child {
                parse: d.parse_definition.clone(),
                header: d.header_definition.clone(),
                value: d.value_definition.clone(),
                packet_depth: d.packet_depth,
                compound: false,
            }
        } else {
            self.ty(&plan.payload)?.ok_or(OrdinaryCarrierError::Shape)?
        };
        self.ty(&plan.semantic)?
            .ok_or(OrdinaryCarrierError::Shape)?;
        let make = if let Some(arm_id) = plan.wrapper {
            let OrdinaryShape::Sum { arms } = &carrier.shape else {
                return Err(OrdinaryCarrierError::Shape);
            };
            let arm = arms
                .iter()
                .find(|a| a.id == arm_id)
                .ok_or(OrdinaryCarrierError::Shape)?;
            if arm.fields.len() != 1
                || arm.fields[0].shape
                    != (OrdinaryShape::Reference {
                        type_id: plan.payload.clone(),
                    })
            {
                return Err(OrdinaryCarrierError::Shape);
            }
            let hex = plan
                .semantic
                .bytes()
                .map(|v| format!("{v:02x}"))
                .collect::<String>();
            Some(format!(
                "{PREFIX}.Structural.T{hex}.Arm.A{}.MakeStorage",
                arm.tag
            ))
        } else {
            if plan.semantic != plan.payload {
                return Err(OrdinaryCarrierError::Shape);
            }
            None
        };
        let name_match_definition = self
            .syntax
            .fields()
            .iter()
            .find(|f| {
                f.owner == plan.contract && f.group == "input_fields" && f.field_id == plan.field
            })
            .map(|f| f.match_definition.clone())
            .ok_or(OrdinaryCarrierError::Linkage)?;
        let adjust = adjust_header(self.b, make.is_some())?;
        let assemble = super::super::json_grammar::assemble(self.b, carrier.depth)?;
        let packet_depth = carrier.depth.max(7) + 1;
        let mut scope = Scope::default();
        let doc = scope.outer(self.b, 3)?;
        let start = scope.outer(self.b, 2)?;
        let ending = scope.outer(self.b, 1)?;
        let mut args = vec![doc, start, ending];
        if child.compound {
            args.push(scope.outer(self.b, 0)?);
        }
        let parsed = call(self.b, &child.parse, args)?;
        let parsed = scope.bind(self.b, child.packet_depth, parsed)?;
        let raw = scope.bound(self.b, parsed)?;
        let head = call(self.b, &child.header, vec![raw])?;
        let depth = scope.outer(self.b, 0)?;
        let head = call(self.b, &adjust, vec![head, depth])?;
        let mut value = call(self.b, &child.value, vec![raw])?;
        if let Some(make) = make {
            value = call(self.b, &make, vec![value])?;
        }
        let supplied = call(self.b, &assemble, vec![head, value])?;
        let supplied = scope.bind(self.b, packet_depth, supplied)?;
        let mut body = scope.bound(self.b, supplied)?;
        {
            let literal = if let Some(d) = self.arm_literals.get(b"null".as_slice()) {
                d.clone()
            } else {
                let d = super::super::json_syntax::emit_literal(
                    self.b,
                    self.fragments,
                    self.syntax_helpers.0,
                    self.syntax_helpers.1,
                    b"null".to_vec(),
                    self.chunks,
                )?;
                self.arm_literals.insert(b"null".to_vec(), d.clone());
                d
            };
            let doc = scope.outer(self.b, 3)?;
            let start = scope.outer(self.b, 2)?;
            let depth = scope.outer(self.b, 0)?;
            let length = call(self.b, &self.document.length_definition, vec![doc])?;
            let children = truth(self.b, false)?;
            let begin = call(
                self.b,
                &self.grammar.begin_definition,
                vec![length, start, depth, children],
            )?;
            let matched = call(self.b, &literal.match_definition, vec![doc, start])?;
            let head = call(
                self.b,
                &self.grammar.syntax_definition,
                vec![begin, matched],
            )?;
            let ending = scope.outer(self.b, 1)?;
            let head = call(
                self.b,
                &self.grammar.finish_definition,
                vec![doc, head, ending],
            )?;
            let valid = core_read(self.b, head, 0, 7)?;
            let packet = if let Some(null) = &plan.null {
                let value = self.b.constant(null)?;
                call(self.b, &assemble, vec![head, value])?
            } else {
                // JSON null is always the boundary null state, even if the
                // payload's ordinary representation (e.g. Unit) accepts null.
                zero(self.b, packet_depth)?
            };
            body = mux(self.b, packet_depth, valid, packet, body)?;
        }
        let body = scope.finish(self.b, body)?;
        let identity = format!("{}:{}", plan.contract, plan.field);
        let parse_definition = format!(
            "{PREFIX}.JsonBoundaryFields.H{:x}.Parse",
            Sha256::digest(identity.as_bytes())
        );
        define(
            self.b,
            &parse_definition,
            &[24, 5, 3, 5],
            packet_depth,
            body,
        )?;
        let header_definition =
            super::super::json_values::projection(self.b, packet_depth, 7, false)?;
        let value_definition =
            super::super::json_values::projection(self.b, packet_depth, carrier.depth, true)?;
        Ok(OrdinaryJsonBoundaryFieldDefinition {
            contract_sha256: plan.contract,
            field_id: plan.field,
            source_type_id: plan.source,
            semantic_carrier: carrier,
            payload_type_id: plan.payload,
            name_match_definition,
            parse_definition,
            packet_depth,
            header_definition,
            value_definition,
            missing_value_definition: plan.missing,
            missing_cells: plan.missing_cells,
            nullable: plan.null.is_some(),
            payload_parse_definition: child.parse,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::super::test_eval::{apply, bit, run, V};
    use super::*;
    fn word(n: u32) -> V {
        V::Cube((0..32).map(|i| n & (1 << i) != 0).collect())
    }
    #[test]
    fn boundary_field_semantic_wrapper_cell_and_raw_depth_limits() {
        let mut b = Builder::new().unwrap();
        let plain = adjust_header(&mut b, false).unwrap();
        let wrapped = adjust_header(&mut b, true).unwrap();
        let bytes = b.finish().unwrap();
        let cert = decode_canonical_certificate(&bytes).unwrap();
        crate::csharp_practical_vc_model::validate_csharp_practical_certificate_structure(&cert)
            .unwrap();
        let mut count = 0;
        for (name, extra) in [(plain, 0u32), (wrapped, 1)] {
            for n in [0, 1, 2, 16384, 16385, 65535, 65536, 65537, u32::MAX] {
                for depth in [0, 1, 31, 32, 33, u32::MAX] {
                    for invalid in [None, Some(0), Some(66), Some(127)] {
                        let mut input = vec![false; 128];
                        input[0] = true;
                        input[1] = true;
                        for i in 0..32 {
                            input[2 + i] = 7 & (1 << i) != 0;
                            input[34 + i] = n & (1 << i) != 0;
                        }
                        if let Some(i) = invalid {
                            input[i] = !input[i];
                        }
                        let actual = run(&cert, &name, vec![V::Cube(input), word(depth)]);
                        let mut expected = vec![false; 128];
                        if invalid.is_none()
                            && depth <= 32
                            && n > 0
                            && u64::from(n) + u64::from(extra) <= 65536
                        {
                            expected[0] = true;
                            expected[1] = true;
                            for i in 0..32 {
                                expected[2 + i] = 7 & (1 << i) != 0;
                                expected[34 + i] = (n + extra) & (1 << i) != 0;
                            }
                        }
                        for (index, expected) in expected.into_iter().enumerate() {
                            let mut leaf = actual.clone();
                            for i in 0..7 {
                                leaf = apply(&cert, leaf, V::Bit(index & (1 << i) != 0));
                            }
                            assert_eq!(
                                bit(leaf),
                                expected,
                                "{name}:cells{n},depth{depth},invalid{invalid:?},bit{index}"
                            );
                        }
                        count += 1;
                    }
                }
            }
        }
        eprintln!("Boundary field headers: {count} complete packets");
    }
}
