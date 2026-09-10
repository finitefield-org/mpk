# W09 JSON keyword decoding component review

The existing canonical boundary route needs ordinary token recognition before
schema-typed parsing can distinguish null from Boolean values. This emitter
matches the exact lowercase ASCII words null, false and true over the existing
C24 document. Source context, foundation and boundary identities are retained
and import reconstructs exact metadata/certificate bytes from validated VIR.
No public route, source value carrier, checker rule or proof axiom is added.

The matcher separately checks the full u32 length is within 1 MiB and covers
all token bytes. Nonzero inactive storage cannot make a truncated token valid.
A success packet contains prefix-valid, whole-valid (exact length equality),
is-null, Boolean payload and a full u32 count of four or five bytes. The three
mutually exclusive literal matches select over an all-zero packet. Thus failure,
null/false payload bits and every unused result bit have explicit zero behavior.
Prefix success deliberately does not validate a following separator or suffix;
the enclosing parser must discharge that remaining grammar obligation.

The independent byte oracle tests every first byte for the true-shaped input,
every single-bit mutation at each token byte, every truncation, leading BOM/
whitespace/quote, and valid/arbitrary suffixes. Every result bit is observed.
The bounds suite keeps all physical document/index bits: full u32 lengths,
nonzero inactive token bytes, 12 high-offset ordinary Slice-to-parser
compositions and 16 token-split Concat-to-parser compositions. The original
source suite checks 68 contexts, three nonempty programs, exact import and
metadata/cross-source/certificate mutations. These observations are not
universal theorem proofs or a complete typed JSON grammar.

The final direct component review checked each packet branch, the full-u32
length guard and exact-consumption condition, all-zero failure/padding,
shared Slice/Concat composition, context/metadata reconstruction and source
mutation rejection. No remaining component findings were identified.

The three targeted tests passed in 266.63 seconds: 401 lexical cases with all
64 result bits, 33 length cases, 12 slices, 16 concatenations and 68 source
contexts/three nonempty programs. Exact pin replay passed in 30.82 seconds.
Each certificate has 11,122 terms, 167 declarations and 109 counted static
transformers. All three passed same-byte Rust/Go checking with zero axioms and
hash mutation rejection (12.680 seconds including startup). All five inventory
tests passed in 44.27 seconds; targeted clippy and format checks passed.

Two invocation/staging errors are retained in the progress receipt: an
unsupported test option was stopped before tests ran, and an incorrect raw
SHA-256 comparison prevented initial pin copying, causing expected missing-pin
failures in the first checker/replay runs. Staging was corrected to validate
the existing MPK-MODULE-CERT-0.1 domain hash before copying. The complete corrected
replay and checker runs passed; no certificate/checker acceptance rule changed.

This review closes only the keyword component. Full typed JSON grammar,
source/control/transition relations and universal application proofs remain
open. Units 3–8 and W09 are incomplete; no component-only commit is made.
The whole check-fast.sh gate remains deferred to T06-W12.
