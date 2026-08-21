# MPK fuzz targets

Run the fuzz targets with cargo-fuzz:

```sh
cargo fuzz run certificate_decoder
cargo fuzz run theory_certificates
cargo fuzz run vir_parser
cargo fuzz run source_map_parser
```

For bounded local smoke runs without the cargo-fuzz subcommand:

```sh
cargo run --manifest-path fuzz/Cargo.toml --bin certificate_decoder -- -runs=256
cargo run --manifest-path fuzz/Cargo.toml --bin theory_certificates -- -runs=256
cargo run --manifest-path fuzz/Cargo.toml --bin vir_parser -- -runs=256
cargo run --manifest-path fuzz/Cargo.toml --bin source_map_parser -- -runs=256
```
