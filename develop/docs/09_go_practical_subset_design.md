# Go Practical Subset Expansion Design

Status: implementation authorized on 2026-10-16, starting with
`GO-PRAC-T01-W01`. The user-authorized scheduling amendment permits Go work
before the practical C# release. No specification, profile, registry entry,
or public command is activated by this document.
The implementation sequence is in
[the companion work plan](09_go_practical_subset_design-todo.md).

## 1. Decision and delivery order

Extend verification of ordinary Go business functions in the priority order
requested by the user. Keep application source free of MPK imports, generated
application code, and MPK runtime dependencies. Contracts and verification
configuration live in a separate overlay.

Establish selected-function verification inside existing modules first. Then
deliver these capabilities in order:

| Priority | Capability | Intended use | Delivery |
| --- | --- | --- | --- |
| Prerequisite | Captured dependencies and a precise selected-function closure | Verify a policy function in a package that also contains ordinary application code | v1 |
| 1 | Target-sized integers, domain types, constants, ordinary control and assignment syntax | Use existing amount/status types and helpers | v1 |
| 2 | Multi-result calls and assignments, closed `error` values | Verify success and failure outcomes | v1 |
| 3 | Bounded byte strings | Validate identifiers, status codes, and messages | v1 |
| 4 | Bounded slices, `range`, and controlled construction | Validate and aggregate lists; build bounded results | v1 |
| 5 | Bounded maps with lookup and controlled updates | Check membership, deduplicate, and update keyed state | v2 |
| 6 | Restricted pointers and mutable receivers | Verify application state transitions and their frame conditions | v2 |

The first useful public milestone is priorities 1–4: a function taking
application structs, strings, and slices and returning `(T, error)`, verified
in its original module. Maps and pointers follow as a second immutable
profile. `mpk.go.practical.v1` and `mpk.go.practical.v2` are proposed names in
this design, not registered identities. v1 is never widened in place to
implement v2. There are no caller-selectable experimental feature switches.

“Supported” requires source admission, faithful lowering, independent artifact
validation, VC generation, usable proof construction, acceptance by both
source-free checkers, and installed-release replay. Parsing, lowering, or
checking a definition alone does not establish support for a proof claim.

## 2. Baseline, authority, and scheduling

The baseline is [GO_VIR_PROFILE_V0.md](../specs/GO_VIR_PROFILE_V0.md), not the
historical `GO_SUBSET_V0.md`. Its existing value domain is Bool, fixed-width
integers, fixed arrays, and nominal structs. Restricted loops and static calls
already exist. Functions can declare multiple results, but called functions
currently have exactly one result. The new work therefore extends the call
and assignment model rather than introducing result lists from nothing.

Current integration obstacles include package-wide subset rejection,
dependency-free `go.mod`, unit-local struct identities, and the absence of
strings, slices, maps, and pointers. The principal code owners are
`go-tools/go2vir/{preflight,loader,features,lower_types,lower,contract}.go`,
`crates/mpk-vc/src/{vir,vir_validate,program_wp,program_encode}.rs`, and the
successor artifact/profile consumers. The current policy strategy in
`crates/mpk-api/src/policy_strategy.rs` covers a small payment-oriented set;
new source syntax does not automatically expand that strategy.

This design is subordinate to Certificate v0, the trust boundary, the active
semantic registry, and the current shared-artifact specifications. The fixed
Go profile retains its accepted inputs, diagnostics, limits, and semantics.
The new profile carries its own source rules, dependency policy, target,
limits, operations, contracts, and conformance vectors.

The user-authorized serial order is:

```text
registered Go/Rust/scalar-C#/Java release + audited current source baseline
  -> Go practical v1 (T01–T07)
  -> Go practical v2
  -> resume and finish practical C# from its retained checkpoint
  -> Dart, TypeScript, and Python
```

This replaces the original proposal to wait for practical C#. The amendment
is also recorded in [the multi-language design](06_multilanguage_frontend_design.md).
T01-W01 pins the clean source baseline and the last installed-release receipt
separately: that historical receipt does not certify subsequent source
changes. Go's own feasibility, freeze, stage gates, and native release gates
remain required. Earlier Go integration value justifies moving this work onto
the critical path; the cost is deferring remaining practical C# work and
reconciling its shared artifacts before it resumes. No production language
phases run concurrently.

The practical C# work plan currently records reopened foundation work and
pending proof completion. Its
[current status and amendment](08_csharp_practical_subset_design-todo.md)
are authoritative; older completion prose is not evidence that Go can already
reuse a finished foundation. Planning this Go extension does not depend on
declaring that work complete. Its release receipt is no longer an entry
condition for GO-PRAC. A C# completion claim cannot replace Go's own proofs.

## 3. Trust and artifact strategy

The implementation retains the certificate-first pipeline:

```text
captured Go source + dependency snapshot + verification overlay
  -> registered Go compiler/type information
  -> selected declaration/effect closure
  -> profile-bound typed operations and control flow
  -> independently validated VIR and source linkage
  -> safety, contract, binding, and frame obligations
  -> untrusted proof construction
  -> canonical certificate -> Rust checker + independent Go checker
```

All stages before checker acceptance remain untrusted helpers. Compiler
output, dependency hashes, ownership annotations, solver answers, and source
maps are not proofs. A source hash establishes which bytes were analyzed; it
does not prove the Go compiler, frontend, machine code, or surrounding service
correct. Evidence must state the selected functions, captured context,
preconditions, and theorem scope.

Use ordinary, closed semantic definitions and checked lemmas under a
zero-axiom policy for the new Go profiles. Keep Certificate v0, checker input
interfaces, and the four axiom categories. Do not solve an encoding problem by
adding an unchecked string/map/heap oracle or an assumption that a callee is
correct. If a required construction cannot be expressed and checked within
the available kernel and resource limits, the corresponding milestone stays
incomplete; a checker change would need its own explicit design and review.

Start from the registered release and audited source baseline. Common
mechanisms for closed specialization, ordinary definitions, explicit control,
and source binding may be reused only after their Go-required scope passes
Go's own independent validation and proof gates. Inactive C# code is a
candidate implementation, not an established dependency. If a needed common
facility is unfinished, complete and verify that bounded facility in GO-PRAC
instead of waiting for the entire C# profile or assuming its correctness.
Go owns its bindings: C# UTF-16 strings, ordered
collections, exceptions, and overflow rules are not Go semantics. Do not
copy the C# frontend's source-specific types into a Go interface.

Before implementing a changed serialized shape, freeze its exact producer,
consumer, schema, hash-domain, limits, and migration inventory. Extending a
closed operation union, call result shape, or contract expression requires
successor shared identities where its meaning changes. Migrate all affected
active producers and consumers atomically; retain one installed shared
artifact family. Old-profile semantic preservation does not imply that old
and successor helper bytes can be interchanged.

## 4. Existing-project integration prerequisite

### 4.1 Two different closures

The capture/type-check closure contains everything consulted to resolve and
type-check the selected package: relevant source files, module metadata,
dependency identities, build selection, and compiler-produced export data.
It must be complete and reproducible even for declarations outside proof.

The verification closure starts from an explicitly selected function or
method and includes every statically called body, referenced constant and
type definition, required method signature, and contract dependency. Compute
the closure to a fixed point; retain an acyclic call graph initially. An
unknown indirect call is rejected. A branch believed unreachable is still
included unless its unreachability has an accepted proof under the contract.

Type-check the original package. Do not manufacture a replacement Go package
by deleting functions, changing imports, or stubbing types. Unsupported
operations in an unselected function may remain type-check-only. Unsupported
operations in the verification closure reject with a path from the selected
root to the offending declaration. Syntax and type errors in the original
package still reject.

The manifest records declaration disposition: selected root, body in proof,
type/constant dependency, type-check-only, or rejected dependency. All consulted
bytes remain hashed even when their bodies are outside proof. Unrelated
implementation changes may invalidate capture identity without changing a
normalized theorem; they must not reuse stale source evidence.

### 4.2 Initialization and ambient effects

The verified theorem starts at function entry. Package initialization and
application startup are outside that theorem. MPK never executes application
initializers while loading source. Unrelated `init` functions and package
variables can exist in the captured package; reaching mutable package state
from a selected body rejects. Constants are evaluated from compiler-resolved
constant expressions; mutable globals are not treated as constants merely
because a programmer intends them to be immutable.

A selected function cannot call I/O, clocks, randomness, logging, goroutines,
or an unmodeled library operation. For slices, maps, and pointers, entry
contracts state the accessible regions and prohibit concurrent external
mutation for the duration of the call. These are caller obligations, not a
proof of race freedom for the whole application. Capture must not omit a
relevant initializer, global read, callback, or hidden effect by calling it
“unrelated.”

### 4.3 Dependencies and build context

Allow ordinary `require` directives and nonempty `go.sum`. Provision a
separate dependency snapshot before verification; the verifier reads only
sealed captured inputs and does no downloading, VCS access, generation, or
automatic toolchain selection. Record the resolved module graph, all
consulted module files, replacement mapping, source/archive digests, and
toolchain export-data provenance. A checksum is an identity check, not proof
of a dependency's behavior. This follows the separation between module
resolution and packages documented in the [Go modules reference](https://go.dev/ref/mod).

Initially accept versioned replacements and local replacements whose complete
roots are explicitly included in the capture capability. Never follow an
arbitrary host path. Capture selection-relevant `exclude`/`retract` metadata;
do not invent a new module-selection algorithm. Defer `go.work`, vendor mode,
cgo, custom build hooks, and unmodeled compiler directives. Generated files
can be ordinary captured source; generation is never part of verification.

Record GOOS, GOARCH, architecture tuning, build tags, effective per-file
language versions, module directives, and exact compiler release. Start with
`linux/amd64`, 64-bit `int`/`uint`, and the existing target tuning. T01 pins
one available registered compiler and an explicit supported-language-version
table, checked against its bundled specification. No “latest” or host
fallback exists. Higher module/toolchain requirements and effective language
versions outside that table reject with an actionable diagnostic. Build tags
are explicit captured configuration; test files remain outside the ordinary
function selection.

A reachable external helper has two acceptable implementations: verify its
captured source in the same profile closure, or use a registered, exact-symbol
semantic adapter with checked binding obligations. A checked imported
certificate must match the callee semantics, source/context, signature,
preconditions, result relation, and effects. An arbitrary user-supplied
contract with no proof is insufficient. Merely importing an unused HTTP or
database package no longer rejects a pure selected function.

## 5. Priority 1: ordinary types and syntax

Admit `int` and `uint` at the selected target width, named Bool/integer/array
types, aliases to admitted types, inferred and grouped constants, `iota`, and
cross-package domain structs/constants. Keep `uintptr`, floats, complex
numbers, embedded-field promotion, and generic instantiation outside this
release. A source nominal type is identified by its captured module instance,
package, and declaration, with a separate underlying value representation.
An alias preserves its target identity. The validator must not erase nominal
assignment/conversion restrictions when lowering to bitvectors.

Admit `else if`, expression `switch`, scalar compound assignments, local
increments/decrements, parameter reassignment to local copies, omitted-field
zero initialization, and local value-struct/array updates. Normalize them to
existing branches and explicit value updates where possible. Switch cases
are evaluated in source order; this release excludes type switches,
`fallthrough`, labels, and `goto`. Parameter reassignment does not modify a
caller's value. Reference-valued fields introduced later retain their region
identity when their containing struct is copied.

Retain wrapping Go arithmetic. A business requirement such as “the sum never
overflows” is an additional theorem, not a change to integer addition.
Constant evaluation remains exact until the compiler's representability
check. Preserve short-circuiting in every newly admitted expression context.
Lower an assignment target once, without duplicating calls or index checks.
For value updates, initially allow only side-effect-free address/index
expressions; more general effects wait for priority 6. These rules must match
the pinned [Go specification](https://go.dev/ref/spec), including its order of
evaluation rather than a blanket left-to-right assumption.

Required examples: imported `type Amount int64`, `iota` status constants,
switch-based eligibility, partial struct literals, and local fixed-array
updates. Required negatives: nominal-type confusion, constant overflow,
target mismatch, unknown switch forms, and a proof that incorrectly treats
wrapping addition as unbounded arithmetic.

## 6. Priority 2: multi-result calls and errors

Represent every function result as one typed internal product with fixed
arity 0–16 and ordered projections. This is an internal call representation,
not an application-visible wrapper struct. A call executes once; all its
results and, later, post-state refer to that same execution. Add tuple and
parallel assignments, mixed declaration/reassignment with `:=`, discarded
results, forwarding returns, named result initialization, and explicit
normalization of naked returns. Preserve two-phase assignment behavior and
shadowing. Defer `defer` statements.

Model `error` as a closed interface value:

```text
ErrorValue = NilInterface | ConcreteError(dynamic_type, payload)
```

The finite dynamic-type set comes from resolved constructors, parameter
contracts, and reachable return paths. Initially allow application-owned,
non-pointer concrete Bool/integer/struct error values and comparison with
`nil`. Open interface dispatch, error-to-error comparison, type assertions,
`errors.Is`/`As`, `errors.New`, and `fmt.Errorf` are outside this milestone.

Implementing the Go `error` interface requires an `Error() string` method.
That method's signature is a type dependency; conversion to `error` does not
call its body. Thus priority 2 can handle `(T, error)` before general string
operations exist: the method body is type-check-only unless actually called.
Selecting or calling it requires priority 3's admitted body and closure.
There is no trusted “error message” evaluation hidden in priority 2.

Contracts name results by position and describe success/failure separately.
They do not assume `err != nil` implies a zero first result; that is proved
only if the function's contract requests it. The closed error domain and any
restriction on an error parameter are explicit entry preconditions.

Priority 6 may add pointer concrete errors. Its representation must preserve
the difference between a nil interface and an interface containing a typed
nil pointer; they are not the same value. This distinction is explained in
the [Go FAQ](https://go.dev/doc/faq#nil_error). Until that extension, returning
a pointer error is an explicit subset rejection, not conversion to a nil tag.

Required examples: `(Amount, bool)`, `(Decision, error)`, nested forwarding,
discarded results, and a nonzero first result on failure. Negatives include
repeated call evaluation, tuple reordering, stale result/effect projection,
unlisted dynamic types, and incorrect nil collapsing.

## 7. Priority 3: bounded strings

Use a closed byte-sequence carrier with a logical length and a declared
maximum. Strings may contain arbitrary bytes; do not require valid UTF-8 or
normalize Unicode. Admit literals, zero values, equality, bytewise ordering,
`len`, byte indexing, two-index substrings, and concatenation subject to a
proved result bound. Nil is not a string value. The byte interpretation is
documented in [Strings, bytes, runes and characters in Go](https://go.dev/blog/strings).

Provide checked byte equality, prefix, and suffix relations in the internal
foundation. A source call to `strings.HasPrefix` or `strings.HasSuffix` needs
an exact registered symbol adapter and a checked source/model binding for the
pinned library. Source helpers implementing the same logic may instead enter
the verification closure. Do not authorize the entire `strings` package by
name. Regex, locale operations, Unicode normalization, string/rune conversion,
and `range` over strings are deferred. `Error()` methods can now be selected
when their bodies fit this subset.

Bounds on incoming byte length are function preconditions. Concatenation's
capacity obligation is a profile-domain obligation, not a fictitious Go
panic. Index/substring definedness generates ordinary panic-freedom VCs.
Prove length/content equations and the preservation of the source-to-carrier
relation; do not prove only that the carrier can be constructed. Tail bytes
outside the logical string can be canonicalized only after proving they are
unobservable under the admitted operations.

Required cases: empty strings, embedded NUL, invalid UTF-8, multibyte text,
equal prefixes of different lengths, substring endpoints, and concatenation
at the bound and one above it. “Character count” must not silently mean byte
length in a contract or diagnostic.

Private T01-W03 substring/concatenation feasibility evidence is recorded in
[the implementation ledger, section 23](go-practical-implementation-traceability-ledger.md#23-t01-w03-guarded-substring-and-bounded-concatenation).
It derives substring lengths and source indexes from actual signed checks,
preserves all eight byte bits under input-tail normalization, and derives
concatenation carry and index relations with an explicit result-bound
obligation. Semantic bounds and finite padding remain separate. This evidence
does not activate a profile or complete source/library binding. The next
[private prefix/suffix checkpoint](go-practical-implementation-traceability-ledger.md#24-t01-w03-byte-prefixsuffix-and-selected-body-correspondence)
adds checked relations, actual range/index facts and exact selected Go 1.25.0
source identities. Production adapters and full application composition remain
due; the whole `strings` package is not admitted by name.

## 8. Priority 4: slices and useful loops

### 8.1 Representation and admission

Use an explicit slice descriptor and backing store:

```text
Slice<T> = (nil_tag, region_id, offset, length, capacity)
Heap[region_id] = bounded initialized storage of T
```

Bounds include both length and capacity. Preserve nil versus non-nil empty
slices, slice descriptor copies, shared backing arrays, and the distinction
between array copying and slicing. Sub-slicing changes a descriptor rather
than copying elements. Data beyond `length` but within `capacity` cannot be
discarded: a later accepted re-slice may expose it. These choices follow the
storage behavior described in [Go Slices: usage and internals](https://go.dev/blog/slices-intro).

Descriptor validity requires `0 <= length <= capacity` and, for a non-nil
descriptor, a live typed region with `0 <= offset` and
`offset + capacity <= storage_length`. A nil descriptor has zero length and
capacity and no accessible region. A non-nil empty descriptor retains its
non-nil tag even when it has no accessible element. Input/model binding must
preserve sharing and offsets across parameters; assigning a fresh region ID
to every incoming slice would be an unsound way to satisfy separation.

Implement read-only parameters first: `[]T`, `len`, `cap`, `== nil`, indexing,
two- and three-index slicing, array-to-slice views, and slice descriptor
assignment. Then add fresh, bounded local construction with literals,
`make`, element assignment, `copy`, and `append` that fits existing capacity.
Element types are closed admitted value types with no nested mutable
references in v1; structs containing ordinary scalar/string fields are useful
elements. More general nested slice/map graphs remain outside v1.

Read-only borrowed input regions may alias one another. Fresh writable
regions are disjoint from input regions. Local aliases of a fresh region
share one store and must observe each other's writes. Calls cannot hide a
write through a borrowed view. `copy` must handle overlap using its specified
source-value behavior; it is not lowered to a naive forward loop.

The first `append` implementation generates a proof obligation that the new
length fits the existing capacity. It therefore supports the common
preallocate-and-append pattern without modeling runtime growth policy.
Reallocating append is deferred, including in v2. Failure to prove the
no-reallocation condition is a profile restriction, not evidence that Go
would panic. `make` with explicit length/capacity has definedness checks and
profile resource obligations. The theorem scope excludes process-level OOM.

### 8.2 Control flow and contracts

Add `range` over arrays/slices, condition-only `for`, structured nested loops,
unlabeled `break`/`continue`, and early returns. These are required for
validation and lookup code. Bind contracts to stable source loop identities
and verify that normalization maps each contract to exactly one loop.

Capture the range descriptor and length once. Read an element at its proper
iteration step. Account for array value copies. A continue edge passes
through the appropriate loop post operation; a break edge establishes the
exit condition without incorrectly assuming the loop test is false. Early
return establishes the function postcondition directly. Use explicit control
exits rather than pretending every loop is the old single-exit Go form.

Lower induction variables and range bindings according to the effective
language version. In particular, the compiler-version table must account
for the per-iteration bindings introduced in
[Go 1.22](https://go.dev/doc/go1.22). Reject integer, string, channel, map, and
iterator-function range forms in v1; their syntax does not authorize them.

Every loop requires initialization and preservation of an invariant, checks
on each actual exit, and operation safety. Total correctness additionally
requires a decreasing, well-founded variant. A finite slice bound is not by
itself proof that an arbitrary loop terminates. Bounded range loops can get
a generated counter variant; inferred invariants are untrusted candidates
and must be proved. Output bounds and business arithmetic bounds are proved
separately from loop termination.

Required cases: nil/empty input, full-capacity re-slicing, alias observations,
overlapping copy, zero-length construction, preallocated append, nested
validation, continue/post behavior, and early-return failure. A negative
fixture must catch an unsound “slice equals copied list” model.

## 9. Priority 5: bounded maps

v2 starts with `map[K]V`, nil/empty states, literals, `make`, `len`, lookup,
comma-ok lookup, insertion/replacement, and `delete`. Initially K is Bool,
an admitted integer, or a bounded string, including named versions. V is a
closed value type without mutable references. Float keys, interface keys,
pointer keys, nested reference values, reflection, and source map iteration
remain unsupported.

Represent a map value as a reference to a finite partial mapping plus a nil
tag. Preserve assignment aliasing and distinguish an absent key from a
present key with the zero value. Read-only input maps may alias. Writable
maps must be fresh or supplied with an exclusive-region entry contract.
Permission transfer and all local aliases refer to the same map store.

Prove lookup/insert/delete laws, unrelated-key preservation, cardinality,
and frame conditions. Nil lookup/delete/length behavior and nil-map assignment
failure have distinct rules. Inserting a new key must prove the cardinality
bound; replacement does not necessarily increase it. Source `make` hints
are not semantic cardinality limits. The basic observable behavior is
described in [Go maps in action](https://go.dev/blog/maps).

Canonical key sorting may encode the internal finite mapping, but it cannot
define a source iteration order. The v2 rejection of `range` over maps is
intentional. A later version could admit iteration with universally
quantified iteration choices or order-independent proofs; it must not reuse
C# ordered-map semantics. Required negative cases include nil writes, missing
versus zero values, aliased mutation, bound overflow on a new key, and
acceptance that depends on a convenient iteration order.

## 10. Priority 6: restricted pointers and state transitions

Admit `*T`, nil tests, dereference, address of an addressable local root,
`&T{...}`, `new(T)`, direct pointer-receiver calls, and field/element updates
through an admitted root. Initially T is a nonzero-sized scalar or an acyclic
value aggregate containing scalars, strings, and fixed arrays. Recursive
pointer graphs, reference-valued fields, interior address taking such as
`&slice[i]`, pointer arithmetic, `uintptr`, unsafe code, and pointer escape to
ambient state are deferred. This supports account-like records without
claiming support for arbitrary Go heaps.

Use typed region identities and functional heap updates. Read-only input
pointers may alias. Each writable parameter needs exclusive access to its
region, disjoint from other live borrowed/writable parameters. Aliases
created locally retain the same region identity and observe its updates;
ownership means control of external aliases, not pretending local aliases
do not exist. A callee receives and returns explicit effect/permission state.
Returned pointers must refer to an admitted input region or a tracked fresh
allocation, with ownership transfer recorded.

Pointer equality is admitted only for these nonzero-sized root objects;
derive it from region identity and retain nil as a distinct value. Selecting
a pointer receiver does not itself assert that it is non-nil: a method that
handles a nil receiver may be valid. Generate the nil check at actual
dereference. Mutable calls embedded in expressions with interfering sibling
reads/writes reject initially; explicit sequencing makes their effects
reviewable without choosing a stronger evaluation order than Go guarantees.

Contracts add `old` observations, a finite modifies set, permission
requirements, and normal-return state invariants. Generate obligations for
nil safety, region validity, every write's permission, callee footprints,
unchanged locations outside the modifies set, and the post-state relation.
An “error leaves state unchanged” promise needs an explicit theorem; it is
not an automatic consequence of returning `error`. Preserve typed-nil error
boxing as specified in section 6.

The first business example is a guarded balance reservation with `(Amount,
error)`: sufficient funds decrease the balance by the approved amount;
failure returns the agreed error and preserves all state. Include adversarial
tests for overlapping input aliases, hidden callee writes, nil receivers,
frame violations, address identity, typed nil, and failure after a write.

## 11. Contract and proof model

Extend the overlay with closed, typed forms for:

| Contract area | Required meaning |
| --- | --- |
| Results | Ordered projections, closed error tags, success/failure postconditions |
| Domain bounds | Byte length, slice length/capacity, map cardinality, heap/region limits |
| Data observations | String bytes; slice elements; map presence/value; typed fields |
| Loops | Invariant, optional totality variant, all normal/abrupt exits |
| Calls | Preconditions, joint result relation, effect footprint, checked dependency identity |
| Mutation | Entry/old/current state, borrows, exclusivity, modifies and frame conditions |

Any quantified observation is bounded and typed. The overlay is declarative;
it does not execute arbitrary Go or allow a caller-defined semantic adapter.
Imported contracts must agree on nominal types, bound parameters, and result
and region identities. No implicit conversion between two instantiations
with different bounds is allowed without a checked relation.

For an admitted operation, prove representation validity, source/model
binding, definedness, and its value/effect relation. For a function, prove
callee preconditions and dependencies, each runtime safety condition, loop
obligations, requested postconditions, and relevant frame/totality claims.
The new product policy must route these obligations to checked lemmas,
finite-domain reasoning, and arithmetic candidates as appropriate; the old
payment-strategy allowlist is insufficient.

The minimum v1 proof library includes a checked bridge from bounded signed
bitvector arithmetic to the mathematical sums used by business contracts,
product/error projection laws, byte length/content laws, slice read/update
and alias laws, and loop induction. v2 adds finite-map update laws and heap
frame/permission composition. Reuse symbolic lemmas and shared proof DAGs;
do not enumerate all bit patterns of a bounded collection. Existing ground
bitvector evaluation is not a substitute for these symbolic application
proofs. T01/T08 feasibility measurements must exercise these actual proof
shapes before their profile scope is frozen.

Bounds are proof-domain restrictions. For example, a contract with
`len(lines) <= B` proves behavior for those inputs; it does not prove rejection
of arbitrarily long slices. Even a source guard rejecting larger inputs does
not extend a finite input carrier to all Go slices. The application must
establish the entry predicate, or a separately proved outer model must cover
the broader domain. Never truncate input, silently strengthen a precondition,
or report a profile-size limit as a Go runtime failure.

Operationally distinguish source rejection, accepted model with pending
obligations, failed candidate, and a fully checked claim. These are conceptual
states until their exact policy/evidence fields are frozen. Timeout, unknown,
unsupported strategy, failed proof, and a checked counterexample are distinct
outcomes. Diagnostics must not label every unsuccessful proof “false.”

## 12. First release acceptance example

This is ordinary application code, not a proposed runtime API. The `Error`
method becomes part of proof only if selected or called.

```go
package policy

type Amount int64
type Line struct { Amount Amount }
type Decision struct { Total Amount }
type RejectCode uint8

const (
    BadInput RejectCode = iota + 1
    LimitExceeded
)

func (RejectCode) Error() string { return "order rejected" }

func Quote(customerID string, lines []Line, limit Amount) (Decision, error) {
    if customerID == "" || limit < 0 {
        return Decision{}, BadInput
    }
    total := Amount(0)
    for _, line := range lines {
        if line.Amount < 0 {
            return Decision{}, BadInput
        }
        if line.Amount > limit-total {
            return Decision{}, LimitExceeded
        }
        total += line.Amount
    }
    return Decision{Total: total}, nil
}
```

The overlay gives byte and slice/capacity bounds, stable read access to the
input region, a closed error set, and the loop invariant
`0 <= total <= limit` plus equality with the mathematical sum of the processed
prefix. The invariant is attached only after the initial negative-limit
return. Prove no arithmetic overflow on the continuing path, correct error
precedence, zero `Decision` on failure, the exact bounded sum on success, and
no input mutation. The summation relation itself needs checked definitions
and lemmas; it is not an uninterpreted promise.

The corpus also records concrete admissible inputs reaching success and
failure, such as amounts `[2, 3]` with limit `5` and amount `[6]` with limit
`5`, using a nonempty customer ID. This catches an accidentally vacuous entry
contract without treating concrete executions as the universal proof.

The acceptance fixture places this function in a real multi-package module
with an unused handler and a pinned external dependency, without rewriting
the application or adding an MPK dependency. Run the application tests as
well as the verification pipeline. A second fixture builds a filtered output
with `make([]T, 0, len(input))` and capacity-preserving `append` so the v1
release also demonstrates construction, not only read-only aggregation.

## 13. Implementation ownership

The following are change surfaces, not permission to grow existing frozen
parsers in place. New profile-specific files sit beside these owners.

| Surface | Existing owner | Planned change |
| --- | --- | --- |
| Capture/resolution | `go-tools/go2vir/preflight.go`, `snapshot.go`, `loader.go` | Immutable module snapshot, explicit build context, two closures |
| Source admission | `features.go`, `lower_types.go`, `ssa.go` | Profile-specific type, syntax, call/effect and declaration checks |
| Source lowering | `lower.go`, `vir_types.go`, `source_map.go` | Typed products, byte data, regions, explicit exits and source origins |
| Contracts/input identity | `contract.go`, `manifest.go`, `hash.go` | Bounds, error universe, loop attachment, permissions and dependency closure |
| Registry/release | `crates/mpk-vc/src/semantic_profile*.rs`, `crates/mpk-cli/src/successor_*` | Compiled profile contracts, registered bundles, atomic activation |
| Shared validation/VCs | `crates/mpk-vc/src/vir_validate.rs`, `program_wp.rs`, `successor_vc.rs`, `successor_source_artifacts.rs` | Independently checked typed operations, call/effect relations and obligations |
| Ordinary proof construction | Existing practical foundation/ordinary-definition machinery, with Go-owned bindings | Closed byte/sequence/map/region lemmas and actual obligation proofs |
| Policy/API | `crates/mpk-api/src/policy_strategy.rs` and CLI policy/evidence consumers | New obligation families, precise readiness, checked evidence and assumptions |
| Regression/release tests | Existing Go corpus, profile vectors, source-free checker and installed-release tests | Paired source/VIR/VC/proof cases, both-checker acceptance and replay |

T01 and T08 inventory exact symbols/files after their entry revisions are
pinned; the current C# implementation can move these common owners before
Go work starts. Every frozen requirement gets one primary executor and all
affected consumers. Test-only model evaluation is not a production executor.

## 14. Validation and release gates

Each priority has a complete source-to-certificate example and a negative
case designed to expose its most likely unsound abstraction. Validate:

1. Compiler/runtime differential observations on captured, pinned Go inputs.
   These detect translation errors; runtime tests are not formal proof.
2. Independent VIR import and contract/type/effect reconstruction, including
   forged type IDs, region aliases, bounds, and dependency/context hashes.
3. VCs for all paths and exits, including nontrivial failure paths. A fixture
   of semantic definitions alone cannot satisfy this gate.
4. Actual universal or contract-conditional application theorems accepted by
   both checkers, with the declared zero-axiom policy and no pending members.
5. Byte-identical replay of canonical helper/certificate artifacts for the
   same context, and rejection of crossed versions and tampered inputs.
6. Old fixed-profile preservation and all consumers affected by a shared
   artifact migration. Other languages retain their own semantics.
7. Installed Linux execution through the registered runner, then the complete
   local release gates and two-pass receipts required by the root README.

Probe resource feasibility before freezing each profile. Candidate workload
points are strings of 0/1/32/256 bytes, slices of 0/1/8/32/128 elements, maps
of 0/1/8/32 entries, and 1/2/16 mutable root objects. These are measurement
targets, not claims of supported limits. Use representative symbolic
obligations, not only ground evaluations or empty definitions. Record proof
construction time, checker time per checker, peak memory, certificate bytes,
term/node counts, and aggregate-bound interactions on identified hardware.

Freeze exact limits and local resource budgets from those measurements in
T01 for v1 and T08 for v2. Measure limit-minus-one, limit, and limit-plus-one
at each relevant boundary. The implementation cannot raise a frozen limit to
hide a failed acceptance example, nor mark an entire workload supported when
its final theorem times out. Distinguish semantic capacity from transport and
proof-resource limits; their diagnostics and evidence differ.

During implementation, each W runs targeted tests/lint and a diff review.
Only the last W of a T runs `scripts/check-fast.sh` over that T's cumulative
changes. The next T waits for that gate to pass. Final public activation also
requires the root README's native Linux release gates. All checks are local;
GitHub Actions/workflows are not part of this plan.

## 15. Deferred capability boundaries

General interface dispatch, generics, unrestricted standard libraries,
reallocating append, map iteration, arbitrary recursive heaps, error wrapping,
Unicode-oriented text processing, floats/complex numbers, goroutines,
channels, `select`, synchronization, `defer`/`recover`, reflection, unsafe
code, and cgo need later profiles. Unused declarations may use some of these
while remaining outside the verification closure, subject to capture and
type-check constraints.

These exclusions are explicit product scope, not placeholders interpreted
as successful proof. The two releases above must already verify useful
application functions with clear caller obligations and reproducible
certificate evidence.
