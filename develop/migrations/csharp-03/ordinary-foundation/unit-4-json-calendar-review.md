# Calendar JSON token component review

This component adds reachable date/time token definitions to W09 unit 4.
It does not complete typed object/collection grammar, native/transition
relations, universal propositions/proofs, or W09 acceptance.

Both standalone and structural boundary generators select calendar codecs from
independently reconstructed concrete carriers. Date requires the exact C5
32-bit day carrier and time the exact C6 64-bit tick carrier. Empty calendar
metadata is omitted, preserving unrelated serialized programs. Existing shared
JSON helpers retain their definitions and budgets. The original calendar
parser definitions are emitted into the shared Builder.
Formatters retain their complete fixed-width semantics with the cheaper
conversion described below.

Each wrapper binds the existing C20 string frame once. Under that Let, Var 0
is the frame and Var 1 the original ending. It projects the C19 UTF-16 text
into the original calendar parser. A successful token requires the original
frame validity, the closed parser's zero success tag, and ending below four.
The C7 result preserves the whole flag, every day/tick payload bit (zero
extending date), absolute end, and zero padding. Every invalid result is zero.
The frame remains responsible for canonical JSON spelling and original cursor,
UTF-8 and document bounds; a failed frame cannot become a valid calendar value.

Four real C# source captures cover date, time, both, and both plus decimal.
Source tests compare complete calendar -> JSON -> structural declaration
closures, validate cumulative limits, exercise import tampering, and pin exact
programs/certificates. Runtime tests observe every bit of 30 C7 packets,
including Gregorian leap rules, endpoint values, seven fractional digits,
invalid ranges, absolute offsets and all admitted delimiters.

Review checked carrier selection, closed-sum role/bit addressing, Let indices,
result padding, failed-frame gating, ending validation and serialization of
empty metadata. Source compile failures in the test helper were corrected by
using the public VC and VIR APIs. Verification results and unresolved work
are recorded separately in unit-4-json-calendar-progress.json. No passing
runtime or checker result is inferred from generation or this source review.

Only the new composition and affected source/inventory/checker tests are run.
Existing calendar semantics remain protected by full dependency-closure
comparison; unchanged vectors need no repeated checker invocation. The full
check-fast gate stays at T06-W12. No component-only commit is made.

The cumulative-limit finding is resolved in generated programs: the actual
`date-time-decimal` source now generates all definitions at 243,567 terms,
3,199 declarations and 8,490 transformers. All four source contexts pass
complete calendar -> token -> structural definition closure, imports and
mutations. The current eight JSON/structural, three calendar-codec and one
shared collection vector are pinned exactly; ten distinct byte sequences are
accepted by both unchanged checkers.
No new checker acceptance is inferred from source generation.

Formatting uses two reductions. The constant divider maintains a remainder
strictly below the divisor and keeps only the needed remainder width; an
initial prefix below the divisor has an identically zero quotient bit. Tests
compare every output bit with the original divider and independent arithmetic
on 14,002 cases, including full-width high-bit and quotient/remainder edges.

Decimal output uses fixed-width binary-to-BCD conversion. Before every binary
shift, each nibble is in 0..9. For nibble bits d3..d0, let l=d1 OR d0 and
q=d3 OR (d2 AND l). The add-three adjustment has bits
(d0 XOR q), (d1 XOR (q AND NOT d0)), (d2 XOR (q AND l)), q.
These are the exact adjusted values for all ten possible digits. Shifting
passes the decimal carry to the next nibble and preserves the 0..9 invariant.
Discarding the top carry computes modulo 10^digits, matching the old fixed
number of restoring divisions even for out-of-domain 32/64-bit inputs. All
required codec definitions remain emitted; none was removed to fit the limit.

The reduced BCD circuit matches original restoring conversion and host digits
on all 4,096 12-bit inputs plus 32/64-bit bit/decimal boundaries. The 68 affected
ordinary formatting and parse(format) cases pass exact independent text,
header, padding and restored-value observations. The prior 30 full C7 token
cases passed; all 177 token roots and their 2,325 dependencies are exactly
preserved after format changes. Original parser-only matrices are reused via
full archived parser-closure comparison across 67 source contexts.

The shared collection/calendar finding is also resolved in generation and
semantics. Calendar FormatCharacters directly emits up to 128 Boolean gates
per state transformer. Each gate is still computed in its original order;
references within the current block use the preceding let-bound gates, and
references to earlier blocks read the immutable input state. The aligned input
prefix adds only false padding. All later gate IDs and output roots are shifted
by the same amount before pruning; input IDs and function argument widths do
not change. Block addressing selects seven low bits and preserves the rest of
the state. Each resulting transformer is charged to the original cumulative
counter. No previous transformer calls are hidden in uncharged wrappers.

Unrelated callers retain 32-gate emission. Exact standalone decimal and shared
collection/JSON/decimal bytes and counters pass preservation checks. Alignment
tests cover all gate kinds and root/dependency preservation across five input
widths and three supported block widths. The 68 affected core formatting cases
and full previous JSON token dependencies pass again after the block change.
The shared maximum-capacity collection/calendar/JSON vector now validates at
207,455 terms, 2,780 declarations and 16,293 static transformers.

The earlier 32-gate checker batch was intentionally stopped after this change
superseded its bytes, retaining the two completed calendar-codec checker cases
as historical evidence. It was neither a timeout restart nor a proof rejection.
The former vectors are archived under previous-wide-blocks; the current batch
checks the ten distinct new byte sequences. Source review found no remaining
actionable issue in these changes. All ten distinct current vectors passed both unchanged checkers (terminal
session 76173, exit 0; retained log in the progress receipt). Typed JSON
grammar and the remaining W09 work are still open; this is not a
completion review for W09.
