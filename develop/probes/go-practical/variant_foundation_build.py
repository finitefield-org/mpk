"""Reuse the pinned private checker proposal with a new untrusted producer."""
import json
import shutil
from pathlib import Path
import termination_foundation_build as prior
from run_proof_pilot import HERE, ROOT, json_bytes, sha

PRODUCER = HERE / "variant-certificates"
SCOPE = "unchanged private Bool.recType/Nat.induct checkers; new ordinary arbitrary-Nat variant termination producer"


def sources():
    rows = {r["path"]: r["sha256"] for r in prior.sources()}
    for p in list(PRODUCER.rglob("*")) + [Path(__file__)]:
        if p.is_file():
            rows[p.relative_to(ROOT).as_posix()] = sha(p.read_bytes())
    return [{"path": p, "sha256": h} for p, h in sorted(rows.items())]


def prepare(destination):
    destination = Path(destination)
    prior.prepare(destination)
    shutil.rmtree(destination / "probe")
    shutil.copytree(PRODUCER, destination / "probe")
    p = destination / "probe/Cargo.toml"
    p.write_text(p.read_text().replace("[workspace]", "").replace("../../../../crates/", "../crates/"))
    p = destination / "probe/build.rs"
    p.write_text(p.read_text().replace("let root = manifest.ancestors().nth(4).unwrap();",
                                      "let root = Path::new(" + json.dumps(str(ROOT)) + ");"))
    (destination / "Cargo.lock").write_bytes((PRODUCER / "Cargo.lock").read_bytes())
    receipt = {"scope": SCOPE, "inputs": sources(),
               "generated": [{"path": p, "sha256": sha(raw)} for p, raw in sorted(prior.files().items())]}
    (destination / "private-build-inputs.json").write_bytes(json_bytes(receipt))
    return receipt


if __name__ == "__main__":
    import sys
    print(json.dumps(prepare(sys.argv[1])["scope"]))
