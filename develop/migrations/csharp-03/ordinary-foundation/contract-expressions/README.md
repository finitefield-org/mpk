# Ordinary contract attachment expressions (partial W09)

The new program lowers every captured contract attachment from canonical W03
data VCs into a value function and its exact definedness function. Unsupported
recipes reject the whole generation. This includes method pre/postconditions,
loop invariants and non-Boolean measures; it does not establish application
proofs or instantiate contracts at concrete control-flow use points.

Metadata preserves the original owner, attachment, expression hash, nominal
subject order, result type and old/exception mode. Frozen loop owners may be
empty: their concrete program point remains the control VC owner's responsibility.
Program metadata binds VIR, foundation and data-VC hashes. Function names use
the complete attachment hash, so identical expressions with different argument
scopes do not collide. Exact regeneration rejects changed metadata, reordered
subjects and substitution into a different validated source context.

The shared source-public compiler also fixes identical expression hashes on
different nominal subject signatures. Both the value and definedness symbols
receive attachment suffixes in that case. Unambiguous legacy names and bytes
are preserved. Partial public clauses still require use-point discharge.

Three unchanged original control fixtures exercise nine attachments and 27
value/definedness sample pairs, including non-Boolean decreases. Two fresh source
contexts attach total or undefined arithmetic invariants to i32/i64 source
owners with carrier depths 5 and 6. Their total integrated source dependency
closure is preserved; partial integrated admission rejects. A fresh loop-free
method provides five independent current/entry/result probes for `result ==
old(parameter:0)`. Counterfactual assignments expose binder aliasing; they are
not claims about reachable source states. The initial data capture of a loop
fixture correctly rejected `array_loop_handoff`; the dedicated control fixtures
retain loop coverage.

All nine certificates in `certificates.json` pass both unchanged checkers on
identical bytes with zero axioms. Actual hash corruptions reject. Four affected
predecessor clause families retain their pinned outputs. Lint, scoped format,
artifact inventory and published-request replay pass. The progress receipt
`../unit-4-contract-expressions-progress.json` retains exact logs, test selection
and corrected failed attempts; `capture-receipt.json` identifies fresh captures.

Remaining contract recipes, native/control use-point bindings, transition/replay
and application proof assembly remain open. This is a component checkpoint,
not completion of an original internal unit or W09. There is no component-only
commit/push; the full gate remains deferred to T06-W12.
