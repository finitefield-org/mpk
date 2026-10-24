//! Complete original body/contract candidates for an explicit root set. Raw
//! annotations are optional and retain their actual captured identities. This
//! does not produce canonical capture/scope/context wire or a program proof.
use super::*;
use expressions::{error, overlay_error};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GoSourceContractOrigin {
    Default,
    SourceOverlay { object_id: String },
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GoSourceLoopCandidateOrigin {
    Supplied,
    GeneratedInvariant,
    GeneratedRange,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoSourceFunctionLoopContract {
    typing: GoTypedOverlayLoop,
    invariant_origin: GoSourceLoopCandidateOrigin,
    variant_origin: Option<GoSourceLoopCandidateOrigin>,
    pending_total_variant: bool,
}
impl GoSourceFunctionLoopContract {
    pub fn typing(&self) -> &GoTypedOverlayLoop {
        &self.typing
    }
    pub fn invariant_origin(&self) -> GoSourceLoopCandidateOrigin {
        self.invariant_origin
    }
    pub fn variant_origin(&self) -> Option<GoSourceLoopCandidateOrigin> {
        self.variant_origin
    }
    pub fn pending_total_variant(&self) -> bool {
        self.pending_total_variant
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoSourceFunctionContract {
    function: GoSourceSelectionRef,
    signature: GoSourceSignature,
    selected_root: bool,
    origin: GoSourceContractOrigin,
    termination: String,
    requires: Vec<GoContractExpression>,
    ensures: Vec<GoContractExpression>,
    entries: Vec<GoBoundErrorEntry>,
    loops: Vec<GoSourceFunctionLoopContract>,
}
impl GoSourceFunctionContract {
    pub fn function(&self) -> &GoSourceSelectionRef {
        &self.function
    }
    pub fn signature(&self) -> &GoSourceSignature {
        &self.signature
    }
    pub fn is_selected_root(&self) -> bool {
        self.selected_root
    }
    pub fn origin(&self) -> &GoSourceContractOrigin {
        &self.origin
    }
    pub fn overlay_object_id(&self) -> Option<&str> {
        match &self.origin {
            GoSourceContractOrigin::Default => None,
            GoSourceContractOrigin::SourceOverlay { object_id } => Some(object_id),
        }
    }
    pub fn termination(&self) -> &str {
        &self.termination
    }
    pub fn requires(&self) -> &[GoContractExpression] {
        &self.requires
    }
    pub fn ensures(&self) -> &[GoContractExpression] {
        &self.ensures
    }
    pub fn error_inputs(&self) -> &[GoBoundErrorEntry] {
        &self.entries
    }
    pub fn loops(&self) -> &[GoSourceFunctionLoopContract] {
        &self.loops
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoCapturedFunctionOverlay {
    function: GoSourceSelectionRef,
    overlay: GoSourceOverlay,
    body: bool,
}
impl GoCapturedFunctionOverlay {
    pub fn function(&self) -> &GoSourceSelectionRef {
        &self.function
    }
    pub fn overlay(&self) -> &GoSourceOverlay {
        &self.overlay
    }
    pub fn in_body_closure(&self) -> bool {
        self.body
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoSourceContractModule {
    annotations: GoTypedSourceOverlaySet,
    functions: Vec<GoSourceFunctionContract>,
    captured: Vec<GoCapturedFunctionOverlay>,
    expression_nodes: usize,
}
impl GoSourceContractModule {
    pub fn selected_roots(&self) -> &[GoSourceSelectionRef] {
        self.selection().roots()
    }
    pub fn selection(&self) -> &GoSourceSelection {
        self.annotations.value_domain().selection()
    }
    pub fn finite_selection(&self) -> &GoSourceErrorSelection {
        self.annotations.source_selection().selection()
    }
    pub fn value_domain(&self) -> &GoSourceValueDomain {
        self.annotations.value_domain()
    }
    pub fn functions(&self) -> &[GoSourceFunctionContract] {
        &self.functions
    }
    /// Captured unselected annotations remain raw inputs, never checked clauses.
    pub fn captured_overlays(&self) -> &[GoCapturedFunctionOverlay] {
        &self.captured
    }
    pub fn annotation_typing(&self) -> &GoTypedSourceOverlaySet {
        &self.annotations
    }
    pub fn expression_nodes(&self) -> usize {
        self.expression_nodes
    }
}
fn body(d: &GoSourceSelectionDeclaration) -> bool {
    d.kind() == "function"
        && d.roles().any(|r| {
            matches!(
                r,
                GoSourceSelectionRole::SelectedRoot | GoSourceSelectionRole::BodyDependency
            )
        })
}
fn annotate_error(e: GoSourceOverlayError, overlay: &GoSourceOverlay) -> GoSourceOverlayError {
    GoSourceOverlayError {
        overlay: Some(overlay.raw.object().id.clone()),
        ..e
    }
}
/// Derive all original selected body contracts together. Each supplied sidecar
/// is an owned capture input; only annotations on reached bodies seed U or type
/// clauses. Defaults explicitly constrain each error path to the resulting U.
/// Roots and body dependencies keep different original selection roles.
pub fn derive_go_source_contract_module(
    keys: &GoTypeKeyComponents,
    roots: &[GoSourceFunctionSelector],
    overlays: &[GoSourceOverlay],
) -> std::result::Result<GoSourceContractModule, GoContractTypingError> {
    derive_contract_module(keys, Roots::Named(roots), overlays)
}

pub(in super::super) fn derive_go_source_contract_module_at(
    keys: &GoTypeKeyComponents,
    roots: &[GoSourceSelectionRef],
    overlays: &[GoSourceOverlay],
) -> std::result::Result<GoSourceContractModule, GoContractTypingError> {
    derive_contract_module(keys, Roots::Physical(roots), overlays)
}

enum Roots<'a> {
    Named(&'a [GoSourceFunctionSelector]),
    Physical(&'a [GoSourceSelectionRef]),
}

fn derive_contract_module(
    keys: &GoTypeKeyComponents,
    roots: Roots<'_>,
    overlays: &[GoSourceOverlay],
) -> std::result::Result<GoSourceContractModule, GoContractTypingError> {
    let root_count = match roots {
        Roots::Named(r) => r.len(),
        Roots::Physical(r) => r.len(),
    };
    // Frozen capture candidate/input count and aggregate contract-byte limits.
    if root_count > 8192
        || overlays.len().saturating_add(keys.sources.len()) > 32768
        || overlays
            .iter()
            .try_fold(0usize, |n, o| n.checked_add(o.raw.bytes().len()))
            .is_none_or(|n| n > 8_388_608)
    {
        return Err(error(GoContractTypingErrorKind::Limit));
    }
    let scopes = Arc::new(
        build_go_source_scopes(keys)
            .map_err(|e| overlay_error(failure(GoSourceOverlayErrorKind::Scope(e))))?,
    );
    let binder = Binder { keys, scopes };
    let root_refs = match roots {
        Roots::Named(roots) => roots
            .iter()
            .map(|r| binder.resolve(r).map_err(overlay_error))
            .collect::<std::result::Result<Vec<_>, _>>()?,
        Roots::Physical(roots) => roots.to_vec(),
    };
    let mut captured = BTreeMap::new();
    let mut paths = BTreeSet::new();
    for overlay in overlays {
        if !paths.insert((
            overlay.raw.object().capability.clone(),
            overlay.raw.object().path.clone(),
        )) || keys.sources.values().any(|s| {
            s.object().capability == overlay.raw.object().capability
                && s.object().path == overlay.raw.object().path
        }) {
            return Err(overlay_error(annotate_error(
                failure(GoSourceOverlayErrorKind::DuplicateOverlay),
                overlay,
            )));
        }
        let r = binder
            .function(overlay)
            .map_err(|e| overlay_error(annotate_error(e, overlay)))?;
        if captured.insert(r, overlay.clone()).is_some() {
            return Err(overlay_error(annotate_error(
                failure(GoSourceOverlayErrorKind::DuplicateOverlay),
                overlay,
            )));
        }
    }
    let mut bound = BTreeMap::new();
    let selection = source_scalar_typing::derive_go_source_error_module_selection(
        keys,
        &root_refs,
        |selected| {
            let mut count = 0;
            for d in selected.declarations().iter().filter(|d| body(d)) {
                count += 1;
                if count > 8192 {
                    return Err(failure(GoSourceOverlayErrorKind::Limit));
                }
                if let Some(overlay) = captured.get(d.locator()) {
                    if !bound.contains_key(d.locator()) {
                        let original = binder
                            .bind(overlay)
                            .map_err(|e| annotate_error(e, overlay))?;
                        bound.insert(d.locator().clone(), original);
                    }
                }
            }
            Ok(bound
                .iter()
                .map(|(r, b)| (r.clone(), b.entries.clone()))
                .collect())
        },
    )
    .map_err(|e| match e {
        source_scalar_typing::GoSourceModuleSelectionError::Selection(e) => {
            overlay_error(failure(GoSourceOverlayErrorKind::Selection(Box::new(e))))
        }
        source_scalar_typing::GoSourceModuleSelectionError::Contract(e) => overlay_error(e),
    })?;
    let original = GoSourceOverlaySelection {
        overlays: bound.into_values().collect(),
        selection,
    };
    let annotations = expressions::type_source_overlays(keys, original)?;
    let typed = annotations
        .overlays()
        .iter()
        .map(|t| (t.root(), t))
        .collect::<BTreeMap<_, _>>();
    let bound = annotations
        .source_selection()
        .overlays()
        .iter()
        .map(|b| (b.root(), b))
        .collect::<BTreeMap<_, _>>();
    let domain = annotations.value_domain();
    let mut functions = vec![];
    let mut expression_nodes = 0usize;
    for d in domain.selection().declarations().iter().filter(|d| body(d)) {
        let r = d.locator();
        let supplied = typed.get(r).copied();
        let signature = if let Some(b) = bound.get(r) {
            b.signature.clone()
        } else {
            source_signatures::derive_shared_signature(
                &binder.scopes,
                r.source_object_id(),
                r.node(),
            )
            .map_err(|e| overlay_error(failure(GoSourceOverlayErrorKind::Signature(e))))?
        };
        let original_loops = if let Some(b) = bound.get(r) {
            b.loops.clone()
        } else {
            original_loops(&binder.scopes, r, &signature).map_err(overlay_error)?
        };
        let headers =
            expressions::header_bindings(&binder.scopes, r, &signature, &original_loops, domain)?;
        let overlay = captured.get(r);
        let termination = overlay.map_or("partial", |o| o.termination());
        let total = termination == "total";
        let requires = supplied.map_or_else(Vec::new, |t| t.requires().to_vec());
        let ensures = supplied.map_or_else(
            || vec![expressions::true_candidate()],
            |t| t.ensures().to_vec(),
        );
        let mut loops = vec![];
        for l in original_loops {
            let previous = supplied.map(|t| &t.loops()[l.ordinal]);
            let range = binder.scopes.source(r.source_object_id()).unwrap().nodes()[l.node].kind()
                == "RangeStmt";
            let invariant_origin = if previous.is_some_and(|p| p.annotation_row().is_some()) {
                GoSourceLoopCandidateOrigin::Supplied
            } else {
                GoSourceLoopCandidateOrigin::GeneratedInvariant
            };
            let variant_origin = if previous.is_some_and(|p| p.variant().is_some()) {
                Some(GoSourceLoopCandidateOrigin::Supplied)
            } else if total && range {
                Some(GoSourceLoopCandidateOrigin::GeneratedRange)
            } else {
                None
            };
            let typing = expressions::complete_loop(
                l.node,
                l.ordinal,
                headers[&l.node].clone(),
                previous,
                total,
                range,
            );
            loops.push(GoSourceFunctionLoopContract {
                pending_total_variant: total && typing.variant().is_none(),
                typing,
                invariant_origin,
                variant_origin,
            });
        }
        let function_start = expression_nodes;
        let mut work = requires.iter().chain(&ensures).collect::<Vec<_>>();
        for l in &loops {
            work.extend(l.typing.invariants());
            work.extend(l.typing.variant());
        }
        while let Some(e) = work.pop() {
            expression_nodes += 1;
            if expression_nodes > 8192 || expression_nodes - function_start > 1024 {
                return Err(error(GoContractTypingErrorKind::Limit));
            }
            work.extend(e.children());
        }
        functions.push(GoSourceFunctionContract {
            function: r.clone(),
            signature,
            selected_root: domain.selection().roots().contains(r),
            origin: overlay.map_or(GoSourceContractOrigin::Default, |o| {
                GoSourceContractOrigin::SourceOverlay {
                    object_id: o.raw.object().id.clone(),
                }
            }),
            termination: termination.into(),
            requires,
            ensures,
            entries: domain
                .entries()
                .iter()
                .filter(|e| e.root() == r)
                .cloned()
                .collect(),
            loops,
        });
    }
    functions.sort_by(|a, b| a.function.cmp(&b.function));
    let bodies = functions
        .iter()
        .map(|f| &f.function)
        .collect::<BTreeSet<_>>();
    let captured = captured
        .into_iter()
        .map(|(function, overlay)| {
            let body = bodies.contains(&function);
            GoCapturedFunctionOverlay {
                function,
                overlay,
                body,
            }
        })
        .collect();
    Ok(GoSourceContractModule {
        annotations,
        functions,
        captured,
        expression_nodes,
    })
}
