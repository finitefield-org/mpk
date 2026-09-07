# W06 task review

Scope: CSHARP-03-T04-W06 only. Review was performed locally against the full
implementation diff, retained source captures, ordinary VIR validators and
runtime differential tests. No review agent, workflow or hosted CI was used.

## Findings resolved during implementation and review

1. Control joins needed explicit SSA values and publication types. Short-circuit
   conditions, pattern bindings, loop backedges and pending returns now preserve
   source evaluation order. Array updates have separate memory SSA versions;
   alias and nullable-return publication freezes the exact immutable value.
2. Exception search had to precede cross-call unwind. Explicit filter execution,
   closed subjects, resumable escape checkpoints and finally continuation slots
   preserve the original exception when a filter throws, and replace it when
   finally throws. The separate ordinary-VIR observer agrees with all retained
   handler CLR traces.
3. A function-wide construction discard set lost path ownership. Cleanup now
   binds the live allocation/receiver origins to each actual exceptional edge.
   Normal finally entry discards abandoned sequence state; published aliases
   survive as immutable values. Native mutations removing or corrupting cleanup
   reject. Object initializer RHS effects occur before the next member write,
   so catch observes earlier local effects without observing the partial object.
4. Constructor loops needed the same SSA and ownership rules as ordinary bodies.
   Class receivers and value fields now participate in loop lowering. Source
   handlers inside a constructor would carry an unfinished receiver across the
   forbidden boundary; both class and struct forms explicitly reject without
   artifacts. Caller-side catches of construction failure remain supported.
5. Pruning an impossible catch left empty exception regions, orphan construction
   records and unused operation signatures. The emitter removes those together,
   repairs retained metadata, and keeps the exact captured declaration closure
   for maps and obligations. Independent import verifies every retained object
   transaction against its source plan and reconstructs control correspondence.
   The `unreachable_catch_object` CLR differential exercises this case.
6. Current producer manifests legitimately change while W05 receipts remain
   historical. The W05 conformance test now binds its retained receipt and old
   manifest hash, and checks current producer files separately. Historical
   conformance and verification files were not rewritten.
7. Added search/filter/anchor inventories needed transport-time resource bounds.
   They now share the frozen per-function block limit. Retained mutations cover
   oversized inventories as well as malformed targets, dominance and provenance.
   Standard Clippy findings were fixed and formatting was rechecked.

8. A void return through finally has no saved operand. Unwrapping the absent
   pending-return carrier would invent an `InvalidOperationException` after
   successful cleanup. Void completion now yields unit directly. The retained
   `void_return_finally` case compares six original CLR runs, including cleanup
   failure and normal completion, through ordinary VIR.

## Final review

No open findings. The 709 published vectors and 26 specification vector sets
remain unchanged. Public profile activation, installed frontend publication and
proof discharge remain with their existing later owners. W06 emits and imports
private ordinary-VIR bundles; it adds no production source interpreter and no
second specialization engine.

`verification.json` records the required local gate and evidence hashes. The
optional `cargo clippy --workspace --all-targets --all-features -- -D warnings`
experiment failed in unchanged CLI feature code: the legacy Go policy evidence
fixture is absent and that feature combination reports unused AI-explain code.
Those unrelated feature paths were not changed as part of W06.
