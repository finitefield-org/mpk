# Basic certificate fixtures

These hex fixtures are canonical `.mpcert` byte streams for the smallest
positive certificate cases:

- `zero-axiom.hex`: an empty zero-axiom module.
- `one-theorem.hex`: one theorem whose proof term is `Sort 0` and theorem type
  is `Sort 1`, with no axiom dependencies.

`hashes.csv` pins the export and axiom-report hashes embedded in each fixture,
plus the recomputed hash of the canonical fixture byte stream.
