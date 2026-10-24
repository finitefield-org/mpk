//! Complete source-only candidates from the original-package producer. This
//! boundary admits no native type graph, declaration adapter or call facts.
//! Captured context authority and source/program proofs remain separate.
use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoSourceInputs {
    components: GoTypeKeyComponents,
}

impl GoSourceInputs {
    pub fn languages(&self) -> std::result::Result<GoSourceLanguages, GoSourceLanguageError> {
        derive_go_source_languages(self)
    }
    pub fn components(&self) -> &GoTypeKeyComponents {
        &self.components
    }

    pub fn source_scopes(&self) -> std::result::Result<GoSourceScopes, GoSourceScopeError> {
        build_go_source_scopes(&self.components)
    }
}

/// Import the complete package/source set, using only the fixed logical Bool
/// seed. Actual source types are derived later from the owned source bytes;
/// their number and shape are not restricted by this seed-only wire phase.
/// This does not authenticate a launcher, compiler image or installed context.
pub fn import_go_source_inputs(
    record: &[u8],
    sources: &[GoTypeSourceBuffer],
) -> Result<GoSourceInputs> {
    let components = import_go_type_key_components(record, sources)?;
    let wire = &components.wire;
    let [GoSourceTypeKey::Builtin {
        id,
        name,
        width: None,
        signed: None,
        ..
    }] = wire.types.as_slice()
    else {
        return Err(GoTypeKeyError::Shape);
    };
    if name != "bool"
        || wire.type_roots.as_slice() != std::slice::from_ref(id)
        || !wire.declarations.is_empty()
        || !wire.underlying.is_empty()
        || !wire.aliases.is_empty()
        || wire.package_roots
            != wire
                .packages
                .iter()
                .map(|p| p.id.clone())
                .collect::<Vec<_>>()
    {
        return Err(GoTypeKeyError::Shape);
    }
    let complete_sources = wire
        .packages
        .iter()
        .flat_map(|package| {
            std::iter::once(&package.module_source_object_id).chain(package.source_objects.iter())
        })
        .collect::<BTreeSet<_>>();
    if complete_sources
        != wire
            .source_objects
            .iter()
            .map(|s| &s.id)
            .collect::<BTreeSet<_>>()
    {
        return Err(GoTypeKeyError::Source);
    }
    Ok(GoSourceInputs { components })
}
