"""Record inputs and run the affected Linux source-root tests locally."""
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
TEMP = Path('/private/tmp/mpk-go-source-root-matching')
CARGO_REGISTRY = Path('/Users/kazuyoshitoshiya/.cargo/registry')
IMAGE = 'sha256:7cf1e580ef5539f03b58560753e8ab84c8c360960d99dff714004aa98f203977'


def seal(path):
    data = path.read_bytes()
    return {'path': str(path.resolve()), 'bytes': len(data), 'sha256': hashlib.sha256(data).hexdigest(), 'mode': path.stat().st_mode & 0o777}


def main():
    epoch, mode = sys.argv[1:]
    assert epoch.replace('-', '').isalnum() and REPO == Path.cwd()
    out = ROOT / epoch
    out.mkdir(exist_ok=False)
    TEMP.mkdir(exist_ok=True)
    cargo = TEMP / 'cargo'; cargo.mkdir(exist_ok=True)
    (cargo / 'registry').mkdir(exist_ok=True)
    target = TEMP / 'target'; target.mkdir(exist_ok=True)
    paths = {Path(__file__).resolve(), REPO / 'Cargo.toml', REPO / 'Cargo.lock'}
    for crate in ['mpk-cli', 'mpk-api', 'mpk-cert', 'mpk-core', 'mpk-kernel', 'mpk-linux-sandbox', 'mpk-theory', 'mpk-vc']:
        base = REPO / 'crates' / crate
        paths.add(base / 'Cargo.toml')
        paths.update(p for p in (base / 'src').rglob('*') if p.is_file())
        paths.update(p for p in base.glob('build.rs') if p.is_file())
    paths.add(REPO / 'crates/mpk-cli/tests/go_practical_source_roots.rs')
    paths.update(p for p in (REPO / 'go-tools/go2vir').rglob('*') if p.is_file())
    originals = ROOT.parent / 'analysis-namespace/native'
    paths.add(originals / 'native.json')
    paths.update(p for p in (originals / 'original-bytes').rglob('*') if p.is_file())
    paths.add(ROOT.parent / 'analysis-build-list/native/native.json')
    paths.add(ROOT.parent / 'analysis-build-list/native/inputs.json')
    paths.add(ROOT.parent / 'analysis-build-list/native/inputs.tar.gz')
    paths.update(p for p in (REPO / 'release/bundles').rglob('*') if p.is_file())
    # Include the complete private codec and every directly included frozen
    # member as well as the crate sources and registries above.
    probe = REPO / 'develop/probes/go-practical'
    sys.dont_write_bytecode = True
    sys.path.insert(0, str(probe))
    from frontend_native_build import linked_inputs
    paths.update(REPO / item['path'] for item in linked_inputs())
    paths.update(probe / name for name in ['frontend_native_build.py', 'run_proof_pilot.py', 'check_scope_report.py'])
    if mode == 'snapshot-tests':
        paths.add(REPO / 'crates/mpk-vc/tests/go_practical_snapshot_inputs.rs')
        namespace = ROOT.parent / 'snapshot-inputs/native/native-originals/namespace'
        paths.update(p for p in namespace.rglob('*') if p.is_file())
    before = [seal(p) for p in sorted(paths)]
    archive = out / 'inputs.tar.gz'
    with tarfile.open(archive, 'x:gz', dereference=True) as target_archive:
        for path in sorted(paths):
            target_archive.add(path, arcname=str(path.relative_to(REPO)), recursive=False)
    environment = {'CARGO_HOME': '/cargo', 'CARGO_TARGET_DIR': '/target', 'CARGO_NET_OFFLINE': 'true', 'CARGO_INCREMENTAL': '0', 'CARGO_PROFILE_TEST_DEBUG': '0', 'CARGO_PROFILE_TEST_OPT_LEVEL': '1', 'RUST_TEST_THREADS': '1', 'HOME': '/tmp'}
    command = ['/usr/local/bin/docker', 'run', '--pull=never', '--rm', '--platform', 'linux/arm64', '--network', 'none', '--read-only', '--cap-drop', 'ALL', '--security-opt', 'no-new-privileges', '--user', f'{os.getuid()}:{os.getgid()}', '--tmpfs', '/tmp:rw,exec,nosuid,nodev,size=1g,mode=1777', '--mount', f'type=bind,src={REPO},dst=/workspace,readonly', '--mount', f'type=bind,src={cargo},dst=/cargo', '--mount', f'type=bind,src={CARGO_REGISTRY},dst=/cargo/registry,readonly', '--mount', f'type=bind,src={target},dst=/target', '--workdir', '/workspace']
    for key, value in environment.items():
        command.extend(['--env', f'{key}={value}'])
    selected = {
        'root-tests': ['cargo', 'test', '--offline', '--locked', '-p', 'mpk-cli', '--test', 'go_practical_source_roots', '--', '--nocapture'],
        'snapshot-tests': ['cargo', 'test', '--offline', '--locked', '-p', 'mpk-vc', '--test', 'go_practical_snapshot_inputs', 'snapshot_inputs_'],
        'clippy': ['cargo', 'clippy', '--offline', '--locked', '-p', 'mpk-cli', '--lib', '--test', 'go_practical_source_roots', '--', '-D', 'warnings'],
    }
    command.extend([IMAGE, *selected[mode]])
    (out / 'inputs.json').write_text(json.dumps({'selection_reason': 'Restore and rehash all 328 original frontend files from their retained archive, then compare the complete 213-object Go snapshot and its 52 captured directories with held Linux roots. Retain the full original frontend, source buffers, frozen includes, private codec and affected Rust inputs; no whole T gate or native release acceptance.', 'inputs': before, 'archive': seal(archive), 'command': command, 'environment': environment}, indent=2) + '\n')
    report = {'status': 'running', 'native_Linux_release_acceptance': False, 'full_T_gate': 'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    (out / 'result.json').write_text(json.dumps(report, indent=2) + '\n')
    start = time.monotonic()
    with (out / 'stdout.log').open('xb') as stdout, (out / 'stderr.log').open('xb') as stderr:
        code = subprocess.run(command, stdout=stdout, stderr=stderr, check=False).returncode
    unchanged = before == [seal(p) for p in sorted(paths)]
    report.update(status='passed' if code == 0 and unchanged else 'failed', exit_code=code, seconds=time.monotonic()-start, original_inputs_unchanged=unchanged, stdout=seal(out/'stdout.log'), stderr=seal(out/'stderr.log'))
    (out / 'result.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps({k: report[k] for k in ['status', 'exit_code', 'seconds', 'original_inputs_unchanged']}))
    raise SystemExit(report['status'] != 'passed')


if __name__ == '__main__':
    main()
