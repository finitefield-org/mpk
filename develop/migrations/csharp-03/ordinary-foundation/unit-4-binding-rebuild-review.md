# Source reconstruction review — partial W09

Reviewed binary reconstruction against the frozen binding requirement to
preserve source observations and the forward conversion's concrete layouts.

- A unary inverse cannot invent varying unmapped fields. Keep an explicit
  source completion argument, retain pending unary witnesses, and never map
  this binary symbol to the requested unary VC symbol. Completion admissibility
  and native/source-invariant proofs remain separate unfinished work.
- Retain unmapped product fields and inactive source-product payloads. Replace
  only semantic-mapped active fields, recursively, and restore exact original
  enum tags. An unknown semantic tag falls back to completion outside the
  required semantic domain; this is not acceptance of an unknown tag.
- Sequence output length comes from the semantic argument, and active cells
  use matching completion indices. Inactive output cells clear. Generic sums
  reconstruct the selected arm and its payload using the completion storage;
  same-type identity returns semantic storage directly.
- All generated functions use semantic/source argument depths and source result
  depth. Child definitions are cached by exact type pair; cycle, shape, linkage
  and encoded-size errors propagate. Canonical import recomputes bytes and
  metadata. Existing forward metadata and definition closures are preserved.
- Review finding: round-trip-only tests would also accept an implementation
  returning the completion argument unchanged. Added 111 different semantic
  inputs and a required count of changed outputs, with projection back to the
  requested semantic value. The first strengthened run exposed incomplete test
  inputs at an ordered-map payload. Explicit input preparation now supplies
  needed source cells/payloads rather than assuming nonexistent defaults.
- Added completion-length zero/overflow cases with explicit retained cells,
  and prior nonzero output addresses to detect stale cells after shrinking.
  Small carriers are checked exhaustively; large carriers use sparse address
  observations. These finite checks do not establish a universal theorem.
- The first Go invocation omitted the required build tag and ran no tests;
  its exit also reported a sandboxed Go cache write. It is excluded from
  verification. The replacement uses the existing writable temporary Go cache
  and the build tag, with an exact 45-fixture assertion in the harness.

The strengthened source/pin suite subsequently passed: 267 source observations,
63 changed outputs, two raw completion-length cases and 32,331 storage-bit
observations, together with all 45 fixed-source certificate replays and importer
mutations. All 45 same-byte checker cases passed in 166.916 seconds, including
zero-axiom checks and hash-corruption rejection. Latest scoped lint, inventory
and format checks passed. Direct rereview found no additional actionable issue
within this binary reconstruction component. Results are tracked in
`unit-4-binding-rebuild-progress.json`; the missing unary/source/native proofs
remain explicit. No unit or W09 completion, commit or push follows from this
partial component.
