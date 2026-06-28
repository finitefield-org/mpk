//! Weak-head normalization skeleton for beta and zeta reduction.

use crate::{substitute_top, SubstError, TermArena, TermId, TermNode};

pub const DEFAULT_WHNF_FUEL: u32 = 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReduceError {
    FuelExhausted,
    Substitution(SubstError),
}

impl From<SubstError> for ReduceError {
    fn from(error: SubstError) -> Self {
        Self::Substitution(error)
    }
}

pub fn whnf(arena: &mut TermArena, term: TermId) -> Result<TermId, ReduceError> {
    whnf_with_fuel(arena, term, DEFAULT_WHNF_FUEL)
}

pub fn whnf_with_fuel(
    arena: &mut TermArena,
    term: TermId,
    fuel: u32,
) -> Result<TermId, ReduceError> {
    WhnfReducer { arena, fuel }.reduce(term)
}

struct WhnfReducer<'a> {
    arena: &'a mut TermArena,
    fuel: u32,
}

impl WhnfReducer<'_> {
    fn reduce(&mut self, mut term: TermId) -> Result<TermId, ReduceError> {
        loop {
            match self.arena.node(term).clone() {
                TermNode::Let { value, body, .. } => {
                    self.consume_fuel()?;
                    term = substitute_top(self.arena, body, value)?;
                }
                TermNode::App {
                    function,
                    arguments,
                } => match self.reduce_application(term, function, arguments)? {
                    WhnfStep::Continue(next) => term = next,
                    WhnfStep::Done(done) => return Ok(done),
                },
                TermNode::Sort(_)
                | TermNode::Var(_)
                | TermNode::Const { .. }
                | TermNode::Lam { .. }
                | TermNode::Pi { .. } => return Ok(term),
            }
        }
    }

    fn reduce_application(
        &mut self,
        original: TermId,
        function: TermId,
        arguments: Vec<TermId>,
    ) -> Result<WhnfStep, ReduceError> {
        let reduced_function = self.reduce(function)?;
        let TermNode::Lam { body, .. } = self.arena.node(reduced_function).clone() else {
            if reduced_function == function {
                return Ok(WhnfStep::Done(original));
            }
            return Ok(WhnfStep::Done(self.arena.app(reduced_function, arguments)));
        };

        let Some((argument, remaining_arguments)) = arguments.split_first() else {
            return Ok(WhnfStep::Done(reduced_function));
        };

        self.consume_fuel()?;
        let reduced = substitute_top(self.arena, body, *argument)?;
        if remaining_arguments.is_empty() {
            Ok(WhnfStep::Continue(reduced))
        } else {
            Ok(WhnfStep::Continue(
                self.arena.app(reduced, remaining_arguments.iter().copied()),
            ))
        }
    }

    fn consume_fuel(&mut self) -> Result<(), ReduceError> {
        if self.fuel == 0 {
            return Err(ReduceError::FuelExhausted);
        }

        self.fuel -= 1;
        Ok(())
    }
}

enum WhnfStep {
    Continue(TermId),
    Done(TermId),
}

#[cfg(test)]
mod tests {
    use crate::{whnf, whnf_with_fuel, LevelArena, ReduceError, TermArena, TermId, TermNode};

    fn sort(terms: &mut TermArena, levels: &mut LevelArena, name: &str) -> TermId {
        let level = levels.parse_param(name).expect("valid level param name");
        terms.sort(level)
    }

    fn assert_var(terms: &TermArena, term: TermId, expected: u32) {
        assert_eq!(terms.node(term), &TermNode::Var(expected));
    }

    #[test]
    fn whnf_beta_reduces_lambda_application() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let ty = sort(&mut terms, &mut levels, "u");
        let body = terms.var(0);
        let lambda = terms.lam(ty, body);
        let argument = sort(&mut terms, &mut levels, "v");
        let application = terms.app(lambda, [argument]);

        let reduced = whnf(&mut terms, application).expect("whnf succeeds");

        assert_eq!(reduced, argument);
    }

    #[test]
    fn whnf_beta_preserves_remaining_spine_arguments() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let ty = sort(&mut terms, &mut levels, "u");
        let body = terms.var(0);
        let lambda = terms.lam(ty, body);
        let argument = terms.var(2);
        let remaining = terms.var(1);
        let application = terms.app(lambda, [argument, remaining]);

        let reduced = whnf(&mut terms, application).expect("whnf succeeds");

        match terms.node(reduced) {
            TermNode::App {
                function,
                arguments,
            } => {
                assert_eq!(*function, argument);
                assert_eq!(arguments.as_slice(), [remaining]);
            }
            node => panic!("expected spine application, got {node:?}"),
        }
    }

    #[test]
    fn whnf_continues_after_spine_beta_exposes_lambda() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let ty = sort(&mut terms, &mut levels, "u");
        let inner_body = terms.var(0);
        let inner_lambda = terms.lam(ty, inner_body);
        let outer_lambda = terms.lam(ty, inner_lambda);
        let first_argument = terms.var(3);
        let second_argument = terms.var(2);
        let application = terms.app(outer_lambda, [first_argument, second_argument]);

        let reduced = whnf(&mut terms, application).expect("whnf succeeds");

        assert_eq!(reduced, second_argument);
    }

    #[test]
    fn whnf_zeta_reduces_let() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let ty = sort(&mut terms, &mut levels, "u");
        let value = sort(&mut terms, &mut levels, "v");
        let body = terms.var(0);
        let let_term = terms.let_term(ty, value, body);

        let reduced = whnf(&mut terms, let_term).expect("whnf succeeds");

        assert_eq!(reduced, value);
    }

    #[test]
    fn whnf_reduces_function_head_before_beta() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let ty = sort(&mut terms, &mut levels, "u");
        let lambda_body = terms.var(0);
        let lambda = terms.lam(ty, lambda_body);
        let let_body = terms.var(0);
        let function = terms.let_term(ty, lambda, let_body);
        let argument = sort(&mut terms, &mut levels, "v");
        let application = terms.app(function, [argument]);

        let reduced = whnf(&mut terms, application).expect("whnf succeeds");

        assert_eq!(reduced, argument);
    }

    #[test]
    fn whnf_does_not_reduce_under_lambda() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let ty = sort(&mut terms, &mut levels, "u");
        let value = sort(&mut terms, &mut levels, "v");
        let let_body = terms.var(0);
        let body = terms.let_term(ty, value, let_body);
        let lambda = terms.lam(ty, body);

        let reduced = whnf(&mut terms, lambda).expect("whnf succeeds");

        assert_eq!(reduced, lambda);
    }

    #[test]
    fn whnf_fuel_exhaustion_is_deterministic() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let ty = sort(&mut terms, &mut levels, "u");
        let value = sort(&mut terms, &mut levels, "v");
        let body = terms.var(0);
        let let_term = terms.let_term(ty, value, body);

        let error = whnf_with_fuel(&mut terms, let_term, 0).unwrap_err();

        assert_eq!(error, ReduceError::FuelExhausted);
    }

    #[test]
    fn whnf_leaves_neutral_app_head_in_whnf() {
        let mut terms = TermArena::new();
        let function = terms.var(0);
        let argument = terms.var(1);
        let application = terms.app(function, [argument]);

        let reduced = whnf(&mut terms, application).expect("whnf succeeds");

        match terms.node(reduced) {
            TermNode::App {
                function,
                arguments,
            } => {
                assert_var(&terms, *function, 0);
                assert_eq!(arguments.len(), 1);
                assert_var(&terms, arguments[0], 1);
            }
            node => panic!("expected neutral application, got {node:?}"),
        }
    }
}
