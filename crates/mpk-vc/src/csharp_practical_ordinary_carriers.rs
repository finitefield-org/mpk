//! W09 internal unit 1: concrete Boolean-cube carriers and checked core helpers.
//! Storage zero is not a public default or an invariant/proof discharge.
use super::*;
use crate::csharp_practical_vir_validation::ValidatedPracticalVir;
use mpk_cert::encode::{
    Certificate, CertificateHashes, Declaration, DeclarationKind, DefinitionReducibility,
    LevelNode, TermNode,
};
use mpk_cert::{
    axiom_report_hash_for_report, build_axiom_report, build_export_block,
    decode_canonical_certificate, encode_certificate_bounded, export_block_hash,
};
const BOOL: &str = "Std.Bool";
const PREFIX: &str = "Mpk.CSharp.Ordinary";
const BOOL_HEX: &[u8] = include_bytes!("../../../proofs/std/bool/std-bool.hex");
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrdinaryCarrierError {
    Linkage,
    Shape,
    Cycle,
    Limit,
}
type R<T> = Result<T, OrdinaryCarrierError>;
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum OrdinaryShape {
    Bits {
        width: u32,
    },
    RoleBound {
        maximum: u32,
        value: Box<OrdinaryShape>,
    },
    Reference {
        type_id: String,
    },
    Product {
        fields: Vec<OrdinaryField>,
    },
    Sum {
        arms: Vec<OrdinaryArm>,
    },
    Array {
        capacity: u32,
        element: Box<OrdinaryShape>,
    },
    Sequence {
        capacity: u32,
        element: Box<OrdinaryShape>,
    },
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryField {
    pub id: String,
    pub shape: OrdinaryShape,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryArm {
    pub tag: u32,
    pub id: String,
    pub fields: Vec<OrdinaryField>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryCarrier {
    pub type_id: String,
    pub depth: u32,
    pub shape: OrdinaryShape,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryCarrierProgram {
    schema: String,
    source_ir_sha256: String,
    foundation_sha256: String,
    carriers: Vec<OrdinaryCarrier>,
    certificate_sha256: String,
    #[serde(skip)]
    certificate: Vec<u8>,
}
impl OrdinaryCarrierProgram {
    pub fn carriers(&self) -> &[OrdinaryCarrier] {
        &self.carriers
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("typed carrier program")
    }
}
fn bits(width: u32) -> OrdinaryShape {
    OrdinaryShape::Bits { width }
}
fn reference(id: &str) -> OrdinaryShape {
    OrdinaryShape::Reference { type_id: id.into() }
}
fn field(id: &str, shape: OrdinaryShape) -> OrdinaryField {
    OrdinaryField {
        id: id.into(),
        shape,
    }
}
fn text<'a>(v: &'a Value, k: &str) -> R<&'a str> {
    v.get(k)
        .and_then(Value::as_str)
        .ok_or(OrdinaryCarrierError::Shape)
}
fn array<'a>(v: &'a Value, k: &str) -> R<&'a [Value]> {
    v.get(k)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .ok_or(OrdinaryCarrierError::Shape)
}
fn product(fields: Vec<OrdinaryField>) -> OrdinaryShape {
    OrdinaryShape::Product { fields }
}
fn sequence(capacity: u32, element: OrdinaryShape) -> OrdinaryShape {
    OrdinaryShape::Sequence {
        capacity,
        element: Box::new(element),
    }
}
fn representation(v: &Value) -> R<OrdinaryShape> {
    Ok(match text(v, "kind")? {
        "concrete" => reference(text(v, "type_id")?),
        "product" => product(
            array(v, "fields")?
                .iter()
                .map(|f| {
                    Ok(field(
                        f[0].as_str().ok_or(OrdinaryCarrierError::Shape)?,
                        representation(&f[1])?,
                    ))
                })
                .collect::<R<_>>()?,
        ),
        "sum" => OrdinaryShape::Sum {
            arms: array(v, "arms")?
                .iter()
                .enumerate()
                .map(|(i, a)| {
                    Ok(OrdinaryArm {
                        tag: i as u32,
                        id: a[0].as_str().ok_or(OrdinaryCarrierError::Shape)?.into(),
                        fields: a[1]
                            .as_array()
                            .ok_or(OrdinaryCarrierError::Shape)?
                            .iter()
                            .enumerate()
                            .map(|(j, t)| Ok(field(&j.to_string(), representation(t)?)))
                            .collect::<R<_>>()?,
                    })
                })
                .collect::<R<_>>()?,
        },
        "sequence" | "ordered_set" => sequence(4096, representation(&v["element"])?),
        "ordered_map" => sequence(
            4096,
            product(vec![
                field("key", representation(&v["key"])?),
                field("value", representation(&v["value"])?),
            ]),
        ),
        "construction" => product(vec![
            field("length", bits(32)),
            field(
                "cells",
                OrdinaryShape::Array {
                    capacity: 16384,
                    element: Box::new(representation(&v["element"])?),
                },
            ),
            field(
                "initialized",
                OrdinaryShape::Array {
                    capacity: 16384,
                    element: Box::new(bits(1)),
                },
            ),
        ]),
        _ => return Err(OrdinaryCarrierError::Shape),
    })
}
fn scalar_width(name: &str) -> Option<u32> {
    Some(match name {
        "unit" => 0,
        "bool" => 1,
        "i8" | "u8" => 8,
        "i16" | "u16" | "char" => 16,
        "i32" | "u32" | "f32" | "date" | "day_of_week" | "parse_error" => 32,
        "i64" | "u64" | "f64" | "time" | "duration" | "instant" => 64,
        "guid" => 128,
        _ => return None,
    })
}
fn primitive(name: &str, vir: &ValidatedPracticalVir) -> R<OrdinaryShape> {
    if let Some(w) = scalar_width(name) {
        return Ok(bits(w));
    }
    Ok(match name {
        // Explicit fields rather than CLR decimal object/storage layout.
        "decimal" => product(vec![
            field("negative", bits(1)),
            field("scale", bits(8)),
            field("coefficient", bits(96)),
        ]),
        "string" => sequence(16384, bits(16)),
        "exception" => {
            let (_, r, _) = vir.construction_context();
            let u = derive_closed_exception_universe(r, vir.data_closed(), vir.source_exceptions())
                .map_err(|_| OrdinaryCarrierError::Shape)?;
            OrdinaryShape::Sum {
                arms: u
                    .arms()
                    .iter()
                    .map(|a| OrdinaryArm {
                        tag: a.tag,
                        id: a.type_id.clone(),
                        fields: a
                            .payload_member_ids
                            .iter()
                            .zip(&a.payload_type_ids)
                            .map(|(id, t)| field(id, reference(t)))
                            .collect(),
                    })
                    .collect(),
            }
        }
        _ => return Err(OrdinaryCarrierError::Shape),
    })
}
struct Layouts<'a> {
    vir: &'a ValidatedPracticalVir,
    entries: BTreeMap<String, OrdinaryCarrier>,
    active: BTreeSet<String>,
    lifts: BTreeSet<(u32, u32)>,
}
fn address_bits(n: u32) -> u32 {
    if n <= 1 {
        0
    } else {
        32 - (n - 1).leading_zeros()
    }
}
impl Layouts<'_> {
    fn add(&mut self, id: &str) -> R<u32> {
        if let Some(c) = self.entries.get(id) {
            return Ok(c.depth);
        }
        if !self.active.insert(id.into()) {
            return Err(OrdinaryCarrierError::Cycle);
        }
        let (b, r, _) = self.vir.construction_context();
        let shape = if let Some(s) = r.source_types.get(id) {
            if s.kind == SourceKind::Enum {
                bits(
                    scalar_width(
                        s.enum_underlying
                            .as_deref()
                            .ok_or(OrdinaryCarrierError::Shape)?,
                    )
                    .ok_or(OrdinaryCarrierError::Shape)?,
                )
            } else {
                product(
                    s.members
                        .iter()
                        .map(|m| {
                            Ok(field(
                                &m.id,
                                reference(
                                    &closed_type_id(b, &m.ty)
                                        .map_err(|_| OrdinaryCarrierError::Shape)?,
                                ),
                            ))
                        })
                        .collect::<R<_>>()?,
                )
            }
        } else if let Some(e) = self
            .vir
            .data_closed()
            .entries()
            .iter()
            .find(|e| e["instance_id"] == id)
        {
            let mut shape = representation(&e["type_definition"]["representation"])?;
            if e["template_id"] == "mpk.csharp.semantic.validation.v1" {
                let OrdinaryShape::Sum { arms } = &mut shape else {
                    return Err(OrdinaryCarrierError::Shape);
                };
                let payload = &mut arms
                    .get_mut(1)
                    .ok_or(OrdinaryCarrierError::Shape)?
                    .fields
                    .get_mut(0)
                    .ok_or(OrdinaryCarrierError::Shape)?
                    .shape;
                *payload = OrdinaryShape::RoleBound {
                    maximum: 256,
                    value: Box::new(payload.clone()),
                };
            }
            shape
        } else if let Some(p) = id
            .strip_prefix("mpk.csharp.value.")
            .and_then(|p| p.strip_suffix(".v1"))
        {
            primitive(p, self.vir)?
        } else {
            return Err(OrdinaryCarrierError::Shape);
        };
        let depth = self.depth(&shape)?;
        if depth + 3 > 256 {
            return Err(OrdinaryCarrierError::Limit);
        }
        self.entries.insert(
            id.into(),
            OrdinaryCarrier {
                type_id: id.into(),
                depth,
                shape,
            },
        );
        self.active.remove(id);
        Ok(depth)
    }
    fn product_depth(&mut self, fields: &[OrdinaryField]) -> R<u32> {
        let ds = fields
            .iter()
            .map(|f| self.depth(&f.shape))
            .collect::<R<Vec<_>>>()?;
        let max = ds.iter().copied().max().unwrap_or(0);
        for d in ds {
            self.lifts.insert((d, max));
        }
        Ok(address_bits(fields.len() as u32) + max)
    }
    fn depth(&mut self, s: &OrdinaryShape) -> R<u32> {
        let d = match s {
            OrdinaryShape::Bits { width } => address_bits(*width),
            OrdinaryShape::Reference { type_id } => self.add(type_id)?,
            OrdinaryShape::RoleBound { value, .. } => self.depth(value)?,
            OrdinaryShape::Product { fields } => self.product_depth(fields)?,
            OrdinaryShape::Array { capacity, element } => {
                address_bits(*capacity) + self.depth(element)?
            }
            OrdinaryShape::Sequence { capacity, element } => {
                let e = address_bits(*capacity) + self.depth(element)?;
                let max = 5.max(e);
                self.lifts.insert((5, max));
                self.lifts.insert((e, max));
                1 + max
            }
            OrdinaryShape::Sum { arms } => {
                let mut ds = vec![];
                for a in arms {
                    ds.push(self.product_depth(&a.fields)?);
                }
                let payload = ds.iter().copied().max().unwrap_or(0);
                for d in ds {
                    self.lifts.insert((d, payload));
                }
                let max = 5.max(payload);
                self.lifts.insert((5, max));
                self.lifts.insert((payload, max));
                1 + max
            }
        };
        if d > 253 {
            Err(OrdinaryCarrierError::Limit)
        } else {
            Ok(d)
        }
    }
}
/// Reconstruct storage layouts from the independently validated original VIR.
/// The returned certificate checks only carrier/helper definitions, not W09's
/// still-separate operation semantics, application invariants or VC theorems.
pub fn generate_csharp_practical_ordinary_carriers(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryCarrierProgram> {
    let mut l = Layouts {
        vir,
        entries: BTreeMap::new(),
        active: BTreeSet::new(),
        lifts: BTreeSet::new(),
    };
    for t in crate::csharp_practical_vc_model::build_type_encodings(vir)
        .map_err(|_| OrdinaryCarrierError::Linkage)?
    {
        l.add(t.type_id())?;
    }
    for e in vir.data_closed().entries() {
        l.add(text(e, "instance_id")?)?;
    }
    let mut builder = Builder::new()?;
    let mut depths = l.entries.values().map(|c| c.depth).collect::<BTreeSet<_>>();
    for (a, b) in &l.lifts {
        depths.extend([*a, *b]);
    }
    for d in depths {
        builder.helpers(d)?;
    }
    for (a, b) in l.lifts {
        builder.lift(a, b)?;
    }
    for c in l.entries.values() {
        let ty = builder.cube(c.depth)?;
        let encoded_id = c
            .type_id
            .as_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>();
        builder.define(&format!("{PREFIX}.Type.T{encoded_id}"), builder.sort, ty)?;
    }
    let certificate = builder.finish()?;
    let program = OrdinaryCarrierProgram {
        schema: "mpk.csharp.ordinary_carriers.v1".into(),
        source_ir_sha256: vir.hash().into(),
        foundation_sha256: vir.construction_context().0.content_sha256().into(),
        carriers: l.entries.into_values().collect(),
        certificate_sha256: mpk_cert::hash_hex(&mpk_cert::certificate_hash(&certificate)),
        certificate,
    };
    if program.canonical_bytes().len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    Ok(program)
}
pub fn import_csharp_practical_ordinary_carriers(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryCarrierProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let expected = generate_csharp_practical_ordinary_carriers(vir)?;
    if input != expected.canonical_bytes() || certificate != expected.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(expected)
}
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
enum Key {
    Sort(u32),
    Var(u32),
    Const(u32, Vec<u32>),
    App(u32, Vec<u32>),
    Lam(u32, u32),
    Pi(u32, u32),
    Let(u32, u32, u32),
}
fn key(n: &TermNode) -> Key {
    match n {
        TermNode::Sort(x) => Key::Sort(*x),
        TermNode::Var(x) => Key::Var(*x),
        TermNode::Const { global, levels } => Key::Const(*global, levels.clone()),
        TermNode::App {
            function,
            arguments,
        } => Key::App(*function, arguments.clone()),
        TermNode::Lam { ty, body } => Key::Lam(*ty, *body),
        TermNode::Pi { ty, body } => Key::Pi(*ty, *body),
        TermNode::Let { ty, value, body } => Key::Let(*ty, *value, *body),
    }
}
struct Builder {
    c: Certificate,
    terms: BTreeMap<Key, u32>,
    globals: BTreeMap<String, u32>,
    binders: Vec<u32>,
    boolean: u32,
    sort: u32,
    static_transformers: usize,
    shared_scalar_circuits: BTreeMap<Vec<u8>, OrdinaryScalarDefinition>,
}
impl Builder {
    fn new() -> R<Self> {
        use sha2::{Digest, Sha256};
        // Frozen predecessor bytes are part of this generator's trust boundary.
        if format!("{:x}", Sha256::digest(BOOL_HEX))
            != "88a37f9df68a18bc19d51c0832279ff97a2944fc1676326022269766933ee806"
        {
            return Err(OrdinaryCarrierError::Linkage);
        }
        let h = std::str::from_utf8(BOOL_HEX)
            .map_err(|_| OrdinaryCarrierError::Linkage)?
            .split_whitespace()
            .collect::<String>();
        let bytes = (0..h.len())
            .step_by(2)
            .map(|i| {
                u8::from_str_radix(&h[i..i + 2], 16).map_err(|_| OrdinaryCarrierError::Linkage)
            })
            .collect::<R<Vec<_>>>()?;
        let mut c =
            decode_canonical_certificate(&bytes).map_err(|_| OrdinaryCarrierError::Linkage)?;
        let globals = c
            .declarations
            .iter()
            .enumerate()
            .map(|(i, d)| (c.name_table[d.name as usize].clone(), i as u32))
            .collect::<BTreeMap<_, _>>();
        let DeclarationKind::Inductive { ty: sort } =
            c.declarations[*globals.get(BOOL).ok_or(OrdinaryCarrierError::Linkage)? as usize].kind
        else {
            return Err(OrdinaryCarrierError::Linkage);
        };
        c.module = "Mpk.CSharp.Ordinary.Carriers".into();
        c.source_manifest = None;
        c.export_block.clear();
        c.hashes = CertificateHashes::default();
        let initial = std::mem::take(&mut c.term_table);
        let mut b = Self {
            c,
            terms: BTreeMap::new(),
            globals,
            binders: vec![],
            boolean: 0,
            sort,
            static_transformers: 0,
            shared_scalar_circuits: BTreeMap::new(),
        };
        // Preserve frozen global term IDs even if the foundation contains duplicates.
        for n in initial {
            let d = b.depth(&n)?;
            let i = b.c.term_table.len() as u32;
            b.terms.entry(key(&n)).or_insert(i);
            b.binders.push(d);
            b.c.term_table.push(n);
        }
        b.boolean = b.constant(BOOL)?;
        Ok(b)
    }
    fn depth(&self, n: &TermNode) -> R<u32> {
        let d = |i: &u32| {
            self.binders
                .get(*i as usize)
                .copied()
                .ok_or(OrdinaryCarrierError::Shape)
        };
        Ok(match n {
            TermNode::Sort(_) | TermNode::Var(_) | TermNode::Const { .. } => 0,
            TermNode::App {
                function,
                arguments,
            } => {
                let mut x = d(function)?;
                for a in arguments {
                    x = x.max(d(a)?)
                }
                x
            }
            TermNode::Lam { ty, body } | TermNode::Pi { ty, body } => d(ty)?.max(d(body)? + 1),
            TermNode::Let { ty, value, body } => d(ty)?.max(d(value)?).max(d(body)? + 1),
        })
    }
    fn term(&mut self, n: TermNode) -> R<u32> {
        let k = key(&n);
        if let Some(i) = self.terms.get(&k) {
            return Ok(*i);
        }
        let d = self.depth(&n)?;
        if d > 256 || self.c.term_table.len() >= 262144 {
            return Err(OrdinaryCarrierError::Limit);
        }
        let i = self.c.term_table.len() as u32;
        self.terms.insert(k, i);
        self.binders.push(d);
        self.c.term_table.push(n);
        Ok(i)
    }
    fn constant(&mut self, n: &str) -> R<u32> {
        self.term(TermNode::Const {
            global: *self.globals.get(n).ok_or(OrdinaryCarrierError::Linkage)?,
            levels: vec![],
        })
    }
    fn var(&mut self, i: u32) -> R<u32> {
        self.term(TermNode::Var(i))
    }
    fn app(&mut self, f: u32, a: Vec<u32>) -> R<u32> {
        if a.is_empty() {
            Ok(f)
        } else {
            self.term(TermNode::App {
                function: f,
                arguments: a,
            })
        }
    }
    fn lam(&mut self, t: u32, b: u32) -> R<u32> {
        self.term(TermNode::Lam { ty: t, body: b })
    }
    fn pi(&mut self, t: u32, b: u32) -> R<u32> {
        self.term(TermNode::Pi { ty: t, body: b })
    }
    fn cube(&mut self, d: u32) -> R<u32> {
        let mut t = self.boolean;
        for _ in 0..d {
            t = self.pi(self.boolean, t)?
        }
        Ok(t)
    }
    fn define(&mut self, n: &str, ty: u32, value: u32) -> R<()> {
        if self.globals.contains_key(n) {
            return Err(OrdinaryCarrierError::Linkage);
        }
        if self.c.declarations.len() >= 8192 {
            return Err(OrdinaryCarrierError::Limit);
        }
        let name = self.c.name_table.len() as u32;
        self.c.name_table.push(n.into());
        self.globals
            .insert(n.into(), self.c.declarations.len() as u32);
        self.c.declarations.push(Declaration {
            name,
            kind: DeclarationKind::Def {
                ty,
                value,
                reducibility: DefinitionReducibility::Reducible,
            },
        });
        Ok(())
    }
    fn selectors(&mut self, d: u32) -> R<Vec<u32>> {
        (0..d).rev().map(|i| self.var(i)).collect()
    }
    fn wrap_selectors(&mut self, d: u32, mut t: u32) -> R<u32> {
        for _ in 0..d {
            t = self.lam(self.boolean, t)?
        }
        Ok(t)
    }
    fn helpers(&mut self, d: u32) -> R<()> {
        let s = self.cube(d)?;
        let f = self.constant("Std.Bool.false")?;
        let z = self.wrap_selectors(d, f)?;
        self.define(&format!("{PREFIX}.Cube.D{d}.Zero"), s, z)?;
        // c, then, else, followed by least-significant-first address binders.
        let a = self.selectors(d)?;
        let e = self.var(d)?;
        let e = self.app(e, a.clone())?;
        let t = self.var(d + 1)?;
        let t = self.app(t, a)?;
        let c = self.var(d + 2)?;
        let rec = self.constant("Std.Bool.rec")?;
        let leaf = self.app(rec, vec![e, t, c])?;
        let body = self.wrap_selectors(d, leaf)?;
        let body = self.lam(s, body)?;
        let body = self.lam(s, body)?;
        let body = self.lam(self.boolean, body)?;
        let ty = self.pi(s, s)?;
        let ty = self.pi(s, ty)?;
        let ty = self.pi(self.boolean, ty)?;
        self.define(&format!("{PREFIX}.Cube.D{d}.Mux"), ty, body)?;
        // Ordered composition: f then g. No recursor returns a function carrier.
        let ss = self.pi(s, s)?;
        let f = self.var(2)?;
        let x = self.var(0)?;
        let first = self.app(f, vec![x])?;
        let g = self.var(2)?;
        let y = self.var(0)?;
        let second = self.app(g, vec![y])?;
        let body = self.term(TermNode::Let {
            ty: s,
            value: first,
            body: second,
        })?;
        let body = self.lam(s, body)?;
        let body = self.lam(ss, body)?;
        let body = self.lam(ss, body)?;
        let ty = self.pi(ss, ss)?;
        let ty = self.pi(ss, ty)?;
        self.define(&format!("{PREFIX}.Cube.D{d}.Compose"), ty, body)?;
        let identity = self.compose(d, &[])?;
        self.define(&format!("{PREFIX}.Cube.D{d}.Identity"), ss, identity)
    }
    // Inputs are concrete S -> S terms in their surrounding value-binder
    // context; certificate declarations close those binders. Count occurrences before
    // DAG sharing; repeated equal leaves cannot evade the expansion budget.
    // A balanced tree preserves left-to-right state flow without linear depth.
    fn compose(&mut self, d: u32, steps: &[u32]) -> R<u32> {
        self.compose_using(d, &format!("{PREFIX}.Cube.D{d}.Compose"), steps)
    }
    // The caller supplies an ordinary, concrete (S->S)->(S->S)->S->S
    // definition. Counting and balanced expansion are identical for guarded
    // and unconditional composition; sharing cannot remove occurrences.
    fn compose_using(&mut self, d: u32, name: &str, steps: &[u32]) -> R<u32> {
        // Keep the predecessor term insertion order, including the empty case.
        self.compose_in_context(d, Some(name), None, steps)
    }
    fn compose_term(&mut self, d: u32, composition: u32, steps: &[u32]) -> R<u32> {
        self.compose_in_context(d, None, Some(composition), steps)
    }
    fn compose_in_context(
        &mut self,
        d: u32,
        name: Option<&str>,
        composition: Option<u32>,
        steps: &[u32],
    ) -> R<u32> {
        let count = self
            .static_transformers
            .checked_add(steps.len())
            .ok_or(OrdinaryCarrierError::Limit)?;
        if count > crate::csharp_practical_vc_model::STATIC_TRANSFORMERS_MAX as usize {
            return Err(OrdinaryCarrierError::Limit);
        }
        self.static_transformers = count;
        fn tree(b: &mut Builder, compose: u32, steps: &[u32]) -> R<u32> {
            if steps.len() == 1 {
                return Ok(steps[0]);
            }
            let mid = steps.len() / 2;
            let left = tree(b, compose, &steps[..mid])?;
            let right = tree(b, compose, &steps[mid..])?;
            b.app(compose, vec![left, right])
        }
        if steps.is_empty() {
            let s = self.cube(d)?;
            let x = self.var(0)?;
            return self.lam(s, x);
        }
        for step in steps {
            if self.c.term_table.get(*step as usize).is_none() {
                return Err(OrdinaryCarrierError::Shape);
            }
        }
        let c = match (name, composition) {
            (Some(name), None) => self.constant(name)?,
            (None, Some(term)) if self.c.term_table.get(term as usize).is_some() => term,
            _ => return Err(OrdinaryCarrierError::Shape),
        };
        tree(self, c, steps)
    }
    fn lift(&mut self, from: u32, to: u32) -> R<()> {
        if from > to {
            return Err(OrdinaryCarrierError::Shape);
        }
        let small = self.cube(from)?;
        let large = self.cube(to)?;
        let f = self.constant("Std.Bool.false")?;
        let src = self.var(to)?;
        let args = self.selectors(from)?;
        let value = self.app(src, args)?;
        let mut padding = f;
        let or = self.constant("Std.Bool.or")?;
        for i in from..to {
            let v = self.var(i)?;
            padding = self.app(or, vec![padding, v])?;
        }
        let rec = self.constant("Std.Bool.rec")?;
        let value = self.app(rec, vec![value, f, padding])?;
        let body = self.wrap_selectors(to, value)?;
        let body = self.lam(small, body)?;
        let ty = self.pi(small, large)?;
        self.define(&format!("{PREFIX}.Pad.D{from}.D{to}"), ty, body)?;
        let src = self.var(from)?;
        let mut args = vec![f; (to - from) as usize];
        args.extend(self.selectors(from)?);
        let value = self.app(src, args)?;
        let body = self.wrap_selectors(from, value)?;
        let body = self.lam(large, body)?;
        let ty = self.pi(large, small)?;
        self.define(&format!("{PREFIX}.Unpad.D{to}.D{from}"), ty, body)
    }
    fn finish(mut self) -> R<Vec<u8>> {
        let names = self.c.name_table.clone();
        self.c.name_table.sort();
        for d in &mut self.c.declarations {
            d.name = self
                .c
                .name_table
                .binary_search(&names[d.name as usize])
                .map_err(|_| OrdinaryCarrierError::Linkage)? as u32;
        }
        for l in &mut self.c.level_table {
            if let LevelNode::Param(n) = l {
                *n = self
                    .c
                    .name_table
                    .binary_search(&names[*n as usize])
                    .map_err(|_| OrdinaryCarrierError::Linkage)? as u32;
            }
        }
        self.c.export_block =
            build_export_block(&self.c).map_err(|_| OrdinaryCarrierError::Linkage)?;
        self.c.axiom_report =
            build_axiom_report(&self.c).map_err(|_| OrdinaryCarrierError::Linkage)?;
        crate::csharp_practical_vc_model::validate_csharp_practical_certificate_structure(&self.c)
            .map_err(|_| OrdinaryCarrierError::Limit)?;
        self.c.hashes.export_hash = export_block_hash(&self.c.export_block);
        self.c.hashes.axiom_report_hash = axiom_report_hash_for_report(&self.c.axiom_report);
        encode_certificate_bounded(&self.c, 16 * 1024 * 1024)
            .map_err(|_| OrdinaryCarrierError::Limit)
    }
}

#[cfg(test)]
#[path = "csharp_practical_ordinary_test_eval.rs"]
mod test_eval;

#[cfg(test)]
mod tests {
    pub(super) use super::test_eval::{apply, bit, eval, run, Env, V};
    use super::*;
    use std::{cell::RefCell, rc::Rc};
    #[test]
    fn ordinary_core_observer_preserves_unused_arguments() {
        let mut b = Builder::new().unwrap();
        let yes = b.constant("Std.Bool.true").unwrap();
        let body = b.lam(b.boolean, yes).unwrap();
        let ty = b.pi(b.boolean, b.boolean).unwrap();
        b.define("Test.Ignore", ty, body).unwrap();
        let cert = decode_canonical_certificate(&b.finish().unwrap()).unwrap();
        // A poison suspension detects demand without recognizing an operation
        // in the evaluator. It is a test argument, never certificate evidence.
        let poison = || V::Thunk(u32::MAX, Rc::new(Env::Empty), Rc::new(RefCell::new(None)));
        assert!(bit(run(&cert, "Test.Ignore", vec![poison()])));
        for choice in [false, true] {
            let branches = if choice {
                vec![poison(), V::Bit(true)]
            } else {
                vec![V::Bit(true), poison()]
            };
            assert!(bit(apply(&cert, V::Rec(branches), V::Bit(choice))));
        }
    }
    #[test]
    fn ordinary_cube_helpers_preserve_order_and_zero_padding() {
        let mut b = Builder::new().unwrap();
        for d in [0, 1, 3] {
            b.helpers(d).unwrap();
        }
        b.lift(1, 3).unwrap();
        // x -> !x at Bool and address identity at C(1).
        let ty = b.cube(1).unwrap();
        let v = b.var(0).unwrap();
        let ident = b.lam(b.boolean, v).unwrap();
        b.define("Test.AddressIdentity", ty, ident).unwrap();
        let c = decode_canonical_certificate(&b.finish().unwrap()).unwrap();
        for flag in [false, true] {
            for yes in [false, true] {
                for no in [false, true] {
                    assert_eq!(
                        bit(run(
                            &c,
                            &format!("{PREFIX}.Cube.D0.Mux"),
                            vec![V::Bit(flag), V::Bit(yes), V::Bit(no)]
                        )),
                        if flag { yes } else { no }
                    );
                }
            }
        }
        let identity = run(&c, "Test.AddressIdentity", vec![]);
        let pad = run(&c, &format!("{PREFIX}.Pad.D1.D3"), vec![identity.clone()]);
        for mask in 0..8 {
            let got = (0..3).fold(pad.clone(), |f, i| {
                apply(&c, f, V::Bit(mask & (1 << i) != 0))
            });
            assert_eq!(bit(got), mask == 4);
        }
        let unpad = run(&c, &format!("{PREFIX}.Unpad.D3.D1"), vec![pad]);
        for flag in [false, true] {
            assert_eq!(bit(apply(&c, unpad.clone(), V::Bit(flag))), flag);
        }
        let zero = run(&c, &format!("{PREFIX}.Cube.D1.Zero"), vec![]);
        for flag in [false, true] {
            let mux = run(
                &c,
                &format!("{PREFIX}.Cube.D1.Mux"),
                vec![V::Bit(flag), identity.clone(), zero.clone()],
            );
            for addr in [false, true] {
                assert_eq!(bit(apply(&c, mux.clone(), V::Bit(addr))), flag && addr);
            }
        }
        // f=false, g=not: g(f(x)) is true, f(g(x)) is false.
        let neg = c
            .declarations
            .iter()
            .find(|d| c.name_table[d.name as usize] == "Std.Bool.not")
            .unwrap();
        let DeclarationKind::Def { value, .. } = neg.kind else {
            panic!()
        };
        let neg = eval(&c, value, &[]);
        let false_fn = zero;
        let composed = run(
            &c,
            &format!("{PREFIX}.Cube.D0.Compose"),
            vec![false_fn, neg],
        );
        for x in [false, true] {
            assert!(bit(apply(&c, composed.clone(), V::Bit(x))));
        }
    }
    #[test]
    fn ordinary_static_composition_counts_occurrences_before_sharing() {
        let mut b = Builder::new().unwrap();
        b.helpers(0).unwrap();
        b.helpers(1).unwrap();
        let not = b.constant("Std.Bool.not").unwrap();
        let constant_false = b.constant(&format!("{PREFIX}.Cube.D1.Zero")).unwrap();
        let ty = b.pi(b.boolean, b.boolean).unwrap();
        let seq = b
            .compose(0, &[constant_false, constant_false, not])
            .unwrap();
        b.define("Test.Ordered", ty, seq).unwrap();
        let rest = vec![not; 16384 - 3];
        let seq = b.compose(0, &rest).unwrap();
        b.define("Test.Shared", ty, seq).unwrap();
        assert_eq!(b.static_transformers, 16384);
        assert_eq!(b.compose(0, &[not]), Err(OrdinaryCarrierError::Limit));
        // A repeated 16K scan has logarithmic generated depth even though all
        // its transformation occurrences have been charged to the budget.
        let c = decode_canonical_certificate(&b.finish().unwrap()).unwrap();
        for x in [false, true] {
            assert!(bit(run(&c, "Test.Ordered", vec![V::Bit(x)])));
            assert_eq!(bit(run(&c, "Test.Shared", vec![V::Bit(x)])), !x);
        }
    }
    #[test]
    fn ordinary_builder_rejects_unresolved_symbols_and_limits() {
        let mut b = Builder::new().unwrap();
        assert_eq!(b.constant("Unresolved"), Err(OrdinaryCarrierError::Linkage));
        assert_eq!(
            b.term(TermNode::App {
                function: u32::MAX,
                arguments: vec![]
            }),
            Err(OrdinaryCarrierError::Shape)
        );
        assert_eq!(b.lift(4, 3), Err(OrdinaryCarrierError::Shape));
        assert_eq!(b.helpers(254), Err(OrdinaryCarrierError::Limit));
    }
}

#[path = "csharp_practical_ordinary_scalar_bits.rs"]
mod scalar_bits;

#[path = "csharp_practical_ordinary_structural.rs"]
mod structural;
pub use structural::{
    generate_csharp_practical_ordinary_ordered_folds, generate_csharp_practical_ordinary_relations,
    generate_csharp_practical_ordinary_scalar_domains,
    generate_csharp_practical_ordinary_structural, import_csharp_practical_ordinary_ordered_folds,
    import_csharp_practical_ordinary_relations, import_csharp_practical_ordinary_scalar_domains,
    import_csharp_practical_ordinary_structural, OrdinaryArmOperations,
    OrdinaryOrderedFoldDefinition, OrdinaryOrderedFoldProgram, OrdinaryProductOperations,
    OrdinaryProjection, OrdinaryRelationDefinition, OrdinaryRelationProgram,
    OrdinaryScalarDomainDefinition, OrdinaryScalarDomainProgram, OrdinaryScalarDomainRule,
    OrdinaryStructuralDefinition, OrdinaryStructuralOperations, OrdinaryStructuralProgram,
};

pub use scalar_bits::{
    generate_csharp_practical_ordinary_calendar, generate_csharp_practical_ordinary_decimal,
    generate_csharp_practical_ordinary_floating, generate_csharp_practical_ordinary_integers,
    generate_csharp_practical_ordinary_temporal, import_csharp_practical_ordinary_calendar,
    import_csharp_practical_ordinary_decimal, import_csharp_practical_ordinary_floating,
    import_csharp_practical_ordinary_integers, import_csharp_practical_ordinary_temporal,
    OrdinaryCalendarProgram, OrdinaryDecimalProgram, OrdinaryFloatingProgram,
    OrdinaryIntegerDefinition, OrdinaryIntegerProgram, OrdinaryScalarDefinition,
    OrdinaryTemporalProgram,
};

pub use scalar_bits::{
    generate_csharp_practical_ordinary_strings, import_csharp_practical_ordinary_strings,
    OrdinaryStringDefinition, OrdinaryStringProgram,
};
