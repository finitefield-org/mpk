# go2gir

`go2gir` is the untrusted Go frontend command that will convert the accepted Go
subset into GIR.

GO-007 adds canonical GIR emission and deterministic GIR hashing. The command
accepts one package path, loads it through `golang.org/x/tools/go/packages` with
pinned settings, rejects features outside Go subset v0 with an exact JSON
report, builds SSA with `golang.org/x/tools/go/ssa`, and emits deterministic
JSON containing the package summary, source-function SSA dump, and GIR for
supported pure functions including struct field reads and fixed-array indexing.
The `gir_emit` payload contains canonical GIR JSON, a stable binary payload, and
the SHA-256 GIR hash recorded on the GIR module. The `source_manifest` payload
records normalized source paths, source SHA-256 hashes, the aggregate source
hash, Go version, and frontend binary hash for audit traceability.

GO-010 adds `mpk.go.contract.v0` sidecar loading. `go2gir` scans each loaded Go
package directory for `contract.json`, `*.contract.json`, and `*_contract.json`,
validates contract JSON fail-closed, resolves the contract `function` identity to
a lowered GIR function, and attaches accepted `requires`, `ensures`, empty
`modifies`, and `loops` metadata to the GIR function before canonical emission.

```sh
go run . ./testdata/samplepkg
```
