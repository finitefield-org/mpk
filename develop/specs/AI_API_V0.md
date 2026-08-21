# MPK AI API v0 Specification

Status: historical; replaced by `AI_API_V1.md`.

## Goal

The API is designed for AI and automated tools, not for human proof scripting. It should expose structured operations over interned IDs and return deterministic diagnostics.

## Principles

- API output is untrusted until encoded in `.mpcert` and checked.
- Prefer IDs over strings after initial name resolution.
- Return structured errors, not only human prose.
- Make failed proof nodes locally repairable.
- Support batch candidate checking.
- Support cache-friendly repeated subgoal attempts.

## Acceptance boundary

The API may construct terms, proof nodes, candidate DAGs, diagnostics, and canonical certificates. It must not mark a theorem, declaration, module, package, or release artifact as accepted unless the relevant certificate checker verifies the canonical `.mpcert` bytes under the active policy. Session state, cached proof nodes, strategy success, diagnostic `ok` fields, and batch candidate verdicts are helper data only.

## Session operations

```http
POST /module/new
POST /module/import
POST /module/freeze
POST /module/export-certificate
```

## Term operations

```http
POST /term/sort
POST /term/var
POST /term/const
POST /term/app
POST /term/lam
POST /term/pi
POST /term/let
POST /term/check
POST /term/infer
POST /term/defeq
```

## Proof operations

```http
POST /proof/exact
POST /proof/apply
POST /proof/intro
POST /proof/refl
POST /proof/let
POST /proof/rewrite
POST /proof/eq-rec
POST /proof/constructor
POST /proof/recursor
POST /proof/conv
POST /proof/theory
POST /proof/check-node
POST /proof/check-decl
```

`/proof/exact`, `/proof/apply`, `/proof/intro`, `/proof/refl`, and `/proof/conv` are the `core-bootstrap` endpoints required by API-003. `/proof/let`, `/proof/rewrite`, `/proof/eq-rec`, `/proof/constructor`, and `/proof/recursor` require the `mvp-structural` checker profile. `/proof/theory` requires TH-006. `split` is a strategy hint that may expand into constructor or introduction nodes; it is not a certificate proof-node kind.

## VC operations

```http
POST /gir/import
POST /vc/generate
GET  /vc/list
POST /vc/start-proof
POST /vc/attach-candidate
POST /vc/check-candidate
```

## Diagnostic schema

```json
{
  "ok": false,
  "error_code": "DEF_EQ_HEAD_MISMATCH",
  "node_id": 481,
  "expected_type_id": 921,
  "actual_type_id": 877,
  "expected_head": "Core.And",
  "actual_head": "Core.Or",
  "context_summary": [31, 44, 45],
  "repair_hints": ["split", "apply", "rewrite"]
}
```

## Batch checking

AI agents should submit many candidates cheaply:

```json
{
  "module_id": "m1",
  "candidates": [
    {"candidate_id": "c1", "proof_root": 3001},
    {"candidate_id": "c2", "proof_root": 3104}
  ],
  "mode": "fail_fast_per_candidate"
}
```

The checker must return deterministic rejections without mutating accepted module state unless explicitly committed.

## Repair loop

Recommended loop:

1. Generate candidate.
2. Check candidate locally.
3. If rejected, inspect structured error.
4. Repair only the smallest failed subtree.
5. Recheck.
6. Export canonical certificate only after all declarations check.

## Security posture

The API must not provide hidden trusted shortcuts. Any convenience operation must expand into certificate-checkable proof nodes or theory certificates. No endpoint may bypass canonical certificate export and checker verification.
