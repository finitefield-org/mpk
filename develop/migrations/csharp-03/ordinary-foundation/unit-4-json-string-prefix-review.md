# W09 quoted-string prefix parsing review (verification in progress)

Typed object/array decoding must consume one canonical string token while
retaining the following delimiters and fields. The prior parser only accepted
an entire quoted document. The current scan stops at the first valid closing
quote and exposes PrefixValid, PrefixValue and Consumed. Consumed includes both
quotes and is zero on failure; PrefixValue is also all-zero on failure. Prefix
success does not validate suffix bytes or grant whole-document acceptance.

Whole Valid/Value additionally compare the final cursor to the complete u32
document length. Both outputs share the same scan definition as the prefix
operations; no second 8,193-step pipeline is emitted. Opening-quote/minimum-size
and full 1 MiB checks remain in InitialBad. Every packet still checks its full
remaining byte count; cursor advance cannot pass the bounded length. Canonical
UTF-8, lowercase escapes, surrogate-pair restrictions and the 16,384 decoded
UTF-16-unit bound remain unchanged.

Value wrappers bind Scan with an ordinary Let. Inside the nineteen value
selectors, the state is Var(19) and the source is Var(20); the whole-value
predicate must inspect the actual source length, not the state or a selector.
Consumed uses the same shared-state pattern with five output selectors.

The document fragment emitter now accepts the already emitted document
definitions. The parser and Slice/Concat therefore inhabit the same certificate
and share the private document carrier without duplicate definitions or extra
scan budgets. The standalone fragment generator still emits the document first;
its exact old pins must remain unchanged. Prefix composition tests pass the
ordinary slice closure directly to the parser, never copying or re-encoding its
bytes through the host.

Tests retain the complete whole-document regression and add 62 independent
prefix-oracle cases. The oracle tries exact short prefixes with the separate
canonical JSON parser and independently determines the first token endpoint
and UTF-16 value. Trailing whitespace, another string, invalid UTF-8 and JSON
punctuation distinguish prefix success from whole-document rejection. Other
cases cover escaped quotes, lone surrogates, scalar pairs, malformed/truncated
escapes and UTF-8. Document/u32 bounds and 12 Slice-to-parser compositions reach
high offsets through the 1 MiB limit. Current whole and prefix full-decoded-bound
execution is required; historical long-running whole-only tests are not current
revision evidence.

Compilation passed for the initial prefix change. Final generation, current
semantic/full-bound runs, shared-fragment replay and lint are running. The root
json-string-parsers pins still describe the old whole-only revision and must be
archived/replaced after current generation passes. General JSON grammar and
universal typed/source proofs remain open. Units 3–8 and W09 are incomplete;
no component-only commit or final zero-finding review is claimed. The full
repository gate stays deferred to T06-W12.

Current generation/import/mutations passed all 68 contexts and three programs
in 59.74 seconds. New pins have 24,828 terms, 342 declarations and 8,406 counted
transformers; the old pins/manifest are retained in previous-whole-only/ with
old/new hashes recorded in prefix-extension.json. Exact current replay passed
in 50.14 seconds. All three same-byte checker cases accepted with zero axioms
and rejected corrupted hashes (138.026 seconds including build). All five
inventory tests passed; current production lint/format checks passed as well.

The shared fragment emitter preserved all 68 contexts and three standalone
certificate byte sequences exactly (35.49s). Its completed semantic suite passed
both subtests in 561.49s. The prefix document/u32 bounds and twelve actual
Slice-to-parser compositions passed. The 62-case prefix matrix and 296-case
whole-document regression remain live, as do current full-bound tests.

Review added a full-control prefix observation: 16,384 decoded control units
consume exactly 98,306 bytes before the following comma. Full ASCII capacity
alone would not test consumed-byte bit 16. This extra case and its test-only
lint/format checks are running; production and pinned bytes are unchanged.
No current full-capacity completion is claimed before those runs finish.

The complete prefix semantic suite has now passed (session 45385): all 62
prefix oracle cases and the document/full-u32 plus 12 ordinary high-offset
Slice compositions. The whole-document regression and full decoded/control
capacity suites remain separate pending evidence in the progress receipt.

The current whole-document regression also passed: 296 oracle cases plus
full-u32/inactive storage cases in 1474.53 seconds (session 95295). Full decoded
capacity and full control-byte consumption tests remain live and are not
covered by the short matrices.
