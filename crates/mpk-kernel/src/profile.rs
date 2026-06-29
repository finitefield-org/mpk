//! Fast-kernel profiling helpers.

use std::time::Instant;

use crate::cache::CheckerCacheMetrics;
use crate::decl_driver::check_declarations_with_cache_timing;
use crate::proof_check::{check_proof_nodes_with_context, ProofCheckProfile};
use crate::verifier::verify_recomputed_certificate_sections;

use mpk_cert::{certificate_hash, decode_canonical_certificate, encode::HashBytes};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KernelProfileReport {
    pub module: String,
    pub input_bytes: usize,
    pub certificate_hash: HashBytes,
    pub table_counts: KernelProfileTableCounts,
    pub timings: KernelProfileTimings,
    pub declaration_cache_metrics: CheckerCacheMetrics,
    pub proof_cache_metrics: CheckerCacheMetrics,
    pub combined_cache_metrics: CheckerCacheMetrics,
}

impl KernelProfileReport {
    pub fn hotspots_by_elapsed(&self) -> Vec<KernelProfileHotspot> {
        let mut hotspots = vec![
            KernelProfileHotspot {
                name: "decode",
                elapsed_nanos: self.timings.decode_nanos,
            },
            KernelProfileHotspot {
                name: "typecheck",
                elapsed_nanos: self.timings.typecheck_nanos,
            },
            KernelProfileHotspot {
                name: "defeq",
                elapsed_nanos: self.combined_cache_metrics.defeq.elapsed_nanos,
            },
            KernelProfileHotspot {
                name: "proof-node checking",
                elapsed_nanos: self.timings.proof_node_check_nanos,
            },
        ];
        hotspots.sort_by(|lhs, rhs| {
            rhs.elapsed_nanos
                .cmp(&lhs.elapsed_nanos)
                .then_with(|| lhs.name.cmp(rhs.name))
        });
        hotspots
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KernelProfileTableCounts {
    pub levels: usize,
    pub terms: usize,
    pub proof_nodes: usize,
    pub declarations: usize,
    pub theory_certificates: usize,
    pub exports: usize,
    pub axiom_report_entries: usize,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct KernelProfileTimings {
    pub decode_nanos: u128,
    pub typecheck_nanos: u128,
    pub proof_node_check_nanos: u128,
    pub section_recompute_nanos: u128,
    pub total_nanos: u128,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KernelProfileHotspot {
    pub name: &'static str,
    pub elapsed_nanos: u128,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KernelProfileError {
    stage: KernelProfileStage,
    detail: String,
}

impl KernelProfileError {
    pub fn stage(&self) -> KernelProfileStage {
        self.stage
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }

    fn new(stage: KernelProfileStage, detail: impl Into<String>) -> Self {
        Self {
            stage,
            detail: detail.into(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum KernelProfileStage {
    Decode,
    Typecheck,
    ProofNodeCheck,
    SectionRecompute,
}

impl KernelProfileStage {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Decode => "decode",
            Self::Typecheck => "typecheck",
            Self::ProofNodeCheck => "proof-node checking",
            Self::SectionRecompute => "section recompute",
        }
    }
}

pub fn profile_certificate_bytes(bytes: &[u8]) -> Result<KernelProfileReport, KernelProfileError> {
    let total_start = Instant::now();

    let decode_start = Instant::now();
    let certificate = decode_canonical_certificate(bytes).map_err(|error| {
        KernelProfileError::new(
            KernelProfileStage::Decode,
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
    let decode_nanos = decode_start.elapsed().as_nanos();

    let table_counts = KernelProfileTableCounts {
        levels: certificate.level_table.len(),
        terms: certificate.term_table.len(),
        proof_nodes: certificate.proof_node_table.len(),
        declarations: certificate.declarations.len(),
        theory_certificates: certificate.theory_certificates.len(),
        exports: certificate.export_block.len(),
        axiom_report_entries: certificate.axiom_report.entries.len(),
    };

    let typecheck_start = Instant::now();
    let mut declaration_context = check_declarations_with_cache_timing(&certificate, true)
        .map_err(|error| {
            KernelProfileError::new(KernelProfileStage::Typecheck, error.detail().to_owned())
        })?;
    let typecheck_nanos = typecheck_start.elapsed().as_nanos();
    let declaration_cache_metrics = declaration_context.cache_metrics();

    let proof_start = Instant::now();
    check_proof_nodes_with_context(&mut declaration_context, ProofCheckProfile::MvpStrict)
        .map_err(|error| {
            KernelProfileError::new(
                KernelProfileStage::ProofNodeCheck,
                error.detail().to_owned(),
            )
        })?;
    let proof_node_check_nanos = proof_start.elapsed().as_nanos();
    let combined_cache_metrics = declaration_context.cache_metrics();
    let proof_cache_metrics = combined_cache_metrics.saturating_sub(&declaration_cache_metrics);

    let section_start = Instant::now();
    verify_recomputed_certificate_sections(&certificate).map_err(|error| {
        KernelProfileError::new(
            KernelProfileStage::SectionRecompute,
            format!("{:?}: {}", error.kind(), error.detail()),
        )
    })?;
    let section_recompute_nanos = section_start.elapsed().as_nanos();

    Ok(KernelProfileReport {
        module: certificate.module,
        input_bytes: bytes.len(),
        certificate_hash: certificate_hash(bytes),
        table_counts,
        timings: KernelProfileTimings {
            decode_nanos,
            typecheck_nanos,
            proof_node_check_nanos,
            section_recompute_nanos,
            total_nanos: total_start.elapsed().as_nanos(),
        },
        declaration_cache_metrics,
        proof_cache_metrics,
        combined_cache_metrics,
    })
}
