# Ordinary default candidates (W09 internal unit 3)

The 29 pinned certificates contain concrete default values and declaration-level
admission flags, reconstructed from validated original-source VIR, including
contract-only value types. They do not
prove source invariants or application VCs. W09 and internal unit 3 remain open.

`generate_csharp_practical_ordinary_defaults` independently computes structural
eligibility. Scalars use their frozen default allowlist. Source products require
defaults for all members; sealed classes, required members and enums without a
declared zero have no candidate. Option-none and lookup-missing are one-cell
defaults without inspecting an inactive payload's default eligibility. Other
closed templates do not acquire defaults merely because zero storage exists.

Candidates are actual closed ordinary Boolean cubes. Source products count every
stored occurrence, while source requirements are unique by source type and retain
the original source hash and public-default declaration. The inclusive logical
cell bound is 65,536; an excess default has no candidate. A public admission flag
records eligibility under these declarations and does not discharge them.

`certificates.json` pins program metadata and structural costs for 170 carrier
occurrences: 108 candidates and 62 ineligible definitions. Candidates retain 22
source-condition occurrences; 14 candidates are structural-only. The largest
certificate has 108 terms and 25 declarations. Exact regeneration rejects
metadata, linkage and certificate changes; both importer byte inputs have a
16 MiB limit.

Verification recorded in `../unit-3-default-verification.json` covers:

- Original-source replay, independent default-value/cell-count comparison,
  complete captured source-condition sets and exact pinned certificate bytes.
- All leaves for cubes of depth at most 10, and zero/all-one/one-hot selector
  addresses for larger default cubes, through the generic ordinary-term evaluator.
  Large cubes are sampled; this is not an exhaustive domain proof.
- All 29 identical certificate byte sequences accepted by Rust and Go with zero
  axioms; one hash corruption per certificate rejected by both.

The required-member, enum-without-zero, sealed-class and 17-level nested-source
regressions use four additional retained original-source captures. They test
eligibility, condition retention and importer limits; they are separate from the
29-certificate dual-checker corpus.

Source public clauses, semantic operation integration and application proof
assembly remain in the original W09 plan. The repository-wide `check-fast.sh`
gate is deferred to T06-W12.
