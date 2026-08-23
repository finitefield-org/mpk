//! Canonical certificate v0 encoder.
//!
//! The encoder writes certificate sections in the `CERT_V0` logical order,
//! uses one-byte node/declaration tags from [`crate::binary_tags`], and emits
//! variable-width unsigned integers as minimal unsigned LEB128.

use crate::{DeclarationTag, LevelTag, ProofNodeTag, TermTag};

pub const CERT_MAGIC: &[u8; 7] = b"MPKCERT";
pub const CERT_FORMAT: &str = "MPK-CERT-0.1";
pub const CORE_SPEC: &str = "MPK-Core-0.1";
pub const HASH_BYTE_LEN: usize = 32;
pub const ZERO_HASH: HashBytes = [0; HASH_BYTE_LEN];

pub type HashBytes = [u8; HASH_BYTE_LEN];
pub type NameId = u32;
pub type LevelId = u32;
pub type TermId = u32;
pub type ProofNodeId = u32;
pub type DeclarationId = u32;
pub type TheoryCertificateId = u32;
pub type GlobalId = u32;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Certificate {
    pub module: String,
    pub imports: Vec<Import>,
    pub name_table: Vec<String>,
    pub level_table: Vec<LevelNode>,
    pub term_table: Vec<TermNode>,
    pub proof_node_table: Vec<ProofNode>,
    pub declarations: Vec<Declaration>,
    pub theory_certificates: Vec<TheoryCertificate>,
    pub export_block: Vec<ExportEntry>,
    pub axiom_report: AxiomReport,
    pub source_manifest: Option<SourceManifest>,
    pub hashes: CertificateHashes,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CertificateHashes {
    pub export_hash: HashBytes,
    pub axiom_report_hash: HashBytes,
    pub certificate_hash: HashBytes,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Import {
    pub module_name: String,
    pub export_hash: HashBytes,
    pub certificate_hash: Option<HashBytes>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LevelNode {
    Zero,
    Succ(LevelId),
    Max(LevelId, LevelId),
    Param(NameId),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TermNode {
    Sort(LevelId),
    Var(u32),
    Const {
        global: GlobalId,
        levels: Vec<LevelId>,
    },
    App {
        function: TermId,
        arguments: Vec<TermId>,
    },
    Lam {
        ty: TermId,
        body: TermId,
    },
    Pi {
        ty: TermId,
        body: TermId,
    },
    Let {
        ty: TermId,
        value: TermId,
        body: TermId,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProofNode {
    Exact {
        term: TermId,
        expected_type: TermId,
    },
    Apply {
        function_proof: ProofNodeId,
        argument_proofs: Vec<ProofNodeId>,
        expected_type: TermId,
    },
    Intro {
        domain_type: TermId,
        body_proof: ProofNodeId,
        expected_type: TermId,
    },
    LetProof {
        value: TermId,
        body_proof: ProofNodeId,
        expected_type: TermId,
    },
    Refl {
        term: TermId,
        expected_type: TermId,
    },
    Rewrite {
        eq_proof: ProofNodeId,
        target_proof: ProofNodeId,
        expected_type: TermId,
    },
    EqRec {
        motive: TermId,
        eq_proof: ProofNodeId,
        base_proof: ProofNodeId,
        expected_type: TermId,
    },
    Constructor {
        constructor: GlobalId,
        argument_proofs: Vec<ProofNodeId>,
        expected_type: TermId,
    },
    Recursor {
        recursor: GlobalId,
        motive: TermId,
        minor_proofs: Vec<ProofNodeId>,
        major_proof: ProofNodeId,
        expected_type: TermId,
    },
    Conv {
        proof: ProofNodeId,
        expected_type: TermId,
        defeq_witness: Option<TermId>,
    },
    Theory {
        theory_certificate: TheoryCertificateId,
        expected_type: TermId,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Declaration {
    pub name: NameId,
    pub kind: DeclarationKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DeclarationKind {
    Axiom {
        ty: TermId,
    },
    Def {
        ty: TermId,
        value: TermId,
        reducibility: DefinitionReducibility,
    },
    Theorem {
        ty: TermId,
        proof: TermId,
    },
    Inductive {
        ty: TermId,
    },
    Constructor {
        ty: TermId,
        inductive: GlobalId,
        generated: bool,
    },
    Recursor {
        ty: TermId,
        inductive: GlobalId,
        generated: bool,
    },
    TheoryPrimitive {
        ty: TermId,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum DefinitionReducibility {
    Reducible,
    Opaque,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TheoryCertificate {
    pub format: String,
    pub payload: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExportEntry {
    pub name: NameId,
    pub declaration: DeclarationId,
    pub declaration_hash: HashBytes,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AxiomReport {
    pub entries: Vec<AxiomReportEntry>,
    pub declaration_dependencies: Vec<DeclarationAxiomDependencies>,
    pub summary: AxiomReportSummary,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AxiomReportEntry {
    pub category: AxiomCategory,
    pub name: String,
    pub origin_module: String,
    pub type_hash: HashBytes,
    pub declaration_hash: HashBytes,
    pub source_certificate_hash: Option<HashBytes>,
    pub direct_dependent_declarations: Vec<DeclarationId>,
    pub transitive_dependent_declarations: Vec<DeclarationId>,
    pub approval_profile: Option<String>,
    pub reviewer_note: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum AxiomCategory {
    CoreAxiom,
    BuiltinTheoryAxiom,
    GoSemanticsAxiom,
    ExternalAxiom,
}

impl AxiomCategory {
    pub const fn canonical_name(self) -> &'static str {
        match self {
            Self::CoreAxiom => "CoreAxiom",
            Self::BuiltinTheoryAxiom => "BuiltinTheoryAxiom",
            Self::GoSemanticsAxiom => "GoSemanticsAxiom",
            Self::ExternalAxiom => "ExternalAxiom",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeclarationAxiomDependencies {
    pub declaration_name: String,
    pub declaration_hash: HashBytes,
    pub direct_axiom_dependencies: Vec<u32>,
    pub transitive_axiom_dependencies: Vec<u32>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AxiomReportSummary {
    pub core_axiom_count: u64,
    pub builtin_theory_axiom_count: u64,
    pub go_semantics_axiom_count: u64,
    pub external_axiom_count: u64,
    pub total_axiom_count: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceManifest {
    pub payload: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EncodeLimitError;

pub fn encode_certificate(certificate: &Certificate) -> Vec<u8> {
    let mut encoder = Encoder::new();
    write_certificate(&mut encoder, certificate);
    encoder.finish()
}

/// Encodes without ever retaining more than `maximum` bytes.  This is the
/// certificate-output enforcement seam used before checker execution.
pub fn encode_certificate_bounded(
    certificate: &Certificate,
    maximum: usize,
) -> Result<Vec<u8>, EncodeLimitError> {
    let mut encoder = Encoder::new_bounded(maximum);
    write_certificate(&mut encoder, certificate);
    encoder.finish_bounded()
}

fn write_certificate(encoder: &mut Encoder, certificate: &Certificate) {
    encoder.write_bytes(CERT_MAGIC);
    encoder.write_str_slice(CERT_FORMAT);
    encoder.write_str_slice(CORE_SPEC);
    encoder.write_str_slice(&certificate.module);
    encoder.write_vec(&certificate.imports, Encoder::write_import);
    encoder.write_vec(&certificate.name_table, |encoder, name| {
        encoder.write_str_slice(name)
    });
    encoder.write_vec(&certificate.level_table, Encoder::write_level_node);
    encoder.write_vec(&certificate.term_table, Encoder::write_term_node);
    encoder.write_vec(&certificate.proof_node_table, Encoder::write_proof_node);
    encoder.write_vec(&certificate.declarations, Encoder::write_declaration);
    encoder.write_vec(
        &certificate.theory_certificates,
        Encoder::write_theory_certificate,
    );
    encoder.write_vec(&certificate.export_block, Encoder::write_export_entry);
    encoder.write_axiom_report(&certificate.axiom_report);
    encoder.write_source_manifest(&certificate.source_manifest);
    encoder.write_hashes(&certificate.hashes);
}

pub fn encode_theory_certificate(certificate: &TheoryCertificate) -> Vec<u8> {
    let mut encoder = Encoder::new();
    encoder.write_theory_certificate(certificate);
    encoder.finish()
}

pub fn encode_unsigned_varint(value: u64, out: &mut Vec<u8>) {
    Encoder::write_unsigned_varint_to(value, out);
}

struct Encoder {
    bytes: Vec<u8>,
    maximum: Option<usize>,
    exceeded: bool,
}

impl Encoder {
    fn new() -> Self {
        Self {
            bytes: Vec::new(),
            maximum: None,
            exceeded: false,
        }
    }

    fn new_bounded(maximum: usize) -> Self {
        Self {
            bytes: Vec::new(),
            maximum: Some(maximum),
            exceeded: false,
        }
    }

    fn finish(self) -> Vec<u8> {
        self.bytes
    }

    fn finish_bounded(self) -> Result<Vec<u8>, EncodeLimitError> {
        if self.exceeded {
            Err(EncodeLimitError)
        } else {
            Ok(self.bytes)
        }
    }

    fn accepts(&mut self, additional: usize) -> bool {
        if self.exceeded {
            return false;
        }
        let Some(size) = self.bytes.len().checked_add(additional) else {
            self.exceeded = true;
            return false;
        };
        if self.maximum.is_some_and(|maximum| size > maximum) {
            self.exceeded = true;
            return false;
        }
        true
    }

    fn write_u8(&mut self, value: u8) {
        if self.accepts(1) {
            self.bytes.push(value);
        }
    }

    fn write_bool(&mut self, value: bool) {
        self.write_u8(u8::from(value));
    }

    fn write_u32(&mut self, value: u32) {
        self.write_u64(u64::from(value));
    }

    fn write_u64(&mut self, value: u64) {
        let mut value = value;
        loop {
            let mut byte = (value & 0x7f) as u8;
            value >>= 7;
            if value != 0 {
                byte |= 0x80;
            }
            self.write_u8(byte);
            if value == 0 {
                break;
            }
        }
    }

    fn write_len(&mut self, len: usize) {
        let len = u64::try_from(len).expect("certificate section length exceeds u64");
        self.write_u64(len);
    }

    fn write_unsigned_varint_to(mut value: u64, out: &mut Vec<u8>) {
        loop {
            let mut byte = (value & 0x7f) as u8;
            value >>= 7;
            if value != 0 {
                byte |= 0x80;
            }
            out.push(byte);
            if value == 0 {
                break;
            }
        }
    }

    fn write_bytes(&mut self, bytes: &[u8]) {
        if self.accepts(bytes.len()) {
            self.bytes.extend_from_slice(bytes);
        }
    }

    fn write_bytes_with_len(&mut self, bytes: &[u8]) {
        self.write_len(bytes.len());
        self.write_bytes(bytes);
    }

    fn write_str_slice(&mut self, value: &str) {
        self.write_bytes_with_len(value.as_bytes());
    }

    fn write_hash(&mut self, hash: &HashBytes) {
        self.write_bytes(hash);
    }

    fn write_optional_hash(&mut self, hash: &Option<HashBytes>) {
        match hash {
            Some(hash) => {
                self.write_bool(true);
                self.write_hash(hash);
            }
            None => self.write_bool(false),
        }
    }

    fn write_optional_string(&mut self, value: &Option<String>) {
        match value {
            Some(value) => {
                self.write_bool(true);
                self.write_str_slice(value);
            }
            None => self.write_bool(false),
        }
    }

    fn write_optional_u32(&mut self, value: Option<u32>) {
        match value {
            Some(value) => {
                self.write_bool(true);
                self.write_u32(value);
            }
            None => self.write_bool(false),
        }
    }

    fn write_vec<T>(&mut self, values: &[T], mut write_item: impl FnMut(&mut Self, &T)) {
        self.write_len(values.len());
        for value in values {
            write_item(self, value);
        }
    }

    fn write_u32_vec(&mut self, values: &[u32]) {
        self.write_vec(values, |encoder, value| encoder.write_u32(*value));
    }

    fn write_import(&mut self, import: &Import) {
        self.write_str_slice(&import.module_name);
        self.write_hash(&import.export_hash);
        self.write_optional_hash(&import.certificate_hash);
    }

    fn write_level_node(&mut self, node: &LevelNode) {
        match node {
            LevelNode::Zero => self.write_u8(LevelTag::Zero.as_u8()),
            LevelNode::Succ(inner) => {
                self.write_u8(LevelTag::Succ.as_u8());
                self.write_u32(*inner);
            }
            LevelNode::Max(lhs, rhs) => {
                self.write_u8(LevelTag::Max.as_u8());
                self.write_u32(*lhs);
                self.write_u32(*rhs);
            }
            LevelNode::Param(name) => {
                self.write_u8(LevelTag::Param.as_u8());
                self.write_u32(*name);
            }
        }
    }

    fn write_term_node(&mut self, node: &TermNode) {
        match node {
            TermNode::Sort(level) => {
                self.write_u8(TermTag::Sort.as_u8());
                self.write_u32(*level);
            }
            TermNode::Var(index) => {
                self.write_u8(TermTag::Var.as_u8());
                self.write_u32(*index);
            }
            TermNode::Const { global, levels } => {
                self.write_u8(TermTag::Const.as_u8());
                self.write_u32(*global);
                self.write_u32_vec(levels);
            }
            TermNode::App {
                function,
                arguments,
            } => {
                self.write_u8(TermTag::App.as_u8());
                self.write_u32(*function);
                self.write_u32_vec(arguments);
            }
            TermNode::Lam { ty, body } => {
                self.write_u8(TermTag::Lam.as_u8());
                self.write_u32(*ty);
                self.write_u32(*body);
            }
            TermNode::Pi { ty, body } => {
                self.write_u8(TermTag::Pi.as_u8());
                self.write_u32(*ty);
                self.write_u32(*body);
            }
            TermNode::Let { ty, value, body } => {
                self.write_u8(TermTag::Let.as_u8());
                self.write_u32(*ty);
                self.write_u32(*value);
                self.write_u32(*body);
            }
        }
    }

    fn write_proof_node(&mut self, node: &ProofNode) {
        match node {
            ProofNode::Exact {
                term,
                expected_type,
            } => {
                self.write_u8(ProofNodeTag::Exact.as_u8());
                self.write_u32(*term);
                self.write_u32(*expected_type);
            }
            ProofNode::Apply {
                function_proof,
                argument_proofs,
                expected_type,
            } => {
                self.write_u8(ProofNodeTag::Apply.as_u8());
                self.write_u32(*function_proof);
                self.write_u32_vec(argument_proofs);
                self.write_u32(*expected_type);
            }
            ProofNode::Intro {
                domain_type,
                body_proof,
                expected_type,
            } => {
                self.write_u8(ProofNodeTag::Intro.as_u8());
                self.write_u32(*domain_type);
                self.write_u32(*body_proof);
                self.write_u32(*expected_type);
            }
            ProofNode::LetProof {
                value,
                body_proof,
                expected_type,
            } => {
                self.write_u8(ProofNodeTag::LetProof.as_u8());
                self.write_u32(*value);
                self.write_u32(*body_proof);
                self.write_u32(*expected_type);
            }
            ProofNode::Refl {
                term,
                expected_type,
            } => {
                self.write_u8(ProofNodeTag::Refl.as_u8());
                self.write_u32(*term);
                self.write_u32(*expected_type);
            }
            ProofNode::Rewrite {
                eq_proof,
                target_proof,
                expected_type,
            } => {
                self.write_u8(ProofNodeTag::Rewrite.as_u8());
                self.write_u32(*eq_proof);
                self.write_u32(*target_proof);
                self.write_u32(*expected_type);
            }
            ProofNode::EqRec {
                motive,
                eq_proof,
                base_proof,
                expected_type,
            } => {
                self.write_u8(ProofNodeTag::EqRec.as_u8());
                self.write_u32(*motive);
                self.write_u32(*eq_proof);
                self.write_u32(*base_proof);
                self.write_u32(*expected_type);
            }
            ProofNode::Constructor {
                constructor,
                argument_proofs,
                expected_type,
            } => {
                self.write_u8(ProofNodeTag::Constructor.as_u8());
                self.write_u32(*constructor);
                self.write_u32_vec(argument_proofs);
                self.write_u32(*expected_type);
            }
            ProofNode::Recursor {
                recursor,
                motive,
                minor_proofs,
                major_proof,
                expected_type,
            } => {
                self.write_u8(ProofNodeTag::Recursor.as_u8());
                self.write_u32(*recursor);
                self.write_u32(*motive);
                self.write_u32_vec(minor_proofs);
                self.write_u32(*major_proof);
                self.write_u32(*expected_type);
            }
            ProofNode::Conv {
                proof,
                expected_type,
                defeq_witness,
            } => {
                self.write_u8(ProofNodeTag::Conv.as_u8());
                self.write_u32(*proof);
                self.write_u32(*expected_type);
                self.write_optional_u32(*defeq_witness);
            }
            ProofNode::Theory {
                theory_certificate,
                expected_type,
            } => {
                self.write_u8(ProofNodeTag::Theory.as_u8());
                self.write_u32(*theory_certificate);
                self.write_u32(*expected_type);
            }
        }
    }

    fn write_declaration(&mut self, declaration: &Declaration) {
        self.write_u32(declaration.name);
        match &declaration.kind {
            DeclarationKind::Axiom { ty } => {
                self.write_u8(DeclarationTag::Axiom.as_u8());
                self.write_u32(*ty);
            }
            DeclarationKind::Def {
                ty,
                value,
                reducibility,
            } => {
                self.write_u8(DeclarationTag::Def.as_u8());
                self.write_u32(*ty);
                self.write_u32(*value);
                self.write_reducibility(*reducibility);
            }
            DeclarationKind::Theorem { ty, proof } => {
                self.write_u8(DeclarationTag::Theorem.as_u8());
                self.write_u32(*ty);
                self.write_u32(*proof);
            }
            DeclarationKind::Inductive { ty } => {
                self.write_u8(DeclarationTag::Inductive.as_u8());
                self.write_u32(*ty);
            }
            DeclarationKind::Constructor {
                ty,
                inductive,
                generated,
            } => {
                self.write_u8(DeclarationTag::Constructor.as_u8());
                self.write_u32(*ty);
                self.write_u32(*inductive);
                self.write_bool(*generated);
            }
            DeclarationKind::Recursor {
                ty,
                inductive,
                generated,
            } => {
                self.write_u8(DeclarationTag::Recursor.as_u8());
                self.write_u32(*ty);
                self.write_u32(*inductive);
                self.write_bool(*generated);
            }
            DeclarationKind::TheoryPrimitive { ty } => {
                self.write_u8(DeclarationTag::TheoryPrimitive.as_u8());
                self.write_u32(*ty);
            }
        }
    }

    fn write_reducibility(&mut self, reducibility: DefinitionReducibility) {
        match reducibility {
            DefinitionReducibility::Reducible => self.write_u8(0x00),
            DefinitionReducibility::Opaque => self.write_u8(0x01),
        }
    }

    fn write_theory_certificate(&mut self, certificate: &TheoryCertificate) {
        self.write_str_slice(&certificate.format);
        self.write_bytes_with_len(&certificate.payload);
    }

    fn write_export_entry(&mut self, entry: &ExportEntry) {
        self.write_u32(entry.name);
        self.write_u32(entry.declaration);
        self.write_hash(&entry.declaration_hash);
    }

    fn write_axiom_report(&mut self, report: &AxiomReport) {
        self.write_vec(&report.entries, Encoder::write_axiom_report_entry);
        self.write_vec(
            &report.declaration_dependencies,
            Encoder::write_declaration_axiom_dependencies,
        );
        self.write_axiom_report_summary(&report.summary);
    }

    fn write_axiom_report_entry(&mut self, entry: &AxiomReportEntry) {
        self.write_str_slice(entry.category.canonical_name());
        self.write_str_slice(&entry.name);
        self.write_str_slice(&entry.origin_module);
        self.write_hash(&entry.type_hash);
        self.write_hash(&entry.declaration_hash);
        self.write_optional_hash(&entry.source_certificate_hash);
        self.write_u32_vec(&entry.direct_dependent_declarations);
        self.write_u32_vec(&entry.transitive_dependent_declarations);
        self.write_optional_string(&entry.approval_profile);
        self.write_optional_string(&entry.reviewer_note);
    }

    fn write_declaration_axiom_dependencies(
        &mut self,
        dependencies: &DeclarationAxiomDependencies,
    ) {
        self.write_str_slice(&dependencies.declaration_name);
        self.write_hash(&dependencies.declaration_hash);
        self.write_u32_vec(&dependencies.direct_axiom_dependencies);
        self.write_u32_vec(&dependencies.transitive_axiom_dependencies);
    }

    fn write_axiom_report_summary(&mut self, summary: &AxiomReportSummary) {
        self.write_u64(summary.core_axiom_count);
        self.write_u64(summary.builtin_theory_axiom_count);
        self.write_u64(summary.go_semantics_axiom_count);
        self.write_u64(summary.external_axiom_count);
        self.write_u64(summary.total_axiom_count);
    }

    fn write_source_manifest(&mut self, manifest: &Option<SourceManifest>) {
        match manifest {
            Some(manifest) => {
                self.write_bool(true);
                self.write_bytes_with_len(&manifest.payload);
            }
            None => self.write_bool(false),
        }
    }

    fn write_hashes(&mut self, hashes: &CertificateHashes) {
        self.write_hash(&hashes.export_hash);
        self.write_hash(&hashes.axiom_report_hash);
        self.write_hash(&hashes.certificate_hash);
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::{
        encode_certificate, encode_certificate_bounded, encode_unsigned_varint, AxiomReport,
        Certificate, CertificateHashes, Declaration, DeclarationKind, DefinitionReducibility,
        LevelNode, ProofNode, TermNode, ZERO_HASH,
    };

    const CERT_ENCODING_FIXTURE_DIR: &str =
        concat!(env!("CARGO_MANIFEST_DIR"), "/../../fixtures/cert-encoding");

    fn minimal_certificate() -> Certificate {
        Certificate {
            module: "Example.Empty".to_owned(),
            imports: Vec::new(),
            name_table: Vec::new(),
            level_table: Vec::new(),
            term_table: Vec::new(),
            proof_node_table: Vec::new(),
            declarations: Vec::new(),
            theory_certificates: Vec::new(),
            export_block: Vec::new(),
            axiom_report: AxiomReport::default(),
            source_manifest: None,
            hashes: CertificateHashes::default(),
        }
    }

    fn decode_hex_fixture(name: &str) -> Vec<u8> {
        let path = format!("{CERT_ENCODING_FIXTURE_DIR}/{name}");
        let contents = fs::read_to_string(&path).expect("golden fixture is readable");
        let hex = contents
            .chars()
            .filter(|character| !character.is_whitespace())
            .collect::<String>();
        assert_eq!(hex.len() % 2, 0, "fixture hex must use full bytes");

        hex.as_bytes()
            .chunks_exact(2)
            .map(|chunk| {
                let byte = std::str::from_utf8(chunk).expect("fixture hex is utf8");
                u8::from_str_radix(byte, 16).expect("fixture hex byte is valid")
            })
            .collect()
    }

    #[test]
    fn unsigned_varints_are_minimal_leb128() {
        let mut encoded = Vec::new();
        for value in [0, 1, 127, 128, 255, 16_384, 624_485] {
            encode_unsigned_varint(value, &mut encoded);
        }

        assert_eq!(
            encoded,
            [0x00, 0x01, 0x7f, 0x80, 0x01, 0xff, 0x01, 0x80, 0x80, 0x01, 0xe5, 0x8e, 0x26,]
        );
    }

    #[test]
    fn minimal_certificate_matches_golden_fixture() {
        let encoded = encode_certificate(&minimal_certificate());
        let fixture = decode_hex_fixture("minimal-empty.hex");

        assert_eq!(encoded, fixture);
    }

    #[test]
    fn bounded_certificate_encoder_accepts_at_and_refuses_before_excess_allocation() {
        let certificate = minimal_certificate();
        let expected = encode_certificate(&certificate);

        assert_eq!(
            encode_certificate_bounded(&certificate, expected.len()).unwrap(),
            expected
        );
        assert!(encode_certificate_bounded(&certificate, expected.len() - 1).is_err());
    }

    #[test]
    fn certificate_sections_follow_fixed_order() {
        let certificate = Certificate {
            module: "Example.Order".to_owned(),
            imports: Vec::new(),
            name_table: vec![
                "Example.Order.type".to_owned(),
                "Example.Order.ax".to_owned(),
            ],
            level_table: vec![LevelNode::Zero, LevelNode::Succ(0)],
            term_table: vec![TermNode::Sort(1), TermNode::Var(300)],
            proof_node_table: vec![ProofNode::Exact {
                term: 1,
                expected_type: 0,
            }],
            declarations: vec![Declaration {
                name: 1,
                kind: DeclarationKind::Axiom { ty: 0 },
            }],
            theory_certificates: Vec::new(),
            export_block: Vec::new(),
            axiom_report: AxiomReport::default(),
            source_manifest: None,
            hashes: CertificateHashes::default(),
        };

        let encoded = encode_certificate(&certificate);

        let expected_prefix = [
            b'M', b'P', b'K', b'C', b'E', b'R', b'T', 0x0c, b'M', b'P', b'K', b'-', b'C', b'E',
            b'R', b'T', b'-', b'0', b'.', b'1', 0x0c, b'M', b'P', b'K', b'-', b'C', b'o', b'r',
            b'e', b'-', b'0', b'.', b'1', 0x0d, b'E', b'x', b'a', b'm', b'p', b'l', b'e', b'.',
            b'O', b'r', b'd', b'e', b'r',
        ];
        assert_eq!(&encoded[..expected_prefix.len()], expected_prefix);

        let first_name_offset = expected_prefix.len() + 1;
        assert_eq!(encoded[first_name_offset], 0x02);
        assert_eq!(encoded[first_name_offset + 1], 0x12);
        assert_eq!(
            &encoded[first_name_offset + 2..first_name_offset + 20],
            b"Example.Order.type"
        );
    }

    #[test]
    fn encoder_uses_tags_for_core_nodes_and_declarations() {
        let certificate = Certificate {
            module: "Example.Tags".to_owned(),
            imports: Vec::new(),
            name_table: vec!["Example.Tags.def".to_owned()],
            level_table: vec![LevelNode::Zero],
            term_table: vec![TermNode::Sort(0)],
            proof_node_table: vec![ProofNode::Conv {
                proof: 0,
                expected_type: 0,
                defeq_witness: Some(0),
            }],
            declarations: vec![Declaration {
                name: 0,
                kind: DeclarationKind::Def {
                    ty: 0,
                    value: 0,
                    reducibility: DefinitionReducibility::Opaque,
                },
            }],
            theory_certificates: Vec::new(),
            export_block: Vec::new(),
            axiom_report: AxiomReport::default(),
            source_manifest: None,
            hashes: CertificateHashes {
                export_hash: ZERO_HASH,
                axiom_report_hash: ZERO_HASH,
                certificate_hash: ZERO_HASH,
            },
        };

        let encoded = encode_certificate(&certificate);

        assert!(encoded
            .windows([0x01, 0x00].len())
            .any(|bytes| bytes == [0x01, 0x00]));
        assert!(encoded
            .windows([0x09, 0x00, 0x00, 0x01, 0x00].len())
            .any(|bytes| bytes == [0x09, 0x00, 0x00, 0x01, 0x00]));
        assert!(encoded
            .windows([0x00, 0x01, 0x00, 0x00, 0x01].len())
            .any(|bytes| bytes == [0x00, 0x01, 0x00, 0x00, 0x01]));
    }
}
