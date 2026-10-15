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
| GO-PRAC-T01-W02 | Ready | Original-package/dependency/build/version/closure probes |
| GO-PRAC-T01-W03 | Blocked by W02 | Ordinary encodings, conditional lemmas and resource feasibility |
| GO-PRAC-T01-W04 | Blocked by W03 | Exact v1 source/shared/contract/limit freeze |
| GO-PRAC-T01-W05 | Blocked by W04 | Publish inactive package, review and full T01 gate |
| GO-PRAC-T02–T07 | Not started | Sequential v1 implementation and public release |
| GO-PRAC-T08–T11 | Not started | Sequential v2 freeze, maps, pointers and public release |

No T is complete. `scripts/check-fast.sh` is due in T01-W05, not W01.

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
