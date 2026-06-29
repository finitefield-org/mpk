# Package lock fixtures

These fixtures exercise `package-lock.md`.

- `valid/basic-package-lock.json` pins the valid package manifest, its import
  certificate hash, and its checker policy.
- `invalid/import-hash-mismatch.json` changes a locked import export hash and
  must be rejected.
- `invalid/policy-mismatch.json` changes checker policy and must be rejected.

Run `python3 scripts/check-package-lock-fixtures.py` from the repository root to
validate the fixture contract.
