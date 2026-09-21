# Native operation transitions — scoped review

The public control generator appends successful native operation relations after
the existing edge, entry, memory and source-frame declarations. Each record
retains its complete W03 operation, original invocation, all source anchors and
typed argument order. Operands bind at the invocation node; the result binds at
the exact normal successor. The emitted normal relation is the conjunction of
the reconstructed success guard and result relation. It is not the W03 success
implication, which would admit arbitrary results when a check fails.

The definition registry reuses prior family adapters and appends missing
integer/Boolean, string, source-value and lifted definitions. Existing sequence
and construction foundations provide additional invoked operations. Scalar and
enum structural equality/order use the existing bit-relation circuits with the
underlying width and signedness; floating and compound structural relations do
not receive a bitwise fallback. Input domains remain separate obligations.

Ownership failure aliases are installed only for an existing source-bound proof
at the exact function/node/receiver. They are removed after lowering that
operation. No caller-supplied ownership Boolean or global false predicate is
introduced. If any required constant is absent, every execution predicate for
that operation stays pending. Native invocations absent from W03 are listed
separately. The present eighteen-source corpus has 106 W03 operations, 98 defined
normal relations, eight pending operations and one non-data invocation. This is
the W04 loop/pattern function subset; straight-line and constructor functions
outside that subset still need integration.

Runtime checks compare every original invocation, SSA value, successor and
anchor. Integer boundary semantics are checked once per exact shared definition;
each native use point still exercises a nonzero result and a corrupted result.
Failed arithmetic guards reject normal execution even when the supplied result
matches a default scalar output. Collection/string/source-value cases cover
normal results, result corruption, null, range, negative length, construction
capacity and initialization failures at every supported original use point.
Import mutation tests retain exact metadata reconstruction. All eighteen sources
pass 592 observations in 356.44 seconds. Six focused failed-operation cases add
24 observations: the zero failure output satisfies the raw result relation but
does not satisfy normal execution. See `verification.json` for the receipts.

The initial 300-second cumulative thread budget expired after successful earlier
contexts, at the division context. This is retained as a failed budgeted probe,
not a semantic verdict. The revised test avoids redundant standalone result
evaluations and repeated boundary cases for the same definition, while retaining
all use-point and boundary categories. Long tests remain agent-owned.

Eight historical preservation tests confirm exact declaration/term prefixes and
prior metadata. The nine private slot pins and their 2,032 existing observations
replay unchanged. Source-frame regeneration passes all 10,608 observations.
Targeted Clippy passes. The two adapter visibility changes add no semantic body
changes; four other source modules only re-export the new public metadata types.
Same-byte checker verification is required for all fifteen distinct candidates
before any pinned artifact is replaced; all fifteen distinct candidates passed,
and three aliases reuse receipts only by exact byte equality. Format and final
source/artifact fingerprints pass. `export-preservation.json` verifies the
unchanged emitter bodies and exact inverse re-export edits.

Remaining work includes Option constructors/presence and compound equality,
object/native invocations, other data families, literals, transfer definedness,
alias/exception effects, actual execution/selection witnesses, loop invariants
and complete native/application proof composition. These remain open; no
internal unit, W09 completion or component-only commit is justified. The full
repository gate remains at T06-W12.
