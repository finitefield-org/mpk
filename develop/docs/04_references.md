# References

Accessed: 2026-06-26.

## NPA

- NPA repository: https://github.com/finitefield-org/npa
- NPA core spec v0.2.0: https://github.com/finitefield-org/npa/blob/main/develop/core-spec-v0.2.0.md
- NPA proof-corpus AI workflow: https://github.com/finitefield-org/npa/blob/main/develop/proof-corpus-ai-workflow.md

Relevant points used in this plan:

- Certificate-first proof evidence.
- Small trusted base.
- Untrusted parser/elaborator/tactic/automation/AI layers.
- Canonical certificate bytes.
- Rust verifier and source-free reference checker.
- Deterministic hashes and axiom reports.
- Core term grammar using Sort, variable, Const, App, Lam, Pi, Let.
- Deterministic and fuel-limited definitional equality.
- Theorem opacity after checking.
- Canonical binary encoding.
- Explicit out-of-scope items such as eta conversion, proof irrelevance as conversion, theorem unfolding, external SMT trust, theorem graph trust, and AI search trust.

## Go

- Go specification: https://go.dev/ref/spec
- Go SSA package: https://pkg.go.dev/golang.org/x/tools/go/ssa

Relevant points used in this plan:

- Go has fixed-width integer semantics that require careful modeling.
- Unsigned integer operations wrap modulo the bit width.
- Signed integer overflow is legally defined by representation, operation, and operands and does not panic.
- Go SSA tooling is available for frontend engineering, but MPK treats frontend output as untrusted.
