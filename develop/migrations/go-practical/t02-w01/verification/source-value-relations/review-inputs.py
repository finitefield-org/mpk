"""Review changed files and every original preimage under unchanged rules.

Run outside the sandbox. The scoped report is not the final repository gate.
"""
from pathlib import Path
import hashlib
import json
import re
import runpy
import subprocess
import sys
import types
import urllib.parse

REPO = Path.cwd()
ROOT = Path(__file__).resolve().parent
BASE = '7ead1ea45e602338b52a0d9ddf6d30db92782432'
REPORT = ROOT/'input-review.json'
def kept(path):
    p=Path(path);return p.read_bytes(),p.stat().st_mode&0o777


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
module = types.ModuleType('mpk_source_error_selector_review')
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
scope_originals=json.loads((ROOT/'scope-document-originals.json').read_bytes())['sources']
oids = sorted({r['original_git_blob'] for _, epoch in epochs for r in epoch['sources'] if 'original_git_blob' in r} | {r['original_git_blob'] for r in scope_originals})
names = sorted({r['path'] for _, epoch in epochs for r in epoch['sources'] if 'original_git_blob' in r} | {r['path'] for r in scope_originals})
base_files = {}
base_modes = {}
for row in git('ls-tree', '-z', BASE, '--', *names).split(b'\0'):
    if row:
        metadata, name = row.split(b'\t', 1)
        permissions, kind, oid = metadata.decode().split()
        assert kind == 'blob'
        base_files[name.decode()] = oid
        base_modes[name.decode()] = int(permissions, 8) & 0o777
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
            assert base_modes[row['path']] == row['mode']
        source_path = Path(row['complete_changed_input']) if not oid else None
        buf = originals[oid] if oid else kept(source_path)[0]
        assert len(buf) == row['bytes'] and hashlib.sha256(buf).hexdigest() == row['sha256']
        if not oid:
            assert kept(source_path)[1] == row['mode']
        scan(row['path'], buf, basis=bool(oid))
        preimages += 1
    executions.append({'result': seal(path.parent/'result.json'), 'status': result['status'], 'command': result['command']})

for row in scope_originals:
    assert row['original_commit']==BASE and base_files[row['path']]==row['original_git_blob']
    buf=originals[row['original_git_blob']]
    assert len(buf)==row['bytes'] and hashlib.sha256(buf).hexdigest()==row['sha256'] and base_modes[row['path']]==row['mode']
    scan(row['path'],buf,basis=True)

go_preimages = []
for receipt in sorted(ROOT.glob('*/prelaunch-inputs.json')):
    records = json.loads(receipt.read_bytes())
    result = json.loads(receipt.with_name('result.json').read_bytes())
    assert result['status'] == 'passed' and result['whole_inputs_unchanged']
    assert result['complete_prelaunch_inputs'] == seal(receipt)
    for row in records:
        saved = row['complete_prelaunch_snapshot']
        buf = Path(saved['path']).read_bytes()
        assert seal(saved['path']) == saved
        assert len(buf) == row['bytes'] and hashlib.sha256(buf).hexdigest() == row['sha256']
        assert Path(saved['path']).stat().st_mode & 0o777 == row['mode']
        original = Path(row['path'])
        name = str(original.relative_to(REPO)) if original.is_relative_to(REPO) else str(original)
        scan(name, buf)
    go_preimages.append({'receipt':seal(receipt),'input_rows':len(records),'native_kind':'Go paired checker' if 'go_binary' in result else 'Rust diagnostic only'})

archive_originals=[]

paths = git('diff', '--name-only', '-z', 'HEAD', '--') + git('ls-files', '--others', '--exclude-standard', '-z')
paths = sorted(set(paths.decode().split('\0')) - {'', str(REPORT.relative_to(REPO))})
document_origins={}
scope_review=json.loads((ROOT/'scope-document-review/result.json').read_bytes())
assert scope_review['status']=='Go_Rust_scope_document_links_and_registry_inventory_passed'
for row in scope_review['inputs']:
    saved=Path(row['complete_reviewed_document']);raw=saved.read_bytes()
    assert len(raw)==row['bytes'] and hashlib.sha256(raw).hexdigest()==row['sha256'] and saved.stat().st_mode&0o777==row['mode']
    document_origins[str(saved.relative_to(REPO)) if saved.is_absolute() else str(saved)]=row['path']
inputs = []
links = []
for name in paths:
    path = Path(name)
    if not path.exists():
        assert name == '.gitattributes'
        raw = git('show', BASE+':'+name)
        inputs.append({**seal(name, raw), 'deleted': True, 'original_commit': BASE})
        scan(name, raw, basis=True)
        continue
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
            logical_document=Path(document_origins.get(name,name))
            links.append({'document': name, 'logical_original_document':str(logical_document), 'target': target, 'exists': (logical_document.parent/destination).resolve().exists()})
# Literal binary findings stay visible. Only already both-core-accepted
# complete certificate bytes whose FULL text fields are clean may receive a
# nontext-data classification. This is scoped review, never strict-gate success.
verification=json.loads((ROOT/'verification.json').read_bytes())
accepted=set()
for run_ref in verification['paired_executions']:
    paired=json.loads(Path(run_ref['path']).read_bytes())
    assert paired['status']=='passed'
    for row in paired['cases']:
        if row['rust_observation']['verdict']==row['go_observation']['verdict']=='accepted':accepted.add(row['certificate']['sha256'])
classify=runpy.run_path(str(ROOT/'review-certificate-bytes.py'))['inspect']
classified=[];active_findings=[];classified_cache={}
for finding in findings:
    buf,mode=kept(finding['path']);digest=hashlib.sha256(buf).hexdigest()
    if digest not in accepted or not buf.startswith(b'MPKCERT'):
        active_findings.append(finding);continue
    if digest not in classified_cache:classified_cache[digest]=classify(Path(finding['path']))
    observed=classified_cache[digest]
    if not any(row['matcher_id']==finding['matcher_id'] for row in observed['literal_nontext_findings']):
        active_findings.append(finding);continue
    classified.append({**finding,'raw_sha256':digest,'classification':'literal_bytes_in_reviewed_nontext_certificate_hash_or_integer_sections','all_semantic_strings_clean':True,'raw_finding_preserved':True})
semantic_pass=not active_findings and all(row['exists'] for row in links)
report = {'status': ('passed_with_reviewed_nontext_binary_findings' if classified else 'passed') if semantic_pass else 'failed',
          'basis_commit': BASE, 'matcher_count': len(matchers), 'scopes': len(module.SCOPES),
          'seals': len(manifest['sealed_records']), 'exclusions': len(exclusions),
          'scanner_rules_unchanged': True, 'strict_whole_repository_gate_passed': False,
          'actual_driver_prelaunch_sha256': hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
          'inputs': inputs, 'complete_virtual_originals_scanned':archive_originals, 'source_epochs': len(epochs), 'source_preimages': preimages, 'scope_document_originals':scope_originals,
          'executions': executions, 'Go_prelaunch_receipts': go_preimages, 'findings': findings, 'active_interface_findings':active_findings, 'reviewed_nontext_binary_findings':classified, 'binary_semantic_audits':list(classified_cache.values()), 'literal_scanner_passed':not findings, 'preexisting_basis_findings': baseline, 'links': links}
REPORT.write_text(json.dumps(report, indent=2)+'\n')
print(json.dumps({'status': report['status'], 'epochs': len(epochs), 'preimages': preimages,
                  'findings': len(findings), 'basis_findings': len(baseline),
                  'missing_links': sum(not row['exists'] for row in links)}))
raise SystemExit(not semantic_pass)
