"""Scoped unchanged-rule review and literal original-byte audit.

Run outside the sandbox. Every Git invocation uses /usr/bin/git. Existing basis
findings remain visible; this report is not the final whole-repository gate.
"""
from pathlib import Path
import base64
import hashlib
import io
import json
import re
import subprocess
import sys
import tarfile
import types
import urllib.parse

ROOT = Path.cwd()
OUT = Path(__file__).resolve().parent
BASE = '7e5748a5d060555197969a379b90d51f1eed0b1d'
REPORT = OUT/'input-review.json'


def git(*args, data=None):
    return subprocess.check_output(['/usr/bin/git', *args], input=data)


def seal(path, raw=None):
    raw = Path(path).read_bytes() if raw is None else raw
    return {'path': str(path), 'bytes': len(raw), 'sha256': hashlib.sha256(raw).hexdigest()}


assert git('rev-parse', 'HEAD').decode().strip() == BASE
assert not Path('.github/workflows').exists()
scanner = Path('scripts/check-no-active-gir.sh')
code = scanner.read_text().split("<<'PY'\n", 1)[1].rsplit('\nPY', 1)[0].rsplit('raise SystemExit(main())', 1)[0]
module = types.ModuleType('mpk_contract_expression_review')
module.__file__ = str(ROOT/scanner)
sys.modules[module.__name__] = module
sys.argv = [str(scanner), str(ROOT), 'strict']
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
        baseline.extend({**{k:v for k,v in f.items() if k != 'excerpt'}, 'excerpt_hex': f['excerpt'].encode().hex(), 'basis_commit': BASE, 'basis_bytes_exact': True} for f in rows)
    else:
        findings.extend(rows)


decoded = {}


def values(value, name):
    if isinstance(value, list):
        for i, child in enumerate(value):
            values(child, name+f'[{i}]')
    if not isinstance(value, dict):
        return
    for key in ['canonical_components', 'raw', 'raw_hex', 'components_hex']:
        if key not in value:
            continue
        if key == 'raw' and 'object' not in value:
            continue
        encoded = value[key]
        if not isinstance(encoded, str):
            continue
        raw = base64.b64decode(encoded, validate=True) if key in {'raw', 'canonical_components'} else bytes.fromhex(encoded)
        scan(name+'#decoded-'+key, raw)
        decoded[key] = decoded.get(key, 0)+1
        if key in {'raw', 'raw_hex'} and 'object' in value:
            obj = value['object']
            assert obj['bytes'] == len(raw) and obj['raw_sha256'] == hashlib.sha256(raw).hexdigest()
            body = {k:v for k,v in obj.items() if k != 'id'}
            canonical = json.dumps(body, sort_keys=True, separators=(',', ':'), ensure_ascii=False).encode()
            assert obj['id'] == hashlib.sha256(b'MPK-GO-SOURCE-OBJECT-1.0\0'+canonical).hexdigest()
        if key in {'components_hex', 'canonical_components'}:
            values(json.loads(raw), name+'#components')
    if 'whole_original_sources' in value:
        for path, encoded in value['whole_original_sources'].items():
            raw = base64.b64decode(encoded, validate=True)
            scan(name+'#whole-original-source/'+path, raw)
            decoded['whole_original_sources'] = decoded.get('whole_original_sources', 0)+1
    for key, child in value.items():
        if key not in {'canonical_components', 'raw', 'raw_hex', 'components_hex', 'whole_original_sources'}:
            values(child, name+'/'+key)


# Decode the archive and compare all members literally to their original
# receipts. Its filesystem storage encoding does not remove scan coverage.
virtual = {}
for archive_path in [OUT/'native-original/manifest.json', OUT/'native-receivers-original/manifest.json']:
    archive = json.loads(archive_path.read_bytes())
    assert archive['status'] == 'passed'
    assert hashlib.sha256((archive_path.parent/'driver.py').read_bytes()).hexdigest() == archive['actual_driver_prelaunch_sha256']
    packed = bytearray()
    for row in archive['chunks']:
        assert seal(row['path']) == row
        packed.extend(bytes.fromhex(Path(row['path']).read_text()))
    with tarfile.open(fileobj=io.BytesIO(packed), mode='r:*') as tar:
        members = tar.getmembers()
        assert [m.name for m in members] == [r['member'] for r in archive['files']]
        for member, row in zip(members, archive['files']):
            raw = tar.extractfile(member).read()
            assert member.mode == row['mode'] and len(raw) == row['bytes'] and hashlib.sha256(raw).hexdigest() == row['sha256']
            assert row['path'] not in virtual
            virtual[row['path']] = (raw, member.mode)
            scan(row['path'], raw)
            if row['path'].endswith('.json'):
                values(json.loads(raw), row['path'])


def read(path):
    name = str(path)
    if name in virtual:
        return virtual[name]
    p = Path(path)
    return p.read_bytes(), p.stat().st_mode & 0o777


for name in ['derived','derived-receivers']:
    derived = json.loads((OUT/name/'result.json').read_bytes())
    assert derived['status'] == 'passed'
    for row in [derived['actual_driver_prelaunch'], derived['source_receipt'], derived['output']]:
        assert all(seal(row['path'])[k] == row[k] for k in ['path', 'bytes', 'sha256'])
    for row in json.loads(Path(derived['source_receipt']['path']).read_bytes()):
        raw, mode = read(row['path'])
        assert seal(row['path'], raw) == {k:row[k] for k in ['path', 'bytes', 'sha256']} and mode == row['mode']
fixed = Path('develop/migrations/go-practical/t01-w04/contract-wire/expression-witnesses.json')
assert fixed.read_bytes() == git('show', BASE+':'+str(fixed))
scan(str(fixed), fixed.read_bytes(), basis=True)

epochs = []
for p in sorted(OUT.glob('*/inputs.json')):
    if not (p.parent/'driver.py').is_file() or not (p.parent/'result.json').is_file():
        continue
    data = json.loads(p.read_bytes())
    if isinstance(data, dict) and 'sources' in data:
        epochs.append((p, data))
oids = sorted({r['original_git_blob'] for _, e in epochs for r in e['sources'] if 'original_git_blob' in r})
names = sorted({r['path'] for _, e in epochs for r in e['sources'] if 'original_git_blob' in r})
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
for p, epoch in epochs:
    assert seal(p.parent/'driver.py') == epoch['actual_driver_prelaunch']
    result = json.loads((p.parent/'result.json').read_bytes())
    assert result['status'] in {'passed', 'failed'}
    assert result['source_receipt'] == seal(p)
    for key in ['stdout', 'stderr']:
        assert seal(result[key]['path']) == result[key]
    if not result['formatter_mutation_expected']:
        assert result['whole_source_inputs_unchanged']
    for row in epoch['sources']:
        original = row.get('original_git_blob')
        if original:
            assert base_files[row['path']] == original and row['original_commit'] == BASE
        buf = originals[original] if original else Path(row['complete_changed_input']).read_bytes()
        assert len(buf) == row['bytes'] and hashlib.sha256(buf).hexdigest() == row['sha256']
        if not original:
            assert Path(row['complete_changed_input']).stat().st_mode & 0o777 == row['mode']
        scan(row['path'], buf, basis=bool(original))
        preimages += 1
    executions.append({'result': seal(p.parent/'result.json'), 'status': result['status'], 'command': result['command']})

selected = git('diff', '--name-only', '-z', 'HEAD', '--') + git('ls-files', '--others', '--exclude-standard', '-z')
paths = sorted(set(selected.decode().split('\0')) - {'', str(REPORT.relative_to(ROOT))})
inputs = []
links = []
for name in paths:
    path = Path(name)
    assert path.is_file() and not path.is_symlink()
    raw = path.read_bytes()
    inputs.append(seal(name, raw))
    if name not in configured:
        scan(name, raw)
    if path.name in {'corpus.json', 'corpus-receivers.json', 'source-witnesses.json'}:
        values(json.loads(raw), name)
    if path.suffix == '.md':
        for target in re.findall(r'(?<!!)\[[^\]]*\]\(([^\n)]+)\)', raw.decode()):
            target = target.strip().strip('<>')
            if re.match(r'^[a-zA-Z][a-zA-Z0-9+.-]*:', target) or target.startswith('#'):
                continue
            destination = urllib.parse.unquote(target.split('#', 1)[0])
            links.append({'document': name, 'target': target, 'exists': (path.parent/destination).resolve().exists()})

verification = json.loads((OUT/'verification.json').read_bytes())
assert verification['actual_executions'] == executions
assert verification['source_epochs'] == len(epochs) and verification['source_preimages'] == preimages
for name, row in verification['current_checks'].items():
    assert seal(row['path']) == row
    result = json.loads(Path(row['path']).read_bytes())
    assert result['status'] == 'passed'
    receipt = json.loads(Path(result['source_receipt']['path']).read_bytes())
    for source in receipt['sources']:
        p = source['path']
        relevant = p.startswith('go-tools/go2vir/') if name in {'native', 'vet', 'go-format'} else p.startswith('crates/') and ('/tests/' not in p or Path(p).name in {'go_practical_contract_modules.rs','go_practical_source_signatures.rs','go_practical_contract_expressions.rs','go_practical_source_overlays.rs','go_practical_source_scopes.rs','go_practical_source_domains.rs','go_practical_error_dispatch.rs','go_practical_source_selection.rs'})
        if relevant:
            assert seal(p) == {k:source[k] for k in ['path', 'bytes', 'sha256']} and Path(p).stat().st_mode & 0o777 == source['mode'], (name, p)

report = {'status': 'passed' if not findings and all(r['exists'] for r in links) else 'failed',
          'basis_commit': BASE, 'matcher_count': len(matchers), 'scopes': len(module.SCOPES), 'seals': len(manifest['sealed_records']), 'exclusions': len(exclusions),
          'scanner_rules_unchanged': True, 'strict_whole_repository_gate_passed': False,
          'actual_driver_prelaunch_sha256': hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
          'inputs': inputs, 'source_epochs': len(epochs), 'source_preimages': preimages, 'executions': executions,
          'native_original_files_compared_literally': len(virtual), 'native_original_bytes_compared_literally': sum(len(v[0]) for v in virtual.values()),
          'decoded': decoded, 'findings': findings, 'preexisting_basis_findings': baseline, 'links': links}
REPORT.write_text(json.dumps(report, indent=2)+'\n')
print(json.dumps({'status': report['status'], 'epochs': len(epochs), 'preimages': preimages, 'executions': len(executions), 'findings': len(findings), 'basis_findings': len(baseline), 'missing_links': sum(not r['exists'] for r in links)}))
raise SystemExit(report['status'] != 'passed')
