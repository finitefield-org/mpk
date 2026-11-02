"""Reuse unchanged77-file private checker sources with a new ordinary emitter."""
import json
import shutil
from pathlib import Path
import termination_foundation_build as prior
from run_proof_pilot import HERE, ROOT, json_bytes, sha

PRODUCER=HERE/"closed-carriers"
TARGET=Path("/private/tmp/mpk-go-practical-carrier-target")


def prepare(destination):
    destination=Path(destination);prior.prepare(destination)
    shutil.rmtree(destination/"probe");shutil.copytree(PRODUCER,destination/"probe")
    manifest=destination/"probe/Cargo.toml"
    manifest.write_text(manifest.read_text().replace("[workspace]","").replace("../../../../crates/","../crates/"))
    source=destination/"probe/src/main.rs";text=source.read_text()
    old='"../../support/builder.rs"'
    assert text.count(old)==1
    source.write_text(text.replace(old,json.dumps(str(ROOT/"develop/probes/go-practical/support/builder.rs"))))
    (destination/"Cargo.lock").write_bytes((PRODUCER/"Cargo.lock").read_bytes())
    receipt={"generated":[{"path":p,"sha256":sha(raw)} for p,raw in sorted(prior.files().items())]}
    (destination/"private-build-inputs.json").write_bytes(json_bytes(receipt))
    return receipt
