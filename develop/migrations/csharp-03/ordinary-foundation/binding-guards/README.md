# Ordinary binding guards — partial W09 unit 4

These 45 certificates extend the binding-observation relation program with
ordinary definitions for the exact W06 `SourceTag`, `SemanticArm`, `Payload`,
`MemberProjection`, `Bound`, and `NonemptyInvalid` predicates. They resolve 141
additional demanded symbol occurrences: 466 predicate occurrences in total,
with 2,260 other W06 symbols still explicitly unresolved. The largest program
has 44,046 terms; the maximum declaration count is 507 and the maximum static
transformer count is 8,602. No application theorem is claimed by this corpus.

The original source/capture contexts are the same 45 used by
`../binding-relations/`. Generation independently reconstructs the construction
VCs, binding VCs and concrete carriers from the validated VIR. Canonical import
regenerates both metadata and certificate bytes. Context substitution, changed
predicate mappings, changed certificate bytes and the wrong program schema
reject. The base projection, observation and result-agreement definition
closures are checked for preservation by the certificate-generation test.

Source tags compare every bit of their original enum carrier, including a
64-bit signed-minimum tag remapped to semantic `missing`. Semantic tags use the
full 32-bit sum tag. A payload predicate requires both matching source and
semantic arms, then compares independently read and converted source payloads
with the semantic payload. Member predicates read the original mapped source
member: they are not definitions of a field of `Project(source)` compared with
itself. Inactive source payloads do not constrain an unrelated active semantic
payload; all source members remain subject to separate reconstruction checks.
Unknown tags fail these guard predicates. These predicates assume the separate
representation-domain obligations for canonical padding and legal values.

Observation equality preserves IEEE bit patterns (including signed zero and
NaN payloads) and compares decimal cohorts numerically. This is unchanged from
the corrected binding-observation program. It does not change native C# `equal`.

Sequence/map/set and transition-event bounds read the complete unsigned
32-bit length. Validation's error bound is conditional on its invalid arm;
its nonempty predicate requires at least one error on that arm. The valid arm
satisfies both conditions and unknown tags satisfy neither. Separate domain
predicates still enforce all other representation constraints.

The tests evaluate emitted ordinary terms with the existing core evaluator.
They include independent source-field projection oracles, decimal cohort
normalization, IEEE edge representations, signed-minimum and unknown tags,
and lengths 0, 1, 255, 256, 257, 4095, 4096, 4097 and `u32::MAX`. Deep member
observations have a separate test; their execution status must be read from
the progress receipt, not inferred from successful shallow or bound cases.

`TestCheckerAgreementWithRustCLIBindingGuards` passes each exact decoded byte
sequence to both unchanged checkers, requires zero axioms and rejects a
hash-corrupted version. See `../unit-4-binding-guard-progress.json` for actual
run status. Neither helper typing nor these finite observations supplies native
execution proofs, reconstruction witnesses, canonical codecs, default-use
analysis, canonical-order obligations, transition/replay relations or complete
proof assembly. Internal units 3–8 and W09 acceptance remain open. The full
repository gate remains deferred to T06-W12.

The completed guard checkpoint passed all 45 cases through both checkers in
1,329.647 seconds, with zero axioms and rejected hash mutations. The ordinary
semantic tests passed 1,019 small-carrier observations, 321 bound/unknown-tag
observations and 52 deep member/payload observations. These are completed
component tests; the larger W09 scope remains open as described above.
