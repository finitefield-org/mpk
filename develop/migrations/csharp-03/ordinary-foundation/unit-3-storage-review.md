# Direct review: W09 unit 3 product/sum storage component

Scope: ordinary storage construction/projection, source reconstruction/import,
26 certificate fixtures, checker driver registration, and inventory updates.
This component is complete; internal unit 3 and W09 remain incomplete. No
subagent or review skill was delegated.

## Findings addressed

- The original-source mutation test incorrectly assumed every input has a
  structural carrier. Scalar-only sources now test insertion of a forged
  entry; unchanged empty-list mutations are skipped without inventing a type.
- Clippy found an unnecessary Vec in the synthetic carrier fixture; use an array.
- Adding the structural module introduced one standard-namespace consumer.
  The exact old path set is recovered by removing that module. Update its
  fingerprint/count and linked raw inventory hash. The inventory regression
  exposed a second stale value, the aggregate 4,928 count; update it to 4,929.
  All five inventory tests pass after the correction.

## Final review

No remaining actionable findings in this component. Reviewed against the
frozen foundation-definitions.json ordinary_core product, sum, lift, address,
and conditional equations. Product field roles precede leading child padding;
sum payload is shared active-arm storage, with a separate full-width tag.
The de Bruijn indices account for all constructor arguments and selectors.
Padding beyond 32 selectors is tested independently of fixed-width role/tag
address comparison. Empty/singleton/mixed fields, unused roles, sparse/high-bit
tags, wrong/unknown-tag access and binder 256/257 are exercised in ordinary
terms. Exact same bytes pass both unchanged checkers with zero axioms.

The generator derives every product/sum root from independently reconstructed
validated VIR carriers, including uninvoked instances. Types and names are
closed, deterministic and injective. Import regenerates metadata and bytes;
it never trusts supplied layouts or hashes. Existing builder limits apply to
actual terms/declarations/binders and metadata/certificate lengths are bounded.
Source replays retain field IDs/order and arm tags/IDs; substitutions reject.

MakeStorage requires valid children and only zeros the newly introduced
wrapper padding. It does not check source invariants, scalar/collection domains,
default eligibility or validation bounds. An inactive accessor returns storage
zero plus a separately exposed failure flag; storage zero is not a proved valid
source result. These are helper certificates, not application proofs. Remaining
unit 3 work is retained in unit-3-progress.md, and W09/W10 status is unchanged.

Targeted commands and logs are recorded in unit-3-storage-verification.json.
The repository-wide gate remains deferred to T06-W12.
