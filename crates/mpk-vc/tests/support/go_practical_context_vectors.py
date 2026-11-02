"""Build the frozen candidate corpus under the current Go/Rust scope."""
import copy
import json
from pathlib import Path

import context_protocol as protocol
import context_vectors as vectors

root = Path(__file__).resolve().parents[4]
foundation = root / "develop/migrations/go-practical/t01-w04/foundation-limits"
fixture = protocol.registration_fixture()
descriptor = json.loads((foundation / "descriptor.json").read_bytes())
limits = json.loads((foundation / "limits.json").read_bytes())
entry = next(
    row for row in fixture["registry"]["profiles"]
    if row["semantic_profile"] == protocol.GO_PROFILE
)
entry["foundation_descriptor"] = {
    key: descriptor[key] for key in ("schema", "id", "content_sha256")
}
fixture["go_parameters"]["limits_sha256"] = limits["content_sha256"]
fixture["registry"] = vectors.rehash_registry(fixture["registry"])
protocol.registration_fixture = lambda: copy.deepcopy(fixture)
vectors.registration_fixture = lambda: copy.deepcopy(fixture)
rows = vectors.cases()
assert len(rows) == 133 and sum(row["expected_accepted"] for row in rows) == 18
print(json.dumps(rows, ensure_ascii=False, separators=(",", ":")))
