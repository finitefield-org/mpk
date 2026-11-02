# Actual variable shifts over complete64-bit counts

This T02-W01 checkpoint proves the actual source ShiftLeft, ShiftRight and
SignedShiftRight circuits, including large-count zero/sign fill, for arbitrary
8/16/32/64-bit words and every unsigned64-bit count. Actual signed-count
normalization/negative-count guards retain their preceding conversion proofs.
The component has4053 declarations,101285 terms,1454895 bytes and zero axioms,
with logical hash
`2431d715ee124e1a37b29458f045b523150989bd9f8d2ba17e680e2b3e3a4ac9`.
All4096/131072/2097152 caps and1024 core normalization fuel remain unchanged.
W01 and the complete source/program/foundation/application scope remain open.

The [count-bound producer](../../../../../../crates/mpk-vc/src/go_practical_ordinary/word_numeric/shifts/count_bounds.rs)
computes every low-prefix complement from real source count bits. A shared
ordinary Nat lemma proves Low+Complement+1=2^prefix-length and Low<2^length.
The already-proved high-count tree/zero reflection then give, for
d=log2(width), P(c)=PowerTwo(U64(c)) and H=the exact source Or of bitsd..63,

```text
H=false => U64(c)=Digits_d(c)
H=true  => U64(c)=width+Excess_d(c)
Excess_d(c)=Digits_d(c)+width*Sub(HighValue_d(c),1)
Nat.Less(U64(c),width)=Not(H)
H=true => P(c)=Modulus*PowerTwo(Excess_d(c))
```

The branch predicate does not restrict the operation domain: large unsigned
counts remain defined. Excess and all witnesses are actual ordinary definitions
computed from source bits; no truth flags, quotient or correction are supplied
by a caller. Powers stay symbolic and no huge unary count is constructed.

The [variable-stage producer](../../../../../../crates/mpk-vc/src/go_practical_ordinary/word_numeric/shifts/variable.rs)
uses the original source Select/fixed-shift circuits and original Digits_i
prefix exponents. Genuine Bool elimination proves value/top selection and
conditional PowerTwo/PowerMinusOne identities; explicit congruences transport
computed majors through Value/Nat/choice contexts. Shared ordinary scalar
composition proves all actual selected stages. Left discards are computed by
multiplying the preceding discard and adding the actual fixed-left discard;
right remainders add the preceding power times the actual fixed-right low block.
Strict Nat gaps compose their true bounds. Signed right stages preserve the
actual source top, including the exact whole-word Select behavior.

The [saturation producer](../../../../../../crates/mpk-vc/src/go_practical_ordinary/word_numeric/shifts/variable/saturation.rs)
binds the original final high-count selection to actual Zero or the actual
source-sign Select(Ones,Zero). With U unsigned value, M=2^width and b the source
sign for signed right (false for unsigned right), it proves

```text
U(actualLeft)+M*D = P(c)*U(input)
U(input)+M*PowerMinusOne(U64(c))*Bit(b) = P(c)*U(actualRight)+R
0 <= R < P(c)
```

At large counts D=PowerTwo(Excess)*U(input) and
R=U(input)+M*PowerMinusOne(Excess)*Bit(b). Fill value is Ones*Bit(b), with
Ones+1=M. A shared cancellation equation supplies the large-count conservation;
an actual input-complement and bit-complement gap supplies its strict remainder
bound. Unsigned right explicitly simplifies the false-fill term to give
U(input)=P(c)*U(actualResult)+R. No source high bit is cropped or rejected.

The [signed interpretation producer](../../../../../../crates/mpk-vc/src/go_practical_ordinary/word_numeric/shifts/variable/signed.rs)
uses actual established SignedPositive/SignedNegative and Int constructor
projection equations. For signed value S and actual result top bits,

```text
P(c)*S(input) ≈ S(actualLeft) + M*Int.Make(D+target-sign,P(c)*source-sign)
S(input) ≈ P(c)*S(actualSignedRight) + Int.FromNat(R),  0 <= R < P(c)
```

The second equation gives floor-style signed shifting, including negative
inputs, rather than truncating signed division. The private projection wrapper
only reuses the existing conversion projection generator; the complete old
conversion bytes remain exactly pinned. Every one of3105 previous named proof,
definition, level and family identities remains exact. The diagnostic
[metadata observation](retained-identities.json) additionally compares all
accepted3240/3352/3542/3685-declaration stages. Neither fingerprints nor exact
owning imports supply core/source/program authority.

The [verification record](verification.json) binds two affected Rust roots,
current selected Clippy/format and an ignored-helper listing. Both unchanged
production cores check85 complete same-byte streams: two zero-axiom positives
and83 genuine core type rejections. Controls change new real arithmetic
premises, count branch guards/excess/complements, actual shifts to well-typed
Zero results, actual count bit63 in each original high-count selection, computed
discard/remainder witnesses, signed Int interpretations and top/remainder bounds.
An honest valid extra declaration passes both cores but fails owning import.
No format, fuel or resource failure is counted as a semantic negative.

The [finite counterparts](finite-counterparts/result.json) reuse all504 exact
original typed native Go/actual Bool/integer observations because their source
circuits and all3105 named metadata remain exact. Unchanged sources/queries,
the two identical outputs and the original before/after5577-file prepared image
checks are revalidated; native compilation and the unchanged Bool suite are not
repeated. Python checks seven prefix complements per row, exact full-count
small/large branches,2586 actual stage observations, modular signed-left and
floor signed-right equations. There are24 real native negative-count panics,
480 defined results and309 defined high counts. Ninety-six huge-count rows
retain symbolic witnesses without enormous evaluated powers. No new Nat/Int/
witness/theorem bodies execute in the finite observer, which supplies no
universal theorem authority. Both unchanged cores separately check the proofs.

All original source snapshots, candidates and raw failures remain. The first
right-stage complete certificate fails in both cores at
W8.SignedRightStageTop1: Word.Select chooses whole functions, so top extraction
requires a genuine SelectTop Bool-case proof. That explicit theorem repairs the
recipe without a new conversion rule. The first ignored diagnostic fails its
relative path before prefix checking; the repaired repository path/captured old
3352 artifact and optimized ordinary diagnostic identify the original failure.
The private saturation-entry visibility error is repaired through a private
wrapper. No prefix acceptance is claimed. The outer subprocess wait is300s;
this observation wait changes no checker limit and grants no resource/Linux
release admission. The original60s observer remains archived. Only exact
per-file hash/length original libtest stdout EOF observations are allowed;
source, JSON and directory exclusions are not introduced.

All255 frozen design/specification/vector inputs and the official127-rule/
four-scope scanner with39 historical seals remain unchanged. The full source/
carrier/type/operation/storage/lifetime/state/program foundation, bounded56-
family assembly/descriptor/private registry/context and authoritative original
application source/export/selected-closure replay remain required. All169
application demands stay unelaborated and T02-T11 scope remains active. C# tests
stay skipped; the cumulative T02 gate stays W04:
`./scripts/check-fast.sh --skip-csharp`. No public profile, full program proof,
immutable application replay or native Linux release is claimed.
