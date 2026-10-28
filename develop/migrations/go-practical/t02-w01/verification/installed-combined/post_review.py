"""Verify scoped-review inputs and its final records remain unchanged."""
from pathlib import Path
import hashlib
import json
import sys
import types

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[5]
scanner = REPO / 'scripts/check-no-active-gir.sh'
code = scanner.read_text().split("<<'PY'\n", 1)[1].rsplit('\nPY', 1)[0].rsplit('raise SystemExit(main())', 1)[0]
module = types.ModuleType('go_installed_combined_post_review')
module.__file__ = str(scanner)
sys.modules[module.__name__] = module
sys.argv = [str(scanner), str(REPO), 'strict']
exec(compile(code, str(scanner), 'exec'), module.__dict__)
matchers = module.load_matchers()
manifest, _, _, _ = module.validate_manifest()
module.run_fixture_self_test(manifest, matchers)
assert (len(matchers), len(module.SCOPES), len(manifest['sealed_records']), len(module.active_retained_record_exclusions(manifest))) == (127, 4, 39, 51)
review_path = ROOT / 'review-final.json'
review = json.loads(review_path.read_bytes())
assert review['status'] == 'passed' and not review['active_findings']
for row in review['inputs']:
    path = Path(row['path'])
    raw = path.read_bytes()
    assert (len(raw), hashlib.sha256(raw).hexdigest(), path.stat().st_mode & 0o777) == (row['bytes'], row['sha256'], row['mode']), str(path)
verification = json.loads((REPO / 'develop/migrations/go-practical/t02-w01/verification.json').read_bytes())
assert verification['test_only_installed_combined_checkpoint']['scoped_review'] == str(review_path.relative_to(REPO))
requirements = json.loads((REPO / 'develop/migrations/go-practical/t02-w01/requirements.json').read_bytes())
requirement = next(row for row in requirements['requirements'] if row['id'] == 'T02-W01-R06')
assert requirement['status'] == 'pending' and not requirement['preparatory_test_only_combined_compiler_checkpoint']['R06_completed']
paths = [Path(__file__).resolve(), review_path, Path(review['archived_findings']['encoded']['path']), ROOT / 'review-final.stdout', ROOT / 'review-final.stderr']
findings, rows = [], []
for path in paths:
    raw = path.read_bytes()
    findings.extend(vars(item) for item in module.detect(str(path.relative_to(REPO)), raw.decode(errors='ignore'), matchers))
    rows.append({'path': str(path), 'bytes': len(raw), 'sha256': hashlib.sha256(raw).hexdigest(), 'mode': path.stat().st_mode & 0o777})
    if path.suffix == '.json':
        json.loads(raw)
    if path.suffix == '.py':
        compile(raw, str(path), 'exec')
result = {'status': 'passed' if not findings else 'failed', 'selection_reason': 'Verify every reviewed input remains byte-identical and scan the final review records with unchanged rules.', 'inputs': rows, 'reviewed_inputs_unchanged': len(review['inputs']), 'findings': findings, 'full_T_gate': 'Deferred to final T02-W04: ./scripts/check-fast.sh'}
out = ROOT / 'post-review.json'
assert not out.exists()
out.write_text(json.dumps(result, indent=2) + '\n')
print(json.dumps({'status': result['status'], 'reviewed_inputs_unchanged': len(review['inputs']), 'findings': len(findings)}), flush=True)
raise SystemExit(bool(findings))
