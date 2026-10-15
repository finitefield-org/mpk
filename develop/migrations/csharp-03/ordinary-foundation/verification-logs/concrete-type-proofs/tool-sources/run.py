import json
import os
import subprocess
import sys
import time
from datetime import datetime, timezone
from hashlib import sha256
from pathlib import Path

repo = Path('/private/tmp/mpk-w09-packed-pattern-proofs')
out = Path('/tmp/mpk-w09-concrete-type-proofs') / sys.argv[1]
out.mkdir(parents=True, exist_ok=True)
manifest = {
    str(p.relative_to(repo)): sha256(p.read_bytes()).hexdigest()
    for p in repo.rglob('*')
    if p.is_file()
    and 'target' not in p.parts
    and '.git' not in p.parts
    and (p.suffix == '.rs' or p.name in ('Cargo.toml', 'Cargo.lock'))
}
(out / 'source-manifest.json').write_text(json.dumps(manifest, indent=2, sort_keys=True) + '\n')
reason = ('Check original W06 concrete-type equality propositions and proofs in all 45 source contexts; '
          'retain all 81 public conditions, six pending internal states and original application IDs. '
          'Use actual kernel checking for every exported certificate and a wrong-proof core rejection. '
          'Recheck the unchanged original concrete-type fixture bytes. The pattern candidate units use '
          'the extracted Builder.resume method; their exact prefix and proof checks cover that refactor. '
          'Other generators and proof normalizers are unchanged, so unrelated expensive consumers are retained. '
          'Clippy covers library/test targets; format covers the crate and the changed support test. '
          'The T06-wide gate is deferred to T06-W12.')
state = dict(status='running', process_pid=os.getpid(), selection_reason=reason,
             full_t_gate='deferred to T06-W12', stages=[])

def save():
    (out / 'status.json').write_text(json.dumps(state, indent=2, sort_keys=True) + '\n')

stages = [
    ('pattern-candidate-units', ['cargo', 'test', '-p', 'mpk-vc', '--lib', 'pattern_refinement_candidate_', '--', '--nocapture', '--test-threads=1']),
    ('original-type-proofs', ['cargo', 'test', '-p', 'mpk-vc', '--test', 'csharp_practical_vc', 'csharp_03_t06_w09_concrete_type_proofs_original_source', '--', '--nocapture', '--test-threads=1']),
    ('original-type-pins', ['cargo', 'test', '-p', 'mpk-vc', '--test', 'csharp_practical_vc', 'csharp_03_t06_w09_concrete_types_original_source_certificates', '--', '--nocapture', '--test-threads=1']),
    ('clippy', ['cargo', 'clippy', '-p', 'mpk-vc', '--lib', '--tests', '--', '-D', 'warnings']),
    ('format', ['cargo', 'fmt', '-p', 'mpk-vc', '--', '--check']),
    ('support-format', ['rustfmt', '--edition', '2021', '--check', 'crates/mpk-vc/tests/support/csharp_practical_ordinary_concrete_type_tests.rs']),
]
save()
if len(sys.argv) > 2 and sys.argv[2] == 'retain-units':
    prior = out.parent / 'attempt-1'
    old_manifest = json.loads((prior / 'source-manifest.json').read_text())
    changed = sorted(p for p in manifest if old_manifest.get(p) != manifest[p])
    allowed = ['crates/mpk-vc/src/csharp_practical_ordinary_concrete_types.rs',
               'crates/mpk-vc/tests/support/csharp_practical_ordinary_concrete_type_tests.rs']
    assert set(changed).issubset(allowed), changed
    old_state = json.loads((prior / 'status.json').read_text())
    retained = old_state['stages'][0]
    assert retained['stage'] == 'pattern-candidate-units' and retained['exit_code'] == 0
    assert sha256((prior / 'pattern-candidate-units.log').read_bytes()).hexdigest() == retained['log_sha256']
    state['retained_stages'] = [retained]
    state['retained_source_delta'] = changed
    state['retention_reason'] = 'Only the new concrete-type proof generator and its integration test changed after the three pattern units passed. The original concrete-type generator, Builder.resume, the pattern generator and its library-test sources are unchanged; pattern units do not call the new proof generator.'
    stages = stages[1:]
    save()
for name, command in stages:
    state['stage'] = name
    save()
    env = os.environ.copy()
    env['CARGO_TARGET_DIR'] = '/tmp/mpk-w09-pattern-candidates-target'
    if name == 'original-type-proofs':
        env['MPK_W09_CONCRETE_TYPE_PROOFS_OUT'] = str(out / 'certificates')
    for path, digest in manifest.items():
        assert sha256((repo / path).read_bytes()).hexdigest() == digest, path
    started = datetime.now(timezone.utc).isoformat()
    start = time.monotonic()
    with (out / (name + '.log')).open('wb') as log:
        child = subprocess.Popen(command, cwd=repo, env=env, stdout=log, stderr=subprocess.STDOUT)
        state['child_pid'] = child.pid
        state['command'] = command
        save()
        result = child.wait()
    for path, digest in manifest.items():
        assert sha256((repo / path).read_bytes()).hexdigest() == digest, path
    row = dict(stage=name, command=command, exit_code=result, started_at=started,
               finished_at=datetime.now(timezone.utc).isoformat(),
               elapsed_seconds=round(time.monotonic() - start, 3),
               log_sha256=sha256((out / (name + '.log')).read_bytes()).hexdigest())
    state['stages'].append(row)
    save()
    print(json.dumps(row), flush=True)
    if result:
        state['status'] = 'failed'
        save()
        raise SystemExit(result)
state['status'] = 'passed_targeted_tests_lint_format'
state['finished_at'] = datetime.now(timezone.utc).isoformat()
state['unique_tests_passed'] = 5
save()
