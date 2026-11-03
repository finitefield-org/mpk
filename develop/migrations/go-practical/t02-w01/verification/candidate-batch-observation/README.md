# Native batch observation

An exploratory local test compared a single Go 1.25 `list -deps -export`
observation with separate observations for each of three selected packages.
It kept the original HTTP application, complete dependency/module data,
registered SDK, 183-package closure, 5,517 source objects and 182 exports.
Every separate observation retained its own actual selected-root metadata and
was rejected as an observation of the complete candidate request.

The first comparison failed because its complete observation used an empty
build cache and the later observations used a populated cache: the original
`example.com/c` row had different `Stale` and `StaleReason` fields. The test
was extended to retain both native observations and compare the complete
request once cold and once warm before comparing warm single-package runs.
The rerun kept the same source universe, offline compiler and command limits.
All 182 export-bearing packages changed stale state between cold and warm:
142 reported a stale `internal/goarch` dependency, 30 a build-ID mismatch,
and five each a stale `internal/cpu` or `internal/byteorder` dependency.

With that state difference retained rather than relabeled, the warm individual
observations covered all 183 packages. Their complete metadata agreed with the
warm whole observation apart from the independently checked `Match` and
`DepOnly` selection roles, and all 182 compiler export byte streams were
identical. Native diagnostics also agreed. This is evidence for this original
input and cache state, not a general equivalence theorem or a batching
implementation. The original source objects and complete cold, warm and batch
outputs were kept in a temporary verification directory; no archive is
committed.

The test was selected to determine whether command batching can preserve the
same original sources and metadata when argument vectors exceed the operating
system limit. A correct implementation still needs ownership and independent
Rust validation of every actual command, its cache-sensitive state, aggregate
resource bounds, selected roots and errors. Literal `...` import paths and
native argument-size handling remain open. W01 remains incomplete; the T02
`./scripts/check-fast.sh` gate is deferred to final T02-W04. The Go test ran
locally under Linux amd64 emulation, not the native Linux release gate.
