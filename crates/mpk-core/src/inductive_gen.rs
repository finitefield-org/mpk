//! Canonical MVP inductive constructor and recursor generation.

use std::collections::BTreeSet;

use crate::{
    check_mvp_positivity, CoreError, CoreErrorCode, CoreLocation, Environment,
    ExportedInductiveDeclaration, GlobalId, LevelArena, LevelId, LevelNode, MvpInductiveShape,
    Name, NameError, RegisteredInductive, TermArena, TermId,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InductiveGenerationInput {
    pub shape: MvpInductiveShape,
    pub family_name: String,
    pub universe_params: Vec<LevelId>,
    pub ty: TermId,
}

impl InductiveGenerationInput {
    pub fn new(
        shape: MvpInductiveShape,
        family_name: impl Into<String>,
        universe_params: Vec<LevelId>,
        ty: TermId,
    ) -> Self {
        Self {
            shape,
            family_name: family_name.into(),
            universe_params,
            ty,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum GeneratedArtifactKind {
    Inductive,
    Constructor,
    Recursor,
}

impl GeneratedArtifactKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Inductive => "inductive",
            Self::Constructor => "constructor",
            Self::Recursor => "recursor",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct GeneratedArtifactHash(u64);

impl GeneratedArtifactHash {
    pub fn as_u64(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GeneratedArtifact {
    pub kind: GeneratedArtifactKind,
    pub global: GlobalId,
    pub name: Name,
    pub ty: TermId,
    pub hash: GeneratedArtifactHash,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GeneratedInductiveDeclarations {
    pub registered: RegisteredInductive,
    pub family: GeneratedArtifact,
    pub constructors: Vec<GeneratedArtifact>,
    pub recursor: GeneratedArtifact,
    pub interface_hash: GeneratedArtifactHash,
}

impl GeneratedInductiveDeclarations {
    pub fn artifact_hashes_in_dependency_order(&self) -> Vec<GeneratedArtifactHash> {
        let mut hashes = Vec::with_capacity(self.constructors.len() + 2);
        hashes.push(self.family.hash);
        hashes.extend(self.constructors.iter().map(|artifact| artifact.hash));
        hashes.push(self.recursor.hash);
        hashes
    }
}

pub fn generate_mvp_inductive_declarations(
    levels: &mut LevelArena,
    terms: &mut TermArena,
    env: &mut Environment,
    input: InductiveGenerationInput,
) -> Result<GeneratedInductiveDeclarations, CoreError> {
    let generated_names = generated_names(&input)?;
    validate_generation_input(levels, terms, env, &input, &generated_names)?;

    let family = env.register_inductive(&input.family_name, input.ty)?;
    let family_term = terms.constant(family, input.universe_params.iter().copied());

    let constructor_types = generated_constructor_types(input.shape, terms, family_term);
    let mut constructors = Vec::with_capacity(generated_names.constructors.len());
    for (constructor_name, constructor_type) in generated_names
        .constructors
        .iter()
        .zip(constructor_types.iter().copied())
    {
        constructors.push(env.register_generated_constructor(
            constructor_name.as_str(),
            constructor_type,
            family,
        )?);
    }

    let recursor_type = generated_recursor_type(input.shape, terms, family_term);
    let recursor =
        env.register_generated_recursor(generated_names.recursor.as_str(), recursor_type, family)?;

    let registered = RegisteredInductive {
        shape: input.shape,
        universe_params: input.universe_params.clone(),
        family,
        constructors,
        recursor,
    };
    check_mvp_positivity(terms, env, &registered)?;

    build_generated_output(levels, terms, env, &input, registered)
}

fn generated_constructor_types(
    shape: MvpInductiveShape,
    terms: &mut TermArena,
    family_term: TermId,
) -> Vec<TermId> {
    match shape {
        MvpInductiveShape::Bool => vec![family_term, family_term],
        MvpInductiveShape::Nat => vec![family_term, terms.pi(family_term, family_term)],
        MvpInductiveShape::Eq => vec![family_term],
    }
}

fn generated_recursor_type(
    shape: MvpInductiveShape,
    terms: &mut TermArena,
    family_term: TermId,
) -> TermId {
    match shape {
        MvpInductiveShape::Bool => {
            let true_case_to_major = terms.pi(family_term, family_term);
            let false_case_to_true_case = terms.pi(family_term, true_case_to_major);
            terms.pi(family_term, false_case_to_true_case)
        }
        MvpInductiveShape::Nat => {
            let step_result = terms.pi(family_term, family_term);
            let step_type = terms.pi(family_term, step_result);
            let major_to_result = terms.pi(family_term, family_term);
            let step_to_major = terms.pi(step_type, major_to_result);
            terms.pi(family_term, step_to_major)
        }
        MvpInductiveShape::Eq => terms.pi(family_term, family_term),
    }
}

fn build_generated_output(
    levels: &mut LevelArena,
    terms: &mut TermArena,
    env: &Environment,
    input: &InductiveGenerationInput,
    registered: RegisteredInductive,
) -> Result<GeneratedInductiveDeclarations, CoreError> {
    let exported = registered.export(env)?;
    let family = generated_artifact(
        levels,
        terms,
        input,
        GeneratedArtifactKind::Inductive,
        &exported.family,
    );

    let mut constructors = Vec::with_capacity(exported.constructors.len());
    for constructor in &exported.constructors {
        constructors.push(generated_artifact(
            levels,
            terms,
            input,
            GeneratedArtifactKind::Constructor,
            constructor,
        ));
    }

    let recursor = generated_artifact(
        levels,
        terms,
        input,
        GeneratedArtifactKind::Recursor,
        &exported.recursor,
    );
    let interface_hash = generated_interface_hash(
        levels,
        input,
        family.hash,
        constructors.iter().map(|artifact| artifact.hash),
        recursor.hash,
    );

    Ok(GeneratedInductiveDeclarations {
        registered,
        family,
        constructors,
        recursor,
        interface_hash,
    })
}

fn generated_artifact(
    levels: &mut LevelArena,
    terms: &mut TermArena,
    input: &InductiveGenerationInput,
    kind: GeneratedArtifactKind,
    declaration: &ExportedInductiveDeclaration,
) -> GeneratedArtifact {
    let hash = generated_artifact_hash(levels, terms, input, kind, declaration);
    GeneratedArtifact {
        kind,
        global: declaration.global,
        name: declaration.name.clone(),
        ty: declaration.ty,
        hash,
    }
}

fn generated_artifact_hash(
    levels: &mut LevelArena,
    terms: &mut TermArena,
    input: &InductiveGenerationInput,
    kind: GeneratedArtifactKind,
    declaration: &ExportedInductiveDeclaration,
) -> GeneratedArtifactHash {
    let mut hasher = StableGeneratedHasher::new();
    hasher.write_str("mpk.inductive_gen.artifact.v0");
    hasher.write_str(kind.as_str());
    hasher.write_str(input.shape.as_str());
    hasher.write_str(&input.family_name);
    hasher.write_str(declaration.name.as_str());
    hasher.write_u64(terms.structural_hash(levels, declaration.ty).as_u64());
    hasher.write_u64(input.universe_params.len() as u64);
    for param in &input.universe_params {
        hasher.write_u64(levels.stable_hash(*param).as_u64());
    }
    GeneratedArtifactHash(hasher.finish())
}

fn generated_interface_hash(
    levels: &mut LevelArena,
    input: &InductiveGenerationInput,
    family_hash: GeneratedArtifactHash,
    constructor_hashes: impl IntoIterator<Item = GeneratedArtifactHash>,
    recursor_hash: GeneratedArtifactHash,
) -> GeneratedArtifactHash {
    let mut hasher = StableGeneratedHasher::new();
    hasher.write_str("mpk.inductive_gen.interface.v0");
    hasher.write_str(input.shape.as_str());
    hasher.write_str(&input.family_name);
    hasher.write_u64(input.universe_params.len() as u64);
    for param in &input.universe_params {
        hasher.write_u64(levels.stable_hash(*param).as_u64());
    }
    hasher.write_u64(family_hash.as_u64());
    for hash in constructor_hashes {
        hasher.write_u64(hash.as_u64());
    }
    hasher.write_u64(recursor_hash.as_u64());
    GeneratedArtifactHash(hasher.finish())
}

fn generated_names(input: &InductiveGenerationInput) -> Result<GeneratedNames, CoreError> {
    let family = parse_generated_name(
        &input.family_name,
        generation_location().with_field("family_name"),
    )?;
    let constructors = constructor_suffixes(input.shape)
        .iter()
        .map(|suffix| {
            let raw = child_name(&input.family_name, suffix);
            parse_generated_name(&raw, generation_location().with_field("constructors"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let recursor_raw = child_name(&input.family_name, "rec");
    let recursor =
        parse_generated_name(&recursor_raw, generation_location().with_field("recursor"))?;

    Ok(GeneratedNames {
        family,
        constructors,
        recursor,
    })
}

fn constructor_suffixes(shape: MvpInductiveShape) -> &'static [&'static str] {
    match shape {
        MvpInductiveShape::Bool => &["false", "true"],
        MvpInductiveShape::Nat => &["zero", "succ"],
        MvpInductiveShape::Eq => &["refl"],
    }
}

fn child_name(family_name: &str, suffix: &str) -> String {
    format!("{family_name}.{suffix}")
}

fn parse_generated_name(raw: &str, location: CoreLocation) -> Result<Name, CoreError> {
    Name::parse(raw).map_err(|error| invalid_name_error(raw, &error, location))
}

fn validate_generation_input(
    levels: &LevelArena,
    terms: &TermArena,
    env: &Environment,
    input: &InductiveGenerationInput,
    generated_names: &GeneratedNames,
) -> Result<(), CoreError> {
    validate_term_reference(terms, input.ty, generation_location().with_field("type"))?;
    validate_universe_params(levels, &input.universe_params)?;
    validate_generated_names(env, generated_names)
}

fn validate_universe_params(
    levels: &LevelArena,
    universe_params: &[LevelId],
) -> Result<(), CoreError> {
    let mut seen = BTreeSet::new();
    for (index, param) in universe_params.iter().copied().enumerate() {
        let location = generation_location()
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

fn validate_generated_names(
    env: &Environment,
    generated_names: &GeneratedNames,
) -> Result<(), CoreError> {
    let mut seen = BTreeSet::new();
    validate_generated_name(env, &mut seen, &generated_names.family)?;
    for constructor in &generated_names.constructors {
        validate_generated_name(env, &mut seen, constructor)?;
    }
    validate_generated_name(env, &mut seen, &generated_names.recursor)
}

fn validate_generated_name(
    env: &Environment,
    seen: &mut BTreeSet<Name>,
    name: &Name,
) -> Result<(), CoreError> {
    if !seen.insert(name.clone()) {
        return Err(CoreError::new(
            CoreErrorCode::InvalidDeclaration,
            generation_location().with_field("names"),
        )
        .with_detail("kind", "duplicate_generated_name")
        .with_detail("name", name.as_str()));
    }
    if env.lookup_name(name).is_some() {
        return Err(CoreError::new(
            CoreErrorCode::InvalidDeclaration,
            generation_location().with_field("names"),
        )
        .with_detail("kind", "duplicate_declaration")
        .with_detail("name", name.as_str()));
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

#[derive(Clone, Debug, Eq, PartialEq)]
struct GeneratedNames {
    family: Name,
    constructors: Vec<Name>,
    recursor: Name,
}

fn invalid_name_error(raw: &str, error: &NameError, location: CoreLocation) -> CoreError {
    CoreError::new(CoreErrorCode::InvalidName, location)
        .with_detail("name", raw)
        .with_detail("name_error", error.code())
}

fn generation_location() -> CoreLocation {
    CoreLocation::root().with_field("inductive_gen")
}

fn index_as_u32(index: usize) -> Result<u32, CoreError> {
    u32::try_from(index).map_err(|_| {
        CoreError::new(CoreErrorCode::InternalInvariant, generation_location())
            .with_detail("kind", "index_overflow")
            .with_detail("index", index.to_string())
    })
}

struct StableGeneratedHasher(u64);

impl StableGeneratedHasher {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;

    fn new() -> Self {
        Self(Self::OFFSET)
    }

    fn write_u8(&mut self, byte: u8) {
        self.0 ^= u64::from(byte);
        self.0 = self.0.wrapping_mul(Self::PRIME);
    }

    fn write_u64(&mut self, value: u64) {
        self.write_bytes(&value.to_le_bytes());
    }

    fn write_str(&mut self, value: &str) {
        self.write_u64(value.len() as u64);
        self.write_bytes(value.as_bytes());
    }

    fn write_bytes(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.write_u8(*byte);
        }
    }

    fn finish(self) -> u64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::{
        generate_mvp_inductive_declarations, GeneratedArtifactKind, InductiveGenerationInput,
    };

    use crate::{
        DeclarationKind, Environment, LevelArena, MvpInductiveShape, TermArena, TermId, TermNode,
    };

    fn sort(terms: &mut TermArena, levels: &mut LevelArena, name: &str) -> TermId {
        let level = levels.parse_param(name).expect("valid level param name");
        terms.sort(level)
    }

    fn generate_fresh_bool() -> super::GeneratedInductiveDeclarations {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let mut env = Environment::new();
        let ty = sort(&mut terms, &mut levels, "u");
        generate_mvp_inductive_declarations(
            &mut levels,
            &mut terms,
            &mut env,
            InductiveGenerationInput::new(MvpInductiveShape::Bool, "Std.Bool", Vec::new(), ty),
        )
        .expect("Bool generation succeeds")
    }

    #[test]
    fn generates_bool_constructor_and_recursor_declarations_in_dependency_order() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let mut env = Environment::new();
        let ty = sort(&mut terms, &mut levels, "u");

        let generated = generate_mvp_inductive_declarations(
            &mut levels,
            &mut terms,
            &mut env,
            InductiveGenerationInput::new(MvpInductiveShape::Bool, "Std.Bool", Vec::new(), ty),
        )
        .expect("Bool generation succeeds");

        assert_eq!(generated.family.kind, GeneratedArtifactKind::Inductive);
        assert_eq!(generated.family.name.as_str(), "Std.Bool");
        assert_eq!(
            generated
                .constructors
                .iter()
                .map(|artifact| artifact.name.as_str())
                .collect::<Vec<_>>(),
            ["Std.Bool.false", "Std.Bool.true"]
        );
        assert_eq!(generated.recursor.name.as_str(), "Std.Bool.rec");
        assert!(generated.family.global < generated.constructors[0].global);
        assert!(generated.constructors[0].global < generated.constructors[1].global);
        assert!(generated.constructors[1].global < generated.recursor.global);
        assert_eq!(
            env.lookup(generated.family.global)
                .expect("family declaration")
                .kind(),
            DeclarationKind::Inductive { ty }
        );
    }

    #[test]
    fn generated_hashes_are_stable_for_fresh_runs() {
        let first = generate_fresh_bool();
        let second = generate_fresh_bool();

        assert_eq!(
            first
                .constructors
                .iter()
                .map(|artifact| artifact.name.clone())
                .collect::<Vec<_>>(),
            second
                .constructors
                .iter()
                .map(|artifact| artifact.name.clone())
                .collect::<Vec<_>>()
        );
        assert_eq!(
            first.artifact_hashes_in_dependency_order(),
            second.artifact_hashes_in_dependency_order()
        );
        assert_eq!(first.interface_hash, second.interface_hash);
        assert_ne!(first.interface_hash.as_u64(), 0);
    }

    #[test]
    fn generates_nat_succ_as_direct_recursive_constructor() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let mut env = Environment::new();
        let ty = sort(&mut terms, &mut levels, "u");

        let generated = generate_mvp_inductive_declarations(
            &mut levels,
            &mut terms,
            &mut env,
            InductiveGenerationInput::new(MvpInductiveShape::Nat, "Std.Nat", Vec::new(), ty),
        )
        .expect("Nat generation succeeds");
        let family_const = terms.constant(generated.family.global, []);
        let succ = generated.constructors[1].ty;

        assert_eq!(generated.constructors[0].name.as_str(), "Std.Nat.zero");
        assert_eq!(generated.constructors[1].name.as_str(), "Std.Nat.succ");
        assert!(matches!(
            terms.node(succ),
            TermNode::Pi { ty, body } if *ty == family_const && *body == family_const
        ));
    }

    #[test]
    fn generates_eq_with_universe_parameter_in_signature_hash() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let mut env = Environment::new();
        let universe = levels.parse_param("u").expect("valid level param name");
        let ty = terms.sort(universe);

        let generated = generate_mvp_inductive_declarations(
            &mut levels,
            &mut terms,
            &mut env,
            InductiveGenerationInput::new(MvpInductiveShape::Eq, "Std.Eq", vec![universe], ty),
        )
        .expect("Eq generation succeeds");

        assert_eq!(generated.registered.universe_params, [universe]);
        assert_eq!(generated.constructors[0].name.as_str(), "Std.Eq.refl");
        assert_eq!(generated.recursor.name.as_str(), "Std.Eq.rec");
        assert_ne!(generated.constructors[0].hash, generated.recursor.hash);
    }

    #[test]
    fn duplicate_generated_names_reject_before_mutating_environment() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let mut env = Environment::new();
        let ty = sort(&mut terms, &mut levels, "u");
        env.register_axiom("Std.Bool.true", ty)
            .expect("preexisting declaration");

        let error = generate_mvp_inductive_declarations(
            &mut levels,
            &mut terms,
            &mut env,
            InductiveGenerationInput::new(MvpInductiveShape::Bool, "Std.Bool", Vec::new(), ty),
        )
        .unwrap_err();

        assert_eq!(env.len(), 1);
        assert_eq!(
            error.details().get("kind").map(String::as_str),
            Some("duplicate_declaration")
        );
        assert_eq!(
            error.details().get("name").map(String::as_str),
            Some("Std.Bool.true")
        );
    }
}
