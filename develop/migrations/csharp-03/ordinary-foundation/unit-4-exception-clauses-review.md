# Direct review: exceptional contract expressions

No outstanding findings in this changed component after targeted verification.
This does not complete review of an original internal unit or W09.

- The two recipes validate an exact exception argument, exact parameter object,
  registered operation specialization and nominal result. They reuse the finite
  generator once and its existing storage cache. The new program's complete
  finite dependency closure matches standalone output for both actual sources.
- Type predicates retain ancestry. Payload definedness instead aliases the exact
  active-arm predicate specified by the finite operation. Tests distinguish all
  built-in tags, the user tag, unknown and high tags, and nonzero inactive storage.
  Full W03 clauses check lexical-let obligations and conditional guarding; zeroed
  projection storage is never taken as evidence that a read is defined.
- The source regression found a real scope mismatch in data contract attachment:
  control validation supplied the closed universe and exception subject, but the
  subsequent recheck did not restore the current case's scope. The fix retains
  that universe, requires membership, binds the exception subject and preserves
  existing non-control behavior. It keeps the entire recheck. The original
  handler/postcondition mutation tests pass after the correction.
- Both certificates pass unchanged dual checking with zero axioms, matching
  reports and actual hash-corruption rejection. Exact names, hashes, lengths,
  sibling metadata and tested output copies are reconciled in the progress
  receipt. Metadata scope substitution rejects exact regeneration import.
- Development failures are retained: a test used the wrong named-field member,
  the data-only capture rejected constructs owned by the control phase, the
  source regression exposed the production scope bug, and a test assumed a
  separate definedness global rather than its existing active-arm alias. The
  corrected test checks the actual complete W03 conditions and raw value bits.

Only the new exception recipes, shared compiler consumers and affected source
attachment/handler checks are included. Unchanged expensive finite-operation
runtime suites are represented by exact whole dependency-closure comparison.
Application proofs and original internal-unit acceptance remain open. The whole
gate is deferred to T06-W12 and no component-only commit is performed.
