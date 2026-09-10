# W09 quoted semantic scalar review (verification in progress)

The shared JSON environment now lowers char, i64, u64, duration and instant
values using the exact existing canonical string frame. A successful frame is
necessary for every result and independently checks both quotes, byte validity,
original document bounds and the absolute cursor. All value wrappers reject the
colon ending; every ending bit participates. The result keeps the established
C7 valid/whole/u64-value/absolute-end layout and is entirely zero on failure.

A char accepts exactly one decoded UTF-16 unit. It reads all sixteen bits of
that unit from the ordinary string value, allowing lone surrogate code units
and rejecting astral Unicode scalars represented by two units. It does not
confuse UTF-8 byte count with UTF-16 unit count.

For numeric codecs, the interior starts at start+1 and has end-start-2 bytes.
Those arithmetic expressions are only used in an accepted result when complete
frame validity has established start < end <= document length <= 1 MiB and at
least the two quotes. Slice independently fails closed on invalid bounds. The
existing unsigned parser and the width-64 specialization of the existing signed
parser must consume this entire interior. This retains overflow, negative-zero,
leading-zero, optional-minus and ASCII syntax checks. No full aggregate fold is
added: strict JSON canonical spelling already disallows escaped ASCII digits,
so a successful decimal codec is checked directly against the exact raw interior.
Duration ticks and Unix milliseconds use the frozen signed-64 codec as confirmed
in csharp_practical_codecs.rs. No runtime oracle participates in production.

The existing signed width list and helper namespace are explicit parameters;
the old call passes exactly [8,16,32] and the empty namespace prefix. New 64-bit
helpers have a separate prefix and are appended after every existing lexical
and framed-string declaration. The raw admitted scalar list remains unchanged.
All old transitive declaration closures, including the immediately preceding
framed-string checkpoint, must match before earlier semantic evidence is reused.

Targeted cases cover exact signed and unsigned limits, overflow, missing quotes,
partial numeric consumption, redundant signs/zeros, non-ASCII and escaped digits,
all legal value endings and colon/high-tag rejection, surrogate char cases,
inactive delimiter bytes and high original cursor/document bits. All 128 packet
bits are observed. Source/import/metadata mutations, cumulative limits and
same-byte dual-checker results are recorded in the progress receipt. Verification
is still pending for quoted runtime and the subsequent source-specific
structural cases. The 68-context lexical source test, old dependency preservation,
import/mutations, targeted lint, formatting, and both affected consumer tests
passed. Full collection coexistence generated at 12,979 transformers. The two
distinct token/shared certificates passed both checkers with zero axioms and
hash-corruption rejection (63.44s and 63.80s). No actionable implementation
finding was identified in this scoped review; pending checks are not counted as
passed. No complete W09 or internal-unit claim is made.

Full object/array/sum grammar, field ordering/uniqueness/presence rules, other
quoted codecs and nested cumulative limits, native/source/transition relations,
and actual application proposition/proof/certificate assembly remain required.


The final quoted-scalar runtime/source process exited successfully: all 70 runtime
cases, 68 JSON source contexts and four structural/boundary cases passed in
524.57s. Those intermediate structural byte vectors were then superseded by the
quoted hexadecimal codec extension before dual checking. They remain archived
under previous-quoted-codecs with no checker-acceptance claim. Current final
structural byte vectors containing both extensions will be checked; the updated
source tests preserve all earlier quoted-scalar dependencies and semantics.
