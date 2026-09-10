//! Depth of the canonical typed JSON tree, conditional on a valid semantic
//! representation. Raw-node counts, value domains and source VCs are separate.
use super::super::integer_format::circuit_with_block_bits;
use super::super::temporal::literal;
use super::*;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryJsonTypedDepthDefinition {
    pub carrier: OrdinaryCarrier,
    /// Semantic value, C5 root depth -> Bool. Active JSON nodes must be <=32.
    pub definition: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryJsonTypedDepthProgram {
    schema: String,
    source_ir_sha256: String,
    boundary_program_sha256: String,
    definitions: Vec<OrdinaryJsonTypedDepthDefinition>,
    static_transformers: usize,
    certificate_sha256: String,
    #[serde(skip)]
    certificate: Vec<u8>,
}
impl OrdinaryJsonTypedDepthProgram {
    pub fn definitions(&self) -> &[OrdinaryJsonTypedDepthDefinition] {
        &self.definitions
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("typed JSON depth metadata")
    }
}
#[derive(Clone)]
struct Node {
    depth: u32,
    name: String,
}
struct Depths<'a> {
    b: &'a mut Builder,
    carriers: BTreeMap<&'a str, &'a OrdinaryCarrier>,
    atoms: BTreeSet<String>,
    rejected: BTreeSet<String>,
    nodes: BTreeMap<String, Node>,
    active: BTreeSet<String>,
}
const BASE: &str = "Mpk.CSharp.Ordinary.JsonTypedDepth";
fn predicate(b: &mut Builder, suffix: &str, max: u32) -> R<String> {
    let name = format!("{BASE}.{suffix}");
    if !b.globals.contains_key(&name) {
        let mut c = Circuit::new(&[32]);
        let input = c.inputs[0].clone();
        let valid = c.lt(&input, &literal(u128::from(max) + 1, 32), false);
        let f = circuit_with_block_bits(b, &name, c, vec![valid], 7)?;
        let input = b.var(0)?;
        let body = call(b, &f, vec![input])?;
        define(b, &name, &[5], 0, body)?;
    }
    Ok(name)
}
fn next(b: &mut Builder, value: u32) -> R<u32> {
    let name = format!("{BASE}.Next");
    if !b.globals.contains_key(&name) {
        let mut c = Circuit::new(&[32]);
        let input = c.inputs[0].clone();
        // Saturate invalid depths, so u32::MAX can never wrap into acceptance.
        let valid = c.lt(&input, &literal(32, 32), false);
        let n = c.add(&input, &literal(1, 32), F).0;
        let out = c.select(valid, &n, &literal(33, 32));
        let f = circuit_with_block_bits(b, &name, c, out, 7)?;
        let input = b.var(0)?;
        let body = call(b, &f, vec![input])?;
        define(b, &name, &[5], 5, body)?;
    }
    call(b, &name, vec![value])
}
fn conjunction(b: &mut Builder, a: u32, c: u32) -> R<u32> {
    call(b, "Std.Bool.and", vec![a, c])
}
impl Depths<'_> {
    fn finish(&mut self, name: &str, depth: u32, body: u32) -> R<Node> {
        define(self.b, name, &[depth, 5], 0, body)?;
        Ok(Node {
            depth,
            name: name.into(),
        })
    }
    fn ty(&mut self, id: &str) -> R<Node> {
        if let Some(n) = self.nodes.get(id) {
            return Ok(n.clone());
        }
        if !self.active.insert(id.into()) {
            return Err(OrdinaryCarrierError::Cycle);
        }
        let c = (*self.carriers.get(id).ok_or(OrdinaryCarrierError::Linkage)?).clone();
        let name = format!(
            "{BASE}.T{}",
            id.as_bytes()
                .iter()
                .map(|x| format!("{x:02x}"))
                .collect::<String>()
        );
        let node = if self.rejected.contains(id) {
            let body = truth(self.b, false)?;
            self.finish(&name, c.depth, body)?
        } else if self.atoms.contains(id) {
            let helper = predicate(self.b, "Within", 32)?;
            let depth = self.b.var(0)?;
            let body = call(self.b, &helper, vec![depth])?;
            self.finish(&name, c.depth, body)?
        } else {
            self.shape(&c.shape, &name)?
        };
        if node.depth != c.depth {
            return Err(OrdinaryCarrierError::Shape);
        }
        self.active.remove(id);
        self.nodes.insert(id.into(), node.clone());
        Ok(node)
    }
    fn shape(&mut self, shape: &OrdinaryShape, name: &str) -> R<Node> {
        match shape {
            OrdinaryShape::Reference { type_id } => self.ty(type_id),
            OrdinaryShape::RoleBound { value, .. } => self.shape(value, name),
            OrdinaryShape::Bits { width } => {
                let helper = predicate(self.b, "Within", 32)?;
                let depth = self.b.var(0)?;
                let body = call(self.b, &helper, vec![depth])?;
                self.finish(name, address_bits(*width), body)
            }
            OrdinaryShape::Product { fields } => {
                let children = fields
                    .iter()
                    .enumerate()
                    .map(|(i, f)| self.shape(&f.shape, &format!("{name}.F{i}")))
                    .collect::<R<Vec<_>>>()?;
                let roles = address_bits(fields.len() as u32);
                let max = children.iter().map(|n| n.depth).max().unwrap_or(0);
                let depth = roles + max;
                let helper = predicate(self.b, "Within", 32)?;
                let root = self.b.var(0)?;
                let mut body = call(self.b, &helper, vec![root])?;
                for (i, child) in children.iter().enumerate() {
                    let source = self.b.var(1)?;
                    let mut args = (0..roles)
                        .map(|j| truth(self.b, (i >> j) & 1 != 0))
                        .collect::<R<Vec<_>>>()?;
                    args.extend(vec![truth(self.b, false)?; (max - child.depth) as usize]);
                    let value = self.b.app(source, args)?;
                    let root = self.b.var(0)?;
                    let next = next(self.b, root)?;
                    let valid = call(self.b, &child.name, vec![value, next])?;
                    body = conjunction(self.b, body, valid)?;
                }
                self.finish(name, depth, body)
            }
            OrdinaryShape::Sum { arms } => {
                let children = arms
                    .iter()
                    .enumerate()
                    .map(|(i, a)| {
                        if a.fields.len() > 1 {
                            return Err(OrdinaryCarrierError::Shape);
                        }
                        a.fields
                            .first()
                            .map(|f| self.shape(&f.shape, &format!("{name}.A{i}")))
                            .transpose()
                    })
                    .collect::<R<Vec<_>>>()?;
                let max = 5.max(
                    children
                        .iter()
                        .flatten()
                        .map(|c| c.depth)
                        .max()
                        .unwrap_or(0),
                );
                let depth = max + 1;
                let mut result = truth(self.b, false)?;
                for (arm, child) in arms.iter().zip(children) {
                    // The sum object's tag is always a child, even without payload.
                    let helper = predicate(self.b, "RoomForTag", 31)?;
                    let root = self.b.var(0)?;
                    let mut valid = call(self.b, &helper, vec![root])?;
                    if let Some(child) = child {
                        let source = self.b.var(1)?;
                        let mut args = vec![truth(self.b, true)?];
                        args.extend(vec![truth(self.b, false)?; (max - child.depth) as usize]);
                        let value = self.b.app(source, args)?;
                        let root = self.b.var(0)?;
                        let next = next(self.b, root)?;
                        let good = call(self.b, &child.name, vec![value, next])?;
                        valid = conjunction(self.b, valid, good)?;
                    }
                    let source = self.b.var(6)?;
                    let mut args = vec![truth(self.b, false)?; (depth - 5) as usize];
                    args.extend(self.b.selectors(5)?);
                    let tag = self.b.app(source, args)?;
                    let tag = self.b.wrap_selectors(5, tag)?;
                    let mut selected = truth(self.b, true)?;
                    for i in 0..32 {
                        let mut bit = core_read(self.b, tag, i, 5)?;
                        if arm.tag & (1u32 << i) == 0 {
                            bit = call(self.b, "Std.Bool.not", vec![bit])?;
                        }
                        selected = conjunction(self.b, selected, bit)?;
                    }
                    result = core_mux(self.b, selected, valid, result)?;
                }
                self.finish(name, depth, result)
            }
            OrdinaryShape::Sequence { capacity, element } => {
                let child = self.shape(element, &format!("{name}.Element"))?;
                let indices = address_bits(*capacity);
                if *capacity == 0 || indices > 14 {
                    return Err(OrdinaryCarrierError::Shape);
                }
                let payload = 5.max(indices + child.depth);
                let depth = payload + 1;
                let source = self.b.var(6)?;
                let mut args = vec![truth(self.b, false)?; (depth - 5) as usize];
                args.extend(self.b.selectors(5)?);
                let length = self.b.app(source, args)?;
                let length = self.b.wrap_selectors(5, length)?;
                let cap = predicate(self.b, &format!("Capacity{capacity}"), *capacity)?;
                let mut body = call(self.b, &cap, vec![length])?;
                let within = predicate(self.b, "Within", 32)?;
                let root = self.b.var(0)?;
                let valid = call(self.b, &within, vec![root])?;
                body = conjunction(self.b, body, valid)?;
                // Predicate indexes only active elements; inactive storage cannot
                // make an empty or shorter sequence appear more deeply nested.
                let source = self.b.var(indices + 1)?;
                let mut args = vec![truth(self.b, true)?];
                args.extend(vec![
                    truth(self.b, false)?;
                    (payload - indices - child.depth) as usize
                ]);
                args.extend(self.b.selectors(indices)?);
                let value = self.b.app(source, args)?;
                let root = self.b.var(indices)?;
                let next = next(self.b, root)?;
                let valid = call(self.b, &child.name, vec![value, next])?;
                let input = self.b.wrap_selectors(indices, valid)?;
                let fold = super::super::super::structural::emit_aggregate_fold(self.b, indices)?;
                let all = call(self.b, &fold.all_definition, vec![input, length])?;
                body = conjunction(self.b, body, all)?;
                self.finish(name, depth, body)
            }
            OrdinaryShape::Array { .. } => Err(OrdinaryCarrierError::Shape),
        }
    }
}
pub(super) fn emit(
    emitted: &EmittedDataPhase,
    layouts: &OrdinaryCarrierProgram,
    b: &mut Builder,
) -> R<Vec<OrdinaryJsonTypedDepthDefinition>> {
    let vir = emitted.vir();
    let (_, roots, _) = vir.construction_context();
    let mut atoms = BTreeSet::new();
    let mut rejected = BTreeSet::new();
    for carrier in layouts.carriers() {
        let id = &carrier.type_id;
        if id.starts_with("mpk.csharp.value.") {
            if id == "mpk.csharp.value.unit.v1" || id == "mpk.csharp.value.exception.v1" {
                rejected.insert(id.clone());
            } else {
                atoms.insert(id.clone());
            }
        } else if roots
            .source_types
            .get(id)
            .is_some_and(|s| s.kind == SourceKind::Enum)
        {
            atoms.insert(id.clone());
        }
    }
    let ids = emitted
        .boundaries()
        .iter()
        .flat_map(|c| c.input_fields())
        .map(|f| {
            emitted
                .closure()
                .projections()
                .get(f.source_type_id())
                .map(String::as_str)
                .unwrap_or(f.source_type_id())
                .to_owned()
        })
        .collect::<BTreeSet<_>>();
    let mut compiler = Depths {
        b,
        carriers: layouts
            .carriers()
            .iter()
            .map(|c| (c.type_id.as_str(), c))
            .collect(),
        atoms,
        rejected,
        nodes: BTreeMap::new(),
        active: BTreeSet::new(),
    };
    let mut definitions = vec![];
    for id in ids {
        let n = compiler.ty(&id)?;
        definitions.push(OrdinaryJsonTypedDepthDefinition {
            carrier: (*compiler.carriers[id.as_str()]).clone(),
            definition: n.name,
        });
    }
    Ok(definitions)
}
pub fn generate_csharp_practical_ordinary_json_typed_depth(
    emitted: &EmittedDataPhase,
) -> R<OrdinaryJsonTypedDepthProgram> {
    let vir = emitted.vir();
    let layouts = generate_csharp_practical_ordinary_carriers(vir)?;
    let mut b = Builder::new()?;
    let definitions = emit(emitted, &layouts, &mut b)?;
    let static_transformers = b.static_transformers;
    let certificate = b.finish()?;
    let p = OrdinaryJsonTypedDepthProgram {
        schema: "mpk.csharp.ordinary_json_typed_depth.v1".into(),
        source_ir_sha256: vir.hash().into(),
        boundary_program_sha256: generate_boundary_vcs(vir)
            .map_err(|_| OrdinaryCarrierError::Linkage)?
            .hash(),
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
pub fn import_csharp_practical_ordinary_json_typed_depth(
    input: &[u8],
    certificate: &[u8],
    emitted: &EmittedDataPhase,
) -> R<OrdinaryJsonTypedDepthProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let p = generate_csharp_practical_ordinary_json_typed_depth(emitted)?;
    if input != p.canonical_bytes() || certificate != p.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(p)
}

#[cfg(test)]
mod tests {
    use super::super::super::super::test_eval::{bit, run, V};
    use super::*;
    fn word(n: u32) -> V {
        V::Cube((0..32).map(|i| n & (1 << i) != 0).collect())
    }
    fn field(id: &str, shape: OrdinaryShape) -> OrdinaryField {
        OrdinaryField {
            id: id.into(),
            shape,
        }
    }
    #[test]
    fn canonical_typed_depth_envelope_guard_masks_complete_packet() {
        let mut b = Builder::new().unwrap();
        let mut cases = vec![];
        for height in [31, 32] {
            let mut shape = OrdinaryShape::Bits { width: 1 };
            for _ in 0..height {
                shape = OrdinaryShape::Product {
                    fields: vec![field("child", shape)],
                };
            }
            let base = format!("Test.DepthGuard.H{height}");
            let mut compiler = Depths {
                b: &mut b,
                carriers: BTreeMap::new(),
                atoms: BTreeSet::new(),
                rejected: BTreeSet::new(),
                nodes: BTreeMap::new(),
                active: BTreeSet::new(),
            };
            let good = compiler
                .shape(&OrdinaryShape::Bits { width: 1 }, &format!("{base}.Scalar"))
                .unwrap();
            let deep = compiler.shape(&shape, &format!("{base}.Deep")).unwrap();
            let depths = vec![
                OrdinaryJsonTypedDepthDefinition {
                    carrier: OrdinaryCarrier {
                        type_id: "scalar".into(),
                        depth: 0,
                        shape: OrdinaryShape::Bits { width: 1 },
                    },
                    definition: good.name,
                },
                OrdinaryJsonTypedDepthDefinition {
                    carrier: OrdinaryCarrier {
                        type_id: "deep".into(),
                        depth: 0,
                        shape: shape.clone(),
                    },
                    definition: deep.name,
                },
            ];
            for valid in [false, true] {
                let name = format!("{base}.V{valid}");
                let header_name = circuit_with_block_bits(
                    &mut b,
                    &format!("{name}.Header"),
                    Circuit::new(&[]),
                    literal(if valid { 3 | (1u128 << 34) } else { 0 }, 128),
                    7,
                )
                .unwrap();
                let header = b.constant(&header_name).unwrap();
                let leaf = truth(&mut b, true).unwrap();
                let args = b.wrap_selectors(1, leaf).unwrap();
                let assemble = super::super::super::json_grammar::assemble(&mut b, 1).unwrap();
                let body = call(&mut b, &assemble, vec![header, args]).unwrap();
                let original = format!("{name}.Parse");
                define(&mut b, &original, &[24], 8, body).unwrap();
                let mut fields = vec![];
                for (index, id) in [(0, "scalar"), (1, "deep")] {
                    let get = format!("{name}.Get{index}");
                    let args = b.var(0).unwrap();
                    let selector = truth(&mut b, index == 1).unwrap();
                    let body = b.app(args, vec![selector]).unwrap();
                    define(&mut b, &get, &[1], 0, body).unwrap();
                    fields.push(OrdinaryJsonEnvelopeField {
                        field_id: id.into(),
                        semantic_type_id: id.into(),
                        argument_projection_definition: get,
                        step_definition: "unused-test-step".into(),
                    });
                }
                let mut definitions = vec![OrdinaryJsonEnvelopeDefinition {
                    contract_sha256: format!("test-{height}-{valid}"),
                    arguments_shape: OrdinaryShape::Product {
                        fields: vec![
                            field("scalar", OrdinaryShape::Bits { width: 1 }),
                            field("deep", shape.clone()),
                        ],
                    },
                    arguments_depth: 1,
                    fields,
                    parse_definition: original,
                    unguarded_parse_definition: None,
                    packet_depth: 8,
                    header_definition: super::super::super::json_values::projection(
                        &mut b, 8, 7, false,
                    )
                    .unwrap(),
                    value_definition: super::super::super::json_values::projection(
                        &mut b, 8, 1, true,
                    )
                    .unwrap(),
                }];
                super::super::envelopes::depth_guards(&mut b, &mut definitions, &depths).unwrap();
                cases.push((definitions.remove(0), valid && height == 31));
            }
        }
        let bytes = b.finish().unwrap();
        let cert = decode_canonical_certificate(&bytes).unwrap();
        for (d, valid) in cases {
            let result = run(
                &cert,
                &d.parse_definition,
                vec![super::super::super::super::test_eval::sparse_cube(
                    24,
                    BTreeSet::new(),
                )],
            );
            for index in 0..256 {
                let mut value = result.clone();
                for bit_index in 0..8 {
                    value = super::super::super::super::test_eval::apply(
                        &cert,
                        value,
                        V::Bit(index & (1 << bit_index) != 0),
                    );
                }
                let expected =
                    valid && (index == 0 || index == 2 || index == 68 || index == 1 || index == 3);
                assert_eq!(
                    bit(value),
                    expected,
                    "{} packet bit{index}",
                    d.contract_sha256
                );
            }
        }
        eprintln!("Canonical typed-depth envelope guard:1024 full-packet bits;field root1 and second-field failure,depth31/32,prior parse failure");
        if let Some(out) = std::env::var_os("MPK_W09_JSON_DEPTH_GUARD_HELPER_OUT") {
            let out = std::path::PathBuf::from(out);
            std::fs::create_dir_all(&out).unwrap();
            std::fs::write(
                out.join("packet-mask.hex"),
                bytes.iter().map(|v| format!("{v:02x}")).collect::<String>() + "\n",
            )
            .unwrap();
        }
    }
    #[test]
    fn canonical_typed_json_depth_active_nodes_and_overflow() {
        let mut b = Builder::new().unwrap();
        let mut d = Depths {
            b: &mut b,
            carriers: BTreeMap::new(),
            atoms: BTreeSet::new(),
            rejected: BTreeSet::new(),
            nodes: BTreeMap::new(),
            active: BTreeSet::new(),
        };
        let atom = OrdinaryShape::Bits { width: 32 };
        let atom_node = d.shape(&atom, "Test.Atom").unwrap();
        let empty = d
            .shape(&OrdinaryShape::Product { fields: vec![] }, "Test.Empty")
            .unwrap();
        let sum = OrdinaryShape::Sum {
            arms: vec![
                OrdinaryArm {
                    tag: 0,
                    id: "none".into(),
                    fields: vec![],
                },
                OrdinaryArm {
                    tag: 1,
                    id: "some".into(),
                    fields: vec![field(
                        "payload",
                        OrdinaryShape::Product {
                            fields: vec![field("x", atom)],
                        },
                    )],
                },
            ],
        };
        let s = d.shape(&sum, "Test.Sum").unwrap();
        let seq = d
            .shape(
                &OrdinaryShape::Sequence {
                    capacity: 8,
                    element: Box::new(sum),
                },
                "Test.Sequence",
            )
            .unwrap();
        assert_eq!(s.depth, 6);
        assert_eq!(seq.depth, 10);
        assert!(b.static_transformers <= 16384);
        let bytes = b.finish().unwrap();
        let cert = decode_canonical_certificate(&bytes).unwrap();
        let mut cases = 0;
        for depth in [0, 29, 30, 31, 32, 33, u32::MAX] {
            assert_eq!(
                bit(run(&cert, &atom_node.name, vec![word(0), word(depth)])),
                depth <= 32
            );
            assert_eq!(
                bit(run(&cert, &empty.name, vec![V::Bit(false), word(depth)])),
                depth <= 32
            );
            cases += 2;
            for tag in [0u32, 1, 2, u32::MAX] {
                let mut value = vec![false; 64];
                for i in 0..32 {
                    value[i << 1] = tag & (1 << i) != 0;
                }
                let expected = match tag {
                    0 => depth <= 31,
                    1 => depth <= 30,
                    _ => false,
                };
                assert_eq!(
                    bit(run(&cert, &s.name, vec![V::Cube(value), word(depth)])),
                    expected,
                    "tag{tag} depth{depth}"
                );
                cases += 1;
            }
            for len in 0..=9u32 {
                // Change only the last available tag:valid none, deeper some,
                // and invalid2. Tests inspect the whole active prefix.
                for last in [0u32, 1, 2] {
                    let mut value = vec![false; 1024];
                    for i in 0..32 {
                        value[i << 5] = len & (1 << i) != 0;
                    }
                    let index = len.saturating_sub(1).min(7) as usize;
                    for i in 0..32 {
                        let sum_address = i << 1;
                        value[1 | (index << 1) | (sum_address << 4)] = last & (1 << i) != 0;
                    }
                    // Place another invalid tag in an inactive element when possible.
                    if len < 8 {
                        value[1 | ((len as usize) << 1) | (2 << 4)] = true;
                    }
                    let expected = len <= 8
                        && if len == 0 {
                            depth <= 32
                        } else {
                            match last {
                                0 => depth <= 30,
                                1 => depth <= 29,
                                _ => false,
                            }
                        };
                    assert_eq!(
                        bit(run(&cert, &seq.name, vec![V::Cube(value), word(depth)])),
                        expected,
                        "length{len} last{last} depth{depth}"
                    );
                    cases += 1;
                }
            }
        }
        eprintln!("Canonical typed depth:{cases} actual core cases,including active/inactive storage and overflowing root depths");
        if let Some(out) = std::env::var_os("MPK_W09_JSON_TYPED_DEPTH_HELPER_OUT") {
            let out = std::path::PathBuf::from(out);
            std::fs::create_dir_all(&out).unwrap();
            std::fs::write(
                out.join("active-depth.hex"),
                bytes.iter().map(|x| format!("{x:02x}")).collect::<String>() + "\n",
            )
            .unwrap();
        }
    }
}
