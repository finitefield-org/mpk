# Ordinary literal representation edges

Two supplemental core certificates exercise the literal encoder directly.
`deep-padding.hex` covers product role order and a depth-253 constant, including
all 253 single-selector mutations and repeated child sharing. `scalar-storage.hex`
covers eight representations: f32/f64 signed zero and quiet/signaling NaN bit
patterns, signed scale-28 decimal zero and a non-normalized decimal cohort.
Their complete bit vectors are compared with explicit independent expectations.

These are constructed encoder regressions. The actual-source evidence is in
`../literal-definitions/` and must not be replaced by these cases. Certificates
contain definitions rather than native source/application VC proofs.
