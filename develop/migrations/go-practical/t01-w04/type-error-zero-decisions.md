# W04 root values, closed errors and empty slices

Status: draft decisions for the inactive Go practical v1 freeze. These rules
resolve the first semantic questions in the [worklist](freeze-worklist.json).
They do not activate a frontend, change a predecessor schema, or establish
source-to-certificate correctness. The full W04 package still needs its
operation/overlay/wire tables, numeric limits, migration review and freeze.

The [source vectors](source-vectors.json) distinguish native Go facts from
future profile expectations. The
[native receipt](scope-observations/observations.json) records original,
unmodified snippets type-checked with Go 1.25.0 and gc/amd64 sizes, plus
host-runtime observations. Its observer uses reflection, recovery and a
global counter outside verified code. The native observer is not a practical
admission executor or a Linux release gate.

## 1. Root values and backing cells

These are two recursive admissibility predicates over an otherwise admitted
Go type, not two different Go types. Nominal identities, Go assignment and
conversion rules, field order and all representation bounds remain intact.
Aliases preserve their resolved target identity. Every type graph must close
before either predicate is accepted.

| Rule | Meaning |
| --- | --- |
| TG01 | A plain value has no nested mutable reference. Admitted Bool/integer/string values are plain. A fixed array of plain values or a struct whose fields are all plain is plain. Named types and aliases use their underlying shape for this predicate while retaining nominal identity. |
| TG02 | A root value is a plain value, a slice of plain values, a fixed array of root values, or a struct whose fields are all root values. The closed-error rule below adds a finite sum with root payloads. |
| TG03 | A slice backing cell must be plain. Thus a root struct with a slice field and a root fixed array of slices are permitted. A slice of such structs, a slice of arrays containing slices, and a slice of slices are excluded. Taking a view of a root array is permitted only if the resulting slice element type is plain. |
| TG04 | Copying a root aggregate copies its value fields and slice descriptors. It preserves every descriptor's backing association. Rebinding a copied field changes that field alone. Updating through a copied descriptor still requires the backing region's permission and shares its store with every local alias. |
| TG05 | Build the expanded type graph, including every closed-error payload edge, and reject a cycle. A struct with an error field whose closed universe contains that struct is recursive even when a particular constructor contains nil. A recursive slice element cannot be admitted by stopping traversal at the slice node. |
| TG06 | Input capture/binding traverses every descriptor path through root structs, arrays and error payloads. It does not allocate a new physical region for each field or parameter. Actual overlap, capacity-visible tails and typed backing are preserved. |
| TG07 | No new pointer, map, arbitrary interface, generic, floating-point or unsafe type is admitted by these predicates. Original Go type errors remain errors even outside the selected body closure. The complete source-admission and structural-budget tables remain separate freeze deliverables. |

For example, `struct { Lines []Line; Groups [2][]Line }` is a useful root
when `Line` is plain. A `[]Line` cell can contain an amount, a bounded byte
string and a fixed array of Bool values. If `Line` instead contains a slice,
`[]Line` is excluded in v1. A root's slice field does not grant permission
to mutate caller-owned backing.

Primary execution: `practical_types.go`, `practical_slices.go` and
`practical_construction.go` under `go-tools/go2vir/`, with the correspondingly
named planned test files. Independent type/operation/VC validation belongs
to `go_practical_artifacts.rs` and `go_practical_vc.rs`. These planned
files are listed in the [surface inventory](surface-inventory.json).
Required vectors include `root-slice-aggregate`,
`root-fixed-array-of-slices`, `plain-struct-slice-cell`,
`nested-struct-slice-cell`, `nested-array-slice-cell` and
`recursive-slice-element`.

## 2. A closed error universe

An error value is `NilInterface | ConcreteError(type_id, payload)`.
Use one closed, captured dynamic-type universe U for a verification unit,
including its checked callees. The same Go error type does not silently acquire
different universes at different projections. Imported proofs must agree on
U or carry a checked, explicit universe embedding.

| Rule | Meaning |
| --- | --- |
| ER01 | Only the predeclared error interface and aliases preserving that identity use this special carrier. Other named or anonymous interfaces, including interfaces with an Error method, are outside v1. |
| ER02 | U is the union of all concrete types boxed by reachable bodies/return paths and the finite sets explicitly declared for error-bearing entry values. Entries include nested root fields and slice cells. An unconstrained error-bearing input is a configuration rejection. An explicitly empty entry set permits nil only. The resulting restriction is a visible entry predicate. |
| ER03 | Each non-nil case is a captured, application concrete named Bool, integer or struct value with the exact Go Error() string method. Pointer, string, array and interface concrete cases are excluded. Aliases resolve to their target case; they never create a second dynamic tag. All payloads must be root values, and the expanded graph must be acyclic. |
| ER04 | Error<U> is plain exactly when every concrete payload in U is plain. A root error may therefore carry a struct containing a slice descriptor. A []error may be admitted only when the complete U has plain payloads. It cannot silently choose a smaller universe to hide a reference-valued case. |
| ER05 | Boxing and copying preserve the concrete tag, complete payload and any root descriptor associations. A concrete zero value remains non-nil. NilInterface has no concrete tag. Signature checking for boxing is a type dependency and never evaluates Error's body. |
| ER06 | A concrete value's direct Error() call includes that exact method body in the verification closure. An error.Error() invocation is permitted only for closed U and expands to its finite, captured callee table. Include every possible target body, signature, contract and effect dependency. No package-name allowlist or caller-supplied message oracle substitutes for those bodies. |
| ER07 | An interface invocation evaluates its receiver once and selects one target from that same tagged value. It has a non-nil safety obligation and the selected callee's precondition/result/effect obligations. A missing nil guard can leave safety unproved; it is not an original Go type error. No requested obligation is dropped. |
| ER08 | Reaching a global read/write or another excluded operation in any target body rejects with the root-to-target path. A boxing-only use may leave the same method body type-check-only. An uncertified branch guess cannot remove a target from U or the body closure. |
| ER09 | Admit error == nil and error != nil with a resolved untyped nil operand, including parentheses. Error-to-error equality is excluded even when U happens to have comparable payloads. Concrete admitted scalar comparisons retain their ordinary Go rules. Method values, type assertions/switches, errors.Is/As/New and fmt.Errorf remain excluded. |
| ER10 | Pointer concrete errors remain an explicit v1 rejection. In particular, an interface containing a typed nil pointer cannot be replaced by NilInterface. Open dispatch never becomes closed merely because one observed run used one type. |

The Error bodies in the native observer intentionally include an incrementing
counter and a noncomparable slice payload. The retained facts show that boxing
does not increment the counter, invocation does, concrete zero is non-nil,
typed nil is non-nil, and comparing the noncomparable error to itself panics.
These are source semantics, not permission to use observer effects in a
verified function.

Primary execution: `practical_errors.go`, with closure handling in
`practical_closure.go` and joint call binding in `practical_calls.go`.
Tests must distinguish `boxing-signature-only` from
`closed-error-body-global-effect`, direct and interface calls, guarded and
unguarded nil invocation, root versus cell payloads, recursive payloads,
unclosed inputs, wrong Error signatures and typed-nil pointer errors.

## 3. Non-nil slices with capacity zero

Keep the descriptor shape
`(nil_tag, typed_region, offset, length, capacity)` and the live-region
validity rule. Zero capacity is a special observational binding case, not a
new writable region or a relaxation of an index check.

| Rule | Meaning |
| --- | --- |
| ZS01 | A nil descriptor has length/capacity zero and no accessible region. A non-nil descriptor with capacity zero also has length zero, but its non-nil tag is retained by copies, empty append and zero reslicing. |
| ZS02 | For capacity > 0, retain actual typed backing, offsets, sharing and every cell that a permitted reslice can expose, including when length is zero. Do not canonicalize a positive-capacity empty slice into the capacity-zero case. |
| ZS03 | For a non-nil capacity-zero descriptor, source physical region and offset are unobservable in v1. Bind it to offset zero of the first live slot of its exact cell type. If no slot is live, initialize slot zero as a live, zero-length typed witness with canonical padding. This maintains non-nil live-region validity. It is a model representative, not a claim that source allocations share addresses. |
| ZS04 | Such a representative has NoAccess permission. It grants no read, write, freshness, separation or ownership-transfer fact. It need not consume an additional physical backing slot when other typed slots are already live. Later allocation may replace an unused zero-length witness; all capacity-zero descriptors remain valid and NoAccess. Accessible backing must never be overwritten. |
| ZS05 | No v1 source/overlay operation observes a descriptor's numeric region ID, physical pointer or absolute offset, or asks whether two capacity-zero slices have the same backing. Those fields remain internal validity/binding data. Root copies and error payloads cannot create a new observation. |
| ZS06 | Indexing a capacity-zero slice has no safe index. Two-index reslicing permits only 0:0; three-index reslicing permits only 0:0:0. A zero-length range executes no body iteration. The safety obligations still exist; canonicalization cannot turn a failed bound into success. |
| ZS07 | copy uses min(destination length, source length). A zero count leaves every backing cell unchanged and needs no write permission. Empty append retains the descriptor and nil tag. A positive-count write still requires writable storage, even for a slice whose old length was zero. |
| ZS08 | Positive append to capacity zero succeeds in ordinary Go by growing storage. It cannot satisfy v1's no-growth obligation on that path. Report the profile-domain obligation as pending/failed according to evidence; do not invent a Go panic or change the input into a positive-capacity descriptor. |
| ZS09 | Numeric typed-slot counts, physical-storage/domain bounds and their allocation/entry enforcement are still part of the separate limits freeze. Its rule must count accessible physical backing and preserve aliases across all root paths. The zero-access representative does not justify any hidden disjointness or capacity assumption. |

For the admitted operations, a capacity-zero slice can reveal its nil tag
and zero length/capacity, preserve them, perform zero writes, or fail an
actual access bound. No admitted operation can reach a cell through it.
This is why ZS03 can change the model representative. Production binding
must prove that these facts compose through each admitted operation and
callee; native examples alone do not prove the normalization theorem.

The native runtime compares nil, make(0,0), a zero-capacity prefix view and
a zero-capacity tail view. It also checks zero-count copies in both directions
and an empty slice whose positive capacity exposes the whole shared tail.
The observer's alias writes are observations outside the verified root;
borrowed-input writes remain forbidden.

Primary execution: `practical_slices.go` and `practical_construction.go`.
Foundation members must establish zero-access binding, descriptor validity,
zero-count frame preservation and positive-capacity tail/alias preservation.
Complete operation/application proofs remain due in T02–T06. The
[W03 descriptor/construction evidence](../t01-w03/feasibility-exit-audit.md)
is a foundation feasibility prerequisite, not the final source binding.

## 4. Review and remaining freeze work

The [native report checker](../../../probes/go-practical/check_scope_report.py)
checks intact source bytes, selected native call/type facts, target sizes,
the complete independently specified runtime result, source/binary/toolchain
identities and scope. Adversarial cases check that hashes, type errors,
aliasing, nil tags, call kinds or pending production work cannot be hidden.
The draft profile outcomes in the vectors remain expectations for planned
executors; the checker does not report them as observed admissions.

Full source syntax and operation/check order, typed contract/loop expressions,
all structural/capture/proof limits, successor schema/hash preimages and
reviewed consumer migrations remain W04 work. W05 owns inactive normative
publication, source-model feasibility review and the complete T01 gate.
No public selection, production parser, kernel or installed registry changes
at this checkpoint.
