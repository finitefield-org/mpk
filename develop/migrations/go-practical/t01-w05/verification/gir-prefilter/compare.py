"""Differential check of unchanged matcher semantics and fixture cases."""
import ast
from collections import Counter
from dataclasses import asdict
import hashlib
import json
from pathlib import Path
import random
import sys
import time
import types

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[5]


def module(path, name):
    source = path.read_text().split("<<'PY'\n", 1)[1].rsplit("\nPY", 1)[0]
    tree = ast.parse(source)
    assert ast.unparse(tree.body[-1]) == "raise SystemExit(main())"
    tree.body.pop()
    mod = types.ModuleType(name); sys.modules[name] = mod
    previous = sys.argv
    try:
        sys.argv = [str(path), str(ROOT), "strict"]
        exec(compile(tree, str(path), "exec"), mod.__dict__)
    finally:
        sys.argv = previous
    return mod


def main():
    before = Path(sys.argv[1]) if len(sys.argv) > 1 else Path("/private/tmp/mpk-gir-prefilter-before.sh")
    after = ROOT / "scripts/check-no-active-gir.sh"
    old = module(before, "gir_before"); new = module(after, "gir_after")
    matchers = new.load_matchers(); assert old.load_matchers() == matchers
    manifest, *_ = new.validate_manifest()
    old.run_fixture_self_test(manifest, matchers); new.run_fixture_self_test(manifest, matchers)
    cases = [("no-obsolete-symbols.hex", "0123456789abcdef" * 16)]
    for m in matchers:
        scope = m["scope"]
        if scope == "exact-global-token":
            token = m["token"]
            for prefix in ("", "a", "_", " ", "α"):
                for suffix in ("", "z", "_", "\n", "."):
                    cases.append(("sample.txt", prefix + token + suffix))
        elif scope == "exact-path":
            path = m["path"]
            # A filename finding must survive even when content is empty.
            cases += [(path, ""), (path + "/child", ""), (path + "-suffix", ""),
                      ("sample.txt", path), ("sample.txt", "a" + path + "z")]
        elif scope == "type/variant-qualified-code-symbol":
            for separator in ("::", " :: ", "\n::\t"):
                text = separator.join(m["symbol"].split("::"))
                cases += [("sample.rs", text), ("sample.rs", "a" + text), ("sample.rs", text + "z")]
        else:
            for schema in m["schemas"]:
                field = m["field"]
                for key in ("schema", "schema_version"):
                    for nested in (False, True):
                        obj = {key: schema, "children": [{field: None}]} if nested else {key: schema, field: None}
                        text = json.dumps(obj)
                        cases += [("sample.json", text), ("sample.txt", "invalid JSON " + text)]
                        escaped = text.replace('"' + key + '"', '"\\u%04x%s"' % (ord(key[0]), key[1:]))
                        escaped = escaped.replace(schema, schema.replace(".", "\\u002e"))
                        escaped = escaped.replace('"' + field + '"', '"\\u%04x%s"' % (ord(field[0]), field[1:]))
                        cases.append(("escaped.json", escaped))
    rng = random.Random(1974); seeds = list(cases)
    for _ in range(300):
        path, text = rng.choice(seeds)
        cases.append((path, rng.choice(("", "_", "\n", "// ")) + text + rng.choice(("", "z", "\n", " suffix"))))
    results = []
    for path, text in cases:
        a = [asdict(x) for x in old.detect(path, text, matchers)]
        b = [asdict(x) for x in new.detect(path, text, matchers)]
        assert a == b, (path, text[:300], a, b)
        results.append(a)
    timings = {}
    large = "0123456789abcdef" * (1024 * 256)
    for name, model in (("before", old), ("after", new)):
        start = time.monotonic(); found = model.detect("large.hex", large, matchers)
        timings[name] = time.monotonic() - start; assert not found
    report = {"schema": "mpk.gir_prefilter.differential_observation.v0",
              "matcher_counts": dict(Counter(m["scope"] for m in matchers)),
              "existing_fixtures": len(manifest["fixtures"]), "differential_cases": len(cases),
              "existing_fixture_roles": dict(Counter(m["role"] for m in manifest["fixtures"])),
              "before_sha256": hashlib.sha256(before.read_bytes()).hexdigest(),
              "after_sha256": hashlib.sha256(after.read_bytes()).hexdigest(),
              "comparison_sha256": hashlib.sha256(json.dumps(results, sort_keys=True).encode()).hexdigest(),
              "large_absent_input_bytes": len(large), "wall_seconds": timings,
              "all_outputs_identical": True, "scan_scope_reduced": False}
    (HERE / "comparison.json").write_text(json.dumps(report, indent=2, sort_keys=True) + "\n")
    print(json.dumps(report, sort_keys=True))


if __name__ == "__main__": main()
