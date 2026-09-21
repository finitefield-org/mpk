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
initialization remains the separately generated source-slot entry relation.
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
observations and 10,608 source-frame observations, totaling 83,745. Original W04
binding comparisons cover 95 identities in the initial partition and 36 in the
collection partition, plus 68 in count/fill. The type fixture has no such
original goal bindings;
its storage tests do not prove logical observation mapping.

All 15 distinct certificate byte sequences covering the 18 sources pass both
unchanged checkers, matching reports, zero-axiom checks and hash-corruption
rejection. Existing term/declaration prefixes remain exact. The nine source-slot
pins replay unchanged; four additional source files only re-export new metadata
types. See [current verification](../verification-logs/control-edges/verification.json)
and [source-frame review](../verification-logs/control-edges/source-frames/review.md).
Earlier checkpoint receipts are retained as historical evidence.

Remaining work includes other native data/exception families, W04 logical
observation mapping, remaining memory effects, source-exit production,
exception entry/search/finally and complete native/application proof composition.
The `count_fill` entry, edge and receiver-update relations are now covered;
complete loop execution, alias/exception framing and invariants remain open.
Unit 5 and W09 remain incomplete. The full gate is deferred to T06-W12.
