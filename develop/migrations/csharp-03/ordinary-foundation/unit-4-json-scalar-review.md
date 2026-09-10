# W09 scalar JSON grammar consumer review (checkpoint verified)

The lexical parsers alone accept prefixes such as `1x` or `truefalse`. Typed
array/object readers need to require the exact enclosing delimiter while
retaining the original document and an absolute position. The appended scalar
consumers provide that contract for bool, null and i8/u8/i16/u16/i32/u32.
The grammar must still decide whether a field may be null. Semantic i64/u64
remain quoted codecs and are deliberately absent from this raw scalar list.

Arguments are C24 document, C5 absolute unsigned 32-bit start and C3 eight-bit
ending. Ending 0 requires EOF, 1 comma, 2 array close, and 3 object close.
Other full eight-bit tags reject. The caller retains the ending delimiter;
the returned cursor points at it. Input before the cursor and content after
that delimiter are owned by the enclosing grammar, not validated by a leaf.

Slice checks the original document and range before lexical parsing. The finish
circuit separately checks original length <= 1 MiB, start <= length, addition
carry, end <= length and a nonempty successful token. Thus an underflowed
remaining length or wrapped end cannot make a valid packet. ReadByte may receive
an invalid computed address, but it is total and no such packet can pass finish.
EOF is accepted only when end == length; punctuation requires end < length and
an exact byte match, so inactive storage cannot invent a delimiter.

The bool adapter requires a non-null keyword; the null adapter requires null.
They normalize into the existing C7 layout without changing any integer parser.
Small signed payloads retain width-bit two's complement, zero-extended to the
64-bit slot. Finish returns valid/at-EOF, value and absolute end with zero
padding. Every failure zeros every output bit. The parser Let binds its packet
once and shifts doc/start/ending from Var 2/1/0 to Var 3/2/1 under the binding.

The new targeted test observes all 128 packet bits. It covers every admitted
scalar and delimiter, nonzero cursors, range/type rejection, incorrect delimiters,
lexically valid prefixes with invalid suffixes, all high ending-tag bits,
inactive delimiter storage, full-u32 lengths/starts and values at the 1-MiB edge.
Source replay compares every term and resolved declaration/level against both
the unsigned and signed archives. That preservation check transfers the already
passing lexical matrices; they are not rerun after an append-only change.

Compilation, targeted lint and edited-file formatting passed. Source generation
passed all 68 contexts and three nonempty programs, including complete unsigned
and signed definition preservation. Current pins were independently hashed,
copied and read back byte-for-byte against source output. This avoids repeating
the same source generation after a file copy. Each has 53,232 terms, 735
declarations and 8,744 static transformers, within all practical limits.

All three certificate byte vectors are identical, including an explicit byte
comparison independent of their equal hashes. Checker agreement therefore runs
one representative while per-source metadata/import checks cover all contexts.
All 119 scalar semantic cases passed (source and matrix together 644.47s).
The single distinct-certificate checker case passed with zero axioms and hash
corruption rejection (460.35s). Corrected kind-label source replay passed in
85.05s. All v1/v2 certificate bytes are identical and the complete manifests
differ only in the discriminator key. Scoped review has no remaining findings.
This does not complete ordered field handling, missing/null/value policy,
recursive typed codecs, cumulative limits, universal VC proofs or W09. The
whole-project gate remains deferred to T06-W12; no component-only commit.

## Metadata discriminator correction

Direct review found the short tags were named type_id despite not being registered
MPK type identities (and null denotes syntax). They now use kind with an explicit
comment. The ordinary global names and bodies are unchanged. Only affected source
metadata generation is repeated; exact v1/v2 certificate equality must establish
that the existing semantic/checker executions apply to the corrected metadata.
