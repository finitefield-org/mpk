# Go Practical Implementation Traceability Ledger

Updated: 2026-10-18. Objective: execute
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
| GO-PRAC-T01-W03 | Complete | Reviewed implementation `9d4e6fb7855a66d9fff309e6dbd9143652ee6e21`; requirement-by-requirement completion audit in section28 |
| GO-PRAC-T01-W04 | Complete | Commit `68a44416e1b1a6dc3a84587cca784fc4b3a2b862`; exact interfaces, resource bounds, consumer review and historical replay; section58 |
| GO-PRAC-T01-W05 | Complete under user C# test deferral | Reviewed implementation `4b4c3a01ddfa2377d7ccc0116f128649ff3ec3b1`; amended cumulative gate passes594 tests and all other checks. Completion audit and section78. |
| GO-PRAC-T02-W01 | In progress | Source-object codec and rooted multi-capability capture pass20 root tests/43 subtests; the native-asset review follow-up passes all8 affected rooted tests/17 subtests, vet/format and the legacy build. Original module/checksum/replacement checks also pass12 roots/52 subtests. Complete private context/versioned graph/loader/export capture remains due; sections79–81. |
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

## 10. T01-W03 balanced unsigned and conditional signed addition

The [balanced producer](../probes/go-practical/certificates/src/word/balanced.rs)
and [signed layer](../probes/go-practical/certificates/src/word/balanced/signed.rs)
use complete Bool-addressed operands and checked half-word composition.
Neither bit-pattern enumeration nor a reduced source integer width is used.
[The model and measured evidence](../migrations/go-practical/t01-w03/balanced-words/README.md)
cover 48 cases at widths 1/2/4/8/16/32/64/128. Every group has two byte-identical
fresh generations. All 16 positives pass Rust and Go with zero axioms and
matching logical hashes; 25 incorrect candidates fail at core type checking.
Seven offset-removal candidates exhaust conversion fuel and are explicitly
not counted as semantic negatives or certified counterexamples.

Unsigned64 has 5,634 terms / 65,068 bytes and checks in 0.126/0.266 s, using
11,452,416 / 24,969,216 peak resident bytes. Its three-candidate batch takes
0.098 s / 8,142,848 bytes to construct. Signed64 has 7,741 terms / 89,106 bytes
and checks in 0.186/0.353 s. These local observations are not a frozen
application budget. Width128 is an internal widened mathematical-domain probe,
not Go source type admission.

The signed claim proves equality of natural cross sums for two's-complement
balanced differences under the actual sign-stage carry-agreement guard.
Checked sum-sign and carry-binding lemmas connect that predicate to the same
word adder. Replacing it by an unrelated true premise rejects at core checking
at every width. Establishing the guard from a Go source path remains an
application obligation. The positive 16/32-bit fuel findings and original
sources are retained in
[tree-initial](../migrations/go-practical/t01-w03/tree-initial/observations.json)
and its [projection attempt](../migrations/go-practical/t01-w03/tree-initial/projection-attempt/observations.json).
Explicit checked projection/carry equalities resolve those findings without
changing either checker or its conversion budget.

Targeted validation covers the new private representation, arithmetic/sign
bindings, guard substitution, deterministic replay, checker routes and
evidence helpers. The 48-case local measurement, retained-report check and
all 15 new adversarial tests pass. Offline locked release Clippy passes with
warnings denied; Rust formatting and Python syntax pass. The affected older
pilots were remeasured: 27 representation and four local-Nat cases pass, with
all 28 existing report tests passing. The historical word checkpoint and its
nine adversarial tests also pass. Existing 31 certificate/index bytes remain
unchanged. Production frontend/checker, specification and registry source
identities remain at W01, so unrelated production suites are not rerun.

This closes a conditional addition ingredient, not the full W03 scope. Exact
prefix-sum composition, source/operation bindings and guard discharge,
string/slice/descriptor/control-law closure and aggregate/boundary/resource
freeze remain open. W03 stays In progress, W04 has not started, and the whole
T01 gate remains due in W05.

## 11. T01-W03 exact unsigned prefix-sum ingredients

The [prefix producer](../probes/go-practical/certificates/src/word/balanced/signed/fold.rs)
threads actual left-to-right modular64 accumulator states, independently
defines the exact `Std.Nat` mathematical sum, and proves two relations:
exact equality under each actual intermediate zero-carry premise, and an
unconditional equality retaining `2^64` for each actual carry. Checked
definition/projection equalities and ordinary composition/commutation proofs
connect the same full-word adder through partial and complete prefix trees.
No input payload is enumerated and no checker rule, fuel budget, axiom or
unfinished C# theorem is introduced.

[The model and measured evidence](../migrations/go-practical/t01-w03/prefix-sums/README.md)
cover counts 0/1/3/8/32/127/128/129 and 55 candidates. Both fresh generations
are byte-identical. Rust and Go accept 18 zero-axiom claims with matching
logical hashes, reject 23 altered claims at core type checking, and report
14 fuel outcomes for last-summand/correction removal. Fuel outcomes remain
resource findings, never semantic negatives or certified counterexamples.
The two extra empty-prefix claims are valid controls: its safety condition
is vacuous and its overflow correction is the identity.

The 128-element claims each have 355 declarations and 11,549 terms in
138,756/138,764 bytes. In the retained run, conditional checking costs
0.212/0.486 s and 19,628,032/39,731,200 peak resident bytes for Rust/Go;
unconditional checking costs 0.231/0.515 s and 19,595,264/37,076,992 bytes.
All 8,192 storage bits, 64 accumulator bits and the initial natural are
universally quantified. The count129 case retains all 16,384 bits in its
256-cell padded store; this is an internal boundary probe, not Go admission
or a frozen resource budget. Host observations remain Darwin/arm64.

The [original empty-prefix positive fuel failure](../migrations/go-practical/t01-w03/prefix-initial/observations.json)
retains its canonical bytes, raw results and exact source snapshots. The
correction proves right identity for the word's known composite function
and lifts it with checked congruence/transitivity. It does not assume an eta
law for arbitrary functions or silently replace the intended sum claim.

Targeted validation covers actual accumulator/carry bindings, complete
payload/padding quantification, empty and boundary cases, altered claims,
deterministic construction and evidence integrity. The full 55-case local
measurement and separate `--check` proof replay pass; all 19 new adversarial
tests pass, including historical snapshot validation. The changed private
producer also remeasures the 27
representation, four local-Nat and 48 balanced word candidates. Their
earlier certificate/index bytes stay unchanged. All 52 existing evidence
tests pass, for 71 targeted tests total. Offline locked release Clippy passes
with warnings denied; Rust formatting, Python syntax, retained-report checks,
the W01 identity audit, document links and Git whitespace checks pass.
Production frontend/checker, specification and registry identities remain
at W01, so unrelated production suites are not rerun.

Unsigned interpretation does not establish the signed/nonnegative `Quote`
prefix invariant. Signed prefix/order/subtraction and source guard discharge,
complete source/operation bindings, descriptor/collection/control laws and
aggregate boundary/resource closure remain due. W03 remains In progress;
W04 has not started. The full T01 gate stays deferred to W05.

## 12. T01-W03 conditional signed prefix composition

Predecessor checkpoint: `ea627414` retains the unsigned prefix ingredients
and their original feasibility finding. The
[signed prefix layer](../probes/go-practical/certificates/src/word/balanced/signed/fold/signed_prefix.rs)
adds actual sign-stage guard cubes, checked child projections, a carry/sign
balance across all intermediate states, and exact two's-complement prefix
cross sums. It composes the existing unconditional unsigned theorem with
the balance law. All 64-bit values, including negative patterns, remain
quantified; there is no implicit nonnegative source precondition. Its
arithmetic premise is every actual update's sign-stage carry agreement.

[The model and actual 32-case evidence](../migrations/go-practical/t01-w03/signed-prefix-sums/README.md)
cover counts 0/1/3/8/32/127/128/129, with two identical fresh generations.
Both checkers accept nine zero-axiom claims with matching hashes, including
the valid vacuous empty-prefix guard control. Fifteen altered claims fail
at core type checking. Eight dropped-result-sign candidates exhaust
conversion fuel and remain resource findings, not semantic negatives.
No checker rule, conversion budget, source width or public profile changes.

The count128 positive has 537 declarations, 17,736 terms and 214,747 bytes.
It quantifies all 8,192 storage bits, the 64-bit accumulator and initial
natural. Rust/Go checking takes 0.322/0.833 s with 35,684,352/66,895,872 peak
resident bytes in the retained Darwin/arm64 run. Count129 retains the full
256-cell/16,384-bit padded store and is an internal boundary probe. These
observations do not freeze a Go capacity or an application resource budget.

Targeted verification covers signed denotation, actual guard/state linkage,
partial/empty/boundary trees, complete payloads, altered claims and evidence
integrity. The 32-case local measurement and separate signed and unsigned
`--check` proof replays pass; all 17 new adversarial tests pass. The signed
and unsigned runners share the measurement implementation
but retain separate command/schema/theorem validators. Because the producer
and shared runner changed, the prior 55 unsigned-prefix, 48 balanced-word,
27 representation and four local-Nat candidates are remeasured. All earlier
certificate/index bytes remain unchanged. The complete targeted evidence
suite has 88 passing tests. Offline locked release Clippy passes with
warnings denied; Rust formatting, Python syntax, retained-report checks,
W01 identity audit, document links and Git whitespace checks pass.
Unchanged production suites are not rerun. The whole T01 gate remains
deferred to W05.

This closes conditional signed prefix composition, not the source-bound
`Quote` invariant. Actual Go guard/order/subtraction discharge, the
nonnegative invariant, complete operation/source bindings,
descriptor/collection/control-law closure and aggregate boundary/resource
freeze remain due. W03 stays In progress; W04 has not started. The full
11-stage implementation objective remains active.

## 13. T01-W03 comparison/subtraction-to-addition guard discharge

Predecessor checkpoint: `116e962a394ddc08a52483b97d6852732ea27fbf` retains
the conditional signed prefix composition. The
[order producer](../probes/go-practical/certificates/src/word/balanced/signed/order.rs)
and [path layer](../probes/go-practical/certificates/src/word/balanced/signed/order/path.rs)
now define complete word complementation, modular subtraction and signed
comparison, compose a six-state invariant through four actual carry chains,
and derive the addition guard from source-shaped continuing-path conditions.
The conditions are nonnegative limit/accumulator/item, accumulator within
limit, and item within the wrapping remaining amount. The proof derives
nonnegative remaining amount, no unsigned wrap, a nonnegative bounded result,
and actual sign-stage carry agreement. A genuine `Std.Nat` theorem gives
exact addition for every initial natural under those same conditions.
It does not assume the final addition guard, enumerate full operand values,
add an axiom, change conversion fuel or rely on unfinished C# theorems.

[The model and actual 40-case measurements](../migrations/go-practical/t01-w03/order-guards/README.md)
cover carry composition at widths 1/2/4/8/16/32/64/128 and continuing-path /
natural-addition claims at actual signed Go widths 8/16/32/64. Both fresh
generations are identical. Rust and Go accept 16 zero-axiom claims with
matching hashes and reject 24 deliberate substitutions at core type checking.
No fuel failure is counted as a semantic negative. Width128 is an internal
arithmetic probe, not a new Go source type. The 64-bit step claim has 300
declarations, 20,459 terms and 233,086 bytes; the exact-natural claim has
20,387 terms and 232,314 bytes. All 192 operand bits remain quantified;
the latter also quantifies its initial natural. Per-case resources and
exact source/checker identities remain in the report, not a frozen budget.

The [pinned Go observer](../probes/go-practical/order_reference.go) executes
3,428 typed boundary/sign/full-range cases, including wrapping and
noncontinuing inputs. All observations agree with an independent bit-loop
model; the 152 continuing cases also exhibit exact addition and the bounded
nonnegative result. Exact input/output bytes and observer resources/binary
identity are retained separately. These are empirical observations, not a
universal application proof or a production frontend/operator binding.

Two construction findings are recorded with their corrections: rewrite a
dependent guard before its carry subexpression, and include the family in
each candidate ID to prevent file shadowing. The runner rejects duplicate
IDs and any file/index coverage mismatch. The observation validator also
requires actual JSON booleans; numeric zero/one cannot impersonate them.
Neither correction changes the intended theorem or strengthens its premises.

Targeted verification covers actual carry/state binding, modeled signed
comparison and subtraction, continuing-path arithmetic, exact natural
observation, altered claims, typed Go observations and evidence integrity.
The complete 40-case measurement and all 22 new adversarial tests pass.
Because the private producer changes, the prior 27 representation, four
local-Nat, 48 balanced-word, 55 unsigned-prefix and 32 signed-prefix
candidates are remeasured. Certificate bytes for all 166 candidates and their
retained indexes remain unchanged; all 88 earlier evidence tests pass, for
110 targeted tests total.
Offline locked release Clippy passes with warnings denied; Rust formatting,
pinned Go formatting, Python syntax, retained-report checks, W01 identity
audit, local document links and Git whitespace checks pass.
The W02 retained closure report also passes; its analyzer explicitly copies
only `module_probe.go` into an isolated build, so the observer cannot join
that package. Production
frontend/checker, normative specification and registry identities remain
at W01, so unrelated production suites are not rerun. The complete T01 gate
remains deferred to W05. The separate final `--check` replay also passes
all 40 proof candidates and reproduces the exact Go input/output bytes.

Complete production Go operator/branch/source binding, actual
`Quote` loop/prefix/return composition, collection/descriptor/control and
byte-operation law closure and aggregate boundary/resource freeze remain
open. W03 stays In progress; W04 has not started. The entire authorized
11-stage objective remains active.

## 14. T01-W03 continuing-prefix bounds and literal-zero entry

Predecessor checkpoint: `ea164f5599fa7835839ecb0bf5ec3a5be633b136` retains
the step arithmetic. The
[continuation layer](../probes/go-practical/certificates/src/word/balanced/signed/fold/continuation.rs)
now composes that law across the actual left-to-right prefix. Its check cube
contains each actual item's sign/remaining-bound tests at the actual
intermediate accumulator. The low-prefix result proves the high prefix's
initial bounds. No per-step invariant or carry truth is supplied as an
oracle. It derives a nonnegative bounded final accumulator, all actual
zero-carry equalities, and exact processed-prefix natural sums.

Checked zero projection/sign/complement-carry/comparison/value lemmas also
discharge the actual all-zero initializer. Zero-entry state and exact-sum
claims require only the nonnegative limit and actual item checks. The
initial zero-complement proof failed in both checkers because its producer
assumed evaluation of a named recursor major. The
[original failure and source snapshots](../migrations/go-practical/t01-w03/continuation-initial/README.md)
are preserved. Explicit checked congruence transports the zero equality;
the correction adds no premise, axiom, eta/extensionality rule or checker
change. This is a corrected positive proof, not semantic-negative evidence.

[The measured 72-case matrix](../migrations/go-practical/t01-w03/continuing-prefixes/README.md)
covers bounded and zero entry, state and exact-natural conclusions, at
counts 0/1/3/8/32/127/128/129. Every group has two identical fresh generations.
Both checkers accept 36 zero-axiom claims with matching hashes and reject
36 deliberate substitutions at core type checking. Empty-prefix check
replacements remain valid accepted controls. Count128 preserves all 8,192
backing bits and 64 limit bits; bounded-entry claims also quantify 64 initial
accumulator bits. Its zero-entry natural claim has 685 declarations,
30,399 terms and 374,906 bytes, checking in 1.262/2.840 s with
40,878,080/102,449,152 peak resident bytes for Rust/Go in the retained run.
Count129 retains all 16,384 padded backing bits and is an internal boundary
probe, not Go capacity admission or a frozen budget. The host is Darwin/arm64.

The [ordinary-Go Quote fixture](../probes/go-practical/quote_reference.go)
and separate observer execute 769 deterministic cases, including nil and
non-nil empties, entry failures, negative/extreme amounts and limit failures.
The independent bit-loop model matches 169 successes, 456 bad-input and
144 limit-exceeded results, zero decisions on failure and input/nil
preservation. Exact input/output bytes and binary/resources are retained.
These empirical observations do not claim a formal source/return/error/
frame proof. The source-shaped arithmetic prefix remains the certified scope.

Go feasibility builds now explicitly pass `-buildvcs=false`; their automatic
Git metadata/status lookup was still enabled and has been removed. A newly
built reference binary has no VCS build-info fields and SHA-256
`bc21d279807f5994a16287a532db65cac3bbb170bfa4d0ca5c6729825e24c62c`,
matching the measured checker. Explicit Git commands continue to use
`/usr/bin/git` outside the sandbox. Historical helper bytes are copied only
when they match each original report's exact source hash, keeping old
findings independently readable after this build-setting correction.

Targeted verification covers derived intermediate bounds, actual check/
carry projections, zero entry, exact natural observation, complete payload/
padding domains, altered claims, original-Go observations and evidence
integrity. The complete 72-case measurement and all 25 new adversarial tests
pass. The changed producer/measurement helpers also remeasure the earlier
206 candidates, with every canonical certificate/index unchanged; all
110 earlier evidence tests pass, for 135 targeted tests total. Offline locked
release Clippy passes with warnings denied; Rust and pinned Go formatting,
Python syntax, retained-report/historical-snapshot checks, W01 identity
audit and the W02 retained closure report pass. Document links and Git
whitespace checks pass. Unchanged production suites are not rerun. The
separate final `--check` prefix/Go replay also passes all 72 candidates and
reproduces the exact Go input/output bytes. The full T01 gate remains
deferred to W05.

Complete production source/operator binding, actual loop/return/closed-error
and descriptor composition, remaining byte/collection/control laws and
aggregate boundary/resource closure remain open. W03 stays In progress;
W04 has not started. The full 11-stage objective remains active.

## 15. T01-W03 halting returns and success-derived exact sums

Predecessor checkpoint: `157abf5f64804fd293698f60a5108edad51090d2` retains
the continuing-prefix/zero-entry evidence. The
[return-frame producer](../probes/go-practical/certificates/src/word/balanced/signed/fold/continuation/returned.rs),
[halting fold](../probes/go-practical/certificates/src/word/balanced/signed/fold/continuation/returned/run.rs)
and [root composition](../probes/go-practical/certificates/src/word/balanced/signed/fold/continuation/returned/quote.rs)
add ordinary checked control and payload equalities. A frame contains a
complete 64-bit accumulator plus 64 metadata bits. Arbitrary incoming
frames quantify all 128 bits; stopped absorption preserves the original
whole frame, without record eta or function extensionality. Actual negative
items and remaining-bound failures return distinct closed model errors.

The actual low-prefix result feeds the high prefix. A stopped low result
equals the whole enclosing result, retaining its payload and error reason.
An actually running output proves that both actual children were running,
derives each actual item check at the actual accumulator, and yields the
ordinary modular-fold payload. The root's abstract empty-name/negative-limit
guard precedes literal-zero initialization and the halting fold. Actual root
success alone derives its nonnegative limit, all item checks, nonnegative
bounded output and exact genuine-natural sum. Caller-supplied check/carry or
intermediate-invariant oracles are absent. Actual root failure proves zero
payload; this is the Quote policy, not a general restriction on Go error
results. Closed outcome exhaustiveness/disjointness is a representation law.
The empty flag, fixed-count store and closed return carrier still need
production Go string/slice/source/interface bindings.

[The complete 108-case measurement](../migrations/go-practical/t01-w03/halting-returns/README.md)
covers counts 0/1/3/8/32/127/128/129 and seven claim families. First-failure
composition applies to the six non-leaf counts; the other six families span
all eight counts. Every group has two identical fresh generations. Both
checkers accept 54 zero-axiom claims with matching hashes and reject 54
altered stop/payload/success-premise/error/partition/first-failure claims at
core type checking. No new fuel failure is counted or hidden. At count128,
the rooted exact-sum candidate has 1,008 declarations, 38,354 terms and
488,435 bytes. Its observed Rust/Go resources are 1.424/3.148 seconds and
65,372,160/136,167,424 resident bytes. All 8,192 backing bits, 64 limit bits
and the abstract empty flag remain quantified; count129 retains all 16,384
padded backing bits and remains an internal boundary probe. Resources are
actual Darwin/arm64 child measurements, not a frozen profile budget.

The pinned original-Go fixture again matches all 769 independent bit-loop
observations: 169 successes, 456 bad-input and 144 limit-exceeded returns,
with zero failure decisions and preserved input/nil observations. Exact
input/output bytes, binary identity and resources are retained separately.
They do not establish a source-bound Go application or frame theorem. The
model uses no added axiom, checker rule, conversion fuel, unfinished C#
theorem or stronger Go entry precondition. Production frontend/checker,
normative specification and public registry remain at the W01 baseline.

Targeted validation was selected for the changed private producer, actual
halting/status-to-check composition, full frame/padding domains, zero-entry
natural observation and retained evidence readers. All 108 new candidates
were measured; 25 new adversarial evidence tests pass. Because the shared
producer changes, the earlier 27 representation, four local-Nat, 48 balanced,
55 unsigned-prefix, 32 signed-prefix, 40 guard and 72 continuing-prefix
candidates were remeasured. All 278 canonical certificate bytes and complete
candidate indexes remain unchanged against the predecessor commit. Their
135 affected evidence tests pass, for 160 targeted tests in this checkpoint.
The separate final `run_returned_probes.py --check` replay passes all 108
candidates through both checkers and reproduces the exact original-Go
input/output bytes.
Historical diagnostic source/certificate checks also pass; the expensive
unchanged linear-word proof is not unnecessarily replayed.

Offline locked release Clippy passes with warnings denied. Explicit private
Rust formatting, Python syntax, five changed documents/88 local links and
Git whitespace checks pass. W01's 77-owner/198-regression-file/83-artifact
identity audit and W02's 39-case retained closure report pass; unchanged
production suites and unchanged W01/W02 adversarial helpers are not rerun.
The complete T01 gate remains deferred to W05. Complete production source/
operator/string/slice/loop/return/error/descriptor bindings, remaining byte/
collection/control laws and aggregate resource closure remain open. W03
stays In progress and W04 has not started. The full authorized 11-stage
objective remains active.

## 16. T01-W03 canonical index values and finite string tails

Predecessor checkpoint: `53d274dc827e2df7f782e601f92a7aa79886176e` retains
the halting Quote model. The new
[byte producer](../probes/go-practical/certificates/src/word/balanced/signed/bytes.rs)
proves canonical word projections, full bit preservation, idempotence and
congruence using ordinary checked equalities. A total byte read canonicalizes
its full 64-bit index. Pointwise-equal index values therefore read the same
complete byte without adding general function extensionality or record eta.
A checked finite-store lift instantiates this interface with every backing
byte/padding bit; it is not an out-of-range Go read rule. Defined indexing
separately requires the actual nonnegative/less-than-length guard.

Checked typed conditional and canonical-index laws prove mask/read
idempotence and preservation of defined full-byte reads. Balanced Boolean
and continuation folds consume pointwise checked equalities, proving that
string equality and bytewise lexicographic order ignore normalized tails.
Lengths/indices remain full source-width words, both comparison backings
remain quantified, and no complete operand or collection contents are
enumerated. This does not discard slice tail data, which can be observable
after re-slicing. Complete Go admission, source/operator binding and remaining
byte operations remain open.

[The complete 46-case matrix](../migrations/go-practical/t01-w03/byte-tails/README.md)
uses capacities 0/1/32/256 and six families. Empty defined-index cases are
excluded to avoid a vacuous positive. Every group has two identical fresh
generations; both checkers accept 25 zero-axiom claims with identical hashes
and reject 21 altered payload/guard/index-relation/comparison claims at core
type checking. No new fuel failure is hidden or counted as false-claim
evidence. Capacity256 retains 2,048 bits per backing, 64-bit lengths and
indices. Its ordering claim has 150 declarations, 2,811 terms and 31,722
bytes, checking at observed Rust/Go 0.357/0.857 seconds and
5,439,488/14,057,472 resident bytes on Darwin/arm64. These remain private
ingredient measurements, not frozen profile/application budgets.

The [pinned original-Go observer](../probes/go-practical/byte_reference.go)
matches all 2,791 independent bit-loop observations, including NUL, invalid
UTF-8, multibyte text, full/empty/equal-prefix strings and negative/extreme/
end-point 64-bit indices. It observes 646 defined/2,145 undefined indices,
224 equal and 1,249 less-than results, 138 equal-string/different-padding
cases and 337 cases where padding would change the comparison order.
Exact input/output bytes, executable identity and child resources remain
empirical evidence, separate from the ordinary model certificates.
The separate final byte `--check` replay passes all 46 candidates in both
checkers and reproduces the exact original-Go input/output bytes. All 29
new adversarial evidence tests pass. No new axiom, checker rule, conversion
fuel or unfinished C# theorem is used; no Go source entry bound is silently
strengthened.

Review also binds accepted results to their actual retained candidate bytes
using the existing `MPK-MODULE-CERT-0.1` hash domain. An adversarial test
rejects replacing both accepted checker observations with those of another
same-module/same-declaration-count candidate. This integrity check complements
the actual independent proof replay; it does not replace proof execution.

The Go observer actually attempts invalid indices and recovers their bounds
panics outside the proof closure. Its 2,145 undefined cases therefore observe
runtime failures rather than only repeating a bounds guard. This does not
admit recovery or defer in v1. The complete byte measurement/replay and its
29 affected tests are rerun after that observation correction; exact output
bytes remain unchanged.

Selected validation covers the changed private producer and measurement
helper, finite lifts/canonical constructors, actual defined-index guards,
tail observations, complete payloads and evidence integrity. Because earlier
families share the producer/helper, all 386 previous candidates are
remeasured: 27 representation, four local-Nat, 48 balanced, 55 unsigned-
prefix, 32 signed-prefix, 40 guard, 72 continuing-prefix and 108 halting-
return cases. Every canonical certificate and complete candidate index
remains unchanged against the predecessor commit. Their 160 affected
evidence tests pass, for 189 targeted tests including the 29 new tests.
Historical diagnostic source/certificate checks also pass; unchanged
expensive linear-word proofs are not needlessly rerun.

Offline locked release Clippy passes with warnings denied. Explicit private
Rust formatting, pinned Go formatting, Python syntax, six changed documents/
95 local links and Git whitespace checks pass. W01's unchanged production/
regression identity audit and W02's retained 39-case closure report pass;
unaffected production suites and W01/W02 adversarial helpers are not rerun.
Production frontend/checker, normative specification and public registry
remain at W01. The complete T01 gate stays deferred to W05. Complete Go
source/carrier/operator bindings (including bounded-index coverage and guard
equivalence), substring/concatenation/prefix/suffix and remaining byte laws,
slice/descriptor/control composition and aggregate resource closure remain
open. W03 stays In progress, W04 has not started, and the full authorized
11-stage implementation objective remains active.

Review identified two true one-cell index-coherence claims whose proposed
witness supplied an index self-equality to the general theorem. Their type
failures are not false-claim evidence. The [original bytes, both results and
private sources](../migrations/go-practical/t01-w03/byte-initial/README.md)
remain preserved. Reflexivity of the actual single backing byte now proves
both accepted controls, without an axiom or stronger entry predicate. The
other 44 candidate bytes and complete case rows are unchanged. Earlier
families are remeasured with the final producer before this checkpoint.

## 17. T01-W03 full-width bounded-index coverage

Predecessor checkpoint: `1640f295e1c1afa433e47e5d81d8de924160e4e8` retains
the finite byte/tail ingredient and its true one-cell controls. The new
[order library](../probes/go-practical/certificates/src/word/balanced/signed/bytes/bounds.rs)
proves comparison trichotomy/transitivity, equality substitution, sign/order
compatibility and zero/one bounds. Checked Boolean cases and congruence
compose these laws over every source bit; operand values are never enumerated.

The [coverage composition](../probes/go-practical/certificates/src/word/balanced/signed/bytes/bounds/coverage.rs)
derives length nonnegativity from the actual full-width length bound and
proves agreement between the source-shaped signed index condition and the
existing byte-mask condition. An actually valid index then equals its
zero-extended finite embedding as a complete canonical 64-bit value. The
finite-fit predicate is derived internally, with no caller fit oracle.
Capacity1 proves that a valid index is zero; capacity0 proves the absence of
any valid index. This closes an ordinary prerequisite for the finite byte
store lift without replacing full-value equality with modulo equality.
Complete production Go source/operator/carrier bindings remain due.

[The 27-case evidence](../migrations/go-practical/t01-w03/index-bounds/README.md)
covers capacities 0/1/32/256 with two identical fresh generations per group.
Rust and Go accept 12 zero-axiom claims with matching hashes and reject 15
altered-result/erased-bound/erased-guard candidates at core type checking.
No new fuel failure is hidden or counted. Accepted results bind to the exact
retained certificate bytes through the existing module-certificate hash
domain. The capacity256 coverage claim has 243 declarations, 26,360 terms
and 316,659 bytes. Its observed Rust/Go resources are 5.676/13.218 seconds
and 37,126,144/92,520,448 resident bytes. The host reports Darwin27/arm64
and 10 CPUs; model/RAM queries are unavailable. Independent private replays
may overlap; these are child-resource observations, not an isolated-machine
guarantee or a frozen application budget.

The pinned [original-Go observer](../probes/go-practical/index_reference.go)
matches all 116 independent bit-loop observations. These include 15 bounded
safe and 70 bounded unsafe cases, four valid reads beyond private coverage,
30 negative-index panics and 15 maximum-positive-index panics. Every one of
the 15 false mutations has an explicit input/output witness, separate from
its type failure. For example, length2/index1 is a valid Go read outside
capacity1 and differs from the low-bit zero embedding; erasing the source
guard admits index-1 at length0; erasing the zero-capacity bound admits
length1/index0. Exact input/output bytes, binary identity and resources are
retained. These observations are empirical; they establish no captured Go
function theorem or support for observer-only recovery in the proof closure.

Selected validation covers the changed private producer, full-source
comparison/guard/bound composition, finite embedding, empty/one boundaries
and evidence integrity. Because the shared producer changes, all 432 prior
candidates are remeasured: 27 representation, four local-Nat, 48 balanced,
55 unsigned-prefix, 32 signed-prefix, 40 guard, 72 continuing-prefix,
108 halting-return and 46 byte-tail cases. Every canonical certificate,
both standalone indexes and all nine complete case indexes/outcomes remain
unchanged against the predecessor commit. Earlier prose resource examples
describe their respective checkpoints; refreshed JSON reports hold the
current source/binary identities and observations. Unchanged expensive
linear-word proofs are not needlessly replayed.

All 32 new adversarial evidence tests pass. The 189 affected earlier
evidence tests also pass, for 221 selected tests. Offline locked release
Clippy passes with warnings denied; explicit private Rust formatting,
pinned Go formatting and changed Python syntax checks pass. W01's
77-owner/198-regression-file/83-artifact identity audit and W02's retained
39-case closure report pass. Unaffected production suites and unchanged
W01/W02 adversarial helpers are not rerun.

The separate final index `--check` replay passes all 27 candidates through
both checkers and reproduces the exact original-Go input/output bytes.
Final review checks six changed documents/108 local links, balanced code
fences, source/diff scope and Git whitespace. No incomplete measurement
attempt remains.

No new axiom, checker rule, conversion fuel, record eta, function
extensionality, unfinished C# theorem or stronger Go entry condition is
introduced. Production frontend/checker, normative specification and public
registry remain at W01. Remaining string operations, slice/descriptor/control
composition, complete Go source/operator/carrier bindings and aggregate
resource closure stay open. W03 remains In progress, W04 has not started,
and the complete T01 gate remains deferred to W05. The full authorized
11-stage implementation objective remains active.

## 18. T01-W03 descriptor stores, array copies and observable tails

Predecessor checkpoint: `7b0d2e80b3403c863f31cc6795e7f970668e391c` retains
the full-width bounded-index ingredient. The new
[descriptor/store producer](../probes/go-practical/certificates/src/word/balanced/signed/bytes/bounds/slices.rs)
preserves nil/region/offset/length/capacity observations, complete 128-bit
cells and both finite backings/padding. Source numeric fields remain full
64-bit words. The descriptor carrier has 512 quantified packing bits;
region names are two logical identities and do not admit source pointer
operations. Physical addresses use the actual balanced modular64 adder
over canonical source words, rather than an uninterpreted address oracle.

Checked constructor/read/write/key/view lemmas establish that a write through
one view is visible through an alias with the same region/offset, even when
length/capacity metadata differ. Distinct regions preserve complete cell
values. Array copying snapshots the original source cells in the other
region, and a later source update preserves that snapshot. Full re-slicing
retains its region/offset/capacity and exposes existing nonzero tail data;
string tail normalization cannot discard those observable slice cells.
Nil and non-nil empty descriptors remain distinct.

[The 90-case matrix](../migrations/go-practical/t01-w03/slice-stores/README.md)
covers 0/1/8/32/127/128/129 cells and seven claim families. Empty storage has
only the three metadata families. Each group has two identical fresh
generations. Both checkers accept 45 zero-axiom candidates with matching
hashes and reject 45 altered-result/region/array/tail candidates at core
type checking. No current fuel result is counted as a semantic negative.
The capacity128 array-copy candidate has 273 declarations, 8,972 terms and
99,641 bytes. Its observed Rust/Go resources are 0.219/0.509 seconds and
10,829,824/31,686,656 resident bytes. Both 128-cell stores retain 32,768
bits; the internal 129-cell boundary retains 65,536 including padding.
Resources are per-child Darwin27/arm64/10-CPU observations, possibly
overlapping independent private replays, with original model/RAM nulls.
They freeze neither a source/profile limit nor an application budget.

[The pinned original-Go observer](../probes/go-practical/slice_reference.go)
matches all 1,596 independent cell/bit-loop observations: 28 nil and 600
non-nil empty rows, 240 defined shared writes/array snapshots, 300 nonzero
tail observations, 514 restricted-capacity views and 1,356 rows with
recovered original-view bounds failures. A 129-cell source view accepts
index128; the private boundary is not mislabeled as a Go runtime panic.
Every false mutation has a concrete empirical witness, separate from its
checker type failure. Native exact bytes, executable identity and child
resources remain empirical, rather than a captured-source function theorem.
The observer's three arrays and observer-only I/O/recovery do not establish
a shared source binding for the two-region ordinary laws.

[The retained positive diagnostics](../migrations/go-practical/t01-w03/slice-initial/README.md)
preserve three raw failed attempts and two exact private-source snapshots.
The original predicate-based tag projection failed type conversion; a
direct ordinary tag constructor resolves it. Inline view/store composition
then reached conversion fuel. Explicit checked constructor projections,
read/write definitions, key facts and view observations resolve that
composition at the existing budget. These findings remain failed positive
attempts and are not counted as false claims. No checker rule, fuel, axiom,
function extensionality, record eta or unfinished C# theorem is added.

An adjacent read-only `/usr/sbin/sysctl` query outside the workspace sandbox
identifies MacBookPro18,4 / Apple M1 Max, 68,719,476,736 physical memory bytes
(64 GiB) and 10 physical / 10 logical CPUs. The
[hardware receipt](../migrations/go-practical/t01-w03/local-resource-host.json)
retains the exact query output, timestamp and raw slice-report hash. Original
report model/RAM nulls remain intact. This identifies the development host;
it is not an installed native Linux or isolated-budget receipt.

The separate final slice `--check` replay passes all 90 candidates through
both checkers and reproduces exact native bytes. All 40 new adversarial
evidence tests pass. Selected validation covers the changed private producer,
descriptor/physical-store composition, complete cells/frames/padding,
array/view identity, actual native bounds observations and retained evidence.
All 459 earlier candidates are remeasured because they share the producer:
27 representation, four local-Nat, 48 balanced, 55 unsigned-prefix,
32 signed-prefix, 40 guard, 72 continuing-prefix, 108 halting-return,
46 byte-tail and 27 bounded-index cases. Their canonical bytes, both
standalone indexes and all ten complete case indexes/outcomes remain
unchanged against the predecessor commit. All 221 affected earlier evidence
tests pass, giving 261 selected tests with the 40 new tests. Unchanged
expensive linear-word proofs are not needlessly replayed.

Offline locked release Clippy passes with warnings denied. Explicit current
private Rust formatting, pinned Go formatting, changed Python syntax,
six changed documents/113 local links, ordered ledger sections and Git
whitespace checks pass. W01's unchanged production/regression identity audit
and W02's retained 39-case closure report pass. Unaffected production suites
and unchanged W01/W02 adversarial helpers are not rerun; the selected
validation covers the actual private producer/evidence changes. No incomplete
measurement attempt remains. Production frontend/checker, normative
specification and public registry remain at W01.

Absolute-address coverage and no-wrap, descriptor/live-region validity,
ownership/admission and complete Go operation/source bindings remain due.
`SliceDefined` is only the local nil/signed-index guard; it does not discharge
those representation obligations. General slicing, overlapping `copy`,
bounded `append`, remaining byte/control laws and aggregate resource closure
also stay open. W03 remains In progress and W04 has not started. The full
T01 gate stays deferred to W05 and the full authorized 11-stage objective
remains active.

## 19. T01-W03 computed descriptor validity and absolute access closure

Predecessor checkpoint: `f6d56227394536083e703c20101bbbe66171369c` retains
the descriptor/store and array-copy ingredients. The new
[access producer](../probes/go-practical/certificates/src/word/balanced/signed/bytes/bounds/slices/access.rs),
[validity composition](../probes/go-practical/certificates/src/word/balanced/signed/bytes/bounds/slices/access/validity.rs)
and [full-value embedding](../probes/go-practical/certificates/src/word/balanced/signed/bytes/bounds/slices/access/validity/embedding.rs)
connect them to actual absolute-address safety. Source numeric fields,
storage length and indexes remain full 64-bit words. Descriptors retain
512 quantified packing bits, elements 128 bits and two logical region names;
no source pointer operation is admitted.

Non-nil validity computes a live region, nonnegative offset/length/capacity/
storage, length no greater than capacity, the actual zero carry of
offset-plus-capacity, and the actual resulting extent no greater than
storage length. Its 65-bit extent is the computed carry and modular64 sum,
rather than a new Go source type or a caller no-wrap oracle. Nil validity
requires zero length/capacity and leaves inaccessible offset/region ghosts.
Checked descriptor projections prove that assignment copying and full
re-slicing preserve computed validity.

An ordinary six-state comparison/carry relation derives offset-addition
monotonicity across complete words. Computed validity and the actual local
signed index guard then establish the live-region fact, no index-addition
carry, a nonnegative absolute address and strict physical storage bounds.
Only local Boolean flags and bit addresses are split; source numeric values
and payloads remain universally quantified. No caller per-access fit or
safety premise replaces the actual conditions.

The [matrix and semantics](../migrations/go-practical/t01-w03/slice-access/README.md)
keep semantic storage bounds B=0/1/8/32/127/128/129 separate from internal
padding P=1/1/8/32/128/128/256. The checked unsigned-prefix composition proves
equality of the complete canonical 64-bit absolute value to its zero-extended
low-bit reconstruction, then congruence transfers it to a whole-cell read.
Both finite backings and all padding remain quantified. Nil is unindexable;
valid storage bounded by zero has no local defined access. There is no
zero-storage indexed-read positive or promotion of padding to admission.

The pinned [original-Go observer](../probes/go-practical/access_reference.go)
and [independent bit-loop model](../probes/go-practical/access_differential.py)
compare raw computed validity/address arithmetic separately from actual Go
slice construction and subsequent indexing. Erased-validity physical-bound
witnesses use invalid raw length/capacity metadata that Go cannot construct.
Erased live-region validity uses a forged dead model binding, while Go's
real allocation remains readable. Erased storage bounds use legal Go
N=P+1/index=P reads whose complete value differs from the low-bit
reconstruction. These domains remain distinct from bounded source witnesses.
Type failure alone is not a falsity witness.

A total finite-heap lift still reads low bits outside its admitted domain.
Consequently its self-read equality can remain true after bound erasure;
that is not evidence for complete numeric address coverage. The explicit
full-value theorem closes that ordinary gap, and false finite-read
mutations instead replace a nonzero whole payload by zero. Observer-only
allocation, `math/bits`, I/O and panic recovery stay outside the proof
closure. Native observations establish no captured Go function theorem.

The 104 candidates cover nine claim families, with four metadata/exclusion
families at B0. Two fresh generations are identical. Both checkers accept
52 zero-axiom claims with matching hashes and reject 52 false mutations at
core type checking. No new fuel failure is counted as a semantic negative.
The B128 complete-value candidate has 420 declarations, 21,701 terms and
248,640 bytes. Its observed Rust/Go resources are 2.657/5.028 seconds and
22,593,536/68,157,440 resident bytes. B128 keeps 32,768 backing bits and
B129 keeps 65,536 including padding. Direct-child wall/CPU/RSS may overlap
other private replays; these are not aggregate or frozen profile budgets.

All 1,307 original-Go/bit-loop observations match: 157 proof-domain reads,
174 source reads, 98 rejected constructions and 1,035 subsequent index
panics. Overlapping categories include 21 raw-invalid tuples with a true
local guard, seven constructible forged dead bindings, 11 source reads
outside the semantic bound, four readable padding cases outside admission,
and seven source reads whose full address differs from reconstruction.
Every false mutation has an explicit domain-classified witness; exact
input/output bytes, binary identity and resources remain retained.

Selected validation covers the changed private producer, computed full-word
validity/guard/address composition, semantic versus padded capacity, full
cells and native/evidence domain distinctions. All 46 new adversarial tests
and 261 affected earlier evidence tests pass, for 307 selected tests. The
shared producer requires all 549 prior candidates to be remeasured:
27 representation, four local-Nat, 48 balanced, 55 unsigned-prefix,
32 signed-prefix, 40 guard, 72 continuing-prefix, 108 halting-return,
46 byte-tail, 27 bounded-index and 90 slice-store cases. Their canonical
bytes, two standalone indexes and all 11 complete case indexes/outcomes
remain unchanged against the predecessor commit. Earlier prose resources
describe their original checkpoints; refreshed reports retain current
source/binary identities and observations. The adjacent hardware receipt is
refreshed to the exact current slice-store report hash, preserving original
model/RAM nulls. Unchanged expensive linear-word proofs are not replayed.

Offline locked release Clippy passes with warnings denied. Formatting checks
cover the 11 changed current private Rust files and pinned Go observer;
changed Python syntax passes. W01's 77-owner/198-regression-file/83-artifact
identity audit and W02's retained 39-case closure check pass. Unaffected
production suites and unchanged W01/W02 adversarial helpers are not rerun.
Production frontend/checker, normative specifications and public registry
remain at W01.

The separate final access `--check` replay passes all 104 candidates through
both checkers and reproduces exact canonical certificate and original-Go
input/output bytes. Final review checks six documents/131 local links,
balanced fences, ordered ledger sections, source/diff scope and Git
whitespace. No incomplete measurement attempt remains.

No axiom, checker rule, conversion fuel, record eta, function extensionality,
smaller source domain, stronger Go entry condition or unfinished C# theorem
is introduced. Ordinary computed validity/access closure is complete for
these ingredients; production Go capture/model/ownership/operation bindings,
general slicing, overlapping `copy`, bounded construction/`append`, remaining
byte/control laws and aggregate resource closure remain due. W03 stays
In progress, W04 has not started and the full T01 gate remains deferred to
W05. The full authorized 11-stage implementation objective remains active.

## 20. T01-W03 general signed two-/three-index slice closure

Predecessor checkpoint: `94752d34168df9aa71dc420126bb25fdf29bace5` retains
the computed validity/access ingredients. The new
[reslice model](../probes/go-practical/certificates/src/word/balanced/signed/bytes/bounds/slices/access/validity/reslice/model.rs)
computes the actual signed two-index `0 <= low <= high <= capacity`
and three-index `0 <= low <= high <= maximum <= capacity` conditions.
Source words remain 64 bits; descriptor packing retains 512 bits, cells
128 and both logical backings/padding remain quantified. Region names
remain logical identities, not source pointer bits.

Checked modular inverse/commutative/associative identities and actual carry
composition derive full-width non-strict endpoint bounds. A six-state
relation preserves the count of the two carry chains on each association.
Equal endpoints, capacity-end empty views and nil cases stay in scope.
Canonical constructor facts are discharged inside the proof and are not
added as caller assumptions. No checker rule, conversion fuel, axiom,
record eta, function extensionality or unfinished C# theorem is introduced.

[Validity preservation](../probes/go-practical/certificates/src/word/balanced/signed/bytes/bounds/slices/access/validity/reslice/preservation.rs)
connects new offset/length/capacity and the actual 65-bit extent back to old
computed validity. Two-index capacity remains old capacity minus low;
three-index capacity is explicit maximum minus low. Substituting the new
length for the first, or retaining the old capacity in the second, changes
actual observable behavior. Nil and non-nil empty tags remain distinct.

[Mapped guard and read composition](../probes/go-practical/certificates/src/word/balanced/signed/bytes/bounds/slices/access/validity/reslice/readable.rs)
prove that each actually defined new index maps to a defined index in the
old descriptor expanded to its existing capacity. This includes re-slicing
above the old logical length. Actual relative-index no-wrap and complete
physical/key equality preserve full 128-bit reads in the same backing.
Algebraic totalized reads alone do not discharge the source guard.

The [matrix and semantics](../migrations/go-practical/t01-w03/reslicing/README.md)
retain semantic storage bounds B=0/1/8/32/127/128/129 separately from
padding P=1/1/8/32/128/128/256. B0 emits only validity claims, avoiding a
vacuous indexed-read positive or a true zero-capacity control mislabeled as
false. The 100 candidates cover both source forms and validity, exact
capacity, mapped guards and complete-read families, with two identical
fresh generations for each group.

The [original-Go observer](../probes/go-practical/reslice_reference.go) and
[independent bit-loop model](../probes/go-practical/reslice_differential.py)
match all 4,261 observations: 4,103 source operations, 130 operation panics,
28 rejected constructions, 692 reads and 3,411 subsequent index panics.
Overlapping categories include 658 proof-domain reads, 388 tail exposures,
602 restricted three-index capacities, 14 nil results, 1,765 non-nil empty
results and 692 observed shared-backing updates. Twenty-two source reads
are outside the semantic bound, eight remain inside padding but outside
admission, and 14 lose complete-address coverage. They remain valid Go
reads, not invented capacity panics.

Seven source panics nevertheless have a valid total model result: negative
equal variable endpoints on a nil slice yield zero modeled length/capacity.
Thus output validity cannot replace the actual range condition. Invalid raw
parent descriptors, forged dead bindings, construction failures, source
operation panics and subsequent index panics stay distinct. Every false
mutation has an explicit typed source witness, separate from type failure.
Observer-only allocation, writes, I/O and recovery establish neither a
captured-source function theorem nor borrowed-input write permission.

[The diagnostic archive](../migrations/go-practical/t01-w03/reslice-initial/README.md)
retains three raw failed-positive attempts in two exact source groups.
Rust localization accepts two earlier prefixes, then rejects the first
restored sum. Its inverse theorem used nested `not(true)` as the incoming
carry; silently annotating it as `false` was not checked transport.
Explicit Boolean congruence corrects that composition without changing the
claim. Producer-only guard-rewrite ordering and comparison-direction
assertions stop before an operation certificate; they are not counted as
kernel rejections or false claims. The corrected numeric, source guards,
validity and mapped-read library stages pass both unchanged checkers.

Both checkers accept 50 zero-axiom candidates with matching hashes and
reject 50 false guard/capacity/original-length/read mutations at core type
checking. No current fuel outcome is counted as a semantic negative. The
B128 three-index complete-read candidate has 594 declarations, 39,715
terms and 478,655 bytes. Its observed Rust/Go resources are 2.909/6.934
seconds and 42,893,312/143,851,520 resident bytes. Direct-child wall/CPU/RSS
may overlap private replays; these are no isolated or aggregate budgets.

Selected validation covers the new full-width endpoint arithmetic, source
guards, nil/equal endpoints, exact capacities, actual mapped definedness,
whole-cell sharing and evidence/witness domains. All 48 new adversarial
tests and 307 affected earlier evidence tests pass, for 355 selected tests.
The shared producer requires all 653 prior candidates to be remeasured:
27 representation, four local-Nat, 48 balanced, 55 unsigned-prefix,
32 signed-prefix, 40 guard, 72 continuing-prefix, 108 halting-return,
46 byte-tail, 27 bounded-index, 90 slice-store and 104 validity/access cases.
Their canonical bytes, two standalone indexes and 12 complete case indexes/
outcomes remain unchanged against the predecessor commit. Earlier prose
resources describe original checkpoints; refreshed reports retain current
source/binary identities and observations. The adjacent hardware receipt
is refreshed to the current slice-store report hash, preserving model/RAM
nulls. Unchanged expensive linear-word proofs are not replayed.

Offline locked release Clippy passes with warnings denied. Explicit Rust
formatting covers the 17 changed current files and excludes preserved
historical snapshots. Pinned Go formatting and changed Python syntax pass.
W01's 77-owner/198-regression-file/83-artifact identity audit and W02's
retained 39-case closure check pass. Unaffected production suites and
unchanged W01/W02 adversarial helpers are not rerun. Production frontend/
checker, normative specifications and public registry remain at W01.

The separate final reslice `--check` replay passes all 100 candidates through
both checkers and reproduces exact canonical certificates and original-Go
input/output bytes. Final review checks eight documents/168 local links,
balanced fences, ordered ledger sections, source/diff scope and Git
whitespace. No incomplete measurement attempt remains.

Ordinary signed two-/three-index validity, exact-capacity, mapped-guard and
backing-read closure is complete for these ingredients. Production Go
capture/model/ownership/operation bindings, overlapping `copy`, bounded
construction/`append`, remaining byte/control laws and aggregate resource
closure remain due. W03 stays In progress, W04 has not started and the full
T01 gate remains deferred to W05. The full authorized 11-stage implementation
objective remains active.

## 21. T01-W03 overlapping min-length copy closure

Predecessor checkpoint: `9f93641e86bccc77c7c0d38fdb48190828fd5522` retains
ordinary signed reslicing. The new
[copy model](../probes/go-practical/certificates/src/word/balanced/signed/bytes/bounds/slices/access/validity/reslice/copying.rs)
computes unsigned `min` of the complete canonical 64-bit destination/source
lengths. An explicit heap lambda reads every copied cell from the immutable
pre-state heap, so both overlap directions retain Go's source-value behavior.
Neither a forward assignment loop nor the post-state source is substituted.

Checked complement/inverse/commutative identities derive full64 relative-index
cancellation without a new checker rule, axiom, fuel adjustment, eta or
function extensionality. Constructor canonicalization proofs remain internal.
Computed validity derives nonnegative lengths, including nil length zero;
actual `0 <= index < min_length` then derives both actual source/destination
index guards. Nil and non-nil empty states remain distinct. The complete
128-bit copied read, unchanged destination read beyond the copied prefix,
and other-region frame are ordinary checked relations over the same heap.
Production Go capture/operation/ownership bindings remain due; these heap
lemmas do not grant write permission for borrowed input regions.

The [matrix](../migrations/go-practical/t01-w03/bulk-copy/README.md) has 84
candidates at semantic bounds B=0/1/8/32/127/128/129, independently of padding
P=1/1/8/32/128/128/256. Both complete stores and all padding cells remain
quantified. B0 emits count properties only. B1 omits the post-state-source
mutation, which requires multiple overlapping cells to be false. Every
emitted mutation has an explicit bounded-source counterexample, separately
from the proof's rejection class.

The [actual Go observer](../probes/go-practical/copy_reference.go) agrees with
all 341 [independent bit-loop/snapshot observations](../probes/go-practical/copy_differential.py).
It records 278 actual copies, 262 valid bounded operations, 72 copied reads,
22 destination reads outside the copied prefix and 16,595 other-region cells
within the proof domain. Categories overlap: 33 right and 20 left overlaps,
23 different-region operations, 108 unequal-length operations, 48 nil and
53 non-nil empty operations. Twelve observations refute a naive forward loop;
17 refute the post-state-source substitution. Actual descriptor metadata is
preserved, and complete heap cells outside the write range are compared.

The same corpus retains 63 rows with a construction rejection, 253 rows
with a subsequent index panic, six constructible operations with invalid live bindings, ten reads
outside B, three within padding but outside B, and seven source reads without
complete finite-address coverage. Copy on nil/empty slices is not treated as
a panic. Invalid raw metadata, forged region bindings and source construction/
index failures are distinct. Native allocation, I/O, mutation and recovery
are observer-only; no complete captured-source theorem follows from them.

Both unchanged checkers accept all 42 positive candidates with zero axioms
and matching accepted hashes, and reject all 42 false mutations at core type
checking. No current fuel outcome is counted as a semantic negative. The B128
copied-read candidate has 615 declarations, 40,945 terms and 494,268 bytes.
Observed Rust/Go wall time is 2.839/6.543 seconds and peak resident size is
66,191,360/131,547,136 bytes. Direct-child wall/CPU/RSS can overlap other
private replays; no isolated or aggregate budget is inferred.

The shared producer pins all current Rust source, so all 753 predecessor
candidates were remeasured despite unchanged canonical claims. Their raw
certificates, two standalone indexes, 13 complete case indexes, semantic
outcomes and raw checker verdicts remain unchanged against the predecessor.
Production frontend/checkers, specifications and registry remain at W01.
The hardware receipt is refreshed to the current slice-store report hash,
preserving the original report's model/RAM nulls.

Selected verification covers min-length bounds, actual guards, nil/empty
paths, full64 cancellation, complete copied cells, both frames, overlap,
evidence/witness domains and all affected source/report identities. The
403 selected tests comprise 48 new copy tests and 355 affected prior evidence
tests. Initially 402 passed; one negative fixture crosses modules and therefore
hits module identity before its intended certificate-identity check. Changing
that fixture to cross two accepted results inside the same module makes the
focused single-test rerun pass at the intended guard. The other 402 passing
tests are unaffected by that test-only edit and are not repeated. Unchanged
expensive linear-word diagnostic proofs and unrelated production suites are
not rerun. W01's 77-owner/198-regression-file/83-artifact identity audit and
W02's retained 39-case closure check pass.

A separate final copy `--check` replay passes all 84 candidates in both
checkers and reproduces exact canonical certificates and original-Go input/
output bytes. No incomplete measurement attempt remains. Offline locked
release Clippy passes with warnings denied; explicit Rust formatting checks
the 16 changed current files and excludes historical snapshots. Pinned Go
formatting, changed Python syntax, five changed documents/132 local links,
balanced fences, ordered ledger sections and Git whitespace pass. These
checks match the private producer/evidence/doc changes. The complete T01 gate
remains deferred to W05, and unaffected production suites are not run.

The next collection probe must derive fresh initialized construction from
actual signed length/capacity checks, then use a proved no-reallocation
condition for preallocated `append`. It must preserve local aliases and
capacity tails while keeping fresh writable storage disjoint from borrowed
inputs. Failure to establish that capacity condition is a profile-domain
obligation, not a Go panic; reallocating append remains deferred by the design.

Production capture/model/ownership/operation bindings, bounded construction
and capacity-preserving `append`, remaining byte/control composition and
aggregate boundary/resource closure remain open. W03 stays In progress,
W04 has not started and the complete T01 gate remains deferred to W05.
The full authorized 11-stage objective remains active.

## 22. T01-W03 fresh construction and non-reallocating append

Predecessor checkpoint: `dc9a5a9c056f21c9c8f15694774b3937c193661a` retains
ordinary overlapping-copy closure. The new
[construction model and matrix](../migrations/go-practical/t01-w03/construction/README.md)
retain full64 operands, 512-bit descriptors and complete 128-bit cells in
both logical stores/padding at B=0/1/8/32/127/128/129. No source pointer bits,
new checker rule, fuel adjustment, axiom, eta or unfinished C# theorem is used.

Actual signed make size checks produce a non-nil descriptor, including the
zero-length case. Every element within capacity is initialized, so later
re-slicing exposes zero values rather than arbitrary discarded tails. A
computed old-live-mask false fact at the allocation's region excludes equality
with any old live query region. Boolean congruence proves complete old-live
frames; an unrelated NoAlias Boolean does not replace that derivation.
Allocation resource exclusions and private bounds remain explicit.

The append model computes actual full-word length/carry and capacity checks,
then an absolute element interval from old offset/length to the result end.
Each write reads the immutable pre-append source. Existing validity derives
no-wrap/nonnegative/bounded endpoints, result validity and actual result/
saved-full index guards. Source indexes map to old length plus index. Old
descriptor length, nil tag, region, offset and capacity retain their meanings;
full aliases see appended values while the old prefix, unused capacity,
other regions and queries outside the actual interval remain unchanged.

The composed make/append proof preserves an old source's validity as the live
mask grows, preserves its indexed values across fresh allocation, then invokes
the ordinary append read. The result therefore contains values from before
both operations. Formerly live regions stay unchanged, and empty preallocation
plus empty append remains non-nil. The common bounded-span composition and
actual no-reallocation condition are explicit; production source/effect
capture still has to prove generated obligations. Borrowed-write permission
and a complete filter-loop theorem are not inferred from this conditional law.

Representative make validity/zero/read/live-frame, active-index, append
validity/result/alias index/read/prefix/tail/other/physical-frame and composed
read/borrowed-frame stages pass both unchanged checkers. The full matrix has
276 candidates (138 correct and 138 false mutations) across all seven semantic
bounds. B0 avoids vacuous indexed/frame negatives; B1 omits post-state-source
substitution without enough overlapping cells to falsify it.

Actual Go and independent bit-loop/snapshot observations match 2,806 cases.
They include 758 successful make operations and 170 make panics, 211 reads of
initialized capacity tails, 796 standalone append operations, 354 successful
standalone reallocations, 128 bounded copied/alias reads, 150 right and 50
left overlaps, 60 post-state-source and 40 naive-forward-loop counterexamples.
The composed corpus has 949 successful operations, 58 bounded source reads,
126 old-live frame queries, 119 non-nil empty results and 350 successful
reallocations. Growth capacity is checked against the language's constraints;
its runtime policy is not guessed or admitted as a proved append operation.

All 138 false mutations have typed witnesses. Domains distinguish bounded
source behavior, actual make panics, forged fresh-region bindings, and source
reallocation that succeeds outside the no-reallocation condition. Twenty-eight
make reads outside complete finite-address coverage still return zero;
value equality alone cannot raise the semantic bound. Invalid metadata,
construction failures and subsequent index attempts remain separate.
Observer-only seeding, allocation, recovery, I/O and value-based alias probes
do not complete production capture/ownership bindings.

The independent observation reference initially predicted a full-view attempt
when its parent descriptor failed construction. It was corrected to preserve
neither a read nor a read panic in that case, and a selected regression keeps
the raw invalid nil metadata in scope. Explicit no-reallocation observations
beyond B were added to avoid an empty boundary category. These reference/
coverage corrections are not kernel failures or false-claim proof results.

Both unchanged checkers accept 138 zero-axiom candidates with matching
accepted hashes and reject all 138 false mutations at core type checking.
No current resource failure is counted as a semantic negative. The B128
composed-read candidate has 641 declarations, 44,978 terms and 545,819 bytes;
observed Rust/Go wall time is 4.448/10.922 seconds and peak resident size is
64,831,488/139,575,296 bytes. These are direct-child observations, not isolated
or aggregate budgets.

Initial measurement and a separate final `--check` replay pass all 276
candidates in both unchanged checkers and reproduce exact canonical
certificates and original-Go input/output bytes. The new runner measures three
independent capacity groups concurrently, retains canonical ordering and
atomically records active attempts. That count is local to the runner;
wall/CPU/RSS can overlap other work and the 60-second ceiling is not a frozen
or isolated profile budget. No incomplete attempt remains.

Shared-source identity requires refreshing all 837 predecessor reports.
Their canonical certificates, two standalone indexes, 14 complete case
indexes/outcomes and raw checker verdicts remain unchanged against the
predecessor. Production frontend/checkers, normative specifications and
registry remain at W01. The adjacent hardware receipt is refreshed to the
current slice-store report hash, preserving its original model/RAM nulls.
W01's 77-owner/198-regression-file/83-artifact identity audit and W02's retained
39-case closure check pass.

All 473 selected tests pass: 70 new and 403 affected prior evidence tests.
They cover source/proof/witness identity, initialization, nil/empty, actual
guards, all frames, aliases, overlap, growth scope, failed-parent observations
and preallocation composition. This scope follows the changed producer and
all source-bound report consumers. Offline locked release Clippy passes with
warnings denied. Explicit formatting covers the 22 changed current Rust
files and excludes historical snapshots. Pinned Go formatting, changed Python/
JSON syntax, five changed documents/136 local links, balanced fences, ordered
ledger sections and Git whitespace pass. Unchanged expensive linear-word
proofs, unaffected production suites and unchanged W01/W02 adversarial
helpers are not rerun. The full T01 gate remains deferred to W05.

Production Go capture/model/ownership/operation bindings, remaining byte/
control composition and complete application proofs, and aggregate boundary/
resource closure remain due. W03 stays In progress, W04 has not started and
the complete T01 gate remains deferred to W05. The full authorized 11-stage
objective remains active.


## 23. T01-W03 guarded substring and bounded concatenation

Predecessor checkpoint: `7eddc34ac28003c78eddce2223bf267bfba39bfc` retains
fresh construction and capacity-preserving append. The new
[string-operation model and matrix](../migrations/go-practical/t01-w03/stringop/README.md)
keep complete 64-bit lengths/indexes and arbitrary eight-bit bytes. The
private workloads B=0/1/31/32/33/255/256/257 remain separate from padding
P=1/1/32/32/64/256/256/512; all padding bytes and both concat stores remain
quantified. No source/checker rule, axiom, fuel or public profile changes.

Actual signed substring checks `0 <= lower <= upper <= length` derive
nonnegative bounded result length and the actual old-source index for every
defined result index. The complete result byte equals the old byte at
`lower+index`. Concat derives no unsigned length carry from actual nonnegative
input lengths. An explicit result-bound obligation derives the result sign;
actual left/right source indexes lift into the result and preserve complete
byte values. Two input bounds do not imply the concat output bound. The
`concat-length` candidate deliberately retains that obligation explicitly.
Defined-result observations preserve canonicalizing both operations' input
tails, using derived source indexes and the existing byte-tail theorem.

Semantic bounds and actual index checks also derive equality of each complete
canonical index with its zero-extended finite reconstruction. Value equality
at an accidentally equal out-of-bound cell is not used as index coverage.
The kernel checks ordinary difference/restoration/monotonicity/cancellation
proofs; constructor canonicalization and carry/fit facts stay internal.
The pinned compiler/specification inventory identifies the native semantics
reference, while production source/effect/operation linkage remains due.

The matrix has 248 candidates: 124 correct and 124 false mutations. B0 has
five length/carry families and omits empty-domain indexed false claims; each
nonzero bound has 17 families. False witnesses distinguish bounded source
behavior, actual substring panics after guard erasure, and successful source
concat of length B+1 after output-bound erasure. Source panics, profile-domain
conditions and checker resource failures retain separate meanings.

Actual Go agrees with 1,957 independent full64 bit-loop/byte-loop observations:
902 successful substrings, 1,055 substring panics, 290 empty results, 206
defined substring reads and 696 subsequent index panics. All 1,957 concat
operations succeed; 872 exceed B despite both inputs satisfying it. The
corpus includes 696 left/662 right concat source reads, 360 invalid-UTF8
substring observations and 729 concat observations containing NUL. Eight
source reads lie beyond B and three lack complete finite coverage. Extreme
signed endpoints, multibyte byte boundaries and failed-parent index attempts
remain explicit. Native recovery/I/O/allocation is observer-only.

The initial matrix passes both unchanged checker implementations: 124
zero-axiom claims have matching accepted hashes, and 124 false mutations fail
at core type checking. No current fuel/resource outcome is a semantic negative.
The B256 concat-right-read candidate has 550 declarations, 36,360 terms and
436,117 bytes. Observed Rust/Go wall time is 4.718/7.703 seconds and peak
resident size is 39,026,688/116,948,992 bytes. Direct-child observations can
overlap other work and do not establish isolated or aggregate budgets.

A separate final `--check` replay passes all 248 candidates in both checkers
and reproduces exact canonical certificates and original-Go input/output
bytes. Three independent workload groups run concurrently within the runner;
canonical result order, exact source/binary pins and atomic active-attempt
records are retained. No current incomplete attempt remains. The worker count
and 60-second child ceiling are private measurement settings, not a frozen
profile budget.

All 1,113 predecessor candidates are refreshed because the shared producer
pins all current Rust source. Their canonical certificates, two standalone
indexes, 15 complete case indexes/outcomes and raw checker verdicts remain
unchanged against the predecessor. The complete current matrix has 1,361
candidates: 656 accepted, 676 core type mismatches and 29 retained diagnostic
fuel outcomes. Fuel outcomes are not semantic-negative evidence. Production
frontend/checkers, normative specifications and registry remain unchanged
from W01. The adjacent hardware receipt is refreshed to the current slice-store
report hash, preserving original model/RAM nulls. W01's 77-owner/198-regression-
file/83-artifact audit and W02's retained 39-case closure check pass.

All 537 selected tests pass in four disjoint batches: 64 new tests (12.401 s),
267 affected representation/arithmetic/control/byte/index/access tests
(33.254 s), 40 slice-store tests (1.243 s), and 166 reslice/copy/construction
tests (30.822 s). The selection follows the changed private producer and every
source-bound report consumer. It covers actual range/index conditions, all
byte bits, NUL/invalid UTF-8, explicit output-bound obligations, finite
coverage, failed-parent attempts, both checker identities and false-witness
scope. Unchanged expensive linear-word diagnostic proofs, unaffected
production suites and unchanged W01/W02 adversarial helpers are not rerun.
The complete T01 gate remains deferred to W05.

Offline locked release Clippy passes with warnings denied. Explicit Rust
formatting covers 15 changed current files and excludes historical snapshots.
Pinned Go formatting, changed Python syntax, 19 strict JSON artifacts, six
changed documents/143 local links, balanced fences, ordered ledger sections
and Git whitespace pass. These checks match the private producer/evidence/doc
scope. No GitHub Actions/workflow is created, run or consulted.

Production source/model/effect/operation bindings, checked prefix/suffix and
their exact library/helper binding, remaining byte/control/aggregate and
complete application composition, and aggregate boundary/resource closure
remain open. W03 stays In progress, W04 has not started and the complete T01
gate remains deferred to W05. The full authorized 11-stage objective remains
active.


## 24. T01-W03 byte prefix/suffix and selected body correspondence

Predecessor checkpoint: `170bf04ccde2203de1d2821a3ac3021cea0805df` retains
ordinary substring/concat ingredients. The new
[prefix/suffix checkpoint](../migrations/go-practical/t01-w03/affix/README.md)
uses a separate private Rust crate with complete 64-bit lengths/indexes and
arbitrary eight-bit bytes. Semantic B=0/1/31/32/33/255/256/257 stays separate
from padding P=1/1/32/32/64/256/256/512. Both complete stores and all padding
remain quantified. No production frontend, checker, normative specification,
registry or public selection changes.

Actual nonnegative lengths and the computed length comparison derive both
ordinary substring guards. Restoration/cancellation proves the prefix/suffix
substring length equals the pattern length. The signed length/slice/equality
formulas then correspond to the internal relations. Empty patterns match;
longer patterns return false through short-circuiting. A computed true
source-shaped result derives the actual guard, actual source indexes and
all eight byte bits at every defined pattern index. Semantic bounds derive
complete finite-index reconstruction. Input-tail canonicalization preserves
both relations. The nonnegative facts are derived from actual bounded
lengths in emitted claims rather than added as independent caller oracles.

The native observer compiles the original Go 1.25.0 packages and compares
both actual library calls with literal bodies and independent byte/bit
loops. An AST receipt identifies exactly `strings.HasPrefix/HasSuffix` and
their two `internal/stringslite` callees, including whole-file and formatted
body hashes. The toolchain inventory retains the original package context.
The ordinary formulas and exact identities are private correspondence
evidence; they do not register an adapter or prove the production capture/
import/operation pipeline. No Go package is reduced to manufactured stubs.

All 5,395 original-Go observations match. They include 1,055 prefix and 1,061
suffix matches, 316 empty and 1,988 longer patterns, 330 differing prefix/
suffix results, 207/213 defined matched source reads, and 664 high-bit
rejections despite matching low bits. NUL patterns, invalid UTF-8 and
multibyte inputs remain bytes. Eight outside-B inputs yield 16 finite-
projection counterexamples, and eight actual source reads lack full finite
coverage. Original library calls never panic in this corpus; unguarded
substring and optional-index failures retain distinct meanings.

The matrix has 304 candidates: 152 correct and 152 false mutations. Each
mutation has an explicit witness. B0 omits indexed false controls with empty
domains. The full-byte mutation is meaningful at B1; prefix/suffix confusion
is used only where the bound permits distinct results. Longer-pattern guard
erasure uses zero-valued unobservable source padding. The original call
returns false, unguarded Go slicing panics, and the wrong total model returns
true. The initial arbitrary padding did not provide that particular witness;
an explicit zero-padding row was added before measurement. This reference
fixture correction is not a kernel rejection or semantic proof result.

The [initial diagnostic](../migrations/go-practical/t01-w03/affix-initial/scope.json)
retains an intended positive rejected by both kernels at core typing. An
untrusted rewrite of a small sign expression prevented a later larger
signed-order rewrite; the range lemma retained a residual condition, and its
stronger call failed. The correction orders larger rewrites first and checks
exact theorem endpoints. A separate
[fuel diagnostic](../migrations/go-practical/t01-w03/affix-initial/fuel/scope.json)
retains eager conversion of a closed fold. Explicit checked projection
congruence reuses the prior fold lemma without changing either kernel's fuel
or any semantic bound. Eight diagnostic rows pass integrity checks. Neither
failure class is semantic-negative evidence.

The [standalone crate](../probes/go-practical/affix-certificates/Cargo.toml)
keeps all existing producer sources and input identities unchanged. Its
explicit Rust build recipe extends only generated untrusted candidate code;
an independent replay checks all 46 generated inputs. Exact source and binary
pins surround measurement. The prior 1,361 canonical certificates, two
standalone indexes, 16 complete case indexes/outcomes and raw verdicts remain
unchanged against the predecessor. All 16 retained-report integrity checks,
the W01 identity audit and W02 retained closure check are selected; old kernel
measurements and unchanged adversarial suites need no repeat because their
source, binaries and canonical input bytes are unchanged.

The first new matrix passes: both checkers accept all 152 correct claims
with zero axioms and matching hashes, and reject all 152 false mutations at
core type checking. The separate final `--check` replay passes all 304 candidates through
both checkers and reproduces exact canonical certificates and original-Go
input/output bytes. No incomplete measurement attempt remains. The cumulative retained matrix is 1,665 candidates: 808 accepted,
828 core type mismatches and 29 historical diagnostic fuel outcomes. Fuel
failures remain resource evidence, never semantic negatives.

All 81 new targeted tests pass in three disjoint batches: 70 report/native
tests (20.367 s), ten generation/diagnostic tests (0.103 s), and one adjacent
hardware-receipt check (0.039 s). The selection follows the separate producer
and its evidence/native consumers; unchanged predecessor inputs and checkers
are verified by preservation and report integrity rather than redundant old
kernel measurements or adversarial-suite reruns. Explicit formatting of four
current Rust files, pinned Go formatting, changed Python syntax and offline
locked release Clippy with warnings denied pass.

The B256 suffix matched-read certificate has 626 declarations, 50,922 terms
and 635,594 bytes. Observed Rust/Go wall time is 5.506/11.775 seconds and peak
resident size is 69,599,232/173,080,576 bytes. The adjacent hardware receipt
binds the report and identifies an M1 Max Darwin/arm64 host with 64 GiB of RAM
and ten physical/logical cores, retaining the original report's model/RAM
nulls. Direct-child measurements can overlap. The 60-second child ceiling and
three concurrent workload groups do not establish isolated or aggregate
profile budgets.

Final review passes 16 strict JSON artifacts, seven changed documents/156
local links, balanced fences, ordered ledger sections, six Python sources and
staged Git whitespace. Historical snapshots are excluded from current Rust
formatting. No GitHub Actions/workflow is created, run or consulted.

Production capture/model/effect/operation and registered-symbol bindings are
owned by T02–T06, and complete source-to-certificate applications are required
before their feature stages finish. W03's remaining feasibility work is the
coupled aggregate/control composition and boundary/resource measurements;
those must precede W04's exact scope/budget freeze. W03 stays In progress,
W04 has not started and the full T01 gate remains deferred to W05. The full
11-stage objective remains active.


## 25. T01-W03 coupled struct fields, descriptor addresses and byte matches

Predecessor checkpoint: `f54d8bbbbee66d00c615f9894827e2b3ab34099a`. The
[aggregate record](../migrations/go-practical/t01-w03/aggregate/README.md) and
[independent producer](../probes/go-practical/aggregate-certificates/src/aggregate.rs)
compose complete value-cell equality with computed descriptor validity and
complete prefix/suffix byte observations. Source Amount and length words stay
64 bits; byte comparisons retain every bit. All fields, cell padding and both
complete 64-bit-addressed heaps remain arbitrary. Actual validity/index and
semantic storage bound S derive physical-address reconstruction; no caller
fit, no-wrap, carry or sign oracle is introduced.

The actual original cell's source-shaped success and pattern-index guard
then derive equality of its pattern byte with the byte in the complete cell
read through that finite physical projection. Lengths are bounded by B, while
internal padding P is the next power of two of max(1,B,S). Neither source
bound is raised to P. A local value assignment to Amount preserves the entire
SKU-length/byte subframe through explicit product projections. Indexed laws
observe one selected element; they are not entire collection-program proofs.

Nine B/S pairs are 0/0, 1/1, 32/8, 31/31, 32/32, 33/33, 255/127, 256/128 and
257/129. The matrix has 98 candidates: 49 accepted zero-axiom claims with
matching hashes and 49 false mutations rejected at core typing by both
unchanged checker implementations. B0/S0 has only a nonvacuous value-frame
pair. Every negative has an explicit bounded source witness. An independent
final `--check` replay passes all 98 candidates and reproduces canonical
certificates and exact original-Go input/output bytes. A prior interrupted
replay had a missing session handle and no live process; only the subsequent
complete full replay is used as final evidence.

The ordinary original-Go observer and independent full64 bit-loop/byte model
agree on 732 cases: 664 profile-domain inputs, 685 successful line reads,
38 actual index panics, nine invalid descriptors, 68 outside-domain inputs
and nine whole-cell projection counterexamples. There are 128 matched reads
for each affix relation, 676 negative Amount observations, 190 NUL and
366 invalid-UTF8 source values, seven empty views, three nil views and
595 valid nonzero offsets. Profile bounds, physical coverage and actual
source panics retain separate meanings. Native allocation/recovery/I/O and
reflection are observer-only.

The B256/S128 suffix matched-read certificate contains 649 declarations,
53,098 terms and 661,754 bytes. Observed Rust/Go wall time is 5.289/11.990
seconds; peak resident size is 69,533,696/178,896,896 bytes. Its shared padding
is 256 and each complete cell has 8,192 bits. The 4,194,304-bit finite
observable footprint does not bound the quantified original heap. The
adjacent hardware receipt identifies the Darwin/arm64 M1 Max host with
64 GiB RAM and ten physical/logical cores. Direct-child measurements can
overlap; three workload groups and 60-second ceilings are private settings.

The build recipe extends only generated private Rust code and an independent
replay checks all 47 generated source inputs. Production and predecessor
source files remain unchanged. All 1,665 predecessor canonical certificates,
two standalone indexes, 17 complete case indexes/outcomes and raw verdicts
are preserved. The cumulative retained matrix now has 1,763 candidates:
857 accepted, 877 core mismatches and 29 historical diagnostic fuel outcomes.
Fuel/resource outcomes are not semantic-negative evidence. W01's exact
identity audit and W02's retained 39-case closure report pass.

Three actually checked initial positives were rejected at core typing by
both checkers and remain in [a separate diagnostic](../migrations/go-practical/t01-w03/aggregate-initial/README.md).
The untrusted constructor used negation elimination in the wrong direction
for a literal-bound fact and lacked a checked function-valued field relation.
Checked negation congruence and explicit product-field updates correct the
construction without changing either kernel, fuel, bounds or source widths.
Those initial positive rejections do not count as false-claim evidence.

All 37 new targeted tests pass: 32 report/native tests (1.586 s), four
source-generation/diagnostic tests (0.078 s) and one adjacent host test
(0.012 s). Selection follows the new producer and its consumers; unchanged
predecessor measurements and adversarial suites need no redundant repeat.
Offline locked release Clippy with warnings denied, explicit current Rust
formatting, pinned Go formatting and six changed Python sources pass. Final
review passes eight strict JSON artifacts, eight documents/171 local links,
balanced fences, ordered ledger sections and staged Git whitespace. No
GitHub Actions/workflow is created, used or consulted.

The [W03 exit audit](../migrations/go-practical/t01-w03/feasibility-exit-audit.md)
records a real remaining foundation gap: old Std.Nat.rec is Nat-to-Nat only.
A private additive dependent-induction interface and isolated resource review
must finish before W04. Production source/operation/adapter bindings remain
assigned to T02–T06. W03 stays In progress, W04 has not started and the full
T01 gate remains deferred to W05. The full eleven-stage objective stays active.


## 26. T01-W03 proposed dependent Nat induction and abrupt control

Predecessor checkpoint: `cfb9e961a1e96a3ce29654a2e41767d04db61cb9`. The
[private induction record](../migrations/go-practical/t01-w03/control-induction/README.md)
resolves a real feasibility gap in the legacy Nat-to-Nat recursor. The proposed
additive monomorphic Sort0 `Nat.induct` has a dependent motive, checked zero
case and successor induction step. Its canonical family/constructor/eliminator
signature, generated flag, closed constructor set and monomorphic uses are
validated independently in mirrored Rust/Go checkers. Prior actual checked
types, opaque values and proofs cannot hide universe arguments. The old Nat
and Boolean interfaces, production checker sources, wire format and public
registry remain unchanged. There is no added axiom, eta, proof irrelevance or
fuel increase. W04 must freeze this additional core/foundation interface and
its production/migration ownership; T02 must integrate and verify it.

An ordinary full128-frame iteration schema proves an arbitrary invariant for
arbitrary genuine Nat counts from initialization and body preservation proofs.
Those are generated source-body proof obligations, not caller entry oracles.
A halted wrapper preserves the complete stopped frame for every count. Other
schemas prove continue/post, break without post and return-before-break edges.
The claims do not discharge a particular Go body's preservation, exit or
variant obligations; production source/loop linkage remains T06 work.

Both private checkers accept six zero-axiom ordinary claims with matching
hashes and reject six erased-premise/incorrect-edge candidates at core typing.
Fifteen distinct interface cases accept two legacy/unused-term controls and
reject thirteen malformed declarations/uses. Wrong successor typing rejects
at function typing; the other forgeries reject as invalid declarations.
Those thirteen outcomes are interface-validation negatives, not semantic
counterexamples. Two original-checker diagnostics remain separately archived:
the first assumed the wrong old recursor type, and the corrected new-interface
candidate still lacks a production reduction rule. Production rejection is
explicitly retained rather than reported as production induction support.

All 576 original-Go observations agree with independent full64 bit arithmetic:
144 preserved invariants, 32 bad-entry/16 bad-step refutations, 288 whole stopped
frames, 72 continue/post observations, 144 break exits and 144 return-precedence
observations. Concrete counts span 0/1/2/31/32/33/127/128/129; the ordinary
proofs separately quantify arbitrary Nat. Each of the six false ordinary
claims has an explicit nonvacuous source observation. Native allocation,
recovery and I/O are observer-only; this is not production source binding.

The private recipe checks 75 mirrored checker inputs and 45 generated
candidate inputs. Complete source and both binary identities surround the
measurement. The separate final replay passes all 27 cases with two fresh
generations, exact certificate/native bytes and the same checker binary hashes.
The six ordinary claims and fifteen interface cases retain distinct scopes.
They are not added to the unchanged-production 1,763-candidate matrix as if
the original checkers already supported the new rule.

The invariant candidate has 26 declarations, 287 terms and 2,869 bytes.
Observed Rust/Go wall time is 0.006/0.582 seconds; peak resident size is
2,965,504/5,652,480 bytes. The original resources retain cold startup and the
adjacent hardware receipt identifies the Darwin/arm64 M1 Max host with 64 GiB
RAM. These per-child observations do not freeze a profile budget.

Targeted validation passes seven new Rust and five new Go interface tests,
seven existing Rust and four Go Boolean regressions, 35 report/native/build/
diagnostic adversarial tests (1.953 s) and two receipt tests. Retained test
logs and final mirrored checker source identities agree. Offline locked
release Clippy with warnings denied and current-source formatting pass;
archived snapshots are excluded. Existing production matrices and suites
need preservation checks rather than redundant execution. Final review passes
12 strict JSON artifacts, eight documents/179 local links, balanced fences,
ordered ledger sections, six Python sources, four current Rust files, three
pinned Go files and staged Git whitespace. No workflow/Actions is used.

The [W03 exit audit](../migrations/go-practical/t01-w03/feasibility-exit-audit.md)
now leaves the sequential resource envelope and cumulative review. W03 stays
In progress, W04 has not started, and T01's full gate stays deferred to W05.
The eleven-stage objective remains active.


## 27. T01-W03 sequential representative resource envelope

Predecessor checkpoint: `b6b283b0630314883403bfa159a60c82ff0eabfc`. The
[sequential record](../migrations/go-practical/t01-w03/sequential-resources/README.md)
measures fourteen existing positive proof recipes with one direct child at
a time. The selection addresses the remaining upper/boundary and coupled
resource question: exact sums, descriptor reads, overlap snapshots, borrowed
construction, substring/concat/affix reads, struct/descriptor/byte composition
and arbitrary-Nat invariant/stop schemas. Twelve use unchanged production
checker implementations and two use the separately scoped proposed extension.
All source/report/artifact and checker binary identities are checked.

Each fresh complete-group construction reproduces the selected canonical
certificate. Both source-free verifiers accept it with zero axioms and the
original logical hashes. Construction measures the entire group/library,
not just that one theorem. All fourteen pass. Observed maxima are 7.147 s /
466,616,320 RSS bytes for construction, 5.666 s / 72,499,200 bytes for Rust,
and 13.009 s / 188,055,552 bytes for Go. The largest selected certificate is
698,048 bytes; selected maxima are 55,744 terms and 1,333 declarations.
Per-component maxima may come from different rows. The report pins 195 source
inputs and 22 predecessor artifacts, with the adjacent M1 Max/64 GiB hardware
receipt. Other host processes remain uncontrolled; no isolated-machine or
native Linux release claim is made.

These measurements support retaining proposed byte/storage candidates
256/128 for W04. Boundary 257/129 does not enlarge source admission. W04
must freeze exact semantic/transport/resource limits and executors from this
evidence. Actual source-to-certificate feature applications still belong to
T02–T06 and must satisfy those frozen limits; a budget failure cannot be
reported as proof success, a semantic counterexample or a Go panic.

All eighteen new targeted resource-evidence tests pass (2.454 s). The selected
fresh kernel checks are necessary for resource feasibility and original
canonical/hash preservation; unrelated production suites and old adversarial
matrices are not rerun. Retained-report integrity checks across the complete
W03 evidence pass: 18 original-checker groups/1,763 candidates, 27 separate
private control/interface cases and fourteen resource rows. Nine retained
initial/checkpoint diagnostic families pass integrity/classification checks.
Final review passes two strict JSON artifacts, seven documents/178 local
links, balanced fences, ordered ledger sections, three Python sources and
staged Git whitespace. No GitHub Actions/workflow is created or used. T01's full
gate remains deferred to W05. W03's final completion audit follows; no public
profile or complete eleven-stage objective is claimed.


## 28. T01-W03 completion audit and W04 handoff

Reviewed implementation commit: `9d4e6fb7855a66d9fff309e6dbd9143652ee6e21`.
The worktree was clean before these completion-record edits. W01 and W02's
completed predecessors remain unchanged. The
[machine-readable audit](../migrations/go-practical/t01-w03/completion-audit.json)
pins all eighteen original-checker report groups, the private induction report
and sequential resources. The
[readable exit review](../migrations/go-practical/t01-w03/feasibility-exit-audit.md)
matches each explicit W03 representation, conditional-lemma, workload,
projection-negative and resource requirement to its actual retained evidence.

W03 is Complete. Ordinary full-payload nominal/product/error, full64 arithmetic
and genuine-Nat sums, complete bytes/indexes and descriptor/store/control laws
have representative conditional proofs. Proposed byte0/1/32/256 and
slice0/1/8/32/128 workloads, relevant maximum-minus/plus-one probes and coupled
bounds are measured. The necessary dependent-Nat addition is an explicit
freeze amendment with separately scoped mirrored-checker proof evidence;
production integration remains T02 work. No W03 feasibility requirement
remains pending, and production feature applications are still required in
T02–T06 under their existing completion rules.

The final integrity review validates 1,763 original-checker candidates in
18 groups: 857 accepted, 877 core type mismatches and 29 historical diagnostic
fuel outcomes. The fuel outcomes remain non-semantic diagnostics. Separately,
twelve private ordinary claims give six zero-axiom accepts/six core mismatches,
and fifteen interface cases give two accepts/thirteen declaration rejections.
Fourteen selected fresh sequential resource checks preserve exact original
canonical bytes and accepted hashes. Nine initial/checkpoint diagnostic
families retain their actual failure classes. Checkpoint records own the
specific affected tests, independent replays, source/binary/canonical
preservation and lint/format review; unchanged old matrices and adversarial
suites are not redundantly rerun for completion metadata.

This completion-record-only review checks all eight requirement findings,
twenty exact report hashes, strict JSON, four documents/148 local links,
balanced fences, ordered ledger sections and staged Git whitespace. Runtime
or kernel suites need no new execution for these metadata edits.

W04 is Ready. It freezes exact source/operation/check/closure/effect/contract
rules, semantic/transport/resource limits, the additive Nat interface,
changed schema/hash identities, migrations, positive/negative vectors and
one production executor per requirement. W05 still owns inactive publication,
the final source-model feasibility review and the complete `check-fast.sh`
T01 gate. No T is complete; no profile is active and no Linux/public release
or completion of the eleven-stage objective is claimed.


## 29. T01-W04 entry and draft freeze inventory

W03 completion commit: `7e5f94bda47968aeec938098d5e2a324370a6218`, clean
before W04 edits. The [W04 preparation](../migrations/go-practical/t01-w04/README.md)
records 24 requirement groups, 17 planned production/test components and
143 discovered existing implementation/test files with exact hashes. Those
counts describe a draft inventory, not completed coverage or implementation.
Every planned new executor is marked unimplemented. Indirect consumers and
all final migration/retention dispositions still need review.

The active registry's own revision rule requires successor schemas for a
changed common operation vocabulary. The inactive C# v2 context binds its
own C# foundation and cannot silently admit Go. A separate Go-owned v3 family
is the candidate decision; exact schemas/identities/preimages and crossing
vectors remain to be frozen. Existing four-profile source semantics and
certificate verdicts must be preserved, with one atomic installed family.
No release root, active parser, production source or public profile changes.

The resource draft records candidate byte256/storage128, result arity16,
full64 target words and two-slot measured heaps, plus 2MiB/131,072-term/
4,096-declaration and 60s/1GiB proof-budget candidates. These are W03-backed
freeze inputs with explicit headroom, not new runtime guarantees. Typed region
accounting, zero-capacity semantics, root reference aggregates, closed-error
invocation, contract/variant readiness, capture/count budgets and every wire/
hash/migration/vector row still need exact review. No design priority is
removed to make the draft easier.

Validation is selected for metadata/document changes: strict JSON, exact
baseline and discovered/anchor hashes, unique requirement/component IDs and
planned executor/test ownership, resource identity/headroom, local links and
Git whitespace. The check passes three strict draft JSON files, 24 unique
requirements/17 owners, 143 discovered-file identities, measured budget
headroom, three documents/107 local links and ordered ledger sections.
Unchanged production and old kernel suites need no repeat.
W04 remains In progress, W05 is blocked by W04 and the complete T01 gate
remains deferred to W05. The full eleven-stage objective remains active.

## 30. T01-W04 root/error/empty-slice rules and native vectors

Entry: `9cdbb85d3c45fe973788b4923850ec2ab643b161`, clean before private
observer edits. The
[semantic decisions](../migrations/go-practical/t01-w04/type-error-zero-decisions.md)
give 26 draft TG/ER/ZS rules and planned production/test ownership.
Root structs/fixed arrays can carry slice descriptors; slice backing cells
must be plain. Expanded type graphs include all closed-error payloads and
must be acyclic. Root copies preserve descriptor associations rather than
deep-copying backing.

The closed error universe is explicit for inputs and includes all reachable
concrete boxing cases. Application non-pointer Bool/integer/struct payloads
retain complete values and nil distinctions. Boxing uses the exact Error
signature without executing its body. Direct/finite closed interface calls
include every target body, its contract/effects and a nil safety obligation.
Open interfaces, pointer errors and error-to-error comparison remain excluded.
An error carrying a slice-bearing root payload does not become a plain cell.

Capacity-zero non-nil slices use a live typed NoAccess representative without
granting separation or write permissions. The nil tag is retained. Empty
positive-capacity views keep their real capacity-visible tails and sharing.
Zero-count copy in either direction and empty append preserve backing.
Positive append to capacity zero succeeds in Go and leaves the v1 no-growth
domain obligation unproved; it is not a fabricated runtime panic.
The numeric region-count policy and ordinary normalization/frame proof review
remain pending. This checkpoint does not claim those proofs are implemented.

The [49 source vectors](../migrations/go-practical/t01-w04/source-vectors.json)
name an intended executor and test owner for each future profile outcome.
The [native receipt](../migrations/go-practical/t01-w04/scope-observations/observations.json)
retains all intact source bytes and native output with eight input/source pins.
Original Go reports 43 successful type checks, five type errors and one parse
error. Separately, the prospective profile vectors expect 22 accepted models,
one accepted model pending safety, one pending domain, one configuration
rejection and 24 source rejections. These are design expectations; no practical
frontend has executed them.

The private observer uses the exact Go1.25.0 darwin-arm64 distribution, whose
14,496-file tree is revalidated against the retained archive. Type inspection
explicitly selects gc/amd64 sizes and language go1.25.0. Runtime observations
run on the host, outside any verified root; reflection, recovery and the
counter used to observe method evaluation are not admitted product features.
The observer binary hash is
`7cd34b039e0c3c483cb987264b67b2911ba7dfb8a42d98ae5192e4385066f8ad`.
Both runs of each observation are identical, and a fresh build replays the
same input/output and binary identities. This is not a Linux release receipt
or a source/model theorem.

Selected verification covers only the new private observer, report validator,
draft vectors and documentation. `run_scope_probes.py --write` and
`--check` pass; `python3 -m unittest -v test_scope_report` passes 35 checks
(0.104s). They reject dropped/crossed source pins, erased original type errors,
wrong nominal aliases/call targets, repeated method evaluation, lost sharing,
wrong nil tags, lost positive-capacity tails, fictitious append panic,
noncomparable-error equality, type-confused JSON and false completion/support
claims. Strict JSON/owner/link validation and Python/pinned-Go formatting pass.
The [verification record](../migrations/go-practical/t01-w04/scope-observations/verification/verification.json)
retains the commands and tool-captured outputs. Git whitespace is checked at
the commit boundary.
Unchanged production/frontend/kernel and old W03 proof matrices need no rerun.

The complete source/operation/check table, typed loop/contract expressions,
capture/structural/resource limits, exact successor schemas/hash preimages
and exhaustive reviewed migration inventory remain W04 work. W04 stays
In progress; no T is complete. T01's full gate remains deferred to W05.
The installed four-profile family and public selection remain unchanged,
and the complete eleven-stage objective stays active.

## 31. T01-W04 logical operations, evaluation and loop contracts

Entry: `18ac6aee085cefa027ef7064cc1dfdded762f867`, clean before these edits.
The previous goal turn made concrete progress by committing the root/error/
zero-access draft and native evidence. This checkpoint continues that freeze;
it does not replace the full eleven-stage implementation objective.

The [operation table](../migrations/go-practical/t01-w04/operation-table.json)
contains 56 logical families for v1 priorities1–4, each with a signature,
source forms, complete value relation, runtime safety, profile-domain and
permission obligations, effects, evaluation rule and planned primary
executor/test plus independent artifact/VC ownership. GP-OP IDs are design
references, not frozen serialized tags. Integer wrapping, minimum/-1 division,
full shift counts and exact mathematical indices are explicit. String growth
and append no-growth remain domain obligations rather than fabricated panics.
Array/struct equality excludes error fields at every depth.

The [EV/CT/LC decisions](../migrations/go-practical/t01-w04/operation-loop-decisions.md)
specify 25 rules. Parallel assignment freezes operands then stores left-to-right;
local array/struct locations survive whole-value replacement, while slice
targets retain their old descriptors. Field reconstruction uses the current
cell/root. Calls/results/post-state share one invocation. Switch default
position has no priority; switch break is distinct from loop break, and
continue runs its enclosing loop's post.

Array range copies values shallowly and retains descriptor backing; slice
range captures descriptor/length but reads current cells. The constant-length
exception never silently deletes an actual array-returning call. Range
assignment uses the same two-phase destinations. Effective Go>=1.22 binding
transitions happen before post. Every true/false/break/continue/return path
binds its actual complete state.

Selected practical roots require an explicit partial/total termination mode.
Missing loop invariants synthesize true as an untrusted candidate, and total
mode requires checked Nat variants and terminating component calls. Missing
strategy stays pending, not an original Go type error. The draft lists the
remaining ordinary order/well-founded induction, word64/Nat bridge, variant
and abrupt-frame composition members. Existing private Nat.induct evidence
does not establish those general termination/source proofs. Exact overlay
AST, member/instance identities and proof budgets remain freeze work.

The [native evidence](../migrations/go-practical/t01-w04/evaluation-observations/observations.json)
records 54 cases over35 source forms and nine source/input pins under the
unchanged exact Go1.25.0 host distribution. Independent expected facts use
closed values and mathematical counts, rather than the observer's control
implementation. Two executions agree and fresh-build replay agrees. The
observer-only counters, recovery, function literals and JSON aggregation
are outside verified roots. This is not profile admission, a source-model
theorem, a universal termination proof or a Linux release.

The byte/string special copy/append cases preserve NUL, invalidUTF8 and named
types within existing capacity. Their ordinary binding composition and budget
review are explicit outstanding obligations. The operation table does not
pretend the earlier slice-only binding proofs automatically establish them.

Selected checks target the new observer/evidence and changed draft metadata.
`run_evaluation_probes.py --write` and `--check` pass; the new unittest
module passes 52 checks (0.155s), including current-cell merge, old-slice/new-
array locations, call/projection duplication, panic ordering, switch/continue
targets, range snapshot/live reads, byte preservation, full shift counts,
nil-range/count/JSON-type metadata and false support/termination promotion. Cumulative
metadata validation checks exact catalogue consistency, IDs, planned owners,
strict JSON, unchanged 143 predecessor files, old49 scope-evidence integrity,
local links and ledger order; Python/pinned-Go formatting and commit whitespace
are checked. The
[verification record](../migrations/go-practical/t01-w04/evaluation-observations/verification/verification.json)
retains selected outputs and why unrelated production/W03 suites are not rerun.

W04 stays In progress. Full source/type/statement/effect admission, complete
closed overlay/wire schemas and hash preimages, semantic/capture/type/region/
resource limits and enforcement, ordinary foundation signatures/proofs/budget
review and exhaustive migration dispositions remain due. W05 owns inactive
publication, source-model feasibility review and the cumulative T01 gate.
No T is complete, no practical Go profile is active, and the goal remains
the complete T01–T11 sequence.

## 32. T01-W04 general Sort0 strong induction and proposed Bool type selection

Entry: `b728e0f2543a894efb0b1039b4676126bbe6067a`, clean before this work.
The preceding goal turn made progress by committing the logical operation/
loop draft and54 native observations. This checkpoint measures a remaining
foundation requirement without narrowing the complete eleven-stage objective.

The [private proposal and evidence](../migrations/go-practical/t01-w04/nat-termination/README.md)
derive ordinary Nat order and strong induction for arbitrary Nat counts and
arbitrary Sort0 predicates. The step obligation remains explicit and must be
proved by later application members. Predecessor/add/subtraction are definitions
here; complete arithmetic/variant/range/source composition is still due.

The additional core proposal is the fixed canonical
`Bool.recType : Sort0 -> Sort0 -> Bool -> Sort0`, selecting its first type
on false and second on true. It is a proposed trusted computation rule,
independently implemented in private Rust/Go mirrors, not a hidden semantic
axiom or ordinary-definition claim. Full canonical family/constructor/type
checks, closed constructor registration and monomorphism apply, including
neutral/partial/prior checked uses. Existing Nat.rec, Bool rules, retained
Nat.induct, universe/eta/proof-irrelevance/normalization rules and production
source stay intact. CORE_BOOL_TYPE_CASES_V1.md is an additional proposed
specification owner; R019 names its planned production and independent tests.

The retained report covers 16 ordinary claims (8 accepted zero-axiom, 8 altered
candidates rejected with core type mismatch), 20 isolated interface cases
(4 accepts, 16 malformed declaration rejections) and3 legacy certificates.
All39 are checked from identical canonical bytes by both private verifiers.
Accepted export/certificate/axiom hashes agree. Fresh build/replay reproduces
binary and candidate identities. Source receipt counts are143 inputs/77
mirrored checker files, separate from the unchanged143-file predecessor
migration inventory.

Observed maxima are4,596 bytes/464 terms/35 declarations, with slowest checker
samples Rust0.009855s and Go0.337304s. The private 60s child ceiling changes no
normative budget and certifies no complete application cost. Canonical Bool
type selection uses no Bool.cases/Nat.induct fallback in its interface matrix.
Both branches, nongenerated/wrong signatures or constructors, extra constructors,
higher universes, active partial/neutral levels, prior opaque/theorem uses and
unused term nodes have distinct intended boundaries.

Two [intended-positive construction diagnostics](../migrations/go-practical/t01-w04/nat-type-selection-initial/observations.json)
are retained with exact source/checker/candidate bytes. The attempted Nat type
recursor left a computed major neutral; the corrected proposal selects Bool
types directly. A second named History function-type alias failed application
inference; exposing the ordinary Pi type corrected it. Neither failure is
counted as a semantic negative or a checked proof. A mechanical name replacement
also changed five Go field spellings in the initial build; those were corrected
before the type-selection measurements. No production code was involved.

Selected tests cover the proposed private core interface, ordinary proofs and
affected existing elimination rules. Twenty-one report/adversarial checks pass
(0.383s); 7 existing Rust Bool-case and7 retained Rust Nat.induct tests pass;
4 existing Go Bool-case API and5 retained Go Nat.induct tests pass (1.232s).
The old Bool, Nat and arbitrary-Nat invariant certificates retain both-checker
zero-axiom acceptance. Strict JSON/source/snapshot/owner/link/format/whitespace
checks are required at the commit boundary. The
[verification record](../migrations/go-practical/t01-w04/nat-termination/verification/verification.json)
retains selected commands and scope. Unchanged production suites, collection
proof matrices and old native observations need no repeat for this proposal.

W04 stays In progress. Exact joint core/foundation/overlay/wire/hash/limit and
migration freeze, complete source variant termination and application bindings
remain due. W05 owns inactive publication, feasibility review and the complete
T01 gate. No T is complete or public Go profile active. T02–T11 and the full
requested implementation/release scope remain required.

## 33. T01-W04 arbitrary-Nat decreasing-rank finite stop

Entry: `540ee9033324c62b7f204f3bca337f2f4c976e81`, clean before this work.
The preceding goal turn made progress with general Sort0 strong induction and
the private canonical Bool type selector. This checkpoint carries the
foundation forward without changing the complete T01–T11 objective.

The [ordinary variant proof and source-binding decisions](../migrations/go-practical/t01-w04/variant-termination/README.md)
prove finite stopping for arbitrary genuine Nat ranks/counts and complete
128-bit frames. The general theorem accepts an explicit strict rank bound;
the corollary derives rank+1 internally, eliminating a separate bound oracle.
Preservation and strict decrease are required only on continuing backedges.
Already stopped and newly exiting states absorb remaining steps. A checked
constant-zero-rank exit instance supplies every premise and exercises that
boundary. No finite producer unrolling or native machine-word count is used.

The new producer derives ordinary strict-order transitivity, bound lowering,
frame absorption and iteration advance using the retained private Nat.induct,
Bool type selection and Eq rules. No new trusted rule is added; all77 mirrored
checker sources are byte-identical to section32, as is the rebuilt Go checker
binary. The Rust binary changes because it includes the new untrusted
generator. Retained producer/helper snapshots and all production code stay
unchanged. A routine Rust ownership error in the first new producer build was
corrected before proof verification; it was not counted as a semantic result.

Ten ordinary candidates are checked in both implementations: three accepted
zero-axiom claims (general bound, rank+1 and zero-rank exit) and seven altered
candidates rejected with CORE_TYPE_MISMATCH. The alterations weaken
preservation, decrease, entry or bound, relax the general bound by one, change
the stopped result or omit the rank-count successor. They are candidate
rejections, not certified counterexamples. The preceding strong-induction,
arbitrary-Nat invariant and legacy Bool certificates retain zero-axiom
acceptance and matching export/certificate/axiom hashes.

The source binding must separately prove total, safe condition/body/post/callee
execution and exact transition/frame correspondence. H reads a captured
condition/control boundary instead of reevaluating a source expression. Loop
break/return/false-condition exits, continue/post order, switch break targets,
range snapshots and version-specific bindings retain explicit source proof
obligations. A total logical transition cannot hide a diverging Go call. Exact
word/Nat rank bridges, overlay typing, complete state representation and
application/budget composition remain due.

Selected validation covers this new untrusted producer and its evidence,
fresh source/toolchain/binary/canonical-byte replay, current draft metadata,
retained proof/diagnostic integrity, predecessor source pins and formatting.
Twenty-four evidence/adversarial tests pass (0.375s). Fresh replay checks all13
current cases in both implementations. The receipt pins152 source inputs;
observed maxima are8,475 bytes/921 terms/46 declarations. Metadata validation
checks20 strict JSON documents,39 retained cases, two diagnostics,143 unchanged
predecessor files and six documents/152 links. Five new Python sources compile,
and the three new Rust sources pass targeted formatting checks.
The [verification record](../migrations/go-practical/t01-w04/variant-termination/verification/verification.json)
retains commands, results and why unchanged production/core/collection/native
suites are not repeated. The cumulative T01 gate is explicitly deferred to
W05, as required for an intermediate W.

W04 remains In progress. Exact joint source/foundation/overlay/wire/hash/limit
freeze and reviewed migration dispositions remain open. W05 owns inactive
publication, full feasibility review and the T01 gate. No T is complete and no
practical Go profile is active; T02–T11 and full requested scope remain required.

## 34. T01-W04 closed typed overlay and native header scope

Entry: `48ab07a0622c3d5993070f52ddb9eb3b58cdeced`, clean before this work.
The preceding goal turn made progress with the arbitrary-Nat rank stop schema
and its accepted/rejected proof evidence. This checkpoint retains the complete
eleven-stage implementation objective and advances the outstanding overlay
freeze rather than changing its scope.

The [closed typed overlay draft](../migrations/go-practical/t01-w04/overlay-contracts/README.md)
specifies 70 source expression heads and 70 normalized heads, with exact private
root/loop/variant shapes. Logical Bool/Nat/Int remain distinct from source
nominal types; aliases resolve to the same captured type. Input/return/header
phases, old heap observations, mathematical finite sums, word conversions,
full shift counts, byte/sequence/descriptor observations and closed error
tag/payload queries are explicit. Nested error-bearing entry paths include
accessible tails, preserve the global U and assert no alias disjointness.
The source-to-capture-to-normalized identity DAG avoids self-referential hashes;
production identity/preimage and independent importer freeze remains due.

Two identical observations use pinned Go1.25.0 to parse/type check the intact
UTF-8 fixture, including an unselected map function. Native scopes distinguish
named-result/body shadowing, for-init locals, missing future/body declarations,
range := variables before first assignment, outer range variables at nested
headers and range = destinations. Method receivers and source aliases are
captured. Physical byte spans and source/observer pins remain reproducible.
The observer does not execute the application or implement full capture,
closure/source/effect admission or a normalized artifact proof validator.

Twenty-four source-overlay vectors yield five normalized unproved candidates,
one with missing nonrange total-variant candidates and 18 configuration
rejections. Quote's prefix-sum invariant, positional/error postconditions,
header local bindings, nested error domains and value receiver contracts are
typed. Old schemas, wrong contexts/spans, duplicate/stale loop attachment,
wrong nominal/variant/result phases, missing entry error sets, nonplain global
error cells, source calls and external write annotations reject. All generated
invariants/range measures remain candidates; no source/VC proof is discharged.

Review found the prior variant application memo's initial/next-test scheduling
conflicted with LC02. The corrected composition observes invariant/rank at the
header before the current condition. H reads only a stopped tag; N executes
current Test, then false exit or body/binding/post to the next live header.
Init executes once and the next Test belongs to the next N step. This avoids
duplicate tests and preserves effectful-test state. Body backedges decrease
rank even when the later condition will be false. Abstract Nat theorem and all
checked certificate bytes remain unchanged; the prior application scheduling
proposal is superseded, not treated as a source proof.

Twenty targeted tests pass, including a typed witness for every closed head
and exclusion of package variables from header locals. Native predeclared
types cover all admitted word widths even when absent from source signatures;
the unsigned64 maximum shift count is retained exactly.
Fresh build/replay checks original source, observer and normalized candidate
identities. Metadata checks preserve current vector boundaries, both operator
sets, all retained 13+39 proof cases, two construction diagnostics and 143
predecessor inputs; source/format/link/ledger checks are selected for this
private producer/doc change. The [verification record](../migrations/go-practical/t01-w04/overlay-contracts/verification/verification.json)
retains commands/reasons and the T01 gate deferred to W05. Unchanged production
and core/proof/native matrices need no repeat.

W04 remains In progress. Complete source/effect admission, numeric/capture/
region/resource counters, exact shared/Go schema/hash identities, independent
typed/definedness reconstruction, remaining foundation proofs/budgets and
reviewed producer/consumer migrations remain due. No T is complete and no
practical Go profile is active. T02–T11 and full requested scope remain required.

## 35. T01-W04 canonical storage owners and hierarchical typed views

Entry: `9e7e896a6e9be01184c6b173b489fec2f20c8a4b`, clean before this work.
The preceding goal turn made progress with the closed typed overlay and
native header scopes. This checkpoint advances the unresolved storage/counter
boundary while retaining the complete eleven-stage objective.

The [canonical storage draft](../migrations/go-practical/t01-w04/storage-views/README.md)
avoids independent writable stores for parent/child element types. One owner
and validated field/index paths couple nested arrays and struct array fields.
Whole value replacement updates existing views; by-value array copies acquire
distinct storage while descriptor copies retain sharing. Source bounds use
full indices, permissions follow owners through every descendant and NoAccess
zero-capacity markers grant no physical alias/freshness fact.

The private model distinguishes full local value roots from bounded heap
owners. At an exact scope boundary it groups escaping array views by observed
capacity containment, packs accessible root-cell windows without gaps or lost
overlaps, and translates all shared child paths together. Caller roots whose
scopes remain live retain their references; borrowed local owners cannot turn
fresh through promotion. Ended locals are no longer another accessible store.
Returned array values remain separate copies from the original escaping
slices. General exhaustive escape closure, entry/source embedding, padding,
scope/permission and live-state retirement proofs are still required.

Fifteen independently specified native/model observations agree, including
cross-type child writes, whole-parent replacement, sibling preservation,
overlap snapshots, distant capped windows in a 256-element array and helper
returns of caller-local views. Twenty targeted model tests pass, including
128 child views on one owner, exact types/full indices, read-only descendants,
zero-count copy, typed no-access, packing overlap and atomic quota failure.
These are representative probes, not a complete admitted source pipeline.

Five ordinary laws quantify over arbitrary A:Sort0 cells in two 64-cell rows
and two distinct owners. The untrusted producer splits only relevant address
flags (up to 7), never stored value patterns. Existing Bool cases and Eq rules
prove child/parent observations, sibling and other-owner frames. No core rule,
function extensionality, eta or semantic axiom is added. Both production Rust
and Go checkers accept all five positives with zero axioms and matching hashes;
five altered stale/wrong-view candidates fail CORE_TYPE_MISMATCH. Rejection
is not a certified counterexample. Legacy Bool/Nat certificates remain accepted.

The receipt pins 123 inputs and reproduces exact native/binary/certificate
identities after a fresh build. Observed maxima are 51,647 bytes/5,823 stored
terms/23 declarations. Direct children run sequentially with a private 60s
measurement ceiling, not a normative budget. A copied predecessor lockfile
needed offline regeneration for the production source dependencies; no root
lockfile or production source changed. Native snapshot aggregation copies
data before later alias mutations and remains outside verified roots.

The two-slot/128-cell candidates are clarified as positive canonical heap
owners and accessible root storage cells of exact layouts. Child views do not
consume separate physical slots; aggregate payload and local arena costs are
not free. Full live-frame/entry-snapshot accounting and proved retirement,
all structural/resource limits and exact successor reference schemas/hashes
remain due. Planned practical_storage Go lowering and independent Rust
storage/path validation are assigned alongside existing slices/construction
owners. They are not completed production executors.

The [verification record](../migrations/go-practical/t01-w04/storage-views/verification/verification.json)
retains selected native/model/proof replay, metadata and formatting results,
and why unchanged production suites need no repeat. The cumulative T01 gate
is deferred to W05 as required for an intermediate W. W04 remains In progress;
no T is complete or practical Go profile active. Complete T02–T11 source,
application and release scope remains required.

## 36. T01-W04 derived live roots and reusable storage slots

Entry: `cc6874975060743dde67fd399865caf6288d2b8b`, clean before this work.
The preceding checkpoint established hierarchical owner/view coupling and
scoped export, with complete live-root accounting still open. This checkpoint
advances that design while preserving the full eleven-stage objective.

The [lifetime/accounting draft](../migrations/go-practical/t01-w04/storage-lifetimes/README.md)
derives roots from all active declared variables, constant entry argument
values, pending evaluation and joint-result packets, captured targets and
recursive root-array/struct/closed-error payload descriptors. Positive-capacity
empty slices retain owners; nil/NoAccess values do not. Heap cells remain
Plain under the complete global error universe. No caller live mask or escape
list is accepted. Exact source evaluation/scope/call binding and the algorithm's
ordinary root-completeness proof are still required.

Slot reuse retains fresh semantic owner identities: a stale descriptor rejects
instead of aliasing a new owner. Complete scope exit derives surviving escape
references, retires dying owners before quota checks and promotes/rewrites
shared windows atomically. Inherited caller owners remain shared and borrowed
storage cannot become fresh. Ghost identity/history bookkeeping is not a
bounded concrete semantic carrier; canonical finite-slot/world relations,
local arena weights, activation/temporary limits and enforcement remain due.

Active entry values retain their current-store observations after parameter
reassignment. Past snapshots use a separate immutable store namespace. The
same invocation's return-post snapshot is captured before retirement/escape
translation, retaining entry/output/frame observations at that endpoint even
if the caller later reuses a slot. Candidate snapshots/evidence are not an
accepted serialized callee summary or a source/post/frame correctness proof.

Two distinct ordinary statements quantify over arbitrary A:Sort0 data in two
128-cell slots. A mask premise makes replacement of a dead slot preserve all
masked reads; independent old-store reads remain unchanged by current-store
updates. The first proof splits only the slot Boolean, without address/data
enumeration; the second uses ordinary beta/equality rules. No extensionality,
eta, semantic axiom or trusted computation is added. Both unchanged production
checkers accept the two positives with zero axioms and matching hashes. Three
altered claims weaken the mask premise or confuse masked/current/past reads
and fail CORE_TYPE_MISMATCH. Rejection is not a certified counterexample.
Legacy Bool/Nat certificates remain accepted.

Twelve independently specified native/model observations match; 24 targeted
tests check live slot quotas, operand/result/target roots, recursive errors,
copies, entry/post isolation, scope escape and atomic failure. A 1,000-allocation
loop has the native sum 499500 and model peak one positive slot. Physical Go
address reuse, GC, OOM and whole-source bound correctness are not observed or
proved. Observer maps/type assertions/snapshot aggregation remain outside
verified roots. Largest ordinary candidates are3,234 bytes/352 terms/23
declarations. The 60s sequential-child ceiling remains a private measurement.

Planned practical_storage Go lowering, independent Rust storage validation and
their tests keep the existing slice/construction ownership; root/retirement/
translation validation is included in that responsibility. Full ordinary
normalization/lifetime/source proofs, all limits, exact schemas/hash identities,
independent import and migration freeze remain required.

The [verification record](../migrations/go-practical/t01-w04/storage-lifetimes/verification/verification.json)
retains selected model/native/ordinary replay, predecessor/diagnostic integrity,
metadata, source compilation and format checks, with selection reasons.
Unaffected production/kernel suites are not repeated. The cumulative T01 gate
is deferred to W05. W04 remains In progress, no T is complete and no practical
Go profile is active. T02–T11 and the full implementation/release goal remain.

## 37. T01-W04 closed syntax catalog and selected-call closure

Entry: `af62c97bb2788638303b2c9d7e38a5046d664076`, clean before this work.
The preceding checkpoint committed derived lifetime roots and slot-reuse
evidence. This checkpoint advances source-admission freeze preparation while
preserving the complete eleven-stage implementation/release objective.

The [closed source catalog](../migrations/go-practical/t01-w04/source-admission/README.md)
assigns explicit dispositions to all56 concrete pinned Go1.25 AST node kinds
and all18 predeclared builtins. Seventeen AST kinds reject in selected code,
seven describe captured structure and32 have typed/control/effect conditions.
Only append/cap/copy/len/make are conditional builtin candidates. Each row
names planned practical_closure/types/lower and test ownership, with independent
artifact validation assigned to go_practical_artifacts. No production source,
profile, installed registry, core rule or ordinary certificate changes.

An original-source observer parses/type-checks intact single-file packages
using pinned Go1.25.0 and explicit gc/amd64 sizes. It does not execute observed
bodies or initializers. The candidate classifier follows all direct captured
calls from one root, retaining physical UTF-8 declaration identities and
qualified receiver methods. Parenthesized calls and method expressions work.
A generic call retains its statically known body before generic use rejects;
it is not mislabeled as an unknown indirect call. Call cycles and ambient
state yield transitive root-to-declaration diagnostics. Apparently unreachable
branches stay in closure without an accepted unreachability proof.

Unsupported unselected function bodies remain type-check-only. Concrete error
boxing requires the Error() string signature without calling its body. An
explicit concrete method call adds that body; interface dispatch rejects.
Original parse/type errors still reject the original package, including errors
in unselected declarations. Constants and nominal representability are checked
before candidate classification; exact untyped numeric computation is distinct
from runtime floating/complex values and forbidden integer-to-string conversion.

Eighty-one source vectors retain exact native JSON lines and derived results:
31 candidate-unproved,46 rejected candidates, three Go type errors and one
syntax error. Two identical observer runs per source and a fresh pinned rebuild
reproduce native/source/binary identities. All four supported effective
languages appear. The first draft's expected method display names were aligned
to the observer's full captured package path, and an initial new-helper brace
error was corrected before retained evidence; neither was a semantic outcome.

The observer has no importer and is not the production capture/module loader.
W02's retained module/replacement feasibility remains separate. Full captured
type/constant/declaration manifests, compiler directives, adapters, Root/Plain
global-error closure, independent import, source/evaluation/effect/permission/
domain/call/frame/loop proofs and all counters remain required. In particular,
the observer's predeclared error branch defers U/plainness validation; a candidate
never authorizes an arbitrary error payload or unsupported actual application.
Candidate rejection and compiler failure are not certified semantic falsity.

Fifteen targeted tests cover the81-vector matrix, closed catalog, type-only
versus selected/error body scope, transitive paths/cycles/generic calls,
receiver identities, original errors, physical UTF-8 spans and source/target/
binding crossings. The [verification record](../migrations/go-practical/t01-w04/source-admission/verification/verification.json)
retains selected replay, metadata/predecessor/diagnostic integrity, syntax and
format checks with scope/reasons. Unchanged production/kernel and older native/
proof matrices are not rerun. The cumulative T01 gate is deferred to W05.

W04 remains In progress. Complete semantic predicates, numeric/structural/
capture/resource bounds, schema/hash identities, independent import and reviewed
migrations remain due before freeze. W05 owns inactive publication and the
T01 gate. No T is complete; all T02–T11 source/application/release work remains.

## 38. T01-W04 numeric candidates and complete carrier accounting

Entry: `b9ec95fce4361b0a37f6ab9bc3297a772593dac3`, clean before this work.
The preceding checkpoint committed closed source/closure candidates. This
checkpoint advances numeric/counter freeze preparation and retains the full
eleven-stage implementation/release objective.

The [numeric inventory](../migrations/go-practical/t01-w04/structural-limits/README.md)
defines64 separate candidate counters:49 inherited capture/shared/manifest
dimensions, five semantic, four weighted carrier and six proof-resource
dimensions. Original specifications are pinned and production/test owners
assigned. No normative source, installed profile, core rule or proof certificate
changes. Forty-eight inherited numeric candidates stay the same. One concrete
integration measurement requires an explicit practical-only syntax change.

A new offline loaded-syntax observer measures the intact W02 unused-HTTP module
under pinned Go1.25.0/x-tools0.36.0 and Linux/amd64. All materialized standard
and module trees contribute:179 packages,1,025 syntax files,1,417,549 nodes
(1,417,510 standard and39 application). The predecessor's1,000,000 would reject
the required pure-root/unused-handler use case. The practical candidate is
4,194,304, the next power of two above twice the measurement. Existing profiles
keep their1,000,000 rule. Shared function/type limits apply to represented
verification closure rather than all type-check-only library bodies. Complete
sealed capture/export/directive/namespace enforcement remains required.

Fixed abstract units charge full word/Bool slots, all256 byte slots plus string
length, typed slice/location paths, every aggregate field/cell, all global error
payloads and role/owner/observation headers. Values have a2Mi-unit candidate,
individual current/past stores8Mi and total namespaces32Mi. These are not Go
bytes/RSS/proof costs or supported-workload guarantees. A full16-result product
of[256]string costs1,052,689; a[256] array of64-string structs costs4,210,945 and
cannot be assumed to fit from its individual axes. Backing collections are
store totals, with each Plain cell checked as a value;128 cells of64-string
structs therefore use2,105,474 owner units without a spurious per-value failure.

Complete graph validation precedes saturated weight arithmetic, including zero
arrays and inactive error payloads. Nominal/alias edges do not erase source
identity; array/slice/struct/error edges retain exact depth. No huge value
expansion is required. Current owners, inline entry/evaluation/result values,
captured targets and every immutable entry/post namespace are charged. Sharing
one store cannot hide distinct observation packets, while identical contents
in distinct stores do not merge their costs. Full copied values are charged;
shared current heap cells are charged once. Ghost serial/history are not a
proved finite concrete carrier and remain explicitly outside semantic units.

Retained Quote type facts yield input weights257/37/1 and result weights2/2.
A constructed128-Line frame costs1,578 current/past units; return evidence
costs1,421. Packed windows use four current cells while retaining the full256
source cells in post evidence (416 total units). One thousand allocation
iterations have111 peak/36 final units and sum499500. These are accounting
probes, not source executions or allocator/source/finite-ID proofs.

There are192 isolated counter vectors:127 within,64 above, one invalid negative
input below a zero ceiling. Arithmetic preflight avoids unsigned wrap/partial
commit. Failure classes distinguish capture/structural rejection, unproved
domain bounds and resource-pending proof work from Go panics/semantic falsity.
Three constructible array/struct types reach exact2Mi-unit minus/at/plus-one
weights; pinned Go type-checks all three, with measured native sizes
16,711,656/16,711,664/16,711,672. Eighty-five original source layouts and two
identical complete HTTP observations reproduce raw and binary identities.
All source layouts are measured without instantiating their large values.

Twenty selected tests cover arithmetic and shape boundaries, inactive/zero
graphs, shared deep paths, heap/inline/copy/target accounting, namespace sharing
and HTTP syntax fit. A test observation initially named an Int layout absent
from its empty past registry; registering that test type before capture fixed
the invalid fixture. The [verification record](../migrations/go-practical/t01-w04/structural-limits/verification/verification.json)
retains selected native/model/replay/metadata/format results and scope/reasons.
Unchanged production/kernel and older proof/native matrices are not rerun.
The cumulative T01 gate remains deferred to W05.

W04 remains In progress. Full source/counter/lifetime/normalization/finite-ID
and allocator proofs, exact shared/Go schema/hash identities, independent import,
reviewed migrations and production resource supervision remain required. W05
owns inactive publication and the T01 gate; no T is complete. All T02–T11
source/application/release work remains part of the active objective.

## 39. T01-W04 candidate identities and acyclic source-hash DAG

Entry: `47b2a5e9846b5200ca1106ab9407de310af6e169`, clean before this work.
The preceding checkpoint committed numeric/carrier candidates and the measured
HTTP syntax-ceiling correction. This checkpoint advances identity/preimage
and migration preparation while retaining the full eleven-stage objective.

The [identity catalog](../migrations/go-practical/t01-w04/identity-dag/README.md)
proposes101 globally unique identities and41 domains with no collision against
the frozen inactive C# package's102 successor identities/42 domains. Shared
registry/context/VIR/frontend/map/manifest/release names usev3; VC/policy/API/AI
usev4; Go-specific source/foundation names usev1. Compiled profile contracts
usev2 because inactive C# already reserves theirv1 names. Four old semantic
profiles plus Go practical form the proposed future installed family; C#
practical stays inactive. Generic retained domains require exact unchanged
preimages/meaning review. No parser route, profile or normative package activates.

Ten source-identity schemas and five exact source-type arms specify all required
fields and complete self-hash projections. ASCII domain+NUL+narrowed RFC8785
JSON without LF determines lowercase SHA256; only own id is excluded. Every
active arm field is included. Raw source/overlay objects precede module,
package, declaration, nominal/structural type, signature/error-U and capture/
loop references. Capture excludes normalized contract digests. Nominal types
use type-declaration locators before method-set/receiver binding, avoiding a
hash cycle. Aliases retain the target identity rather than minting a new type.
Go int versus int64, unexported-field package identity and arbitrary tag bytes
stay explicit; source semantics are not inferred from equal representations.

A strict26-node/seven-root private graph checks complete typed hashes,
reference closure, capture/module/package membership, logical path and ASCII
case uniqueness, physical UTF-8 spans, receiver/signature and same-context loop
binding. Opaque external context/inventory hashes and supplied roots remain
fixtures. Raw capture, registry validity, compiler/type/underlying definitions,
global-U derivation, loop ordinals and execution correctness are unproved.
The identity layer cannot replace independent source/import/body proofs.

Forty-four canonical/hash vectors cover all26 node preimages plus UTF16 key
ordering, controls/line separators, no Unicode normalization, full64 decimal
strings, safe integers, negative zero and surrogate pairing. Thirty-four
canonical values/digests agree between Python, unchanged Rust canonical_json
and independent Go code. Ten malformed JSON/domain cases reject in all three.
These are hash-observer outcomes, not kernel/checker verdicts or counterexamples.
Both native binaries reproduce exact outputs twice after fresh builds.
The private Rust crate imports the canonicalizer source without editing it;
only its new main is formatted with skip_children=true. No production/core,
installed registry, Certificate v0 or older helper snapshot changes.

A143-path proposal assigns candidate migration actions and planned work/test
owners for inactive C# retention, additive checker preservation, old-profile
producer wrappers, API/policy consumers and atomic whole-image installation/
rollback. Every row still requires exact symbol/preimage/behavior review;
additional affected discovery and complete four-profile source/certificate
equivalence remain required. Candidate routing is not a completed migration.

Nineteen selected tests pass, including stale/extra/foreign schemas, complete
preimages, paths/raw source hashes, alias/nominal distinction, methods/results,
error/loop/context crossings, dangling/duplicate/unreachable graph nodes and
namespace/migration scope. The [verification record](../migrations/go-practical/t01-w04/identity-dag/verification/verification.json)
retains selected model/native/hash replay, metadata/predecessor/diagnostic
integrity, compilation and targeted formatting with reasons. Unaffected
production/kernel and prior native/proof matrices are not rerun. The full T01
gate remains deferred to W05.

W04 remains In progress: complete shared/Go root/nested/operation/expression
unions, foundation/context/profile bindings, independent import/source proofs,
all producer/consumer review and migration/activation/rollback gates remain.
W05 owns inactive publication and the T01 gate. No T is complete; T02–T11
implementation, full application and local release scope remains active.

## 40. T01-W04 closed registry/context/request draft and retained policy

Entry: `4cc82980a6502fc0bb13813d6986307887613316`, clean before this work.
The preceding checkpoint proposed identities and a source-hash DAG. This
checkpoint closes six private root shapes without completing the full shared
schema/foundation/source freeze or changing installed artifacts.

The [context protocol draft](../migrations/go-practical/t01-w04/context-protocol/README.md)
defines exact registryv3, entryv3, contextv3, validated requestv3, practical Go
parametersv1 and Go selectionv1 fields. Complete canonical typed preimages,
registered entry/schema/language/profile bindings and nine compiled profile
contract names are checked. The entire registry must equal the private
compiled registration fixture after self-hash checks; rehashing a changed
contract/profile/foundation/order cannot create authority. This family avoids
the reserved inactive C# identities and does not change their meanings.

Each context has a required foundation field: null for all four retained
profiles, the exact compiled Go reference for practical. The private Go
foundation description explicitly lacks a complete definition bundle. Its
hash and the64-counter/weight limits fixture hash are private data, not final
published foundation/limits identities or proof evidence. Complete schemas,
registered ordinary definitions and production import remain required.
Fixed Go parameters include all target/compiler/effective-language/limits
fields and allow no numeric or feature overrides. A typed repeated context
must match every field; two valid Rust32/64 contexts cannot link. Request
hashes cover the whole context and selection rather than a lossy projection.

A new standalone Rust wrapper calls the actual unchanged revision3 compiled
registry/context/selection validators, including their real Java name/path
rules. It preserves both Rust checked targets and scalar Go/C#/Java settings.
All linked mpk-vc/mpk-cert/mpk-core sources, private Cargo lock and wrapper
binary are pinned. The initial wrapper omitted the registry transport LF and
returned RegistryCanonical; using the existing canonical_registry_transport
function fixes that construction error. No production validator changes.
The31 distinct native-policy projections reproduce exact results twice.
This is configuration-policy evidence rather than full semantic preservation
or a completed four-profile producer/consumer migration.

Raw Go roots contain package/function and a nullable local value-receiver
selector, in unique canonical JSON order. The1–8192 bound, Unicode15 names,
keyword/blank exclusions and package-versus-method init distinction are
explicit. Raw receiver aliases can still resolve to the same method. Actual
declaration/package/method existence, alias-resolved duplication, source
closure and capture identity are later source predicates; no application
source is executed or admitted by the private protocol probe.

All152 vectors match expected outcomes:24 configuration candidates and128
configuration rejections. Candidates are six target contexts, eleven
requests, six identical links and one compiled registry. Negatives cover
profile/language/schema/foundation/limits crossings, reserved/old/future
families, rehashed registry attacks, omitted/extra/nested fields, stale or
incomplete request hashes, raw selector order and strict malformed JSON.
None is a checker verdict, certified false claim or source semantic result.

Twenty targeted tests pass, including actual old policy results, Rust32/64,
8192/8193 roots, full typed context linkage and absent-oracle fatal errors.
Fresh replay rebuilds the native wrapper and preserves all source/binary
identities. The [verification record](../migrations/go-practical/t01-w04/context-protocol/verification/verification.json)
retains selected model/native/replay/metadata/format checks, raw logs and
reasons. Prior receipts/diagnostics and143 predecessor paths remain unchanged;
unaffected production/kernel and old proof/native matrices are not rerun.
The cumulative T01 gate remains deferred to W05.

W04 remains In progress. Complete Go foundation/limits, full source/type/
binding/call/contract and downstream shared root/union schemas, independent
import/source proofs, reviewed migration/semantic preservation and activation/
rollback gates remain required. W05 owns inactive publication and the T01
gate. No T is complete; T02–T11 implementation/application/release remain active.

## 41. T01-W04 exact limits and foundation packaging references

Entry: `15f6833f2931a3a39fa8cad450f1c48a2f813b60`, clean before this work.
The preceding checkpoint closed context/request roots with deliberately
incomplete private foundation/limits fixtures. This checkpoint adds exact
packaging records while keeping full ordinary/source foundation completion
and the eleven-stage objective required.

The [packaging draft](../migrations/go-practical/t01-w04/foundation-limits/README.md)
closes three root shapes: limits, definition recipe inventory and descriptor.
Every required field enters its domain-separated typed hash, except its own
content_sha256. The entire record must match compiled values after hashing;
rehashing caller limits, weights, core interfaces, operations or members
does not authorize a new candidate. Unknown/missing/extra fields reject.
Old normative schemas, active registry, reserved C# meanings and Certificate
v0 remain unchanged.

All64 preceding independent numeric candidates have explicit maximum, unit,
category, counting rule and exceeded outcome. The4,194,304 practical syntax
candidate remains separate from predecessor1,000,000. Units distinguish
byte counts, full word bits, depth, nanoseconds, abstract carrier units and
the exact counted object. Domain/unproved-resource outcomes cannot become
Go panic, semantic falsity or success. Full current/past weights retain all
bytes, fields, inactive global-U payloads, copied values and observations.
The source counter, finite-id, allocator and production supervisor proofs
remain required rather than inferred from serialized numbers.

The definitions record is explicitly operation_recipe_inventory_draft,
containing all56 logical operations in eight component families and six
common source/validity/evaluation/safety/value-effect/frame members. It pins
the exact operation table and six historical evidence receipts. Those proofs
retain their own scopes and do not prove missing concrete specialization or
source body/contract obligations. Empty source-callable and caller-extension
lists cannot register an executable application adapter.

The joint core proposal binds exact Sort0 Nat.induct and Bool.recType types,
iota equations, canonical generated family/constructor checks, monomorphism
including prior opaque/theorem uses and later-constructor rejection. The four
existing Rust/Go implementations and77-file mirrored closure match retained
private evidence. Production core integration remains T02 work. Complete
ordinary concrete definitions and specialization/expansion/binding schemas
are not replaced by this operation recipe index.

Descriptor members are exactly the two canonical limits/definitions files
and the semantic-rules document, with raw SHA256 and byte length including LF.
Missing/extra/stale/altered/escaping members reject; filesystem symlinks reject
before reading. The descriptor excludes its own bytes, contexts, instances
and executable/release outputs. Limits precede definitions, both precede
descriptor and registry/context. Later release binding cannot introduce a
self-hash cycle.

Freshly linked private registry/contexts/requests use the new exact limits
and descriptor references. The earlier private generation remains unchanged;
mixing its registry/foundation/limits rejects. This draft assembly is not
production migration or normative publication. All four retained profiles
keep null foundations and actual compiled policies, including Rust32/64.
The102 reconstructible vectors have16 configuration candidates and86
configuration rejections. They are not checker/source verdicts.

Twenty-one complete canonical preimages cover the three roots, five entries,
registry, six contexts and six requests. Python, unchanged Rust canonicalizer
and independent Go observer agree on every byte/digest, with two identical
native runs each. Fresh replay verifies pinned Go1.25.0 and binary/source
identities. Twenty targeted tests cover compiled authority, counter/unit/
outcome/weight integrity, joint core/source references, all logical operations,
raw member sets/LF/symlink guards and private-generation crossings.
The [verification record](../migrations/go-practical/t01-w04/foundation-limits/verification/verification.json)
retains selected tests, native replay, metadata, Python compilation, complete
raw logs and reasons. Earlier receipts/diagnostics and143 predecessor files
remain unchanged. Unaffected production/kernel and old proof/native matrices
are not rerun. The cumulative T01 gate remains deferred to W05.

W04 remains In progress. Complete ordinary concrete carrier/operation/
contract definitions, specialization/expansion/binding/type/downstream
schemas and source predicates, reviewed producer/consumer migration and
four-profile semantic preservation/activation/rollback remain required.
W05 owns inactive publication and the cumulative T01 gate. No T is complete;
the complete T02–T11 application/implementation/release objective stays active.

## 42. T01-W04 concrete carrier specialization and ordinary expansion

Entry: `4c6a8151b77c1640e7aaaf8c163bfd4dc914bbcc`, clean before this work.
The preceding checkpoint bound limits/recipes/descriptor packaging but did
not emit concrete ordinary carrier definitions. The
[closed expansion](../migrations/go-practical/t01-w04/closed-carriers/README.md)
now emits complete closed carrier/zero members and ordinary laws, preserving
the full source/operation/validity and eleven-stage objective.

Three root shapes cover type definitions, closed instances and expansion,
with exact source/joint instance arms and six ordinary term arms. Two new
candidate identities/domains avoid every prior Go and reserved C# name.
Complete typed context, type-definition hash, global error U, source identity,
ordered dependencies/results and derived shape/accounting fields enter
instance hashes. Full reconstruction rejects extra/missing instances,
unknown/axiom/free-variable forms and rehashed caller equations. Current
source bodies, normative schemas, Certificate v0 and installed routes remain
unchanged; no production core rule changes.

Thirty-three retained original-Go type facts plus six anonymous boundary
shapes give39 source/fixture types. Original raw source, literal fixture
go.mod, source/module/package/declaration/type/signature/capture nodes and
Error method signatures form a checked identity DAG. Literal declaration
locators, ordinal choices, fixture module/capture and supplied roots are
private inputs rather than independent source/root/alias/underlying proof.
Aliases preserve target ids; Amount and OtherAmount keep distinct instances
even though their core carriers are equal. Kernel conversion cannot prove
Go nominal assignment or authorize a source conversion.

Full expanded graph inspection includes zero-array and inactive error
edges, all field/result order and source-sized words. Under U containing
RejectCode and SliceError, error is not Plain and []error backing rejects.
The main set has42 instances. The second explicit U contains only RejectCode
and admits []error, giving43. Source paths/entry contracts establishing these
complete universes remain unproved. Crossed contexts/U/instances/expansions
reject instead of treating one predeclared source error id as uniform data.

Carriers use all8/16/32/64 bits, complete256-byte string storage, actual array
length versus finite padding, ordered heterogeneous structs/results, complete
closed-error tags/payloads and typed slice descriptors. Empty products/arrays
have a canonical-false Bool dummy; arbitrary input validity is a separate
obligation. Slice paths retain NoStep/field/index kind and full index values.
Owner labels remain Nat ghosts and establish no finite-id bound, source
retirement/escape proof, NoAccess live witness or current/past namespace
translation. Those pending requirements are not replaced by a guessed serial
width or by carrier typing.

The main expansion has90 new ordinary definitions and46 new ordinary
theorems:42 explicit zero-recipe equalities plus four construction/projection
laws over arbitrary heterogeneous values. The Plain-error set adds one
instance/zero law. Both independently check in the unchanged joint private
Rust/Go proposal with zero axioms and matching canonical/export/axiom hashes.
Their certificates have171/174 total declarations and25,904/26,336 bytes.
There is no payload enumeration, extensionality axiom, new opcode or source
generic. The complete77-file private TCB proposal and seed remain unchanged.
Only a new untrusted ordinary-term emitter is compiled/formatted.

Six altered ordinary controls reject in both kernels with CoreTypeMismatch:
one wrongly typed word zero, three noncanonical empty zero laws and two false
universal projection claims forcing a result to zero. A well-typed zero
definition alone is insufficient; the canonical zero law detects the altered
Bool dummy. The report keeps malformed definitions, false ordinary claims,
source/configuration errors and resource failures distinct. Two legacy
Bool/Nat certificates reproduce in the same unchanged private checkers.
All eight inputs are emitted twice with exact byte equality. Fresh replay
rebuilds both binaries and reproduces every canonical certificate.

The measured direct children and proof byte/term/declaration counts fit the
preceding candidate envelope on Darwin; this is not Linux release evidence
or an aggregate/application guarantee. Twenty selected tests cover full
closure, aliases/nominal identities, all integer widths, array/struct bounds,
bytes, global-U Plain, hidden cycles, result order/arity, context/schema/AST
crossings, determinism and explicit proof-scope flags. The
[verification record](../migrations/go-practical/t01-w04/closed-carriers/verification/verification.json)
retains raw selected test/replay/metadata/format/compile logs and reasons.
Earlier receipts, two diagnostics and143 predecessor paths remain unchanged;
unaffected old proof/native/kernel suites are not rerun. The cumulative T01
gate stays deferred to W05.

W04 remains In progress. Full independent source/capture/root/alias/nominal/
underlying/U completeness, validity and finite-id normalization proofs, all
operation/contract/control/source-binding laws, downstream artifacts and
reviewed producer/consumer migration/preservation remain required. W05 owns
inactive publication and the T01 gate. No T is complete; all T02–T11
implementation/application/release scope remains active.

## 43. T01-W04 closed source scope and normalized contract wire

Entry: `87d3d4e000339ff155cf72c9e7c8e8285efcc7c0`, clean before this work.
The preceding concrete carrier checkpoint left complete source/contract wire
shapes open. The [contract wire draft](../migrations/go-practical/t01-w04/contract-wire/README.md)
now closes three roots, 70 source heads and 70 normalized heads, preserving
the complete eleven-stage objective and earlier private/core evidence.

Two new scope/binding schema identities and hash domains avoid all prior Go
and reserved C# names. Source-overlay and contract reuse their proposed v1
names. Exact required roots/payloads, complete typed contexts and whole-value
preimages reject unknown/inactive fields and rehashed caller substitutions.
No producer proof-status flag is trusted. Raw overlay bytes, including their
original whitespace/escapes, enter capture before binding/loop identities;
normalized hashes stay outside capture. Duplicate JSON names reject. Physical
loop selectors precede derived capture-dependent loop refs, avoiding a cycle.

The private model uses one original Go source, one overlay, literal fixture
module metadata, 33 native type facts and six synthetic boundary shapes.
Fixture declaration/line/token locators and supplied roots/error U remain
private inputs. Their structural consistency does not prove independent
source/capture/alias/underlying/binding/U completeness. Full immutable project
capture and general independent production import remain T02 work.

Twelve-field bindings retain capture/source/function/type and physical span.
Inputs are signature-ordered, receiver first; joint results retain order.
Header membership uses exact lexical IDs, preserving shadowing. Aliases share
their target type ID while distinct nominal types remain distinct. Field and
error paths retain exact owner/ordinal/case IDs. Entry error paths include all
capacity tails; empty cases mean nil-only and confer no ownership. Lexical
identity supplies no runtime invocation or post-state proof by itself.

Normalized input/result/header/range/heap references retain entry/header/return
namespaces. Whole-observation old differs from an old descriptor observed in
the current state. Entry cannot read result/header/range locals. Finite
quantifiers and sums use typed de Bruijn depths with outer-scope bounds and
a body-only Nat binder. Shape decoding reconstructs and independently retypes
fixture expressions. Scope reconstruction detects mutated authority objects.
Neither reference-model validation nor equal core carriers proves Go source
semantics, nominal assignment, definedness, permission or termination.

All 70 arms retain fixed arity and complete source/logical types, full word
literals and shift counts. Per-function 1024-node/depth 32 and clause/loop
limits have boundary controls. The catalog lists 13 partial-operation proof
member kinds without claiming their implementation. The older private digit
resource guard is not a semantic Nat bound. Truncated Nat subtraction, finite
intervals, guarded definedness and snapshot-sensitive observations retain
their existing meaning. Complete ordinary proofs belong to T02–T06.

Twenty-four source vectors retain 5 normalized unproved candidates, 1 pending
variant and 18 configuration rejections. Three total for-loop variants remain
missing in the pending case; generated invariants/ranks are candidates only.
Seventy typed component queries cover every source/normalized head; they are
not all literal members of one captured overlay. Six complete normalized
contracts retain raw overlay bytes and full scopes for reconstruction.

Ninety-seven complete canonical preimages cover six scopes, six contracts,
fifteen bindings and 70 components. Component hashes use the existing private
pilot domain and create no public expression identity. Fresh unchanged Rust
and Go hash observers agree with Python on canonical bytes/digests, with two
identical native runs each. The receipt pins 577 source/data inputs and the
same 14,496-file Go 1.25.0 inventory. Hash agreement is not source or checker
acceptance evidence.

Twenty selected tests cover source raw bytes, aliases, shadowed/named bindings,
old/current namespaces, nested binders, nominal/full-width types, capacity
paths, limits and rehashed/mutated records. The
[verification record](../migrations/go-practical/t01-w04/contract-wire/verification/verification.json)
retains selected unit/replay/metadata/compile logs and selection reasons.
All earlier receipts/diagnostics and143 predecessor paths remain unchanged.
No Rust/Go source changed, so earlier format/proof/kernel suites are not
rerun. The cumulative T01 gate remains deferred to W05.

W04 remains In progress. Complete shared VIR/map/manifest/VC/policy/API/release
roots/unions and producer/consumer migration review remain required. W05 owns
inactive publication and the T01 full gate. Source/ordinary operation/body
proof implementation and application/release work remain T02–T11; no T is
complete and the Go practical profile is inactive.

## 44. T01-W04 concrete operation signatures and complete check members

Entry: `83674c5fe85443618e70e044a8e54461583401eb`, clean before this work.
The preceding contract wire checkpoint left shared VIR operations and their
exact interfaces open. The [operation-signature draft](../migrations/go-practical/t01-w04/operation-signatures/README.md)
now derives all 56 families' complete concrete input/output/region interfaces,
retaining the full eleven-stage objective. These are typed unproved operation
definitions, not actual VIR or source/application certificates.
Every selected input/result must satisfy Root under the same U. One-case U
admits the fixture's []error backing; two-case U rejects that selected scope
and instead uses a separately valid Quote scope to test finite dispatch.

One new candidate operation-definition identity/domain avoids every earlier
Go and reserved C# name. The complete typed context, capture/scope/function/
signature/type/U, original table hash, operation/parameters, ordered interfaces,
rule members and binding requirements enter its hash. Closed source/location/
joint/ordered-values/state/control arms have no free type parameter. Source
nominal identity, full word values and assignment-versus-explicit-conversion
rules remain separate from equal ordinary carriers.

The private fixture retains original native types/headers, supplied roots/U,
literal module/capture and synthetic boundary shapes. The 56 definitions are
component queries, not 56 parsed operations in one source function. An unchanged
original source or matching signature proves no source/closure/SSA/region,
same-invocation, allocator/finite-id or ordinary semantic law. Actual producers
and independent consumers retain their existing planned T02–T06 owners.

Short-circuit RHS has a guarded state region. Switch groups preserve every
case-expression order/body and separate default position. For init/post return
state; body returns control; missing test has no execution region. Array-range
constant-length mode requires no second value; slice range always captures
its descriptor. Actual constant-expression/source premises remain unproved.
Break/continue select different innermost targets through switch contexts.
Supplied switch spans are not independent AST facts; complete lexical source
reconstruction is still required.

Static calls retain exact declaration/signature, receiver-first parameters,
joint results and post-state shapes. Projection must still prove actual
same-invocation provenance. Function results retain arity 0..16, while ordinary
parallel assignments use a separate ordered sequence and admit 32 values.
Blank stores preserve RHS evaluation. Error boxing invokes no body; finite
invocation requirements include every same-U target and nil/footprint checks.
Prefix/suffix requirements pin exact Go 1.25.0 symbols, compiler inventory and
both original source files; no source/value proof or trusted adapter is inferred.

Every family retains all six common members and its runtime/domain/permission
members. Compound assignment inherits its chosen arithmetic/divide/shift and
store checks, rather than losing implicit divisor/count/write obligations.
Unsigned full-width shifts have no invented count ceiling, and signed min/-1
division has no invented overflow panic. NoAccess and positive copy/append
writes retain their permissions. Member strings are the closed requirement
inventory; ordinary predicate/equation terms and proofs remain T02–T06 work.

Existing complete-source syntax bounds guard large parameter lists without
new case/assignment/append semantic limits. A 129-value append can form a typed
candidate while its no-growth domain remains unprovable. Literal/profile
domain and Go panic safety stay distinct. Actual structural/resource counting
and enforcement remain required in production.

The 224 vectors have 56 typed unproved and 168 configuration-rejected outcomes:
every family has an intact definition plus rehashed extra-parameter, altered-
signature and missing-member controls. Twenty-four targeted tests add nominal,
constant/byte, compound, short-circuit/switch, product/assignment, callee/error,
adapter, Plain/permission, range/abrupt, domain and mutable-authority checks.
All 56 canonical preimages agree in Python and fresh unchanged Rust/Go native
observers, each running twice identically. The receipt pins 587 inputs and the
preceding contract receipt, with the same 14,496-file Go 1.25.0 inventory.

The [verification record](../migrations/go-practical/t01-w04/operation-signatures/verification/verification.json)
retains raw selected unit/replay/metadata/compile logs and selection reasons.
The direct preceding contract consumer is reconstructed; older production,
kernel and ordinary/native matrices are not unnecessarily rerun. All 143
inventoried predecessor paths remain byte-identical. No Rust/Go source changed.
The cumulative T01 gate remains deferred to W05.

W04 remains In progress. Complete shared VIR/VC/map/manifest/policy/API/release
roots/unions, actual source/SSA/ordinary-definition linkage and reviewed
producer/consumer migration remain due. W05 owns inactive publication and the
T01 gate. No T is complete; all T02–T11 work remains active and no practical
Go profile is activated.

## 45. T01-W04 typed function bodies and structured SSA/state/control

Entry: `2c4c557129cc9f3998dc505f02a20edf2a6e3882`, clean before this work.
The preceding 56-operation checkpoint supplied signatures, not a body graph.
The [typed body proposal](../migrations/go-practical/t01-w04/vir-body/README.md)
now closes a Go-owned function-body root, seven nodes, region interfaces and
three terminators, retaining the complete eleven-stage objective. The shared
five-profile VIR wrapper and full source/ordinary bindings remain required.

One new candidate body identity/domain avoids every prior Go and reserved
C# name. Whole typed context, capture/scope/function/signature/contract,
complete concrete definitions, entry arguments, body and derived obligation
sites enter the hash. Canonical value/node/region preorders enforce dominance,
scope and exact explicit captures. Sibling/future values, stale states,
unused/foreign definitions and rehashed altered sites reject. Source authority
is rebuilt from raw buffers instead of mutable producer maps.

Calls/finite error dispatch produce joint values and post-state together.
Projection retains the same static producer and exact post-state; an ordinary
pack or same-typed second call cannot substitute. Historical projection is
read-only and cannot roll back current state. Actual dynamic activation/
invocation correspondence remains unproved. Dependency sites require exact
callee source-body relations and an overlay relation where present; no absent
contract becomes a trusted callee theorem.

Control values bar further instructions until propagated or resumed. Normal
continuations run only after normal exits; abrupt exits propagate. Structured
switches/loops consume only their exact break/continue target. Init/test/body/
post interfaces stay ordered and missing-test/no-break loops have no normal
exit. Static outcome sets do not prove source scheduling or termination.

Internal addresses preserve local-root handles versus frozen slice descriptor
roots and exact typed field/array paths. Read/write checks are attached to the
access state, after target/RHS evaluation where required. Full bounds,
initialization, validity, permission and frame requirements remain explicit;
address formation grants no NoAccess write or source pointer capability.
Explicit declarations and lexical scopes now enforce availability, with
for-init locals shared only by that loop's test/body/post. Ordered multi-store
access sites retain exact target-prefix state references, including blanks.
Source block/activation correspondence, supplied switch spans and source
anchors still require complete declaration/lifetime/AST proofs.

Every explicit SSA definition counts toward 65536 locals; all regions count
toward 8192 function blocks; parameters/captures count toward 4096 block
parameters. All nodes count toward 100000 block/function instructions.
Separate assignment-value sequences admit 32 values without changing result
arity 16. Complete ordinary expansion/runtime accounting for these internal
values, finite source identities and module-wide counting remain required.

A new shared-limit Python canonicalizer and two untrusted native wrappers
retain 192MiB canonical/256MiB input, 1MiB decoded string and 256-container limits.
They preserve the earlier narrow integer/Unicode/domain rules; the old 64-depth
pilot and production canonicalizer bytes stay unchanged. Rust imports that
unchanged canonicalizer. No kernel/Certificate v0 rule changes.

Fifty-six typed component queries cover every operation family. Three full
function candidates cover straight return, loop/abrupt flow and a no-node
candidate for the original discard-only InputErrors function. Blank-assignment
erasure and source equivalence are unproved; the first two candidates are not
claimed to match the source body. Twenty graph vectors classify 3 typed
unproved/17 rejected. Twenty-one selected tests add invocation/pack/error,
capture/dominance/state/location, 32-value,divergence and JSON boundary checks.

Sixty-nine preimages cover three bodies, 56 private pilot component hashes and
ten canonical boundaries. Python and fresh Rust/Go observers agree on 64
accepted and five rejected inputs, with two identical native runs each.
The receipt pins 604 source/data inputs and the same Go 1.25.0 inventory.
Hash agreement is not source or ordinary proof acceptance.

The [verification record](../migrations/go-practical/t01-w04/vir-body/verification/verification.json)
retains selected unit/replay/metadata/new-source format/compile logs and
selection reasons. The preceding operation corpus is reconstructed; unchanged
production/kernel/older ordinary/native matrices are not repeated. All 143
inventoried predecessor paths remain byte-identical. The cumulative T01 gate
stays deferred to W05.

W04 remains In progress. Complete shared VIR and four-profile preservation,
source-map/manifests/VC/policy/API/release roots/unions and reviewed migrations
remain due. Source/dynamic/ordinary operation, permission, frame and totality
implementation remain T02–T06. W05 owns inactive publication and the T01 gate;
all T02–T11 work stays active and no practical Go profile is activated.

## 46. T01-W04 shared VIR/contract and whole Go module closure

Entry: `2cb486e6731d182a7ef62abe82eb72ccedb55dd2`, clean before this work.
The [shared module draft](../migrations/go-practical/t01-w04/shared-vir/README.md)
closes the exact five-profile common VIR v3 and contract v3 envelopes, preserving
the full eleven-stage objective. Full context/registry/entry/parameter/foundation
validation precedes compiled profile-contract and tagged payload dispatch.
Old/future roots, caller fields, profile/context/hash crossings and stale common
callee contract hashes reject before the private observation conversion.

Five new Go candidate identities/domains add source-scope/set v1 and normalized
contract/operation-definition/body v2 without changing earlier private roots or
reserved inactive C# identities. A required nullable source-overlay ID and exact
origin distinguish actual sidecars from derived default contracts. Every actual
raw overlay joins one immutable capture; no absent overlay becomes a fabricated
raw object. All dependent bindings/loops/types/operations/contracts/bodies are
recomputed. Raw whitespace changes propagate while declarations/signatures stay
stable. Alias-resolved duplicate roots/overlays reject.

The Go arm contains exact declaration-ordered scope/contract/body sets. The
consumer rebuilds authority from original buffers, derives every operation/body
and follows dependencies from all selected roots. Missing callees, cycles and
unreachable extras reject. Nine nodes and three regions across each candidate's
three functions count against the existing whole-module caps. Unannotated
callees retain unproved partial/true/all-same-U contracts and require their actual
source-body relation. No default grants a trusted callee theorem.

The source model remains the preceding pinned single Go fixture package with
multiple actual overlays, not a general independent project importer. Quote and
Error bodies remain typed candidates without source equivalence; InputErrors
keeps the preceding discard-only candidate with erasure proof due. Full source,
dynamic activation and ordinary operation/permission/frame/totality proofs remain
required in T02–T06.

A private owned source mirror preserves all236 Rust sources and the original
dependency versions/checksums. One appended observation bridge calls the actual
unchanged retained importer, projection, type validator and profile-specific WP
engine. Five independent old-family baselines cover Go fixed, Rust32/64, C# and
Java; Go/Rust64 use the original fixture roots unchanged. Complete typed
projections and serialized VC data match the common representations exactly.
This is fixture evidence, not a universal preservation theorem or discharged
proof. Thirteen actual native queries add eight malformed-instruction/type
negatives across all four profiles. An unobserved query is an infrastructure
failure, never an assumed rejection.

Seven modules and73 vectors yield five native-accepted, two Go typed-unproved
and66 configuration-rejected outcomes. Twenty-two targeted tests exercise
dispatch, origin/capture authority, deep expressions, exact closure and counters.
All62 preimages agree across Python/Rust/Go: seven modules,17 common contracts,
two scope sets,six scopes,six normalized contracts,18 operations and six bodies.
Each native observer runs twice identically. The receipt pins625 inputs and the
same14,496-file Go1.25.0 inventory. Hash/VC parity supplies no source/ordinary
proof verdict.

The [verification record](../migrations/go-practical/t01-w04/shared-vir/verification/verification.json)
retains selected local unit/replay/metadata/new-source format/compile logs,
selection reasons, repairs and review. Prior body evidence and all143
inventoried predecessor paths remain unchanged. Unaffected older proof/native
matrices are not repeated. The cumulative T01 gate stays deferred to W05.

W04 remains In progress. Source-map/manifest/VC/policy/API/release roots/unions,
complete source/foundation bindings and reviewed producer/consumer migrations
remain due. W05 owns inactive publication and the T01 gate. No T is complete,
all T02–T11 work stays active and no practical Go profile is activated.

## 47. T01-W04 common source maps, frontend manifests and Go input inventories

Entry: `84d66684822644ada2c3a9bf6c2479b350f17dbd`, clean before this work.
The [capture-artifact draft](../migrations/go-practical/t01-w04/capture-artifacts/README.md)
closes common source-map v3 and frontend-manifest v3 envelopes plus a new Go
input-inventory v1 identity/domain. Every root field except its own hash enters
the canonical preimage. Exact full context, compiled profile contract and
tagged arm bind the validated common VIR and map references. Old/future/
certificate roots, caller/proof fields and context/hash/reference crossings
reject. Earlier private/production meanings and reserved inactive C# names
remain unchanged; the complete eleven-stage objective is retained.

Go maps cover every function, all seven body node arms and every terminator,
including scope children and resume continuations. Declaration and canonical
numeric IDs determine order. Functions use declaration spans; nodes use body
anchors; terminators inherit enclosing anchors. Nonempty UTF8 byte boundaries
must lie in captured Go source objects. These locators leave actual AST/
lowering/execution correspondence to its required proof. Function+node+
terminator accounting retains323,728 entries/32MiB; frontend manifests retain
4MiB,32,768 inputs and256 units, alongside shared transport/depth/string bounds.

The inventory binds every original capture object with its unchanged source-
object record and actual size/SHA. ASCII-case collisions within a capability,
missing/fabricated inputs and capture/contract count/byte overruns reject.
Default contracts add no raw files. Exact selection,capture,limit,build,
module/package and independent external provenance references enter the Go
manifest. Source authority is copied from original buffers; successful module
checks are cached by complete canonical VIR bytes rather than a caller digest.
Rehashed nested inventory/object/capture and retained input-set mutations still
reject against the independent byte/model authority.

The new Rust observer calls unchanged public successor VIR/map/frontend-
manifest importers directly, with226 unchanged linked production source files
pinned and no appended facade. Original dependency versions/checksums and the
nonincremental/no-debug/single-codegen-unit configuration are retained. Sixty
native queries preserve original profile-specific range/input/selection/target
and synthetic-permission policies. The extra Go synthetic case requires exact
independent permission; Java still rejects such permission. Both scalar profiles
reject UTF8 splits. Five independent old-family baselines preserve complete
validated map/manifest values, including raw original Go/Rust64 roots.

Existing literal empty go.sum and generated Rust Cargo bytes reproduce their
original manifest size/SHA pins. C#/Java boolean source/contract fixtures are
explicit. The successor external release reference is checked before deliberate
old-family observation conversion. Its private fixture remains incomplete and
uninstalled. Historical Go tool/frontend identity samples supply structure and
linkage only; actual registered practical producer/toolchain/bundle execution
provenance remains required for release. No source or release verdict follows.

Eight artifact triples and187 cases yield eight linked-unproved maps,eight
linked-unproved frontend manifests and171 configuration rejections. Twenty-four
targeted tests cover nested if/scope/resume locators,full reference/input sets,
immutable/cache authority,synthetic boundaries,rehashed metadata and budgets.
Nineteen preimages agree across Python/Rust/Go: eight maps,eight frontend
manifests,two inventories and one private external-reference fixture. Each
native observer runs twice identically. The receipt pins652 inputs and the
same14,496-file Go1.25.0 inventory.

The [verification record](../migrations/go-practical/t01-w04/capture-artifacts/verification/verification.json)
retains selected raw unit/replay/metadata/new-Rust-format/Python-compile logs and
selection reasons. The initial observer omitted its required hash placeholder;
raw queries/results retain that setup failure and the repaired classification
is rechecked. Prior module evidence and all143 predecessor paths stay unchanged.
Unaffected older proof/native matrices are not repeated. The cumulative T01
gate stays deferred to W05.

W04 remains In progress. Certificate-manifest/VC, frontend-envelope, policy/API/
release roots, complete foundation/source signatures and reviewed migrations
remain due. General independent capture/import and all source/AST/control/
lifetime/activation/ordinary operation/permission/frame/totality proofs stay
required in T02–T06. W05 owns inactive publication and the T01 gate; all T02–T11
work remains active and no practical Go profile is activated.

## 48. T01-W04 common VC/certificate-source bindings and Go theorem demands

Entry: `bdd7099a5845b83452149155f695effbe582e1f2`, clean before this work.
The [proof-artifact draft](../migrations/go-practical/t01-w04/proof-artifacts/README.md)
adds common VC v4/certificate-source-manifest v3 roots and retained skeleton v4
regeneration. Two new Go identities/domains define closed theorem-demand
programs and requests,avoiding earlier Go and reserved inactive C# names.
Full context,compiled profile contracts,exact source references and all
function/request data enter canonical hashes. Old/future/crossed/rehashed
members reject against complete source regeneration. The full eleven-stage
objective and every earlier helper/production/core/Certificate v0 meaning stay
unchanged. Complete Go ordinary expansion/final skeleton closure remains due.

Actual unchanged public source validators,VC generate/import,skeleton emit/
import and certificate-stage manifest import preserve full old generated
values in five target contexts. Forty-four native queries cover intact
programs,changed members/groups/conclusions/statement bodies and wrong
required-check profiles. The additional authorized synthetic Go source works.
The observer pins226 unchanged linked production sources and original
dependency versions/checksums; no appended facade or copied semantic validator
is used. Native data and returned skeleton declarations are independently
cloned,preventing producer mutation from changing expected source authority.
The native query itself must match the complete validated common source tuple.

The unchanged canonicalizer rejects configured nesting beyond768. The initial
1024-depth probe failure is retained with raw queries/results. Common VC and
skeleton roots therefore keep retained function/declaration arrays directly at
root,spending no extra container over old terms. New iterative Python
parse/canonical/clone and independent Go hashing retain256MiB,768 containers,
1MiB strings,safe integers,scalar Unicode,UTF16 key order and domain-NUL hashes.
Older64/256-depth helpers remain unchanged. Eleven boundaries retain768/769,
empty containers,duplicates,unsafe integers and Unicode failures separately.

Go has19 closed demand families. Every request binds complete context,VIR,
scope/body/contract,exact requirement/origin and callee request dependencies.
Fixed member coordinates resolve through the exact typed body/concrete
operation,with full access current/prefix-state/location and invocation data.
Function source refinement,entry representation,frame,partial correctness,
clause definedness/postconditions,region control and all loop invariant/rank
obligations remain explicit. Zero-node functions still require function/source
proof work. Demands concern all reachable activations,not one evaluated sample.

Calls retain same-U source-body/contract dependencies and caller precondition/
relation checks; defaults grant no theorem. Total callers add conditional call
termination for their actual invocation arguments without falsely declaring a
default partial callee globally total. Adapters retain registered source-binding
requirements. The current incomplete foundation has no ordinary elaborator for
these requests,so all169 request IDs stay unelaborated. Go skeleton emission
raises a distinct pending exception. Caller ready/expanded flags and empty
contexts cannot override it. These requests are not ordinary propositions or
proofs; final signatures/definitions,statement expansion and program assembly/
skeleton closure remain W04 requirements and T02–T06 implementation/proof work.

Certificate source manifests link exact frontend-manifest and validated common
VC schema/hash. VCs depend on frontend manifests only,so the graph stays
acyclic. Linked metadata does not supply a certificate or proof verdict.
Registered practical producer/toolchain/release provenance remains required.

Eight proof bundles and181 cases yield six retained VCs,six retained skeletons,
eight linked-unproved certificate source manifests,two unelaborated Go VCs,two
pending Go skeleton requests and157 configuration rejections. Twenty-five
targeted tests include false postconditions,conditional call totality,copied
authority,zero-node functions and malformed/deep JSON. Fifty-six project-bound
local fragments retain416 member/access demands. All260 Python/Rust/Go inputs
agree: eight VCs,eight certificate manifests,six private skeleton hashes,two Go
programs,169 requests,56 private fragment hashes and11 boundaries. Native
observers run twice identically; the receipt pins673 inputs and the same14,496-
file Go1.25.0 inventory. No universal source or preservation proof is inferred.

The [verification record](../migrations/go-practical/t01-w04/proof-artifacts/verification/verification.json)
retains selected raw unit/replay/metadata/new-source format/compile logs and
selection reasons,with the initial canonicalizer setup failure. Preceding
capture/module evidence and all143 predecessor paths stay unchanged. Unaffected
older proof/native matrices are not repeated. The cumulative T01 gate stays
deferred to W05.

W04 remains In progress. Complete Go ordinary elaboration/final skeleton,
frontend-envelope/policy/API/release roots,complete foundation/source signatures
and reviewed migrations remain due. All independent source/AST/control/lifetime/
activation and ordinary operation/permission/frame/totality proofs remain
required. W05 owns inactive publication and the T01 gate; all T02–T11 work
stays active and no practical Go profile is activated.

## 49. T01-W04 ordinary word circuits and captured value specializations

Entry: `ee1083c14e7050e08a411f0fcda7e1d78ec1abba`, clean before this work.
The [ordinary word component](../migrations/go-practical/t01-w04/word-foundation/README.md)
adds actual ordinary full-bit8/16/32/64 definitions for arithmetic,bitwise
operators,comparison,division/remainder,shifts and all width conversions.
Shared carry/shift/division stages keep the graph polynomial. Unsigned division
retains the lost high remainder bit; signed magnitude/sign correction preserves
minimum/-1 wrapping and dividend-signed remainders. DivideDefined excludes
zero divisors independently of explicit totalized model values. Shifts retain
all64 count bits,with signed source counts sign-extended and separately guarded.
Source literal representability precedes bit construction.

The word carrier is exactly the retained full Bool-indexed vector. A new
untrusted generator builds ordinary Bool.cases definitions and Eq.congr/trans/
symm proofs; no core opcode,reduction rule,axiom,extensionality or eta rule is
added. Universal Bool parity/majority/equality,compact carry-zero lemmas and
pointwise AddZero/XorZero/AndOnes/NotNot quantify arbitrary input words and all
addresses. They are basic model laws,not a full source arithmetic proof.

Thirteen distinct nominal/builtin word types produce787 concrete captured
GP-OP-01/02/03/04/05/06/07/09 specializations. Each whole operation is regenerated
before selecting separate Value and Defined ordinary definitions. Full context,
scope,operation hash and input/output nominal IDs remain bound; equal core
widths do not identify source types. All word pairs and count types participate
in conversion/shift specializations. Changed operations,caller formulas and
proof flags reject. These are value/definedness components; parent source,
evaluation,state,permission,frame and whole-function goals stay required.

Both unchanged private Rust/Go checkers accept four word sets and the complete
source-value set with zero axioms and identical output hashes. The combined
certificate has2,756 declarations,19,718 terms and513,817 bytes,including2,575
new definitions and146 new ordinary theorems. It fits the preceding candidate
proof envelope. Twelve deliberately false carry/add/division-definedness claims
and four ill-typed shifts reject in both cores. All21 cases emit twice byte
identically. The original77-source joint Bool.recType/Nat.induct proposal and
retained ordinary emitter stay unchanged; no production checker or Certificate
v0 is modified.

All1,640 actual ordinary-AST interpreter observations agree with an independent
integer oracle and real pinned Go operators:1,312 word results,288 Boolean
results and40 runtime-safety failures. Signed minima,overflow,high remainder
bits,negative shifts,full64-bit shift counts and conversion signedness remain
explicit. The real Go observer runs twice identically. Finite comparisons do
not establish universal source correctness or registered capture provenance.
The receipt pins685 inputs and the retained14,496-file Go1.25.0 inventory.

The [verification record](../migrations/go-practical/t01-w04/word-foundation/verification/verification.json)
retains targeted checks,selection reasons and six exact initial failure
snapshots. Balancing finite predicates and naming carry lemmas solves emitter
depth without changing its parser. Correct quantified binders and explicit
congruence solve nested-iota proof composition within existing rules. Prefixing
full operation hashes with H satisfies canonical declaration names. Failed
constructions remain distinct from false source claims and intended controls.
Sixteen targeted tests cover these interfaces and evidence boundaries.
Unchanged older matrices are not rerun; prior proof/capture/module/source pins
and all143 inventoried predecessor paths remain checked.

W04 remains In progress. Full-width Nat/Int interpretation/conservation and
source/arithmetic/division/shift proofs remain required,along with all remaining
contract/operation/source/state/permission/frame/control/totality definitions.
The preceding169 complete Go requests remain unelaborated. Final ordinary
statement/program/skeleton assembly,frontend-envelope/policy/API/release roots,
reviewed migrations and W04 freeze remain due. W05 owns inactive publication
and the cumulative T01 gate; all T02–T11 implementation/application/release
work stays active. No practical Go profile is activated.

## 50. T01-W04 exact Nat/signed bridge for the actual word addition

Entry: `04fd6d9f267b705cdfc93d408b45a1ec5780a8bc`, clean before this work.
The [numeric bridge](../migrations/go-practical/t01-w04/word-numeric-bridge/README.md)
connects the actual full8/16/32/64-bit ripple definitions to the retained
W03 balanced Nat/signed library. Every aligned binary block proves exact
outgoing carry,pointwise sum and sign-stage incoming-carry correspondence.
Parents compose checked children; only local three-bit full-adder flags and
addresses are case-split. Arbitrary complete input words remain quantified.
Pointwise value congruence follows the finite balanced definition and assumes
no function extensionality or eta.

The unsigned theorem proves genuine Std.Nat successor-denotation conservation
with the actual final2^width carry,for every incoming Bool and arbitrary initial
Nat. The signed theorem proves balanced natural cross sums under equality of
the actual incoming/outgoing sign-stage carries,keeping both input signs and
the same result sign. No caller true flag supplies that guard. It is a
conditional mathematical integer-addition claim,while overflowing Go addition
retains its modular result. General logical Int and remaining word arithmetic
proofs are still due.

Thirteen exact preceding GP-OP-05/add Value and Defined declarations are
reconstructed and copied unchanged. A checked word definition equality is
lifted through sign/value/Nat contexts to prove the value component's arithmetic
for each distinct source type. Full operation/scope/nominal references remain
bound; theorems for equal-width nominal types retain different names. These
claims do not establish selected Go AST/evaluation,source correspondence,
state/permission/frame or enclosing function correctness.

Four positive sets pass both unchanged private checkers with zero axioms and
identical certificate/export/axiom hashes. The64-bit set has1,159 declarations,
25,583 terms and337,297 bytes,including the unchanged187-declaration W03 seed.
Every one of20 cases emits twice byte identically. Fourteen altered local
carry,leaf projection,overflow premise or dropped-carry statements reject by
paired core type mismatch. The32/64-bit dropped-final-carry candidates exhaust
conversion fuel in both checkers; they are retained failed candidates rather
than false-source evidence. No resource/format failure is counted as a semantic
negative. The unchanged77-source joint proposal and ordinary emitter are built
fresh; production checkers,Certificate v0,old helpers and installed profiles
are unchanged.

All288 actual ordinary ripple/guard observations agree with actual typed Go
addition,math/bits carries,math/big sums and a separate integer oracle. Both
incoming carries,signed extrema,overflow,high-bit patterns and initial naturals
0,17,10^50+37 remain explicit. All unsigned carry equations hold;210 actual
signed no-overflow cases satisfy the cross sum and78 overflow cases do not.
Finite comparisons do not replace the universal checked Nat theorem or supply
a source/application proof. The same14,496-file Go1.25.0 inventory is retained.

The [verification record](../migrations/go-practical/t01-w04/word-numeric-bridge/verification/verification.json)
retains selected local checks,raw logs and two initial positive conversion
findings. Using the exact incoming parameter avoids root Carry0 conversion.
Explicit word equality before lifting the actual source Value avoids a whole
Nat-expression conversion. In that initial source-wrapper candidate Rust
exhausted fuel while Go accepted; both original outcomes remain recorded.
All final positives pass both without changing checker budgets. Twelve targeted
tests cover proof bytes,isolated mutations,source identities,complete block
closure,actual ripple equations and fuel/pending boundaries. Unchanged older
matrices are not rerun; the relevant seed and all143 predecessor/source paths
remain checked. The cumulative T01 gate remains deferred to W05.

W04 remains In progress. Full word/Nat/Int/contract arithmetic,all remaining
source/state/permission/frame/control/totality definitions and proofs,complete
Go statement/program/skeleton assembly,downstream roots and reviewed migrations
remain required. The preceding169 Go requests stay unelaborated. W05 owns
inactive publication and the T01 gate; all T02–T11 work stays active. No practical
Go profile is activated.

## 51. T01-W04 ordinary unbounded arithmetic and closed logical components

Entry: `61c8569f7ad0b3b6e3f31089f90b9432b9a11046`, clean before this work.
The [ordinary arithmetic component](../migrations/go-practical/t01-w04/ordinary-arithmetic/README.md)
adds599 ordinary members:36 definitions and563 theorems,including small checked
equality checkpoints. Genuine Std.Nat induction proves addition/multiplication
algebra,cancellation,equality/order translation and truncated-subtraction laws.
Theorems quantify unbounded naturals. The unchanged35-declaration induction
seed is rechecked in every certificate; neither core reduction nor parser or
conversion budgets change.

Logical Int uses a redundant Bool->Nat pair p-n. Equiv is genuine Nat cross-sum
equality,not core equality of pair functions. Ordinary proofs establish
equivalence,ring laws,operation congruence and representative independence of
equality/order. Half-open forall,exists,Nat sum and Int sum use the exact count
upper-lower. Empty rules include reversed bounds. Checked fold-preservation
theorems require only actual bounded FoldForall equality over the visited
prefix; no outside-range premise or normalization assumption is introduced.

All27 existing normalized closed logical heads now construct ordinary
Value/Defined definitions with independently inferred types,payloads,arity
and lexical Nat binder scopes. Exact decimal literals use binary declaration
DAGs. The compiler emits unproved Boolean statements; ordinary_proof remains
null. Accepting a false clause's Value definition establishes its type without
proving the clause. Complete reconstruction rejects caller proof/source flags.
Twenty-two component cases retain one typed/unproved clause,19 configuration
rejections and two pending cases. Source/word/storage expressions and private
construction limits remain explicit pending boundaries.

Two ordinary sets pass both independent Rust/Go checkers with zero axioms and
matching certificate/export/axiom hashes. The theory set contains634
declarations,11,424 terms and153,208 bytes. The combined set typechecks149
closed components,including ten large values. Eight altered definitions or
proofs fail by paired ordinary type mismatch:bad addition,unrelated
equality/order premises,positive-only Int equality,incomplete multiplication,
false empty forall,inclusive upper bounds and a forged proof of a false clause.
Every certificate emits twice identically. No resource failure is counted as
false-claim evidence. The unchanged77-source private proposal and emitter are
built fresh; production sources,Certificate v0 and installed profiles do not
change.

All149 real pinned Go/math-big results agree with an independent integer
oracle,including negative/redundant integers,nested binders,empty/reversed
ranges and values beyond2^64 and10^70. Actual ordinary-AST execution also agrees
on139. Ten large cases reach its private512 unary-loop ceiling and remain
model_resource_pending; they still pass component typechecking and native/
integer comparison. This ceiling does not bound logical Nat/Int or invalidate
a proof. Finite comparisons do not establish general elaborator correctness.
The retained14,496-file Go1.25.0 inventory remains pinned along with714 inputs.

The [verification record](../migrations/go-practical/t01-w04/ordinary-arithmetic/verification/verification.json)
retains selected local commands,raw logs and the initial producer error.
A local variable shadowed the negative-projection constructor; renaming it
repairs generation. The exact original sources and traceback remain preserved.
No certificate or source claim was produced by that failed construction.
Sixteen targeted tests cover native proof outcomes,all logical heads,complete
reconstruction,integer representatives,bounded folds,nested binders,large-value
limits and unproved/source boundaries. Unaffected older matrices are not
rerun; preceding numeric/word/proof evidence and all143 predecessor paths are
checked. The cumulative T01 gate remains deferred to W05.

W04 remains In progress. The connection from actual word denotations to this
general arithmetic,remaining word/source arithmetic,partial expressions,
bounded definedness and independent elaborator correctness remain required.
All remaining state/permission/frame/control/totality proofs,complete Go
statement/program/skeleton assembly,downstream roots and reviewed migrations
stay open. All169 retained complete Go requests remain unelaborated. W05 owns
inactive publication and the T01 gate; all T02–T11 work remains active. No
practical Go profile is activated.


## 52. T01-W04 full-word addition in the ordinary Nat/Int library

Entry: `d796283d544fbc17bd435583ff103353bba7dcbe`, clean before this work.
The [word/arithmetic connection](../migrations/go-practical/t01-w04/word-arithmetic/README.md)
uses the retained634-declaration arithmetic seed and exact187-declaration word
seed. A new untrusted exporter checks both and compares complete symbolic
kind/type/value/proof/level/family signatures of22 common declarations. It
exports165 unchanged ordinary members,all rechecked by both independent cores.
The export and every one of20 complete certificates reproduce twice exactly.
No core rule,parser/conversion budget,old helper or semantic identity changes.

At each cube depth,checked composition proves Value/Weight interpretation for
arbitrary unbounded Nat coefficient k and initial n,whole words and an explicit
pointwise premise forall n.f(n)=n+k. Named powers square from1+1 and each has a
checked ordinary definition equality. Successor translation follows from the
preceding AddOne theorem. No function extensionality or full operand-pattern
enumeration is assumed. Actual ripple addition now has genuine Nat carry
conservation and conditional semantic Int addition at8/16/32/64 bits. Signed
interpretation is Int.Make(unsigned,sign-selected2^width); its cross sums use
actual Positive/Negative computation laws and the actual sign-stage carry guard.

Thirteen exact source-typed Value definitions preserve source type,operation
hash,scope and preceding word definition theorem. Checked word equality lifts
each actual Value through the new denotation. Unsigned claims retain final
carry; signed claims retain the actual incoming/outgoing sign-stage equality.
An explicit lemma removes literal zero incoming. These are arithmetic claims
about Value definitions,with source/AST/evaluation/frame/enclosing-function
proofs still unavailable. All169 complete Go requests remain unelaborated.

Four positive sets pass both checkers with zero axioms and matching certificate,
export and axiom hashes. The64-bit set contains1,831 declarations,38,672 terms
and539,484 bytes,including the unchanged634-declaration seed. Sixteen altered
final-carry,overflow-premise,result-sign or signed-denotation candidates reject
by paired core type mismatch. No failed resource/format candidate is counted
as a semantic negative. The unchanged77-source private proposal and ordinary
emitter are rebuilt fresh along with the new untrusted exporter. Production
checkers,Certificate v0 and installed profiles remain unchanged.

All96 actual pinned Go addition/math-bits/math-big observations agree with an
independent integer oracle across all four widths,both incoming bits,signed
extrema,overflow,high-bit patterns and fixed-seed operands. Seventy no-overflow
cases satisfy the signed relation;26 overflow cases stay modular. All24
selected8-bit observations execute the actual new unsigned/signed denotation
ASTs over actual ripple outputs and agree with the independent interpretation.
Larger unary-model denotation executions were not selected; all full-width
ordinary proofs are checked. Finite observations do not prove general source
or compiler correctness,and the preceding ten logical-model resource-pending
cases keep their status. The same14,496-file Go1.25.0 inventory is retained.

The [verification record](../migrations/go-practical/t01-w04/word-arithmetic/verification/verification.json)
preserves raw commands,selection reasons,review and five initial finding groups:
exporter dependency lock,composed producer context,computed Nat/Int projection
typing,large-iteration conversion fuel,observation query-field mismatch and
embedded temporary dependency paths in the new exporter binary.
Exact original sources/plans/certificates/results are retained as applicable.
A wrong initial Go CLI invocation is separate from the proper core result.
Generic checked ApplyEquality and MaybeTranslation theorems give direct result
types for concrete iterations,repairing32/64-bit fuel without a budget change.
Naming powers alone was insufficient. Explicit literal and projection equality
lifting repairs nested recursor-major comparisons. Sixteen affected tests
exercise proof bytes,isolated controls,export closure,all source identities,
actual guards,independent observations and source/pending boundaries. The
receipt pins729 inputs. Earlier native matrices are not rerun; preceding
arithmetic evidence and all143 inventoried predecessors remain checked.

W04 remains In progress. Other word arithmetic/order/division/shifts/conversions,
source/partial expressions,bounded definedness,independent elaborator correctness,
remaining state/permission/frame/control/totality proofs,complete Go statement/
program/skeleton assembly,downstream roots and reviewed migrations remain due.
W05 owns inactive publication and the cumulative T01 gate. All T02–T11
implementation/application/release work remains active; no practical Go profile
is activated.


## 53. T01-W04 common frontend request,success and diagnostic roots

Entry: `b76e65ec96ef6da05e44153569a33808185e02e4`, clean before this work.
The [frontend envelope draft](../migrations/go-practical/t01-w04/frontend-envelope/README.md)
binds complete pre-lowering semantic requests,immutable available snapshots
and independent release references. Closed success and diagnostic roots bind
that complete request,context,selection and profile contract. Success requires
complete linked VIR/map/frontend-manifest artifacts; diagnostic arms have no
artifact fields. Each whole-root hash has its own domain. Caller readiness,
proof and registered-run flags reject.

A new private observer links unchanged public CLI/VC APIs. Native context and
selection validators supply every actual retained request-policy observation;
unknown queries never become assumed success or rejection. Common source
artifacts project into the unchanged public successor process validator.
Whole source-map/manifest values and VIR content hashes match five independent
retained context/target baselines; a sixth retained bundle preserves its exact
explicit synthetic-origin permission. Existing C# generic-code behavior and
its prefixed codebook remain intact,while Java independently retains its closed
codebook. These source-fixture comparisons do not prove a general migration.

Available retained snapshots bind all staged bytes,while the success manifest
selects an exact captured subset. Go snapshots use existing capability-bound
source objects and do not require a returned capture-context ID before lowering.
Successful Go model artifacts still require the whole independent ProjectSpec
validator. Go diagnostic code/status/phase/messages form a closed fixed table;
optional spans resolve captured source-object IDs and integer byte bounds,
including EOF,with optional captured function/source agreement. Raw compiler
prose and host paths are not public Go messages. Diagnostic acceptance remains
model evidence and does not assert a compiler observed that error.

The conformance matrix covers request/response/IR generation mixing,complete
context/selection/request/source/release bindings,artifact omission and
corruption,unknown fields,codebooks,span identities and byte bounds,canonical
transport,exit/status/issue relationships and process/resource precedence.
The actual compiled API is exercised at inclusive stdout/stderr limits,
including real256MiB and256MiB+1 buffers. Canonical depth768 is inherited from
the preceding proof canonicalizer; retained inner limits and public code remain
unchanged. No extra256-depth wrapper precheck narrows retained term inputs.
Native queries reproduce twice exactly,as do full request/success/diagnostic
preimages across Python,Rust and the pinned Go observer.

The [verification record](../migrations/go-practical/t01-w04/frontend-envelope/verification/verification.json)
records selected local validation,raw results,source/binary pins and initial
request-oracle,context-generation,runner-syntax and unobserved-query exception
findings. Their failed
construction/validation stages remain separate from native source rejection
and from ordinary proof verdicts. The old request cache was left unchanged;
a new actual public-policy observer handles the needed source selections.
The current exact foundation-linked context factory replaces the accidentally
chosen earlier context generation. No production code,installed profile,old
helper or checker rule changes.

All611 conformance cases match. The142 actual public-API queries and165 complete
hash preimages reproduce twice identically; every new Go code/allowed phase is
represented across33 fixed codes. Eighteen targeted tests pass,covering the
complete matrix,independent baselines,input subsets,closed codebooks,source
spans,actual limits,process precedence,unknown-query failure,all preimages and
unproved/unregistered boundaries. The receipt pins810 inputs,including310
linked public consumer/build sources. An absent cache query now raises a probe
assertion that cannot be caught as an ordinary configuration rejection. Full
source/model reconstruction remains checked; duplicate reconstruction during
materialization is avoided without dropping any native or case observations.

Every accepted fixture result keeps registered_execution and
source_semantics_proved false. No frontend is invoked; no Go profile is
activated. All169 complete Go requests remain unelaborated. Preceding word/
arithmetic certificates and all143 inventoried predecessors remain pinned;
unaffected native matrices are not rerun. W04 remains In progress. Other
word/source/partial/definedness/elaborator/state/permission/frame/control/
totality proofs,complete Go program/skeleton assembly,policy/API/release roots
and complete reviewed migrations remain due. W05 owns inactive publication
and the cumulative T01 gate; the full T02–T11 objective remains active.


## 54. T01-W04 policy roots and exact-source ordinary certificate evidence

Entry: `e23fa47a2768d9cf41cec7be0cee9341dfe5f7d5`, clean before this work.
The [policy evidence draft](../migrations/go-practical/t01-w04/policy-api/README.md)
adds closed scan,evidence,reproduction and receipt v4 roots. They bind the
complete independent context,selection,compiled contracts,frontend request/
success and source artifact graph. Evidence binds the VC and certificate
manifest. Receipt status is regenerated from actual proof state,not caller
readiness,AI/CI output or a supplied checked flag. Canonical scan identity is
raw SHA256; the other roots use their distinct reserved v4 domains. The DAG
has no evidence/receipt back edge into certificate source metadata.

A new untrusted observer calls unchanged actual public policy and API methods.
All six retained source bundles pass scan and API generation. Two Rust targets
reach actual dual-checker ordinary certificate acceptance at zero axioms.
Four other private source-shape fixtures stop at the existing public assembly
interface with a non-Std.Bool carrier error. Those are recorded as assembly
failures. No false source/compiler verdict or ordinary typing counterexample
is inferred,and fixtures are not replaced to turn the matrix green. Complete
profile-preservation and actual application proof coverage remain required.

For checked sets,the actual complete VC functions and statement declarations
match the retained common source graph. The untrusted adapter changes only
Certificate v0's opaque source-manifest payload to the whole validated common
certificate manifest. Both unchanged source-free checkers recheck identical
resulting bytes. Export and axiom-report hashes remain exactly unchanged;
the certificate hash now covers the actual common source metadata. Checked
declaration interfaces,zero counts,all paired hashes and embedded bytes are
bound into the evidence. Old certificate hashes are not relabeled.

Two actual counterexamples stamp checked Rust proof bytes with each Go
common certificate manifest. Both kernels accept those mathematical proofs,
but policy rejects the purported checked-Go evidence: all169 Go demands are
still unelaborated and no complete ordinary context/statement/skeleton exists.
A source hash plus kernel acceptance is insufficient to prove Go source goals.
A corrupted-magic control records format rejection separately. Imported scan/
evidence controls exercise actual public regeneration and reject altered
source,contract,helper,checker,property and canonical-transport claims.

Six actual API traces cover all seven context-aware operations. Thirty helper
responses reach session/import/VC/list/target states;72 failures leave mutation
counts unchanged. Unknown candidates/proofs,source-context/selection/hash
mixing,unknown fields,transport variants,repeated state transitions and unknown
targets remain fail closed. Existing APIv2 routes remain unchanged. New APIv4
and AI-explanation roots are still unfrozen work; helper responses confer no
source-proof authority.

The [verification record](../migrations/go-practical/t01-w04/policy-api/verification/verification.json)
retains selected local commands,source/binary pins,actual results,review and
initial observer compile/cache/source-binding/report-shape findings. The
observer uses a cached pinned Linux-amd64 Rust1.95 image on the local arm64
Docker engine with no network,read-only repository and unprivileged UID.
Exact dependency archives match the retained root lock and unpack only into
a task-owned cache. This is emulated targeted consumer evidence,not native
Linux release acceptance or checker/process-tree resource measurement. The
native README release gates remain required. Source mutation during an early
bootstrap build was detected and that stale build was discarded; authoritative
builds require complete before/after source-pin equality.

The immutable source-authority comparison prevents a cached ID from blessing
a modified row or a Go-to-retained dispatch change. The retained reference
checker legitimately omits zero counts and uses short hash keys; the new
adapter validates that exact protocol and compares all three hashes to the
Rust report. Probe construction/transport errors remain distinct from actual
assembly,checker and source/readiness outcomes. Production sources,old helpers,
installed profiles and the core/certificate rules remain unchanged.

The866-case matrix,79 actual Linux queries and44 hash vectors reconstruct
exactly. Twenty selected tests pass. A focused adapter trial found that copied
valid reports could accompany an altered byte carrier; the adapter now also
recomputes the existing Certificate v0 domain hash over those exact bytes.
The modified test reruns after repair,and the original source/trial remains
preserved. All real native observations and mathematical proofs remain
unchanged. The receipt pins833 inputs,including313 linked consumer/build files.
Metadata checks186 strict JSON files,143 predecessors and29 documents/504 links.
The full T01 gate remains deferred to W05.

W04 remains In progress. New API/AI/release roots,full Go ordinary statement/
program/assembly/skeleton definitions,remaining word/source/state/frame/control/
permission/definedness/totality and elaborator proofs,complete reviewed migrations
and activation review remain due. W05 owns inactive publication and the
cumulative T01 gate. The full T02–T11 implementation/application/release scope
remains active; no practical Go profile is activated.


## 55. T01-W04 complete API route contracts and source-bound helper transactions

Entry: `c4ba7a93116f22bd1f8c922396928187e48aa63d`, clean before this work.
The [API contract design](../migrations/go-practical/t01-w04/api-contracts/README.md)
closes request,response and session v4 shapes for all33 currently declared
routes,including the ordinary term/proof/module family. Every field is
required,objects are closed,IDs reject Boolean coercion,structural routes
retain checker-profile requirements,and full source/context/selection
identities precede dispatch. Practical CSharp's reserved v3 family and all
existing routes/profiles remain unchanged. A declared route is not evidence
that its executor exists; the model explicitly rejects ordinary/candidate
operations requiring a real typed executor instead of simulating success.

All three root identities are raw SHA256 of complete canonical values without
LF. API transport has exactly one LF. The envelope retains the existing1MiB
allowance and adds four levels around common768-depth source artifacts;
embedded limits and source validation remain independent. The copied
iterative JSON algorithm changes only those two envelope constants. Actual
native raw hashes cover24 small complete documents; they do not claim native
implementation of the full new API envelope budgets. Generated VC output
retains the full common VC and VIR import retains unit/function counts.
The preserved response-shape review prevents a ref-only migration from
silently losing the existing usable VC result.

Snapshots bind server ID,incarnation,revision,full semantic context/selection,
module/proof profile,lifecycle and exact source state. They are server-owned
helper data,not importable authority. Successful mutations commit prepared
state once,after output validation/encoding. Stale snapshots,restarts,cross-
session/context requests,revision exhaustion,failed imports,encoding failures
and frozen writes cannot change source state or leak new interned IDs.
Read-only listing preserves revision. Actual arena rollback/serialization
and production incarnation generation remain explicit T07 obligations.

Across eight source bundles,the96 model transactions include48 atomic
operation failures. All169 Go demands remain unelaborated; list returns no
invented ordinary members,and candidate routes/source export reject while
pending. Ordinary-ready retained VCs are proof targets,not proved claims.
Every response/snapshot is helper-only. Response import compares exact locally
regenerated transaction bytes; valid hashes alone cannot bless a forged result.
Transport/preflight errors,operation errors,executor/setup/resource failures
and theorem/kernel verdicts retain their distinct meanings.

A review reproduced a Python equality defect: replacing an integer with a
Boolean in supplied VIR left Python dictionaries equal despite different
canonical bytes and a stale claimed hash. The exact candidate,accepted private
model response,old source and receipt are preserved. Source/context/selection
comparisons now use complete canonical bytes. The dedicated adversarial test
requires source-binding rejection with no state change. This was a private
model defect,not an observed native API or kernel acceptance bug.

Six fresh unchanged public APIv2 traces reproduce all102 prior operations and
72 atomic failures. The local pinned Linux-amd64 observer and both independent
Rust/Go canonical/raw hash observations each run twice. They preserve the
complete old outputs,errors and mutation counts. Linux execution remains local
emulation on the arm64 engine; native Linux release/resource gates remain due.
The [verification record](../migrations/go-practical/t01-w04/api-contracts/verification/verification.json)
separates19 targeted tests,the affected response-payload rechecks,initial review
findings and the full gate deferred to W05. The6800 configuration/transport
vectors and847 source pins reconstruct exactly. The metadata check retains
all143 inventoried predecessor paths. Production source and old helpers are
unchanged; no new public API endpoint/profile is activated.

The W04 completion boundary is now explicit in the worklist and work plan:
finish AI/release roots,ordinary statement/program/skeleton/source-binding
predicate signatures and budget closure,then audit every consumer migration
and requirement owner. Undefined semantic rules still prevent freeze.
Full general compiler/elaborator/source/operation/definedness/state/frame/
control/totality proofs remain mandatory in their assigned T02–T06 gates;
they are not all preconditions for finishing the design-only W04. W05 owns
reviewed inactive normative publication and the cumulative T01 gate before T02.
W04 and T01 remain open and the complete T02–T11 objective remains active.


## 56. T01-W04 complete AI explanation/redaction/batching contracts

Entry: `021036e407684caa32f66aab4bc6d1af79849187`, clean before this work.
The [AI contract design](../migrations/go-practical/t01-w04/ai-explanation/README.md)
adds exact request/explanation v4 roots and new common projection/redaction
recipes without widening reserved practical-CSharp v3 or active v2 contracts.
Both roots use raw SHA256 of whole canonical JSON without LF. Required nested
fields,language,public parameter/category allowlists,the complete old theory/
axiom summary and local provenance arms are explicit. The existing prompt and
prose response schema remain pinned; prepared provider bytes are inert.

The source authority independently validates full source/context/selection,
policy evidence and receipt. Providers receive no private names,paths,source
text,contract descriptions,diagnostics,foundation/source/evidence hashes or
preprocessor symbols. The local report restores source identities,source
order,statuses and full context/contract/ref bindings from its own immutable
prepared data. Import regenerates the whole report from the same source and
separately retained prose/observations; valid hashes or provider flags alone
are insufficient. Every explanation remains untrusted helper analysis with
proof_evidence=false and source_access=false. Even prose claiming all source
verified cannot promote a pending obligation.

The32-property per-request limit stays fixed. Stable status/category/evidence/
source-index ordering partitions the entire list; aliases have unique global
ordinals across batches. Equal-size swapped responses reject. All169 Go
unelaborated demands appear exactly once,85 and84 across three batches each.
An empty assembly-error property set yields one zero-property diagnostic batch,
with not-run checker state rather than an invented false theorem. Future
materialized Go must use exact policy-owned ordinary targets/properties and
the same full projection; that final binding remains with the ordinary-program
and consumer-migration review. Registered retained axiom policies and all
native Java AI preservation remain mandatory at activation.

The1328-case matrix covers16 English/Japanese bundles and24 complete requests.
All40 complete request/explanation raw identities agree with independent Rust
and Go canonicalizers/hash observers,each repeated twice. Text limits count
UTF8 bytes and reject forbidden controls/bidi,incomplete/duplicate/foreign
aliases,extra status/context/proof/provenance fields and missing batches.
The [verification record](../migrations/go-practical/t01-w04/ai-explanation/verification/verification.json)
keeps targeted test reasons,raw logs,source pins and the initial report review.
That review restored explicit language,locale-correct fixture prose and the
existing checked-theory/format/five-category-axiom summary before final hashes.
No provider was contacted and no Go demand was discharged.

Two existing native test bodies each run twice in the pinned local offline
Linux-amd64 container. They exercise real unchanged public AI consumers over
GoFixed,Rust and CSharp policy/certificate fixtures,including deterministic
redaction,report/provenance/status/prose rejection controls and frozen vector
ownership. Exact root-lock dependency archives provision only a task-owned
cache; repository sources/fixtures stay read-only and their before/after pins
match. Java's native test is not executed here. This is local emulation on
arm64,not a native installed-release/cgroup/process-tree gate. Source-free
proof acceptance in preceding evidence remains scoped to its original claims;
new wire/model validation supplies no new proof or registered-run authority.

W04 remains open for release roots,ordinary-program/materialized-policy
predicate/signature/budget closure and all143 consumer migration dispositions,
including retained axiom-policy preservation and historical-probe/new-source
baseline handling. W05 owns inactive normative publication and the cumulative
T01 gate before T02. All general compiler/source/ordinary/definedness/state/
frame/control/application proofs remain required in T02–T06,actual AI/API/
policy/release integration in T07,and complete v2 T08–T11 work remains active.


## 57. T01-W04 release roots,metadata provenance and non-promotable candidates

Entry: `0a51f97caac7b2fc9654fa95d48896bd118526d5`, clean before this work.
The [release contract design](../migrations/go-practical/t01-w04/release-contracts/README.md)
closes seven root/nested/hash/transport shapes. Generic inventory.v1 retains
its exact member format; executable modes are separately bound by the whole
versioned descriptor hash. Existing runtime/component/host/layout semantics
are retained. New bundle IDs require new component inventory owner scopes,
existing-domain0.1 component hashes and matching target-library refs. A review
preserves the initial wrong-scope projection; actual file bytes/modes do not
change and historical hashes remain at their original source epoch.

The new graph contains4 frontend/4 toolchain metadata projections and5 old
profile/target tuples. All are explicitly metadata-only. The6 required
contexts include an absent practical-Go tuple; no old Go binary is relabeled
as practical Go. Valid metadata and rehashed passed flags cannot authorize
new-protocol execution or activation. Candidate/evidence/receipt states remain
incomplete/not-run,with proof authority none. No source or kernel failure is
invented when a builder,host primitive or gate observation is absent.

The acyclic order is semantic/foundation inputs -> frontends/toolchains ->
bundle registry -> consumer embeds those refs -> outer image registry ->
candidate -> actual gate evidence -> receipt -> atomic activation. A consumer
never embeds its own outer release hash,and a frontend never embeds a registry
hash containing its own binary hash. Complete byte/mode snapshots,two matching
native gate runs,all required contexts,source/corpus/four-profile preservation
and compare-and-swap whole-image activation/rollback remain required in T07.
Historical frontend linkage-fixture hashes cannot identify the new full
registry; T02 must regenerate the whole source/VC/certificate/downstream graph.

A new compact native observer calls the unchanged actual release validator
and selector for every existing tuple. It resolves all4 languages/5 targets
and rejects reserved-v2,unknown-field and hash controls. Two complete runs
match. All13 full descriptor/registry/candidate/evidence/receipt preimages
agree with independent Rust and Go canonical/domain hash implementations,
each run twice. Chunk transport avoids inflating a public decoded-string
limit. The229 vector cases reconstruct exactly and ten targeted tests pass;
three affected tests recheck component-owner/hash linkage and all final graph
refs after review. The [verification record](../migrations/go-practical/t01-w04/release-contracts/verification/verification.json)
records selection reasons and distinguishes targeted checks from the T01 gate
deferred to W05.

Initial setup findings preserve read-only lockfile generation,missing cache
target and a dummy bootstrap executable overwriting the real observer output.
The empty exit0 output was rejected as setup failure,not treated as a validator
success. Bootstrap uses a separate target directory; the actual leaf package
was cleaned/rebuilt and the real protocol was queried. Authoritative builds
pin complete before/after source bytes and exact root-lock dependency versions.
No old helper,installed registry,production source or public profile changes.
Local Linux-amd64 emulation proves metadata/hash observations only; the native
Linux installed-release/cgroup/process-tree gates remain mandatory T07/T11.

W04 remains open for ordinary-program/skeleton/source-binding and materialized
policy/AI signatures,exact Go host/resource budgets,and all143 consumer
migration dispositions including retained axiom policies and historical/new
source epochs. W05 owns reviewed inactive publication and the cumulative T01
gate before T02. All general source/compiler/ordinary/application proofs and
the complete T02–T11 implementation/activation/release objective remain active.


## 58. T01-W04 final interfaces,consumer review and design freeze

Entry: `e19887077b7de9f512be72bfed52227ed4beebec`. The
[program binding design](../migrations/go-practical/t01-w04/program-binding/README.md)
specifies19 ordinary theorem families, complete source/IR/contract/representation
definitions and the source/assembly/binding/VC/skeleton/manifest DAG. An existing
declaration interface hash contains a term index, so the whole context tables,
exact ordinary types and unchanged final-certificate prefix must also bind the
statement. Source well-formedness/correspondence cannot become caller axioms.
All169 Go demands remain unelaborated; all56 concrete operation interfaces
reconstruct. Materialized policy properties must enumerate all exact statements
and dependencies; AI cannot supply proof or target authority.

A distinct practical-Go host fixes120 seconds/2GiB for the frontend tree,
zero swap,128 tasks,512MiB scratch and exact inherited capture/transport limits.
Proof construction and each checker retain60 seconds/1GiB and2MiB/131,072-term/
4,096-declaration certificate caps, with whole-tree enforcement required.
Representative Darwin measurements motivate these budgets without promising
every admitted source fits. Actual native Linux enforcement, process cleanup
and complete application-corpus fit remain T07 gates.

The [migration review](../migrations/go-practical/t01-w04/consumer-review/README.md)
assigns every original143 consumer and40 additional indirect/owner files to
explicit action, source/test owner and work item. All24 v1 requirements have
concrete design evidence and future implementation/proof ownership. Go's
zero-axiom policy and retained mvp-theory formats/allowlists/five-category
accounting remain distinct. Inactive C# namespaces stay reserved. No existing
assembly-error fixture or historical helper is rewritten to imply acceptance.

A real historical checkpoint is replayed from the exact entry commit/tree:
1,025 tracked files/218,743,291 bytes, including the observed reads, explicit896
pins and the complete historical Python package. Export/import uses
`/usr/bin/git` outside the sandbox, changes no ref or checkout and removes the
temporary tree. Exact bytes/modes match before and after the unchanged helper;
229 cases,13 hashes and5 old target tuples reconstruct. The first incomplete
read trace omitted a source module loaded from cached bytecode and correctly
failed with ModuleNotFoundError. The final package includes all historical
Python sources, so cold imports succeed. Initial epoch test setup also needed
exist_ok when an input lives at the repository root; all3 adversarial tests
pass after that correction. These are setup findings, not source/proof failures.

The [completion audit](../migrations/go-practical/t01-w04/completion-audit.json)
closes W04's design boundary. Selected checks cover only changed interfaces,
readiness/retention handoff, epoch replay and metadata consistency; full
`check-fast` is deferred to W05. W05 still owns reviewed inactive normative
publication and the cumulative T01 gate. No actual Go proposition, compiler
correctness proof, native new frontend, host gate or activation is claimed.
All T02–T11 implementation/application/proof/release requirements remain active.


## 59. T01-W05 inactive normative publication; cumulative gate running

Entry: `68a44416e1b1a6dc3a84587cca784fc4b3a2b862`, W04 design complete.
Five inactive [specifications](../migrations/go-practical/t01-w05/README.md)
publish the profile, ordinary foundation, shared artifacts and exact additive
Nat/Bool interfaces. Five conformance sets join the normative manifest; all26
predecessor sets and their contents remain unchanged, giving31 sets total.
Exact generation checks all255 immutable W04 pins, complete new member bytes,
owner/spec references and explicit unimplemented/proof/activation statuses.
No placeholder production test or profile route is added.

Publication reconstruction and the normal specification-vector checker pass.
Both installed registry files retain their original hashes and contain no
practical-Go entry. The [source/feasibility review](../migrations/go-practical/t01-w05/source-feasibility-review.json)
records independent source/IR interpretation, exact context/type binding,
canonical table/name-prefix construction and measured representative proof
feasibility. General source/compiler/operation/application proofs and native
host/application/release gates remain explicitly assigned. All169 Go demands
remain unelaborated.

After publication, the unchanged historical release checkpoint again
reconstructs229 cases/13 hashes/5 target tuples from its original1,025-file Git
epoch. No historical helper, observation or frozen source input is rewritten.
Current implementation evidence must use a fresh source epoch and execution.

The cumulative local `./scripts/check-fast.sh` is actually running for all T01
changes at this final W. Format and workflow-absence checks have passed; the
5GiB/approximately29,000-file obsolete-interface scan is still active. Clippy,
workspace tests and final certificate acceptance/rejection checks have not yet
completed. Its [result record](../migrations/go-practical/t01-w05/verification/check-fast-result.json)
and raw logs are authoritative. W05/T01 remain **In progress** until the complete
gate passes; T02 has not started. This is progress on the active full T01–T11
objective, not a completion or release claim.


## 60. T01-W05 full-gate findings,final vocabulary and retained input epochs

The initial whole-repository scan remained active after1,434 seconds. Equivalent
literal prefilters retain all127 matchers/four scopes and exact offsets/results.
All9 existing positive/negative cases and3,206 differential cases agree, including
escaped/inherited JSON context, symbol whitespace and path-only findings. The
completed scan takes about258 seconds and exposes132 occurrences in35 files;
its original failed output is retained, not counted as a pass.

The final inactive [source-kind amendment](../migrations/go-practical/t01-w05/source-kind/README.md)
uses go_file and an exact eight-member enum. Old candidate graphs cannot be
aliased or relabeled as final source evidence. One complete amended source-object
preimage agrees in existing independent Rust and Go hash engines,two runs each.
This is metadata evidence only. No source admission,ordinary program proof or
registered frontend execution is inferred; the169 preceding demands remain
historical unelaborated fixture data. T02 must create a fresh complete graph.

The successor fixture manifest seals36 exact historical files:27 frozen private
model/fixture/failed-source records,7 intact compressed C# proof receipts with
incidental prose-byte matches,and2 failed-gate raw logs. Independent code/data
tables pin path/hash/size/class/epoch. Ten changed/foreign/path/epoch controls
reject. No directory or production file is exempted;127 detection rules and the
previous12 retained declarations stay unchanged. The full scan now passes over
29,266 active files with48 validated historical/retained records. Scanner output
uses current terminology so its own logs do not create active obsolete prose.

Clippy then passes. The first workspace run exposes a pre-existing retained C#
T05-W01 pin mismatch: its request/response fixtures were rebound by the already
approved Boolean-proof refreeze40469d29,while the conformance record still names
the original b0c9a5d5 bytes. Both original files are restored as exact historical
Git objects. Original conformance,current fixtures,source/contract bodies,limits
and C# profile definitions are unchanged. The retained test checks both epochs,
all56 request/response pairs,complete source and raw sidecar bodies outside
context/own-hash values,unchanged other facts,and current file digests. Deliberate
invalid-Unicode cases remain intact. The focused test passes,and all9 boundary
cases pass in the complete retry. No old receipt is rewritten to describe a new
native run.

The cumulative [gate](../migrations/go-practical/t01-w05/verification/check-fast-result.json)
is still running through the workspace tests at this checkpoint. The transition
trace test has passed; its longer idempotency/snapshot test remains active.
W05/T01 are not complete and T02 has not started. All T02–T11 implementation,
source/application proof,stage gates and native T07/T11 release work remain
mandatory. [Verification details](../migrations/go-practical/t01-w05/verification/verification.json)
retain superseded/failed attempts separately from actual passing checks.

## 61. T01-W05 retained CSharp epochs and cumulative gate findings

Commit `94b544f4887ba4edae3ee6867bbeac82c88e248c` repairs the current
control-capture epoch with actual pinned Roslyn/CLR execution: all26 source
captures and255 CLR results agree outside28 sidecar input identities. All16
affected emission tests pass; original observations stay unchanged.

The next full diagnostic gate uses --no-fail-fast and exposes14 further
metadata/source-golden test failures. The [repair record](../migrations/go-practical/t01-w05/verification/retained-csharp-repair/verification.json)
distinguishes the still-running original compiled test binaries from repaired
source and selected checks. The31-set manifest, exact147-to176 reference
extension, CSharp pending ledger status and historical freeze/publication
replays now pass their selected tests. Three retained epochs separately bind
original published inputs, the later private Boolean candidate and entry-main
status documents. Old Python generators and observations stay unchanged.
Current CSharp T01-W09 remains In progress; T01-W10/T06-W09 remain Blocked,
with all987 application proofs pending. Historical replay grants no present
application-proof or activation authority.

The ordinary aggregate count assertion now matches the existing grouped
pipeline's4 +2,048 charged transformers, with unchanged16,384-cell semantics.
Three new boundary golden directories reconstruct all68 contexts per family.
Their nine hex files contain the same definition bytes as retained parent
files, now linked to current source contexts by actual generation and normal
strict import/golden/mutation checks. Each of three distinct certificates
passes both current Rust and Go source-free checkers with zero axioms and
matching hashes. These are definition certificates; application proofs remain
pending. All four changed VC/source tests pass. Four export-guard tests and
targeted format/vocabulary/publication checks also pass.

The original full gate continues through long CSharp scalar oracle tests. Its
existing failures remain retained. A new full gate over the repaired commit
is still mandatory before W05/T01 complete or T02 begins. No production
source, registered profile, frozen Go input, CSharp payload or installed
registry changes in this repair. The full T02–T11 objective remains active.

## 62. T01-W05 diagnostic supersession and explicit deep-observer stack

The old diagnostic invocation is superseded after4,251 seconds, retaining its
complete raw stdout/stderr and actual exit code -15. It had14 failed assertion
tests plus a separate libtest stack overflow. The failed-test-line count alone
omitted that abort; the complete stderr review records it explicitly. Its
remaining scalar oracle cases did not finish, and no passing gate is claimed.

The abort belongs to the synthetic computed-predicate sharing observer with
2,049 nested Boolean operations. The same original binary passes on a64MiB
test-worker stack, with427,009 shared versus3,664,442 repeated evaluation steps.
Its cfg(test) harness now allocates that explicit stack, matching other deep
semantic oracles. All operations, both semantic results and the >2x repeated
work control remain. Production evaluator/checker code and limits are unchanged.
The affected libtest is checked with RUST_MIN_STACK unset.

The repaired whole gate is next, using test opt-level1 while explicitly
retaining debug assertions and overflow checks. Every workspace test remains
selected; no Cargo source/build descriptor or release profile changes. The
[verification record](../migrations/go-practical/t01-w05/verification/retained-csharp-repair/verification.json)
keeps selected passes, the abort, supersession and the pending full retry
distinct. W05/T01 remain open and T02 remains unstarted.

## 63. T01-W05 JSON archive source correspondence

The source43 full retry confirms all14 previous assertion failures and the
explicit-stack observer pass. Its ongoing409-test VC target then reports a
JSON string parser original-source failure. An exact replay of the same
compiled test with output capture disabled fails in5.87 seconds at a missing
previous-pair-pipeline file. That archive retains original source IDs, while
the current input requests use their later Boolean-context IDs. This is not
a failed parser semantic assertion or a completed full gate.

Two exact original Git objects from09a14bdde6b14408e4f25e55cb787dd989577d7e
are retained in the [JSON boundary epoch record](../migrations/go-practical/t01-w05/retained-csharp/json-boundary-epochs/README.md).
A closed three-pair test correspondence pins both complete epochs, compares
all source/contract/root/native bodies outside context-derived fields and
checks all raw native input digests before comparing. Altered source, contract
and root controls are required. Historical archive lookups use original IDs;
current JSON string/token golden reconstruction uses separate new directories.
No old observations are overwritten or promoted to current source evidence.

The affected single integration target is being compiled in a separate
directory. Fresh generation, normal golden tests, both-checker definition
validation and a repaired cumulative gate remain required. The original full
diagnostic process continues with its original binary and failed raw output.
W05/T01 remain incomplete and T02 is unstarted. All169 Go theorem requests and
987 CSharp application proofs retain their preceding pending status.

The first new Rust guard attempt fails both cases because scalar serde JSON
cannot parse a retained lone UTF16 surrogate inside a sidecar. The original
guard source/binary identity/logs are retained. The guard now uses the existing
practical UTF16-preserving parser and writer, with an additional substitution
control; raw inputs and all comparison obligations remain intact. Its
incremental build and selected verification remain pending at this checkpoint.

## 64. T01-W05 JSON selected verification and diagnostic supersession

The UTF16-preserving guard now passes both tests: all three original/current
source/native-fact pairs agree, nine altered source/contract/root controls and
one lone-surrogate substitution reject. Both actual generators reconstruct all
68 contexts and three nonempty certificates each. The six new JSON string/token
hex files are byte-identical to their unchanged parent definitions. Both normal
tests pass with generation flags removed; every strict import, forged metadata,
cross-context mismatch and complete archived scan/definition comparison remains.
Format and the single affected integration target's Clippy pass without warnings.

All24 original scan archive members and six retained parent files match exact
source43 Git objects. The six new members contain two distinct definition
certificates, both accepted by unchanged Rust/Go development checkers with zero
axioms and matching hashes. These are definition checks, not application/source
proof or native release/resource acceptance. The [selected repair record](../migrations/go-practical/t01-w05/verification/retained-csharp-repair/json-boundary-repair/verification.json)
retains the original failed scalar-only guard and unchanged raw UTF16 fixtures.

The old source43 gate is superseded only after those concrete repairs pass.
Its actual terminal exit143 follows controlled termination of its owned Cargo
parent/test child; original raw logs and hashes are retained after9,962.26 seconds.
It had1,020 passing lines, one JSON archive lookup failure, and no additional
stack/fatal target abort. It was partial and supplies no passing T01 gate.
Every remaining target and all workspace tests must run again over the committed
repaired tree. T01/W05 remain incomplete and T02 remains unstarted. No practical
profile or application-proof status changes.

## 65. T01-W05 calendar counter and structural archive epochs

The source320 cumulative diagnostic confirms the repaired JSON token original
sources, then reports one calendar-token historical preservation assertion.
Its unchanged binary reproduces the failure in2.898 seconds: a former
register-cube count result is compared with the reviewed shared17-bit result.
The [diagnosis](../migrations/go-practical/t01-w05/verification/retained-csharp-repair/json-calendar-diagnosis/result.json)
preserves the original binary/source, both certificate inputs and raw error.

All177 token definitions retain their own bodies. The exact transitive
inventory changes from2,325 to2,299 declarations: only the known count result
body changes and its26 old private helpers retire. All retained types and
the other2,298 bodies remain exact. The repaired check separately compares
the entire current closure with its immutable producer pin, including the
new counter body. It passes in4.778 seconds; altered token/dependency/counter
bodies and a counter type all reject. No definition or fixture changes.

The related original structural-boundary historical test independently fails
on the old counter's missing Success global. Its four pairs also span later
reviewed domain/aggregate changes. The
[retained epoch manifest](../migrations/go-practical/t01-w05/retained-csharp/structural-boundary-epochs/manifest.json)
pins all16 original/checkpoint/current/receipt members against actual Git
objects. Complete before-quoted-decimal to pre-count-refresh comparisons pass
for all four pairs; the two aggregate pairs retain the unchanged exact
8,192-step reassociation audit and every old transitive declaration check.
The successors were recorded by fe9dc4fc, not the earlier09a14bdd epoch; the
failed earlier-path lookup remains a setup observation, not a proof rejection.

The repaired structural test preserves those complete comparisons and adds
changed-checkpoint controls plus separate current canonical-pin checks.
Its actual rebuilt execution now passes all four historical pairs in3.177
seconds and rejects four altered checkpoint definitions. The calendar recheck
passes in6.020 seconds with all four controls rejected. The separate actual
current-source producer passes all four current contexts in51.670 seconds,
including strict import, golden, source/context/program mutations and complete
dependency comparisons. Targeted format and the affected integration target's
Clippy pass without warnings. The [selected record](../migrations/go-practical/t01-w05/verification/retained-csharp-repair/json-calendar-repair/verification.json)
keeps those results distinct from retained historical/checker observations.
No production or existing certificate bytes change. The source320 no-fail-fast
diagnostic keeps running to collect other failures. A new complete repaired gate must
pass before W05/T01 completes or T02 starts. All169 Go demands and987 C#
application proofs retain their pending status; neither profile is activated.

## 66. T01-W05 numeric-capacity comparison epoch

The unchanged source320 diagnostic later fails the JSON numeric-capacity
declaration-name equality assertion. The original compiled case reproduces
the failure in1.056 seconds, exit101; its complete raw name-set assertion and
all five original source/certificate input hashes match the source320 Git tree.
The [diagnosis](../migrations/go-practical/t01-w05/verification/retained-csharp-repair/json-numeric-capacity-diagnosis/README.md)
keeps the original failure separate from the repair. Each current family
retires27 DomainCount helper names and changes the retained Result body in
addition to the original six grammar blocks, without changing retained types.

The [sealed epoch manifest](../migrations/go-practical/t01-w05/retained-csharp/json-numeric-capacity-epochs/manifest.json)
binds eight exact Git members, including both complete after-cell-count
certificates from09a14bdde6b14408e4f25e55cb787dd989577d7e. At those actual
retained before/after epochs all2,092 array and2,295 sum declaration names and
types agree, and exactly the original six grammar bodies change in each pair.
The other2,086 and2,289 bodies remain exact. No helper namespace is exempted.
These are bounded historical syntax comparisons, not current semantic proofs
or additional checker acceptance.

The repaired original assertion passes in29.688 seconds and rejects an
unreviewed body, type and missing-declaration mutation in each pair. Both
separate unchanged actual current-source tests pass: arrays in5.284 seconds
and sums in6.202 seconds, including strict import, component dependency closure,
metadata/certificate mutation and normal golden comparisons. Targeted format
and the affected integration target's Clippy pass without warnings. The
[selected record](../migrations/go-practical/t01-w05/verification/retained-csharp-repair/json-numeric-capacity-repair/verification.json)
records the actual optimized/debug-assertion/overflow-check profile and every
raw input/result digest. No production implementation, existing certificate,
source capture, checker rule or resource limit changes.

The source320 no-fail-fast diagnostic continues with its original binary to
collect remaining failures. Its known two assertion failures have selected
repairs; the run itself cannot pass or complete T01. Preserve its terminal raw
result, then run the complete gate over committed repaired source before
completing W05/T01 or starting T02. All169 Go demands and987 C# application
proofs remain pending, and neither practical profile is activated.

## 67. T01-W05 complete aggregate archive preservation

The same retired-count-helper mismatch reproduces in two still-queued
source320 assertions: aliased consumers fail in0.155 seconds and the historical
decimal parser comparison in0.119 seconds. The current combined/component
closure already passes. The [original diagnosis](../migrations/go-practical/t01-w05/verification/retained-csharp-repair/aggregate-archive-diagnosis/README.md)
retains both raw failures, the original executable/source and all477 input
observations. The [sealed epoch manifest](../migrations/go-practical/t01-w05/retained-csharp/aggregate-preservation-epochs/manifest.json)
binds716 exact Git members, reusing102 unchanged archived checkpoint vectors
and copying123 from the actual09a14 checkpoint without changing existing bytes.

The repaired original tests retain every one of225 vectors and all twelve
non-cost metadata comparisons:133 vectors reassociate exactly8,192 StepTwo
calls and92 remain byte-identical. Every original declaration and dependency
stays checked; the shared comparator/normalizer is unchanged. All twelve
families reject an altered old definition. This test passes in11.739 seconds
and separately validates all225 current certificate pins. The decimal test
passes in0.793 seconds, retaining the complete current component closure and
checking both complete historical closures, with current/historical body
mutations rejected. Targeted format and integration-target Clippy pass.

Both unchanged current producer owners already pass actual regeneration and
normal golden comparisons in the completed source320183-case lib target.
Its exact raw output prefix and unchanged producer Git bytes are retained as
completed-target evidence; it is not a terminal or passing whole gate. The
[selected verification](../migrations/go-practical/t01-w05/verification/retained-csharp-repair/aggregate-archive-repair/verification.json)
keeps this observation, historical regression comparisons, repaired test
executions and the pending whole retry distinct. No production or existing
fixture bytes change, and no new checker acceptance or source/application
proof is inferred. W05/T01 remain incomplete, T02 remains unstarted, all169
Go demands and987 C# application proofs remain pending, and no profile activates.

## 68. T01-W05 live gate resource observation

The same source320 gate and owned test/Cargo processes remain live. A short
read-only process sample records ten test-worker stacks in the unchanged
ordinary test evaluator and a main harness waiting for results. The
[exact observation](../migrations/go-practical/t01-w05/verification/retained-csharp-repair/full-gate-live-profile/observation.json)
and lossless raw sampler bytes are retained. Only two samples per thread were
emitted, so no statistically reliable hot-path attribution, completion or
deadlock-absence claim is made. The fixed-eight-entry identity memo and its
original source320 bytes remain unchanged.

The sampler reports physical footprint54.0G and peak58.2G on the64GiB macOS
host. The next complete repaired gate will use `RUST_TEST_THREADS=4` to reduce
aggregate concurrent host memory pressure, retaining the existing opt-level1,
debug assertions and overflow checks. This changes test-worker scheduling only;
all tests, full bounds, assertions, CLI build and accept/reject checks remain
mandatory. No resource limit or production/checker source changes. The current
diagnostic is not stopped or reconfigured. The retry setting is planned, not
executed or passed, and no performance gain or native Linux resource acceptance
is inferred. T01 remains incomplete and T02 remains unstarted.

## 69. T01-W05 Option source-epoch lookup repair

The unchanged source320 diagnostic reports two additional Option core failures.
Their exact isolated reproductions fail in0.124 and0.126 seconds at the old
document-ID lookup, before certificate loading or packet evaluation. The
[diagnosis](../migrations/go-practical/t01-w05/verification/retained-csharp-repair/json-option-core-diagnosis/README.md)
retains both original sources, complete raw outputs and three separate diagnostic
preparation errors. Ten source/guard/metadata/certificate/request-response members
match their exact original Git bytes. The current seven-row metadata contains
the guarded41dc96 document partner of the original7a157f source; the old row is
absent. No parser-semantic or checker rejection is inferred from these lookups.

Both repaired tests require the unchanged complete three-pair source/native-fact
guard before selecting that current document; the i32 case also requires exactly
one Option with an i32 payload. All17 i32 cases and256 bits per packet pass in
65.590 seconds, preserving4,352 original assertions. All four string cases pass
in45.948 seconds with the original9,804 selected wide-packet observations and
UTF16 cell counts. Original case matrices, assertion suffixes and runtime helpers
remain byte-identical. Wide selected observations are not promoted to universal
or full-packet proof.

The [selected verification](../migrations/go-practical/t01-w05/verification/retained-csharp-repair/json-option-core-repair/verification.json)
records both complete source-correspondence checks, nine altered source/contract/
root controls and one surrogate substitution rejection. The affected current
source producer regenerates and strictly imports the one selected document,
compares its existing program/certificate golden and retains dependency/mutation
checks in6.413 seconds. Unaffected source contexts are not repeated here.
Targeted format and integration-target Clippy pass without warnings. No producer,
evaluator, checker, existing fixture, source capture or resource limit changes.

The original diagnostic continues with four reported failures; seven known
original isolated failures now have selected passing repairs. No partial run or
selected result supplies T01's full gate. Preserve the terminal original run,
resolve remaining findings and execute the complete repaired gate with all
tests and the recorded four-worker scheduling before T02. All169 Go demands and
987 C# application proofs remain pending, and neither practical profile activates.

## 70. T01-W05 current component proof inputs

The source320 diagnostic also reports stale component-record comparisons in the
actual-default and identity proof owners. Exact unchanged-binary reproductions
fail in0.413 and0.704 seconds before their original semantic/kernel checks. The
[original diagnosis](../migrations/go-practical/t01-w05/verification/retained-csharp-repair/default-proofs-diagnosis/archive-git-audit.json)
retains both failures and verifies all138 old foundation/identity archive members
against source320 and their original publication commits. Their original paths
and bytes are unchanged. The complete current producer tree, captured C# fixture
tree, replay helper and registered foundation/profile inputs also match source320.

The unchanged foundation owner generates45 current contexts into a separate
directory. Its87 type sequents,442 operation sequents,1,026 outcome records and
all987 pending application IDs remain exactly equal to the retained records.
The identity owner preserves its two full sequents and projection records.
Every metadata difference is enumerated; hashes and hash-derived theorem names
are not erased to make a current comparison pass. Current proof owners compare
complete fixed current metadata/certificate bytes, including newly added normal
golden checks. Reversing only those additions and input-path associations
recovers all three original sources byte for byte, including99 assertions,
complete loops, prefix/import checks, mutations and64MiB worker stacks.

The [selected verification](../migrations/go-practical/t01-w05/verification/retained-csharp-repair/default-proofs-current-generation/verification.json)
records all three complete generation and separate normal-mode tests passing
over all45 contexts each. The default generation takes1,612.841 seconds and its
normal mode1,915.528 seconds, preserving three default proofs,534 supplied and
453 remaining sequents, all987 pending application IDs,32 source-use defaults
and42 unchanged certificates. Both unchanged independent checkers accept all49
distinct positive certificates with zero axioms and identical hashes and reject
all five false certificates with CoreCheck errors. An initial observer JSON
field-name error remains separately retained with both actual raw rejection
outputs; it is not a semantic checker disagreement. Targeted format and
integration-target Clippy pass. No production/checker implementation, limit or
test bound changes.

After every selected result is actually terminal and passed, the
[sealed manifest](../migrations/go-practical/t01-w05/retained-csharp/current-proof-inputs/manifest.json)
binds all189 current members,138 retained old members,45 nested context records,
all generation/normal/dual-checker receipts and the exact original6ffd owner
executable snapshot. It is created once without replacing any old corpus.
The source320 whole diagnostic still collects known assertion failures. Its
terminal result must be retained and the entire committed repaired gate must
pass before W05/T01 completion or T02. All169 Go demands and987 C# application
proof IDs remain pending, and neither practical profile activates.

## 71. T01-W05 control preservation intervals

The source320 whole diagnostic next reports a control built-in-throw declaration
preservation failure. Its unchanged-binary isolated reproduction fails in0.230
seconds after the first program's declaration checks pass. The
[exact assertion operand audit](../migrations/go-practical/t01-w05/verification/retained-csharp-repair/control-builtin-throw-diagnosis/exact-original-failure-diff.json)
finds only three root context hashes differ. Approved-freeze commitf7333bc9
updates all eighteen current JSON records in exactly those three fields; all
certificate files and every other metadata value are byte-identical to the
complete-source-execution checkpoint561db360. Existing inputs remain unchanged.

The initial same-epoch complete-after candidate passes eight original owners
but exposes five more metadata failures. Their exact four-field differences
are a later resolved alias frame. The
[refined manifest](../migrations/go-practical/t01-w05/retained-csharp/control-preservation-epochs/refined-manifest.json)
retains all528 Git members and all228 original program comparisons. Those five
earlier feature intervals end at the retained pre-source-step checkpoint, and
the unchanged source-step owner checks the final extension for all eighteen
contexts, including exactly two resolved alias frames and their source anchors.
Both legs remain mandatory. No additional field is ignored or normalized.

The [selected verification](../migrations/go-practical/t01-w05/verification/retained-csharp-repair/control-builtin-throw-repair/verification.json)
records all thirteen repaired normal-mode tests passing in9.334 seconds. All46
original helper assertions, all complete declaration/term/level prefixes, nullable
join checks, metadata comparisons, eight native-operation resolutions and
program/memory counts remain byte-identical. Reversing only the added byte-pin
loader and after-input selection recovers the complete original source exactly;
the shared declaration-body comparator and all current producer/runtime code
are unchanged.

Five isolated mutations forge a source hash, source-function identity, public
projection, alias source anchor or canonically valid cross-context certificate.
All reach and fail assertions in the original comparison body; setup, decoding,
pin-loader and runtime failures cannot satisfy these controls. The separate
unchanged current source owner passes all eighteen contexts, strict imports,
complete execution metadata/bridge checks and all36 normal JSON/hex golden
comparisons in13.577 seconds. Targeted format and integration-target Clippy pass.
No production/checker implementation, existing fixture, capture, limit or bound
changes. Failed original/candidate runs and the separate archive-driver parse
error remain preserved with exact inputs and raw outputs.

The same original whole diagnostic continues to collect late findings. Its
terminal result must be preserved and the complete repaired gate must pass over
committed source with the recorded four-worker scheduling before W05/T01
completion or T02. All169 Go demands and987 C# application proof IDs remain
pending, and neither practical profile activates.

## 72. T01-W05 decimal-clause response echo representation

The original source320 whole diagnostic reports the decimal-clause candidate
owner as a new failure. Its unchanged-binary reproduction fails in15.031 seconds
only at the final `responses.json` output, after all nine source contexts,44
operation aliases/closures,49 attachments, current metadata/certificate byte
goldens and original strict import/mutation checks pass. The
[exact assertion audit](../migrations/go-practical/t01-w05/verification/retained-csharp-repair/decimal-clauses-diagnosis/exact-response-operands-audit.json)
retains both136,077-byte operands. Every JSON value is equal;98 object member
orders differ after the existing `serde_json::Value` pretty serialization.

The [fixed serialized echo](../migrations/go-practical/t01-w05/retained-csharp/decimal-clause-response-echo/README.md)
is the exact original right operand. Both representations are SHA256-pinned and
the new helper compares complete JSON values before selecting that separate
fixed byte golden. The original captured input remains unchanged. No field is
removed, normalized or ignored. The original final assertion still compares
every output byte. Reversing only the helper and path selection recovers the
complete original source, all17 assertions and every runtime case/count.

The [selected verification](../migrations/go-practical/t01-w05/verification/retained-csharp-repair/decimal-clauses-repair/verification.json)
records the repaired candidate passing all nine contexts in13.022 seconds,
format and target Clippy. An initial added-helper type reference caused a
compile failure before tests ran; its exact source and raw compiler diagnostic
remain separately retained. Read-only input ordering checks find the six other
response echo families already use sorted object members; those checks do not
claim their tests or native captures passed.

The changed final output helper is exercised by the complete candidate owner.
Unchanged expensive runtime probes remain mandatory in the complete repaired
gate. The original no-fail-fast diagnostic continues with20 reported failures,
all now covered by selected passing repairs. Static input review and the entire
committed-source repaired gate remain pending; neither this selected result nor
any partial diagnostic completes W05/T01 or permits T02. All169 Go demands and
987 C# application proof IDs remain pending, and neither practical profile
activates.

The original-source runtime owner is subsequently also reported as failed by
the still-running source320 diagnostic. The entire repaired normal owner passes
in1,090.264 seconds over all nine contexts, all10 original selected runtime
conditions and5 failure definedness rejections, with no runtime filter/output
flag. Its original runtime assertions, loops and counts remain unchanged.
[Raw selected results](../migrations/go-practical/t01-w05/verification/retained-csharp-repair/decimal-clauses-repair/runtime-normal/result.json)
and the exact executable snapshot are retained. The terminal original panic
details remain to be reconciled. A later fixed-codec original-source owner now
also fails in the original diagnostic and is selected-repaired in section73.
The entire repaired full gate remains mandatory.

## 73. T01-W05 fixed-codec request echo representation

The source320 whole diagnostic next reports the fixed-codec original-source
owner failed. The [unchanged-binary reproduction](../migrations/go-practical/t01-w05/verification/retained-csharp-repair/fixed-codec-clauses-diagnosis/result.json)
passes all six codec contexts,44 source clauses and21,088 formatting/composition
bit observations, then fails only the final `requests.json` echo in139.696
seconds. Its two exact operands retain identical complete JSON values and every
nested UTF8 sidecar string. The original57546-byte ASCII-escaped and observed
57366-byte UTF8 pretty encodings differ in representation only; no object
member order changes. Exact encoders reproduce both observed operands, including
every original full-width invalid digit. All20 old fixture members match source320.

The [fixed echo](../migrations/go-practical/t01-w05/retained-csharp/fixed-codec-clause-request-echo/README.md)
is the exact original right operand. Three guards pin both original and echo
bytes and compare complete JSON values before the original full byte assertion.
The private helper and path selection reverse to the complete original source,
including all23 assertions, valid/invalid/oversized parse cases and all format
and complete parse-format bit loops. No original field, native capture, fixture,
producer, interpreter, checker, test bound or resource limit changes.

The [selected verification](../migrations/go-practical/t01-w05/verification/retained-csharp-repair/fixed-codec-clauses-repair/verification.json)
records the full repaired normal owner passing in142.775 seconds, format and
target Clippy. The mixed owner does not echo requests and remains byte unchanged;
its original whole pass is retained and it remains required in the repaired gate.
A read-only fourteen-input representation audit, validated against two actual
Rust assertion operands, finds no further predicted encoding difference beyond
the two diagnosed input echoes. It is not a test or native-capture pass claim
for the other families.

A final changed-input review precedes the commit, and the entire committed
repaired gate remains required. The original whole diagnostic continues; all
component certificate checks now pass both checkers as recorded in section70. These selected component regressions do not complete W05/T01 or permit
T02, discharge the169 Go demands or987 C# application IDs, or activate profiles.

## 74. T01-W05 transition and rich-literal request echoes

The original source320 diagnostic next reports transition, control source-step
and rich-literal source owner failures. Exact unchanged-binary reproductions
show the transition and literal owners fail only at their final request echo
in0.677 and1.640 seconds. Their complete JSON values and every nested UTF8
source/sidecar string are identical; ASCII Unicode escapes versus literal UTF8
change only the outer encoding. The [exact operands and selected repair](../migrations/go-practical/t01-w05/verification/retained-csharp-repair/input-echo-followup-repair/verification.json)
retain both original inputs and separately fixed actual serialized operands.

Each private output helper pins both representations and compares complete JSON
values before retaining the original full byte assertion. Reversing only these
helpers/path selections recovers both complete original sources, all11 assertion
invocations per file and all original cases, seeds, selector/packet loops, imports,
mutations, integration/definition-closure comparisons and component goldens.
No field, source value, fixture, producer, interpreter, kernel, checker, test
bound or resource limit changes.

The complete repaired transition owner passes both contexts and999 projection
observations in1.724 seconds. The literal owner passes18 constants and1,414
packet observations in2.907 seconds, including source/integrated/public closures.
Build, format and target Clippy pass. Other completed repair source bodies stay
unchanged and are not re-run as an unrelated intermediate suite.

The control source-step original owner is separately running over all18 contexts
with every original step/component/SSA-origin and positive/negative check, no
filters/output flags/ceilings. Its result is not yet known. Final input review
and the entire committed repaired gate remain required; W05/T01 stays open and
T02 is unstarted. All169 Go demands and987 C# application IDs remain pending.

## 75. T01-W05 complete source-step component observer

The [exact original source-step invocation](../migrations/go-practical/t01-w05/verification/retained-csharp-repair/control-source-steps-diagnosis/result.json)
terminates after6,339.122 seconds with target exit101. The original line129
component-count assertion sees actual3/expected2 during the first `count_fill`
case. All eighteen original contexts were selected and every original runtime
check was retained; failure interrupts that invocation before completing them.

The [complete metadata audit](../migrations/go-practical/t01-w05/verification/retained-csharp-repair/control-source-steps-repair/failure-analysis.json)
finds three omitted existing components across all eighteen golden contexts:
memory effects in `count_fill`/`index_update` and the source value constructor in
`type`. All42 original input members and the three relevant production sources
match source320 Git bytes. This is an obsolete test observer, with no producer,
fixture, interpreter, kernel, checker or resource-bound change.

The [additive repair](../migrations/go-practical/t01-w05/verification/retained-csharp-repair/control-source-steps-repair/README.md)
retains all original count summands, exact list comparisons and SSA/native/
literal/slot checks, then validates the additional memory snapshot bindings and
constructor public VIR signature, anchor, hashed definition and SSA argument
indices. Reversing the additions recovers the entire original source file and
all112 original assertion invocations; seven assertions are added. Every
original runtime case, mutation and bit position remains.

The initial E0624 build error and its exact input source/drivers are preserved.
The corrected public-VIR inspection builds in34.270 seconds; format and targeted
Clippy pass in3.246 and4.638 seconds. The complete original normal owner is now
running over all18 contexts, including every original positive/negative check,
strict import and all36 JSON/certificate goldens. No passing runtime result is
yet claimed. Final input review and the entire committed repaired gate remain
required before W05/T01 completion or T02. The original source320 whole
diagnostic also continues; all169 Go demands and987 C# application IDs remain
pending, with no profile activation.

## 76. T01-W05 source-step SSA definition points

The first complete repaired source-step owner fails after5,476.090 seconds at
the newly added memory-binding observer. The [exact failed operands](../migrations/go-practical/t01-w05/verification/retained-csharp-repair/control-source-steps-repair/ssa-origin-followup/binding-origin-analysis.json)
contain eight bindings. Four source snapshots already agree; four SSA entries
differ only in `node_id` because the observer copied invocation use sites in
place of the original VIR definition points. The failed source/executable/raw
logs are retained. No production semantic or original fixture change is needed.

The [follow-up](../migrations/go-practical/t01-w05/verification/retained-csharp-repair/control-source-steps-repair/ssa-origin-followup/README.md)
independently reconstructs the unique definition point from original parameters,
phis, literals, invocation normal results and handler exceptions, checking both
value and type IDs. It retains every original assertion and runtime path. The
original `run_steps` entry hardcodes true; existing metadata owners additionally
share the same component observer with false, ending before runtime evaluation.
There is no environment switch or runtime-bound reduction.

The source audit recovers the entire original step file and the committed145675
harness byte for byte:112 step and329 harness assertion calls remain, with nine
additional step assertions. All42 original input members and three relevant
production sources match source320 Git bytes. The corrected build passes in
30.721 seconds. Both affected existing metadata consumers pass: all18 contexts/
582 complete source steps in6.680 seconds, and the dedicated41-step constructor
context in0.650 seconds. These are metadata checks, not runtime or application
proof claims. Format and targeted Clippy pass in2.585 and4.046 seconds.

The full original eighteen-context runtime retry is now running with every
original example, mutation, bit position, strict import and all36 JSON/certificate
goldens. Final input review and the entire committed repaired gate remain
required before W05/T01 completion or T02. The original source320 whole diagnostic
continues. All169 Go demands and987 C# application IDs remain pending, with no
profile activation.

## 77. User amendment: defer C# tests and proceed with Go

On 2026-10-19 the user explicitly instructs: “C#のテストはスキップして開発を進めてください。”
The [recorded amendment](../migrations/go-practical/t01-w05/verification/go-development-gate/amendment.json)
supersedes historical C# whole-gate retry prerequisites for GO-PRAC development.
The final W of each Go T uses `./scripts/check-fast.sh --skip-csharp`. The
default command and native Linux installed-release gates keep their scope;
C# validation/release remains deferred, and no profile is activated.

The [routing check](../migrations/go-practical/t01-w05/verification/go-development-gate/routing-check.json)
verifies the exact Cargo workspace partition: all70 other integration targets,
every unit library, test-enabled binary and doctest stay selected across eight
packages;37 dedicated C# integration targets and C# unit/doc names are deferred.
Shared tests, format, strict scanner, Clippy, CLI build and two accepting/two
rejecting fixtures remain required. No C# test is restarted for this amendment.

The [already terminal selected runtime](../migrations/go-practical/t01-w05/verification/go-development-gate/previous-selected-runtime.json)
passes all18 original contexts in10,017.322 seconds:582 local relations,
9,852 true/10,853 false observations,20,725 total observations and36 current
JSON/certificate goldens. All earlier failed attempts and raw logs remain.
The original [source320 diagnostic](../migrations/go-practical/t01-w05/verification/go-development-gate/source320-disposition.json)
retains1,300 reported passes and28 failures, but has no terminal exit status.
Actual processes and old sessions are absent. It remains partial evidence and
counts as no passing gate. No C# application proof is promoted by these results.

The amended cumulative Go development gate remains pending. T01 is incomplete
and T02 is unstarted until it passes and T01's completion audit is recorded.
All987 C# application IDs and169 Go demands remain pending.

## 78. T01 completion under the explicit C# test deferral

The [committed-source Go development gate](../migrations/go-practical/t01-w05/verification/go-development-gate/run/result.json)
passes in598.171 seconds at implementation `4b4c3a01ddfa2377d7ccc0116f128649ff3ec3b1`.
All594 selected tests pass, and all16 planned Cargo commands and70 integration
targets are observed. Format, strict scanner, Clippy, no-workflow check, CLI
build and two accepting/two rejecting fixtures pass. The37 dedicated C# test
targets and C# unit/doc names are deferred under section77's explicit user
instruction. This result supplies no default all-language or C# release pass.

The [current completion audit](../migrations/go-practical/t01-w05/completion-audit.json)
checks W01–W05 against the work plan. It rechecks all255 frozen design members,
all five published specifications and five new vector sets, the31-set manifest,
seven source-model/proof-review inputs and both unchanged installed registries.
The original targeted publication checks pass. No semantic design decision is
undefined. The reviewed133-input checkpoint has zero scanner/whitespace/link
findings. No public profile activates; source semantics and all169 application
demands remain assigned to T02–T06. All987 C# application IDs remain pending.

T01/W05 is complete under the user amendment. T02-W01 is ready to implement
private contexts and immutable capture with real module/replacement/export/
compiler identities and explicit rejection of escape, stale data, network and
toolchain fallback. Native Linux release acceptance remains owned by T07/T11,
and the complete T01–T11 objective remains open.

## 79. T02-W01 entry and final source-object executor

T02-W01 starts from T01 completion commit
`fe1b85cb70c7ea390c02469706d09cdc913f973f`. Its [full requirement matrix](../migrations/go-practical/t02-w01/requirements.json)
keeps private context contracts, real dependency/module/replacement capture,
source/export/compiler identities, immutable namespaces and all required
escape/stale/network/toolchain rejection cases pending until implemented.

The first [executor component](../../go-tools/go2vir/practical_source.go)
constructs the final complete source-object identity from independently owned
original bytes. It accepts only the eight frozen kinds, including `go_file`,
and rejects preceding candidate tags. Strict import checks all seven members,
exact original data and identity, integer/null/Boolean distinction, normalized
logical paths and file/overlay byte boundaries. Returned metadata/byte copies
cannot mutate the stored source. This supplies no source/compiler/module/proof
authority and does not activate the public profile.

[Targeted verification](../migrations/go-practical/t02-w01/verification.json)
passes eight root tests and26 subtests in4.302 seconds, package vet in0.929
seconds and changed-file format in0.014 seconds with pinned Go1.25.0. Existing
shared canonical/strict JSON and public-profile argument consumers are selected
because the new component calls those codecs and must preserve public admission.
A first shared-cache setup failure happens before tests; isolated writable
caches pass with network/toolchain fallback disabled and vendored dependencies.
No C# tests execute. No full gate is run during this intermediate W; the
cumulative T02 gate remains deferred to W04 as required. W01 remains In progress.

The unchanged strict source reviewer rejects two obsolete-kind literals in the
initial negative tests. The corrected conformance test reads the exact original
negative source object specified by the final frozen catalog and checks the
fixture SHA before rejecting both its kind and complete record. Historical
graphs stay unchanged. Final targeted tests/vet/format pass. Three exact failed
diagnostics are sealed by path/hash/size/class/source epoch under the existing
historical policy; all127 matcher definitions/four scopes and36 previous seals
remain. Twelve altered-seal controls, existing fixtures and scanner shell syntax
pass. No production/directory exemption is added. Original diagnostics remain
byte-identical. Disk pressure is resolved by deleting only24GiB of ignored Rust
incremental cache; sources, executables and verification records stay retained.

## 80. T02-W01 rooted filesystem capture and owned snapshots

The [rooted capture checkpoint](../migrations/go-practical/t02-w01/verification/rooted-capture/README.md)
starts from `2de5a51de7aaa4a9f639eaf4133b5dc6de332713`. Actual file reads bind
explicit directory capabilities, exact original namespace spelling and stable
descriptor identities. Go1.25 rooted/nonblocking opens reject outside/inside
links, FIFO/file replacements and special inputs. No recursive scan rejects
unrelated application assets. All frozen capture budgets aggregate across
capabilities, repeated reads are not double-charged and errors prevent sealing.
Finalization checks original and owned bytes and observed namespace changes;
snapshots return copies in deterministic source-ID order. Original source files,
replacement directives and unsupported unused function bodies remain complete.

The first selection exposes missing nested test-fixture directories. After that
setup fix, a focused regression reproduces case-alias admission on a simulated
case-insensitive host; exact captured name membership corrects it. Both original
diagnostics/code are retained. The [final results](../migrations/go-practical/t02-w01/verification/rooted-capture/verification.json)
pass20 root tests and43 subtests in2.184 seconds, vet in0.398 seconds and format
in0.007 seconds. Existing raw-buffer, snapshot, shared JSON/hash and closed public
argument consumers are selected because the new executor uses those boundaries.
A further source review reproduces rejection of unrelated unread native names
containing a newline or backslash. The original failing logs and inputs remain
retained. Private directory observations now retain native names while requested
source paths still require exact spelling and strict logical-path validation.
The positive test also requires every expected original input and role, including
zero-byte `go.sum`. All8 affected rooted tests/17 subtests pass in2.083 seconds,
vet in0.459 seconds and format in0.007 seconds. Unchanged source-object/shared
JSON/prior-snapshot tests keep their passing20-test baseline; they are not rerun.
The current Go1.23 build passes3 root tests/11 subtests in2.034 seconds and requires unavailable
rooted capture to fail; it grants no practical-profile compiler/target support.

This is original-byte capture, not a registered source/program proof or native
Linux namespace-isolation claim. The design-stage context fixture is not promoted
to a finished checked foundation. Private context/foundation authority, actual
loader namespace materialization, complete module/replacement/archive/sum/export
metadata and compiler inventory remain required under the full W01 matrix.
All169 Go demands remain unelaborated. C# tests are deferred by the user; the
whole T02 development gate remains deferred to W04, with no intermediate full
gate run. T02-W01 is still In progress and the complete goal remains open.

## 81. T02-W01 original module inputs and replacement capabilities

The [module input checkpoint](../migrations/go-practical/t02-w01/verification/module-inputs/README.md)
starts from `770e0f17bf8510cfb5a977562de57665503d231f`. Original complete manifests
retain require/exclude/retract/local and versioned replacements, comments and
byte spelling. Full module/compiler patch requirements are checked before
normalizing proof language minors. Type-check-only dependencies can use older
versions; their toolchain suggestions cannot override the selected main compiler.

Strict original Go sums reject malformed/noncanonical/conflicting identities.
Exact `go.mod` Hash1 verification matches the retained independent Go1.25 vector;
missing/stale bytes and archive-only sums reject. Local replacements bind the
most specific explicit capability, with root-prefix/link/ambiguity/owner guards.
Original bytes are reparsed before following a directive, so mutable parser
metadata cannot redirect an input read. Go retains responsibility for version
selection. The actual pinned offline Go graph agrees with original main and
local replacement manifests; all4 original input buffers seal unchanged.

The current target selection passes12 root tests/52 subtests in2.217 seconds,
vet in0.461 seconds and format in0.006 seconds. The independent legacy development
build passes7 roots/40 subtests in2.181 seconds. It adds no practical compiler or
target. Original full-package type checking, checked private foundation/context,
compiler/library inventory, archive/content and full graph binding, immutable
loader materialization and export/build metadata remain due. W01 stays In
progress. C# tests remain deferred; the cumulative T02 gate is deferred to W04.
