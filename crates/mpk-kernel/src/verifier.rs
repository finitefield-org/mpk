//! Source-free certificate verifier driver.

use crate::decl_driver::{
    check_declarations_with_context, DeclarationCheckError, DeclarationCheckErrorKind,
};
use crate::proof_check::{
    check_proof_nodes_with_context, ProofCheckError, ProofCheckErrorKind, ProofCheckProfile,
};

use mpk_cert::encode::ZERO_HASH;
use mpk_cert::{
    axiom_report_hash_for_report, build_axiom_report, build_export_block, certificate_hash,
    decode_canonical_certificate,
    encode::{AxiomReport, Certificate, HashBytes},
    export_block_hash,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerificationReport {
    pub module: String,
    pub declaration_count: usize,
    pub axiom_count: u64,
    pub export_hash: HashBytes,
    pub axiom_report_hash: HashBytes,
    pub certificate_hash: HashBytes,
    pub axiom_report: AxiomReport,
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
    OutOfOrderDeclarationDependency,
    CoreCheck,
    InternalInvariant,
}

impl VerificationErrorKind {
    pub fn code(self) -> &'static str {
        match self {
            Self::CanonicalCertificate => "KERNEL_CANONICAL_CERTIFICATE",
            Self::UnsupportedFeature => "KERNEL_UNSUPPORTED_FEATURE",
            Self::ExportBlockMismatch => "KERNEL_EXPORT_BLOCK_MISMATCH",
            Self::AxiomReportMismatch => "KERNEL_AXIOM_REPORT_MISMATCH",
            Self::HashMismatch => "KERNEL_HASH_MISMATCH",
            Self::MissingName => "KERNEL_MISSING_NAME",
            Self::MissingGlobal => "KERNEL_MISSING_GLOBAL",
            Self::OutOfOrderDeclarationDependency => "KERNEL_OUT_OF_ORDER_DECLARATION_DEPENDENCY",
            Self::CoreCheck => "KERNEL_CORE_CHECK",
            Self::InternalInvariant => "KERNEL_INTERNAL_INVARIANT",
        }
    }
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
    reject_unsupported_imports(&certificate)?;
    let mut declaration_context =
        check_declarations_with_context(&certificate).map_err(VerificationError::declaration)?;
    let declaration_count = declaration_context.declaration_count();
    check_proof_nodes_with_context(&mut declaration_context, ProofCheckProfile::MvpStrict)
        .map_err(VerificationError::proof)?;
    let axiom_report = verify_recomputed_certificate_sections(&certificate)?;

    Ok(VerificationReport {
        module: certificate.module,
        declaration_count,
        axiom_count: axiom_report.summary.total_axiom_count,
        export_hash: certificate.hashes.export_hash,
        axiom_report_hash: certificate.hashes.axiom_report_hash,
        certificate_hash: computed_certificate_hash,
        axiom_report,
    })
}

impl VerificationError {
    fn declaration(error: DeclarationCheckError) -> Self {
        match error.kind() {
            DeclarationCheckErrorKind::UnsupportedDeclarationKind => {
                Self::unsupported(error.detail())
            }
            DeclarationCheckErrorKind::MissingName => {
                Self::new(VerificationErrorKind::MissingName, error.detail())
            }
            DeclarationCheckErrorKind::MissingGlobal => {
                Self::new(VerificationErrorKind::MissingGlobal, error.detail())
            }
            DeclarationCheckErrorKind::OutOfOrderDeclarationDependency => Self::new(
                VerificationErrorKind::OutOfOrderDeclarationDependency,
                error.detail(),
            ),
            DeclarationCheckErrorKind::CoreCheck => {
                Self::new(VerificationErrorKind::CoreCheck, error.detail())
            }
            DeclarationCheckErrorKind::InternalInvariant => {
                Self::new(VerificationErrorKind::InternalInvariant, error.detail())
            }
        }
    }

    fn proof(error: ProofCheckError) -> Self {
        match error.kind() {
            ProofCheckErrorKind::UnsupportedDeclarationKind
            | ProofCheckErrorKind::UnsupportedProofNodeKind => Self::unsupported(error.detail()),
            ProofCheckErrorKind::MissingName => {
                Self::new(VerificationErrorKind::MissingName, error.detail())
            }
            ProofCheckErrorKind::MissingGlobal => {
                Self::new(VerificationErrorKind::MissingGlobal, error.detail())
            }
            ProofCheckErrorKind::MissingProofNode => {
                Self::new(VerificationErrorKind::InternalInvariant, error.detail())
            }
            ProofCheckErrorKind::OutOfOrderDeclarationDependency => Self::new(
                VerificationErrorKind::OutOfOrderDeclarationDependency,
                error.detail(),
            ),
            ProofCheckErrorKind::CoreCheck => {
                Self::new(VerificationErrorKind::CoreCheck, error.detail())
            }
            ProofCheckErrorKind::InternalInvariant => {
                Self::new(VerificationErrorKind::InternalInvariant, error.detail())
            }
        }
    }
}

fn reject_unsupported_imports(certificate: &Certificate) -> Result<(), VerificationError> {
    if !certificate.imports.is_empty() {
        return Err(VerificationError::unsupported(
            "import resolution is not implemented by KERN-001",
        ));
    }
    Ok(())
}

fn verify_recomputed_certificate_sections(
    certificate: &Certificate,
) -> Result<AxiomReport, VerificationError> {
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

    Ok(rebuilt_axiom_report)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use mpk_cert::{
        axiom_report_hash_for_report, build_axiom_report, build_export_block,
        encode::{
            AxiomReport, Certificate, CertificateHashes, Declaration, DeclarationKind, LevelNode,
            ProofNode, TermNode, TheoryCertificate,
        },
        encode_certificate, export_block_hash,
    };
    use mpk_theory::BOOL_CERT_FORMAT;

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

    fn out_of_order_dependency_certificate() -> Certificate {
        let mut certificate = Certificate {
            module: "Example.Kernel.OutOfOrder".to_owned(),
            imports: Vec::new(),
            name_table: vec![
                "Example.Kernel.OutOfOrder.future".to_owned(),
                "Example.Kernel.OutOfOrder.usesFuture".to_owned(),
            ],
            level_table: vec![LevelNode::Zero, LevelNode::Succ(0)],
            term_table: vec![
                TermNode::Sort(0),
                TermNode::Sort(1),
                TermNode::Const {
                    global: 1,
                    levels: Vec::new(),
                },
            ],
            proof_node_table: Vec::new(),
            declarations: vec![
                Declaration {
                    name: 1,
                    kind: DeclarationKind::Theorem { ty: 1, proof: 2 },
                },
                Declaration {
                    name: 0,
                    kind: DeclarationKind::Axiom { ty: 1 },
                },
            ],
            theory_certificates: Vec::new(),
            export_block: Vec::new(),
            axiom_report: AxiomReport::default(),
            source_manifest: None,
            hashes: CertificateHashes::default(),
        };
        certificate.export_block = build_export_block(&certificate).expect("export block builds");
        certificate.hashes.export_hash = export_block_hash(&certificate.export_block);
        certificate.hashes.axiom_report_hash =
            axiom_report_hash_for_report(&certificate.axiom_report);
        certificate
    }

    fn bootstrap_proof_node_certificate() -> Certificate {
        finalize_certificate(Certificate {
            module: "Example.Kernel.ProofBootstrap".to_owned(),
            imports: Vec::new(),
            name_table: vec!["Example.Kernel.ProofBootstrap.x".to_owned()],
            level_table: vec![LevelNode::Zero, LevelNode::Succ(0)],
            term_table: vec![
                TermNode::Sort(0),
                TermNode::Sort(1),
                TermNode::Const {
                    global: 0,
                    levels: Vec::new(),
                },
                TermNode::Var(0),
                TermNode::Lam { ty: 0, body: 3 },
                TermNode::Pi { ty: 0, body: 0 },
            ],
            proof_node_table: vec![
                ProofNode::Exact {
                    term: 2,
                    expected_type: 0,
                },
                ProofNode::Exact {
                    term: 4,
                    expected_type: 5,
                },
                ProofNode::Apply {
                    function_proof: 1,
                    argument_proofs: vec![0],
                    expected_type: 0,
                },
                ProofNode::Exact {
                    term: 3,
                    expected_type: 0,
                },
                ProofNode::Intro {
                    domain_type: 0,
                    body_proof: 3,
                    expected_type: 5,
                },
                ProofNode::Refl {
                    term: 2,
                    expected_type: 0,
                },
                ProofNode::Conv {
                    proof: 0,
                    expected_type: 0,
                    defeq_witness: Some(0),
                },
            ],
            declarations: vec![Declaration {
                name: 0,
                kind: DeclarationKind::Axiom { ty: 0 },
            }],
            theory_certificates: Vec::new(),
            export_block: Vec::new(),
            axiom_report: AxiomReport::default(),
            source_manifest: None,
            hashes: CertificateHashes::default(),
        })
    }

    fn bool_tautology_payload() -> Vec<u8> {
        let mut payload = Vec::new();
        payload.extend_from_slice(b"MPKBOOL0");
        payload.push(0);
        payload.push(0x01);
        payload.extend_from_slice(&1u16.to_be_bytes());
        payload.push(0);
        payload.push(1);
        payload
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
    fn rejects_out_of_order_declaration_dependencies_before_report_recompute() {
        let certificate = out_of_order_dependency_certificate();
        let bytes = encode_certificate(&certificate);

        let error = verify_certificate_bytes(&bytes).unwrap_err();

        assert_eq!(
            error.kind(),
            VerificationErrorKind::OutOfOrderDeclarationDependency
        );
    }

    #[test]
    fn verifies_core_bootstrap_proof_nodes() {
        let certificate = bootstrap_proof_node_certificate();
        let bytes = encode_certificate(&certificate);

        let report = verify_certificate_bytes(&bytes).expect("proof-node certificate verifies");

        assert_eq!(report.module, "Example.Kernel.ProofBootstrap");
        assert_eq!(report.declaration_count, 1);
    }

    #[test]
    fn verifies_theory_proof_node_fixture() {
        let mut certificate = bootstrap_proof_node_certificate();
        certificate.theory_certificates.push(TheoryCertificate {
            format: BOOL_CERT_FORMAT.to_owned(),
            payload: bool_tautology_payload(),
        });
        certificate.proof_node_table.push(ProofNode::Theory {
            theory_certificate: 0,
            expected_type: 0,
        });
        certificate = finalize_certificate(certificate);
        let bytes = encode_certificate(&certificate);

        let report = verify_certificate_bytes(&bytes).expect("theory proof-node fixture verifies");

        assert_eq!(report.module, "Example.Kernel.ProofBootstrap");
        assert_eq!(report.declaration_count, 1);
    }

    #[test]
    fn rejects_malformed_theory_proof_node_fixture() {
        let mut certificate = bootstrap_proof_node_certificate();
        certificate.theory_certificates.push(TheoryCertificate {
            format: BOOL_CERT_FORMAT.to_owned(),
            payload: Vec::new(),
        });
        certificate.proof_node_table.push(ProofNode::Theory {
            theory_certificate: 0,
            expected_type: 0,
        });
        certificate = finalize_certificate(certificate);
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
