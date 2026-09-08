# W09 internal unit 2: ordinal UTF-16 comparison and search

Correction baseline: `31d69a3`. This is a component review, not a W09 completion receipt.

The scalar coverage audit reopened this component: the previously accepted
predicate/value reducers did not satisfy the frozen `ordinary_core.folds`
requirement for static balanced ordered composition of concrete S->S
transformers. Passing kernel checks and semantic observations did not prove
that construction contract. The current implementation replaces those reducers;
the previous zero-finding conclusion is superseded.

## Implementation

Eight signatures now have ordinary definitions: equality and inequality
operators, static and instance ordinal Equals, ordinal Compare, Contains,
StartsWith and EndsWith. The original closed string signature determines the
input carrier, result type and ordered checks. Source/foundation metadata and
canonical bytes are independently regenerated on import.

The generated First.D0..D14 and Any.D0..D14 adapters use one shared ordinary
pipeline. State S is the concrete C6 cube containing a Word32 index and Word32
first nonzero result. The read-only C19 predicate and Word32 length are fixed
ordinary value arguments; the pipeline declaration closes all binders. There
are no type arguments, templates, recursive globals or new checker rules.

## Direct review

- Each StepTwo instance has concrete S->S type after fixing predicate and
  length. It visits index and index+1 in that order with fixed-width arithmetic;
  the second read contributes only when index+1 is below length. A completed
  input state is returned unchanged. A first nonzero result wins over the
  second. The pipeline starts at zero, so its even indices advance to at most 16384.
- GuardedCompose fixes the concrete length, applies f before g, shares the
  intermediate state with an ordinary Let, and skips g when the index is out
  of range or a nonzero result was found. The builder creates a balanced tree
  of 8192 explicit StepTwo occurrences and counts all of them before DAG
  sharing, plus 42 helper transformers. The existing inclusive 16384 limit
  remains enforced. A guarded composition test uses distinct f and g
  predicates to expose reversed order or wrong de Bruijn binder references.
- First.Dd widens the predicate to C19 by retaining low index selectors and
  all result selectors, and clamps the count to 2^d. Any.Dd encodes its Boolean
  predicate in bit zero of a Word32 and uses the same First pipeline. Thus it
  adds no second 8192-occurrence pipeline. Zero count returns zero/false;
  odd counts mask the extra read. Tests include count boundaries and the last
  address include/exclude pair. An empty needle bypasses its 16385 candidate
  count before applying the adapter.
- Char subtraction zero-extends both sixteen-bit code units to 32 bits before
  subtraction. The signed difference therefore covers -65,535 through 65,535,
  preserving NUL, high bits and isolated surrogates. Compare uses length
  difference only when the whole common prefix is equal.
- Static equality treats two null strings as equal and null/empty as different.
  Instance Equals fails on a null receiver. Compare orders null before present
  and returns -1/0/1 for the null cases; no null payload is needed for that result.
- A window compares the right text at index i with the left text at start+i.
  EndsWith uses the length difference as start; StartsWith uses zero. Contains
  folds over exactly receiver length - needle length + 1 candidate starts.
  An empty needle succeeds before that count is consumed, avoiding a 16,385th
  candidate address at maximum receiver length. A longer needle fails normally.
- Search checks null receiver before null argument. Failed Boolean results are
  false and failure flags are mutually exclusive. Input-domain assumptions remain
  explicit: invalid lengths/padding and construction of those domain predicates
  are subsequent W09 work, not silently certified by these definitions.
- Basic, construction and ordinal helpers initialize lazily with separate names.
  Shared cube helpers are defined only when absent. Construction's word emitter
  is reused with ordinal-specific identities. The existing basic/construction
  source certificate bytes are retained.
- Eight original source captures cover every new signature. Unlike the basic
  and construction cohorts, this cohort contains both depth-19 and depth-20
  text carriers. The shared source test now pins each cohort's actual depth set.
  The earlier Contains rejection becomes a positive source case; metadata and
  byte substitutions still reject, and unknown signatures remain fail-closed.
- The new source file is the sole new standard-namespace consumer: 104 becomes
  105, with the aggregate and historical inventory hash link updated together.
  Existing cache paths and inventory add/remove rejection behavior are retained.

## Verification

The C6 correction passed the 680-case actual-core oracle matrix, 40 address
boundary cases, 405 Any-fold cases, full 16384-unit equality, and nine direct
step/composition state cases. The final direct review has zero findings.

The combined certificates have 5360/5363 terms, 152 declarations and 8234
counted transformer occurrences. All ten ordinal certificates passed same-byte
zero-axiom checking and corrupted-hash rejection in both unchanged checkers.
Source replay covered the eight original ordinal captures and retained the
sixteen basic/construction captures and their bytes. Builder budget/order,
fixture replay, inventory, lint and format checks also passed. Exact commands
and artifact/log hashes are in `unit-2-string-ordinal-verification.json`.

The final scalar reconciliation is recorded in `unit-2-review.md` and
`unit-2-coverage-audit.json`; internal unit 2 is complete. W09 units 3-8 remain
outstanding. The full `check-fast.sh` gate remains deferred to T06-W12.
