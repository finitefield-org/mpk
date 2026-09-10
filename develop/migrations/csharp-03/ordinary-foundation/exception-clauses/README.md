# Ordinary exceptional contract expressions (partial W09)

`exception_is` and `exception_payload` now reuse the existing finite exception
definitions. Type predicates retain the frozen ancestry relation. Payload
projections retain masking and require the exact active source-exception tag
through canonical W03 definedness; inactive zero storage is not a normal result.
The shared contract compiler emits the finite definitions once per builder and
validates the exact specialization, argument types and result type.

The original-source regression exposed an attachment recheck bug: the control
owner validated exceptional clauses with their closed exception scope, while the
data attachment recheck omitted the exception variable and admitted only built-in
scope identifiers. The recheck now preserves the validated closed universe and
exception subject. The original control owner still verifies source identity,
scope, payload membership, old restrictions and contract attachment. No source
or contract validation is skipped, and non-control routes keep their prior gate.

Two exact source contexts cover nine built-in exception predicates and a sealed
source exception with i32, Bool and char payloads. The source predicates also
include a user-exception test, three payload reads in lexical lets and a guarded
payload read. The 2,808 observations cover all built-in tags, the user tag,
unknown tags, high-bit tags, nonzero inactive storage, full payload bits and exact
W03 value/definedness behavior. Raw invalid representations are binding/storage
probes, not public-domain admission claims. Complete finite-definition dependency
closures match standalone generation. Exception-scope metadata mutation rejects.

The first data-only capture correctly rejected later-owner constructs. The
replacement uses the unchanged control-emission harness with 38 hash-verified
frozen inputs. Both source/contract contexts were captured fresh, then imported
and emitted through the normal Rust path. `capture-receipt.json` identifies the
requests, responses, pinned control inputs and isolation settings.

Both certificates in `certificates.json` pass the unchanged Rust and Go checkers
on identical bytes with zero axioms and matching reports; actual hash corruptions
reject. Previous method/loop/source attachment cases, original exception-handler
and postcondition mutation cases, lint and the affected inventory passed.
`../unit-4-exception-clauses-progress.json` records exact results and test selection.

These functions do not discharge application VCs or establish concrete control
use-point bindings. Other contract recipes, native/control, transition/replay and
proof assembly remain open. No original internal unit or W09 is complete, no
component-only commit/push is made, and the full gate remains deferred to T06-W12.
