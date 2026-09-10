# Integer codec contract component (W09 internal unit 4)

The ten requests preserve the original Quantifiers C# source and callable identity,
replacing only its method contract. Each contract exercises an integer codec
(or Duration/Instant codec) through `codec_format` and `codec_parse`.
Frontend capture accepted all ten original source/sidecar contexts.

Contract compilation reuses the standalone integer format and parse emitters.
The source test checks nominal recipe aliases and their complete transitive
ordinary definition closures against the standalone definitions. The independent
BoundaryCodec oracle checks actual aliased formatter output (length, characters,
header padding and inactive positions) and every parser result bit. Source
clauses check format/parse round-trip values and parse result tags. W03 format definedness
uses the actual complete 32-bit output length and the inclusive 16384 limit.

The ten source contexts passed 76 contract conditions, 40 formatter cases and
140 parser cases, and all 20 recipe dependency closures matched standalone
definitions. Ten completed candidates are pinned. All ten identical-byte
candidates passed both unchanged checkers with zero axioms, agreeing reports and hash corruption
rejection; current bytes, metadata and terminal evidence were reconciled.
This component does not implement decimal, hex or calendar codec contract
adapters, and does not prove application VCs.
The W09 eight-unit completion condition is unchanged; check-fast.sh is deferred
to T06-W12.
