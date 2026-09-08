# W09 internal unit 2: UTF-16 construction review

Baseline: `06996db`. This component does not complete W09 or unblock W10-W12.

## Scope and representation

Substring(start, length), two/three/four-string concatenation, the three
string/char operator combinations and each admitted restricted interpolation
shape have ordinary definitions. Character arguments retain all sixteen UTF-16
bits, including isolated surrogates and NUL. Null string arguments contribute
zero length to concatenation. Empty interpolation returns the empty string.
The output is the full depth-19 string cube; nullable inputs have depth 20.
Input-domain proofs and typed literals belong to subsequent W09 work.

## Direct review

- Substring checks the signs of both indices, start <= source length and length
  <= source length - start, avoiding overflow in a computed end index. Null
  receiver precedes range failure, which precedes output-bound failure.
- The full 32-bit summed length is checked against 16,384. Even the maximum
  structurally admitted argument count times the input bound fits in 32 bits.
  A failed operation exposes false success, one ordered failure and zero output.
- Output index words capture all fourteen address bits beneath the additional
  five word selectors. Substring adds start; concatenation subtracts the chosen
  segment's prefix length. First matching prefix end selects the segment, so
  empty segments cannot hide subsequent nonempty segments.
- All content outside the result length and all header padding are zero.
  The result length is bound once outside the nineteen output selectors.
- Helpers derive string nullability from the registered string signature,
  including when the first interpolation argument is a char or there are no
  arguments. Actual-source metadata is reconstructed rather than trusted.
- The initial 64-argument cutoff was incorrect: the registered binder limit
  is 256. It was replaced with that limit and the builder's composed-depth
  checks. An N-argument construction has depth N + 25. Tests accept 65 and
  231 arguments and reject 232 and 257. The pinned 231-argument certificate
  has 8,211 terms and 100 declarations; both unchanged checkers must accept it.
- Nine previous basic-string certificates retain their exact bytes. Seven
  original construction captures reconstruct exact operation sets, including
  interpolation .sc and .sscs and mixed Length/construction. Source, foundation,
  operation, check, result-name, byte and cross-context substitutions reject.
- The independent oracle supplies expected values only in tests. Actual core
  observations check success/failures, all length bits, thirteen header-padding
  probes, and all sixteen bits of selected content cells. Cells include the
  first 32, last active/first inactive, final storage slot and neighbours of
  every binary address carry. This is boundary sampling for long results,
  not an exhaustive observation of every physical bit of every long string.
- Ordinal comparison/search remains fail-closed. No application theorem or
  full-foundation completion is inferred from definition typing or observations.
- The additional implementation file is the sole new Std namespace consumer
  relative to the basic-string increment: 103 becomes 104. Aggregate count and
  the historical inventory hash link are updated without changing cache paths.

## Verification status

Final core, source-replay, same-byte dual-checker and lint checks are recorded in
`unit-2-string-construction-verification.json`. All recorded checks passed; the final direct review has zero findings.
The repository-wide `check-fast.sh` remains deferred to T06-W12.
