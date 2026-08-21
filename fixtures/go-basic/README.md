# Go Basic Corpus

`fixtures/go-basic` is the GO-009 corpus for the Go subset v0 frontend.

The `positive/` fixtures are expected to lower through `go2vir` into VIR. They
cover the currently accepted pure-function subset: fixed-width integer
operations, boolean operations, local variables, `if` control flow, structs, and
fixed arrays.

The `negative/` fixtures are expected to be rejected with explicit subset-v0
reasons. They cover representative unsupported features from the spec, including
maps, pointers, generics, and strings.

`go-tools/go2vir` tests execute every entry in `manifest.json`, so this corpus is
covered by the repository CI path.
