# T06-W09 internal unit 5: native literal results

This checkpoint connects every `literal_values` entry in each emitted control
function to its exact native producing block, value ID and concrete type.
It adds a closed ordinary value and a unary complete-storage equality relation.
The relation does not establish node reachability, an invocation result, source
slot updates, or application correctness. `application_scope_pending` stays true.

The generator reads validated VIR directly. Repeated values share one closed
value definition, while each producing node retains its own relation and source
record. The strict importer reconstructs both the metadata and certificate.
No output is inferred from captured runtime measurements. Complete physical
comparison includes inactive sum storage and padding, rather than only logical
value equality. The original literal encoder still validates every value against
the registered closed specialization context.

The existing exception pass has already emitted shared literal fragments.
A separate private `ControlLiteralPart` scope prevents redefinition when this
new pass appends to the same builder. Existing callers retain `LiteralPart`,
including the independent boundary-literal producer. This scope changes names
only for the new pass; it does not relax duplicate-definition checks or reset
builder costs. Old terms and declarations must remain an exact prefix.

Validation selection: all 18 affected original loop/pattern contexts; exact
source-binding comparison; independent flat storage observations; full-cube
execution with correct values and changed low/middle/last bits; changed, missing
and extra metadata rejection; all retained control prefix/metadata checks;
ordinary and boundary literal consumers; literal helper unit tests; consumer
inventory and targeted lint/format. Each distinct extended certificate requires
same-byte unchanged Go/Rust acceptance, matching reports, zero axioms and hash
corruption rejection before fixture promotion.

All scoped validation passed, including same-byte checking of all 15 distinct
certificates. Final scoped review findings: 0. The initial compilation identified one boundary
literal initializer requiring the preserved default scope; it was corrected
before generating candidates. This is not a checker rejection.

Remaining W09 work includes native source-node execution composition, actual
incoming selection witnesses, alias and exceptional state, handler/filter/
finally semantics, uncovered data/control families, transition/replay and
application proof assembly, and the complete W09 acceptance audit. The whole
T gate remains deferred to T06-W12 under repository instructions.
