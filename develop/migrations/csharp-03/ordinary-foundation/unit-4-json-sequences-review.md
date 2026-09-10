# Ordinary sequence JSON parser review (historical capacity result consumed)

The compiler now dispatches exact closed bounded_sequence entries and resolves
their concrete element parser recursively. Source arrays use this closed
boundary carrier with capacity 4,096. The original capture rejects jagged
source arrays; the accepted fixture instead uses bool[], int[], and an array
of source structs each containing int[]. The captured types, reconstructed
carriers, complete defined/deferred partition and metadata/import mutations
are checked without modifying frontend rules.

A sequence parser starts with a valid one-cell header and zero storage. Its
finite step reads the stored length, consumes a comma only after a previous
element, and calls the original child parser with either closing-bracket or
comma ending. These endings are mutually exclusive; a successful first packet
is bound once, and the second attempt supplies the alternative. No primitive
parser or token rule is weakened. The child delimiter remains unconsumed until
the next step or the final close. Missing values, repeated/trailing commas,
wrong child types, overflow and suffixes therefore fail the existing exact
syntax/child/header checks.

Each accepted child is appended at the current length, and the length advances
once. Fixed sequence storage preserves the original role layout: length with
leading zero padding, followed by complete element slots. Zero initialization
and single-slot writes leave unused storage zero. All header and value output
is masked on failure. After the fixed capacity, the closing bracket must be
at the cursor; an additional element cannot be silently ignored. The active
predicate permits short-circuiting only on invalid headers, EOF or ']'. Final
syntax and document-ending checks still decide acceptance.

The initial depth check distinguishes an empty array from one with a child by
the exact next byte after '['. Each compound child receives parent depth + 1.
Examples cover an empty array at depth 32, nonempty arrays failing at that
depth, and arrays of structs containing arrays at the adjacent valid/invalid
depths. Field names do not consume value depth; struct field values do.

The concrete pipeline accepts a fixed-depth state predicate and transformer;
it does not quantify over C# types. StepEight contains eight original guarded
steps, and the 4,096-capacity pipeline contains 512 group occurrences. Both
levels use the existing Builder accounting. Definitions are shared by exact
state depth and capacity, with no counter reset or removed dependencies. Tests
check exact reuse, the 16,384-capacity helper and the inclusive cumulative
transformer bound. The new full source program has 8,889 counted transformers.

Checker review found a real shared-helper bug: integer_format::define computed
input widths as usize, then bind_inputs narrowed those widths to u32. C32 and
larger arguments consequently became Bool. The Go checker identified
JsonValues.Packet.D32.Project.D7.R0, even though the runtime evaluator's erased
types allowed the example calculations to pass. The generator now constructs
input Pi/Lam types directly from their cube depths. A regression compares
exact legacy/current bytes at every depth 0..31 and checks the retained input
types at depths 32, 33, 64, 128 and 253. No checker or acceptance rule changed.

The original new vector (137,602 terms) is archived as rejected. The corrected
vector has 137,606 terms and the same 2,092 declarations. Its narrow bool/int
sequence metadata and complete dependency closures are identical, so their
previous runtime observations and the live capacity test are retained. Only
the five affected wide cases were rerun, all passing in 171.57s. The new vector's
same-byte dual-checker result is tracked independently in the progress receipt.
The separately changed existing long[] source vector passed both checkers and
remains byte-identical after the shared-helper correction. Its six sibling
metadata rows and certificate files are unchanged.

An audit of the shared definition helper's callers found fixed small inputs
in scalar/calendar/decimal/token/document functions. Dynamic wide inputs occur
in packet projection, packet assembly and the new sequence functions. They
are covered by the corrected nested/source fixture. Existing scalar matrices
are not rerun: depths below 32 have exact byte preservation, and changed-wide
runtime/checker evidence is recorded separately.

The inventory adds exactly one Std consumer, the new sequence emitter.
Removing that path reconstructs the prior count and path-set hash. The recorded
fingerprint, inventory hash linkage and aggregate family/path count are updated;
the fingerprint mutation test and aggregate inventory test both pass.

No additional actionable source finding remained at this component checkpoint.
Capacity cases 4,095/4,096/4,097 were still pending at that time; their terminal
result is recorded below. The progress receipt distinguishes historical runs
from current-byte evidence.
Map/set ordering and implicit entry cells, sum/Transition roles, absence/null,
whole boundary relations and W09 units 3-8 remain open. No component-only commit
or W09 completion is issued; the whole T gate stays at T06-W12.

The corrected nested-sequence vector subsequently passed the standard same-byte
Go/Rust checker and hash-mutation harness (544.681s; terminal session 43052).
Capacity verification remains separate. The sum extension is now in progress
in `unit-4-json-sums-review.md`; it does not close the outstanding roles above.

On 2026-09-11, the original capacity run passed all three complete documents in
44,333.77 seconds. Complete header/length and selected storage observations
confirmed 4,095/4,096 acceptance and 4,097 rejection. Its process had exited.
The recorded historical/current array hashes still match the exact
six-definition comparison in `unit-4-json-cell-count-capacity-scope.md`, and the
current array's separate dual-checker PASS was revalidated. This closes that
pending numeric capacity observation with the stated bounded-input inference;
it is not current-byte runtime execution or a universal equivalence proof.
Later parser components retain their own receipts. Original units 3-8 and W09
remain open.
