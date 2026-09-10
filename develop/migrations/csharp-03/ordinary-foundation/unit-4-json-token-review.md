# W09 unsigned JSON token checkpoint review

This checkpoint combines canonical string/keyword functions and u8/u16/u32/u64
raw unsigned token parsers over one C24 document/fragment environment. The
certificates and manifest are archived in json-tokens/previous-unsigned while
the signed extension is implemented and verified separately.

The C7 scan retains a u64 accumulator, five-bit count, failure and stopped flags.
Twenty counted steps cover every valid u64 token. Each active digit checks
leading zero and the exact MAX/10 threshold before retaining the arithmetic
result. Failure is sticky. Finishing rejects empty/failed scans, a remaining
digit (including a 21st digit), over-bound full-u32 document lengths and values
outside the selected width. Exact consumed==length distinguishes whole success.
Invalid packets and every unused bit are zero. Suffix delimiters remain the
enclosing grammar's responsibility; prefix success does not admit fractions,
exponents or arbitrary trailing bytes as a complete typed document.

Review found duplicate C7 helper registration in the first shared program.
All three integrated tests reproduced Linkage during generation. The fix checks
for the existing Compose declaration before helper emission, matching the
established circuit emitter; an archived string certificate independently
confirms the helper was already present. Corrected generation, semantics and
checker verification all passed. No remaining unsigned-checkpoint findings were
identified after reviewing the complete circuit, packet and shared environment.

The corrected three-test suite passed in 1045.51 seconds: 228 independent
prefix/value/range cases with all 128 packet bits, 32 full-u32 length cases,
four high-offset slices, shared string/keyword consumers and 68 original source
contexts/three nonempty programs with exact import and mutation rejection.
Every certificate has 39,066 terms, 533 declarations and 8,585 counted static
transformers. Exact pinned replay passed in 75.52 seconds. The three identical-
byte Rust/Go cases passed with zero axioms and hash-corruption rejection in
171.269 seconds including startup. Inventory's five cases, corrected lint,
format and Git whitespace checks passed.

The prior standalone string/keyword emitters preserve all old metadata and
certificate bytes after sharing fragments: both 68-context/three-certificate
replays passed in 83.62 seconds. The startup delay was observed at dyld entry;
no evaluator frames occurred in that sample, and code signature verification
passed. It is not classified as an implementation failure or diagnosed cause.

The source design and existing typed conversion also require signed i8/i16/i32
raw tokens; semantic i64/u64 remain quoted. The signed extension is tracked in
unit-4-json-signed-token-progress.json and must preserve every prior ordinary
term/declaration. This archived checkpoint does not verify that unfinished
extension, full typed JSON grammar, source/control/transition relations or
universal application proofs. Units 3–8 and W09 remain incomplete; no component-
only commit is made, and check-fast.sh is deferred to T06-W12.
