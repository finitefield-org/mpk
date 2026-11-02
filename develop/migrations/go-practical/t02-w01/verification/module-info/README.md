# General Go module cache info validation: in progress

The original module-data reader now uses the same Go cache-info validator as
the analysis frontend owner. It checks the original `.info` version, time and
Origin field types and Go's strict JSON rules. It accepts Go's valid folded
field names and preserves unknown fields. Previously the general reader checked
only an exactly spelled `Version` and could accept a malformed `Time` or
`Origin`. The validator remains a metadata consistency check; VCS and original
root authority are not inferred.

The [Go observations](native/result.json) use the exact cached `.info` from the
original versioned application capture. Go and Rust agree on 14 cases: seven
accept and seven reject. The [Rust application test](generic-tests/result.json)
rebinds each case through the complete snapshot, capture, export and module
records, preserving all 5,490 original source objects and the original single
Root contract. It checks unchanged original inputs as well as lowercase/Unicode
field names, wrong versions, malformed times and Origin values, and opaque
metadata. The two original local/versioned owners and the existing eleven
rebound module-data controls also pass.

The [independent inspection](inspection.json) rehashes all 22,323 files in
both original application/SDK input owners, recomputes each changed `.info`
identity and compares every Go observation with the Rust application result.
The direct analysis-info, module, namespace and archive consumers, scoped
Clippy, Go vet and changed-file formats pass. The original raw sources and
previous receipts remain intact; no application package or proof is replaced.

This remains an intermediate T02-W01 checkpoint. The complete capture/context
authority, source-to-binary binding and application proofs remain due, and the
practical Go profile remains inactive. Go ran under local Linux emulation, not
native Linux release acceptance. The cumulative T02 gate `./scripts/check-fast.sh`
is deferred to the final T02-W04.
