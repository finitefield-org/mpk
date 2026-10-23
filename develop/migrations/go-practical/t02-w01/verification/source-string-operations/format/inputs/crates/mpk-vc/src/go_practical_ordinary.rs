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

mod array;
mod composition;
mod dependencies;
mod error_layout;
mod error_value;
mod finite_vector;
mod integer;
mod natural;
mod polynomial;
mod product;
mod product_relation;
mod range;
mod sequence;
mod slice_descriptor;
mod source_error_boxing;
mod source_error_cases;
mod source_error_dispatch;
mod source_error_layout;
mod source_error_operations;
mod source_error_receiver;
mod source_error_representation;
mod source_string;
mod source_string_index;
mod source_string_length;
mod source_string_operation;
mod source_value_relations;
mod storage_lens;
mod string;
mod syntax;
mod word;
mod word_numeric;
mod word_proofs;
pub use source_error_boxing::{
    generate_go_practical_source_error_boxing_foundation,
    import_go_practical_source_error_boxing_foundation, GoSourceErrorBoxingFoundation,
};
pub use source_error_cases::{
    generate_go_practical_source_error_case_catalogue, GoSourceErrorCaseCatalogue,
    GoSourceErrorCaseRow,
};
pub use source_error_dispatch::{
    generate_go_practical_source_error_dispatch_catalogue, GoSourceErrorDispatchCatalogue,
    GoSourceErrorDispatchRow,
};
pub use source_error_layout::{
    generate_go_practical_source_error_layout_foundation,
    import_go_practical_source_error_layout_foundation, GoSourceErrorLayoutFoundation,
};
pub use source_error_operations::{
    generate_go_practical_source_error_operations_foundation,
    import_go_practical_source_error_operations_foundation, GoSourceErrorOperationsFoundation,
};
pub use source_error_receiver::GoSourceErrorReceiverRow;
pub use source_error_representation::GoSourceErrorRepresentationRow;
pub use source_string::{
    generate_go_practical_source_string_catalogue, GoSourceStringCatalogue, GoSourceStringRow,
};
pub use source_string_index::{
    generate_go_practical_source_string_index_catalogue, GoSourceStringIndexCatalogue,
    GoSourceStringIndexRow,
};
pub use source_string_length::{
    generate_go_practical_source_string_length_catalogue, GoSourceStringLengthCatalogue,
    GoSourceStringLengthRow,
};
pub use source_string_operation::{
    generate_go_practical_source_string_operation_catalogue,
    GoSourceStringOperationCatalogue, GoSourceStringOperationRow,
};
pub use source_value_relations::{
    generate_go_practical_source_value_relation_catalogue, GoSourceValueRelationCatalogue,
    GoSourceValueRelationRow,
};

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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoWordSignedProductFoundation {
    certificate: Vec<u8>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoWordSignedMagnitudeFoundation {
    certificate: Vec<u8>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoWordSignedBoundsFoundation {
    certificate: Vec<u8>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoWordZeroFoundation {
    certificate: Vec<u8>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoWordSignedDivisionPrerequisites {
    certificate: Vec<u8>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoWordSignedDivisionRefinement {
    certificate: Vec<u8>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoWordSignedDivisionBounds {
    certificate: Vec<u8>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoWordConversions {
    certificate: Vec<u8>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoWordShiftFoundation {
    certificate: Vec<u8>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoWordSignedOrderFoundation {
    certificate: Vec<u8>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoWordSignedEqualityFoundation {
    certificate: Vec<u8>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoProductFoundation {
    certificate: Vec<u8>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoProductRelationFoundation {
    certificate: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoErrorValueFoundation {
    certificate: Vec<u8>,
}

impl GoErrorValueFoundation {
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoErrorLayoutFoundation {
    case_count: usize,
    certificate: Vec<u8>,
}
impl GoErrorLayoutFoundation {
    pub fn case_count(&self) -> usize {
        self.case_count
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoSequenceFoundation {
    certificate: Vec<u8>,
}

impl GoSequenceFoundation {
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoFiniteVectorFoundation {
    certificate: Vec<u8>,
}

impl GoFiniteVectorFoundation {
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoWordIndexFoundation {
    certificate: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoStringFoundation {
    certificate: Vec<u8>,
}
impl GoStringFoundation {
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
}

/// Source-free data only. Original Go evaluation/nominal/context authority is
/// deliberately not inferred from a native value or its ordinary bit proofs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoStringValue {
    certificate: Vec<u8>,
    bytes: Vec<u8>,
}

/// Ordinary carrier validity only; no original source or nominal authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoValidStringValue {
    certificate: Vec<u8>,
    bytes: Vec<u8>,
}
impl GoValidStringValue {
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
    pub fn native_bytes(&self) -> &[u8] {
        &self.bytes
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoStringValidityFoundation {
    certificate: Vec<u8>,
}
impl GoStringValidityFoundation {
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoStringNormalizationFoundation {
    certificate: Vec<u8>,
}
impl GoStringNormalizationFoundation {
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoStringEqualityFoundation {
    certificate: Vec<u8>,
}
impl GoStringEqualityFoundation {
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoStringOrderFoundation {
    certificate: Vec<u8>,
}
impl GoStringOrderFoundation {
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoStringBoundedOrderFoundation {
    certificate: Vec<u8>,
}
impl GoStringBoundedOrderFoundation {
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoStringSliceFoundation {
    certificate: Vec<u8>,
}
impl GoStringSliceFoundation {
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoStringConcatFoundation {
    certificate: Vec<u8>,
}
impl GoStringConcatFoundation {
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoStringAffixFoundation {
    certificate: Vec<u8>,
}
impl GoStringAffixFoundation {
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoArrayFoundation {
    certificate: Vec<u8>,
}
impl GoArrayFoundation {
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoArrayEqualityFoundation {
    certificate: Vec<u8>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoSliceDescriptorFoundation {
    certificate: Vec<u8>,
}
impl GoSliceDescriptorFoundation {
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoStorageLensFoundation {
    certificate: Vec<u8>,
}
impl GoStorageLensFoundation {
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
}
impl GoArrayEqualityFoundation {
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
}
impl GoStringValue {
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
    pub fn native_bytes(&self) -> &[u8] {
        &self.bytes
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GoStringValueError {
    Domain { bytes: usize },
    Foundation(GoOrdinaryError),
}
impl From<GoOrdinaryError> for GoStringValueError {
    fn from(error: GoOrdinaryError) -> Self {
        Self::Foundation(error)
    }
}

impl GoWordIndexFoundation {
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
}
impl GoProductRelationFoundation {
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoWordProductEqualityFoundation {
    certificate: Vec<u8>,
}
impl GoWordProductEqualityFoundation {
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
}
impl GoProductFoundation {
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
}
impl GoWordSignedEqualityFoundation {
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
}
impl GoWordSignedOrderFoundation {
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
}
impl GoWordShiftFoundation {
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
}
impl GoWordConversions {
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
}
impl GoWordSignedDivisionBounds {
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
}
impl GoWordSignedDivisionRefinement {
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
}
impl GoWordSignedDivisionPrerequisites {
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
}
impl GoWordZeroFoundation {
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
}
impl GoWordSignedBoundsFoundation {
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
}
impl GoWordSignedMagnitudeFoundation {
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
}
impl GoWordSignedProductFoundation {
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
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

#[derive(Clone)]
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

/// Compile the actual signed word product, computed modular correction and
/// equivalence of its exact-result guard with balanced-Int equality.
pub fn generate_go_practical_word_signed_product_foundation(
) -> Result<GoWordSignedProductFoundation> {
    let mut builder = range_builder()?;
    let recipes = word::append_with_recipes(&mut builder)?;
    word_numeric::append_signed_product(&mut builder, &recipes)?;
    builder.certificate.module = "Mpk.Go.Practical.WordSignedProductFoundation".into();
    Ok(GoWordSignedProductFoundation {
        certificate: builder.finish()?,
    })
}
pub fn import_go_practical_word_signed_product_foundation(
    candidate: &[u8],
) -> Result<GoWordSignedProductFoundation> {
    if candidate.len() > MAX_BYTES {
        return Err(GoOrdinaryError::Limit);
    }
    let expected = generate_go_practical_word_signed_product_foundation()?;
    if candidate != expected.certificate_bytes() {
        return Err(GoOrdinaryError::Linkage);
    }
    Ok(expected)
}

/// Compile actual word negation, its exact-result guard and signed magnitudes.
/// Complete source/program/profile authority remains separate work.
pub fn generate_go_practical_word_signed_magnitude_foundation(
) -> Result<GoWordSignedMagnitudeFoundation> {
    let mut builder = range_builder()?;
    let recipes = word::append_with_recipes(&mut builder)?;
    word_numeric::append_signed_magnitude(&mut builder, &recipes)?;
    builder.certificate.module = "Mpk.Go.Practical.WordSignedMagnitudeFoundation".into();
    Ok(GoWordSignedMagnitudeFoundation {
        certificate: builder.finish()?,
    })
}
pub fn import_go_practical_word_signed_magnitude_foundation(
    candidate: &[u8],
) -> Result<GoWordSignedMagnitudeFoundation> {
    if candidate.len() > MAX_BYTES {
        return Err(GoOrdinaryError::Limit);
    }
    let expected = generate_go_practical_word_signed_magnitude_foundation()?;
    if candidate != expected.certificate_bytes() {
        return Err(GoOrdinaryError::Linkage);
    }
    Ok(expected)
}

/// Compile the actual signed word magnitude's computed gap and half-range bounds.
/// Complete source/program/profile authority remains separate work.
pub fn generate_go_practical_word_signed_bounds_foundation() -> Result<GoWordSignedBoundsFoundation>
{
    let mut builder = range_builder()?;
    let recipes = word::append_with_recipes(&mut builder)?;
    word_numeric::append_signed_bounds(&mut builder, &recipes)?;
    builder.certificate.module = "Mpk.Go.Practical.WordSignedBoundsFoundation".into();
    Ok(GoWordSignedBoundsFoundation {
        certificate: builder.finish()?,
    })
}
pub fn import_go_practical_word_signed_bounds_foundation(
    candidate: &[u8],
) -> Result<GoWordSignedBoundsFoundation> {
    if candidate.len() > MAX_BYTES {
        return Err(GoOrdinaryError::Limit);
    }
    let expected = generate_go_practical_word_signed_bounds_foundation()?;
    if candidate != expected.certificate_bytes() {
        return Err(GoOrdinaryError::Linkage);
    }
    Ok(expected)
}

/// Compile direct actual word zero/positive-divisor correspondence.
/// Complete source/program/profile authority remains separate work.
pub fn generate_go_practical_word_zero_foundation() -> Result<GoWordZeroFoundation> {
    let mut builder = range_builder()?;
    let recipes = word::append_with_recipes(&mut builder)?;
    word_numeric::append_zero(&mut builder, &recipes)?;
    builder.certificate.module = "Mpk.Go.Practical.WordZeroFoundation".into();
    Ok(GoWordZeroFoundation {
        certificate: builder.finish()?,
    })
}
pub fn import_go_practical_word_zero_foundation(candidate: &[u8]) -> Result<GoWordZeroFoundation> {
    if candidate.len() > MAX_BYTES {
        return Err(GoOrdinaryError::Limit);
    }
    let expected = generate_go_practical_word_zero_foundation()?;
    if candidate != expected.certificate_bytes() {
        return Err(GoOrdinaryError::Linkage);
    }
    Ok(expected)
}

/// Compose the complete checked arithmetic prerequisites for signed division.
/// This owns no signed-division/source-program completeness claim.
pub fn generate_go_practical_word_signed_division_prerequisites(
) -> Result<GoWordSignedDivisionPrerequisites> {
    let buffers = [
        generate_go_practical_word_division_foundation()?
            .certificate_bytes()
            .to_vec(),
        generate_go_practical_word_signed_bounds_foundation()?
            .certificate_bytes()
            .to_vec(),
        generate_go_practical_word_zero_foundation()?
            .certificate_bytes()
            .to_vec(),
        generate_go_practical_word_foundation()?
            .certificate_bytes()
            .to_vec(),
    ];
    let sources = buffers
        .iter()
        .map(|b| mpk_cert::decode_canonical_certificate(b).map_err(|_| GoOrdinaryError::Foundation))
        .collect::<Result<Vec<_>>>()?;
    let mut builder = composition::signed_division_prerequisites(&sources)?;
    builder.certificate.module = "Mpk.Go.Practical.WordSignedDivisionPrerequisites".into();
    Ok(GoWordSignedDivisionPrerequisites {
        certificate: builder.finish()?,
    })
}
pub fn import_go_practical_word_signed_division_prerequisites(
    candidate: &[u8],
) -> Result<GoWordSignedDivisionPrerequisites> {
    if candidate.len() > MAX_BYTES {
        return Err(GoOrdinaryError::Limit);
    }
    let expected = generate_go_practical_word_signed_division_prerequisites()?;
    if candidate != expected.certificate_bytes() {
        return Err(GoOrdinaryError::Linkage);
    }
    Ok(expected)
}

/// Prove actual signed division/remainder against their mathematical magnitudes
/// with computed modular corrections. Correction classification remains separate.
pub fn generate_go_practical_word_signed_division_refinement(
) -> Result<GoWordSignedDivisionRefinement> {
    let buffers = [
        generate_go_practical_word_division_foundation()?
            .certificate_bytes()
            .to_vec(),
        generate_go_practical_word_signed_bounds_foundation()?
            .certificate_bytes()
            .to_vec(),
        generate_go_practical_word_zero_foundation()?
            .certificate_bytes()
            .to_vec(),
        generate_go_practical_word_foundation()?
            .certificate_bytes()
            .to_vec(),
    ];
    let sources = buffers
        .iter()
        .map(|b| mpk_cert::decode_canonical_certificate(b).map_err(|_| GoOrdinaryError::Foundation))
        .collect::<Result<Vec<_>>>()?;
    let roots = [
        "Reflexive",
        "Symmetric",
        "Transitive",
        "AddRespects",
        "MulRespectsLeft",
        "MulRespectsRight",
    ]
    .map(|name| format!("Mpk.Go.Practical.Int.{name}"));
    let mut builder = composition::signed_division_prerequisites_with_roots(&sources, &roots)?;
    word_numeric::append_signed_division_refinement(&mut builder)?;
    builder.certificate.module = "Mpk.Go.Practical.WordSignedDivisionRefinement".into();
    Ok(GoWordSignedDivisionRefinement {
        certificate: builder.finish()?,
    })
}
pub fn import_go_practical_word_signed_division_refinement(
    candidate: &[u8],
) -> Result<GoWordSignedDivisionRefinement> {
    if candidate.len() > MAX_BYTES {
        return Err(GoOrdinaryError::Limit);
    }
    let expected = generate_go_practical_word_signed_division_refinement()?;
    if candidate != expected.certificate_bytes() {
        return Err(GoOrdinaryError::Linkage);
    }
    Ok(expected)
}

/// Prove actual quotient/remainder ranges and classify their modular corrections.
/// Complete source/program/profile authority remains separate work.
pub fn generate_go_practical_word_signed_division_bounds() -> Result<GoWordSignedDivisionBounds> {
    let buffers = [
        generate_go_practical_word_division_foundation()?
            .certificate_bytes()
            .to_vec(),
        generate_go_practical_word_signed_bounds_foundation()?
            .certificate_bytes()
            .to_vec(),
        generate_go_practical_word_zero_foundation()?
            .certificate_bytes()
            .to_vec(),
        generate_go_practical_word_foundation()?
            .certificate_bytes()
            .to_vec(),
    ];
    let sources = buffers
        .iter()
        .map(|bytes| {
            mpk_cert::decode_canonical_certificate(bytes).map_err(|_| GoOrdinaryError::Foundation)
        })
        .collect::<Result<Vec<_>>>()?;
    let mut roots = [
        "Reflexive",
        "Symmetric",
        "Transitive",
        "AddRespects",
        "MulRespectsLeft",
        "MulRespectsRight",
    ]
    .map(|name| format!("Mpk.Go.Practical.Int.{name}"))
    .to_vec();
    for width in [8_usize, 16, 32, 64] {
        roots.push(format!(
            "Mpk.Go.Practical.Word.SignedBounds.HalfPositive{}",
            width.trailing_zeros()
        ));
        roots.push(format!(
            "Mpk.Go.Practical.Word.SignedBounds.W{width}.MagnitudeHalfBalance"
        ));
    }
    let mut builder = composition::signed_division_prerequisites_with_roots(&sources, &roots)?;
    word_numeric::append_signed_division_refinement(&mut builder)?;
    word_numeric::append_signed_division_bounds(&mut builder)?;
    builder.certificate.module = "Mpk.Go.Practical.WordSignedDivisionBounds".into();
    Ok(GoWordSignedDivisionBounds {
        certificate: builder.finish()?,
    })
}
pub fn import_go_practical_word_signed_division_bounds(
    candidate: &[u8],
) -> Result<GoWordSignedDivisionBounds> {
    if candidate.len() > MAX_BYTES {
        return Err(GoOrdinaryError::Limit);
    }
    let expected = generate_go_practical_word_signed_division_bounds()?;
    if candidate != expected.certificate_bytes() {
        return Err(GoOrdinaryError::Linkage);
    }
    Ok(expected)
}

/// Prove the actual word conversions and signed shift-count normalization.
/// Complete source/program/profile binding remains separate work.
pub fn generate_go_practical_word_conversions() -> Result<GoWordConversions> {
    let mut builder = range_builder()?;
    let recipes = word::append_with_recipes(&mut builder)?;
    word_numeric::append_conversions(&mut builder, &recipes)?;
    builder.certificate.module = "Mpk.Go.Practical.WordConversions".into();
    Ok(GoWordConversions {
        certificate: builder.finish()?,
    })
}

pub fn import_go_practical_word_conversions(candidate: &[u8]) -> Result<GoWordConversions> {
    if candidate.len() > MAX_BYTES {
        return Err(GoOrdinaryError::Limit);
    }
    let expected = generate_go_practical_word_conversions()?;
    if candidate != expected.certificate_bytes() {
        return Err(GoOrdinaryError::Linkage);
    }
    Ok(expected)
}

/// Generate ordinary shift proof terms from exact original closed inputs.
/// Complete source/program/profile authority remains separate work.
pub fn generate_go_practical_word_shift_foundation() -> Result<GoWordShiftFoundation> {
    let buffers = [
        generate_go_practical_word_conversions()?
            .certificate_bytes()
            .to_vec(),
        generate_go_practical_word_arithmetic_foundation()?
            .certificate_bytes()
            .to_vec(),
        generate_go_practical_word_foundation()?
            .certificate_bytes()
            .to_vec(),
    ];
    let inputs = buffers
        .iter()
        .map(|b| mpk_cert::decode_canonical_certificate(b).map_err(|_| GoOrdinaryError::Foundation))
        .collect::<Result<Vec<_>>>()?;
    let mut builder = composition::word_shift_prerequisites(&inputs)?;
    word_numeric::append_shifts(&mut builder)?;
    builder.certificate.module = "Mpk.Go.Practical.WordShiftFoundation".into();
    Ok(GoWordShiftFoundation {
        certificate: builder.finish()?,
    })
}
pub fn import_go_practical_word_shift_foundation(
    candidate: &[u8],
) -> Result<GoWordShiftFoundation> {
    if candidate.len() > MAX_BYTES {
        return Err(GoOrdinaryError::Limit);
    }
    let expected = generate_go_practical_word_shift_foundation()?;
    if candidate != expected.certificate_bytes() {
        return Err(GoOrdinaryError::Linkage);
    }
    Ok(expected)
}

/// Prove signed comparison of the actual word circuits and balanced integers.
/// This component supplies no complete source/program/profile authority.
pub fn generate_go_practical_word_signed_order_foundation() -> Result<GoWordSignedOrderFoundation> {
    let mut builder = signed_order_builder()?;
    builder.certificate.module = "Mpk.Go.Practical.WordSignedOrderFoundation".into();
    Ok(GoWordSignedOrderFoundation {
        certificate: builder.finish()?,
    })
}
fn signed_order_builder() -> Result<Builder> {
    let buffers = [
        generate_go_practical_word_order_foundation()?
            .certificate_bytes()
            .to_vec(),
        generate_go_practical_word_shift_foundation()?
            .certificate_bytes()
            .to_vec(),
    ];
    let inputs = buffers
        .iter()
        .map(|bytes| {
            mpk_cert::decode_canonical_certificate(bytes).map_err(|_| GoOrdinaryError::Foundation)
        })
        .collect::<Result<Vec<_>>>()?;
    let mut builder = composition::word_signed_order_prerequisites(&inputs)?;
    word_numeric::append_signed_order(&mut builder)?;
    Ok(builder)
}
pub fn import_go_practical_word_signed_order_foundation(
    candidate: &[u8],
) -> Result<GoWordSignedOrderFoundation> {
    if candidate.len() > MAX_BYTES {
        return Err(GoOrdinaryError::Limit);
    }
    let expected = generate_go_practical_word_signed_order_foundation()?;
    if candidate != expected.certificate_bytes() {
        return Err(GoOrdinaryError::Linkage);
    }
    Ok(expected)
}

/// Connect actual equality/inequality circuits with balanced Int equivalence.
/// Complete source/program/profile authority remains separate work.
pub fn generate_go_practical_word_signed_equality_foundation(
) -> Result<GoWordSignedEqualityFoundation> {
    let mut builder = signed_order_builder()?;
    word_numeric::append_signed_equality(&mut builder)?;
    builder.certificate.module = "Mpk.Go.Practical.WordSignedEqualityFoundation".into();
    Ok(GoWordSignedEqualityFoundation {
        certificate: builder.finish()?,
    })
}
pub fn import_go_practical_word_signed_equality_foundation(
    candidate: &[u8],
) -> Result<GoWordSignedEqualityFoundation> {
    if candidate.len() > MAX_BYTES {
        return Err(GoOrdinaryError::Limit);
    }
    let expected = generate_go_practical_word_signed_equality_foundation()?;
    if candidate != expected.certificate_bytes() {
        return Err(GoOrdinaryError::Linkage);
    }
    Ok(expected)
}

/// Ordinary heterogeneous value products, observations, frames and validity.
/// Source types, invocation identity and complete context remain separate work.
pub fn generate_go_practical_product_foundation() -> Result<GoProductFoundation> {
    let mut builder = core_builder()?;
    product::append(&mut builder)?;
    builder.certificate.module = "Mpk.Go.Practical.ProductFoundation".into();
    Ok(GoProductFoundation {
        certificate: builder.finish()?,
    })
}
pub fn import_go_practical_product_foundation(candidate: &[u8]) -> Result<GoProductFoundation> {
    if candidate.len() > MAX_BYTES {
        return Err(GoOrdinaryError::Limit);
    }
    let expected = generate_go_practical_product_foundation()?;
    if candidate != expected.certificate_bytes() {
        return Err(GoOrdinaryError::Linkage);
    }
    Ok(expected)
}

/// Ordinary indexed values, prefix relations, updates and validity laws.
/// Connecting these mathematical sequences to frozen Go carriers/source
/// operations remains an ordinary proof obligation, not a new representation.
pub fn generate_go_practical_sequence_foundation() -> Result<GoSequenceFoundation> {
    let mut builder = natural_builder()?;
    product::append(&mut builder)?;
    sequence::append(&mut builder)?;
    builder.certificate.module = "Mpk.Go.Practical.SequenceFoundation".into();
    Ok(GoSequenceFoundation {
        certificate: builder.finish()?,
    })
}

pub fn import_go_practical_sequence_foundation(candidate: &[u8]) -> Result<GoSequenceFoundation> {
    if candidate.len() > MAX_BYTES {
        return Err(GoOrdinaryError::Limit);
    }
    let expected = generate_go_practical_sequence_foundation()?;
    if candidate != expected.certificate_bytes() {
        return Err(GoOrdinaryError::Linkage);
    }
    Ok(expected)
}

/// Preserve every arbitrary element of a finite Bool-indexed vector when
/// observing it as a Nat-indexed sequence. Source word/count conversions,
/// nominal identities and the complete Go context still require proofs.
pub fn generate_go_practical_finite_vector_foundation() -> Result<GoFiniteVectorFoundation> {
    let mut builder = natural_builder()?;
    product::append(&mut builder)?;
    sequence::append(&mut builder)?;
    finite_vector::append(&mut builder)?;
    builder.certificate.module = "Mpk.Go.Practical.FiniteVectorFoundation".into();
    Ok(GoFiniteVectorFoundation {
        certificate: builder.finish()?,
    })
}

pub fn import_go_practical_finite_vector_foundation(
    candidate: &[u8],
) -> Result<GoFiniteVectorFoundation> {
    if candidate.len() > MAX_BYTES {
        return Err(GoOrdinaryError::Limit);
    }
    let expected = generate_go_practical_finite_vector_foundation()?;
    if candidate != expected.certificate_bytes() {
        return Err(GoOrdinaryError::Linkage);
    }
    Ok(expected)
}

/// Bind the complete unsigned64 Nat value to finite slot observations under
/// its actual range proof. No source index or length is implicitly truncated.
/// Complete source, signed-domain and private-context authority remains due.
fn word_index_builder() -> Result<Builder> {
    let buffers = [
        generate_go_practical_word_shift_foundation()?
            .certificate_bytes()
            .to_vec(),
        generate_go_practical_finite_vector_foundation()?
            .certificate_bytes()
            .to_vec(),
    ];
    let inputs = buffers
        .iter()
        .map(|bytes| {
            mpk_cert::decode_canonical_certificate(bytes).map_err(|_| GoOrdinaryError::Foundation)
        })
        .collect::<Result<Vec<_>>>()?;
    let mut builder = composition::word_index_prerequisites(&inputs)?;
    word_numeric::append_indices(&mut builder)?;
    Ok(builder)
}

pub fn generate_go_practical_word_index_foundation() -> Result<GoWordIndexFoundation> {
    let mut builder = word_index_builder()?;
    builder.certificate.module = "Mpk.Go.Practical.WordIndexFoundation".into();
    Ok(GoWordIndexFoundation {
        certificate: builder.finish()?,
    })
}

pub fn import_go_practical_word_index_foundation(
    candidate: &[u8],
) -> Result<GoWordIndexFoundation> {
    if candidate.len() > MAX_BYTES {
        return Err(GoOrdinaryError::Limit);
    }
    let expected = generate_go_practical_word_index_foundation()?;
    if candidate != expected.certificate_bytes() {
        return Err(GoOrdinaryError::Linkage);
    }
    Ok(expected)
}

/// The frozen curried Word64/256 Word8 string shape and actual guarded ordinary
/// observations. This is not a source evaluation or registered Go context.
pub fn generate_go_practical_string_foundation() -> Result<GoStringFoundation> {
    let mut builder = word_index_builder()?;
    string::append(&mut builder)?;
    builder.certificate.module = "Mpk.Go.Practical.StringFoundation".into();
    Ok(GoStringFoundation {
        certificate: builder.finish()?,
    })
}

pub fn import_go_practical_string_foundation(candidate: &[u8]) -> Result<GoStringFoundation> {
    if candidate.len() > MAX_BYTES {
        return Err(GoOrdinaryError::Limit);
    }
    let expected = generate_go_practical_string_foundation()?;
    if candidate != expected.certificate_bytes() {
        return Err(GoOrdinaryError::Linkage);
    }
    Ok(expected)
}

/// Construct source-free ordinary data, with all64 length and2048 slot bits
/// proved equal to the supplied native bytes. The original Go expression and
/// its source/nominal/context/program obligations are still separate.
pub fn generate_go_practical_string_value(
    bytes: &[u8],
) -> std::result::Result<GoStringValue, GoStringValueError> {
    if bytes.len() > 256 {
        return Err(GoStringValueError::Domain { bytes: bytes.len() });
    }
    let mut builder = word_index_builder()?;
    string::append(&mut builder)?;
    string::append_value(&mut builder, bytes)?;
    builder.certificate.module = "Mpk.Go.Practical.StringValue".into();
    Ok(GoStringValue {
        certificate: builder.finish()?,
        bytes: bytes.to_vec(),
    })
}

pub fn import_go_practical_string_value(
    candidate: &[u8],
    bytes: &[u8],
) -> std::result::Result<GoStringValue, GoStringValueError> {
    if candidate.len() > MAX_BYTES {
        return Err(GoOrdinaryError::Limit.into());
    }
    let expected = generate_go_practical_string_value(bytes)?;
    if candidate != expected.certificate_bytes() {
        return Err(GoOrdinaryError::Linkage.into());
    }
    Ok(expected)
}

/// Generic actual length/padding witnesses used by the source-free literal
/// constructor. Existing string/data component bytes remain unchanged.
pub fn generate_go_practical_string_validity_foundation() -> Result<GoStringValidityFoundation> {
    let mut builder = word_index_builder()?;
    string::append(&mut builder)?;
    string::append_validity(&mut builder)?;
    builder.certificate.module = "Mpk.Go.Practical.StringValidityFoundation".into();
    Ok(GoStringValidityFoundation {
        certificate: builder.finish()?,
    })
}
pub fn import_go_practical_string_validity_foundation(
    candidate: &[u8],
) -> Result<GoStringValidityFoundation> {
    if candidate.len() > MAX_BYTES {
        return Err(GoOrdinaryError::Limit);
    }
    let expected = generate_go_practical_string_validity_foundation()?;
    if candidate != expected.certificate_bytes() {
        return Err(GoOrdinaryError::Linkage);
    }
    Ok(expected)
}

/// Full-word length and guarded byte observations survive canonical padding.
/// These ordinary data laws grant no original Go source or context authority.
pub fn generate_go_practical_string_normalization_foundation(
) -> Result<GoStringNormalizationFoundation> {
    let mut builder = word_index_builder()?;
    string::append(&mut builder)?;
    string::append_validity(&mut builder)?;
    string::append_normalization(&mut builder)?;
    builder.certificate.module = "Mpk.Go.Practical.StringNormalizationFoundation".into();
    Ok(GoStringNormalizationFoundation {
        certificate: builder.finish()?,
    })
}
pub fn import_go_practical_string_normalization_foundation(
    candidate: &[u8],
) -> Result<GoStringNormalizationFoundation> {
    if candidate.len() > MAX_BYTES {
        return Err(GoOrdinaryError::Limit);
    }
    let expected = generate_go_practical_string_normalization_foundation()?;
    if candidate != expected.certificate_bytes() {
        return Err(GoOrdinaryError::Linkage);
    }
    Ok(expected)
}

/// Actual full-byte string equality and its reflected logical content relation.
/// Source evaluation, nominal identity and profile/context admission are separate.
pub fn generate_go_practical_string_equality_foundation() -> Result<GoStringEqualityFoundation> {
    let mut builder = word_index_builder()?;
    string::append(&mut builder)?;
    string::append_validity(&mut builder)?;
    string::append_normalization(&mut builder)?;
    string::append_equality(&mut builder)?;
    builder.certificate.module = "Mpk.Go.Practical.StringEqualityFoundation".into();
    Ok(GoStringEqualityFoundation {
        certificate: builder.finish()?,
    })
}
pub fn import_go_practical_string_equality_foundation(
    candidate: &[u8],
) -> Result<GoStringEqualityFoundation> {
    if candidate.len() > MAX_BYTES {
        return Err(GoOrdinaryError::Limit);
    }
    let expected = generate_go_practical_string_equality_foundation()?;
    if candidate != expected.certificate_bytes() {
        return Err(GoOrdinaryError::Linkage);
    }
    Ok(expected)
}

/// Ordinary unsigned-byte lexicographic order on the frozen complete carrier.
/// Original Go source/evaluation/nominal/context admission remains separate.
pub fn generate_go_practical_string_order_foundation() -> Result<GoStringOrderFoundation> {
    let mut builder = string_order_builder()?;
    builder.certificate.module = "Mpk.Go.Practical.StringOrderFoundation".into();
    Ok(GoStringOrderFoundation {
        certificate: builder.finish()?,
    })
}
fn string_order_builder() -> Result<Builder> {
    let buffers = [
        generate_go_practical_string_equality_foundation()?
            .certificate_bytes()
            .to_vec(),
        generate_go_practical_word_order_foundation()?
            .certificate_bytes()
            .to_vec(),
    ];
    let inputs = buffers
        .iter()
        .map(|bytes| {
            mpk_cert::decode_canonical_certificate(bytes).map_err(|_| GoOrdinaryError::Foundation)
        })
        .collect::<Result<Vec<_>>>()?;
    let mut builder = composition::string_order_prerequisites(&inputs)?;
    string::append_order(&mut builder)?;
    Ok(builder)
}
pub fn import_go_practical_string_order_foundation(
    candidate: &[u8],
) -> Result<GoStringOrderFoundation> {
    if candidate.len() > MAX_BYTES {
        return Err(GoOrdinaryError::Limit);
    }
    let expected = generate_go_practical_string_order_foundation()?;
    if candidate != expected.certificate_bytes() {
        return Err(GoOrdinaryError::Linkage);
    }
    Ok(expected)
}

/// Bound witnesses connect rank order to actual full-size string equality.
/// This ordinary foundation does not admit original Go source or a context.
pub fn generate_go_practical_string_bounded_order_foundation(
) -> Result<GoStringBoundedOrderFoundation> {
    let mut builder = string_order_builder()?;
    string::append_bounded_order(&mut builder)?;
    builder.certificate.module = "Mpk.Go.Practical.StringBoundedOrderFoundation".into();
    Ok(GoStringBoundedOrderFoundation {
        certificate: builder.finish()?,
    })
}
pub fn import_go_practical_string_bounded_order_foundation(
    candidate: &[u8],
) -> Result<GoStringBoundedOrderFoundation> {
    if candidate.len() > MAX_BYTES {
        return Err(GoOrdinaryError::Limit);
    }
    let expected = generate_go_practical_string_bounded_order_foundation()?;
    if candidate != expected.certificate_bytes() {
        return Err(GoOrdinaryError::Linkage);
    }
    Ok(expected)
}

/// Complete frozen substring carrier, actual signed guards and byte equations.
/// Original source/type/context authority remains a separate obligation.
pub fn generate_go_practical_string_slice_foundation() -> Result<GoStringSliceFoundation> {
    let mut builder = string_slice_builder()?;
    builder.certificate.module = "Mpk.Go.Practical.StringSliceFoundation".into();
    Ok(GoStringSliceFoundation {
        certificate: builder.finish()?,
    })
}
fn string_slice_builder() -> Result<Builder> {
    let buffers = [
        generate_go_practical_string_bounded_order_foundation()?
            .certificate_bytes()
            .to_vec(),
        generate_go_practical_word_order_foundation()?
            .certificate_bytes()
            .to_vec(),
    ];
    let inputs = buffers
        .iter()
        .map(|bytes| {
            mpk_cert::decode_canonical_certificate(bytes).map_err(|_| GoOrdinaryError::Foundation)
        })
        .collect::<Result<Vec<_>>>()?;
    let mut builder = composition::string_slice_prerequisites(&inputs)?;
    string::append_slice(&mut builder)?;
    Ok(builder)
}
pub fn import_go_practical_string_slice_foundation(
    candidate: &[u8],
) -> Result<GoStringSliceFoundation> {
    if candidate.len() > MAX_BYTES {
        return Err(GoOrdinaryError::Limit);
    }
    let expected = generate_go_practical_string_slice_foundation()?;
    if candidate != expected.certificate_bytes() {
        return Err(GoOrdinaryError::Linkage);
    }
    Ok(expected)
}

/// Complete frozen string concatenation under a separate result-capacity premise.
/// Source/type/evaluation/context authority remains a separate obligation.
pub fn generate_go_practical_string_concat_foundation() -> Result<GoStringConcatFoundation> {
    let mut builder = string_concat_builder()?;
    builder.certificate.module = "Mpk.Go.Practical.StringConcatFoundation".into();
    Ok(GoStringConcatFoundation {
        certificate: builder.finish()?,
    })
}
fn string_concat_builder() -> Result<Builder> {
    let mut builder = string_slice_builder()?;
    string::append_concat(&mut builder)?;
    Ok(builder)
}
pub fn import_go_practical_string_concat_foundation(
    candidate: &[u8],
) -> Result<GoStringConcatFoundation> {
    if candidate.len() > MAX_BYTES {
        return Err(GoOrdinaryError::Limit);
    }
    let expected = generate_go_practical_string_concat_foundation()?;
    if candidate != expected.certificate_bytes() {
        return Err(GoOrdinaryError::Linkage);
    }
    Ok(expected)
}

/// Checked prefix/suffix on complete frozen strings, with full-width positions.
/// Original source/library symbol/type/context authority remains separate.
pub fn generate_go_practical_string_affix_foundation() -> Result<GoStringAffixFoundation> {
    let mut builder = string_affix_builder()?;
    builder.certificate.module = "Mpk.Go.Practical.StringAffixFoundation".into();
    Ok(GoStringAffixFoundation {
        certificate: builder.finish()?,
    })
}
fn string_affix_builder() -> Result<Builder> {
    let mut builder = string_concat_builder()?;
    string::append_affix(&mut builder)?;
    Ok(builder)
}
/// Full frozen fixed-array value laws. Source/nominal/storage authority is separate.
pub fn generate_go_practical_array_foundation() -> Result<GoArrayFoundation> {
    let mut builder = array_builder()?;
    builder.certificate.module = "Mpk.Go.Practical.ArrayFoundation".into();
    Ok(GoArrayFoundation {
        certificate: builder.finish()?,
    })
}
fn array_builder() -> Result<Builder> {
    let mut builder = string_affix_builder()?;
    array::append(&mut builder)?;
    Ok(builder)
}
/// Complete fixed-array comparison laws; source comparability/admission is separate.
pub fn generate_go_practical_array_equality_foundation() -> Result<GoArrayEqualityFoundation> {
    let mut builder = array_builder()?;
    array::append_equality(&mut builder)?;
    builder.certificate.module = "Mpk.Go.Practical.ArrayEqualityFoundation".into();
    Ok(GoArrayEqualityFoundation {
        certificate: builder.finish()?,
    })
}
pub fn import_go_practical_array_equality_foundation(
    candidate: &[u8],
) -> Result<GoArrayEqualityFoundation> {
    if candidate.len() > MAX_BYTES {
        return Err(GoOrdinaryError::Limit);
    }
    let expected = generate_go_practical_array_equality_foundation()?;
    if candidate != expected.certificate_bytes() {
        return Err(GoOrdinaryError::Linkage);
    }
    Ok(expected)
}
/// Frozen complete slice metadata. Typed live storage and source binding are separate.
pub fn generate_go_practical_slice_descriptor_foundation() -> Result<GoSliceDescriptorFoundation> {
    let mut builder = string_slice_builder()?;
    slice_descriptor::append(&mut builder)?;
    builder.certificate.module = "Mpk.Go.Practical.SliceDescriptorFoundation".into();
    Ok(GoSliceDescriptorFoundation {
        certificate: builder.finish()?,
    })
}
pub fn import_go_practical_slice_descriptor_foundation(
    candidate: &[u8],
) -> Result<GoSliceDescriptorFoundation> {
    if candidate.len() > MAX_BYTES {
        return Err(GoOrdinaryError::Limit);
    }
    let expected = generate_go_practical_slice_descriptor_foundation()?;
    if candidate != expected.certificate_bytes() {
        return Err(GoOrdinaryError::Linkage);
    }
    Ok(expected)
}
/// Compositional typed value views. Source paths, live owners and permissions are separate.
pub fn generate_go_practical_storage_lens_foundation() -> Result<GoStorageLensFoundation> {
    let mut builder = array_builder()?;
    product_relation::append(&mut builder)?;
    storage_lens::append(&mut builder)?;
    builder.certificate.module = "Mpk.Go.Practical.StorageLensFoundation".into();
    Ok(GoStorageLensFoundation {
        certificate: builder.finish()?,
    })
}
pub fn import_go_practical_storage_lens_foundation(
    candidate: &[u8],
) -> Result<GoStorageLensFoundation> {
    if candidate.len() > MAX_BYTES {
        return Err(GoOrdinaryError::Limit);
    }
    let expected = generate_go_practical_storage_lens_foundation()?;
    if candidate != expected.certificate_bytes() {
        return Err(GoOrdinaryError::Linkage);
    }
    Ok(expected)
}
pub fn import_go_practical_array_foundation(candidate: &[u8]) -> Result<GoArrayFoundation> {
    if candidate.len() > MAX_BYTES {
        return Err(GoOrdinaryError::Limit);
    }
    let expected = generate_go_practical_array_foundation()?;
    if candidate != expected.certificate_bytes() {
        return Err(GoOrdinaryError::Linkage);
    }
    Ok(expected)
}
pub fn import_go_practical_string_affix_foundation(
    candidate: &[u8],
) -> Result<GoStringAffixFoundation> {
    if candidate.len() > MAX_BYTES {
        return Err(GoOrdinaryError::Limit);
    }
    let expected = generate_go_practical_string_affix_foundation()?;
    if candidate != expected.certificate_bytes() {
        return Err(GoOrdinaryError::Linkage);
    }
    Ok(expected)
}

/// Preserve every old bit proof and additionally prove String.Valid of the
/// actual full carrier. The supplied bytes still confer no source authority.
pub fn generate_go_practical_valid_string_value(
    bytes: &[u8],
) -> std::result::Result<GoValidStringValue, GoStringValueError> {
    if bytes.len() > 256 {
        return Err(GoStringValueError::Domain { bytes: bytes.len() });
    }
    let mut builder = word_index_builder()?;
    string::append(&mut builder)?;
    string::append_value(&mut builder, bytes)?;
    string::append_validity(&mut builder)?;
    string::append_value_validity(&mut builder, bytes.len())?;
    builder.certificate.module = "Mpk.Go.Practical.ValidStringValue".into();
    Ok(GoValidStringValue {
        certificate: builder.finish()?,
        bytes: bytes.to_vec(),
    })
}
pub fn import_go_practical_valid_string_value(
    candidate: &[u8],
    bytes: &[u8],
) -> std::result::Result<GoValidStringValue, GoStringValueError> {
    if candidate.len() > MAX_BYTES {
        return Err(GoOrdinaryError::Limit.into());
    }
    let expected = generate_go_practical_valid_string_value(bytes)?;
    if candidate != expected.certificate_bytes() {
        return Err(GoOrdinaryError::Linkage.into());
    }
    Ok(expected)
}

/// Lift actual typed field relations and comparator proofs to products.
/// Nominal/source types and complete Go context remain separate work.
pub fn generate_go_practical_product_relation_foundation() -> Result<GoProductRelationFoundation> {
    let mut builder = core_builder()?;
    product::append(&mut builder)?;
    product_relation::append(&mut builder)?;
    builder.certificate.module = "Mpk.Go.Practical.ProductRelationFoundation".into();
    Ok(GoProductRelationFoundation {
        certificate: builder.finish()?,
    })
}
pub fn import_go_practical_product_relation_foundation(
    candidate: &[u8],
) -> Result<GoProductRelationFoundation> {
    if candidate.len() > MAX_BYTES {
        return Err(GoOrdinaryError::Limit);
    }
    let expected = generate_go_practical_product_relation_foundation()?;
    if candidate != expected.certificate_bytes() {
        return Err(GoOrdinaryError::Linkage);
    }
    Ok(expected)
}

/// Full 64-bit tags paired with an entire payload in an observation view.
/// Exact frozen-layout adapters, source universes and dispatch remain separate.
pub fn generate_go_practical_error_value_foundation() -> Result<GoErrorValueFoundation> {
    let mut builder = error_value_builder()?;
    builder.certificate.module = "Mpk.Go.Practical.ErrorValueFoundation".into();
    Ok(GoErrorValueFoundation {
        certificate: builder.finish()?,
    })
}

fn error_value_builder() -> Result<Builder> {
    error_value_builder_with_recipes().map(|(builder, _)| builder)
}

fn error_value_builder_with_recipes() -> Result<(Builder, BTreeMap<String, syntax::Expr>)> {
    let mut builder = range_builder()?;
    let recipes = word::append_with_recipes(&mut builder)?;
    product::append(&mut builder)?;
    product_relation::append(&mut builder)?;
    error_value::append(&mut builder, &recipes)?;
    Ok((builder, recipes))
}

/// Specialize the exact balanced error carrier for arbitrary case payload types.
/// Case count supplies no source-universe, tag-assignment or validity authority.
pub fn generate_go_practical_error_layout_foundation(
    case_count: usize,
) -> Result<GoErrorLayoutFoundation> {
    // Each payload necessarily emits a getter and six complete observation
    // laws. This is the existing certificate declaration budget, not a new
    // case-count admission rule. Check it before allocating recipe parameters.
    if case_count
        .checked_mul(7)
        .is_none_or(|n| n > MAX_DECLARATIONS)
    {
        return Err(GoOrdinaryError::Limit);
    }
    let (mut builder, recipes) = error_value_builder_with_recipes()?;
    error_layout::append(&mut builder, case_count, &recipes)?;
    builder.certificate.module = format!("Mpk.Go.Practical.ErrorLayout.N{case_count}");
    Ok(GoErrorLayoutFoundation {
        case_count,
        certificate: builder.finish()?,
    })
}

pub fn import_go_practical_error_layout_foundation(
    candidate: &[u8],
    case_count: usize,
) -> Result<GoErrorLayoutFoundation> {
    if candidate.len() > MAX_BYTES {
        return Err(GoOrdinaryError::Limit);
    }
    let expected = generate_go_practical_error_layout_foundation(case_count)?;
    if candidate != expected.certificate_bytes() {
        return Err(GoOrdinaryError::Linkage);
    }
    Ok(expected)
}

pub fn import_go_practical_error_value_foundation(
    candidate: &[u8],
) -> Result<GoErrorValueFoundation> {
    if candidate.len() > MAX_BYTES {
        return Err(GoOrdinaryError::Limit);
    }
    let expected = generate_go_practical_error_value_foundation()?;
    if candidate != expected.certificate_bytes() {
        return Err(GoOrdinaryError::Linkage);
    }
    Ok(expected)
}

/// Compare arbitrary pairs of actual word models through Int equivalence.
/// Source struct comparability, evaluation and program correspondence remain due.
pub fn generate_go_practical_word_product_equality_foundation(
) -> Result<GoWordProductEqualityFoundation> {
    let mut builder = signed_order_builder()?;
    word_numeric::append_signed_equality(&mut builder)?;
    product::append(&mut builder)?;
    product_relation::append(&mut builder)?;
    product_relation::append_word_pairs(&mut builder)?;
    builder.certificate.module = "Mpk.Go.Practical.WordProductEqualityFoundation".into();
    Ok(GoWordProductEqualityFoundation {
        certificate: builder.finish()?,
    })
}
pub fn import_go_practical_word_product_equality_foundation(
    candidate: &[u8],
) -> Result<GoWordProductEqualityFoundation> {
    if candidate.len() > MAX_BYTES {
        return Err(GoOrdinaryError::Limit);
    }
    let expected = generate_go_practical_word_product_equality_foundation()?;
    if candidate != expected.certificate_bytes() {
        return Err(GoOrdinaryError::Linkage);
    }
    Ok(expected)
}
