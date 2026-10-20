# Original namespace reconciliation and owned verified image views

Entry: c9e105f340d46562c7c8c9c14934175359fc7225. This is private
T02-W01 work. R06 and W01 remain pending.

The [namespace composer](../../../../../../go-tools/go2vir/practical_namespace.go)
compares knowledge of original physical names across every authorized root.
Overlapping complete directory observations must agree on all names and native
types. Captured aliases must agree on original file role, full raw digest and
length. Ancestors must retain directory type. A separately authorized nested
root may refine an unvisited directory; an observed absence cannot be replaced
by a different capability's invented name. Compatible aliases preserve their
separate original source identities and original root bindings.

The initial regression demonstrates the actual old defect: after a second
capture added later.go under the same physical root, the image builder accepted
the union despite contradictory complete directory listings. The
[failed epoch](original-contradiction/regression/result.json) retains the full
two original inputs, source epochs and actual accepted image observation.
The [first fixed epoch](reconciled/regression/result.json) and
[current checks](sealed-modes/selected/result.json) reject it. The original
failure is a namespace-composition defect, not a semantic source counterexample.

The [owned view](../../../../../../go-tools/go2vir/practical_view.go) opens a
rooted reader only after checking the entire expected materialized closure:
every directory and file, exact original length/digest, original executable
mode and read-only permissions. Missing, extra, linked, special, writable or
changed backing rejects. The complete original per-capability namespace and
captured source objects have separately owned metadata. A nested capability's
captured bytes cannot supply an outer capability's unvisited source.
Only original observed absence returns the absence result. Missing backing
never acquires that authority. Invalid lookup, role, not-captured or changed
input makes the transaction's failure sticky, including after byte restoration.

Live views prevent image cleanup. Closing a view or image is idempotent;
closed ownership cannot be reopened. The Go1.25
[cleanup implementation](../../../../../../go-tools/go2vir/practical_cleanup_go125.go)
keeps permission changes and removals beneath an os.Root, changes directory
permissions only and preserves outside symlink-target bytes and mode. The
Go1.23 build retains the explicit unavailable boundary and compiles without
executing practical Go tests.

All30 affected roots/192 subtests pass in120.65s, along with current vet/format
and Go1.23 compile/link with zero tests. Five namespace contradictions and six
backing mutations reject; compatible aliases, unvisited nested refinement,
original absence, copied metadata, sticky failures and lifetime checks pass.
The original mode regression retains four Darwin setup failures where requested
setuid/setgid flags did not survive the backing filesystem, and two genuine
sticky-bit acceptances. On Linux tmpfs all six real file/directory setuid,
setgid and sticky controls [fail before](linux-special-permission/before/result.json)
and [pass after](linux-special-permission/after/result.json) the fix, with no
skipped controls. Darwin explicitly records four unmaterialized controls and
passes both sticky cases. An opened view also rejects source mode mutation;
restoring permissions cannot erase its failure.

The original [format diagnostic](sealed-modes/format/result.json) retains one
test map-value alignment. Its [complete before source and exact formatter
comparison](final-format/format-correction.json) establish the formatting-only
change; every production source and all frontend build inputs remain exact.
The [two affected roots](final-format/permission/result.json), current vet and
format pass after correction. The prior full source epochs remain exact.

Two independent empty-cache frontend builds from180 original inputs,
44 frontend/133 vendor files plus metadata, agree on8592898 bytes and SHA256
5158c73313b8928b980961e6337fd6a1e0f905b595c06f2f444b3f571b368216.

The actual Linux producer opens the new verified view over5590 materialized
files and6441 reconciled original namespace nodes, including the full5577-file
compiler inventory. It preserves5493 original source objects, all181 whole
source-typed packages and1472495 syntax nodes. Original relative, absolute and
versioned replacements, unused HTTP handler and panic initializer remain byte
exact. The initializer and application are compiled without execution.
Two fresh independent kernel-readonly replays reproduce all4 original module
objects, all182 original package objects except the derived export-cache locator,
and all181 complete original archives/79406918 bytes per replay. The
[first comparison](sealed-modes/linux-replay-1/comparison.json) and
[second comparison](sealed-modes/linux-replay-2/comparison.json) retain complete
archive bytes, build IDs and native outputs. The archive total belongs to this
producer's original paths and is checked against its own complete originals.

The local offline harness is byte-identical to the
[prior kernel harness](../replay-image/README.md). Its three readonly-input,
hidden-SDK and missing-source negative controls are retained without repetition.
Both new executions have separate empty caches, original path mounts, hidden
ambient SDK, UID501, dropped capabilities and no new privileges. The pinned
Go1.25.0 Linux/amd64 binary runs emulated on local Linux/aarch64. These are
development results; native Linux release acceptance remains outstanding.

[Independent reconstruction](verification.json) checks22668 complete original
source-object preimages and131 additional complete namespace/view payloads,
the exact entire image closure, original snapshot/image framing, all5577 copied
compiler files and all255 unchanged frozen members. The initial independent observer's130
payload-count assumption is retained and corrected to the complete actual131
payloads, including all109 canonical source-object views. Original records remain
byte-exact; nested fixture payloads also have lossless independently decoded
views. [Actual handle observations](terminal-handles.json) confirm all22
drivers' terminal results and the12 removed/exited containers before staging.
Strict changed-input/staged/committed-blob checks retain127 rules/four scopes/
39 seals, scan every changed file and decoded payload, and add no exclusions.

Production kernel lifecycle/private context ownership and complete general
consulted namespace/selected-closure bindings remain due. The view establishes
an owned reader transaction and does not register a context. Full56-family
source/carrier/nominal/type/operation/storage/lifetime/state/program foundation,
all six common members/19 theorem families, compiler-derived complete operation/
VC roots/private descriptor/registry/context/retained policies and all169 demands/
T02-T11 remain required. C# tests stay skipped. The cumulative T02 gate stays
W04: ./scripts/check-fast.sh --skip-csharp. No public profile or whole source/
program/application proof activates.
