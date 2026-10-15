import hashlib
import json
import pathlib
import subprocess
import sys
import time

base = pathlib.Path(__file__).parent
mode = sys.argv[1]
assert mode in ['old', 'new', 'new-2']
root = pathlib.Path('/private/tmp/mpk-w09-context-application-integration')
request_root = base / 'before-inputs' if mode == 'old' else (base / 'request-candidates-6' if mode == 'new-2' else root)
response_root = base / 'before-responses' if mode == 'old' else base / ('response-candidates-5' if mode == 'new-2' else 'response-candidates-3')
binary = base / ('replay-facts-old' if mode == 'old' else 'replay-facts-new')
out = base / ('facts-replay-' + mode)
out.mkdir(exist_ok=False)
plan = base / ('response-candidate-plan-2.json' if mode == 'new-2' else 'response-candidate-plan.json')
paths = [binary, plan, base / 'replay-facts.rs']
for row in json.loads(plan.read_bytes()):
    paths.extend([request_root / row['request_path'], response_root / row['response_path']])
paths.extend(root.glob('crates/mpk-vc/src/**/*.rs'))
paths.append(root / 'crates/mpk-cli/tests/support/csharp_practical_data_context.rs')
digest = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
manifest = {str(p): digest(p) for p in sorted(set(paths))}
(out / 'source-manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
started = time.time()
with (out / 'stdout.jsonl').open('wb') as stdout, (out / 'stderr.txt').open('wb') as stderr:
    process = subprocess.Popen([str(binary), str(plan), str(request_root), str(response_root),
        str(out / 'completion.txt')], stdout=stdout, stderr=stderr)
    (out / 'status.json').write_text(json.dumps({'status':'running', 'pid':process.pid,
        'started_unix':started}, indent=2) + '\n')
    code = process.wait()
unchanged = all(digest(pathlib.Path(p)) == h for p, h in manifest.items())
completed = code == 0 and unchanged and (out / 'completion.txt').read_bytes() == b'completed\n'
status = {'status':'passed' if completed else 'failed', 'exit_code':code,
    'source_and_input_bytes_unchanged':unchanged, 'elapsed_seconds':time.time()-started}
(out / 'status.json').write_text(json.dumps(status, indent=2) + '\n')
print(json.dumps(status), flush=True)
sys.exit(0 if completed else 1)
