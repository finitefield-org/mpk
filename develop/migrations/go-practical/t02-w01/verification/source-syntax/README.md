# Closed original syntax and physical source origins

Entry: `11946e845d64e29f242ed8bc5d3477a4ac80dd02`. This is a preparatory
source-binding component of T02-W01; R06 and W01 remain pending.

The [private syntax image](../../../../../../go-tools/go2vir/practical_syntax.go)
reads only exact owned `go_file` objects from a sealed snapshot. Original
bytes remain intact after removal of the physical inputs. Parsing retains
complete original declarations and unsupported bodies, without stubs or
source rewriting. Missing, unread, wrong-role, corrupted, case-crossed or
escaping inputs fail the transaction; an ignored failure cannot lead to a
partial successful image. Repeated access reuses the same original tree.

Every token file uses its complete source-object ID. A capability containing
slashes cannot collide with another capability/path's joined spelling.
Physical node spans retain that source ID and original byte offsets. Adjusted
`//line` filenames/line numbers never replace physical origins. Raw-string
and comment carriage returns are recovered from the complete actual token,
including the ends inherited by enclosing declarations. Returned node bytes
are separately owned, including parsed-source views on first and cache reads.
Foreign/position-changed compiler nodes and identifiers
that differ from the original token reject. Trees remain untrusted compiler
inputs; these locators confer no nominal or semantic proof authority.

The syntax count includes every materialized comment group/comment, including
unattached comments skipped by `ast.Walk(File)`. Repeated Doc/Comment/Imports
edges count a node once. The practical4194304-node budget is separate from the
unchanged predecessor1000000-node budget. Exact remaining-budget acceptance,
duplicate-tree reuse and over-budget transactional failure are checked.

[The receipt](verification.json) binds12 selected roots/99 subtests in92.16s,
current vet/format and a Go1.23 compile/link check with zero tests. All81 frozen
original admission sources remain byte-exact:80 parse and the original syntax
error remains an error. The80 include three original Go type errors; syntax
success is not package type-check success or profile admission. Independent
reflection over every AST field confirms all2090 materialized distinct nodes.
Independent comparison with the frozen native facts preserves every1710
original function-node kind/start/end obligation. The separate physical
witness preserves all five comment tokens, Unicode identifiers, a CR-bearing
raw string and its full enclosing declaration under a forged line directive.

The independent receipt builder reconstructs253 complete source-object
preimages:81 original catalog sources, the physical witness and all171 current
frontend/vendor build inputs. Two actual independent empty-cache builds from
35 frontend/133 vendor files plus metadata produce identical8592898-byte
binaries, SHA256 `ddec23feecaedcdef80fc2c6f1942aecb7dfdceb0e7724dda933e359453e38a6`.
Full sources, metadata, raw outputs and all255 unchanged frozen inputs are
retained. The new syntax helper is private and does not activate a profile.

[The initial epoch](initial/regression/result.json) preserves its complete
source/raw failed budget assumption and the original missing-unattached-comment
path. [The complete-comment epoch](complete-comments/regression/result.json)
preserves a new test-only reflection observer that stopped at interface/pointer
aliases. After repairing the observer's traversal, every original case passes.
The [returned-source regression](source-view-before/ownership/result.json)
then reproduces a source-view/cache alias: mutating returned bytes changed
cached node evidence. First and cache views now return separate byte storage.
The [earlier passing component receipt](verification-before-source-return-ownership.json)
and its full11-root sources/outputs remain intact. All12 corrected roots and
two new actual fresh builds pass. None of these construction/observer/data-
ownership failures is a semantic source counterexample. All relevant Go/build
handles are terminal before staging.

Complete original package build/file selection, effective-language/module/
compiler/source/export ownership, original package type checking and
authoritative immutable package/export/selected-closure replay remain due.
Full bounded56-family source/carrier/nominal/type/operation/storage/lifetime/
state/program foundation, all six common members/19 theorem families, complete
compiler-derived operation/VC roots/private descriptor/registry/context/retained
policies and all169 application demands/T02-T11 remain required. Previous
ordinary proofs and unrelated native/archive/module matrices are retained
without rerunning them. C# tests stay skipped; the cumulative T02 gate remains
T02-W04: `./scripts/check-fast.sh --skip-csharp`. No package/profile/source-program
proof, application verification or native Linux release claim is made.
