//! Canonical certificate byte validation.
//!
//! This layer rejects byte streams whose decoded structure does not re-encode
//! to the exact same bytes, and checks canonical ordering rules that are not
//! implied by structural decoding alone.

use crate::{
    decode_certificate,
    encode::{AxiomReportEntry, DeclarationAxiomDependencies},
    encode_certificate,
    imports::{validate_certificate_imports, ImportValidationError, ImportValidationErrorKind},
    Certificate, DecodeError,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalError {
    kind: CanonicalErrorKind,
    detail: Option<String>,
    decode_error: Option<DecodeError>,
}

impl CanonicalError {
    pub fn kind(&self) -> CanonicalErrorKind {
        self.kind
    }

    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }

    pub fn decode_error(&self) -> Option<&DecodeError> {
        self.decode_error.as_ref()
    }

    fn decode(error: DecodeError) -> Self {
        Self {
            kind: CanonicalErrorKind::DecodeRejected,
            detail: error.detail().map(ToOwned::to_owned),
            decode_error: Some(error),
        }
    }

    fn noncanonical(kind: CanonicalErrorKind, detail: impl Into<String>) -> Self {
        Self {
            kind,
            detail: Some(detail.into()),
            decode_error: None,
        }
    }

    fn import(error: ImportValidationError) -> Self {
        let kind = match error.kind() {
            ImportValidationErrorKind::NonCanonicalOrder => CanonicalErrorKind::NonCanonicalOrder,
            ImportValidationErrorKind::DuplicateImport => CanonicalErrorKind::DuplicateEntry,
            ImportValidationErrorKind::InvalidModuleName
            | ImportValidationErrorKind::ZeroExportHash
            | ImportValidationErrorKind::ZeroCertificateHash => CanonicalErrorKind::ImportRejected,
        };
        Self {
            kind,
            detail: Some(error.detail().to_owned()),
            decode_error: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum CanonicalErrorKind {
    DecodeRejected,
    ImportRejected,
    ReencodeMismatch,
    NonCanonicalOrder,
    DuplicateEntry,
}

pub fn validate_canonical_certificate(bytes: &[u8]) -> Result<(), CanonicalError> {
    decode_canonical_certificate(bytes).map(|_| ())
}

pub fn decode_canonical_certificate(bytes: &[u8]) -> Result<Certificate, CanonicalError> {
    let certificate = decode_certificate(bytes).map_err(CanonicalError::decode)?;
    validate_certificate_imports(&certificate).map_err(CanonicalError::import)?;
    validate_canonical_order(&certificate)?;
    let reencoded = encode_certificate(&certificate);
    if reencoded != bytes {
        return Err(CanonicalError::noncanonical(
            CanonicalErrorKind::ReencodeMismatch,
            format!(
                "decoded certificate re-encoded to {} bytes but input had {} bytes",
                reencoded.len(),
                bytes.len()
            ),
        ));
    }
    Ok(certificate)
}

fn validate_canonical_order(certificate: &Certificate) -> Result<(), CanonicalError> {
    check_sorted_by(
        &certificate.name_table,
        "name_table",
        |name| name.clone(),
        |name| name.clone(),
    )?;
    check_sorted_by(
        &certificate.axiom_report.entries,
        "axiom_report.entries",
        axiom_entry_key,
        axiom_entry_identity,
    )?;
    check_sorted_by(
        &certificate.axiom_report.declaration_dependencies,
        "axiom_report.declaration_dependencies",
        declaration_axiom_dependencies_key,
        declaration_axiom_dependencies_identity,
    )?;

    for entry in &certificate.axiom_report.entries {
        check_sorted_u32s(
            &entry.direct_dependent_declarations,
            "axiom_report.entry.direct_dependent_declarations",
        )?;
        check_sorted_u32s(
            &entry.transitive_dependent_declarations,
            "axiom_report.entry.transitive_dependent_declarations",
        )?;
    }
    for dependencies in &certificate.axiom_report.declaration_dependencies {
        check_sorted_u32s(
            &dependencies.direct_axiom_dependencies,
            "axiom_report.declaration.direct_axiom_dependencies",
        )?;
        check_sorted_u32s(
            &dependencies.transitive_axiom_dependencies,
            "axiom_report.declaration.transitive_axiom_dependencies",
        )?;
    }
    Ok(())
}

fn check_sorted_by<T, K: Ord>(
    values: &[T],
    field: &str,
    key: impl Fn(&T) -> K,
    identity: impl Fn(&T) -> String,
) -> Result<(), CanonicalError> {
    for pair in values.windows(2) {
        let lhs = &pair[0];
        let rhs = &pair[1];
        let lhs_key = key(lhs);
        let rhs_key = key(rhs);
        if lhs_key == rhs_key {
            return Err(CanonicalError::noncanonical(
                CanonicalErrorKind::DuplicateEntry,
                format!("{field}: {}", identity(lhs)),
            ));
        }
        if lhs_key > rhs_key {
            return Err(CanonicalError::noncanonical(
                CanonicalErrorKind::NonCanonicalOrder,
                format!("{field}: {} before {}", identity(lhs), identity(rhs)),
            ));
        }
    }
    Ok(())
}

fn check_sorted_u32s(values: &[u32], field: &str) -> Result<(), CanonicalError> {
    check_sorted_by(values, field, |value| *value, |value| value.to_string())
}

fn axiom_entry_key(entry: &AxiomReportEntry) -> (String, String, String, [u8; 32], [u8; 32]) {
    (
        entry.category.canonical_name().to_owned(),
        entry.name.clone(),
        entry.origin_module.clone(),
        entry.type_hash,
        entry.declaration_hash,
    )
}

fn axiom_entry_identity(entry: &AxiomReportEntry) -> String {
    format!(
        "{}:{}:{}",
        entry.category.canonical_name(),
        entry.origin_module,
        entry.name
    )
}

fn declaration_axiom_dependencies_key(
    dependencies: &DeclarationAxiomDependencies,
) -> (String, [u8; 32]) {
    (
        dependencies.declaration_name.clone(),
        dependencies.declaration_hash,
    )
}

fn declaration_axiom_dependencies_identity(dependencies: &DeclarationAxiomDependencies) -> String {
    dependencies.declaration_name.clone()
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use crate::{
        decode_canonical_certificate, decode_certificate, validate_canonical_certificate,
        CanonicalErrorKind,
    };

    const CERT_ENCODING_FIXTURE_DIR: &str =
        concat!(env!("CARGO_MANIFEST_DIR"), "/../../fixtures/cert-encoding");
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

    fn read_noncanonical_fixtures() -> Vec<(String, Vec<u8>)> {
        let mut entries = fs::read_dir(CERT_CANONICAL_NONCANONICAL_FIXTURE_DIR)
            .expect("non-canonical fixture directory exists")
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

    #[test]
    fn canonical_golden_fixture_accepts() {
        let bytes =
            decode_hex_fixture(&Path::new(CERT_ENCODING_FIXTURE_DIR).join("minimal-empty.hex"));

        let certificate = decode_canonical_certificate(&bytes).expect("canonical fixture accepts");

        assert_eq!(certificate.module, "Example.Empty");
    }

    #[test]
    fn noncanonical_fixtures_reject() {
        let fixtures = read_noncanonical_fixtures();
        assert!(!fixtures.is_empty());

        for (name, bytes) in fixtures {
            let error = match validate_canonical_certificate(&bytes) {
                Ok(()) => panic!("fixture `{name}` should reject"),
                Err(error) => error,
            };
            assert!(
                matches!(
                    error.kind(),
                    CanonicalErrorKind::DecodeRejected | CanonicalErrorKind::NonCanonicalOrder
                ),
                "fixture `{name}` rejected with unexpected kind {:?}",
                error.kind()
            );
        }
    }

    #[test]
    fn unsorted_name_table_decodes_but_fails_canonical_check() {
        let bytes = decode_hex_fixture(
            &Path::new(CERT_CANONICAL_NONCANONICAL_FIXTURE_DIR).join("unsorted-name-table.hex"),
        );

        decode_certificate(&bytes).expect("fixture is structurally decodable");
        let error = validate_canonical_certificate(&bytes).unwrap_err();

        assert_eq!(error.kind(), CanonicalErrorKind::NonCanonicalOrder);
        assert!(error
            .detail()
            .is_some_and(|detail| detail.contains("name_table")));
    }
}
