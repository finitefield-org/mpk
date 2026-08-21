# Go Alpha Corpus

`fixtures/go-alpha` is the ALPHA-001 corpus for the Go subset v0 frontend.

The corpus contains 100 small pure functions split across arithmetic, branch,
and fixed-array packages. Every package is expected to compile with `go test
./...` and lower through `go2vir` without rejected features.

`go-tools/go2vir` tests execute every entry in `manifest.json`, verify the
manifested function count, and require all 100 functions to lower to VIR.
