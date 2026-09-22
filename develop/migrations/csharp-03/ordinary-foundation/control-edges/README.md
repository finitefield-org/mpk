# Source guards, edge transport and node entry

The generator reconstructs the original W03/W04 programs and retains the exact
source graph, edges, native guard inputs, slot identities and phi operands.
Missing semantic constants leave guards and their dependent relations pending.
No unknown exception or ownership check is replaced by a constant.

For an enabled edge, slot transport preserves assignedness and each assigned
slot's complete physical value from source exit to an edge-specific target
snapshot. Unassigned payloads are irrelevant; a disabled guard imposes no
transport constraint. Native phi relations perform simultaneous predecessor
copies, including complete private construction storage. A backedge result is
never identified with the preceding iteration's entry snapshot.

A separate node-entry relation selects exactly one incoming edge, requires
its guard and compares its target snapshot with a fresh entry state. Other
incoming states may differ. Missing/multiple selection, disabled selected
edges, assignedness changes and altered selected slot/phi values reject.
Unknown incoming guards keep the whole node relation pending. Function entry
initialization and successful load/store/pattern-bind relations are now emitted
in this same certificate, with their original source and SSA anchors.
When a flat entry would exceed the binder limit, exact argument mappings bind
an exactly-one selector and per-edge arrival relations as a conjunction. The
count/fill exit has 501 logical arguments and twelve components, each with at
most 81 arguments.
Execution producing source-exit snapshots and constraining the actual incoming
selection is still an application obligation; these are open state relations.

Successful array rewrites in two original sources connect receiver slots to exact
before/after memory snapshots, rejecting failed checks and stale source values.
The source expression returns the assigned element; the native invocation
returns the new array. Alias-aware update frames and exceptional execution
remain open.

Source-local frames preserve assignedness and complete physical storage for
assigned locals at exact source/native anchor endpoints. Store and pattern-bind
targets use separate transfer relations; unassigned payloads may differ. The
703 original-node records contain 580 frame definitions, 18 separate function
entries, 93 exception nodes, ten unreachable nodes and two pending alias frames.
These constraints alone do not establish operation execution or result values.

Native operation records bind original operands at invocation and the result at
the normal successor. Normal execution requires both the success guard and the
result relation; failure never becomes success by matching a default result.
The W04 loop/pattern subset contains 106 data operations, all with defined normal
relations, including Option constructors/presence and compound equality. One non-data invocation
and functions outside this W04 subset still require native integration. Exact
source-scoped ownership aliases cannot transfer to another operation point.

Array bindings resolve original allocation identities to exact current memory
SSA values. Internal bounded snapshots allow partial initialization; complete
public projections additionally require initialization of every active element.
Nullable slots retain explicit Option storage overrides while original source
slot metadata remains unchanged. The captured SwitchExpressionException throw
has a locally true guard only after checking its exact source/native anchor,
closed tag-8 literal, exception check and target. This does not prove reachability
or handler search.

The current 18-source corpus has 852 guards, 813 slot transports, 78 native phi
joins (140 values), and 746 node-entry records. There are 50,854 edge
observations, 22,257 entry/selection observations and 26 receiver-update
observations, 10,608 source-frame observations and 697 native-operation
observations, 6,342 integrated slot observations and 6,136 exceptional-result
observations, totaling 96,920. Original W04
binding comparisons cover 95 identities in the initial partition and 36 in the
collection partition, plus 68 in count/fill. The type fixture has no such
original goal bindings;
its storage tests do not prove logical observation mapping.

All 15 distinct certificate byte sequences covering the 18 sources pass both
unchanged checkers, matching reports, zero-axiom checks and hash-corruption
rejection. Existing term/declaration prefixes remain exact. The nine source-slot
pins replay unchanged. The latest extension retains the relation/storage caches
with the owning builder; its standalone family candidates remain byte-identical. See [current verification](../verification-logs/control-edges/verification.json)
and [source-slot integration review](../verification-logs/control-edges/integrated-slots/review.md).
Earlier checkpoint receipts are retained as historical evidence.

Remaining work includes other native data/exception families, W04 logical
observation mapping, remaining memory effects, source-exit production,
exception entry/search/finally and complete native/application proof composition.
The `count_fill` entry, edge and receiver-update relations are now covered;
complete loop execution, alias/exception framing and invariants remain open.
Unit 5 and W09 remain incomplete. The full gate is deferred to T06-W12.

The latest source-slot integration includes 18 function entries and 207 transfers
(114 loads, 88 stores and five pattern binds). Complete metadata and declaration
dependencies agree with standalone slot generation. Ten preservation tests and
the nine standalone pins pass; all fifteen distinct current certificates have
fresh same-byte dual-checker acceptance and hash-corruption rejection. Actual
execution witnesses and full application proofs remain open.


## Native exceptional invocation result checkpoint

The original W04 loop/pattern subset now has 59 ordinary exceptional-result
relations across 18 source contexts. Each relation requires the ordered failure
guard and the exact closed exception storage, and has no normal-result argument.
Its value ID, type, target and literal agree with the independently generated
W05 exception edge. Each individual check is exercised in both enabled and
disabled states; all 6,136 new observations pass. Twelve preservation/slot-candidate
tests retain the preceding term/declaration prefixes and metadata. Targeted lint,
format and the five inventory tests pass. All 15 distinct certificates pass both unchanged checkers, report agreement,
zero-axiom checks and hash-corruption rejection; see `../verification-logs/control-edges/native-exceptions/verification.json`
and its review for the final result before using the promoted pins.

These local relations do not establish source reachability, exceptional source
frames, handler/filter/finally execution or complete application proofs. Unit 5
and W09 remain incomplete, W10 remains blocked, and the whole gate stays deferred
to T06-W12. The review's earlier W05 value-identity finding was corrected before
promotion; obsolete candidate checks remain separate historical evidence.


## Native literal result checkpoint

All 101 retained native literal results in the 18 original control contexts now
have exact producing-node, value-ID and type bindings plus ordinary full-storage
equality relations in the same certificate. The 3,143 new observations cover
independent storage encoding and changed low/middle/last bits, including padding
and inactive sum arms. Thirteen preservation/slot checks retain every previous
term/declaration prefix and metadata, preserving the earlier 96,920 observations
(100,063 scoped observations in total). Standalone literal generation across 64
source contexts and the boundary-literal consumer retain their existing pins;
three shared-literal helper tests, inventory, targeted lint and format pass.
All 15 distinct extended certificates pass same-byte Go/Rust acceptance, matching
reports, zero axioms and hash-corruption rejection. See `../verification-logs/control-edges/native-literals/verification.json` and its review.

Literal results do not establish source-node reachability or compose a complete
execution. Incoming witnesses, alias/exception state, handlers/filter/finally,
transition/replay and application proof assembly remain open. Unit 5 and W09
remain incomplete; the whole T gate is deferred to T06-W12.

## Source normal-step composition checkpoint

The 18 original control contexts now include 579 complete local normal-step
predicates across 703 source nodes. Each predicate composes the exact source
frame, successful transfer, native normal result and literal result over one
shared SSA argument list. The remaining 124 nodes stay explicitly pending: 18
function entries, 93 exceptional nodes, ten unreachable nodes, two alias updates
and one non-data invocation.

The unbounded local runtime completed 20,584 observations in 8,193.415 seconds.
Its 36 files match the independent metadata candidate byte for byte. All 15
distinct certificates pass both unchanged checkers with matching reports, zero
axioms and hash-corruption rejection. See
`../verification-logs/control-edges/source-steps/verification.json`.

These relations assume their incoming source/SSA state. Entry selection,
inter-step transport, exceptional and alias execution, and complete application
proof assembly remain open. Unit 5 and W09 remain incomplete.

## Normal source execution checkpoint

The promoted program now contains 625 normal edge executions for the 579
complete local source steps. Each execution fixes the selected source entry,
bridges exact assignedness and assigned payloads into the local step, requires
the outgoing guard, join and optional phi, then selects that exact incoming edge
at the target entry. Return edges retain the complete source exit and guard
without inventing a target join.

The full reconstructed argument/component map remains available in memory.
Canonical metadata stores exact counts and its SHA-256 commitment so the largest
candidate remains 13,540,383 bytes, below the 16 MiB importer limit. Import
regenerates and compares the complete candidate. All serialized execution fields
and both owning collections reject mutation.

Three independent 36-file generations match the promoted fixture byte for byte.
All 15 distinct certificates pass unchanged Go/Rust checking with matching
reports, zero axioms and hash-corruption rejection. The prior 579-step metadata,
declaration preservation, inventory, clippy and format checks also pass. See
`../verification-logs/control-edges/source-executions/verification.json`.

Function entry initialization, exceptional and alias execution, ten unreachable
nodes, one non-data call and application proof assembly remain open. Unit 5 and
W09 remain incomplete.

## Complete reachable source execution checkpoint

The current program extends the retained normal-only checkpoint to 761 source
execution paths. Function entries, normal and exceptional paths, alias updates,
the constructor call and terminal steps are connected to their exact source
state and control relations. All reachable execution obligations in the 18
contexts are resolved. Ten structurally unreachable source IDs are stored in
`excluded_unreachable_source_node_ids` and independently checked; the pending
reachable execution set is empty.

The 703 source nodes comprise 582 complete local steps and 121 locally special
steps: 18 function entries, 93 exceptional steps and ten unreachable nodes.
Canonical execution metadata commits to as many as 366 arguments and 126
components; the largest JSON file is 13,595,052 bytes. Removal or forgery of
execution, pending or exclusion collections rejects during import.

Two independent 36-file metadata generations and these promoted fixtures match
byte for byte. All 15 distinct certificates pass the unchanged Go and Rust
checkers with matching reports and zero axioms. Both checkers reject all hash
corruptions. Source-step replay, declaration preservation, the five inventory
tests, targeted Clippy and format checks pass. See
`../verification-logs/control-edges/complete-source-executions/verification.json`.

Complete native-body integration and application proof assembly remain open.
Unit 5 and W09 remain incomplete, and the full T06 gate remains deferred to
T06-W12.
