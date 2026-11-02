# Private ordinary Boolean foundation component

This component generates five actual ordinary definitions (Not, And, Or, Xor,
Equal) and five universal ordinary theorems (right identity cases, DoubleNot,
EqualReflexive) under `Mpk.Go.Practical.Bool`. It starts from the exact retained
zero-axiom standard Boolean and equality certificates, remapping their complete
name/level/term/declaration tables. It adds the already supported dependent
Boolean cases recursor and emits fresh closed certificate bytes. No new checker
rule, proof opcode, semantic axiom or source-callable adapter is introduced.

Double negation explicitly uses equality congruence/transitivity to rewrite a
computed major before constructor iota reduction. The constructor does not
assume a stronger reduction rule. The importer regenerates the whole component
and requires byte equality; a caller hash, export inventory or receipt cannot
supply an alternative definition set. All imports are empty because the pinned
standard definitions are included in the closed certificate.

The targeted integration test checks the fresh certificate with the current Rust
kernel, deterministic regeneration, exact import and a well-encoded incorrect
ordinary Eq Bool false true claim. It emits its exact observed bytes so the
current independently built Go checker can consume the same positive/negative
certificates. Only this new integration target is selected; no C# tests execute.
The first generation stops with missing-equality Linkage before checking. Its
[original inputs and diagnostics](initial/construction-failure.json) remain
retained; the correction includes the exact standard equality definitions.
The [verification record](verification.json) binds the current source and all
terminal results. Three targeted Rust tests pass in83.387 seconds including
compilation; strict Clippy passes in22.525 seconds and format in0.176 seconds.
The freshly built Go checker consumes the same two certificates: both accept
the3402-byte/435-term/26-declaration positive certificate with zero axioms and
matching complete hashes; both reject the well-encoded incorrect claim.

The first paired observer assumes Go always emits `axiom_count`. The exact
current Go CLI omits this uint64 when it is zero, so the original positive
checker acceptance is retained alongside the [observer failure](final/paired/result.json).
The corrected parser honors that exact contract and requires the Rust explicit
zero count and matching complete certificate/export/axiom-report hashes. Only
the two affected observation cases are rerun; the passing Rust tests and
unchanged Go checker build are retained.

This is a source-free Boolean basis. It does not complete the Go foundation's
word/Nat bridges, operation/source/state/frame/control/termination definitions,
all56 operation families or19 application theorem families. No private complete
foundation descriptor/registry/context is registered yet, and no production
immutable loader namespace is materialized. All169 source application demands
remain unelaborated. Native Linux acceptance and public profile activation stay
with later gates; T02-W01 remains In progress. The full T02 gate is deferred to
W04 (`check-fast.sh --skip-csharp`).

A [source-copy correction](initial/source-copy-correction.json) records that the
initial generator and integration test shared a basename. The initial copy
retained the test but overwrote the generator copy. The original generator is
now recovered byte for byte against its pre-execution SHA256 and length under
a distinct filename. Existing raw results/copies remain unchanged. Production
code and passing checks are unaffected; no runtime tests are repeated.
