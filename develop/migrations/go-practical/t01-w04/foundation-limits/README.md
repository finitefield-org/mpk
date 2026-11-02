# Private limits, definition inventory and foundation descriptor

Status: W04 packaging draft; complete ordinary foundation/source freeze open.
Entry: `15f6833f2931a3a39fa8cad450f1c48a2f813b60`.

The [three root shapes](schemas.json) and [semantic rules](semantic-rules.md)
close limits, definition-inventory and foundation-descriptor records. All
fields are required, unknown fields reject and only each record's own hash
is omitted from its complete typed preimage. The [private model](../../../../probes/go-practical/foundation_limits_protocol.py)
compares the entire record against compiled values after hash checks.
Rehashing a caller-supplied maximum, weight, core rule, operation or member
does not authorize it. No installed schema/profile or production core changes.

The [limits](limits.json) carry all64 independent preceding numeric candidates,
their exact maxima, units, counting rules and distinct overflow outcomes.
This keeps the4,194,304 practical syntax candidate separate from old Go's
1,000,000. Byte counts, bits, depth, monotonic nanoseconds and complete abstract
carrier weights remain explicit. Domain or proof-resource failure cannot be
reported as a Go panic, semantic falsity or success. Counter enforcement and
source/finite-id/allocator proofs still require production implementation.

The [definitions record](definitions.json) is explicitly an operation recipe
inventory draft. Eight component families cover all56 logical operations and
their six common proof members. Exact source/hash references bind the original
operation table and six prior evidence receipts. The registered joint core
proposal contains Sort0 Nat.induct and Bool.recType signatures, equations,
constructor/universe/order guards, four implementation files and the entire
77-file mirrored source closure. Prior private checker results establish
only their own retained scopes. Full closed ordinary definitions, concrete
specialization/expansion and source/contract/body proofs are still required;
this inventory cannot substitute for them. Both caller extension and source
callable lists are empty.

The [descriptor](descriptor.json) binds exact limits/definitions references
and exactly three path-sorted raw members: their two JSON files and the
semantic-rules document. Member digests include original final LF and sizes;
record hashes use ASCII domain+NUL+narrowed RFC8785 without LF. Its own bytes,
context, compilation instances and executable/release data are excluded from
the member graph. Limits precede definitions and descriptor; registry/context
then follow. Executable release registration must bind the finished semantic
bundle later rather than create a self-hash cycle.

The [linked private contexts](linked-contexts.json) replace the preceding
context checkpoint's deliberately incomplete fixture references with this
exact candidate package. Its old fixture/evidence remain unchanged. Mixing
either private generation's registry/foundation/limits rejects. This is an
assembly exercise before normative publication, not a production migration.
All four retained profiles keep null foundations and actual old compiled
parameter/selection policy; both Rust32 and Rust64 contexts remain valid.
Installed active and reserved inactive C# identities/preimages are preserved.

The [102 reconstructible vectors](vectors.json) exercise three record shapes,
rehash attacks, complete raw-member sets and altered/LF/size/path payloads,
both private generations and six valid target contexts/requests. Each mutation
is applied to the full canonical compiled fixture. Sixteen configuration
candidates and86 configuration rejections are expected. They are neither
checker verdicts nor program source acceptance.

Twenty-one [complete preimages](hash-vectors.json) cover all three packaging
records, five entries, registry, six contexts and six requests. Python and
unchanged Rust/Go canonicalization observers agree on every canonical byte
and digest, with two identical runs each. Fresh replay rebuilds both native
observers and verifies the pinned Go1.25.0 host toolchain. These are hash
observations, not evidence that recipes are valid ordinary kernel definitions.
Twenty targeted tests cover typed authority, counter units/outcomes, full
weights and member bytes, symlink-before-read rejection, both Rust targets,
context crossings and absence of reverse/self descriptor members.

The [verification record](verification/verification.json) retains selected
tests, fresh native replay, metadata and Python compilation with raw logs
and reasons. Prior proof/native matrices are not rerun; their source/receipt
integrity and both retained failed-construction diagnostics are checked.
W04 stays In progress. Complete foundation/limits/source/contract/downstream
schema predicates and reviewed migration remain required. W05 owns inactive
publication and the cumulative T01 gate; all T02–T11 remain required.

```sh
python3 -B develop/probes/go-practical/run_foundation_limits.py --check
PYTHONPATH=develop/probes/go-practical python3 -B -m unittest -v test_foundation_limits
python3 -B develop/probes/go-practical/check_foundation_limits_metadata.py
```
