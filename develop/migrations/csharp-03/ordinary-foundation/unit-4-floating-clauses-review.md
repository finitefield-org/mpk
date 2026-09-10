# Floating contract adapter direct review

The production change connects unary/binary floating and numeric.conversion
recipes to the existing ordinary emitter. Its bodies and names are unchanged;
only emitter visibility changes. Each alias validates exact nominal argument
and result types, ordered checks and the number of failure definitions. IEEE
arithmetic has no exception check. The two checked conversions have a single
overflow predicate, so the existing ContractFails mapping remains independent
of failure precedence. Operations with overlapping checks are not generalized
by this change.

Five native-source contexts cover all 44 operation IDs with exact alias targets
and complete declaration-dependency comparison. Eighteen runtime paths include
NaN, infinity, signed-zero equality/negation, four nonthrowing conversions, two
valid checked conversions and two overflowing checked conversions. All 46
attachments compile and their VC metadata/import linkage is checked. Result
metadata corruption rejects. This targeted test passed in 57.99 seconds.

The request helper can override the source body while retaining the original
callable signature and recomputing the raw source hash. All 17 existing integer
and fixed-codec contract pins remain unchanged after that refactor. The original
whole-double source exceeds the frozen limit and now has an explicit passing
Limit rejection regression. Positive double fixtures separate comparisons,
arithmetic and division using the recorded ordinary operation costs; no limits
or operation coverage were relaxed. Initial invalid comparison-tag and fixture
table/size assumptions are retained in the verification logs.

Scoped clippy, format, inventory and the Rust CLI build passed. All five source
candidates passed both checkers on identical bytes with zero axioms, report
agreement and hash corruption rejection (1316.119 seconds). The five terminal
PASS names, certificate and metadata hashes/sizes, tested copies, capture
receipts and retained log hashes were reconciled. The final component review
found no actionable findings. These helper definitions and finite observations do not
prove native method bodies or application VCs. Original units 3-8 and W09 remain
open; check-fast.sh is deferred to T06-W12 and no component commit is made.
