# Explicit source reconstruction completions — partial W09 unit 4

This corpus pins 45 actual-source contexts and 37 ordinary binary definitions
with argument order `(semantic_value, source_completion)` and a source-carrier
result. It includes the existing forward projection definitions unchanged.
The schema is `mpk.csharp.ordinary_binding_rebuilds.v1`; canonical import
regenerates the program from validated VIR, construction and binding VCs.

For a bound source product, reconstruction replaces mapped active fields from
the semantic value and preserves every unmapped or inactive stored field from
the explicit completion. Source enum tags retain their original width and
signed values, including the remapped `Int64.MinValue` boundary tag. Recursive
sequences take their length from the semantic argument, rebuild each active
element with its corresponding completion cell, and zero inactive output
storage. Generic sum reconstruction selects the semantic arm, recursively
rebuilds its payload and clears padding. Identity conversion returns the
semantic argument. A completion is an input; it is never a trusted default.

These definitions do not supply unary `Reconstruct` witnesses. The manifest
retains all 35 nonidentity witness occurrences as pending; only the two existing
identity witnesses are already defined. Source invariants, admissible completion
selection, native constructors, both universal round trips and application
proofs remain required. An arbitrary varying unmapped field cannot be recovered
from a semantic value alone. Finite observations and well-typed definitions
cannot establish that missing inverse.

Generation tests check all 45 contexts, exact metadata, the forward definitions'
transitive closures, pinned bytes and metadata/hash/cross-context mutations.
The independent source oracle uses captured member declarations, binding maps
and closed arguments. Initial 156 observations passed. Direct review added
111 different-semantic-input observations so that returning the completion
unchanged cannot satisfy the test. Explicit test completions cover newly active
payloads. Two additional raw-storage cases change the completion sequence length
to zero and `u32::MAX` while retaining its cells; output length must still come
from the semantic input. These cases exercise the binary helper outside the
completion representation domain and do not claim that the completion is a
valid source value. Output checks include prior nonzero cells to catch stale
values when a semantic sequence shrinks.

`TestCheckerAgreementWithRustCLIBindingRebuilds` requires the `checkeragreement`
Go build tag and checks all 45 pinned byte sequences with both unchanged
checkers, zero axioms and rejected hash corruptions. Terminal results, failed
attempts and running jobs are recorded in `../unit-4-binding-rebuild-progress.json`.
The strengthened suite passed with 267 source observations, 63 changed outputs,
two raw completion-length cases and 32,331 storage-bit observations. All 45
same-byte checker cases passed in 166.916 seconds. Exact pinned replay, importer
mutations, lint, inventory and format checks passed. The maximum certificate
contains 1,616 terms and 62 declarations.

No uncompleted test is counted as passed. All remaining internal units 3–8 and
the original W09 acceptance requirements remain open. The full repository gate
is deferred to T06-W12.
