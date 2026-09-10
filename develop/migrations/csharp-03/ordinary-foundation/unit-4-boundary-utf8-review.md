# W09 unit 4 boundary UTF-8 review (verification in progress)

This component generates ordinary byte-validity definitions over the full private
1 MiB document carrier. It is not the JSON grammar, typed field codec, source
invocation relation or a universal proof of a boundary/application VC. Units 3–8
and the original W09 exit gate remain open.

## Finding and correction

The first implementation used all 16,384 static transformer occurrences for
64-byte scan steps. The ordinary Boolean circuits also consume transformer
occurrences, so actual-source generation rejected with `Limit`. All four v1
source/semantic test processes terminated with that same generation failure;
none reached semantic validation. The bounds and Builder enforcement were kept.

The correction counts a two-Step64 composition and 8,192 invocations of that
composition, together with all circuit helper occurrences. The descriptor now
reports block bytes, blocks per step, outer scan steps and the actual total
Builder transformer count separately. This still scans every byte through
1,048,576. No counter reset, hidden recursion or checker extension was added.

## Direct implementation review

- The 32-bit cursor advances by 64 only while below the bounded document length.
  ReadBlock combines its aligned high cursor bits with six byte offset and
  three byte-bit selectors. The circuit masks inactive bytes using the full
  remaining length before using any byte classification result.
- The DFA tracks continuation count, the next-byte lower/upper bounds and a
  sticky failure bit. C0/C1 and F5–FF are invalid leads. E0, ED, F0 and F4
  restrict the first continuation to exclude overlong sequences, surrogates
  and scalars above U+10FFFF. Later continuations use 80–BF.
- A complete incoming DFA state followed only by active ASCII bytes preserves
  success while advancing the cursor. This is ordinary Boolean circuitry;
  every active byte's high bit is inspected. A pending multibyte sequence
  cannot use that path. An invalid state remains invalid.
- Guarded composition shares the first state and stops subsequent work at the
  document end or sticky failure. Finished rejects an outstanding continuation.
  Valid first guards the complete u32 document bound and independently rejects
  EF BB BF in the first three active bytes. An embedded BOM is valid UTF-8.
- Input storage in tests is a concrete Boolean cube. std::str::from_utf8 is an
  independent test oracle only. Empty data, NUL and other ASCII control bytes
  are valid UTF-8; their JSON restrictions belong to the subsequent grammar.
- Only actual boundary contracts enable the definitions. Exact regeneration
  binds source, foundation, boundary program, metadata and certificate bytes.
  Import rejects mutations and cross-context substitutions.

## Verification scope

The actual-source suite covers the existing 65 general contexts plus three
captured boundary document contexts. It requires three nonempty programs,
checks frozen structural limits and pins their exact bytes. The semantic suite
covers every single byte, every second byte for seven representative lead
classes, truncations, invalid later continuations and inactive storage. Separate
suites cover 64/16,384/65,536 crossings and three complete 1 MiB observations.
These suites and the same-byte dual-checker test must pass before scoped review
can close. Current commands/results are recorded in the companion progress JSON.

No component-only commit or W09 completion is claimed. The full local
check-fast.sh gate remains deferred to T06-W12.

Source generation/import/mutation passed all 68 contexts and three nonempty
programs in 84.48 seconds. Each generated certificate has 88,146 terms, 856
declarations and 8,992 counted static transformers, within frozen limits.
Targeted lint and formatting passed. All five inventory checks passed (24.87
seconds); shared document emitter/test visibility changes preserved all 68
contexts and three document certificate bytes exactly (67.38 seconds). Semantic
execution, UTF-8 pinned replay and both-checker verification remain in progress.

All three same-byte Rust/Go checker cases passed with zero axioms and corrupted
hash rejection (1190.346 seconds including CLI rebuild waits). Exact pinned
replay also passed across 68 contexts and three certificates (82.86 seconds).
Short semantic, cross-block and full 1 MiB execution remain live; these checker
results establish ordinary certificate acceptance, not universal UTF-8 laws or
application/boundary VC discharge.

## Full-capacity termination and state retention correction

The unpacked full 1 MiB suite terminated by SIGKILL during its first ASCII
case. Its log establishes no semantic verdict and does not establish why the
process was killed. It remains failed execution evidence, not a passed bound.
Review independently found that pointwise guard closures can retain unused
old states and that raw circuit result closures retain register histories.

PackState takes 64 Bool arguments in a closed environment and uses only Bool
recursors at the output leaves to demand all 64 arguments. SealState shares
one input state and passes its read leaves to PackState. Both Step64 and the
custom Compose seal the entire selected result; sealing only the selected
true branch would leave the surrounding Mux holding the unused old state.
Both concrete SealState occurrences are explicitly counted as one-element
S->S compositions, with no budget reset or capacity change. The evaluator and
both checker acceptance rules remain unchanged.

New regression tests check all 64 output leaves of the zero, all-one and 64
basis states; use weak references to require captured bit-environment release
after one packed leaf; and require a sealed guard to release an unselected
poisoned old-state branch without evaluating it. These checks plus new full
capacity execution, final pins/checkers and current semantic cases are pending.
Old in-flight semantic jobs are preserved as prior-revision observations.

## Packing dependency-walk correction

Directly nesting 64 Bool recursors with duplicate result branches caused the
unchanged certificate dependency walker to revisit the shared body at every
level. Its cycle-detection set is removed on return, so this construction
requires at least 2^64 subtree visits. This is a generation defect established
from the construction and walker, not inferred from test duration. The three
obsolete generation processes were explicitly interrupted; their terminal
results are retained in the progress receipt.

The ordinary ForceBit definition contains the duplicate recursor branches once.
PackState calls that definition with each body subtree once. No checker or
observer behavior changed. A regression measures the expanded PackState term
walk before Builder.finish: 585 visits, below the 4,096 guard. The old nested
construction exceeds the guard before entering the expensive collector. This
regression and current targeted clippy passed. Current source generation,
packing lifetime and full-bound tests remain live, so these results do not yet
close the semantic or certificate verification gates for the new revision.

The final packing lifetime regression passed in 6.10 seconds: all 66 states
preserve all leaves, the first output observation releases the pending input
environments, and an unused poisoned guard branch is released without being
evaluated. Current formatting also passed. Source generation and full-bound
execution progressed past an initial macOS loader wait without restarting;
the full-bound suite reached its first 1 MiB ASCII observation. Fresh short
semantic and boundary suites are running on the ForceBit revision. No passing
full-capacity result is claimed yet.

Final ForceBit source generation/import/mutations passed 68 contexts and three
nonempty programs in 104.21 seconds. Each current certificate has 88,486 terms,
859 declarations and 8,994 counted transformers. These pins replace the root
fixtures; the prior three certificates and manifest are preserved under
`boundary-utf8/previous-unpacked`, with old/new file hashes recorded in
`boundary-utf8/packing-correction.json`. Exact replay and both-checker checks
are now running against the current pins.
