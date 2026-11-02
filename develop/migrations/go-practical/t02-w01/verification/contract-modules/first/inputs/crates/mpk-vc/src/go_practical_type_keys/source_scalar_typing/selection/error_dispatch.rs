//! Finite Error call closure over original source and explicit entry predicates.
//! Receiver evaluation, nil safety and callee contracts still need runtime proof.
use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GoSourceErrorSelectionError {
    Selection(Box<GoSourceSelectionError>),
    Domain(Box<GoSourceDomainError>),
}
impl From<GoSourceSelectionError> for GoSourceErrorSelectionError {
    fn from(e: GoSourceSelectionError) -> Self {
        Self::Selection(Box::new(e))
    }
}
impl From<GoSourceDomainError> for GoSourceErrorSelectionError {
    fn from(e: GoSourceDomainError) -> Self {
        Self::Domain(Box::new(e))
    }
}
impl std::fmt::Display for GoSourceErrorSelectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Selection(e) => e.fmt(f),
            Self::Domain(e) => e.fmt(f),
        }
    }
}
impl std::error::Error for GoSourceErrorSelectionError {}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoSourceErrorDispatch {
    call: GoSourceErrorCall,
    targets: Vec<GoErrorImplementation>,
}
impl GoSourceErrorDispatch {
    pub fn call(&self) -> &GoSourceErrorCall {
        &self.call
    }
    pub fn targets(&self) -> &[GoErrorImplementation] {
        &self.targets
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoSourceErrorSelection {
    domain: GoSourceValueDomain,
    dispatches: Vec<GoSourceErrorDispatch>,
}
impl GoSourceErrorSelection {
    pub fn domain(&self) -> &GoSourceValueDomain {
        &self.domain
    }
    pub fn selection(&self) -> &GoSourceSelection {
        self.domain.selection()
    }
    pub fn dispatches(&self) -> &[GoSourceErrorDispatch] {
        &self.dispatches
    }
}
impl Selection<'_> {
    fn drain(&mut self, cursor: &mut usize) -> SelectionResult<()> {
        while *cursor < self.work.len() {
            let w = &self.work[*cursor];
            let current = Pending {
                locator: w.locator.clone(),
                role: w.role,
                path: w.path.clone(),
            };
            *cursor += 1;
            self.process(&current)?;
        }
        Ok(())
    }
    fn snapshot(
        &self,
        scopes: Arc<GoSourceScopes>,
        keys: &GoTypeKeyComponents,
        roots: &[GoSourceSelectionRef],
    ) -> GoSourceSelection {
        let declarations = self
            .originals
            .values()
            .map(|o| {
                let mut d = o.declaration.clone();
                if d.roles.is_empty() {
                    d.roles.insert(GoSourceSelectionRole::TypeCheckOnly);
                }
                d
            })
            .collect();
        GoSourceSelection {
            scopes,
            packages: keys.wire.packages.clone(),
            roots: roots.to_vec(),
            declarations,
            dependencies: self.dependencies.iter().cloned().collect(),
            calls: self.calls.iter().cloned().collect(),
            error_boxings: self.error_boxings.iter().cloned().collect(),
            boxed_error_types: self
                .error_boxings
                .iter()
                .map(|s| s.value_type_id().to_string())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect(),
            error_calls: self.error_calls.keys().cloned().collect(),
            body_types: self.body_types.clone(),
        }
    }
}
/// Select every possible Error target under the single source-derived case
/// universe. New body boxings and newly reached dispatch sites grow the same
/// worklist until no call/case pair remains. No branch/entry-local guess prunes U.
/// The result is a source candidate, not a capture context or runtime proof. In
/// particular an empty target table retains its receiver's non-nil obligation.
pub fn derive_go_source_error_selection(
    keys: &GoTypeKeyComponents,
    roots: &[GoSourceSelectionRef],
    entries: &[GoErrorEntryConstraint],
) -> std::result::Result<GoSourceErrorSelection, GoSourceErrorSelectionError> {
    match derive_selection(keys, roots, Some(entries), |_| {
        Ok::<_, std::convert::Infallible>(BTreeMap::new())
    }) {
        Ok(value) => Ok(value),
        Err(GoSourceModuleSelectionError::Selection(e)) => Err(e),
        Err(GoSourceModuleSelectionError::Contract(never)) => match never {},
    }
}
pub(in super::super::super) enum GoSourceModuleSelectionError<E> {
    Selection(GoSourceErrorSelectionError),
    Contract(E),
}
impl<E> From<GoSourceSelectionError> for GoSourceModuleSelectionError<E> {
    fn from(e: GoSourceSelectionError) -> Self { Self::Selection(e.into()) }
}
impl<E> From<GoSourceDomainError> for GoSourceModuleSelectionError<E> {
    fn from(e: GoSourceDomainError) -> Self { Self::Selection(e.into()) }
}
pub(in super::super::super) fn derive_go_source_error_module_selection<E, F>(
    keys: &GoTypeKeyComponents,
    roots: &[GoSourceSelectionRef],
    contracts: F,
) -> std::result::Result<GoSourceErrorSelection, GoSourceModuleSelectionError<E>>
where F: FnMut(&GoSourceSelection) -> std::result::Result<BTreeMap<GoSourceSelectionRef, Vec<GoErrorEntryConstraint>>, E> {
    derive_selection(keys, roots, None, contracts)
}
fn derive_selection<E, F>(
    keys: &GoTypeKeyComponents,
    roots: &[GoSourceSelectionRef],
    fixed_entries: Option<&[GoErrorEntryConstraint]>,
    mut contracts: F,
) -> std::result::Result<GoSourceErrorSelection, GoSourceModuleSelectionError<E>>
where F: FnMut(&GoSourceSelection) -> std::result::Result<BTreeMap<GoSourceSelectionRef, Vec<GoErrorEntryConstraint>>, E> {
    let first = roots
        .first()
        .cloned()
        .unwrap_or_else(|| GoSourceSelectionRef::new("", 0));
    let scopes = Arc::new(
        build_go_source_scopes(keys)
            .map_err(|_| fail(&first, first.node, GoSourceSelectionErrorKind::Locator, &[]))?,
    );
    let intrinsic = keys
        .wire
        .packages
        .iter()
        .filter(|p| p.import_path == "unsafe")
        .map(|p| p.id.clone())
        .collect();
    let mut builder = Selection::index(&scopes, &intrinsic);
    builder.allow_error_calls = true;
    let mut roots = roots.to_vec();
    roots.sort();
    if roots.is_empty() {
        return Err(fail(&first, first.node, GoSourceSelectionErrorKind::Locator, &[]).into());
    }
    for (i, r) in roots.iter().enumerate() {
        if i > 0 && roots[i - 1] == *r {
            return Err(fail(r, r.node, GoSourceSelectionErrorKind::DuplicateRoot, &[]).into());
        }
        if !builder
            .originals
            .get(r)
            .is_some_and(|d| d.declaration.kind == "function")
        {
            return Err(fail(r, r.node, GoSourceSelectionErrorKind::Locator, &[]).into());
        }
        builder.need(r.clone(), GoSourceSelectionRole::SelectedRoot, &[])?;
    }
    let mut cursor = 0;
    let mut seeded = BTreeSet::new();
    let mut expanded = BTreeSet::new();
    loop {
        builder.drain(&mut cursor)?;
        let selected = builder.snapshot(Arc::clone(&scopes), keys, &roots);
        let annotations = if fixed_entries.is_none() {
            contracts(&selected).map_err(GoSourceModuleSelectionError::Contract)?
        } else { BTreeMap::new() };
        let dynamic_entries = annotations.values().flatten().cloned().collect::<Vec<_>>();
        let entries = fixed_entries.unwrap_or(&dynamic_entries);
        let domain = if fixed_entries.is_some() {
            derive_go_source_value_domain(&selected, entries)?
        } else {
            super::super::domains::derive_contract_domain(&selected, &annotations)?
        };
        // A newly selected Error body can introduce another actual annotation.
        // Preserve each function-to-case dependency and its original root path.
        for row in entries {
            for case in row.cases() {
                if seeded.insert((row.root().clone(), case.clone())) {
                    let path = builder.work.iter().find(|w| w.locator == *row.root()
                        && matches!(w.role, GoSourceSelectionRole::SelectedRoot | GoSourceSelectionRole::BodyDependency))
                        .map(|w| w.path.clone()).unwrap_or_else(|| vec![row.root().clone()]);
                    builder.need(
                        case.clone(),
                        GoSourceSelectionRole::TypeDependency,
                        &path,
                    )?;
                }
            }
        }
        let sites = builder
            .error_calls
            .iter()
            .map(|(c, p)| (c.clone(), p.clone()))
            .collect::<Vec<_>>();
        for (call, path) in sites {
            for implementation in domain.error_cases() {
                if !expanded.insert((call.clone(), implementation.value_type_id().to_string())) {
                    continue;
                }
                let method = implementation.method();
                let target = GoSourceSelectionRef::new(
                    method
                        .declaration_source_object_id()
                        .expect("checked original case method"),
                    method
                        .declaration_node()
                        .expect("checked original case method"),
                );
                builder.calls.insert(GoSourceSelectionCall {
                    caller: call.caller.clone(),
                    callee: target.clone(),
                    node: call.node,
                });
                builder.need(
                    target.clone(),
                    GoSourceSelectionRole::SignatureDependency,
                    &path,
                )?;
                builder.need(target, GoSourceSelectionRole::BodyDependency, &path)?;
            }
        }
        if cursor == builder.work.len() {
            builder.cycles(&roots)?;
            let dispatches = builder
                .error_calls
                .keys()
                .map(|call| GoSourceErrorDispatch {
                    call: call.clone(),
                    targets: domain.error_cases().to_vec(),
                })
                .collect();
            return Ok(GoSourceErrorSelection { domain, dispatches });
        }
    }
}
