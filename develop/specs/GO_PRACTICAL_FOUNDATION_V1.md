# Go practical ordinary foundation v1

Scope amendment: [LANGUAGE_SUPPORT_SCOPE_V1.md](LANGUAGE_SUPPORT_SCOPE_V1.md)
limits implementation and retained contexts to Go/Rust. Four future contexts
replace earlier six-context retention statements; all Go semantics, common
artifact members, proof obligations and resource caps remain required.


Status: frozen inactive design. This document owns
`mpk.go.practical.foundation.conformance.v1` at
`develop/specs/vectors/go-practical-foundation-v1.json`. Primary test owner:
`crates/mpk-vc/tests/go_practical_foundation.rs`; publication checks:
`scripts/check-go-practical-spec.sh`. General source and application proofs are
required in T02–T06; no public foundation is installed by this publication.

The vector set pins the complete limits, source-carrier, operation, arithmetic,
storage/lifetime and program-binding design members. Its exact core interface
records are owned by [Nat induction](CORE_NAT_INDUCTION_V1.md) and
[Boolean type cases](CORE_BOOL_TYPE_CASES_V1.md). All other foundation functions,
relations, laws and application proofs must be ordinary checked definitions or
theorems. No caller extension, semantic axiom, special runtime library import or
source-callable MPK adapter is allowed. The application remains ordinary Go
with an external typed contract overlay.

Concrete specializations preserve source nominal identity and full underlying
values. Words retain all8/16/32/64 bits and checked signed/unsigned Nat/integer
bridges. Strings retain byte length and all256 possible byte values. Product
results and effects share one invocation. Closed errors include the complete
finite payload universe. Stores model canonical owners/typed views, all aliases,
initialized capacity tails, fresh-owner generations and every active/pending/
result/assignment-target/error root. Retiring a root requires a source-correlated
proof; a supplied live-set or finite carrier assumption is insufficient.

All56 operation families have the frozen concrete input/output/control types
and six common member categories: source binding, input validity, evaluation,
safety/domain/permission, value/effect/output validity and frame/joint invocation.
Their complete member ordinals, access points and dependencies are derived;
none may be omitted, replaced by free propositions or imported as axioms.
The independent source interpreter and IR semantics remain distinct. Concrete
source/IR/contract/entry/completion definitions and all19 theorem families are
exactly the program-binding member referenced by the conformance set.

Strong Nat induction and the rank+1 stopping theorem use arbitrary Sort0
predicates and unbounded Nat histories. They are not bounded loop unrolling.
Initialization/preservation/decrease obligations are proved at the actual loop;
only continuing edges need strict decrease. Early exits still preserve complete
return/effect/frame relations. Safety and partial correctness quantify over
every finite reachable prefix; total correctness additionally proves a normal
terminating run. Source faults, modeled-domain failures, unavailable strategies
and proof resource/fuel exhaustion retain their distinct meanings.

The semantic foundation DAG is limits -> definition inventory -> descriptor ->
registry/context -> captured specialization/ordinary program -> executable
release. All required fields enter the declared domain+NUL+canonical preimage
except the own hash. Raw member hashes include original LF and byte length.
No descriptor includes its own bytes or a descendant context/release hash.
Recipe inventories and private context fixtures are design inputs, not finished
checked foundation exports. T02 must generate actual closed ordinary definitions,
obtain both-checker acceptance and bind exact imports before materialization.

The W03/W04 receipts establish representative feasibility for words, bytes,
storage, abrupt control and well-founded ranks. They retain every original
failed construction and source limitation. They do not prove a general source
compiler, complete operation library or application. Production acceptance
requires exact concrete statements, source correspondence and complete
application-corpus proofs within the frozen certificate/host budgets.
