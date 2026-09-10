//! Ordinary forward semantic bindings. Inverses and source invariants need proofs.
use super::*;
use sha2::{Digest, Sha256};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryBindingProjectionDefinition {
    pub projection: BindingTypeProjection,
    pub source_carrier: OrdinaryCarrier,
    pub semantic_carrier: OrdinaryCarrier,
    pub project_definition: String,
    /// Only identity has an unconditional inverse here. Source reconstruction
    /// must account for every stored member, including inactive/unmapped ones.
    pub reconstruct_definition: Option<String>,
    pub reconstruction_member_ids: Vec<String>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryBindingProjectionProgram {
    schema: String,
    source_ir_sha256: String,
    foundation_sha256: String,
    binding_vc_sha256: String,
    definitions: Vec<OrdinaryBindingProjectionDefinition>,
    certificate_sha256: String,
    #[serde(skip)]
    certificate: Vec<u8>,
}
impl OrdinaryBindingProjectionProgram {
    pub fn definitions(&self) -> &[OrdinaryBindingProjectionDefinition] {
        &self.definitions
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("typed binding projections")
    }
}
fn named(kind: &str, value: &impl Serialize) -> String {
    format!(
        "{PREFIX}.BindingProjection.{kind}.H{:x}",
        Sha256::digest(serde_json::to_vec(value).expect("typed projection part"))
    )
}
pub(super) fn conversion_name(from: &str, to: &str) -> String {
    named("Convert", &(from, to))
}
fn reference_type(shape: &OrdinaryShape) -> R<&str> {
    match shape {
        OrdinaryShape::Reference { type_id } => Ok(type_id),
        OrdinaryShape::RoleBound { value, .. } => reference_type(value),
        _ => Err(OrdinaryCarrierError::Shape),
    }
}
struct Projections<'a> {
    b: Builder,
    closed: &'a ClosedInstanceSet,
    carriers: BTreeMap<&'a str, &'a OrdinaryCarrier>,
    bindings: BTreeMap<&'a str, &'a BindingRepresentationVc>,
    nodes: BTreeMap<(String, String), String>,
    active: BTreeSet<(String, String)>,
}
impl Projections<'_> {
    fn carrier(&self, id: &str) -> R<OrdinaryCarrier> {
        self.carriers
            .get(id)
            .map(|c| (*c).clone())
            .ok_or(OrdinaryCarrierError::Linkage)
    }
    fn packing(&self, fields: &[OrdinaryField]) -> R<(Vec<u32>, u32)> {
        let ds = fields
            .iter()
            .map(|f| shape_depth(&f.shape, &self.carriers))
            .collect::<R<Vec<_>>>()?;
        let roles =
            address_bits(u32::try_from(fields.len()).map_err(|_| OrdinaryCarrierError::Limit)?);
        Ok((ds.clone(), roles + ds.into_iter().max().unwrap_or(0)))
    }
    fn getter(&mut self, source: &OrdinaryCarrier, depth: u32, address: Vec<bool>) -> R<String> {
        let name = named("Get", &(source.type_id.as_str(), depth, &address));
        if !self.b.globals.contains_key(&name) {
            project(&mut self.b, &name, source.depth, depth, &address, None)?;
        }
        Ok(name)
    }
    fn field_getter(
        &mut self,
        source: &OrdinaryCarrier,
        fields: &[OrdinaryField],
        i: usize,
        mut leading: Vec<bool>,
    ) -> R<(String, String)> {
        let (ds, _) = self.packing(fields)?;
        let depth = *ds.get(i).ok_or(OrdinaryCarrierError::Shape)?;
        leading.extend(prefix(address_bits(fields.len() as u32), i as u32));
        leading.extend(vec![
            false;
            (ds.iter().copied().max().unwrap_or(0) - depth) as usize
        ]);
        Ok((
            self.getter(source, depth, leading)?,
            reference_type(&fields[i].shape)?.into(),
        ))
    }
    fn member(
        &mut self,
        source: &OrdinaryCarrier,
        binding: &Value,
        role: &str,
    ) -> R<(String, String)> {
        let id = text(&binding["member_map"], role)?;
        let OrdinaryShape::Product { fields } = &source.shape else {
            return Err(OrdinaryCarrierError::Shape);
        };
        let index = fields
            .iter()
            .position(|f| f.id == id)
            .ok_or(OrdinaryCarrierError::Linkage)?;
        self.field_getter(source, fields, index, vec![])
    }
    fn transformed(&mut self, source: u32, getter: &str, from: &str, to: &str) -> R<u32> {
        let convert = self.convert(from, to)?;
        let child = call(&mut self.b, getter, vec![source])?;
        call(&mut self.b, &convert, vec![child])
    }
    fn packed_leaf(
        &mut self,
        outer: u32,
        start: u32,
        fields: &[OrdinaryField],
        children: &[u32],
    ) -> R<u32> {
        if fields.len() != children.len() {
            return Err(OrdinaryCarrierError::Shape);
        }
        let (ds, depth) = self.packing(fields)?;
        if start + depth != outer {
            return Err(OrdinaryCarrierError::Shape);
        }
        let roles = address_bits(fields.len() as u32);
        let max = ds.iter().copied().max().unwrap_or(0);
        let mut body = bit(&mut self.b, false)?;
        for (i, (&d, &child)) in ds.iter().zip(children).enumerate().rev() {
            let selectors = self.b.selectors(d)?;
            let value = self.b.app(child, selectors)?;
            let value = zero_padding(&mut self.b, outer, start + roles, max - d, value)?;
            let at = equal_address(&mut self.b, outer, start, roles, i as u32)?;
            body = mux(&mut self.b, at, value, body)?;
        }
        Ok(body)
    }
    fn arm_leaf(
        &mut self,
        target: &OrdinaryCarrier,
        arm: &OrdinaryArm,
        children: &[u32],
    ) -> R<u32> {
        let outer = target.depth;
        let payload = outer - 1;
        let (_, arm_depth) = self.packing(&arm.fields)?;
        let value = self.packed_leaf(outer, 1 + payload - arm_depth, &arm.fields, children)?;
        let value = zero_padding(&mut self.b, outer, 1, payload - arm_depth, value)?;
        let mut tag = bit(&mut self.b, false)?;
        for i in 0..32 {
            if arm.tag & (1 << i) != 0 {
                let at = equal_address(&mut self.b, outer, outer - 5, 5, i)?;
                tag = call(&mut self.b, "Std.Bool.or", vec![tag, at])?;
            }
        }
        let tag = zero_padding(&mut self.b, outer, 1, payload - 5, tag)?;
        let role = self.b.var(outer - 1)?;
        mux(&mut self.b, role, value, tag)
    }
    fn tag_test(
        &mut self,
        source: &OrdinaryCarrier,
        getter: &str,
        depth: u32,
        width: u32,
        n: u128,
    ) -> R<String> {
        if width > 128 || address_bits(width) != depth {
            return Err(OrdinaryCarrierError::Shape);
        }
        let name = named(
            "Tag",
            &(source.type_id.as_str(), getter, width, n.to_string()),
        );
        if !self.b.globals.contains_key(&name) {
            let x = self.b.var(0)?;
            let tag = call(&mut self.b, getter, vec![x])?;
            let mut active = bit(&mut self.b, true)?;
            for i in 0..width {
                let address = prefix(depth, i)
                    .into_iter()
                    .map(|v| bit(&mut self.b, v))
                    .collect::<R<Vec<_>>>()?;
                let mut value = self.b.app(tag, address)?;
                if n & (1u128 << i) == 0 {
                    value = call(&mut self.b, "Std.Bool.not", vec![value])?;
                }
                active = call(&mut self.b, "Std.Bool.and", vec![active, value])?;
            }
            define(&mut self.b, &name, &[source.depth], 0, active)?;
        }
        Ok(name)
    }
    fn index_word(&mut self, bits: u32, outer: u32, start: u32) -> R<u32> {
        if bits > 14 || start + bits > outer {
            return Err(OrdinaryCarrierError::Shape);
        }
        let name = named("Index", &bits);
        if !self.b.globals.contains_key(&name) {
            let mut body = bit(&mut self.b, false)?;
            for i in 0..bits {
                let at = equal_address(&mut self.b, 5, 0, 5, i)?;
                let v = self.b.var(5 + bits - 1 - i)?;
                body = mux(&mut self.b, at, v, body)?;
            }
            let body = self.b.wrap_selectors(5, body)?;
            define(&mut self.b, &name, &vec![0; bits as usize], 5, body)?;
        }
        let args = (0..bits)
            .map(|i| self.b.var(outer - 1 - start - i))
            .collect::<R<Vec<_>>>()?;
        call(&mut self.b, &name, args)
    }
    fn element_type(&self, shape: &OrdinaryShape) -> R<String> {
        match shape {
            OrdinaryShape::Reference { type_id } => Ok(type_id.clone()),
            OrdinaryShape::RoleBound { value, .. } => self.element_type(value),
            OrdinaryShape::Product { fields }
                if fields.len() == 2 && fields[0].id == "key" && fields[1].id == "value" =>
            {
                let args = fields
                    .iter()
                    .map(|f| reference_type(&f.shape).map(str::to_owned))
                    .collect::<R<Vec<_>>>()?;
                let matches = self
                    .closed
                    .entries()
                    .iter()
                    .filter_map(|e| {
                        let id = e["instance_id"].as_str()?;
                        (e["template_id"] == "mpk.csharp.semantic.ordered_entry.v1"
                            && self.closed.metadata[id].argument_ids == args)
                            .then_some(id)
                    })
                    .collect::<Vec<_>>();
                if matches.len() != 1 || self.carrier(matches[0])?.shape != *shape {
                    return Err(OrdinaryCarrierError::Linkage);
                }
                Ok(matches[0].to_owned())
            }
            _ => Err(OrdinaryCarrierError::Shape),
        }
    }
    fn sequence_body(&mut self, source: &OrdinaryCarrier, target: &OrdinaryCarrier) -> R<u32> {
        let OrdinaryShape::Sequence {
            capacity: sc,
            element: se,
        } = &source.shape
        else {
            return Err(OrdinaryCarrierError::Shape);
        };
        let OrdinaryShape::Sequence {
            capacity: tc,
            element: te,
        } = &target.shape
        else {
            return Err(OrdinaryCarrierError::Shape);
        };
        if sc != tc {
            return Err(OrdinaryCarrierError::Shape);
        }
        let from = self.element_type(se)?;
        let to = self.element_type(te)?;
        let sd = shape_depth(se, &self.carriers)?;
        let td = shape_depth(te, &self.carriers)?;
        let bits = address_bits(*sc);
        let mut length_address = vec![false];
        length_address.extend(vec![false; (source.depth - 1 - 5) as usize]);
        let length = self.getter(source, 5, length_address)?;
        let read = named("Read", &source.type_id);
        if !self.b.globals.contains_key(&read) {
            let x = self.b.var(sd + 1)?;
            let index = self.b.var(sd)?;
            let mut address = vec![bit(&mut self.b, true)?];
            address.extend(
                (0..source.depth - 1 - bits - sd)
                    .map(|_| bit(&mut self.b, false))
                    .collect::<R<Vec<_>>>()?,
            );
            for i in 0..bits {
                address.push(ordered_fold::read_bit(&mut self.b, index, i)?);
            }
            address.extend(self.b.selectors(sd)?);
            let body = self.b.app(x, address)?;
            let body = self.b.wrap_selectors(sd, body)?;
            define(&mut self.b, &read, &[source.depth, 5], sd, body)?;
        }
        let convert = self.convert(&from, &to)?;
        let outer = target.depth;
        let data_depth = bits + td;
        let x = self.b.var(outer)?;
        let len = call(&mut self.b, &length, vec![x])?;
        let selectors = self.b.selectors(5)?;
        let header = self.b.app(len, selectors)?;
        let header = zero_padding(&mut self.b, outer, 1, outer - 1 - 5, header)?;
        let index = self.index_word(bits, outer, outer - data_depth)?;
        let active = ordered_fold::helper(&mut self.b, "Less", vec![index, len])?;
        let value = call(&mut self.b, &read, vec![x, index])?;
        let value = call(&mut self.b, &convert, vec![value])?;
        let selectors = self.b.selectors(td)?;
        let value = self.b.app(value, selectors)?;
        let zero = bit(&mut self.b, false)?;
        let value = mux(&mut self.b, active, value, zero)?;
        let value = zero_padding(&mut self.b, outer, 1, outer - 1 - data_depth, value)?;
        let role = self.b.var(outer - 1)?;
        let body = mux(&mut self.b, role, value, header)?;
        self.b.wrap_selectors(outer, body)
    }
    fn sum_body(
        &mut self,
        source: &OrdinaryCarrier,
        target: &OrdinaryCarrier,
        binding: Option<&Value>,
    ) -> R<u32> {
        let OrdinaryShape::Sum { arms } = &target.shape else {
            return Err(OrdinaryCarrierError::Shape);
        };
        let (tag_getter, tag_depth, tag_width) = if let Some(binding) = binding {
            let (get, ty) = self.member(source, binding, "tag")?;
            let carrier = self.carrier(&ty)?;
            let OrdinaryShape::Bits { width } = carrier.shape else {
                return Err(OrdinaryCarrierError::Shape);
            };
            (get, carrier.depth, width)
        } else {
            let OrdinaryShape::Sum { arms: source_arms } = &source.shape else {
                return Err(OrdinaryCarrierError::Shape);
            };
            if source_arms.len() != arms.len() {
                return Err(OrdinaryCarrierError::Shape);
            }
            let mut address = vec![false];
            address.extend(vec![false; (source.depth - 1 - 5) as usize]);
            (self.getter(source, 5, address)?, 5, 32)
        };
        let outer = target.depth;
        let x = self.b.var(outer)?;
        let mut body = bit(&mut self.b, false)?;
        for arm in arms.iter().rev() {
            let mut children = vec![];
            let source_tag;
            if let Some(binding) = binding {
                source_tag = text(&binding["tag_arms"], &arm.id)?
                    .parse::<i128>()
                    .map_err(|_| OrdinaryCarrierError::Shape)? as u128;
                if arm.fields.len() > 1 {
                    return Err(OrdinaryCarrierError::Shape);
                }
                if let Some(field) = arm.fields.first() {
                    let role = match (text(binding, "role")?, arm.id.as_str()) {
                        ("result", "error") => "error",
                        ("validation", "invalid") => "errors",
                        _ => "value",
                    };
                    let (get, from) = self.member(source, binding, role)?;
                    children.push(self.transformed(
                        x,
                        &get,
                        &from,
                        reference_type(&field.shape)?,
                    )?);
                }
            } else {
                let OrdinaryShape::Sum { arms: source_arms } = &source.shape else {
                    return Err(OrdinaryCarrierError::Shape);
                };
                let source_arm = source_arms
                    .iter()
                    .find(|a| a.id == arm.id && a.tag == arm.tag)
                    .ok_or(OrdinaryCarrierError::Shape)?;
                if source_arm.fields.len() != arm.fields.len() {
                    return Err(OrdinaryCarrierError::Shape);
                }
                source_tag = arm.tag as u128;
                let (_, source_payload_depth) = self.packing(&source_arm.fields)?;
                let mut leading = vec![true];
                leading.extend(vec![
                    false;
                    (source.depth - 1 - source_payload_depth) as usize
                ]);
                for (i, field) in arm.fields.iter().enumerate() {
                    let (get, from) =
                        self.field_getter(source, &source_arm.fields, i, leading.clone())?;
                    children.push(self.transformed(
                        x,
                        &get,
                        &from,
                        reference_type(&field.shape)?,
                    )?);
                }
            }
            let condition = self.tag_test(source, &tag_getter, tag_depth, tag_width, source_tag)?;
            let active = call(&mut self.b, &condition, vec![x])?;
            let value = self.arm_leaf(target, arm, &children)?;
            body = mux(&mut self.b, active, value, body)?;
        }
        self.b.wrap_selectors(outer, body)
    }
    fn bound_body(
        &mut self,
        source: &OrdinaryCarrier,
        target: &OrdinaryCarrier,
        binding: &Value,
    ) -> R<u32> {
        let role = text(binding, "role")?;
        match role {
            "instant" => {
                let (get, from) = self.member(source, binding, "milliseconds")?;
                if from != "mpk.csharp.value.i64.v1"
                    || target.type_id != "mpk.csharp.value.instant.v1"
                    || source.depth > 253
                {
                    return Err(OrdinaryCarrierError::Shape);
                }
                let x = self.b.var(0)?;
                call(&mut self.b, &get, vec![x])
            }
            "bounded_sequence" | "ordered_map" | "ordered_set" => {
                let (get, from) = self.member(
                    source,
                    binding,
                    if role == "ordered_map" {
                        "entries"
                    } else {
                        "elements"
                    },
                )?;
                let x = self.b.var(0)?;
                self.transformed(x, &get, &from, &target.type_id)
            }
            "money" | "ordered_entry" | "transition" => {
                let OrdinaryShape::Product { fields } = &target.shape else {
                    return Err(OrdinaryCarrierError::Shape);
                };
                let x = self.b.var(target.depth)?;
                let mut children = vec![];
                for field in fields {
                    let (get, from) = self.member(source, binding, &field.id)?;
                    children.push(self.transformed(
                        x,
                        &get,
                        &from,
                        reference_type(&field.shape)?,
                    )?);
                }
                let body = self.packed_leaf(target.depth, 0, fields, &children)?;
                self.b.wrap_selectors(target.depth, body)
            }
            "option" | "result" | "lookup" | "validation" | "boundary_field" => {
                self.sum_body(source, target, Some(binding))
            }
            _ => Err(OrdinaryCarrierError::Shape),
        }
    }
    fn convert(&mut self, from: &str, to: &str) -> R<String> {
        let key = (from.to_owned(), to.to_owned());
        if let Some(name) = self.nodes.get(&key) {
            return Ok(name.clone());
        }
        if !self.active.insert(key.clone()) {
            return Err(OrdinaryCarrierError::Cycle);
        }
        let source = self.carrier(from)?;
        let target = self.carrier(to)?;
        let name = conversion_name(from, to);
        let body = if from == to {
            self.b.var(0)?
        } else if let Some(binding) = self.bindings.get(from).copied() {
            if binding.projection.semantic_type_id != to {
                return Err(OrdinaryCarrierError::Linkage);
            }
            self.bound_body(&source, &target, &binding.binding)?
        } else {
            match (&source.shape, &target.shape) {
                (OrdinaryShape::Sequence { .. }, OrdinaryShape::Sequence { .. }) => {
                    self.sequence_body(&source, &target)?
                }
                (OrdinaryShape::Sum { .. }, OrdinaryShape::Sum { .. }) => {
                    self.sum_body(&source, &target, None)?
                }
                _ => return Err(OrdinaryCarrierError::Shape),
            }
        };
        define(&mut self.b, &name, &[source.depth], target.depth, body)?;
        self.nodes.insert(key.clone(), name.clone());
        self.active.remove(&key);
        Ok(name)
    }
}
pub(super) fn emit_binding_projections(
    vir: &ValidatedPracticalVir,
    vc: &BindingVcProgram,
    layouts: &OrdinaryCarrierProgram,
    b: Builder,
) -> R<(Builder, Vec<OrdinaryBindingProjectionDefinition>)> {
    let mut p = Projections {
        b,
        closed: vir.data_closed(),
        carriers: layouts
            .carriers()
            .iter()
            .map(|c| (c.type_id.as_str(), c))
            .collect(),
        bindings: vc
            .representations()
            .iter()
            .map(|r| (r.projection.source_type_id.as_str(), r))
            .collect(),
        nodes: BTreeMap::new(),
        active: BTreeSet::new(),
    };
    ordered_fold::auxiliary(&mut p.b)?;
    let mut definitions = vec![];
    for projection in vir.binding_projections() {
        let name = p.convert(&projection.source_type_id, &projection.semantic_type_id)?;
        let source = p.carrier(&projection.source_type_id)?;
        let target = p.carrier(&projection.semantic_type_id)?;
        let reconstruction_member_ids =
            if let Some(rep) = p.bindings.get(projection.source_type_id.as_str()) {
                rep.reconstruction_member_ids.clone()
            } else {
                vec![]
            };
        definitions.push(OrdinaryBindingProjectionDefinition {
            projection: projection.clone(),
            source_carrier: source,
            semantic_carrier: target,
            project_definition: name.clone(),
            reconstruct_definition: (projection.binding_id == "binding.identity").then_some(name),
            reconstruction_member_ids,
        });
    }
    Ok((p.b, definitions))
}
pub fn generate_csharp_practical_ordinary_binding_projections(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryBindingProjectionProgram> {
    let construction = generate_construction_vcs(vir).map_err(|_| OrdinaryCarrierError::Linkage)?;
    let vc = generate_binding_vcs(vir, &construction).map_err(|_| OrdinaryCarrierError::Linkage)?;
    let layouts = generate_csharp_practical_ordinary_carriers(vir)?;
    let (b, definitions) = emit_binding_projections(vir, &vc, &layouts, Builder::new()?)?;
    let certificate = b.finish()?;
    let program = OrdinaryBindingProjectionProgram {
        schema: "mpk.csharp.ordinary_binding_projections.v1".into(),
        source_ir_sha256: vir.hash().into(),
        foundation_sha256: vir.construction_context().0.content_sha256().into(),
        binding_vc_sha256: vc.hash(),
        definitions,
        certificate_sha256: mpk_cert::hash_hex(&mpk_cert::certificate_hash(&certificate)),
        certificate,
    };
    if program.canonical_bytes().len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    Ok(program)
}
pub fn import_csharp_practical_ordinary_binding_projections(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryBindingProjectionProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let program = generate_csharp_practical_ordinary_binding_projections(vir)?;
    if input != program.canonical_bytes() || certificate != program.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(program)
}

#[path = "csharp_practical_ordinary_binding_rebuilds.rs"]
mod rebuilds;
pub use rebuilds::{
    generate_csharp_practical_ordinary_binding_rebuilds,
    import_csharp_practical_ordinary_binding_rebuilds, OrdinaryBindingRebuildDefinition,
    OrdinaryBindingRebuildProgram,
};
