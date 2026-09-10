# Construction storage operations (W09 unit 3)

The generator independently reconstructs every expanded construction instance,
its element carrier and published bounded-sequence dependency. It checks the
exact length/cells/bitmap representation, the 16,384 internal capacity, the 4096
published capacity and all five expanded operation signatures, equations and
ordered errors. Imports reconstruct exact metadata and certificate bytes.

Allocate stores the full length and zero cells, and sets precisely the initial
length's bitmap prefix if the supplied default-eligibility argument is true.
False always produces uninitialized storage. A true argument remains subject
to a source default-eligibility proof and its public conditions; successful
construction/type checking of this equation cannot establish those conditions.

Read projects the stored element. Fill and rewrite share a functional update
body that replaces one complete element and sets its bitmap entry; their
initialization gates differ. Updates compare the complete 32-bit index with
the internal address word and cannot alias high input bits. Complete checks
both the full length bound and every bitmap entry in the active prefix through
the ordinary shared fold. The explicit bound check prevents a clamped fold
from accepting an invalid length. Freeze projects length and the 4096-slot
published carrier, retaining all element bits.

Every operation records ordered failures and the exact normal-argument indices
used by each storage predicate. Ownership is explicitly a pending source-state
obligation with no Boolean definition. Normal-result bodies are meaningful only
after input storage/domain requirements and all operation gates are established.
They do not decide normal versus exceptional application execution on their own.
For freeze, source ownership, lifetime/version effects, complete initialization,
public element invariants, target-role limits and aggregate live-cell bounds
remain part of source-state/VC assembly. Internal storage carries no owner,
borrower, lifetime or version field. No caller flag can substitute for these
source facts. Borrow, transfer, discard and post-freeze source reads remain
native body/control relation work in the original implementation plan.

Five actual-source contexts are pinned: the existing nested product capture,
the float-array capture, and new bool, readonly product and nonzero-enum array
initializers in `../construction-sources/`. There are five construction
instances and 25 operations across these contexts, with at most 4509 terms and
82 declarations per certificate. One source type ID occurs in two different
source contexts; both are checked independently using the source VIR hash.

Semantic tests evaluate emitted ordinary terms against an independent sparse
storage model for allocation, reverse-order first writes, complete rewrites,
initialized reads, publication, negative/high indices and ordered predicates.
Every expected nonzero output leaf and its neighboring addresses is checked;
padding is sampled across every selector and role. This sampling is not a
universal padding proof. The large bitmap test covers 16,383/16,384 initialized
prefixes, a missing final cell, the same full bitmap at invalid length 16,385,
and the final published slot at 4096. Source semantics, the full bitmap test,
exact pinned replay and same-byte dual checking all pass; the checkpoint records
their log/artifact hashes separately from pending domain/source-state integration.

Targeted replay:

```sh
cargo test -p mpk-vc --test csharp_practical_vc csharp_03_t06_w09_constructions_original_sources_and_mutations
cargo test -p mpk-vc --test csharp_practical_vc csharp_03_t06_w09_constructions_storage_semantics
cargo test -p mpk-vc --test csharp_practical_vc csharp_03_t06_w09_construction_full_bitmap_boundary
```

Candidate regeneration uses a separate `MPK_W09_CONSTRUCTIONS_OUT` directory.
The checker-agreement test is
`TestCheckerAgreementWithRustCLIConstructionOperations`; it sends all five
pinned byte strings to both unchanged checkers, requires zero axioms and rejects
hash-corrupted versions. Helper acceptance does not discharge source/application
VCs. No W09 completion receipt is issued, and the full T06 gate stays at W12.
