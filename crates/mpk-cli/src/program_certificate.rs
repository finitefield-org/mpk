//! Self-contained program-certificate assembly for the alpha dual-checker profile.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::reference_checker::execute_reference_checker;
use mpk_cert::encode::{
    AxiomReport, Certificate, CertificateHashes, Declaration, DeclarationKind, HashBytes,
    LevelNode, SourceManifest, TermNode, ZERO_HASH,
};
use mpk_cert::{
    axiom_report_hash_for_report, build_axiom_report, build_export_block, certificate_hash,
    declaration_interface_hash, decode_canonical_certificate, encode_certificate_bounded,
    export_block_hash, hash_hex,
};
use mpk_kernel::{verify_certificate_bytes, VerificationErrorKind, VerificationReport};
use mpk_vc::{
    group_body, validate_verification_limit, GroupedTheoremDeclaration, VcDocument, VcFunction,
    VcGroup, VcMember, VcTerm, VcTypeTerm, VerificationLimitId,
};
use serde::Deserialize;

pub const PROGRAM_CERTIFICATE_ALPHA_PROFILE: &str = "mpk.program_certificate.alpha.v0";
pub const PROGRAM_CERTIFICATE_MODULE: &str = "Policy.Generated";

const STD_BOOL: &str = "Std.Bool";
const STD_BOOL_FALSE: &str = "Std.Bool.false";
const STD_BOOL_TRUE: &str = "Std.Bool.true";
const STD_BOOL_IF: &str = "Std.Bool.if";
const STD_BOOL_NOT: &str = "Std.Bool.not";
const STD_BOOL_AND_VALUE: &str = "Std.Bool.and";
const STD_BOOL_OR: &str = "Std.Bool.or";
const STD_EQ: &str = "Std.Eq";
const STD_EQ_REFL: &str = "Std.Eq.refl";
const STD_LOGIC_IMP: &str = "Std.Logic.Imp";
const STD_LOGIC_AND: &str = "Std.Logic.And";
const STD_LOGIC_AND_INTRO: &str = "Std.Logic.And.intro";
const STD_LOGIC_AND_REC: &str = "Std.Logic.And.rec";
const PROGRAM_BOOL: &str = "Std.Program.Base.Bool";
const PROGRAM_BOOL_FALSE: &str = "Std.Program.Base.Bool.false";
const PROGRAM_BOOL_TRUE: &str = "Std.Program.Base.Bool.true";

const PROGRAM_BASE_HEX: &[u8] = include_bytes!("../../../proofs/program/base/std-program-base.hex");
const STD_BOOL_HEX: &[u8] = include_bytes!("../../../proofs/std/bool/std-bool.hex");
const STD_EQ_HEX: &[u8] = include_bytes!("../../../proofs/std/eq/std-eq.hex");
const STD_LOGIC_HEX: &[u8] = include_bytes!("../../../proofs/std/logic/std-logic.hex");

#[derive(Clone, Copy)]
struct FoundationRegistration {
    bytes: &'static [u8],
    module: &'static str,
    export_hash: &'static str,
    axiom_report_hash: &'static str,
    certificate_hash: &'static str,
}

const FOUNDATION_REGISTRY: [FoundationRegistration; 4] = [
    FoundationRegistration {
        bytes: PROGRAM_BASE_HEX,
        module: "Std.Program.Base",
        export_hash: "3fb122716cace09261f17a630d455b49ce83748851a6543c8a079f48ddc8626b",
        axiom_report_hash: "0ebc281c3a8d37e2d1a9ce033773e2865f96a13186a6364cb3446204c6a990d5",
        certificate_hash: "ecade3952277881551650e3babc5f322f4fe22429af218c4620650adc8f0e373",
    },
    FoundationRegistration {
        bytes: STD_BOOL_HEX,
        module: "Std.Bool",
        export_hash: "1a605be7fdd509a45af48d8c362eb0b54a7b1cdce805de3fe3a8673b07ee50fa",
        axiom_report_hash: "0ebc281c3a8d37e2d1a9ce033773e2865f96a13186a6364cb3446204c6a990d5",
        certificate_hash: "25d957da9ca47d02d917536e12b973a4e328730e12527c98e36f9dbac249a2cd",
    },
    FoundationRegistration {
        bytes: STD_EQ_HEX,
        module: "Std.Eq",
        export_hash: "c8ad36978f94e7c91bd6d09a998c54560fd43c170cd6f37eb3877b61a5856f01",
        axiom_report_hash: "0ebc281c3a8d37e2d1a9ce033773e2865f96a13186a6364cb3446204c6a990d5",
        certificate_hash: "fbd853e1e80194ef65f82b1d59206c2747e4aa6e901f0280cc36e149a4e3ae58",
    },
    FoundationRegistration {
        bytes: STD_LOGIC_HEX,
        module: "Std.Logic",
        export_hash: "5ca1998680609eeb37f09d4447bf8f208242dc0d9284f245de7303fa48a77f73",
        axiom_report_hash: "0ebc281c3a8d37e2d1a9ce033773e2865f96a13186a6364cb3446204c6a990d5",
        certificate_hash: "55a1a55573c15a403d369ec6d29e3a7c294e11310d515e23a6aa15b4b397c0ad",
    },
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProgramCertificateErrorKind {
    Foundation,
    Skeleton,
    Interface,
    CheckerExecution,
    CheckerRejected,
    CheckerDisagreement,
    Limit(VerificationLimitId),
    Internal,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProgramCertificateError {
    kind: ProgramCertificateErrorKind,
    detail: String,
}

impl ProgramCertificateError {
    pub const fn kind(&self) -> ProgramCertificateErrorKind {
        self.kind
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }

    fn new(kind: ProgramCertificateErrorKind, detail: impl Into<String>) -> Self {
        Self {
            kind,
            detail: detail.into(),
        }
    }
}

impl fmt::Display for ProgramCertificateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:?}: {}", self.kind, self.detail)
    }
}

impl std::error::Error for ProgramCertificateError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlannedProgramDeclaration {
    pub name: String,
    pub declaration_hash: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReferenceCheckerReport {
    pub module: String,
    pub declaration_count: usize,
    pub axiom_count: u64,
    pub export_hash: String,
    pub axiom_report_hash: String,
    pub certificate_hash: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedProgramCertificate {
    pub bytes: Vec<u8>,
    pub certificate: Certificate,
    pub rust_report: VerificationReport,
    pub reference_report: ReferenceCheckerReport,
    pub generated_declarations: Vec<PlannedProgramDeclaration>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProgramCheckerVerdict {
    Accepted,
    Rejected,
}

impl ProgramCheckerVerdict {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Accepted => "accepted",
            Self::Rejected => "rejected",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnacceptedProgramCertificate {
    pub bytes: Vec<u8>,
    pub certificate: Certificate,
    pub rust_report: Option<VerificationReport>,
    pub reference_report: Option<ReferenceCheckerReport>,
    pub rust_verdict: ProgramCheckerVerdict,
    pub reference_verdict: ProgramCheckerVerdict,
    pub failure_kind: ProgramCertificateErrorKind,
    pub failure_detail: String,
    pub generated_declarations: Vec<PlannedProgramDeclaration>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProgramCertificateOutcome {
    Pending {
        generated_declarations: Vec<PlannedProgramDeclaration>,
        missing_member_ids: Vec<String>,
    },
    Candidate(Box<CheckedProgramCertificate>),
    Unaccepted(Box<UnacceptedProgramCertificate>),
}

impl ProgramCertificateOutcome {
    pub fn generated_declarations(&self) -> &[PlannedProgramDeclaration] {
        match self {
            Self::Pending {
                generated_declarations,
                ..
            } => generated_declarations,
            Self::Candidate(candidate) => &candidate.generated_declarations,
            Self::Unaccepted(candidate) => &candidate.generated_declarations,
        }
    }
}

/// Builds the complete alpha interface before proof planning and returns an
/// all-or-nothing pending result, a dual-accepted candidate, or a deterministically
/// unaccepted candidate. Execution/protocol failures and unequal accepted reports
/// remain errors and retain no publishable outcome.
pub fn assemble_program_certificate_alpha(
    vc: &VcDocument,
    skeleton: &[GroupedTheoremDeclaration],
    source_manifest: SourceManifest,
) -> Result<ProgramCertificateOutcome, ProgramCertificateError> {
    assemble_program_certificate_alpha_from_functions(&vc.functions, skeleton, source_manifest)
}

/// Internal source-neutral entry point used by the successor policy path. The
/// caller must first validate its versioned VC and skeleton;
/// this function independently rechecks the complete function/declaration
/// projection before invoking the unchanged Certificate v0 assembly path.
pub(crate) fn assemble_program_certificate_alpha_from_functions(
    functions: &[VcFunction],
    skeleton: &[GroupedTheoremDeclaration],
    source_manifest: SourceManifest,
) -> Result<ProgramCertificateOutcome, ProgramCertificateError> {
    validate_skeleton_projection(functions, skeleton)?;
    let model = ProgramModel::new(functions, skeleton)?;
    let requested = requested_foundations(functions, &model)?;
    let foundations = load_foundations()?;
    let selected = select_foundation_closure(&foundations, &requested)?;
    verify_selected_foundations_with_reference(&foundations, &selected)?;
    let generated_names = skeleton
        .iter()
        .map(|declaration| declaration.name.clone())
        .collect::<Vec<_>>();
    let mut builder =
        CertificateBuilder::from_foundations(&foundations, &selected, generated_names)?;

    let foundation_declaration_count = builder.declarations.len();
    let generated_globals = skeleton
        .iter()
        .enumerate()
        .map(|(index, declaration)| {
            let offset = foundation_declaration_count
                .checked_add(index)
                .ok_or_else(|| {
                    ProgramCertificateError::new(
                        ProgramCertificateErrorKind::Internal,
                        "generated declaration count overflow",
                    )
                })?;
            let global = u32::try_from(offset).map_err(|_| {
                ProgramCertificateError::new(
                    ProgramCertificateErrorKind::Internal,
                    "generated declaration count exceeds u32 IDs",
                )
            })?;
            Ok((declaration.name.clone(), global))
        })
        .collect::<Result<BTreeMap<_, _>, ProgramCertificateError>>()?;
    builder.globals.extend(generated_globals.clone());

    let mut planned_types = Vec::with_capacity(skeleton.len());
    for declaration in skeleton {
        let function = model.function(&declaration.function_id)?;
        let group = model.group(&declaration.name)?.1;
        let mut context = LowerContext::default();
        let ty = lower_parameter_binders(&mut builder, &model, function, group, &mut context, 0)?;
        planned_types.push(ty);
    }

    let generated_declarations = skeleton
        .iter()
        .zip(&planned_types)
        .map(|(skeleton_declaration, ty)| {
            let declaration = Declaration {
                name: builder.name_id(&skeleton_declaration.name)?,
                kind: DeclarationKind::Theorem { ty: *ty, proof: 0 },
            };
            let hash = declaration_interface_hash(&builder.name_table, &declaration)
                .map_err(|error| interface_error(error.detail()))?;
            Ok(PlannedProgramDeclaration {
                name: skeleton_declaration.name.clone(),
                declaration_hash: hash_hex(&hash),
            })
        })
        .collect::<Result<Vec<_>, ProgramCertificateError>>()?;

    let interface_term_count = builder.term_table.len();
    builder.enforce_generated_proof_depth = true;
    let mut proofs = Vec::with_capacity(skeleton.len());
    let mut missing = BTreeSet::new();
    for declaration in skeleton {
        let function = model.function(&declaration.function_id)?;
        let group = model.group(&declaration.name)?.1;
        let mut context = LowerContext::default();
        match prove_parameter_binders(
            &mut builder,
            &model,
            function,
            group,
            &mut context,
            0,
            &mut missing,
        )? {
            Some(proof) => {
                enforce_verification_limit(
                    VerificationLimitId::GeneratedProofDepth,
                    builder.term_depth(proof)?,
                )?;
                proofs.push(proof);
            }
            None => proofs.push(0),
        }
    }
    if !missing.is_empty() {
        builder.term_table.truncate(interface_term_count);
        builder.term_depths.truncate(interface_term_count);
        return Ok(ProgramCertificateOutcome::Pending {
            generated_declarations,
            missing_member_ids: missing.into_iter().collect(),
        });
    }

    for ((declaration, ty), proof) in skeleton.iter().zip(planned_types).zip(proofs) {
        builder.declarations.push(Declaration {
            name: builder.name_id(&declaration.name)?,
            kind: DeclarationKind::Theorem { ty, proof },
        });
    }
    let mut certificate = Certificate {
        module: PROGRAM_CERTIFICATE_MODULE.to_owned(),
        imports: Vec::new(),
        name_table: builder.name_table,
        level_table: builder.level_table,
        term_table: builder.term_table,
        proof_node_table: Vec::new(),
        declarations: builder.declarations,
        theory_certificates: Vec::new(),
        export_block: Vec::new(),
        axiom_report: AxiomReport::default(),
        source_manifest: Some(source_manifest),
        hashes: CertificateHashes::default(),
    };
    certificate.export_block =
        build_export_block(&certificate).map_err(|error| interface_error(error.detail()))?;
    let actual_generated = certificate
        .export_block
        .iter()
        .skip(foundation_declaration_count)
        .map(|entry| {
            let name = certificate
                .name_table
                .get(entry.name as usize)
                .ok_or_else(|| interface_error("generated export has a missing name"))?;
            Ok(PlannedProgramDeclaration {
                name: name.clone(),
                declaration_hash: hash_hex(&entry.declaration_hash),
            })
        })
        .collect::<Result<Vec<_>, ProgramCertificateError>>()?;
    if actual_generated != generated_declarations {
        return Err(interface_error(
            "candidate declaration hashes differ from the complete interface plan",
        ));
    }
    certificate.axiom_report =
        build_axiom_report(&certificate).map_err(|error| interface_error(error.detail()))?;
    validate_alpha_candidate_shape(&certificate, foundation_declaration_count)?;
    certificate.hashes.export_hash = export_block_hash(&certificate.export_block);
    certificate.hashes.axiom_report_hash = axiom_report_hash_for_report(&certificate.axiom_report);
    certificate.hashes.certificate_hash = ZERO_HASH;
    let certificate_maximum = usize::try_from(
        VerificationLimitId::CanonicalCertificateBytes.maximum(),
    )
    .map_err(|_| {
        limit_error(
            VerificationLimitId::CanonicalCertificateBytes,
            "maximum exceeds usize",
        )
    })?;
    let bytes = encode_certificate_bounded(&certificate, certificate_maximum).map_err(|_| {
        limit_error(
            VerificationLimitId::CanonicalCertificateBytes,
            "canonical certificate exceeds the registered byte maximum",
        )
    })?;
    // Always submit the identical candidate bytes to both implementations
    // before classifying their acceptance pair.
    let rust_result = verify_certificate_bytes(&bytes).map_err(|error| {
        let detail = format!("Rust fast kernel failed candidate: {}", error.detail());
        if error.kind() == VerificationErrorKind::InternalInvariant {
            checker_execution(detail)
        } else {
            ProgramCertificateError::new(ProgramCertificateErrorKind::CheckerRejected, detail)
        }
    });
    let reference_result = run_reference_checker(&bytes);
    match reconcile_checker_results(&bytes, rust_result, reference_result)? {
        ReconciledCheckerResults::Accepted {
            rust_report,
            reference_report,
        } => Ok(ProgramCertificateOutcome::Candidate(Box::new(
            CheckedProgramCertificate {
                bytes,
                certificate,
                rust_report,
                reference_report,
                generated_declarations,
            },
        ))),
        ReconciledCheckerResults::Unaccepted {
            rust_report,
            reference_report,
            rust_verdict,
            reference_verdict,
            failure,
        } => Ok(ProgramCertificateOutcome::Unaccepted(Box::new(
            UnacceptedProgramCertificate {
                bytes,
                certificate,
                rust_report,
                reference_report,
                rust_verdict,
                reference_verdict,
                failure_kind: failure.kind,
                failure_detail: failure.detail,
                generated_declarations,
            },
        ))),
    }
}

fn validate_alpha_candidate_shape(
    certificate: &Certificate,
    generated_start: usize,
) -> Result<(), ProgramCertificateError> {
    if !certificate.imports.is_empty()
        || !certificate.proof_node_table.is_empty()
        || !certificate.theory_certificates.is_empty()
    {
        return Err(interface_error(
            "alpha candidate has a nonempty import, proof-node, or theory-certificate table",
        ));
    }
    if certificate.declarations.iter().any(|declaration| {
        matches!(
            declaration.kind,
            DeclarationKind::Axiom { .. } | DeclarationKind::TheoryPrimitive { .. }
        )
    }) {
        return Err(interface_error(
            "alpha candidate contains an axiom or theory primitive",
        ));
    }
    if certificate.axiom_report.summary.total_axiom_count != 0
        || !certificate.axiom_report.entries.is_empty()
        || !certificate.axiom_report.declaration_dependencies.is_empty()
    {
        return Err(interface_error(
            "alpha candidate has a non-zero axiom report",
        ));
    }
    for declaration in certificate.declarations.iter().skip(generated_start) {
        let DeclarationKind::Theorem { ty, .. } = declaration.kind else {
            return Err(interface_error(
                "generated alpha declaration is not a theorem",
            ));
        };
        if terminal_type_is_raw_boolean(certificate, ty)? {
            return Err(interface_error(
                "generated alpha theorem has a raw Boolean carrier as its proposition",
            ));
        }
    }
    Ok(())
}

fn terminal_type_is_raw_boolean(
    certificate: &Certificate,
    mut term: u32,
) -> Result<bool, ProgramCertificateError> {
    loop {
        match certificate
            .term_table
            .get(term as usize)
            .ok_or_else(|| interface_error("generated theorem type term is missing"))?
        {
            TermNode::Pi { body, .. } => term = *body,
            TermNode::Const { global, .. } => {
                let declaration = certificate
                    .declarations
                    .get(*global as usize)
                    .ok_or_else(|| interface_error("generated theorem type global is missing"))?;
                let name = certificate
                    .name_table
                    .get(declaration.name as usize)
                    .ok_or_else(|| interface_error("generated theorem type name is missing"))?;
                return Ok(matches!(name.as_str(), STD_BOOL | PROGRAM_BOOL));
            }
            _ => return Ok(false),
        }
    }
}

fn validate_skeleton_projection(
    functions: &[VcFunction],
    skeleton: &[GroupedTheoremDeclaration],
) -> Result<(), ProgramCertificateError> {
    let expected_count = functions
        .iter()
        .map(|function| function.groups.len())
        .sum::<usize>();
    if expected_count != skeleton.len() {
        return Err(skeleton_error(
            "grouped skeleton declaration count differs from the VC",
        ));
    }
    let expected = functions
        .iter()
        .flat_map(|function| function.groups.iter().map(move |group| (function, group)));
    for ((function, group), declaration) in expected.zip(skeleton) {
        let body = group_body(function, group)
            .map_err(|error| skeleton_error(format!("rebuild grouped body: {error}")))?;
        if declaration.name != group.declaration_name
            || declaration.function_id != function.function_id
            || declaration.group_id != group.id
            || declaration.group_kind != group.kind
            || declaration.member_ids != group.member_ids
            || declaration.dependencies != group.dependencies
            || declaration.theorem_type.binders != function.parameters
            || declaration.theorem_type.body != body
        {
            return Err(skeleton_error(
                "grouped skeleton differs from the independently reconstructed VC projection",
            ));
        }
    }
    Ok(())
}

struct ProgramModel<'a> {
    functions: BTreeMap<&'a str, &'a VcFunction>,
    groups: BTreeMap<&'a str, (&'a VcFunction, &'a VcGroup)>,
    positions: BTreeMap<&'a str, usize>,
}

impl<'a> ProgramModel<'a> {
    fn new(
        vc_functions: &'a [VcFunction],
        skeleton: &'a [GroupedTheoremDeclaration],
    ) -> Result<Self, ProgramCertificateError> {
        let mut functions = BTreeMap::new();
        let mut groups = BTreeMap::new();
        for function in vc_functions {
            if functions
                .insert(function.function_id.as_str(), function)
                .is_some()
            {
                return Err(skeleton_error("duplicate VC function identity"));
            }
            for group in &function.groups {
                if groups
                    .insert(group.declaration_name.as_str(), (function, group))
                    .is_some()
                {
                    return Err(skeleton_error("duplicate generated declaration name"));
                }
            }
        }
        let positions = skeleton
            .iter()
            .enumerate()
            .map(|(index, declaration)| (declaration.name.as_str(), index))
            .collect::<BTreeMap<_, _>>();
        for (index, declaration) in skeleton.iter().enumerate() {
            for dependency in &declaration.dependencies {
                let Some(position) = positions.get(dependency.as_str()) else {
                    return Err(skeleton_error("generated dependency is absent"));
                };
                if *position >= index {
                    return Err(skeleton_error(
                        "generated dependency is not an earlier declaration",
                    ));
                }
            }
            let (function, group) = groups
                .get(declaration.name.as_str())
                .copied()
                .ok_or_else(|| skeleton_error("generated declaration is absent from the VC"))?;
            for requirement in &function.requires {
                validate_generated_references(
                    requirement,
                    declaration,
                    &positions,
                    &groups,
                    index,
                )?;
            }
            for member_id in &group.member_ids {
                let member = function
                    .members
                    .iter()
                    .find(|member| &member.id == member_id)
                    .ok_or_else(|| skeleton_error("group member is absent from its function"))?;
                for assumption in &member.assumptions {
                    validate_generated_references(
                        assumption,
                        declaration,
                        &positions,
                        &groups,
                        index,
                    )?;
                }
                validate_generated_references(
                    &member.conclusion,
                    declaration,
                    &positions,
                    &groups,
                    index,
                )?;
            }
        }
        Ok(Self {
            functions,
            groups,
            positions,
        })
    }

    fn function(&self, id: &str) -> Result<&'a VcFunction, ProgramCertificateError> {
        self.functions
            .get(id)
            .copied()
            .ok_or_else(|| skeleton_error("skeleton function is absent from the VC"))
    }

    fn group(&self, name: &str) -> Result<(&'a VcFunction, &'a VcGroup), ProgramCertificateError> {
        self.groups
            .get(name)
            .copied()
            .ok_or_else(|| skeleton_error("generated declaration is absent from the VC"))
    }

    fn is_group_name(&self, name: &str) -> bool {
        self.groups.contains_key(name)
    }

    fn is_earlier(&self, name: &str, current: &str) -> bool {
        matches!(
            (self.positions.get(name), self.positions.get(current)),
            (Some(target), Some(source)) if target < source
        )
    }

    fn is_declared_dependency(&self, name: &str, current: &str) -> bool {
        self.is_earlier(name, current)
            && self
                .groups
                .get(current)
                .is_some_and(|(_, group)| group.dependencies.iter().any(|entry| entry == name))
    }
}

fn validate_generated_references(
    term: &VcTerm,
    declaration: &GroupedTheoremDeclaration,
    positions: &BTreeMap<&str, usize>,
    groups: &BTreeMap<&str, (&VcFunction, &VcGroup)>,
    current_position: usize,
) -> Result<(), ProgramCertificateError> {
    match term {
        VcTerm::Apply { function, args } => {
            if groups.contains_key(function.as_str())
                && (!declaration
                    .dependencies
                    .iter()
                    .any(|dependency| dependency == function)
                    || positions
                        .get(function.as_str())
                        .is_none_or(|position| *position >= current_position))
            {
                return Err(skeleton_error(
                    "generated proposition reference is not an exact declared earlier dependency",
                ));
            }
            for argument in args {
                validate_generated_references(
                    argument,
                    declaration,
                    positions,
                    groups,
                    current_position,
                )?;
            }
            Ok(())
        }
        VcTerm::Forall { body, .. } => {
            validate_generated_references(body, declaration, positions, groups, current_position)
        }
        VcTerm::Convert { value, .. } => {
            validate_generated_references(value, declaration, positions, groups, current_position)
        }
        VcTerm::Var { .. }
        | VcTerm::Bound { .. }
        | VcTerm::Constant { .. }
        | VcTerm::BitVecLiteral { .. } => Ok(()),
    }
}

struct FoundationSource {
    bytes: Vec<u8>,
    certificate: Certificate,
    certificate_hash: HashBytes,
    rust_report: VerificationReport,
}

fn load_foundations() -> Result<Vec<FoundationSource>, ProgramCertificateError> {
    let mut sources = Vec::with_capacity(FOUNDATION_REGISTRY.len());
    for registration in FOUNDATION_REGISTRY {
        let bytes = decode_hex(registration.bytes)?;
        let certificate = decode_canonical_certificate(&bytes).map_err(|error| {
            ProgramCertificateError::new(
                ProgramCertificateErrorKind::Foundation,
                format!(
                    "registered foundation {} is not canonical: {error:?}",
                    registration.module
                ),
            )
        })?;
        let report = verify_certificate_bytes(&bytes).map_err(|error| {
            ProgramCertificateError::new(
                if error.kind() == VerificationErrorKind::InternalInvariant {
                    ProgramCertificateErrorKind::CheckerExecution
                } else {
                    ProgramCertificateErrorKind::Foundation
                },
                format!(
                    "registered foundation {} was rejected: {}",
                    registration.module,
                    error.detail()
                ),
            )
        })?;
        if certificate.module != registration.module
            || hash_hex(&report.export_hash) != registration.export_hash
            || hash_hex(&report.axiom_report_hash) != registration.axiom_report_hash
            || hash_hex(&report.certificate_hash) != registration.certificate_hash
            || report.axiom_count != 0
            || !certificate.imports.is_empty()
            || !certificate.proof_node_table.is_empty()
            || !certificate.theory_certificates.is_empty()
        {
            return Err(ProgramCertificateError::new(
                ProgramCertificateErrorKind::Foundation,
                format!(
                    "registered foundation tuple mismatch for {}",
                    registration.module
                ),
            ));
        }
        sources.push(FoundationSource {
            bytes,
            certificate,
            certificate_hash: report.certificate_hash,
            rust_report: report,
        });
    }
    sources.sort_by(|left, right| {
        (
            left.certificate.module.as_str(),
            left.certificate.hashes.export_hash,
            left.certificate_hash,
        )
            .cmp(&(
                right.certificate.module.as_str(),
                right.certificate.hashes.export_hash,
                right.certificate_hash,
            ))
    });
    Ok(sources)
}

fn verify_selected_foundations_with_reference(
    sources: &[FoundationSource],
    selected: &SelectedFoundationClosure,
) -> Result<(), ProgramCertificateError> {
    for source_index in selected.declarations.keys() {
        let source = sources
            .get(*source_index)
            .ok_or_else(|| foundation_error("selected foundation source is missing"))?;
        let reference = run_reference_checker(&source.bytes)
            .map_err(|error| foundation_reference_error(&source.certificate.module, error))?;
        require_checker_agreement(&source.bytes, &source.rust_report, &reference).map_err(
            |error| {
                foundation_error(format!(
                    "registered foundation {:?} checker disagreement: {}",
                    source.certificate.module,
                    error.detail()
                ))
            },
        )?;
    }
    Ok(())
}

fn decode_hex(input: &[u8]) -> Result<Vec<u8>, ProgramCertificateError> {
    let compact = input
        .iter()
        .copied()
        .filter(|byte| !byte.is_ascii_whitespace())
        .collect::<Vec<_>>();
    if compact.len() % 2 != 0 {
        return Err(ProgramCertificateError::new(
            ProgramCertificateErrorKind::Foundation,
            "registered foundation hex has an odd number of digits",
        ));
    }
    compact
        .chunks_exact(2)
        .map(|pair| {
            let pair = std::str::from_utf8(pair).map_err(|_| {
                ProgramCertificateError::new(
                    ProgramCertificateErrorKind::Foundation,
                    "registered foundation hex is not ASCII",
                )
            })?;
            u8::from_str_radix(pair, 16).map_err(|_| {
                ProgramCertificateError::new(
                    ProgramCertificateErrorKind::Foundation,
                    "registered foundation contains a non-hex digit",
                )
            })
        })
        .collect()
}

fn requested_foundations(
    functions: &[VcFunction],
    model: &ProgramModel<'_>,
) -> Result<BTreeSet<String>, ProgramCertificateError> {
    let mut names = BTreeSet::from([
        STD_LOGIC_IMP.to_owned(),
        STD_EQ.to_owned(),
        STD_EQ_REFL.to_owned(),
        STD_BOOL.to_owned(),
        STD_BOOL_TRUE.to_owned(),
    ]);
    let mut has_conjunction = false;
    let mut has_projectable_conjunction = false;
    for function in functions {
        for parameter in &function.parameters {
            collect_type_names(&parameter.r#type, &mut names)?;
            if parameter.r#type
                == (VcTypeTerm::Constant {
                    name: PROGRAM_BOOL.to_owned(),
                })
            {
                names.insert(PROGRAM_BOOL_TRUE.to_owned());
            }
        }
        for requirement in &function.requires {
            collect_term_names(requirement, model, &mut names)?;
        }
        has_conjunction |= function.requires.len() > 1;
        has_projectable_conjunction |= function.requires.len() > 1;
        for member in &function.members {
            for binder in &member.local_binders {
                collect_type_names(binder, &mut names)?;
            }
            for assumption in &member.assumptions {
                collect_term_names(assumption, model, &mut names)?;
            }
            collect_term_names(&member.conclusion, model, &mut names)?;
            has_conjunction |= member.assumptions.len() > 1;
            has_projectable_conjunction |= member.assumptions.len() > 1;
        }
        has_conjunction |= function
            .groups
            .iter()
            .any(|group| group.member_ids.len() > 1);
    }
    if has_conjunction {
        names.insert(STD_LOGIC_AND.to_owned());
        names.insert(STD_LOGIC_AND_INTRO.to_owned());
    }
    if has_projectable_conjunction {
        names.insert(STD_LOGIC_AND_REC.to_owned());
    }
    if names.contains(PROGRAM_BOOL) || names.contains(PROGRAM_BOOL_FALSE) {
        names.insert(PROGRAM_BOOL_TRUE.to_owned());
    }
    Ok(names)
}

fn collect_type_names(
    ty: &VcTypeTerm,
    names: &mut BTreeSet<String>,
) -> Result<(), ProgramCertificateError> {
    match ty {
        VcTypeTerm::Constant { name } => {
            names.insert(name.clone());
            Ok(())
        }
        VcTypeTerm::Apply { .. }
        | VcTypeTerm::NatLiteral { .. }
        | VcTypeTerm::StringLiteral { .. } => Err(interface_error(
            "alpha profile has no registered lowering for applied or literal VC types",
        )),
    }
}

fn collect_term_names(
    term: &VcTerm,
    model: &ProgramModel<'_>,
    names: &mut BTreeSet<String>,
) -> Result<(), ProgramCertificateError> {
    match term {
        VcTerm::Var { .. } | VcTerm::Bound { .. } => Ok(()),
        VcTerm::Constant { name } => {
            if !model.is_group_name(name) {
                names.insert(name.clone());
            }
            Ok(())
        }
        VcTerm::Apply { function, args } => {
            if !model.is_group_name(function) {
                names.insert(function.clone());
            }
            for argument in args {
                collect_term_names(argument, model, names)?;
            }
            Ok(())
        }
        VcTerm::Forall { binder_type, body } => {
            collect_type_names(binder_type, names)?;
            collect_term_names(body, model, names)
        }
        VcTerm::BitVecLiteral { .. } | VcTerm::Convert { .. } => Err(interface_error(
            "alpha profile has no registered bitvector-literal or conversion lowering",
        )),
    }
}

#[derive(Default)]
struct SelectedFoundationClosure {
    declarations: BTreeMap<usize, BTreeSet<u32>>,
}

fn select_foundation_closure(
    sources: &[FoundationSource],
    requested: &BTreeSet<String>,
) -> Result<SelectedFoundationClosure, ProgramCertificateError> {
    let mut exported = BTreeMap::<&str, (usize, u32)>::new();
    for (source_index, source) in sources.iter().enumerate() {
        for entry in &source.certificate.export_block {
            let name = source
                .certificate
                .name_table
                .get(entry.name as usize)
                .ok_or_else(|| foundation_error("foundation export has a missing name"))?;
            if exported
                .insert(name.as_str(), (source_index, entry.declaration))
                .is_some()
            {
                return Err(foundation_error("duplicate global foundation export"));
            }
        }
    }
    let mut selected = SelectedFoundationClosure::default();
    for name in requested {
        let Some((source, declaration)) = exported.get(name.as_str()).copied() else {
            return Err(interface_error(format!(
                "unregistered alpha foundation interface {name:?}"
            )));
        };
        collect_declaration_closure(sources, source, declaration, &mut selected)?;
    }
    Ok(selected)
}

fn collect_declaration_closure(
    sources: &[FoundationSource],
    source_index: usize,
    declaration_id: u32,
    selected: &mut SelectedFoundationClosure,
) -> Result<(), ProgramCertificateError> {
    if !selected
        .declarations
        .entry(source_index)
        .or_default()
        .insert(declaration_id)
    {
        return Ok(());
    }
    let source = &sources[source_index].certificate;
    let declaration = source
        .declarations
        .get(declaration_id as usize)
        .ok_or_else(|| foundation_error("foundation declaration ID is missing"))?;
    let mut roots = Vec::new();
    match &declaration.kind {
        DeclarationKind::Axiom { .. } | DeclarationKind::TheoryPrimitive { .. } => {
            return Err(foundation_error(
                "alpha foundation closure reached an axiom or theory primitive",
            ));
        }
        DeclarationKind::Def { ty, value, .. } => roots.extend([*ty, *value]),
        DeclarationKind::Theorem { ty, proof } => roots.extend([*ty, *proof]),
        DeclarationKind::Inductive { ty } => roots.push(*ty),
        DeclarationKind::Constructor { ty, inductive, .. }
        | DeclarationKind::Recursor { ty, inductive, .. } => {
            roots.push(*ty);
            collect_declaration_closure(sources, source_index, *inductive, selected)?;
        }
    }
    let mut visited = BTreeSet::new();
    for root in roots {
        collect_term_declarations(
            sources,
            source_index,
            root,
            declaration_id,
            selected,
            &mut visited,
        )?;
    }
    Ok(())
}

fn collect_term_declarations(
    sources: &[FoundationSource],
    source_index: usize,
    term_id: u32,
    current_declaration: u32,
    selected: &mut SelectedFoundationClosure,
    visited: &mut BTreeSet<u32>,
) -> Result<(), ProgramCertificateError> {
    if !visited.insert(term_id) {
        return Ok(());
    }
    let source = &sources[source_index].certificate;
    let term = source
        .term_table
        .get(term_id as usize)
        .ok_or_else(|| foundation_error("foundation term ID is missing"))?;
    match term {
        TermNode::Sort(_) | TermNode::Var(_) => {}
        TermNode::Const { global, .. } => {
            if *global >= current_declaration {
                return Err(foundation_error(
                    "foundation declaration contains a forward global reference",
                ));
            }
            collect_declaration_closure(sources, source_index, *global, selected)?;
        }
        TermNode::App {
            function,
            arguments,
        } => {
            collect_term_declarations(
                sources,
                source_index,
                *function,
                current_declaration,
                selected,
                visited,
            )?;
            for argument in arguments {
                collect_term_declarations(
                    sources,
                    source_index,
                    *argument,
                    current_declaration,
                    selected,
                    visited,
                )?;
            }
        }
        TermNode::Lam { ty, body } | TermNode::Pi { ty, body } => {
            for child in [*ty, *body] {
                collect_term_declarations(
                    sources,
                    source_index,
                    child,
                    current_declaration,
                    selected,
                    visited,
                )?;
            }
        }
        TermNode::Let { ty, value, body } => {
            for child in [*ty, *value, *body] {
                collect_term_declarations(
                    sources,
                    source_index,
                    child,
                    current_declaration,
                    selected,
                    visited,
                )?;
            }
        }
    }
    Ok(())
}

struct CertificateBuilder {
    name_table: Vec<String>,
    name_ids: BTreeMap<String, u32>,
    level_table: Vec<LevelNode>,
    term_table: Vec<TermNode>,
    term_depths: Vec<u64>,
    declarations: Vec<Declaration>,
    globals: BTreeMap<String, u32>,
    shift_cache: BTreeMap<(u32, u32, u32), u32>,
    enforce_generated_proof_depth: bool,
}

fn enforce_verification_limit(
    limit: VerificationLimitId,
    observed: u64,
) -> Result<(), ProgramCertificateError> {
    validate_verification_limit(limit.as_str(), observed)
        .map_err(|error| limit_error(limit, error.to_string()))
}

fn limit_error(limit: VerificationLimitId, detail: impl Into<String>) -> ProgramCertificateError {
    ProgramCertificateError::new(ProgramCertificateErrorKind::Limit(limit), detail)
}

fn term_node_depth(node: &TermNode, depths: &[u64]) -> Result<u64, ProgramCertificateError> {
    let child = |id: u32| {
        depths
            .get(id as usize)
            .copied()
            .ok_or_else(|| interface_error("term references a missing earlier depth"))
    };
    let maximum_child = match node {
        TermNode::Sort(_) | TermNode::Var(_) | TermNode::Const { .. } => return Ok(1),
        TermNode::App {
            function,
            arguments,
        } => {
            let mut maximum = child(*function)?;
            for argument in arguments {
                maximum = maximum.max(child(*argument)?);
            }
            maximum
        }
        TermNode::Lam { ty, body } | TermNode::Pi { ty, body } => child(*ty)?.max(child(*body)?),
        TermNode::Let { ty, value, body } => child(*ty)?.max(child(*value)?).max(child(*body)?),
    };
    maximum_child.checked_add(1).ok_or_else(|| {
        limit_error(
            VerificationLimitId::GeneratedProofDepth,
            "generated proof depth counter overflow",
        )
    })
}

impl CertificateBuilder {
    fn from_foundations(
        sources: &[FoundationSource],
        selected: &SelectedFoundationClosure,
        generated_names: Vec<String>,
    ) -> Result<Self, ProgramCertificateError> {
        let mut all_names = BTreeSet::new();
        for (source_index, declaration_ids) in &selected.declarations {
            let source = &sources[*source_index].certificate;
            for declaration_id in declaration_ids {
                let declaration = &source.declarations[*declaration_id as usize];
                all_names.insert(source.name_table[declaration.name as usize].clone());
            }
        }
        for name in generated_names {
            if !all_names.insert(name) {
                return Err(interface_error(
                    "generated declaration duplicates a foundation global name",
                ));
            }
        }
        let name_table = all_names.into_iter().collect::<Vec<_>>();
        let name_ids = name_table
            .iter()
            .enumerate()
            .map(|(index, name)| {
                Ok((
                    name.clone(),
                    u32::try_from(index)
                        .map_err(|_| interface_error("name table exceeds u32 IDs"))?,
                ))
            })
            .collect::<Result<BTreeMap<_, _>, ProgramCertificateError>>()?;
        let mut builder = Self {
            name_table,
            name_ids,
            level_table: Vec::new(),
            term_table: Vec::new(),
            term_depths: Vec::new(),
            declarations: Vec::new(),
            globals: BTreeMap::new(),
            shift_cache: BTreeMap::new(),
            enforce_generated_proof_depth: false,
        };

        let mut global_maps = BTreeMap::<(usize, u32), u32>::new();
        for (source_index, source) in sources.iter().enumerate() {
            let Some(declaration_ids) = selected.declarations.get(&source_index) else {
                continue;
            };
            for declaration_id in declaration_ids {
                let declaration = &source.certificate.declarations[*declaration_id as usize];
                let name = source.certificate.name_table[declaration.name as usize].clone();
                let global = u32::try_from(builder.globals.len())
                    .map_err(|_| interface_error("foundation closure exceeds u32 globals"))?;
                if builder.globals.insert(name, global).is_some() {
                    return Err(interface_error("duplicate foundation global name"));
                }
                global_maps.insert((source_index, *declaration_id), global);
            }
        }

        let mut level_maps = BTreeMap::<(usize, u32), u32>::new();
        let mut term_maps = BTreeMap::<(usize, u32), u32>::new();
        for (source_index, source) in sources.iter().enumerate() {
            let Some(declaration_ids) = selected.declarations.get(&source_index) else {
                continue;
            };
            for declaration_id in declaration_ids {
                let declaration = &source.certificate.declarations[*declaration_id as usize];
                let name = source.certificate.name_table[declaration.name as usize].clone();
                let kind = copy_declaration_kind(
                    &mut builder,
                    source_index,
                    &source.certificate,
                    &global_maps,
                    &mut level_maps,
                    &mut term_maps,
                    &declaration.kind,
                )?;
                builder.declarations.push(Declaration {
                    name: builder.name_id(&name)?,
                    kind,
                });
            }
        }
        Ok(builder)
    }

    fn name_id(&self, name: &str) -> Result<u32, ProgramCertificateError> {
        self.name_ids.get(name).copied().ok_or_else(|| {
            interface_error(format!("name {name:?} is absent from the fixed name table"))
        })
    }

    fn global(&self, name: &str) -> Result<u32, ProgramCertificateError> {
        self.globals.get(name).copied().ok_or_else(|| {
            interface_error(format!("global {name:?} is absent from the alpha closure"))
        })
    }

    fn intern_level(&mut self, node: LevelNode) -> Result<u32, ProgramCertificateError> {
        if let Some(index) = self
            .level_table
            .iter()
            .position(|existing| existing == &node)
        {
            return u32::try_from(index).map_err(|_| interface_error("level ID exceeds u32"));
        }
        let id = u32::try_from(self.level_table.len())
            .map_err(|_| interface_error("level table exceeds u32 IDs"))?;
        self.level_table.push(node);
        Ok(id)
    }

    fn intern_term(&mut self, node: TermNode) -> Result<u32, ProgramCertificateError> {
        if let Some(index) = self
            .term_table
            .iter()
            .position(|existing| existing == &node)
        {
            if self.enforce_generated_proof_depth {
                enforce_verification_limit(
                    VerificationLimitId::GeneratedProofDepth,
                    *self
                        .term_depths
                        .get(index)
                        .ok_or_else(|| interface_error("term depth table is incomplete"))?,
                )?;
            }
            return u32::try_from(index).map_err(|_| interface_error("term ID exceeds u32"));
        }
        let depth = term_node_depth(&node, &self.term_depths)?;
        if self.enforce_generated_proof_depth {
            enforce_verification_limit(VerificationLimitId::GeneratedProofDepth, depth)?;
        }
        let id = u32::try_from(self.term_table.len())
            .map_err(|_| interface_error("term table exceeds u32 IDs"))?;
        self.term_table.push(node);
        self.term_depths.push(depth);
        Ok(id)
    }

    fn term_depth(&self, term: u32) -> Result<u64, ProgramCertificateError> {
        self.term_depths
            .get(term as usize)
            .copied()
            .ok_or_else(|| interface_error("generated proof term depth is missing"))
    }

    fn constant(&mut self, name: &str) -> Result<u32, ProgramCertificateError> {
        let global = self.global(name)?;
        self.intern_term(TermNode::Const {
            global,
            levels: Vec::new(),
        })
    }

    fn app(
        &mut self,
        function: u32,
        arguments: impl IntoIterator<Item = u32>,
    ) -> Result<u32, ProgramCertificateError> {
        let arguments = arguments.into_iter().collect::<Vec<_>>();
        if arguments.is_empty() {
            return Ok(function);
        }
        self.intern_term(TermNode::App {
            function,
            arguments,
        })
    }

    fn var(&mut self, index: u32) -> Result<u32, ProgramCertificateError> {
        self.intern_term(TermNode::Var(index))
    }

    fn lam(&mut self, ty: u32, body: u32) -> Result<u32, ProgramCertificateError> {
        self.intern_term(TermNode::Lam { ty, body })
    }

    fn pi(&mut self, ty: u32, body: u32) -> Result<u32, ProgramCertificateError> {
        self.intern_term(TermNode::Pi { ty, body })
    }

    fn shift(
        &mut self,
        term: u32,
        amount: u32,
        cutoff: u32,
    ) -> Result<u32, ProgramCertificateError> {
        if amount == 0 {
            return Ok(term);
        }
        if let Some(shifted) = self.shift_cache.get(&(term, amount, cutoff)) {
            return Ok(*shifted);
        }
        let node = self
            .term_table
            .get(term as usize)
            .cloned()
            .ok_or_else(|| interface_error("cannot shift a missing term"))?;
        let shifted_node = match node {
            TermNode::Sort(level) => TermNode::Sort(level),
            TermNode::Var(index) => TermNode::Var(if index >= cutoff {
                index
                    .checked_add(amount)
                    .ok_or_else(|| interface_error("de Bruijn shift overflow"))?
            } else {
                index
            }),
            TermNode::Const { global, levels } => TermNode::Const { global, levels },
            TermNode::App {
                function,
                arguments,
            } => TermNode::App {
                function: self.shift(function, amount, cutoff)?,
                arguments: arguments
                    .into_iter()
                    .map(|argument| self.shift(argument, amount, cutoff))
                    .collect::<Result<_, _>>()?,
            },
            TermNode::Lam { ty, body } => TermNode::Lam {
                ty: self.shift(ty, amount, cutoff)?,
                body: self.shift(body, amount, cutoff + 1)?,
            },
            TermNode::Pi { ty, body } => TermNode::Pi {
                ty: self.shift(ty, amount, cutoff)?,
                body: self.shift(body, amount, cutoff + 1)?,
            },
            TermNode::Let { ty, value, body } => TermNode::Let {
                ty: self.shift(ty, amount, cutoff)?,
                value: self.shift(value, amount, cutoff)?,
                body: self.shift(body, amount, cutoff + 1)?,
            },
        };
        let shifted = self.intern_term(shifted_node)?;
        self.shift_cache.insert((term, amount, cutoff), shifted);
        Ok(shifted)
    }
}

fn copy_declaration_kind(
    builder: &mut CertificateBuilder,
    source_index: usize,
    source: &Certificate,
    globals: &BTreeMap<(usize, u32), u32>,
    levels: &mut BTreeMap<(usize, u32), u32>,
    terms: &mut BTreeMap<(usize, u32), u32>,
    kind: &DeclarationKind,
) -> Result<DeclarationKind, ProgramCertificateError> {
    let term = |builder: &mut CertificateBuilder,
                id: u32,
                levels: &mut BTreeMap<(usize, u32), u32>,
                terms: &mut BTreeMap<(usize, u32), u32>| {
        copy_term(builder, source_index, source, globals, levels, terms, id)
    };
    Ok(match kind {
        DeclarationKind::Axiom { .. } | DeclarationKind::TheoryPrimitive { .. } => {
            return Err(foundation_error("unsupported foundation declaration kind"));
        }
        DeclarationKind::Def {
            ty,
            value,
            reducibility,
        } => DeclarationKind::Def {
            ty: term(builder, *ty, levels, terms)?,
            value: term(builder, *value, levels, terms)?,
            reducibility: *reducibility,
        },
        DeclarationKind::Theorem { ty, proof } => DeclarationKind::Theorem {
            ty: term(builder, *ty, levels, terms)?,
            proof: term(builder, *proof, levels, terms)?,
        },
        DeclarationKind::Inductive { ty } => DeclarationKind::Inductive {
            ty: term(builder, *ty, levels, terms)?,
        },
        DeclarationKind::Constructor {
            ty,
            inductive,
            generated,
        } => DeclarationKind::Constructor {
            ty: term(builder, *ty, levels, terms)?,
            inductive: *globals
                .get(&(source_index, *inductive))
                .ok_or_else(|| foundation_error("constructor inductive is outside the closure"))?,
            generated: *generated,
        },
        DeclarationKind::Recursor {
            ty,
            inductive,
            generated,
        } => DeclarationKind::Recursor {
            ty: term(builder, *ty, levels, terms)?,
            inductive: *globals
                .get(&(source_index, *inductive))
                .ok_or_else(|| foundation_error("recursor inductive is outside the closure"))?,
            generated: *generated,
        },
    })
}

fn copy_term(
    builder: &mut CertificateBuilder,
    source_index: usize,
    source: &Certificate,
    globals: &BTreeMap<(usize, u32), u32>,
    levels: &mut BTreeMap<(usize, u32), u32>,
    terms: &mut BTreeMap<(usize, u32), u32>,
    id: u32,
) -> Result<u32, ProgramCertificateError> {
    if let Some(mapped) = terms.get(&(source_index, id)) {
        return Ok(*mapped);
    }
    let node = source
        .term_table
        .get(id as usize)
        .cloned()
        .ok_or_else(|| foundation_error("foundation term is missing"))?;
    let mapped = match node {
        TermNode::Sort(level) => {
            TermNode::Sort(copy_level(builder, source_index, source, levels, level)?)
        }
        TermNode::Var(index) => TermNode::Var(index),
        TermNode::Const {
            global,
            levels: source_levels,
        } => TermNode::Const {
            global: *globals
                .get(&(source_index, global))
                .ok_or_else(|| foundation_error("foundation term global is outside the closure"))?,
            levels: source_levels
                .into_iter()
                .map(|level| copy_level(builder, source_index, source, levels, level))
                .collect::<Result<_, _>>()?,
        },
        TermNode::App {
            function,
            arguments,
        } => TermNode::App {
            function: copy_term(
                builder,
                source_index,
                source,
                globals,
                levels,
                terms,
                function,
            )?,
            arguments: arguments
                .into_iter()
                .map(|argument| {
                    copy_term(
                        builder,
                        source_index,
                        source,
                        globals,
                        levels,
                        terms,
                        argument,
                    )
                })
                .collect::<Result<_, _>>()?,
        },
        TermNode::Lam { ty, body } => TermNode::Lam {
            ty: copy_term(builder, source_index, source, globals, levels, terms, ty)?,
            body: copy_term(builder, source_index, source, globals, levels, terms, body)?,
        },
        TermNode::Pi { ty, body } => TermNode::Pi {
            ty: copy_term(builder, source_index, source, globals, levels, terms, ty)?,
            body: copy_term(builder, source_index, source, globals, levels, terms, body)?,
        },
        TermNode::Let { ty, value, body } => TermNode::Let {
            ty: copy_term(builder, source_index, source, globals, levels, terms, ty)?,
            value: copy_term(builder, source_index, source, globals, levels, terms, value)?,
            body: copy_term(builder, source_index, source, globals, levels, terms, body)?,
        },
    };
    let mapped = builder.intern_term(mapped)?;
    terms.insert((source_index, id), mapped);
    Ok(mapped)
}

fn copy_level(
    builder: &mut CertificateBuilder,
    source_index: usize,
    source: &Certificate,
    levels: &mut BTreeMap<(usize, u32), u32>,
    id: u32,
) -> Result<u32, ProgramCertificateError> {
    if let Some(mapped) = levels.get(&(source_index, id)) {
        return Ok(*mapped);
    }
    let node = source
        .level_table
        .get(id as usize)
        .cloned()
        .ok_or_else(|| foundation_error("foundation level is missing"))?;
    let mapped = match node {
        LevelNode::Zero => LevelNode::Zero,
        LevelNode::Succ(inner) => {
            LevelNode::Succ(copy_level(builder, source_index, source, levels, inner)?)
        }
        LevelNode::Max(left, right) => LevelNode::Max(
            copy_level(builder, source_index, source, levels, left)?,
            copy_level(builder, source_index, source, levels, right)?,
        ),
        LevelNode::Param(name) => {
            let name = source
                .name_table
                .get(name as usize)
                .ok_or_else(|| foundation_error("foundation level parameter name is missing"))?;
            LevelNode::Param(builder.name_id(name)?)
        }
    };
    let mapped = builder.intern_level(mapped)?;
    levels.insert((source_index, id), mapped);
    Ok(mapped)
}

#[derive(Clone)]
struct ValueBinding {
    term: u32,
    ty: u32,
    depth: u32,
}

#[derive(Clone)]
struct Hypothesis {
    proposition: u32,
    proof: u32,
    depth: u32,
}

#[derive(Clone, Default)]
struct LowerContext {
    depth: u32,
    named: BTreeMap<String, ValueBinding>,
    anonymous: Vec<ValueBinding>,
    hypotheses: Vec<Hypothesis>,
}

impl LowerContext {
    fn push_named_value(
        &mut self,
        builder: &mut CertificateBuilder,
        name: &str,
        ty: u32,
    ) -> Result<(), ProgramCertificateError> {
        self.depth = self
            .depth
            .checked_add(1)
            .ok_or_else(|| interface_error("binder depth overflow"))?;
        let binding = ValueBinding {
            term: builder.var(0)?,
            ty: builder.shift(ty, 1, 0)?,
            depth: self.depth,
        };
        if self.named.insert(name.to_owned(), binding).is_some() {
            return Err(interface_error("duplicate named theorem binder"));
        }
        Ok(())
    }

    fn push_anonymous_value(
        &mut self,
        builder: &mut CertificateBuilder,
        ty: u32,
    ) -> Result<(), ProgramCertificateError> {
        self.depth = self
            .depth
            .checked_add(1)
            .ok_or_else(|| interface_error("binder depth overflow"))?;
        self.anonymous.push(ValueBinding {
            term: builder.var(0)?,
            ty: builder.shift(ty, 1, 0)?,
            depth: self.depth,
        });
        Ok(())
    }

    fn value(
        &self,
        builder: &mut CertificateBuilder,
        binding: &ValueBinding,
    ) -> Result<LoweredValue, ProgramCertificateError> {
        let shift = self
            .depth
            .checked_sub(binding.depth)
            .ok_or_else(|| interface_error("binding depth exceeds current context"))?;
        Ok(LoweredValue {
            term: builder.shift(binding.term, shift, 0)?,
            ty: builder.shift(binding.ty, shift, 0)?,
        })
    }

    fn named_value(
        &self,
        builder: &mut CertificateBuilder,
        name: &str,
    ) -> Result<LoweredValue, ProgramCertificateError> {
        let binding = self
            .named
            .get(name)
            .ok_or_else(|| interface_error(format!("unbound VC variable {name:?}")))?;
        self.value(builder, binding)
    }

    fn anonymous_value(
        &self,
        builder: &mut CertificateBuilder,
        index: u32,
    ) -> Result<LoweredValue, ProgramCertificateError> {
        let offset = usize::try_from(index).map_err(|_| interface_error("bound index overflow"))?;
        let position = self
            .anonymous
            .len()
            .checked_sub(offset + 1)
            .ok_or_else(|| interface_error("VC bound index is out of scope"))?;
        self.value(builder, &self.anonymous[position])
    }

    fn find_hypothesis(
        &self,
        builder: &mut CertificateBuilder,
        goal: u32,
    ) -> Result<Option<u32>, ProgramCertificateError> {
        for hypothesis in self.hypotheses.iter().rev() {
            let shift = self
                .depth
                .checked_sub(hypothesis.depth)
                .ok_or_else(|| interface_error("hypothesis depth exceeds current context"))?;
            if builder.shift(hypothesis.proposition, shift, 0)? == goal {
                return Ok(Some(builder.shift(hypothesis.proof, shift, 0)?));
            }
        }
        Ok(None)
    }
}

#[derive(Clone, Copy)]
struct LoweredValue {
    term: u32,
    ty: u32,
}

#[derive(Clone)]
enum PropTree {
    Leaf(u32),
    And {
        proposition: u32,
        left: Box<PropTree>,
        right: Box<PropTree>,
    },
}

impl PropTree {
    const fn proposition(&self) -> u32 {
        match self {
            Self::Leaf(proposition) | Self::And { proposition, .. } => *proposition,
        }
    }
}

fn lower_parameter_binders(
    builder: &mut CertificateBuilder,
    model: &ProgramModel<'_>,
    function: &VcFunction,
    group: &VcGroup,
    context: &mut LowerContext,
    index: usize,
) -> Result<u32, ProgramCertificateError> {
    if let Some(binder) = function.parameters.get(index) {
        let ty = lower_type(builder, &binder.r#type)?;
        let mut inner = context.clone();
        inner.push_named_value(builder, &binder.id, ty)?;
        let body = lower_parameter_binders(builder, model, function, group, &mut inner, index + 1)?;
        builder.pi(ty, body)
    } else {
        lower_group_body(builder, model, function, group, context)
    }
}

fn lower_group_body(
    builder: &mut CertificateBuilder,
    model: &ProgramModel<'_>,
    function: &VcFunction,
    group: &VcGroup,
    context: &LowerContext,
) -> Result<u32, ProgramCertificateError> {
    let requirements = lower_stored_conjunction(builder, model, &function.requires, context)?;
    let members = group
        .member_ids
        .iter()
        .map(|member_id| {
            let member = function
                .members
                .iter()
                .find(|member| &member.id == member_id)
                .ok_or_else(|| skeleton_error("group member is absent from its function"))?;
            lower_member_type(builder, model, member, context, 0)
        })
        .collect::<Result<Vec<_>, ProgramCertificateError>>()?;
    let members = lower_generated_conjunction(builder, members)?;
    lower_imp(builder, requirements.proposition(), members.proposition())
}

fn lower_member_type(
    builder: &mut CertificateBuilder,
    model: &ProgramModel<'_>,
    member: &VcMember,
    context: &LowerContext,
    index: usize,
) -> Result<u32, ProgramCertificateError> {
    if let Some(binder) = member.local_binders.get(index) {
        let ty = lower_type(builder, binder)?;
        let mut inner = context.clone();
        inner.push_anonymous_value(builder, ty)?;
        let body = lower_member_type(builder, model, member, &inner, index + 1)?;
        builder.pi(ty, body)
    } else {
        let assumptions = lower_stored_conjunction(builder, model, &member.assumptions, context)?;
        let conclusion = lower_prop(builder, model, &member.conclusion, context, None)?;
        lower_imp(builder, assumptions.proposition(), conclusion)
    }
}

fn lower_stored_conjunction(
    builder: &mut CertificateBuilder,
    model: &ProgramModel<'_>,
    terms: &[VcTerm],
    context: &LowerContext,
) -> Result<PropTree, ProgramCertificateError> {
    let propositions = terms
        .iter()
        .map(|term| lower_prop(builder, model, term, context, None))
        .collect::<Result<Vec<_>, _>>()?;
    lower_generated_conjunction(builder, propositions)
}

fn lower_generated_conjunction(
    builder: &mut CertificateBuilder,
    propositions: Vec<u32>,
) -> Result<PropTree, ProgramCertificateError> {
    fn recurse(
        builder: &mut CertificateBuilder,
        propositions: &[u32],
    ) -> Result<PropTree, ProgramCertificateError> {
        match propositions {
            [] => Ok(PropTree::Leaf(lower_generated_true(builder)?)),
            [proposition] => Ok(PropTree::Leaf(*proposition)),
            many => {
                let split = many.len() / 2;
                let left = recurse(builder, &many[..split])?;
                let right = recurse(builder, &many[split..])?;
                let and = builder.constant(STD_LOGIC_AND)?;
                let proposition = builder.app(and, [left.proposition(), right.proposition()])?;
                Ok(PropTree::And {
                    proposition,
                    left: Box::new(left),
                    right: Box::new(right),
                })
            }
        }
    }
    recurse(builder, &propositions)
}

fn lower_generated_true(builder: &mut CertificateBuilder) -> Result<u32, ProgramCertificateError> {
    let carrier = builder.constant(STD_BOOL)?;
    let value = builder.constant(STD_BOOL_TRUE)?;
    lower_eq(builder, carrier, value, value)
}

fn lower_imp(
    builder: &mut CertificateBuilder,
    antecedent: u32,
    consequent: u32,
) -> Result<u32, ProgramCertificateError> {
    let implication = builder.constant(STD_LOGIC_IMP)?;
    builder.app(implication, [antecedent, consequent])
}

fn lower_eq(
    builder: &mut CertificateBuilder,
    ty: u32,
    left: u32,
    right: u32,
) -> Result<u32, ProgramCertificateError> {
    let equality = builder.constant(STD_EQ)?;
    builder.app(equality, [ty, left, right])
}

fn lower_type(
    builder: &mut CertificateBuilder,
    ty: &VcTypeTerm,
) -> Result<u32, ProgramCertificateError> {
    match ty {
        VcTypeTerm::Constant { name } if supported_type_constant(name) => builder.constant(name),
        VcTypeTerm::Constant { name } => Err(interface_error(format!(
            "unregistered alpha VC type constant {name:?}"
        ))),
        VcTypeTerm::Apply { .. }
        | VcTypeTerm::NatLiteral { .. }
        | VcTypeTerm::StringLiteral { .. } => Err(interface_error(
            "alpha profile has no applied or literal VC type lowering",
        )),
    }
}

fn supported_type_constant(name: &str) -> bool {
    matches!(
        name,
        STD_BOOL
            | PROGRAM_BOOL
            | "Std.Program.Base.Int8"
            | "Std.Program.Base.Uint8"
            | "Std.Program.Base.Int16"
            | "Std.Program.Base.Uint16"
            | "Std.Program.Base.Int32"
            | "Std.Program.Base.Uint32"
            | "Std.Program.Base.Int64"
            | "Std.Program.Base.Uint64"
            | "Std.Program.Base.Array.Length"
            | "Std.Program.Base.Struct.Shape"
    )
}

fn lower_prop(
    builder: &mut CertificateBuilder,
    model: &ProgramModel<'_>,
    term: &VcTerm,
    context: &LowerContext,
    current_group: Option<&str>,
) -> Result<u32, ProgramCertificateError> {
    match term {
        VcTerm::Forall { binder_type, body } => {
            let ty = lower_type(builder, binder_type)?;
            let mut inner = context.clone();
            inner.push_anonymous_value(builder, ty)?;
            let body = lower_prop(builder, model, body, &inner, current_group)?;
            builder.pi(ty, body)
        }
        VcTerm::Apply { function, args } if function == STD_LOGIC_IMP && args.len() == 2 => {
            let antecedent = lower_prop(builder, model, &args[0], context, current_group)?;
            let consequent = lower_prop(builder, model, &args[1], context, current_group)?;
            lower_imp(builder, antecedent, consequent)
        }
        VcTerm::Apply { function, args } if function == STD_EQ && args.len() == 2 => {
            let left = lower_value(builder, model, &args[0], context)?;
            let right = lower_value(builder, model, &args[1], context)?;
            if left.ty != right.ty {
                return Err(interface_error(
                    "Std.Eq operands have different inferred types",
                ));
            }
            lower_eq(builder, left.ty, left.term, right.term)
        }
        VcTerm::Apply { function, args } if model.is_group_name(function) => {
            if let Some(current) = current_group {
                if !model.is_declared_dependency(function, current) {
                    return Err(interface_error(
                        "generated proposition is not an exact declared earlier dependency",
                    ));
                }
            }
            instantiate_group_prop(builder, model, function, args, context)
        }
        _ => {
            let value = lower_value(builder, model, term, context)?;
            let true_value = boolean_true(builder, value.ty)?;
            lower_eq(builder, value.ty, value.term, true_value)
        }
    }
}

fn instantiate_group_prop(
    builder: &mut CertificateBuilder,
    model: &ProgramModel<'_>,
    name: &str,
    args: &[VcTerm],
    context: &LowerContext,
) -> Result<u32, ProgramCertificateError> {
    let (function, group) = model.group(name)?;
    if args.len() != function.parameters.len() {
        return Err(interface_error(
            "generated proposition argument count differs from its declaration binders",
        ));
    }
    let mut substitution = LowerContext {
        depth: context.depth,
        ..LowerContext::default()
    };
    for (binder, argument) in function.parameters.iter().zip(args) {
        let argument = lower_value(builder, model, argument, context)?;
        let expected = lower_type(builder, &binder.r#type)?;
        if argument.ty != expected {
            return Err(interface_error(
                "generated proposition argument type differs from its declaration binder",
            ));
        }
        substitution.named.insert(
            binder.id.clone(),
            ValueBinding {
                term: argument.term,
                ty: argument.ty,
                depth: context.depth,
            },
        );
    }
    lower_group_body(builder, model, function, group, &substitution)
}

fn lower_value(
    builder: &mut CertificateBuilder,
    model: &ProgramModel<'_>,
    term: &VcTerm,
    context: &LowerContext,
) -> Result<LoweredValue, ProgramCertificateError> {
    match term {
        VcTerm::Var { name } => context.named_value(builder, name),
        VcTerm::Bound { index } => context.anonymous_value(builder, *index),
        VcTerm::Constant { name } => {
            let ty_name = match name.as_str() {
                STD_BOOL_TRUE | STD_BOOL_FALSE => STD_BOOL,
                PROGRAM_BOOL_TRUE | PROGRAM_BOOL_FALSE => PROGRAM_BOOL,
                _ => {
                    return Err(interface_error(format!(
                        "unregistered alpha value constant {name:?}"
                    )))
                }
            };
            Ok(LoweredValue {
                term: builder.constant(name)?,
                ty: builder.constant(ty_name)?,
            })
        }
        VcTerm::Apply { function, args }
            if matches!(
                function.as_str(),
                STD_BOOL_NOT | STD_BOOL_AND_VALUE | STD_BOOL_OR | STD_BOOL_IF
            ) =>
        {
            let expected_arity = if function == STD_BOOL_NOT {
                1
            } else if function == STD_BOOL_IF {
                3
            } else {
                2
            };
            if args.len() != expected_arity {
                return Err(interface_error("Std.Bool operation has the wrong arity"));
            }
            let carrier = builder.constant(STD_BOOL)?;
            let arguments = args
                .iter()
                .map(|argument| lower_value(builder, model, argument, context))
                .collect::<Result<Vec<_>, _>>()?;
            if arguments.iter().any(|argument| argument.ty != carrier) {
                return Err(interface_error(
                    "Std.Bool operation received a non-Std.Bool carrier",
                ));
            }
            let function = builder.constant(function)?;
            Ok(LoweredValue {
                term: builder.app(function, arguments.iter().map(|argument| argument.term))?,
                ty: carrier,
            })
        }
        VcTerm::Apply { function, .. } if model.is_group_name(function) => Err(interface_error(
            "a generated theorem marker cannot be used as a value",
        )),
        VcTerm::Apply { function, .. } => Err(interface_error(format!(
            "unregistered alpha value function {function:?}"
        ))),
        VcTerm::BitVecLiteral { .. } | VcTerm::Convert { .. } => Err(interface_error(
            "alpha profile has no bitvector-literal or conversion value lowering",
        )),
        VcTerm::Forall { .. } => Err(interface_error("a forall term cannot be used as a value")),
    }
}

fn boolean_true(
    builder: &mut CertificateBuilder,
    carrier: u32,
) -> Result<u32, ProgramCertificateError> {
    let std_bool = builder.constant(STD_BOOL)?;
    if carrier == std_bool {
        return builder.constant(STD_BOOL_TRUE);
    }
    if builder.globals.contains_key(PROGRAM_BOOL) {
        let program_bool = builder.constant(PROGRAM_BOOL)?;
        if carrier == program_bool {
            return builder.constant(PROGRAM_BOOL_TRUE);
        }
    }
    Err(interface_error(
        "VC proposition value does not have a checked Boolean carrier",
    ))
}

fn prove_parameter_binders(
    builder: &mut CertificateBuilder,
    model: &ProgramModel<'_>,
    function: &VcFunction,
    group: &VcGroup,
    context: &mut LowerContext,
    index: usize,
    missing: &mut BTreeSet<String>,
) -> Result<Option<u32>, ProgramCertificateError> {
    if let Some(binder) = function.parameters.get(index) {
        let ty = lower_type(builder, &binder.r#type)?;
        let mut inner = context.clone();
        inner.push_named_value(builder, &binder.id, ty)?;
        let Some(body) = prove_parameter_binders(
            builder,
            model,
            function,
            group,
            &mut inner,
            index + 1,
            missing,
        )?
        else {
            return Ok(None);
        };
        Ok(Some(builder.lam(ty, body)?))
    } else {
        prove_group_body(builder, model, function, group, context, missing)
    }
}

fn prove_group_body(
    builder: &mut CertificateBuilder,
    model: &ProgramModel<'_>,
    function: &VcFunction,
    group: &VcGroup,
    context: &LowerContext,
    missing: &mut BTreeSet<String>,
) -> Result<Option<u32>, ProgramCertificateError> {
    let requirements = lower_stored_conjunction(builder, model, &function.requires, context)?;
    let mut inner = push_proof_hypothesis(builder, context, &requirements)?;
    let members = group
        .member_ids
        .iter()
        .map(|member_id| {
            function
                .members
                .iter()
                .find(|member| &member.id == member_id)
                .ok_or_else(|| skeleton_error("group member is absent from its function"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let body = prove_member_conjunction(builder, model, &members, &mut inner, group, missing)?;
    body.map(|body| builder.lam(requirements.proposition(), body))
        .transpose()
}

fn prove_member_conjunction(
    builder: &mut CertificateBuilder,
    model: &ProgramModel<'_>,
    members: &[&VcMember],
    context: &mut LowerContext,
    group: &VcGroup,
    missing: &mut BTreeSet<String>,
) -> Result<Option<u32>, ProgramCertificateError> {
    match members {
        [] => Ok(Some(prove_generated_true(builder)?)),
        [member] => prove_member(builder, model, member, context, group, 0, missing),
        many => {
            let split = many.len() / 2;
            let left_type = lower_member_conjunction_type(builder, model, &many[..split], context)?;
            let right_type =
                lower_member_conjunction_type(builder, model, &many[split..], context)?;
            let left =
                prove_member_conjunction(builder, model, &many[..split], context, group, missing)?;
            let right =
                prove_member_conjunction(builder, model, &many[split..], context, group, missing)?;
            let (Some(left), Some(right)) = (left, right) else {
                return Ok(None);
            };
            let intro = builder.constant(STD_LOGIC_AND_INTRO)?;
            Ok(Some(
                builder.app(intro, [left_type, right_type, left, right])?,
            ))
        }
    }
}

fn lower_member_conjunction_type(
    builder: &mut CertificateBuilder,
    model: &ProgramModel<'_>,
    members: &[&VcMember],
    context: &LowerContext,
) -> Result<u32, ProgramCertificateError> {
    let propositions = members
        .iter()
        .map(|member| lower_member_type(builder, model, member, context, 0))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(lower_generated_conjunction(builder, propositions)?.proposition())
}

fn prove_member(
    builder: &mut CertificateBuilder,
    model: &ProgramModel<'_>,
    member: &VcMember,
    context: &LowerContext,
    group: &VcGroup,
    index: usize,
    missing: &mut BTreeSet<String>,
) -> Result<Option<u32>, ProgramCertificateError> {
    if let Some(binder) = member.local_binders.get(index) {
        let ty = lower_type(builder, binder)?;
        let mut inner = context.clone();
        inner.push_anonymous_value(builder, ty)?;
        let Some(body) = prove_member(builder, model, member, &inner, group, index + 1, missing)?
        else {
            return Ok(None);
        };
        return Ok(Some(builder.lam(ty, body)?));
    }
    let assumptions = lower_stored_conjunction(builder, model, &member.assumptions, context)?;
    let mut inner = push_proof_hypothesis(builder, context, &assumptions)?;
    let proof = prove_prop(
        builder,
        model,
        &member.conclusion,
        &mut inner,
        &group.declaration_name,
    )?;
    let Some(proof) = proof else {
        missing.insert(member.id.clone());
        return Ok(None);
    };
    Ok(Some(builder.lam(assumptions.proposition(), proof)?))
}

fn prove_prop(
    builder: &mut CertificateBuilder,
    model: &ProgramModel<'_>,
    term: &VcTerm,
    context: &mut LowerContext,
    current_group: &str,
) -> Result<Option<u32>, ProgramCertificateError> {
    let goal = lower_prop(builder, model, term, context, Some(current_group))?;
    if let Some(proof) = context.find_hypothesis(builder, goal)? {
        return Ok(Some(proof));
    }
    match term {
        VcTerm::Forall { binder_type, body } => {
            let ty = lower_type(builder, binder_type)?;
            let mut inner = context.clone();
            inner.push_anonymous_value(builder, ty)?;
            let Some(body) = prove_prop(builder, model, body, &mut inner, current_group)? else {
                return Ok(None);
            };
            Ok(Some(builder.lam(ty, body)?))
        }
        VcTerm::Apply { function, args } if function == STD_LOGIC_IMP && args.len() == 2 => {
            let antecedent = lower_prop(builder, model, &args[0], context, Some(current_group))?;
            let tree = PropTree::Leaf(antecedent);
            let mut inner = push_proof_hypothesis(builder, context, &tree)?;
            let Some(body) = prove_prop(builder, model, &args[1], &mut inner, current_group)?
            else {
                return Ok(None);
            };
            Ok(Some(builder.lam(antecedent, body)?))
        }
        VcTerm::Apply { function, args } if function == STD_EQ && args.len() == 2 => {
            let left = lower_value(builder, model, &args[0], context)?;
            let right = lower_value(builder, model, &args[1], context)?;
            let std_bool = builder.constant(STD_BOOL)?;
            let definitionally_equal = left.term == right.term
                || (left.ty == std_bool
                    && normalize_std_bool_value(&args[0]) == normalize_std_bool_value(&args[1]));
            if left.ty == right.ty && definitionally_equal {
                let refl = builder.constant(STD_EQ_REFL)?;
                Ok(Some(builder.app(refl, [left.ty, left.term])?))
            } else {
                Ok(None)
            }
        }
        VcTerm::Apply { function, args } if model.is_group_name(function) => {
            if !model.is_declared_dependency(function, current_group) {
                return Err(interface_error(
                    "proof attempted to use a generated declaration outside the exact dependency set",
                ));
            }
            let (target, _) = model.group(function)?;
            if args.len() != target.parameters.len() {
                return Err(interface_error(
                    "generated proof application arity mismatch",
                ));
            }
            let mut lowered = Vec::with_capacity(args.len());
            for (argument, binder) in args.iter().zip(&target.parameters) {
                let argument = lower_value(builder, model, argument, context)?;
                if argument.ty != lower_type(builder, &binder.r#type)? {
                    return Err(interface_error("generated proof argument type mismatch"));
                }
                lowered.push(argument.term);
            }
            let theorem = builder.constant(function)?;
            Ok(Some(builder.app(theorem, lowered)?))
        }
        _ => {
            let value = lower_value(builder, model, term, context)?;
            let true_value = boolean_true(builder, value.ty)?;
            let std_bool = builder.constant(STD_BOOL)?;
            let definitionally_true = value.term == true_value
                || (value.ty == std_bool
                    && normalize_std_bool_value(term)
                        == VcTerm::Constant {
                            name: STD_BOOL_TRUE.to_owned(),
                        });
            if definitionally_true {
                let refl = builder.constant(STD_EQ_REFL)?;
                Ok(Some(builder.app(refl, [value.ty, value.term])?))
            } else {
                Ok(None)
            }
        }
    }
}

fn normalize_std_bool_value(term: &VcTerm) -> VcTerm {
    let VcTerm::Apply { function, args } = term else {
        return term.clone();
    };
    let normalized = args
        .iter()
        .map(normalize_std_bool_value)
        .collect::<Vec<_>>();
    let bool_constant = |name: &str| VcTerm::Constant {
        name: name.to_owned(),
    };
    let is_constant = |term: &VcTerm, expected: &str| matches!(term, VcTerm::Constant { name } if name == expected);
    match (function.as_str(), normalized.as_slice()) {
        (STD_BOOL_NOT, [value]) if is_constant(value, STD_BOOL_TRUE) => {
            bool_constant(STD_BOOL_FALSE)
        }
        (STD_BOOL_NOT, [value]) if is_constant(value, STD_BOOL_FALSE) => {
            bool_constant(STD_BOOL_TRUE)
        }
        (STD_BOOL_AND_VALUE, [left, _]) if is_constant(left, STD_BOOL_FALSE) => {
            bool_constant(STD_BOOL_FALSE)
        }
        (STD_BOOL_AND_VALUE, [left, right]) if is_constant(left, STD_BOOL_TRUE) => right.clone(),
        (STD_BOOL_OR, [left, right]) if is_constant(left, STD_BOOL_FALSE) => right.clone(),
        (STD_BOOL_OR, [left, _]) if is_constant(left, STD_BOOL_TRUE) => {
            bool_constant(STD_BOOL_TRUE)
        }
        (STD_BOOL_IF, [condition, then_case, _]) if is_constant(condition, STD_BOOL_TRUE) => {
            then_case.clone()
        }
        (STD_BOOL_IF, [condition, _, else_case]) if is_constant(condition, STD_BOOL_FALSE) => {
            else_case.clone()
        }
        _ => VcTerm::Apply {
            function: function.clone(),
            args: normalized,
        },
    }
}

fn prove_generated_true(builder: &mut CertificateBuilder) -> Result<u32, ProgramCertificateError> {
    let carrier = builder.constant(STD_BOOL)?;
    let value = builder.constant(STD_BOOL_TRUE)?;
    let refl = builder.constant(STD_EQ_REFL)?;
    builder.app(refl, [carrier, value])
}

fn push_proof_hypothesis(
    builder: &mut CertificateBuilder,
    context: &LowerContext,
    tree: &PropTree,
) -> Result<LowerContext, ProgramCertificateError> {
    let mut inner = context.clone();
    inner.depth = inner
        .depth
        .checked_add(1)
        .ok_or_else(|| interface_error("proof binder depth overflow"))?;
    let shifted = shift_tree(builder, tree, 1)?;
    let proof = builder.var(0)?;
    collect_hypotheses(builder, &shifted, proof, inner.depth, &mut inner.hypotheses)?;
    Ok(inner)
}

fn shift_tree(
    builder: &mut CertificateBuilder,
    tree: &PropTree,
    amount: u32,
) -> Result<PropTree, ProgramCertificateError> {
    Ok(match tree {
        PropTree::Leaf(proposition) => PropTree::Leaf(builder.shift(*proposition, amount, 0)?),
        PropTree::And {
            proposition,
            left,
            right,
        } => PropTree::And {
            proposition: builder.shift(*proposition, amount, 0)?,
            left: Box::new(shift_tree(builder, left, amount)?),
            right: Box::new(shift_tree(builder, right, amount)?),
        },
    })
}

fn collect_hypotheses(
    builder: &mut CertificateBuilder,
    tree: &PropTree,
    proof: u32,
    depth: u32,
    hypotheses: &mut Vec<Hypothesis>,
) -> Result<(), ProgramCertificateError> {
    hypotheses.push(Hypothesis {
        proposition: tree.proposition(),
        proof,
        depth,
    });
    let PropTree::And { left, right, .. } = tree else {
        return Ok(());
    };
    let left_proof = project_and(
        builder,
        left.proposition(),
        right.proposition(),
        proof,
        true,
    )?;
    collect_hypotheses(builder, left, left_proof, depth, hypotheses)?;
    let right_proof = project_and(
        builder,
        left.proposition(),
        right.proposition(),
        proof,
        false,
    )?;
    collect_hypotheses(builder, right, right_proof, depth, hypotheses)
}

fn project_and(
    builder: &mut CertificateBuilder,
    left: u32,
    right: u32,
    proof: u32,
    project_left: bool,
) -> Result<u32, ProgramCertificateError> {
    let recursor = builder.constant(STD_LOGIC_AND_REC)?;
    let shifted_right = builder.shift(right, 1, 0)?;
    let body = builder.var(if project_left { 1 } else { 0 })?;
    let inner = builder.lam(shifted_right, body)?;
    let minor = builder.lam(left, inner)?;
    let target = if project_left { left } else { right };
    builder.app(recursor, [left, right, target, minor, proof])
}

fn run_reference_checker(bytes: &[u8]) -> Result<ReferenceCheckerReport, ProgramCertificateError> {
    let output = execute_reference_checker(bytes)
        .map_err(|error| checker_execution(format!("launch reference checker: {error}")))?;
    parse_reference_checker_output(output.status_code(), output.stdout(), output.stderr())
}

fn parse_reference_checker_output(
    status_code: Option<i32>,
    stdout: &[u8],
    stderr: &[u8],
) -> Result<ReferenceCheckerReport, ProgramCertificateError> {
    let parsed: ReferenceCheckerOutput = serde_json::from_slice(stdout).map_err(|error| {
        checker_execution(format!(
            "reference checker emitted invalid JSON: {error}; stderr={}",
            compact_output(stderr)
        ))
    })?;
    match parsed {
        ReferenceCheckerOutput::Accepted {
            module,
            declaration_count,
            axiom_count,
            hashes,
        } => {
            if status_code != Some(0) {
                return Err(checker_execution(format!(
                    "reference checker acceptance used an invalid exit status: {status_code:?}"
                )));
            }
            if !reference_hash_scalar(&hashes.export)
                || !reference_hash_scalar(&hashes.axiom_report)
                || !reference_hash_scalar(&hashes.certificate)
            {
                return Err(checker_execution(
                    "reference checker accepted report contains a malformed hash",
                ));
            }
            Ok(ReferenceCheckerReport {
                module,
                declaration_count,
                axiom_count,
                export_hash: hashes.export,
                axiom_report_hash: hashes.axiom_report,
                certificate_hash: hashes.certificate,
            })
        }
        ReferenceCheckerOutput::Rejected {
            error_kind,
            error_detail,
        } => {
            if status_code != Some(1) {
                return Err(checker_execution(format!(
                    "reference checker rejection used an invalid exit status: {status_code:?}"
                )));
            }
            let detail = format!(
                "reference checker rejected candidate: kind={error_kind:?} detail={error_detail:?} stderr={}",
                compact_output(stderr)
            );
            if reference_candidate_rejection(&error_kind) {
                Err(ProgramCertificateError::new(
                    ProgramCertificateErrorKind::CheckerRejected,
                    detail,
                ))
            } else {
                Err(checker_execution(detail))
            }
        }
    }
}

fn reference_candidate_rejection(kind: &str) -> bool {
    matches!(
        kind,
        "canonical_certificate"
            | "unsupported_feature"
            | "export_block_mismatch"
            | "axiom_report_mismatch"
            | "hash_mismatch"
            | "missing_name"
            | "missing_global"
            | "out_of_order_declaration_dependency"
            | "core_check"
    )
}

fn reference_hash_scalar(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}

fn validate_reference_checker_report(
    bytes: &[u8],
    report: &ReferenceCheckerReport,
) -> Result<(), ProgramCertificateError> {
    let certificate = decode_canonical_certificate(bytes).map_err(|error| {
        checker_execution(format!(
            "accepted reference report names noncanonical candidate bytes: {error:?}"
        ))
    })?;
    let export_hash = export_block_hash(&certificate.export_block);
    let axiom_report = build_axiom_report(&certificate).map_err(|error| {
        checker_execution(format!(
            "recompute accepted reference axiom report: {}",
            error.detail()
        ))
    })?;
    let axiom_report_hash = axiom_report_hash_for_report(&axiom_report);
    if certificate.export_block
        != build_export_block(&certificate).map_err(|error| {
            checker_execution(format!("recompute reference export: {}", error.detail()))
        })?
        || certificate.axiom_report != axiom_report
        || certificate.hashes.export_hash != export_hash
        || certificate.hashes.axiom_report_hash != axiom_report_hash
        || report.module != certificate.module
        || report.declaration_count != certificate.declarations.len()
        || report.axiom_count != 0
        || report.axiom_count != axiom_report.summary.total_axiom_count
        || report.export_hash != hash_hex(&export_hash)
        || report.axiom_report_hash != hash_hex(&axiom_report_hash)
        || report.certificate_hash != hash_hex(&certificate_hash(bytes))
    {
        return Err(checker_execution(
            "accepted reference report is not bound to the submitted zero-axiom candidate bytes",
        ));
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum ReconciledCheckerResults {
    Accepted {
        rust_report: VerificationReport,
        reference_report: ReferenceCheckerReport,
    },
    Unaccepted {
        rust_report: Option<VerificationReport>,
        reference_report: Option<ReferenceCheckerReport>,
        rust_verdict: ProgramCheckerVerdict,
        reference_verdict: ProgramCheckerVerdict,
        failure: ProgramCertificateError,
    },
}

fn reconcile_checker_results(
    bytes: &[u8],
    rust: Result<VerificationReport, ProgramCertificateError>,
    reference: Result<ReferenceCheckerReport, ProgramCertificateError>,
) -> Result<ReconciledCheckerResults, ProgramCertificateError> {
    match (rust, reference) {
        (Ok(rust), Ok(reference)) => {
            // Policy evidence v1 has one shared report-hash triple and no
            // per-checker report fields. Two accepted but different reports
            // therefore cannot be retained faithfully as evidence.
            require_checker_agreement(bytes, &rust, &reference)?;
            Ok(ReconciledCheckerResults::Accepted {
                rust_report: rust,
                reference_report: reference,
            })
        }
        (Err(rust), Err(reference))
            if rust.kind() == ProgramCertificateErrorKind::CheckerRejected
                && reference.kind() == ProgramCertificateErrorKind::CheckerRejected =>
        {
            Ok(ReconciledCheckerResults::Unaccepted {
                rust_report: None,
                reference_report: None,
                rust_verdict: ProgramCheckerVerdict::Rejected,
                reference_verdict: ProgramCheckerVerdict::Rejected,
                failure: ProgramCertificateError::new(
                    ProgramCertificateErrorKind::CheckerRejected,
                    format!(
                        "both source-free checkers rejected the candidate: {rust}; {reference}"
                    ),
                ),
            })
        }
        (Err(rust), Ok(reference))
            if rust.kind() == ProgramCertificateErrorKind::CheckerRejected =>
        {
            validate_reference_checker_report(bytes, &reference)?;
            Ok(ReconciledCheckerResults::Unaccepted {
                rust_report: None,
                reference_report: Some(reference),
                rust_verdict: ProgramCheckerVerdict::Rejected,
                reference_verdict: ProgramCheckerVerdict::Accepted,
                failure: ProgramCertificateError::new(
                    ProgramCertificateErrorKind::CheckerDisagreement,
                    format!("checker acceptance disagreement: {rust}; reference checker accepted"),
                ),
            })
        }
        (Ok(rust), Err(reference))
            if reference.kind() == ProgramCertificateErrorKind::CheckerRejected =>
        {
            Ok(ReconciledCheckerResults::Unaccepted {
                rust_report: Some(rust),
                reference_report: None,
                rust_verdict: ProgramCheckerVerdict::Accepted,
                reference_verdict: ProgramCheckerVerdict::Rejected,
                failure: ProgramCertificateError::new(
                    ProgramCertificateErrorKind::CheckerDisagreement,
                    format!(
                        "checker acceptance disagreement: Rust fast kernel accepted; {reference}"
                    ),
                ),
            })
        }
        (Err(rust), Ok(_)) => Err(rust),
        (Ok(_), Err(reference)) => Err(reference),
        (Err(rust), Err(reference)) => {
            if rust.kind() != ProgramCertificateErrorKind::CheckerRejected {
                Err(rust)
            } else {
                Err(reference)
            }
        }
    }
}

fn require_checker_agreement(
    bytes: &[u8],
    rust: &VerificationReport,
    reference: &ReferenceCheckerReport,
) -> Result<(), ProgramCertificateError> {
    let expected_certificate_hash = hash_hex(&certificate_hash(bytes));
    if rust.module != reference.module
        || rust.declaration_count != reference.declaration_count
        || rust.axiom_count != reference.axiom_count
        || hash_hex(&rust.export_hash) != reference.export_hash
        || hash_hex(&rust.axiom_report_hash) != reference.axiom_report_hash
        || hash_hex(&rust.certificate_hash) != reference.certificate_hash
        || reference.certificate_hash != expected_certificate_hash
        || rust.axiom_count != 0
        || !rust.axiom_report.entries.is_empty()
        || !rust.axiom_report.declaration_dependencies.is_empty()
    {
        return Err(ProgramCertificateError::new(
            ProgramCertificateErrorKind::CheckerDisagreement,
            "Rust and Go checker reports differ or are not the empty-axiom alpha report",
        ));
    }
    Ok(())
}

#[derive(Deserialize)]
#[serde(tag = "verdict", deny_unknown_fields)]
enum ReferenceCheckerOutput {
    #[serde(rename = "accepted")]
    Accepted {
        module: String,
        #[serde(default)]
        declaration_count: usize,
        #[serde(default)]
        axiom_count: u64,
        hashes: ReferenceHashes,
    },
    #[serde(rename = "rejected")]
    Rejected {
        error_kind: String,
        error_detail: String,
    },
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ReferenceHashes {
    export: String,
    axiom_report: String,
    certificate: String,
}

fn compact_output(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).trim().to_owned()
}

fn foundation_error(detail: impl Into<String>) -> ProgramCertificateError {
    ProgramCertificateError::new(ProgramCertificateErrorKind::Foundation, detail)
}

fn foundation_reference_error(
    module: &str,
    error: ProgramCertificateError,
) -> ProgramCertificateError {
    ProgramCertificateError::new(
        if error.kind() == ProgramCertificateErrorKind::CheckerExecution {
            ProgramCertificateErrorKind::CheckerExecution
        } else {
            ProgramCertificateErrorKind::Foundation
        },
        format!(
            "reference-check registered foundation {module:?}: {}",
            error.detail()
        ),
    )
}

fn skeleton_error(detail: impl Into<String>) -> ProgramCertificateError {
    ProgramCertificateError::new(ProgramCertificateErrorKind::Skeleton, detail)
}

fn interface_error(detail: impl Into<String>) -> ProgramCertificateError {
    ProgramCertificateError::new(ProgramCertificateErrorKind::Interface, detail)
}

fn checker_execution(detail: impl Into<String>) -> ProgramCertificateError {
    ProgramCertificateError::new(ProgramCertificateErrorKind::CheckerExecution, detail)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn alpha_certificate() -> (Certificate, usize) {
        let bytes = decode_hex(include_bytes!(
            "../../../fixtures/program-certificate/alpha-module-calls.hex"
        ))
        .expect("program-certificate fixture hex");
        let certificate = decode_canonical_certificate(&bytes).expect("canonical alpha fixture");
        let generated_start = certificate.declarations.len() - 6;
        (certificate, generated_start)
    }

    fn rust_acceptance() -> VerificationReport {
        VerificationReport {
            module: PROGRAM_CERTIFICATE_MODULE.to_owned(),
            declaration_count: 0,
            axiom_count: 0,
            export_hash: ZERO_HASH,
            axiom_report_hash: ZERO_HASH,
            certificate_hash: ZERO_HASH,
            axiom_report: AxiomReport::default(),
        }
    }

    fn reference_acceptance() -> ReferenceCheckerReport {
        ReferenceCheckerReport {
            module: PROGRAM_CERTIFICATE_MODULE.to_owned(),
            declaration_count: 0,
            axiom_count: 0,
            export_hash: hash_hex(&ZERO_HASH),
            axiom_report_hash: hash_hex(&ZERO_HASH),
            certificate_hash: hash_hex(&ZERO_HASH),
        }
    }

    #[test]
    fn checker_acceptance_pairs_have_distinct_fail_closed_classifications() {
        let candidate_bytes = decode_hex(include_bytes!(
            "../../../fixtures/program-certificate/alpha-module-calls.hex"
        ))
        .expect("program-certificate fixture hex");
        let candidate_report =
            verify_certificate_bytes(&candidate_bytes).expect("fixture passes Rust checker");
        let bound_reference = ReferenceCheckerReport {
            module: candidate_report.module.clone(),
            declaration_count: candidate_report.declaration_count,
            axiom_count: candidate_report.axiom_count,
            export_hash: hash_hex(&candidate_report.export_hash),
            axiom_report_hash: hash_hex(&candidate_report.axiom_report_hash),
            certificate_hash: hash_hex(&candidate_report.certificate_hash),
        };
        let reference_rejection = || {
            ProgramCertificateError::new(
                ProgramCertificateErrorKind::CheckerRejected,
                "reference rejected",
            )
        };
        let rust_rejection = || {
            ProgramCertificateError::new(
                ProgramCertificateErrorKind::CheckerRejected,
                "Rust rejected",
            )
        };

        let outcome = reconcile_checker_results(
            b"candidate",
            Ok(rust_acceptance()),
            Err(reference_rejection()),
        )
        .unwrap();
        let ReconciledCheckerResults::Unaccepted {
            rust_verdict,
            reference_verdict,
            failure,
            ..
        } = outcome
        else {
            panic!("accept/reject must remain an unaccepted candidate");
        };
        assert_eq!(rust_verdict, ProgramCheckerVerdict::Accepted);
        assert_eq!(reference_verdict, ProgramCheckerVerdict::Rejected);
        assert_eq!(
            failure.kind(),
            ProgramCertificateErrorKind::CheckerDisagreement
        );

        let outcome =
            reconcile_checker_results(&candidate_bytes, Err(rust_rejection()), Ok(bound_reference))
                .unwrap();
        let ReconciledCheckerResults::Unaccepted {
            rust_verdict,
            reference_verdict,
            failure,
            ..
        } = outcome
        else {
            panic!("reject/accept must remain an unaccepted candidate");
        };
        assert_eq!(rust_verdict, ProgramCheckerVerdict::Rejected);
        assert_eq!(reference_verdict, ProgramCheckerVerdict::Accepted);
        assert_eq!(
            failure.kind(),
            ProgramCertificateErrorKind::CheckerDisagreement
        );

        let outcome = reconcile_checker_results(
            b"candidate",
            Err(rust_rejection()),
            Err(reference_rejection()),
        )
        .unwrap();
        let ReconciledCheckerResults::Unaccepted {
            rust_verdict,
            reference_verdict,
            failure,
            ..
        } = outcome
        else {
            panic!("dual rejection must remain an unaccepted candidate");
        };
        assert_eq!(rust_verdict, ProgramCheckerVerdict::Rejected);
        assert_eq!(reference_verdict, ProgramCheckerVerdict::Rejected);
        assert_eq!(failure.kind(), ProgramCertificateErrorKind::CheckerRejected);

        let error = reconcile_checker_results(
            b"candidate",
            Ok(rust_acceptance()),
            Ok(reference_acceptance()),
        )
        .unwrap_err();
        assert_eq!(
            error.kind(),
            ProgramCertificateErrorKind::CheckerDisagreement,
            "policy v1 cannot faithfully retain two accepted but different reports"
        );

        let error = reconcile_checker_results(
            b"candidate",
            Err(checker_execution("Rust internal invariant")),
            Ok(reference_acceptance()),
        )
        .unwrap_err();
        assert_eq!(error.kind(), ProgramCertificateErrorKind::CheckerExecution);

        let error = reconcile_checker_results(
            b"candidate",
            Err(rust_rejection()),
            Err(checker_execution("reference internal invariant")),
        )
        .unwrap_err();
        assert_eq!(error.kind(), ProgramCertificateErrorKind::CheckerExecution);
        assert!(reference_candidate_rejection("core_check"));
        assert!(!reference_candidate_rejection("internal_invariant"));
        assert!(!reference_candidate_rejection("future_unknown_kind"));

        assert_eq!(
            foundation_reference_error(
                "Std.Bool",
                checker_execution("reference internal invariant"),
            )
            .kind(),
            ProgramCertificateErrorKind::CheckerExecution
        );
        assert_eq!(
            foundation_reference_error("Std.Bool", reference_rejection()).kind(),
            ProgramCertificateErrorKind::Foundation
        );
    }

    #[test]
    fn reference_checker_protocol_is_verdict_discriminated_and_zero_count_compatible() {
        let zero_hash = "00".repeat(32);
        let accepted = format!(
            r#"{{"verdict":"accepted","module":"Policy.Generated","hashes":{{"export":"{zero_hash}","axiom_report":"{zero_hash}","certificate":"{zero_hash}"}}}}"#
        );
        let report = parse_reference_checker_output(Some(0), accepted.as_bytes(), b"")
            .expect("omitempty zero counts are valid");
        assert_eq!(report.declaration_count, 0);
        assert_eq!(report.axiom_count, 0);

        let rejected =
            br#"{"verdict":"rejected","error_kind":"core_check","error_detail":"bad proof"}"#;
        assert_eq!(
            parse_reference_checker_output(Some(1), rejected, b"")
                .unwrap_err()
                .kind(),
            ProgramCertificateErrorKind::CheckerRejected
        );

        let invalid = [
            format!(
                r#"{{"verdict":"accepted","module":"Policy.Generated","error_kind":"internal_invariant","error_detail":"bug","hashes":{{"export":"{zero_hash}","axiom_report":"{zero_hash}","certificate":"{zero_hash}"}}}}"#
            ),
            format!(
                r#"{{"verdict":"rejected","error_kind":"core_check","error_detail":"bad proof","module":"Policy.Generated","hashes":{{"export":"{zero_hash}","axiom_report":"{zero_hash}","certificate":"{zero_hash}"}}}}"#
            ),
            r#"{"verdict":"accepted","module":"Policy.Generated"}"#.to_owned(),
            r#"{"verdict":"future","module":"Policy.Generated"}"#.to_owned(),
            format!(
                r#"{{"verdict":"accepted","module":"Policy.Generated","hashes":{{"export":"AA{zero_hash}","axiom_report":"{zero_hash}","certificate":"{zero_hash}"}}}}"#
            ),
        ];
        for output in invalid {
            assert_eq!(
                parse_reference_checker_output(Some(0), output.as_bytes(), b"")
                    .unwrap_err()
                    .kind(),
                ProgramCertificateErrorKind::CheckerExecution,
                "invalid protocol output: {output}"
            );
        }
        assert_eq!(
            parse_reference_checker_output(Some(1), accepted.as_bytes(), b"")
                .unwrap_err()
                .kind(),
            ProgramCertificateErrorKind::CheckerExecution
        );
        assert_eq!(
            parse_reference_checker_output(Some(0), rejected, b"")
                .unwrap_err()
                .kind(),
            ProgramCertificateErrorKind::CheckerExecution
        );
    }

    #[test]
    fn accepted_reference_report_is_bound_to_the_submitted_candidate_bytes() {
        let bytes = decode_hex(include_bytes!(
            "../../../fixtures/program-certificate/alpha-module-calls.hex"
        ))
        .expect("program-certificate fixture hex");
        let rust = verify_certificate_bytes(&bytes).expect("fixture passes the Rust checker");
        let reference = ReferenceCheckerReport {
            module: rust.module.clone(),
            declaration_count: rust.declaration_count,
            axiom_count: rust.axiom_count,
            export_hash: hash_hex(&rust.export_hash),
            axiom_report_hash: hash_hex(&rust.axiom_report_hash),
            certificate_hash: hash_hex(&rust.certificate_hash),
        };
        validate_reference_checker_report(&bytes, &reference)
            .expect("exact report is bound to the fixture");

        let mut wrong = reference;
        wrong.certificate_hash = hash_hex(&ZERO_HASH);
        let reference_error = validate_reference_checker_report(&bytes, &wrong).unwrap_err();
        assert_eq!(
            reference_error.kind(),
            ProgramCertificateErrorKind::CheckerExecution
        );
        let rust_rejection = ProgramCertificateError::new(
            ProgramCertificateErrorKind::CheckerRejected,
            "Rust rejected",
        );
        assert_eq!(
            reconcile_checker_results(&bytes, Err(rust_rejection), Err(reference_error))
                .unwrap_err()
                .kind(),
            ProgramCertificateErrorKind::CheckerExecution,
            "an unbound Go acceptance must not become reportable disagreement evidence"
        );
    }

    #[test]
    fn alpha_shape_rejects_reserved_tables_theory_primitives_and_raw_bool_types() {
        let (certificate, generated_start) = alpha_certificate();
        validate_alpha_candidate_shape(&certificate, generated_start)
            .expect("frozen program certificate has the alpha shape");

        let mut with_import = certificate.clone();
        with_import.imports.push(mpk_cert::encode::Import {
            module_name: "Forbidden.Import".to_owned(),
            export_hash: ZERO_HASH,
            certificate_hash: None,
        });
        assert_eq!(
            validate_alpha_candidate_shape(&with_import, generated_start)
                .unwrap_err()
                .kind(),
            ProgramCertificateErrorKind::Interface
        );

        let mut with_theory_proof = certificate.clone();
        with_theory_proof
            .proof_node_table
            .push(mpk_cert::encode::ProofNode::Theory {
                theory_certificate: 0,
                expected_type: 0,
            });
        assert_eq!(
            validate_alpha_candidate_shape(&with_theory_proof, generated_start)
                .unwrap_err()
                .kind(),
            ProgramCertificateErrorKind::Interface
        );

        let mut with_theory_table = certificate.clone();
        with_theory_table
            .theory_certificates
            .push(mpk_cert::encode::TheoryCertificate {
                format: "forbidden".to_owned(),
                payload: Vec::new(),
            });
        assert_eq!(
            validate_alpha_candidate_shape(&with_theory_table, generated_start)
                .unwrap_err()
                .kind(),
            ProgramCertificateErrorKind::Interface
        );

        let mut with_primitive = certificate.clone();
        let primitive_type = match with_primitive.declarations[generated_start].kind {
            DeclarationKind::Theorem { ty, .. } => ty,
            _ => panic!("generated fixture declaration is a theorem"),
        };
        with_primitive.declarations[generated_start].kind =
            DeclarationKind::TheoryPrimitive { ty: primitive_type };
        assert_eq!(
            validate_alpha_candidate_shape(&with_primitive, generated_start)
                .unwrap_err()
                .kind(),
            ProgramCertificateErrorKind::Interface
        );

        let bool_global = certificate
            .declarations
            .iter()
            .position(|declaration| certificate.name_table[declaration.name as usize] == STD_BOOL)
            .expect("fixture contains Std.Bool") as u32;
        let bool_type = certificate
            .term_table
            .iter()
            .position(|term| {
                matches!(term, TermNode::Const { global, levels } if *global == bool_global && levels.is_empty())
            })
            .expect("fixture contains a Std.Bool term") as u32;
        let mut with_raw_bool_type = certificate;
        let proof = match with_raw_bool_type.declarations[generated_start].kind {
            DeclarationKind::Theorem { proof, .. } => proof,
            _ => panic!("generated fixture declaration is a theorem"),
        };
        with_raw_bool_type.declarations[generated_start].kind = DeclarationKind::Theorem {
            ty: bool_type,
            proof,
        };
        assert_eq!(
            validate_alpha_candidate_shape(&with_raw_bool_type, generated_start)
                .unwrap_err()
                .kind(),
            ProgramCertificateErrorKind::Interface
        );
    }
}
