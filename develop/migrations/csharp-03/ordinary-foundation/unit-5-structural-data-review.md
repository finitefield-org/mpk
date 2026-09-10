# W09 structural equality data connection review

The new adapter reconstructs W03 data definitions and use points from validated
VIR. Signature tags, matching operand types, Boolean/i32 result type, and empty
failure tables are checked before connecting the exact existing semantic
relation body. Structural equality retains the type-specific implementation;
the adapter compares its Boolean result with the actual SSA result. Canonical
comparison delegates to the existing total-only Compare helper and compares all
32 result bits through the already tested shared result comparator.

Original W03 success guard, relation and goal are lowered with the shared
subject-index to de Bruijn conversion. Metadata preserves source/function/node,
operand/result SSA identities and successor links. All non-structural data
families remain explicitly pending. Native != still consists of structural
equality and a separate pending Boolean negation; the equality body is not
silently inverted. No input domain or complete native-body proof is implied.

Five native inputs contain seven use points over i32, i64, Boolean and declared
enum operands. Ninety-eight actual-result cases pass, including high i64 bits
and both truth values of the claimed result. Complete legacy definition closures
match; source-function/successor and byte mutations reject. All five runtime
outputs equal candidate/checker pins. Both unchanged checkers accept all five
same-byte certificates with zero axioms and matching reports/hashes; corruptions
reject. Lint and inventory pass. Shared-helper extraction preserves all three
integer data pins, whose 80 runtime cases have separately completed.

No actionable findings remain in the native StructuralEqual connection scope.
The corpus contains no fresh native CanonicalCompare use point; that limitation
is retained in the progress receipt. This is not a review of complete native
bodies or original W09 units. Units 3-8 and the full acceptance work remain open.
