"""Bind original capture-time roots to the real staged installed-root reader."""
from pathlib import Path
import json
import os
import shutil
import subprocess
import sys
import time
import uuid

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[5]
VOLUME = Path('/private/tmp/mpk-go-combined-recovery-20261029-fs')
WORK = VOLUME / 'live-observation'
SDK = Path('/private/tmp/mpk-installed-compiler-205-original-go-output/toolchain')
ORIGINAL_CACHE = Path('/private/tmp/mpk-installed-compiler-205-go-cache')
RUST_IMAGE = 'sha256:7cf1e580ef5539f03b58560753e8ab84c8c360960d99dff714004aa98f203977'
GO_IMAGE = 'docker.io/library/golang@sha256:ebd54034f076819b3054f155db53660ded951612bb4dfd277f933e62059e5d5a'
OUT = ROOT / 'live-original'
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT.parent / 'installed-compiler'))
from run_scoped import seal, sdk_files, write
sys.path.insert(0, str(REPO / 'scripts'))
import successor_release_bundles as release


def main():
    assert Path.cwd() == REPO and VOLUME.is_mount() and not WORK.exists() and not OUT.exists()
    build = json.loads((ROOT / 'observer-final/result.json').read_bytes())
    stage = json.loads((ROOT / 'installed-stage-retry/result.json').read_bytes())
    source = json.loads((ROOT / 'observer-build/inputs.json').read_bytes())['sources']
    assert build['status'] == stage['status'] == 'passed'
    assert build['source_inputs_unchanged'] and stage['original_inputs_unchanged']
    assert source == [seal(Path(row['path'])) for row in source]
    binary = Path(build['executable']['path'])
    assert seal(binary) == build['executable']
    installed = Path(stage['installed_root'])
    qemu = Path(stage['qemu_root'])
    assert seal(installed / 'bin/mpk') == stage['installed_executable']
    sdk = sdk_files()
    registry, _ = release.validate_release_models()
    bundles = {item['bundle_id']: item for kind in ['frontend_bundles', 'toolchain_bundles'] for item in registry[kind]}
    assert set(bundles) == set(stage['complete_bundle_ids'])
    for identifier, bundle in bundles.items():
        release.validate_root(installed / 'libexec/mpk/bundles' / identifier, bundle)
    WORK.mkdir()
    OUT.mkdir()
    cache = WORK / 'go-cache'
    cache_files = [seal(path) for path in sorted(ORIGINAL_CACHE.rglob('*')) if path.is_file()]
    assert cache_files and not any(path.is_symlink() for path in ORIGINAL_CACHE.rglob('*'))
    shutil.copytree(ORIGINAL_CACHE, cache, copy_function=shutil.copy2)
    for row in cache_files:
        copied = seal(cache / Path(row['path']).relative_to(ORIGINAL_CACHE))
        assert all(copied[key] == row[key] for key in ['bytes', 'sha256', 'mode'])
    folders = {name: WORK / name for name in ['native-originals', 'source-coord', 'export-coord', 'combined-coord']}
    for path in folders.values():
        path.mkdir(mode=0o700)
    volume = 'mpk-go-installed-release-live-' + uuid.uuid4().hex
    create = ['/usr/local/bin/docker', 'volume', 'create', volume]
    created = subprocess.run(create, cwd=REPO, capture_output=True, text=True, check=False)
    assert created.returncode == 0 and created.stdout.strip() == volume and not created.stderr
    initialize = ['/usr/local/bin/docker', 'run', '--pull=never', '--rm', '--platform', 'linux/arm64', '--network', 'none', '--read-only', '--user', '0:0', '--cap-drop', 'ALL', '--cap-add', 'CHOWN', '--mount', f'type=volume,src={volume},dst=/volume,volume-nocopy', RUST_IMAGE, '/bin/chown', f'{os.getuid()}:{os.getgid()}', '/volume']
    initialized = subprocess.run(initialize, cwd=REPO, capture_output=True, text=True, check=False)
    assert initialized.returncode == 0 and not initialized.stdout and not initialized.stderr
    verify = ['/usr/local/bin/docker', 'run', '--pull=never', '--rm', '--platform', 'linux/arm64', '--network', 'none', '--read-only', '--cap-drop', 'ALL', '--security-opt', 'no-new-privileges', '--user', f'{os.getuid()}:{os.getgid()}', '--mount', f'type=volume,src={volume},dst=/volume,volume-nocopy', RUST_IMAGE, 'python3', '-c', 'import os; assert os.listdir("/volume") == []']
    verified = subprocess.run(verify, cwd=REPO, capture_output=True, text=True, check=False)
    assert verified.returncode == 0 and not verified.stdout and not verified.stderr
    common = ['/usr/local/bin/docker', 'run', '--pull=never', '--rm', '--network', 'none', '--read-only', '--cap-drop', 'ALL', '--security-opt', 'no-new-privileges', '--user', f'{os.getuid()}:{os.getgid()}', '--mount', f'type=bind,src={REPO},dst=/workspace,readonly', '--mount', f'type=volume,src={volume},dst=/tmp,volume-nocopy', '--mount', f'type=bind,src={folders["native-originals"]},dst=/evidence']
    for name in ['source-coord', 'export-coord', 'combined-coord']:
        common += ['--mount', f'type=bind,src={folders[name]},dst=/{name}']
    genv = {'GOROOT': '/sdk/go', 'GOTOOLCHAIN': 'local', 'GOENV': 'off', 'GOWORK': 'off', 'GOFLAGS': '-mod=vendor', 'GOPROXY': 'off', 'GOSUMDB': 'off', 'CGO_ENABLED': '0', 'GOOS': 'linux', 'GOARCH': 'amd64', 'GOAMD64': 'v1', 'GOMAXPROCS': '1', 'GOCACHE': '/cache/build', 'GOMODCACHE': '/cache/mod', 'MPK_GO_PRACTICAL_SYNTAX_EVIDENCE': '/evidence', 'MPK_GO_LIVE_ROOT_COORD': '/source-coord', 'MPK_GO_LIVE_EXPORT_ROOT_COORD': '/export-coord', 'MPK_GO_LIVE_COMBINED_COORD': '/combined-coord', 'MPK_GO_LIVE_EXPORT_MUTATE_COPY': '1'}
    go = common + ['--platform', 'linux/amd64', '--mount', f'type=bind,src={SDK},dst=/sdk,readonly', '--mount', f'type=bind,src={cache},dst=/cache', '--workdir', '/workspace/go-tools/go2vir']
    for key, value in genv.items():
        go += ['--env', f'{key}={value}']
    go += [GO_IMAGE, '/sdk/go/bin/go', 'test', '-json', '-count=1', '-timeout=40m', '-run', '^TestPracticalInstalledCompilerInputsKeepWholeRegisteredSDKAndPackages$']
    observer = common + ['--platform', 'linux/arm64', '--mount', f'type=bind,src={installed},dst=/installed,readonly', '--mount', f'type=bind,src={qemu / "qemu-backports/qemu-x86_64"},dst=/qemu,readonly', '--mount', f'type=bind,src={qemu / "qemu-amd64-runtime"},dst=/x86root,readonly', '--workdir', '/workspace', '--env', 'HOME=/tmp', RUST_IMAGE, '/qemu', '-L', '/x86root', '/installed/bin/mpk', '/source-coord', '/export-coord', '/combined-coord']
    write(OUT / 'inputs.json', {'selection_reason': 'The complete original Go capture and both independently held capture-time root sets are matched while the real installed-release loader opens the complete staged Go/Rust namespace. The executable is a test observer; QEMU is local emulation, not native Linux release or launcher authority.', 'driver': seal(Path(__file__)), 'observer_build': seal(ROOT / 'observer-final/result.json'), 'installed_stage': seal(ROOT / 'installed-stage-retry/result.json'), 'original_cache': cache_files, 'source_inputs': source, 'registered_sdk': sdk, 'volume': volume, 'volume_create': create, 'volume_initialize': initialize, 'volume_empty_check': verify, 'go_command': go, 'observer_command': observer, 'go_environment': genv, 'temporary_root': str(WORK)})
    result = {'status': 'running', 'native_Linux_release_acceptance': False,
              'full_T_gate': 'Deferred to GO-PRAC-T02-W04: ./scripts/check-fast.sh'}
    write(OUT / 'result.json', result)
    started = time.monotonic()
    with (OUT / 'go.stdout').open('xb') as gout, (OUT / 'go.stderr').open('xb') as gerr, (OUT / 'observer.stdout').open('xb') as rout, (OUT / 'observer.stderr').open('xb') as rerr:
        gp = subprocess.Popen(go, cwd=REPO, stdout=gout, stderr=gerr)
        rp = subprocess.Popen(observer, cwd=REPO, stdout=rout, stderr=rerr)
        print(json.dumps({'phase': 'waiting_for_original_roots', 'go_pid': gp.pid, 'observer_pid': rp.pid}), flush=True)
        last = started
        while gp.poll() is None or rp.poll() is None:
            if rp.poll() not in (None, 0):
                for name, signals in [('source-coord', ['opened.json', 'matched.json']), ('export-coord', ['opened.json', 'matched.json']), ('combined-coord', ['owner-matched.json'])]:
                    for signal in signals:
                        path = folders[name] / signal
                        if not path.exists():
                            write(path, {'status': 'failed', 'diagnostic': 'installed observer exited before its signal'})
            if gp.poll() not in (None, 0):
                for name, signals in [('source-coord', ['roots-ready.json', 'snapshot-ready.json']), ('export-coord', ['roots-ready.json', 'snapshot-ready.json']), ('combined-coord', ['owner-ready.json'])]:
                    for signal in signals:
                        path = folders[name] / signal
                        if not path.exists():
                            write(path, {'status': 'failed', 'diagnostic': 'Go capture exited before its signal'})
            time.sleep(1)
            if time.monotonic() - last >= 30:
                phase = 'binding_installed_owner' if (folders['combined-coord'] / 'owner-ready.json').exists() else 'matching_source_roots' if (folders['source-coord'] / 'snapshot-ready.json').exists() else 'matching_export_roots' if (folders['export-coord'] / 'snapshot-ready.json').exists() else 'capturing_original_application' if (folders['source-coord'] / 'opened.json').exists() else 'opening_source_roots'
                print(json.dumps({'phase': phase, 'seconds': round(time.monotonic()-started)}), flush=True)
                last = time.monotonic()
        go_code, observer_code = gp.wait(), rp.wait()
    for name in ['source-coord', 'export-coord', 'combined-coord']:
        destination = OUT / name
        destination.mkdir()
        for path in sorted(folders[name].iterdir()):
            if path.is_file():
                shutil.copy2(path, destination / path.name)
    evidence = folders['native-originals']
    files = [seal(path) for path in sorted(evidence.rglob('*')) if path.is_file()]
    write(OUT / 'original-evidence-manifest.json', {'root': str(evidence), 'files': files,
          'file_count': len(files), 'total_bytes': sum(row['bytes'] for row in files)})
    owner_path = OUT / 'combined-coord/owner-matched.json'
    owner = json.loads(owner_path.read_bytes()) if owner_path.exists() else {}
    source_unchanged = source == [seal(Path(row['path'])) for row in source]
    sdk_unchanged = sdk == sdk_files()
    cache_unchanged = cache_files == [seal(Path(row['path'])) for row in cache_files]
    binary_unchanged = seal(binary) == build['executable'] and seal(installed / 'bin/mpk') == stage['installed_executable']
    for identifier, bundle in bundles.items():
        release.validate_root(installed / 'libexec/mpk/bundles' / identifier, bundle)
    passed = (go_code == observer_code == 0 and owner.get('status') == 'passed'
              and owner.get('both_root_sets_retained') is True
              and owner.get('export_cache_retirement_checked_before_binding') is True
              and (owner.get('source_objects'), owner.get('export_objects'), owner.get('total_objects')) == (5516, 181, 5697)
              and (owner.get('registered_sdk_files'), owner.get('registered_sdk_bytes')) == (5577, 166354960)
              and owner.get('compiler_inputs_imported') is True and owner.get('staged_installed_root_opened') is True
              and owner.get('staged_test_executable') is True and not owner.get('launcher_authority_claimed')
              and not owner.get('compiler_to_source_proved') and not owner.get('application_proof_claimed')
              and source_unchanged and sdk_unchanged and cache_unchanged and binary_unchanged)
    result.update(status='passed' if passed else 'failed', go_exit_code=go_code, observer_exit_code=observer_code,
                  seconds=time.monotonic()-started, original_inputs_unchanged=source_unchanged,
                  registered_sdk_unchanged=sdk_unchanged, original_go_cache_unchanged=cache_unchanged,
                  staged_binary_unchanged=binary_unchanged, complete_staged_bundles_unchanged=True,
                  matched=owner, original_evidence_manifest=seal(OUT / 'original-evidence-manifest.json'),
                  go_stdout=seal(OUT / 'go.stdout'), go_stderr=seal(OUT / 'go.stderr'),
                  observer_stdout=seal(OUT / 'observer.stdout'), observer_stderr=seal(OUT / 'observer.stderr'))
    write(OUT / 'result.json', result)
    print(json.dumps({key: result[key] for key in ['status', 'go_exit_code', 'observer_exit_code', 'seconds', 'original_inputs_unchanged', 'registered_sdk_unchanged', 'original_go_cache_unchanged', 'staged_binary_unchanged']}), flush=True)
    raise SystemExit(not passed)


if __name__ == '__main__':
    main()
