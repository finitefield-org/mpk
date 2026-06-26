# Risk Register

| ID | Risk | Impact | Probability | Mitigation | Owner |
|---|---|---:|---:|---|---|
| R-001 | Trusted-base creep | Critical | High | Enforce trust-boundary gate and review every checker dependency. | Kernel lead |
| R-002 | Definitional equality becomes too powerful | Critical | Medium | Keep conversion rules small; reject eta, proof irrelevance, theorem unfolding, and SMT-backed conversion. | Core lead |
| R-003 | Go frontend semantics mismatch | High | High | Keep frontend untrusted; hash source/GIR/VC; fail closed on unsupported features. | Frontend lead |
| R-004 | Integer overflow modeled incorrectly | High | Medium | Model fixed-width integers as bitvectors with explicit signed views. | VC lead |
| R-005 | Solver yes/no accidentally trusted | Critical | Medium | Only accept checkable solver certificates; add negative tests. | Theory lead |
| R-006 | Certificate decoder vulnerabilities | High | Medium | Fuzz decoder; reject non-canonical encodings; avoid panics. | Cert lead |
| R-007 | Rust and Go checkers diverge | High | Medium | Checker agreement gate blocks release. | Release lead |
| R-008 | Standard library admits unproved convenience theorem | High | Medium | Every theorem must have a certificate; axiom report reviewed in CI. | Stdlib lead |
| R-009 | AI API becomes an implicit tactic trusted path | High | Medium | API can only construct/check/export certificates; no special acceptance path. | API lead |
| R-010 | Performance optimization changes semantics | High | Medium | Snapshot verdicts; checker agreement tests; optimize cache only after correctness tests. | Kernel lead |
| R-011 | Loops make VC generation too hard early | Medium | High | Start with straight-line and branch VCs; add partial-correctness loops with explicit invariants before optional total-correctness variants. | VC lead |
| R-012 | Scope expands to full Go too early | High | High | Enforce Go subset v0; unsupported features fail closed. | Project lead |
| R-013 | Import/package hashes are mishandled | High | Medium | Hash-pinned imports; high-trust mode; fixture tests for mismatch. | Cert lead |
| R-014 | Axiom reports are ignored by users | Medium | Medium | Make release gates fail on unapproved axioms and print reports by default. | Release lead |
| R-015 | Clean-room checker unintentionally shares logic | Medium | Medium | Separate language and implementation; no shared kernel crates; review dependency graph. | Reference lead |
