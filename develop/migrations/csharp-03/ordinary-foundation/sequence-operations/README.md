# Bounded-sequence ordinary operations (W09 unit 3)

The generator reconstructs all expanded bounded-sequence instances from the
validated original VIR. Each instance has length, indexed read, equality and,
only for total element types, lexicographic comparison. It compares the entire
expanded operation signature/equation/error list with the frozen expectation;
uninvoked operations are included. Import rebuilds exact metadata and bytes.

The index-range failure checks the full 32-bit index against both the stored
length and the physical 4096 bound before the read's 12 address bits are used.
Negative indices and high-bit aliases fail. The read returns storage zero on
failure; that value is not an accepted normal result. Input representation and
public domains and the false range-failure obligation remain mandatory.
Equality and comparison use the shared ordinary finite fold and semantic child
relations, including IEEE non-reflexive NaN equality. No IEEE-containing
instance receives a total comparison operation.

The pinned corpus has 12 actual-source contexts and 16 concrete instances,
including nested products, map dependencies, decimal/string elements and a new
float-array capture in `../sequence-sources/`. Generation/import tests check
all expanded instance IDs and mutated metadata/certificate bytes. The separate
semantic test includes 4096 slots, valid/invalid index boundaries and small
sequence pairs. Supplemental tests check every high index bit 12..31 at two low
addresses, plus equal-length differences at the first and later elements in
scalar and source-product sequences. Large child outputs sample padding while
checking every nonzero expected leaf and neighboring addresses; this is not an
exhaustive proof over every padding address or every application input.

`certificates.json` records exact original VIR/foundation identities, generated
operation names, certificate hashes, actual terms/declarations and counted
static transformers. Maximum sizes are 42,816 terms and 457 declarations.
The semantic pass completed in 1868.22 seconds with 457 indexed reads and its
full byte/metadata output exactly matches the pinned dual-checked corpus.
The certificate-only test performs no semantic observations. It is separate
from the long-running semantic test so neither coverage nor completion is
inferred from successful certificate construction.

Targeted replay:

```sh
cargo test -p mpk-vc --test csharp_practical_vc csharp_03_t06_w09_sequences_original_source_certificates
cargo test -p mpk-vc --test csharp_practical_vc csharp_03_t06_w09_sequences_original_sources_and_boundaries
cargo test -p mpk-vc --test csharp_practical_vc csharp_03_t06_w09_sequence_index_high_bits_never_alias
cargo test -p mpk-vc --test csharp_practical_vc csharp_03_t06_w09_sequences_equal_length_element_order
```

Candidate regeneration sets `MPK_W09_SEQUENCES_OUT` to a separate directory.
Normal tests compare checked-in bytes. The checker-agreement test is
`TestCheckerAgreementWithRustCLISequenceOperations` and requires all 12
identical byte strings to pass both unchanged checkers with zero axioms, and
their hash-corrupted counterparts to reject. See the adjacent unit-3 checkpoint
for actual results and remaining integration work. These helper certificates do not prove
an application VC; unit 3, W09 and units 4-8 remain incomplete. The full T06 gate
is deferred to W12.
