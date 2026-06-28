//! Global environment and declaration skeletons.

use crate::{
    CoreError, CoreErrorCode, CoreLocation, CoreLocationPart, GlobalId, Name, NameResolver, TermId,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum DefinitionReducibility {
    Reducible,
    Opaque,
}

impl DefinitionReducibility {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Reducible => "reducible",
            Self::Opaque => "opaque",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum DeclarationKind {
    Axiom {
        ty: TermId,
    },
    Definition {
        ty: TermId,
        value: TermId,
        reducibility: DefinitionReducibility,
    },
    Theorem {
        ty: TermId,
        proof: TermId,
    },
    Inductive {
        ty: TermId,
    },
    Constructor {
        ty: TermId,
        inductive: GlobalId,
    },
    Recursor {
        ty: TermId,
        inductive: GlobalId,
    },
}

impl DeclarationKind {
    pub fn ty(self) -> TermId {
        match self {
            Self::Axiom { ty }
            | Self::Definition { ty, .. }
            | Self::Theorem { ty, .. }
            | Self::Inductive { ty }
            | Self::Constructor { ty, .. }
            | Self::Recursor { ty, .. } => ty,
        }
    }

    pub fn definition_value(self) -> Option<TermId> {
        match self {
            Self::Definition { value, .. } => Some(value),
            Self::Axiom { .. }
            | Self::Theorem { .. }
            | Self::Inductive { .. }
            | Self::Constructor { .. }
            | Self::Recursor { .. } => None,
        }
    }

    pub fn theorem_proof(self) -> Option<TermId> {
        match self {
            Self::Theorem { proof, .. } => Some(proof),
            Self::Axiom { .. }
            | Self::Definition { .. }
            | Self::Inductive { .. }
            | Self::Constructor { .. }
            | Self::Recursor { .. } => None,
        }
    }

    pub fn is_reducible_definition(self) -> bool {
        matches!(
            self,
            Self::Definition {
                reducibility: DefinitionReducibility::Reducible,
                ..
            }
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Declaration {
    global: GlobalId,
    name: Name,
    kind: DeclarationKind,
}

impl Declaration {
    pub fn global(&self) -> GlobalId {
        self.global
    }

    pub fn name(&self) -> &Name {
        &self.name
    }

    pub fn kind(&self) -> DeclarationKind {
        self.kind
    }

    pub fn ty(&self) -> TermId {
        self.kind.ty()
    }
}

#[derive(Debug, Default)]
pub struct Environment {
    names: NameResolver,
    declarations: Vec<Declaration>,
}

impl Environment {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.declarations.len()
    }

    pub fn is_empty(&self) -> bool {
        self.declarations.is_empty()
    }

    pub fn names(&self) -> &NameResolver {
        &self.names
    }

    pub fn register_axiom(
        &mut self,
        name: impl AsRef<str>,
        ty: TermId,
    ) -> Result<GlobalId, CoreError> {
        self.register(name, DeclarationKind::Axiom { ty })
    }

    pub fn register_definition(
        &mut self,
        name: impl AsRef<str>,
        ty: TermId,
        value: TermId,
        reducibility: DefinitionReducibility,
    ) -> Result<GlobalId, CoreError> {
        self.register(
            name,
            DeclarationKind::Definition {
                ty,
                value,
                reducibility,
            },
        )
    }

    pub fn register_theorem(
        &mut self,
        name: impl AsRef<str>,
        ty: TermId,
        proof: TermId,
    ) -> Result<GlobalId, CoreError> {
        self.register(name, DeclarationKind::Theorem { ty, proof })
    }

    pub fn register_inductive(
        &mut self,
        name: impl AsRef<str>,
        ty: TermId,
    ) -> Result<GlobalId, CoreError> {
        self.register(name, DeclarationKind::Inductive { ty })
    }

    pub fn register_constructor(
        &mut self,
        name: impl AsRef<str>,
        ty: TermId,
        inductive: GlobalId,
    ) -> Result<GlobalId, CoreError> {
        self.validate_inductive_reference(inductive, "constructor")?;
        self.register(name, DeclarationKind::Constructor { ty, inductive })
    }

    pub fn register_recursor(
        &mut self,
        name: impl AsRef<str>,
        ty: TermId,
        inductive: GlobalId,
    ) -> Result<GlobalId, CoreError> {
        self.validate_inductive_reference(inductive, "recursor")?;
        self.register(name, DeclarationKind::Recursor { ty, inductive })
    }

    pub fn register(
        &mut self,
        name: impl AsRef<str>,
        kind: DeclarationKind,
    ) -> Result<GlobalId, CoreError> {
        let raw = name.as_ref();
        let name = Name::parse(raw).map_err(|error| invalid_name_error(raw, error.code()))?;
        self.register_name(name, kind)
    }

    pub fn register_name(
        &mut self,
        name: Name,
        kind: DeclarationKind,
    ) -> Result<GlobalId, CoreError> {
        if self.names.resolve_name(&name).is_some() {
            return Err(duplicate_declaration_error(name.as_str()));
        }

        let global = self.names.register_name(name.clone());
        if global.index() != self.declarations.len() {
            return Err(
                CoreError::new(CoreErrorCode::InternalInvariant, declarations_location())
                    .with_detail("kind", "declaration_table_misaligned")
                    .with_detail("global", global.as_u32().to_string())
                    .with_detail("len", self.declarations.len().to_string()),
            );
        }

        self.declarations.push(Declaration { global, name, kind });
        Ok(global)
    }

    pub fn resolve(&self, name: impl AsRef<str>) -> Result<Option<GlobalId>, CoreError> {
        let raw = name.as_ref();
        let name = Name::parse(raw).map_err(|error| invalid_name_error(raw, error.code()))?;
        Ok(self.names.resolve_name(&name))
    }

    pub fn lookup(&self, global: GlobalId) -> Option<&Declaration> {
        self.declarations.get(global.index())
    }

    pub fn lookup_name(&self, name: &Name) -> Option<&Declaration> {
        self.names
            .resolve_name(name)
            .and_then(|global| self.lookup(global))
    }

    pub fn lookup_by_name(&self, name: impl AsRef<str>) -> Result<Option<&Declaration>, CoreError> {
        let raw = name.as_ref();
        let name = Name::parse(raw).map_err(|error| invalid_name_error(raw, error.code()))?;
        Ok(self.lookup_name(&name))
    }

    pub fn iter(&self) -> impl Iterator<Item = &Declaration> {
        self.declarations.iter()
    }

    fn validate_inductive_reference(
        &self,
        inductive: GlobalId,
        artifact_kind: &'static str,
    ) -> Result<(), CoreError> {
        match self.lookup(inductive).map(Declaration::kind) {
            Some(DeclarationKind::Inductive { .. }) => Ok(()),
            Some(_) => Err(invalid_inductive_reference_error(
                inductive,
                artifact_kind,
                "not_inductive",
            )),
            None => Err(invalid_inductive_reference_error(
                inductive,
                artifact_kind,
                "unknown_inductive",
            )),
        }
    }
}

fn declarations_location() -> CoreLocation {
    CoreLocation::root().with_field("declarations")
}

fn duplicate_declaration_error(name: &str) -> CoreError {
    CoreError::new(CoreErrorCode::InvalidDeclaration, declarations_location())
        .with_detail("kind", "duplicate_declaration")
        .with_detail("name", name)
}

fn invalid_name_error(name: &str, name_error_code: &str) -> CoreError {
    CoreError::new(
        CoreErrorCode::InvalidName,
        CoreLocation::new([
            CoreLocationPart::field("declarations"),
            CoreLocationPart::field("name"),
        ]),
    )
    .with_detail("name", name)
    .with_detail("name_error", name_error_code)
}

fn invalid_inductive_reference_error(
    inductive: GlobalId,
    artifact_kind: &'static str,
    kind: &'static str,
) -> CoreError {
    CoreError::new(CoreErrorCode::InvalidDeclaration, declarations_location())
        .with_detail("kind", kind)
        .with_detail("artifact_kind", artifact_kind)
        .with_detail("inductive", inductive.as_u32().to_string())
}

#[cfg(test)]
mod tests {
    use crate::{
        DeclarationKind, DefinitionReducibility, Environment, LevelArena, TermArena, TermId,
    };

    fn sort(terms: &mut TermArena, levels: &mut LevelArena, name: &str) -> TermId {
        let level = levels.parse_param(name).expect("valid level param name");
        terms.sort(level)
    }

    #[test]
    fn registers_and_looks_up_axiom_declarations() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let ty = sort(&mut terms, &mut levels, "u");
        let mut env = Environment::new();

        let global = env.register_axiom("Core.Prop", ty).expect("valid axiom");
        let declaration = env.lookup(global).expect("registered declaration");

        assert_eq!(env.len(), 1);
        assert_eq!(declaration.global(), global);
        assert_eq!(declaration.name().as_str(), "Core.Prop");
        assert_eq!(declaration.kind(), DeclarationKind::Axiom { ty });
        assert_eq!(declaration.ty(), ty);
        assert_eq!(env.resolve("Core.Prop").unwrap(), Some(global));
        assert_eq!(env.lookup_by_name("Core.Prop").unwrap(), Some(declaration));
    }

    #[test]
    fn registers_definition_with_reducibility_metadata() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let ty = sort(&mut terms, &mut levels, "u");
        let value = terms.var(0);
        let mut env = Environment::new();

        let global = env
            .register_definition("Core.id", ty, value, DefinitionReducibility::Reducible)
            .expect("valid definition");
        let declaration = env.lookup(global).expect("registered declaration");

        assert_eq!(
            declaration.kind(),
            DeclarationKind::Definition {
                ty,
                value,
                reducibility: DefinitionReducibility::Reducible,
            }
        );
        assert_eq!(declaration.kind().definition_value(), Some(value));
        assert!(declaration.kind().is_reducible_definition());
        assert_eq!(DefinitionReducibility::Opaque.as_str(), "opaque");
    }

    #[test]
    fn registers_theorem_as_opaque_declaration_skeleton() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let ty = sort(&mut terms, &mut levels, "u");
        let proof = terms.var(0);
        let mut env = Environment::new();

        let global = env
            .register_theorem("Core.id_eq", ty, proof)
            .expect("valid theorem");
        let declaration = env.lookup(global).expect("registered declaration");

        assert_eq!(declaration.name().as_str(), "Core.id_eq");
        assert_eq!(declaration.kind(), DeclarationKind::Theorem { ty, proof });
        assert_eq!(declaration.kind().theorem_proof(), Some(proof));
        assert!(!declaration.kind().is_reducible_definition());
    }

    #[test]
    fn registers_constructor_and_recursor_against_inductive() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let ty = sort(&mut terms, &mut levels, "u");
        let mut env = Environment::new();

        let inductive = env
            .register_inductive("Std.Bool", ty)
            .expect("valid inductive");
        let constructor = env
            .register_constructor("Std.Bool.true", ty, inductive)
            .expect("valid constructor");
        let recursor = env
            .register_recursor("Std.Bool.rec", ty, inductive)
            .expect("valid recursor");

        assert_eq!(
            env.lookup(constructor).expect("constructor").kind(),
            DeclarationKind::Constructor { ty, inductive }
        );
        assert_eq!(
            env.lookup(recursor).expect("recursor").kind(),
            DeclarationKind::Recursor { ty, inductive }
        );
        assert_eq!(env.lookup(inductive).expect("inductive").ty(), ty);
    }

    #[test]
    fn constructor_rejects_non_inductive_dependency() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let ty = sort(&mut terms, &mut levels, "u");
        let mut env = Environment::new();

        let axiom = env.register_axiom("Core.A", ty).expect("valid axiom");
        let error = env
            .register_constructor("Core.A.mk", ty, axiom)
            .unwrap_err();

        assert_eq!(
            error.to_deterministic_json(),
            "{\"code\":\"CORE_INVALID_DECLARATION\",\"location\":[{\"field\":\"declarations\"}],\"details\":{\"artifact_kind\":\"constructor\",\"inductive\":\"0\",\"kind\":\"not_inductive\"}}"
        );
    }

    #[test]
    fn duplicate_declarations_reject_deterministically() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let ty = sort(&mut terms, &mut levels, "u");
        let mut env = Environment::new();

        env.register_axiom("Core.Prop", ty).expect("valid axiom");
        let error = env.register_axiom("Core.Prop", ty).unwrap_err();

        assert_eq!(
            error.to_deterministic_json(),
            "{\"code\":\"CORE_INVALID_DECLARATION\",\"location\":[{\"field\":\"declarations\"}],\"details\":{\"kind\":\"duplicate_declaration\",\"name\":\"Core.Prop\"}}"
        );
    }

    #[test]
    fn invalid_declaration_names_reject_deterministically() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let ty = sort(&mut terms, &mut levels, "u");
        let mut env = Environment::new();

        let error = env.register_axiom("Core.", ty).unwrap_err();

        assert_eq!(
            error.to_deterministic_json(),
            "{\"code\":\"CORE_INVALID_NAME\",\"location\":[{\"field\":\"declarations\"},{\"field\":\"name\"}],\"details\":{\"name\":\"Core.\",\"name_error\":\"EMPTY_COMPONENT\"}}"
        );
    }

    #[test]
    fn declarations_iterate_in_registration_order() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let ty = sort(&mut terms, &mut levels, "u");
        let value = terms.var(0);
        let mut env = Environment::new();

        env.register_axiom("Core.A", ty).expect("valid axiom");
        env.register_definition("Core.B", ty, value, DefinitionReducibility::Opaque)
            .expect("valid definition");
        env.register_theorem("Core.C", ty, value)
            .expect("valid theorem");

        let names: Vec<_> = env
            .iter()
            .map(|declaration| declaration.name().as_str())
            .collect();

        assert_eq!(names, ["Core.A", "Core.B", "Core.C"]);
        assert_eq!(env.names().len(), env.len());
    }
}
