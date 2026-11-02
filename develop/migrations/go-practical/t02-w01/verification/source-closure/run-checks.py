"""Affected source-type checks; save whole original inputs before launch.
Every Git call uses /usr/bin/git and this driver runs outside the sandbox.
"""
from pathlib import Path
import datetime, fcntl, hashlib, json, os, subprocess, sys, time

ROOT = Path.cwd()
OUT = Path(__file__).resolve().parent
BASE = 'd2dcb7bf26c87daf1dae1a578ae9e5ae896a5d5e'
epoch, mode = sys.argv[1:]
assert epoch.replace('-', '').isalnum()


def git(*args):
    return subprocess.check_output(['/usr/bin/git', *args])


def seal(path):
    raw = Path(path).read_bytes()
    return {'path': str(path), 'bytes': len(raw), 'sha256': hashlib.sha256(raw).hexdigest()}


assert git('rev-parse', 'HEAD').decode().strip() == BASE
archive = OUT / epoch
archive.mkdir(exist_ok=True)
with (Path('/private/tmp') / ('mpk-go-source-closure-' + epoch + '-preimages.lock')).open('a+b') as lock:
    fcntl.flock(lock, fcntl.LOCK_EX)
    original = {}
    for row in git('ls-files', '--stage', '-z').split(b'\0'):
        if row:
            meta, name = row.split(b'\t', 1)
            permissions, oid, stage = meta.decode().split()
            assert stage == '0'
            original[name.decode()] = oid
    sources = sorted(p for root in [Path('crates'), Path('go-tools/go2vir'), Path('proofs/std'),
        OUT.relative_to(ROOT).parent/'type-keys/native-components', OUT.relative_to(ROOT).parent/'source-types/native-components', OUT.relative_to(ROOT).parent/'source-scopes/native-components']
        for p in root.rglob('*') if p.is_file()) + [Path('Cargo.toml'), Path('Cargo.lock'), OUT.relative_to(ROOT).parent/'source-signatures/corpus-binders.json', OUT.relative_to(ROOT).parent/'scalar-typing/corpus.json', OUT.relative_to(ROOT).parent/'value-typing/corpus-aliases.json', OUT.relative_to(ROOT).parent/'slice-builtins/corpus.json', OUT.relative_to(ROOT).parent/'method-typing/corpus-calls.json', OUT.relative_to(ROOT).parent/'assignment-typing/corpus.json', OUT.relative_to(ROOT).parent/'return-typing/corpus-import.json', OUT.relative_to(ROOT).parent/'source-constants/corpus.json', OUT.relative_to(ROOT).parent/'body-typing/corpus.json'] + [p.relative_to(ROOT) for p in sorted(OUT.glob('corpus*.json'))]
    inputs = []
    for path in sources:
        raw = path.read_bytes()
        row = seal(path)
        row['mode'] = path.stat().st_mode & 0o777
        oid = original.get(str(path))
        if oid and hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest() == oid:
            row.update(original_commit=BASE, original_git_blob=oid)
        else:
            target = archive / 'inputs' / path
            target.parent.mkdir(parents=True, exist_ok=True)
            if target.exists():
                assert target.read_bytes() == raw
            else:
                target.write_bytes(raw)
                target.chmod(row['mode'])
            row['complete_changed_input'] = str(target.relative_to(ROOT))
        inputs.append(row)
    driver = archive / 'run-checks.py'
    if driver.exists():
        assert driver.read_bytes() == Path(__file__).read_bytes()
    else:
        driver.write_bytes(Path(__file__).read_bytes())
    receipt = {'basis_commit': BASE, 'sources': inputs, 'actual_driver_prelaunch': seal(driver)}
    input_path = archive / 'inputs.json'
    if input_path.exists():
        assert json.loads(input_path.read_bytes()) == receipt
    else:
        input_path.write_text(json.dumps(receipt, indent=2) + '\n')

changed=['practical_closure.go','practical_closure_test.go']
goroot=Path('/private/tmp/mpk-go-practical-original-packages-sdk-20261021/go');go=str(goroot/'bin/go')
commands={
 'format-write':[str(goroot/'bin/gofmt'),'-w',*changed],
 'format':[str(goroot/'bin/gofmt'),'-l',*changed],
 'compile':[go,'test','-run','^$'],
 'closure':[go,'test','-json','-count=1','-timeout=40m','-run','^TestPracticalSourceClosure'],
 'owner-regression':[go,'test','-json','-count=1','-timeout=40m','-run','^TestPracticalOriginalPackages'],
 'vet':[go,'vet','./...'],
 'frontend':[go,'test','-json','-count=1','-timeout=20m','-run','^TestPracticalAnalysisCapturedInputsRebuildTheSameActualFrontend$'],
 'typecheck-regression':[go,'test','-json','-count=1','-timeout=40m','-run','^TestPracticalSourceTypes(RetainAll81OriginalGoTypeResults|RejectErrorInUnusedDependencyBody)$'],
}
assert mode in commands
is_go=True
environment={'GOROOT':str(goroot),'GOTOOLCHAIN':'local','GOENV':'off','GOWORK':'off','GOFLAGS':'-mod=vendor','GOPROXY':'off','GOSUMDB':'off','CGO_ENABLED':'0','GOMAXPROCS':'1','GOCACHE':'/private/tmp/mpk-go-practical-t02-go-cache','GOMODCACHE':'/private/tmp/mpk-go-practical-t02-go-mod'}
if mode in {'closure','owner-regression','typecheck-regression','frontend'}:
 output=Path('/private/tmp')/('mpk-go-source-closure-'+epoch+'-'+mode+'-outputs');output.mkdir(exist_ok=False);environment['MPK_GO_PRACTICAL_ANALYSIS_EVIDENCE' if mode=='frontend' else 'MPK_GO_PRACTICAL_SYNTAX_EVIDENCE']=str(output)
env = os.environ.copy()
env.update(environment)
directory = archive / mode
directory.mkdir(exist_ok=False)
result = directory / 'result.json'
record = {
    'status': 'running', 'command': commands[mode], 'cwd': 'go-tools/go2vir' if is_go else '.', 'environment': environment,
    'sources': inputs, 'source_receipt': seal(input_path), 'actual_driver_prelaunch': seal(driver),
    'selection_reason':'New private original-package source/call/declaration closure observer reads intact owned native packages. Validate only new closure cases and affected original-owner consumers, Go format/vet. No Rust/source evaluator/ordinary/kernel semantics changed, so those unrelated tests are not rerun. Full T gate stays W04.',
    'formatter_mutation_expected': mode=='format-write',
    'full_T_gate': 'deferred to GO-PRAC-T02-W04: ./scripts/check-fast.sh',
    'started_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
}
result.write_text(json.dumps(record, indent=2) + '\n')
start = time.monotonic()
with (directory / 'stdout.log').open('wb') as stdout, (directory / 'stderr.log').open('wb') as stderr:
    code = subprocess.run(commands[mode], cwd=record['cwd'], env=env, stdout=stdout, stderr=stderr, check=False).returncode
record.update(status='passed' if code == 0 else 'failed', exit_code=code,
    seconds=time.monotonic()-start, finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),
    stdout=seal(directory/'stdout.log'), stderr=seal(directory/'stderr.log'),
    whole_source_inputs_unchanged=all(seal(r['path']) == {k:r[k] for k in ['path','bytes','sha256']}
        and Path(r['path']).stat().st_mode & 0o777 == r['mode'] for r in inputs))
if is_go and mode in {'closure','owner-regression','typecheck-regression','frontend'}:
    events=[json.loads(line)for line in(directory/'stdout.log').read_text().splitlines()]
    record['passed_tests']=[e['Test']for e in events if e['Action']=='pass'and'Test'in e]
    record['failed_tests']=[e['Test']for e in events if e['Action']=='fail'and'Test'in e]
    record['skipped_tests']=[e['Test']for e in events if e['Action']=='skip'and'Test'in e]
    if not record['passed_tests']or record['failed_tests']or record['skipped_tests']:record['status']='failed'
if mode=='format'and(directory/'stdout.log').stat().st_size:record['status']='failed'
if not record['formatter_mutation_expected'] and not record['whole_source_inputs_unchanged']:
    record['status'] = 'failed'
result.write_text(json.dumps(record, indent=2) + '\n')
print(json.dumps({k:record[k] for k in ['status','exit_code','seconds','whole_source_inputs_unchanged']}))
raise SystemExit(record['status'] != 'passed')
