# GO-PRAC T01-W05 inactive publication and cumulative gate

Entry: W04 completion commit `68a44416e1b1a6dc3a84587cca784fc4b3a2b862`.
Publication and targeted checks pass. The cumulative T01 gate is running;
T01 and W05 are not complete until its successful result is recorded here.

## Full-gate review fixes

The final [source-kind amendment](source-kind/README.md) uses go_file and
requires a new T02 source/artifact epoch. Its complete preimage agrees in the
independent Rust and Go engines, twice each. The old private candidate models,
their generated fixtures and failed source snapshots remain byte-identical.

The obsolete-interface scanner now uses equivalent literal prefilters, with
3,206 unchanged-result controls, all9 existing cases and the fixture manifest.
All127 matchers and4 scopes are unchanged. Its new fixture-manifest version
additionally seals36 individually reviewed historical files by exact path,
hash,size,class and source epoch. It preserves the12 existing retained-record
declarations and permits no directory/production-file exclusions. Ten changed
seal controls reject; [details and original findings](verification/gir-prefilter/README.md)
distinguish fixed history from current production. The complete scan passes.

The first completed workspace run then exposed a pre-existing C# T05-W01
input-epoch mismatch introduced by its approved later Boolean-proof refreeze.
[Original requests/responses](../../csharp-03/boundary-attachment/retained-t05-w01/README.md)
are restored from their original Git objects; old conformance and current
fixtures stay unchanged. The updated retained-evidence test pins both epochs,
compares complete source/contract bodies and native facts, and binds current
response digests to actual current request bytes. Its targeted test and all9
boundary-suite cases pass in the full retry. The remaining full gate is still
running; no partial attempt is counted as completion.

Five frozen inactive specifications own five new conformance vector sets:
[profile](../../../specs/GO_PRACTICAL_PROFILE_V1.md),
[foundation](../../../specs/GO_PRACTICAL_FOUNDATION_V1.md),
[shared artifacts](../../../specs/GO_PRACTICAL_SHARED_ARTIFACTS_V1.md),
[Nat induction](../../../specs/CORE_NAT_INDUCTION_V1.md) and
[Boolean type cases](../../../specs/CORE_BOOL_TYPE_CASES_V1.md).
The [publication audit](publication-audit.json) reconstructs their exact bytes
and manifest entries, verifies all255 immutable W04 pins and preserves all26
prior vector sets. The manifest now declares31 sets. Declared future Rust/Go
test owners remain implementation handoffs; the actual publication checker is
`scripts/check-go-practical-spec.sh`. No placeholder production test is added.

Normative semantic/schema data is incorporated through exact byte/size-pinned
W04 members. Historical draft-status descriptions do not reopen frozen design
decisions or imply finished production definitions. Observations retain their
original scope. The [source/feasibility review](source-feasibility-review.json)
records the independent source/IR interpretation, complete context-table/type
binding, nineteen required theorem families, measured ordinary feasibility and
all remaining implementation/application/native gates. All169 Go requests remain
unelaborated. Metadata publication grants no proof or installed profile route.

The [retained manifest snapshot](predecessor-spec-manifest.json) records the
unchanged predecessor26 entries. Existing inactive C# bytes/identities and
installed registries remain unchanged. Historical reconstruction also
[passes after publication](verification/historical-replay-after-publication.json)
from the original1,025-file Git epoch, without editing old helpers or pretending
the new source tree is byte-identical to the old one.

The [selected checks](verification/targeted-results.json) cover new publication,
manifest/schema ownership, byte pins and retained-vector preservation. This is
the last W in T01, so the full local `./scripts/check-fast.sh` is executed once
for cumulative T01 changes. Its [result](verification/check-fast-result.json)
and raw stdout/stderr distinguish completed validation from a running or failed
gate. The native Linux installed-release/resource/isolation/application gates
remain T07/T11 requirements; this macOS development gate cannot replace them.
