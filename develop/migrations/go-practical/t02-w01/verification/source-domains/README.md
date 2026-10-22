# Source-derived error entry domains and static value properties

This Go/Rust-only T02-W01 checkpoint starts from parent
`15e087a6a51c7307355e5750c2a28c6ddac2ba4a`. R01/R06/W01 remain open.
The public practical profile remains inactive.

The [domain constructor](../../../../../../crates/mpk-vc/src/go_practical_type_keys/source_scalar_typing/domains.rs)
owns an existing independently derived source selection. Its only new input is
an explicit finite entry predicate: an original selected function, receiver-first
input ordinal, original field ordinal/element steps and original type-declaration
references for allowed cases. These are user constraints, not supplied compiler
type graphs or expected proof results. Every source error leaf needs exactly one
row, including zero-length arrays and slice backing cells. An explicit empty
case list permits only nil at that entry path; an absent list rejects. Extra,
repeated, misplaced and alias-equivalent duplicate cases reject.

One global case union combines every selected body boxing with all entry cases.
Aliases resolve to original nominal identities. Cases require named Bool/word/
struct payloads and exact value `Error() string` compatibility. Promoted Error
methods retain the current source-selection restriction. The constructor records
original signature dependencies and rejects their original function Doc compiler
directives without selecting or evaluating the method bodies.

An explicit graph walk expands each source error leaf to every global payload.
It checks cycles even through zero arrays and inactive cases. It computes Plain,
aggregate depth and per-value static weight over original types, with nominal
wrappers adding neither depth nor weight. Every slice's element must be Plain
under the same global case union, including a locally nil-only slice. Existing
array256/struct64/depth16/value2097152 limits remain; strings weigh257, slice
values37, arrays1+n*element and struct/error values1+sum. There is no new helper,
root, argument or error-case count cap. Binding emits original owner type IDs
and distinguishes array length from slice capacity.

These checks establish static properties for the currently admitted source
subset. The type-package envelope still lacks authoritative application/stdlib
role information. Complete capture/module/compiler/context authority and raw
source-overlay import/binding remain required. This API therefore does not
establish full application-case eligibility or public contract authority. Finite
Error dispatch, dynamic entry obligations, backing-store and joint frame/result
accounting, call/store/loop/state/runtime/program proofs remain due. The existing
body-only getter remains explicitly incomplete without entry predicates.

## Verification scope

The [verification record](verification.json) retains original prelaunch inputs,
drivers, raw stdout/stderr and the reason for selecting each target. Actual Go
compiler observations are comparison evidence only; the independent constructor
receives only original source buffers and a Boolean type seed. Original native
outputs are copied literally with modes and audited against their producer files.
Failed executions retain their original inputs and output.

New Rust tests cover original compiler property comparisons, complete entry
paths, aliases, imported body cases, signature-only bodies, directives, payload
exclusions, global slice Plain checks, expanded cycles, zero-array reachability,
depth/weight limits, receiver order, ownership and 70 explicit case tags. Existing
selection and error-boxing consumers cover the only modified shared surface.
Targeted Clippy, Rust/Go formatting and Go vet accompany those tests. Unchanged
parsers/scopes/constants/ordinary semantics and unrelated languages need no run.
Only a new Go test changes; production frontend inputs remain byte-identical to
the preceding two original empty-cache build observations.

Full56 foundation/private registry, all169 requirements, native Linux and
T02–T11 remain due. The cumulative T02 full gate stays final W04:
`./scripts/check-fast.sh`. No intermediate full/equivalent gate is run.
