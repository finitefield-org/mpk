# W09 unit 6: snapshots and retained history (scoped checks passed)

The new adapter independently reconstructs the W08 transition VC program from
validated original VIR, retains its hash and exact snapshot nodes, and checks
each rule, ordered child list and complete source-member inventory against the
shared structural DAG. Every node must be recursively total; type eligibility
precedes value inspection. No semantic binding projection is substituted for
complete source storage.

Each `Mpk.CSharp.Transition.SourceEqual.<type>` resolves to the existing ordinary
source-observation equality for its actual concrete carrier. The exact W08
`CanonicalFieldEncodingsEqual` predicate is the conjunction of those complete
equalities for command/retained-command and context/retained-context. All source
members participate, including storage omitted by semantic binding projections.
Existing scalar/sequence/product/sum semantics and ordinary proof-format limits
apply. The input representation and public domains remain caller obligations.

`HistoryCapacity4096` compares the exact retained-history sequence length with
4096. `RetainedKeyPresent`, `RetainedRecord`, and `RetainedKeysUnique` derive the
history, command-key, and record-key member identities from the W08 structural
terms and use a bounded finite fold over every active history element. Lookup
returns the first matching record. Its result is intentionally unspecified when
presence is false; every W08 call is guarded by `RetainedKeyPresent`. Uniqueness
requires each active record to be its key's first occurrence, so duplicate keys
reject while record order is preserved.

`AppendCompleteSnapshot` requires the next retained history to be exactly one
element longer and compares the last record's key, complete command, complete
context, and response with the successful call. `PreserveRetainedHistoryOrder`
uses complete record equality at each active old-history index. Together with
the successful path's existing no-key and capacity assumptions, these relations
preserve prior order and add exactly one fresh complete snapshot.

Every other W08 constant is retained in an explicit pending-definition list.
This does not prove that source equality or canonical serialization helpers
compute the expected relation, and it does not serialize canonical JSON.
Admission, replay and application proof assembly remain separate work.
Negative source examples may generate these correct expected-value definitions
without becoming accepted application proofs.

The targeted test replays all eleven W08 original contexts, compares snapshot
and unresolved-name inventories, evaluates independently encoded monomorphic
values against the structural oracle, changes one stored source member at a
time, evaluates all 18 paired command/context samples independently, exercises
history lengths 0/1/4095/4096, checks empty/single/distinct/duplicate lookup and
uniqueness cases, and rejects length, key, command, context, response, and old
prefix mutations of a successful append.
Metadata and certificate mutations must reject. The 11 metadata contexts
contain three distinct certificate byte sets; all pass Go/Rust acceptance on
identical bytes with matching reports, zero axioms and hash corruptions rejected.
Two source contexts contain seven snapshot nodes each; nine have no snapshot
obligations and do not exercise snapshot comparisons. The semantic cases pass
200 observations and detect 14 isolated member changes.
Module/re-export additions preserve all 18 control candidates and all three
standalone/integrated transition-product contexts. Lint and formatting pass;
see verification.json for terminal receipts and the inventory result.
W09 and unit 6 remain incomplete; the whole gate remains with T06-W12.
