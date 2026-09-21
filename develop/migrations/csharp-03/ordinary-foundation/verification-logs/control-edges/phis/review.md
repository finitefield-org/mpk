# Native phi incoming relations — review

The W04 edges and original validated native block phis determine each exact
predecessor operand and target value. Phi relations reuse the compiled edge
guard, retain both endpoint node IDs and the incoming edge ID, and copy the
complete native carrier. There is no source-slot/native-value type coercion.
All inputs precede all outputs semantically; a backedge target is an incoming
snapshot, not the preceding iteration entry. A missing guard leaves both slot
transport and phi relations absent. Native execution, selected-edge/node-entry
merging and application proofs remain outstanding.

The native array investigation found an additional obligation, not an implicit
frame law: index_update local:0 has a published sequence source slot but a
private-construction native SSA representation. Source updates evaluate to the
assigned element and separately produce memory SSA versions. A source load can
retain the allocation identity while native memory operands advance. A generic
non-store frame over the public sequence slot would therefore be unsound.
Nullable pattern bindings can likewise change representation and span multiple
native blocks. The existing six slot fixtures do not prove these projections.
These cases must be explicitly modeled before claiming complete node frames.

Verification selection: regenerate and execute all seventeen edge sources,
because native phis appear in loops, conditional expressions, pattern joins and
private memory. Check exact native predecessor/type bindings, matching and
mismatched values, high physical bits, disabled guards, original W04 slot goals,
metadata/certificate mutations and the remaining unsupported exception guard.
The guard, slot and other data emitters are unchanged. Check changed certificate
bytes with both unchanged kernels and reject hash corruption. Full repository
gate remains deferred to T06-W12.

The preservation check decodes the predecessor and extended certificates and
compares the full old term/level prefix, every declaration with canonical name
remapping, module/import identities, and empty proof/theory tables. Removing the
new phi metadata and restoring the old certificate hash gives exactly the old
program metadata. A review tightened the check to require at least as many
declarations, preventing zip truncation from hiding an omitted old declaration.

An initial build used the wrong module path for PracticalVirFunction; the
correct validation-module path compiled and all runtime tests passed. A final
regeneration command mistakenly combined artifact-writing tests and the
artifact-reading preservation test in one run; the reader ran first and failed
because the new directory was empty. Generation and preservation now run as
separate commands. Neither failure is a semantic acceptance receipt.

A second review strengthened the phi input patterns: Boolean inputs alternate
false/true, scalar/cube inputs use distinct ordinal markers, and corruption
flips an actual low or high bit rather than replacing all inputs with a common
zero. This distinguishes simultaneous copies with different values and avoids
a no-op corruption for small carriers. The affected runtime suite was rerun
with these stronger inputs; candidate bytes are compared to the same checked
certificates before promotion.

Final result: all 17 identical candidate byte sequences passed Go and Rust,
report agreement, zero-axiom checks and hash-corruption rejection (68 successful
stages). The reviewed runtime suite passed 38,059 observations in 124.72 seconds;
regeneration exactly matched those checked candidate files. Preservation, lint
and format checks passed. There are 62 phi joins and 100 selected phi values.
No additional defect was found in this scoped relation component. The native
representation audit records six mismatched source/native transfers across
index_update and type. They remain explicit implementation obligations, along
with node entry/execution/application composition; unit 5 and W09 are not done.
