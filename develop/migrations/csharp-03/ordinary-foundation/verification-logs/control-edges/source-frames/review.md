# Source local-slot frames — scoped review

The public edge generator appends a frame record for each original source graph
node, retaining its exact source identity, operation, native anchor endpoints
and original transfer where one exists. A frame preserves assignedness and
complete physical storage for every framed assigned local. Unassigned payloads
may differ. Store/pattern-bind targets are excluded because the separate
successful-transfer relation establishes their assignedness and new value.
Loads retain their slot frame but still require their separate load definedness
and SSA value relation. A frame does not prove that its operation executes.

Supported scalar, literal, conversion, pattern, immutable member/construction
and neutral control operations do not assign caller locals: source assignment
is a distinct store node. Section 8.3 of the design requires pure getters,
immutable source state, admitted by-value constructor/method arguments and
rejects ref/in/out arguments. This justifies caller-local framing; constructor
execution, result values and invariants remain separate obligations. Unknown
source operations stay pending, rather than receiving an identity fallback.

The two array update nodes stay pending for alias framing. Existing receiver
update relations alone cannot frame other aliases unchanged. Function entry
uses its separate initialization relation; exception search/throw/completion
execution is not established by these frames. Original graph reachability uses
both normal and exceptional edges without assuming any guard truth. Ten
unreachable source nodes receive explicit records without invented native
endpoints; this includes the four auxiliary jumps whose missing anchors were
identified in the initial run. The native emitter independently prunes source
nodes before building anchors.

The final eighteen-source regression covers 703 records: 580 frame definitions,
18 separate function entries, 93 exception nodes, ten unreachable nodes and two
pending array-alias frames. There are 10,608 core observations. Each emitted
frame uses nonzero assigned values, rejects altered output assignedness and a
physical-bit value corruption in every framed slot, and accepts differing
unassigned payloads. The corruption toggles the selected physical bit instead
of replacing the value, including small carriers. Metadata comparisons retain
all original nodes, slots, storage overrides, transfers and anchor endpoints;
imports reject changed or omitted frame records.

Seven historical declaration-preservation tests pass, including an exact
18-program prefix check against the preceding checkpoint. All nine private
source-slot pins replay unchanged because source frames append only in the
public edge generator. The other four source-file changes are metadata
re-exports, whose exact inverse hashes match the preceding source versions.
No new consumer path, core or checker behavior is introduced.

All fifteen distinct certificate byte sequences require fresh same-byte Go/Rust
acceptance and hash rejection before promotion; equivalent source contexts may
reuse a receipt only after exact byte identity. Actual completion is recorded
in verification.json. The final runtime bytes equal those checker inputs.
The preexisting 73,137 observations are retained through complete declaration
and metadata preservation; broad unrelated tests are not rerun.

These are source-local preservation constraints, not a complete native body.
Transfer values/definedness, array alias effects, exception state and execution,
reachability witnesses and application proof composition remain open. No unit
or W09 completion is claimed; the whole gate remains deferred to T06-W12.
