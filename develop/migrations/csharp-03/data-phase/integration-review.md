# W14 integration review

Task: CSHARP-03-T03-W14 is complete. The final task review has zero findings. The baseline is codec prerequisite commit
`a59bd66da8078e9ce4f7c3cb991cca5b04ff5271`, following W13 commit
`78c8f7295f75baf3ea0efc68c684d31d95e6bc46`.

## Implemented scope and review corrections

Actual selected source and sidecar bytes feed the pinned compiler. The handoff
retains the compilation/input inventory, original source/body hashes, spans,
logical declaration identities, exact reachable type facts, original operation
ordinals and owning typed recipes. Native validation independently reconstructs
closure, defaults, signatures and source provenance before emission.

All 33 frozen contract expression forms attach strictly. Complete canonical
contracts and source obligations participate in VIR hashes and survive as
pending VC subjects. Native import reattaches them against the original inputs;
recomputing outer hashes cannot hide dropped contracts, fabricated obligations
or stale source facts. Exact family/kind validation rejects unknown obligations.
The sole T02 engine derives concrete definitions and roots, including shared
structural equality, applicable ordering, codecs, nullable and business routes.

The approved binding/error/commutation amendment retains exhaustive enum-arm
maps, returned Result error routes, explicit rounding enum operands and ordered
unmatched source exceptions. ErrorOutcome checks are carrier-free; ParseError
and Exception checks retain their actual carriers. Source helper bodies remain
ordinary functions. Intentionally incorrect instant/money helpers retain
unresolved commutation VCs and are never reported as proved. Foundation/profile
producers and downstream operation/VC linkage fixtures were regenerated.

Source arrays use the existing engine-expanded construction operations with
linear SSA ownership and the shared default/completion relations. Stores retain
address/value/write order; compound updates retain address/read/value/write.
Branches merge versions of the same origin, and publication consumes live
construction state. Exact normal and exceptional discard validation rejects
missing, duplicate, forged or stale cleanup. The concrete T02 action model
remains independently covered.

The approved constructor transaction amendment resolves the required-member
handoff described in `construction-transaction-amendment.md`. Private constructor
execution transfers an owned transaction; Missing slots have no public payload,
nullable None is assigned, and only finalization creates the source owner.
Original logical signatures and contract subjects remain intact. The importer
checks actual W05 plans, constructor Must/May summaries, delegation, early
returns, same-origin/current-version writes, finalization and exceptional
transfer. Nested initializers preserve separate transaction identities.

Exhaustive replay found and corrected getter obligation IDs, synthesized
constructor declaration-root retention, foreign nested-initializer receivers,
nullable annotation widening, null patterns, parameter/discard assignment,
interpolation shape signatures and accepted-limit stack use. Write-only property
targets do not create getter calls. W03 getters and W04 synthesized constructors
remain declaration proof roots with original source provenance. Interpolation
uses one existing relation after source-ordered part evaluation. Source-call DAG
scheduling and a bounded worker stack handle the admitted deep/call-chain cases.

Rehashed mutations cover contracts/bindings, source facts, routing bypass,
constructor protocols and plans, premature publication, transaction swaps,
stale SSA state and lost cleanup, with unchanged positive controls. All nine
frozen builtin data exceptions retain independently validated values, checks
and exceptional successors. Explicit source exceptions/handlers and loop
positives remain T04-owned; boundary invocation remains T05-owned, proof
construction T06-owned, and installed activation T07/T08-owned.

## Actual-source coverage

`data-stage-replay.json` records 777 distinct original source selections from
all 13 existing T03 stage harnesses. Their original assertions also execute;
typed internal handoff variants continue to exercise their production owners.
The real data frontend then processes every captured source selection:

| Result | Cases | Evidence |
| --- | ---: | --- |
| Emitted and independently imported | 403 | Complete source facts, original bytes, native import |
| Owning source/sidecar rejection | 372 | Compiler rejection with `artifact_count: 0` |
| Owning native resource rejection | 2 | Exact W08 33-allocation and 9-live-allocation negatives |

The resource negatives have the frozen `CSHARP_PRACTICAL_VIR_LIMIT` diagnostic.
Their 32-allocation and 8-live positive controls are among the 403 imports.
Transport/reference failures before source observation still execute their
original stage assertions and do not masquerade as accepted source fixtures.

`data-source-cases.json` adds 79 selected-source regressions, including twelve
fixed unary fuzz seeds (0, 1, 2, 3, 5, 8, 13, 21, 34, 55, 128, 500). Actual sidecar
requests/responses cover 21 attachment cases. Eight additional constructor
requests/responses cover required string/array members, implicit constructors,
nullable defaults, delegation, early returns and constructor/initializer
exceptions. All eight pass actual emission and independent import; rehashed
protocol and exception-cleanup mutations reject.

## Final verification

The final pinned C# input manifest SHA-256 is
`f4bb313adcac648e13b576af7ade0037de225d9abf47d8028379ed0934e4f50b`.
`verification.json` records exact fixture hashes and final gate outcomes.
The W14 native suite has 16 passing tests. The macOS availability guard is not
counted as execution of the Linux-only test: explicit offline linux/amd64 runs
supply that evidence, with fresh isolated compiler builds and exact byte
comparison on each run.

Both isolated pinned runs passed the full stage replay, source matrix/fuzz,
source default, actual sidecars, constructor requests and every legacy frontend
fuzz seed. Legacy subset/contracts/lowering/emission, frontend `--check`, private
`--check-build-inputs`, and `./scripts/check-fast.sh` passed. Final generated
artifact checks and the complete task diff review passed. T04-W01 is now ready;
no installed profile or public production route is activated.

## Historical inventory cache correction

The 17 pre-existing search-fingerprint failures were traced to Python bytecode
cache paths accidentally present in the T01-W02 capture. For every failing row,
adding the exact recorded `scripts/__pycache__/*.cpython-312.pyc` path set to the
consumer paths reproduces the original count and SHA-256 exactly.
`historical-inventory-cache-correction.json` binds those path sets to the
unchanged original inventory bytes. No historical fixture/hash is rewritten,
and no bytecode cache is recreated or treated as a consumer. The validator
excludes generated caches and reconstructs the historical path-set preimage
only for frozen-hash comparison. Real consumer additions/deletions still fail.
Earlier failed check-fast runs remain failures; the final gate is recorded
separately. The first final gate's test-only `useless_vec` clippy finding was
fixed before rerunning the standard gate.
