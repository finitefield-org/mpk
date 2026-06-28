//! Fuel-limited definitional equality for core terms.

use crate::{
    error::{CoreError, CoreErrorCode, CoreLocation},
    level::LevelId,
    reduce::{whnf_with_budget, ReduceError},
    term::{TermArena, TermId, TermNode},
};

pub const DEFAULT_DEFEQ_FUEL: u32 = 1024;

pub fn definitionally_equal(
    terms: &mut TermArena,
    lhs: TermId,
    rhs: TermId,
) -> Result<bool, CoreError> {
    definitionally_equal_with_fuel(terms, lhs, rhs, DEFAULT_DEFEQ_FUEL)
}

pub fn definitionally_equal_with_fuel(
    terms: &mut TermArena,
    lhs: TermId,
    rhs: TermId,
    fuel: u32,
) -> Result<bool, CoreError> {
    DefEqChecker { terms, fuel }.equal(lhs, rhs)
}

struct DefEqChecker<'a> {
    terms: &'a mut TermArena,
    fuel: u32,
}

impl DefEqChecker<'_> {
    fn equal(&mut self, lhs: TermId, rhs: TermId) -> Result<bool, CoreError> {
        self.consume_defeq_step(lhs, rhs)?;
        if lhs == rhs {
            return Ok(true);
        }

        let lhs_whnf = self.whnf("lhs", lhs, lhs, rhs)?;
        let rhs_whnf = self.whnf("rhs", rhs, lhs, rhs)?;
        if lhs_whnf == rhs_whnf {
            return Ok(true);
        }
        if lhs_whnf != lhs || rhs_whnf != rhs {
            return self.equal(lhs_whnf, rhs_whnf);
        }

        match (
            self.terms.node(lhs_whnf).clone(),
            self.terms.node(rhs_whnf).clone(),
        ) {
            (TermNode::Sort(lhs), TermNode::Sort(rhs)) => Ok(lhs == rhs),
            (TermNode::Var(lhs), TermNode::Var(rhs)) => Ok(lhs == rhs),
            (
                TermNode::Const {
                    global: lhs_global,
                    levels: lhs_levels,
                },
                TermNode::Const {
                    global: rhs_global,
                    levels: rhs_levels,
                },
            ) => Ok(lhs_global == rhs_global && levels_equal(&lhs_levels, &rhs_levels)),
            (
                TermNode::App {
                    function: lhs_function,
                    arguments: lhs_arguments,
                },
                TermNode::App {
                    function: rhs_function,
                    arguments: rhs_arguments,
                },
            ) => self.equal_app(lhs_function, lhs_arguments, rhs_function, rhs_arguments),
            (
                TermNode::Lam {
                    ty: lhs_ty,
                    body: lhs_body,
                },
                TermNode::Lam {
                    ty: rhs_ty,
                    body: rhs_body,
                },
            )
            | (
                TermNode::Pi {
                    ty: lhs_ty,
                    body: lhs_body,
                },
                TermNode::Pi {
                    ty: rhs_ty,
                    body: rhs_body,
                },
            ) => self.equal_binder(lhs_ty, lhs_body, rhs_ty, rhs_body),
            _ => Ok(false),
        }
    }

    fn equal_app(
        &mut self,
        lhs_function: TermId,
        lhs_arguments: Vec<TermId>,
        rhs_function: TermId,
        rhs_arguments: Vec<TermId>,
    ) -> Result<bool, CoreError> {
        if lhs_arguments.len() != rhs_arguments.len() {
            return Ok(false);
        }
        if !self.equal(lhs_function, rhs_function)? {
            return Ok(false);
        }
        for (lhs_argument, rhs_argument) in lhs_arguments.into_iter().zip(rhs_arguments) {
            if !self.equal(lhs_argument, rhs_argument)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn equal_binder(
        &mut self,
        lhs_ty: TermId,
        lhs_body: TermId,
        rhs_ty: TermId,
        rhs_body: TermId,
    ) -> Result<bool, CoreError> {
        if !self.equal(lhs_ty, rhs_ty)? {
            return Ok(false);
        }
        self.equal(lhs_body, rhs_body)
    }

    fn whnf(
        &mut self,
        side: &'static str,
        term: TermId,
        lhs: TermId,
        rhs: TermId,
    ) -> Result<TermId, CoreError> {
        whnf_with_budget(self.terms, term, &mut self.fuel)
            .map_err(|error| defeq_reduce_error(side, term, lhs, rhs, error))
    }

    fn consume_defeq_step(&mut self, lhs: TermId, rhs: TermId) -> Result<(), CoreError> {
        if self.fuel == 0 {
            return Err(
                CoreError::new(CoreErrorCode::FuelExhausted, defeq_location())
                    .with_detail("kind", "defeq_fuel_exhausted")
                    .with_detail("lhs_term_index", lhs.index().to_string())
                    .with_detail("rhs_term_index", rhs.index().to_string()),
            );
        }

        self.fuel -= 1;
        Ok(())
    }
}

fn levels_equal(lhs: &[LevelId], rhs: &[LevelId]) -> bool {
    lhs == rhs
}

fn defeq_reduce_error(
    side: &'static str,
    term: TermId,
    lhs: TermId,
    rhs: TermId,
    error: ReduceError,
) -> CoreError {
    CoreError::from_reduce_error(defeq_location().with_field(side).with_field("whnf"), error)
        .with_detail("operation", "defeq_whnf")
        .with_detail("side", side)
        .with_detail("term_index", term.index().to_string())
        .with_detail("lhs_term_index", lhs.index().to_string())
        .with_detail("rhs_term_index", rhs.index().to_string())
}

fn defeq_location() -> CoreLocation {
    CoreLocation::root().with_field("defeq")
}

#[cfg(test)]
mod tests {
    use super::{definitionally_equal, definitionally_equal_with_fuel};

    use crate::{LevelArena, TermArena, TermId};

    fn sort(terms: &mut TermArena, levels: &mut LevelArena, name: &str) -> TermId {
        let level = levels.parse_param(name).expect("valid level param");
        terms.sort(level)
    }

    #[test]
    fn defeq_reduces_beta_and_zeta_at_whnf() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let ty = sort(&mut terms, &mut levels, "u");
        let value = sort(&mut terms, &mut levels, "v");
        let body = terms.var(0);
        let lambda = terms.lam(ty, body);
        let application = terms.app(lambda, [value]);
        let let_term = terms.let_term(ty, value, body);

        assert!(definitionally_equal(&mut terms, application, value).expect("beta defeq"));
        assert!(definitionally_equal(&mut terms, let_term, value).expect("zeta defeq"));
    }

    #[test]
    fn defeq_reduces_under_matching_binders() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let ty = sort(&mut terms, &mut levels, "u");
        let value = terms.var(0);
        let let_body = terms.var(0);
        let body = terms.let_term(ty, value, let_body);
        let lhs = terms.lam(ty, body);
        let rhs_body = terms.var(0);
        let rhs = terms.lam(ty, rhs_body);

        assert!(definitionally_equal(&mut terms, lhs, rhs).expect("binder body defeq"));
    }

    #[test]
    fn defeq_returns_false_for_distinct_neutral_terms() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let lhs = sort(&mut terms, &mut levels, "u");
        let rhs = sort(&mut terms, &mut levels, "v");

        assert!(!definitionally_equal(&mut terms, lhs, rhs).expect("defeq completes"));
    }

    #[test]
    fn defeq_fuel_exhaustion_returns_stable_error() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let term = sort(&mut terms, &mut levels, "u");

        let error = definitionally_equal_with_fuel(&mut terms, term, term, 0).unwrap_err();

        assert_eq!(
            error.to_deterministic_json(),
            "{\"code\":\"CORE_FUEL_EXHAUSTED\",\"location\":[{\"field\":\"defeq\"}],\"details\":{\"kind\":\"defeq_fuel_exhausted\",\"lhs_term_index\":\"0\",\"rhs_term_index\":\"0\"}}"
        );
    }
}
