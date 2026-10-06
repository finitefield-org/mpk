# Approved Boolean proof elimination — implementation checkpoint

The user approved the generic Sort0-dependent cases rule on 2026-10-07.
Rust and Go now validate its complete canonical family/constructor/eliminator
interface, infer its dependent result, and apply its two iota equations.
Existing Boolean recursor declarations and equations remain unchanged. The
certificate format, proof-node tables, theory interfaces and axiom policy
remain unchanged.

Direct review found and fixed two registration/inference gaps. A third
constructor added after registering cases was accepted when the major stayed
neutral; the family now closes at registration. Extra universe arguments were
ignored when inference needed no iota equation; the new monomorphic interface
now rejects them during inference too. Both gaps were reproduced in both
executed predecessor binaries before fixing them. Their inputs, reports,
binary hashes and source manifests are preserved in the checkpoint receipt.

After the fixes, 64 current Rust tests and 16 selected Go top-level tests pass,
including the CSHARP-03-T01-W09 feasibility owner. Changed-package Clippy and
format checks pass. The earlier selected reduction, definitional-equality and
generation evidence is preserved separately; it is not counted as a fresh
run after the inference fix. Both standalone checkers agree on 19 same-byte
new certificates: six accepted zero-axiom proofs and thirteen core rejections.
Three predecessor certificates also retain acceptance and matching hashes.
The corpus includes universal right identity, both conjunction projections,
both constructor equations and open dependent motives.

The unchanged-core capability diagnostic separately completed on Linux at
exact public source `f02dcb32ee0ff420a86083ea95067879455dec81`; its ten checker
results agree with the pinned local observations. This is baseline evidence,
not Linux validation of the new rule. The new rule separately passes 86 Rust
tests, 16 selected Go top-level tests, lint/format and 44 dual-checker stages
on exact public Linux source `9755987798ec65b266035f1d0418fc7f2890182a`.
Its 1,088 source/fixture Git blobs, executed binaries, control inputs and
downloaded reports are independently audited. The renewed T01-W09 metadata
and affected private candidate consumers are being validated. T01-W10 and
original T06-W09 proof assembly remain blocked until that freeze. All 987
original application proof IDs remain pending and the practical profile remains inactive.

The receipt records why these tests were selected. No intermediate whole gate
was run: renewed T01's whole gate belongs to final W10; T06's belongs to W12.
Second-pass review checked de Bruijn indices, branch order, neutral behavior,
constructor closure, empty-universe enforcement and predecessor compatibility;
no further finding remains in this reviewed implementation scope.


The actual descriptor changes to
`99369543ab96e97e971118fe980322fe0b138f253a31ab7f8a9b08565bad0844` and
candidate revision 5; definitions and semantic template identities are retained.
Selected renewed freeze/registry/foundation checks pass 18 tests and the
replayed predecessor capacity/recursor probes retain all 108 checker results.
All 384 actual predecessor/current native-fact import/emission replays preserve
source and contract bodies. Pinned Linux Roslyn execution independently matches
all 384 final native responses; the three invalid parent-hash corrections have
separate local and native receipts. All 17 selected consumer/regeneration tests
pass, with one earlier failed snapshot selector run retained. Ten regenerated
VC/boundary corpora preserve semantic contents and counts after accounting for
actual context-bound digest changes. Their ten normal pinned checks, the focused shared JSON/collection library
fixture regression and affected VC/CLI lint/format pass. Remaining
ordinary-definition receipt lineage is still in progress. This review does
not close renewed W09/W10 or any of the 987 original application proofs.

The ordinary-consumer checkpoint now covers 29 actual source owner corpora
and 622 certificates: 607 exact byte retentions, fifteen validated context-name
graph substitutions with fresh dual-checker acceptance. Seven owners also
pass on public Linux source `40469d29`; all 234 generated files match Darwin
exactly and all 4,285 source/fixture Git blobs match the executed manifest.
The five JSON owners preserve nineteen certificate byte sets; boundary field
owner IDs are rebound only to actual canonical contract hash outputs. Failed
comparison records remain separate. A preexisting parser archive mismatch
from the earlier count-helper refresh is repaired by comparing the complete
current pin closure without normalization; the integer owner passes and the
remaining scalar/codec owner run is ongoing. Other ordinary metadata lineage
and all original application proofs remain pending.

The scalar/storage/codec follow-up now passes all ten affected owner corpora
and preserves all 190 certificate byte sets and semantic metadata. Both
parser owners compare complete current pinned definition closures; the initial
stale-archive failure is retained. Four new certificate-only test entries
share their original owner body; a Git-backed syntax audit verifies that every
original source/import/mutation and semantic observation remains exactly
unchanged apart from the explicit observation branch. The original full
semantic owners still execute all their value cases. Their separate ongoing
run is not included in this selected pass. The first owner-split compilation
failed because its selector shadowed an existing observation function; the
renamed selector's complete nine-owner rerun passes and both logs are retained.

The ordinary checkpoint totals 39 owner corpora and 812 certificates: 797
exact byte retentions and fifteen validated context-name graph changes with
fresh dual-checker acceptance. The actual predecessor/current 384-source
identity replay independently derives 251 distinct VIR hash rebindings, with
the same 266 emissions, 104 expected emission rejections and fourteen native
source rejections. See `ordinary-metadata-4` and `actual-vir-identities`.
Other ordinary consumer and alias lineage is pending; all 987 application
proof IDs remain pending. The final whole gates remain T01-W10 and T06-W12.

Six normal scalar/codec pin consumers pass after promotion. The selected
aggregate regrouping mutation audit independently finds the same stale
count-helper archive dependency; its failed log remains retained. Its repaired
baseline first compares all retained aggregate types and non-pipeline bodies,
then copies only the independent historical pipeline tree into the current
producer-pinned declaration graph. `same_aggregate_scan` remains unchanged
and still validates the exact 8,192-step order plus every baseline declaration
closure; all shortened-group, changed-helper and changed-argument mutations
reject. The actual current aggregate producer pin, repaired mutation audit,
affected VC-test Clippy and format checks pass. Second review confirms the
copy preserves term/argument/binder/level order, resolves globals only by their
unchanged names and leaves the reviewed count-helper/body and original archive
intact. See `ordinary-checkpoint-review/receipt.json`. This remains an
intermediate checkpoint; other ordinary consumer metadata and all original
proofs are pending, and whole gates are deferred to final T01-W10/T06-W12.


The following intermediate linkage checkpoint promotes only thirteen completed
source owners: binding guards/orders plus string/data/ownership/fold corpora.
All 146 certificate byte sets are preserved; 70 metadata files exactly match
the successful original-source outputs. A consumer dependency failure in the
next integrated control owner is retained rather than counted as a pass. Its
full log is stored losslessly and the current control-edge source owner runs
before retry. Post-promotion pins are required by the separate checkpoint
receipt. Remaining source owners, changed contract/control names and nested
JSON wrapper linkage remain under validation. All 987 original application
proof IDs remain pending; W09/W10 are not complete.

The thirteen normal owner pin tests and format check pass after promotion.
The first pin run failed during compilation with ENOSPC and executed no test;
its exit 101 and log are retained separately from the successful retry.
Only unused incremental build caches were removed to recover disk capacity.
Source and fixture manifests match before and after both runs.
Whole gates remain deferred to final T01-W10 and T06-W12.


Twelve additional original source owner corpora now pass: the ten remaining
binding reconstruction, collection, literal, transition, default and boundary
owners, plus control edges and baseline control predicates. Their 186
certificate sets comprise 172 exact byte retentions and fourteen complete
context-name graph rebindings with fresh matching zero-axiom acceptance from
both checkers. The checkpoint totals 64 owner corpora and 1,144 certificates:
1,115 exact byte retentions and 29 validated name rebindings. Ten overlapping
JSON token/parser aliases also reconcile through their actual original-source
producers; their definition bytes stay unchanged and are not added to corpus
totals. Exact output-file checks verify every promoted file against the passed
source-owner outputs on identical production sources.

The same eighteen original source-frame cases pass after correcting the
expected equality frame to exclude the array receiver slot changed by its
memory effect. All original value and mutation observations remain. The
selected C# practical VC integration-test Clippy and format checks pass. The
initial source-frame failure and the copied runner's test-selection correction
are retained with real exit codes; no incomplete child test is counted as a
pass. The corrected runner passes the baseline control pin and integrated
execution source owner. The next pattern-scope owner detects the not-yet-
promoted execution dependency and fails 101; the execution lineage and changed
certificate acceptance are checked before that dependency is promoted.

See `ordinary-control-and-phase6-checkpoint/receipt.json` for exact selection,
completed stages, predecessor bytes and manifests. Remaining integrated
control dependencies, standalone/wrapper changed certificates and relation
metrics are still under validation. All 987 original application proof IDs
remain pending. W09/W10 are not complete; whole gates remain final T01-W10
and T06-W12.
