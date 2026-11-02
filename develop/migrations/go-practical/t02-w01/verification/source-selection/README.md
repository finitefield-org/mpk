# Independent original-source selection checkpoint

This Go/Rust-only intermediate T02-W01 checkpoint reconstructs static Go
selection dependencies in Rust from original source buffers. It does not
complete R01, R06, W01 or W02 and does not activate a public practical profile.
The parent is `185fb16d24fbd617cefa875e0ec11192c96f252a`.

The new private [selection constructor](../../../../../../crates/mpk-vc/src/go_practical_type_keys/source_scalar_typing/selection.rs)
uses the existing independent parser, lexical scopes, source types, constants,
signatures, value and body checking. Its inputs are original owned source/type
package envelopes and physical function locators. Native AST/type/call rows are
absent from its API. The corpus envelope contains only a Boolean type seed;
actual selected types and bodies are independently derived. Native expected
rows are comparison data in the test only.

Each selected function and actual reachable static callee has its entire body
checked, including unreachable branches. Explicit queues reconstruct body,
type, constant and signature dependencies, with source paths on errors and a
separate iterative call-cycle check. Concrete Error implementations contribute
signatures unless called or selected. Repeated constant declarations resolve
at the actual current specification. Constant array len/cap operands use type
dependencies without executing a variable initializer. Inferred array types
retain original nominal element declarations. Mutable package-state reads,
indirect/interface/promoted calls, unsupported forms and missing bodies reject
within this private reader's supported scope. No new helper/root/parameter cap
is introduced; existing source, aggregate and checker budgets stay unchanged.

Both implementations now inventory every original nested declaration,
including those inside unselected functions, as explicit type-check-only rows.
The Go change only adds that inventory walk; selected-local declarations keep
their actual roles. Full original standard-library unsafe source and its module
file remain input buffers. Its intrinsic package has no selected declaration
inventory. No stub or manufactured standard-library package is substituted.

## Selected verification

[The terminal record](verification.json) identifies each original command,
input epoch and actual stdout/stderr. Tests run locally, and their selection
rationale is saved before launch.

- All 33 original Go cases pass their native assertions. Rust matches all 15
  positive declaration/name/role/call/dependency graphs exactly and independently
  rejects the 18 negative cases. The positives contain 9,795 declaration rows,
  516 call edges and 1,071 dependency edges. Negative diagnostics need not have
  identical categories or partial graphs.
- Three new Rust roots include source ownership after dropping inputs, invalid
  and duplicate roots, canonical root order, 512 linked functions, 70 parameters
  and 70 simultaneous roots. The 70-root graph has 70 physical calls.
- Eleven affected existing Rust roots pass: four body, three scope and four
  source-type roots. The original 495 body cases remain 176 accepted, 299 rejected
  and 20 outside this reader. Existing engines retain their previous semantics;
  scope changes add only the internal package-binding predicate. No unrelated
  Rust or other-language semantic tests run.
- Scoped Clippy, Rust/Go formatting and Go vet pass on the current consumed
  files. The final Go format receipt follows the two added inferred-array cases.
- The 190 actual frontend inputs rebuild twice from independent empty caches to
  identical 8,592,898-byte outputs, SHA256
  `764e57a7db286f4a76fc41bb1b15e92cf23a57fc8d6400f4fec84712afc02d9b`.
  This observes Darwin/arm64 builds. Native Linux release acceptance remains due.

The retained 21 source epochs contain 17,902 whole-source preimages and 34
actual executions, including three failures. The first failed new-test macro
was corrected. The next comparison found missing unselected nested Go rows,
incorrect handling of Rust range declaration names and local-variable edge
ownership; these were corrected. The inferred-array comparison then exposed a
missing nested nominal element dependency, corrected before the final passing
run. Original failures and later outputs remain separate. An unchanged-input
formatter invocation after a failed local edit search and an early formatter
that targeted old aggregate files supply no current-format evidence.

[Eleven original archives](original-byte-audit/result.json) retain 23,775 files
and 232,605,508 bytes, compared literally with original modes. They include
failed and passing Rust graphs, both successive native case sets, old consumer
outputs and frontend observations. No consumer is recompiled to reconstruct
those outputs. The SDK check preserves all 14,496 files/202,722,007 bytes; frozen
255 inputs and the previous ordinary448 basis remain unchanged.

[The conversion correspondence audit](derived-input-audit/result.json) checks
all original raw sources and Boolean-only seed envelopes against the actual
native archives. The historical converter printed the parent manifest field
even when an explicit fresh directory was used. Its original receipts/drivers
remain unchanged; this supplemental audit gives the correct actual directory
and retained corpus path. The original parent and inventory-only corpora are
retained under their new filenames. The manual `original-reference.json` was
never a test consumer input. This is an after-run correspondence check, not a
replacement prelaunch receipt.

[The unchanged-rule review](input-review.json) covers active files, saved source
preimages, decoded source packets, original archives and local Markdown links.
The scanner remains at 127 matchers, four scopes, 39 seals and 51 exclusions,
with no new exclusions. No agents or workflow facilities are used.

## Remaining authority and gate

The constructor establishes this private dependency/type/body analysis only.
A native wire importer, complete captured package/build/version/namespace
validation, full v1 Root/Plain/closed-error/selector/conversion admission and
contract-dependent verification closure remain due. Unselected body/package
validation is not established by type-check-only labels. Actual source runtime
evaluation, calls/targets/two-phase stores, zero results, loop safety/halting,
state/program proofs, all56 ordinary foundation/registry families and all169
requirements through T02–T11 remain open.

R01/R06/W01 remain open, the public profile stays inactive, and the full T02 gate
is deferred to final W04: `./scripts/check-fast.sh`. No intermediate equivalent
whole gate or native release acceptance is claimed.
