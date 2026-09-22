# Additional count/domain consumer pin review

Status: source regeneration, strict import, declaration review, selected
semantic regressions and promotion passed. The two checker batches listed in
`verification.json` must finish before this consumer refresh is complete.

The affected-consumer inventory found 268 source-bound programs in 137 original
VIR contexts, plus six source-free helper certificates. The regeneration driver
reconstructs the captured context and facts, emits the data phase, matches its
exact VIR hash and invokes the original ordinary generator and strict importer.
The initial request/response inventory found 261 programs. The seven remaining
integer-parser contexts use existing data-stage replay rows; the continuation
found every original hash. `initial-generation-run.json` retains the initial
missing-context failure rather than presenting it as a successful full run.

The declaration comparator uses the existing independent syntax comparison by
resolved global names. Term-table numbering and sharing may differ. Its driver
compares all common types and bodies in batches; on a mismatch it enumerates
individual names. This changes comparison cost, not the comparison relation.
The exact temporary driver sources are preserved in `tool-sources/` as evidence.

For 262 source programs, every retained declaration type is unchanged, the only
changed body is saturated logical-cell addition, and deletions are its obsolete
register/cube helpers. Six programs also retain earlier domain implementations:
the conditional, multiowner, literal, structural and total clause integrations,
and the combined date/time/decimal structural boundary program. Their additional
changes are confined to domain counts, all-zero regions and inactive sequence
tails. The complete name lists remain in `declaration-audit.json`.

The domain review checks that constant-one folding is limited to full-width
unconstrained bit leaves without public clauses. Product padding and public
clauses remain checked. Sequence counts still validate active elements, declared
length and all physical padding/inactive slots. Fixed zero-region definitions
cover both selector branches. The existing recursive-domain tests passed, and
all six original source tests regenerated the exact migrated bytes while
retaining their independent value assertions and component-closure comparisons.
No parser, source clause, codec or other retained body/type changed in these
six cases. `additional-domain-review.json` resolves their explicit raw-audit
findings; it does not relabel them as count-helper-only differences.

The six helper tests cover decimal/calendar/JSON collection integration budgets,
all primitive codec routes, typed JSON depth including 252 active/overflow
cases, typed-node saturation and container counts. The shared JSON integration
also compares its standalone bytes with the newly regenerated source pin.
These certificates have no source context and are recorded separately.

Promotion updates 268 source pins, six helper pins and their current canonical
metadata copies. Allowed metadata changes are certificate hashes, metadata-file
hashes, byte sizes and measured term/declaration/transformer counts. Source,
foundation, contract, VC, type, predicate, failure-order and observed-result
metadata are preserved. The original formatting of hex pins and canonical
program JSON is retained. Historical extension receipts are not rewritten.

The 274 occurrences contain 224 distinct raw byte sequences. The checker
selection covers each once, including zero-axiom acceptance, matching Go/Rust
reports and rejection of actual hash-corrupted bytes. Duplicate occurrences
retain exact byte/hash links to their representative. The complete retained
body/type comparison and the targeted helper/domain tests determine this
rerun's scope; unrelated scalar and native-body tests are not repeated.

This is a scoped repair of consumers affected by the prior helper/domain
changes, not a repository-wide gate or application proof. Unit 4 and W09 remain
in progress; the full T06 gate stays deferred to W12.
