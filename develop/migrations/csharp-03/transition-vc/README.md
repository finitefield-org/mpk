# T06-W08: pure transition VC handoff

`TransitionVcProgram` rederives pending typed ordinary recipes from the exact
transition sidecars in independently validated source VIR. It retains all
original native functions, lossless canonical contracts, source snapshot DAGs,
ordered paths and sequents. The current VC wire binds the complete program hash
and sequent IDs; W08 groups depend on applicable W02-W07 groups. Standalone
import reconstructs every field from the original source and compares canonical
bytes, so a caller cannot validate an edited recipe using a replacement digest.

Admission explicitly equates the admitted-input predicate with public input
domains, method preconditions, input invariant definedness and truth, explicit
time validity and (when enabled) retained-key uniqueness. These are caller
requirements, not assertions that every representable State has a unique
history. Contract free subjects are substituted without capturing local binders;
W03's lexical and short-circuit definedness generator is shared unchanged.

The actual selected Apply result is projected through its validated bindings.
Success goals refer to its returned State, ordered Events and Response. New
success requires the next-state invariant, checked version increment, event
and response definedness/relations, 4096 event and 65536 aggregate-value bounds.
Pure/total source and complete input-State preservation remain mandatory goals.
Errors require the exact returned error arm/tag and unchanged input State.
Time comes from the explicit Context member. Persistence, locking, transport,
clock reads, identity generation and infrastructure idempotency are outside the
certificate's scope.

Complete-snapshot mode derives replay from a retained key and complete source
Command/Context equality. The captured total helper must agree with this
equality, which must also agree with canonical field encodings. Every stored
source member, including inactive fields and Missing/null/value distinctions,
is retained in the structural DAG. Non-reflexive storage rejects the claim.
Replay preserves State, emits no events and returns the complete stored
Response. New success appends the full snapshot and preserves prior history
order. Digests provide linkage only; they do not discharge equality.

The ordered partition is replay/snapshot conflict, version conflict, capacity,
version exhaustion, business errors, then new success. Disabled idempotency
omits its own branches. Fixed rows are selected by validated mode and ordinal,
so a disabled-mode business error may legally be named `history_capacity` or
`idempotency_conflict`. Business predicate definedness is required only when
its prefix is reached. Path identities distinguish errors from success/replay.
Pairwise partition size is bounded before cloning; all serialized path guards,
sequent terms, free/local binders and definition names enter resource accounting.

`goldens.json` pins 11 accepted original-source programs: seven T05 transition
cases, two T05 idempotency cases and two fresh Linux Roslyn captures for the
business-error name regression. Existing capture facts remain unchanged.
Tests check typed terms/signatures, exact strict reconstruction and hostile
omissions, Boolean branch partitions and replay priority, complete snapshot
members, and finite actual CLR counterexamples for broken invariant, version,
event, response and helper equality. The existing transition/idempotency source
matrix covers effects, non-reflexive storage, capacity and competing errors.
These finite observations are test oracles, not proof receipts.

W09 owns expansion of these pending recipes into ordinary definitions, proof
construction and identical-byte dual-checker verification. No checker rule or
certificate schema changes here. The full `./scripts/check-fast.sh` gate is
deferred to T06-W12, the final W of T06. Exact local verification results and
direct review are recorded alongside this file.
