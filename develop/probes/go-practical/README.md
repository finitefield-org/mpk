# Go Practical Feasibility Probes

These are private feasibility/audit tools. They register no profile, add no
public command, and cannot establish a program proof by themselves.

## T01-W01 baseline audit

From the repository root:

```sh
python3 develop/probes/go-practical/entry_audit.py --check
python3 -B -m unittest discover -s develop/probes/go-practical -p 'test_entry_audit.py' -v
```

The audit checks the last installed-release receipt against the raw registered
bytes, pins its Go toolchain metadata and specification witnesses, checks the
fixed Go corpus's indexed artifact hashes, and inventories initial consumers
and regression files. It does not execute the Linux compiler or verify all
application theorems. Its historical receipt does not certify later source
changes. The ledger records the separate clean Git baseline check; the audit
does not invoke Git.

`--write` deliberately regenerates the report only while establishing W01.
After production changes, retain the original hashes and compare against a
checkout of the recorded baseline; do not rewrite them to make drift pass.
Tests corrupt receipt state, registry identity, target width, language
versions, artifact digests, negative-case coverage, and duplicate JSON keys.

The proposed effective-language-version table is go1.22–go1.25. T01-W02 must
probe it using the exact Go 1.25.0 analysis release before T01-W04 can freeze
admission. Host Go 1.23.2 regression results are explicitly separate evidence.

## T01-W02 module, version and closure probes

Preparation is explicit and separate from measurement. The pinned host
archive identity comes from the [official Go download metadata](https://go.dev/dl/?mode=json&include=all).
The private host pin is not an installed Linux release tuple.

```sh
python3 -B develop/probes/go-practical/provision_host_toolchain.py
python3 -B develop/probes/go-practical/run_module_probes.py --check
python3 -B develop/probes/go-practical/check_module_report.py
python3 -B -m unittest discover -s develop/probes/go-practical -p 'test_module_report.py' -v
```

The preparation command may download the exact pinned Go 1.25.0 archive,
checks its SHA-256 and every extracted file, and keeps it under `/private/tmp`.
The measurement command performs no downloads. It builds the private analyzer
from the pinned vendored analysis libraries, prepares a synthetic local-file
dependency, seals application/dependency inputs, and evaluates every case
twice from fresh roots with Go downloads disabled. Cases that pass preflight
load the original package; metadata rejections stop before loading. It records 39
cases: 20 closed early-exclusion closures, 17 explicit rejections, and two
original source/module errors.

`--write` updates retained evidence only for an intentional reviewed change.
`--probe --case CASE_ID` runs selected debug cases without overwriting the
complete report. The report checker verifies retained source pins, original
fixture identity, case coverage and evidence scope without repeating Go loads.
Its adversarial tests reject missing cases, changed source, lost selected
bodies, host/target confusion and invented theorem claims.

See the [measured matrix](../../migrations/go-practical/t01-w02/module-matrix.md)
and [snapshot contract](../../migrations/go-practical/t01-w02/offline-snapshot-contract.md).
These are compiler/closure observations, not VIR/VC generation, an application
proof, or an OS-level Linux isolation receipt. W03 owns the next actual
ordinary-encoding and both-checker feasibility work.

## T01-W03 ordinary proof pilot (in progress)

```sh
python3 -B develop/probes/go-practical/run_proof_pilot.py --check
python3 -B develop/probes/go-practical/check_proof_report.py
python3 -B develop/probes/go-practical/run_proof_pilot.py --natural --check
python3 -B develop/probes/go-practical/check_proof_report.py --natural
python3 -B -m unittest discover -s develop/probes/go-practical -p 'test_proof_report.py' -v
CARGO_TARGET_DIR=/private/tmp/mpk-go-practical-proof-target cargo clippy --offline --locked --manifest-path develop/probes/go-practical/certificates/Cargo.toml -- -D warnings
```

The runner builds a private Rust candidate producer/verifier and the separate
production Go reference verifier, verifies the pinned host compiler tree,
constructs byte-identical certificates twice and runs both source-free
checkers on every candidate. It downloads nothing. `--write` is reserved for
an intentional reviewed evidence update. The fast retained-report check
validates evidence integrity; only the replay command executes the checkers.

The first 27 candidates cover full-payload nominal/result/error projections,
abrupt-control frame properties, a three-input carry lemma, and symbolic
backing/cell updates at all proposed v1 string/slice workload capacities. The
store and value payloads are universally quantified. Only finite addresses
are split into cases; complete collection payloads are never enumerated.
This pilot is **not complete W03 feasibility**, a complete signed-integer or
mathematical-sum bridge, a full operation library, a Go application proof, or
public-profile admission. Retained evidence and remaining requirements are in
[the W03 checkpoint](../../migrations/go-practical/t01-w03/README.md).

The separate four-candidate `--natural` measurement adds genuine `Std.Nat`
value conservation for one full-adder stage, generalized conservation for
every unary natural-number step, and two dropped-carry mutations. It uses
ordinary constructor/equality proofs, with an arbitrary initial natural
number. It does not complete the signed 64-bit, overflow or prefix-sum
bridge. Its report retains that pending scope explicitly.

All accepted candidates have zero axioms, no imported proof premise and no
theory certificate. A reused archived C# file is only an untrusted term
encoder; this pilot does not rely on unfinished C# proof status. Initial
candidate failures are retained separately. Their correction supplies
explicit checked congruence/transitivity steps and changes neither checker.

Resource measurements use Darwin `os.wait4` usage for each direct child
(RSS is bytes), wall time and CPU time. Hardware model/RAM are unavailable
inside the sandbox and remain explicitly unknown. A construction/checker
timeout is an unresolved resource finding, never a semantic rejection or
permission to declare the profile supported. No whole T gate runs here;
`scripts/check-fast.sh` remains due in T01-W05.

## T01-W03 compositional word diagnostics (in progress)

```sh
python3 -B develop/probes/go-practical/check_word_checkpoint.py
python3 -B -m unittest discover -s develop/probes/go-practical -p 'test_word_checkpoint.py' -v
```

The private Rust binary also accepts `word-generate REPO_ROOT OUTPUT WIDTH`
for widths 1–72. Its ordinary function-equality library composes the local
full adder into a universal unsigned word/natural-value theorem, retaining
the final carry. The current prototype emits the positive claim, a dropped
carry and a low-result-bit mutation. It has no public frontend route.

The [retained diagnostics](../../migrations/go-practical/t01-w03/word-initial/README.md)
include actual zero-axiom acceptance of the 64-bit claim by Rust and Go,
plus the 32-bit dropped-carry fuel failure. Source snapshots and binary
identities distinguish those historical runs from later experiments. The
retained-checkpoint command checks evidence integrity only; it does not
execute a proof replay or certify the incomplete matrix.

The initial 64-bit check costs about 198 s in Rust and 476 s in Go. A full
application budget cannot be inferred from that expensive partial proof.
The next candidate uses the existing Bool-addressed cube and a balanced
low/high split to reduce binder/lemma work while retaining the complete
input domain. It still requires actual both-checker and resource evidence.
Signed arithmetic, exact prefix sums, operation/source bindings and full
W03 closure remain pending. No checker source or logical rule is changed.

## T01-W03 balanced word bridges (in progress)

```sh
python3 -B develop/probes/go-practical/run_balanced_probes.py --check
python3 -B develop/probes/go-practical/check_balanced_report.py
python3 -B -m unittest discover -s develop/probes/go-practical -p 'test_balanced_report.py' -v
```

The [balanced model and evidence](../../migrations/go-practical/t01-w03/balanced-words/README.md)
cover the complete operand space through Bool-addressed cubes. Half-word
composition plus checked projection/carry lemmas replaces the long operand
telescope without changing either checker. The runner measures 48 cases at
widths 1/2/4/8/16/32/64/128, produces identical bytes twice and executes both
source-free checkers. All 16 positive claims are zero-axiom. Type-mismatch
negatives and candidate fuel failures have separate recorded classifications.
`--write` is only for an intentional reviewed evidence update.

Unsigned claims retain final carry. Signed claims compare natural cross sums
for the two's-complement interpretation, under the actual sign-stage carry
agreement. They do not prove a Go application establishes that guard. The
128-bit cases probe an internal widened domain, not a new Go source type.
Exact prefix sums, Go operation/source binding, collection/control closure
and aggregate/boundary budgets remain open. W03 is not complete.

Private debug commands are `tree-generate` / `tree-trace` and
`signed-generate` / `signed-trace`, each taking `REPO_ROOT OUTPUT DEPTH`;
depth 0–7 represents 1–128 bits. Trace mode retains intermediate certificates.
The archived failed producers belong to their original observations and must
not be used as the current producer or silently rewritten. Full T validation
remains due in T01-W05.

## T01-W03 exact unsigned prefix folds (in progress)

```sh
python3 -B develop/probes/go-practical/run_prefix_probes.py --check
python3 -B develop/probes/go-practical/check_prefix_report.py
python3 -B -m unittest discover -s develop/probes/go-practical -p 'test_prefix_report.py' -v
```

The [prefix model and evidence](../../migrations/go-practical/t01-w03/prefix-sums/README.md)
measure 55 candidates at counts `0/1/3/8/32/127/128/129`. The fold threads
actual modular64 accumulator states. One theorem uses each actual step's
zero-carry condition; a second unconditionally retains all actual overflow
corrections. Both relate to an exact `Std.Nat` sum with universally quantified
payloads, padding, accumulator and initial natural. Every candidate is
generated identically twice and checked by Rust and Go on the same bytes.

Eighteen zero-axiom claims pass, including valid empty-prefix controls;
23 mutations reject at core type checking. Fourteen last-summand/correction
removals exhaust conversion fuel and count only as resource findings.
The checker preserves the original positive fuel failure using its source
snapshots. `--write` intentionally refreshes retained measurement evidence;
the quick report check validates integrity without executing proofs.

Debug commands `fold-generate` and `fold-trace` take `REPO_ROOT OUTPUT COUNT`
through the internal count129 boundary probe. They do not expose a Go
frontend or freeze a size limit. Signed/nonnegative prefix interpretation,
source guard/order bindings, remaining operation/collection/control laws and
aggregate resource freeze remain due. W03 is In progress; the complete T01
gate remains deferred to W05.

## T01-W03 conditional signed prefix folds (in progress)

```sh
python3 -B develop/probes/go-practical/run_signed_prefix_probes.py --check
python3 -B develop/probes/go-practical/check_signed_prefix_report.py
python3 -B -m unittest discover -s develop/probes/go-practical -p 'test_signed_prefix_report.py' -v
```

The [signed prefix model and evidence](../../migrations/go-practical/t01-w03/signed-prefix-sums/README.md)
measure 32 candidates at the same eight prefix counts, with all 64-bit values
and backing padding quantified. A checked balance composes actual unsigned
overflow corrections with actual operand/result signs. It proves exact
two's-complement cross sums under every actual sign-stage carry guard.
Both generations are identical; nine zero-axiom claims pass Rust and Go,
15 altered claims fail at core type checking, and eight sign-removal
candidates reach conversion fuel and remain resource findings.

The shared prefix measurement runner keeps independent schemas, theorem
validators, commands and retained evidence for each mode. Debug commands
`signed-fold-generate` / `signed-fold-trace` take `REPO_ROOT OUTPUT COUNT`.
No checker rule, axiom, source width or public profile changes. Go source
guard/order/subtraction discharge and the nonnegative `Quote` invariant,
remaining collection/operation/control laws and aggregate resource freeze
remain due. W03 stays In progress; the full T01 gate stays deferred to W05.

## T01-W03 order/subtraction and continuing-path guards (in progress)

```sh
python3 -B develop/probes/go-practical/run_order_probes.py --check
python3 -B develop/probes/go-practical/check_order_report.py
python3 -B -m unittest discover -s develop/probes/go-practical -p 'test_order_report.py' -v
```

The [model and measured evidence](../../migrations/go-practical/t01-w03/order-guards/README.md)
cover 40 ordinary proof candidates. Carry composition spans widths 1–128;
source-shaped signed path and exact-Nat addition use actual Go widths
8/16/32/64. Each group is generated twice identically and checked by both
source-free verifiers. Sixteen zero-axiom claims pass; 24 altered
premise/operand bindings reject at core type checking. The runner rejects
duplicate/shadowed candidate IDs and mismatched file/index coverage.

The same command builds a pinned Go observation tool and compares 3,428
actual typed arithmetic/compare observations with an independent bit-loop
model. It retains exact input/output bytes and separates that empirical
scope from proof acceptance. Negative/noncontinuing inputs remain observed.
The quick retained-report check does not rerun proofs or Go execution;
`--write` is reserved for intentional reviewed evidence refreshes.

Debug commands `order-generate` / `order-trace` take `REPO_ROOT OUTPUT DEPTH`
with depth0–7. Width128 is an internal probe, not Go source admission.
The actual addition carry guard, nonnegative bounded result and exact sum
follow from source-shaped comparisons; production Go operator/branch and
whole-loop bindings remain due. W03 is In progress. Aggregate resources and
remaining collection/control/byte laws stay open, with the full T01 gate
deferred to W05.
