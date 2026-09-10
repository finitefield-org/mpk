# W09 canonical typed JSON depth — partial boundary implementation

The new ordinary generator reconstructs input semantic types from immutable
emitted boundaries and binding projections. Its predicate takes the semantic
carrier and a C5 root depth, and checks the depth32 bound on active JSON nodes.
It is conditional on a valid semantic representation: this is not a domain
predicate, raw-node counter, source invariant or application proof.

Primitive values and source enums encode as JSON atoms even when their storage
is a product or sequence (notably decimal and UTF-16 strings). Source products,
Money, ordered entries and transition roles descend through their JSON members.
Map entry objects and transition event arrays retain their actual JSON levels.
Tagged sums always include a child tag string, including none/missing/null;
only the active payload is traversed. Unknown tags reject. Bounded sequences
check their actual length and traverse only its active prefix with the existing
counted aggregate fold. Empty arrays and empty objects may occur at depth32.
The root-depth increment saturates at33, so u32 overflow cannot wrap to zero.
Unit/exception values reject because the frozen typed JSON encoder rejects them.

Direct review compared product/sum/sequence selectors with the established
carrier layout and compared JSON levels with `boundary_input::typed_json` and
`value_limits`. Role-bound wrappers affect representation validity, not JSON
depth, and therefore do not add a JSON node. The public import regenerates
metadata and exact certificate bytes from the supplied emitted source.

The helper test passed252 ordinary-core observations, including depth32/33 and
u32::MAX, tag-only versus deeper payload arms, invalid tags, length8/9 and
inactive invalid storage. Fifteen original attachment sources passed167 depth
observations. Their oracle traverses the independent input capture's canonical
typed JSON, including explicit defaults and nullable/presence wrappers.
Metadata/certificate mutations reject. Affected library/integration lint passed.
All16 hash-checked candidates (15 sources and the helper) passed unchanged
Rust/Go same-byte checking, zero-axiom comparison and hash mutations in72.903s.
The exact set of successful subtests matches all16 published files. All15 source
metadata/byte pins and the affected consumer inventory regression also passed.

The new depth-guarded envelope variant now applies this generator to every
decoded/defaulted semantic field at root depth1. Its combined validation is
tracked in `unit-4-json-depth-guard-progress.json` and is not complete yet.
Additional compound/map/set/Money/transition source coverage, raw/canonical
node counts, source conditions, reconstruction, output and the remaining W09
native/control/replay/proof/assembly acceptance are still required. No unit4 or
W09 completion claim, component-only commit/push or T-wide gate is made here.
