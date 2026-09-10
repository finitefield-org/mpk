# Finite non-template operations (W09 internal unit 3)

`generate_csharp_practical_ordinary_finite_operations` generates all registered
unit and parse-error operations for reachable carriers, and construction, type
tests and payload access for every arm of the reconstructed exception universe.
Generation includes operations not invoked by the source. Instant operations
remain with the earlier scalar component.

Unit make is the zero Boolean cube; equality is true and comparison is zero on
the sole valid unit value. Parse-error tag returns u32; equality and comparison
observe the frozen unsigned tag order 0 through 4. Exception constructors retain
all source payload members with canonical storage padding. Type tests include
ArgumentException's two admitted descendants and the admitted
InvalidOperationException descendant, rather than testing only tag equality.

Each exception operation records its closed type or member specialization. A
payload read records the required active tag. Returning zero storage on another
tag does not establish a normal result: the caller must discharge that tag
condition. All input representation/public domains and source construction
conditions remain caller obligations. These operation definitions do not prove
application VCs or execute arbitrary CLR constructors.

Seven retained original-source contexts generate 103 operations, with at most
2,408 terms and 66 declarations in one certificate. Tests observe 94 constructor
cases, all target-type comparisons (including inherited and unrelated types),
high invalid tag bits, source payload values, the sole unit value, and all five
parse-error tags and 25 ordered pairs. The parse-error source uses a captured type
contract; provenance is in `../finite-sources/capture.json`.

The seven source certificates plus the scalar helper certificate are accepted
with identical bytes by both unchanged checkers and zero axioms. One hash
corruption per certificate rejects. Exact importer reconstruction also rejects
metadata and omitted active-tag requirements. The log and artifact hashes are in
`../unit-3-finite-verification.json`.

Adding the actual parse-error contract exposed missing contract-only carriers.
The correction and retained prior certificates are documented in
`../contract-carrier-extension/changes.json`. Full source-body semantics,
remaining collection/outcome operations and application proof assembly stay in
the original W09 plan; `check-fast.sh` remains deferred to T06-W12.
