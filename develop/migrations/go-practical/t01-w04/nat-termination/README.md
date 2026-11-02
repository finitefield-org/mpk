# Private ordinary Nat order and strong induction

Status: measured W04 foundation proposal, not a published core/profile interface.
Entry: `b728e0f2543a894efb0b1039b4676126bbe6067a`.
The [receipt](observations.json) records 16 ordinary candidates, 20 isolated
type-selection interface cases and three retained regression certificates,
all independently checked from identical canonical bytes by Rust and Go.

## Exact proposed additional core interface

Keep the retained dependent `Std.Nat.induct` Sort0 prototype and all existing
Nat.rec/Bool interfaces. Add a distinct, fixed Boolean type selector:

```text
Bool.recType : Sort0 -> Sort0 -> Bool -> Sort0
Bool.recType A B false = A
Bool.recType A B true  = B
```

This is a proposed trusted computation rule selecting a type, not an ordinary
definition, a semantic axiom or an executable caller adapter. It is
monomorphic: no universe arguments or Sort1 result extension is allowed.
Its family must be Sort0 with exactly the generated false/true constructors,
in canonical order with their exact types. Validate the full eliminator type
before registering its equations. Close the family against later constructors.
Reject actual neutral/partial/prior opaque/prior theorem uses carrying universe
arguments; ignore unused term-table nodes. Registration and reduction implement
the same interface independently in the two private checker mirrors.

The proposal changes no production source, Certificate v0 byte format, axiom
category, legacy equation, eta/proof-irrelevance rule or normalization budget.
Exact normative ownership is proposed as CORE_BOOL_TYPE_CASES_V1.md, alongside
CORE_NAT_INDUCTION_V1.md. Both still need the complete W04 review/freeze and
T02 production integration before use by a public Go profile.

## Ordinary definitions and checked statements

The producer defines ordinary Nat predecessor, addition and truncated
subtraction, plus a Boolean less-than predicate with these equations:

```text
Less(m, zero)            = false
Less(zero, succ(n))      = true
Less(succ(m), succ(n))   = Less(m, n)
Lt(m,n)                 = Eq Bool (Less(m,n)) true
```

The general checked strong-induction statement is:

```text
(P : Nat -> Sort0) ->
((n : Nat) -> ((m : Nat) -> Lt(m,n) -> P(m)) -> P(n)) ->
(n : Nat) -> P(n)
```

P and n are arbitrary. This is not a finite unrolling, a Boolean-only
predicate restriction or a bound on loop iterations. A checked split lemma
separates m<n from m=n under m<succ(n). The proof builds the history of
strictly smaller values by ordinary Nat.induct, then applies the supplied
step obligation. That obligation must be proved at an application boundary;
it cannot become an extra hidden function-entry assumption.

The small type selector supports an ordinary contradiction eliminator from
Eq Bool false true to an arbitrary Sort0 target, using Std.Eq.rewrite. It
does not construct that contradictory equality or add an axiom. The weakened
premise case is rejected. Named History definitions are retained for interface
readability; application binders expose their ordinary Pi shape because the
existing application inference deliberately performs beta/zeta, not delta
unfolding of function types.

Eight positive claims cover the defining order equations, irreflexivity,
general strong induction, both type-selection branches and contradiction
elimination. Eight altered candidates fail with CORE_TYPE_MISMATCH in both
checkers. These are candidate rejections, not certified counterexample proofs.
Predecessor/add/subtraction are checked definitions at this checkpoint;
additional arithmetic laws and complete loop-variant composition remain due.

## Scope and measured results

The interface matrix starts with only Bool and its two constructors. It
contains no Bool.cases or Nat.induct fallback that could supply the new
interface's guards. Four cases are accepted: both type branches, the unchanged
legacy prefix, and an unused term-table node. Sixteen malformed cases reject:
wrong/nongenerated interface, changed/nongenerated constructors, extra
constructors before/after registration, higher family/result universes,
actual partial/neutral universe arguments and prior checked uses.

The legacy Bool certificate, legacy Nat certificate and retained arbitrary-Nat
invariant certificate remain accepted, zero-axiom and hash-identical in the two
new mirrors. All 39 cases are replayed after a fresh build with identical
candidate and checker-binary identities. The report pins 143 source inputs
and 77 mirrored checker files; those counts describe this private construction,
not the separate predecessor migration inventory.

Observed maxima are 4,596 canonical bytes, 464 stored terms and35 declarations.
Rust's slowest case was 0.009855s and Go's 0.337304s in this local sample.
The 60s child setting is private, not a normative profile limit or an
application-cost guarantee. The whole application resource freeze is unchanged.

Selected regressions additionally pass seven existing Rust Bool-case API
tests, seven retained Rust Nat.induct tests, four existing Go Bool-case API
tests and five retained Go Nat.induct tests. Twenty-one report/adversarial
tests preserve scope, identity, independent checker coverage, zero-axiom
acceptance, resource/failure classifications and deferred work.
Unchanged production suites and earlier collection proof matrices are not rerun.
The [verification record](verification/verification.json) retains this selection.

## Retained construction diagnostics

Two intended-positive failures are retained separately in
[the initial diagnostic package](../nat-type-selection-initial/observations.json)
and [the History-alias diagnostic](../nat-type-selection-initial/history-alias-observations.json).
The initial Nat type-recursion attempt left a computed Bool-to-Nat major
neutral under the retained shallow iota reducer. Direct Bool type selection
avoids a global normalization change. The second attempt used a named function
type where application inference requires an exposed Pi; expanding that binder
shape corrected the ordinary proof. Both failures have exact source/candidate/
checker identities and are excluded from semantic-negative and acceptance counts.

Complete source loop/variant/range/word64 binding, component-call termination,
typed overlay/wire/hash/limit/migration freeze and public release remain due.
The [operation/loop decisions](../operation-loop-decisions.md) still require
every actual head/test/body/post/abrupt/frame obligation. W04 stays In progress;
W05 owns inactive publication, feasibility review and the cumulative T01 gate.
