# W09 unit 4: ordinary VIR literal definitions

The generator reconstructs every block literal and failed-check exception value
from the validated VIR. It emits an ordinary closed core value and records the
exact function/block/value or check origin. Equal typed values share a root;
different source references remain separate bindings. Import regenerates the
entire program and requires exact metadata and certificate bytes, including the
source and foundation identities. No caller supplies a trusted constant body.

The encoder preserves source member order, low zero padding, active sum tags,
sequence lengths and active elements. It retains raw IEEE bits and decimal
sign/scale/coefficient storage. Decimal observation equality is a separate
relation and does not change a literal's stored cohort. Child constants are
shared under content-derived names before placement beneath selectors, avoiding
nested-lambda depth growth. Existing builder limits apply and excessive programs
fail closed. No checker rule, proof axiom or public application route is added.

The current source corpus contains 64 contexts: 43 reused actual source captures
and 21 additional captures in `../literal-sources/`. Thirty-three have literals:
88 distinct typed definitions, 105 source references and 6,550 ordinary-core bit
observations against the independent flat-storage encoder. The other 31 contexts
check exact empty inventories; they are not positive literal-semantic evidence.
Twelve top-level value variants occur. Generation, origin/value linkage,
metadata/byte and cross-context mutations passed. Source certificates have at
most 362 terms and 20 declarations. Large string/storage padding is sampled;
these observations are not universal application proofs.

The `literal-f32-nan` and `literal-f64-infinity` source expressions lower to
arithmetic over zero/one literals. Their names describe the source expression,
not captured NaN/infinity literal bodies. Supplemental `../literal-edges/`
cases directly check NaN payload/sign bits, signed zeros, decimal cohorts and
all selectors of a depth-253 constant; these constructed edge cases supplement
the actual-source corpus. They are not source execution proofs.

All 64 source pins passed exact replay and both unchanged checkers with zero
axioms and hash-corruption rejection (285.593 seconds). The two edge pins also
passed exact replay and both checkers (3.359 seconds). Final scoped lint, format
and five inventory tests passed. See
`../unit-4-literal-progress.json` for exact terminal results and live jobs.
Boundary-run values now have a separate checked body/symbol bridge in
`../boundary-literals/`. The proposition compiler, codecs, source/control
relations and application certificate assembly remain outstanding under the
original eight-unit plan.
This does not complete unit 4, unit 3 or W09. Full check-fast.sh stays at T06-W12.
