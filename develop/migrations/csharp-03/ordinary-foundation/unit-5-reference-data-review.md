# Nullable-reference extraction review

Reference extraction uses its independently validated `reference.value` Data
signature, not the option Foundation signature. Only nullable string, sealed
source class and published-sequence payload kinds admitted by the existing
signature validator are eligible. The helper selects the frozen option value
body and its exact active-tag failure predicate. It checks one operand, result
carrier, one frozen helper failure and the original ordered exception.

The low-level tag failure is shared with Nullable<T>.Value, while the native
W03 check and exceptional edge remain `System.NullReferenceException`, never
`System.InvalidOperationException`. No tagged error value is invented. Null
success guards are false and guarded success goals remain vacuous for both
correct and deliberately corrupted output storage. Domains and branch
reachability are separate obligations, not inferred from these observations.

The computed normal payload is bound once and compared over its complete
physical carrier, including inactive padding. Original subjects, normal edge,
exceptional edge and exception value are preserved from the validated W03 VC.
All unrelated definitions stay explicit pending IDs. Import independently
regenerates both metadata and certificate bytes from the exact original VIR.
Mutation tests reject context/symbol/function/node/successor/hash substitutions
and an InvalidOperationException substitution on the original failure check.

Four original captures cover an explicit null guard, conditional access,
null-forgiving access after a pattern guard, and a property receiver. Their
null cases and canonical import/mutation tests pass. All four certificate
artifacts are covered by two same-byte Rust/Go checks: conditional, suppressed
and property have identical certificate bytes, definition and SSA records,
and payload source-type facts used by the fixed sample oracle. This exact
equivalence is recorded before reusing runtime evidence. Source byte spans and source digests are excluded only from the sample-oracle
comparison; they are not interchangeable source contexts. Source-context
metadata for every capture is still independently reconstructed and checked.

Both distinct normal partitions passed in the user run: 41.17 and 39.13 seconds
wall. Together with null cases this yields 30 executed observations; exact
revalidated equivalence covers another 18 context observations. Current source,
input and certificate hashes match the original handoff; logs and receipts are
preserved without rerunning tests. Both partitions become agent-owned on future
affected runs because measured times are under one minute. The earlier
45-second timeout is retained as diagnostic history, not a speedup claim.
The twelve preceding option candidates remain byte-identical. Direct review
corrected the test's copied role-filter spelling before its candidate run.
No checker/evaluator or prior operation body changed. Full native control,
public-domain and proof integration remain open, as do unit 5 and W09.
