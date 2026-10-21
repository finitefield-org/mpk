The [original body reader](../../../../../../crates/mpk-vc/src/go_practical_type_keys/source_scalar_typing/bodies.rs)
checks all supported statements of one original function, including unreachable
syntax. It reuses assignment/return/expression work in a shared flat engine;
local var/type/const declarations, conditions, expression switches, array/slice
ranges, actual break/continue targets and final-return coverage are checked.
ConstSpec checking includes every blank member, inherited values/types and the
actual iota/scope. Local-variable reads distinguish simple assignment targets
from compound/index/field reads; unused locals reject. Duplicate constant cases
use both the actual value and type; repeated nil cases retain Go's behavior.

Readonly GoBodyTyping retains the complete original signature, every original
statement and destination type, condition/case/range use types, branch targets,
constant spec/member values and actual local bindings/types. Invalid syntax or
types retain original source/node error locations. The [Go1.25 terminating-statement
rules](https://raw.githubusercontent.com/golang/go/go1.25.0/doc/go_spec.html)
are syntactic: both if arms, switch defaults/cases and exact break targets matter;
an absent-condition for without a break can end a result-bearing function.
That classification proves no runtime halting or loop invariant. Clause/container
rows additionally retain their statement-list classification.

The [actual native reference](../../../../../../go-tools/go2vir/practical_body_typing_reference_test.go)
saves complete original sources/candidates before pinned Go1.25 gc/amd64
parser/typechecking. All495 observations remain:174 exact positive body/type/
control outcomes,299 agreeing rejections and22 valid forms outside the reader.
Native physical AST positions and actual Go objects agree on1665 signature
slots,1972 statements,442 condition/case/range expressions,113 branch targets,
14 const specs/16 members,57 locals and28 syntactically terminating bodies.
A1001-statement body and70 inputs preserve the unchanged source/helper limits.

All four new Rust roots pass, including owned source/dependencies, independent
callee-body rejection, blank/repeated const values, original unused/unreachable/
missing-return/branch error locations and20000-node contextual work. Sixteen
affected old value/statement roots, two constant roots and four source-type
consumers pass: original4985/950/1659/461/1433/630/5117 cases remain exact. Existing
builtin checker crashes remain separate. Strict Clippy, changed formats and Go
vet pass. Signature/scope/parser/ordinary semantics are unchanged; unrelated
semantic tests are not repeated. Each check records its selection rationale.

The [verification](verification.json) retains four epochs/3305 complete preimages
and17 passed scoped executions. Ten lossless transports preserve32576 original
files/92971348 bytes with a [literal byte/mode audit](original-byte-audit/result.json).
[Review](input-review.json) uses unchanged127 matchers/four scopes/39 seals/51
exclusions and no new exceptions. SDK14496/frontend189/frozen255/ordinary448 basis
remains exact. Value/constant work-loop semantics are unchanged by the reusable
compute refactor; the original comparison helper only gains sibling visibility.

Two of the22 outside-reader positives are known remaining v1 forms: const
len/cap of an array. They remain required, and are not reclassified as a v1
exclusion. The other20 positive controls exercise excluded range/control/
closure/pointer/panic forms. Whole-v1 coverage is not complete. Checking one
body does not validate unused imports or other bodies/packages, close selected
callees, or grant promoted-selector/closed-error/Plain admission. A caller's
header can typecheck while its actual selected callee body remains unsupported.

Actual source evaluation, result/target/call execution order, loop safety/halting,
allocation/alias/permission/NoAccess/lifetime/state/program proofs, full nominal/
module/import/build/version/capture/context/resources/U/Plain/weights/final counts,
full56 foundation/registry and nativeLinux/generalnamespace/selectedclosure/
all169/T02–T11 remain due. Go/Rust only; R01/R06/W01 stay open, the public practical
profile stays inactive and the full T02 gate stays finalW04:
`./scripts/check-fast.sh`.
