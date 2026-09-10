# Contract-only carrier coverage correction

The original carrier inventory followed executable VIR operations and values,
source exceptions, bindings and closed instances. A retained actual-source type
contract using `parse_error_kind(syntax)` showed that a type used only in a
contract was omitted. The added finite-operation source test failed because no
parse-error operations were generated.

Carrier generation now also follows validated contract terms, lambda parameter
types, definition signatures and subject bindings. Ordinary function arrow types
are excluded from stored value cubes. Literal text is not scanned for apparent
type names. The actual-source regression now generates and observes all three
parse-error operations.

The correction adds only the Bool carrier to eleven existing helper certificates:
four carrier programs, four scalar-domain programs, one relation program, one
recursive-domain program and one default program. Existing carrier layouts and
source VIR hashes are identical. The other 144 regenerated certificates are
unchanged, including all examined structural-storage and ordered-fold programs.

`changes.json` maps every changed artifact to preserved previous and current
bytes in `checker-cases/`. Both sets (22 certificates) pass both unchanged
checkers with zero axioms; all corresponding hash corruptions reject. Pinned
replays for all seven affected/examined families and the new finite-operation
family pass. Previous metadata is retained in `previous-metadata/` so older
verification checkpoints remain interpretable.

This is a coverage correction and scoped verification record. It does not
discharge application VCs, complete W09, or replace the pending recursive-domain
semantic boundary tests. The repository-wide gate stays deferred to T06-W12.
