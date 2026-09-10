# W09 canonical typed JSON node counts — review in progress

The generator consumes immutable EmittedDataPhase and derives exact projected
input roots from its validated boundary fields. Primitive scalar/source enum
values count as one JSON node, including UTF-16 strings and textual numeric
codecs. Products count their object plus field values. Sums count their object,
explicit tag and active payload;unknown tags return262145. Sequences count their
array plus active elements and reject lengths above capacity. Semantic Map entry
objects and Transition events arrays count because they are JSON nodes, unlike
the corresponding semantic cell count. Unit/exception have no typed JSON form.
Role bounds add no node. Representation domains remain a separate precondition.

Every count saturates at262145, one above the inclusive262144 JSON-node bound.
Arithmetic widens u32 operands to35 bits before add/shift so high values cannot
wrap. Sequence counting reuses the existing65537-saturated sum pipeline twice:
sum floor(child/4) and sum(child mod4), then reconstruct4*quotients+residues.
There are at most16384 residues, each at most3, so their sum is at most49152
and is exact. If the quotient sum saturates, the reconstructed count necessarily
exceeds262144. This adds no second concrete fold pipeline. Invalid lengths are
checked before accepting the resulting count;inactive elements do not contribute.

268 actual-core arithmetic/container cases pass, including exact upper-bound
neighbors, u32::MAX, unknown/inactive sum tags and sequence capacity overflow.
15 original source contexts pass37 independent canonical-tree count comparisons,
full32-bit result observations, validity checks and exact import/mutation tests.
All16 additional compound/direct-root source inputs pass in16.04s. Their tests use
existing independent captures;no full parser runtime or source compiler capture
is repeated. Affected lint passes. The exact new Std consumer path was removed
from the observed path set to verify the previous hash before updating it.

This component is not yet an envelope admission guard, raw JSON counter, source
reconstruction relation or application proof. Whole W09 acceptance and all
remaining implementation-plan units remain open;no component-only commit/push
or T-wide gate is performed.

Review found a test-only allocation defect:the first compound source has
carrier depth35,and the dense storage oracle expands2^35 leaves. After
observing14GB RSS and no completed cases,only that test process was terminated.
The corrected test uses the existing sparse projection-storage oracle with an
explicit carrier-depth equality assertion. No source case,expected node count
or result bit was dropped;production definitions remain unchanged. The
corrected16-case run passed16.04s. Full consumer closure passes27.37s.

All31 original source candidates and one helper are now hash-verified and
pinned under json-typed-nodes/. Final affected lint passes9.37s and scoped
formatting passes. All32 pinned candidates passed identical-byte unchanged dual checking,zero axioms
and hash mutations. The exact PASS filename set and candidate hashes reconcile.
