# Forward binding contracts (partial W09)

This component connects `source_project` to the existing ordinary binding
projection generator. The binding ID and exact nominal source/result types must
select one reconstructed projection. The compiler emits the complete existing
projection set once and reuses it across contract recipes. It adds no source
completion or inverse witness; `source_reconstruct` remains a separate pending
part of W09.

The fifteen source contexts preserve the original C# source and semantic binding
bytes and paths. They cover all twelve original binding families, float and
nullable map values, and a nested sequence with remapped signed tags and extra
source fields. Each method contract reads its real source parameter through
`source_project`. Its projected self-equality is compared with the independent
structural model; the NaN case must be false. These are ordinary expression
observations, not accepted application VC proofs.

Direct calls to the actual contract alias are also checked for four source
samples against the existing independent projection oracle. Full small outputs
and large-output selector/edge/nonzero-neighbour observations cover payloads,
padding and source field selection. The alias and its complete transitive
definition closure must match the standalone projection program. Exact imports
and result-metadata corruption are tested as well.

`capture-receipt.json` records the fresh frontend inputs and isolation. All
fifteen frontend contexts were accepted. All fifteen source contexts passed
4,354 observations and exact dependency/import/mutation checks. Both unchanged
checkers accepted all fifteen identical-byte certificates with zero axioms and
matching reports, and rejected hash corruptions. Current hashes, metadata and
tested output copies match the terminal checker cases. The completed component
receipt is `../unit-4-binding-clauses-progress.json`, with direct review in
`../unit-4-binding-clauses-review.md`. Original units 3-8 and W09 remain open.
The whole repository gate remains deferred to T06-W12.
