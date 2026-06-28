//! Canonical certificate v0 decoder and structural shape validation.

use crate::{
    encode::{
        AxiomCategory, AxiomReport, AxiomReportEntry, AxiomReportSummary, Certificate,
        CertificateHashes, Declaration, DeclarationAxiomDependencies, DeclarationKind,
        DefinitionReducibility, ExportEntry, HashBytes, Import, LevelNode, ProofNode,
        SourceManifest, TermNode, TheoryCertificate, CERT_FORMAT, CERT_MAGIC, CORE_SPEC,
        HASH_BYTE_LEN,
    },
    DeclarationTag, LevelTag, ProofNodeTag, TermTag,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecodeError {
    kind: DecodeErrorKind,
    offset: usize,
    detail: Option<String>,
}

impl DecodeError {
    pub fn kind(&self) -> DecodeErrorKind {
        self.kind
    }

    pub fn offset(&self) -> usize {
        self.offset
    }

    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }

    fn new(kind: DecodeErrorKind, offset: usize) -> Self {
        Self {
            kind,
            offset,
            detail: None,
        }
    }

    fn with_detail(kind: DecodeErrorKind, offset: usize, detail: impl Into<String>) -> Self {
        Self {
            kind,
            offset,
            detail: Some(detail.into()),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum DecodeErrorKind {
    UnexpectedEof,
    TrailingBytes,
    InvalidMagic,
    InvalidFormat,
    InvalidCoreSpec,
    NonMinimalVarint,
    VarintOverflow,
    LengthOverflow,
    InvalidUtf8,
    InvalidName,
    InvalidBool,
    UnknownTag,
    UnknownAxiomCategory,
    UnknownReducibility,
    InvalidReference,
    FutureReference,
}

pub fn decode_certificate(bytes: &[u8]) -> Result<Certificate, DecodeError> {
    let mut decoder = Decoder::new(bytes);
    decoder.read_magic()?;

    let format = decoder.read_string()?;
    if format != CERT_FORMAT {
        return Err(DecodeError::with_detail(
            DecodeErrorKind::InvalidFormat,
            0,
            format,
        ));
    }

    let core_spec = decoder.read_string()?;
    if core_spec != CORE_SPEC {
        return Err(DecodeError::with_detail(
            DecodeErrorKind::InvalidCoreSpec,
            0,
            core_spec,
        ));
    }

    let module = decoder.read_string()?;
    validate_name(&module, "module")?;
    let imports = decoder.read_vec(Decoder::read_import)?;
    let name_table = decoder.read_vec(Decoder::read_name)?;
    let level_table = decoder.read_vec(Decoder::read_level_node)?;
    let term_table = decoder.read_vec(Decoder::read_term_node)?;
    let proof_node_table = decoder.read_vec(Decoder::read_proof_node)?;
    let declarations = decoder.read_vec(Decoder::read_declaration)?;
    let theory_certificates = decoder.read_vec(Decoder::read_theory_certificate)?;
    let export_block = decoder.read_vec(Decoder::read_export_entry)?;
    let axiom_report = decoder.read_axiom_report()?;
    let source_manifest = decoder.read_source_manifest()?;
    let hashes = decoder.read_hashes()?;

    if !decoder.is_finished() {
        return Err(DecodeError::new(
            DecodeErrorKind::TrailingBytes,
            decoder.offset(),
        ));
    }

    let certificate = Certificate {
        module,
        imports,
        name_table,
        level_table,
        term_table,
        proof_node_table,
        declarations,
        theory_certificates,
        export_block,
        axiom_report,
        source_manifest,
        hashes,
    };
    validate_certificate_shape(&certificate)?;
    Ok(certificate)
}

struct Decoder<'bytes> {
    bytes: &'bytes [u8],
    offset: usize,
}

impl<'bytes> Decoder<'bytes> {
    fn new(bytes: &'bytes [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn offset(&self) -> usize {
        self.offset
    }

    fn is_finished(&self) -> bool {
        self.offset == self.bytes.len()
    }

    fn read_magic(&mut self) -> Result<(), DecodeError> {
        let offset = self.offset;
        let bytes = self.read_exact(CERT_MAGIC.len())?;
        if bytes != CERT_MAGIC {
            return Err(DecodeError::new(DecodeErrorKind::InvalidMagic, offset));
        }
        Ok(())
    }

    fn read_exact(&mut self, len: usize) -> Result<&'bytes [u8], DecodeError> {
        let start = self.offset;
        let end = start
            .checked_add(len)
            .ok_or_else(|| DecodeError::new(DecodeErrorKind::LengthOverflow, start))?;
        let bytes = self
            .bytes
            .get(start..end)
            .ok_or_else(|| DecodeError::new(DecodeErrorKind::UnexpectedEof, start))?;
        self.offset = end;
        Ok(bytes)
    }

    fn read_u8(&mut self) -> Result<u8, DecodeError> {
        Ok(self.read_exact(1)?[0])
    }

    fn read_bool(&mut self) -> Result<bool, DecodeError> {
        let offset = self.offset;
        match self.read_u8()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(DecodeError::new(DecodeErrorKind::InvalidBool, offset)),
        }
    }

    fn read_u32(&mut self) -> Result<u32, DecodeError> {
        let offset = self.offset;
        let value = self.read_u64()?;
        u32::try_from(value).map_err(|_| DecodeError::new(DecodeErrorKind::LengthOverflow, offset))
    }

    fn read_u64(&mut self) -> Result<u64, DecodeError> {
        let start = self.offset;
        let mut result = 0_u64;

        for byte_index in 0..10 {
            let byte = self.read_u8()?;
            let low = u64::from(byte & 0x7f);

            if byte_index == 9 && low > 1 {
                return Err(DecodeError::new(DecodeErrorKind::VarintOverflow, start));
            }

            result |= low << (byte_index * 7);
            if byte & 0x80 == 0 {
                let used = byte_index + 1;
                if minimal_varint_len(result) != used {
                    return Err(DecodeError::new(DecodeErrorKind::NonMinimalVarint, start));
                }
                return Ok(result);
            }
        }

        Err(DecodeError::new(DecodeErrorKind::VarintOverflow, start))
    }

    fn read_len(&mut self) -> Result<usize, DecodeError> {
        let offset = self.offset;
        let len = self.read_u64()?;
        usize::try_from(len).map_err(|_| DecodeError::new(DecodeErrorKind::LengthOverflow, offset))
    }

    fn read_bytes_with_len(&mut self) -> Result<Vec<u8>, DecodeError> {
        let len = self.read_len()?;
        Ok(self.read_exact(len)?.to_vec())
    }

    fn read_string(&mut self) -> Result<String, DecodeError> {
        let offset = self.offset;
        let bytes = self.read_bytes_with_len()?;
        String::from_utf8(bytes).map_err(|_| DecodeError::new(DecodeErrorKind::InvalidUtf8, offset))
    }

    fn read_name(&mut self) -> Result<String, DecodeError> {
        let name = self.read_string()?;
        validate_name(&name, "name_table")?;
        Ok(name)
    }

    fn read_hash(&mut self) -> Result<HashBytes, DecodeError> {
        let mut hash = [0; HASH_BYTE_LEN];
        hash.copy_from_slice(self.read_exact(HASH_BYTE_LEN)?);
        Ok(hash)
    }

    fn read_optional_hash(&mut self) -> Result<Option<HashBytes>, DecodeError> {
        if self.read_bool()? {
            self.read_hash().map(Some)
        } else {
            Ok(None)
        }
    }

    fn read_optional_string(&mut self) -> Result<Option<String>, DecodeError> {
        if self.read_bool()? {
            self.read_string().map(Some)
        } else {
            Ok(None)
        }
    }

    fn read_optional_u32(&mut self) -> Result<Option<u32>, DecodeError> {
        if self.read_bool()? {
            self.read_u32().map(Some)
        } else {
            Ok(None)
        }
    }

    fn read_vec<T>(
        &mut self,
        mut read_item: impl FnMut(&mut Self) -> Result<T, DecodeError>,
    ) -> Result<Vec<T>, DecodeError> {
        let len = self.read_len()?;
        let mut values = Vec::new();
        for _ in 0..len {
            values.push(read_item(self)?);
        }
        Ok(values)
    }

    fn read_u32_vec(&mut self) -> Result<Vec<u32>, DecodeError> {
        self.read_vec(Decoder::read_u32)
    }

    fn read_import(&mut self) -> Result<Import, DecodeError> {
        let module_name = self.read_string()?;
        validate_name(&module_name, "import.module_name")?;
        Ok(Import {
            module_name,
            export_hash: self.read_hash()?,
            certificate_hash: self.read_optional_hash()?,
        })
    }

    fn read_level_node(&mut self) -> Result<LevelNode, DecodeError> {
        let offset = self.offset;
        match LevelTag::from_u8(self.read_u8()?) {
            Some(LevelTag::Zero) => Ok(LevelNode::Zero),
            Some(LevelTag::Succ) => Ok(LevelNode::Succ(self.read_u32()?)),
            Some(LevelTag::Max) => Ok(LevelNode::Max(self.read_u32()?, self.read_u32()?)),
            Some(LevelTag::Param) => Ok(LevelNode::Param(self.read_u32()?)),
            None => Err(DecodeError::new(DecodeErrorKind::UnknownTag, offset)),
        }
    }

    fn read_term_node(&mut self) -> Result<TermNode, DecodeError> {
        let offset = self.offset;
        match TermTag::from_u8(self.read_u8()?) {
            Some(TermTag::Sort) => Ok(TermNode::Sort(self.read_u32()?)),
            Some(TermTag::Var) => Ok(TermNode::Var(self.read_u32()?)),
            Some(TermTag::Const) => Ok(TermNode::Const {
                global: self.read_u32()?,
                levels: self.read_u32_vec()?,
            }),
            Some(TermTag::App) => Ok(TermNode::App {
                function: self.read_u32()?,
                arguments: self.read_u32_vec()?,
            }),
            Some(TermTag::Lam) => Ok(TermNode::Lam {
                ty: self.read_u32()?,
                body: self.read_u32()?,
            }),
            Some(TermTag::Pi) => Ok(TermNode::Pi {
                ty: self.read_u32()?,
                body: self.read_u32()?,
            }),
            Some(TermTag::Let) => Ok(TermNode::Let {
                ty: self.read_u32()?,
                value: self.read_u32()?,
                body: self.read_u32()?,
            }),
            None => Err(DecodeError::new(DecodeErrorKind::UnknownTag, offset)),
        }
    }

    fn read_proof_node(&mut self) -> Result<ProofNode, DecodeError> {
        let offset = self.offset;
        match ProofNodeTag::from_u8(self.read_u8()?) {
            Some(ProofNodeTag::Exact) => Ok(ProofNode::Exact {
                term: self.read_u32()?,
                expected_type: self.read_u32()?,
            }),
            Some(ProofNodeTag::Apply) => Ok(ProofNode::Apply {
                function_proof: self.read_u32()?,
                argument_proofs: self.read_u32_vec()?,
                expected_type: self.read_u32()?,
            }),
            Some(ProofNodeTag::Intro) => Ok(ProofNode::Intro {
                domain_type: self.read_u32()?,
                body_proof: self.read_u32()?,
                expected_type: self.read_u32()?,
            }),
            Some(ProofNodeTag::LetProof) => Ok(ProofNode::LetProof {
                value: self.read_u32()?,
                body_proof: self.read_u32()?,
                expected_type: self.read_u32()?,
            }),
            Some(ProofNodeTag::Refl) => Ok(ProofNode::Refl {
                term: self.read_u32()?,
                expected_type: self.read_u32()?,
            }),
            Some(ProofNodeTag::Rewrite) => Ok(ProofNode::Rewrite {
                eq_proof: self.read_u32()?,
                target_proof: self.read_u32()?,
                expected_type: self.read_u32()?,
            }),
            Some(ProofNodeTag::EqRec) => Ok(ProofNode::EqRec {
                motive: self.read_u32()?,
                eq_proof: self.read_u32()?,
                base_proof: self.read_u32()?,
                expected_type: self.read_u32()?,
            }),
            Some(ProofNodeTag::Constructor) => Ok(ProofNode::Constructor {
                constructor: self.read_u32()?,
                argument_proofs: self.read_u32_vec()?,
                expected_type: self.read_u32()?,
            }),
            Some(ProofNodeTag::Recursor) => Ok(ProofNode::Recursor {
                recursor: self.read_u32()?,
                motive: self.read_u32()?,
                minor_proofs: self.read_u32_vec()?,
                major_proof: self.read_u32()?,
                expected_type: self.read_u32()?,
            }),
            Some(ProofNodeTag::Conv) => Ok(ProofNode::Conv {
                proof: self.read_u32()?,
                expected_type: self.read_u32()?,
                defeq_witness: self.read_optional_u32()?,
            }),
            Some(ProofNodeTag::Theory) => Ok(ProofNode::Theory {
                theory_certificate: self.read_u32()?,
                expected_type: self.read_u32()?,
            }),
            None => Err(DecodeError::new(DecodeErrorKind::UnknownTag, offset)),
        }
    }

    fn read_declaration(&mut self) -> Result<Declaration, DecodeError> {
        let name = self.read_u32()?;
        let offset = self.offset;
        let kind = match DeclarationTag::from_u8(self.read_u8()?) {
            Some(DeclarationTag::Axiom) => DeclarationKind::Axiom {
                ty: self.read_u32()?,
            },
            Some(DeclarationTag::Def) => DeclarationKind::Def {
                ty: self.read_u32()?,
                value: self.read_u32()?,
                reducibility: self.read_reducibility()?,
            },
            Some(DeclarationTag::Theorem) => DeclarationKind::Theorem {
                ty: self.read_u32()?,
                proof: self.read_u32()?,
            },
            Some(DeclarationTag::Inductive) => DeclarationKind::Inductive {
                ty: self.read_u32()?,
            },
            Some(DeclarationTag::Constructor) => DeclarationKind::Constructor {
                ty: self.read_u32()?,
                inductive: self.read_u32()?,
                generated: self.read_bool()?,
            },
            Some(DeclarationTag::Recursor) => DeclarationKind::Recursor {
                ty: self.read_u32()?,
                inductive: self.read_u32()?,
                generated: self.read_bool()?,
            },
            Some(DeclarationTag::TheoryPrimitive) => DeclarationKind::TheoryPrimitive {
                ty: self.read_u32()?,
            },
            None => return Err(DecodeError::new(DecodeErrorKind::UnknownTag, offset)),
        };
        Ok(Declaration { name, kind })
    }

    fn read_reducibility(&mut self) -> Result<DefinitionReducibility, DecodeError> {
        let offset = self.offset;
        match self.read_u8()? {
            0x00 => Ok(DefinitionReducibility::Reducible),
            0x01 => Ok(DefinitionReducibility::Opaque),
            _ => Err(DecodeError::new(
                DecodeErrorKind::UnknownReducibility,
                offset,
            )),
        }
    }

    fn read_theory_certificate(&mut self) -> Result<TheoryCertificate, DecodeError> {
        Ok(TheoryCertificate {
            format: self.read_string()?,
            payload: self.read_bytes_with_len()?,
        })
    }

    fn read_export_entry(&mut self) -> Result<ExportEntry, DecodeError> {
        Ok(ExportEntry {
            name: self.read_u32()?,
            declaration: self.read_u32()?,
            declaration_hash: self.read_hash()?,
        })
    }

    fn read_axiom_report(&mut self) -> Result<AxiomReport, DecodeError> {
        Ok(AxiomReport {
            entries: self.read_vec(Decoder::read_axiom_report_entry)?,
            declaration_dependencies: self
                .read_vec(Decoder::read_declaration_axiom_dependencies)?,
            summary: self.read_axiom_report_summary()?,
        })
    }

    fn read_axiom_report_entry(&mut self) -> Result<AxiomReportEntry, DecodeError> {
        let category = self.read_axiom_category()?;
        let name = self.read_string()?;
        validate_name(&name, "axiom_report.name")?;
        let origin_module = self.read_string()?;
        validate_name(&origin_module, "axiom_report.origin_module")?;
        Ok(AxiomReportEntry {
            category,
            name,
            origin_module,
            type_hash: self.read_hash()?,
            declaration_hash: self.read_hash()?,
            source_certificate_hash: self.read_optional_hash()?,
            direct_dependent_declarations: self.read_u32_vec()?,
            transitive_dependent_declarations: self.read_u32_vec()?,
            approval_profile: self.read_optional_string()?,
            reviewer_note: self.read_optional_string()?,
        })
    }

    fn read_axiom_category(&mut self) -> Result<AxiomCategory, DecodeError> {
        let offset = self.offset;
        match self.read_string()?.as_str() {
            "CoreAxiom" => Ok(AxiomCategory::CoreAxiom),
            "BuiltinTheoryAxiom" => Ok(AxiomCategory::BuiltinTheoryAxiom),
            "GoSemanticsAxiom" => Ok(AxiomCategory::GoSemanticsAxiom),
            "ExternalAxiom" => Ok(AxiomCategory::ExternalAxiom),
            _ => Err(DecodeError::new(
                DecodeErrorKind::UnknownAxiomCategory,
                offset,
            )),
        }
    }

    fn read_declaration_axiom_dependencies(
        &mut self,
    ) -> Result<DeclarationAxiomDependencies, DecodeError> {
        let declaration_name = self.read_string()?;
        validate_name(&declaration_name, "axiom_report.declaration_name")?;
        Ok(DeclarationAxiomDependencies {
            declaration_name,
            declaration_hash: self.read_hash()?,
            direct_axiom_dependencies: self.read_u32_vec()?,
            transitive_axiom_dependencies: self.read_u32_vec()?,
        })
    }

    fn read_axiom_report_summary(&mut self) -> Result<AxiomReportSummary, DecodeError> {
        Ok(AxiomReportSummary {
            core_axiom_count: self.read_u64()?,
            builtin_theory_axiom_count: self.read_u64()?,
            go_semantics_axiom_count: self.read_u64()?,
            external_axiom_count: self.read_u64()?,
            total_axiom_count: self.read_u64()?,
        })
    }

    fn read_source_manifest(&mut self) -> Result<Option<SourceManifest>, DecodeError> {
        if self.read_bool()? {
            Ok(Some(SourceManifest {
                payload: self.read_bytes_with_len()?,
            }))
        } else {
            Ok(None)
        }
    }

    fn read_hashes(&mut self) -> Result<CertificateHashes, DecodeError> {
        Ok(CertificateHashes {
            export_hash: self.read_hash()?,
            axiom_report_hash: self.read_hash()?,
            certificate_hash: self.read_hash()?,
        })
    }
}

fn validate_certificate_shape(certificate: &Certificate) -> Result<(), DecodeError> {
    for (index, level) in certificate.level_table.iter().enumerate() {
        match level {
            LevelNode::Zero => {}
            LevelNode::Succ(inner) => check_future_index(*inner, index, "level.succ")?,
            LevelNode::Max(lhs, rhs) => {
                check_future_index(*lhs, index, "level.max.lhs")?;
                check_future_index(*rhs, index, "level.max.rhs")?;
            }
            LevelNode::Param(name) => {
                check_index(*name, certificate.name_table.len(), "level.param")?
            }
        }
    }

    for (index, term) in certificate.term_table.iter().enumerate() {
        match term {
            TermNode::Sort(level) => {
                check_index(*level, certificate.level_table.len(), "term.sort")?
            }
            TermNode::Var(_) => {}
            TermNode::Const { levels, .. } => {
                check_indices(levels, certificate.level_table.len(), "term.const.level")?;
            }
            TermNode::App {
                function,
                arguments,
            } => {
                check_future_index(*function, index, "term.app.function")?;
                check_future_indices(arguments, index, "term.app.argument")?;
            }
            TermNode::Lam { ty, body } | TermNode::Pi { ty, body } => {
                check_future_index(*ty, index, "term.binder.ty")?;
                check_future_index(*body, index, "term.binder.body")?;
            }
            TermNode::Let { ty, value, body } => {
                check_future_index(*ty, index, "term.let.ty")?;
                check_future_index(*value, index, "term.let.value")?;
                check_future_index(*body, index, "term.let.body")?;
            }
        }
    }

    for (index, proof) in certificate.proof_node_table.iter().enumerate() {
        validate_proof_node_shape(certificate, proof, index)?;
    }

    for (index, declaration) in certificate.declarations.iter().enumerate() {
        check_index(declaration.name, certificate.name_table.len(), "decl.name")?;
        match &declaration.kind {
            DeclarationKind::Axiom { ty }
            | DeclarationKind::Inductive { ty }
            | DeclarationKind::TheoryPrimitive { ty } => {
                check_index(*ty, certificate.term_table.len(), "decl.ty")?;
            }
            DeclarationKind::Def { ty, value, .. } => {
                check_index(*ty, certificate.term_table.len(), "decl.def.ty")?;
                check_index(*value, certificate.term_table.len(), "decl.def.value")?;
            }
            DeclarationKind::Theorem { ty, proof } => {
                check_index(*ty, certificate.term_table.len(), "decl.theorem.ty")?;
                check_index(*proof, certificate.term_table.len(), "decl.theorem.proof")?;
            }
            DeclarationKind::Constructor { ty, inductive, .. }
            | DeclarationKind::Recursor { ty, inductive, .. } => {
                check_index(*ty, certificate.term_table.len(), "decl.generated.ty")?;
                check_future_index(*inductive, index, "decl.generated.inductive")?;
            }
        }
    }

    for export in &certificate.export_block {
        check_index(export.name, certificate.name_table.len(), "export.name")?;
        check_index(
            export.declaration,
            certificate.declarations.len(),
            "export.declaration",
        )?;
    }

    validate_axiom_report_shape(certificate)?;
    Ok(())
}

fn validate_proof_node_shape(
    certificate: &Certificate,
    proof: &ProofNode,
    index: usize,
) -> Result<(), DecodeError> {
    match proof {
        ProofNode::Exact {
            term,
            expected_type,
        }
        | ProofNode::Refl {
            term,
            expected_type,
        } => {
            check_index(*term, certificate.term_table.len(), "proof.term")?;
            check_index(
                *expected_type,
                certificate.term_table.len(),
                "proof.expected_type",
            )?;
        }
        ProofNode::Apply {
            function_proof,
            argument_proofs,
            expected_type,
        } => {
            check_future_index(*function_proof, index, "proof.apply.function")?;
            check_future_indices(argument_proofs, index, "proof.apply.argument")?;
            check_index(
                *expected_type,
                certificate.term_table.len(),
                "proof.expected_type",
            )?;
        }
        ProofNode::Intro {
            domain_type,
            body_proof,
            expected_type,
        } => {
            check_index(
                *domain_type,
                certificate.term_table.len(),
                "proof.intro.domain",
            )?;
            check_future_index(*body_proof, index, "proof.intro.body")?;
            check_index(
                *expected_type,
                certificate.term_table.len(),
                "proof.expected_type",
            )?;
        }
        ProofNode::LetProof {
            value,
            body_proof,
            expected_type,
        } => {
            check_index(*value, certificate.term_table.len(), "proof.let.value")?;
            check_future_index(*body_proof, index, "proof.let.body")?;
            check_index(
                *expected_type,
                certificate.term_table.len(),
                "proof.expected_type",
            )?;
        }
        ProofNode::Rewrite {
            eq_proof,
            target_proof,
            expected_type,
        } => {
            check_future_index(*eq_proof, index, "proof.rewrite.eq")?;
            check_future_index(*target_proof, index, "proof.rewrite.target")?;
            check_index(
                *expected_type,
                certificate.term_table.len(),
                "proof.expected_type",
            )?;
        }
        ProofNode::EqRec {
            motive,
            eq_proof,
            base_proof,
            expected_type,
        } => {
            check_index(*motive, certificate.term_table.len(), "proof.eq_rec.motive")?;
            check_future_index(*eq_proof, index, "proof.eq_rec.eq")?;
            check_future_index(*base_proof, index, "proof.eq_rec.base")?;
            check_index(
                *expected_type,
                certificate.term_table.len(),
                "proof.expected_type",
            )?;
        }
        ProofNode::Constructor {
            argument_proofs,
            expected_type,
            ..
        } => {
            check_future_indices(argument_proofs, index, "proof.constructor.argument")?;
            check_index(
                *expected_type,
                certificate.term_table.len(),
                "proof.expected_type",
            )?;
        }
        ProofNode::Recursor {
            motive,
            minor_proofs,
            major_proof,
            expected_type,
            ..
        } => {
            check_index(
                *motive,
                certificate.term_table.len(),
                "proof.recursor.motive",
            )?;
            check_future_indices(minor_proofs, index, "proof.recursor.minor")?;
            check_future_index(*major_proof, index, "proof.recursor.major")?;
            check_index(
                *expected_type,
                certificate.term_table.len(),
                "proof.expected_type",
            )?;
        }
        ProofNode::Conv {
            proof,
            expected_type,
            defeq_witness,
        } => {
            check_future_index(*proof, index, "proof.conv.proof")?;
            check_index(
                *expected_type,
                certificate.term_table.len(),
                "proof.expected_type",
            )?;
            if let Some(defeq_witness) = defeq_witness {
                check_index(
                    *defeq_witness,
                    certificate.term_table.len(),
                    "proof.conv.defeq_witness",
                )?;
            }
        }
        ProofNode::Theory {
            theory_certificate,
            expected_type,
        } => {
            check_index(
                *theory_certificate,
                certificate.theory_certificates.len(),
                "proof.theory_certificate",
            )?;
            check_index(
                *expected_type,
                certificate.term_table.len(),
                "proof.expected_type",
            )?;
        }
    }
    Ok(())
}

fn validate_axiom_report_shape(certificate: &Certificate) -> Result<(), DecodeError> {
    for entry in &certificate.axiom_report.entries {
        check_indices(
            &entry.direct_dependent_declarations,
            certificate.declarations.len(),
            "axiom_report.direct_dependent_declaration",
        )?;
        check_indices(
            &entry.transitive_dependent_declarations,
            certificate.declarations.len(),
            "axiom_report.transitive_dependent_declaration",
        )?;
    }

    for dependencies in &certificate.axiom_report.declaration_dependencies {
        check_indices(
            &dependencies.direct_axiom_dependencies,
            certificate.axiom_report.entries.len(),
            "axiom_report.direct_axiom_dependency",
        )?;
        check_indices(
            &dependencies.transitive_axiom_dependencies,
            certificate.axiom_report.entries.len(),
            "axiom_report.transitive_axiom_dependency",
        )?;
    }
    Ok(())
}

fn check_index(id: u32, len: usize, field: &str) -> Result<(), DecodeError> {
    if (id as usize) < len {
        Ok(())
    } else {
        Err(DecodeError::with_detail(
            DecodeErrorKind::InvalidReference,
            0,
            format!("{field}={id} len={len}"),
        ))
    }
}

fn check_indices(ids: &[u32], len: usize, field: &str) -> Result<(), DecodeError> {
    for id in ids {
        check_index(*id, len, field)?;
    }
    Ok(())
}

fn check_future_index(id: u32, current: usize, field: &str) -> Result<(), DecodeError> {
    if (id as usize) < current {
        Ok(())
    } else {
        Err(DecodeError::with_detail(
            DecodeErrorKind::FutureReference,
            0,
            format!("{field}={id} current={current}"),
        ))
    }
}

fn check_future_indices(ids: &[u32], current: usize, field: &str) -> Result<(), DecodeError> {
    for id in ids {
        check_future_index(*id, current, field)?;
    }
    Ok(())
}

fn validate_name(name: &str, field: &str) -> Result<(), DecodeError> {
    mpk_core::Name::parse(name).map(|_| ()).map_err(|error| {
        DecodeError::with_detail(
            DecodeErrorKind::InvalidName,
            0,
            format!("{field}:{}", error.code()),
        )
    })
}

fn minimal_varint_len(mut value: u64) -> usize {
    let mut len = 1;
    while value >= 0x80 {
        value >>= 7;
        len += 1;
    }
    len
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use crate::{
        decode_certificate,
        encode::{
            encode_certificate, AxiomCategory, AxiomReport, AxiomReportEntry, AxiomReportSummary,
            Certificate, CertificateHashes, Declaration, DeclarationAxiomDependencies,
            DeclarationKind, DefinitionReducibility, ExportEntry, LevelNode, ProofNode,
            SourceManifest, TermNode, TheoryCertificate,
        },
        DecodeErrorKind,
    };

    const CERT_ENCODING_FIXTURE_DIR: &str =
        concat!(env!("CARGO_MANIFEST_DIR"), "/../../fixtures/cert-encoding");
    const CERT_DECODE_INVALID_FIXTURE_DIR: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/cert-decode/invalid"
    );

    fn decode_hex_fixture(path: &Path) -> Vec<u8> {
        let contents = fs::read_to_string(path).expect("hex fixture is readable");
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

    fn read_invalid_fixtures() -> Vec<(String, Vec<u8>)> {
        let mut entries = fs::read_dir(CERT_DECODE_INVALID_FIXTURE_DIR)
            .expect("invalid fixture directory exists")
            .map(|entry| entry.expect("fixture dir entry is readable").path())
            .filter(|path| path.extension().is_some_and(|extension| extension == "hex"))
            .collect::<Vec<_>>();
        entries.sort();

        entries
            .into_iter()
            .map(|path| {
                let name = path
                    .file_name()
                    .expect("fixture has file name")
                    .to_string_lossy()
                    .into_owned();
                (name, decode_hex_fixture(&path))
            })
            .collect()
    }

    fn hash(byte: u8) -> [u8; 32] {
        [byte; 32]
    }

    fn structural_certificate() -> Certificate {
        Certificate {
            module: "Example.Structural".to_owned(),
            imports: vec![crate::encode::Import {
                module_name: "Std.Core".to_owned(),
                export_hash: hash(1),
                certificate_hash: Some(hash(2)),
            }],
            name_table: vec![
                "Example.Structural.Type".to_owned(),
                "Example.Structural.ind".to_owned(),
                "Example.Structural.def".to_owned(),
                "Example.Structural.thm".to_owned(),
                "Example.Structural.ctor".to_owned(),
                "Example.Structural.rec".to_owned(),
                "Example.Structural.theory".to_owned(),
            ],
            level_table: vec![
                LevelNode::Zero,
                LevelNode::Param(0),
                LevelNode::Succ(0),
                LevelNode::Max(1, 2),
            ],
            term_table: vec![
                TermNode::Sort(0),
                TermNode::Sort(1),
                TermNode::Var(0),
                TermNode::Const {
                    global: 1,
                    levels: vec![0, 1],
                },
                TermNode::App {
                    function: 3,
                    arguments: vec![2],
                },
                TermNode::Lam { ty: 0, body: 2 },
                TermNode::Pi { ty: 0, body: 0 },
                TermNode::Let {
                    ty: 0,
                    value: 2,
                    body: 2,
                },
            ],
            proof_node_table: vec![
                ProofNode::Exact {
                    term: 2,
                    expected_type: 0,
                },
                ProofNode::Apply {
                    function_proof: 0,
                    argument_proofs: vec![0],
                    expected_type: 0,
                },
                ProofNode::Intro {
                    domain_type: 0,
                    body_proof: 0,
                    expected_type: 0,
                },
                ProofNode::LetProof {
                    value: 2,
                    body_proof: 0,
                    expected_type: 0,
                },
                ProofNode::Refl {
                    term: 2,
                    expected_type: 0,
                },
                ProofNode::Rewrite {
                    eq_proof: 0,
                    target_proof: 4,
                    expected_type: 0,
                },
                ProofNode::EqRec {
                    motive: 0,
                    eq_proof: 0,
                    base_proof: 4,
                    expected_type: 0,
                },
                ProofNode::Constructor {
                    constructor: 4,
                    argument_proofs: vec![0],
                    expected_type: 0,
                },
                ProofNode::Recursor {
                    recursor: 5,
                    motive: 0,
                    minor_proofs: vec![0],
                    major_proof: 0,
                    expected_type: 0,
                },
                ProofNode::Conv {
                    proof: 0,
                    expected_type: 0,
                    defeq_witness: Some(0),
                },
                ProofNode::Theory {
                    theory_certificate: 0,
                    expected_type: 0,
                },
            ],
            declarations: vec![
                Declaration {
                    name: 1,
                    kind: DeclarationKind::Inductive { ty: 0 },
                },
                Declaration {
                    name: 0,
                    kind: DeclarationKind::Axiom { ty: 0 },
                },
                Declaration {
                    name: 2,
                    kind: DeclarationKind::Def {
                        ty: 0,
                        value: 2,
                        reducibility: DefinitionReducibility::Reducible,
                    },
                },
                Declaration {
                    name: 3,
                    kind: DeclarationKind::Theorem { ty: 0, proof: 2 },
                },
                Declaration {
                    name: 4,
                    kind: DeclarationKind::Constructor {
                        ty: 0,
                        inductive: 0,
                        generated: true,
                    },
                },
                Declaration {
                    name: 5,
                    kind: DeclarationKind::Recursor {
                        ty: 0,
                        inductive: 0,
                        generated: true,
                    },
                },
                Declaration {
                    name: 6,
                    kind: DeclarationKind::TheoryPrimitive { ty: 0 },
                },
            ],
            theory_certificates: vec![TheoryCertificate {
                format: "bool-normalize-v0".to_owned(),
                payload: vec![1, 2, 3],
            }],
            export_block: vec![ExportEntry {
                name: 1,
                declaration: 0,
                declaration_hash: hash(3),
            }],
            axiom_report: AxiomReport {
                entries: vec![AxiomReportEntry {
                    category: AxiomCategory::CoreAxiom,
                    name: "Example.Structural.ax".to_owned(),
                    origin_module: "Example.Structural".to_owned(),
                    type_hash: hash(4),
                    declaration_hash: hash(5),
                    source_certificate_hash: Some(hash(6)),
                    direct_dependent_declarations: vec![1],
                    transitive_dependent_declarations: vec![1],
                    approval_profile: Some("core-mvp".to_owned()),
                    reviewer_note: Some("fixture".to_owned()),
                }],
                declaration_dependencies: vec![DeclarationAxiomDependencies {
                    declaration_name: "Example.Structural.thm".to_owned(),
                    declaration_hash: hash(7),
                    direct_axiom_dependencies: vec![0],
                    transitive_axiom_dependencies: vec![0],
                }],
                summary: AxiomReportSummary {
                    core_axiom_count: 1,
                    builtin_theory_axiom_count: 0,
                    go_semantics_axiom_count: 0,
                    external_axiom_count: 0,
                    total_axiom_count: 1,
                },
            },
            source_manifest: Some(SourceManifest {
                payload: br#"{"source":"fixture"}"#.to_vec(),
            }),
            hashes: CertificateHashes {
                export_hash: hash(8),
                axiom_report_hash: hash(9),
                certificate_hash: hash(10),
            },
        }
    }

    #[test]
    fn golden_minimal_certificate_decodes() {
        let path = Path::new(CERT_ENCODING_FIXTURE_DIR).join("minimal-empty.hex");
        let bytes = decode_hex_fixture(&path);

        let certificate = decode_certificate(&bytes).expect("golden fixture decodes");

        assert_eq!(certificate.module, "Example.Empty");
        assert!(certificate.imports.is_empty());
        assert!(certificate.name_table.is_empty());
        assert!(certificate.level_table.is_empty());
        assert!(certificate.term_table.is_empty());
        assert!(certificate.proof_node_table.is_empty());
        assert!(certificate.declarations.is_empty());
        assert!(certificate.source_manifest.is_none());
    }

    #[test]
    fn decoder_round_trips_structural_certificate() {
        let certificate = structural_certificate();
        let encoded = encode_certificate(&certificate);

        let decoded = decode_certificate(&encoded).expect("structural certificate decodes");

        assert_eq!(decoded, certificate);
    }

    #[test]
    fn invalid_byte_fixtures_reject() {
        let fixtures = read_invalid_fixtures();
        assert!(!fixtures.is_empty());

        for (name, bytes) in fixtures {
            let error = match decode_certificate(&bytes) {
                Ok(_) => panic!("fixture `{name}` should reject"),
                Err(error) => error,
            };
            assert!(
                matches!(
                    error.kind(),
                    DecodeErrorKind::UnexpectedEof
                        | DecodeErrorKind::TrailingBytes
                        | DecodeErrorKind::InvalidMagic
                        | DecodeErrorKind::NonMinimalVarint
                        | DecodeErrorKind::UnknownTag
                        | DecodeErrorKind::FutureReference
                ),
                "fixture `{name}` rejected with unexpected kind {:?}",
                error.kind()
            );
        }
    }

    #[test]
    fn decoder_rejects_non_minimal_varint() {
        let bytes = [b'M', b'P', b'K', b'C', b'E', b'R', b'T', 0x8c, 0x00];

        let error = decode_certificate(&bytes).unwrap_err();

        assert_eq!(error.kind(), DecodeErrorKind::NonMinimalVarint);
        assert_eq!(error.offset(), 7);
    }
}
