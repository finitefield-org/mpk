# W09 unit 4 binding-relation component review

Scope: exact shared projection emission, ordinary W06 observation/equality/member
predicates, projected-result agreement, explicit unresolved symbols, source tests
and fixed-byte checker coverage. Full unit 4/W09 review remains outstanding.

Resolved implementation findings:

- Calling the ordinary cube helper constructor twice caused duplicate globals
  and Linkage failures on the first original source. The relation generator now
  reuses the projection emitter's idempotent ordered-word helper installation.
  Both generation and semantic tests failed before that fix.
- Bool equality is a W06 source-invariant predicate even when Bool is absent
  from source signature carriers. Emit the concrete one-bit equality circuit
  directly, rather than silently leaving this supported symbol unresolved.
- An identity reconstruction already supplied by the projection program must
  be counted as resolved. Only Some reconstruction definitions contribute the
  reconstruct symbol; nonidentity witnesses remain explicitly unresolved.

The test harness uses the public validated full-VC generator to reconstruct W06
requirements; private internal generators are not exposed merely for tests.
The previous-certificate mutation state has an explicit sized byte-vector type.

Review checks: exact source/context/VC reconstruction, source and semantic type
signatures, source observation versus non-reflexive semantic equality, every
stored member, projection dependency-closure preservation, duplicate declarations,
ordinary size/transformer limits and unchanged kernel acceptance. The Extra/NaN
examples demonstrate that projected equality cannot substitute for reconstruction
observation. Unresolved names are metadata acceptance inputs and mutation-tested.

See unit-4-binding-relation-progress.json for current verification and pending
work. No review completion or application proof receipt is inferred while that
component's required verification remains pending. The larger W09 source-domain,
reconstruction witness, native/control, transition/replay and complete proof
assembly requirements stay in the approved eight-unit plan.
