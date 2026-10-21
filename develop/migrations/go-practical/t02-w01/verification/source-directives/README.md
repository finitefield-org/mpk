# Independent function-directive admission checkpoint

This Go/Rust-only intermediate T02-W01 checkpoint closes the independent
source-selection reader's missing function-documentation directive check.
The parent is `59310cba83d5a20a292103153622dadec623d7fc`. R01/R06/W01 stay open;
no public practical profile or whole-source/program proof is activated.

The [new collector](../../../../../../crates/mpk-vc/src/go_practical_type_keys/source_scalar_typing/selection/directives.rs)
uses only original token gaps and physical lines. Token gaps exclude string
and character literals. Comment grouping accounts for preceding-token line
comments, blank-line-separated groups, block comments and inserted semicolons,
including a semicolon inside a block comment. `//line` changes no physical
source locator. Prefix classification follows Go's CR-normalized comment text,
while diagnostic spans retain the complete original token, including CR bytes.
The pinned Go1.25 parser source and unchanged SDK inventory are recorded.

The private [selection constructor](../../../../../../crates/mpk-vc/src/go_practical_type_keys/source_scalar_typing/selection.rs)
rejects a reached function's `//go:` documentation directive for both body and
signature roles. Signature-only Error compatibility also checks that dependency;
it never executes the Error body. Unselected functions retain type-check-only
roles. No caller AST/type/comment/adaptor facts or unreachable-branch oracle are
accepted. Existing lexer/parser/scopes/type/body engines, Go production, ordinary
proofs, kernel, registries and all resource limits remain unchanged.

## Scoped checks and original observations

[The terminal record](verification.json) retains every actual command, input
receipt, selection rationale and raw stdout/stderr. All checks run locally.

- The fixed Go compiler passes all 51 current original cases. Rust independently
  matches 24 positive graphs and rejects 27 negative cases. Nine positive and
  nine negative comment/signature cases extend the earlier native fixture.
- The unchanged original 33-case corpus additionally yields 15 exact positives
  and 18 independent rejections. Across both corpora, 39 graphs have 25,995
  declaration rows, 1,032 calls and 2,142 dependency edges; 45 cases reject.
- Eleven independently recovered directive spans match native physical source
  bytes exactly. Each error identifies the actual function, selected root and
  a path composed of original dependency edges. Direct roots, actual callees,
  signature-only Error dependencies, internal/trailing CR, CRLF, blank lines,
  blocks, spacing, raw/quoted strings and a physical `//line` case are covered.
- All four affected Rust roots pass, including ownership after dropping inputs,
  original-root membership/order, 512 linked functions, 70 parameters and 70
  simultaneous roots. Selected Clippy, Rust/Go formats and Go vet pass.

Twelve source epochs retain 10,229 source preimages and 16 actual executions,
including three failures. Two malformed directive placements first failed real
Go export compilation; their complete original source and compiler diagnostics
remain. A later old test assertion incorrectly included a new unrelated Error
method and was corrected to check the actual selected signature dependency.
The first independent Rust span comparison then detected a normalized-length
versus physical-CRLF-end bug, corrected before the passing runs.

That first Rust assertion preceded saving both case27 actual rows. Its 114
actually available output files and complete input/code/raw diagnostic remain;
those two missing rows are not reconstructed. The test now saves each actual
row before assertions. The previous 33-case input was also missing an explicit
row in the first two Rust input inventories, although its original Git blob
was immutable at the parent. The final prelaunch inventory adds that exact
complete blob reference and reruns all four roots. Both passing 174-file output
maps compare byte for byte. Earlier receipts and drivers are not rewritten.

[Six original archives and their final audit](complete-byte-audit/result.json)
retain 22,619 files/254,849,781 bytes, compared literally with original modes.
[The earlier five-archive audit](original-byte-audit/result.json) remains separate.
The conversion driver records the actual final native directory and every
input before deriving source-only packets with a Boolean type seed. Native
expected graphs remain test comparison data, not constructor inputs.

The SDK remains byte-exact at 14,496 files/202,722,007 bytes. All 190 prior
frontend production inputs remain exact, so the prior two independent build
observations and binary identity remain applicable without rebuilding unchanged
production. The frozen255 and ordinary448 bases remain unchanged. The
[unchanged-rule input review](input-review.json) scans changed files, saved source
preimages and decoded original payloads and checks local Markdown links.
Its 127 matchers, four scopes, 39 seals and 51 exclusions remain unchanged,
with no new exclusions. No other-language tests, agents or workflows run.

## Remaining work and gate

This is the private function-documentation directive check used by the existing
selection analysis. Full original package/build/version/capture authority, a
native wire consumer, complete v1 Root/Plain/closed-error/selector/conversion
admission and contract-dependent closure remain due. Full Go compilation still
rejects malformed/misplaced directives before any selected-source comparison;
a partial Rust graph alone does not establish whole-package validity.

Actual source runtime/call/target/store/result/loop/state/program proofs, full56
ordinary foundation/private registry and nativeLinux/all169/T02–T11 remain
unimplemented or incomplete. R01/R06/W01 stay open and the public profile stays
inactive. The cumulative T02 gate stays at final W04: `./scripts/check-fast.sh`.
No intermediate full or equivalent gate, or native release acceptance, is claimed.
