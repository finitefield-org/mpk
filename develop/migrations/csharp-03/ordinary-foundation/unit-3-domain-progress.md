# W09 unit 3 recursive-domain work in progress

This is an implementation checkpoint under verification, not a completion
receipt. Unit 3 and W09 remain open; W10-W12 remain blocked. Default candidates
now have separate coverage in `defaults/README.md`; source-condition proof assembly,
remaining collection/foundation operations and units 4-8 retain their original scope in
`implementation-plan.md` and `unit-3-progress.md`.

## Current implementation

The shared First/Any/All and saturated Sum helper uses one concrete C6 state
pipeline with 8,192 StepTwo occurrences. Its unsigned adder extends operands
before addition and saturates at 65,537, retaining rejection short-circuiting.
The pipeline and finite adder have 8,217 counted transformers. Callers share
the adder. Previous ordered-fold and relation generators preserve pinned bytes.

Recursive domains reconstruct reachable carriers from validated VIR. Count
returns logical cells or invalid sentinel 65,537; Valid tests against the
inclusive 65,536 limit. Checks include scalar/source-enum rules, full sum tags,
active payloads, every product role and leading padding bit, sequence bounds,
unused sequence storage, strictly increasing map keys and set elements. Inline
map entries and transition event storage introduce no extra logical wrapper;
validation's real sequence does count and must contain 1..256 errors. Closed
exceptions are admitted only at roots; active nested values reject. Construction
state is excluded from storable domains. Source public clauses and application
VCs remain separate obligations. The default-candidate generator computes structural
eligibility and retains source default conditions without discharging them.

Zero checks use fixed Boolean expressions for up to 32 leaves; wider regions
partition selectors into groups of at most 14 while retaining a scalar word at
the bottom. Aggregate predicates, domain element readers, field getters and
key-relation element readers return selected cubes directly rather than
eta-expanding and reapplying them per output bit. Concrete transformer counts
and domain checks are unchanged. No new recursor, generic type, extensionality
assumption, axiom or checker rule is introduced. Capacity clamping is never the
explicit domain bound.

## Original sources and input representation

Five additional actual C# sources were captured through the frozen frontend in
the local isolated Linux image: string-key map, decimal-key map, decimal set,
compound-key map and a source product/array for the total-cell limit. Their
requests replay exactly. Two accepted user-exception sources and one rejected
nested-exception source were captured separately through the control-emission
harness. All 28 accepted source contexts are represented in the current pinned
domain corpus, with 160 domain-root occurrences after the contract-only Bool
extension, and maxima of 45,348 terms,
536 declarations and 8,576 transformers. Capture and accepted helper definitions
do not prove application VCs or replace root semantic observations.

String/compound maps use C33/C34 physical storage even for two short keys. A
sparse test input stores true Boolean leaf addresses without allocating every
false bit. Every demanded selector still executes in the generic core observer;
absent bits return false and no generated operation or predicate is recognized.
It does not skip padding validation. All-address dense/sparse observations at
depths 0..8 and high/mixed C40 addresses passed with delayed selectors (0.02
seconds). Six full sparse/dense map encoding comparisons over original-source
integer and decimal maps also passed (0.88 seconds).

## Passed checks and remaining verification

Exact chronological logs and hashes are in `unit-3-domain-progress.json`.
Earlier checker passes apply to their recorded bytes, not later generations.

- The current 28-context domain corpus generated and passed import mutations
  (50.03 seconds), then all 28 pinned certificates replayed exactly and passed
  mutations (114.16 seconds). Adding the two exception contexts preserved all
  26 existing certificate bytes and metadata. All 21 predecessor relation
  certificates also remained byte-identical and passed mutations (36.30 seconds).
- Small values across all 26 source contexts passed after direct getters
  (13.88 seconds). The earlier post-reader zero/padding pass included D20 across
  the 14-selector boundary (30.64 seconds).
- The unchanged aggregate core passed four tests, including all 16,384 words
  returning 65,536, odd counts, saturation and poisoned unused arguments
  (502.21 seconds). Its computed-predicate sharing regression passed: 1,967,080
  generic evaluator transitions versus 15,628,958 in the negative control.
- The aggregate plus 26 domain certificates before the final getter adjustment
  passed both unchanged checkers with zero axioms, and all 27 hash corruptions
  rejected (603.325 seconds). The aggregate bytes remain unchanged. The 26
  direct-getter domain bytes subsequently passed both checkers and corruptions
  (492.057 seconds). Both added exception certificates passed too (41.919 seconds),
  closing same-byte zero-axiom checks for all 28 current domain certificates.
  `unit-3-domain-certificates.json` records exact hashes and scopes.
- Before direct getters, the complete 16,384-unit UTF-16 source value returned
  16,385 cells and rejected excessive length and dirty length padding (637.29
  seconds). High sum tags, dirty empty payloads and unused product roles rejected
  in 18/8/2 cases (5.91 seconds). These affected cases were included in the seven-test source semantic run
  with three test threads; its UTF-16 and sum/unused-storage subtests passed.
- The seven-test semantic run terminated with SIGKILL; the cause is not
  established, and this is neither a complete pass nor a deterministic assertion
  failure. Its completed subtests remain in the retained log. The outstanding
  scope still includes exact 65,536/65,537 logical-cell values,
  collection/role bounds, decimal scale/signed-zero numeric duplicate rejection,
  descending and increasing order, string prefix-length comparisons, and compound
  keys whose equal strings force comparison of the next source field. Those
  outcomes are pending; input-encoding and helper certificates do not establish
  them.
- User-exception domain observations confirmed 2/5 logical cells for empty/mixed
  source payloads and rejected eight tag/payload/padding mutations (1.12 seconds).
  The nested source rejects with `exception_value_api`, zero artifacts and no
  facts. All eight source requests replay byte-for-byte (0.07 seconds); see
  `exception-domain-sources/README.md` for capture and proof-scope details.
- At the exception-coverage checkpoint, scoped clippy passed (10.84 seconds), format
  passed, and all five inventory tests passed again (15.95 seconds). The inventory has
  109 Std-namespace paths and aggregate count 4,932. Removing exactly the two new
  implementation paths restores the prior 107-path fingerprint; historical
  cache entries are unchanged.
- The subsequent defaults and finite-operation additions have a current inventory
  of 112 Std-namespace paths and aggregate count 4,935; all five checks pass.
  The contract-only Bool carrier extension preserves the previous changed
  certificate and verifies both versions with both checkers. Current pinned
  replay covers all 28 domain programs; see `contract-carrier-extension/changes.json`.

Superseded collection, total-cell, decimal and string/compound test processes
were explicitly stopped after diagnosed implementation replacements. Terminal
handles and preserved logs distinguish these stops from assertion failures.
No process was restarted merely because an observation interval expired.
Initial untagged Go/no-test and inaccessible default-cache attempts do not count
as verification; real runs use the checkeragreement tag and temporary GOCACHE.

## Observer cache correction

On 2026-09-11, the test-only evaluator was corrected to populate its existing
Boolean lambda cache when the body first demands a suspended argument. A fresh
closure regression fails before the fix; six observer, three aggregate, and
one original-source small-domain test pass after it, as do scoped lint/format.
No production certificate changed. See `unit-3-observer-bool-cache-review.md`
and `unit-3-observer-bool-cache-verification.json`. This does not close the
outstanding long-running semantic tests or establish their runtime's cause.

## Remaining work

Finish the four outstanding semantic tests and fix findings. On 2026-09-11,
the serial run in `/tmp/mpk-w09-domains-outstanding-semantics-v1.log` has passed
collection/rejection and decimal-order subtests. String/compound map ordering
is still evaluating its first duplicate string-map count; the exact total-cell
boundary subtest follows it. The live cargo/test PIDs are 31738/31739. The old
seven-test command is historical, and an unavailable session handle is not a
terminal verification receipt. The two new user-exception domains were verified
separately. Complete direct review
and record the exact verified scope before committing/pushing this component.
Default candidates and finite unit/parse-error/exception operations have separate
scoped verification in `defaults/README.md` and `finite-operations/README.md`.
Semantic constructors, collection mutations, construction state, outcome
operations and source-condition proof assembly remain open; bindings, controls,
transitions and application proof assembly remain units 4-8. No domain, unit 3
or W09 completion receipt is issued. The full repository gate is deferred to
T06-W12.
