# Direct review: W09 unit 3 ordered finite fold component

Scope: concrete ordered state transformers, First/Any/All wrappers, reconstructed
source index widths, exact import, regression corpus and checker registration.
No review skill or subagent was delegated. Unit 3 and W09 remain incomplete.

## Finding addressed

The first unsigned comparison expression referred to the accumulated low-bit
comparison in two branches at every bit. Although the stored term DAG was small,
ordinary axiom-reference traversal expanded that syntax exponentially. The old
core/source test processes (PIDs 34040 and 34966) were confirmed live, explicitly
terminated because the implementation was superseded, and their logs retained.
This was a diagnosed construction problem, not a timeout verdict or checker
failure. Replacing the expression with one reference to the low-bit result fixes
it without changing the core or either checker. A regression counts expanded
syntax using memoized arithmetic before certificate building, requires fewer
than 8,192 visits for this 32-bit comparator, and checks 256 boundary pairs.
Reverting to the original doubled expression fails that regression promptly.

## Final review

No remaining actionable findings in this component. Reviewed de Bruijn scopes,
word/field selector order, unsigned comparison and increment carries, the concrete
C6->C6 step type, predicate/length value binders, balanced left-to-right composition,
intermediate stopping, first-result preservation and the second-read length guard.
The state contains only the current u32 index and first nonzero i32 word. Neither
Bool nor Nat elimination returns a function. Two ordered reads are a finite
circuit inside one transformer; every one of the 8,192 transformer occurrences
is charged before DAG sharing. All wrappers share the same closed pipeline.

First preserves the first nonzero word, including its sign bits. Any/All map
Boolean predicates into a word and implement false/true on empty input. Index
widths 0..14 clamp count to their capacity; depth 15 rejects. The last element at
16,384 is observed and the second read at odd count 16,383 is excluded. Those
actual core executions completed in the same corrected run, alongside smaller
truth/order/carry tests. Budget accounting separately accepts 16,384 cumulative
transformers and rejects the next occurrence; the emitted fold certificates
contain 8,192, not 16,384, transformer occurrences.

The generator scans all independently reconstructed reachable array/sequence
shapes, following the complete carrier inventory rather than only invoked calls.
Original captures cover sequences, validation's shared 4,096-slot carrier,
ordered maps, UTF-16 at 16,384 and a scalar source requiring no pipeline. Source,
foundation, rule/capacity, count metadata and bytes are regenerated at import.
Same bytes pass both unchanged checkers with zero axioms; corrupted hashes reject.
The consumer inventory passes without a fingerprint update.

Count clamping does not prove a source length or validation error count is valid.
Callers retain domain and role-bound obligations. These are common iteration
helpers; element relations, complete structural/collection equality/order,
recursive domains and application proof assembly remain open. Concrete comparison
requirements discovered during review are recorded in unit-3-comparison-notes.md.
Targeted commands/results are in the verification receipt. check-fast.sh stays
deferred to T06-W12.
