# W09 unit 3 domain implementation notes

These are inspected requirements, not a completion receipt. A recursive-domain
implementation is now under verification; see `unit-3-domain-progress.md`.
Comparisons require their input domains; passing relation/helper certificates
does not establish application invariants or discharge application VCs.

Reconstruct all storable carriers and exact closed-instance roles from validated
VIR. Reuse the existing scalar/source-enum domain definitions. Recursive domains
must enforce source field membership, full sum tags, the active payload, every
unused role/leading-padding bit, and zero storage outside active sequence length.
An empty product's storage Bool is false. The shared sum payload is one physical
region, not separate inactive-arm arrays. A fold's capacity clamp must never
stand in for an explicit length bound.

The existing value model fixes these inclusive bounds: arrays, sequences,
map/set entries and transition events 4,096; UTF-16 strings 16,384; validation
invalid errors 1..256; total logical value cells 65,536. Validation's 256 bound
is attached to the invalid payload role even though the shared sequence carrier
has capacity 4,096. Ordered map keys and set elements must have eligible total
ordering and be strictly increasing, so numerically equal decimal encodings
cannot evade uniqueness by changing their stored scale or zero sign.

Cell counting is semantic, not the physical cube size or number of helper
products. Primitive values (including decimal) count as one. Source products,
ordered entries, options, boundary fields, sequences and tagged sums add their
own one cell to their logical children. String counts one plus UTF-16 length.
Two particular storage adapters must not introduce extra logical cells:

- A map is one plus each key and value count; its inline entry storage product
  does not add one per entry.
- A transition is one plus state, each event and response; its storage sequence
  for events does not add a sequence-wrapper cell.

By contrast, validation's invalid arm contains an actual sequence value and
counts that sequence cell. Closed exceptions add their active source payload
value where present; nested closed-exception values are rejected by the current
value model. Counters must not wrap: saturation above the inclusive bound or an
explicit overflow predicate must preserve rejection. Capacity/padding tests and
logical-cell tests need separate boundary cases.

Default eligibility follows `default_with_obligations`, not storage zero:
source enums require a declared zero; sealed classes and required members are
ineligible; recursive source defaults retain their public-default requirements.
Among closed templates the existing algorithm admits option-none and
lookup-missing, not every all-zero sum or empty collection. A declared public
invariant remains an obligation; eligibility metadata is not a proof of it.
Source constructor/body proofs remain with the later W09 assembly work.

Implementation must stay within the frozen term/declaration/binder/transformer
limits with concrete ordinary terms. No host value validator, byte hash,
extensionality assumption, generic recursor or checker rule may replace the
recursive ordinary predicates/proofs. Large padding and nested logical-count
cases require real term-cost and same-byte dual-checker verification.

Inspected sources: `csharp_practical_vir_model.rs` constants,
`validate_value_inner`, `validate_tagged_sum`, `ensure_strictly_increasing`;
`csharp_practical_domain.rs::default_with_obligations`; the reconstructed
validation role in `csharp_practical_ordinary_carriers.rs`; and the completed
scalar-domain helper definitions. The original eight-unit plan and W09 exit
condition remain unchanged.
