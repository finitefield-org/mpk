//! De Bruijn-safe lifting and substitution for core terms.

use crate::{TermArena, TermId, TermNode};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SubstError {
    VariableIndexOverflow { index: u32, amount: u32 },
    BinderDepthOverflow { depth: u32 },
    TargetIndexOverflow { target: u32, depth: u32 },
}

pub fn lift(arena: &mut TermArena, term: TermId, amount: u32) -> Result<TermId, SubstError> {
    lift_from(arena, term, amount, 0)
}

pub fn lift_from(
    arena: &mut TermArena,
    term: TermId,
    amount: u32,
    cutoff: u32,
) -> Result<TermId, SubstError> {
    if amount == 0 {
        return Ok(term);
    }

    lift_at_cutoff(arena, term, amount, cutoff)
}

pub fn substitute(
    arena: &mut TermArena,
    term: TermId,
    target: u32,
    replacement: TermId,
) -> Result<TermId, SubstError> {
    substitute_at_depth(arena, term, target, replacement, 0)
}

pub fn substitute_top(
    arena: &mut TermArena,
    body: TermId,
    replacement: TermId,
) -> Result<TermId, SubstError> {
    open_binder_at_depth(arena, body, replacement, 0)
}

pub fn beta_substitute(
    arena: &mut TermArena,
    body: TermId,
    argument: TermId,
) -> Result<TermId, SubstError> {
    substitute_top(arena, body, argument)
}

fn lift_at_cutoff(
    arena: &mut TermArena,
    term: TermId,
    amount: u32,
    cutoff: u32,
) -> Result<TermId, SubstError> {
    match arena.node(term).clone() {
        TermNode::Sort(_) | TermNode::Const { .. } => Ok(term),
        TermNode::Var(index) => {
            if index < cutoff {
                return Ok(term);
            }

            Ok(arena.var(
                index
                    .checked_add(amount)
                    .ok_or(SubstError::VariableIndexOverflow { index, amount })?,
            ))
        }
        TermNode::App {
            function,
            arguments,
        } => {
            let function = lift_at_cutoff(arena, function, amount, cutoff)?;
            let arguments = lift_many_at_cutoff(arena, arguments, amount, cutoff)?;
            Ok(arena.app(function, arguments))
        }
        TermNode::Lam { ty, body } => {
            let ty = lift_at_cutoff(arena, ty, amount, cutoff)?;
            let body = lift_at_cutoff(arena, body, amount, next_depth(cutoff)?)?;
            Ok(arena.lam(ty, body))
        }
        TermNode::Pi { ty, body } => {
            let ty = lift_at_cutoff(arena, ty, amount, cutoff)?;
            let body = lift_at_cutoff(arena, body, amount, next_depth(cutoff)?)?;
            Ok(arena.pi(ty, body))
        }
        TermNode::Let { ty, value, body } => {
            let ty = lift_at_cutoff(arena, ty, amount, cutoff)?;
            let value = lift_at_cutoff(arena, value, amount, cutoff)?;
            let body = lift_at_cutoff(arena, body, amount, next_depth(cutoff)?)?;
            Ok(arena.let_term(ty, value, body))
        }
    }
}

fn substitute_at_depth(
    arena: &mut TermArena,
    term: TermId,
    target: u32,
    replacement: TermId,
    depth: u32,
) -> Result<TermId, SubstError> {
    match arena.node(term).clone() {
        TermNode::Sort(_) | TermNode::Const { .. } => Ok(term),
        TermNode::Var(index) => {
            let shifted_target = target
                .checked_add(depth)
                .ok_or(SubstError::TargetIndexOverflow { target, depth })?;
            if index == shifted_target {
                lift_from(arena, replacement, depth, 0)
            } else {
                Ok(term)
            }
        }
        TermNode::App {
            function,
            arguments,
        } => {
            let function = substitute_at_depth(arena, function, target, replacement, depth)?;
            let arguments = substitute_many_at_depth(arena, arguments, target, replacement, depth)?;
            Ok(arena.app(function, arguments))
        }
        TermNode::Lam { ty, body } => {
            let ty = substitute_at_depth(arena, ty, target, replacement, depth)?;
            let body = substitute_at_depth(arena, body, target, replacement, next_depth(depth)?)?;
            Ok(arena.lam(ty, body))
        }
        TermNode::Pi { ty, body } => {
            let ty = substitute_at_depth(arena, ty, target, replacement, depth)?;
            let body = substitute_at_depth(arena, body, target, replacement, next_depth(depth)?)?;
            Ok(arena.pi(ty, body))
        }
        TermNode::Let { ty, value, body } => {
            let ty = substitute_at_depth(arena, ty, target, replacement, depth)?;
            let value = substitute_at_depth(arena, value, target, replacement, depth)?;
            let body = substitute_at_depth(arena, body, target, replacement, next_depth(depth)?)?;
            Ok(arena.let_term(ty, value, body))
        }
    }
}

fn open_binder_at_depth(
    arena: &mut TermArena,
    term: TermId,
    replacement: TermId,
    depth: u32,
) -> Result<TermId, SubstError> {
    match arena.node(term).clone() {
        TermNode::Sort(_) | TermNode::Const { .. } => Ok(term),
        TermNode::Var(index) => {
            if index == depth {
                lift_from(arena, replacement, depth, 0)
            } else if index > depth {
                Ok(arena.var(index - 1))
            } else {
                Ok(term)
            }
        }
        TermNode::App {
            function,
            arguments,
        } => {
            let function = open_binder_at_depth(arena, function, replacement, depth)?;
            let arguments = open_many_at_depth(arena, arguments, replacement, depth)?;
            Ok(arena.app(function, arguments))
        }
        TermNode::Lam { ty, body } => {
            let ty = open_binder_at_depth(arena, ty, replacement, depth)?;
            let body = open_binder_at_depth(arena, body, replacement, next_depth(depth)?)?;
            Ok(arena.lam(ty, body))
        }
        TermNode::Pi { ty, body } => {
            let ty = open_binder_at_depth(arena, ty, replacement, depth)?;
            let body = open_binder_at_depth(arena, body, replacement, next_depth(depth)?)?;
            Ok(arena.pi(ty, body))
        }
        TermNode::Let { ty, value, body } => {
            let ty = open_binder_at_depth(arena, ty, replacement, depth)?;
            let value = open_binder_at_depth(arena, value, replacement, depth)?;
            let body = open_binder_at_depth(arena, body, replacement, next_depth(depth)?)?;
            Ok(arena.let_term(ty, value, body))
        }
    }
}

fn lift_many_at_cutoff(
    arena: &mut TermArena,
    terms: Vec<TermId>,
    amount: u32,
    cutoff: u32,
) -> Result<Vec<TermId>, SubstError> {
    let mut lifted = Vec::with_capacity(terms.len());
    for term in terms {
        lifted.push(lift_at_cutoff(arena, term, amount, cutoff)?);
    }
    Ok(lifted)
}

fn substitute_many_at_depth(
    arena: &mut TermArena,
    terms: Vec<TermId>,
    target: u32,
    replacement: TermId,
    depth: u32,
) -> Result<Vec<TermId>, SubstError> {
    let mut substituted = Vec::with_capacity(terms.len());
    for term in terms {
        substituted.push(substitute_at_depth(
            arena,
            term,
            target,
            replacement,
            depth,
        )?);
    }
    Ok(substituted)
}

fn open_many_at_depth(
    arena: &mut TermArena,
    terms: Vec<TermId>,
    replacement: TermId,
    depth: u32,
) -> Result<Vec<TermId>, SubstError> {
    let mut opened = Vec::with_capacity(terms.len());
    for term in terms {
        opened.push(open_binder_at_depth(arena, term, replacement, depth)?);
    }
    Ok(opened)
}

fn next_depth(depth: u32) -> Result<u32, SubstError> {
    depth
        .checked_add(1)
        .ok_or(SubstError::BinderDepthOverflow { depth })
}

#[cfg(test)]
mod tests {
    use crate::{
        beta_substitute, lift, lift_from, substitute, substitute_top, LevelArena, TermArena,
        TermNode,
    };

    fn sort(terms: &mut TermArena, levels: &mut LevelArena, name: &str) -> crate::TermId {
        let level = levels.parse_param(name).expect("valid level param name");
        terms.sort(level)
    }

    fn assert_var(terms: &TermArena, term: crate::TermId, expected: u32) {
        assert_eq!(terms.node(term), &TermNode::Var(expected));
    }

    #[test]
    fn lift_shifts_free_variables_at_or_above_cutoff() {
        let mut terms = TermArena::new();
        let var0 = terms.var(0);
        let var1 = terms.var(1);
        let pair = terms.app(var1, [var0]);

        let lifted = lift_from(&mut terms, pair, 2, 1).expect("lift succeeds");

        match terms.node(lifted) {
            TermNode::App {
                function,
                arguments,
            } => {
                assert_var(&terms, *function, 3);
                assert_eq!(arguments.len(), 1);
                assert_var(&terms, arguments[0], 0);
            }
            node => panic!("expected lifted app, got {node:?}"),
        }
    }

    #[test]
    fn lift_respects_binders() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let ty = sort(&mut terms, &mut levels, "u");
        let bound = terms.var(0);
        let free = terms.var(1);
        let body = terms.app(free, [bound]);
        let lambda = terms.lam(ty, body);

        let lifted = lift(&mut terms, lambda, 1).expect("lift succeeds");

        match terms.node(lifted) {
            TermNode::Lam { body, .. } => match terms.node(*body) {
                TermNode::App {
                    function,
                    arguments,
                } => {
                    assert_var(&terms, *function, 2);
                    assert_eq!(arguments.len(), 1);
                    assert_var(&terms, arguments[0], 0);
                }
                node => panic!("expected lifted lambda body app, got {node:?}"),
            },
            node => panic!("expected lifted lambda, got {node:?}"),
        }
    }

    #[test]
    fn substitute_lifts_replacement_under_binders() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let ty = sort(&mut terms, &mut levels, "u");
        let target_under_binder = terms.var(1);
        let lambda = terms.lam(ty, target_under_binder);
        let replacement = terms.var(2);

        let substituted =
            substitute(&mut terms, lambda, 0, replacement).expect("substitution succeeds");

        match terms.node(substituted) {
            TermNode::Lam { body, .. } => assert_var(&terms, *body, 3),
            node => panic!("expected substituted lambda, got {node:?}"),
        }
    }

    #[test]
    fn beta_substitution_replaces_bound_variable() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let argument = sort(&mut terms, &mut levels, "u");
        let body = terms.var(0);

        let reduced = beta_substitute(&mut terms, body, argument).expect("beta substitution");

        assert_eq!(reduced, argument);
    }

    #[test]
    fn beta_substitution_avoids_capture_under_nested_binder() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let ty = sort(&mut terms, &mut levels, "u");
        let body = terms.var(1);
        let nested = terms.lam(ty, body);
        let argument = terms.var(0);

        let reduced = substitute_top(&mut terms, nested, argument).expect("beta substitution");

        match terms.node(reduced) {
            TermNode::Lam { body, .. } => assert_var(&terms, *body, 1),
            node => panic!("expected reduced lambda, got {node:?}"),
        }
    }

    #[test]
    fn beta_substitution_decrements_outer_free_variables() {
        let mut terms = TermArena::new();
        let body = terms.var(1);
        let argument = terms.var(0);

        let reduced = substitute_top(&mut terms, body, argument).expect("beta substitution");

        assert_var(&terms, reduced, 0);
    }
}
