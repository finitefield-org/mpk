# Hex and calendar codec contract review (W09 internal unit 4)

Status: fixed-codec contract component targeted verification passed.

The two standalone generation loops were extracted without changing operation
order, shape checks, codec configuration checks or helper bodies. The contract
cache reconstructs each family once within its owned builder and selects by
both exact codec ID and nominal type. Parse retains its exact closed Result
shape; format uses the existing W03 output-length condition. Decimal variants
continue to fail closed until their adapter is implemented.

Seven fresh contexts cover each codec and all six together. Non-reflexive NaN
is tested through actual bits and successful Result tags; structural equality
is not assumed reflexive. Direct parse cases include canonical values,
noncanonical hex spelling, malformed/non-ASCII input, date/time range errors,
and oversized full-width length words. Actual parse(format(value)) results are
checked at every payload and padding bit. Each alias's transitive declaration
closure matches the independently generated standalone program.

Verification selection: the emitter extraction can affect the 11 existing hex
and 3 calendar pins; the dispatch extension and request helper refactor can
affect the 10 integer contract pins. These 24 pins passed byte-for-byte. Earlier
standalone runtime arithmetic suites and the 15-minute integer contract runtime
suite are not repeated. New alias/source observations, mixed-family linking,
exact imports, metadata mutation, inventory and scoped lint/format cover the
changed connection. All seven candidates passed both unchanged checkers on identical bytes, with
zero axioms, agreeing reports and actual hash-corruption rejection (254.248s).

No application VC proof or W09 completion follows from these observations.
Commit/push remains at the approved original internal-unit boundary.


## Final evidence and review

Two source tests passed in 96.24 seconds: 44 runtime contract conditions,
56 direct parse cases, and 21,088 formatter/composition bit observations.
The mixed context reconstructs 44 attachments and checks all 12 nominal aliases
and complete standalone dependency closures. Its individual runtime suites are
not repeated. Exact import and individual result-metadata mutation passed.
Scoped inventory, clippy, Rust/Go formatting and the checker build passed.

Final direct review found no remaining actionable issue in this connection.
The complete seven-file checker PASS set matches current pins. Module hashes,
byte lengths, metadata hashes, tested output copies, capture records and retained
terminal log hashes were reconciled. Raw frontend response transport is retained;
pretty-printing preserved all facts and array order. The largest candidate is
1,283,008 bytes and the corpus totals 2,730,888 bytes.

This is component evidence only. Decimal contract adapters, other original-unit
work and application VC proofs remain open. No component commit/push was made;
the existing dirty work, including user AGENTS.md, remains preserved on main.
The full gate remains deferred to T06-W12.
