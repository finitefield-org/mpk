# Aggregate reassociation for shared decimal/JSON generation

Scope: a component of W09 units 3 and 4. W09 and units 3–8 remain incomplete.

The unreassociated aggregate pipeline exceeded the cumulative transformer
budget when added after the full collection and JSON definitions. The old
collection/JSON count was 13,002; decimal emission reached the limit while
attempting to add its 8,192-step aggregate pipeline. The reproduction failed
with `OrdinaryCarrierError::Limit`, not a checker proof rejection.

The changed aggregate pipeline groups four original `StepTwo` applications
using the original guarded `Compose`, then composes 2,048 `StepEight`
applications. The expanded order remains 8,192 two-cell steps. The Builder
charges the four group occurrences and all 2,048 outer occurrences; no counter
is reset and no limit is raised. Standalone aggregate generation now uses 2,077
transformers. Complete collection/JSON/decimal generation uses 15,434, with
107,519 terms and 1,696 declarations.

Direct review checked binder indices, both modes, the original `Active`
short-circuit, odd-length second-cell guarding, final clamping, cumulative
counting and idempotent emission. Guarded composition is associative: after
the first function, either bracketing returns immediately on an inactive
result; otherwise each executes the second and conditionally the third.

The regression audit expands the actual term tree and checks every call's
mode/predicate/length arguments and the total of 8,192 `StepTwo` occurrences.
Only that validated tree is normalized in a test-only certificate clone, then
all old named declarations and their transitive dependencies are compared.
Mutations to group length, helper bodies and all three step arguments reject.
The combined-certificate audit also preserves every old collection/JSON
definition and every standalone decimal definition after reassociation.
These audits do not discharge universal application VC propositions.

The initial search covered only the `emit_aggregate_fold` alias and missed
direct module calls and transitive consumers. The corrected scope includes
twelve additional component families identified by call sites and actual
certificate declarations, alongside the previously checked integer/decimal
parsers, core group edges, maximum capacity and consumer inventories. All
affected families require source regeneration and definition preservation. Unchanged
JSON/hex/calendar/formatter runtime suites are excluded. Changed certificate
bytes still require both unchanged checkers, zero axioms and hash corruption
rejection. Details and terminal/pending status are recorded in
`unit-4-aggregate-step-eight-progress.json`; runtime and checker completion
must not be inferred from this source review.

The consumer-inventory finding is resolved: all additional affected pins were
regenerated, their old definition closures and metadata preserved, and all 122
distinct changed byte vectors in twelve families passed both unchanged checkers,
zero-axiom checks and hash-corruption rejection. The retained terminal log and
its verified hash are recorded in the progress receipt. Outstanding domain
runtime cases remain separate. This is not the final W09 review. The full gate remains
deferred to T06-W12. No component-only commit or push is made.

Verification completion: all 13 distinct parser vectors and the core/shared vectors passed both checkers; all three aggregate runtime cases passed in 777.48s. Source/preservation/import/limit/consumer/lint checks are complete. The additional 122 distinct consumer vectors have now also passed; this does not close the remaining domain runtime or W09 requirements.

Correction history: the call-site search omitted direct `aggregate_fold::emit_fold` calls and transitive structural consumers. The statement that only integer/decimal consumers were affected was incorrect. The subsequent regeneration, preservation and dual checking now cover those consumers as well. See the affected-pin inventory and corrected progress status.
