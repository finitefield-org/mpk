"""Affected local checks with complete original inputs saved before launch.

Run outside the sandbox because every Git call uses /usr/bin/git.
"""
from pathlib import Path
import datetime
import hashlib
import json
import os
import subprocess
import sys
import time

ROOT = Path.cwd()
OUT = Path(__file__).resolve().parent
BASE = '4b6ded3931d37aa5b9fec44244d097ab63e29f2b'
epoch, mode = sys.argv[1:]
assert epoch.replace('-', '').isalnum()


def git(*args):
    return subprocess.check_output(['/usr/bin/git', *args])


def seal(path):
    raw = Path(path).read_bytes()
    return {'path': str(path), 'bytes': len(raw), 'sha256': hashlib.sha256(raw).hexdigest()}


assert git('rev-parse', 'HEAD').decode().strip() == BASE
archive = OUT / epoch
archive.mkdir(exist_ok=False)
tracked = {}
for row in git('ls-files', '--stage', '-z').split(b'\0'):
    if row:
        meta, name = row.split(b'\t', 1)
        permissions, oid, stage = meta.decode().split()
        assert stage == '0'
        tracked[name.decode()] = oid
sources = sorted(p for root in [Path('crates'), Path('proofs/std'), Path('go-tools/go2vir')]
                 for p in root.rglob('*') if p.is_file())
sources += [Path('Cargo.toml'), Path('Cargo.lock'), Path(__file__).relative_to(ROOT)]
for group in ['contract-expressions', 'source-overlays', 'source-scopes', 'source-selection', 'source-domains', 'error-dispatch']:
    sources += sorted((OUT.parent / group).glob('corpus*.json'))
sources += sorted(OUT.glob('source-witnesses.json'))
inputs = []
for path in sorted(set(p.relative_to(ROOT) if p.is_absolute() else p for p in sources)):
    raw = path.read_bytes()
    row = seal(path)
    row['mode'] = path.stat().st_mode & 0o777
    oid = tracked.get(str(path))
    if oid and hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest() == oid:
        row.update(original_commit=BASE, original_git_blob=oid)
    else:
        target = archive / 'inputs' / path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(raw)
        target.chmod(row['mode'])
        row['complete_changed_input'] = str(target.relative_to(ROOT))
    inputs.append(row)
driver = archive / 'driver.py'
driver.write_bytes(Path(__file__).read_bytes())
receipt = archive / 'inputs.json'
receipt.write_text(json.dumps({'basis_commit': BASE, 'sources': inputs, 'actual_driver_prelaunch': seal(driver)}, indent=2) + '\n')

changed = [
    'crates/mpk-vc/src/go_practical_type_keys.rs',
    'crates/mpk-vc/src/go_practical_type_keys/source_scopes.rs',
    'crates/mpk-vc/src/go_practical_type_keys/source_types.rs',
    'crates/mpk-vc/src/go_practical_type_keys/source_scalar_typing.rs',
    'crates/mpk-vc/src/go_practical_type_keys/source_scalar_typing/selection.rs',
    'crates/mpk-vc/src/go_practical_type_keys/source_scalar_typing/domains.rs',
    'crates/mpk-vc/src/go_practical_type_keys/source_overlays.rs',
    'crates/mpk-vc/src/go_practical_type_keys/source_overlays/expressions.rs',
]
test = Path('crates/mpk-vc/tests/go_practical_contract_expressions.rs')
if test.exists():
    changed.append(str(test))
goroot = Path('/private/tmp/mpk-go-practical-original-packages-sdk-20261021/go')
go = str(goroot / 'bin/go')
commands = {
    'check': ['cargo', 'check', '--offline', '-p', 'mpk-vc', '--lib'],
    'format-write': ['rustfmt', '--edition=2021', *changed],
    'format': ['rustfmt', '--edition=2021', '--check', *changed],
    'go-format-write': [str(goroot/'bin/gofmt'), '-w', 'practical_contract_expressions_test.go'],
    'go-format': [str(goroot/'bin/gofmt'), '-l', 'practical_contract_expressions_test.go'],
    'go-native': [go, 'test', '-json', '-count=1', '-timeout=40m', '-run', '^TestPracticalContractExpressionOriginalScopes$'],
    'vet': [go, 'vet', './...'],
    'clippy': ['cargo', 'clippy', '--offline', '-p', 'mpk-vc', '--lib', '--test', 'go_practical_contract_expressions', '--test', 'go_practical_source_overlays', '--test', 'go_practical_source_scopes', '--test', 'go_practical_source_domains', '--test', 'go_practical_error_dispatch', '--test', 'go_practical_source_selection', '--', '-D', 'warnings'],
}
tests = ['contract_expressions', 'source_overlays', 'source_scopes', 'source_domains', 'error_dispatch', 'source_selection']
commands.update({'rust-'+name: ['cargo', 'test', '--offline', '-p', 'mpk-vc', '--test', 'go_practical_'+name, '--', '--nocapture'] for name in tests})
affected = ['source_overlays', 'source_scopes', 'source_domains', 'error_dispatch', 'source_selection']
commands['rust-affected'] = ['cargo', 'test', '--offline', '-p', 'mpk-vc', *[part for name in affected for part in ['--test', 'go_practical_'+name]], '--', '--nocapture']
is_go = mode in {'go-format-write', 'go-format', 'go-native', 'vet'}
environment = {'GOROOT': str(goroot), 'GOTOOLCHAIN': 'local', 'GOENV': 'off', 'GOWORK': 'off', 'GOFLAGS': '-mod=vendor', 'GOPROXY': 'off', 'GOSUMDB': 'off', 'CGO_ENABLED': '0', 'GOMAXPROCS': '1', 'GOCACHE': '/private/tmp/mpk-go-practical-t02-go-cache', 'GOMODCACHE': '/private/tmp/mpk-go-practical-t02-go-mod'} if is_go else {}
if mode == 'go-native' or mode.startswith('rust-'):
    evidence = archive / 'observations'
    evidence.mkdir()
    key = 'MPK_GO_PRACTICAL_SYNTAX_EVIDENCE' if mode == 'go-native' else 'MPK_GO_'+mode[5:].upper()+'_RUST_OUTPUT'
    environment[key] = str(evidence)
    if mode == 'rust-affected':
        for name in affected:
            path = evidence/name
            path.mkdir()
            environment['MPK_GO_'+name.upper()+'_RUST_OUTPUT'] = str(path)
record = {
    'status': 'running', 'command': commands[mode], 'cwd': 'go-tools/go2vir' if is_go else '.', 'environment': environment,
    'source_receipt': seal(receipt), 'actual_driver_prelaunch': seal(driver),
    'selection_reason': 'New original contract-expression typing extends the source domain and caches checked local types. Check all70 heads, lexical/header/old namespaces, native original Go header bindings, and affected overlay/scopes/domain/selection consumers. No production Go frontend change; unrelated semantics and frontend rebuilds are unnecessary.',
    'full_T_gate': 'Deferred to final GO-PRAC-T02-W04: ./scripts/check-fast.sh',
    'formatter_mutation_expected': mode.endswith('format-write'),
    'started_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
}
result = archive / 'result.json'
result.write_text(json.dumps(record, indent=2) + '\n')
env = os.environ.copy()
env.update(environment)
start = time.monotonic()
with (archive/'stdout.log').open('wb') as stdout, (archive/'stderr.log').open('wb') as stderr:
    code = subprocess.run(commands[mode], cwd=record['cwd'], env=env, stdout=stdout, stderr=stderr, check=False).returncode
record.update(status='passed' if code == 0 else 'failed', exit_code=code, seconds=time.monotonic()-start,
              stdout=seal(archive/'stdout.log'), stderr=seal(archive/'stderr.log'),
              whole_source_inputs_unchanged=all(seal(r['path']) == {k:r[k] for k in ['path','bytes','sha256']}
                  and Path(r['path']).stat().st_mode & 0o777 == r['mode'] for r in inputs),
              finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat())
if not record['formatter_mutation_expected'] and not record['whole_source_inputs_unchanged']:
    record['status'] = 'failed'
if mode == 'go-format' and (archive/'stdout.log').stat().st_size:
    record['status'] = 'failed'
if mode == 'go-native':
    events = [json.loads(line) for line in (archive/'stdout.log').read_text().splitlines()]
    for action in ['pass', 'fail', 'skip']:
        record[action+'_tests'] = [e['Test'] for e in events if e['Action'] == action and 'Test' in e]
    if not record['pass_tests'] or record['fail_tests'] or record['skip_tests']:
        record['status'] = 'failed'
result.write_text(json.dumps(record, indent=2) + '\n')
print(json.dumps({k:record[k] for k in ['status', 'exit_code', 'seconds', 'whole_source_inputs_unchanged']}))
raise SystemExit(record['status'] != 'passed')
