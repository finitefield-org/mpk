//! SHA-256 hash domains for canonical certificate payloads.

use sha2::{Digest, Sha256};

use crate::encode::{HashBytes, HASH_BYTE_LEN};

/// Certificate-side hash domains defined by `CERT_V0`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum HashDomain {
    ModuleExport,
    ModuleCertificate,
    AxiomReport,
    Level,
    Term,
    ProofNode,
    Declaration,
    TheoryCertificate,
    SourceManifest,
}

impl HashDomain {
    pub const ALL: [Self; 9] = [
        Self::ModuleExport,
        Self::ModuleCertificate,
        Self::AxiomReport,
        Self::Level,
        Self::Term,
        Self::ProofNode,
        Self::Declaration,
        Self::TheoryCertificate,
        Self::SourceManifest,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ModuleExport => "MPK-MODULE-EXPORT-0.1",
            Self::ModuleCertificate => "MPK-MODULE-CERT-0.1",
            Self::AxiomReport => "MPK-AXIOM-REPORT-0.1",
            Self::Level => "MPK-LEVEL-0.1",
            Self::Term => "MPK-TERM-0.1",
            Self::ProofNode => "MPK-PROOF-NODE-0.1",
            Self::Declaration => "MPK-DECL-0.1",
            Self::TheoryCertificate => "MPK-THEORY-CERT-0.1",
            Self::SourceManifest => "MPK-SOURCE-MANIFEST-0.1",
        }
    }

    pub fn from_tag(tag: &str) -> Option<Self> {
        match tag {
            "MPK-MODULE-EXPORT-0.1" => Some(Self::ModuleExport),
            "MPK-MODULE-CERT-0.1" => Some(Self::ModuleCertificate),
            "MPK-AXIOM-REPORT-0.1" => Some(Self::AxiomReport),
            "MPK-LEVEL-0.1" => Some(Self::Level),
            "MPK-TERM-0.1" => Some(Self::Term),
            "MPK-PROOF-NODE-0.1" => Some(Self::ProofNode),
            "MPK-DECL-0.1" => Some(Self::Declaration),
            "MPK-THEORY-CERT-0.1" => Some(Self::TheoryCertificate),
            "MPK-SOURCE-MANIFEST-0.1" => Some(Self::SourceManifest),
            _ => None,
        }
    }
}

pub fn hash_with_domain(domain: HashDomain, canonical_payload: &[u8]) -> HashBytes {
    let mut hasher = Sha256::new();
    hasher.update(domain.as_str().as_bytes());
    hasher.update([0x00]);
    hasher.update(canonical_payload);

    let digest = hasher.finalize();
    let mut hash = [0; HASH_BYTE_LEN];
    hash.copy_from_slice(&digest);
    hash
}

pub fn hash_hex(hash: &HashBytes) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";

    let mut encoded = String::with_capacity(HASH_BYTE_LEN * 2);
    for byte in hash {
        encoded.push(HEX[(byte >> 4) as usize] as char);
        encoded.push(HEX[(byte & 0x0f) as usize] as char);
    }
    encoded
}

pub fn certificate_hash(canonical_certificate: &[u8]) -> HashBytes {
    hash_with_domain(HashDomain::ModuleCertificate, canonical_certificate)
}

pub fn export_hash(canonical_export_payload: &[u8]) -> HashBytes {
    hash_with_domain(HashDomain::ModuleExport, canonical_export_payload)
}

pub fn axiom_report_hash(canonical_axiom_report_payload: &[u8]) -> HashBytes {
    hash_with_domain(HashDomain::AxiomReport, canonical_axiom_report_payload)
}

pub fn level_hash(canonical_level_payload: &[u8]) -> HashBytes {
    hash_with_domain(HashDomain::Level, canonical_level_payload)
}

pub fn term_hash(canonical_term_payload: &[u8]) -> HashBytes {
    hash_with_domain(HashDomain::Term, canonical_term_payload)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::encode::HASH_BYTE_LEN;

    use super::{
        axiom_report_hash, certificate_hash, export_hash, hash_hex, hash_with_domain, level_hash,
        term_hash, HashDomain,
    };

    const HASH_VECTOR_FIXTURE: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/cert-hash/vectors.csv"
    );

    fn decode_hex(input: &str) -> Vec<u8> {
        assert_eq!(input.len() % 2, 0, "hex payload must use full bytes");

        input
            .as_bytes()
            .chunks_exact(2)
            .map(|chunk| {
                let byte = std::str::from_utf8(chunk).expect("fixture hex is utf8");
                u8::from_str_radix(byte, 16).expect("fixture hex byte is valid")
            })
            .collect()
    }

    fn decode_hash_hex(input: &str) -> [u8; HASH_BYTE_LEN] {
        let bytes = decode_hex(input);
        assert_eq!(bytes.len(), HASH_BYTE_LEN, "hash hex must be 32 bytes");

        let mut hash = [0; HASH_BYTE_LEN];
        hash.copy_from_slice(&bytes);
        hash
    }

    #[test]
    fn hash_domain_tags_match_cert_v0() {
        let tags = HashDomain::ALL.map(HashDomain::as_str);

        assert_eq!(
            tags,
            [
                "MPK-MODULE-EXPORT-0.1",
                "MPK-MODULE-CERT-0.1",
                "MPK-AXIOM-REPORT-0.1",
                "MPK-LEVEL-0.1",
                "MPK-TERM-0.1",
                "MPK-PROOF-NODE-0.1",
                "MPK-DECL-0.1",
                "MPK-THEORY-CERT-0.1",
                "MPK-SOURCE-MANIFEST-0.1",
            ]
        );
    }

    #[test]
    fn hash_vectors_are_stable() {
        let contents =
            fs::read_to_string(HASH_VECTOR_FIXTURE).expect("hash vector fixture must be readable");

        for (line_index, raw_line) in contents.lines().enumerate() {
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let fields = line.split(',').collect::<Vec<_>>();
            assert_eq!(
                fields.len(),
                3,
                "fixture line {} must be csv",
                line_index + 1
            );

            let domain = HashDomain::from_tag(fields[0])
                .unwrap_or_else(|| panic!("fixture line {} has known domain", line_index + 1));
            let payload = decode_hex(fields[1]);
            let expected = decode_hash_hex(fields[2]);

            assert_eq!(
                hash_with_domain(domain, &payload),
                expected,
                "fixture line {} hash mismatch",
                line_index + 1
            );
            assert_eq!(
                hash_hex(&expected),
                fields[2],
                "fixture line {} renders lowercase hex",
                line_index + 1
            );
        }
    }

    #[test]
    fn named_hash_helpers_use_their_domains() {
        let payload = [0x00, 0x01, 0x7f];

        assert_eq!(
            certificate_hash(&payload),
            hash_with_domain(HashDomain::ModuleCertificate, &payload)
        );
        assert_eq!(
            export_hash(&payload),
            hash_with_domain(HashDomain::ModuleExport, &payload)
        );
        assert_eq!(
            axiom_report_hash(&payload),
            hash_with_domain(HashDomain::AxiomReport, &payload)
        );
        assert_eq!(
            level_hash(&payload),
            hash_with_domain(HashDomain::Level, &payload)
        );
        assert_eq!(
            term_hash(&payload),
            hash_with_domain(HashDomain::Term, &payload)
        );
    }

    #[test]
    fn domain_separator_changes_hashes() {
        let payload = [];

        assert_ne!(level_hash(&payload), term_hash(&payload));
        assert_ne!(export_hash(&payload), certificate_hash(&payload));
    }

    #[test]
    fn hash_hex_renders_lowercase_hex() {
        let mut hash = [0; HASH_BYTE_LEN];
        hash[0] = 0xab;
        hash[31] = 0xef;

        assert_eq!(
            hash_hex(&hash),
            "ab000000000000000000000000000000000000000000000000000000000000ef"
        );
    }
}
