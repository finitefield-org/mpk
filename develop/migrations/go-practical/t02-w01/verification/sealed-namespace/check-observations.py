"""Independently reconstruct private snapshot framing and full source identities."""
from pathlib import Path
import datetime
import hashlib
import json
import struct


def seal(path):
    path = Path(path)
    raw = path.read_bytes()
    return {"path": str(path), "sha256": hashlib.sha256(raw).hexdigest(), "bytes": len(raw)}


root = Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
selected = json.loads((root / "final/selected/result.json").read_text())
assert selected["status"] == "passed" and selected["sources_unchanged_during_execution"]
for record in selected["sources"]:
    assert seal(record["path"]) == record
    archived = seal(Path(selected["source_snapshot"]) / record["path"])
    assert (archived["sha256"], archived["bytes"]) == (record["sha256"], record["bytes"])
observation = root / "final/selected/namespace-vectors.json"
vectors = json.loads(observation.read_text())["vectors"]
assert len(vectors) == 17
directory = root / "independent-preimages"
directory.mkdir(exist_ok=False)
rows = []
hashes = {}
source_objects = 0
for index, vector in enumerate(vectors):
    parts = [b"MPK-GO-PRIVATE-SOURCE-SNAPSHOT-1.0\x00"]

    def number(value):
        assert type(value) is int and 0 <= value < 2**64
        parts.append(struct.pack("<Q", value))

    def raw_string(value):
        assert isinstance(value, bytes)
        number(len(value))
        parts.append(value)

    def text(value):
        raw_string(value.encode("utf-8"))

    roots = vector["capabilities"]
    assert roots == sorted(set(roots))
    number(len(roots))
    for capability in roots:
        text(capability)
    directories = vector["directories"]
    keys = [(d["capability"] + "\x00" + d["path"]).encode("utf-8") for d in directories]
    assert keys == sorted(set(keys))
    number(len(directories))
    for entry in directories:
        text(entry["capability"])
        text(entry["path"])
        names = [bytes.fromhex(e["name_hex"]) for e in entry["entries"]]
        assert names == sorted(set(names))
        number(len(entry["entries"]))
        for name, native in zip(names, entry["entries"]):
            assert name.hex() == native["name_hex"]
            raw_string(name)
            assert native["type_bits"] < 2**32
            number(native["type_bits"])
    ids = vector["source_ids"] or []
    assert ids == sorted(set(ids))
    assert ids == [source["object"]["id"] for source in vector["sources"]]
    for source in vector["sources"]:
        obj = source["object"]
        assert set(obj) == {"schema", "id", "capability", "path", "kind", "raw_sha256", "bytes"}
        assert obj["schema"] == "mpk.go.source_object.v1"
        raw = bytes.fromhex(source["bytes_hex"])
        assert raw.hex() == source["bytes_hex"]
        assert type(obj["bytes"]) is int and len(raw) == obj["bytes"]
        assert hashlib.sha256(raw).hexdigest() == obj["raw_sha256"]
        unsigned = {key: value for key, value in obj.items() if key != "id"}
        canonical = json.dumps(unsigned, sort_keys=True, ensure_ascii=False, separators=(",", ":")).encode("utf-8")
        assert hashlib.sha256(b"MPK-GO-SOURCE-OBJECT-1.0\x00" + canonical).hexdigest() == obj["id"]
        source_objects += 1
    number(len(ids))
    for source_id in ids:
        text(source_id)
    raw = b"".join(parts)
    expected = hashlib.sha256(raw).hexdigest()
    assert expected == vector["sha256"]
    name = directory / (f"{index:02d}-" + vector["label"] + ".bin")
    name.write_bytes(raw)
    rows.append({"label": vector["label"], "scope": vector["scope"], "preimage": seal(name), "sha256": expected,
                 "sources_checked": len(ids), "visited_directories": len(directories)})
    hashes[vector["label"]] = expected
assert hashes["original"] == hashes["reordered"] == hashes["unread-content-only"]
for label in ["added-entry", "renamed-unread", "changed-native-type", "visited-empty-now-nonempty", "source-bytes", "source-role", "additional-visited-directory", "additional-authorized-root"]:
    assert hashes[label] != hashes["original"]
raw_units = [row for row in rows if row["label"].startswith("raw-name-")]
assert len(raw_units) == len({row["sha256"] for row in raw_units}) == 6
assert {source["object"]["kind"] for source in vectors[0]["sources"]} == {
    "go_file", "go_mod", "go_sum", "overlay", "metadata", "export_data", "excluded_source", "auxiliary"
}
record = {"status": "passed", "observed_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
          "observer": seal(__file__), "original_Go_observations": seal(observation),
          "vectors": rows, "complete_preimages_checked": len(rows), "source_objects_reconstructed": source_objects,
          "native_capture_vectors": 11, "raw_byte_framing_units": 6,
          "scope": "Independent Python reconstruction of every byte-length/count/type/source-ID frame and all complete original source-object identities. Raw invalid-name units exercise framing only and cannot be imported as captured namespaces. Unread asset content is deliberately not a captured source; complete visited names/types are still retained. This supplies no source equivalence, registry/context, immutable application replay, kernel proof or native Linux release authority."}
(root / "independent-observations.json").write_text(json.dumps(record, indent=2) + "\n")
print(json.dumps({key: record[key] for key in ["status", "complete_preimages_checked", "source_objects_reconstructed", "native_capture_vectors", "raw_byte_framing_units"]}))
