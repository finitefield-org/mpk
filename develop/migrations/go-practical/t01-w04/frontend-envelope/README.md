# Common frontend request and process draft

This W04 component closes a private draft for `mpk.frontend.request.v3`,
`mpk.frontend.success.v3`, `mpk.frontend.diagnostic.v3` and the shared
`mpk.frontend.cli.v3` protocol tag. [schema.json](schema.json) owns the exact
field sets, domains, snapshots, transport limits and new Go diagnostic codebook.
It does not activate a profile or invoke a frontend. W04 remains In progress.

The request binds an independently validated semantic request, available
immutable input snapshot and exact independent release reference. It can be
formed before lowering. A response binds its complete context, selection,
profile contract and request reference. Each hash includes every root field
except its own hash, under its separate domain and NUL separator. The success
IR wrapper references the complete validated common VIR hash; source map and
frontend manifest are mandatory. A diagnostic has no artifact fields. Unknown
fields, including caller `ready`, `proved` or `registered_execution`, reject.
[requests.json](requests.json) and [successes.json](successes.json) retain all
eight source-bound fixture pairs.

The retained snapshot uses exact kind/path/size/raw-hash input entries and the
unchanged `MPK-INPUT-SET-0.1` hash. Its success manifest selects an exact subset
of the available immutable buffers. Unused staged bytes remain bound by the
request but need not enter the selected closure. The Go snapshot instead uses
the existing capability/path/kind/bytes/raw-hash source objects. A returned
capture-context ID is not needed to make the request. Successful Go model
artifacts still pass the complete independent captured ProjectSpec validator;
their actual read set must be a subset of the request snapshot.

The new observer links unchanged public `mpk-cli` and `mpk-vc` APIs. It validates
retained context/selection through the actual registry validators, projects
common source artifacts into the retained family and calls
`validate_successor_frontend_process`. Whole regenerated source-map and
manifest values and the VIR content hash match the five independent retained
Go/Rust64/Rust32/scalar-C#/Java baselines. A sixth successful retained bundle
preserves an explicitly authorized synthetic origin. Existing codebooks,
path checks, issue ordering and process failure precedence remain owned by
that API. C#'s existing generic-code behavior is retained alongside its closed
prefixed codebook; Java keeps its independently closed codebook. These fixture
comparisons do not constitute a general producer/consumer migration proof.

The common envelope uses the already retained narrow canonicalizer with depth
768. It adds no extra 256-depth precheck around source terms. The projected
retained envelope still passes its actual native limits, including depth256,
node and string limits. One canonical UTF8 JSON object plus exactly one LF is
required. Duplicate keys, invalid UTF8, BOM, alternative whitespace, trailing
values and unknown schemas reject. Actual compiled stdout/stderr constants and
inclusive boundaries are observed, including real 256MiB and 256MiB+1 stdout
buffers. Such resource/process failures remain separate from source rejection
and from a proof verdict. Stderr is a trusted byte count and never public prose.

New Go diagnostics use a fixed closed code/status/phase/message table. Optional
spans resolve exact captured source-object IDs with integer byte bounds,
including zero-length EOF. This avoids ambiguity when two module capabilities
have the same path, and remains meaningful for invalid UTF8 source. An optional
function declaration must resolve in the independent graph and agree with the
span's source object. Public messages contain no raw compiler prose or host
paths. These are model diagnostics, not observations that Go rejected a file.
Both successful and diagnostic Go fixtures explicitly return unregistered
model classifications. Request hashes and literal release references do not
authenticate a registered runner or prove lowering correctness.

[vectors.json](vectors.json), [outcomes.json](outcomes.json),
[native-queries.json](native-queries.json), [limits.json](limits.json) and
[preservation.json](preservation.json) retain the closed conformance matrix and
actual API queries. Both native runs reproduce byte for byte. Complete request,
success and diagnostic preimages, Unicode/depth/duplicate/integer/domain
boundaries agree across Python, Rust and the pinned independent Go observer.
[observations.json](observations.json) pins source inputs and native binaries.
The [verification record](verification/verification.json) retains selected
local commands, initial findings, fixes, outcomes and review.

The retained run covers611 cases,142 actual public-API queries and165 complete
Python/Rust/Go hash vectors. All18 targeted tests pass. It compares five
independent retained context/target baselines,six retained success bundles and
two Go model bundles. The new Go codebook has33 entries with every allowed
phase represented. The source closure contains810 inputs,including310 linked
public consumer/build sources. Initial query-cache coverage,wrong private
context generation,runner syntax and unobserved-query exception findings are
retained separately; none is counted as a source or ordinary proof rejection.

Run only the affected component and its metadata closure:

```sh
python3 -B develop/probes/go-practical/run_frontend_wire.py --check
PYTHONPATH=develop/probes/go-practical python3 -B -m unittest test_frontend_wire -v
python3 -B develop/probes/go-practical/check_frontend_metadata.py
```

The unchanged arithmetic predecessor and all143 inventoried predecessors stay
pinned; their unaffected native matrices are not rerun. Production code,
installed profiles and old helpers remain unchanged. All169 complete Go
theorem requests remain unelaborated. Other word/source/partial/definedness,
elaborator/state/permission/frame/control/totality proofs, complete Go program
and skeleton assembly, policy/API/release roots and reviewed migrations remain
due. W05 owns inactive publication and the cumulative T01 gate. All T02–T11
implementation, application and local Linux release work remains active.
