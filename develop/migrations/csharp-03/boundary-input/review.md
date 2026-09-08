# W02 local task review

Scope: CSHARP-03-T05-W02. Review covers the complete task diff, shared parser
changes, typed conversion, immutable handoff, cumulative source linkage,
retained evidence and local regression tests. No delegated review or hosted CI
is used.

## Review findings addressed

1. The earlier transport-only input helper did not establish typed invocation
   evidence. W02 has a sealed byte-only run type, independently decodes all
   arguments and rebuilds both manifest links. A T02 receipt and an input-shaped
   sidecar cannot substitute for this handoff.
2. Practical JSON preserved surrogate values but rejected surrogate member
   names admitted by W01. The shared representation and canonical parser now
   preserve both, reject duplicate decoded names, and avoid sentinel collisions.
   Closed protocol/contract schemas still reject unknown names.
3. Per-field limits did not establish an aggregate run bound. W02 counts the
   complete typed arguments with the existing monomorphic validator, including
   string units. Independent actual-source matrices exercise cell and byte
   boundaries without accidentally exceeding a different limit first.
4. Expanding defaults and presence/nullable tags can enlarge typed evidence.
   The canonical input document and evidence transport use their respective
   frozen byte bounds rather than imposing the document limit on both.
5. Hash equality alone cannot establish reproduction. The persisted importer
   reparses original canonical bytes and compares regenerated capture/manifest/
   artifact contents; in-memory replay additionally compares exact bytes and
   complete typed values. Rehashed identities and stripped links reject.

6. The first full gate found that extending the JSON enum required editing a
   CLI source file pinned by both historical recursor/capacity measurements.
   The frozen file is now byte-identical to its retained digest. Lossless names
   use internal, escaped Object-key tokens within the existing enum instead;
   they never enter canonical transport. Literal marker-prefix names remain
   distinct from surrogate names. Both historical reproducibility tests pass
   without changing their records, vectors or measured source inventories.

## Final review

No open findings. Only W02 input capture/conversion and its prerequisite parser
support are implemented. Source reconstruction remains the already validated
ordinary VIR operation; invariants and commutation remain pending obligations.
No application constructor, second binding engine, source output implementation,
public activation, frozen vector or historical receipt was added or changed.
`verification.json` records the completed local verification and reviewed hashes.
