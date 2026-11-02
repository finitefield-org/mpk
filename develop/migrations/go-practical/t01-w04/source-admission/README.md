# Closed source syntax and selected-call closure draft

Status: W04 private draft; production admission and source/model proofs pending.
Entry: `af62c97bb2788638303b2c9d7e38a5046d664076`.
The [catalog](catalog.json) assigns every concrete Go1.25 AST node and every
predeclared builtin an explicit disposition and planned executor/test owner.
Its 56 AST nodes are checked against all concrete pointer-receiver Pos methods
in the pinned compiler's go/ast/ast.go. Its 18 builtin names match the pinned
go/types universe; only append, cap, copy, len and make are conditional v1
candidates. A name or parse success alone never grants practical support.

## 1. Closed forms and dependent checks

Seventeen AST kinds reject in a selected verification closure: malformed
nodes, map/channel/interface literal types, pointers/dereferences, type
assertions/switches, function literals, generic multi-index forms, labels,
goroutines, defer, channel send/select and communication clauses. Seven
capture-structure kinds describe original source without authorizing code
execution. The other 32 kinds carry explicit conditions: exact source/type
identity, Root/Plain shape, Go assignability/conversion, static calls, ordered
results, structured control, evaluation and lifetime/effect obligations.

The array/slice/string forms, inferred array literal extent, local declarations,
Go struct tags and compatible tag-ignoring conversions retain their original
compiler meaning. Integer-to-string, string/slice conversion, slice-to-array,
unsafe and pointer conversion stay excluded under GP-OP-03. Exact untyped
constant arithmetic can precede materialization into an admitted word;
runtime floating/complex values remain excluded. The compiler checks constants
and nominal assignment before this draft classifier observes the package.

Range is limited to value arrays and slices. The effective-language table
remains Go1.22–1.25 with pinned compiler Go1.25.0 and target gc/amd64 sizes.
Unlabeled break/continue retain structured targets; fallthrough/goto reject.
Error values may be compared only with nil. Value-method calls retain their
qualified receiver declaration; interface dispatch and function values reject.
Initial function signatures are nongeneric/nonvariadic with result arity0..16.

## 2. Keep type-check closure separate from body proof

The [observer](../../../../probes/go-practical/admission_go_facts.go) parses and
type-checks intact original single-file packages. It never runs application
initializers or bodies. All original parse/type errors are reported even in
an unselected function. Every function's physical byte identity, AST forms,
direct calls and typed issues are retained as untrusted compiler facts.

The [candidate classifier](../../../../probes/go-practical/admission_closure.py)
starts from one explicit root and follows all direct captured calls, including
value methods, parenthesized calls and a statically resolved generic body
before its generic use rejects. Recursive call cycles reject. Calls in an
apparently unreachable branch still enter the closure; the prototype accepts
no caller unreachability oracle. Declaration paths identify transitive
unsupported syntax or ambient-state access. Package variable reads/writes
reject rather than becoming constants or implicit entry assumptions.

Unselected function bodies remain type-check-only. In particular, constructing
an application concrete error requires its Error() string method signature
but does not call the body. Calling that concrete method includes its body;
dispatching through error rejects. An unused map function or init containing
panic can remain outside proof without deleting/stubbing source declarations.

This probe has no importer and deliberately tests self-contained packages.
It is not a production module/capture loader: multi-package declarations,
sealed snapshots, compiler directives, adapter identities and type/constant
dependency manifests still need their complete implementation and independent
validation. The retained [W02 capture/module matrix](../../t01-w02/module-matrix.md)
supplies separate feasibility evidence for actual module imports/replacements;
its sources and observations are preserved without unrelated reruns.

## 3. Native classification is not source/proof acceptance

The [81 original sources](source-vectors.json) and exact native output in
native-facts.jsonl are pinned by the [receipt](observations.json). Two identical
observer runs per source yield:

- 31 candidate-unproved classifications, including all four effective versions,
  arrays/slices/strings, error boxing, methods and ordered results.
- 46 rejected candidates at named syntax/type/call/builtin/ambient boundaries.
- Three original Go type errors and one original syntax error, including
  errors in unselected declarations.

The candidate status carries all pending type/closure/source/effect/domain/
permission/call/frame/loop proof gates. The observer's special predeclared
error treatment is conditional: it does not compute/prove the complete U,
error-payload recursion or global Plainness. An error entry candidate is never
an acceptance of an unlisted dynamic value. Fresh slice construction is not
proof of input ownership or a no-reallocation/source-resource bound. Native
constant/type failure and candidate rejection are not certified false claims.

The exact AST/builtin catalog and sample classifications do not establish
independent compiler correctness, complete semantic admission or application
proof. Source hashes identify bytes; typed observations remain untrusted.
No ordinary certificate is added at this checkpoint and no checker/core/
profile/installed-registry/production-source meaning changes.

## 4. Validation and next freeze boundary

Fifteen selected tests replay all retained vectors and check type-only versus
body scope, error signature/body distinction, transitive ambient paths,
recursive/false-branch and generic call closure, receiver identities, original
type errors, physical UTF-8 spans, source/target crossings and missing bindings.
Fresh replay rebuilds the pinned observer, checks source/toolchain/binary and
native-byte identity, and validates catalog/fixture/classification linkage.

The planned practical_closure, practical_types and practical_lower executors
and their tests retain primary ownership. Independent artifact validation
belongs to go_practical_artifacts.rs; operation/call/safety/domain/frame proofs
belong to their existing VC/foundation owners. These executors remain planned.

The [verification record](verification/verification.json) retains commands,
raw selected-check output and selection reasons. General predicates, exact
numeric/structural/capture/resource limits, schema/hash/independent import and
reviewed migrations remain W04 obligations. W04 stays In progress; W05 owns
inactive publication and the cumulative T01 gate. T02–T11 and the complete
implementation/release goal remain required.
