# W09 unit 4 canonical JSON string encoding (verification in progress)

This component encodes an admitted C19 UTF-16 string to a quoted C24 byte
document. It does not yet implement JSON parsing, object/field order or duplicate
checks, typed reconstruction, registered invocation results, or universal
boundary/application VC proofs. The full original W09 plan remains required.

## Direct review

- The encoder reads the complete 32-bit input length and admits at most 16,384
  units. ReadUnit separately checks both the active length and physical storage
  limit before selecting fourteen address bits. A high surrogate at the final
  active position cannot pair with inactive storage.
- A finite ordinary packet circuit encodes controls as six-byte lowercase
  unicode escapes, quote/backslash as two-byte escapes, and other Unicode
  scalar values as shortest UTF-8. It preserves lone high or low surrogates as
  lowercase unicode escapes. A high/low pair overrides the lone-surrogate path,
  emits one scalar and advances two input units. The packet's unused bits and
  bytes are zero. No host encoder output defines this circuit.
- State uses the private C24 byte layout with a C6 header: full input cursor and
  output count. The initial buffer is a single opening quote. Each update writes
  only its output interval and preserves preceding bytes, then increments both
  cursors independently. Header padding is zero. A closing-quote packet has
  zero input advance, and the final state is shared before converting to the
  document carrier with its different header layout.
- The guarded pair step and 8,192 outer steps cover all 16,384 input units even
  when every packet consumes only one. All concrete composition occurrences
  and packet/helper circuit transformations count toward the unchanged Builder
  limit. No recursion, counter reset or C# checker rule is introduced.
- The worst output is 98,306 bytes (six bytes per input unit plus two quotes),
  below the existing 1 MiB document capacity. Out-of-bound inputs have a false
  Bounded predicate and a zero document output; they are not accepted encodings.
- Generated programs bind actual source/foundation/boundary identities and
  enforce exact metadata/certificate regeneration. Only the three actual
  boundary contexts enable the definition; the remaining source contexts must
  remain empty.

## Verification

The source suite covers 68 contexts, requires three nonempty programs and pins
actual ordinary certificates. The semantic suite checks 344 cases against the
existing independent canonical UTF-16 writer: every unit 0..255, scalar-width
edges, all surrogate category pairs, controls and quoting. It also checks three
full-u32 over-bound lengths and three inactive-storage cases. A separate full
capacity test verifies the exact output length and bytes at both ends and every
storage address-bit crossing for all-control and all-surrogate-pair inputs.
These sampled output observations do not prove universal encoding correctness;
all original ordinary/source proof requirements remain open.

The initial source-test build failed because a newly added core-value read helper
shadowed the fixture read function. The fixture call now names its parent module.
No production semantics were inferred from that build failure. Production
compilation passed before the final shared-state optimization; the current
source, lint and semantic builds/results are tracked in the progress record.

No component-only commit, internal-unit completion or W09 completion is claimed.
check-fast.sh remains deferred to T06-W12.

The current production and integration tests passed clippy with warnings denied
(4m 25s including build wait), and formatting passed. Source/semantic/full-bound
execution and actual certificate checking remain pending.

All 68 source contexts passed regeneration/import and mutation tests, including
three nonempty programs and empty-string encoding (32.00 seconds). Each pinned
certificate has 12,971 terms, 174 declarations and 8,292 counted static
transformers. All five inventory tests passed (47.13 seconds). Same-byte dual
checking, exact pinned replay, 344-case semantic execution and both full-capacity
observations remain in progress; no scoped completion is claimed yet.

The complete 68-context/three-certificate pinned replay passed in 43.63 seconds.
All three identical-byte Rust/Go checker cases passed in 69.851 seconds, with
zero axioms and hash-corruption rejection. Semantic and full-capacity execution
remain running; these certificate acceptance results do not discharge source
commutation or universal encoding laws.

All 344 semantic encoder cases, full-u32 bound rejections and inactive storage
checks passed in 1970.70 seconds. The complete source/pin/checker/lint/inventory
results are already recorded. Both full-capacity cases remain live, so scoped
review and the original universal/source obligations remain open.
