# Source-scoped construction operation proofs

This partial T06-W09 checkpoint adds an opt-in proof route for original invoked
construction operations at their exact source ownership points. Across all 45
original contexts it supplies 12 source-specific candidates (fill and freeze)
and leaves no eligible invoked source operation pending. Six contexts contain
these points. Generic construction operations remain pending: the root retains
all 25 original pending operations, including uninvoked operations, and all 987
application proof IDs. W09 stays In progress and W10-W12 stay Blocked; the full
T06 gate is deferred to T06-W12.

## Scope and construction

Each candidate retains its complete original DataOperationVc and checks the
original signature, function/node/receiver identity and symbolic ownership
state. The scoped failure definition contains checked Let dependencies on the
existing whole-flow theorem and exact receiver-point theorem, and returns the
existing closed ownership witness. Its only parameter is the original receiver;
there is no carrier owner bit or caller-supplied capability.

The unchanged construction storage domain enforces complete length, initialized
cells, zero uninitialized storage, bitmap, padding and recursive child validity.
It explicitly retains private_storage_only=true and ownership_pending=true.
The adapter resolves only the ownership failure at its exact source point. It
preserves the complete original operation argument/result types, normal recipe,
ordered remaining failures and every original W06 subject, premise, guard and
goal. The operation proof includes the complete ordinary equation with all
previous supplied operation proofs. Its strict importer regenerates the exact
source metadata and certificate vector. The old terms/declarations, public
domains, source clauses and observations remain unchanged.

These source-specific candidates do not establish generic ownership, native
execution or application assembly. The 25 generic operations and seven original
is_binding refinements remain unresolved. Existing exact source points cover
fill and freeze; no read/rewrite coverage is claimed. Foundation schemas,
Certificate v0, kernels and checker acceptance rules are unchanged.

## Verification and direct review

Three targeted local tests pass: all 45 source contexts and 12 scoped candidates,
the affected legacy 442-operation allocation proof route and all 45 original
operation pins. Shared serialized operation/proof structures and domain-symbol
lowering changed, so both legacy consumers are included. Crate Clippy with
warnings denied, crate format and both changed support-file format checks pass.
The 749-source/173-fixture audit passes. All 70 new exports are retained, and
all 92 legacy exports exactly match the published allocation checkpoint.

The source test independently computes eligible invoked operations from the
original full VC, matches every source descriptor, compares complete W06
sequents and old definitions/conditions, and inspects both checked Let
dependencies and the original certificate prefix. The actual Rust kernel accepts
every candidate with zero axioms and empty proof-node/theory-certificate tables.
A well-typed always-true concrete ownership failure is accepted in the
definition-only certificate and rejected at core checking with supplied operation
proofs. Receiver metadata and incomplete certificate-vector mutations reject.

An initial tuple-destructuring compile error was repaired before the successful
library check. A passing Bool-only prototype is retained separately; the final
verification reruns every original source after the final metadata/test changes.

Both unchanged Go/Rust checkers pass all 50 fresh stages. The independent
terminal audit matches every actual input with the final source export and
verifies checker binary, raw report and stderr hashes. All 24 positive stages
agree on export/certificate/axiom-report hashes, module/declaration counts and
zero axioms; all 24 hash mutations and both typed ownership mutations reject.
Requested Linux verification of exact public source 2c4613aa passes all three
targeted tests, Clippy and format. The independent terminal archive audit verifies
922 source/fixture Git blobs, all test logs/binary hashes, 70 byte-identical new
exports and 92 unchanged legacy exports. The long lint stage completed normally;
the earlier SSH observation failures did not terminate or restart the job.
See verification-logs/scoped-construction-proofs/.
