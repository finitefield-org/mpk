"""Review changed files and every original preimage under unchanged rules.

Run outside the sandbox. The scoped report is not the final repository gate.
"""
from pathlib import Path
import hashlib
import json
import re
import subprocess
import sys
import types
import urllib.parse

REPO = Path.cwd()
ROOT = Path(__file__).resolve().parent
BASE = 'd4ac004547a575e5f4e81ec48385dc50f93ed435'
REPORT = ROOT/'input-review.json'


def git(*args, data=None):
    return subprocess.check_output(['/usr/bin/git', *args], input=data)


def seal(path, raw=None):
    raw = Path(path).read_bytes() if raw is None else raw
    return {'path': str(path), 'bytes': len(raw), 'sha256': hashlib.sha256(raw).hexdigest()}


assert git('rev-parse', 'HEAD').decode().strip() == BASE
assert not Path('.github/workflows').exists()
if not REPORT.exists():
    REPORT.write_text(json.dumps({'status': 'pending_review'})+'\n')
scanner = Path('scripts/check-no-active-gir.sh')
code = scanner.read_text().split("<<'PY'\n", 1)[1].rsplit('\nPY', 1)[0].rsplit('raise SystemExit(main())', 1)[0]
module = types.ModuleType('mpk_error_layout_review')
module.__file__ = str(REPO/scanner)
sys.modules[module.__name__] = module
sys.argv = [str(scanner), str(REPO), 'strict']
exec(compile(code, str(scanner), 'exec'), module.__dict__)
matchers = module.load_matchers()
manifest, scanner_files, fixtures, _ = module.validate_manifest()
module.run_fixture_self_test(manifest, matchers)
exclusions = module.active_retained_record_exclusions(manifest)
assert (len(matchers), len(module.SCOPES), len(manifest['sealed_records']), len(exclusions)) == (127, 4, 39, 51)
for name in scanner_files | fixtures:
    assert Path(name).read_bytes() == git('show', BASE+':'+name)
configured = scanner_files | fixtures | exclusions
cache = {}
findings = []
baseline = []
path_rules = [m for m in matchers if m['scope'] == 'exact-path']


def scan(name, raw, basis=False):
    key = (hashlib.sha256(raw).hexdigest(), tuple(module.path_matches(name, m['path']) for m in path_rules))
    if key not in cache:
        cache[key] = [vars(f) for f in module.detect(name, raw.decode(errors='ignore'), matchers)]
    rows = [{**f, 'path': name} for f in cache[key]]
    if basis:
        baseline.extend({**{k:v for k,v in f.items() if k != 'excerpt'},
                         'excerpt_hex': f['excerpt'].encode().hex(), 'basis_commit': BASE} for f in rows)
    else:
        findings.extend(rows)


epochs = [(p, json.loads(p.read_bytes())) for p in sorted(ROOT.glob('*/inputs.json'))]
oids = sorted({r['original_git_blob'] for _, epoch in epochs for r in epoch['sources'] if 'original_git_blob' in r})
names = sorted({r['path'] for _, epoch in epochs for r in epoch['sources'] if 'original_git_blob' in r})
base_files = {}
for row in git('ls-tree', '-z', BASE, '--', *names).split(b'\0'):
    if row:
        metadata, name = row.split(b'\t', 1)
        permissions, kind, oid = metadata.decode().split()
        assert kind == 'blob'
        base_files[name.decode()] = oid
raw = git('cat-file', '--batch', data=('\n'.join(oids)+'\n').encode())
originals = {}
cursor = 0
for oid in oids:
    end = raw.index(b'\n', cursor)
    actual, kind, size = raw[cursor:end].decode().split()
    size = int(size)
    assert (actual, kind) == (oid, 'blob')
    originals[oid] = raw[end+1:end+1+size]
    cursor = end+size+2
assert cursor == len(raw)
preimages = 0
executions = []
for path, epoch in epochs:
    assert seal(path.parent/'driver.py') == epoch['actual_driver_prelaunch']
    result = json.loads((path.parent/'result.json').read_bytes())
    assert result['status'] in {'passed', 'failed'} and result['source_receipt'] == seal(path)
    assert all(seal(result[key]['path']) == result[key] for key in ['stdout', 'stderr'])
    if not result['formatter_mutation_expected']:
        assert result['whole_source_inputs_unchanged']
    for row in epoch['sources']:
        oid = row.get('original_git_blob')
        if oid:
            assert base_files[row['path']] == oid and row['original_commit'] == BASE
        buf = originals[oid] if oid else Path(row['complete_changed_input']).read_bytes()
        assert len(buf) == row['bytes'] and hashlib.sha256(buf).hexdigest() == row['sha256']
        if not oid:
            assert Path(row['complete_changed_input']).stat().st_mode & 0o777 == row['mode']
        scan(row['path'], buf, basis=bool(oid))
        preimages += 1
    executions.append({'result': seal(path.parent/'result.json'), 'status': result['status'], 'command': result['command']})

paths = git('diff', '--name-only', '-z', 'HEAD', '--') + git('ls-files', '--others', '--exclude-standard', '-z')
paths = sorted(set(paths.decode().split('\0')) - {'', str(REPORT.relative_to(REPO))})
inputs = []
links = []
for name in paths:
    path = Path(name)
    assert path.is_file() and not path.is_symlink()
    raw = path.read_bytes()
    inputs.append(seal(name, raw))
    if name not in configured:
        scan(name, raw)
    if path.suffix == '.md':
        for target in re.findall(r'(?<!!)\[[^\]]*\]\(([^\n)]+)\)', raw.decode()):
            target = target.strip().strip('<>')
            if re.match(r'^[a-zA-Z][a-zA-Z0-9+.-]*:', target) or target.startswith('#'):
                continue
            destination = urllib.parse.unquote(target.split('#', 1)[0])
            links.append({'document': name, 'target': target, 'exists': (path.parent/destination).resolve().exists()})
report = {'status': 'passed' if not findings and all(row['exists'] for row in links) else 'failed',
          'basis_commit': BASE, 'matcher_count': len(matchers), 'scopes': len(module.SCOPES),
          'seals': len(manifest['sealed_records']), 'exclusions': len(exclusions),
          'scanner_rules_unchanged': True, 'strict_whole_repository_gate_passed': False,
          'actual_driver_prelaunch_sha256': hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
          'inputs': inputs, 'source_epochs': len(epochs), 'source_preimages': preimages,
          'executions': executions, 'findings': findings, 'preexisting_basis_findings': baseline, 'links': links}
REPORT.write_text(json.dumps(report, indent=2)+'\n')
print(json.dumps({'status': report['status'], 'epochs': len(epochs), 'preimages': preimages,
                  'findings': len(findings), 'basis_findings': len(baseline),
                  'missing_links': sum(not row['exists'] for row in links)}))
raise SystemExit(report['status'] != 'passed')
