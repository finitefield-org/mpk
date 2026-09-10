# Builtin JSON values review

Existing boundary encoding and value import represent day-of-week as a quoted
integer in 0..6, and parse-error as one of five exact strings. The ordinary
definitions retain those encodings and the existing scalar carrier layouts.
They are not general JSON string normalization or a relaxation of the domains.

Day-of-week reuses the existing signed64 quoted parser and enum membership
converter, restricted to 0..6 before retaining its signed32 carrier. Source
enum selection still reads its own validated underlying type and declared set.
The previous complete eight-underlying source-enum/product certificate and
metadata remain byte-identical after the shared selection change.

Parse-error matches the exact quoted spellings input_bound, syntax,
noncanonical, scale_precision and range. Their ordinal order agrees with the
existing literal encoder and domain. All matches use the same actual document
and absolute start. The selector requires exactly one valid match and emits
its original EOF/end, one cell and its full 32-bit ordinal in a C8 role packet.
Every other bit is zero. No matches or multiple matches produce all-zero output;
all 32 match-mask combinations passed complete-packet evaluation.

The final parser binds the selected packet in one Let. Under it, Var0 is the
packet, Var1 the ending, Var2 the original start and Var3 the original document.
The existing header/value projections feed Finish(document,header,ending) and
Assemble(final_header,value). Finish independently binds the header's end/EOF
to the document and checks the requested unconsumed delimiter. Assemble masks
the entire packet when the header fails. The five lexical results cannot
bypass a failed outer delimiter or revive an invalid match.

One original C# fixture holds a System.DayOfWeek field. A reconstructed type
contract contains a constant parse_error_kind predicate, making parse-error
reachable even though it is not a boundary parameter. The frozen offline
compiler double capture accepted the source and all three sidecars. Source
replay verifies both builtin carriers, all vocabulary names/literal bytes,
complete old primitive/syntax closures and metadata import/mutation behavior.
The complete certificate has 141,862 terms and 2,046 declarations; actual costs,
hash and retained logs are in the progress receipt.

Direct review checked exact names/ordinals, signedness, complete storage,
exactly-one selection, Let scopes, delimiter handling, shared syntax chunk
reuse, scalar child integration, explicit carrier partition and import
regeneration. Selector/source/preservation/inventory/lint/format checks passed.
All 39 complete actual-core builtin/product cases passed (terminal 5538,
exit 0,112.10s), including strict spelling, full output/padding, prefix/endings
and source product depth boundaries. The new vector also passed both unchanged checkers, report/zero-axiom checks
and hash-corruption rejection (terminal 37888, exit 0, 269.924s). Retained
terminal evidence is in the progress receipt. No remaining actionable finding
was identified in this component; the remaining W09 scope is unchanged.

The prior nine product programs contain neither new builtin in their previously
verified complete carrier partition. Both new branches return before emission
when their carrier is absent, and empty vocabulary metadata is omitted. Their
unaffected runtime/checker matrices are not repeated. The affected shared source
enum branch is checked by exact regeneration of its complete original-source
program. The selection evidence is retained separately.

Semantic products, collections/sums, field absence/null, full boundary relations
and all outstanding W09 units 3-8 remain open. These definition certificates do
not prove application propositions. No component-only commit is made, and the
full T gate remains at T06-W12.
