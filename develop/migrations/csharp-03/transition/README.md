# CSHARP-03-T05-W04 pure transition attachment

The private data emitter accepts captured `mpk.csharp.transition.v1` sidecars
for an exact selected static source method named `Apply`, with arguments
`State, Command, Context` and an application-owned immutable result. The two
content-bound, non-generic source bindings must project to exactly
`result<transition<State,Event,Response>,DomainError>`. The existing closed
specialization and source-binding engines validate these projections; source
signatures and compiled application bytes contain no MPK dependency.

W04 binds the error domain to the source enum declaration identified by the
result binding's error member. `domain_error_binding_id` is that source ID;
`errors` maps every carrier of that exact enum once. It does not introduce a
new semantic-binding role, infer meanings from names, or reuse a framework enum.
The original captured source and the result binding bind the enum's contents.

## Nested contract shape and expression scope

The frozen root schema, field order and hash domain remain unchanged. These
nested attachment objects are strict, in the following field order:

- `version_rule`: `state_member_id`, `expected_member_id`, `increment`,
  `effective_time`. The first two identify exact `u64` members of State and
  Command; `increment` is `checked_u64_one`.
- `effective_time`: `member_id`, `codec_id`. The member belongs to Context.
  `unix_milliseconds` accepts an exact source `i64` field or a source type
  already bound to instant; `date` accepts the admitted date value. Only the
  selected signed 64-bit field is classified as raw instant. Instant uses the
  frozen signed 64-bit millisecond domain, including negative values.
- `idempotency`: `mode`, which is `disabled` in W04. `complete_snapshot` remains
  with W05 and cannot produce a successful W04 plan.
- each `accepted_commands` row: `case_id`, `condition`. Cases have unique IDs
  and typed Boolean conditions. A nonempty bounded list is required; logical
  coverage is a pending obligation, not inferred from the mere presence of rows.
- each `errors` row: `code`, `source_tag`, `condition`. The first two are exactly
  `version_conflict` and `version_exhausted`, with null conditions. Later rows
  have unique application error codes and typed Boolean conditions. Carriers
  are canonical source enum values, unique and exhaustive.

`state_invariant` is a Boolean expression scoped only to `state`.
Command/error conditions may also use `command` and `context`. Success event
and response relations additionally have `next_state`, `events`, and `response`.
All expressions use the existing frozen union, source-member identity checks,
constructor/property/binding environment, specialization roots, and shared
contract byte/depth/node limits. No expression evaluator is added to production.

## Exact pending recipes

`EmittedDataPhase::transitions()` exposes immutable validated plans and the
following obligations. Every obligation is explicitly undischarged.

1. Input invariant and explicit time's frozen value domain are preconditions.
2. Accepted command cases cover admitted commands. Error precedence tests
   `state.version != command.expected` first, then `state.version == u64::MAX`,
   then the declared business conditions in order. The first true guard fixes
   the exact result error arm/carrier; only absence of all guards admits new
   success. Source result/transition projection obligations remain pending too.
3. On new success, the invariant is explicitly rebound from `state` to
   `next_state`; checked `old_version + 1` must equal the returned version;
   the exact event/response predicates hold; event order is source array order,
   at most 4,096 events, and all returned values satisfy the common recursive
   value/collection and 65,536-cell bounds. Source arithmetic failures remain
   represented by the ordinary emitted exception checks.
4. Every error preserves the complete original input State, including any
   recursively stored array contents. This is not an equality assertion about
   the inactive success payload in an error result.

These recipes are attached to the original source, original sidecar and exact
binding identities. The VIR retains the canonical contract; its independent
importer recaptures sidecars and repeats attachment. Frontend manifests and
source artifacts retain transition references, including cumulative W02/W03
boundary runs. No new VIR field, proof rule, axiom or certificate is introduced.
T06-W08 owns semantic discharge of the transition and binding requirements.

Persistence, transactions, locks, retries, delivery, clock/identity generation,
authentication and transport are external assumptions. The selected closure
continues to use the existing effect firewall. Neither these assumptions nor
truthfulness of explicit context values becomes a proved application fact.

## Reproduction evidence

`Entry.cs` is an ordinary C# application with immutable product types, an error
enum and ordered events. `Run` is a second selected root used only by the
existing CLR observation harness to call the actual `Apply`; it exposes the
tag/carrier, new version/balance, event count/order/time, response and complete
original State. It does not participate in the transition contract.

`requests.json` and `responses.json` retain 37 source/sidecar cases captured
twice by the pinned local Linux compiler harness. The matrix includes seven
accepted attachments (five deliberately wrong implementations with pending
obligations), 27 rejected attachments, and three rejected external effects.
The seven accepted cases each have 66 actual CLR integer observations covering
success, every error, conflicting guards, zero/negative explicit time, event
order and input preservation. A test-only expression observer evaluates the
emitted invariant/event/response predicates against those observations; each
wrong implementation supplies the corresponding counterexample without being
misrepresented as a proof. The separate import test removes/splices contracts
and bindings and rejects source-fact reuse across captured snapshots.

```sh
cargo test -p mpk-cli --test csharp_practical_transition
./scripts/check-fast.sh
```

Capture command, offline in the existing `mpk-java-t10-gate:local` image on
`linux/amd64`:

```sh
./scripts/build-csharp-practical-frontend.sh --test-control-emission-requests < requests.json
```

`review.md` records resolved review findings and `verification.json` records
final local checks and file hashes. Frozen producer inventories, historical
capacity/recursor receipts, 709 published vectors and 26 vector sets remain
unchanged. W05 idempotency and public activation are outside this task.
