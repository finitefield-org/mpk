# MPK fuzz targets

Run the fuzz targets with cargo-fuzz:

```sh
cargo fuzz run certificate_decoder
cargo fuzz run theory_certificates
```

For bounded local smoke runs without the cargo-fuzz subcommand:

```sh
cargo run --manifest-path fuzz/Cargo.toml --bin certificate_decoder -- -runs=256
cargo run --manifest-path fuzz/Cargo.toml --bin theory_certificates -- -runs=256
```
