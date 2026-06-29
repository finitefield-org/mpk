//! Checker-local caches for core inference, reduction, and conversion queries.

use std::collections::HashMap;
use std::time::Instant;

use mpk_core::{
    check as core_check, definitionally_equal, infer as core_infer, whnf as core_whnf, CoreError,
    DeclarationKind as CoreDeclarationKind, Environment, LevelArena, LocalContext, LocalDecl,
    ReduceError, TermArena, TermId, TermNode,
};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CheckerCacheStats {
    pub inferred_type_entries: usize,
    pub whnf_entries: usize,
    pub defeq_entries: usize,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CheckerCacheMetrics {
    pub infer: CheckerCacheOperationMetrics,
    pub whnf: CheckerCacheOperationMetrics,
    pub defeq: CheckerCacheOperationMetrics,
    pub check: CheckerCacheOperationMetrics,
}

impl CheckerCacheMetrics {
    pub fn saturating_sub(&self, baseline: &Self) -> Self {
        Self {
            infer: self.infer.saturating_sub(&baseline.infer),
            whnf: self.whnf.saturating_sub(&baseline.whnf),
            defeq: self.defeq.saturating_sub(&baseline.defeq),
            check: self.check.saturating_sub(&baseline.check),
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CheckerCacheOperationMetrics {
    pub calls: u64,
    pub hits: u64,
    pub misses: u64,
    pub elapsed_nanos: u128,
}

impl CheckerCacheOperationMetrics {
    fn record_call(&mut self) {
        self.calls += 1;
    }

    fn record_hit(&mut self) {
        self.hits += 1;
    }

    fn record_miss(&mut self) {
        self.misses += 1;
    }

    fn record_elapsed(&mut self, start: Option<Instant>) {
        if let Some(start) = start {
            self.elapsed_nanos += start.elapsed().as_nanos();
        }
    }

    fn saturating_sub(&self, baseline: &Self) -> Self {
        Self {
            calls: self.calls.saturating_sub(baseline.calls),
            hits: self.hits.saturating_sub(baseline.hits),
            misses: self.misses.saturating_sub(baseline.misses),
            elapsed_nanos: self.elapsed_nanos.saturating_sub(baseline.elapsed_nanos),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct CheckerCache {
    inferred_types: HashMap<InferKey, TermId>,
    whnfs: HashMap<WhnfKey, TermId>,
    defeqs: HashMap<DefEqKey, bool>,
    metrics: CheckerCacheMetrics,
    timing_enabled: bool,
}

impl CheckerCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.inferred_types.clear();
        self.whnfs.clear();
        self.defeqs.clear();
    }

    pub fn stats(&self) -> CheckerCacheStats {
        CheckerCacheStats {
            inferred_type_entries: self.inferred_types.len(),
            whnf_entries: self.whnfs.len(),
            defeq_entries: self.defeqs.len(),
        }
    }

    pub fn metrics(&self) -> CheckerCacheMetrics {
        self.metrics.clone()
    }

    pub fn enable_timing(&mut self) {
        self.timing_enabled = true;
    }

    pub fn infer(
        &mut self,
        levels: &mut LevelArena,
        terms: &mut TermArena,
        context: &LocalContext,
        env: &Environment,
        term: TermId,
    ) -> Result<TermId, CoreError> {
        self.metrics.infer.record_call();
        let start = self.timing_start();
        let key = InferKey::new(terms, env, context, term);
        if let Some(inferred) = self.inferred_types.get(&key).copied() {
            self.metrics.infer.record_hit();
            self.metrics.infer.record_elapsed(start);
            return Ok(inferred);
        }

        self.metrics.infer.record_miss();
        let inferred = core_infer(levels, terms, context, env, term);
        self.metrics.infer.record_elapsed(start);
        let inferred = inferred?;
        self.inferred_types.insert(key, inferred);
        Ok(inferred)
    }

    pub fn whnf(&mut self, terms: &mut TermArena, term: TermId) -> Result<TermId, ReduceError> {
        self.metrics.whnf.record_call();
        let start = self.timing_start();
        let key = WhnfKey::new(terms, term);
        if let Some(reduced) = self.whnfs.get(&key).copied() {
            self.metrics.whnf.record_hit();
            self.metrics.whnf.record_elapsed(start);
            return Ok(reduced);
        }

        self.metrics.whnf.record_miss();
        let reduced = core_whnf(terms, term);
        self.metrics.whnf.record_elapsed(start);
        let reduced = reduced?;
        self.whnfs.insert(key, reduced);
        Ok(reduced)
    }

    pub fn definitionally_equal(
        &mut self,
        env: &Environment,
        terms: &mut TermArena,
        lhs: TermId,
        rhs: TermId,
    ) -> Result<bool, CoreError> {
        self.metrics.defeq.record_call();
        let start = self.timing_start();
        let key = DefEqKey::new(terms, env, lhs, rhs);
        if let Some(equal) = self.defeqs.get(&key).copied() {
            self.metrics.defeq.record_hit();
            self.metrics.defeq.record_elapsed(start);
            return Ok(equal);
        }

        self.metrics.defeq.record_miss();
        let equal = definitionally_equal(env, terms, lhs, rhs);
        self.metrics.defeq.record_elapsed(start);
        let equal = equal?;
        self.defeqs.insert(key, equal);
        Ok(equal)
    }

    pub fn check(
        &mut self,
        levels: &mut LevelArena,
        terms: &mut TermArena,
        context: &LocalContext,
        env: &Environment,
        term: TermId,
        expected: TermId,
    ) -> Result<(), CoreError> {
        self.metrics.check.record_call();
        let start = self.timing_start();
        let result = self.check_uncounted(levels, terms, context, env, term, expected);
        self.metrics.check.record_elapsed(start);
        result
    }

    fn check_uncounted(
        &mut self,
        levels: &mut LevelArena,
        terms: &mut TermArena,
        context: &LocalContext,
        env: &Environment,
        term: TermId,
        expected: TermId,
    ) -> Result<(), CoreError> {
        match (terms.node(term).clone(), terms.node(expected).clone()) {
            (
                TermNode::Lam { ty, body },
                TermNode::Pi {
                    ty: expected_ty,
                    body: expected_body,
                },
            ) => {
                self.infer(levels, terms, context, env, expected)?;
                if !self.definitionally_equal(env, terms, ty, expected_ty)? {
                    return core_check(levels, terms, context, env, term, expected);
                }

                let mut body_context = context.clone();
                body_context.push_binder(expected_ty);
                self.check(levels, terms, &body_context, env, body, expected_body)
            }
            _ => {
                let inferred = self.infer(levels, terms, context, env, term)?;
                if self.definitionally_equal(env, terms, inferred, expected)? {
                    return Ok(());
                }

                core_check(levels, terms, context, env, term, expected)
            }
        }
    }

    fn timing_start(&self) -> Option<Instant> {
        self.timing_enabled.then(Instant::now)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
struct InferKey {
    arena: usize,
    env: EnvironmentKey,
    context: ContextKey,
    term: TermId,
}

impl InferKey {
    fn new(terms: &TermArena, env: &Environment, context: &LocalContext, term: TermId) -> Self {
        Self {
            arena: term_arena_id(terms),
            env: EnvironmentKey::new(env),
            context: ContextKey::new(context),
            term,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
struct ContextKey(Vec<LocalDecl>);

impl ContextKey {
    fn new(context: &LocalContext) -> Self {
        Self(context.iter_outer_to_inner().collect())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
struct WhnfKey {
    arena: usize,
    term: TermId,
}

impl WhnfKey {
    fn new(terms: &TermArena, term: TermId) -> Self {
        Self {
            arena: term_arena_id(terms),
            term,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
struct DefEqKey {
    arena: usize,
    env: EnvironmentKey,
    lhs: TermId,
    rhs: TermId,
}

impl DefEqKey {
    fn new(terms: &TermArena, env: &Environment, lhs: TermId, rhs: TermId) -> Self {
        let (lhs, rhs) = if lhs <= rhs { (lhs, rhs) } else { (rhs, lhs) };
        Self {
            arena: term_arena_id(terms),
            env: EnvironmentKey::new(env),
            lhs,
            rhs,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
struct EnvironmentKey(Vec<EnvironmentEntryKey>);

impl EnvironmentKey {
    fn new(env: &Environment) -> Self {
        Self(
            env.iter()
                .map(|declaration| EnvironmentEntryKey {
                    global: declaration.global().as_u32(),
                    name: declaration.name().as_str().to_owned(),
                    kind: declaration.kind(),
                })
                .collect(),
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
struct EnvironmentEntryKey {
    global: u32,
    name: String,
    kind: CoreDeclarationKind,
}

fn term_arena_id(terms: &TermArena) -> usize {
    terms as *const TermArena as usize
}

#[cfg(test)]
mod tests {
    use super::CheckerCache;

    use mpk_core::{
        check as core_check, definitionally_equal, infer, whnf, CoreError, Environment, LevelArena,
        LocalContext, TermArena, TermId,
    };

    fn sort(terms: &mut TermArena, levels: &mut LevelArena, name: &str) -> TermId {
        let level = levels.parse_param(name).expect("valid level param name");
        terms.sort(level)
    }

    fn error_json(error: Result<(), CoreError>) -> Result<(), String> {
        error.map_err(|error| error.to_deterministic_json())
    }

    #[test]
    fn cached_infer_matches_core_and_reuses_entry() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let mut env = Environment::new();
        let context = LocalContext::new();
        let ty = sort(&mut terms, &mut levels, "u");
        let global = env.register_axiom("Cache.Infer.x", ty).expect("axiom");
        let constant = terms.constant(global, []);
        let expected =
            infer(&mut levels, &mut terms, &context, &env, constant).expect("core infer succeeds");
        let mut cache = CheckerCache::new();

        let first = cache
            .infer(&mut levels, &mut terms, &context, &env, constant)
            .expect("cached infer succeeds");
        let second = cache
            .infer(&mut levels, &mut terms, &context, &env, constant)
            .expect("cached infer reuses entry");

        assert_eq!(first, expected);
        assert_eq!(second, expected);
        assert_eq!(cache.stats().inferred_type_entries, 1);
    }

    #[test]
    fn inferred_type_cache_keys_include_environment_snapshot() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let mut first_env = Environment::new();
        let mut second_env = Environment::new();
        let context = LocalContext::new();
        let sort0 = {
            let zero = levels.zero();
            terms.sort(zero)
        };
        let sort1 = {
            let one = levels.succ(levels.zero());
            terms.sort(one)
        };
        let global = first_env
            .register_axiom("Cache.Env.first", sort0)
            .expect("first axiom");
        second_env
            .register_axiom("Cache.Env.second", sort1)
            .expect("second axiom");
        let constant = terms.constant(global, []);
        let mut cache = CheckerCache::new();

        let first = cache
            .infer(&mut levels, &mut terms, &context, &first_env, constant)
            .expect("first env infers");
        let second = cache
            .infer(&mut levels, &mut terms, &context, &second_env, constant)
            .expect("second env infers");

        assert_eq!(first, sort0);
        assert_eq!(second, sort1);
        assert_eq!(cache.stats().inferred_type_entries, 2);
    }

    #[test]
    fn inferred_type_cache_keys_include_term_arena_identity() {
        let mut first_levels = LevelArena::new();
        let mut first_terms = TermArena::new();
        let mut second_levels = LevelArena::new();
        let mut second_terms = TermArena::new();
        let env = Environment::new();
        let context = LocalContext::new();
        let first_sort = {
            let zero = first_levels.zero();
            first_terms.sort(zero)
        };
        let second_sort = {
            let zero = second_levels.zero();
            second_terms.sort(zero)
        };
        assert_eq!(first_sort, second_sort);
        let mut cache = CheckerCache::new();

        cache
            .infer(
                &mut first_levels,
                &mut first_terms,
                &context,
                &env,
                first_sort,
            )
            .expect("first arena infers");
        cache
            .infer(
                &mut second_levels,
                &mut second_terms,
                &context,
                &env,
                second_sort,
            )
            .expect("second arena infers");

        assert_eq!(cache.stats().inferred_type_entries, 2);
    }

    #[test]
    fn cached_whnf_matches_core_and_reuses_entry() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let ty = sort(&mut terms, &mut levels, "u");
        let body = terms.var(0);
        let lambda = terms.lam(ty, body);
        let argument = sort(&mut terms, &mut levels, "v");
        let application = terms.app(lambda, [argument]);
        let expected = whnf(&mut terms, application).expect("core whnf succeeds");
        let mut cache = CheckerCache::new();

        let first = cache.whnf(&mut terms, application).expect("cached whnf");
        let second = cache
            .whnf(&mut terms, application)
            .expect("cached whnf hit");

        assert_eq!(first, expected);
        assert_eq!(second, expected);
        assert_eq!(cache.stats().whnf_entries, 1);
    }

    #[test]
    fn cached_defeq_matches_core_and_reuses_entry() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let env = Environment::new();
        let ty = sort(&mut terms, &mut levels, "u");
        let value = sort(&mut terms, &mut levels, "v");
        let body = terms.var(0);
        let let_term = terms.let_term(ty, value, body);
        let expected =
            definitionally_equal(&env, &mut terms, let_term, value).expect("core defeq succeeds");
        let mut cache = CheckerCache::new();

        let first = cache
            .definitionally_equal(&env, &mut terms, let_term, value)
            .expect("cached defeq");
        let second = cache
            .definitionally_equal(&env, &mut terms, value, let_term)
            .expect("cached defeq symmetric hit");

        assert_eq!(first, expected);
        assert_eq!(second, expected);
        assert_eq!(cache.stats().defeq_entries, 1);
        assert_eq!(cache.metrics().defeq.calls, 2);
        assert_eq!(cache.metrics().defeq.hits, 1);
        assert_eq!(cache.metrics().defeq.misses, 1);
        assert_eq!(cache.metrics().defeq.elapsed_nanos, 0);
    }

    #[test]
    fn cached_check_matches_core_for_successes_and_failures() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let env = Environment::new();
        let context = LocalContext::new();
        let zero = levels.zero();
        let sort0 = terms.sort(zero);
        let sort1 = {
            let one = levels.succ(zero);
            terms.sort(one)
        };
        let lambda_body = terms.var(0);
        let lambda = terms.lam(sort0, lambda_body);
        let expected_pi = terms.pi(sort0, sort0);
        let bad_lambda = terms.lam(sort0, sort0);
        let mut cache = CheckerCache::new();

        assert_eq!(
            error_json(core_check(
                &mut levels,
                &mut terms,
                &context,
                &env,
                sort0,
                sort1
            )),
            error_json(cache.check(&mut levels, &mut terms, &context, &env, sort0, sort1))
        );
        assert_eq!(
            error_json(core_check(
                &mut levels,
                &mut terms,
                &context,
                &env,
                lambda,
                expected_pi
            )),
            error_json(cache.check(&mut levels, &mut terms, &context, &env, lambda, expected_pi))
        );
        assert_eq!(
            error_json(core_check(
                &mut levels,
                &mut terms,
                &context,
                &env,
                bad_lambda,
                expected_pi
            )),
            error_json(cache.check(
                &mut levels,
                &mut terms,
                &context,
                &env,
                bad_lambda,
                expected_pi
            ))
        );
        assert_eq!(
            error_json(core_check(
                &mut levels,
                &mut terms,
                &context,
                &env,
                sort0,
                sort0
            )),
            error_json(cache.check(&mut levels, &mut terms, &context, &env, sort0, sort0))
        );
    }
}
