# Package manifest fixtures

These fixtures exercise `package-manifest.md`.

- `valid/basic-package.json` is a minimal package manifest that imports one
  basic module and checks one canonical certificate fixture.
- `invalid/missing-certificate-hash.json` omits a required certificate hash and
  must be rejected.

Run `python3 scripts/check-package-manifest-fixtures.py` from the repository root
to validate the fixture contract.

By default the validator runs `cargo run --quiet -p mpk-cli -- check` for each
listed certificate. Set `MPK_BIN=/path/to/mpk` only when intentionally validating
against a specific checker binary.
