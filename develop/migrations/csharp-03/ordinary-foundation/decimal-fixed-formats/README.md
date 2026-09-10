# Fixed decimal formatting — W09 unit 4 in progress

This component emits all 145 valid fixed-decimal configurations: scales 0..28
and ToEven, AwayFromZero, ToZero, ToNegativeInfinity and ToPositiveInfinity.
Each exported ordinary definition takes the concrete decimal carrier only; its
scale and rounding mode are static, explicit metadata and ordinary constants.
The five internal scale-argument functions remain monomorphic value functions.
Exact linkage from a captured sidecar/closed codec operation to these definitions
and the associated source and codec proofs remain required.

Rounding reuses the existing ordinary decimal round operations. Reducing scale
uses their discarded-digit/sticky-bit rules; increasing scale leaves the rounded
numeric value unchanged. The renderer emits every required trailing zero itself,
without multiplying a maximum 96-bit coefficient to create a larger numeric
value. Digit division is shared with normalized formatting. The common helper
extraction is intended to preserve normalized certificate bytes and is checked
by independent replay, not assumed to do so.

The rounded coefficient is divided into 29 reversed decimal digits. Layout
uses the rounded value's actual scale and the configured display scale
separately. Integer width is at least one; the point appears only for a nonzero
display scale. Fraction positions beyond the actual scale emit explicit ASCII
zeros. Zero suppresses a negative stored sign. Length and inactive output use
the full C19 text layout; valid output can reach 59 code units for negative
96-bit maximum at display scale 28.

The source suite reconstructs 65 captured contexts and requires six decimal
contexts with every 5-by-29 configuration. Its independent oracle matrix covers
all configurations plus sign, half-way/even-parity, sticky-tail, scaled-zero,
96-bit maximum, very small and scale-extension cases. It observes every active
output bit and header/padding bit and samples high/inactive index boundaries.
No host-computed formatting result becomes a production definition or proof.

Compilation, 65-source generation, exact normalized and fixed byte replay,
all six same-byte zero-axiom checker cases (286.362 seconds), hash corruption
rejection, lint and inventory passed. Programs have 34,535 terms and 518
declarations. Semantic execution remains ongoing and is tracked in
`../unit-4-decimal-fixed-format-progress.json`. Fixed parsing, universal
round-trip/source proofs, remaining codecs/JSON, and all original W09 units
remain open. No unit or W09 completion/commit is claimed; check-fast.sh remains
at T06-W12.
