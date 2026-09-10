# W09 unit 4: ordinary forward binding projections

These definitions lower the exact W06 BindingTypeProjection descriptors to
ordinary core terms. They cover all twelve binding roles, source products and
enums, nested sequence elements, ordered-map entry dependencies and semantic
sum payloads. Member access follows the source carrier's stored-member layout;
role selection follows the content-bound binding member map. Map/set projection
preserves order and duplicates. It cannot establish the target domain by sorting
or removing source data.

Sequence conversion copies the complete length and converts only active indices.
Every inactive element and padding bit is zero, including when projecting a zero
source element would produce a nonzero semantic tag. Bound enum tests compare
all source bits (including signed i64 tags), and emit the frozen semantic tag.
An inline ordered-map key/value shape must resolve to exactly one registered
ordered-entry instance with the same argument IDs and physical shape.

The program binds source IR, foundation and independently regenerated binding-VC
identities. Import regenerates and compares all metadata and certificate bytes.
Only identity projections expose an unconditional reconstruction definition.
Every stored source member, including unmapped/inactive fields, remains in the
reconstruction obligation metadata. Forward conversion alone does not prove a
source invariant, target domain, inverse, operation commutation or native body.

The corpus contains 43 existing original-source contexts plus one independently
captured C# source. It pins 34 projection definitions across 44 certificates
(maximum 1,230 terms and 44 declarations). Empty projection inventories remain
explicit. A test oracle derives values from captured member names, frozen role
maps and closed-instance argument IDs, independently of generated core getters.
144 source-value/projection comparisons inspect 16,522 bits: all bits for small carriers and
active/nonzero, neighboring and high-selector bits for larger carriers.

The supplemental source contains a three-state field wrapper with i64 enum tags,
a sequence wrapper, and unmapped Extra fields. Source zero maps to the nonzero
semantic value tag; missing uses i64::MIN. Tests observe every bit at the first
two inactive indices and index 4095, and change Extra independently while keeping
the projected value identical. Reconstruction is explicitly absent for both
wrappers. These observations are concrete tests, not universal proofs.

See ../unit-4-projection-progress.json for executed checks and exact file hashes,
and ../unit-4-projection-review.md for the bounded review. Original units 3-8,
complete W09 assembly/application proofs, and W09 acceptance remain open.
The whole T06 check-fast.sh gate is deferred to T06-W12.
