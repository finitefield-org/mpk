# String construction runtime correction (2026-09-20)

The user reported a 600.23-second timeout in the first non-null `cs`
interpolation case. This was an evaluator deadline, not a semantic rejection.

The constructor now binds the output index and its range predicate before the
four UTF-16 bit selectors. The result's success guard retains the projected
code unit, so these bindings survive while its sixteen bits are compared. A
direct unsigned comparison of the fourteen address bits with all thirty-two
length bits replaces repeated scalar-circuit projection. Index bits above 13
are zero, so an upper length bit makes the comparison true, preserving the
unguarded Value expression as well as bounded Result behavior.

The review checked the de Bruijn offsets in both branches: the header still
selects the original five length bits and rejects all other header addresses;
the role selector and input variables account for the new lets. The false
success branch still returns zero lazily, without demanding an invalid Value.
The full C19 storage comparator, all operand cases, output bound, ordered
failures, and last-physical-bit mutation remain unchanged. Temporary generic
evaluator-cache experiments were discarded; the evaluator is byte-identical
to the pre-investigation source snapshot.

A regression projects and retains one C4 code unit while measuring each bit.
It checks isolated surrogates, embedded zero, active and inactive units and the
last physical position. Each subsequent inactive bit costs 102 transitions;
the direct comparison also reduces first-bit work at index 16383 to 1417.
The assertion bounds each subsequent bit, rather than requiring their sum to
be smaller than the now much faster first bit. This catches the original
per-bit reconstruction while allowing the direct comparison improvement.

The binder cap remains 256. Moving nested selectors changes the maximum
accepted all-char argument count from 231 to 234. The boundary test accepts
65/234 and rejects 235/257; the changed boundary certificate is regenerated.

Fourteen certificate byte sequences changed: eight constructor fixtures, two
string contract contexts, and four string data contexts. Their previous bytes,
metadata and manifests are retained in `previous-pins/` beside the verification
receipt. Historical checker receipts apply to those preimages. Candidate
reconstruction checks and checker runs are scoped to the changed families;
basic/ordinal and lifted certificates are unchanged and are not rechecked.
The user completed the 44-case oracle in 30.63 seconds and the final binder
checker in 21.53 seconds. All 25 remaining runtime partitions also passed.
Imported logs and the disjoint 48-signature/320-observation coverage audit are
in `verification-logs/string-data/user-long-completed/verification.json`.
These measured sub-minute regressions are agent-owned for future changes.

The completed receipt closes this component’s pending test list. Intermediate
45/52-second diagnostics and rejected cache experiments are historical evidence,
not completed runtime verification. This correction does not close W09 or unit 5.
The repository-wide gate remains deferred to T06-W12.
