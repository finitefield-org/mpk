# Nullable-reference data relations

The adapter connects reference extraction to original W03 subjects and the
NullReferenceException edge. Four captured C# contexts contain four definition
occurrences and SSA points. Their null cases pass (twelve result observations).
All four certificates are covered by two unique same-byte Rust/Go checks,
including corrupted-hash rejection. Identical conditional/suppressed/property
certificates, definitions, SSA records and oracle payload-type facts are
explicitly compared before reuse; source linkage is checked for each context.

Both distinct normal-value partitions now pass: guarded/some in 41.17 seconds
wall and conditional/some in 39.13 seconds wall. Including the null cases,
30 result observations were executed; exact-equivalence reuse covers 18 more
observations for the other source contexts (48 context observations in total).
The equivalence was revalidated against current artifacts at result import.

Logs, original manifest and receipts are retained in
`../verification-logs/reference-data/user-long-completed/`. No tests were rerun
to import these results. The pending list is empty; no rerun is requested.
Both partitions are agent-owned on future affected runs because the measured
completion times are under one minute. The initial 45-second cutoff remains
diagnostic history; no source change or speedup is claimed by this import.
Full native reachability, domain proofs, unit 5 and W09 remain incomplete.
