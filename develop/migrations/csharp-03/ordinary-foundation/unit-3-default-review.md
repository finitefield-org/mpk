# Default-candidate direct review

Scope: `csharp_practical_ordinary_defaults.rs`, its exports, the source replay and
eligibility-edge tests, 29 pinned default certificates, the added Go agreement
entry point, and the accompanying inventory/coverage records. This is a component
review; it does not close the full pending W09 diff or internal unit 3.

The initial replay verified every emitted source condition against captured
facts, but would not detect a missing condition. It now independently builds the
complete source-condition set from each captured default graph and compares it
exactly. The 29-source replay passes with this stronger assertion. An additional
17-source nesting case rejects omission of one condition, a changed logical cell
count and a substituted definition, as well as oversized importer inputs.

The first nesting test used an incorrect expected source count (18). Inspection
of the retained original source established T0 through T16, or 17 products and
one scalar leaf. The corrected test passes. This was a test expectation failure,
not evidence of a production eligibility or checker failure.

Reviewed independently against `default_with_obligations` and captured source
graphs: source kinds, required members, declared enum zero, recursive occurrence
counting versus unique source requirements, intrinsic scalar defaults, and the
inactive option/lookup payload rule. Production generation does not use the host
default-value oracle. Public admission remains explicitly declaration-level;
neither metadata nor accepted helper certificates discharge source invariants.

The generator uses the common bounded certificate builder and the importer
requires exact regenerated metadata and bytes. The 29 unchanged certificates
pass both existing checkers with zero axioms; their hash corruptions reject.
No checker acceptance rule or Certificate v0 format changed in this component.
Scoped clippy, formatting and consumer inventory checks pass; see
`unit-3-default-verification.json` for the precise scope and log hashes.

Latest direct review of this component has no remaining actionable findings.
Large default cubes have sampled leaf observations, not exhaustive domain
verification. Recursive-domain semantic checks, semantic operations, source
condition discharge and application certificate assembly remain outstanding
under the original plan. `check-fast.sh` stays deferred to T06-W12.
