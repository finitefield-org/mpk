//! MVP inductive declaration metadata, registration, and export.

use std::collections::BTreeSet;

use crate::{
    CoreError, CoreErrorCode, CoreLocation, DeclarationKind, Environment, GlobalId, LevelArena,
    LevelId, LevelNode, Name, NameError, TermArena, TermId,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum MvpInductiveShape {
    Bool,
    Nat,
    Eq,
}

impl MvpInductiveShape {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Bool => "Bool",
            Self::Nat => "Nat",
            Self::Eq => "Eq",
        }
    }

    fn constructor_count(self) -> usize {
        match self {
            Self::Bool | Self::Nat => 2,
            Self::Eq => 1,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InductiveSpec {
    pub shape: MvpInductiveShape,
    pub family_name: String,
    pub universe_params: Vec<LevelId>,
    pub ty: TermId,
    pub constructors: Vec<ConstructorSignature>,
    pub recursor: RecursorSignature,
}

impl InductiveSpec {
    pub fn new(
        shape: MvpInductiveShape,
        family_name: impl Into<String>,
        universe_params: Vec<LevelId>,
        ty: TermId,
        constructors: Vec<ConstructorSignature>,
        recursor: RecursorSignature,
    ) -> Self {
        Self {
            shape,
            family_name: family_name.into(),
            universe_params,
            ty,
            constructors,
            recursor,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConstructorSignature {
    pub name: String,
    pub ty: TermId,
}

impl ConstructorSignature {
    pub fn new(name: impl Into<String>, ty: TermId) -> Self {
        Self {
            name: name.into(),
            ty,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursorSignature {
    pub name: String,
    pub ty: TermId,
}

impl RecursorSignature {
    pub fn new(name: impl Into<String>, ty: TermId) -> Self {
        Self {
            name: name.into(),
            ty,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegisteredInductive {
    pub shape: MvpInductiveShape,
    pub universe_params: Vec<LevelId>,
    pub family: GlobalId,
    pub constructors: Vec<GlobalId>,
    pub recursor: GlobalId,
}

impl RegisteredInductive {
    pub fn export(&self, env: &Environment) -> Result<ExportedInductive, CoreError> {
        export_registered_inductive(env, self)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExportedInductive {
    pub shape: MvpInductiveShape,
    pub universe_params: Vec<LevelId>,
    pub family: ExportedInductiveDeclaration,
    pub constructors: Vec<ExportedInductiveDeclaration>,
    pub recursor: ExportedInductiveDeclaration,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExportedInductiveDeclaration {
    pub global: GlobalId,
    pub name: Name,
    pub ty: TermId,
}

pub fn register_mvp_inductive(
    levels: &LevelArena,
    terms: &TermArena,
    env: &mut Environment,
    spec: InductiveSpec,
) -> Result<RegisteredInductive, CoreError> {
    validate_mvp_inductive(levels, terms, env, &spec)?;

    let family = env.register_inductive(&spec.family_name, spec.ty)?;
    let mut constructors = Vec::with_capacity(spec.constructors.len());
    for constructor in &spec.constructors {
        constructors.push(env.register_constructor(&constructor.name, constructor.ty, family)?);
    }
    let recursor = env.register_recursor(&spec.recursor.name, spec.recursor.ty, family)?;

    Ok(RegisteredInductive {
        shape: spec.shape,
        universe_params: spec.universe_params,
        family,
        constructors,
        recursor,
    })
}

pub fn export_registered_inductive(
    env: &Environment,
    registered: &RegisteredInductive,
) -> Result<ExportedInductive, CoreError> {
    let family = export_family(env, registered.family)?;

    let mut constructors = Vec::with_capacity(registered.constructors.len());
    for constructor in &registered.constructors {
        constructors.push(export_artifact(
            env,
            *constructor,
            registered.family,
            "constructor",
        )?);
    }

    let recursor = export_artifact(env, registered.recursor, registered.family, "recursor")?;

    Ok(ExportedInductive {
        shape: registered.shape,
        universe_params: registered.universe_params.clone(),
        family,
        constructors,
        recursor,
    })
}

fn validate_mvp_inductive(
    levels: &LevelArena,
    terms: &TermArena,
    env: &Environment,
    spec: &InductiveSpec,
) -> Result<(), CoreError> {
    validate_constructor_count(spec)?;
    validate_universe_params(levels, &spec.universe_params)?;
    validate_names(env, spec)?;
    validate_term_dependencies(terms, spec.ty, inductive_location().with_field("type"))?;

    for (index, constructor) in spec.constructors.iter().enumerate() {
        validate_term_dependencies(
            terms,
            constructor.ty,
            inductive_location()
                .with_field("constructors")
                .with_index(index_as_u32(index)?)
                .with_field("type"),
        )?;
    }

    validate_term_dependencies(
        terms,
        spec.recursor.ty,
        inductive_location()
            .with_field("recursor")
            .with_field("type"),
    )
}

fn validate_constructor_count(spec: &InductiveSpec) -> Result<(), CoreError> {
    let expected = spec.shape.constructor_count();
    if spec.constructors.len() == expected {
        return Ok(());
    }

    Err(CoreError::new(
        CoreErrorCode::InvalidDeclaration,
        inductive_location().with_field("constructors"),
    )
    .with_detail("kind", "constructor_count_mismatch")
    .with_detail("shape", spec.shape.as_str())
    .with_detail("expected", expected.to_string())
    .with_detail("actual", spec.constructors.len().to_string()))
}

fn validate_universe_params(
    levels: &LevelArena,
    universe_params: &[LevelId],
) -> Result<(), CoreError> {
    let mut seen = BTreeSet::new();
    for (index, param) in universe_params.iter().copied().enumerate() {
        let location = inductive_location()
            .with_field("universe_params")
            .with_index(index_as_u32(index)?);

        validate_level_reference(levels, param, location.clone())?;
        if !matches!(levels.node(param), LevelNode::Param(_)) {
            return Err(CoreError::new(CoreErrorCode::InvalidDeclaration, location)
                .with_detail("kind", "universe_param_not_param")
                .with_detail("level", param.index().to_string()));
        }
        if !seen.insert(param) {
            return Err(CoreError::new(CoreErrorCode::InvalidDeclaration, location)
                .with_detail("kind", "duplicate_universe_param")
                .with_detail("level", param.index().to_string()));
        }
    }

    Ok(())
}

fn validate_level_reference(
    levels: &LevelArena,
    level: LevelId,
    location: CoreLocation,
) -> Result<(), CoreError> {
    if level.index() < levels.len() {
        return Ok(());
    }

    Err(
        CoreError::new(CoreErrorCode::InvalidLevelReference, location)
            .with_detail("kind", "unknown_level")
            .with_detail("level", level.index().to_string())
            .with_detail("arena_len", levels.len().to_string()),
    )
}

fn validate_names(env: &Environment, spec: &InductiveSpec) -> Result<(), CoreError> {
    let mut seen = BTreeSet::new();
    validate_name(
        env,
        &mut seen,
        &spec.family_name,
        inductive_location().with_field("family_name"),
    )?;

    for (index, constructor) in spec.constructors.iter().enumerate() {
        validate_name(
            env,
            &mut seen,
            &constructor.name,
            inductive_location()
                .with_field("constructors")
                .with_index(index_as_u32(index)?)
                .with_field("name"),
        )?;
    }

    validate_name(
        env,
        &mut seen,
        &spec.recursor.name,
        inductive_location()
            .with_field("recursor")
            .with_field("name"),
    )
}

fn validate_name(
    env: &Environment,
    seen: &mut BTreeSet<Name>,
    raw: &str,
    location: CoreLocation,
) -> Result<(), CoreError> {
    let name =
        Name::parse(raw).map_err(|error| invalid_name_error(raw, &error, location.clone()))?;
    if !seen.insert(name.clone()) {
        return Err(CoreError::new(CoreErrorCode::InvalidDeclaration, location)
            .with_detail("kind", "duplicate_inductive_artifact_name")
            .with_detail("name", raw));
    }
    if env.lookup_name(&name).is_some() {
        return Err(CoreError::new(CoreErrorCode::InvalidDeclaration, location)
            .with_detail("kind", "duplicate_declaration")
            .with_detail("name", raw));
    }

    Ok(())
}

fn validate_term_dependencies(
    terms: &TermArena,
    term: TermId,
    location: CoreLocation,
) -> Result<(), CoreError> {
    validate_term_reference(terms, term, location.clone())?;
    let mut stack = vec![term];
    let mut seen = BTreeSet::new();

    while let Some(current) = stack.pop() {
        validate_term_reference(terms, current, location.clone())?;
        if !seen.insert(current) {
            continue;
        }

        for dependency in terms.dependencies(current) {
            validate_term_reference(terms, dependency, location.clone())?;
            if dependency.index() >= current.index() {
                return Err(CoreError::new(CoreErrorCode::InvalidDeclaration, location)
                    .with_detail("kind", "non_topological_term_dependency")
                    .with_detail("term", current.index().to_string())
                    .with_detail("dependency", dependency.index().to_string()));
            }
            stack.push(dependency);
        }
    }

    Ok(())
}

fn validate_term_reference(
    terms: &TermArena,
    term: TermId,
    location: CoreLocation,
) -> Result<(), CoreError> {
    if term.index() < terms.len() {
        return Ok(());
    }

    Err(
        CoreError::new(CoreErrorCode::InvalidTermReference, location)
            .with_detail("kind", "unknown_term")
            .with_detail("term", term.index().to_string())
            .with_detail("arena_len", terms.len().to_string()),
    )
}

fn export_family(
    env: &Environment,
    global: GlobalId,
) -> Result<ExportedInductiveDeclaration, CoreError> {
    let declaration = lookup_export_declaration(env, global, "family")?;
    match declaration.kind() {
        DeclarationKind::Inductive { ty } => Ok(ExportedInductiveDeclaration {
            global,
            name: declaration.name().clone(),
            ty,
        }),
        kind => Err(export_kind_error(
            global,
            "inductive",
            declaration_kind_tag(kind),
        )),
    }
}

fn export_artifact(
    env: &Environment,
    global: GlobalId,
    inductive: GlobalId,
    artifact_kind: &'static str,
) -> Result<ExportedInductiveDeclaration, CoreError> {
    let declaration = lookup_export_declaration(env, global, artifact_kind)?;
    match declaration.kind() {
        DeclarationKind::Constructor {
            ty,
            inductive: actual,
            ..
        } if artifact_kind == "constructor" && actual == inductive => {
            Ok(ExportedInductiveDeclaration {
                global,
                name: declaration.name().clone(),
                ty,
            })
        }
        DeclarationKind::Recursor {
            ty,
            inductive: actual,
            ..
        } if artifact_kind == "recursor" && actual == inductive => {
            Ok(ExportedInductiveDeclaration {
                global,
                name: declaration.name().clone(),
                ty,
            })
        }
        DeclarationKind::Constructor {
            inductive: actual, ..
        }
        | DeclarationKind::Recursor {
            inductive: actual, ..
        } if actual != inductive => Err(CoreError::new(
            CoreErrorCode::InvalidDeclaration,
            inductive_location().with_field("export"),
        )
        .with_detail("kind", "artifact_inductive_mismatch")
        .with_detail("artifact_kind", artifact_kind)
        .with_detail("global", global.as_u32().to_string())
        .with_detail("expected_inductive", inductive.as_u32().to_string())
        .with_detail("actual_inductive", actual.as_u32().to_string())),
        kind => Err(export_kind_error(
            global,
            artifact_kind,
            declaration_kind_tag(kind),
        )),
    }
}

fn lookup_export_declaration<'a>(
    env: &'a Environment,
    global: GlobalId,
    export_kind: &'static str,
) -> Result<&'a crate::Declaration, CoreError> {
    env.lookup(global).ok_or_else(|| {
        CoreError::new(
            CoreErrorCode::InvalidDeclaration,
            inductive_location().with_field("export"),
        )
        .with_detail("kind", "missing_export_declaration")
        .with_detail("export_kind", export_kind)
        .with_detail("global", global.as_u32().to_string())
    })
}

fn export_kind_error(global: GlobalId, expected: &'static str, actual: &'static str) -> CoreError {
    CoreError::new(
        CoreErrorCode::InvalidDeclaration,
        inductive_location().with_field("export"),
    )
    .with_detail("kind", "export_kind_mismatch")
    .with_detail("global", global.as_u32().to_string())
    .with_detail("expected", expected)
    .with_detail("actual", actual)
}

fn declaration_kind_tag(kind: DeclarationKind) -> &'static str {
    match kind {
        DeclarationKind::Axiom { .. } => "axiom",
        DeclarationKind::Definition { .. } => "definition",
        DeclarationKind::Theorem { .. } => "theorem",
        DeclarationKind::Inductive { .. } => "inductive",
        DeclarationKind::Constructor { .. } => "constructor",
        DeclarationKind::Recursor { .. } => "recursor",
    }
}

fn invalid_name_error(raw: &str, error: &NameError, location: CoreLocation) -> CoreError {
    CoreError::new(CoreErrorCode::InvalidName, location)
        .with_detail("name", raw)
        .with_detail("name_error", error.code())
}

fn inductive_location() -> CoreLocation {
    CoreLocation::root().with_field("inductive")
}

fn index_as_u32(index: usize) -> Result<u32, CoreError> {
    u32::try_from(index).map_err(|_| {
        CoreError::new(CoreErrorCode::InternalInvariant, inductive_location())
            .with_detail("kind", "index_overflow")
            .with_detail("index", index.to_string())
    })
}

#[cfg(test)]
mod tests {
    use crate::{
        register_mvp_inductive, ConstructorSignature, CoreErrorCode, DeclarationKind, Environment,
        InductiveSpec, LevelArena, MvpInductiveShape, RecursorSignature, TermArena, TermId,
    };

    fn sort(terms: &mut TermArena, levels: &mut LevelArena, name: &str) -> TermId {
        let level = levels.parse_param(name).expect("valid level param name");
        terms.sort(level)
    }

    fn basic_spec(
        shape: MvpInductiveShape,
        family_name: &str,
        ty: TermId,
        constructor_names: &[&str],
        recursor_name: &str,
    ) -> InductiveSpec {
        InductiveSpec::new(
            shape,
            family_name,
            Vec::new(),
            ty,
            constructor_names
                .iter()
                .map(|name| ConstructorSignature::new(*name, ty))
                .collect(),
            RecursorSignature::new(recursor_name, ty),
        )
    }

    #[test]
    fn registers_and_exports_bool_shape_in_dependency_order() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let ty = sort(&mut terms, &mut levels, "u");
        let mut env = Environment::new();

        let registered = register_mvp_inductive(
            &levels,
            &terms,
            &mut env,
            basic_spec(
                MvpInductiveShape::Bool,
                "Std.Bool",
                ty,
                &["Std.Bool.false", "Std.Bool.true"],
                "Std.Bool.rec",
            ),
        )
        .expect("Bool registers");
        let exported = registered.export(&env).expect("Bool exports");

        assert_eq!(exported.shape, MvpInductiveShape::Bool);
        assert_eq!(exported.family.name.as_str(), "Std.Bool");
        assert_eq!(
            exported
                .constructors
                .iter()
                .map(|constructor| constructor.name.as_str())
                .collect::<Vec<_>>(),
            ["Std.Bool.false", "Std.Bool.true"]
        );
        assert_eq!(exported.recursor.name.as_str(), "Std.Bool.rec");
        assert!(registered.family < registered.constructors[0]);
        assert!(registered.constructors[0] < registered.constructors[1]);
        assert!(registered.constructors[1] < registered.recursor);
        assert_eq!(
            env.lookup(registered.family).expect("family").kind(),
            DeclarationKind::Inductive { ty }
        );
        assert_eq!(
            env.lookup(registered.constructors[0])
                .expect("constructor")
                .kind(),
            DeclarationKind::Constructor {
                ty,
                inductive: registered.family,
                generated: false,
            }
        );
    }

    #[test]
    fn registers_and_exports_nat_shape() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let ty = sort(&mut terms, &mut levels, "u");
        let mut env = Environment::new();

        let registered = register_mvp_inductive(
            &levels,
            &terms,
            &mut env,
            basic_spec(
                MvpInductiveShape::Nat,
                "Std.Nat",
                ty,
                &["Std.Nat.zero", "Std.Nat.succ"],
                "Std.Nat.rec",
            ),
        )
        .expect("Nat registers");
        let exported = registered.export(&env).expect("Nat exports");

        assert_eq!(exported.shape, MvpInductiveShape::Nat);
        assert_eq!(
            exported
                .constructors
                .iter()
                .map(|constructor| constructor.name.as_str())
                .collect::<Vec<_>>(),
            ["Std.Nat.zero", "Std.Nat.succ"]
        );
        assert_eq!(exported.recursor.ty, ty);
    }

    #[test]
    fn registers_and_exports_eq_shape_with_universe_params() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let universe = levels.parse_param("u").expect("valid level param name");
        let ty = terms.sort(universe);
        let mut env = Environment::new();

        let registered = register_mvp_inductive(
            &levels,
            &terms,
            &mut env,
            InductiveSpec::new(
                MvpInductiveShape::Eq,
                "Std.Eq",
                vec![universe],
                ty,
                vec![ConstructorSignature::new("Std.Eq.refl", ty)],
                RecursorSignature::new("Std.Eq.rec", ty),
            ),
        )
        .expect("Eq registers");
        let exported = registered.export(&env).expect("Eq exports");

        assert_eq!(exported.shape, MvpInductiveShape::Eq);
        assert_eq!(exported.universe_params, [universe]);
        assert_eq!(exported.constructors[0].name.as_str(), "Std.Eq.refl");
        assert_eq!(
            env.lookup(registered.recursor).expect("recursor").kind(),
            DeclarationKind::Recursor {
                ty,
                inductive: registered.family,
                generated: false,
            }
        );
    }

    #[test]
    fn rejects_mvp_shape_with_wrong_constructor_count() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let ty = sort(&mut terms, &mut levels, "u");
        let mut env = Environment::new();

        let error = register_mvp_inductive(
            &levels,
            &terms,
            &mut env,
            basic_spec(
                MvpInductiveShape::Bool,
                "Std.Bool",
                ty,
                &["Std.Bool.false"],
                "Std.Bool.rec",
            ),
        )
        .unwrap_err();

        assert_eq!(env.len(), 0);
        assert_eq!(error.code(), CoreErrorCode::InvalidDeclaration);
        assert_eq!(
            error.details().get("kind").map(String::as_str),
            Some("constructor_count_mismatch")
        );
    }

    #[test]
    fn rejects_duplicate_universe_params_without_mutating_environment() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let universe = levels.parse_param("u").expect("valid level param name");
        let ty = terms.sort(universe);
        let mut env = Environment::new();

        let error = register_mvp_inductive(
            &levels,
            &terms,
            &mut env,
            InductiveSpec::new(
                MvpInductiveShape::Eq,
                "Std.Eq",
                vec![universe, universe],
                ty,
                vec![ConstructorSignature::new("Std.Eq.refl", ty)],
                RecursorSignature::new("Std.Eq.rec", ty),
            ),
        )
        .unwrap_err();

        assert_eq!(env.len(), 0);
        assert_eq!(
            error.details().get("kind").map(String::as_str),
            Some("duplicate_universe_param")
        );
    }

    #[test]
    fn rejects_duplicate_artifact_names_without_mutating_environment() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let ty = sort(&mut terms, &mut levels, "u");
        let mut env = Environment::new();

        let error = register_mvp_inductive(
            &levels,
            &terms,
            &mut env,
            basic_spec(
                MvpInductiveShape::Nat,
                "Std.Nat",
                ty,
                &["Std.Nat.zero", "Std.Nat.zero"],
                "Std.Nat.rec",
            ),
        )
        .unwrap_err();

        assert_eq!(env.len(), 0);
        assert_eq!(
            error.details().get("kind").map(String::as_str),
            Some("duplicate_inductive_artifact_name")
        );
    }
}
