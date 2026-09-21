# Option SSA adapter review

The native nullable emission already maps to frozen option foundation IDs.
The adapter validates the complete closed operation signature before registering
result or failure symbols. It reuses the existing outcome emitter with one
owning builder; no new evaluator primitive, axiom or checker rule is introduced.
All other W03 definition families remain explicit pending IDs. The original
W03 operation records, subject order and normal/exceptional edges are preserved.

Computed output is bound once. Within that Let, the computed value is variable
0 and the actual result is variable 1. The comparator checks the full physical
carrier, preserving representation details. Failure wrappers project the frozen
argument indices in the original operand order and accept only the frozen
invalid-operation exception. No tagged failure result is manufactured.

Original-source cases cover none/some, HasValue, active and absent Value,
explicit/default fallbacks, bool/integer/floating/decimal payloads and source
structs, including an option containing a struct containing nullable string.
The independent OutcomeModel supplies expected values; ordinary terms alone
produce the observed results. Tests mutate first/last physical result bits,
source/foundation/data/certificate identities, relation/failure symbols, original
function/node/normal-successor metadata and actual certificate bytes. Import
independently regenerates both canonical artifacts and rejects substitutions.
The source closure comparison catches missing/extra definitions, SSA points or
pending IDs. Unknown runtime selectors fail instead of passing zero cases.

Review caught an oracle assumption before runtime verification: W03 emits data
relations only for invoked signatures, so the `some` constructor used to obtain
payload types must come from the full frozen outcome closure, not the invoked
subset. Native equality has its own lifted adapter and is not claimed here.
Existing sixteen outcome and seven source-value certificate corpora regenerate
byte-identically after this additive wrapper and comparator visibility change.

All twelve new certificate pairs passed Rust/Go checks. The user completed the
remaining nested-string value-or partition in 235.32 test seconds, bringing the
scoped runtime corpus to twelve contexts and 177 result observations. Import
verified the original manifest, source/fixture hashes and log without rerunning
tests. The earlier bounded timeout is retained as diagnostic history.
No whole internal unit is marked complete or committed. Native control flow,
ownership, construction/publication domains and universal proofs remain open.
