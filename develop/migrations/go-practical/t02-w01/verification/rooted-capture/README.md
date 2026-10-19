# Rooted source capture checkpoint

The new `go-tools/go2vir/practical_capture.go` reads original files from multiple
explicit directory capabilities. A source-object hash does not authorize a
filesystem root. The caller still owes registered context/root authority and
complete module/compiler binding. No practical profile is publicly activated.

Reads are on demand, so unrelated assets outside the loader's visited closure
are not recursively treated as source candidates. Every visited directory's
exact names/types and identity are retained. Source paths must use those exact
names, including on a case-insensitive filesystem. Root/ancestor/leaf symlinks,
special files, replaced roots and changed descriptors reject. Go1.25 `os.Root`
keeps resolution within the bound tree; nonblocking opens prevent a file-to-FIFO
swap from blocking before its descriptor type is checked. This is a file-input
component, not native Linux namespace/mount/resource enforcement evidence.

All file/input/byte/overlay/directory/entry counters are shared across roots.
An ignored error still prevents sealing. Seal rechecks the root/visited namespace
and complete source bytes, even if a source's mtime was restored. Owned buffer
corruption also rejects. The resulting snapshot retains independently owned
bytes, complete final `go_file` identities and deterministic source-ID order.
Returned arrays/bytes are copies; later filesystem edits do not change a sealed
snapshot. Handles are scoped to each operation, rather than held per dependency.

The [verification record](verification.json) binds current sources and exact
terminal logs. The initial affected-consumer selection passes20 root tests/43
subtests. Following the native-asset review fix, all8 affected rooted tests/17
subtests pass in2.083 seconds, vet in0.459 seconds and format in0.007 seconds.
The source-object, shared JSON and prior snapshot implementations were unchanged
after the20-test selection. Cases retain two original
modules, their replacement directive and an unsupported unused function without
rewriting or extracting source. They also cover deterministic leaf swaps,
observable namespace/content changes, case aliases, remaining-budget boundaries
across capabilities, immutable ownership and existing snapshot/JSON/public
argument consumers. These tests do not yet load or verify the module graph.

The [first run](initial/result.json) failed during fixture setup because nested
test directories were missing. The [focused case-alias regression](initial/namespace-alias-result.json)
then reproduced admission of non-original spelling; its original code/logs are
preserved. The final capture checks exact directory membership before lookup.

Source review also reproduces rejection of unread native asset names containing
a newline or backslash. The [original regression](initial/unvisited-assets-result.json)
and its inputs remain retained. Directory observations now keep native names;
only requested logical source paths receive source-path validation. The positive
case checks every original input and its kind, including zero-byte `go.sum`.

The separately recorded [Go1.23 build check](reviewed/legacy/result.json) passes
three root tests/eleven subtests. It checks existing development compilation,
closed public arguments and an explicit error for unavailable rooted capture.
It adds no supported practical-profile compiler/target. Both test routes disable
network/toolchain fallback and use isolated writable caches and vendored inputs.

W01 remains In progress: actual private context/foundation authority, immutable
loader namespace materialization, exact module/replacement/archive/sum/export
capture and compiler inventories remain required. Complete source/IR/application
proofs stay with their assigned T02–T06 gates. No C# tests or intermediate full
gate execute; the cumulative T02 gate remains deferred to W04.
