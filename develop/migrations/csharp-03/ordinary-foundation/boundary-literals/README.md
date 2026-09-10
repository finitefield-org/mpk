# W09 unit 4: ordinary boundary-run literal bodies

The generator accepts a validated VIR and an immutable BoundaryRunVcProgram
created by the existing input/output capture revalidation path. It recomputes
the static boundary VC hash and requires both its identity and the exact source
IR identity to match. The output binds the foundation, static boundary program
and complete run hash, including input/output captures, provenance and manifests.
Import regenerates and compares every metadata and certificate byte.

Every boundary literal symbol is checked against the frozen value-derived name
and mapped, with its exact type ID, to a core-safe ordinary definition. A symbolic
boundary-VC name is not blindly registered as a core global. The existing VIR
literal encoder emits each actual body, retaining all source fields and exact
scalar representations. Values are shared by content, while all boundary symbol
bindings and the run identity remain present. No parser/serializer result is
used as a trusted axiom or as a proof of native application execution.

The corpus uses all three existing actual source captures from boundary-output:
a payload with integer/string/decimal/nullable fields, a 32-field payload with
16 long arrays and 16 strings, and a void root. Nine run variants exercise base
values, changed provenance, none, decimal cohort changes and the final stored
field. They contain 11 root-definition occurrences and six distinct certificate
byte strings. Same-valued runs with different provenance have identical literal
certificates but different metadata; cross-run import rejects in both directions.
Cross-source runs, symbol/type mappings, metadata and byte mutations also reject.

All 20,258 sampled/active bit observations match the independent flat-storage
encoder. The returned decimal cohort and its independently reparsed form remain
two distinct stored literals even though decimal observation considers them equal.
The final field is observed in the wide payload. Frozen i64 input rules reject
JSON numbers, overflow and quoted negative zero before a run can be constructed.
These checks are concrete examples, not universal codec/source-execution proofs.

Generation, exact pin replay, existing W07 run goldens/hostile serializer cases,
all 64 existing VIR-literal pins, scoped lint, format and five inventory tests
passed. Both unchanged checkers accepted all nine same-byte pins with zero
axioms and rejected all hash corruptions (101.971 seconds). Maximum source
certificate sizes are 792 terms and 29 declarations.

See ../unit-4-boundary-literal-progress.json and its review attachment. This
component supplies literal definitions and symbol bindings; the later ordinary
proposition compiler must use those bindings. Binding/codec relations, native
execution proofs, complete certificate assembly and original units 4-8 remain
outstanding. Unit 3's deep checks/review also remain open. The T06 full gate stays
at T06-W12; neither W09 nor an original internal unit is completed here.
