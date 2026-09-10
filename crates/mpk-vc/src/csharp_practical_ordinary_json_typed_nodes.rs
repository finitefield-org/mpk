//! Canonical typed JSON node counts, conditional on valid semantic storage.
//! Names and UTF-16 units are not nodes; tags and implicit JSON containers are.
use super::super::integer_format::circuit_with_block_bits;
use super::super::temporal::literal;
use super::*;
const BASE: &str = "Mpk.CSharp.Ordinary.JsonTypedNodes";
const INVALID: u32 = 262_145;
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryJsonTypedNodeDefinition {
    pub carrier: OrdinaryCarrier,
    /// Semantic value -> C5 node count, saturated at262145.
    pub count_definition: String,
    pub valid_definition: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryJsonTypedNodeProgram {
    schema: String,
    source_ir_sha256: String,
    boundary_program_sha256: String,
    definitions: Vec<OrdinaryJsonTypedNodeDefinition>,
    static_transformers: usize,
    certificate_sha256: String,
    #[serde(skip)]
    certificate: Vec<u8>,
}
impl OrdinaryJsonTypedNodeProgram {
    pub fn definitions(&self) -> &[OrdinaryJsonTypedNodeDefinition] {
        &self.definitions
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("typed JSON node metadata")
    }
}
fn arithmetic(
    b: &mut Builder,
    suffix: &str,
    c: Circuit,
    out: Vec<usize>,
    inputs: usize,
) -> R<String> {
    let name = format!("{BASE}.{suffix}");
    if !b.globals.contains_key(&name) {
        // The output may be Boolean or a complete32-bit count.
        let depth = if out.len() == 1 { 0 } else { 5 };
        let f = circuit_with_block_bits(b, &name, c, out, 7)?;
        let args = (0..inputs)
            .rev()
            .map(|i| b.var(i as u32))
            .collect::<R<Vec<_>>>()?;
        let body = call(b, &f, args)?;
        define(b, &name, &vec![5; inputs], depth, body)?;
    }
    Ok(name)
}
fn sum_helper(b: &mut Builder, recombine: bool) -> R<String> {
    let suffix = if recombine { "Recombine" } else { "Add" };
    let name = format!("{BASE}.{suffix}");
    if b.globals.contains_key(&name) {
        return Ok(name);
    }
    let mut c = Circuit::new(&[32, 32]);
    // Recombination shifts in34 bits; arbitrary u32 operands cannot wrap.
    let mut left = Circuit::extend(&c.inputs[0], 35, false);
    if recombine {
        left.rotate_right(2);
    }
    let right = Circuit::extend(&c.inputs[1], 35, false);
    let sum = c.add(&left, &right, F).0;
    let below = c.lt(&sum, &literal(u128::from(INVALID), 35), false);
    let out = c.select(below, &sum[..32], &literal(u128::from(INVALID), 32));
    arithmetic(b, suffix, c, out, 2)
}
fn part(b: &mut Builder, quotient: bool) -> R<String> {
    let suffix = if quotient { "Quotient4" } else { "Remainder4" };
    let c = Circuit::new(&[32]);
    let mut out = vec![F; 32];
    if quotient {
        out[..30].copy_from_slice(&c.inputs[0][2..]);
    } else {
        out[..2].copy_from_slice(&c.inputs[0][..2]);
    }
    arithmetic(b, suffix, c, out, 1)
}
fn within(b: &mut Builder, maximum: u32) -> R<String> {
    let mut c = Circuit::new(&[32]);
    let input = c.inputs[0].clone();
    let out = c.lt(&input, &literal(u128::from(maximum) + 1, 32), false);
    arithmetic(b, &format!("Within{maximum}"), c, vec![out], 1)
}
fn add(b: &mut Builder, left: u32, right: u32) -> R<u32> {
    let helper = sum_helper(b, false)?;
    call(b, &helper, vec![left, right])
}
#[derive(Clone)]
struct Node {
    depth: u32,
    name: String,
}
struct Nodes<'a> {
    b: &'a mut Builder,
    carriers: BTreeMap<&'a str, &'a OrdinaryCarrier>,
    atoms: BTreeSet<String>,
    rejected: BTreeSet<String>,
    nodes: BTreeMap<String, Node>,
    active: BTreeSet<String>,
}
impl Nodes<'_> {
    fn finish(&mut self, name: &str, depth: u32, body: u32) -> R<Node> {
        define(self.b, name, &[depth], 5, body)?;
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
        let n = if self.rejected.contains(id) || self.atoms.contains(id) {
            let count = if self.rejected.contains(id) {
                INVALID
            } else {
                1
            };
            let body = word(self.b, count, 5)?;
            self.finish(&name, c.depth, body)?
        } else {
            self.shape(&c.shape, &name)?
        };
        if n.depth != c.depth {
            return Err(OrdinaryCarrierError::Shape);
        }
        self.active.remove(id);
        self.nodes.insert(id.into(), n.clone());
        Ok(n)
    }
    fn shape(&mut self, s: &OrdinaryShape, name: &str) -> R<Node> {
        match s {
            OrdinaryShape::Reference { type_id } => self.ty(type_id),
            OrdinaryShape::RoleBound { value, .. } => self.shape(value, name),
            OrdinaryShape::Bits { width } => {
                let body = word(self.b, 1, 5)?;
                self.finish(name, address_bits(*width), body)
            }
            OrdinaryShape::Product { fields } => {
                let children = fields
                    .iter()
                    .enumerate()
                    .map(|(i, f)| self.shape(&f.shape, &format!("{name}.F{i}")))
                    .collect::<R<Vec<_>>>()?;
                let roles = address_bits(fields.len() as u32);
                let max = children.iter().map(|c| c.depth).max().unwrap_or(0);
                let mut count = word(self.b, 1, 5)?;
                for (i, child) in children.iter().enumerate() {
                    let source = self.b.var(0)?;
                    let mut args = (0..roles)
                        .map(|j| truth(self.b, (i >> j) & 1 != 0))
                        .collect::<R<Vec<_>>>()?;
                    args.extend(vec![truth(self.b, false)?; (max - child.depth) as usize]);
                    let value = self.b.app(source, args)?;
                    let n = call(self.b, &child.name, vec![value])?;
                    count = add(self.b, count, n)?;
                }
                self.finish(name, roles + max, count)
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
                let mut result = word(self.b, INVALID, 5)?;
                for (arm, child) in arms.iter().zip(children) {
                    let mut count = word(self.b, 2, 5)?; // Object and explicit tag string.
                    if let Some(child) = child {
                        let source = self.b.var(0)?;
                        let mut args = vec![truth(self.b, true)?];
                        args.extend(vec![truth(self.b, false)?; (max - child.depth) as usize]);
                        let value = self.b.app(source, args)?;
                        let n = call(self.b, &child.name, vec![value])?;
                        count = add(self.b, count, n)?;
                    }
                    let source = self.b.var(5)?;
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
                        selected = call(self.b, "Std.Bool.and", vec![selected, bit])?;
                    }
                    result = sequences::mux(self.b, 5, selected, count, result)?;
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
                let source = self.b.var(5)?;
                let mut args = vec![truth(self.b, false)?; (depth - 5) as usize];
                args.extend(self.b.selectors(5)?);
                let length = self.b.app(source, args)?;
                let length = self.b.wrap_selectors(5, length)?;
                let cap = within(self.b, *capacity)?;
                let valid = call(self.b, &cap, vec![length])?;
                let fold = super::super::super::structural::emit_aggregate_fold(self.b, indices)?;
                let mut totals = vec![];
                // Sum quotients and residues separately using the existing65537
                // fold. At most16384 residues sum to49152, so that sum is exact.
                // A saturated quotient sum already forces the final count invalid.
                for quotient in [true, false] {
                    let source = self.b.var(indices)?;
                    let mut args = vec![truth(self.b, true)?];
                    args.extend(vec![
                        truth(self.b, false)?;
                        (payload - indices - child.depth) as usize
                    ]);
                    args.extend(self.b.selectors(indices)?);
                    let value = self.b.app(source, args)?;
                    let count = call(self.b, &child.name, vec![value])?;
                    let split = part(self.b, quotient)?;
                    let part = call(self.b, &split, vec![count])?;
                    let input = self.b.wrap_selectors(indices, part)?;
                    totals.push(call(self.b, &fold.sum_definition, vec![input, length])?);
                }
                let combine = sum_helper(self.b, true)?;
                let count = call(self.b, &combine, totals)?;
                let one = word(self.b, 1, 5)?;
                let count = add(self.b, one, count)?;
                let invalid = word(self.b, INVALID, 5)?;
                let count = sequences::mux(self.b, 5, valid, count, invalid)?;
                self.finish(name, depth, count)
            }
            OrdinaryShape::Array { .. } => Err(OrdinaryCarrierError::Shape),
        }
    }
}
pub(super) fn emit(
    emitted: &EmittedDataPhase,
    layouts: &OrdinaryCarrierProgram,
    b: &mut Builder,
) -> R<Vec<OrdinaryJsonTypedNodeDefinition>> {
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
    let mut compiler = Nodes {
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
        let source = compiler.b.var(0)?;
        let count = call(compiler.b, &n.name, vec![source])?;
        let valid = within(compiler.b, INVALID - 1)?;
        let body = call(compiler.b, &valid, vec![count])?;
        let valid_definition = format!("{}.Valid", n.name);
        define(compiler.b, &valid_definition, &[n.depth], 0, body)?;
        definitions.push(OrdinaryJsonTypedNodeDefinition {
            carrier: (*compiler.carriers[id.as_str()]).clone(),
            count_definition: n.name,
            valid_definition,
        });
    }
    Ok(definitions)
}
pub fn generate_csharp_practical_ordinary_json_typed_nodes(
    emitted: &EmittedDataPhase,
) -> R<OrdinaryJsonTypedNodeProgram> {
    let vir = emitted.vir();
    let layouts = generate_csharp_practical_ordinary_carriers(vir)?;
    let mut b = Builder::new()?;
    let definitions = emit(emitted, &layouts, &mut b)?;
    let static_transformers = b.static_transformers;
    let certificate = b.finish()?;
    let p = OrdinaryJsonTypedNodeProgram {
        schema: "mpk.csharp.ordinary_json_typed_nodes.v1".into(),
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
pub fn import_csharp_practical_ordinary_json_typed_nodes(
    input: &[u8],
    certificate: &[u8],
    emitted: &EmittedDataPhase,
) -> R<OrdinaryJsonTypedNodeProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let p = generate_csharp_practical_ordinary_json_typed_nodes(emitted)?;
    if input != p.canonical_bytes() || certificate != p.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(p)
}

#[cfg(test)]
mod tests {
    use super::super::super::super::test_eval::{apply, bit, run, V};
    use super::*;
    fn w(n: u32) -> V {
        V::Cube((0..32).map(|i| n & (1 << i) != 0).collect())
    }
    fn number(c: &Certificate, value: V) -> u32 {
        (0..32).fold(0, |n, i| {
            let mut v = value.clone();
            for k in 0..5 {
                v = apply(c, v, V::Bit(i & (1 << k) != 0));
            }
            n | ((bit(v) as u32) << i)
        })
    }
    #[test]
    fn canonical_typed_node_guard_masks_complete_packet() {
        let mut b = Builder::new().unwrap();
        let valid_definition = within(&mut b, INVALID - 1).unwrap();
        let identity = b.var(0).unwrap();
        define(&mut b, "Test.CountProxy", &[5], 5, identity).unwrap();
        let carrier = OrdinaryCarrier {
            type_id: "count-proxy".into(),
            depth: 5,
            shape: OrdinaryShape::Bits { width: 32 },
        };
        let nodes = vec![OrdinaryJsonTypedNodeDefinition {
            carrier: carrier.clone(),
            count_definition: "Test.CountProxy".into(),
            valid_definition,
        }];
        let mut cases = vec![];
        for count in [1u32, 262_143, 262_144, 262_145, u32::MAX] {
            for parse_valid in [false, true] {
                let name = format!("Test.NodeGuard.N{count}.V{parse_valid}");
                // Packet layout uses role first, then active value selectors,
                // then trailing padding. Keep nonzero payload on a failed
                // synthetic parse to test that the guard itself checks validity.
                let mut packet = vec![F; 256];
                if parse_valid {
                    for index in [0, 2, 68] {
                        packet[index] = T;
                    }
                }
                packet[1] = T;
                for i in 0..32 {
                    packet[3 | (i << 2)] = if count & (1 << i) != 0 { T } else { F };
                }
                let packet = circuit_with_block_bits(
                    &mut b,
                    &format!("{name}.Packet"),
                    Circuit::new(&[]),
                    packet,
                    7,
                )
                .unwrap();
                let body = b.constant(&packet).unwrap();
                let original = format!("{name}.Parse");
                define(&mut b, &original, &[24], 8, body).unwrap();
                let mut fields = vec![];
                for index in 0..2 {
                    let get = format!("{name}.Get{index}");
                    let args = b.var(0).unwrap();
                    let index_bit = truth(&mut b, index == 1).unwrap();
                    let body = b.app(args, vec![index_bit]).unwrap();
                    define(&mut b, &get, &[6], 5, body).unwrap();
                    fields.push(OrdinaryJsonEnvelopeField {
                        field_id: format!("field{index}"),
                        semantic_type_id: carrier.type_id.clone(),
                        argument_projection_definition: get,
                        step_definition: "unused-test-step".into(),
                    });
                }
                let mut definitions = vec![OrdinaryJsonEnvelopeDefinition {
                    contract_sha256: name,
                    arguments_shape: OrdinaryShape::Product {
                        fields: (0..2)
                            .map(|i| OrdinaryField {
                                id: format!("field{i}"),
                                shape: OrdinaryShape::Reference {
                                    type_id: carrier.type_id.clone(),
                                },
                            })
                            .collect(),
                    },
                    arguments_depth: 6,
                    fields,
                    parse_definition: original.clone(),
                    unguarded_parse_definition: Some(original.clone()),
                    packet_depth: 8,
                    header_definition: super::super::super::json_values::projection(
                        &mut b, 8, 7, false,
                    )
                    .unwrap(),
                    value_definition: super::super::super::json_values::projection(
                        &mut b, 8, 6, true,
                    )
                    .unwrap(),
                }];
                super::super::envelopes::node_guards(&mut b, &mut definitions, &nodes).unwrap();
                assert_eq!(definitions[0].unguarded_parse_definition, Some(original));
                cases.push((definitions.remove(0), count, parse_valid && count < INVALID));
            }
        }
        let bytes = b.finish().unwrap();
        let cert = decode_canonical_certificate(&bytes).unwrap();
        for (d, count, valid) in cases {
            let packet = run(
                &cert,
                &d.parse_definition,
                vec![super::super::super::super::test_eval::sparse_cube(
                    24,
                    BTreeSet::new(),
                )],
            );
            for index in 0..256 {
                let mut value = packet.clone();
                for i in 0..8 {
                    value = apply(&cert, value, V::Bit(index & (1 << i) != 0));
                }
                let mut expected = matches!(index, 0 | 2 | 68 | 1);
                for i in 0..32 {
                    expected |= index == (3 | (i << 2)) && count & (1 << i) != 0;
                }
                assert_eq!(
                    bit(value),
                    valid && expected,
                    "{} bit{index}",
                    d.contract_sha256
                );
            }
        }
        eprintln!("Typed node envelope guard:2560 complete-packet bits;inclusive upper bound,second-field failure,prior parse failure");
        if let Some(out) = std::env::var_os("MPK_W09_JSON_TYPED_GUARD_HELPER_OUT") {
            let out = std::path::PathBuf::from(out);
            std::fs::create_dir_all(&out).unwrap();
            std::fs::write(
                out.join("node-packet-mask.hex"),
                bytes.iter().map(|v| format!("{v:02x}")).collect::<String>() + "\n",
            )
            .unwrap();
        }
    }
    #[test]
    fn canonical_typed_json_nodes_saturation_and_active_containers() {
        let mut b = Builder::new().unwrap();
        let add = sum_helper(&mut b, false).unwrap();
        let combine = sum_helper(&mut b, true).unwrap();
        let valid = within(&mut b, INVALID - 1).unwrap();
        let identity = b.var(0).unwrap();
        define(&mut b, "Test.CountIdentity", &[5], 5, identity).unwrap();
        let mut d = Nodes {
            b: &mut b,
            carriers: BTreeMap::new(),
            atoms: BTreeSet::new(),
            rejected: BTreeSet::new(),
            nodes: BTreeMap::from([(
                "count".into(),
                Node {
                    depth: 5,
                    name: "Test.CountIdentity".into(),
                },
            )]),
            active: BTreeSet::new(),
        };
        let seq = d
            .shape(
                &OrdinaryShape::Sequence {
                    capacity: 8,
                    element: Box::new(OrdinaryShape::Reference {
                        type_id: "count".into(),
                    }),
                },
                "Test.CountSequence",
            )
            .unwrap();
        let sum = d
            .shape(
                &OrdinaryShape::Sum {
                    arms: vec![
                        OrdinaryArm {
                            tag: 0,
                            id: "none".into(),
                            fields: vec![],
                        },
                        OrdinaryArm {
                            tag: 1,
                            id: "some".into(),
                            fields: vec![OrdinaryField {
                                id: "payload".into(),
                                shape: OrdinaryShape::Product { fields: vec![] },
                            }],
                        },
                    ],
                },
                "Test.NodeSum",
            )
            .unwrap();
        let empty = d
            .shape(&OrdinaryShape::Product { fields: vec![] }, "Test.Empty")
            .unwrap();
        let bytes = b.finish().unwrap();
        let cert = decode_canonical_certificate(&bytes).unwrap();
        let mut cases = 0;
        for x in [
            0u32,
            1,
            2,
            3,
            49_152,
            65_536,
            65_537,
            262_143,
            262_144,
            262_145,
            u32::MAX,
        ] {
            for y in [
                0u32,
                1,
                2,
                3,
                49_152,
                65_536,
                65_537,
                262_143,
                262_144,
                262_145,
                u32::MAX,
            ] {
                for (name, scale) in [(&add, 1u64), (&combine, 4)] {
                    let expected =
                        (u64::from(x) * scale + u64::from(y)).min(u64::from(INVALID)) as u32;
                    assert_eq!(
                        number(&cert, run(&cert, name, vec![w(x), w(y)])),
                        expected,
                        "{name} {x} {y}"
                    );
                    cases += 1;
                }
            }
            assert_eq!(bit(run(&cert, &valid, vec![w(x)])), x < INVALID);
            cases += 1;
        }
        assert_eq!(
            number(&cert, run(&cert, &empty.name, vec![V::Bit(false)])),
            1
        );
        cases += 1;
        for tag in [0u32, 1, 2, u32::MAX] {
            let mut value = vec![true; 64];
            for i in 0..32 {
                value[i << 1] = tag & (1 << i) != 0;
            }
            assert_eq!(
                number(&cert, run(&cert, &sum.name, vec![V::Cube(value)])),
                match tag {
                    0 => 2,
                    1 => 3,
                    _ => INVALID,
                }
            );
            cases += 1;
        }
        for values in [
            vec![],
            vec![0],
            vec![1, 2, 3],
            vec![3; 8],
            vec![262_143],
            vec![262_144],
            vec![262_145],
            vec![131_071, 131_072],
            vec![u32::MAX],
            vec![1; 9],
        ] {
            let mut stored = vec![true; 1 << seq.depth]; // Inactive words must not contribute.
            let length = values.len() as u32;
            for i in 0..32 {
                stored[i << 4] = length & (1 << i) != 0;
            }
            for (j, n) in values.iter().take(8).enumerate() {
                for i in 0..32 {
                    stored[1 | (j << 1) | (i << 4)] = n & (1 << i) != 0;
                }
            }
            let expected = if values.len() > 8 {
                INVALID
            } else {
                (1 + values.iter().map(|n| u64::from(*n)).sum::<u64>()).min(u64::from(INVALID))
                    as u32
            };
            assert_eq!(
                number(&cert, run(&cert, &seq.name, vec![V::Cube(stored)])),
                expected,
                "{values:?}"
            );
            cases += 1;
        }
        eprintln!("Typed JSON node counts:{cases} core arithmetic/container cases");
        if let Some(out) = std::env::var_os("MPK_W09_JSON_TYPED_NODES_HELPER_OUT") {
            let out = std::path::PathBuf::from(out);
            std::fs::create_dir_all(&out).unwrap();
            std::fs::write(
                out.join("node-counts.hex"),
                bytes.iter().map(|v| format!("{v:02x}")).collect::<String>() + "\n",
            )
            .unwrap();
        }
    }
}
