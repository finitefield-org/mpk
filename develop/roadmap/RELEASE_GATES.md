# Release Gates

## Gate A: Trust-boundary gate

Before any release:

- No source parser is called by the checker.
- No tactic script is called by the checker.
- No AI trace is read by the checker.
- No solver yes/no answer is trusted.
- All accepted proof evidence is in `.mpcert` or checked theory certificates.

## Gate B: Canonicality gate

- Decoder rejects non-minimal varints.
- Decoder rejects duplicate table entries after canonicalization.
- Decoder rejects unreachable table entries.
- Re-encode result equals original bytes.
- Hashes recompute exactly.

## Gate C: Determinism gate

- Same input produces same certificate bytes.
- Same input produces same export hash.
- Same input produces same axiom report hash.
- Same rejection produces same machine-readable error code and location.

## Gate D: Checker agreement gate

- Rust fast kernel and Go reference checker agree on valid fixtures.
- Rust fast kernel and Go reference checker agree on invalid fixtures.
- Any disagreement blocks release.

## Gate E: Axiom gate

- Axiom report is generated and reviewed.
- Release profile specifies allowed axioms.
- Any new axiom is a release blocker unless explicitly approved.

## Gate F: Go frontend gate

- Unsupported Go features fail closed.
- Go source hash is recorded.
- Go version is recorded.
- Frontend binary hash is recorded.
- GIR hash is recorded.
- VC hash is recorded.

## Gate G: Performance gate

- Full MVP fixture suite checks under target resource budgets.
- 10,000 invalid candidates reject without memory growth regressions.
- Defeq fuel exhaustion is deterministic.

## Gate H: Security gate

- Kernel crates forbid unsafe code in MVP.
- Fuzz tests cover certificate decoder.
- Malformed certificates never panic.
- Public API cannot bypass certificate verification.
