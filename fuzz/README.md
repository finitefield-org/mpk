# MPK fuzz targets

Run the certificate decoder boundary target with cargo-fuzz:

```sh
cargo fuzz run certificate_decoder
```

For a bounded local smoke run without the cargo-fuzz subcommand:

```sh
cargo run --manifest-path fuzz/Cargo.toml --bin certificate_decoder -- -runs=256
```
