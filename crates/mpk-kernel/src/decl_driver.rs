//! Declaration checking orchestration for canonical certificates.

use crate::cache::CheckerCache;

use mpk_cert::encode::{Certificate, DeclarationKind, DefinitionReducibility, LevelNode, TermNode};
use mpk_core::{
    CoreError, CoreErrorCode, CoreLocation, Environment, GlobalId, LevelArena, LevelId,
    LocalContext, Name, TermArena, TermId, TermNode as CoreTermNode,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeclarationCheckReport {
    pub declaration_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeclarationCheckError {
    kind: DeclarationCheckErrorKind,
    detail: String,
}

impl DeclarationCheckError {
    pub fn kind(&self) -> DeclarationCheckErrorKind {
        self.kind
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }

    fn new(kind: DeclarationCheckErrorKind, detail: impl Into<String>) -> Self {
        Self {
            kind,
            detail: detail.into(),
        }
    }

    fn unsupported(detail: impl Into<String>) -> Self {
        Self::new(
            DeclarationCheckErrorKind::UnsupportedDeclarationKind,
            detail,
        )
    }

    fn core(error: CoreError) -> Self {
        Self::new(
            DeclarationCheckErrorKind::CoreCheck,
            error.to_deterministic_json(),
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum DeclarationCheckErrorKind {
    UnsupportedDeclarationKind,
    MissingName,
    MissingGlobal,
    OutOfOrderDeclarationDependency,
    CoreCheck,
    InternalInvariant,
}

pub fn check_declarations(
    certificate: &Certificate,
) -> Result<DeclarationCheckReport, DeclarationCheckError> {
    let context = check_declarations_with_context(certificate)?;
    Ok(DeclarationCheckReport {
        declaration_count: context.declaration_count(),
    })
}

pub(crate) fn check_declarations_with_context(
    certificate: &Certificate,
) -> Result<CheckedDeclarationContext<'_>, DeclarationCheckError> {
    let mut context = CheckedDeclarationContext::new(certificate);
    context.check_declarations()?;
    Ok(context)
}

pub(crate) struct CheckedDeclarationContext<'certificate> {
    certificate: &'certificate Certificate,
    levels: LevelArena,
    terms: TermArena,
    env: Environment,
    level_cache: Vec<Option<LevelId>>,
    term_cache: Vec<Option<TermId>>,
    globals: Vec<GlobalId>,
    cache: CheckerCache,
}

impl<'certificate> CheckedDeclarationContext<'certificate> {
    fn new(certificate: &'certificate Certificate) -> Self {
        Self {
            certificate,
            levels: LevelArena::new(),
            terms: TermArena::new(),
            env: Environment::new(),
            level_cache: vec![None; certificate.level_table.len()],
            term_cache: vec![None; certificate.term_table.len()],
            globals: Vec::with_capacity(certificate.declarations.len()),
            cache: CheckerCache::new(),
        }
    }

    pub(crate) fn certificate(&self) -> &'certificate Certificate {
        self.certificate
    }

    pub(crate) fn declaration_count(&self) -> usize {
        self.certificate.declarations.len()
    }

    pub(crate) fn core_parts(&mut self) -> (&mut LevelArena, &mut TermArena, &Environment) {
        (&mut self.levels, &mut self.terms, &self.env)
    }

    pub(crate) fn cached_core_parts(
        &mut self,
    ) -> (
        &mut LevelArena,
        &mut TermArena,
        &Environment,
        &mut CheckerCache,
    ) {
        (
            &mut self.levels,
            &mut self.terms,
            &self.env,
            &mut self.cache,
        )
    }

    fn check_declarations(&mut self) -> Result<(), DeclarationCheckError> {
        for (index, declaration) in self.certificate.declarations.iter().enumerate() {
            let name = self.name_table_entry(declaration.name)?.to_owned();
            let global = match &declaration.kind {
                DeclarationKind::Axiom { ty } => {
                    let ty = self.translate_term(*ty)?;
                    self.expect_term_type_is_sort(index, "axiom_type", ty)?;
                    self.env
                        .register_axiom(name, ty)
                        .map_err(DeclarationCheckError::core)?
                }
                DeclarationKind::Def {
                    ty,
                    value,
                    reducibility,
                } => {
                    let ty = self.translate_term(*ty)?;
                    let value = self.translate_term(*value)?;
                    self.expect_term_type_is_sort(index, "definition_type", ty)?;
                    self.cache
                        .check(
                            &mut self.levels,
                            &mut self.terms,
                            &LocalContext::new(),
                            &self.env,
                            value,
                            ty,
                        )
                        .map_err(DeclarationCheckError::core)?;
                    self.env
                        .register_definition(name, ty, value, convert_reducibility(*reducibility))
                        .map_err(DeclarationCheckError::core)?
                }
                DeclarationKind::Theorem { ty, proof } => {
                    let ty = self.translate_term(*ty)?;
                    let proof = self.translate_term(*proof)?;
                    self.expect_term_type_is_sort(index, "theorem_type", ty)?;
                    self.cache
                        .check(
                            &mut self.levels,
                            &mut self.terms,
                            &LocalContext::new(),
                            &self.env,
                            proof,
                            ty,
                        )
                        .map_err(DeclarationCheckError::core)?;
                    self.env
                        .register_theorem(name, ty, proof)
                        .map_err(DeclarationCheckError::core)?
                }
                DeclarationKind::Inductive { ty } => {
                    let ty = self.translate_term(*ty)?;
                    self.expect_term_type_is_sort(index, "inductive_type", ty)?;
                    self.env
                        .register_inductive(name, ty)
                        .map_err(DeclarationCheckError::core)?
                }
                DeclarationKind::Constructor {
                    ty,
                    inductive,
                    generated,
                } => {
                    let ty = self.translate_term(*ty)?;
                    let inductive = self.global_by_dependency(*inductive)?;
                    self.expect_term_type_is_sort(index, "constructor_type", ty)?;
                    if *generated {
                        self.env
                            .register_generated_constructor(name, ty, inductive)
                            .map_err(DeclarationCheckError::core)?
                    } else {
                        self.env
                            .register_constructor(name, ty, inductive)
                            .map_err(DeclarationCheckError::core)?
                    }
                }
                DeclarationKind::Recursor {
                    ty,
                    inductive,
                    generated,
                } => {
                    let ty = self.translate_term(*ty)?;
                    let inductive = self.global_by_dependency(*inductive)?;
                    self.expect_term_type_is_sort(index, "recursor_type", ty)?;
                    if *generated {
                        self.env
                            .register_generated_recursor(name, ty, inductive)
                            .map_err(DeclarationCheckError::core)?
                    } else {
                        self.env
                            .register_recursor(name, ty, inductive)
                            .map_err(DeclarationCheckError::core)?
                    }
                }
                DeclarationKind::TheoryPrimitive { .. } => {
                    return Err(DeclarationCheckError::unsupported(format!(
                        "declaration {index} uses a declaration kind not implemented by KERN-002"
                    )));
                }
            };

            self.push_global(index, global)?;
            self.cache.clear();
        }
        Ok(())
    }

    fn expect_term_type_is_sort(
        &mut self,
        declaration_index: usize,
        field: &'static str,
        term: TermId,
    ) -> Result<(), DeclarationCheckError> {
        let inferred = self
            .cache
            .infer(
                &mut self.levels,
                &mut self.terms,
                &LocalContext::new(),
                &self.env,
                term,
            )
            .map_err(DeclarationCheckError::core)?;
        if matches!(self.terms.node(inferred), CoreTermNode::Sort(_)) {
            return Ok(());
        }

        let declaration_index = u32::try_from(declaration_index).map_err(|_| {
            DeclarationCheckError::new(
                DeclarationCheckErrorKind::InternalInvariant,
                "declaration index exceeds u32",
            )
        })?;
        Err(DeclarationCheckError::core(
            CoreError::new(
                CoreErrorCode::TypeMismatch,
                CoreLocation::root()
                    .with_field("decl_driver")
                    .with_field("declarations")
                    .with_index(declaration_index)
                    .with_field(field),
            )
            .with_detail("kind", "declaration_type_not_sort")
            .with_detail("term_index", term.index().to_string())
            .with_detail("inferred_term_index", inferred.index().to_string()),
        ))
    }

    fn translate_level(&mut self, level: u32) -> Result<LevelId, DeclarationCheckError> {
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
                    DeclarationCheckError::new(DeclarationCheckErrorKind::MissingName, error.code())
                })?;
                self.levels.param(name)
            }
        };

        self.level_cache[index] = Some(translated);
        Ok(translated)
    }

    pub(crate) fn translate_term(&mut self, term: u32) -> Result<TermId, DeclarationCheckError> {
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
                let global = self.global_by_dependency(global)?;
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

    pub(crate) fn global_by_index(&self, global: u32) -> Result<GlobalId, DeclarationCheckError> {
        let index = usize::try_from(global).expect("u32 id fits in usize");
        self.globals.get(index).copied().ok_or_else(|| {
            DeclarationCheckError::new(
                DeclarationCheckErrorKind::MissingGlobal,
                format!("certificate references missing global {global}"),
            )
        })
    }

    fn global_by_dependency(&self, global: u32) -> Result<GlobalId, DeclarationCheckError> {
        let index = usize::try_from(global).expect("u32 id fits in usize");
        if let Some(translated) = self.globals.get(index).copied() {
            return Ok(translated);
        }

        if index < self.certificate.declarations.len() {
            return Err(DeclarationCheckError::new(
                DeclarationCheckErrorKind::OutOfOrderDeclarationDependency,
                format!(
                    "declaration references global {global} before that declaration is checked"
                ),
            ));
        }

        Err(DeclarationCheckError::new(
            DeclarationCheckErrorKind::MissingGlobal,
            format!("declaration references missing global {global}"),
        ))
    }

    fn name_table_entry(&self, name: u32) -> Result<&'certificate str, DeclarationCheckError> {
        self.certificate
            .name_table
            .get(usize::try_from(name).expect("u32 id fits in usize"))
            .map(String::as_str)
            .ok_or_else(|| {
                DeclarationCheckError::new(
                    DeclarationCheckErrorKind::MissingName,
                    format!("missing name id {name}"),
                )
            })
    }

    fn push_global(
        &mut self,
        declaration_index: usize,
        global: GlobalId,
    ) -> Result<(), DeclarationCheckError> {
        let expected = u32::try_from(declaration_index).map_err(|_| {
            DeclarationCheckError::new(
                DeclarationCheckErrorKind::InternalInvariant,
                "declaration index exceeds u32",
            )
        })?;
        if global.as_u32() != expected {
            return Err(DeclarationCheckError::new(
                DeclarationCheckErrorKind::InternalInvariant,
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

fn convert_reducibility(reducibility: DefinitionReducibility) -> mpk_core::DefinitionReducibility {
    match reducibility {
        DefinitionReducibility::Reducible => mpk_core::DefinitionReducibility::Reducible,
        DefinitionReducibility::Opaque => mpk_core::DefinitionReducibility::Opaque,
    }
}

#[cfg(test)]
mod tests {
    use mpk_cert::encode::{
        AxiomReport, Certificate, CertificateHashes, Declaration, DeclarationKind, LevelNode,
        TermNode,
    };

    use super::{check_declarations, DeclarationCheckErrorKind};

    fn one_theorem_certificate(proof: u32) -> Certificate {
        Certificate {
            module: "Example.Driver.OneTheorem".to_owned(),
            imports: Vec::new(),
            name_table: vec!["Example.Driver.OneTheorem.sort0IsSort1".to_owned()],
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
        }
    }

    #[test]
    fn checks_declarations_in_dependency_order() {
        let report = check_declarations(&one_theorem_certificate(0)).expect("declarations check");

        assert_eq!(report.declaration_count, 1);
    }

    #[test]
    fn rejects_out_of_order_declaration_dependency() {
        let mut certificate = one_theorem_certificate(0);
        certificate.name_table = vec![
            "Example.Driver.UsesFuture".to_owned(),
            "Example.Driver.Future".to_owned(),
        ];
        certificate.term_table.push(TermNode::Const {
            global: 1,
            levels: Vec::new(),
        });
        certificate.declarations = vec![
            Declaration {
                name: 0,
                kind: DeclarationKind::Theorem { ty: 1, proof: 2 },
            },
            Declaration {
                name: 1,
                kind: DeclarationKind::Axiom { ty: 1 },
            },
        ];

        let error = check_declarations(&certificate).unwrap_err();

        assert_eq!(
            error.kind(),
            DeclarationCheckErrorKind::OutOfOrderDeclarationDependency
        );
    }

    #[test]
    fn rejects_missing_declaration_dependency() {
        let mut certificate = one_theorem_certificate(0);
        certificate.term_table.push(TermNode::Const {
            global: 99,
            levels: Vec::new(),
        });
        certificate.declarations[0].kind = DeclarationKind::Theorem { ty: 1, proof: 2 };

        let error = check_declarations(&certificate).unwrap_err();

        assert_eq!(error.kind(), DeclarationCheckErrorKind::MissingGlobal);
    }
}
