# T02-W01 guarded frozen string substrings

The [ordinary component](../../../../../../crates/mpk-vc/src/go_practical_ordinary/string/slice.rs)
constructs the complete frozen substring carrier. Its 2,142 declarations,
67,192 terms and 865,248 bytes preserve all 1,729 previous named identities and
the exact 687,164-byte bounded-order producer. Closed full-width prerequisites
plus six substring definitions and 53 ordinary laws remain within the unchanged
4,096-declaration, 131,072-term, 2,097,152-byte and 1,024-fuel limits. There are no
axioms, new core rules, function extensionality or caller-selected roots.

The actual Boolean guard is equivalent to `0 <= low <= high <= len(s)` under
the existing balanced signed64 interpretation. Both sign bits and complete
unsigned comparison circuits are used. Result length comes from full64
subtraction. Each observable source position comes from actual full64 addition;
its exact natural value and absence of wrap are proved before use. Converting
a bounded output position to eight address bits requires its real strict bound.
In particular, a length of 256 retains its ninth bit.

Length and byte equations, upper capacity, canonical zero padding, validity,
existing observable content equivalence and normalization are proved. Input
capacity is a separate profile precondition, rather than a new Go panic.
Neither source-tail padding nor function equality is assumed to recover content.
The signed64 correspondence is mathematical: original Go source, types,
evaluation, nominal identity and the source-to-carrier relation still need binding.

The [validation receipt](verification.json) records five affected Rust roots,
strict target Clippy and format for the ten changed Rust files. Both unchanged
independent cores agree on [125 complete same-byte inputs](paired/result.json):
two zero-axiom acceptances and 123 genuine type mismatches. The latter change
53 conclusions, 66 actual typed witness domains and four typed definitions.
Actual definition countermodels demonstrate the wrong guard, all-zero data,
fixed source position and empty result. Every candidate is saved before checking.

Actual-DAG observations cover 39 complete carriers in raw and normalized form,
78 whole outputs, 307 signed guards and 113,263,317 AST steps. They include empty
ranges, NUL, invalid UTF-8, a multibyte UTF-8 sequence, both endpoints, lengths
255 and 256, negative endpoints, reversed ranges, oversized endpoints and the
extreme signed64 bit patterns. Only core constructors/eliminators and complete
input data are native. No arithmetic/string callback or theorem body supplies
an observation. Long output observations replay 64 slots at a time with the
same complete 256-slot input and all64 endpoint bits. The 50M-step, 2M-value
and 2M-environment ceilings are unchanged.

Eleven source epochs retain 4,251 complete original preimages and 13 checks,
including seven original failures. Three construction failures occur before a
candidate can be emitted; the missing fixed `W64.Add` dependency is added.
Two complete fuel-rejected candidates and their original outputs remain; explicit
word-equality congruence replaces implicit full-value conversion. The initial
format syntax failure and the observer ceiling failure, including every available
partial output, also remain. Fresh output replays repair the harness without
truncating inputs or increasing its ceilings. No original candidate or output
is reconstructed from a later pass.

The [input review](input-review.json) checks changed sources, complete decoded
data payloads, original failures, links and exact source preimages. All 255 frozen
inputs, 186 Go build inputs, 127 obsolete-IR matchers, four scopes and 39 seals
stay exact, with no new exclusion. Go and Rust remain the only source languages.
Unchanged arithmetic, frontend, native, core and protocol suites are retained
without repeat. Prefix/suffix, bounded concatenation, original source binding,
private foundation/context, native Linux acceptance and the remaining T02–T11
work remain due. R01/R06/W01 stay open. The cumulative T02 gate remains at W04:
`./scripts/check-fast.sh`.
