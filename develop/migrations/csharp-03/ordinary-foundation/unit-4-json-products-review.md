# Source-product JSON component review

This component implements ordinary source-product parsing within W09 unit 4.
It preserves the eight approved units and does not establish application
propositions, proof terms, or complete boundary grammar.

The generated parser takes document, start, ending and depth. Sequential Let
bindings retain each complete child parse packet. Outer arguments are addressed
after the current binding count; previously bound packets use their reverse
de Bruijn distance. The compiler wraps these bindings in reverse before wrapping
the four arguments. Recursive compound children receive depth plus one. Scalar
children need no depth argument because the parent's Begin requires room for
every direct child. Empty products are permitted at depth 32.

Syntax matching consumes exact braces, stored-member names and colons, commas,
and the final brace. The final outer delimiter is checked but not consumed.
Every step requires a valid prior header and strictly increasing cursor, so an
earlier parse failure cannot be repaired by a later successful token. Child
headers contribute their entire logical cell count to a checked sum; syntax
contributes none. The parent starts with one cell. Begin and Finish bind the
cursor to the document, and Finish validates EOF consistency and ending tags.
The helpers' header/cursor inputs are not proof of provenance on their own;
the actual product parser supplies them through these ordinary definitions.

Source member IDs select the structural constructor fields, while source names
select JSON lexemes. The initial implementation conflated these and failed
source generation. The corrected metadata records both and tests names against
captured facts and IDs against independently reconstructed carriers. Raw facts
do not contain member IDs; an additional test incorrectly assumed they did,
and was corrected after its deterministic failure.

Actual source MakeStorage preserves the original field layout. Product-carrier
padding occurs before each child selector; the outer parser packet instead
preserves the child carrier's low selectors and pads afterward. These layouts
must not be interchanged. The complete role packet is zeroed on invalid input,
including unused value/header storage. Ten actual-core calendar/decimal product
cases passed every packet bit, including nonzero cursor, failed field order,
missing/duplicate members, depth 31/32 and nonzero decimal scale.

Nested/empty fixtures are compiled from original C# with the frozen local
toolchain, a read-only repository mount, disabled network and deterministic
double capture. The first launch omitted Docker stdin attachment and produced
no result. After attachment, the initial nested source correctly failed the
existing dead-declaration rule for unused constructors. Calling both from the
root produced two accepted captures. The source replay passed (two complete programs, three source products).
All 13 recursive/empty runtime cases passed (terminal session 31246,
exit 0, 45.75s), checking
root depth 30/31, scalar -2, three total cells, empty depth 32/33, exact syntax,
truncation, duplicate nested members and prefix/outer comma.

Review inspected recursive child selection, cycle rejection, active/deferred
sets, field linkage, Let scopes, source storage assembly, packet projections,
inclusive limits, import regeneration and cumulative builder accounting.
The source tests preserve complete previous primitive and syntax declaration
closures. Seven complete certificate vectors are distinct by full bytes; their
dual-checker run passed; terminal evidence is retained in the progress receipt. Nested/empty replay, exact pin retention and import/mutation checks passed;
all 13 actual-core packet cases also passed. Both nested/empty vectors also passed unchanged dual checking (terminal18559,
exit0,604.161s), zero-axiom/report checks and hash-corruption rejection. The
seven earlier source-product vectors also passed both unchanged checkers
(terminal37186, exit0,3072.577s). All nine source-product vectors are now
accepted with zero axioms and hash-corruption rejection. No passing checker or semantic
result is inferred from this source review.

Verification is limited to changed grammar, its source consumers, new certificate
bytes, affected lint/format and the two consumer inventory checks. Unchanged
scalar/token runtime matrices are not repeated. The whole T gate stays at
T06-W12. Collections, sums, enums/errors, field absence/null rules and remaining
units 3-8 remain open; no component-only commit is made.
