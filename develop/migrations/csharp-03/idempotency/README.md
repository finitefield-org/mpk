# CSHARP-03-T05-W05: complete-snapshot idempotency

The implementation attaches optional complete retained snapshots to the W04
source-bound transition plan. It does not discharge their semantic proof
obligations; T06-W08 owns that work. A source helper that omits a field can still
be attached, but receives the same complete, undischarged equivalence obligation
as a correct helper. Checking syntax for field reads would not prove equality.

## Strict attachment

The `idempotency` object in `mpk.csharp.transition.v1` has exactly these fields
in this order when enabled:

1. `mode`: `complete_snapshot`;
2. `history_member_id`: a State member of bounded source-record array type;
3. `command_key_member_id`: the Command key member;
4. `record_key_member_id`: a record member of exactly the Command key type;
5. `record_command_member_id`: a member of the entire source Command type;
6. `record_context_member_id`: a member of the entire source Context type;
7. `record_response_member_id`: a member of the entire source Response type;
8. `equality_callable_id`: an actual source static method with explicit arguments
   `(Command, Context, Command, Context)` and result `bool`.

The four record member IDs must be distinct. The helper must be reachable from
Apply and have its own captured, source-hash-bound total method contract.
Selecting it through an unrelated second root does not satisfy reachability.
Ordinary source/control validation still checks the reachable total call/loop
closure. The disabled object remains exactly `{"mode":"disabled"}`.

No caller-selected field projection, digest-equality substitute, collision
assumption, capacity override, or implicit eviction is admitted. History uses
the shared 4096-element array bound and ordinary total-value bounds.

`SnapshotEqualityObligation` retains the shared structural DAG for each whole
snapshot. Each source node records all stored members in declaration order,
including fields not projected by a semantic binding and inactive-arm storage.
The node recipes and original source/binding artifacts identify the exact
canonical source-field encoding relation. The helper must be equivalent to the
conjunction of the complete Command and Context relations. There is no second
production equality evaluator. Shared structural eligibility rejects f32/f64
recursively, including fields behind a boundary-field binding and element
types of empty arrays. Bound Missing and null tags remain distinct.

## Pending path obligations

After ordinary boundary preconditions, the plan orders retained-key lookup,
complete snapshot equality/replay or idempotency conflict, expected version,
history capacity, u64 exhaustion, declared business errors, and new success.
The fixed error codes are `idempotency_conflict`, `version_conflict`,
`history_capacity`, and `version_exhausted`, followed by declared business errors.
All source error carriers remain exhaustively mapped.

The input retained keys must be unique. Replay preserves the complete State,
emits no events, and returns the complete stored Response. Each error preserves
the complete input State. New success appends one record containing the key,
entire Command/Context, and newly computed Response, preserving every prior
record and its order. W04's checked version increment, state invariant,
source-ordered event relation, response relation, and value bounds still apply
to new success. Replay also retains the output value-bound obligation: returning
a stored Response alongside State can duplicate logical value cells even with
zero events. All recipes remain undischarged.

## Actual-source evidence

`Entry.cs` has ordinary immutable application-owned types and no MPK dependency.
`Presence` is bound to `boundary_field<i32>` with distinct Missing, Null, Value
arms. `Run` exposes integer observations of the actual CLR Apply. Its normal
implementation scans all retained records, copies history in order, and appends
without eviction. The replay response is deliberately different from the
current balance so a recomputed-response substitution is observable.

`requests.json` contains captured C# and exact sidecar bytes produced by the
Rust test. `responses.json` assembles the unmodified response rows from two batches of the
pinned local Linux/amd64 Roslyn/CLR control-emission harness. That harness captures each input
twice and checks byte determinism. The Rust test imports each response against
the original captured inputs before attachment. `loops.json` contains the
source-derived loop IDs, modifies sets and index slots; the final source facts
revalidate them. Decreases are `4096 - index`, with undischarged loop contracts.
These fixtures do not claim a complete termination or transition proof.

The cases cover complete field plans, source-body omission, incorrect record
members/types, selected projection/digest/capacity extensions, absent/unreachable/
wrong-signature/partial equality helpers, missing helper contracts, reordered/
missing fixed errors, and direct/nested/empty-array floating point storage.
Finite CLR observations exercise replay against stale version/full capacity/
exhaustion/business errors, retained-key mismatch in each non-key snapshot field,
Missing versus null and inactive payloads, lookup at the end of history,
new-key precedence, stored response, original-state preservation, append order,
and ordered new events. These observations are test evidence, not proof-positive
or proof-counterexample execution. The omitted-Context helper intentionally
exhibits a finite mismatch while retaining the full unproved obligation.

W04's retained no-idempotency matrix still runs. The valid W05 emission also
passes independent import; replacing its embedded enabled contract with a
rehash-bound disabled contract cannot bypass original capture linkage.

## Verification cadence

Run `cargo test -p mpk-cli --test csharp_practical_transition`, the affected VC
transition unit test and inventory consumer, scoped lint, and format checks.
`verification.json` records the exact completed checks and evidence hashes.
`./scripts/check-fast.sh` is deliberately deferred to CSHARP-03-T05-W06 under
AGENTS.md. W05 does not claim a full-gate pass. Frozen vectors, source producer,
probe receipts, and W04 evidence remain unchanged.
