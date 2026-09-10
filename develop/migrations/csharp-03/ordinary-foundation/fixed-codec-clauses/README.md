# Fixed-width codec contract adapters (W09 internal unit 4)

Six source contexts exercise binary32, binary64, GUID N/D, Date and Time through
codec_parse and codec_format. A seventh combines all six codecs in one method
contract to exercise family caching and codec-ID selection. All preserve the
original Quantifiers C# source and callable identity. Fresh frontend capture
accepted all seven sidecar contexts.

The compiler delegates to the unchanged ordinary hex and calendar codec bodies.
The shared W03 format condition checks the complete generated length against
16384; input domains and application VC proofs remain separate. Source clauses
observe successful parse(format(value)) tags and erroneous text tags. Direct
calls compare every parser result bit and selected formatter output positions,
including every active UTF16 bit, header words/padding, and inactive edge positions.
Direct composition checks every Result payload bit, including NaN payloads and
signed zero. The mixed case checks all 12 nominal aliases and their complete
standalone definition closures, without repeating the six runtime suites.

Both source tests passed: 44 runtime contract conditions, 56 direct parse cases
and 21,088 format/composition bit observations; the mixed context checked all
12 aliases and complete dependency closures. All seven pinned candidates passed
both unchanged checkers on identical bytes, with zero axioms, agreeing reports
and actual hash-corruption rejection. Current bytes, metadata and terminal
evidence were reconciled. Decimal contract adapters remain outside this
component, and original W09 units 3-8 are incomplete. The full check-fast.sh gate
is deferred to T06-W12. Component completion does not prove an application VC.
