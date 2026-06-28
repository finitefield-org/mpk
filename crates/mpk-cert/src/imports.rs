//! Import table sorting and validation.

use std::cmp::Ordering;

use mpk_core::Name;

use crate::{
    encode::{Certificate, HashBytes, Import, ZERO_HASH},
    hash::hash_hex,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImportValidationError {
    kind: ImportValidationErrorKind,
    detail: String,
}

impl ImportValidationError {
    pub fn kind(&self) -> ImportValidationErrorKind {
        self.kind
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }

    fn new(kind: ImportValidationErrorKind, detail: impl Into<String>) -> Self {
        Self {
            kind,
            detail: detail.into(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum ImportValidationErrorKind {
    InvalidModuleName,
    ZeroExportHash,
    ZeroCertificateHash,
    NonCanonicalOrder,
    DuplicateImport,
}

pub fn sort_import_table(imports: &mut [Import]) {
    imports.sort_by(compare_imports);
}

pub fn validate_certificate_imports(
    certificate: &Certificate,
) -> Result<(), ImportValidationError> {
    validate_import_table(&certificate.imports)
}

pub fn validate_import_table(imports: &[Import]) -> Result<(), ImportValidationError> {
    for (index, import) in imports.iter().enumerate() {
        validate_import(index, import)?;

        if let Some(previous) = index
            .checked_sub(1)
            .and_then(|previous| imports.get(previous))
        {
            if same_normal_import_identity(previous, import) {
                return Err(ImportValidationError::new(
                    ImportValidationErrorKind::DuplicateImport,
                    format!(
                        "imports[{index}] duplicates {} with export_hash {}",
                        import.module_name,
                        hash_hex(&import.export_hash)
                    ),
                ));
            }

            if compare_imports(previous, import) == Ordering::Greater {
                return Err(ImportValidationError::new(
                    ImportValidationErrorKind::NonCanonicalOrder,
                    format!(
                        "imports[{index}] {}:{} appears after {}:{}",
                        previous.module_name,
                        hash_hex(&previous.export_hash),
                        import.module_name,
                        hash_hex(&import.export_hash)
                    ),
                ));
            }
        }
    }

    Ok(())
}

fn validate_import(index: usize, import: &Import) -> Result<(), ImportValidationError> {
    Name::parse(&import.module_name).map_err(|error| {
        ImportValidationError::new(
            ImportValidationErrorKind::InvalidModuleName,
            format!(
                "imports[{index}].module_name `{}` is not canonical: {}",
                import.module_name,
                error.code()
            ),
        )
    })?;

    validate_hash(
        import.export_hash,
        ImportValidationErrorKind::ZeroExportHash,
        format!("imports[{index}].export_hash for {}", import.module_name),
    )?;

    if let Some(certificate_hash) = import.certificate_hash {
        validate_hash(
            certificate_hash,
            ImportValidationErrorKind::ZeroCertificateHash,
            format!(
                "imports[{index}].certificate_hash for {}",
                import.module_name
            ),
        )?;
    }

    Ok(())
}

fn validate_hash(
    hash: HashBytes,
    kind: ImportValidationErrorKind,
    field: String,
) -> Result<(), ImportValidationError> {
    if hash == ZERO_HASH {
        return Err(ImportValidationError::new(
            kind,
            format!("{field} must not be all-zero"),
        ));
    }

    Ok(())
}

fn compare_imports(lhs: &Import, rhs: &Import) -> Ordering {
    lhs.module_name
        .cmp(&rhs.module_name)
        .then_with(|| lhs.export_hash.cmp(&rhs.export_hash))
        .then_with(|| lhs.certificate_hash.cmp(&rhs.certificate_hash))
}

fn same_normal_import_identity(lhs: &Import, rhs: &Import) -> bool {
    lhs.module_name == rhs.module_name && lhs.export_hash == rhs.export_hash
}

#[cfg(test)]
mod tests {
    use crate::{
        encode::{encode_certificate, AxiomReport, Certificate, CertificateHashes, Import},
        validate_canonical_certificate, CanonicalErrorKind,
    };

    use super::{
        sort_import_table, validate_certificate_imports, validate_import_table,
        ImportValidationErrorKind,
    };

    fn hash(byte: u8) -> [u8; 32] {
        [byte; 32]
    }

    fn import(
        module_name: &str,
        export_hash: [u8; 32],
        certificate_hash: Option<[u8; 32]>,
    ) -> Import {
        Import {
            module_name: module_name.to_owned(),
            export_hash,
            certificate_hash,
        }
    }

    fn certificate_with_imports(imports: Vec<Import>) -> Certificate {
        Certificate {
            module: "Example.ImportUser".to_owned(),
            imports,
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

    #[test]
    fn import_table_sorts_by_module_export_and_certificate_hash() {
        let mut imports = vec![
            import("Std.Logic", hash(2), Some(hash(4))),
            import("Std.BitVec", hash(3), None),
            import("Std.Logic", hash(1), Some(hash(9))),
            import("Std.Logic", hash(1), Some(hash(5))),
        ];

        sort_import_table(&mut imports);

        assert_eq!(imports[0].module_name, "Std.BitVec");
        assert_eq!(imports[1].module_name, "Std.Logic");
        assert_eq!(imports[1].export_hash, hash(1));
        assert_eq!(imports[1].certificate_hash, Some(hash(5)));
        assert_eq!(imports[2].module_name, "Std.Logic");
        assert_eq!(imports[2].export_hash, hash(1));
        assert_eq!(imports[2].certificate_hash, Some(hash(9)));
        assert_eq!(imports[3].export_hash, hash(2));
    }

    #[test]
    fn sorted_hash_pinned_imports_validate() {
        let imports = vec![
            import("Std.BitVec", hash(1), None),
            import("Std.Logic", hash(2), Some(hash(3))),
        ];

        validate_import_table(&imports).expect("sorted import table validates");
    }

    #[test]
    fn all_zero_export_hash_rejects() {
        let imports = vec![import("Std.Logic", [0; 32], Some(hash(3)))];

        let error = validate_import_table(&imports).unwrap_err();

        assert_eq!(error.kind(), ImportValidationErrorKind::ZeroExportHash);
        assert!(error.detail().contains("export_hash"));
    }

    #[test]
    fn all_zero_certificate_hash_rejects() {
        let imports = vec![import("Std.Logic", hash(2), Some([0; 32]))];

        let error = validate_import_table(&imports).unwrap_err();

        assert_eq!(error.kind(), ImportValidationErrorKind::ZeroCertificateHash);
        assert!(error.detail().contains("certificate_hash"));
    }

    #[test]
    fn invalid_module_name_rejects_direct_validation() {
        let imports = vec![import("Std..Logic", hash(2), None)];

        let error = validate_import_table(&imports).unwrap_err();

        assert_eq!(error.kind(), ImportValidationErrorKind::InvalidModuleName);
        assert!(error.detail().contains("EMPTY_COMPONENT"));
    }

    #[test]
    fn noncanonical_import_order_rejects() {
        let imports = vec![
            import("Std.Logic", hash(2), None),
            import("Std.BitVec", hash(1), None),
        ];

        let error = validate_import_table(&imports).unwrap_err();

        assert_eq!(error.kind(), ImportValidationErrorKind::NonCanonicalOrder);
    }

    #[test]
    fn duplicate_module_export_identity_rejects() {
        let imports = vec![
            import("Std.Logic", hash(2), Some(hash(3))),
            import("Std.Logic", hash(2), Some(hash(4))),
        ];

        let error = validate_import_table(&imports).unwrap_err();

        assert_eq!(error.kind(), ImportValidationErrorKind::DuplicateImport);
    }

    #[test]
    fn certificate_imports_validate_through_canonical_decoder() {
        let certificate =
            certificate_with_imports(vec![import("Std.Logic", hash(2), Some(hash(3)))]);
        let bytes = encode_certificate(&certificate);

        validate_canonical_certificate(&bytes).expect("valid imports pass canonical validation");
    }

    #[test]
    fn bad_import_hash_rejects_through_canonical_decoder() {
        let certificate =
            certificate_with_imports(vec![import("Std.Logic", [0; 32], Some(hash(3)))]);
        let bytes = encode_certificate(&certificate);

        let error = validate_canonical_certificate(&bytes).unwrap_err();

        assert_eq!(error.kind(), CanonicalErrorKind::ImportRejected);
        assert!(error
            .detail()
            .is_some_and(|detail| detail.contains("export_hash")));
    }

    #[test]
    fn certificate_imports_api_rejects_bad_hashes() {
        let certificate =
            certificate_with_imports(vec![import("Std.Logic", hash(2), Some([0; 32]))]);

        let error = validate_certificate_imports(&certificate).unwrap_err();

        assert_eq!(error.kind(), ImportValidationErrorKind::ZeroCertificateHash);
    }
}
