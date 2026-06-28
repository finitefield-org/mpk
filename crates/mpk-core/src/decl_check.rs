//! Checked declaration registration for core declarations.

use crate::{
    check, infer, CoreError, CoreErrorCode, CoreLocation, Environment, GlobalId, LevelArena,
    LocalContext, TermArena, TermId, TermNode,
};

pub fn register_checked_theorem(
    levels: &mut LevelArena,
    terms: &mut TermArena,
    env: &mut Environment,
    name: impl AsRef<str>,
    ty: TermId,
    proof: TermId,
) -> Result<GlobalId, CoreError> {
    check_theorem(levels, terms, env, ty, proof)?;
    env.register_theorem(name, ty, proof)
}

pub fn check_theorem(
    levels: &mut LevelArena,
    terms: &mut TermArena,
    env: &Environment,
    ty: TermId,
    proof: TermId,
) -> Result<(), CoreError> {
    let context = LocalContext::new();
    let ty_type = infer(levels, terms, &context, env, ty)?;
    expect_theorem_type_sort(terms, ty, ty_type)?;
    check(levels, terms, &context, env, proof, ty)
}

fn expect_theorem_type_sort(
    terms: &TermArena,
    theorem_type: TermId,
    inferred: TermId,
) -> Result<(), CoreError> {
    match terms.node(inferred) {
        TermNode::Sort(_) => Ok(()),
        node => Err(CoreError::new(
            CoreErrorCode::TypeMismatch,
            CoreLocation::root()
                .with_field("decl_check")
                .with_field("theorem")
                .with_field("type"),
        )
        .with_detail("kind", "theorem_type_not_sort")
        .with_detail("subject_term_index", theorem_type.index().to_string())
        .with_detail("inferred_term_index", inferred.index().to_string())
        .with_detail("expected", "sort")
        .with_detail("actual", term_kind(node))),
    }
}

fn term_kind(node: &TermNode) -> &'static str {
    match node {
        TermNode::Sort(_) => "sort",
        TermNode::Var(_) => "var",
        TermNode::Const { .. } => "const",
        TermNode::App { .. } => "app",
        TermNode::Lam { .. } => "lam",
        TermNode::Pi { .. } => "pi",
        TermNode::Let { .. } => "let",
    }
}

#[cfg(test)]
mod tests {
    use super::{check_theorem, register_checked_theorem};

    use crate::{
        definitionally_equal, CoreErrorCode, DeclarationKind, Environment, LevelArena, TermArena,
        TermId,
    };

    fn theorem_type_and_proof(levels: &mut LevelArena, terms: &mut TermArena) -> (TermId, TermId) {
        let zero = levels.zero();
        let theorem_type_level = levels.succ(zero);
        let theorem_type = terms.sort(theorem_type_level);
        let proof = terms.sort(zero);
        (theorem_type, proof)
    }

    #[test]
    fn registers_checked_theorem_after_type_and_proof_check() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let mut env = Environment::new();
        let (theorem_type, proof) = theorem_type_and_proof(&mut levels, &mut terms);

        let global = register_checked_theorem(
            &mut levels,
            &mut terms,
            &mut env,
            "Core.CheckedTheorem",
            theorem_type,
            proof,
        )
        .expect("checked theorem registers");
        let declaration = env.lookup(global).expect("registered declaration");

        assert_eq!(
            declaration.kind(),
            DeclarationKind::Theorem {
                ty: theorem_type,
                proof
            }
        );
        assert_eq!(declaration.ty(), theorem_type);
        assert_eq!(declaration.kind().definition_value(), None);
        assert!(!declaration.kind().is_reducible_definition());
    }

    #[test]
    fn checked_theorem_proof_never_unfolds_in_downstream_defeq() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let mut env = Environment::new();
        let (theorem_type, proof) = theorem_type_and_proof(&mut levels, &mut terms);
        let global = register_checked_theorem(
            &mut levels,
            &mut terms,
            &mut env,
            "Core.OpaqueTheorem",
            theorem_type,
            proof,
        )
        .expect("checked theorem registers");
        let theorem_const = terms.constant(global, []);

        assert!(
            !definitionally_equal(&env, &mut terms, theorem_const, proof)
                .expect("theorem proof remains opaque")
        );
    }

    #[test]
    fn rejects_theorem_type_that_does_not_infer_to_sort() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let mut env = Environment::new();
        let non_sort_type = terms.var(0);
        let global = env
            .register_axiom("Core.NonSortType", non_sort_type)
            .expect("raw axiom registration");
        let theorem_type = terms.constant(global, []);
        let proof = terms.var(1);

        let error = check_theorem(&mut levels, &mut terms, &env, theorem_type, proof).unwrap_err();

        assert_eq!(
            error.to_deterministic_json(),
            "{\"code\":\"CORE_TYPE_MISMATCH\",\"location\":[{\"field\":\"decl_check\"},{\"field\":\"theorem\"},{\"field\":\"type\"}],\"details\":{\"actual\":\"var\",\"expected\":\"sort\",\"inferred_term_index\":\"0\",\"kind\":\"theorem_type_not_sort\",\"subject_term_index\":\"1\"}}"
        );
    }

    #[test]
    fn rejects_bad_theorem_proof_without_mutating_environment() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let mut env = Environment::new();
        let zero = levels.zero();
        let theorem_type_level = levels.succ(zero);
        let theorem_type = terms.sort(theorem_type_level);
        let bad_proof = theorem_type;

        let error = register_checked_theorem(
            &mut levels,
            &mut terms,
            &mut env,
            "Core.BadProof",
            theorem_type,
            bad_proof,
        )
        .unwrap_err();

        assert_eq!(env.len(), 0);
        assert_eq!(error.code(), CoreErrorCode::TypeMismatch);
    }
}
