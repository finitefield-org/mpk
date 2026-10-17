# Ordinary unbounded arithmetic and closed logical contract components

Status: private GO-PRAC-T01-W04 component. Entry:
`61c8569f7ad0b3b6e3f31089f90b9432b9a11046`. W04 remains open. This adds
ordinary mathematical definitions and checked library theorems,then constructs
typed values and unproved propositions from27 existing normalized logical
contract heads. It supplies no complete Go source or application proof.

The [catalogue](schema.json) records the exact boundary and planned production
owners. The [complete expansion](expansion.json) contains599 new ordinary
declarations:36 definitions and563 theorems,including small named equality
checkpoints. The [producer](../../../../probes/go-practical/ordinary_arithmetic.py)
starts with the unchanged35-declaration [Nat induction seed](../nat-termination/type-select-zero-valid.mpcert).
Each certificate rechecks that seed. The production core/kernel,Certificate v0,
prior helpers and semantics,private77-source checker proposal and emitter,
parser depth and conversion budgets remain unchanged.

Nat is genuine unbounded Std.Nat. Addition,multiplication and truncated
subtraction are recursive ordinary definitions; equality and order use Nat
induction into Bool. Ordinary proofs establish arithmetic computation rules,
addition/multiplication associativity and commutativity,distributivity,
cancellation,equality reflection and translation,order translation and the
subtraction laws needed by half-open ranges. These quantify arbitrary naturals
and do not enumerate a bounded set of machine integers.

The [integer layer](../../../../probes/go-practical/ordinary_integers.py) uses
Bool->Nat to represent p-n. Representations may be redundant:5-3 and2-0 denote
the same integer. Semantic Equiv is genuine Nat equality of cross sums.
Core equality of the pair functions never serves as integer equality. Checked
theorems establish equivalence,ring laws,operation congruence and representative
independence of Boolean equality and order. No normalization assumption,
function extensionality or extra integer axiom enters the proof.

The [range layer](../../../../probes/go-practical/ordinary_ranges.py) defines
forall,exists,Nat sum and Int sum by unbounded Nat induction over the count
upper-lower with truncated subtraction. The interval is [lower,upper).
Zero/step rules and empty rules cover upper<=lower,including reversed bounds.
Each fold has a checked preservation theorem whose premise is the actual
bounded FoldForall of element equality over the visited prefix. Integer sums
use semantic Equiv. No equality outside that prefix becomes an extra premise.
These are logical total-body folds; partial source expressions and their
bounded definedness obligations remain separate work.

The [closed component compiler](../../../../probes/go-practical/logical_elaboration.py)
uses the existing normalized contract wire vocabulary. It independently checks
type arms,payload fields,arity,operand/result types and lexical Nat binder depth.
It handles Bool/Nat/Int literals and operations,comparisons,typed conditionals,
bounded quantifiers and sums. Every [component](compilations.json) is completely
reconstructed from its [input expression](expressions.json). Exact decimal
literals become shared binary Nat declaration DAGs,with no machine-width or
JSON-number narrowing. Its private construction budget produces pending
status; it does not make a mathematical integer invalid.

The compiler emits Value and Defined=True definitions for this closed total
logical component and an Eq Bool Value true statement for Boolean clauses.
The statement's ordinary_proof remains null,even for a false condition. An
accepted Value definition establishes its type,not the truth of that clause.
Caller proof or source-status overrides fail reconstruction. The
[22 component cases](vectors.json) produce [one typed/unproved clause,19
configuration rejections and two explicit pending cases](outcomes.json).
Source,word and storage expressions remain outside this component.

The [replay](../../../../probes/go-practical/run_ordinary_arithmetic.py) rebuilds
the unchanged joint private checker and emitter in a fresh workspace. Two
positive sets pass Rust and Go independently with zero axioms and identical
certificate/export/axiom hashes. The theory set has634 declarations,11,424
terms and153,208 bytes. The combined set also typechecks all149 components,
including ten large integers. Eight altered proof/definition candidates fail
by paired core type mismatch:incorrect addition,unrelated equality/order
premises,positive-only integer equality,incomplete multiplication,false empty
forall,inclusive upper bounds and an invalid proof of a false clause.
Every one of the ten certificates emits twice byte identically. Resource or
format failure is never counted as a semantic negative.

The [independent pinned Go observer](../../../../probes/go-practical/arithmetic_observer.go)
executes math/big arithmetic and half-open folds for149 expressions,including
negative and redundant integers,nested binders,empty/reversed ranges and values
beyond2^64 and10^70. Two identical [native outputs](native.json) agree with an
independent Python integer oracle. The [actual ordinary-AST interpreter](model.json)
also agrees on139 expressions. Ten large cases reach its private512 unary-loop
ceiling and remain model_resource_pending. They still pass component
typechecking and independent native/integer comparison. This observation
ceiling is neither a bound on logical Nat/Int nor a failed ordinary proof.
Finite observations do not establish general compiler or source correctness.

The [receipt](observations.json) pins714 inputs and the retained14,496-file
Go1.25.0 inventory. The [16 targeted tests](../../../../probes/go-practical/test_ordinary_arithmetic.py)
exercise paired proof outcomes,all heads,complete reconstruction,integer
representatives,ranges and nested binders,large-value boundaries,false-clause
handling and the still-unelaborated169 complete Go requests. The
[verification record](verification/verification.json) retains local commands,
raw logs,selection reasons and the initial producer failure. A local variable
shadowed the negative-projection constructor; renaming it repairs generation.
The exact original inputs and traceback remain available. It produced no
certificate and made no source claim.

The connection between the preceding word denotations and this general
Nat/Int library remains required,as do remaining word/source arithmetic,
source/storage/partial expression semantics,bounded definedness,independent
elaborator correctness,state/permission/frame/control/totality proofs and
complete Go statement/program/skeleton assembly. Downstream envelope/policy/
API/release roots and reviewed migrations remain W04 work. W05 owns inactive
publication and the cumulative T01 gate; all T02–T11 work remains active.

```sh
python3 -B develop/probes/go-practical/run_ordinary_arithmetic.py --check
PYTHONPATH=develop/probes/go-practical python3 -B -m unittest -v test_ordinary_arithmetic
python3 -B develop/probes/go-practical/check_arithmetic_metadata.py
```
