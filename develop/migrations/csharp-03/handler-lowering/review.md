# CSHARP-03-T04-W05 review

Baseline: `80ac614043395370fb733d9c4ddc67a220e7a4a4` (T04-W04).

## Scope and representation

The opt-in `allowHandlers` path emits the private
`mpk.csharp_practical.t04_w05.handler_lowering.v1` schema. It carries normalized
source/facts and the existing sequence handoff, every reachable callable's
register CFG, and a handler graph per callable. Contexts distinguish try,
filter, catch and finally execution. Search transfers list typed catches in
lexical inner-to-outer order, their optional filter entry, and the exact finally
entries to execute after that candidate wins. Escape transfers retain cleanup
for propagation. Normal/return/break/continue transfers retain their original
value and target. Filter-result and finally-resume nodes have explicit possible
continuation edges; the selected continuation is retained in control state.

The native resumable planner uses the existing closed exception universe and
ancestry. It validates search/unwind tables against their contexts, checks
region-entry contexts, Boolean filter-result operands, rethrow scope and all
routing destinations, and emits explicit filter/selection/propagation steps.
A stack search holds callee cleanup until caller filters have completed. All
frames search for the same original exception value. A thrown filter exception
has its own search/cleanup and a `FilterFalse` boundary; it cannot replace the
original search subject. A finally that completes normally preserves the
pending completion; one that throws starts a new search and replaces it.
The selected try's own finally executes after its catch completes.

Source validation restricts catches to admitted closed types and catch-local
uses to immutable stored payload members of the exact source exception.
Identity, Message, StackTrace and escaping/rethrowing a catch variable reject.
Bare `throw;` instead names its active catch's original value. Boolean filter
validation checks transitive source calls/getters for writes beyond their local
state, and direct filter assignments/increments reject. Compiler diagnostics
reject return and outward break/continue/goto from finally; loops wholly inside
finally remain admitted. Non-exhaustive switch fallback uses the existing exact
SwitchExpressionException arm and participates in handler search.

W06 still owns independent whole-VIR emission/import, source-map closure,
complete data-operation linkage, and construction-state composition. The W05
handler graph is a private staged representation, not an accepted VIR artifact.
Unique array construction across handler boundaries retains an explicit W06
handoff rejection. T06 owns proof discharge, including exact throws-set and
catch-or-unreachable obligations. No runtime exception identity, installed
frontend, profile/registry activation or workflow was added.

## Review corrections

- Handler search and unwind are separate passes. The call-stack planner keeps
  inner cleanup pending while an outer filter evaluates, including when the
  selected handler lives in the caller rather than in the throwing function.
- A filter's throwing helper still executes its own finally before the filter
  failure is discarded. Nested filter failures retain the original exception.
- Rethrow resolves the nearest active catch and preserves its original payload
  even after local state changes. An active-catch value with an incompatible
  closed arm rejects. Finally overrides abandon only exited cleanup
  continuations; a catch nested inside the same finally preserves its parent.
- Catch payload reads use a dedicated non-throwing `exception_payload`
  instruction; they do not treat the caught tagged exception as an ordinary
  object receiver or introduce a spurious null-reference outcome.
- The filter-result edge inventory includes subsequent filter entries, not just
  catch/finally entries. Native validation checks continuation inventories and
  rejects removed/reordered candidates and modified unwind suffixes.
- Filter mutation negatives reach the purity gate itself; unreachable trailing
  return statements were removed from those test sources so compiler warnings
  cannot mask the intended rejection.
- A stack search cannot substitute an outer frame's exception type or value for
  its innermost original subject. Protocol tests cover unknown filter exceptions,
  wrong result timing and reuse after selection.

## Evidence

The retained corpus has 42 source cases: 29 private handoffs and 13 artifact-free
rejections. Each positive executes five inputs, yielding 145 original CLR runs
and 145 instrumented CLR runs with identical outcomes. The Rust register
interpreter compares results, closed exception types, immutable Code payloads,
and every recorded try/filter/catch/finally event. Source helper calls execute
retained CFGs; the fixture's simple immutable Code construction uses the
existing T03/W04 constructor model. Whole construction ownership remains W06.

Six task tests cover source/trace differentials, more than 100 routing and type
mutations, filter protocol and cross-call subject retention, all completion
alternatives, source rejection barriers, and corpus/input hash bindings.
W04's 39-case replay remains byte-identical. Legacy T03 replay, frozen package
checks and the standard local gate are recorded in `verification.json`.

Latest task review: no findings.
