# SSA definition points in the source-step observer

The [first complete runtime attempt](../normal/result.json) fails after5,476.090
seconds at the added memory-binding comparison. Its [exact source and executable](previous-attempt-snapshot.json)
remain preserved. The [decoded operands](binding-origin-analysis.json) contain
eight bindings: four source snapshots already match, and four SSA bindings
differ only in `node_id`. The added observer copied invocation use sites, while
the step correctly uses the original VIR definition points. This is a test
observer error; production semantics and all42 original inputs stay unchanged.

The expected binding now reconstructs each unique SSA definition from original
VIR parameters, phis, literals, invocation normal results and handler exception
values, matching both value ID and type ID. It does not take the expected node
from the actual step. The existing independent SSA-origin assertions remain.

The original `run_steps` entry hardcodes the shared observer's runtime argument
to true. Every original runtime example, scalar computation, literal forgery,
component conjunction, argument/bit mutation, positive/negative check, strict
import and all36 exact current JSON/certificate goldens remain. Two existing
`StepMetadata` consumers additionally call the same observer with false, ending
before runtime evaluation. This checks metadata early without claiming runtime
validation or providing an environment switch for the original owner.

The [source audit](source-preservation-audit.json) reverses these additions and
recovers the entire original step source and the committed145675 harness byte
for byte. All112 original step assertion calls and329 existing harness assertion
calls remain; nine assertions are added in the step file. All42 original input
members and the three relevant production sources match source320 Git bytes.

The [build](build/result.json) passes in30.721 seconds with test optimization1,
debug assertions and overflow checks. The [existing full metadata owner](metadata/result.json)
passes all eighteen contexts and582 complete source steps in6.680 seconds,
including the exact same memory/constructor/SSA component checks. The
[dedicated constructor metadata owner](constructor-metadata/result.json) passes
its original41-step type context in0.650 seconds. These counts describe metadata
checks; they do not establish runtime or application proofs.

[Format](format/result.json) and [target Clippy](clippy/result.json) pass in2.585
and4.046 seconds. The [complete original runtime retry](normal/result.json) is
passed in10,017.322 seconds with all eighteen contexts and original bounds, no output flags, context
filters, stack overrides or core ceilings. The [verification record](verification.json)
keeps the failed previous attempt, terminal metadata/static checks and running
full retry separate.

The user now explicitly defers C# tests. Final changed-input review and the
cumulative `check-fast.sh --skip-csharp` Go development gate remain required. W05/T01 is open and T02 is unstarted. All987 C# application IDs and169
Go demands remain pending; no profile is activated.
