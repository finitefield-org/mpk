//! Ordinary semantic equality and eligible canonical ordering over validated
//! concrete carriers. Representation/public domains remain caller obligations.
use super::super::scalar_bits::{bits_relation, special_relation, ScalarRelations};
use super::*;
#[derive(Clone, Debug)]
struct Node {
    depth: u32,
    equal: String,
    compare: Option<String>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryRelationDefinition {
    pub carrier: OrdinaryCarrier,
    pub equality_definition: String,
    /// Absent for every non-total type, independent of the particular value.
    pub compare_definition: Option<String>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryRelationProgram {
    schema: String,
    source_ir_sha256: String,
    foundation_sha256: String,
    definitions: Vec<OrdinaryRelationDefinition>,
    static_transformers: usize,
    certificate_sha256: String,
    #[serde(skip)]
    certificate: Vec<u8>,
}
impl OrdinaryRelationProgram {
    pub fn definitions(&self) -> &[OrdinaryRelationDefinition] {
        &self.definitions
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("typed relation program")
    }
}
fn n(id: &str) -> String {
    format!(
        "{PREFIX}.Relation.T{}",
        id.as_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    )
}
fn and(b: &mut Builder, x: u32, y: u32) -> R<u32> {
    let f = bit(b, false)?;
    mux(b, x, y, f)
}
fn wmux(b: &mut Builder, c: u32, t: u32, e: u32) -> R<u32> {
    call(b, &format!("{PREFIX}.Cube.D5.Mux"), vec![c, t, e])
}
struct Relations<'a> {
    vir: &'a ValidatedPracticalVir,
    carriers: BTreeMap<String, OrdinaryCarrier>,
    b: Builder,
    nodes: BTreeMap<String, Node>,
    active: BTreeSet<String>,
    raw: BTreeMap<(u32, bool), ScalarRelations>,
    special: BTreeMap<String, ScalarRelations>,
}
impl Relations<'_> {
    fn raw(&mut self, width: u32, signed: bool) -> R<Node> {
        let key = (width, signed);
        if !self.raw.contains_key(&key) {
            let d = bits_relation(&mut self.b, &format!("raw.{width}.{signed}"), width, signed)?;
            self.raw.insert(key, d);
        }
        let d = &self.raw[&key];
        Ok(Node {
            depth: address_bits(width),
            equal: d.equal.clone(),
            compare: d.compare.clone(),
        })
    }
    fn special(&mut self, token: &str) -> R<Node> {
        if !self.special.contains_key(token) {
            self.special
                .insert(token.to_owned(), special_relation(&mut self.b, token)?);
        }
        let d = &self.special[token];
        Ok(Node {
            depth: if token == "decimal" {
                9
            } else if token == "f32" {
                5
            } else {
                6
            },
            equal: d.equal.clone(),
            compare: d.compare.clone(),
        })
    }
    fn template(&self, id: &str) -> Option<String> {
        self.vir
            .data_closed()
            .entries()
            .iter()
            .find(|e| e["instance_id"] == id)
            .and_then(|e| e["template_id"].as_str())
            .map(str::to_owned)
    }
    fn internal(&self, id: &str) -> bool {
        self.template(id).as_deref() == Some("mpk.csharp.semantic.sequence_construction.v1")
    }
    fn ty(&mut self, id: &str) -> R<Node> {
        if let Some(node) = self.nodes.get(id) {
            return Ok(node.clone());
        }
        if self.internal(id) {
            return Err(OrdinaryCarrierError::Shape);
        }
        if !self.active.insert(id.to_owned()) {
            return Err(OrdinaryCarrierError::Cycle);
        }
        let carrier = self
            .carriers
            .get(id)
            .ok_or(OrdinaryCarrierError::Linkage)?
            .clone();
        let (bundle, roots, _) = self.vir.construction_context();
        let total = generate_structural_program(bundle, roots, self.vir.data_closed(), id)
            .map_err(|_| OrdinaryCarrierError::Shape)?
            .is_total();
        let scalar = if let Some(source) = roots.source_types.get(id) {
            if source.kind == SourceKind::Enum {
                source.enum_underlying.clone()
            } else {
                None
            }
        } else {
            id.strip_prefix("mpk.csharp.value.")
                .and_then(|s| s.strip_suffix(".v1"))
                .map(str::to_owned)
        };
        let mut node = match scalar.as_deref() {
            Some(token @ ("decimal" | "f32" | "f64")) => self.special(token)?,
            Some(token) if scalar_width(token).is_some() => self.raw(
                scalar_width(token).unwrap(),
                token.starts_with('i') && token != "instant"
                    || matches!(token, "duration" | "instant"),
            )?,
            _ => {
                let order = if self.template(id).as_deref() == Some("mpk.csharp.semantic.money.v1")
                {
                    Some(vec![1usize, 0])
                } else {
                    None
                };
                if scalar.as_deref() == Some("exception") {
                    let OrdinaryShape::Sum { arms } = &carrier.shape else {
                        return Err(OrdinaryCarrierError::Shape);
                    };
                    self.sum(arms, &n(id), false)?
                } else {
                    self.shape(&carrier.shape, &n(id), order.as_deref())?
                }
            }
        };
        if node.depth != carrier.depth {
            return Err(OrdinaryCarrierError::Shape);
        }
        if !total {
            node.compare = None;
        }
        if total && node.compare.is_none() {
            return Err(OrdinaryCarrierError::Shape);
        }
        self.nodes.insert(id.to_owned(), node.clone());
        self.active.remove(id);
        Ok(node)
    }
    fn eq_word(&mut self, left: u32, right: u32) -> R<u32> {
        let eq = self.raw(32, false)?.equal;
        call(&mut self.b, &eq, vec![left, right])
    }
    fn empty_word(&mut self, value: u32) -> R<u32> {
        let zero = ordered_fold::word(&mut self.b, 0)?;
        self.eq_word(value, zero)
    }
    fn getter(&mut self, name: &str, source: u32, target: u32, prefix: &[bool]) -> R<String> {
        project(&mut self.b, name, source, target, prefix, None)?;
        Ok(name.to_owned())
    }
    fn product(
        &mut self,
        fields: &[OrdinaryField],
        name: &str,
        order: Option<&[usize]>,
    ) -> R<Node> {
        let mut children = vec![];
        for (i, f) in fields.iter().enumerate() {
            children.push(self.shape(&f.shape, &format!("{name}.Child.F{i}"), None)?);
        }
        let max = children.iter().map(|n| n.depth).max().unwrap_or(0);
        let roles = address_bits(fields.len() as u32);
        let depth = roles + max;
        let mut getters = vec![];
        for (i, child) in children.iter().enumerate() {
            let mut address = prefix(roles, i as u32);
            address.extend(vec![false; (max - child.depth) as usize]);
            getters.push(self.getter(
                &format!("{name}.Field.F{i}"),
                depth,
                child.depth,
                &address,
            )?);
        }
        let left = self.b.var(1)?;
        let right = self.b.var(0)?;
        let mut equal = bit(&mut self.b, true)?;
        for (child, getter) in children.iter().zip(&getters) {
            let a = call(&mut self.b, getter, vec![left])?;
            let v = call(&mut self.b, getter, vec![right])?;
            let eq = call(&mut self.b, &child.equal, vec![a, v])?;
            equal = and(&mut self.b, equal, eq)?;
        }
        let equal_name = format!("{name}.Equal");
        define(&mut self.b, &equal_name, &[depth, depth], 0, equal)?;
        let mut compare = None;
        if children.iter().all(|c| c.compare.is_some()) {
            let default_order = (0..children.len()).collect::<Vec<_>>();
            let order = order.unwrap_or(&default_order);
            if order.iter().copied().collect::<BTreeSet<_>>()
                != (0..children.len()).collect::<BTreeSet<_>>()
                || order.len() != children.len()
            {
                return Err(OrdinaryCarrierError::Shape);
            }
            let mut result = ordered_fold::word(&mut self.b, 0)?;
            for &i in order.iter().rev() {
                let a = call(&mut self.b, &getters[i], vec![left])?;
                let v = call(&mut self.b, &getters[i], vec![right])?;
                let value = call(
                    &mut self.b,
                    children[i].compare.as_ref().unwrap(),
                    vec![a, v],
                )?;
                let empty = self.empty_word(value)?;
                result = wmux(&mut self.b, empty, result, value)?;
            }
            let cmp = format!("{name}.Compare");
            define(&mut self.b, &cmp, &[depth, depth], 5, result)?;
            compare = Some(cmp);
        }
        Ok(Node {
            depth,
            equal: equal_name,
            compare,
        })
    }
    fn sum(&mut self, arms: &[OrdinaryArm], name: &str, allow_compare: bool) -> R<Node> {
        let mut children = vec![];
        for arm in arms {
            children.push(self.product(&arm.fields, &format!("{name}.Arm.A{}", arm.tag), None)?);
        }
        let payload = 5.max(children.iter().map(|n| n.depth).max().unwrap_or(0));
        let depth = 1 + payload;
        let mut address = vec![false];
        address.extend(vec![false; (payload - 5) as usize]);
        let tag = self.getter(&format!("{name}.Tag"), depth, 5, &address)?;
        let left = self.b.var(1)?;
        let right = self.b.var(0)?;
        let lt = call(&mut self.b, &tag, vec![left])?;
        let rt = call(&mut self.b, &tag, vec![right])?;
        let tags_equal = self.eq_word(lt, rt)?;
        let mut equal = bit(&mut self.b, false)?;
        let mut result = ordered_fold::word(&mut self.b, 0)?;
        let eligible = allow_compare && children.iter().all(|c| c.compare.is_some());
        let mut seen = BTreeSet::new();
        for (arm, child) in arms.iter().zip(&children).rev() {
            if !seen.insert(arm.tag) {
                return Err(OrdinaryCarrierError::Shape);
            }
            let mut address = vec![true];
            address.extend(vec![false; (payload - child.depth) as usize]);
            let getter = self.getter(
                &format!("{name}.Payload.A{}", arm.tag),
                depth,
                child.depth,
                &address,
            )?;
            let a = call(&mut self.b, &getter, vec![left])?;
            let v = call(&mut self.b, &getter, vec![right])?;
            let tag_value = ordered_fold::word(&mut self.b, arm.tag)?;
            let active = self.eq_word(lt, tag_value)?;
            let eq = call(&mut self.b, &child.equal, vec![a, v])?;
            equal = mux(&mut self.b, active, eq, equal)?;
            if eligible {
                let cmp = call(&mut self.b, child.compare.as_ref().unwrap(), vec![a, v])?;
                result = wmux(&mut self.b, active, cmp, result)?;
            }
        }
        let equal = and(&mut self.b, tags_equal, equal)?;
        let eq = format!("{name}.Equal");
        define(&mut self.b, &eq, &[depth, depth], 0, equal)?;
        let mut compare = None;
        if eligible {
            let tag_cmp = self.raw(32, false)?.compare.unwrap();
            let tag_cmp = call(&mut self.b, &tag_cmp, vec![lt, rt])?;
            let result = wmux(&mut self.b, tags_equal, result, tag_cmp)?;
            let cmp = format!("{name}.Compare");
            define(&mut self.b, &cmp, &[depth, depth], 5, result)?;
            compare = Some(cmp);
        }
        Ok(Node {
            depth,
            equal: eq,
            compare,
        })
    }
    fn sequence(&mut self, capacity: u32, element: &OrdinaryShape, name: &str) -> R<Node> {
        let child = self.shape(element, &format!("{name}.Element"), None)?;
        let index_bits = address_bits(capacity);
        let array_depth = index_bits + child.depth;
        let payload = 5.max(array_depth);
        let depth = payload + 1;
        let fold = ordered_fold::emit_fold(&mut self.b, index_bits)?;
        let mut length_address = vec![false];
        length_address.extend(vec![false; (payload - 5) as usize]);
        let length = self.getter(&format!("{name}.Length"), depth, 5, &length_address)?;
        let source = self.b.var(child.depth + 1)?;
        let index = self.b.var(child.depth)?;
        let mut address = vec![bit(&mut self.b, true)?];
        address.extend(vec![
            bit(&mut self.b, false)?;
            (payload - array_depth) as usize
        ]);
        for i in 0..index_bits {
            let a = prefix(5, i)
                .into_iter()
                .map(|v| bit(&mut self.b, v))
                .collect::<R<Vec<_>>>()?;
            address.push(self.b.app(index, a)?);
        }
        address.extend(self.b.selectors(child.depth)?);
        let body = self.b.app(source, address)?;
        let body = self.b.wrap_selectors(child.depth, body)?;
        let read = format!("{name}.ReadAt");
        define(&mut self.b, &read, &[depth, 5], child.depth, body)?;
        // Build a 32-bit index from the low-first index selector group.
        // The high word bits are zero, so no truncation can alias a valid index.
        let index_word = |b: &mut Builder, extra: u32| -> R<u32> {
            let mut body = bit(b, false)?;
            for i in 0..index_bits {
                let at = equal_address(b, 5, 0, 5, i)?;
                let value = b.var(5 + extra + index_bits - 1 - i)?;
                body = mux(b, at, value, body)?;
            }
            b.wrap_selectors(5, body)
        };
        let left = self.b.var(1)?;
        let right = self.b.var(0)?;
        let llen = call(&mut self.b, &length, vec![left])?;
        let rlen = call(&mut self.b, &length, vec![right])?;
        let len_eq = self.eq_word(llen, rlen)?;
        let length_cmp = self.raw(32, false)?.compare.unwrap();
        let lengths = call(&mut self.b, &length_cmp, vec![llen, rlen])?;
        let sign = prefix(5, 31)
            .into_iter()
            .map(|v| bit(&mut self.b, v))
            .collect::<R<Vec<_>>>()?;
        let less = self.b.app(lengths, sign)?;
        let min = wmux(&mut self.b, less, llen, rlen)?;
        let a = self.b.var(index_bits + 1)?;
        let v = self.b.var(index_bits)?;
        let idx = index_word(&mut self.b, 0)?;
        let a = call(&mut self.b, &read, vec![a, idx])?;
        let v = call(&mut self.b, &read, vec![v, idx])?;
        let equal = call(&mut self.b, &child.equal, vec![a, v])?;
        let predicate = self.b.wrap_selectors(index_bits, equal)?;
        let equal = call(&mut self.b, &fold.all_definition, vec![predicate, min])?;
        let equal = and(&mut self.b, len_eq, equal)?;
        let eq = format!("{name}.Equal");
        define(&mut self.b, &eq, &[depth, depth], 0, equal)?;
        let mut compare = None;
        if let Some(child_cmp) = &child.compare {
            let a = self.b.var(index_bits + 1)?;
            let v = self.b.var(index_bits)?;
            let idx = index_word(&mut self.b, 0)?;
            let a = call(&mut self.b, &read, vec![a, idx])?;
            let v = call(&mut self.b, &read, vec![v, idx])?;
            let cmp = call(&mut self.b, child_cmp, vec![a, v])?;
            let predicate = self.b.wrap_selectors(index_bits, cmp)?;
            let first = call(&mut self.b, &fold.first_definition, vec![predicate, min])?;
            let empty = self.empty_word(first)?;
            let result = wmux(&mut self.b, empty, lengths, first)?;
            let cmp = format!("{name}.Compare");
            define(&mut self.b, &cmp, &[depth, depth], 5, result)?;
            compare = Some(cmp);
        }
        Ok(Node {
            depth,
            equal: eq,
            compare,
        })
    }
    fn shape(&mut self, shape: &OrdinaryShape, name: &str, order: Option<&[usize]>) -> R<Node> {
        match shape {
            OrdinaryShape::Bits { width } => self.raw(*width, false),
            OrdinaryShape::Reference { type_id } => self.ty(type_id),
            OrdinaryShape::RoleBound { value, .. } => self.shape(value, name, order),
            OrdinaryShape::Product { fields } => self.product(fields, name, order),
            OrdinaryShape::Sum { arms } => self.sum(arms, name, true),
            OrdinaryShape::Sequence { capacity, element } => {
                self.sequence(*capacity, element, name)
            }
            // Raw arrays only occur inside erased construction state. Storable
            // source arrays use the length-bearing sequence representation.
            OrdinaryShape::Array { .. } => Err(OrdinaryCarrierError::Shape),
        }
    }
}
pub fn generate_csharp_practical_ordinary_relations(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryRelationProgram> {
    let layouts = generate_csharp_practical_ordinary_carriers(vir)?;
    let carriers = layouts
        .carriers()
        .iter()
        .map(|c| (c.type_id.clone(), c.clone()))
        .collect();
    let mut r = Relations {
        vir,
        carriers,
        b: Builder::new()?,
        nodes: BTreeMap::new(),
        active: BTreeSet::new(),
        raw: BTreeMap::new(),
        special: BTreeMap::new(),
    };
    r.b.helpers(5)?;
    let mut definitions = vec![];
    for carrier in layouts.carriers() {
        if r.internal(&carrier.type_id) {
            continue;
        }
        let d = r.ty(&carrier.type_id)?;
        definitions.push(OrdinaryRelationDefinition {
            carrier: carrier.clone(),
            equality_definition: d.equal,
            compare_definition: d.compare,
        });
    }
    let static_transformers = r.b.static_transformers;
    let certificate = r.b.finish()?;
    let p = OrdinaryRelationProgram {
        schema: "mpk.csharp.ordinary_relations.v1".into(),
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
pub fn import_csharp_practical_ordinary_relations(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryRelationProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let expected = generate_csharp_practical_ordinary_relations(vir)?;
    if input != expected.canonical_bytes() || certificate != expected.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(expected)
}
