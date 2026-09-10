# W09 framed JSON string review (component verified)

The existing prefix parser leaves delimiter validation and absolute cursor handling
to its caller. The shared JSON environment now appends Frame.Parse, Frame.Header
and Frame.Value. Parse slices from the full-u32 caller position and binds Scan once.
Its original canonical byte/UTF-16 validation is reused. Finish independently checks
the original document limit, start order, end addition carry, end order, nonempty
consumption, successful scan termination and an exact live ending. All eight ending
bits participate; only 0 through 4 are admitted. Colon is available for decoded
field names without changing the scalar-value framing contract.

The C20 result has a role-zero C7 header and role-one C19 string. Packing explicitly
zeroes padding, and invalid header validity gates every result leaf. The header
contains validity, EOF, absolute end and decoded length; the ordinary string has
its usual length and UTF-16 storage. Projection binders use thirteen fixed zero
selectors followed by seven header selectors, or the value role and nineteen
selectors. The scan/state, header and result binder indices were inspected directly.

The change is appended after all existing lexical definitions. Source tests compare
all prior declarations and their transitive syntax/type/binder/dependency closures,
including the immediately preceding archived certificate. This preserves prior
semantic evidence rather than rerunning unchanged lexical matrices. Source-specific
structural integration and the full collection pipeline must still fit unchanged
practical limits and their changed bytes must pass both unchanged checkers.

All 44 targeted runtime cases passed. Coverage includes every valid ending, empty/ASCII/astral/control
strings, nonzero and high absolute positions, inactive delimiter bytes, original
full-u32 document bounds, wrong/unknown endings, noncanonical escapes, malformed
UTF-8, header bits, ordinary decoded length/units and result padding. All 68
original JSON contexts and four actual-source structural contexts passed full
dependency closure/import/mutation checks. The three tests passed in 231.56s.
The full collection helper integration passed at 12,905 cumulative transformers.
Token and collection certificates passed both unchanged checkers with zero
axioms and rejection of actual hash-corrupted bytes (86.44s and 54.21s).
Targeted clippy, formatting and the two affected consumer-edge tests passed.
The three changed source-specific structural certificates passed both checkers.
The first two passed in session 10205; its last case hit a Rust CLI rebuild
while the subsequent quoted-scalar edit was incomplete. That build failure was
kept separate from proof rejection and only the affected case was retried in
session 45111, which passed in 148.52s. All completed logs are retained.
Direct review of this component found no actionable issues. This does not close
unit 4 or the remaining W09 implementation/review requirements.

This is a typed-grammar prerequisite, not the complete object/array parser or a
proof of source/application VCs. Field-name uniqueness/order, presence and null
rules, nested cumulative limits, remaining codecs, source/native/transition
relations and actual proof/certificate assembly remain required by W09.
