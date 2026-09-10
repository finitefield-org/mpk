# W09 unit 4 normalized decimal formatting review (scoped review complete)

The scope is the normalized codec only. Fixed-scale rounding, parsing, exact
registered boundary linkage, universal codec/source proofs and all remaining
original W09 units remain required.

## Direct review

- Checked the independent unit-1 product addresses for sign, eight scale bits
  and 96 coefficient bits. Unused input storage does not enter arithmetic.
- Reviewed the 96-bit long division invariant: each remainder is 0..9 and each
  next partial dividend is 0..19. Twenty-nine quotient/remainder stages cover
  the full coefficient, including 2^64 and 2^96-1. The remainder stored above
  bit 95 is deliberately ignored when the pair becomes the next dividend.
- Counted the environments across digit, count, 28 trim-state and layout lets.
  Pair i has index 30 at trim stage i because each prior trim adds one binding.
  The final layout, trim and digit cube indices under text selectors agree with
  their declared cube types and the certificate checker verdicts.
- Trailing-zero removal is limited by the original scale and stops permanently
  on a nonzero digit. Zero receives a one-digit effective count and suppresses
  the sign even when the stored zero has negative sign and nonzero scale.
- Checked point/index arithmetic independently: integer width is at least one;
  the fractional digit index compensates for the point, while the trim count
  offsets into original reversed digits. Generated high quotient digits supply
  required leading zeros. Full 14-bit indices and length guard inactive text;
  all header padding is zero. Valid decimal outputs require at most 31 units.
- Production computes ordinary circuits and terms only. BoundaryCodec appears
  in tests as an independent formatting oracle, never as a production answer or
  proof. Domain validity of the input scale/coefficient remains an explicit
  precondition for future proposition assembly.

## Verification

Generation and exact metadata/hash/cross-context import checks completed for
65 captured contexts, producing six nonempty pinned programs. Both unchanged
checkers accepted their identical bytes with zero axioms and rejected corrupted
hashes (six cases, 92.206 seconds). The current stream-separated harness was used.
Independent pinned replay passed in 10.00 seconds, latest lint passed, and all
five inventory tests passed without a consumer count adjustment.

The completed semantic matrix compares actual ordinary outputs against the independent
model. It includes all scales 0..28, 96-bit boundaries, trailing-zero cohorts,
fractional leading zeros, negative values and signed/scaled zero. Every active
UTF-16 output bit and header bit is observed; inactive cells are sampled at the
first inactive position, high index bits and capacity boundaries. The complete matrix passed all 72 observations in 5841.65 seconds. Its running
executable predated shared helper extraction; the separately recorded exact-byte
replay confirms that extraction preserved all six generated certificates.

The initial unnecessary mutable circuit binding was removed before the successful
cargo check; its captured warning exposed an existing checker harness stream
mixing defect, recorded and fixed in checker-stream-review.md. After the complete semantic matrix passed, the final scoped review found no
additional issue. This conclusion covers only the normalized formatter definitions;
registered/source linkage and universal proofs remain open. No component-only commit or W09 receipt
is issued; check-fast.sh remains deferred to T06-W12.
