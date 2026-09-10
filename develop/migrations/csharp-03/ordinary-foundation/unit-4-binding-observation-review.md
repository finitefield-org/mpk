# W09 unit 4 proof-comparison semantics correction

Finding: the first binding-relation component mapped proof-level Binding.Equal
and projected-result agreement to C# structural equality. That is not sufficient
for the retained W06 obligations. Normal commutation explicitly uses
Binding.ObserveEqual. Semantic and identity round trips, and concrete-definition
equivalence, compare produced values as proof subjects; they must preserve
admitted observations. A NaN compared with itself must not falsify an identity
round trip, and substituting +0 for -0 must not pass result preservation.

The design's binding section requires both round trips to preserve every admitted
observation. The normative profile also retains exact floating-point bit behavior.
The names of these proof predicates do not make them source-language equality
operations. The actual C# foundation .equal operation remains IEEE equality and
is unchanged in the scalar/structural generators.

Fix: both Binding.Equal and Binding.ObserveEqual now use ordinary observation
relations. Projected-result agreement uses ObserveEqual(Project(result), expected)
and records its exact W06 comparison symbol. Every demanded public comparison
is lowered at its concrete type, including primitive operation results. Linear
sequence-construction states retain explicit unresolved symbols: they need the
unit-5 ownership/initialization relation, not public value equality. Other
lowering errors are not silently skipped.

Regression evidence: same NaN bits are accepted by proof agreement; different
NaN payloads, signaling/quiet changes and opposite zero signs are rejected;
source-language equality still reports NaN unequal and signed zeros equal in the
independent native-value model. The previous IEEE lowering would fail these
cases. A new captured Make source produces an actual W06 normal-commutation VC;
the test checks its predicate and result type against the ordinary metadata.
It also covers the newly induced Bool/float identity projections and actual
identity reconstruction definitions. 126 ordinary comparisons passed.

The preceding 44 certificate bytes passed both unchanged checkers with zero
axioms and all hash corruptions rejected (693.002 seconds). Those results prove
ordinary typing/linkage, not that the wrong proof-comparison choice was correct.
The bytes, metadata and receipt are retained under binding-relations/
pre-observation-agreement; they are superseded, not current acceptance evidence.

Direct review covers the semantic distinction, demanded symbol/type coverage,
identity and internal-state classification, exact original-source and W06 links,
shared projection/observation dependency closure, metadata/byte mutations and
unchanged source equality/checker rules. Current remaining verification is
recorded in unit-4-binding-observation-progress.json. This is not full unit 4 or
W09 completion. Reconstruction witnesses, native/control semantics, canonical
codecs, transitions and complete proof/certificate assembly remain required.
