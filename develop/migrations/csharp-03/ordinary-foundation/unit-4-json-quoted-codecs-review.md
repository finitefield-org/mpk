# W09 quoted hexadecimal codec review (verification in progress)

The existing binary32, binary64, guid.n and guid.d ordinary codec definitions
are emitted into the owning shared JSON Builder. Their algorithms are unchanged;
the only edit in the standalone codec file exposes its emitter to the sibling
JSON module. The new public metadata type records the exact parse/format codec
pair and its JSON-framed parse definition. Existing source reconstruction and
import comparison bind all of this metadata and all certificate bytes.

Each wrapper binds the complete string frame once, projects decoded UTF-16 and
its header, invokes the existing codec, and checks the closed result's zero
success tag along with frame validity. All three argument binders shift under
that Let; no result is accepted solely because a codec accepts an empty or
otherwise substituted string. All eight ending bits are checked against 0..3.
The C8 result keeps all 128 value bits and the exact absolute end. Smaller float
payloads are zero-extended, preserving signed zero and NaN payload bits rather
than applying host floating-point equality. Invalid results have all 256 bits
zero. The zero result uses a bit vector directly; the u128 literal helper must
not shift beyond 127 when constructing a 256-bit packet.

Preservation compares the prior lexical certificate and the full dependency
closures of all four parse/format codecs from the checked floats and guid
standalone sources. These comparisons include types, bodies, binders and imports;
codec metadata must match too. Existing lexical/codec runtime matrices therefore
do not need repetition. New tests observe all packet bits for values, malformed
and noncanonical strings, widths and delimiters, including high document/cursor
bits and inactive ending bytes. Source-specific combined generation and actual
same-byte dual-checker acceptance remain mandatory for current changed vectors.

The implementation review has not identified an actionable issue after replacing
the over-wide u128 zero literal construction. Verification is still pending.
This supplies quoted codecs, not complete typed grammar, cumulative nested limits,
source/native/transition relations or actual application proofs. The full W09
scope and unit boundaries remain unchanged.


The targeted source tests passed in 151.43s: all 68 JSON contexts, four actual
structural contexts and full dependency preservation for old lexical and reused
hex codec definitions. Largest measured source program: 112,362 terms, 1,706
declarations and 13,412 transformers. Full collection coexistence passed at
13,002 transformers. Current source/pin files were independently domain-hashed
and copied/read back byte-for-byte. Lint, formatting and the two affected
consumer-edge tests passed. The shared token vector passed both checkers in
115.76s and the collection subcase passed in 77.46s; the remaining structural
cases and runtime observations are pending. Current implementation snapshots
and retained completed-log hashes were checked against the receipt. No new
actionable issue was found in this scoped review; pending checks are not passes.

Runtime completion: session 14430 exited 0; all 48 C8 packet cases passed in 1360.69s. The retained log and hash are in the progress receipt. This completes the quoted-codec component verification, not W09.
