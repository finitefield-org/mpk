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
| GO-PRAC-T01-W03 | In progress | Halting Quote, byte/index, descriptor/store, computed validity/access, reslice and overlapping copy ingredients measured; production source/ownership/operation bindings, remaining laws and aggregate resources stay open |
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
