# W09 string contract adapter review (component in progress)

The unary/binary adapter reconstructs the closed-context operation signature,
requires exact argument/result/check equality, and chooses the nullable carrier
from the same canonical string signature registry as the standalone generator.
The per-compiler cache rejects cross-context and mismatched signature reuse.
No legacy string operation body changes. Fifty-seven generated DAG comparisons
cover the frozen signatures and both carrier variants of twenty-one operations.

Contract failed-check symbols are independent predicates. The old ordered
failure results remain unchanged; only index-range and search null-argument
aliases need additional unmasked predicates. Thirty-three observations include
both-null search arguments, absent receivers, index boundaries and output bounds.
The shared Clauses initializer regression preserves empty/invalid quantifier
body non-evaluation. These tests do not discharge native-body or application VCs.

Source integration review found two fixture issues: warning-producing unused
constant locals/possible nullable dereferences, and a CompareOrdinal call outside
the frozen framework surface. Fixtures now declare only used locals, initialize
nullable declarations with known non-null values, and use the supported
Compare overload with StringComparison.Ordinal. Null operands remain explicit
contract test values. The strict compiler diagnostic and API policies are intact.
Native capture and integration checks are pending; no clean final component or
W09 review is claimed yet. Original units 3-8 remain open.


The same-byte checker review exposed a production issue: the old literal slot
encoder built a linear mux chain for each active slot, causing Rust checker
stack overflow at 16,384 UTF-16 units. A larger test worker stack alone was
insufficient; that temporary harness change is removed. The encoder now uses
balanced low-to-high address selection above 256 slots and retains existing
small-literal bytes. New tests independently observe all 16,384 full addresses
and every padded/unused address in a 257-element partial carrier. Their results,
new source candidates and unchanged checker checks are pending. The two failed
construction candidates remain explicitly unaccepted until replaced and checked.


Final targeted verification passed. The balanced selector uses role bits in
least-significant-first order, keeps absent slots false at every level, and
preserves child padding and nominal carrier depth. All 32,768 independently
expected bit observations pass, along with the original scalar/deep-padding
pins. Default-stack source replay passes. Only the two construction certificate
pairs changed; all four remaining certificate/metadata pairs are byte-identical.
Twenty-four affected construction observations pass after the fix, combined with
43 unchanged observations for all 67 attachments and 18 definedness failures.
Both changed candidates pass both unchanged checkers with zero axioms, matching
reports/hashes, and rejection of hash mutations. Four identical candidates retain
their passing v1 checker cases; the overall v1 failure is not hidden or relabeled.
Final lint and scoped format checks pass. No actionable findings remain within
this component. This is not an original-unit or W09 completion review; units
3-8 remain open, and the full gate is deferred to T06-W12.
