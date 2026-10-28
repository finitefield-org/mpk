"""Inspect only the newly retained compressed test-input archives."""
from pathlib import Path
import hashlib
import json
import sys
import types

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[5]
scanner = REPO / 'scripts/check-no-active-gir.sh'
code = scanner.read_text().split("<<'PY'\n", 1)[1].rsplit('\nPY', 1)[0].rsplit('raise SystemExit(main())', 1)[0]
module = types.ModuleType('installed_combined_archive_precheck')
module.__file__ = str(scanner)
sys.modules[module.__name__] = module
sys.argv = [str(scanner), str(REPO), 'strict']
exec(compile(code, str(scanner), 'exec'), module.__dict__)
matchers = module.load_matchers()
manifest, _, _, _ = module.validate_manifest()
module.run_fixture_self_test(manifest, matchers)
assert (len(matchers), len(module.SCOPES), len(manifest['sealed_records']), len(module.active_retained_record_exclusions(manifest))) == (127, 4, 39, 51)
paths = [ROOT / epoch / 'inputs.tar.gz' for epoch in ['cfg-original', 'preflight-original', 'scope-original', 'scoped-final']]
rows = []
for path in paths:
    raw = path.read_bytes()
    name = str(path.relative_to(REPO))
    findings = [vars(item) for item in module.detect(name, raw.decode(errors='ignore'), matchers)]
    rows.append({'path': name, 'bytes': len(raw), 'sha256': hashlib.sha256(raw).hexdigest(), 'mode': path.stat().st_mode & 0o777, 'findings': findings})
result = {'status': 'completed', 'selection_reason': 'Identify any incidental tokens in compressed machine bytes before the complete scoped review, preserving all original archives and unchanged scanner rules.', 'archives': rows, 'scanner_rules_changed': False}
out = ROOT / 'archive-precheck.json'
assert not out.exists()
out.write_text(json.dumps(result, indent=2) + '\n')
print(json.dumps({'status': result['status'], 'archives': len(rows), 'findings': sum(len(row['findings']) for row in rows)}), flush=True)
