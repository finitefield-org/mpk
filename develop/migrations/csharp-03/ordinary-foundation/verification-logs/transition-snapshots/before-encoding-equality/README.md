# Complete source snapshot equality definitions

The eleven metadata contexts replay the original W08 source contexts. Two
idempotency contexts contain seven complete snapshot nodes each; nine transition
contexts have no snapshot obligations. The two distinct certificate byte sets
are checked by both unchanged checkers with zero axioms and hash corruptions
rejected. Source-value equality covers all stored members, including values a
semantic binding projection might omit. Fourteen isolated member mutations are
detected in 140 ordinary observations.

These are expected-value definitions, not proofs that a source equality helper
implements them. The omitted-context source counterexample produces correct
expected relations while its application proof remains pending. All other W08
constants remain explicitly unresolved, including canonical field encoding
equivalence, admission, lookup, history and replay. No complete application
certificate is asserted. See
`../verification-logs/transition-snapshots/verification.json` and `review.md` in
that directory. Unit 6/W09 remain open; the full T06 gate stays with W12.
