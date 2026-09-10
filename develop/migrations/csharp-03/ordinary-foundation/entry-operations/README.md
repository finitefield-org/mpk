# Ordinary ordered-entry operations (W09 unit 3)

The generator reconstructs each expanded ordered-entry instance, including
standalone/uninvoked instances, from the validated original VIR. It checks the
exact key/value fields, concrete arguments, empty dependency set and all frozen
operation signatures, equations and error lists. It emits make, key, value,
semantic equality and, only for totally ordered types, canonical comparison.
Imports regenerate exact metadata and certificate bytes under the 16 MiB limits.

Storage construction and projections reuse the ordinary product definitions.
Comparison is lexicographic over key then value; IEEE-containing entries omit
comparison and retain NaN's non-reflexive equality. These operations do not
discharge representation/public domains, source binding correspondence or any
application VC.

Six actual-source contexts produce six entry instances and 29 operations. The
existing entry/map/string/decimal/compound captures are supplemented by the
original Bool-key/float-value source and content-bound sidecar in
`../entry-sources/`. All six pinned certificates passed both unchanged checkers
with zero axioms; hash corruptions reject. Maximum sizes are 39,739 terms,
403 declarations and 8,256 counted static transformers. Exact source-request
and pinned-certificate replays passed.

The separate ordinary-term semantic test covers construction, projections,
equal-key/different-value comparisons, opposing key/value order and NaN. It
passed all six source/instance contexts with 248 observations in 1805.97 seconds.
Large padding observations are sampled around
all expected nonzero leaves and boundary/one-hot addresses, not universal proofs.
See `../unit-3-entry-collection-progress.json` for current evidence and live jobs.

Targeted tests are `csharp_03_t06_w09_entry_source_request`,
`csharp_03_t06_w09_entries_original_source_certificates`,
`csharp_03_t06_w09_entries_original_source_semantics` and the Go checkeragreement
test `TestCheckerAgreementWithRustCLIEntryOperations`. Generation uses
`MPK_W09_ENTRY_REQUESTS_OUT` or `MPK_W09_ENTRIES_OUT` only for a separate candidate
destination; ordinary replay requires exact checked-in files.

Unit 3/W09 remain incomplete. Full map/set acceptance, source-state integration
and the original units 4-8 remain required. The full T06 gate stays at W12.
