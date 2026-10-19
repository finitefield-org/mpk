//! Private, untrusted ordinary foundation definitions for practical Go.
//!
//! This component generates real closed terms. It is not the complete Go
//! foundation descriptor, a source binding, or authority to activate a profile.
use std::collections::BTreeMap;

use mpk_cert::encode::{
    Certificate, CertificateHashes, Declaration, DeclarationKind, DefinitionReducibility,
    LevelNode, TermNode,
};
use sha2::{Digest, Sha256};

mod dependencies;
mod integer;
mod natural;
mod polynomial;
mod range;
mod syntax;
mod word;
mod word_numeric;
mod word_proofs;

const BOOL: &str = "Std.Bool";
const PREFIX: &str = "Mpk.Go.Practical.Bool";
const BOOL_HEX: &[u8] = include_bytes!("../../../proofs/std/bool/std-bool.hex");
const EQ_HEX: &[u8] = include_bytes!("../../../proofs/std/eq/std-eq.hex");
const NAT_HEX: &[u8] = include_bytes!("../../../proofs/std/nat/std-nat.hex");
const MAX_BYTES: usize = 2_097_152;
const MAX_TERMS: usize = 131_072;
const MAX_DECLARATIONS: usize = 4_096;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GoOrdinaryError {
    Foundation,
    Linkage,
    Limit,
}

type Result<T> = std::result::Result<T, GoOrdinaryError>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoBooleanFoundation {
    certificate: Vec<u8>,
}

impl GoBooleanFoundation {
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoCoreFoundation {
    certificate: Vec<u8>,
}

impl GoCoreFoundation {
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoNaturalFoundation {
    certificate: Vec<u8>,
}

impl GoNaturalFoundation {
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoIntegerFoundation {
    certificate: Vec<u8>,
}

impl GoIntegerFoundation {
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoRangeFoundation {
    certificate: Vec<u8>,
}

impl GoRangeFoundation {
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoWordFoundation {
    certificate: Vec<u8>,
}

impl GoWordFoundation {
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoWordNumericFoundation {
    certificate: Vec<u8>,
}

impl GoWordNumericFoundation {
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoWordArithmeticFoundation {
    certificate: Vec<u8>,
}

impl GoWordArithmeticFoundation {
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoWordOrderFoundation {
    certificate: Vec<u8>,
}

impl GoWordOrderFoundation {
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoWordDivisionFoundation {
    certificate: Vec<u8>,
}
impl GoWordDivisionFoundation {
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
enum Key {
    Sort(u32),
    Var(u32),
    Constant(u32, Vec<u32>),
    Apply(u32, Vec<u32>),
    Lambda(u32, u32),
    Pi(u32, u32),
    Let(u32, u32, u32),
}

fn key(term: &TermNode) -> Key {
    match term {
        TermNode::Sort(level) => Key::Sort(*level),
        TermNode::Var(index) => Key::Var(*index),
        TermNode::Const { global, levels } => Key::Constant(*global, levels.clone()),
        TermNode::App {
            function,
            arguments,
        } => Key::Apply(*function, arguments.clone()),
        TermNode::Lam { ty, body } => Key::Lambda(*ty, *body),
        TermNode::Pi { ty, body } => Key::Pi(*ty, *body),
        TermNode::Let { ty, value, body } => Key::Let(*ty, *value, *body),
    }
}

struct Builder {
    certificate: Certificate,
    terms: BTreeMap<Key, u32>,
    globals: BTreeMap<String, u32>,
    boolean: u32,
    sort: u32,
}

fn standard_certificate(hex: &[u8], expected: &str) -> Result<Certificate> {
    if format!("{:x}", Sha256::digest(hex)) != expected {
        return Err(GoOrdinaryError::Foundation);
    }
    let hex: String = std::str::from_utf8(hex)
        .map_err(|_| GoOrdinaryError::Foundation)?
        .split_whitespace()
        .collect();
    let bytes = (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).map_err(|_| GoOrdinaryError::Foundation))
        .collect::<Result<Vec<_>>>()?;
    let certificate =
        mpk_cert::decode_canonical_certificate(&bytes).map_err(|_| GoOrdinaryError::Foundation)?;
    if !certificate.imports.is_empty()
        || !certificate.proof_node_table.is_empty()
        || !certificate.theory_certificates.is_empty()
        || certificate.source_manifest.is_some()
        || certificate.axiom_report.summary.total_axiom_count != 0
    {
        return Err(GoOrdinaryError::Foundation);
    }
    Ok(certificate)
}

// Link two exact closed standard certificates, retaining every ordinary
// declaration. This is private fixed input composition, not an import resolver.
fn append_standard(certificate: &mut Certificate, source: Certificate) -> Result<()> {
    let name_offset = certificate.name_table.len() as u32;
    let term_offset = certificate.term_table.len() as u32;
    let global_offset = certificate.declarations.len() as u32;
    for declaration in &source.declarations {
        if certificate.declarations.iter().any(|d| {
            certificate.name_table[d.name as usize] == source.name_table[declaration.name as usize]
        }) {
            return Err(GoOrdinaryError::Foundation);
        }
    }
    certificate.name_table.extend(source.name_table);
    let mut levels = Vec::new();
    for level in source.level_table {
        let level = match level {
            LevelNode::Zero => LevelNode::Zero,
            LevelNode::Succ(parent) => LevelNode::Succ(levels[parent as usize]),
            LevelNode::Max(a, b) => LevelNode::Max(levels[a as usize], levels[b as usize]),
            LevelNode::Param(name) => LevelNode::Param(name + name_offset),
        };
        let index = if let Some(index) = certificate.level_table.iter().position(|l| *l == level) {
            index as u32
        } else {
            let index = certificate.level_table.len() as u32;
            certificate.level_table.push(level);
            index
        };
        levels.push(index);
    }
    for term in source.term_table {
        certificate.term_table.push(match term {
            TermNode::Sort(level) => TermNode::Sort(levels[level as usize]),
            TermNode::Var(index) => TermNode::Var(index),
            TermNode::Const {
                global,
                levels: arguments,
            } => TermNode::Const {
                global: global + global_offset,
                levels: arguments.iter().map(|l| levels[*l as usize]).collect(),
            },
            TermNode::App {
                function,
                arguments,
            } => TermNode::App {
                function: function + term_offset,
                arguments: arguments.iter().map(|a| a + term_offset).collect(),
            },
            TermNode::Lam { ty, body } => TermNode::Lam {
                ty: ty + term_offset,
                body: body + term_offset,
            },
            TermNode::Pi { ty, body } => TermNode::Pi {
                ty: ty + term_offset,
                body: body + term_offset,
            },
            TermNode::Let { ty, value, body } => TermNode::Let {
                ty: ty + term_offset,
                value: value + term_offset,
                body: body + term_offset,
            },
        });
    }
    for declaration in source.declarations {
        let kind = match declaration.kind {
            DeclarationKind::Def {
                ty,
                value,
                reducibility,
            } => DeclarationKind::Def {
                ty: ty + term_offset,
                value: value + term_offset,
                reducibility,
            },
            DeclarationKind::Theorem { ty, proof } => DeclarationKind::Theorem {
                ty: ty + term_offset,
                proof: proof + term_offset,
            },
            DeclarationKind::Inductive { ty } => DeclarationKind::Inductive {
                ty: ty + term_offset,
            },
            DeclarationKind::Constructor {
                ty,
                inductive,
                generated,
            } => DeclarationKind::Constructor {
                ty: ty + term_offset,
                inductive: inductive + global_offset,
                generated,
            },
            DeclarationKind::Recursor {
                ty,
                inductive,
                generated,
            } => DeclarationKind::Recursor {
                ty: ty + term_offset,
                inductive: inductive + global_offset,
                generated,
            },
            _ => return Err(GoOrdinaryError::Foundation),
        };
        certificate.declarations.push(Declaration {
            name: declaration.name + name_offset,
            kind,
        });
    }
    Ok(())
}

impl Builder {
    fn append_natural_numbers(&mut self) -> Result<()> {
        append_standard(
            &mut self.certificate,
            standard_certificate(
                NAT_HEX,
                "65bc5b701561d07b0f675668d942917da23c43606c0cfe0f9edfa113be99f583",
            )?,
        )?;
        self.globals = self
            .certificate
            .declarations
            .iter()
            .enumerate()
            .map(|(i, d)| {
                (
                    self.certificate.name_table[d.name as usize].clone(),
                    i as u32,
                )
            })
            .collect();
        self.terms = self
            .certificate
            .term_table
            .iter()
            .enumerate()
            .map(|(i, t)| (key(t), i as u32))
            .collect();
        Ok(())
    }

    fn frozen_core_interfaces(&mut self) -> Result<()> {
        let nat = self.constant("Std.Nat")?;
        let zero = self.constant("Std.Nat.zero")?;
        let successor = self.constant("Std.Nat.succ")?;
        let motive_type = self.pi(nat, self.sort)?;
        let p = self.var(0)?;
        let zero_type = self.app(p, vec![zero])?;
        let p = self.var(2)?;
        let n = self.var(0)?;
        let ih = self.app(p, vec![n])?;
        let p = self.var(3)?;
        let n = self.var(1)?;
        let next = self.app(successor, vec![n])?;
        let step_result = self.app(p, vec![next])?;
        let step_result = self.pi(ih, step_result)?;
        let step = self.pi(nat, step_result)?;
        let p = self.var(3)?;
        let n = self.var(0)?;
        let result = self.app(p, vec![n])?;
        let ty = self.pi(nat, result)?;
        let ty = self.pi(step, ty)?;
        let ty = self.pi(zero_type, ty)?;
        let ty = self.pi(motive_type, ty)?;
        self.declare(
            "Std.Nat.induct".into(),
            DeclarationKind::Recursor {
                ty,
                inductive: self.globals["Std.Nat"],
                generated: true,
            },
        )?;
        let ty = self.pi(self.boolean, self.sort)?;
        let ty = self.pi(self.sort, ty)?;
        let ty = self.pi(self.sort, ty)?;
        self.declare(
            "Std.Bool.recType".into(),
            DeclarationKind::Recursor {
                ty,
                inductive: self.globals[BOOL],
                generated: true,
            },
        )
    }

    fn natural_identity(&mut self) -> Result<()> {
        const ID: &str = "Mpk.Go.Practical.Nat.Identity";
        let nat = self.constant("Std.Nat")?;
        let zero = self.constant("Std.Nat.zero")?;
        let successor = self.constant("Std.Nat.succ")?;
        let ih = self.var(0)?;
        let step = self.app(successor, vec![ih])?;
        let step = self.lam(nat, step)?;
        let step = self.lam(nat, step)?;
        let n = self.var(0)?;
        let value = self.call("Std.Nat.rec", vec![zero, step, n])?;
        let value = self.lam(nat, value)?;
        let ty = self.pi(nat, nat)?;
        self.declare(
            ID.into(),
            DeclarationKind::Def {
                ty,
                value,
                reducibility: DefinitionReducibility::Reducible,
            },
        )?;

        let identity_n = self.call(ID, vec![n])?;
        let proposition = self.call("Std.Eq", vec![nat, identity_n, n])?;
        let motive = self.lam(nat, proposition)?;
        let base = self.call("Std.Eq.refl", vec![nat, zero])?;
        let ih_type = self.call("Std.Eq", vec![nat, identity_n, n])?;
        let n_under_ih = self.var(1)?;
        let identity_under_ih = self.call(ID, vec![n_under_ih])?;
        let ih = self.var(0)?;
        let step = self.call(
            "Std.Eq.congr",
            vec![nat, nat, successor, identity_under_ih, n_under_ih, ih],
        )?;
        let step = self.lam(ih_type, step)?;
        let step = self.lam(nat, step)?;
        let proof = self.call("Std.Nat.induct", vec![motive, base, step, n])?;
        let proof = self.lam(nat, proof)?;
        let ty = self.pi(nat, proposition)?;
        self.declare(
            "Mpk.Go.Practical.Nat.IdentityLaw".into(),
            DeclarationKind::Theorem { ty, proof },
        )
    }

    fn boolean_contradiction_elimination(&mut self) -> Result<()> {
        let no = self.constant("Std.Bool.false")?;
        let yes = self.constant("Std.Bool.true")?;
        let contradiction = self.equality(no, yes)?;
        // Under A : Sort0, h : Eq false true, b : Bool, A is index2.
        let a_under_tag = self.var(2)?;
        let tag = self.var(0)?;
        let selected = self.call("Std.Bool.recType", vec![self.boolean, a_under_tag, tag])?;
        let motive = self.lam(self.boolean, selected)?;
        let hypothesis = self.var(0)?;
        let proof = self.call(
            "Std.Eq.rewrite",
            vec![self.boolean, no, yes, motive, hypothesis, yes],
        )?;
        let proof = self.lam(contradiction, proof)?;
        let proof = self.lam(self.sort, proof)?;
        let a = self.var(1)?;
        let ty = self.pi(contradiction, a)?;
        let ty = self.pi(self.sort, ty)?;
        self.declare(
            "Mpk.Go.Practical.Bool.FalseElim".into(),
            DeclarationKind::Theorem { ty, proof },
        )
    }

    fn new() -> Result<Self> {
        let mut certificate = standard_certificate(
            BOOL_HEX,
            "88a37f9df68a18bc19d51c0832279ff97a2944fc1676326022269766933ee806",
        )?;
        let equality = standard_certificate(
            EQ_HEX,
            "80d71a2ef388a6ad1dea166cca4bd10d9093d538edc4999743b6fb1ad80d03a1",
        )?;
        append_standard(&mut certificate, equality)?;
        certificate.module = "Mpk.Go.Practical.BooleanFoundation".into();
        certificate.source_manifest = None;
        certificate.hashes = CertificateHashes::default();
        let globals = certificate
            .declarations
            .iter()
            .enumerate()
            .map(|(i, d)| (certificate.name_table[d.name as usize].clone(), i as u32))
            .collect::<BTreeMap<_, _>>();
        let bool_global = *globals.get(BOOL).ok_or(GoOrdinaryError::Foundation)?;
        let DeclarationKind::Inductive { ty: sort } =
            certificate.declarations[bool_global as usize].kind
        else {
            return Err(GoOrdinaryError::Foundation);
        };
        let terms = certificate
            .term_table
            .iter()
            .enumerate()
            .map(|(i, t)| (key(t), i as u32))
            .collect();
        let mut builder = Self {
            certificate,
            terms,
            globals,
            boolean: 0,
            sort,
        };
        builder.boolean = builder.constant(BOOL)?;
        Ok(builder)
    }

    fn term(&mut self, term: TermNode) -> Result<u32> {
        let key = key(&term);
        if let Some(&index) = self.terms.get(&key) {
            return Ok(index);
        }
        if self.certificate.term_table.len() >= MAX_TERMS {
            return Err(GoOrdinaryError::Limit);
        }
        let index = self.certificate.term_table.len() as u32;
        self.certificate.term_table.push(term);
        self.terms.insert(key, index);
        Ok(index)
    }

    fn constant(&mut self, name: &str) -> Result<u32> {
        let global = *self.globals.get(name).ok_or(GoOrdinaryError::Linkage)?;
        self.term(TermNode::Const {
            global,
            levels: vec![],
        })
    }

    fn var(&mut self, index: u32) -> Result<u32> {
        self.term(TermNode::Var(index))
    }
    fn pi(&mut self, ty: u32, body: u32) -> Result<u32> {
        self.term(TermNode::Pi { ty, body })
    }
    fn lam(&mut self, ty: u32, body: u32) -> Result<u32> {
        self.term(TermNode::Lam { ty, body })
    }
    fn app(&mut self, function: u32, arguments: Vec<u32>) -> Result<u32> {
        self.term(TermNode::App {
            function,
            arguments,
        })
    }
    fn call(&mut self, name: &str, arguments: Vec<u32>) -> Result<u32> {
        let function = self.constant(name)?;
        self.app(function, arguments)
    }

    fn declare(&mut self, name: String, kind: DeclarationKind) -> Result<()> {
        if self.globals.contains_key(&name) {
            return Err(GoOrdinaryError::Linkage);
        }
        if self.certificate.declarations.len() >= MAX_DECLARATIONS {
            return Err(GoOrdinaryError::Limit);
        }
        let name_id = self.certificate.name_table.len() as u32;
        self.certificate.name_table.push(name.clone());
        self.globals
            .insert(name, self.certificate.declarations.len() as u32);
        self.certificate.declarations.push(Declaration {
            name: name_id,
            kind,
        });
        Ok(())
    }

    fn cases(&mut self) -> Result<()> {
        let no = self.constant("Std.Bool.false")?;
        let yes = self.constant("Std.Bool.true")?;
        let motive = self.var(0)?;
        let no_type = self.app(motive, vec![no])?;
        let motive = self.var(1)?;
        let yes_type = self.app(motive, vec![yes])?;
        let motive = self.var(3)?;
        let major = self.var(0)?;
        let result = self.app(motive, vec![major])?;
        let major_type = self.pi(self.boolean, result)?;
        let yes_branch = self.pi(yes_type, major_type)?;
        let no_branch = self.pi(no_type, yes_branch)?;
        let motive_type = self.pi(self.boolean, self.sort)?;
        let ty = self.pi(motive_type, no_branch)?;
        self.declare(
            "Std.Bool.cases".into(),
            DeclarationKind::Recursor {
                ty,
                inductive: self.globals[BOOL],
                generated: true,
            },
        )
    }

    fn definition(&mut self, suffix: &str, arity: usize, value: u32) -> Result<()> {
        let mut ty = self.boolean;
        let mut body = value;
        for _ in 0..arity {
            ty = self.pi(self.boolean, ty)?;
            body = self.lam(self.boolean, body)?;
        }
        self.declare(
            format!("{PREFIX}.{suffix}"),
            DeclarationKind::Def {
                ty,
                value: body,
                reducibility: DefinitionReducibility::Reducible,
            },
        )
    }

    fn equality(&mut self, left: u32, right: u32) -> Result<u32> {
        self.call("Std.Eq", vec![self.boolean, left, right])
    }
    fn reflexivity(&mut self, value: u32) -> Result<u32> {
        self.call("Std.Eq.refl", vec![self.boolean, value])
    }

    fn double_not_case(&mut self, value: u32, flipped: u32) -> Result<u32> {
        let not = self.constant(&format!("{PREFIX}.Not"))?;
        let once = self.app(not, vec![value])?;
        let twice = self.app(not, vec![once])?;
        let middle = self.app(not, vec![flipped])?;
        let first = self.reflexivity(flipped)?;
        // The core reduces constructor majors. Congruence explicitly rewrites
        // the computed inner major; no additional iota rule is assumed.
        let congruence = self.call(
            "Std.Eq.congr",
            vec![self.boolean, self.boolean, not, once, flipped, first],
        )?;
        let last = self.reflexivity(value)?;
        self.call(
            "Std.Eq.trans",
            vec![self.boolean, twice, middle, value, congruence, last],
        )
    }

    fn finish(mut self) -> Result<Vec<u8>> {
        let previous = self.certificate.name_table.clone();
        self.certificate.name_table.sort();
        for declaration in &mut self.certificate.declarations {
            declaration.name = self
                .certificate
                .name_table
                .binary_search(&previous[declaration.name as usize])
                .map_err(|_| GoOrdinaryError::Linkage)? as u32;
        }
        for level in &mut self.certificate.level_table {
            if let LevelNode::Param(name) = level {
                *name = self
                    .certificate
                    .name_table
                    .binary_search(&previous[*name as usize])
                    .map_err(|_| GoOrdinaryError::Linkage)? as u32;
            }
        }
        self.certificate.export_block = mpk_cert::build_export_block(&self.certificate)
            .map_err(|_| GoOrdinaryError::Linkage)?;
        self.certificate.axiom_report = mpk_cert::build_axiom_report(&self.certificate)
            .map_err(|_| GoOrdinaryError::Linkage)?;
        if self.certificate.axiom_report.summary.total_axiom_count != 0 {
            return Err(GoOrdinaryError::Foundation);
        }
        self.certificate.hashes.export_hash =
            mpk_cert::export_block_hash(&self.certificate.export_block);
        self.certificate.hashes.axiom_report_hash =
            mpk_cert::axiom_report_hash_for_report(&self.certificate.axiom_report);
        mpk_cert::encode_certificate_bounded(&self.certificate, MAX_BYTES)
            .map_err(|_| GoOrdinaryError::Limit)
    }
}

fn boolean_builder() -> Result<Builder> {
    let mut builder = Builder::new()?;
    builder.cases()?;
    let no = builder.constant("Std.Bool.false")?;
    let yes = builder.constant("Std.Bool.true")?;
    let x = builder.var(0)?;
    let not = builder.call("Std.Bool.rec", vec![yes, no, x])?;
    builder.definition("Not", 1, not)?;
    let a = builder.var(1)?;
    let b = builder.var(0)?;
    let and = builder.call("Std.Bool.rec", vec![no, a, b])?;
    builder.definition("And", 2, and)?;
    let or = builder.call("Std.Bool.rec", vec![a, yes, b])?;
    builder.definition("Or", 2, or)?;
    let not_a = builder.call(&format!("{PREFIX}.Not"), vec![a])?;
    let xor = builder.call("Std.Bool.rec", vec![a, not_a, b])?;
    builder.definition("Xor", 2, xor)?;
    let equal = builder.call("Std.Bool.rec", vec![not_a, a, b])?;
    builder.definition("Equal", 2, equal)?;
    // These are universal proofs about ordinary computation, not free facts
    // or caller-supplied propositions. No source statement is claimed here.
    for (suffix, operation, argument, result) in [
        ("AndTrueRight", "And", yes, x),
        ("OrFalseRight", "Or", no, x),
        ("XorFalseRight", "Xor", no, x),
    ] {
        let left = builder.call(&format!("{PREFIX}.{operation}"), vec![x, argument])?;
        let equality = builder.equality(left, result)?;
        let ty = builder.pi(builder.boolean, equality)?;
        let refl = builder.reflexivity(result)?;
        let proof = builder.lam(builder.boolean, refl)?;
        builder.declare(
            format!("{PREFIX}.{suffix}"),
            DeclarationKind::Theorem { ty, proof },
        )?;
    }
    for suffix in ["DoubleNot", "EqualReflexive"] {
        let (left, right) = if suffix == "DoubleNot" {
            let once = builder.call(&format!("{PREFIX}.Not"), vec![x])?;
            (builder.call(&format!("{PREFIX}.Not"), vec![once])?, x)
        } else {
            (builder.call(&format!("{PREFIX}.Equal"), vec![x, x])?, yes)
        };
        let proposition = builder.equality(left, right)?;
        let motive = builder.lam(builder.boolean, proposition)?;
        let (false_proof, true_proof) = if suffix == "DoubleNot" {
            (
                builder.double_not_case(no, yes)?,
                builder.double_not_case(yes, no)?,
            )
        } else {
            (builder.reflexivity(yes)?, builder.reflexivity(yes)?)
        };
        let cases = builder.call("Std.Bool.cases", vec![motive, false_proof, true_proof, x])?;
        let ty = builder.pi(builder.boolean, proposition)?;
        let proof = builder.lam(builder.boolean, cases)?;
        builder.declare(
            format!("{PREFIX}.{suffix}"),
            DeclarationKind::Theorem { ty, proof },
        )?;
    }
    Ok(builder)
}

pub fn generate_go_practical_boolean_foundation() -> Result<GoBooleanFoundation> {
    Ok(GoBooleanFoundation {
        certificate: boolean_builder()?.finish()?,
    })
}

// Regenerate the complete compiled component, rather than trusting a caller
// hash, export list or self-declared checker receipt. Checkers verify its proof.
pub fn import_go_practical_boolean_foundation(candidate: &[u8]) -> Result<GoBooleanFoundation> {
    if candidate.len() > MAX_BYTES {
        return Err(GoOrdinaryError::Limit);
    }
    let expected = generate_go_practical_boolean_foundation()?;
    if candidate != expected.certificate_bytes() {
        return Err(GoOrdinaryError::Linkage);
    }
    Ok(expected)
}

pub fn generate_go_practical_core_foundation() -> Result<GoCoreFoundation> {
    Ok(GoCoreFoundation {
        certificate: core_builder()?.finish()?,
    })
}

fn core_builder() -> Result<Builder> {
    let mut builder = boolean_builder()?;
    builder.append_natural_numbers()?;
    builder.frozen_core_interfaces()?;
    builder.natural_identity()?;
    builder.boolean_contradiction_elimination()?;
    builder.certificate.module = "Mpk.Go.Practical.CoreFoundation".into();
    Ok(builder)
}

pub fn import_go_practical_core_foundation(candidate: &[u8]) -> Result<GoCoreFoundation> {
    if candidate.len() > MAX_BYTES {
        return Err(GoOrdinaryError::Limit);
    }
    let expected = generate_go_practical_core_foundation()?;
    if candidate != expected.certificate_bytes() {
        return Err(GoOrdinaryError::Linkage);
    }
    Ok(expected)
}

/// Compile unbounded Nat arithmetic and its ordinary universal laws. This
/// component carries no Go source claim or complete foundation registration.
pub fn generate_go_practical_natural_foundation() -> Result<GoNaturalFoundation> {
    Ok(GoNaturalFoundation {
        certificate: natural_builder()?.finish()?,
    })
}

fn natural_builder() -> Result<Builder> {
    let mut builder = core_builder()?;
    natural::append(&mut builder)?;
    builder.certificate.module = "Mpk.Go.Practical.NaturalFoundation".into();
    Ok(builder)
}

pub fn import_go_practical_natural_foundation(candidate: &[u8]) -> Result<GoNaturalFoundation> {
    if candidate.len() > MAX_BYTES {
        return Err(GoOrdinaryError::Limit);
    }
    let expected = generate_go_practical_natural_foundation()?;
    if candidate != expected.certificate_bytes() {
        return Err(GoOrdinaryError::Linkage);
    }
    Ok(expected)
}

/// Compile balanced mathematical integers with semantic cross-sum equality.
/// These ordinary library laws do not claim Go source correctness.
pub fn generate_go_practical_integer_foundation() -> Result<GoIntegerFoundation> {
    Ok(GoIntegerFoundation {
        certificate: integer_builder()?.finish()?,
    })
}

fn integer_builder() -> Result<Builder> {
    let mut builder = natural_builder()?;
    integer::append(&mut builder)?;
    builder.certificate.module = "Mpk.Go.Practical.IntegerFoundation".into();
    Ok(builder)
}

pub fn import_go_practical_integer_foundation(candidate: &[u8]) -> Result<GoIntegerFoundation> {
    if candidate.len() > MAX_BYTES {
        return Err(GoOrdinaryError::Limit);
    }
    let expected = generate_go_practical_integer_foundation()?;
    if candidate != expected.certificate_bytes() {
        return Err(GoOrdinaryError::Linkage);
    }
    Ok(expected)
}

/// Compile half-open logical folds and laws about total mathematical bodies.
/// Source-expression definedness and application proofs remain separate work.
pub fn generate_go_practical_range_foundation() -> Result<GoRangeFoundation> {
    Ok(GoRangeFoundation {
        certificate: range_builder()?.finish()?,
    })
}

fn range_builder() -> Result<Builder> {
    let mut builder = integer_builder()?;
    range::append(&mut builder)?;
    builder.certificate.module = "Mpk.Go.Practical.RangeFoundation".into();
    Ok(builder)
}

pub fn import_go_practical_range_foundation(candidate: &[u8]) -> Result<GoRangeFoundation> {
    if candidate.len() > MAX_BYTES {
        return Err(GoOrdinaryError::Limit);
    }
    let expected = generate_go_practical_range_foundation()?;
    if candidate != expected.certificate_bytes() {
        return Err(GoOrdinaryError::Linkage);
    }
    Ok(expected)
}

/// Compile full-width ordinary word circuits and their basic universal laws.
/// Source binding and general numeric/source correspondence remain required.
pub fn generate_go_practical_word_foundation() -> Result<GoWordFoundation> {
    let mut builder = range_builder()?;
    word::append(&mut builder)?;
    builder.certificate.module = "Mpk.Go.Practical.WordFoundation".into();
    Ok(GoWordFoundation {
        certificate: builder.finish()?,
    })
}

pub fn import_go_practical_word_foundation(candidate: &[u8]) -> Result<GoWordFoundation> {
    if candidate.len() > MAX_BYTES {
        return Err(GoOrdinaryError::Limit);
    }
    let expected = generate_go_practical_word_foundation()?;
    if candidate != expected.certificate_bytes() {
        return Err(GoOrdinaryError::Linkage);
    }
    Ok(expected)
}

/// Compile actual word addition conservation in unbounded mathematical Nat.
/// Source binding and complete Go profile authority remain separate work.
pub fn generate_go_practical_word_numeric_foundation() -> Result<GoWordNumericFoundation> {
    let mut builder = range_builder()?;
    let recipes = word::append_with_recipes(&mut builder)?;
    word_numeric::append(&mut builder, &recipes)?;
    builder.certificate.module = "Mpk.Go.Practical.WordNumericFoundation".into();
    Ok(GoWordNumericFoundation {
        certificate: builder.finish()?,
    })
}

pub fn import_go_practical_word_numeric_foundation(
    candidate: &[u8],
) -> Result<GoWordNumericFoundation> {
    if candidate.len() > MAX_BYTES {
        return Err(GoOrdinaryError::Limit);
    }
    let expected = generate_go_practical_word_numeric_foundation()?;
    if candidate != expected.certificate_bytes() {
        return Err(GoOrdinaryError::Linkage);
    }
    Ok(expected)
}

/// Compile ordinary numeric correspondence for the actual word arithmetic.
/// Source binding and complete Go profile authority remain separate work.
pub fn generate_go_practical_word_arithmetic_foundation() -> Result<GoWordArithmeticFoundation> {
    let mut builder = range_builder()?;
    let recipes = word::append_with_recipes(&mut builder)?;
    word_numeric::append_arithmetic(&mut builder, &recipes)?;
    builder.certificate.module = "Mpk.Go.Practical.WordArithmeticFoundation".into();
    Ok(GoWordArithmeticFoundation {
        certificate: builder.finish()?,
    })
}

pub fn import_go_practical_word_arithmetic_foundation(
    candidate: &[u8],
) -> Result<GoWordArithmeticFoundation> {
    if candidate.len() > MAX_BYTES {
        return Err(GoOrdinaryError::Limit);
    }
    let expected = generate_go_practical_word_arithmetic_foundation()?;
    if candidate != expected.certificate_bytes() {
        return Err(GoOrdinaryError::Linkage);
    }
    Ok(expected)
}

/// Compile actual word bounds, subtraction and unsigned order correspondence.
/// Full source/program and complete foundation authority remain separate work.
pub fn generate_go_practical_word_order_foundation() -> Result<GoWordOrderFoundation> {
    let mut builder = range_builder()?;
    let recipes = word::append_with_recipes(&mut builder)?;
    word_numeric::append_order(&mut builder, &recipes)?;
    builder.certificate.module = "Mpk.Go.Practical.WordOrderFoundation".into();
    Ok(GoWordOrderFoundation {
        certificate: builder.finish()?,
    })
}

pub fn import_go_practical_word_order_foundation(
    candidate: &[u8],
) -> Result<GoWordOrderFoundation> {
    if candidate.len() > MAX_BYTES {
        return Err(GoOrdinaryError::Limit);
    }
    let expected = generate_go_practical_word_order_foundation()?;
    if candidate != expected.certificate_bytes() {
        return Err(GoOrdinaryError::Linkage);
    }
    Ok(expected)
}

/// Compile actual unsigned restoring-division laws from closed ordinary roots.
/// Complete source/program/profile authority remains separate work.
pub fn generate_go_practical_word_division_foundation() -> Result<GoWordDivisionFoundation> {
    let mut builder = range_builder()?;
    let recipes = word::append_with_recipes(&mut builder)?;
    word_numeric::append_division(&mut builder, &recipes)?;
    builder.certificate.module = "Mpk.Go.Practical.WordDivisionFoundation".into();
    Ok(GoWordDivisionFoundation {
        certificate: builder.finish()?,
    })
}

pub fn import_go_practical_word_division_foundation(
    candidate: &[u8],
) -> Result<GoWordDivisionFoundation> {
    if candidate.len() > MAX_BYTES {
        return Err(GoOrdinaryError::Limit);
    }
    let expected = generate_go_practical_word_division_foundation()?;
    if candidate != expected.certificate_bytes() {
        return Err(GoOrdinaryError::Linkage);
    }
    Ok(expected)
}
