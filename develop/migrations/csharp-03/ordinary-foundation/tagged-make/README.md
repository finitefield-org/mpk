# Ordinary tagged contract construction (partial W09)

`tagged_make` reuses the existing structural sum constructors for Option,
Lookup, Result, Validation and BoundaryField. The recipe checks the exact
semantic instance/result identity, admitted template, arm, payload arity and
nominal payload type. An absent payload must retain the canonical null recipe
parameter. A present payload is a typed argument, including when its physical
carrier is a single Bool. Unknown or mismatched parameters reject.

Validation's error sequence retains its role-bound metadata and the complete
underlying sequence carrier. This recipe performs no new normalization,
truncation, admission or exception conversion. Existing public-domain and
use-point obligations remain with their owners; storage definitions alone do
not prove that a constructed value belongs to an admitted public domain.

One freshly captured source context binds all five sum families and a nested
Result whose success payload is Option. Thirteen original method-contract
expressions exercise every arm and nested absent/present values. Twelve
distinct ordinary recipe aliases are checked against the precise existing
constructor globals, including complete dependency-closure equality. Three
payload samples per recipe supply 2,406 full small-carrier or selected large
carrier bit observations, covering tags, padding and payload storage. The
large Validation carrier is not exhaustively evaluated here; its complete
constructor dependency closure matches the existing structural implementation.

`../unit-4-tagged-make-progress.json` records scoped verification, including
the separately reported same-byte checker gate. Existing tag predicates and
partial reads retain their prior pinned bytes. Source/context/metadata import
continues to use exact reconstruction. These helpers do not discharge
application VCs or complete an original internal unit/W09. The whole gate
remains deferred to T06-W12, and no component-only commit/push is made.

The component's targeted checks passed. Both unchanged checkers accept the
same 35,202-byte certificate with zero axioms and matching reports, and reject
actual hash corruption. The retained candidate, metadata and tested output
bytes agree. Existing consumers, inventory and final affected lint/format
checks passed; the final component review has no actionable findings.
