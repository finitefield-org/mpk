//! Reconstruction from semantic values with explicit source completions.
//! A unary Reconstruct witness and its source-invariant/round-trip proofs remain required.
use super::*;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryBindingRebuildDefinition {
    pub projection: OrdinaryBindingProjectionDefinition,
    /// Arguments: semantic value, source completion. Result: source value.
    pub rebuild_definition: String,
    /// This component never supplies or assumes a unary source-completion witness.
    pub completion_witness_required: bool,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryBindingRebuildProgram {
    schema: String,
    source_ir_sha256: String,
    foundation_sha256: String,
    binding_vc_sha256: String,
    definitions: Vec<OrdinaryBindingRebuildDefinition>,
    pending_reconstruct_symbols: Vec<String>,
    certificate_sha256: String,
    #[serde(skip)]
    certificate: Vec<u8>,
}
impl OrdinaryBindingRebuildProgram {
    pub fn definitions(&self) -> &[OrdinaryBindingRebuildDefinition] {
        &self.definitions
    }
    pub fn pending_reconstruct_symbols(&self) -> &[String] {
        &self.pending_reconstruct_symbols
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("typed source reconstruction completions")
    }
}
struct Rebuild<'a> {
    p: Projections<'a>,
    nodes: BTreeMap<(String, String), String>,
    active: BTreeSet<(String, String)>,
}
impl Rebuild<'_> {
    fn choose(&mut self, depth: u32, active: u32, yes: u32, no: u32) -> R<u32> {
        let name = format!("{PREFIX}.Cube.D{depth}.Mux");
        if !self.p.b.globals.contains_key(&name) {
            self.p.b.helpers(depth)?;
        }
        call(&mut self.p.b, &name, vec![active, yes, no])
    }
    fn tag_value(&mut self, ty: &str, n: u128) -> R<u32> {
        let c = self.p.carrier(ty)?;
        let OrdinaryShape::Bits { width } = c.shape else {
            return Err(OrdinaryCarrierError::Shape);
        };
        if width == 0 || width > 64 {
            return Err(OrdinaryCarrierError::Shape);
        }
        let name = named("RebuildTag", &(ty, n.to_string()));
        if !self.p.b.globals.contains_key(&name) {
            let mut body = bit(&mut self.p.b, false)?;
            for i in 0..width {
                if n & (1u128 << i) != 0 {
                    let at = equal_address(&mut self.p.b, c.depth, 0, c.depth, i)?;
                    body = call(&mut self.p.b, "Std.Bool.or", vec![body, at])?;
                }
            }
            let body = self.p.b.wrap_selectors(c.depth, body)?;
            define(&mut self.p.b, &name, &[], c.depth, body)?;
        }
        call(&mut self.p.b, &name, vec![])
    }
    fn active_arm(&mut self, semantic: &OrdinaryCarrier, arm: &OrdinaryArm, value: u32) -> R<u32> {
        let get = self
            .p
            .getter(semantic, 5, vec![false; (semantic.depth - 5) as usize])?;
        let test = self.p.tag_test(semantic, &get, 5, 32, arm.tag as u128)?;
        call(&mut self.p.b, &test, vec![value])
    }
    fn arm_field(
        &mut self,
        c: &OrdinaryCarrier,
        arm: &OrdinaryArm,
        index: usize,
    ) -> R<(String, String)> {
        let (_, depth) = self.p.packing(&arm.fields)?;
        let mut address = vec![true];
        address.extend(vec![false; (c.depth - 1 - depth) as usize]);
        self.p.field_getter(c, &arm.fields, index, address)
    }
    fn child(&mut self, source: &str, semantic: &str, value: u32, completion: u32) -> R<u32> {
        let rebuild = self.ty(source, semantic)?;
        call(&mut self.p.b, &rebuild, vec![value, completion])
    }
    fn bound(
        &mut self,
        source: &OrdinaryCarrier,
        semantic: &OrdinaryCarrier,
        binding: &Value,
    ) -> R<u32> {
        let OrdinaryShape::Product { fields } = &source.shape else {
            return Err(OrdinaryCarrierError::Shape);
        };
        let role = text(binding, "role")?;
        let outer = source.depth;
        let value = self.p.b.var(outer + 1)?;
        let completion = self.p.b.var(outer)?;
        let mut children = vec![];
        for (index, field) in fields.iter().enumerate() {
            let (seed_get, from) = self.p.field_getter(source, fields, index, vec![])?;
            let seed = call(&mut self.p.b, &seed_get, vec![completion])?;
            let roles = binding["member_map"]
                .as_object()
                .ok_or(OrdinaryCarrierError::Shape)?
                .iter()
                .filter(|(_, v)| v.as_str() == Some(field.id.as_str()))
                .map(|(k, _)| k.as_str())
                .collect::<Vec<_>>();
            if roles.len() > 1 {
                return Err(OrdinaryCarrierError::Linkage);
            }
            let Some(member_role) = roles.first().copied() else {
                children.push(seed);
                continue;
            };
            let child = match role {
                "instant" => {
                    if member_role != "milliseconds"
                        || from != "mpk.csharp.value.i64.v1"
                        || semantic.type_id != "mpk.csharp.value.instant.v1"
                    {
                        return Err(OrdinaryCarrierError::Shape);
                    }
                    value
                }
                "money" | "ordered_entry" | "transition" => {
                    let OrdinaryShape::Product { fields: targets } = &semantic.shape else {
                        return Err(OrdinaryCarrierError::Shape);
                    };
                    let i = targets
                        .iter()
                        .position(|f| f.id == member_role)
                        .ok_or(OrdinaryCarrierError::Linkage)?;
                    let (get, to) = self.p.field_getter(semantic, targets, i, vec![])?;
                    let selected = call(&mut self.p.b, &get, vec![value])?;
                    self.child(&from, &to, selected, seed)?
                }
                "bounded_sequence" | "ordered_map" | "ordered_set" => {
                    self.child(&from, &semantic.type_id, value, seed)?
                }
                "option" | "result" | "lookup" | "validation" | "boundary_field" => {
                    let OrdinaryShape::Sum { arms } = &semantic.shape else {
                        return Err(OrdinaryCarrierError::Shape);
                    };
                    let mut result = seed;
                    for arm in arms.iter().rev() {
                        let selected = if member_role == "tag" {
                            let n = text(&binding["tag_arms"], &arm.id)?
                                .parse::<i128>()
                                .map_err(|_| OrdinaryCarrierError::Shape)?
                                as u128;
                            self.tag_value(&from, n)?
                        } else {
                            let wanted = match (role, arm.id.as_str()) {
                                ("result", "error") => "error",
                                ("validation", "invalid") => "errors",
                                _ => "value",
                            };
                            if wanted != member_role || arm.fields.is_empty() {
                                continue;
                            }
                            if arm.fields.len() != 1 {
                                return Err(OrdinaryCarrierError::Shape);
                            }
                            let (get, to) = self.arm_field(semantic, arm, 0)?;
                            let selected = call(&mut self.p.b, &get, vec![value])?;
                            self.child(&from, &to, selected, seed)?
                        };
                        let active = self.active_arm(semantic, arm, value)?;
                        result =
                            self.choose(self.p.carrier(&from)?.depth, active, selected, result)?;
                    }
                    result
                }
                _ => return Err(OrdinaryCarrierError::Shape),
            };
            children.push(child);
        }
        let body = self.p.packed_leaf(outer, 0, fields, &children)?;
        self.p.b.wrap_selectors(outer, body)
    }
    fn read(&mut self, c: &OrdinaryCarrier, child: u32, indices: u32) -> R<String> {
        let name = named("RebuildRead", &c.type_id);
        if !self.p.b.globals.contains_key(&name) {
            let value = self.p.b.var(1)?;
            let index = self.p.b.var(0)?;
            let mut address = vec![bit(&mut self.p.b, true)?];
            address.extend(vec![
                bit(&mut self.p.b, false)?;
                (c.depth - 1 - indices - child) as usize
            ]);
            for i in 0..indices {
                address.push(ordered_fold::read_bit(&mut self.p.b, index, i)?);
            }
            let body = self.p.b.app(value, address)?;
            define(&mut self.p.b, &name, &[c.depth, 5], child, body)?;
        }
        Ok(name)
    }
    fn sequence(&mut self, source: &OrdinaryCarrier, semantic: &OrdinaryCarrier) -> R<u32> {
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
        } = &semantic.shape
        else {
            return Err(OrdinaryCarrierError::Shape);
        };
        if sc != tc {
            return Err(OrdinaryCarrierError::Shape);
        }
        let from = self.p.element_type(se)?;
        let to = self.p.element_type(te)?;
        let sd = shape_depth(se, &self.p.carriers)?;
        let td = shape_depth(te, &self.p.carriers)?;
        let indices = address_bits(*sc);
        let source_read = self.read(source, sd, indices)?;
        let semantic_read = self.read(semantic, td, indices)?;
        let get_length = self
            .p
            .getter(semantic, 5, vec![false; (semantic.depth - 5) as usize])?;
        let outer = source.depth;
        let value = self.p.b.var(outer + 1)?;
        let completion = self.p.b.var(outer)?;
        let length = call(&mut self.p.b, &get_length, vec![value])?;
        let selectors = self.p.b.selectors(5)?;
        let header = self.p.b.app(length, selectors)?;
        let header = zero_padding(&mut self.p.b, outer, 1, outer - 1 - 5, header)?;
        let index = self.p.index_word(indices, outer, outer - indices - sd)?;
        let visible = ordered_fold::helper(&mut self.p.b, "Less", vec![index, length])?;
        let selected = call(&mut self.p.b, &semantic_read, vec![value, index])?;
        let seed = call(&mut self.p.b, &source_read, vec![completion, index])?;
        let rebuilt = self.child(&from, &to, selected, seed)?;
        let selectors = self.p.b.selectors(sd)?;
        let leaf = self.p.b.app(rebuilt, selectors)?;
        let zero = bit(&mut self.p.b, false)?;
        let leaf = mux(&mut self.p.b, visible, leaf, zero)?;
        let leaf = zero_padding(&mut self.p.b, outer, 1, outer - 1 - indices - sd, leaf)?;
        let data = self.p.b.var(outer - 1)?;
        let body = mux(&mut self.p.b, data, leaf, header)?;
        self.p.b.wrap_selectors(outer, body)
    }
    fn sum(&mut self, source: &OrdinaryCarrier, semantic: &OrdinaryCarrier) -> R<u32> {
        let OrdinaryShape::Sum { arms: sa } = &source.shape else {
            return Err(OrdinaryCarrierError::Shape);
        };
        let OrdinaryShape::Sum { arms: ta } = &semantic.shape else {
            return Err(OrdinaryCarrierError::Shape);
        };
        if sa.len() != ta.len() {
            return Err(OrdinaryCarrierError::Shape);
        }
        let outer = source.depth;
        let value = self.p.b.var(outer + 1)?;
        let completion = self.p.b.var(outer)?;
        let selectors = self.p.b.selectors(outer)?;
        let mut body = self.p.b.app(completion, selectors)?;
        for arm in sa.iter().rev() {
            let target = ta
                .iter()
                .find(|a| a.id == arm.id && a.tag == arm.tag)
                .ok_or(OrdinaryCarrierError::Shape)?;
            if arm.fields.len() != target.fields.len() {
                return Err(OrdinaryCarrierError::Shape);
            }
            let mut children = vec![];
            for i in 0..arm.fields.len() {
                if arm.fields[i].id != target.fields[i].id {
                    return Err(OrdinaryCarrierError::Shape);
                }
                let (get, from) = self.arm_field(source, arm, i)?;
                let seed = call(&mut self.p.b, &get, vec![completion])?;
                let (get, to) = self.arm_field(semantic, target, i)?;
                let selected = call(&mut self.p.b, &get, vec![value])?;
                children.push(self.child(&from, &to, selected, seed)?);
            }
            let rebuilt = self.p.arm_leaf(source, arm, &children)?;
            let active = self.active_arm(semantic, target, value)?;
            body = mux(&mut self.p.b, active, rebuilt, body)?;
        }
        self.p.b.wrap_selectors(outer, body)
    }
    fn ty(&mut self, source: &str, semantic: &str) -> R<String> {
        let key = (source.to_owned(), semantic.to_owned());
        if let Some(n) = self.nodes.get(&key) {
            return Ok(n.clone());
        }
        if !self.active.insert(key.clone()) {
            return Err(OrdinaryCarrierError::Cycle);
        }
        let sc = self.p.carrier(source)?;
        let tc = self.p.carrier(semantic)?;
        let name = named("Rebuild", &key);
        let body = if source == semantic {
            self.p.b.var(1)?
        } else if let Some(binding) = self.p.bindings.get(source).copied() {
            if binding.projection.semantic_type_id != semantic {
                return Err(OrdinaryCarrierError::Linkage);
            }
            self.bound(&sc, &tc, &binding.binding)?
        } else {
            match (&sc.shape, &tc.shape) {
                (OrdinaryShape::Sequence { .. }, OrdinaryShape::Sequence { .. }) => {
                    self.sequence(&sc, &tc)?
                }
                (OrdinaryShape::Sum { .. }, OrdinaryShape::Sum { .. }) => self.sum(&sc, &tc)?,
                _ => return Err(OrdinaryCarrierError::Shape),
            }
        };
        define(&mut self.p.b, &name, &[tc.depth, sc.depth], sc.depth, body)?;
        self.nodes.insert(key.clone(), name.clone());
        self.active.remove(&key);
        Ok(name)
    }
}
pub fn generate_csharp_practical_ordinary_binding_rebuilds(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryBindingRebuildProgram> {
    let construction = generate_construction_vcs(vir).map_err(|_| OrdinaryCarrierError::Linkage)?;
    let vc = generate_binding_vcs(vir, &construction).map_err(|_| OrdinaryCarrierError::Linkage)?;
    let layouts = generate_csharp_practical_ordinary_carriers(vir)?;
    let (b, projections) = emit_binding_projections(vir, &vc, &layouts, Builder::new()?)?;
    let mut r = Rebuild {
        p: Projections {
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
        },
        nodes: BTreeMap::new(),
        active: BTreeSet::new(),
    };
    let mut definitions = vec![];
    let mut pending_reconstruct_symbols = vec![];
    for projection in projections {
        let rebuild_definition = r.ty(
            &projection.source_carrier.type_id,
            &projection.semantic_carrier.type_id,
        )?;
        let completion_witness_required = projection.reconstruct_definition.is_none();
        if completion_witness_required {
            pending_reconstruct_symbols.push(projection.projection.reconstruct.id.clone());
        }
        definitions.push(OrdinaryBindingRebuildDefinition {
            projection,
            rebuild_definition,
            completion_witness_required,
        });
    }
    pending_reconstruct_symbols.sort();
    pending_reconstruct_symbols.dedup();
    let certificate = r.p.b.finish()?;
    let p = OrdinaryBindingRebuildProgram {
        schema: "mpk.csharp.ordinary_binding_rebuilds.v1".into(),
        source_ir_sha256: vir.hash().into(),
        foundation_sha256: vir.construction_context().0.content_sha256().into(),
        binding_vc_sha256: vc.hash(),
        definitions,
        pending_reconstruct_symbols,
        certificate_sha256: mpk_cert::hash_hex(&mpk_cert::certificate_hash(&certificate)),
        certificate,
    };
    if p.canonical_bytes().len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    Ok(p)
}
pub fn import_csharp_practical_ordinary_binding_rebuilds(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryBindingRebuildProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let p = generate_csharp_practical_ordinary_binding_rebuilds(vir)?;
    if input != p.canonical_bytes() || certificate != p.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(p)
}
