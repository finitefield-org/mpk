# Ordinary binding order predicates — partial W09 unit 4

This corpus contains the nine original-source contexts that demand W06
`Mpk.CSharp.Binding.CanonicalOrder.<semantic-type>` predicates. They demand ten
predicates in total because one context has two independently specialized maps.
The other 36 contexts from the 45-source binding corpus do not demand ordering;
the generator leaves their certificate bytes identical to the guard program.
All 45 base guard metadata/certificate pins are checked during generation.

The ordinary definition follows source-design sections 8.5 and 9.3: a sequence
of at most 4,096 entries/elements is canonical exactly when each adjacent pair
of active keys/elements is strictly increasing under the frozen canonical
comparator. An empty or singleton sequence has no adjacent pair. Map values
and storage outside the active length do not participate in ordering. Equal
keys reject even if their map values differ. Decimal scale cohorts and signed
zeros compare by decimal value; ordinal strings and structural keys use the
existing concrete canonical comparators. Unorderable key types have no
comparator and cannot be emitted through this path.

The compiler selects the exact key type from reconstructed closed-instance
metadata and checks it against the concrete sequence/entry layout. A map key
getter reads only the key field. The adjacent-key predicate is evaluated by the
existing counted concrete aggregate fold. The full unsigned 32-bit length is
checked before the fold; an out-of-range length cannot wrap the 12-bit storage
index. The last active element does not read a subsequent key. No source sort,
source invariant, constructor, or native operation is treated as proved by
these definitions. Representation domains, including padding and key validity,
remain independent obligations.

The program extends the guard assembly without replacing its projections,
observations, payload/member conditions or result agreements. Canonical import
regenerates the whole program; metadata/context substitution, changed bytes
and the wrong schema reject. The scoped corpus has a maximum of 43,258 terms,
511 declarations and 8,594 counted static transformers. The generated program
schema is `mpk.csharp.ordinary_binding_orders.v1`.

`csharp_03_t06_w09_binding_orders_original_source_certificates` checks all 45
original-source contexts, freezes these nine new certificates, verifies exact
unresolved-symbol reduction and compares every base definition's transitive
closure. The source semantics test covers 140 observations, including empty,
singleton, ascending, descending, duplicate, changed values, inactive storage,
overlength, decimal cohorts and signed zero. The separate full-capacity test
checks both integer map and set at 4,096 elements, then duplicates key 2,048 and
key 4,095 to expose high-index/last-pair mistakes. These are test scopes, not
claims that a still-running test has passed.

`TestCheckerAgreementWithRustCLIBindingOrders` checks exactly these nine pinned
byte sequences with both unchanged checkers, zero axioms and hash-corruption
rejection. All nine cases passed both checkers with zero axioms and rejected their
hash-corrupted variants in 247.406 seconds. Other terminal results and live jobs are recorded in
`../unit-4-binding-order-progress.json`. Internal units 3–8, reconstruction,
canonical codecs, native/control semantics, transition/replay and complete
proof/certificate assembly remain open. The repository gate remains deferred
to T06-W12.

Direct review also added the separate eight-case
`csharp_03_t06_w09_binding_orders_string_compound_edges` test. Equal first
fields force a structural key comparison to reach its second field; another
case makes the first field dominate an opposing second-field order. String
cases check equal prefixes, prefix length, duplicates and UTF-16 code-unit
order across the surrogate boundary. Its actual run status is recorded in
the progress receipt independently of the earlier 140-case semantic test.

All six full-capacity map/set observations and all eight separate
string/compound edge observations have passed. The 140-case source semantic
test remains independently tracked; it must finish before the component is
reported as fully verified.

The 140-case source semantic test subsequently completed successfully. The
three-test generation/semantic/full-capacity process passed in 1,124.53 seconds.
Together with the separately passed eight-edge test, pinned replay, same-byte
dual-checker cases, lint, format and inventory, this closes the order component
verification. It does not complete internal unit 4 or W09.
