//! Source-free certificate verifier driver.

use mpk_cert::encode::ZERO_HASH;
use mpk_cert::{
    axiom_report_hash_for_report, build_axiom_report, build_export_block, certificate_hash,
    decode_canonical_certificate,
    encode::{Certificate, DeclarationKind, HashBytes, LevelNode, TermNode},
    export_block_hash,
};
use mpk_core::{
    check, infer, register_checked_theorem, CoreError, CoreErrorCode, CoreLocation, Environment,
    GlobalId, LevelArena, LevelId, LocalContext, Name, TermArena, TermId, TermNode as CoreTermNode,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerificationReport {
    pub module: String,
    pub declaration_count: usize,
    pub axiom_count: u64,
    pub export_hash: HashBytes,
    pub axiom_report_hash: HashBytes,
    pub certificate_hash: HashBytes,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerificationError {
    kind: VerificationErrorKind,
    detail: String,
}

impl VerificationError {
    pub fn kind(&self) -> VerificationErrorKind {
        self.kind
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }

    fn new(kind: VerificationErrorKind, detail: impl Into<String>) -> Self {
        Self {
            kind,
            detail: detail.into(),
        }
    }

    fn canonical(detail: impl Into<String>) -> Self {
        Self::new(VerificationErrorKind::CanonicalCertificate, detail)
    }

    fn unsupported(detail: impl Into<String>) -> Self {
        Self::new(VerificationErrorKind::UnsupportedFeature, detail)
    }

    fn core(error: CoreError) -> Self {
        Self::new(
            VerificationErrorKind::CoreCheck,
            error.to_deterministic_json(),
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum VerificationErrorKind {
    CanonicalCertificate,
    UnsupportedFeature,
    ExportBlockMismatch,
    AxiomReportMismatch,
    HashMismatch,
    MissingName,
    MissingGlobal,
    CoreCheck,
    InternalInvariant,
}

pub fn verify_certificate_bytes(bytes: &[u8]) -> Result<VerificationReport, VerificationError> {
    let certificate = decode_canonical_certificate(bytes).map_err(|error| {
        VerificationError::canonical(
            error
                .detail()
                .map(ToOwned::to_owned)
                .or_else(|| {
                    error
                        .decode_error()
                        .and_then(|decode| decode.detail().map(ToOwned::to_owned))
                })
                .unwrap_or_else(|| format!("{:?}", error.kind())),
        )
    })?;
    verify_certificate(certificate, certificate_hash(bytes))
}

fn verify_certificate(
    certificate: Certificate,
    computed_certificate_hash: HashBytes,
) -> Result<VerificationReport, VerificationError> {
    reject_unsupported_certificate_features(&certificate)?;
    verify_recomputed_certificate_sections(&certificate)?;

    let mut verifier = CertificateVerifier::new(&certificate);
    verifier.verify_declarations()?;

    Ok(VerificationReport {
        module: certificate.module,
        declaration_count: certificate.declarations.len(),
        axiom_count: certificate.axiom_report.summary.total_axiom_count,
        export_hash: certificate.hashes.export_hash,
        axiom_report_hash: certificate.hashes.axiom_report_hash,
        certificate_hash: computed_certificate_hash,
    })
}

fn reject_unsupported_certificate_features(
    certificate: &Certificate,
) -> Result<(), VerificationError> {
    if !certificate.imports.is_empty() {
        return Err(VerificationError::unsupported(
            "import resolution is not implemented by KERN-001",
        ));
    }
    if !certificate.proof_node_table.is_empty() {
        return Err(VerificationError::unsupported(
            "proof-node checking is not implemented by KERN-001",
        ));
    }
    if !certificate.theory_certificates.is_empty() {
        return Err(VerificationError::unsupported(
            "theory certificate checking is not implemented by KERN-001",
        ));
    }
    Ok(())
}

fn verify_recomputed_certificate_sections(
    certificate: &Certificate,
) -> Result<(), VerificationError> {
    let rebuilt_export_block = build_export_block(certificate).map_err(|error| {
        VerificationError::new(
            VerificationErrorKind::ExportBlockMismatch,
            error.detail().to_owned(),
        )
    })?;
    if rebuilt_export_block != certificate.export_block {
        return Err(VerificationError::new(
            VerificationErrorKind::ExportBlockMismatch,
            "export block does not match checked declarations",
        ));
    }

    let rebuilt_axiom_report = build_axiom_report(certificate).map_err(|error| {
        VerificationError::new(
            VerificationErrorKind::AxiomReportMismatch,
            error.detail().to_owned(),
        )
    })?;
    if rebuilt_axiom_report != certificate.axiom_report {
        return Err(VerificationError::new(
            VerificationErrorKind::AxiomReportMismatch,
            "axiom report does not match checked declarations",
        ));
    }

    let export_hash = export_block_hash(&certificate.export_block);
    if export_hash != certificate.hashes.export_hash {
        return Err(VerificationError::new(
            VerificationErrorKind::HashMismatch,
            "embedded export hash does not match recomputed export block hash",
        ));
    }

    let axiom_report_hash = axiom_report_hash_for_report(&certificate.axiom_report);
    if axiom_report_hash != certificate.hashes.axiom_report_hash {
        return Err(VerificationError::new(
            VerificationErrorKind::HashMismatch,
            "embedded axiom report hash does not match recomputed axiom report hash",
        ));
    }
    // The certificate hash field is inside the v0 payload, so KERN-001 reports
    // the recomputed byte-stream hash externally and requires a zero placeholder.
    if certificate.hashes.certificate_hash != ZERO_HASH {
        return Err(VerificationError::new(
            VerificationErrorKind::HashMismatch,
            "embedded certificate hash must be the zero placeholder in KERN-001",
        ));
    }

    Ok(())
}

struct CertificateVerifier<'certificate> {
    certificate: &'certificate Certificate,
    levels: LevelArena,
    terms: TermArena,
    env: Environment,
    level_cache: Vec<Option<LevelId>>,
    term_cache: Vec<Option<TermId>>,
    globals: Vec<GlobalId>,
}

impl<'certificate> CertificateVerifier<'certificate> {
    fn new(certificate: &'certificate Certificate) -> Self {
        Self {
            certificate,
            levels: LevelArena::new(),
            terms: TermArena::new(),
            env: Environment::new(),
            level_cache: vec![None; certificate.level_table.len()],
            term_cache: vec![None; certificate.term_table.len()],
            globals: Vec::with_capacity(certificate.declarations.len()),
        }
    }

    fn verify_declarations(&mut self) -> Result<(), VerificationError> {
        for (index, declaration) in self.certificate.declarations.iter().enumerate() {
            let name = self.name_table_entry(declaration.name)?.to_owned();
            let global = match &declaration.kind {
                DeclarationKind::Axiom { ty } => {
                    let ty = self.translate_term(*ty)?;
                    self.expect_term_type_is_sort(index, "axiom_type", ty)?;
                    self.env
                        .register_axiom(name, ty)
                        .map_err(VerificationError::core)?
                }
                DeclarationKind::Def {
                    ty,
                    value,
                    reducibility,
                } => {
                    let ty = self.translate_term(*ty)?;
                    let value = self.translate_term(*value)?;
                    self.expect_term_type_is_sort(index, "definition_type", ty)?;
                    check(
                        &mut self.levels,
                        &mut self.terms,
                        &LocalContext::new(),
                        &self.env,
                        value,
                        ty,
                    )
                    .map_err(VerificationError::core)?;
                    self.env
                        .register_definition(name, ty, value, convert_reducibility(*reducibility))
                        .map_err(VerificationError::core)?
                }
                DeclarationKind::Theorem { ty, proof } => {
                    let ty = self.translate_term(*ty)?;
                    let proof = self.translate_term(*proof)?;
                    register_checked_theorem(
                        &mut self.levels,
                        &mut self.terms,
                        &mut self.env,
                        name,
                        ty,
                        proof,
                    )
                    .map_err(VerificationError::core)?
                }
                DeclarationKind::Inductive { .. }
                | DeclarationKind::Constructor { .. }
                | DeclarationKind::Recursor { .. }
                | DeclarationKind::TheoryPrimitive { .. } => {
                    return Err(VerificationError::unsupported(format!(
                        "declaration {index} uses a declaration kind not implemented by KERN-001"
                    )));
                }
            };

            self.push_global(index, global)?;
        }
        Ok(())
    }

    fn expect_term_type_is_sort(
        &mut self,
        declaration_index: usize,
        field: &'static str,
        term: TermId,
    ) -> Result<(), VerificationError> {
        let inferred = infer(
            &mut self.levels,
            &mut self.terms,
            &LocalContext::new(),
            &self.env,
            term,
        )
        .map_err(VerificationError::core)?;
        if matches!(self.terms.node(inferred), CoreTermNode::Sort(_)) {
            return Ok(());
        }

        let declaration_index = u32::try_from(declaration_index).map_err(|_| {
            VerificationError::new(
                VerificationErrorKind::InternalInvariant,
                "declaration index exceeds u32",
            )
        })?;
        Err(VerificationError::core(
            CoreError::new(
                CoreErrorCode::TypeMismatch,
                CoreLocation::root()
                    .with_field("verifier")
                    .with_field("declarations")
                    .with_index(declaration_index)
                    .with_field(field),
            )
            .with_detail("kind", "declaration_type_not_sort")
            .with_detail("term_index", term.index().to_string())
            .with_detail("inferred_term_index", inferred.index().to_string()),
        ))
    }

    fn translate_level(&mut self, level: u32) -> Result<LevelId, VerificationError> {
        let index = usize::try_from(level).expect("u32 id fits in usize");
        if let Some(translated) = self.level_cache[index] {
            return Ok(translated);
        }

        let node = self.certificate.level_table[index].clone();
        let translated = match node {
            LevelNode::Zero => self.levels.zero(),
            LevelNode::Succ(inner) => {
                let inner = self.translate_level(inner)?;
                self.levels.succ(inner)
            }
            LevelNode::Max(lhs, rhs) => {
                let lhs = self.translate_level(lhs)?;
                let rhs = self.translate_level(rhs)?;
                self.levels.max(lhs, rhs)
            }
            LevelNode::Param(name) => {
                let name = Name::parse(self.name_table_entry(name)?).map_err(|error| {
                    VerificationError::new(VerificationErrorKind::MissingName, error.code())
                })?;
                self.levels.param(name)
            }
        };

        self.level_cache[index] = Some(translated);
        Ok(translated)
    }

    fn translate_term(&mut self, term: u32) -> Result<TermId, VerificationError> {
        let index = usize::try_from(term).expect("u32 id fits in usize");
        if let Some(translated) = self.term_cache[index] {
            return Ok(translated);
        }

        let node = self.certificate.term_table[index].clone();
        let translated = match node {
            TermNode::Sort(level) => {
                let level = self.translate_level(level)?;
                self.terms.sort(level)
            }
            TermNode::Var(index) => self.terms.var(index),
            TermNode::Const { global, levels } => {
                let global = self.global_by_declaration(global)?;
                let levels = levels
                    .into_iter()
                    .map(|level| self.translate_level(level))
                    .collect::<Result<Vec<_>, _>>()?;
                self.terms.constant(global, levels)
            }
            TermNode::App {
                function,
                arguments,
            } => {
                let function = self.translate_term(function)?;
                let arguments = arguments
                    .into_iter()
                    .map(|argument| self.translate_term(argument))
                    .collect::<Result<Vec<_>, _>>()?;
                self.terms.app(function, arguments)
            }
            TermNode::Lam { ty, body } => {
                let ty = self.translate_term(ty)?;
                let body = self.translate_term(body)?;
                self.terms.lam(ty, body)
            }
            TermNode::Pi { ty, body } => {
                let ty = self.translate_term(ty)?;
                let body = self.translate_term(body)?;
                self.terms.pi(ty, body)
            }
            TermNode::Let { ty, value, body } => {
                let ty = self.translate_term(ty)?;
                let value = self.translate_term(value)?;
                let body = self.translate_term(body)?;
                self.terms.let_term(ty, value, body)
            }
        };

        self.term_cache[index] = Some(translated);
        Ok(translated)
    }

    fn global_by_declaration(&self, global: u32) -> Result<GlobalId, VerificationError> {
        self.globals
            .get(usize::try_from(global).expect("u32 id fits in usize"))
            .copied()
            .ok_or_else(|| {
                VerificationError::new(
                    VerificationErrorKind::MissingGlobal,
                    format!("declaration references unavailable global {global}"),
                )
            })
    }

    fn name_table_entry(&self, name: u32) -> Result<&'certificate str, VerificationError> {
        self.certificate
            .name_table
            .get(usize::try_from(name).expect("u32 id fits in usize"))
            .map(String::as_str)
            .ok_or_else(|| {
                VerificationError::new(
                    VerificationErrorKind::MissingName,
                    format!("missing name id {name}"),
                )
            })
    }

    fn push_global(
        &mut self,
        declaration_index: usize,
        global: GlobalId,
    ) -> Result<(), VerificationError> {
        let expected = u32::try_from(declaration_index).map_err(|_| {
            VerificationError::new(
                VerificationErrorKind::InternalInvariant,
                "declaration index exceeds u32",
            )
        })?;
        if global.as_u32() != expected {
            return Err(VerificationError::new(
                VerificationErrorKind::InternalInvariant,
                format!(
                    "registered global {} does not match declaration index {expected}",
                    global.as_u32()
                ),
            ));
        }
        self.globals.push(global);
        Ok(())
    }
}

fn convert_reducibility(
    reducibility: mpk_cert::encode::DefinitionReducibility,
) -> mpk_core::DefinitionReducibility {
    match reducibility {
        mpk_cert::encode::DefinitionReducibility::Reducible => {
            mpk_core::DefinitionReducibility::Reducible
        }
        mpk_cert::encode::DefinitionReducibility::Opaque => {
            mpk_core::DefinitionReducibility::Opaque
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use mpk_cert::{
        axiom_report_hash_for_report, build_axiom_report, build_export_block,
        encode::{
            AxiomReport, Certificate, CertificateHashes, Declaration, DeclarationKind, LevelNode,
            TermNode,
        },
        encode_certificate, export_block_hash,
    };

    use super::{verify_certificate_bytes, VerificationErrorKind};

    const CERT_BASIC_FIXTURE_DIR: &str =
        concat!(env!("CARGO_MANIFEST_DIR"), "/../../fixtures/cert-basic");
    const CERT_CANONICAL_NONCANONICAL_FIXTURE_DIR: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/cert-canonical/non-canonical"
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

    fn finalize_certificate(mut certificate: Certificate) -> Certificate {
        certificate.export_block = build_export_block(&certificate).expect("export block builds");
        certificate.axiom_report = build_axiom_report(&certificate).expect("axiom report builds");
        certificate.hashes.export_hash = export_block_hash(&certificate.export_block);
        certificate.hashes.axiom_report_hash =
            axiom_report_hash_for_report(&certificate.axiom_report);
        certificate
    }

    fn one_theorem_certificate(proof: u32) -> Certificate {
        finalize_certificate(Certificate {
            module: "Example.Kernel.OneTheorem".to_owned(),
            imports: Vec::new(),
            name_table: vec!["Example.Kernel.OneTheorem.sort0IsSort1".to_owned()],
            level_table: vec![LevelNode::Zero, LevelNode::Succ(0)],
            term_table: vec![TermNode::Sort(0), TermNode::Sort(1)],
            proof_node_table: Vec::new(),
            declarations: vec![Declaration {
                name: 0,
                kind: DeclarationKind::Theorem { ty: 1, proof },
            }],
            theory_certificates: Vec::new(),
            export_block: Vec::new(),
            axiom_report: AxiomReport::default(),
            source_manifest: None,
            hashes: CertificateHashes::default(),
        })
    }

    #[test]
    fn verifies_basic_certificate_fixtures() {
        for name in ["zero-axiom", "one-theorem"] {
            let bytes =
                decode_hex_fixture(&Path::new(CERT_BASIC_FIXTURE_DIR).join(format!("{name}.hex")));
            let report = verify_certificate_bytes(&bytes).expect("fixture verifies");

            assert!(report.module.starts_with("Example.Basic."));
        }
    }

    #[test]
    fn rejects_noncanonical_certificate_bytes() {
        let bytes = decode_hex_fixture(
            &Path::new(CERT_CANONICAL_NONCANONICAL_FIXTURE_DIR).join("unsorted-name-table.hex"),
        );

        let error = verify_certificate_bytes(&bytes).unwrap_err();

        assert_eq!(error.kind(), VerificationErrorKind::CanonicalCertificate);
    }

    #[test]
    fn rejects_theorem_that_fails_core_checking() {
        let certificate = one_theorem_certificate(1);
        let bytes = encode_certificate(&certificate);

        let error = verify_certificate_bytes(&bytes).unwrap_err();

        assert_eq!(error.kind(), VerificationErrorKind::CoreCheck);
    }

    #[test]
    fn rejects_nonzero_embedded_certificate_hash() {
        let mut certificate = one_theorem_certificate(0);
        certificate.hashes.certificate_hash = [1; 32];
        let bytes = encode_certificate(&certificate);

        let error = verify_certificate_bytes(&bytes).unwrap_err();

        assert_eq!(error.kind(), VerificationErrorKind::HashMismatch);
    }
}
