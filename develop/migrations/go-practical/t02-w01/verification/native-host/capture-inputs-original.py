"""Outside sandbox: retain original Git source bytes plus complete changed inputs.

Unchanged original inputs remain in their actual immutable Git blobs; both the
current whole bytes and those blobs are read and compared, not inferred from
names or a self-hash. New/changed complete bytes are additionally retained.
"""
from pathlib import Path
import base64, datetime, hashlib, json, subprocess
root = Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
base = '528b599aacfe439d4abd8f50e046a754c9a24435'
def git(*args):
    return subprocess.run(['/usr/bin/git', *args], check=True, capture_output=True).stdout
assert git('rev-parse', 'HEAD').decode().strip() == base
assert git('branch', '--show-current').decode().strip() == 'codex/go-practical'
files = {p for p in Path('crates').glob('**/*.rs')}
files.update(Path('crates').glob('**/Cargo.toml'))
files.update([Path('Cargo.toml'), Path('Cargo.lock')])
frozen = json.loads(Path('develop/migrations/go-practical/t02-w01/verification/native-observation/frozen-design-preservation.json').read_text())
files.update(Path(row['path']) for row in frozen['inputs'])
tracked = {}
for raw in git('ls-tree', '-r', '-z', base).split(b'\0'):
    if not raw: continue
    info, path = raw.split(b'\t'); mode, kind, oid = info.decode().split()
    tracked[path.decode()] = (kind, oid)
rows = []
with subprocess.Popen(['/usr/bin/git', 'cat-file', '--batch'], stdin=subprocess.PIPE, stdout=subprocess.PIPE) as process:
    for path in sorted(files):
        name, data = str(path), path.read_bytes()
        digest = hashlib.sha256(data).hexdigest()
        row = {'path': name, 'sha256': digest, 'bytes': len(data)}
        original = tracked.get(name)
        equal = False
        if original is not None:
            kind, oid = original; assert kind == 'blob'
            process.stdin.write(oid.encode() + b'\n'); process.stdin.flush()
            fields = process.stdout.readline().decode().split(); assert fields[:2] == [oid, 'blob']
            source = process.stdout.read(int(fields[2])); assert process.stdout.read(1) == b'\n'
            assert hashlib.sha1(b'blob ' + str(len(source)).encode() + b'\0' + source).hexdigest() == oid
            row['original_git_blob'] = oid
            row['original_git_blob_sha256'] = hashlib.sha256(source).hexdigest()
            row['original_git_blob_bytes'] = len(source)
            equal = source == data
        row['whole_current_bytes_equal_original_git_blob'] = equal
        if not equal:
            out = root / 'inputs' / path
            out.parent.mkdir(parents=True, exist_ok=True); out.write_bytes(data)
            row['complete_changed_input'] = str(out)
        rows.append(row)
    process.stdin.close(); assert process.wait() == 0
assert sum(not row['whole_current_bytes_equal_original_git_blob'] for row in rows) == 4
(root / 'inputs.json').write_text(json.dumps({'status': 'complete_original_inputs_retained', 'base_commit': base,
    'observed_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(), 'sources': rows,
    'all_unchanged_original_git_blobs_actually_read': True, 'new_or_changed_complete_inputs_retained': 4,
    'unavailable_original_input_replaced_by_a_fresh_run': False, 'CSharp_tests_executed': False}, indent=2) + '\n')
sdk = Path('/private/tmp/mpk-go-practical-toolchains/go1.25.0-darwin-arm64/go')
for relative in ['src/runtime/os_linux.go', 'src/syscall/exec_linux.go', 'src/os/pidfd_linux.go']:
    data = (sdk / relative).read_bytes(); out = root / 'pinned-go' / (relative.replace('/', '-') + '.bytes.json')
    out.parent.mkdir(exist_ok=True)
    out.write_text(json.dumps({'schema': 'mpk.original_bytes.v1', 'original_path': str(sdk / relative),
        'bytes': len(data), 'raw_sha256': hashlib.sha256(data).hexdigest(), 'base64': base64.b64encode(data).decode()}, indent=2) + '\n')
print(json.dumps({'status': 'passed', 'whole_original_inputs': len(rows), 'changed_full_inputs': 4}))
