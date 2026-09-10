# Ordinary checked contract definedness (partial W09)

Source public-clause generation now accepts the supported integer unary/binary
recipes with checks. Each such clause retains its original value definition and
an additional `definedness_definition`, bound to the same expression, attachment,
source and VIR identities. The latter is a direct lowering of the existing W03
`data_vc::definedness` term. It is an obligation to instantiate at the original
contract use point; its presence does not prove that condition or an application
VC. The failure-side zero result must never be used as a normal contract value.

W03 Boolean constants use the existing ordinary Bool definitions. Integer checks
use the existing scalar failure functions with exact nominal signatures and
ordered-check metadata. For these integer operations, the only possible checks
are overflow and division by zero; divide/remainder overflow requires a divisor
of -1, so those two failure predicates cannot overlap. Their ordered failure
functions therefore also implement the individual W03 predicates. This reasoning
must not be generalized to operations with overlapping failure predicates.
The scalar circuits and their published metadata/bytes are unchanged.

Conditional definedness guards the selected branch. Let definedness checks the
value even if its binding is unused, and retains its lexical scope in the body.
At this checkpoint,index/payload partial conditions and bounded quantifiers
remained unsupported. The subsequent `../partial-read-clauses/` checkpoint adds
index/payload recipes;bounded quantifiers remain unsupported. Partial source clauses are rejected by all three public
consumers (domains, defaults and structural-public). They cannot silently assume
the condition or narrow the existing PublicDomain equation to avoid proving it.
The use-point owner and native/control integration remain outstanding.

The source fixture contains checked add/negate/divide/remainder, safe and unsafe
conditional branches, an unused checked let value, a local divisor, and an outer
divisor referenced beneath a second lexical binding. Its nine clauses are observed
at nine i32 boundary/interior values, with an independent checked-arithmetic
oracle. Results are inspected only when the oracle says the expression is defined.
The source expression hashes, exact import and removal of definedness metadata
are checked. Prior total and structural source/integrated certificates must remain
byte-identical. See `../unit-4-definedness-clauses-progress.json` for actual results.

Only affected tests, lint, formatting and artifact-consumer inventory are selected.
The repository-wide gate remains deferred to T06-W12. This checkpoint does not
complete any original internal work unit, W09, or unblock W10. There is no
component-only commit or push.
