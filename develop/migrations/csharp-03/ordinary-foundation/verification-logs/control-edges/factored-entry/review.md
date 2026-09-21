# Factored node-entry relations — scoped review

The original `count_fill` exit has eleven incoming edges and needs 501 logical
arguments (shared state and distinct predecessor snapshots). The previous flat
emitter rejected it at the 256-argument binder bound. The generator now keeps
that exact ordered argument metadata and emits twelve conjunctive components:
one exactly-one selector and eleven selected-arrival checks. The largest
component has 81 arguments. A component refers directly to parent argument
indices; no independently claimed guard/arrival-success Boolean is accepted.

Each selected arrival requires its exact existing guard, equal assignedness,
complete physical equality for assigned slot values and unconditional equality
for native phi values. Unassigned slot payloads are irrelevant. Unselected
arrivals impose no snapshot constraint. Selection and arrivals refer to the
same selection indices and fresh shared entry state. Missing guards still
leave the entire entry pending. Compact entries preserve their previous single
relation and byte-identical metadata/certificate.

Before expanding metadata, the generator bounds the selector and each arrival
against the binder limit. The global metadata and certificate structural limits
remain enforced. If individual components cannot fit, generation still fails.
No core/checker changes or new source-consumer paths are introduced.

The final original-source regressions check 11,904 edge/transport/phi/native
memory observations, all 129 node entries in 6,633 observations using the
independent guard oracle, all 2,048 selector inputs, and thirteen receiver
rewrite observations. This totals 20,598 new observations. The earlier 497
focused arrival checks against separately evaluated guards are retained as a
cross-check and are not counted twice. There are 68 exact original W04 goal
binding comparisons. Import rejects omitted components, wrong roles/definitions
and altered argument mappings.

The expanded edge test initially assumed every memory slot needed a public
projection. The captured array temporary already has the native carrier. The
corrected test requires exactly one such temporary, exact type equality and
absent public projection fields; three public bindings still require the full
projection/snapshot definitions. Both representations retain all native-memory
equality checks and exact ownership-state resolution. The affected
`index_update` regression reran and retained its 4,559 observations. This is a
test applicability correction, not a relaxation of source array projection.

The 902,957-byte certificate passes Go/Rust acceptance with matching reports,
zero axioms and hash-corruption rejection. Go took 425.910/434.939 seconds for
acceptance/hash rejection; Rust took 1.658/1.639 seconds. The agent ran all tests,
including these long checks. Actual receipts and fingerprints are retained.

All seventeen prior source programs are byte-identical. Their existing semantic
observations are retained through exact artifact preservation. The private slot
builder does not generate node entries; its nine pins remain unchanged.

The receiver update uses native rewrite semantics, despite the count/fill
fixture name. Its before/after snapshots, both write indices, assignedness,
initialization, bounds, stale memory and source/ownership binding mutations are
covered. These scoped relational checks do not establish source-exit production,
alias framing, exceptional execution, loop invariants or application proofs.
These remain W09 obligations.
Passing zero-axiom core checking establishes well-typed ordinary definitions,
not proof of an entire source program. Unit 5 and W09 remain incomplete.

After promotion, all three pinned entry/selector/receiver tests pass in 14.83
test seconds. Artifact hashes, all source fingerprints, eighteen contexts,
fifteen distinct certificate files and the 73,137 observation partition were
reconciled with the current verification record. No test processes remain live.
