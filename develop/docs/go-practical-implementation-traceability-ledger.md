# Go Practical Implementation Traceability Ledger

Updated: 2026-10-16. Objective: execute
[the Go design](09_go_practical_subset_design.md) and
[work plan](09_go_practical_subset_design-todo.md), starting at T01-W01.
This ledger owns implementation status. No practical Go profile is active.

## 1. Scheduling authority and preserved scope

The user explicitly authorized Go implementation before the practical C#
release on 2026-10-16. The amended sequence is the current registered
Go/Rust/scalar-C#/Java baseline -> practical Go v1 -> practical Go v2 ->
remaining practical C# -> Dart -> TypeScript -> Python. The change is recorded
in the 06/08/09 design/work-plan documents and the roadmap. It removes only the
C#-release prerequisite. All Go source-semantics, proof, stage and release
gates remain required; the full 11-stage scope remains intact.

Retain the unresolved CSHARP-03 checkpoint. It is not a completed foundation
or a live-process claim. Any common implementation that Go uses must meet
Go's own independent validation and both-checker proof gates. Resume C# only
after reconciling the intervening shared artifacts and active registry.

## 2. Status

| Work item | Status | Evidence / next condition |
| --- | --- | --- |
| GO-PRAC-T01-W01 | Complete | Reviewed implementation commit `bd5ee8047d8925f9c9d0f7502627fca33bb2c836`; completion audit below |
| GO-PRAC-T01-W02 | Complete | Reviewed implementation commit `fed1cf6047f97b1f73c0fc37c5023e19f4f9f035`; 39 cases reproduced twice |
| GO-PRAC-T01-W03 | In progress | Representation/local Nat pilots plus both-checker 64-bit unsigned diagnostic; signed/sum bindings, operation laws and resource closure remain open |
| GO-PRAC-T01-W04 | Blocked by W03 | Exact v1 source/shared/contract/limit freeze |
| GO-PRAC-T01-W05 | Blocked by W04 | Publish inactive package, review and full T01 gate |
| GO-PRAC-T02–T07 | Not started | Sequential v1 implementation and public release |
| GO-PRAC-T08–T11 | Not started | Sequential v2 freeze, maps, pointers and public release |

No T is complete. `scripts/check-fast.sh` is due in T01-W05, not an intermediate W.

## 3. T01-W01 baseline and deliverables

Clean source baseline: `b61e48c65ad1fb007121d8d0a3b01a8d754218cf`, tree
`edee70236ab38763d34afcc4b02bdcba037031d5`, on `codex/go-practical`.
The baseline commits the previously reviewed Go design and navigation links;
its parent is main `1c34ed5701df2e53fd515d49ad4d158cc728cb4b`. A separate
`/usr/bin/git status --porcelain=v1` was empty at baseline capture. Git
operations were run outside the sandbox using `/usr/bin/git`.

The last installed-release evidence is
[the Java T10 native receipt](../migrations/archive/java-03-t10-native-receipt.json),
dated 2026-09-03. Its semantic-registry and bundle-registry raw hashes match
the current checked-in registered bytes. That proves the identity of the
historical four-language release; it does not certify subsequent source
changes, practical C# completion, or a practical Go theorem.

The machine-readable inputs and observations are:

- [Baseline selection](../migrations/go-practical/t01-w01/baseline-selection.json):
  exact commits, target, candidate compiler and proposed language table.
- [Entry audit](../migrations/go-practical/t01-w01/entry-audit.json): exact
  raw-byte hashes, release/profile identity, compiler/specification witnesses,
  77 initial consumer/owner files, 198 regression files, 83 checked indexed
  artifacts, and all 130 fixed-profile vector IDs accounted for.
- [Reproducible audit helper](../probes/go-practical/entry_audit.py) and
  [adversarial checks](../probes/go-practical/test_entry_audit.py).

The compiler candidate is the registered `go1.25.0` Linux-amd64 toolchain,
with executable SHA-256
`b93cdfdbc72f1afc3f21498c80bf3d155a44a9b95e2d690c940511051574bc25`.
The target is `linux/amd64`, pointer/int width 64, GOAMD64 v1. The exact
distribution/content identities and bundled specification/type-version
witnesses are in the audit. This is a metadata pin; W01 did not execute that
Linux compiler on the macOS host.

The proposed effective-language versions are go1.22, go1.23, go1.24 and
go1.25. W02 must measure each version and version-rejection boundary under the
exact Go 1.25.0 analysis release before W04 freezes them. Module patch-level
toolchain requirements must also be satisfied by the pinned compiler; the
minor-version table does not permit automatic compiler downloads.

The consumer inventory is the direct Go-profile reference set plus reviewed
frontend/shared owners. W04 still owns the exhaustive changed-schema migration
inventory. The regression file inventory is a preservation baseline, not an
assertion that every inventoried file is a completed application proof.

## 4. T01-W01 targeted verification

Selected these checks because W01 establishes release/consumer identity and
the old Go behavior. No runtime source or normative specification is changed.
The new audit helper is private and checked against adversarial inputs.

| Check | Observed result | Scope |
| --- | --- | --- |
| `/usr/bin/git status --porcelain=v1` and `rev-parse HEAD HEAD^{tree}` at baseline | Clean; hashes above | Exact source starting point, not a release receipt |
| `python3 develop/probes/go-practical/entry_audit.py --write` then `--check` | Passed | 77 owners, 198 regression files, 83 indexed artifacts; no practical profile admitted |
| `python3 -B -m unittest discover -s develop/probes/go-practical -p 'test_entry_audit.py' -v` | 8 tests passed | Audit tampering, target/version substitution and missing negative coverage |
| Host `go test -mod=vendor -count=1 -run 'Test(GoProfileV0VectorsAreExhaustivelyAccountedFor\|GoProfileIdentityIsClosed\|ActiveGoCorpus)$' ./...` in `go-tools/go2vir` | Passed, 1.207 s package run | Existing fixed-profile vectors/identity and 13 positive / 8 negative source cases; two deterministic generations |
| `cargo test --offline -p mpk-vc --test go_profile_vectors --test go_vir_corpus` | 3 tests passed | Fixed-profile independent import and linked deterministic Go VIR/VC corpus |

The Go command used host Go 1.23.2 darwin/arm64, vendor-only dependencies,
`GOWORK=off`, `GOTOOLCHAIN=local`, `GOPROXY=off`, `CGO_ENABLED=0`, and a
temporary GOCACHE. It is regression evidence, not the Go 1.25.0/version-table
probe or native Linux acceptance. Cargo ran offline.

Adversarial helper tests passed. Final review checked 11 documents and 57
local links, balanced code fences, Python syntax, and `git diff --check`.
There were no changes under `crates`, `go-tools`, `scripts`, `release`,
`fixtures`, `examples`, `proofs`, or `develop/specs` relative to the source
baseline. The implementation and evidence were committed as
`bd5ee8047d8925f9c9d0f7502627fca33bb2c836` before this completion handoff.

Completion audit: the authorized schedule amendment, clean starting point,
historical release witness, ledger, initial consumer inventory, fixed-profile
corpus, candidate compiler/target and proposed language table each have the
evidence described above. No W01 requirement remains pending. W02 is Ready.
Full T01 validation remains explicitly deferred to W05. No native
practical-Go release or new application-proof claim is made.

## 5. T01-W02 original-package and closure measurements

W01 completed before this work began. The measurement uses the exact
Go 1.25.0 analysis release, not host Go 1.23.2. The official
`go1.25.0.darwin-arm64.tar.gz` archive has SHA-256
`544932844156d8172f7a28f77f2ac9c15a23046698b6243f633b0a0b00c0749c` and
57,906,112 bytes. All 14,496 extracted files, digests, membership and
executable modes match the archive. Its specification witness matches the
registered Linux release. The host remains darwin/arm64; the compiler analyzes
linux/amd64 with GOAMD64 v1 and 64-bit target sizes.

Artifacts and implementation owners:

- [Pinned host compiler](../probes/go-practical/host-toolchain.json) and
  [explicit provisioner](../probes/go-practical/provision_host_toolchain.py).
- [Original-package analyzer](../probes/go-practical/module_probe.go),
  [offline two-pass runner](../probes/go-practical/run_module_probes.py), and
  [39 input/expectation cases](../probes/go-practical/module-cases.json).
- [Actual observations](../migrations/go-practical/t01-w02/module-probes.json),
  [readable closure matrix](../migrations/go-practical/t01-w02/module-matrix.md),
  and [candidate snapshot contract](../migrations/go-practical/t01-w02/offline-snapshot-contract.md).
- [Retained-report checker](../probes/go-practical/check_module_report.py),
  [adversarial checks](../probes/go-practical/test_module_report.py), and
  [preliminary findings](../migrations/go-practical/t01-w02/preliminary-findings.json).

The original package is loaded without removing an unused handler, unsupported
function or generic declaration. Captured external helpers can enter the
verification closure; unused module metadata, external initializers and HTTP
imports remain in the type-check/capture context. Reachable mutable globals,
unknown/indirect calls, recursive calls, unsupported maps/generics and assembly
bodies reject under the candidate v1 early exclusions. Original package type
errors and missing offline modules remain source errors. The probe does not
implement complete production profile admission.

Both fresh-source passes give the same normalized observations in all 39
cases: 20 closed early-exclusion closures, 17 explicit rejections, and two
source/module errors. Target/tag-excluded files and auxiliary assembly files
are inventoried. Local/versioned replacements, `require`/`exclude`/`retract`,
nonempty checksums, initializer and generation non-execution, imported domain
types and error-signature dependencies have concrete cases. Source and sealed
dependency-cache bytes remain unchanged.

All four proposed language minors were measured. A file downgraded to go1.21
by its build expression rejects. A module requiring Go 1.25.1 or a newer
toolchain rejects without downloading it. The initial probe incorrectly
compared raw `go1.25.0` file-version data to the `go1.25` language table;
the corrected representation retains both raw version and language minor,
while preserving the full toolchain requirement check. The complete final
evidence uses the corrected source. Preliminary attempts have no retained
source hash and are not promoted to positive proof evidence.

## 6. T01-W02 targeted verification and remaining gate

Selected compiler/source/metadata/replay tests because W02 changes only private
feasibility tools and their candidate snapshot contract. It changes no
production frontend, frozen specification, registry or old Go fixture. W01's
old-Go baseline is rechecked by identity rather than unnecessarily rerunning
unaffected production suites.

| Check | Observed result | Scope |
| --- | --- | --- |
| `python3 -B develop/probes/go-practical/provision_host_toolchain.py` | Passed, 14,496 files match | Actual host archive/tree identity; no Linux-execution claim |
| `python3 -B develop/probes/go-practical/run_module_probes.py --write` | 39 cases x 2 fresh-source passes passed | Original package, version, dependency, closure and unchanged-input measurements |
| `python3 -B develop/probes/go-practical/check_module_report.py` | Passed | Exact source pins, case coverage, observed loaded-source identity and evidence scope |
| `python3 -B -m unittest discover -s develop/probes/go-practical -p 'test_module_report.py' -v` | 7 tests passed | Reject lost cases/bodies, altered inputs, stale source, host/target confusion and invented theorem claims |
| `python3 -B develop/probes/go-practical/entry_audit.py --check` | Passed | Retained W01 source/fixture/compiler baseline identities still match |

The Go transport settings disable proxy/checksum/toolchain/telemetry access
during measurement; synthetic dependency preparation uses only a local file
proxy. The macOS probes do not claim OS-level egress isolation or native
Linux release acceptance. They produce no VIR, VC or accepted application
theorem. Final review passed Python/strict-JSON syntax, exact Go 1.25.0
`gofmt`, five documents and 27 local links, the matrix/report digest and all
39 rows, plus `git diff --check`. Production sources, frozen specifications,
registries, and old Go fixtures remain unchanged from W01. Implementation and
evidence were committed as `fed1cf6047f97b1f73c0fc37c5023e19f4f9f035` before
this completion handoff.

Completion audit: every W02 package/dependency/replacement/build/initialization/
global/version requirement has an observed case, a source identity and an
expected result in the retained matrix. The capture/verification-closure
matrix and exact candidate offline snapshot contract are present, and no W02
requirement remains pending. W03 is Ready and owns the next ordinary-encoding
and both-checker proof feasibility work. The full T01 gate remains explicitly
deferred to W05; T01 is not complete.

## 7. T01-W03 symbolic pilot checkpoint (not completion)

W02 completed before W03 began. The initial pilot checkpoint is committed as
`488ccea4` (`test(go): retain ordinary symbolic proof feasibility pilot`). The
[W03 checkpoint](../migrations/go-practical/t01-w03/README.md) retains the
initial failed candidates, correction and exact final evidence. The
[private Rust producer](../probes/go-practical/certificates/src/main.rs),
[local measurement runner](../probes/go-practical/run_proof_pilot.py),
[retained-report checker](../probes/go-practical/check_proof_report.py) and
[adversarial tests](../probes/go-practical/test_proof_report.py) register no
profile and modify neither checker.

The corrected pilot constructs 27 universal ordinary candidates twice with
identical bytes: 17 accepted zero-axiom lemmas and ten deliberate false claims
rejected at core checking by both Rust and Go. It measures the proposed v1
string and slice capacity points using complete symbolic stores and payloads.
The largest positive certificate has 13,392 terms and 113,603 bytes. Exact
per-case construction/checker times, CPU time and peak resident bytes are in
[the final observation report](../migrations/go-practical/t01-w03/pilot-corrected/observations.json).
Measurements identify Darwin/arm64; unavailable hardware model/RAM remain
unknown. No native Linux release check is claimed.

The initial untrusted normalizer incorrectly assumed nested recursor-major
conversion. Four intended positives rejected in both checkers. The correction
retains the original claims and supplies explicit checked congruence and
transitivity; it does not alter conversion, add an axiom or use unfinished C#
theorems. The archived C# encoder contributes only untrusted candidate terms.

Targeted verification was selected for the changed private producer and
evidence/measurement helpers; production source and old Go fixtures are
unchanged. The complete local `run_proof_pilot.py --write` measurement passed,
including two generations, isolated per-case construction and both independent
verifiers for every candidate. The retained-report check passed and all 13
adversarial tests passed. Private-crate offline locked Clippy passed with
warnings denied, and Rust formatting passed. The W01 identity audit passed;
unaffected production regression suites were not rerun. Documentation links,
balanced fences and Git whitespace checks are part of this checkpoint review.

This is partial feasibility. Complete source/model bindings, descriptor
validity and operation-law closure, the full signed bitvector/exact
mathematical-sum bridge, aggregate interactions and resource boundaries
remain open. The local three-input carry lemma is not that full arithmetic
bridge. W03 remains In progress, W04 has not started, and the full T01 gate
remains deferred to W05. No W03 or T completion is claimed.

## 8. T01-W03 local exact arithmetic ingredient (not completion)

[The next private producer](../probes/go-practical/certificates/src/natural.rs)
adds four candidates with the actual zero-axiom `Std.Nat` constructors:
one-bit full-adder value conservation, the same law for an arbitrary unary
natural-number step, and a dropped-carry mutation for each. The
[arithmetic report](../migrations/go-practical/t01-w03/natural-conservation/observations.json)
records both-checker acceptance for two universal claims and core rejection
for both false variants. The initial natural number is arbitrary. These are
actual equality theorems, with no axioms, imported premises or new checker
rule; they are not the full signed 64-bit/overflow/mathematical-prefix-sum
bridge. Positive certificates have 753/754 terms and 8,024/7,972 bytes.

The same local runner measures both pilots using separate `--natural` mode
and report directory. Both replay generations are byte-identical and match
isolated case construction. The 27 representation certificates remain
byte-identical after the arithmetic addition; their source/binary/resource
observations are refreshed against the new build. Report-integrity checks
cover both modes and preserve the same explicit pending requirements.

The targeted validation covers the new natural-number source/foundation,
changed measurement/report routing and affected representation pilot. Offline
locked private-crate Clippy passed with warnings denied; Rust formatting and
both retained-report checks passed. The local two-generation measurement
passed in both modes: 19 accepted zero-axiom claims and 12 core-rejected false
claims across 31 candidates, each checked by Rust and Go. All 28 report
adversarial tests passed. Exact Git byte comparison confirms that the 27
representation certificate/index files are unchanged from the initial pilot
commit. The natural-mode `--check` exercises measurement replay against the
retained arithmetic bytes as well as the report-integrity branch.

The whole T01 gate remains deferred to W05; W03 remains In progress.

## 9. T01-W03 compositional unsigned-word diagnostic (not completion)

The [private word producer](../probes/go-practical/certificates/src/word.rs)
builds ordinary function-equality lemmas for composition, commutation and
width induction. Its full-adder proof splits three local flags; the complete
operand space is not enumerated. The
[model](../migrations/go-practical/t01-w03/word-conservation-model.md) explains
the exact natural denotation and outstanding signed/sum obligations.

The [retained diagnostic](../migrations/go-practical/t01-w03/word-initial/README.md)
and [raw observations](../migrations/go-practical/t01-w03/word-initial/observations.json)
prove actual both-checker acceptance of the unsigned 64-bit/final-carry
theorem with zero axioms and matching hashes. It quantifies all 128 operand
bits, the incoming carry and the initial natural. Its self-contained
certificate contains 423 declarations, 81,514 terms and 1,043,626 bytes.
Construction takes 69.432 s / 688,193,536 peak resident bytes; optimized Rust
checking takes 198.099 s / 431,833,088 bytes and Go takes 476.319 s /
1,018,724,352 bytes. This is one measured diagnostic, not a complete two-pass
matrix, a signed bridge, a Go theorem or a frozen resource budget.

The original matrix stopped at a 32-bit dropped-carry candidate. Rust reports
`CORE_FUEL_EXHAUSTED`; Go reports `core reduction fuel exhausted`. Both raw
results, original candidate bytes and exact private source snapshots are
retained. They count as candidate resource failures, never core type-mismatch
negatives or certified counterexamples. Batch output lacking retained raw
checker results is explicitly only an artifact. The current producer's
additional low-bit mutation has a both-checker type-mismatch debug result at
one bit; its larger matrix remains unmeasured.

Targeted validation covers the new word producer, unchanged earlier pilot
outputs, and diagnostic evidence classification. Offline locked Clippy passes
with warnings denied; Rust formatting and Python syntax pass. Both earlier
pilot modes were remeasured against the current producer: 27 representation
and four local-Nat cases, with all 28 existing evidence tests passing. The
new diagnostic-integrity check and nine adversarial tests pass. Kernel,
reference checker, normative specification and registry sources are unchanged.
The full T01 gate remains deferred to W05.

Before resource freeze, the next private candidate replaces the long operand
bit telescope with the existing Bool-addressed cube and balanced half-word
composition. It preserves the full 64-bit domain and exact final-carry
statement. No performance or proof success is assumed for that redesign;
both-checker validation and full workload/boundary measurement remain due.
Signed interpretation, no-overflow/order, mathematical prefix sums, complete
source/operation bindings and aggregate resource closure remain open. W03 is
In progress and W04 has not started.
