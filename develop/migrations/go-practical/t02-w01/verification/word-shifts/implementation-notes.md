# In-progress actual variable shift implementation

Status: accepted fixed-right/count prerequisites; full variable-shift implementation remains in progress.
The [original dependency measurement](prerequisite-analysis/result.json) retains
all1882 conversion declarations plus783 exact original arithmetic/source shift
root dependencies,2665 selected. The existing Builder seed additionally retains
four exact original Std.Bool definitions, yielding2669 original declarations.
Current436 new laws give3105 total, leaving991 under4096. The original planning
measurement and failed count expectation remain; complete metadata includes the
seed entries. Full caps4096/131072/2097152,
all core rules, axioms/fuel and original32 conversions remain unchanged. All
three actual shift operations, full64 counts and saturation remain required;
none is replaced by a small-count-only operation. Steps1-3 and the high-count
balance/zero reflection in step4 now pass both unchanged cores. The low-prefix
bound in step4 and all later variable-stage/saturation/projection work remain. W01/T02-T11 and all169
application demands remain open. C# tests stay skipped; cumulativeT02 gateW04.

Implementation sequence (not proof authority):

1. Compose the three freshly generated complete original inputs with exact
   original names and full metadata overlap checks. The existing signed-division
   composer keeps its original substitution policy; its complete raw result must
   stay exact. Both production cores still check every assembled proof term.
2. Define ordinary PowerMinusOne(n) by0,2*previous+1, with unbounded induction
   proving PowerMinusOne(n)+1=PowerTwo(n), exponent addition and positivity.
   Closed small powers only use indices<=64, never huge unary data counts.
3. A shared PopHigh cube deletes the lowest bit and inserts a highest fill bit.
   Prove Ux+M*Bit(fill)=2*U(PopHigh(x,fill))+Bit(lowest x) by a generic two-block
   Nat compose law and small cube equalities lifted through Value. The actual
   top of PopHigh is fill. Generic cube value bounds use original complement
   equations. Actual fixed right shifts1,2,4,... are bound through all literal
   bits. Double steps reuse a single composition law; the discarded remainder
   is the actual lowest power-of-two source block, hence bounded. Signed fills
   preserve the source top and imply floor-style signed equations, not Go's
   truncating signed division.
4. For depthd=3..6, source High(count) is exactly the original balanced Or of
   count bitsd..63. HighValue is the matching balanced sum of relative weighted
   bits. Recursively compose original Digits prefix equations over the same
   range tree to prove U64count=Digits_d+PowerTwo(d)*HighValue. Keep proofs as
   balanced terms/checkpoints rather than emitting every linear prefix proof.
   A generic weighted-bit zero lemma and sum-zero reflection prove
   Nat.Equal(HighValue,0)=Not(High). No enumeration of whole count operands.
   A six-step complement balance bounds each actual low prefix below2^d.
5. High=false gives U64count=the real low prefix. High=true gives HighValue>0,
   HighValue=1+Sub(HighValue,1), and U64count=width+Excess, where the computed
   Excess=low+width*Sub(HighValue,1). Thus Nat.Less(U64count,width)=Not(High).
   PowerTwoAdd and original PowerTwoWeight prove
   PowerTwo(U64count)=M*PowerTwo(Excess) without evaluating giant powers.
6. Low-count stages use the actual source Select/fixed-shift circuits. Digits_i
   is their exponent, with the real count bit selecting the next fixed amount.
   Shared scalar laws compose actual left discards and actual right remainders.
   Bool equalities must be transported through Value/Nat/choice before literal
   reduction. Signed right stages retain the original source top. Remainder
   bounds compose through actual strict gaps, not supplied truth flags.
7. Final source High selection gives true full-count laws. Left saturation has
   result0 and computed discard PowerTwo(Excess)*Ux. For a general right fillb,
   let P=PowerTwo(count),E=PowerMinusOne(count). The universal Nat law is
   Ux+M*E*Bit(b)=P*UactualResult+R. At large counts, Ufill=(M-1)*Bit(b) and
   R=Ux+M*PowerMinusOne(Excess)*Bit(b). A complement gap proves R<P even for
   enormous counts. Unsigned fillb=false yields ordinary quotient/remainder
   conservation; actual signed fillb=sourceTop yields exact signed floor
   conservation after real top-bit equality and balanced-Int projection.
8. Actual left signed interpretation uses the computed correction with positive
   part Discard+targetTop and negative part P*sourceTop. Bind all source types
   and count normalization through established real converter/guard proofs.
   Full source/context/program/application authority remains separate work.

Finite validation must retain all504 original complete shift rows, both count
signedness routes, real negative-count panics and full high counts. The Bool
model executes actual circuits only. Host/native arithmetic must not materialize
2^(huge64-bitcount): represent the large-count branch symbolically and compare
actual zero/sign-fill outputs. Universal Nat/Int laws require complete proof
checking by both unchanged cores; finite models supply no theorem authority.
Every failure/source/candidate/raw diagnostic must remain. Only exact per-file
libtest EOF observations are allowed; no source/JSON/directory exclusions.
