//! Axiom report recomputation from checked certificate declarations.

use std::collections::{BTreeMap, BTreeSet};

use crate::{
    encode::{
        AxiomCategory, AxiomReport, AxiomReportEntry, AxiomReportSummary, Certificate, Declaration,
        DeclarationAxiomDependencies, DeclarationId, DeclarationKind, HashBytes, LevelId,
        LevelNode, TermId, TermNode,
    },
    export::{declaration_interface_hash, ExportBuildError},
    hash::{axiom_report_hash, term_hash},
    LevelTag, TermTag,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AxiomReportBuildError {
    kind: AxiomReportBuildErrorKind,
    detail: String,
}

impl AxiomReportBuildError {
    pub fn kind(&self) -> AxiomReportBuildErrorKind {
        self.kind
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }

    fn new(kind: AxiomReportBuildErrorKind, detail: impl Into<String>) -> Self {
        Self {
            kind,
            detail: detail.into(),
        }
    }

    fn export(error: ExportBuildError) -> Self {
        Self::new(
            AxiomReportBuildErrorKind::ExportBuildFailed,
            error.detail().to_owned(),
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum AxiomReportBuildErrorKind {
    MissingName,
    MissingDeclaration,
    MissingTerm,
    MissingLevel,
    FutureDeclarationReference,
    CyclicTermReference,
    CyclicLevelReference,
    ExportBuildFailed,
}

pub fn build_axiom_report(certificate: &Certificate) -> Result<AxiomReport, AxiomReportBuildError> {
    AxiomReportBuilder::new(certificate).build()
}

pub fn encode_axiom_report(report: &AxiomReport) -> Vec<u8> {
    let mut encoder = ReportEncoder::new();
    encoder.write_axiom_report(report);
    encoder.finish()
}

pub fn axiom_report_hash_for_report(report: &AxiomReport) -> HashBytes {
    axiom_report_hash(&encode_axiom_report(report))
}

#[derive(Clone, Debug, Default)]
struct DeclarationDependencies {
    direct_axioms: BTreeSet<DeclarationId>,
    transitive_axioms: BTreeSet<DeclarationId>,
}

#[derive(Clone, Debug)]
struct AxiomCandidate {
    declaration_id: DeclarationId,
    entry: AxiomReportEntry,
}

struct AxiomReportBuilder<'a> {
    certificate: &'a Certificate,
    observed_axioms: BTreeMap<DeclarationId, AxiomCategory>,
    declaration_dependencies: Vec<DeclarationDependencies>,
}

impl<'a> AxiomReportBuilder<'a> {
    fn new(certificate: &'a Certificate) -> Self {
        Self {
            certificate,
            observed_axioms: observed_axioms(certificate),
            declaration_dependencies: Vec::new(),
        }
    }

    fn build(mut self) -> Result<AxiomReport, AxiomReportBuildError> {
        self.declaration_dependencies = self.compute_declaration_dependencies()?;
        let mut candidates = self.build_axiom_entries()?;
        candidates
            .sort_by(|lhs, rhs| axiom_entry_key(&lhs.entry).cmp(&axiom_entry_key(&rhs.entry)));

        let mut entry_index_by_declaration = BTreeMap::new();
        for (index, candidate) in candidates.iter().enumerate() {
            let index = DeclarationId::try_from(index).map_err(|_| {
                AxiomReportBuildError::new(
                    AxiomReportBuildErrorKind::MissingDeclaration,
                    "axiom report entry count exceeds u32 ids",
                )
            })?;
            entry_index_by_declaration.insert(candidate.declaration_id, index);
        }

        let entries = candidates
            .into_iter()
            .map(|candidate| candidate.entry)
            .collect::<Vec<_>>();
        let declaration_dependencies =
            self.build_declaration_dependency_entries(&entry_index_by_declaration)?;
        let summary = summarize_axioms(&entries);

        Ok(AxiomReport {
            entries,
            declaration_dependencies,
            summary,
        })
    }

    fn compute_declaration_dependencies(
        &self,
    ) -> Result<Vec<DeclarationDependencies>, AxiomReportBuildError> {
        let mut computed: Vec<DeclarationDependencies> =
            Vec::with_capacity(self.certificate.declarations.len());

        for (index, declaration) in self.certificate.declarations.iter().enumerate() {
            let declaration_id = declaration_id(index)?;
            let direct_references =
                self.collect_declaration_references(declaration_id, declaration)?;
            let mut direct_axioms = BTreeSet::new();
            if self.observed_axioms.contains_key(&declaration_id) {
                direct_axioms.insert(declaration_id);
            }
            for reference in &direct_references {
                if self.observed_axioms.contains_key(reference) {
                    direct_axioms.insert(*reference);
                }
            }

            let mut transitive_axioms = direct_axioms.clone();
            for reference in direct_references {
                let reference_index = usize::try_from(reference).expect("u32 id fits in usize");
                transitive_axioms
                    .extend(computed[reference_index].transitive_axioms.iter().copied());
            }

            computed.push(DeclarationDependencies {
                direct_axioms,
                transitive_axioms,
            });
        }

        Ok(computed)
    }

    fn collect_declaration_references(
        &self,
        declaration_id: DeclarationId,
        declaration: &Declaration,
    ) -> Result<BTreeSet<DeclarationId>, AxiomReportBuildError> {
        let mut references = BTreeSet::new();

        match &declaration.kind {
            DeclarationKind::Axiom { ty }
            | DeclarationKind::Inductive { ty }
            | DeclarationKind::TheoryPrimitive { ty } => {
                self.collect_term_references(declaration_id, *ty, &mut references)?;
            }
            DeclarationKind::Def { ty, value, .. } => {
                self.collect_term_references(declaration_id, *ty, &mut references)?;
                self.collect_term_references(declaration_id, *value, &mut references)?;
            }
            DeclarationKind::Theorem { ty, proof } => {
                self.collect_term_references(declaration_id, *ty, &mut references)?;
                self.collect_term_references(declaration_id, *proof, &mut references)?;
            }
            DeclarationKind::Constructor { ty, inductive, .. }
            | DeclarationKind::Recursor { ty, inductive, .. } => {
                self.collect_term_references(declaration_id, *ty, &mut references)?;
                self.add_declaration_reference(declaration_id, *inductive, &mut references)?;
            }
        }

        Ok(references)
    }

    fn collect_term_references(
        &self,
        declaration_id: DeclarationId,
        term: TermId,
        references: &mut BTreeSet<DeclarationId>,
    ) -> Result<(), AxiomReportBuildError> {
        let mut visiting = BTreeSet::new();
        self.collect_term_references_inner(declaration_id, term, references, &mut visiting)
    }

    fn collect_term_references_inner(
        &self,
        declaration_id: DeclarationId,
        term: TermId,
        references: &mut BTreeSet<DeclarationId>,
        visiting: &mut BTreeSet<TermId>,
    ) -> Result<(), AxiomReportBuildError> {
        if !visiting.insert(term) {
            return Err(AxiomReportBuildError::new(
                AxiomReportBuildErrorKind::CyclicTermReference,
                format!("term {term} references itself"),
            ));
        }

        let node = self.term(term)?;
        match node {
            TermNode::Sort(_) | TermNode::Var(_) => {}
            TermNode::Const { global, .. } => {
                self.add_declaration_reference(declaration_id, *global, references)?;
            }
            TermNode::App {
                function,
                arguments,
            } => {
                self.collect_term_references_inner(
                    declaration_id,
                    *function,
                    references,
                    visiting,
                )?;
                for argument in arguments {
                    self.collect_term_references_inner(
                        declaration_id,
                        *argument,
                        references,
                        visiting,
                    )?;
                }
            }
            TermNode::Lam { ty, body } | TermNode::Pi { ty, body } => {
                self.collect_term_references_inner(declaration_id, *ty, references, visiting)?;
                self.collect_term_references_inner(declaration_id, *body, references, visiting)?;
            }
            TermNode::Let { ty, value, body } => {
                self.collect_term_references_inner(declaration_id, *ty, references, visiting)?;
                self.collect_term_references_inner(declaration_id, *value, references, visiting)?;
                self.collect_term_references_inner(declaration_id, *body, references, visiting)?;
            }
        }

        visiting.remove(&term);
        Ok(())
    }

    fn add_declaration_reference(
        &self,
        owner: DeclarationId,
        reference: DeclarationId,
        references: &mut BTreeSet<DeclarationId>,
    ) -> Result<(), AxiomReportBuildError> {
        if usize::try_from(reference)
            .ok()
            .and_then(|index| self.certificate.declarations.get(index))
            .is_none()
        {
            return Err(AxiomReportBuildError::new(
                AxiomReportBuildErrorKind::MissingDeclaration,
                format!("declaration {owner} references missing declaration {reference}"),
            ));
        }
        if reference >= owner {
            return Err(AxiomReportBuildError::new(
                AxiomReportBuildErrorKind::FutureDeclarationReference,
                format!("declaration {owner} references non-previous declaration {reference}"),
            ));
        }

        references.insert(reference);
        Ok(())
    }

    fn build_axiom_entries(&self) -> Result<Vec<AxiomCandidate>, AxiomReportBuildError> {
        let mut candidates = Vec::with_capacity(self.observed_axioms.len());

        for (declaration_id, category) in &self.observed_axioms {
            let declaration = self.declaration(*declaration_id)?;
            let name = self.name_table_entry(declaration.name)?.to_owned();
            let type_hash = self.declaration_type_hash(declaration)?;
            let declaration_hash =
                declaration_interface_hash(&self.certificate.name_table, declaration)
                    .map_err(AxiomReportBuildError::export)?;
            let direct_dependent_declarations =
                self.dependent_declarations(*declaration_id, DependencyKind::Direct);
            let transitive_dependent_declarations =
                self.dependent_declarations(*declaration_id, DependencyKind::Transitive);

            candidates.push(AxiomCandidate {
                declaration_id: *declaration_id,
                entry: AxiomReportEntry {
                    category: *category,
                    name,
                    origin_module: self.certificate.module.clone(),
                    type_hash,
                    declaration_hash,
                    source_certificate_hash: None,
                    direct_dependent_declarations,
                    transitive_dependent_declarations,
                    approval_profile: None,
                    reviewer_note: None,
                },
            });
        }

        Ok(candidates)
    }

    fn build_declaration_dependency_entries(
        &self,
        entry_index_by_declaration: &BTreeMap<DeclarationId, DeclarationId>,
    ) -> Result<Vec<DeclarationAxiomDependencies>, AxiomReportBuildError> {
        let mut entries = Vec::new();

        for (index, dependencies) in self.declaration_dependencies.iter().enumerate() {
            if dependencies.transitive_axioms.is_empty() {
                continue;
            }

            let declaration = &self.certificate.declarations[index];
            let declaration_name = self.name_table_entry(declaration.name)?.to_owned();
            let declaration_hash =
                declaration_interface_hash(&self.certificate.name_table, declaration)
                    .map_err(AxiomReportBuildError::export)?;
            let direct_axiom_dependencies = map_axiom_ids_to_entry_indices(
                &dependencies.direct_axioms,
                entry_index_by_declaration,
            )?;
            let transitive_axiom_dependencies = map_axiom_ids_to_entry_indices(
                &dependencies.transitive_axioms,
                entry_index_by_declaration,
            )?;

            entries.push(DeclarationAxiomDependencies {
                declaration_name,
                declaration_hash,
                direct_axiom_dependencies,
                transitive_axiom_dependencies,
            });
        }

        entries.sort_by(|lhs, rhs| {
            declaration_axiom_dependencies_key(lhs).cmp(&declaration_axiom_dependencies_key(rhs))
        });
        Ok(entries)
    }

    fn dependent_declarations(
        &self,
        axiom: DeclarationId,
        kind: DependencyKind,
    ) -> Vec<DeclarationId> {
        self.declaration_dependencies
            .iter()
            .enumerate()
            .filter_map(|(index, dependencies)| {
                let set = match kind {
                    DependencyKind::Direct => &dependencies.direct_axioms,
                    DependencyKind::Transitive => &dependencies.transitive_axioms,
                };
                set.contains(&axiom)
                    .then(|| declaration_id(index).expect("index fits in u32"))
            })
            .collect()
    }

    fn declaration_type_hash(
        &self,
        declaration: &Declaration,
    ) -> Result<HashBytes, AxiomReportBuildError> {
        let ty = match &declaration.kind {
            DeclarationKind::Axiom { ty }
            | DeclarationKind::Def { ty, .. }
            | DeclarationKind::Theorem { ty, .. }
            | DeclarationKind::Inductive { ty }
            | DeclarationKind::Constructor { ty, .. }
            | DeclarationKind::Recursor { ty, .. }
            | DeclarationKind::TheoryPrimitive { ty } => *ty,
        };
        let payload = self.encode_term_payload(ty)?;
        Ok(term_hash(&payload))
    }

    fn encode_term_payload(&self, term: TermId) -> Result<Vec<u8>, AxiomReportBuildError> {
        let mut encoder = ReportEncoder::new();
        let mut visiting = BTreeSet::new();
        self.write_term_payload(term, &mut encoder, &mut visiting)?;
        Ok(encoder.finish())
    }

    fn write_term_payload(
        &self,
        term: TermId,
        encoder: &mut ReportEncoder,
        visiting: &mut BTreeSet<TermId>,
    ) -> Result<(), AxiomReportBuildError> {
        if !visiting.insert(term) {
            return Err(AxiomReportBuildError::new(
                AxiomReportBuildErrorKind::CyclicTermReference,
                format!("term {term} references itself"),
            ));
        }

        match self.term(term)? {
            TermNode::Sort(level) => {
                encoder.write_u8(TermTag::Sort.as_u8());
                self.write_level_payload(*level, encoder, &mut BTreeSet::new())?;
            }
            TermNode::Var(index) => {
                encoder.write_u8(TermTag::Var.as_u8());
                encoder.write_u32(*index);
            }
            TermNode::Const { global, levels } => {
                encoder.write_u8(TermTag::Const.as_u8());
                encoder.write_u32(*global);
                encoder.write_len(levels.len());
                for level in levels {
                    self.write_level_payload(*level, encoder, &mut BTreeSet::new())?;
                }
            }
            TermNode::App {
                function,
                arguments,
            } => {
                encoder.write_u8(TermTag::App.as_u8());
                self.write_term_payload(*function, encoder, visiting)?;
                encoder.write_len(arguments.len());
                for argument in arguments {
                    self.write_term_payload(*argument, encoder, visiting)?;
                }
            }
            TermNode::Lam { ty, body } => {
                encoder.write_u8(TermTag::Lam.as_u8());
                self.write_term_payload(*ty, encoder, visiting)?;
                self.write_term_payload(*body, encoder, visiting)?;
            }
            TermNode::Pi { ty, body } => {
                encoder.write_u8(TermTag::Pi.as_u8());
                self.write_term_payload(*ty, encoder, visiting)?;
                self.write_term_payload(*body, encoder, visiting)?;
            }
            TermNode::Let { ty, value, body } => {
                encoder.write_u8(TermTag::Let.as_u8());
                self.write_term_payload(*ty, encoder, visiting)?;
                self.write_term_payload(*value, encoder, visiting)?;
                self.write_term_payload(*body, encoder, visiting)?;
            }
        }

        visiting.remove(&term);
        Ok(())
    }

    fn write_level_payload(
        &self,
        level: LevelId,
        encoder: &mut ReportEncoder,
        visiting: &mut BTreeSet<LevelId>,
    ) -> Result<(), AxiomReportBuildError> {
        if !visiting.insert(level) {
            return Err(AxiomReportBuildError::new(
                AxiomReportBuildErrorKind::CyclicLevelReference,
                format!("level {level} references itself"),
            ));
        }

        match self.level(level)? {
            LevelNode::Zero => encoder.write_u8(LevelTag::Zero.as_u8()),
            LevelNode::Succ(inner) => {
                encoder.write_u8(LevelTag::Succ.as_u8());
                self.write_level_payload(*inner, encoder, visiting)?;
            }
            LevelNode::Max(lhs, rhs) => {
                encoder.write_u8(LevelTag::Max.as_u8());
                self.write_level_payload(*lhs, encoder, visiting)?;
                self.write_level_payload(*rhs, encoder, visiting)?;
            }
            LevelNode::Param(name) => {
                encoder.write_u8(LevelTag::Param.as_u8());
                let name = self.name_table_entry(*name)?;
                encoder.write_str_slice(name);
            }
        }

        visiting.remove(&level);
        Ok(())
    }

    fn declaration(
        &self,
        declaration: DeclarationId,
    ) -> Result<&'a Declaration, AxiomReportBuildError> {
        self.certificate
            .declarations
            .get(usize::try_from(declaration).expect("u32 id fits in usize"))
            .ok_or_else(|| {
                AxiomReportBuildError::new(
                    AxiomReportBuildErrorKind::MissingDeclaration,
                    format!("missing declaration {declaration}"),
                )
            })
    }

    fn term(&self, term: TermId) -> Result<&'a TermNode, AxiomReportBuildError> {
        self.certificate
            .term_table
            .get(usize::try_from(term).expect("u32 id fits in usize"))
            .ok_or_else(|| {
                AxiomReportBuildError::new(
                    AxiomReportBuildErrorKind::MissingTerm,
                    format!("missing term {term}"),
                )
            })
    }

    fn level(&self, level: LevelId) -> Result<&'a LevelNode, AxiomReportBuildError> {
        self.certificate
            .level_table
            .get(usize::try_from(level).expect("u32 id fits in usize"))
            .ok_or_else(|| {
                AxiomReportBuildError::new(
                    AxiomReportBuildErrorKind::MissingLevel,
                    format!("missing level {level}"),
                )
            })
    }

    fn name_table_entry(&self, name: u32) -> Result<&'a str, AxiomReportBuildError> {
        self.certificate
            .name_table
            .get(usize::try_from(name).expect("u32 id fits in usize"))
            .map(String::as_str)
            .ok_or_else(|| {
                AxiomReportBuildError::new(
                    AxiomReportBuildErrorKind::MissingName,
                    format!("missing name id {name}"),
                )
            })
    }
}

#[derive(Clone, Copy)]
enum DependencyKind {
    Direct,
    Transitive,
}

fn observed_axioms(certificate: &Certificate) -> BTreeMap<DeclarationId, AxiomCategory> {
    certificate
        .declarations
        .iter()
        .enumerate()
        .filter_map(|(index, declaration)| {
            let category = match declaration.kind {
                DeclarationKind::Axiom { .. } => AxiomCategory::CoreAxiom,
                DeclarationKind::TheoryPrimitive { .. } => AxiomCategory::BuiltinTheoryAxiom,
                _ => return None,
            };
            Some((declaration_id(index).expect("index fits in u32"), category))
        })
        .collect()
}

fn map_axiom_ids_to_entry_indices(
    axiom_ids: &BTreeSet<DeclarationId>,
    entry_index_by_declaration: &BTreeMap<DeclarationId, DeclarationId>,
) -> Result<Vec<DeclarationId>, AxiomReportBuildError> {
    let mut indices = axiom_ids
        .iter()
        .map(|axiom| {
            entry_index_by_declaration
                .get(axiom)
                .copied()
                .ok_or_else(|| {
                    AxiomReportBuildError::new(
                        AxiomReportBuildErrorKind::MissingDeclaration,
                        format!("missing report entry for axiom declaration {axiom}"),
                    )
                })
        })
        .collect::<Result<Vec<_>, _>>()?;
    indices.sort_unstable();
    Ok(indices)
}

fn declaration_id(index: usize) -> Result<DeclarationId, AxiomReportBuildError> {
    DeclarationId::try_from(index).map_err(|_| {
        AxiomReportBuildError::new(
            AxiomReportBuildErrorKind::MissingDeclaration,
            "declaration count exceeds u32 ids",
        )
    })
}

fn axiom_entry_key(entry: &AxiomReportEntry) -> (&str, &str, &str, HashBytes, HashBytes) {
    (
        entry.category.canonical_name(),
        entry.name.as_str(),
        entry.origin_module.as_str(),
        entry.type_hash,
        entry.declaration_hash,
    )
}

fn declaration_axiom_dependencies_key(
    dependencies: &DeclarationAxiomDependencies,
) -> (&str, HashBytes) {
    (
        dependencies.declaration_name.as_str(),
        dependencies.declaration_hash,
    )
}

fn summarize_axioms(entries: &[AxiomReportEntry]) -> AxiomReportSummary {
    let mut summary = AxiomReportSummary::default();
    for entry in entries {
        match entry.category {
            AxiomCategory::CoreAxiom => summary.core_axiom_count += 1,
            AxiomCategory::BuiltinTheoryAxiom => summary.builtin_theory_axiom_count += 1,
            AxiomCategory::GoSemanticsAxiom => summary.go_semantics_axiom_count += 1,
            AxiomCategory::ExternalAxiom => summary.external_axiom_count += 1,
        }
        summary.total_axiom_count += 1;
    }
    summary
}

struct ReportEncoder {
    bytes: Vec<u8>,
}

impl ReportEncoder {
    fn new() -> Self {
        Self { bytes: Vec::new() }
    }

    fn finish(self) -> Vec<u8> {
        self.bytes
    }

    fn write_u8(&mut self, value: u8) {
        self.bytes.push(value);
    }

    fn write_bool(&mut self, value: bool) {
        self.write_u8(u8::from(value));
    }

    fn write_u32(&mut self, value: u32) {
        self.write_u64(u64::from(value));
    }

    fn write_u64(&mut self, value: u64) {
        Self::write_unsigned_varint_to(value, &mut self.bytes);
    }

    fn write_len(&mut self, len: usize) {
        let len = u64::try_from(len).expect("axiom report section length exceeds u64");
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
        self.bytes.extend_from_slice(bytes);
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

    fn write_vec<T>(&mut self, values: &[T], mut write_item: impl FnMut(&mut Self, &T)) {
        self.write_len(values.len());
        for value in values {
            write_item(self, value);
        }
    }

    fn write_u32_vec(&mut self, values: &[u32]) {
        self.write_vec(values, |encoder, value| encoder.write_u32(*value));
    }

    fn write_axiom_report(&mut self, report: &AxiomReport) {
        self.write_vec(&report.entries, ReportEncoder::write_axiom_report_entry);
        self.write_vec(
            &report.declaration_dependencies,
            ReportEncoder::write_declaration_axiom_dependencies,
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
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::{
        axiom_report_hash_for_report, build_axiom_report,
        encode::{
            AxiomReport, Certificate, CertificateHashes, Declaration, DeclarationKind,
            DefinitionReducibility, LevelNode, TermNode,
        },
        hash_hex,
    };

    use super::{encode_axiom_report, AxiomReportBuildErrorKind};

    const AXIOM_REPORT_FIXTURE_DIR: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/cert-axiom-report"
    );

    fn fixture_certificate() -> Certificate {
        Certificate {
            module: "Example.AxiomReport".to_owned(),
            imports: Vec::new(),
            name_table: vec![
                "Example.AxiomReport.ax".to_owned(),
                "Example.AxiomReport.def".to_owned(),
                "Example.AxiomReport.thm".to_owned(),
                "Example.AxiomReport.theory".to_owned(),
                "Example.AxiomReport.usesTheory".to_owned(),
            ],
            level_table: vec![LevelNode::Zero],
            term_table: vec![
                TermNode::Sort(0),
                TermNode::Const {
                    global: 0,
                    levels: Vec::new(),
                },
                TermNode::Const {
                    global: 1,
                    levels: Vec::new(),
                },
                TermNode::Var(0),
                TermNode::Const {
                    global: 3,
                    levels: Vec::new(),
                },
            ],
            proof_node_table: Vec::new(),
            declarations: vec![
                Declaration {
                    name: 0,
                    kind: DeclarationKind::Axiom { ty: 0 },
                },
                Declaration {
                    name: 1,
                    kind: DeclarationKind::Def {
                        ty: 0,
                        value: 1,
                        reducibility: DefinitionReducibility::Reducible,
                    },
                },
                Declaration {
                    name: 2,
                    kind: DeclarationKind::Theorem { ty: 2, proof: 3 },
                },
                Declaration {
                    name: 3,
                    kind: DeclarationKind::TheoryPrimitive { ty: 0 },
                },
                Declaration {
                    name: 4,
                    kind: DeclarationKind::Theorem { ty: 4, proof: 3 },
                },
            ],
            theory_certificates: Vec::new(),
            export_block: Vec::new(),
            axiom_report: AxiomReport::default(),
            source_manifest: None,
            hashes: CertificateHashes::default(),
        }
    }

    fn render_report_fixture(report: &AxiomReport) -> String {
        let mut output = String::new();
        output.push_str(&format!(
            "summary core={} builtin={} go={} external={} total={}\n",
            report.summary.core_axiom_count,
            report.summary.builtin_theory_axiom_count,
            report.summary.go_semantics_axiom_count,
            report.summary.external_axiom_count,
            report.summary.total_axiom_count
        ));
        output.push_str("entries\n");
        for (index, entry) in report.entries.iter().enumerate() {
            output.push_str(&format!(
                "{index} category={} name={} origin={} type_hash={} declaration_hash={} direct={:?} transitive={:?}\n",
                entry.category.canonical_name(),
                entry.name,
                entry.origin_module,
                hash_hex(&entry.type_hash),
                hash_hex(&entry.declaration_hash),
                entry.direct_dependent_declarations,
                entry.transitive_dependent_declarations
            ));
        }
        output.push_str("declarations\n");
        for dependencies in &report.declaration_dependencies {
            output.push_str(&format!(
                "{} declaration_hash={} direct={:?} transitive={:?}\n",
                dependencies.declaration_name,
                hash_hex(&dependencies.declaration_hash),
                dependencies.direct_axiom_dependencies,
                dependencies.transitive_axiom_dependencies
            ));
        }
        output
    }

    #[test]
    fn axiom_fixture_matches_expected_report() {
        let report = build_axiom_report(&fixture_certificate()).expect("report builds");
        let actual = render_report_fixture(&report);
        let expected = fs::read_to_string(format!("{AXIOM_REPORT_FIXTURE_DIR}/basic-report.txt"))
            .expect("expected report fixture is readable");

        assert_eq!(actual, expected, "actual report:\n{actual}");
    }

    #[test]
    fn direct_and_transitive_dependencies_are_computed() {
        let report = build_axiom_report(&fixture_certificate()).expect("report builds");

        let axiom = report
            .entries
            .iter()
            .find(|entry| entry.name == "Example.AxiomReport.ax")
            .expect("core axiom entry exists");
        assert_eq!(axiom.direct_dependent_declarations, [0, 1]);
        assert_eq!(axiom.transitive_dependent_declarations, [0, 1, 2]);

        let theorem_dependencies = report
            .declaration_dependencies
            .iter()
            .find(|dependencies| dependencies.declaration_name == "Example.AxiomReport.thm")
            .expect("theorem dependencies exist");
        assert!(theorem_dependencies.direct_axiom_dependencies.is_empty());
        assert_eq!(theorem_dependencies.transitive_axiom_dependencies.len(), 1);
    }

    #[test]
    fn theory_primitives_are_reported_as_builtin_theory_axioms() {
        let report = build_axiom_report(&fixture_certificate()).expect("report builds");

        let theory = report
            .entries
            .iter()
            .find(|entry| entry.name == "Example.AxiomReport.theory")
            .expect("theory primitive entry exists");
        assert_eq!(theory.category.canonical_name(), "BuiltinTheoryAxiom");
        assert_eq!(theory.direct_dependent_declarations, [3, 4]);
        assert_eq!(theory.transitive_dependent_declarations, [3, 4]);
    }

    #[test]
    fn axiom_report_hash_uses_encoded_report_payload() {
        let report = build_axiom_report(&fixture_certificate()).expect("report builds");

        assert_eq!(
            axiom_report_hash_for_report(&report),
            crate::axiom_report_hash(&encode_axiom_report(&report))
        );
    }

    #[test]
    fn future_declaration_references_reject() {
        let mut certificate = fixture_certificate();
        certificate.declarations[1].kind = DeclarationKind::Def {
            ty: 0,
            value: 4,
            reducibility: DefinitionReducibility::Reducible,
        };

        let error = build_axiom_report(&certificate).unwrap_err();

        assert_eq!(
            error.kind(),
            AxiomReportBuildErrorKind::FutureDeclarationReference
        );
    }
}
