# W09 unit 4 canonical JSON string parsing (verification in progress)

This component decodes an entire quoted JSON string document to C19 UTF-16.
Private Valid and Value definitions expose validity and a zero-on-invalid value;
no registered result identity or source invocation proof is invented. General
JSON/container grammar, typed field reconstruction, cumulative resource bounds
and the complete original W09 source/proof obligations remain open.

## Direct review

- A full-u32 document length guard requires 2..1,048,576 bytes and an opening
  quote. It initializes a failed state for invalid inputs. Captured bytes use
  the private document ReadByte function, preserving full index/length bounds.
- A finite packet circuit consumes one raw scalar, one canonical escape, or
  the final quote. It validates the complete local UTF-8 sequence, including
  continuation ranges, E0/ED/F0/F4 restrictions and required remaining bytes.
  Four-byte scalars become a high/low UTF-16 pair; BMP scalars become one unit.
  Raw controls, stray continuations, overlongs, raw surrogates and values above
  U+10FFFF reject. Literal slash and an embedded U+FEFF are valid string data.
- Only quote/backslash short escapes and lowercase unicode escapes for controls
  or lone surrogates are admitted. Escaping a printable scalar, short control
  escapes, escaped slash and uppercase hex reject. Header state records whether
  the preceding unit came from an escaped high surrogate; an immediately
  following escaped low surrogate rejects because that pair must use UTF-8.
- A closing quote succeeds only when it is the final document byte. Missing
  quotes, a truncated escape or multibyte sequence, and trailing data reject.
  The scanner stops only at failure or the final quote. Two counted packet
  steps and 8,193 outer steps cover 16,384 one-unit packets plus the closing
  quote, with every helper's static transformations counted as well.
- Header state retains the full byte cursor and decoded unit count, plus
  escaped-high/failure/done flags. Every append checks the new decoded count
  against 16,384 before writing. Failure preserves old unit storage, and the
  Value definition returns a completely zero cube on failure or noncompletion.
  A valid result converts the private C7 state header to the existing C5 text
  length layout. Initial, padding and never-written unit slots are zero.
- Actual source and foundation/boundary hashes control generation. Exact import
  rejects every metadata, certificate or context substitution. Uncontracted
  source contexts produce no parser definition. The existing Rust canonical
  parser/writer is used only as an independent test oracle, never as a proof or
  an ordinary implementation shortcut.

## Validation requirements

The source suite covers 68 actual contexts and requires three nonempty pinned
programs. The semantic matrix includes every one-byte string body, scalar width
edges, controls, quotes, lone surrogates and pairs, UTF-8 and canonicality
mutations, full-u32 document-length overflow and inactive-storage poison.
Full-capacity cases include all-control, ASCII and paired-surrogate strings,
plus decoded-length limit plus one. Full result length and bytes/units at every
address-bit crossing are observed; such finite observations do not establish
universal parsing/round-trip/source laws.

Production compilation passed (1m 22s). Remaining command state and exact results
are recorded in the progress JSON. Scoped review is not closed until those
checks pass. No component-only commit, unit completion or W09 completion is
claimed; check-fast.sh remains deferred to T06-W12.

The source suite passed all 68 contexts and three nonempty programs, with exact
import/mutation checks and empty quoted-string validity/value (43.77 seconds).
Each pinned certificate has 19,894 terms, 247 declarations and 8,340 counted
static transformations. Current lint (1m 35s including wait), formatting and all
five inventory checks (60.96 seconds) passed. Semantic execution reached case
48/296, including nonempty ASCII values, without failure at this observation.
Full-bound execution, exact pinned replay and both-checker verification remain
live. No scoped or W09 completion follows from the partial results.

All 296 semantic oracle cases, full-u32 document-length rejections and inactive
storage checks passed (888.34 seconds). All three same-byte Rust/Go checker
cases passed with zero axioms and hash-corruption rejection (172.622 seconds
including rebuild wait). Exact 68-context/three-certificate pinned replay passed
in 64.40 seconds. The four full-bound cases remain live; scoped review stays
open pending those results and does not discharge universal/source obligations.
