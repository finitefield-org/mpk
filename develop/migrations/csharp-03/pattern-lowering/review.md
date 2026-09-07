# CSHARP-03-T04-W03 review

Baseline: `4c4030235fdb5f82aa3fc5b3df2b40727ab0b21e` (T04-W02).

## Implemented boundary

`PracticalPatternLowering.cs` extends W02's private register/slot CFG. The
explicit `allowPatterns` route consumes the existing syntax, exact type,
immutable construction, array/sequence and loop-contract stages. Its wire
schema is `mpk.csharp_practical.t04_w03.pattern_lowering.v1`. The W02 entry
and schema remain separate. Neither route is added to the installed csproj.

The governing expression is evaluated once. Switch expression arms and
statement case guards are tested in source order. A textual `default` is
selected only after every non-default candidate fails, even when its section
appears first. Shared statement sections, nested switches, loop continues,
switch breaks, returns and throwing guards retain their actual destinations.
All expression switches retain the pinned closed
`System.Runtime.CompilerServices.SwitchExpressionException` fallback. A
Roslyn exhaustiveness observation never substitutes for proof of an
unreachable fallback. Statement non-match goes to the switch exit.

Decision opcodes retain their original normalized operation ordinal, exact
closed type, constant bits, operators and symbols. Constant matching uses C#
constant-pattern equivalence: ordinal UTF-16 string content, enum/scalar
values, nullable null, decimal value equality, and NaN matching NaN. It must
not be replaced with the ordinary floating-point binary equality operation.
Relational patterns fail on null/NaN and use the admitted scalar relation.
`and`/`or`/`not` have explicit short-circuit edges; parentheses preserve the
original tree. Exact type patterns need only their admitted sealed/value
family and null test; object, dynamic and open inheritance remain excluded.

A successful `pattern_bind` explicitly transfers the incoming value to a
source-identified local with its narrowed type and a distinct result value.
Each arm's declarations have separate source-derived identities, and failed
patterns cannot enter their successful binding path. W06 maps these private
parameter transfers to the final SSA/block-parameter representation together
with W02's existing local state. No caller-supplied execution or source-text
interpreter is embedded in the producer.

List patterns admit exact rank-one SZ arrays with fixed lengths and no slices.
Null and length tests precede indexed reads. Property paths are resolved field
or getter symbols. Data declarations enforce immutable storage and pure source
closure; getters must additionally appear in the independently supplied exact
`totalGetters` claim set. Duplicate, missing and unused claims reject. Native
attachment checks that retained claims equal its separately supplied set and
refer to captured callables. These are pending totality claims, not proofs:
T06-W04 owns their semantic discharge, including termination/exception freedom.

Pattern array aliases freeze the source storage. Switch selection/guards hold
a read borrow on the governing array; arm bodies release that borrow while
preserving any enclosing foreach borrow. Guard writes that could change cached
list reads therefore reject. The existing internal borrow diagnostic is
`active_foreach_read_borrow`; both new negative cases must reach that exact
barrier. W06 still owns final ownership composition and construction-state
revalidation across complete control bodies.

`prepare_pattern_lowering` reuses phase-0 budgets and W01 contract/provenance
attachment. It validates the required function inventory, normalized-body hash
linkage, decision inventory, operand arity/source-kind linkage, list lengths
and indices, successful binding destinations, register dominance, loop targets,
getter claims and exact unmatched exception sites. Source regeneration also
rejects candidate changes to guard successors, binding slots and unmatched
exception tags. W06 owns independent whole-control emission/import; T06 owns
proofs. This handoff has zero frontend-success, VIR or certificate artifacts.

## Review and fixes

- Non-exhaustive switches were stopped at diagnostics, synthesized-IL
  inspection and normalization recompilation. Only W03 handles CS8509/CS8524,
  and always retains the modeled fallback. Synthesis inspection downgrades
  only those two diagnostics on its temporary emission clone. All other
  diagnostics and the fixed source compilation options stay enforced.
- Existing capture/normalizer order tests used the previous exact call spelling.
  Their anchors now include the W03 opt-in argument while preserving every
  diagnostic/declaration/normalization ordering assertion.
- Roslyn erases annotations on pattern input/narrowing and implicit receivers.
  W03 derives null-test/narrowed annotations from exact admitted pattern types
  and declared locals; default normalization and W01 identity behavior remain
  unchanged. New pattern locals follow existing local/foreach ordinals.
- A guarded var/discard was incorrectly rejected by the foundation validator
  as an unconditional catch-all. It is now an ordered candidate; an unguarded
  catch-all still requires the final exhaustive position, and a guarded final
  arm cannot itself justify exhaustive acceptance.
- A default section preceding guarded cases initially saw stale ownership
  state. Candidate selection now runs first; default observes all failed-guard
  effects. Both execution and alias-freeze regressions cover this ordering.
- Switch arm ownership paths initially did not share the merge site prefix
  consumed by T03 sequence analysis. Arm/clause/case paths now retain that
  prefix; a regression requires both arm allocations to reach the final freeze
  as construction states rather than being mislabeled immutable publication.
- List guards could mutate the governing unique array between tests, exposing
  Roslyn's cached reads. The selection read borrow rejects both expression and
  statement forms, while a selected arm can update its still-unique array.
  Initial negative fixtures hit an earlier array syntax rejection; they were
  corrected and now assert the exact borrow diagnostic.
- Native mutations could erase a fallback tag or change a list length without
  changing generic graph arity. Decision/fallback inventories and source-derived
  list length/index checks now reject those mutations. W02 also rejects any
  W03-only claim field, including an explicit null.

## Verification

The pinned Linux harness executes the original C# with Roslyn/runtime from the
existing offline toolchain cache. `source-cases.json` retains the actual source,
normalized operations, graph, deterministic rejection codes and CLR outcomes.
The native tests independently execute the register/slot graph. The immutable
Box fixture has an explicitly modeled one-field constructor/getter; this
comparison tests pattern selection, while T03 owns general construction and
getter emission. Floating-point and decimal fixture comparisons retain their
constant representations rather than reducing all tests to integers.

`conformance.json` binds these cases to the unchanged frozen T01-W05 probe and
its compiler observations. An upgrade must reproduce the pinned graphs and
source-regeneration mutation rejections, or explicitly review the differences.
The receipt records exact final counts, file hashes and completed commands.
Final self-review: no findings. All relevant local verification passed.

| Command | Result |
| --- | --- |
| Pinned Linux `--test-pattern-lowering` | 52 sources: 35 private handoffs, 17 exact rejections; 2,940 CLR/CFG comparisons agree. |
| Pinned Linux `--test-loop-lowering` | W02's 32-case file reproduces byte-for-byte. |
| Pinned Linux `--test-data-phase-replay` | All 777 legacy selections reproduce byte-for-byte. |
| Pinned Linux `--check-build-inputs` | All live source manifests and the closed toolchain validate. |
| `cargo test -p mpk-cli --tests csharp_03_` | 131 C# owner tests pass, including all 20 W01–W03 control tests. |
| Freeze/package/spec-vector checks | 709 frozen vectors and 26 vector sets remain unchanged. |
| `./scripts/check-fast.sh` | No workflows, formatting, obsolete-interface scan, workspace clippy/tests/build, and all four certificate CLI checks pass. |

The final gate was rerun after the two existing source-order test anchors were
updated. W06/T06 work described above remains outside this task; no production
activation or proof completion is claimed.
