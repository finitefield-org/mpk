# Ordinary program, source binding and execution budget design

This is the final W04 interface design. The [catalogue](interfaces.json) closes
the nineteen theorem families, complete ordinary context, source binding,
materialized VC/skeleton, and policy/AI target rules. The [audit](audit.json)
checks the existing two Go models and all 56 concrete typed operation fragments.
It generates no ordinary program proposition or source proof: all 169 requests
remain unelaborated. T02–T06 own their actual definitions, proofs and applications.

## Source and statement authority

`Source(f)` is a typed captured-source tree independently reconstructed from
the immutable original package, compiler type/selection facts and the exact
function scope. Its closed node/type/call/evaluation rules are the existing
[source catalogue](../source-admission/README.md),
[evaluation rules](../operation-loop-decisions.md),
[operation signatures](../operation-signatures/README.md),
[typed body rules](../vir-body/README.md) and
[contract rules](../contract-wire/README.md). `IR(f)` is independently imported
from the complete VIR body. Neither is reconstructed by treating the other as
its meaning. Constants, receivers, named types, bindings, scopes, source loop
identities, error alternatives, versions and finite selected callee closure
are computed from those inputs. All preconditions and postconditions use the
same independently normalized contract. Hashes select bytes; they are not proofs
that a compiler transformation is correct.

`e` is the full concrete entry observation, including input values and original
storage. `s,h,k,p` are typed semantic states, headers or finite execution prefixes;
`o` is a completion with its result product, state and control tag. Each function
has its concrete argument/product/state carriers, not an untyped universal value.
`EntryRep` couples full source and modeled values, nominal types, nil/empty tags,
typed owners, offset/length/capacity, initialized backing tails and all sharing.
The [storage](../storage-views/README.md) and
[lifetime](../storage-lifetimes/README.md) rules determine live roots and permissions.
Representation grants no separation or write permission. `AdmittedEntry` is the
computed profile-domain predicate; it cannot contain a caller oracle for fit,
no-wrap, source well-formedness or correct lowering.

`SourceWF` and `IRWF` are closed computed well-formedness propositions. The
source-binding theorem must prove them and `Correspond`. Correspondence is
two-way finite-prefix simulation at source evaluation boundaries with allowed
finite administrative IR steps; it preserves full values, storage, effects,
faults and normal/return/break/continue completion. Administrative steps cannot
diverge. The source and IR small-step definitions remain distinct. `Reach`,
`Run` and `ReachSite` derive from those definitions, not from a supplied trace.
`Run` is unbounded finite execution to completion; `Reach` includes every finite
prefix. Infinite execution satisfies partial correctness only if every prefix
is safe. Explicit totality requires actual normal termination. Proof search
fuel is outside this semantic definition.

The nineteen [signature rows](interfaces.json) quantify over those concrete
carriers. `Pre` is admitted entry plus ordered requires; `NormalPost` requires
every normal return's complete ensures/result relation. `SafePrefixes` covers
runtime faults, modeled-domain bounds and frame permissions on all reachable
paths. `Frame` forbids borrowed-input writes even if later restored. `Member`
and `Access` are precisely the operation table's derived concrete predicates.
They cannot be arbitrary proposition parameters. Calls use the single evaluated
actual-argument product and the same invocation's result and output state.
The complete callee dependency DAG must already be checked; recursive calls and
theorem cycles remain rejected.

`ClausePhase` is the exact normalized clause namespace and evaluation guard.
For requires, only admitted representation and already proved definedness/truth
of earlier requires are available, never the clause's own truth. For ensures,
entry requires and the proved normal-return path are available. For invariants
and variants, initialization and preservation proofs establish the header facts;
short-circuit definedness follows the expression's real guard. A candidate
invariant, even `true`, still needs all initialization/preservation/exit members.
Continuing loop edges include the post step on continue; false-test, break and
return exits remain separate. Nat ranks decrease only on continuing edges.
The ordinary strong-induction/termination probes establish representative
feasibility; they do not prove a source loop total.

## Closed roots and acyclic materialization

`mpk.program_certificate.ordinary_context.v3` uses the exact fields in the
catalogue and `MPK-PROGRAM-ASSEMBLY-3.0`. All non-own-hash fields form its narrowed
canonical JSON preimage, without LF. Its context bytes are canonical
[Certificate v0](../../../../specs/CERT_V0.md), lowercase hex chunks of at most
65,536 characters, at most 2,097,152 decoded bytes. Imports bind exact independently
checked foundation exports. The context contains the concrete source, IR,
contract, entry and completion definitions, type terms and their dependency
closure. It has no source manifest, axiom, theory primitive or new trusted rule.
The full context tables and declarations must be an unchanged prefix of the
final certificate; candidate proof terms and theorem declarations append.

Definitions have exact `function_decl_id,role,name,declaration_id` fields. Roles
are source, ir, contract, entry_relation, completion_relation. There is exactly
one row for each role of every selected or transitively selected function, in
function/role catalogue order. Required theorem rows are exactly
`request_hash,name,type_term_id,declaration_hash`, in dependency-topological
callee order and original request order within a function. Names are
`GoPractical.Request.r` followed by the full lowercase request hash. IDs index
the bound context; duplicates, foreign requests, missing members or reordering
reject. Canonical certificate import recomputes tables and definitions; checking
the metadata or its digest is insufficient.

The existing declaration-interface hash encodes name, declaration kind and
type **term index**. It does not hash the term table. Therefore the statement
identity is the whole assembly hash plus exact request/name/type binding, with
the skeleton VC hash binding that assembly. Never compare the old declaration
hash alone and call the proposition equal. Proof import independently regenerates
the whole assembly, checks exact context-prefix preservation and theorem types,
then runs both checkers on the same final bytes and embedded common manifest.

`mpk.go.semantic_binding.v1` uses the exact root/function fields in the catalogue
and `MPK-GO-SEMANTIC-BINDING-1.0`, over all fields except binding_hash. Function
definition references are the corresponding concrete context declaration IDs;
request lists contain all and only that function's generated demands. Binding
metadata states which theorem is required; it cannot establish it is proved.
The materialized VC expansion is exactly `kind,assembly,source_binding` with
kind `materialized`. It is available only after every demand has a generated
ordinary type and every required strategy candidate has been resolved. Otherwise
the current exact `{"kind":"unavailable"}` arm and unelaborated IDs remain.
Proofs may still be pending after types are materialized.

Order: source/typed scope/VIR/contract -> obligation program -> ordinary assembly
-> source binding -> VC -> skeleton -> certificate manifest -> checked final
certificate -> policy/API/AI. Assembly and binding never contain their descendant
VC/skeleton/manifest/certificate/receipt hash. Skeleton's Go declaration rows are
the required table projected to request_hash,name,declaration_hash; the common
root's source_vc_hash supplies the complete context/statement binding.

## Policy, AI and retained profiles

Materialized Go properties have exactly the catalogue's fields and cover every
request once. Source binding, frame, definedness and loop/call obligations cannot
be omitted from readiness. A checked property requires exact source/assembly/VC/
statement/manifest identities, identical accepted certificate bytes in both
checkers, zero Go axioms and a fully discharged dependency DAG. A pending member,
candidate rejection, unavailable elaborator, resource limit and source rejection
remain separate outcomes. Grouping by user claim changes presentation only.
AI receives the complete policy-owned property set through the already frozen
redaction/batching rules; provider text cannot supply a status or proof target.

The four retained profiles keep their exact compiled axiom policies and formats,
including mvp-theory's allowlists and five-category report. The private helper
that observes two zero-axiom Rust fixtures is not a generic policy for all Rust,
C# or Java certificates. Their current assembly errors remain unchanged recorded
observations. Actual native four-profile preservation is required before cutover.

## Go host and resource budgets

The [host contract](host-resources.json) is a distinct practical-Go host/probe
identity: native Linux x86_64 GNU, kernel ABI at least 6.4.0, cgroup v2,
swap-disabled tmpfs, namespace isolation and complete descendant cleanup. Old
GoFixed's weaker host contract is preserved. Reusing compiler content requires
independent byte/mode/runtime validation; it cannot relabel an old frontend.

One frontend tree has 120 seconds, 2GiB cgroup memory, zero swap, 128 tasks,
16GiB address-space limit, 1,024 open descriptors and no core dumps. Scratch has
512MiB and 131,072 inodes, noswap/nodev/nosuid/noexec. GOMAXPROCS=1 and the soft
GOMEMLIMIT=1.5GiB do not replace cgroup enforcement. A monotonic parent watchdog
starts before spawning; setup, compiler, children, output and validation stay
inside the tree budget. No child runs before atomic cgroup placement. Output
limits are the existing frontend transport limits, enforced while reading.

Construction and each source-free checker run sequentially with their existing
60-second/1GiB per-child proof ceilings; the new runner additionally caps the
whole child tree at 1GiB cgroup memory and 128 tasks. Proof scratch is 64MiB /
16,384 inodes. The existing certificate caps remain 2MiB,131,072 terms and4,096
declarations. Enforcement includes descendants; a proxy's RSS is no substitute.
The measured Darwin maxima were 466,616,320 bytes construction and 13.009 seconds
reference checking. The original unused-HTTP load measured about367MiB and
1.308 seconds for1,417,549 syntax nodes. These motivate headroom; they guarantee
neither every admitted 4,194,304-node input nor native Linux application fit.
Every existing structural/capture/transport counter retains its exact counting
rule. Counts are checked before allocating/decoding the corresponding object.

T02 implements capture preflight; T07 implements the full native runner and
fault/corpus gates at the listed production/test owners. Missing host primitives
produce environment_error; timeout/OOM/task/scratch exhaustion produce
resource_pending with original diagnostics. Neither is source rejection, a
false theorem, permission to widen limits or a fallback host. A receipt requires
an empty descendant cgroup and independently captured outputs after termination.
No native enforcement was observed by this design audit.
