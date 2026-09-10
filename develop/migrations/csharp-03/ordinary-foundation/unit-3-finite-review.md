# Finite-operation and contract-carrier direct review

Reviewed the new finite-operation generator, its exports, source and scalar core
tests, captured parse-error contract, checker corpus, and contract-carrier
collection correction. The review compared operation coverage with the frozen
non-template descriptor and the derived closed exception universe.

The actual-source parse-error test exposed an implementation finding: carriers
omitted types appearing only in verified contracts. Collecting typed terms,
definition signatures and subjects fixes it without inspecting arbitrary literal
text or treating function arrow types as value cubes. The test fails before the
fix and passes afterward. Eleven existing certificates gain Bool carrier coverage;
their earlier bytes are preserved, and both versions pass both checkers.

Reviewed payload-member ordering and complete projection, full tag comparison,
the two admitted builtin inheritance relationships, empty source payloads, and
the explicit payload active-tag condition. Input domains/public source conditions
remain mandatory and are not reported as discharged by helper acceptance.
The generator expands uninvoked unit/parse-error operations and exception arms,
uses the common bounded ordinary builder, and imports by exact regeneration.

The seven actual-source certificates and scalar helper certificate, 94 constructor
cases, unit value and every parse-error tag/pair, importer mutations, scoped lint,
inventory checks and eight pinned-family replays pass. Full evidence and preserved
byte identities are in `unit-3-finite-verification.json` and
`contract-carrier-extension/changes.json`.

Latest direct review of this component has no remaining actionable findings.
This does not review or complete all outstanding unit 3 work or the whole W09
diff. Semantic constructors, collection/outcome operations and the original
units 4-8 remain open. `check-fast.sh` stays deferred to T06-W12.
