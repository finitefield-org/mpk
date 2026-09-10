# W09 unit 4 Transition JSON: component review

The compiler recognizes the exact closed Transition template and checks its
state and response argument IDs and events sequence element ID/capacity. It
parses the required fields in state/events/response order using the existing
typed children and structural product constructor. Unsupported child types
leave the parent deferred. No source or frontend rule is changed.

Only the events member uses ChildContainerPayload. Its array still consumes
JSON depth, but its wrapper does not contribute a MonomorphicValue cell.
State, every event and response retain their complete counts. Empty events
contribute zero; source Outcome.Events arrays keep their container count.
Event order and repeated events are retained without the Map/Set ordering
predicate. Any child or syntax failure masks the complete resulting packet.

Map and Transition share one lazily emitted container join in a single Builder.
The scalar-map source fixture reaches both roles and asserts one definition;
the compound fixture reaches Transition without a Map. Existing compound Map/Set
and six-sum complete metadata and certificate replay passed after extraction.
All prior primitive and syntax dependencies are also compared in the new
source tests. No definition or transformer accounting is discarded or reset.

Two frozen original C# captures were accepted. Source generation/import,
excluded-container metadata mutations and structural limits passed. The new
runtime test observes complete logical headers and selected wide storage bits,
including state values, events lengths and occupied slots, response tags,
padding and inactive endpoints. It covers repeated/decreasing events, an
otherwise identical source product's additional cell, empty events, optional
response, exact outer endings, missing/null/misordered fields and depth bounds.
It is not an exhaustive observation of every storage bit or a universal proof.

All24 runtime cases passed (773.01s), and both new certificates passed the
unchanged Go/Rust identical-byte check with matching zero-axiom reports and hash
corruption rejection (766.649s). Direct component review has no actionable
findings. The progress receipt records terminal evidence and selection rationale. Maximum original-document sequence/Validation and earlier Map/Set
runs continue separately; this component does not claim their pending results.
Full boundary relations and W09 units 3-8 remain open, including actual
propositions/proofs and certificate assembly. No component-only commit/push;
the T whole gate remains deferred to T06-W12.
