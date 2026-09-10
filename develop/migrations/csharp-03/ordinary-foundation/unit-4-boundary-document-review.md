# W09 boundary document representation correction (scoped review complete)

## Finding and implementation

The pending W07 boundary VC used `mpk.csharp.value.string.v1` for the document
binder and EncodeOutput result. The ordinary application string carrier has
only 16,384 UTF-16 storage slots. Omitting its PublicDomain predicate cannot
add storage, so documents with a longer common prefix could lose distinctions
in their tails. The frozen boundary limit is instead 1,048,576 UTF-8 bytes.

W09 now names a private ordinary core type,
`Mpk.CSharp.Ordinary.BoundaryDocument`, at those two logical interfaces. Its
definition is an ordinary C24 alias with a full u32 length and a 1 MiB byte
sequence. It is not added to the registered source value vocabulary. Application
strings retain their existing C19 shape and bound. Source/profile/foundation
artifacts and the core checker rules are unchanged.

The dedicated generator links its definition to the independently reconstructed
boundary VC hash, contract IDs, source IR and foundation hash. Its Make, Length,
Bounded and ReadByte helpers preserve every byte index through 1,048,575. ReadByte
checks the full u32 index against both logical length and storage capacity before
reading low index bits, so oversized indices cannot alias earlier bytes. Make
preserves oversized length sentinels for a separate Bounded rejection, clears
inactive bytes, and zeros all length-header padding.

This change is a required foundation for JSON parsing and encoding, not their
completion. UTF-8 and JSON grammar, typed fields, duplicate/name ordering, decoded
cell limits, native/source linkage and universal codec/boundary proofs remain
required by the original W09 plan.

## Direct review and regression evidence

- The W07 subject assertion would fail on the previous string-typed binder.
  Source tests check that the private type is absent from application carriers
  and that application strings still have depth 19.
- Storage tests exercise every significant high byte-index boundary, full
  length-bit preservation, capacity and capacity+1, and u32::MAX indices.
  Both helper reads and raw stored byte bits are observed independently.
- Two canonical JSON documents contain the same 16,384-byte prefix and differ
  only in a later `true`/`null` field. Their decoded string has exactly 16,384
  units, while their complete documents exceed that application-string limit.
  Tests require their distinct tails to remain observable.
- The W07 contract/run golden hashes and W09 boundary-literal metadata require
  regeneration because the logical boundary type changes their linkage hash.
  Prior goldens are retained with explicit before/after provenance; literal
  certificate bytes must remain identical since decoded value definitions do
  not change.
- Initial integration compilation found an accidental call to a private
  generator from the test. The test now uses the public PracticalVcSource/VC
  generation path. This was a test build failure, not a checker rejection.

The initial 65-context storage corpus contained no attached boundary contracts;
its coverage assertion correctly failed. The final corpus adds three existing
captured boundary-output sources, retaining that assertion and the production
presence guard. All 68 contexts and storage regressions passed in 45.11 seconds.
Three pinned certificates contain 2,506 terms and 44 declarations each. Both
unchanged checkers accepted all three with zero axioms and rejected hash
corruption (5.773 seconds); exact pinned replay passed in 45.46 seconds.

All 15 W07 contract and two run golden hashes were regenerated. Field-by-field
comparison against archived prior records confirmed only the expected linkage
hash changes. All nine boundary-literal certificate bytes, definitions, values,
bindings and 20,258 observations were preserved. Four affected W07/W09 tests
passed against the installed revised goldens and manifest (33.63 seconds).
Current lint and formatting passed. All five inventory tests passed again after
the final corpus extension and fixture installation (16.44 seconds). The final
scoped review found no additional issue in this representation correction;
no component-only commit, internal-unit completion or W09 completion is claimed.
`check-fast.sh` remains deferred to T06-W12.
