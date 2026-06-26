# Task Backlog

This backlog is dependency-ordered. Estimates are intentionally omitted; use dependency order and release gates rather than artificial dates.

## P0: Governance and Spec Freeze

### GOV-001 — Create project charter

- **Component:** Governance
- **Priority:** P0
- **Dependencies:** none
- **Deliverable:** `docs/CHARTER.md`
- **Description:** Write the mission, non-goals, MVP scope, and acceptance philosophy.
- **Acceptance:** Charter reviewed and merged.

### GOV-002 — Approve trust boundary

- **Component:** Governance
- **Priority:** P0
- **Dependencies:** GOV-001
- **Deliverable:** `specs/TRUST_BOUNDARY_V0.md`
- **Description:** Define trusted and untrusted artifacts with examples and taboo list.
- **Acceptance:** All checker-facing crates reference the boundary.

### GOV-003 — Define axiom policy

- **Component:** Governance
- **Priority:** P0
- **Dependencies:** GOV-002
- **Deliverable:** `specs/AXIOM_POLICY_V0.md`
- **Description:** Classify core, semantic, theory, and external axioms.
- **Acceptance:** Axiom report categories are fixed.

### GOV-004 — Define unsafe-code policy

- **Component:** Governance
- **Priority:** P0
- **Dependencies:** GOV-002
- **Deliverable:** `specs/UNSAFE_POLICY_V0.md`
- **Description:** Set no-unsafe policy for MVP kernel and checker crates.
- **Acceptance:** CI can enforce unsafe bans.

### SPEC-001 — Freeze core grammar draft

- **Component:** Spec
- **Priority:** P0
- **Dependencies:** GOV-002
- **Deliverable:** `specs/CORE_V0.md`
- **Description:** Finalize levels, terms, declarations, and defeq.
- **Acceptance:** Core grammar is stable for implementation.

### SPEC-002 — Freeze certificate schema draft

- **Component:** Spec
- **Priority:** P0
- **Dependencies:** SPEC-001
- **Deliverable:** `specs/CERT_V0.md`
- **Description:** Finalize binary layout, canonical rules, and hash domains.
- **Acceptance:** Schema has rejection conditions.

### SPEC-003 — Freeze GIR draft

- **Component:** Spec
- **Priority:** P0
- **Dependencies:** GOV-002
- **Deliverable:** `specs/GIR_V0.md`
- **Description:** Finalize GIR module/function/block/instruction schema.
- **Acceptance:** GIR has supported and rejected features.

### SPEC-004 — Freeze Go subset draft

- **Component:** Spec
- **Priority:** P0
- **Dependencies:** SPEC-003
- **Deliverable:** `specs/GO_SUBSET_V0.md`
- **Description:** Define accepted and rejected Go features.
- **Acceptance:** Unsupported behavior fails closed.

### SPEC-005 — Freeze AI API draft

- **Component:** Spec
- **Priority:** P0
- **Dependencies:** SPEC-001
- **Deliverable:** `specs/AI_API_V0.md`
- **Description:** Define ID-based proof API and diagnostics.
- **Acceptance:** API cannot bypass certificate checking.

## P1: Core Data Model

### CORE-001 — Create Rust workspace and crates

- **Component:** mpk-core
- **Priority:** P0
- **Dependencies:** SPEC-001
- **Deliverable:** `Cargo workspace`
- **Description:** Initialize mpk-core, mpk-cert, mpk-kernel, mpk-cli crates.
- **Acceptance:** Workspace builds with empty crates.

### CORE-002 — Implement level arena

- **Component:** mpk-core
- **Priority:** P0
- **Dependencies:** CORE-001
- **Deliverable:** `level.rs`
- **Description:** Add LevelId, LevelNode, normalization, hashing.
- **Acceptance:** Unit tests for level normalization pass.

### CORE-003 — Implement term arena

- **Component:** mpk-core
- **Priority:** P0
- **Dependencies:** CORE-002
- **Deliverable:** `term.rs`
- **Description:** Add TermId, TermNode, structural hashing, spine application.
- **Acceptance:** Terms are interned and topologically inspectable.

### CORE-004 — Implement names and global IDs

- **Component:** mpk-core
- **Priority:** P0
- **Dependencies:** CORE-003
- **Deliverable:** `name.rs`
- **Description:** Add canonical name validation and GlobalId resolution.
- **Acceptance:** Invalid names reject deterministically.

### CORE-005 — Implement local context

- **Component:** mpk-core
- **Priority:** P0
- **Dependencies:** CORE-003
- **Deliverable:** `context.rs`
- **Description:** Represent binders and local definitions.
- **Acceptance:** Var lookup and local definition lookup tested.

### CORE-006 — Implement substitution and lifting

- **Component:** mpk-core
- **Priority:** P0
- **Dependencies:** CORE-005
- **Deliverable:** `subst.rs`
- **Description:** Add de Bruijn-safe lift/subst operations.
- **Acceptance:** Beta substitution tests pass.

### CORE-007 — Implement weak-head normalization skeleton

- **Component:** mpk-core
- **Priority:** P0
- **Dependencies:** CORE-006
- **Deliverable:** `reduce.rs`
- **Description:** Support beta and zeta first.
- **Acceptance:** WHNF tests for lambda and let pass.

### CORE-008 — Implement structured core errors

- **Component:** mpk-core
- **Priority:** P0
- **Dependencies:** CORE-001
- **Deliverable:** `error.rs`
- **Description:** Define stable error codes and locations.
- **Acceptance:** Errors serialize to deterministic JSON.

## P2: Type Checking and Definitional Equality

### TYPE-001 — Implement environment and declarations

- **Component:** mpk-core
- **Priority:** P0
- **Dependencies:** CORE-004
- **Deliverable:** `env.rs`
- **Description:** Add Axiom, Def, Theorem skeletons.
- **Acceptance:** Declarations can be registered and looked up.

### TYPE-002 — Implement Sort inference

- **Component:** mpk-core
- **Priority:** P0
- **Dependencies:** TYPE-001
- **Deliverable:** `infer.rs`
- **Description:** Check Sort u : Sort succ(u).
- **Acceptance:** Sort tests pass.

### TYPE-003 — Implement Var and Const inference

- **Component:** mpk-core
- **Priority:** P0
- **Dependencies:** TYPE-002
- **Deliverable:** `infer.rs`
- **Description:** Check context and environment lookup.
- **Acceptance:** Var/Const tests pass.

### TYPE-004 — Implement Pi formation

- **Component:** mpk-core
- **Priority:** P0
- **Dependencies:** TYPE-003
- **Deliverable:** `infer.rs`
- **Description:** Compute sort of Pi types.
- **Acceptance:** Dependent Pi tests pass.

### TYPE-005 — Implement Lam inference/checking

- **Component:** mpk-core
- **Priority:** P0
- **Dependencies:** TYPE-004
- **Deliverable:** `infer.rs`
- **Description:** Infer/check lambda against Pi.
- **Acceptance:** Lambda tests pass.

### TYPE-006 — Implement App inference

- **Component:** mpk-core
- **Priority:** P0
- **Dependencies:** TYPE-005
- **Deliverable:** `infer.rs`
- **Description:** WHNF function type to Pi and check arguments.
- **Acceptance:** Application tests pass.

### TYPE-007 — Implement Let inference

- **Component:** mpk-core
- **Priority:** P0
- **Dependencies:** TYPE-006
- **Deliverable:** `infer.rs`
- **Description:** Check value and infer body with local definition.
- **Acceptance:** Let tests pass.

### DEFEQ-001 — Implement deterministic defeq fuel

- **Component:** mpk-core
- **Priority:** P0
- **Dependencies:** CORE-007
- **Deliverable:** `defeq.rs`
- **Description:** Add fuel budget and exhaustion rejection.
- **Acceptance:** Fuel exhaustion returns stable error.

### DEFEQ-002 — Implement delta for reducible definitions

- **Component:** mpk-core
- **Priority:** P0
- **Dependencies:** TYPE-001,DEFEQ-001
- **Deliverable:** `defeq.rs`
- **Description:** Unfold reducible defs only.
- **Acceptance:** Opaque defs never unfold.

### DEFEQ-003 — Implement theorem opacity

- **Component:** mpk-core
- **Priority:** P0
- **Dependencies:** TYPE-007,DEFEQ-002
- **Deliverable:** `decl_check.rs`
- **Description:** Check theorem body but export opaque signature.
- **Acceptance:** Downstream defeq cannot unfold theorem proof.

### DEFEQ-004 — Add negative defeq fixtures

- **Component:** mpk-core
- **Priority:** P0
- **Dependencies:** DEFEQ-003
- **Deliverable:** `fixtures/core-negative`
- **Description:** Test eta/proof-irrelevance/theorem-unfolding rejection.
- **Acceptance:** All forbidden conversions reject.

### IND-001 — Represent MVP inductive declarations

- **Component:** mpk-core
- **Priority:** P0
- **Dependencies:** TYPE-007
- **Deliverable:** `inductive.rs`
- **Description:** Add inductive family metadata, constructors, recursor signatures, universe params, and dependency validation.
- **Acceptance:** MVP Bool/Eq/Nat shapes can be registered and exported.

### IND-002 — Implement conservative positivity check

- **Component:** mpk-core
- **Priority:** P0
- **Dependencies:** IND-001
- **Deliverable:** `positivity.rs`
- **Description:** Reject negative, nested, and unknown-functor occurrences; accept only documented MVP shapes.
- **Acceptance:** Positive and negative inductive fixtures are deterministic.

### IND-003 — Generate constructor and recursor declarations

- **Component:** mpk-core
- **Priority:** P0
- **Dependencies:** IND-002
- **Deliverable:** `inductive_gen.rs`
- **Description:** Generate canonical constructor and recursor artifacts from accepted inductives.
- **Acceptance:** Generated artifacts have stable names and hashes.

### IND-004 — Implement recursor iota reduction

- **Component:** mpk-core
- **Priority:** P0
- **Dependencies:** IND-003,DEFEQ-001
- **Deliverable:** `reduce_inductive.rs`
- **Description:** Reduce only generated recursor applications under deterministic fuel.
- **Acceptance:** Generated recursor fixtures reduce and unknown recursor equations reject.

## P3: Certificate Format

### CERT-001 — Define binary tags

- **Component:** mpk-cert
- **Priority:** P0
- **Dependencies:** SPEC-002
- **Deliverable:** `binary_tags.rs`
- **Description:** Assign numeric tags for levels, terms, proof nodes, declarations.
- **Acceptance:** Tags documented and tested.

### CERT-002 — Implement canonical encoder

- **Component:** mpk-cert
- **Priority:** P0
- **Dependencies:** CERT-001
- **Deliverable:** `encode.rs`
- **Description:** Encode certificate with fixed field order and minimal varints.
- **Acceptance:** Golden encoding fixture produced.

### CERT-003 — Implement canonical decoder

- **Component:** mpk-cert
- **Priority:** P0
- **Dependencies:** CERT-002
- **Deliverable:** `decode.rs`
- **Description:** Decode and validate structural shape.
- **Acceptance:** Invalid byte fixtures reject.

### CERT-004 — Implement re-encode check

- **Component:** mpk-cert
- **Priority:** P0
- **Dependencies:** CERT-003
- **Deliverable:** `canonical.rs`
- **Description:** Reject if decoded then encoded bytes differ.
- **Acceptance:** Non-canonical fixture rejects.

### CERT-005 — Implement hash domains

- **Component:** mpk-cert
- **Priority:** P0
- **Dependencies:** CERT-002
- **Deliverable:** `hash.rs`
- **Description:** Add certificate, export, axiom, term, and level hashes.
- **Acceptance:** Hash test vectors stable.

### CERT-006 — Implement import table validation

- **Component:** mpk-cert
- **Priority:** P0
- **Dependencies:** CERT-005
- **Deliverable:** `imports.rs`
- **Description:** Sort and validate module/export/certificate hashes.
- **Acceptance:** Bad import hashes reject.

### CERT-007 — Implement export block builder

- **Component:** mpk-cert
- **Priority:** P0
- **Dependencies:** TYPE-001,CERT-005
- **Deliverable:** `export.rs`
- **Description:** Derive public interfaces from checked declarations.
- **Acceptance:** Opaque theorem proof body excluded.

### CERT-008 — Implement axiom report builder

- **Component:** mpk-cert
- **Priority:** P0
- **Dependencies:** TYPE-001,CERT-005
- **Deliverable:** `axiom_report.rs`
- **Description:** Compute direct and transitive axiom dependencies.
- **Acceptance:** Axiom fixtures match expected report.

### CERT-009 — Create minimal valid certificates

- **Component:** mpk-cert
- **Priority:** P0
- **Dependencies:** CERT-008
- **Deliverable:** `fixtures/cert-basic`
- **Description:** Add zero-axiom and one-theorem fixtures.
- **Acceptance:** Fixtures decode and hash.

## P4: Fast Rust Kernel

### KERN-001 — Create verifier driver

- **Component:** mpk-kernel
- **Priority:** P0
- **Dependencies:** CERT-004,TYPE-007
- **Deliverable:** `verifier.rs`
- **Description:** Load canonical certificate and invoke core checks.
- **Acceptance:** CLI can verify minimal cert.

### KERN-002 — Implement declaration checker orchestration

- **Component:** mpk-kernel
- **Priority:** P0
- **Dependencies:** KERN-001
- **Deliverable:** `decl_driver.rs`
- **Description:** Check declarations in dependency order.
- **Acceptance:** Out-of-order dependencies reject.

### KERN-003 — Implement bootstrap proof-node checker

- **Component:** mpk-kernel
- **Priority:** P0
- **Dependencies:** KERN-002
- **Deliverable:** `proof_check.rs`
- **Description:** Check Exact, Apply, Intro, Refl, and Conv under the core-bootstrap profile; reject other node tags by profile.
- **Acceptance:** Bootstrap proof-node fixtures pass.

### KERN-004 — Implement checker caches

- **Component:** mpk-kernel
- **Priority:** P0
- **Dependencies:** KERN-003
- **Deliverable:** `cache.rs`
- **Description:** Cache inferred types, WHNF, and defeq results.
- **Acceptance:** Cache does not change verdicts.

### KERN-005 — Implement structured JSON output

- **Component:** mpk-kernel
- **Priority:** P0
- **Dependencies:** KERN-001
- **Deliverable:** `json_output.rs`
- **Description:** Emit verdict, hashes, axiom report, error code.
- **Acceptance:** Output stable under snapshot tests.

### KERN-006 — Implement mpk check

- **Component:** mpk-cli
- **Priority:** P0
- **Dependencies:** KERN-005
- **Deliverable:** `mpk check`
- **Description:** Command to verify one certificate.
- **Acceptance:** Valid and invalid fixtures handled.

### KERN-007 — Implement mpk axiom-report

- **Component:** mpk-cli
- **Priority:** P0
- **Dependencies:** CERT-008,KERN-005
- **Deliverable:** `mpk axiom-report`
- **Description:** Command to print recomputed axiom report.
- **Acceptance:** Report matches certificate hash.

### KERN-008 — Fuzz certificate decoder boundary

- **Component:** mpk-kernel
- **Priority:** P0
- **Dependencies:** CERT-003,KERN-001
- **Deliverable:** `fuzz target`
- **Description:** Run fuzz target against malformed inputs.
- **Acceptance:** No panics on malformed inputs.

### KERN-009 — Implement structural proof-node checker

- **Component:** mpk-kernel
- **Priority:** P0
- **Dependencies:** KERN-003,IND-004
- **Deliverable:** `proof_structural.rs`
- **Description:** Check LetProof, Rewrite, EqRec, Constructor, and Recursor nodes; reject unsupported nodes by active profile.
- **Acceptance:** Structural proof-node fixtures pass.

## P5: Independent Go Reference Checker

### REF-001 — Create Go checker module

- **Component:** mpk-checker-ref
- **Priority:** P0
- **Dependencies:** SPEC-002
- **Deliverable:** `go.mod`
- **Description:** Initialize clean-room Go module.
- **Acceptance:** Module builds independently.

### REF-002 — Implement Go decoder

- **Component:** mpk-checker-ref
- **Priority:** P0
- **Dependencies:** REF-001,CERT-001
- **Deliverable:** `decoder.go`
- **Description:** Decode canonical certificate bytes without Rust code.
- **Acceptance:** Positive/negative decode fixtures pass.

### REF-003 — Implement hash recomputation

- **Component:** mpk-checker-ref
- **Priority:** P0
- **Dependencies:** REF-002,CERT-005
- **Deliverable:** `hash.go`
- **Description:** Recompute certificate/export/axiom hashes.
- **Acceptance:** Hash vectors match Rust.

### REF-004 — Implement core term checker

- **Component:** mpk-checker-ref
- **Priority:** P0
- **Dependencies:** REF-002,SPEC-001
- **Deliverable:** `core_check.go`
- **Description:** Implement MVP term typing.
- **Acceptance:** Core fixtures match Rust verdicts.

### REF-005 — Implement defeq

- **Component:** mpk-checker-ref
- **Priority:** P0
- **Dependencies:** REF-004
- **Deliverable:** `defeq.go`
- **Description:** Implement deterministic WHNF and defeq.
- **Acceptance:** Defeq fixtures match Rust.

### REF-006 — Implement proof-node checker

- **Component:** mpk-checker-ref
- **Priority:** P0
- **Dependencies:** REF-005,KERN-009
- **Deliverable:** `proof_check.go`
- **Description:** Check bootstrap and structural MVP proof node rules.
- **Acceptance:** Proof fixtures match Rust.

### REF-007 — Implement axiom report verifier

- **Component:** mpk-checker-ref
- **Priority:** P0
- **Dependencies:** REF-006,CERT-008
- **Deliverable:** `axiom_report.go`
- **Description:** Recompute and compare axiom report.
- **Acceptance:** Axiom fixtures match Rust.

### REF-008 — Add checker agreement tests

- **Component:** ci
- **Priority:** P0
- **Dependencies:** REF-007,KERN-006
- **Deliverable:** `CI job`
- **Description:** Run Rust and Go checker against same corpus.
- **Acceptance:** Any disagreement fails CI.

## P6: Standard Library Foundation

### STD-001 — Define implication, conjunction, disjunction interfaces

- **Component:** Std.Logic
- **Priority:** P1
- **Dependencies:** KERN-006,KERN-009
- **Deliverable:** `proofs/std/logic`
- **Description:** Create core logical constants/inductives needed by VCs.
- **Acceptance:** Certificates verify.

### STD-002 — Define equality and rewrite lemmas

- **Component:** Std.Eq
- **Priority:** P1
- **Dependencies:** STD-001
- **Deliverable:** `proofs/std/eq`
- **Description:** Add refl, symm, trans, congruence MVP.
- **Acceptance:** Rewrite fixtures verify.

### STD-003 — Define Bool and boolean normalization

- **Component:** Std.Bool
- **Priority:** P1
- **Dependencies:** STD-001
- **Deliverable:** `proofs/std/bool`
- **Description:** Add true/false, if, and/or/not.
- **Acceptance:** Bool certificates verify.

### STD-004 — Define minimal Nat

- **Component:** Std.Nat
- **Priority:** P1
- **Dependencies:** STD-002
- **Deliverable:** `proofs/std/nat`
- **Description:** Add zero, succ, basic order if needed.
- **Acceptance:** Nat fixtures verify.

### STD-005 — Define Int interface

- **Component:** Std.Int
- **Priority:** P1
- **Dependencies:** STD-002
- **Deliverable:** `proofs/std/int`
- **Description:** Add integer order and linear arithmetic hooks.
- **Acceptance:** Axiom use reviewed.

### STD-006 — Define BitVec interfaces

- **Component:** Std.BitVec
- **Priority:** P1
- **Dependencies:** STD-003
- **Deliverable:** `proofs/std/bitvec`
- **Description:** Add BV8/BV16/BV32/BV64 operations and signed views.
- **Acceptance:** Ground eval fixtures verify.

### STD-007 — Define fixed-array interface

- **Component:** Std.Array.Fixed
- **Priority:** P1
- **Dependencies:** STD-006
- **Deliverable:** `proofs/std/array`
- **Description:** Add read, write, length, index safety predicates.
- **Acceptance:** Read/write fixtures verify.

### STD-008 — Define Go semantic base types

- **Component:** Std.Go.Base
- **Priority:** P1
- **Dependencies:** STD-006,STD-007
- **Deliverable:** `proofs/go/base`
- **Description:** Map Go bool/int/uint/array/struct to core types.
- **Acceptance:** Go base certs verify.

## P7: Go Frontend and GIR

### GO-001 — Create Go frontend CLI

- **Component:** go2gir
- **Priority:** P1
- **Dependencies:** SPEC-003,SPEC-004
- **Deliverable:** `go2gir`
- **Description:** Initialize go2gir command.
- **Acceptance:** CLI accepts package path.

### GO-002 — Load Go packages

- **Component:** go2gir
- **Priority:** P1
- **Dependencies:** GO-001
- **Deliverable:** `loader.go`
- **Description:** Use Go package loading with pinned settings.
- **Acceptance:** Can load sample package.

### GO-003 — Build SSA

- **Component:** go2gir
- **Priority:** P1
- **Dependencies:** GO-002
- **Deliverable:** `ssa.go`
- **Description:** Build SSA for target package.
- **Acceptance:** SSA dump available for sample.

### GO-004 — Implement feature detector

- **Component:** go2gir
- **Priority:** P1
- **Dependencies:** GO-003,SPEC-004
- **Deliverable:** `features.go`
- **Description:** Reject unsupported Go features with exact reason.
- **Acceptance:** Unsupported fixtures reject.

### GO-005 — Lower pure functions to GIR

- **Component:** go2gir
- **Priority:** P1
- **Dependencies:** GO-004
- **Deliverable:** `lower.go`
- **Description:** Convert params, locals, blocks, binops, if, return.
- **Acceptance:** Max64 lowers to GIR.

### GO-006 — Lower structs and fixed arrays

- **Component:** go2gir
- **Priority:** P1
- **Dependencies:** GO-005
- **Deliverable:** `lower_types.go`
- **Description:** Support field and fixed-array indexing.
- **Acceptance:** Struct/array fixtures lower.

### GO-007 — Emit canonical GIR JSON/binary

- **Component:** go2gir
- **Priority:** P1
- **Dependencies:** GO-005
- **Deliverable:** `emit.go`
- **Description:** Stable field order and hashes.
- **Acceptance:** GIR hash deterministic.

### GO-008 — Emit source manifest

- **Component:** go2gir
- **Priority:** P1
- **Dependencies:** GO-007
- **Deliverable:** `manifest.go`
- **Description:** Record source hashes, Go version, frontend hash.
- **Acceptance:** Manifest stable and complete.

### GO-009 — Create Go frontend corpus

- **Component:** fixtures
- **Priority:** P1
- **Dependencies:** GO-008
- **Deliverable:** `fixtures/go-basic`
- **Description:** Add positive and negative subset examples.
- **Acceptance:** CI covers Go subset.

### GO-010 — Validate contract sidecars

- **Component:** go2gir
- **Priority:** P1
- **Dependencies:** GO-007,SPEC-004
- **Deliverable:** `contract_sidecar.go`
- **Description:** Parse mpk.go.contract.v0 JSON, resolve function identities, attach requires/ensures/modifies/loop metadata to GIR, and reject unsupported expression operators.
- **Acceptance:** Sample contract validates and malformed or unsupported contracts reject.

## P8: VC Generator

### VC-001 — Create VC generator crate

- **Component:** mpk-vc
- **Priority:** P1
- **Dependencies:** GO-010,STD-008
- **Deliverable:** `mpk-vc`
- **Description:** Initialize GIR importer and VC data model.
- **Acceptance:** Crate builds.

### VC-002 — Encode GIR types into MPK terms

- **Component:** mpk-vc
- **Priority:** P1
- **Dependencies:** VC-001
- **Deliverable:** `type_encode.rs`
- **Description:** Map Go bool/int/uint/struct/array types.
- **Acceptance:** Type encoding snapshots pass.

### VC-003 — Encode expressions into MPK terms

- **Component:** mpk-vc
- **Priority:** P1
- **Dependencies:** VC-002
- **Deliverable:** `expr_encode.rs`
- **Description:** Map constants, variables, binops, comparisons.
- **Acceptance:** Expression fixtures pass.

### VC-004 — Generate straight-line VCs

- **Component:** mpk-vc
- **Priority:** P1
- **Dependencies:** VC-003
- **Deliverable:** `wp.rs`
- **Description:** Handle pure return functions.
- **Acceptance:** Simple functions produce VCs.

### VC-005 — Generate branch VCs

- **Component:** mpk-vc
- **Priority:** P1
- **Dependencies:** VC-004
- **Deliverable:** `wp_branch.rs`
- **Description:** Handle if/else path conditions.
- **Acceptance:** Max64 VCs produced.

### VC-006 — Generate runtime-safety VCs

- **Component:** mpk-vc
- **Priority:** P1
- **Dependencies:** VC-005
- **Deliverable:** `safety.rs`
- **Description:** Division by zero, shift count, index bounds.
- **Acceptance:** Unsafe operations produce obligations.

### VC-007 — Generate loop invariant VCs

- **Component:** mpk-vc
- **Priority:** P1
- **Dependencies:** VC-006
- **Deliverable:** `loops.rs`
- **Description:** Initial, preservation, exit, and optional total-correctness variant obligations.
- **Acceptance:** Annotated loop fixture produces expected partial-correctness VCs and total-correctness variants when requested.

### VC-008 — Emit theorem obligations

- **Component:** mpk-vc
- **Priority:** P1
- **Dependencies:** VC-007,CERT-002
- **Deliverable:** `obligation_emit.rs`
- **Description:** Create core declarations for VCs.
- **Acceptance:** VC cert skeletons created.

### VC-009 — Implement Max64 end-to-end example

- **Component:** examples
- **Priority:** P1
- **Dependencies:** VC-008
- **Deliverable:** `examples/max64`
- **Description:** Go source -> GIR -> VC theorem obligations.
- **Acceptance:** Example documented.

## P9: AI Proof API

### API-001 — Create API crate/server

- **Component:** mpk-api
- **Priority:** P1
- **Dependencies:** SPEC-005,KERN-006
- **Deliverable:** `mpk-api`
- **Description:** Provide local server or library API.
- **Acceptance:** Can start session.

### API-002 — Implement term construction endpoints

- **Component:** mpk-api
- **Priority:** P1
- **Dependencies:** API-001
- **Deliverable:** `term_api.rs`
- **Description:** Sort/Var/Const/App/Lam/Pi/Let over interned IDs.
- **Acceptance:** Term API integration tests pass.

### API-003 — Implement bootstrap proof construction endpoints

- **Component:** mpk-api
- **Priority:** P1
- **Dependencies:** API-002,KERN-003
- **Deliverable:** `proof_api.rs`
- **Description:** Exact/Apply/Intro/Refl/Conv core-bootstrap endpoints.
- **Acceptance:** Proof API can build simple theorem.

### API-004 — Implement check-node endpoint

- **Component:** mpk-api
- **Priority:** P1
- **Dependencies:** API-003
- **Deliverable:** `check_api.rs`
- **Description:** Validate individual proof nodes.
- **Acceptance:** Bad nodes return structured errors.

### API-005 — Implement batch candidate checking

- **Component:** mpk-api
- **Priority:** P1
- **Dependencies:** API-004
- **Deliverable:** `batch.rs`
- **Description:** Check many candidates and return per-candidate verdict.
- **Acceptance:** 10k fake candidates handled.

### API-006 — Implement repair diagnostics

- **Component:** mpk-api
- **Priority:** P1
- **Dependencies:** API-004
- **Deliverable:** `diagnostics.rs`
- **Description:** Return expected/actual head, local context, hints.
- **Acceptance:** Diagnostics snapshots stable.

### API-007 — Implement JSONL import/export

- **Component:** mpk-api
- **Priority:** P1
- **Dependencies:** API-005
- **Deliverable:** `jsonl.rs`
- **Description:** Support offline AI generation workflows.
- **Acceptance:** JSONL round trip works.

### API-008 — Add minimal automated strategies

- **Component:** mpk-api
- **Priority:** P1
- **Dependencies:** API-006
- **Deliverable:** `strategies.rs`
- **Description:** Try exact, refl, split, and apply in safe order without theory shortcuts.
- **Acceptance:** Simple propositional and equality fixtures prove with strategies.

## P10: Theory Certificates

### TH-001 — Implement Bool normalization checker

- **Component:** theory-bool
- **Priority:** P1
- **Dependencies:** STD-003,KERN-003
- **Deliverable:** `bool_cert.rs`
- **Description:** Check small boolean tautology certificates.
- **Acceptance:** Malformed certs reject.

### TH-002 — Implement BitVec ground evaluator

- **Component:** theory-bitvec
- **Priority:** P1
- **Dependencies:** STD-006,KERN-003
- **Deliverable:** `bitvec_eval.rs`
- **Description:** Normalize ground BV expressions.
- **Acceptance:** Ground BV fixtures pass.

### TH-003 — Define BitVec certificate schema

- **Component:** theory-bitvec
- **Priority:** P1
- **Dependencies:** TH-002
- **Deliverable:** `bitvec_cert.md`
- **Description:** Schema for operation trace and normalized result.
- **Acceptance:** Schema reviewed.

### TH-004 — Implement linear arithmetic certificate checker

- **Component:** theory-int
- **Priority:** P1
- **Dependencies:** STD-005,KERN-003
- **Deliverable:** `linarith_cert.rs`
- **Description:** Start with simple Farkas-style non-strict inequalities.
- **Acceptance:** Linear fixtures pass.

### TH-005 — Implement array read/write certificate checker

- **Component:** theory-array
- **Priority:** P1
- **Dependencies:** STD-007,KERN-003
- **Deliverable:** `array_cert.rs`
- **Description:** Check fixed-array read-over-write reasoning.
- **Acceptance:** Array fixtures pass.

### TH-006 — Integrate Theory proof node

- **Component:** theory
- **Priority:** P1
- **Dependencies:** TH-001,TH-002,TH-004,TH-005
- **Deliverable:** `proof_theory.rs`
- **Description:** Connect theory certificates to proof-node checker.
- **Acceptance:** Theory node fixtures verify.

### TH-007 — Fuzz malformed theory certificates

- **Component:** theory
- **Priority:** P1
- **Dependencies:** TH-006
- **Deliverable:** `fuzz targets`
- **Description:** Fuzz each theory checker.
- **Acceptance:** No panics and deterministic rejects.

### TH-008 — Add theory-backed proof strategy hook

- **Component:** mpk-api
- **Priority:** P1
- **Dependencies:** TH-006,API-008
- **Deliverable:** `theory_strategy.rs`
- **Description:** Enable API strategy dispatcher to use checked theory certificate builders for enabled theory profiles.
- **Acceptance:** Max64 simple VCs prove only through checked theory certificates.

## P11: CI and Package Verification

### CI-001 — Create fast local gate

- **Component:** ci
- **Priority:** P2
- **Dependencies:** KERN-006
- **Deliverable:** `scripts/check-fast.sh`
- **Description:** Rust fmt, clippy, unit tests, small fixtures.
- **Acceptance:** Fast gate passes locally.

### CI-002 — Create reference-checker gate

- **Component:** ci
- **Priority:** P2
- **Dependencies:** REF-008
- **Deliverable:** `scripts/check-reference.sh`
- **Description:** Run Go checker on fixture certificates.
- **Acceptance:** Reference gate passes.

### CI-003 — Create full corpus gate

- **Component:** ci
- **Priority:** P2
- **Dependencies:** VC-009,TH-008
- **Deliverable:** `scripts/check-all.sh`
- **Description:** Run frontend, VC generator, both checkers, hash checks.
- **Acceptance:** Full gate reproducible.

### CI-004 — Define package manifest

- **Component:** package
- **Priority:** P2
- **Dependencies:** CERT-006
- **Deliverable:** `package-manifest.md`
- **Description:** Module name, imports, cert paths, expected hashes.
- **Acceptance:** Manifest fixtures validate.

### CI-005 — Define lock file

- **Component:** package
- **Priority:** P2
- **Dependencies:** CI-004
- **Deliverable:** `package-lock.md`
- **Description:** Hash-pinned imports and checker policy.
- **Acceptance:** Lock verification works.

### CI-006 — Implement package check command

- **Component:** cli
- **Priority:** P2
- **Dependencies:** CI-004,KERN-006
- **Deliverable:** `mpk package check`
- **Description:** Validate package manifest and imports.
- **Acceptance:** Bad package rejects.

### CI-007 — Implement package verify-certs command

- **Component:** cli
- **Priority:** P2
- **Dependencies:** CI-006,REF-008
- **Deliverable:** `mpk package verify-certs`
- **Description:** Run all certificates through both checkers if configured.
- **Acceptance:** Fixture package verifies.

### CI-008 — Generate release evidence report

- **Component:** release
- **Priority:** P2
- **Dependencies:** CI-007
- **Deliverable:** `release-report.json`
- **Description:** Collect versions, hashes, axiom reports, checker verdicts.
- **Acceptance:** Report stable and complete.

## P12: Alpha Corpus and Hardening

### ALPHA-001 — Create 100-function Go corpus

- **Component:** corpus
- **Priority:** P2
- **Dependencies:** VC-009
- **Deliverable:** `fixtures/go-alpha`
- **Description:** Small pure functions covering arithmetic, branches, arrays.
- **Acceptance:** Corpus compiles and lowers.

### ALPHA-002 — Generate 1,000+ VCs

- **Component:** corpus
- **Priority:** P2
- **Dependencies:** ALPHA-001,VC-008
- **Deliverable:** `fixtures/vc-alpha`
- **Description:** Expand contracts and branch cases.
- **Acceptance:** VC count and hash recorded.

### ALPHA-003 — Generate invalid candidate benchmark

- **Component:** api
- **Priority:** P2
- **Dependencies:** API-005
- **Deliverable:** `bench/invalid-candidates`
- **Description:** Produce 10,000 invalid proof candidates.
- **Acceptance:** Benchmark corpus available.

### ALPHA-004 — Profile fast kernel

- **Component:** perf
- **Priority:** P2
- **Dependencies:** ALPHA-002,KERN-004
- **Deliverable:** `perf report`
- **Description:** Measure decode, typecheck, defeq, proof-node checking.
- **Acceptance:** Hotspots identified.

### ALPHA-005 — Optimize without changing trust boundary

- **Component:** perf
- **Priority:** P2
- **Dependencies:** ALPHA-004
- **Deliverable:** `optimization PRs`
- **Description:** Improve caches/arena layout only.
- **Acceptance:** Checker agreement maintained.

### ALPHA-006 — Write alpha demo guide

- **Component:** docs
- **Priority:** P2
- **Dependencies:** ALPHA-002,TH-008
- **Deliverable:** `docs/alpha-demo.md`
- **Description:** End-to-end Go source to certificate verification.
- **Acceptance:** Guide reproduces locally.

### ALPHA-007 — Run alpha release gates

- **Component:** release
- **Priority:** P2
- **Dependencies:** CI-008,ALPHA-006
- **Deliverable:** `alpha-release-report`
- **Description:** Apply all gates from RELEASE_GATES.md.
- **Acceptance:** All gates pass or exceptions documented.
