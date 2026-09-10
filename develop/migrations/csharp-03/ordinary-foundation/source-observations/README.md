# Ordinary source observations (W09 unit 3, in progress)

Source binding round trips preserve admitted observations, including every source
stored field. IEEE value equality alone cannot express this: NaN must be
observationally equal to the same bits, NaN payload/sign differences remain
observable, and signed floating zero remains distinct. Decimal representation is
not observable: equivalent coefficient/scale cohorts and signed decimal zero
compare by numeric value. This is the rule used by the existing source binding
reference models; it is not a new application equality policy.

The generator reconstructs every reachable storable carrier from validated VIR.
It recursively emits product/active-sum/sequence/entry/map/outcome/exception
observation definitions. Source products include all stored fields, including
fields a representation binding may consider inactive. Semantic sums compare
the active payload and tag; sequences compare lengths and all active elements.
The operand domains are caller preconditions. Erased sequence-construction
state is listed explicitly and remains subject to source-state obligations.

Observation emission reuses the ordinary relation machinery with IEEE leaves
compared by their raw bits. All existing consumers retain semantic equality.
Separate cache keys and generated namespaces prevent a warmed semantic-equality
entry from being used for source observation or vice versa. Scalar/decimal and
bounded-iteration definitions are shared within one Builder. No checker rule,
trusted primitive or certificate format changes.

Four actual-source candidates cover Money, exceptions, a Bool-key/float-value map
and the added complete source snapshot. The added source has float, double,
decimal, nullable float, float array, tag and other stored fields. Its original
requests and deterministic Linux capture are in `../observation-sources/`.
Generation and metadata/certificate mutations passed. The initial semantic test
used TaggedSum for an Option carrier and correctly failed input validation; it
was corrected to the frozen Option representation without changing the values
or expected observations. All 23 resulting snapshot/scalar pairs and ten IEEE
semantic-equality contrasts passed ordinary-term evaluation in 380.03 seconds.

The four observation pins contain 30 concrete carrier occurrences. Maximum costs
are 46,558 terms, 500 declarations and 8,624 static transformers. Their bytes are
unchanged after introducing separate semantic/observation cache keys. Exact pinned replay and all four same-byte dual-checker cases pass, with zero
axioms and hash-corruption rejection (430.96 seconds). The extended 43-pin integrated
checker run also passed with zero axioms and hash-corruption rejection (1,232.277 seconds).

The same observation definitions are now included in the 43-source integrated
unit-3 program. All 116 standalone source-component pairs, 1,160 roots and 11,207
transitive declarations match the integrated definitions. Root/dependency body
mutations reject, and exact observation metadata comparison detects selection
of the wrong cached equality definition. The full integrated checker rerun passed. Previous 42-source integrated pins and their receipt are preserved in
`../structural-foundations/pre-observations/`.

These are definitions and finite observations, not universal binding or source
reconstruction proofs. Bindings/codecs, native/control and transition relations,
application proof assembly, final W09 acceptance and full unit review remain
required. See `../unit-3-observation-progress.json`. The T06 full gate remains at
W12; no W09 or unit-3 completion is claimed.
