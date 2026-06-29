# ALPHA-005 Cache Key Optimization

Schema: `mpk.alpha005.cache_key_optimization.v0`

## Scope

ALPHA-005 optimizes the Rust fast-kernel cache layout without changing proof
acceptance rules or trusted evidence.

The ALPHA-004 profile identified proof-node checking, typecheck, and nested
cache key construction plus defeq hit/miss costs as the first follow-up area.
This change replaces the kernel cache's per-lookup environment snapshot with a
compact `EnvironmentCacheKey` made from:

- an environment-local id; and
- a revision incremented after each successful declaration registration.

The cache still keys by term arena identity, local context, and term ids. It now
avoids rebuilding a Vec of every environment declaration and cloning every
declaration name while preserving invalidation when the environment changes.

## Trust Boundary

No checker-facing evidence was added or removed. The verifier still accepts only
canonical certificate bytes after decode, declaration checking, proof-node
checking, export/axiom report recomputation, and hash validation.

The optimization is local to in-memory cache keys. It does not alter:

- canonical certificate encoding or decoding;
- core inference, reduction, defeq, or proof-node rules;
- axiom report construction;
- Rust fast-kernel verdict JSON; or
- Go reference-checker logic.

## Validation

The cache-key behavior is covered by focused tests:

- distinct `Environment` instances produce distinct cache keys;
- successful declaration registration changes an environment's cache revision;
- inferred-type cache entries separate different environments; and
- inferred-type cache entries separate different revisions of the same
  environment.

Checker agreement was validated with:

```sh
./scripts/checker-agreement.sh
```

The full release-style validation remains:

```sh
./scripts/check-all.sh
```
