"""Check exact historical diagnostic seals without running a whole-stage gate."""

from pathlib import Path
import copy
import datetime
import hashlib
import json
import sys
import types

repo = Path.cwd()
root = Path(__file__).resolve().parent.relative_to(repo)
script = Path("scripts/check-no-active-gir.sh")
source = script.read_text().split("<<'PY'\n", 1)[1].rsplit("\nPY", 1)[0]
source = source.rsplit("raise SystemExit(main())", 1)[0]
module = types.ModuleType("mpk_t02_seal_controls")
module.__file__ = str(repo / script)
sys.modules[module.__name__] = module
sys.argv = [str(script), str(repo), "strict"]
exec(compile(source, str(script), "exec"), module.__dict__)
matchers = module.load_matchers()
assert len(matchers) == 127 and len(module.SCOPES) == 4
manifest, _, _, _ = module.validate_manifest()
module.run_fixture_self_test(manifest, matchers)
amendment = json.loads((root / "seal-amendment.json").read_text())
original_read = module.read_strict_json
controls = []
diagnostic_matches = {}
seals = amendment["new_seals"] + [json.loads((root / "additional-review-fixture-seal.json").read_text())["new_seal"]]
for seal in seals:
    path = Path(seal["path"])
    raw = path.read_bytes()
    assert hashlib.sha256(raw).hexdigest() == seal["sha256"]
    assert len(raw) == seal["size_bytes"]
    findings = module.detect(str(path), raw.decode(), matchers)
    assert findings, "sealed diagnostic must retain actual historical findings"
    diagnostic_matches[str(path)] = len(findings)
    for field, value in [("sha256", "0" * 64), ("size_bytes", seal["size_bytes"] + 1),
                         ("source_epoch", "worktree:" + "0" * 64),
                         ("path", "go-tools/go2vir/practical_source.go")]:
        changed = copy.deepcopy(manifest)
        item = next(item for item in changed["sealed_records"] if item["path"] == str(path))
        item[field] = value
        module.read_strict_json = lambda p, changed=changed: changed if p == repo / module.MANIFEST_PATH else original_read(p)
        try:
            module.validate_manifest()
        except module.GateError:
            controls.append({"path": str(path), "mutation": field, "status": "rejected"})
        else:
            raise AssertionError(f"altered diagnostic seal accepted: {field}")
        finally:
            module.read_strict_json = original_read
for name in ["practical_source.go", "practical_source_test.go"]:
    p = Path("go-tools/go2vir") / name
    assert not module.detect(str(p), p.read_text(), matchers)
result = {"status": "passed", "observed_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
          "scope": "Changed scanner seal entries only, all existing fixture controls and the two new Go source files; no whole repository/stage gate and no CSharp test execution.",
          "matcher_rules": 127, "scopes": 4, "exact_sealed_records_validated": len(manifest["sealed_records"]),
          "existing_fixture_checks": "passed", "diagnostic_matches_preserved": diagnostic_matches,
          "mutated_seal_rejections": controls, "production_sources_scanned_without_exclusions": 2,
          "production_or_directory_exclusions_added": False,
          "full_T02_gate": "deferred to W04: check-fast.sh --skip-csharp"}
(root / "final-result.json").write_text(json.dumps(result, indent=2) + "\n")
print(json.dumps({"status": "passed", "matchers": 127, "scopes": 4,
                  "exact_seals": len(manifest["sealed_records"]), "negative_controls": len(controls)}))
