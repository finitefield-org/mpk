# Native exceptional invocation results — scoped review

The public ordinary control generator appends exceptional-result relations after
all existing slot, edge, entry, frame and normal-operation declarations. It
reconstructs each W03 exception check and finds exactly one W04 exceptional edge
at the same native operation/check, with the same target and ordered failure
guard. Static obligations and tagged error outcomes do not become exceptions.

Each guard takes only invocation operands. The exceptional relation conjoins
that guard with complete physical equality to the validated exception literal;
there is no normal-result argument, assumed default result, or implication that
would make a disabled failure vacuously executable. Literal emission reuses the
ordinary carrier layout, including tag, inactive payload and padding. The
exception argument retains its specific edge, target, check and carrier type.
Ownership predicates remain scoped to the exact original function/node/receiver
and are removed after each emission. Missing semantic constants leave the guard
and execution definition explicitly pending. Existing declaration, term and
binder limits apply to the resulting certificate.

The original-source test covers 59 exception relations in 18 contexts, with
6,136 observations. Independent numeric/sequence/string/nullable guard oracles
exercise positive/negative cases and competing null/index failures. Independent
storage encoding checks every literal bit. Each execution relation checks the
exact exception, a changed tag and a changed final storage bit, under both enabled
and disabled guards, with an explicit coverage assertion for each individual
check. Metadata is compared to original native invocations, exception literals
and independently generated W05 exception edges. Every new metadata field, omitted records and extra records
are rejected by independent importer reconstruction. Twelve preservation and
slot-candidate tests pass, including exact prior declaration/term prefixes and
complete metadata after removing only the new field.

The first runtime test exposed a missing subtraction-overflow input in its
coverage assertion. Adding MIN-1 and MAX-(-1) samples exercises both subtraction
boundaries without weakening the assertion; all 18 contexts pass. The inventory
test also exposed the new exceptional module and an earlier normal-operation
module omitted at the previous checkpoint. Removing exactly these two paths
reproduces the old fingerprint. The corrected 138-path foundation inventory,
4,961 total hits, five inventory tests, targeted Clippy and format check pass.
The canonical ledger now accurately marks W09 In progress, with W10 blocked.

A subsequent review found that the first candidate used its check ID as the
exception value identity. W05 instead uses the complete edge-qualified output
identity and the `exception_value` binding kind. The corrected candidate uses
those exact identities, and the test compares every binding's ID, carrier,
post-node and frozen literal with W05 `ExceptionEdgeVc`. Former candidate
results are retained under `before-w05-value-linkage`; none was promoted. The
obsolete count/fill checker was deliberately interrupted, which is neither an
acceptance nor a proof rejection. Every changed certificate requires fresh
checking after this correction.

Fresh same-byte Rust/Go acceptance, report agreement, zero-axiom checks and
hash-corruption rejection remain required before promoting the changed pins.
The 15 unique certificate byte sequences cover three additional contexts only
through exact byte equality. Checker results are recorded in verification.json
after completion; historical acceptance is not transferred to these bytes.

This component constrains the local result of a reached failing invocation.
It does not establish reachability, source-state framing on exceptions, handler
search/filter execution, finally unwinding, loop invariants or a complete
application theorem. Explicit throws and non-data native calls remain separate.
Units 3–8 and W09 remain incomplete. The full repository gate is deferred to
T06-W12; no component-only commit or downstream unblock is made.

All current 15 distinct certificate values now pass Go/Rust acceptance, matching
reports, zero axioms and hash-corruption rejection. The 18 promoted file pairs
match these exact checked bytes; current prefix/metadata preservation passes.
Final scoped review findings: 0. The remaining unit/W09 scope above is unchanged.

The final local release build of the Rust checker also reproduces all 30
positive/hash-corruption JSON reports exactly; see `rust-checker-build.json`
and `rust-rebuilt-verification.json`. The Go checker inputs and outputs remain
the same. Promotion integrity checks match all 18 pairs and their recorded
source/frozen hashes without repeating unaffected semantic runtime tests.
