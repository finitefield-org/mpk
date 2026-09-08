# W09 internal unit 2: consolidated scalar review

Status: complete for internal unit 2 in this reviewed commit; final findings: 0.
The uninterrupted ordinal full-capacity case passed. This document does not
complete W09.

The approved unit owns fixed-width scalar operations and their operation checks,
not the complete input domains, every reachable structural foundation instance,
source body/control relations, bindings/codecs, application propositions or
proof assembly. Those remain explicitly owned by W09 internal units 3-8.

## Reconciliation

The current scalar, business, numeric and string registries were compared with
the ordinary emitters and their independent oracle/core tests. The complete
canonical inventory contains 505 fixed nonliteral operation IDs, with restricted
string interpolation recorded separately as a parameterized family. Counts,
source/test functions and verification receipt references are recorded in
`unit-2-coverage-audit.json`. Counts alone are not evidence of semantics.

Integer arithmetic covers 160 promoted-width/mode forms and 162 conversions,
including signed division errors, masked shift counts, checked overflow and
mixed-width signedness. The audit added actual certificate generation for all
322 forms because seven pinned examples could not establish this coverage.
It also added exhaustive emitted-core observations for all six Boolean
operations, since the original representative core test covered only XOR.

Temporal/calendar tests cover explicit operands, precision-before-range error
order, checked signed arithmetic, the Gregorian cycle and unsigned GUID N-field
ordering. Floating and decimal tests cover exact rounding, subnormals, NaN
payload/priority, signed zero, decimal scale/coefficient representation and
ordered failures. The corresponding emission tests measure actual concrete
certificates. Decimal's 45 definitions are additionally tested together with
shared finite helpers. Representative actual-core tests supplement the complete
network oracle families; neither form of observation proves an application VC.

UTF-16 tests cover basic operations, every concat form, restricted interpolation,
substring, ordinal comparison/search and both nullable/nonnullable carriers.
The audit reopened ordinal folding: the previous predicate/value reducers were
accepted by the kernels but violated the frozen concrete S->S composition
contract. The correction uses one balanced ordered pipeline of 8192 concrete
steps and preserves the frozen before-sharing transformer count. Its final
verification and direct state/order cases are recorded separately in
`unit-2-string-ordinal-review.md` and its linked receipt.

## Construction and integration review

The shared circuit emitter retains only finite Bool circuitry and ordinary
concrete transformer composition. No result from an oracle, compiler runtime
or source execution is installed as a trusted definition or proof. Generated
names resolve to prior ordinary declarations; imports independently reconstruct
operation signatures, source/foundation metadata and exact certificate bytes.
The added builder entry point fixes concrete value arguments around S->S
composition; it introduces no type/template arguments. Counting still occurs
before DAG sharing, and the predecessor constant-based composition path keeps
its term insertion order. The seven predecessor integer fixtures and the
sixteen basic/construction source captures were replayed without byte changes.

The review checked differing f/g transformers, de Bruijn indices through the
composition Let, first/second element priority, terminal-state identity, odd
length masking, full index bounds and empty-needle bypass. It also checked
that the new serialized fixtures do not introduce a public route, workflow,
checker rule, axiom, theory primitive or change to Certificate v0.

All three audit findings are resolved. The ordinal construction correction
passed matrix, address, Any-fold, source/fixture replay, both checkers, direct
order/state checks and uninterrupted full-capacity evaluation. The scoped
four-test core command completed in 2349.95 seconds. The final direct review
has zero findings. Internal unit 2 is complete; W09 units 3-8 and the complete
W09 acceptance gate remain outstanding. The complete T06 gate stays deferred
to W12.
