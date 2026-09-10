# W09 signed JSON token checkpoint review

The source design and existing type-directed input/output conversion use raw
JSON tokens for small signed semantic integers. Metadata-only unsigned token
coverage is therefore insufficient. The new i8/i16/i32 wrappers retain the
existing unsigned/string/keyword definitions and append signed conversion.

An optional minus selects a Slice with start 0 or 1 and remaining original
length. Slice enforces the original full-u32 document bound, so subtracting a
sign cannot make an over-bound document acceptable. The finish circuit checks
the original bound independently, requires unsigned prefix validity, permits
magnitude through the signed minimum only for a negative input, and rejects
negative zero. Count adds the sign and whole-valid compares the result against
the original length. Value uses the exact width's two's-complement bits and
zeros the rest of the 64-bit slot. Every invalid packet is entirely zero.

The independent oracle parses the maximal optional-minus/ASCII-digit prefix,
checks canonical spelling and signed range using i128, then observes all 128
packet bits. Cases include each signed width's minima/maxima and adjacent
values, leading/negative zeros, signs, suffixes, large magnitudes, full-u32
lengths, inactive bytes and three 1-MiB-edge Slice compositions.

The source and exact pinned replay tests compared every prior term node,
declaration kind/body and resolved declaration/level name against the archived
unsigned certificates: all 39,066 terms and 533 declarations were preserved.
The unsigned 228-case matrix passed against that unchanged checkpoint.

The signed source/bounds/semantic suite passed in 689.12 seconds, including all
126 independent semantic cases and 24 full-u32 bounds plus three high-offset
Slice compositions. Exact replay passed in 75.87 seconds. All three signed
certificates passed same-byte Rust/Go checking with zero axioms and corrupted
hash rejection in 235.14 seconds. Targeted lint, formatting and inventory passed.
Direct review found no remaining issue in this signed lexical component.

The signed certificate checkpoint (46,834 terms, 644 declarations and 8,678
static transformers per certificate) is retained in json-tokens/previous-signed.
The current source appends scalar grammar consumers; their changed behavior and
preservation of this checkpoint require separate targeted verification.
Enclosing typed structure, cumulative limits, units 3–8 and W09 remain incomplete.
No component-only commit or push is made.
