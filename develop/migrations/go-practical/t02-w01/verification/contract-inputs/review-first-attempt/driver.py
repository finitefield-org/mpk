"""Scoped unchanged-rule review; run outside sandbox for authorized system Git."""
from pathlib import Path
import hashlib
import json
import re
import subprocess
import sys
import tarfile
import types
import urllib.parse

REPO = Path.cwd()
ROOT = Path(__file__).resolve().parent
BASIS = "8ac557f1eb3d13ebf6ef7b41b6679075857b1a0a"


def git(*args): return subprocess.check_output(["/usr/bin/git", *args])
def seal(path):
    raw = path.read_bytes()
    return {"path": str(path), "bytes": len(raw), "sha256": hashlib.sha256(raw).hexdigest(), "mode": path.stat().st_mode & 0o777}
def logical(path):
    p = Path(path)
    return str(p.relative_to(REPO)) if p.is_absolute() else str(p)


def main():
    assert git("rev-parse", "HEAD").decode().strip() == BASIS
    scanner = REPO / "scripts/check-no-active-gir.sh"
    code = scanner.read_text().split("<<'PY'\n", 1)[1].rsplit("\nPY", 1)[0].rsplit("raise SystemExit(main())", 1)[0]
    module = types.ModuleType("contract_input_scoped_review")
    module.__file__ = str(scanner)
    sys.modules[module.__name__] = module
    sys.argv = [str(scanner), str(REPO), "strict"]
    exec(compile(code, str(scanner), "exec"), module.__dict__)
    matchers = module.load_matchers()
    manifest, configured, fixtures, _ = module.validate_manifest()
    module.run_fixture_self_test(manifest, matchers)
    retained = module.active_retained_record_exclusions(manifest)
    assert (len(matchers), len(module.SCOPES), len(manifest["sealed_records"]), len(retained)) == (127, 4, 39, 51)
    for name in configured | fixtures:
        assert (REPO / name).read_bytes() == git("show", BASIS + ":" + name)
    bases = {}
    for item in git("ls-tree", "-rz", BASIS, "--", "crates", "proofs/std", "fixtures", "develop/specs/vectors", "release/bundles", "go-tools/mpk-checker-ref", "go-tools/go2vir").split(b"\0"):
        if not item:
            continue
        meta, name = item.split(b"\t", 1)
        mode, kind, oid = meta.decode().split()
        assert kind == "blob"
        bases[name.decode()] = (oid, int(mode, 8) & 0o777)
    cache, findings, baseline = {}, [], []
    path_rules = [rule for rule in matchers if rule["scope"] == "exact-path"]
    def scan(name, raw, permissions):
        blob = hashlib.sha1(b"blob " + str(len(raw)).encode() + b"\0" + raw).hexdigest()
        is_basis = bases.get(name) == (blob, permissions)
        key = (hashlib.sha256(raw).hexdigest(), tuple(module.path_matches(name, rule["path"]) for rule in path_rules))
        if key not in cache:
            cache[key] = [vars(row) for row in module.detect(name, raw.decode(errors="ignore"), matchers)]
        for row in cache[key]:
            target = baseline if is_basis else findings
            target.append({**{k: v for k, v in row.items() if k != "excerpt"}, "path": name, "excerpt_hex": row["excerpt"].encode().hex()})
    epochs, count = [], 0
    for path in sorted(ROOT.glob("*/inputs.json")):
        receipt = json.loads(path.read_bytes())
        result = json.loads(path.with_name("result.json").read_bytes())
        assert result["status"] in ["passed", "failed"]
        assert result["inputs"] == seal(path)
        for field in ["stdout", "stderr", "native_stdout", "native_stderr"]:
            if field in result:
                assert result[field] == seal(Path(result[field]["path"]))
        assert result["consumed_inputs_unchanged"] or result["expected_formatter_mutation"]
        archive = Path(receipt["complete_archive"]["path"])
        assert receipt["complete_archive"] == seal(archive)
        with tarfile.open(archive, "r:gz") as packed:
            assert len(packed.getmembers()) == len(receipt["sources"])
            for row in receipt["sources"]:
                name = logical(row["path"])
                member = packed.getmember(name)
                raw = packed.extractfile(member).read()
                assert len(raw) == row["bytes"] and hashlib.sha256(raw).hexdigest() == row["sha256"]
                assert member.mode == row["mode"]
                scan(name, raw, row["mode"])
                count += 1
        epochs.append({"result": seal(path.with_name("result.json")), "status": result["status"], "command": result["command"], "inputs": len(receipt["sources"])})
    storage = json.loads(subprocess.check_output([sys.executable, "-B", str(ROOT / "preserve-originals.py"), "--verify"]))
    assert storage["status"] == "passed" and storage["all_original_paths_bytes_and_modes_exact"]
    snapshot = json.loads((ROOT / "snapshot-storage/manifest.json").read_bytes())
    packed = bytes.fromhex("".join((ROOT / "snapshot-storage" / row["file"]).read_text().strip() for row in snapshot["chunks"]))
    assert len(packed) == snapshot["archive_bytes"] and hashlib.sha256(packed).hexdigest() == snapshot["archive_sha256"]
    import io
    with tarfile.open(fileobj=io.BytesIO(packed), mode="r:xz") as archive:
        assert len(archive.getmembers()) == len(snapshot["originals"])
        for row, member in zip(snapshot["originals"], archive.getmembers()):
            raw = archive.extractfile(member).read()
            assert member.name == row["member"] and member.mode == row["mode"]
            assert len(raw) == row["bytes"] and hashlib.sha256(raw).hexdigest() == row["sha256"]
            scan(logical(row["path"]), raw, row["mode"])
    inspection = json.loads((ROOT / "inspection/result.json").read_bytes())
    assert inspection["status"] == "passed" and inspection["consumed_inputs_unchanged"]
    assert inspection["prelaunch"] == seal(ROOT / "inspection/prelaunch.json")
    for row in json.loads((ROOT / "inspection/prelaunch.json").read_bytes())["sources"]:
        assert row == seal(Path(row["path"]))
    report_path = ROOT / "input-review.json"
    names = (git("diff", "--name-only", "-z", "HEAD") + git("ls-files", "--others", "--exclude-standard", "-z")).decode().split("\0")
    current, links = [], []
    for name in sorted(set(names) - {"", logical(report_path)}):
        p = REPO / name
        assert p.is_file() and not p.is_symlink(), name
        raw = p.read_bytes()
        scan(name, raw, p.stat().st_mode & 0o777)
        current.append(seal(p))
        if p.suffix == ".md":
            for target in re.findall(r"\]\(([^)]+)\)", raw.decode()):
                target = target.strip("<>").split("#", 1)[0]
                if not target or re.match(r"^[a-zA-Z][a-zA-Z0-9+.-]*:", target):
                    continue
                target = urllib.parse.unquote(target)
                resolved = p.parent / re.sub(r":\d+$", "", target)
                links.append({"source": name, "target": target, "exists": resolved.exists()})
    assert all(row["exists"] for row in links)
    report = {"status": "passed" if not findings else "failed", "basis_commit": BASIS,
              "matcher_count": len(matchers), "scopes": len(module.SCOPES), "seals": len(manifest["sealed_records"]), "exclusions": len(retained),
              "rules_unchanged": True, "whole_repository_strict_gate": False,
              "complete_preserved_source_epochs": epochs, "complete_prelaunch_members_verified_and_scanned": count,
              "current_inputs": current, "new_findings": findings, "unchanged_basis_findings": baseline, "links": links,
              "exact_archive_storage": storage, "archive_storage_manifest": seal(ROOT / "snapshot-storage/manifest.json"),
              "independent_inspection": seal(ROOT / "inspection/result.json"),
              "complete_snapshot_members_verified_and_scanned": len(snapshot["originals"]),
              "full_gate": "Deferred to final T02-W04. All original failures remain, without a new seal, exclusion or narrowed test/cap."}
    report_path.write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps({"status": report["status"], "epochs": len(epochs), "source_preimages": count, "new_findings": len(findings), "basis_findings": len(baseline), "links": len(links)}))
    raise SystemExit(bool(findings))


if __name__ == "__main__":
    main()
