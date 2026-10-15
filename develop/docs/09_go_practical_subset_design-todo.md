# Go Practical Subset Implementation Work Plan

Status: implementation authorized 2026-10-16. T01-W01 and W02 are complete;
T01-W03 is In progress. Its representation pilots and balanced unsigned /
conditional signed addition bridges have both-checker evidence. Exact prefix
sums, complete operation/binding/collection laws and resource boundaries
remain open, so W04 has not started.
The [implementation ledger](go-practical-implementation-traceability-ledger.md)
owns detailed current status and verification evidence.
The user-authorized amendment moves Go before the practical C# release.
Writing and reviewing this plan does not complete T01 or authorize a new
public profile. The scope and semantic decisions are
owned by [the design](09_go_practical_subset_design.md).

## 1. Entry, ordering, and execution

Go practical implementation may start before the practical C# release, as
explicitly authorized by the user on 2026-10-16. T01-W01 records the amendment,
an exact clean source baseline, and the last installed Go/Rust/scalar-C#/Java
release receipt. The historical receipt proves only its own release, not all
later source changes. The current C#
[work-plan status](08_csharp_practical_subset_design-todo.md) includes reopened
foundation work; it is neither a completed prerequisite nor proof of shared
facilities. Go validates every facility it needs through its own gates.
Remaining practical C# work resumes after GO-PRAC and reconciles any intervening
shared-artifact changes. Production language phases remain serial.

Execute T01 through T11 serially, and W01 through the last W serially within
each T. Each W has a concrete review boundary. Multiple requested work items
can be carried out in order; passing a work-item gate does not require a new
user confirmation. Do not begin the next T before the preceding T's gate
passes. A failed feasibility or proof gate requires a recorded correction to
the affected design/freeze; it must not be bypassed with an assumption.

| Stage | Scope | Priority | Result |
| --- | --- | --- | --- |
| GO-PRAC-T01 | Feasibility, profile v1 and shared-contract freeze | Prerequisite | Frozen inactive package and evidence |
| GO-PRAC-T02 | Dependency capture and selected-function closure | Prerequisite | Private existing-module integration |
| GO-PRAC-T03 | Types, constants, and ordinary syntax | 1 | Private domain-value verification |
| GO-PRAC-T04 | Multi-result calls and closed errors | 2 | Private success/failure verification |
| GO-PRAC-T05 | Bounded byte strings | 3 | Private identifier/text verification |
| GO-PRAC-T06 | Slices, construction, and structured loops | 4 | Private collection verification |
| GO-PRAC-T07 | Policy integration and first release | 1–4 | Public practical v1 |
| GO-PRAC-T08 | Profile v2 and successor-contract freeze | Prerequisite | Frozen inactive v2 package |
| GO-PRAC-T09 | Bounded maps | 5 | Private membership/update verification |
| GO-PRAC-T10 | Pointers, receivers, and state transitions | 6 | Private state-transition verification |
| GO-PRAC-T11 | Second release and final regression closure | 5–6 | Public practical v2 |

At T01-W01 create `go-practical-implementation-traceability-ledger.md` beside
this plan. It records each W's status, predecessor, reviewed commit, exact
inputs and hashes, specification rows, implementation/test owners, commands,
results, outstanding findings, and deferred checks. The ledger becomes the
implementation status authority; the unchecked work list below is a plan.

## 2. Common completion rule and verification routing

Every implementation W includes the code or freeze change, affected tests,
source/contract/VIR/VC/proof evidence where applicable, documentation, and
review fixes. A semantic definition certificate is not an application proof.
Each feature-stage final W needs a nontrivial source-to-certificate example,
all requested claims discharged, both-checker acceptance, and a deliberately
incorrect case that fails at the intended boundary.

Run targeted checks during W items. The final W of every T additionally runs
`./scripts/check-fast.sh` for that T's cumulative changes and cannot complete
until it passes. Do not reproduce that whole gate during intermediate W
items. Record why the selected tests cover the change and explicitly record
the full gate as deferred until the final W. T07 and T11 also run the complete
local native Linux release gates specified by the root README, with two-pass
installed-release evidence. All execution stays local; no workflow files or
GitHub Actions are introduced or used.

| Changed surface | Targeted validation family |
| --- | --- |
| Freeze/specification | Strict schema/vector validation, ownership inventory, `python3 scripts/check-spec-vectors.py --check` after normative publication |
| Capture and loading | Go preflight, snapshot, loader and newly added module/closure tests; module graph tampering and offline replay |
| Go typing/lowering | New practical-profile source tests plus affected existing Go-profile and corpus tests |
| Shared artifacts | Independent import, context/hash/type/effect negatives, each inventoried consumer and predecessor migration |
| VC/proof | Exact path/obligation cases, checked binding/lemma proofs, application theorem fixtures, both source-free checkers |
| Policy/API | Readiness/claim status, pending obligations, assumptions, rejected or stale evidence, exact strategy registration |
| Release | Registered-runner isolation, build-input and output replay, target/identity crossing, current README Linux gates |

Existing executable anchors include `go-tools/go2vir` tests,
`crates/mpk-vc/tests/go_vir_corpus.rs`, and
`crates/mpk-cli/tests/successor_atomic_cutover.rs`. The freeze inventories name
the new primary executors before implementation. Avoid running every existing
test family for a local change without an impact reason.

## 3. GO-PRAC-T01 — Feasibility and v1 freeze

Entry: the user-authorized pre-C# order, the last installed four-language
release, and an audited clean source baseline. Practical C# completion is not
required. T01-W01 must distinguish historical release evidence from the
current source state; T01-W05 still requires its cumulative whole gate.

| Work item | Deliverable and acceptance evidence |
| --- | --- |
| T01-W01 | Record the pre-C# schedule amendment, exact clean source baseline and last installed-release receipt with their distinct scopes, new ledger, current consumer inventory, and fixed-profile regression corpus. Pin the proposed target/compiler and supported effective-language-version table. |
| T01-W02 | Probe original-package type checking with unused unsupported functions, external modules, replacements, module metadata, build tags, initializers, and reachable global effects. Produce a capture/verification-closure matrix and exact offline snapshot contract. |
| T01-W03 | Demonstrate ordinary encodings for nominal values, multi-results/errors, bounded bytes, slice descriptors/stores and abrupt loop control. Both checkers must accept representative conditional lemmas, not only definitions. Measure the design's v1 workload points and reject unsound projections. |
| T01-W04 | Freeze v1 source admission/rejection, operation/check table, closure/effect rules, contract expressions, exact numeric limits/budgets, all changed schema/hash identities, producer/consumer migrations, and positive/negative vectors. Assign every requirement a production executor. |
| T01-W05 | Publish the reviewed inactive specification package and manifest entries, review source-model binding and proof feasibility, run targeted freeze checks and the full T gate. Keep public selection and the installed registry unchanged. |

Proposed specification owners, whose exact names are frozen in W04:
`GO_PRACTICAL_PROFILE_V1.md`, a Go practical foundation specification, and a
shared-artifact delta specification if required. These are future files, not
existing normative references. Kernel feasibility must not be inferred from
the C# design alone.

Exit: a measured, implementable frozen v1 scope with zero unresolved semantic
questions. Any proposed workload above a viable bound is explicitly excluded
or redesigned before freeze, rather than advertised as supported.

## 4. GO-PRAC-T02 — Captured modules and function selection

| Work item | Deliverable and acceptance evidence |
| --- | --- |
| T02-W01 | Implement private profile/context contracts and immutable dependency capture. Verify exact module resolution, replacement roots, source/export metadata and compiler identities; reject path escape, missing snapshots, stale checksums, and forbidden network/toolchain fallback. |
| T02-W02 | Implement original-package loading and the two closure algorithms. Record every declaration's disposition and root-to-rejection path. Prove by tests that unused handlers are allowed while reachable unsupported helpers, globals, callbacks and cycles reject. |
| T02-W03 | Implement the frozen common artifact migration, nominal cross-package identity, overlay linkage and independent consumer validation. Exercise all affected existing languages and reject mixed versions or caller-supplied adapters. |
| T02-W04 | Run a scalar policy end to end inside a captured multi-package application with an unused external dependency. Verify conditional scope at function entry, both-checker proof evidence, deterministic replay, old Go behavior, and the full T gate. |

Exit: dependency support and selection work through real private consumers.
A source-extraction script or a manufactured stub package is insufficient.

## 5. GO-PRAC-T03 — Domain types and ordinary syntax

| Work item | Deliverable and acceptance evidence |
| --- | --- |
| T03-W01 | Implement target-sized integers, nominal primitive/array types, aliases, cross-package structs/constants, grouped/inferred constants and `iota`. Test target width, representability and identity confusion. |
| T03-W02 | Implement zero-filled literals, local value updates, compound assignment, increments, parameter local copies, `else if`, expression switches and expanded short-circuit contexts with faithful source origins. Test source evaluation and case ordering. |
| T03-W03 | Connect typed operations, VC generation and proof recipes. Add switch/amount/aggregate examples and negatives for overflow-as-integer reasoning and duplicated target evaluation. |
| T03-W04 | Close source-to-certificate coverage for the priority-1 matrix, compare pinned Go observations, check both verifiers and regressions, review findings, and pass the full T gate. |

## 6. GO-PRAC-T04 — Multiple results and closed errors

| Work item | Deliverable and acceptance evidence |
| --- | --- |
| T04-W01 | Implement zero-to-16-result products and projections, single call execution, joint contract results, tuple/parallel assignment, `:=` shadowing, blank destinations, forwarding and normalized named returns. |
| T04-W02 | Implement the closed non-pointer error universe and exact nil-interface cases. Treat `Error()` as a signature dependency when unused; reject open dispatch, unlisted dynamic types and pointer errors. |
| T04-W03 | Generate branch-specific result obligations and checked call composition. Verify `(Amount, bool)` and `(Decision, error)` examples, including a valid nonzero first result on failure and incorrect tuple/result binding. |
| T04-W04 | Complete differential observations, independent import negatives, source-free application proofs and the full T gate. Verify that proof failure is not reported as a compiler rejection or success. |

## 7. GO-PRAC-T05 — Bounded byte strings

| Work item | Deliverable and acceptance evidence |
| --- | --- |
| T05-W01 | Implement the frozen byte carrier, literals/zero values and operations: equality/order, length, indexing, substring and bounded concatenation. Preserve arbitrary bytes and exact nominal types. |
| T05-W02 | Connect checked representation/binding laws, safety and result-bound obligations, plus exact-symbol prefix/suffix adapters. Verify the library binding rather than trusting package names. |
| T05-W03 | Verify identifier-validation and selected `Error()` functions. Exercise NUL, invalid UTF-8, multibyte strings, endpoint bounds and failed concatenation-capacity claims. |
| T05-W04 | Close both-checker application proofs, measured capacity/budget cases, old-profile and affected-consumer tests, review and full T gate. |

## 8. GO-PRAC-T06 — Slices and structured loops

| Work item | Deliverable and acceptance evidence |
| --- | --- |
| T06-W01 | Implement read-only descriptors/stores, nil/empty distinction, length/capacity, index and slice expressions, array copy/view behavior, and alias-preserving import. Include observable data beyond logical length. |
| T06-W02 | Implement fresh bounded construction, local alias updates, overlap-correct copy, explicit-capacity make and non-reallocating append. Prove region separation and construction bounds; reject hidden input writes. |
| T06-W03 | Implement array/slice range, condition-only and nested loops, break/continue and early return, with effective-language-version bindings and exact source contract attachment. |
| T06-W04 | Generate and discharge initialization, preservation, exit, postcondition and totality obligations. Add counterexamples for copied-list alias semantics, wrong continue/post behavior and omitted return paths. |
| T06-W05 | Verify the design's `Quote` and preallocated-filter examples end to end, including mathematical-sum binding, all failure paths and no input mutation. Measure actual proof/checker costs, close review and pass the full T gate. |

Exit: the first product milestone works privately in an existing module.
There are no pending required application theorems behind a “supported” label.

## 9. GO-PRAC-T07 — Practical v1 public release

| Work item | Deliverable and acceptance evidence |
| --- | --- |
| T07-W01 | Complete the practical-Go strategy, CLI/API and evidence routes for all v1 obligations. Display exact entry bounds, selected functions and remaining obligations; reject old-strategy/context substitution. |
| T07-W02 | Build registered bundles, installation inventory and replay corpus. Demonstrate unchanged application builds without MPK dependencies and preserve fixed-Go semantic behavior under the common artifact migration. |
| T07-W03 | Rehearse atomic activation, wrong-version rejection and rollback from reviewed bundles. Run installed native Linux isolation and release checks, with two matching complete receipts. |
| T07-W04 | Apply the reviewed v1 registry/consumer activation atomically, finish documentation and regression review, pass the full T gate and required final release checks, and record the exact shipped scope. |

Exit: v1 is a public immutable profile with a useful, completely checked
business-function corpus. Missing Linux evidence prevents release completion;
a local host that cannot run the gate does not substitute a simulated receipt.

## 10. GO-PRAC-T08 — v2 feasibility and freeze

Entry: the practical v1 release is complete.

| Work item | Deliverable and acceptance evidence |
| --- | --- |
| T08-W01 | Capture v1 as the preservation baseline. Inventory map references, root pointers, permissions, frame observations, typed-nil errors and every affected shared consumer. |
| T08-W02 | Probe Go alias/nil/receiver observations and ordinary map/heap encodings. Both checkers must accept representative lookup-update and state/frame lemmas. Measure the map/root-object workload points and reject ordered-map substitutions. |
| T08-W03 | Freeze the immutable v2 profile, restricted key/value/pointer matrices, effect and ownership rules, contract additions, exact bounds/budgets, adapter rules, schemas/hash changes and vectors. Publish the inactive package and pass the full T gate. |

Exit: v1 remains immutable; no map/pointer form is admitted via a v1 feature
flag. Any shared migration has a complete existing-consumer preservation plan.

## 11. GO-PRAC-T09 — Bounded maps

| Work item | Deliverable and acceptance evidence |
| --- | --- |
| T09-W01 | Implement map handles, finite mappings, admitted keys/values, nil/empty states, lookup and comma-ok projection with nominal/type and region validation. |
| T09-W02 | Implement fresh/exclusive map construction, aliased insert/replace/delete and explicit footprint state. Generate cardinality and unrelated-key/frame obligations. |
| T09-W03 | Verify membership and deduplication/update examples. Test nil writes, zero versus missing, alias-visible updates, capacity boundary, and map-range rejection; prove the canonical carrier cannot introduce source ordering. |
| T09-W04 | Close source-to-certificate proofs, performance/resource measurements, independent corruption cases, v1/fixed-Go regressions, review and the full T gate. |

## 12. GO-PRAC-T10 — Pointers and mutable receivers

| Work item | Deliverable and acceptance evidence |
| --- | --- |
| T10-W01 | Implement nonzero-sized root objects, pointer/nil identity, local allocation/address taking and dereference. Validate permitted type graphs and reject interior addresses, unsafe paths and unsupported escapes. |
| T10-W02 | Implement direct pointer receivers, local aliases, effect/permission transfer, writable-root exclusivity and automatic address/dereference normalization where admitted. Test nil-safe methods and interfering expression rejection. |
| T10-W03 | Implement `old`, modifies/frame and state invariants, joint result/post-state contracts, and pointer concrete errors with typed-nil preservation. Prove callee effects instead of trusting annotations. |
| T10-W04 | Verify guarded account reservation, both success and unchanged-state failure. Include overlapping root inputs, hidden writes, frame violations, failure after mutation and incorrectly collapsed typed nil. |
| T10-W05 | Close all required application proofs, both-checker costs, independent reconstruction and predecessor regressions, review and the full T gate. |

## 13. GO-PRAC-T11 — Practical v2 public release

| Work item | Deliverable and acceptance evidence |
| --- | --- |
| T11-W01 | Complete v2 policy/API/evidence and registered bundles, with precise ownership and caller obligations. Verify fixed-Go and v1 preservation and reject crossed profiles or effects. |
| T11-W02 | Rehearse atomic shared-artifact/profile activation and rollback, run the complete real-module corpus and native Linux two-pass release gates, and close release findings. |
| T11-W03 | Activate the reviewed v2 release, document the exact map/pointer limits and deferred forms, pass the full T and final release gates, and publish the final traceability ledger and receipts. |

Exit: both practical releases have reproducible proof evidence for their
advertised functions, and the multi-language serial program can continue.

## 14. Required decisions before normative publication

The design fixes semantics and priority, while these measured implementation
facts are deliberately assigned to their freeze owners:

| Fact to freeze | Owner | Evidence required |
| --- | --- | --- |
| Exact compiler/toolchain and effective-language-version table | T01-W01/W02 | Registered inventory, module/file version and compiler-shape probes |
| v1 bounds and resource budgets | T01-W03/W04 | Representative conditional proofs, both checker receipts and cost measurements |
| v1 wire shapes, hash domains and complete consumer migration | T01-W04/W05 | Closed schemas/vectors, ownership inventory and reviewed migration plan |
| v2 bounds, ownership/effect encoding and budgets | T08-W02/W03 | Map/heap application-shaped lemmas and alias/frame adversarial cases |
| v2 wire shapes and consumer migration | T08-W03 | Closed specification/vector package and predecessor preservation inventory |

Do not fill these entries with guessed digests, speculative success reports,
or an unmeasured “unbounded” model. Their completion is part of implementation
feasibility work, not missing user input or permission to skip a gate.

## 15. Design-only validation record — 2026-10-16

This change adds the design and work plan plus navigation links. No production
code, frozen specification, vector bytes, or registry is edited. Validation
was selected for that documentation-only scope:

- Local Markdown destinations and balanced code fences: passed for the two
  new documents and both edited README files.
- Work-item IDs: 45 unique, sequential W items across T01–T11; passed.
- The embedded Go example: parsed with `gofmt` and built with `go test ./...`
  in a disposable module, with networking/toolchain switching disabled;
  passed under host Go 1.23.2 on darwin/arm64, with no behavioral test files.
- Whitespace/diff review: passed, including checks on the new document files.

The example build establishes syntax and type correctness only. It is not an
MPK proof or a target-Linux release receipt. No implementation T/W was started
or completed, so its full gate is not due. Production tests and
`scripts/check-fast.sh` were not run for this documentation change; they are
assigned to the implementation work items and their stage gates above.
