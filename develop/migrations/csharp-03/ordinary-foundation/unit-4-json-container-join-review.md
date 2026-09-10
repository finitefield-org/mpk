# JSON implicit-container cell join review

The existing ordinary Child emission has been extracted without changing its
circuit gates, name, placement or metadata. The full existing seven-product
source/semantic program regenerates identical pinned metadata and certificate
bytes, protecting its prior runtime and dual-checker results.

ChildContainerPayload validates both C7 headers before accepting the join:
positive cell counts at most 16,384, end cursors at most 1,048,576, valid bits
and all reserved zeros. The parent must not be at EOF, and the child cursor
must strictly advance. The output carries the child's EOF/cursor and zeros
all 128 bits on failure. These are the same cursor obligations as Child;
full document/ending consistency is still checked by the caller's Finish.

For an implicit JSON wrapper, the accepted cell total is parent + child - 1.
Subtraction occurs on the child before the bounded parent addition. This
admits an empty wrapper with child count 1 and zero payload contribution,
including a parent already at 16,384. It also admits parent 1 with child
16,384, avoiding a prematurely rejected intermediate sum of 16,385. A
zero-cell or oversized child remains invalid. Since valid parent >= 1,
a valid total implies child <= 16,384; retaining the child's bound does not
exclude a valid sum. Arithmetic remains in ordinary finite circuits.

The 99 new complete C7 packet observations cover the sum boundaries, empty
contributions, invalid count/overflow inputs, cursor bounds/advance, EOF and
reserved bits. Existing grammar header boundaries and role/payload assembly
also pass. These finite observations do not discharge universal program
propositions. The new helper certificate is checked separately with both
unchanged checkers; it contains 46 declarations and zero axioms. Exact
checker/report and mutation results are retained in the progress receipt.

Direct review found no actionable issue in this helper. Caller integration
is explicitly incomplete: only actual implicit Transition events arrays and
anonymous map entry objects may remove this cell. Their role bounds, shape,
order, raw depth, complete storage, surrounding sum and full boundary rules
remain to be implemented. Ordinary source products, standalone ordered_entry,
and Money continue to include their own product cell. No component-only
commit is made, and this does not complete W09 or unblock W10.

The test-enabled lint run exposed two existing test-only warnings. Replacing
a single-reference clone slice with slice::from_ref and indexing a fixed
prefix through enumerate().take(9) preserves the tested inputs and coverage.
Their unchanged domain/runtime matrices are not rerun for these style edits.
