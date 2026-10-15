import json
import os
import subprocess
import sys
import time
from datetime import datetime, timezone
from hashlib import sha256
from pathlib import Path

repo = Path('/private/tmp/mpk-w09-operation-proof-transport')
out = Path('/tmp/mpk-w09-concrete-operation-proofs') / sys.argv[1]
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
reason = ('Check complete original W06 concrete-definition equivalence sequents in all 45 source contexts: '
          'all input domains and normal guards, every ordered first-failure and success equality, and the complete normal result. '
          'Require actual Rust kernel acceptance for every emitted certificate and core rejection of a well-typed wrong implementation '
          'whose definition-only certificate remains accepted. Preserve all 436 original defined operations, 31 pending operations '
          'and all 987 application IDs. Recheck all 45 original operation pins; the appended proof route does not change the old generator '
          'or shared Builder.resume. Existing type proof generators and pattern routes are unchanged. Clippy covers library/tests '
          'and crate/support format checks run. The full T06 gate remains deferred to W12.')
state = dict(status='running', process_pid=os.getpid(), selection_reason=reason,
             full_t_gate='deferred to T06-W12', stages=[])

def save():
    (out / 'status.json').write_text(json.dumps(state, indent=2, sort_keys=True) + '\n')

stages = [
    ('original-operation-proofs', ['cargo', 'test', '-p', 'mpk-vc', '--test', 'csharp_practical_vc', 'csharp_03_t06_w09_concrete_operation_proofs_original_source', '--', '--nocapture', '--test-threads=1']),
    ('clippy', ['cargo', 'clippy', '-p', 'mpk-vc', '--lib', '--tests', '--', '-D', 'warnings']),
    ('format', ['cargo', 'fmt', '-p', 'mpk-vc', '--', '--check']),
    ('support-format', ['rustfmt', '--edition', '2021', '--check', 'crates/mpk-vc/tests/support/csharp_practical_ordinary_concrete_operation_tests.rs']),
]
save()
for name, command in stages:
    state['stage'] = name
    save()
    env = os.environ.copy()
    env.update(CARGO_BUILD_JOBS='2', CARGO_PROFILE_TEST_OPT_LEVEL='2', CARGO_PROFILE_TEST_DEBUG='0', CARGO_PROFILE_TEST_DEBUG_ASSERTIONS='true', CARGO_PROFILE_TEST_OVERFLOW_CHECKS='true')
    env['CARGO_TARGET_DIR'] = '/tmp/mpk-w09-operation-proof-transport-target'
    if name == 'original-operation-proofs':
        env['MPK_W09_CONCRETE_OPERATION_PROOFS_OUT'] = str(out / 'certificates')
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
state['unique_tests_passed'] = 2
prior = json.loads((out.parent / 'transport-attempt-1/status.json').read_bytes())
assert prior['stages'][1]['stage'] == 'original-operation-pins' and prior['stages'][1]['exit_code'] == 0
old_manifest = json.loads((out.parent / 'transport-attempt-1/source-manifest.json').read_bytes())
delta = [name for name in manifest if old_manifest.get(name) != manifest[name]]
assert delta == ['crates/mpk-vc/src/csharp_practical_ordinary_concrete_operation_proofs.rs']
state['retained_stages'] = [prior['stages'][1]]
state['retained_source_delta'] = delta
state['retention_reason'] = 'Only the new proof helper argument grouping changed after the original-operation pin test passed. Original source definitions, generator, test and fixture bytes are unchanged; all 45 original pin results are retained. The affected proof/import integration, Clippy and format are rerun.'
save()
