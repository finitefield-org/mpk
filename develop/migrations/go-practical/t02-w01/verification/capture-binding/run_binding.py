"""Seal affected inputs and run only the local binding and root tests."""
from pathlib import Path
import hashlib
import json
import os
import subprocess
import sys
import tarfile
import time

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[5]
LIVE = ROOT.parent / 'source-root-live'
TEMP = Path('/private/tmp/mpk-go-source-root-matching')
CARGO_REGISTRY = Path('/Users/kazuyoshitoshiya/.cargo/registry')
IMAGE = 'sha256:7cf1e580ef5539f03b58560753e8ab84c8c360960d99dff714004aa98f203977'


def seal(path):
    digest = hashlib.sha256()
    with path.open('rb') as source:
        for block in iter(lambda: source.read(1024 * 1024), b''):
            digest.update(block)
    return {'path': str(path.resolve()), 'bytes': path.stat().st_size, 'sha256': digest.hexdigest(), 'mode': path.stat().st_mode & 0o777}


def main():
    epoch, mode = sys.argv[1:]
    assert epoch.replace('-', '').isalnum() and Path.cwd() == REPO
    out = ROOT / epoch
    out.mkdir(exist_ok=False)
    paths = {Path(__file__).resolve(), ROOT / 'restore_fixture.py', LIVE / 'restore_evidence.py', REPO / 'Cargo.toml', REPO / 'Cargo.lock', REPO / 'crates/mpk-cli/tests/go_practical_capture_binding.rs', REPO / 'crates/mpk-cli/tests/go_practical_source_roots.rs'}
    for crate in ['mpk-cli', 'mpk-api', 'mpk-cert', 'mpk-core', 'mpk-kernel', 'mpk-linux-sandbox', 'mpk-theory', 'mpk-vc']:
        base = REPO / 'crates' / crate
        paths.add(base / 'Cargo.toml')
        paths.update(p for p in (base / 'src').rglob('*') if p.is_file())
        paths.update(p for p in base.glob('build.rs') if p.is_file())
    paths.update(p for p in (REPO / 'release/bundles').rglob('*') if p.is_file())
    paths.update(p for p in (REPO / 'go-tools/go2vir').rglob('*') if p.is_file())
    probe = REPO / 'develop/probes/go-practical'
    sys.dont_write_bytecode = True
    sys.path.insert(0, str(probe))
    from frontend_native_build import linked_inputs
    paths.update(REPO / row['path'] for row in linked_inputs())
    paths.update(probe / name for name in ['frontend_native_build.py', 'run_proof_pilot.py', 'check_scope_report.py'])
    live = LIVE / 'live-original'
    paths.update([live / 'coord/snapshot-ready.json', live / 'evidence-storage.json', live / 'original-evidence-manifest.json'])
    chunks = sorted((live / 'evidence-storage').glob('*.hex'))
    assert len(chunks) == 24
    paths.update(chunks)
    before = [seal(p) for p in sorted(paths)]
    # The already retained 24 lossless chunks remain the exact-byte archive;
    # seal them above and below without duplicating their 194 MB encoding.
    with tarfile.open(out / 'inputs.tar.gz', 'x:gz', dereference=True) as archive:
        for path in sorted(paths - set(chunks)):
            archive.add(path, arcname=str(path.relative_to(REPO)), recursive=False)
    if mode in ('check', 'clippy'):
        selected = {'check': ['cargo', 'check', '--offline', '--locked', '-p', 'mpk-cli', '--test', 'go_practical_capture_binding', '--target', 'x86_64-unknown-linux-gnu'], 'clippy': ['cargo', 'clippy', '--offline', '--locked', '-p', 'mpk-cli', '--lib', '--test', 'go_practical_capture_binding', '--target', 'x86_64-unknown-linux-gnu', '--', '-D', 'warnings']}
        command = selected[mode]
        environment = {'CARGO_TARGET_DIR': '/private/tmp/mpk-go-source-root-clippy-target', 'CARGO_NET_OFFLINE': 'true', 'CARGO_INCREMENTAL': '0', 'CARGO_PROFILE_DEV_DEBUG': '0'}
    else:
        TEMP.mkdir(exist_ok=True)
        cargo = TEMP / 'cargo'; cargo.mkdir(exist_ok=True); (cargo / 'registry').mkdir(exist_ok=True)
        target = TEMP / 'target'; target.mkdir(exist_ok=True)
        environment = {'CARGO_HOME': '/cargo', 'CARGO_TARGET_DIR': '/target', 'CARGO_NET_OFFLINE': 'true', 'CARGO_INCREMENTAL': '0', 'CARGO_PROFILE_TEST_DEBUG': '0', 'CARGO_PROFILE_TEST_OPT_LEVEL': '1', 'RUST_TEST_THREADS': '1', 'HOME': '/tmp'}
        command = ['/usr/local/bin/docker', 'run', '--pull=never', '--rm', '--platform', 'linux/arm64', '--network', 'none', '--read-only', '--cap-drop', 'ALL', '--security-opt', 'no-new-privileges', '--user', f'{os.getuid()}:{os.getgid()}', '--tmpfs', '/tmp:rw,exec,nosuid,nodev,size=2g,mode=1777', '--mount', f'type=bind,src={REPO},dst=/workspace,readonly', '--mount', f'type=bind,src={cargo},dst=/cargo', '--mount', f'type=bind,src={CARGO_REGISTRY},dst=/cargo/registry,readonly', '--mount', f'type=bind,src={target},dst=/target', '--workdir', '/workspace']
        for key, value in environment.items(): command.extend(['--env', f'{key}={value}'])
        selected = {'binding': ['cargo', 'test', '--offline', '--locked', '-p', 'mpk-cli', '--test', 'go_practical_capture_binding', 'complete_capture_binding_rejects_coherent_mismatches', '--', '--nocapture'], 'roots': ['cargo', 'test', '--offline', '--locked', '-p', 'mpk-cli', '--test', 'go_practical_source_roots', '--', '--nocapture']}
        command.extend([IMAGE, *selected[mode]])
    reason = 'Verify the changed compiler/root binding and affected root consumer using all retained original app, SDK, package graph, source objects and observed namespaces; retain all 24 existing exact evidence chunks. This does not run the T gate or establish native Linux release acceptance.'
    (out / 'inputs.json').write_text(json.dumps({'selection_reason': reason, 'inputs': before, 'archive': seal(out / 'inputs.tar.gz'), 'command': command, 'environment': environment}, indent=2) + '\n')
    report = {'status': 'running', 'full_T_gate': 'Deferred to final T02-W04: ./scripts/check-fast.sh', 'native_Linux_release_acceptance': False}
    (out / 'result.json').write_text(json.dumps(report, indent=2) + '\n')
    start = time.monotonic()
    with (out / 'stdout.log').open('xb') as stdout, (out / 'stderr.log').open('xb') as stderr:
        code = subprocess.run(command, cwd=REPO, env={**os.environ, **environment} if mode in ('check', 'clippy') else os.environ, stdout=stdout, stderr=stderr, check=False).returncode
    unchanged = before == [seal(p) for p in sorted(paths)]
    report.update(status='passed' if code == 0 and unchanged else 'failed', exit_code=code, seconds=time.monotonic()-start, original_inputs_unchanged=unchanged, stdout=seal(out/'stdout.log'), stderr=seal(out/'stderr.log'))
    (out / 'result.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps({k: report[k] for k in ['status', 'exit_code', 'seconds', 'original_inputs_unchanged']}), flush=True)
    raise SystemExit(report['status'] != 'passed')


if __name__ == '__main__': main()
