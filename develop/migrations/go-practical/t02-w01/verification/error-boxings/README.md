# Original error boxing and selected body type union

This Go/Rust-only T02-W01 checkpoint derives concrete-to-error value-use
provenance and the body-derived type union from original selected source.
The parent is `aaf5055ef92575d94bc00b541605001665cbc8cb`. R01/R06/W01 remain
open and no practical public profile, complete closed U or source/program
proof is activated.

The [value engine](../../../../../../crates/mpk-vc/src/go_practical_type_keys/source_scalar_typing/error_boxings.rs)
records the actual original operand, concrete source type identity and result
projection after checking existing assignment compatibility. It covers explicit
conversion, variable declarations, short/reused/parallel assignment, named and
forwarded returns, function/method arguments, aggregate literals, append and
range assignment. Tuple result indices and range element projections remain
distinct. Aliases use the actual concrete target identity; nil and error-to-error
copies add no new concrete case. Zero concrete values retain their non-nil tag.
No Error method body is executed or pulled into proof merely for boxing.

The [source selection constructor](../../../../../../crates/mpk-vc/src/go_practical_type_keys/source_scalar_typing/selection.rs)
filters those sites using its original-source walk. All branches and actual
selected callees contribute. Constant array len/cap operands, unrelated bodies
and unevaluated global initializers do not contribute runtime boxing cases.
Original-source ownership and canonical root ordering remain intact. No caller
AST/type/expected-target/boxing/unreachability facts are constructor inputs.

The implementation also corrects array range evaluation in both Go metadata
and independent Rust selection. With zero or one iteration variable, including
a blank second variable, a constant array length supplies the bound without
evaluating the operand. Two real iteration variables and nonconstant-call
operands retain evaluation. Sources and type dependencies still remain captured.
The pinned compiler's types2 range and walk code supplies the original rule.

## Exact verification scope

[The terminal record](verification.json) retains actual commands, original
prelaunch inputs/drivers, raw outputs and test-selection reasons. Tests run
locally; no agents, workflows or other-language semantic tests run.

- All 46 new intact Go cases pass. Both independent Rust roots pass and match
  all 125 body boxing sites and every body type union exactly. The cases cover
  imported and alias payloads, zero/nil, tuples, method expressions, field/index
  writes, blank destinations, nested literals, ignored operands and all branches.
  Seventy arguments and sixteen tuple results preserve the existing bounds.
- Value-engine evidence emits 128 checked sites, all of them actual original
  Go typed sites. The compiler observes 129 sites in total: the extra ConstLen
  site belongs to the separate constant reader's compile-only declaration.
  This getter is not advertised as a complete compiler typed-site inventory.
  That difference changes no body-site or body-union equality.
- Twenty-four affected existing Rust roots pass across scalar/value/method/
  assignment/return/slice/body/selection consumers. The previous 84 exact-source
  selection cases remain 39 exact positive graphs and 45 rejections; 512 helpers
  and 70 original roots remain supported. Clippy, Rust/Go formats and Go vet pass.
  Unchanged parser/scopes/constants/core/ordinary engines need no unrelated run.
- All 51 previous native source-closure cases pass in both earlier mixed runs.
  Those runs fail overall only because the new observer initially shares an
  exclusive output basename, then lacks its new child directory. Neither mixed
  run is labeled a whole-run pass. The corrected new observer runs separately;
  unchanged old roots are not needlessly rerun.
- The 190 current frontend inputs rebuild twice from independent empty caches
  to identical 8,592,898-byte outputs, SHA256
  `cf8dc06b5f000f8256f481c82135713e89a93876e420014485b4893846f97121`.
  This is a Darwin/arm64 observation. Native Linux acceptance remains due.

Fifteen input epochs retain 12,854 source preimages and 26 actual executions,
including the two output-directory failures and the typed-evidence scope
assertion. Earlier drivers/receipts/results remain unchanged. The first new
owner failed before its application input was retained; only actually available
original output and complete prelaunch source bytes are claimed. Later actual
sources and observations retain their own identities.

[Fourteen original archives](original-byte-audit/result.json) retain 54,753 files
and 385,437,972 bytes, compared literally with their original modes. No output
is recreated by recompiling an earlier consumer. Source-only packets use the
actual original module/package buffers and only a Boolean type seed. Native
comparison rows never enter the independent constructor. SDK14,496 files/
202,722,007 bytes remain exact; frozen255 and ordinary448 bases remain unchanged.
[The input review](input-review.json) scans changed files, complete source
preimages and decoded original payloads and verifies local Markdown links.
Its 127 matchers/four scopes/39 seals/51 exclusions remain unchanged.

## Remaining full obligations

The body union is only one required input to global closed U. Every error-bearing
entry value still needs a bound finite case set, including nested fields and
backing cells. Root/Plain, permitted payload shapes, expanded error cycles,
depth/weights and finite Error dispatch remain required. An error-bearing entry
with no body boxing does not thereby become nil-only. Full capture/version/
namespace/context authority, native candidate import, complete contracts and
actual runtime/call/store/loop/state/program proofs remain incomplete.

Full56 ordinary foundation/private registry, all169 requirements, native Linux
and T02–T11 remain due. R01/R06/W01 stay open and the public profile stays
inactive. The cumulative T02 gate remains final W04: `./scripts/check-fast.sh`.
No intermediate full/equivalent gate or native release acceptance is claimed.
