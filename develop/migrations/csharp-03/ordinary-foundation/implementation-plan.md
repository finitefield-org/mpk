# CSHARP-03-T06-W09 implementation work plan (not a completion receipt)

Baseline: `1e1f6a4` (T06-W08). The canonical work-item ledger is unchanged;
W09 is not complete and W10 must not become ready.

## Current state

The predecessor assembly path builds and checks ordinary Certificate v0 values,
using the registered Bool/Eq/logic foundations. The practical assembly plan
retains context/VC/skeleton linkage and zero-axiom requirements. Its theorem
skeletons identify obligation groups but do not yet contain core propositions
or proofs. Concrete foundation entries contain substituted type IDs,
representation descriptions, equation strings and ordered errors. W02-W08
programs preserve typed formulas and source recipes, but many constants in those
formulas still need ordinary definitions. Existing feasibility probes demonstrate
Boolean-cube carrier/transformer type feasibility, not full foundation semantics.

Consequently, merely invoking the predecessor assembler, proving reflexivity
of opaque recipe hashes, replacing missing definitions with assumptions or
accepting finite runtime observations would not satisfy W09. All eight steps
below are within the existing W09 completion condition; none may be omitted.
They are internal work units, not new canonical task IDs. The user approved
implementing, directly reviewing, committing and pushing each unit separately.
The W09 completion condition still requires all eight units. This supersedes
the whole-W commit timing for this task only.

## Internal work units

1. **Ordinary term builder and complete concrete value layouts.** Generate
   actual Sort/Var/Const/App/Lam/Pi/Let definitions using checked Bool leaves,
   little-endian Boolean cubes, source stored-member order, active semantic
   sum payloads, fixed role bounds, zero padding, pointwise mux and concrete
   static transformer composition. Include scalar, source product/enum,
   exception, outcome, sequence, map/set, money and transition layouts.
   Validate exact source/descriptor/closed-instance reconstruction and reject
   every unresolved symbol, residual generic, cycle and exceeded limit.
   Test actual generated certificate bytes with both unchanged checkers.

2. **Scalar operations and semantic predicates.** Encode fixed-width integer,
   UTF-16, float, decimal, Guid, date/time/duration/instant operations and checks
   as finite ordinary circuits. Translate the specified exact rounding,
   overflow, NaN, zero, normalization and error behavior. Reuse T03 reference
   algorithms only as independent test oracles; runtime results cannot define
   or discharge a universal theorem. Test semantic boundary/counterexamples
   and actual term/declaration/binder/transformer costs.

3. **Structural and collection foundations.** Generate field/active-payload
   equality/order, complete source-snapshot observation, constructors/accessors,
   defaults/domains, construction state, bounded sequence operations, ordered
   map/set operations and validation/outcome operations. Expand every operation
   of each reachable concrete foundation instance, including uninvoked ones.
   Check inactive storage, non-reflexive types, bounds and frozen error order.

4. **Bindings, business values and boundary codecs.** Lower projections,
   reconstruction, operation/outcome commutation, money and canonical
   parse/format relations, strict JSON rules and typed literal bodies. Preserve
   all byte/value/source links. Independently test loss, three-state fields,
   canonicality, precision, cumulative limits and failed conversions.

5. **Native body and control relations.** Define source invocation/result,
   SSA/ownership state, constructor/field effects, loop cutpoints/decreases,
   pattern decisions, normal/exceptional postconditions, filter search and
   finally unwinding. Compile W02-W05 predicates and scopes to ordinary
   propositions; no native CFG observation may become a trusted axiom.

6. **Transition and replay relations.** Define admission, pure/total source,
   actual projected outcomes, state preservation, explicit time, checked
   versioning, ordered events/responses, retained-key lookup/uniqueness,
   complete snapshot encoding/equality, append/history preservation and frozen
   priority. Discharge W06-W08 groups from the same exact ordinary definitions.

7. **Proof and certificate assembly.** Derive each complete theorem type from
   the independently reconstructed VC groups and definitions; admit proof terms
   only against those types. Preserve all predecessor dependencies. Missing
   proofs/definitions, limits, checker execution failures and disagreements
   cannot produce an accepted candidate. Bind source/context/foundation/VC/
   skeleton/ordinary-program identities and enforce empty proof/theory tables,
   no axioms/theory primitives and recomputed zero axiom inventory.

8. **W09 acceptance and direct review loop.** Feed identical canonical bytes
   to both checkers, compare reports and hashes, reject proof/context/foundation/
   hash mutations, rerun the predecessor certificate corpus, and review/fix
   until no findings remain. Record actual semantic coverage and limits, then
   update W09/W10 status and commit/push the final internal unit. The repository-wide
   `check-fast.sh` remains deferred to T06-W12.

## Changes currently implemented (partial W09 only)

- The checker-agreement harness now sends one decoded byte sequence to both
  checkers, requires a verdict-bearing Rust JSON report with a matching byte
  hash, separates build/process/internal failures from proof rejection, and
  compares accepted module/declaration/axiom counts and all report hashes.
  Protocol and changed-report regression cases accompany this change.
- The practical Certificate v0 structural pre-check now measures actual term
  and declaration counts, composes binder depth across the term DAG, rejects
  forward/missing term references and enforces the inclusive practical limits.
  Boundary-value tests cover terms, lambda/Pi/Let depth and declaration excess.

These checks do not implement the missing ordinary definitions or proofs and
must not be presented as completion of W09.

## Approved execution status

Unit 1 is complete as an internal implementation unit. See `README.md`,
`unit-1-verification.json` and `unit-1-review.md` for its exact coverage and
local checks. Unit 2 has Boolean/integer, Time/Duration/Instant and
Date/Guid/DayOfWeek, floating-operation, numeric-conversion and decimal
conversion/rounding increments, documented in `unit-2-progress.md`; decimal
arithmetic/comparisons, UTF-16
string operations and units 3-8 remain outstanding. W09 remains Ready/in progress and W10-W12 remain
Blocked. No W09 completion receipt is issued by this unit.
