# W09 byte-document composition review (verification in progress)

Nested typed JSON processing needs to isolate an exact byte interval and compose
encoded documents while enforcing the document's cumulative 1 MiB limit. The
new ordinary definitions operate on the existing private C24 document carrier.
They do not assert that any sliced boundary is a code-point/token boundary,
validate JSON grammar or encode an application VC proof.

SliceValid checks the full u32 document length, start <= length and
count <= length - start. Checking start separately prevents subtraction wrap
from accepting an invalid range. Empty slices at the end are valid, including
at 1 MiB. Slice returns a document of exactly count bytes with source index
start + relative index; the valid-range guard bounds every active addition.
Invalid ranges return the all-zero document rather than a wrapped/truncated
interval. The twenty physical index selectors are expanded to a complete u32
before arithmetic, so each high address bit remains significant.

ConcatValid separately bounds both lengths, checks the complete 32-bit sum
against 1 MiB and rejects carry. Concat reads left while index < left length,
otherwise right at index - left length. Only the selected branch is evaluated;
normal document construction masks output after the summed length and zeros
header padding. Empty inputs, exact capacity and invalid sums retain these
rules. The output is an ordinary address function, with no host byte copying,
smaller scan bound, new recursor, trusted registered result or checker rule.

The source suite reconstructs all 68 contexts and requires exactly three
nonempty programs, metadata/context/byte mutation rejection and exact pins.
Semantic tests cover all 256 byte values, 48 slice ranges and 36 complete short
concatenations with nonzero inactive input storage. A separate suite covers all
20 physical address bits, four exact-capacity concatenations, end slices and
high-u32/carry/range rejection. Full-bound observations use sparse concrete
storage with full lengths and edge-address checks; they do not enumerate every
byte of a 1 MiB output or constitute universal proofs.

Source generation/import/mutations passed 68 contexts and three nonempty
programs in 33.64 seconds. Each pinned certificate contains 8,208 terms,
135 declarations and 81 counted transformers. Exact replay, all three same-byte
checker cases (zero axioms and hash rejection), inventory, lint and format
passed. Both semantic/full-bound subtests passed in 561.49 seconds. The later
shared-document emitter refactor preserved all 68 contexts and three pinned
certificates exactly in 35.49 seconds. The progress receipt identifies each
command and hashed log. Direct review found no remaining component findings.

These results do not establish grammar validity or universal application laws.
Original units 3–8 and W09 acceptance remain open. No component-only commit;
the whole gate stays deferred to T06-W12.
