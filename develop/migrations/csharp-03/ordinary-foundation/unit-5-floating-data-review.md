# W09 floating data connection review

The adapter independently reconstructs W03 definitions and use points from
validated VIR. It selects floating and numeric-conversion signatures in the
FloatingDecimal family, checks the exact frozen scalar signature and failure
tables, and rejects multiple checks or tagged-result failures. The actual scalar
bodies are unchanged. Arithmetic has no failure checks and checked conversion
has only overflow, so no earlier exception can mask its individual predicate.

The result relation compares all physical Boolean-cube bits, including sign,
NaN payload and high double/integer bits. It uses the existing result comparator
and original subject-index to de Bruijn conversion. Success guard/relation/goal
and failure prefix/predicate/guard preserve source/function/node, SSA identity
and successor linkage. Other families remain explicit pending definitions.

Three native sources yield eight definition occurrences and ten use points.
Existing scalar dependency closures match exactly, independent import
reconstructs canonical bytes, and successor/byte mutations reject. New code
passes lint, inventory and scoped formatting. All 84 result/guard cases pass
in the optimized build, including signed zeros, NaN payload negation/comparison,
infinities, subnormal addition, rounding and checked-conversion overflow.
Wrong high result bits reject. All three outputs match candidate metadata and
certificate pins; both unchanged checkers accept the same canonical bytes with
zero axioms and matching reports/hashes, and reject hash corruption.

This scope covers the seven native operations listed in the progress receipt,
not all native floating signatures, complete native bodies, or application
proofs. No actionable findings remain in this scoped review after the selected
verification passed. Original units 3-8 remain open.
