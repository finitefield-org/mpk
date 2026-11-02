# Original finite Error invocation selection

This Go/Rust-only T02-W01 checkpoint starts at
`34b79b08aa7b8b2e3d0029279aa2d241f8593de1`. R01/R06/W01 remain open and the
public practical profile stays inactive.

The [independent constructor](../../../../../../crates/mpk-vc/src/go_practical_type_keys/source_scalar_typing/selection/error_dispatch.rs)
accepts original selected roots and complete explicit entry predicates. It types
the actual predeclared error's invocation header without inventing a FuncDecl.
It records the original call and one receiver operand, including bound calls,
method expressions, aliases and parentheses. The previous static-only selection
entrypoint still rejects open interface calls; callers use the complete entry
constructor to obtain finite source candidates.

A single worklist selects every Error method in the union of original body
boxings and all entry cases. Invoked target bodies can add boxings, static
callees and more error calls. Each new call/case pair adds its original target
and all existing type/constant/signature/body dependencies; processing continues
until no pair remains. The selection retains lightweight checked body type graphs
and reuses its original method index; each graph still attaches only its own
dependencies. Global targets apply even to a locally nil-only entry or
an uncertified false branch. Entry-only type/alias declarations also go through
the existing original dependency reader. Original directives, excluded effects,
missing bodies and call cycles retain root-to-target diagnostics. Boxing alone
continues to require signatures without selecting Error bodies.

The existing source-domain checks apply to each growing global union. The
returned candidate owns its original source graph, complete finite dispatch
rows, original receiver spans and static value domain. An empty case union keeps
an invocation row with its non-nil obligation; acceptance of this static candidate
does not prove that a nil receiver can run. Receiver-once evaluation, non-nil
safety, target preconditions/results/effects and all dynamic call/store/loop/
frame/state/program obligations still require independent lowering and proof.
Application/stdlib role capture and complete raw overlay/context authority also
remain due. No caller native type graph, target list, message oracle or branch
certificate is accepted as an independent constructor input.

The [Go receiver walk](../../../../../../go-tools/go2vir/practical_closure.go)
now visits bound receiver expressions as evaluated operands while retaining
the method name as a signature reference. This covers nested receiver calls,
parenthesized selectors and excluded state in a receiver producer. Method
expression receivers are already evaluated through their actual argument.
Native compiler metadata remains untrusted source preparation.

## Local verification

[Terminal check records](verification.json) retain full original prelaunch
inputs/drivers, raw outputs and target-selection rationale. New original Go
comparisons cover23 cases,70 explicit entry tags and70 successive global-union
growth rounds, imported calls, branch conservatism, multiple roots, boxing-only
bodies, receiver operands and target effects/cycles. Existing method/header,
body, selection, domain and boxing consumers are the affected Rust targets.
Original old method vectors are retained verbatim; exactly direct builtin-error
invocation headers move from the old outside subset into exact native comparison.
Pointer/promoted/method-value exclusions remain. Scoped Clippy/formats/vet and
fresh independent empty-cache frontend builds accompany the semantic checks.

Original native and Rust outputs remain literally preserved, including failures.
An actual process sample identified redundant method indexing and body rechecks.
That partial first suite was deliberately interrupted for the implementation
repair; it is not reported as a full baseline pass. The complete case suite runs
again after the repair, without changing the original fixture or expected facts.
Frozen design, SDK and ordinary bases are checked without unrelated semantic
runs. No workflow, agent or other-language tests run. Full56 foundation/private
registry, all169 requirements, native Linux and T02–T11 remain due. The cumulative
T02 gate remains final W04: `./scripts/check-fast.sh`; no intermediate full or
equivalent gate is run.
