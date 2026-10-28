"""Review the exact-output whitespace audit added after the scoped review."""
from pathlib import Path
import hashlib
import json
import sys
import types

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[5]
scanner = REPO / 'scripts/check-no-active-gir.sh'
code = scanner.read_text().split("<<'PY'\n", 1)[1].rsplit('\nPY', 1)[0].rsplit('raise SystemExit(main())', 1)[0]
module = types.ModuleType('go_installed_combined_staged_audit_review')
module.__file__ = str(scanner)
sys.modules[module.__name__] = module
sys.argv = [str(scanner), str(REPO), 'strict']
exec(compile(code, str(scanner), 'exec'), module.__dict__)
matchers = module.load_matchers()
manifest, _, _, _ = module.validate_manifest()
module.run_fixture_self_test(manifest, matchers)
assert (len(matchers), len(module.SCOPES), len(manifest['sealed_records']), len(module.active_retained_record_exclusions(manifest))) == (127, 4, 39, 51)
paths = [Path(__file__).resolve(), ROOT / 'check_staged_whitespace.py', ROOT / 'staged-whitespace.json', ROOT / 'post-review.json']
findings, rows = [], []
for path in paths:
    raw = path.read_bytes()
    findings.extend(vars(item) for item in module.detect(str(path.relative_to(REPO)), raw.decode(errors='ignore'), matchers))
    rows.append({'path': str(path), 'bytes': len(raw), 'sha256': hashlib.sha256(raw).hexdigest(), 'mode': path.stat().st_mode & 0o777})
    if path.suffix == '.json':
        json.loads(raw)
    if path.suffix == '.py':
        compile(raw, str(path), 'exec')
result = {'status': 'passed' if not findings else 'failed', 'selection_reason': 'Review the staged whitespace audit while retaining every original native test output byte.', 'inputs': rows, 'findings': findings, 'scanner_rules_changed': False, 'full_T_gate': 'Deferred to final T02-W04: ./scripts/check-fast.sh'}
out = ROOT / 'staged-audit-review.json'
assert not out.exists()
out.write_text(json.dumps(result, indent=2) + '\n')
assert not list(module.detect(str(out.relative_to(REPO)), out.read_text(), matchers))
print(json.dumps({'status': result['status'], 'files': len(rows), 'findings': len(findings)}), flush=True)
raise SystemExit(bool(findings))
