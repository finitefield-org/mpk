# MPK fuzz targets

Run the fuzz targets with cargo-fuzz:

```sh
cargo fuzz run certificate_decoder
cargo fuzz run theory_certificates
cargo fuzz run vir_parser
cargo fuzz run source_map_parser
cargo fuzz run frontend_protocol
cargo fuzz run source_manifest_parser
cargo fuzz run vc_parser
cargo fuzz run policy_v1
```

For bounded local smoke runs without the cargo-fuzz subcommand:

```sh
cargo run --manifest-path fuzz/Cargo.toml --bin certificate_decoder -- -runs=256
cargo run --manifest-path fuzz/Cargo.toml --bin theory_certificates -- -runs=256
cargo run --manifest-path fuzz/Cargo.toml --bin vir_parser -- -runs=256
cargo run --manifest-path fuzz/Cargo.toml --bin source_map_parser -- -runs=256
cargo run --manifest-path fuzz/Cargo.toml --bin frontend_protocol -- -runs=256
cargo run --manifest-path fuzz/Cargo.toml --bin source_manifest_parser -- -runs=256
cargo run --manifest-path fuzz/Cargo.toml --bin vc_parser -- -runs=256
cargo run --manifest-path fuzz/Cargo.toml --bin policy_v1 -- -runs=256
```

These root targets are parser-only and cap arbitrary input before calling a
public/shared importer. The isolated private driver and contract targets live
under `rust-tools/rust2vir/fuzz`; their bounded acceptance gate is
`scripts/check-fuzz-smoke.sh`. Unbounded `cargo fuzz run` sessions are local
diagnostics only: elapsed time and local artifacts are never acceptance data.
