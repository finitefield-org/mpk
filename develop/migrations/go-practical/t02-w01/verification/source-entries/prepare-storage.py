"""Copy exact completed input containers; activate storage after workers finish."""
from pathlib import Path
import hashlib
import json
import os
import sys

ROOT = Path(__file__).resolve().parent
REPO = Path.cwd()
OUT = ROOT / "archive-storage"
PART = 4 * 1024 * 1024


def seal(path):
    raw = path.read_bytes()
    return {"path": str(path), "bytes": len(raw), "sha256": hashlib.sha256(raw).hexdigest(), "mode": path.stat().st_mode & 0o777}


def decoded(row):
    digest, size = hashlib.sha256(), 0
    for part in row["chunks"]:
        path = Path(part["encoded"]["path"])
        assert path.parent.parent == OUT and not path.is_symlink()
        assert seal(path) == part["encoded"]
        encoded = path.read_bytes()
        assert encoded.endswith(b"\n") and encoded.count(b"\n") == 1
        raw = bytes.fromhex(encoded[:-1].decode("ascii"))
        assert len(raw) == part["raw_bytes"] and hashlib.sha256(raw).hexdigest() == part["raw_sha256"]
        digest.update(raw)
        size += len(raw)
        yield raw
    assert size == row["archive"]["bytes"] and digest.hexdigest() == row["archive"]["sha256"]


def verify(manifest, restore=False):
    for row in manifest["archives"]:
        path = Path(row["archive"]["path"])
        assert path.parent.parent == ROOT and path.name == "inputs.tar.gz" and not path.is_symlink()
        assert row["authority"] == seal(Path(row["authority"]["path"]))
        for _ in decoded(row):
            pass
        if not path.exists():
            assert restore
            with path.open("xb") as output:
                for raw in decoded(row):
                    output.write(raw)
            os.chmod(path, row["archive"]["mode"])
        assert seal(path) == row["archive"]
    return {"status": "passed", "complete_original_containers": len(manifest["archives"]),
            "complete_original_bytes": sum(r["archive"]["bytes"] for r in manifest["archives"]),
            "all_original_bytes_and_modes_exact": True, "regenerated_or_recompiled": False}


def prepare():
    OUT.mkdir(exist_ok=False)
    sources = []
    for receipt in sorted(ROOT.glob("*/inputs.json")):
        archive = json.loads(receipt.read_bytes())["complete_archive"]
        if archive["bytes"] > 64 * 1024 * 1024:
            sources.append((archive, receipt, True))
    interruption = ROOT / "interruption.json"
    partial = json.loads(interruption.read_bytes())["partial_capture"]["retained_incomplete_container"]
    if partial["bytes"] > 64 * 1024 * 1024:
        sources.append((partial, interruption, False))
    rows = []
    for archive, authority, complete in sources:
        path = Path(archive["path"])
        assert path.parent.parent == ROOT and path.name == "inputs.tar.gz"
        assert seal(path) == archive
        directory = OUT / path.parent.name
        directory.mkdir(exist_ok=False)
        parts = []
        with path.open("rb") as source:
            while raw := source.read(PART):
                encoded = directory / f"part-{len(parts):03}.hex"
                with encoded.open("xb") as output:
                    output.write(raw.hex().encode("ascii") + b"\n")
                parts.append({"encoded": seal(encoded), "raw_bytes": len(raw), "raw_sha256": hashlib.sha256(raw).hexdigest()})
        rows.append({"archive": archive, "authority": seal(authority), "complete_prelaunch_container": complete, "chunks": parts})
    record = {"status": "prepared-exact-read-only-copies", "driver": seal(Path(__file__).resolve()), "archives": rows,
              "scope": "Copies only: original source/input containers and all consumed worker inputs remain unchanged. An interrupted incomplete container stays incomplete. No source scan/proof/checker exclusion is added."}
    verified = verify(record)
    with (OUT / "prepared.json").open("x") as output:
        json.dump(record, output, indent=2)
        output.write("\n")
    print(json.dumps(verified))


def activate():
    record = json.loads((OUT / "prepared.json").read_bytes())
    verified = verify(record)
    for receipt in ROOT.glob("*/inputs.json"):
        assert json.loads(receipt.with_name("result.json").read_bytes())["status"] in ["passed", "failed"]
    ignore = REPO / ".gitignore"
    previous = ignore.read_bytes()
    addition = b"\n# Exact local archive copies; tracked archive-storage chunks restore every byte.\n"
    for row in record["archives"]:
        addition += ("/" + str(Path(row["archive"]["path"]).relative_to(REPO)) + "\n").encode()
    with ignore.open("ab") as output:
        output.write(addition)
    record.update(status="passed", prepared_record=seal(OUT / "prepared.json"), verification=verified,
                  original_gitignore_hex=previous.hex(), gitignore_after=seal(ignore),
                  scope="Only duplicate local compressed containers are ignored. All source/proof/input/log/receipt bytes and modes remain retained and reviewed. Original containers remain physically unchanged and tracked chunks restore them exactly. No active-source scan exclusion or proof/checker admission changes.")
    with (OUT / "manifest.json").open("x") as output:
        json.dump(record, output, indent=2)
        output.write("\n")
    print(json.dumps(verified))


if __name__ == "__main__":
    if sys.argv[1:] == ["--prepare"]:
        prepare()
    elif sys.argv[1:] == ["--activate"]:
        activate()
    else:
        assert sys.argv[1:] in [["--verify"], ["--restore"]]
        record = json.loads((OUT / "manifest.json").read_bytes())
        assert record["status"] == "passed"
        print(json.dumps(verify(record, sys.argv[1:] == ["--restore"])))
