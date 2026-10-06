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
