# Ordinary full-width word definitions and source value specializations

Status: private W04 ordinary foundation component. Entry:
`ee1083c14e7050e08a411f0fcda7e1d78ec1abba`. W04 and the full T02–T11 objective
remain open. This component supplies actual ordinary definitions and checked
basic laws; complete Go theorem-request elaboration and universal source/Nat
correspondence remain required.

The [catalogue](schema.json) fixes the carrier, operations, exact source
specialization boundary and production owners. The complete
[ordinary expansion](expansion.json) uses the preceding full-bit Bool-indexed
carriers for8/16/32/64-bit words. Address arguments are most significant first;
integer bit0 is the least significant bit. All source values retain every bit,
including values outside JSON's safe-integer range. The
[generator](../../../../probes/go-practical/word_foundation.py) uses ordinary
const/var/app/pi/lam terms, Bool.cases and Eq.congr/trans/symm. There is no
arithmetic opcode, new axiom, checker reduction, function extensionality or
product eta rule.

The definitions cover Boolean bit circuits, bitwise operations, wrapping
addition/subtraction/negation/multiplication, signed/unsigned comparisons,
three shifts, division/remainder and all width conversions. Shared carry
definitions and balanced selector/predicate trees keep the graph bounded.
Multiplication composes shift/add stages. Restoring unsigned division keeps
the high remainder bit in its comparison, accounting for the full shifted
value before modular subtraction. Signed division applies unsigned
magnitude division and the required signs. Minimum/-1 wraps to minimum and
its remainder is zero. Remainders follow the dividend's sign.

`DivideDefined(y)` requires a nonzero divisor. Division/remainder functions
have explicit zero totalizers at y=0, which are model values with no Go
execution claim. Shifts consume the entire unsigned64-bit count. Counts at
or above the width produce zero, or sign fill for signed right shift.
Signed source counts first sign-extend to64 and require
`SignedCountDefined`; negative counts cannot borrow the unsigned execution
case. Go integer constants must pass original source representability checks
before literal bits are constructed. Runtime conversions use the source
signedness and low target bits, retaining Go's widening/narrowing behavior.

The [source specializer](../../../../probes/go-practical/word_bindings.py)
independently reconstructs each complete captured operation definition before
selecting a circuit. The [fixture](bindings.json) has13 distinct nominal/builtin
source word types and787 operations across GP-OP-01/02/03/04/05/06/07/09. Each
has two concrete definitions: Value and Defined. Operation hashes, source
scope, full context and ordered input/output source IDs remain bound. Equal
core carrier widths do not erase Amount/OtherAmount or int/int64 identities.
Every supported source count type participates in both shifts; all source
word pairs participate in conversions. A full hash prefixed with H forms
each canonical declaration-name component. Caller formulas, extra flags,
changed source IDs or proof-status overrides reject against reconstruction.

These specializations are pure value and arithmetic-definedness definitions.
They do not discharge the source/evaluation/input-validity/state/permission/
frame/control members of their parent operation. Compound assignment, memory
and whole-function elaboration still need their own definitions and proofs.
The prior169 Go theorem requests remain unelaborated; installing this component
cannot promote them or emit an empty successful Go skeleton.

The actual ordinary theorems include arbitrary-Bool parity/majority/equality
laws, named carry-zero lemmas for each word bit, and pointwise AddZero,
XorZero, AndOnes and NotNot for arbitrary word values at every address. They
do not enumerate every input word or assume extensionality. In the retained
checker, iota needs a direct constructor discriminant: a nested definition is
not evaluated automatically in that position. The untrusted proof constructor
therefore supplies explicit Eq.congr and transitivity steps. Each carry step
uses a named prior theorem, avoiding deeply nested evaluation proofs.

The [replay](../../../../probes/go-practical/run_word_foundation.py) reuses the
unchanged closed-carrier ordinary emitter and the same77-source joint private
Bool.recType/Nat.induct checker proposal. It copies/builds those sources in a
fresh temporary workspace and checks their hashes. Production core/kernel,
Certificate v0, preceding helpers and installed profiles stay unchanged.
Both independent checkers accept four word sets and the complete source-value
set with zero axioms and identical certificate/export/axiom hashes. The combined
set has2,756 declarations,19,718 terms and513,817 bytes:2,575 new definitions,
146 new theorems and35 retained seed declarations. It fits the preceding
2MiB/131,072-term/4,096-declaration candidate envelope.

Sixteen controls use the actual definitions and reject in both cores. Twelve
are deliberately false carry/add-identity/division-definedness claims; four
pass a Bool as a word shift count and fail ordinary type checking. These are
distinct from Go source errors, serialization errors or resource failures.
Each of the21 certificate cases is emitted twice byte identically. Reported
time/RSS covers one direct proof child on Darwin, not a Linux release or a
process-tree bound.

The [1,640 queries](queries.json) cover all widths, wrapping boundaries,
signed minima, high remainder bits, arbitrary byte patterns, fixed-seed values,
conversion signedness and full64-bit shift counts. The independent pinned
[Go observer](../../../../probes/go-practical/word_observer.go) runs actual Go
operators, catches actual runtime panics and runs twice identically. Its
[raw output](native.json) agrees with both the [actual ordinary-AST evaluation](model.json)
and a separate integer oracle:1,312 word results,288 Boolean results and40
runtime-safety failures. The AST interpreter executes emitted definitions;
it does not substitute high-level arithmetic operations. These finite
observations are comparison evidence, not universal source-correctness proofs.

The [receipt](observations.json) pins685 source/data inputs and the retained
14,496-file Go1.25.0 inventory. The [16 targeted tests](../../../../probes/go-practical/test_word_foundation.py)
cover nominal separation, source representability, sign-extended count guards,
division-by-zero guards, caller mutation, actual core controls and prior Go
elaboration boundaries. The [verification record](verification/verification.json)
retains raw commands/logs, reasons, exact initial failed probes and repairs.
Those failures include emitter/parser depth, duplicated binders, nested iota,
carry-proof argument composition and canonical declaration names. No failure
is reclassified as an application proof or a Go counterexample.

Universal full-width Nat/Int interpretation/conservation, general arithmetic/
division/source correspondence, remaining contract/operation/state/permission/
frame/control/totality definitions and complete Go statement/program/skeleton
assembly remain required. Frontend-envelope/policy/API/release roots and
reviewed migrations remain W04 work. W05 owns inactive publication and the
cumulative T01 gate. All T02–T11 implementation and application/release gates
remain active.

```sh
python3 -B develop/probes/go-practical/run_word_foundation.py --check
PYTHONPATH=develop/probes/go-practical python3 -B -m unittest -v test_word_foundation
python3 -B develop/probes/go-practical/check_word_metadata.py
```
