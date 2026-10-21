"""Affected source-declaration checks; save whole original inputs before launch.
Every Git call uses /usr/bin/git and this driver runs outside the sandbox.
"""
from pathlib import Path
import datetime, fcntl, hashlib, json, os, subprocess, sys, time

ROOT = Path.cwd()
OUT = Path(__file__).resolve().parent
BASE = '900c08645d0d54666d260d2f2914f680c8e09050'
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
with (Path('/private/tmp') / ('mpk-go-source-declarations-' + epoch + '-preimages.lock')).open('a+b') as lock:
    fcntl.flock(lock, fcntl.LOCK_EX)
    original = {}
    for row in git('ls-files', '--stage', '-z').split(b'\0'):
        if row:
            meta, name = row.split(b'\t', 1)
            permissions, oid, stage = meta.decode().split()
            assert stage == '0'
            original[name.decode()] = oid
    sources = sorted(p for root in [Path('crates'), Path('go-tools/go2vir'), Path('proofs/std'),
        Path('develop/migrations/go-practical/t02-w01/verification/type-keys/native-components')]
        for p in root.rglob('*') if p.is_file()) + [Path('Cargo.toml'), Path('Cargo.lock')]
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

changed = ['crates/mpk-vc/src/go_practical_type_keys.rs',
           'crates/mpk-vc/src/go_practical_type_keys/source_declarations.rs',
           'crates/mpk-vc/tests/go_practical_type_keys.rs']
commands = {
    'rust-format-write': ['rustfmt', '--edition=2021', *changed],
    'rust-selected': ['cargo', 'test', '--offline', '-p', 'mpk-vc', '--test', 'go_practical_type_keys', '--', '--nocapture'],
    'clippy': ['cargo', 'clippy', '--offline', '-p', 'mpk-vc', '--lib', '--test', 'go_practical_type_keys', '--', '-D', 'warnings'],
    'rust-format': ['rustfmt', '--edition=2021', '--check', *changed],
}
assert mode in commands
environment = {}
if mode == 'rust-selected':
    output = Path('/private/tmp') / ('mpk-go-source-declarations-' + epoch + '-rust-outputs')
    output.mkdir(exist_ok=False)
    environment['MPK_GO_TYPE_KEYS_RUST_OUTPUT'] = str(output)
env = os.environ.copy()
env.update(environment)
directory = archive / mode
directory.mkdir(exist_ok=False)
result = directory / 'result.json'
record = {
    'status': 'running', 'command': commands[mode], 'cwd': '.', 'environment': environment,
    'sources': inputs, 'source_receipt': seal(input_path), 'actual_driver_prelaunch': seal(driver),
    'selection_reason': 'The Rust component importer now independently parses all candidate package Go files and binds every declaration to exact original TypeSpec/name/alias syntax. Run all affected five existing importer roots plus new rehashed source/package/declaration/alias/generic and whole-file controls, actual native five-stream membership and constructed physical/local/Unicode/untyped boundary cases. Strict lib/test Clippy and changed-source format cover consumers. The parser and lexer implementations, Go production/test sources, 189-input frontend, SDK, ordinary proof/core/evaluator and registry remain unchanged; retain their passed evidence without unrelated reexecution. No source name/type/constant/module/capture/program proof inferred and no intermediate whole/equivalent T gate.',
    'formatter_mutation_expected': mode == 'rust-format-write',
    'full_T_gate': 'deferred to GO-PRAC-T02-W04: ./scripts/check-fast.sh',
    'started_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
}
result.write_text(json.dumps(record, indent=2) + '\n')
start = time.monotonic()
with (directory / 'stdout.log').open('wb') as stdout, (directory / 'stderr.log').open('wb') as stderr:
    code = subprocess.run(commands[mode], env=env, stdout=stdout, stderr=stderr, check=False).returncode
record.update(status='passed' if code == 0 else 'failed', exit_code=code,
    seconds=time.monotonic()-start, finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),
    stdout=seal(directory/'stdout.log'), stderr=seal(directory/'stderr.log'),
    whole_source_inputs_unchanged=all(seal(r['path']) == {k:r[k] for k in ['path','bytes','sha256']}
        and Path(r['path']).stat().st_mode & 0o777 == r['mode'] for r in inputs))
if not record['formatter_mutation_expected'] and not record['whole_source_inputs_unchanged']:
    record['status'] = 'failed'
result.write_text(json.dumps(record, indent=2) + '\n')
print(json.dumps({k:record[k] for k in ['status','exit_code','seconds','whole_source_inputs_unchanged']}))
raise SystemExit(record['status'] != 'passed')
