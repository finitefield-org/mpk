# T06-W07 direct review

Scope: boundary contract/run VC programs, strict reconstruction APIs, existing
VC groups/resources, the two affected contract readers, original-source tests
and task records. Review was performed directly without delegation.

## Findings resolved

- The existing control/exception readers used direct serde deserialization of
  retained contracts. A real boundary field named by a lone UTF-16 surrogate
  failed VC generation. Both readers now use the existing lossless canonical
  parser. The complete original attachment matrix exercises this regression.
- JSON syntax cells and decoded public value cells are distinct limits. The
  input equation now binds 262144 syntax cells and 65536 decoded cells, including
  defaults. Document text is not constrained to the public 16384-unit string
  domain; that limit applies to parsed names/string values. Existing boundary
  limit tests cover document bytes, depth, aggregate cells and UTF-16 lengths.
- Missing and null cannot share a predicate. Missing is absence, null implies
  presence, and required/nullability/default rules are explicit implications.
  An independent eight-case truth table per field checks both legal states and
  inconsistent absent-null inputs, with valid/invalid payloads.
- Universal output equations must use the exact source-reachable return summary
  under accepted boundary inputs and method preconditions. Otherwise an
  unreachable large value of the return type could make a valid entry fail its
  byte-limit obligation. The selected-source predicate is bound by the retained
  contract/VIR and W02-W06 dependencies; it is not an arbitrary assumption.
- A formatting-defined guard must not contain output equality. The output goal
  separately requires source observation equality, preserving inactive fields,
  float bits and order while allowing frozen decimal and array-snapshot value
  equivalence. Finite countermodels break the encoder; real run tests inspect
  complete literal bodies and reject payload/evidence changes.
- Run evidence cannot be trusted merely because it is immutable. Generation
  replays input/output import against the requested source context and exact
  capture/manifest/artifact bytes. A different source context, changed canonical
  output or omitted handoff field is rejected. Deterministic byte mutations
  cannot reuse the original receipt even if they form another legal input.
- Literal bodies need resource accounting in addition to formula references.
  Run generation counts retained monomorphic value cells and formula nodes
  together, with the fixed transport cap; changing the retained counter rejects.

## Final review

No findings. Rechecked canonical source/contract linkage, typed fields and
literal bodies, sequent identity, group dependencies, resource accounting,
strict importer equality and the staged task scope. No schema/proof-status or
public production acceptance change is introduced. W09 retains proof/checker
ownership, W08 alone becomes ready and the full T06 gate remains deferred to W12.
