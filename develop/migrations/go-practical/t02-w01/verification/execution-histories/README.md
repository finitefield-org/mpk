# Ordinary unbounded execution histories

This private mathematical component adds ten ordinary definitions and seventeen
checked theorems. A history is `Nat -> State`; its length is an unbounded Nat.
`Path` requires the actual transition relation on every edge before that length.
`Starts` binds the complete initial state. `SafePrefixes` quantifies over every
length and every history with those witnesses, including the zero-step prefix.
It is not a caller-supplied trace, an observation budget or bounded loop unrolling.

Actual Nat induction proves invariant preservation for every such prefix.
Head/tail/prepend laws preserve each edge, and mapping requires proof that the
state mapping preserves the transition relation. Partial correctness contains
both all-prefix safety and postconditions for every normal run. A `NormalRun`
needs its own start, path and normal-completion witnesses. `ObserveNormalRun`
derives safety and postcondition observations from that same history and endpoint.
Partial correctness alone provides no termination witness.

The test models prove identity-state invariants, partial correctness, complete
normal runs, mapped paths and prepended paths at every arbitrary length. Another
model has checked partial correctness and a checked proof that no normal run
exists at any length. A complete true/false/true history has valid transitions
and a safe endpoint, while its unsafe middle-state claim rejects. Invalid edges,
unsafe entry and fabricated normal completion also reject. A valid additional
mathematical theorem passes both kernels but fails this exact component import.

[Both unchanged full checkers](paired/comparison.json) compare all fifteen actual
streams, including the failed proof candidates: five zero-axiom acceptances and
ten genuine type rejections. [Independent statement inspection](statement-review.json)
reconstructs all ten complete definition bodies and seventeen complete theorem
types, with the exact 27-member export set. [Verification](verification.json)
records 2,370 complete original declaration comparisons, resource measurements and
the actual successful and failed epochs. The two originally passing Rust roots
remain evidence; only the failed control root reruns after its test-proof fix.
Strict library/test Clippy and changed-file format checks pass.
The largest whole certificate has 165 declarations, 3,628 terms and 34,108 bytes,
within the unchanged 4,096/131,072/2,097,152 caps and 1,024 conversion fuel.

All complete prelaunch source bytes and modes remain in each epoch's input
archive. The original syntax/borrow failures, rejected endpoint constructions,
observer syntax failure and comparison protocol failures retain their real
inputs and outputs. Explicit equality congruence proves the original alternating
history's endpoint without changing that history or a checker budget. The Go
comparison recognizes its existing omitted-zero axiom count and human core-error
detail; original reports remain untouched and completed executions are reused
by exact inputs, without rerunning them. [Scoped review](input-review.json)
checks all 16 source epochs and 15,987 complete preserved input members with
zero new or basis findings and valid current links. It keeps the existing 127
rules, four scopes, 39 seals and 51 exclusions. The first review correctly
rejected a link to its own not-yet-created output; its complete original driver
and observed tool diagnostic remain in [that attempt](review-first-attempt/observed-result.json).
Creating the output placeholder lets the unchanged reviewer finish.
Raw producer stdout/stderr keep their exact original bytes, including final
blank lines; whitespace checks apply to all other staged files.

`State`, the step relation and predicates are mathematical parameters of this
library. They are not Go semantic extension points. Independently reconstructed
closed Go and IR transitions, source/IR simulation, complete storage/frame and
callee proofs, all 169 program demands, the complete 56-family foundation and
private descriptor/registry/context remain required. No source program or
practical profile is promoted. Native Linux release gates and remaining T02-T11
work remain due. This intermediate W01 check does not run the cumulative T02
gate, deferred to final W04: `./scripts/check-fast.sh`. Go/Rust only.
