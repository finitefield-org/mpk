# Strict original standard-module facts

Status: scoped checks passed. Go/Rust only; T02-W01 remains open and the
practical profile remains inactive.

The original Go SDK owner uses modfile.Parse, while the independent Rust SDK
index previously used the dependency ParseLax projection. A source such as an
unknown directive or a malformed tool/godebug row could therefore survive that
projection. The SDK index now uses independent general strict Parse facts and
retains the whole original manifest. Standard-library binding separately requires
module std, an explicit Go directive and no newer compiler requirement, matching
the existing native check. The existing application-admission reader and dependency
ParseLax reader retain their own rules; general parse acceptance activates neither.

The new reader preserves optional/missing fields, exact Go/toolchain spelling,
canonical dependency versions, replacement/retraction rows, arbitrary string/path
bytes, ordered/duplicate requirements and all godebug/tool/ignore data. It keeps
nil-fixer retract spellings and the native version comparison rather than applying
an application's language range to SDK metadata. A toolchain suggestion remains
data and does not select or execute a compiler. No source, syntax-node, directive-
count, bundle, kernel or certificate budget is changed.

The [native observations](native/observations/complete-original-strict-modules.json)
retain1427 complete cases: all656 earlier source manifests,656 standard-path
variants,18 real raw-SDK manifests and97 extra version/directive/byte cases. The
extra corpus includes40000 duplicate debug rows and its original suffix. Rust
compares every parsed field and the separate standard-library decision. Existing
application/lax preservation cases are selected too. The Go standard-library
predicate is extracted into a helper with the same existing behavior, and its
actual original source/type-check consumer with no filesystem fallback passes.

Four controls change the actual original SDK go.mod bytes while retaining the
complete5577-file SDK and182-package/5697-object application. They rebuild all
original source/package/declaration/type/root identity links, component inventories
and distribution hashes, contract/selection/snapshot/capture/export/compiler links,
then successfully reimport complete original exports. The three unknown/tool/debug
malformations reject at StandardModule; valid complete additional directives remain
accepted. Every application Go body, unused body/init and Root contract byte stays
unchanged. This original fixture has1033 component sources,181 source package
keys,zero declaration keys andone builtin type key; all counts remain. No richer
declaration/type rekey coverage or source/program proof is inferred from these
controls. The rekeyer itself is adversarial test construction, not a production
source/provider/metadata adapter.

All20 affected Rust roots, both native roots, strict affected Clippy, Go vet and
changed formats pass. The13 source epochs preserve11 passes and2 failures: the
initial ownership error in new test construction and the native-consumer driver's
initial missing evidence directory. Both are repaired without dropping cases or
changing production acceptance to satisfy the tests. Full original SDK/source/
package/type/error/proof data remain at their original paths/epochs, with previous
compiler/member/content/executable/source-role controls selected. Earlier32 proof
streams and43-root/zero-or70-error universes retain prior evidence without being
relabeled as current kernel replay.

This component establishes strict original SDK manifest facts. Remaining generic
executable-reader cases, installed compiler/root/capture/context/descriptor/registry/
launcher/read bindings, all required foundation/source/program proofs, native Linux
release gates and remaining T02–T11 stay due. R01/R03/R04/R05/R06/W01 remain open
and the profile inactive. No scanner rule/seal/exclusion, premise, mathematical
rule or frozen source/type/error/proof universe changes. Final T02-W04 owns the
cumulative `./scripts/check-fast.sh`; no whole-workspace gate or removed-language
test runs here. See the [verification driver](run.py), [native consumer driver](stdlib.py),
[inspection driver](inspect.py) and [scoped review driver](review.py).
