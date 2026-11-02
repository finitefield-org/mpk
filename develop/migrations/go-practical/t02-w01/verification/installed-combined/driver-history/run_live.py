"""Bind both actual root sets to a test-only original compiler owner."""
from pathlib import Path
import hashlib
import json
import os
import shutil
import subprocess
import sys
import time
import uuid

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[5]
SDK = Path('/private/tmp/mpk-installed-compiler-205-original-go-output/toolchain')
GOCACHE = Path('/private/tmp/mpk-installed-compiler-205-go-cache')
RUST_IMAGE = 'sha256:7cf1e580ef5539f03b58560753e8ab84c8c360960d99dff714004aa98f203977'
GO_IMAGE = 'docker.io/library/golang@sha256:ebd54034f076819b3054f155db53660ded951612bb4dfd277f933e62059e5d5a'
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT.parent / 'installed-compiler'))
from run_scoped import seal, sdk_files, write


def main():
    epoch = sys.argv[1]
    assert epoch.replace('-', '').isalnum() and Path.cwd() == REPO
    scope = json.loads((ROOT / 'scoped-final/result.json').read_bytes())
    inputs = json.loads((ROOT / 'scoped-final/inputs.json').read_bytes())
    assert scope['status'] == 'passed' and scope['original_inputs_unchanged']
    binary_receipt = json.loads((ROOT / 'binary-final/receipt.json').read_bytes())
    binary_restore = json.loads((ROOT / 'binary-final/restore-check.json').read_bytes())
    assert binary_receipt['status'] == binary_restore['status'] == 'passed' and binary_receipt['original_binary_unchanged']
    binary = Path(binary_receipt['runtime_copy'])
    def binary_matches():
        raw = binary.read_bytes()
        return len(raw) == binary_receipt['bytes'] and hashlib.sha256(raw).hexdigest() == binary_receipt['sha256'] and binary.stat().st_mode & 0o777 == binary_receipt['mode']
    assert binary_matches() and binary_restore['sha256'] == binary_receipt['sha256']
    benchmark_path = ROOT.parent / 'combined-root-live/storage-benchmark-original/result.json'
    benchmark = json.loads(benchmark_path.read_bytes())
    assert benchmark['status'] == 'passed' and benchmark['original_inputs_unchanged']
    host, volume_measurement = benchmark['worker']['results']
    assert host['original_files'] == volume_measurement['original_files'] == 5577
    assert host['original_bytes'] == volume_measurement['original_bytes'] == 166354960
    assert host['elapsed_seconds'] > volume_measurement['elapsed_seconds']
    source = inputs['inputs']
    sdk = inputs['registered_sdk']
    assert source == [seal(Path(row['path'])) for row in source]
    assert sdk == sdk_files() and seal(Path(inputs['archive']['path'])) == inputs['archive']
    out = ROOT / epoch
    out.mkdir(exist_ok=False)
    work = Path('/private/tmp') / ('mpk-go-installed-combined-live-' + epoch)
    assert not work.exists()
    work.mkdir(mode=0o700)
    folders = {name: work / name for name in ['native-originals', 'source-coord', 'export-coord', 'combined-coord']}
    for path in folders.values():
        path.mkdir(mode=0o700)
    volume = 'mpk-go-installed-combined-' + uuid.uuid4().hex
    create_volume = ['/usr/local/bin/docker', 'volume', 'create', volume]
    created = subprocess.run(create_volume, cwd=REPO, capture_output=True, text=True, check=False)
    assert created.returncode == 0 and created.stdout.strip() == volume and not created.stderr
    init_volume = ['/usr/local/bin/docker', 'run', '--pull=never', '--rm', '--platform', 'linux/arm64', '--network', 'none', '--read-only', '--user', '0:0', '--cap-drop', 'ALL', '--cap-add', 'CHOWN', '--mount', f'type=volume,src={volume},dst=/volume,volume-nocopy', RUST_IMAGE, '/bin/chown', f'{os.getuid()}:{os.getgid()}', '/volume']
    initialized = subprocess.run(init_volume, cwd=REPO, capture_output=True, text=True, check=False)
    assert initialized.returncode == 0 and not initialized.stdout and not initialized.stderr
    verify_volume = ['/usr/local/bin/docker', 'run', '--pull=never', '--rm', '--platform', 'linux/arm64', '--network', 'none', '--read-only', '--cap-drop', 'ALL', '--security-opt', 'no-new-privileges', '--user', f'{os.getuid()}:{os.getgid()}', '--mount', f'type=volume,src={volume},dst=/volume,volume-nocopy', RUST_IMAGE, 'python3', '-c', 'import os; assert os.listdir("/volume") == []']
    verified = subprocess.run(verify_volume, cwd=REPO, capture_output=True, text=True, check=False)
    assert verified.returncode == 0 and not verified.stdout and not verified.stderr
    common = ['/usr/local/bin/docker', 'run', '--pull=never', '--rm', '--network', 'none', '--read-only', '--cap-drop', 'ALL', '--security-opt', 'no-new-privileges', '--user', f'{os.getuid()}:{os.getgid()}', '--mount', f'type=bind,src={REPO},dst=/workspace,readonly', '--mount', f'type=volume,src={volume},dst=/tmp,volume-nocopy', '--mount', f'type=bind,src={folders["native-originals"]},dst=/evidence']
    for name in ['source-coord', 'export-coord', 'combined-coord']:
        common += ['--mount', f'type=bind,src={folders[name]},dst=/{name}']
    genv = {'GOROOT': '/sdk/go', 'GOTOOLCHAIN': 'local', 'GOENV': 'off', 'GOWORK': 'off', 'GOFLAGS': '-mod=vendor', 'GOPROXY': 'off', 'GOSUMDB': 'off', 'CGO_ENABLED': '0', 'GOOS': 'linux', 'GOARCH': 'amd64', 'GOAMD64': 'v1', 'GOMAXPROCS': '1', 'GOCACHE': '/cache/build', 'GOMODCACHE': '/cache/mod', 'MPK_GO_PRACTICAL_SYNTAX_EVIDENCE': '/evidence', 'MPK_GO_LIVE_ROOT_COORD': '/source-coord', 'MPK_GO_LIVE_EXPORT_ROOT_COORD': '/export-coord', 'MPK_GO_LIVE_COMBINED_COORD': '/combined-coord', 'MPK_GO_LIVE_EXPORT_MUTATE_COPY': '1'}
    go = common + ['--platform', 'linux/amd64', '--mount', f'type=bind,src={SDK},dst=/sdk,readonly', '--mount', f'type=bind,src={GOCACHE},dst=/cache', '--workdir', '/workspace/go-tools/go2vir']
    for key, value in genv.items():
        go += ['--env', f'{key}={value}']
    go += [GO_IMAGE, '/sdk/go/bin/go', 'test', '-json', '-count=1', '-timeout=40m', '-run', '^TestPracticalInstalledCompilerInputsKeepWholeRegisteredSDKAndPackages$']
    renv = {'HOME': '/tmp'}
    rust = common + ['--platform', 'linux/arm64', '--mount', f'type=bind,src={binary},dst=/observer,readonly', '--workdir', '/workspace']
    for key, value in renv.items():
        rust += ['--env', f'{key}={value}']
    rust += [RUST_IMAGE, '/observer', '/source-coord', '/export-coord', '/combined-coord']
    write(out / 'inputs.json', {'selection_reason': 'Open both original root sets before capture and bind them to a test-only compiler owner built from the complete independently verified SDK; no production installed-root authority is claimed.', 'driver': seal(Path(__file__)), 'scope_inputs': seal(ROOT / 'scoped-final/inputs.json'), 'scope_result': seal(ROOT / 'scoped-final/result.json'), 'scope_archive': inputs['archive'], 'binary_receipt': seal(ROOT / 'binary-final/receipt.json'), 'binary_restoration': seal(ROOT / 'binary-final/restore-check.json'), 'storage_benchmark': seal(benchmark_path), 'volume': volume, 'volume_create': create_volume, 'volume_init': init_volume, 'volume_empty_check': verify_volume, 'go_command': go, 'rust_command': rust, 'go_environment': genv, 'rust_environment': renv, 'temporary_root': str(work)})
    result = {'status': 'running', 'native_Linux_release_acceptance': False, 'full_T_gate': 'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    write(out / 'result.json', result)
    started = time.monotonic()
    with (out / 'go.stdout').open('xb') as gout, (out / 'go.stderr').open('xb') as gerr, (out / 'rust.stdout').open('xb') as rout, (out / 'rust.stderr').open('xb') as rerr:
        gp = subprocess.Popen(go, cwd=REPO, stdout=gout, stderr=gerr)
        rp = subprocess.Popen(rust, cwd=REPO, stdout=rout, stderr=rerr)
        print(json.dumps({'phase': 'waiting_for_original_roots', 'go_pid': gp.pid, 'rust_pid': rp.pid}), flush=True)
        last = started
        while gp.poll() is None or rp.poll() is None:
            if rp.poll() not in (None, 0):
                for name, signals in [('source-coord', ['opened.json', 'matched.json']), ('export-coord', ['opened.json', 'matched.json']), ('combined-coord', ['owner-matched.json'])]:
                    for signal in signals:
                        path = folders[name] / signal
                        if not path.exists():
                            write(path, {'status': 'failed', 'diagnostic': 'Rust observer exited before its signal'})
            if gp.poll() not in (None, 0):
                for name, signals in [('source-coord', ['roots-ready.json', 'snapshot-ready.json']), ('export-coord', ['roots-ready.json', 'snapshot-ready.json']), ('combined-coord', ['owner-ready.json'])]:
                    for signal in signals:
                        path = folders[name] / signal
                        if not path.exists():
                            write(path, {'status': 'failed', 'diagnostic': 'Go capture exited before its signal'})
            time.sleep(1)
            if time.monotonic() - last >= 30:
                phase = 'binding_original_owner' if (folders['combined-coord'] / 'owner-ready.json').exists() else 'matching_source_roots' if (folders['source-coord'] / 'snapshot-ready.json').exists() else 'matching_export_roots' if (folders['export-coord'] / 'snapshot-ready.json').exists() else 'capturing_original_application' if (folders['source-coord'] / 'opened.json').exists() else 'opening_source_roots'
                print(json.dumps({'phase': phase, 'seconds': round(time.monotonic() - started)}), flush=True)
                last = time.monotonic()
        go_code, rust_code = gp.wait(), rp.wait()
    for name in ['source-coord', 'export-coord', 'combined-coord']:
        dest = out / name
        dest.mkdir()
        for path in sorted(folders[name].iterdir()):
            if path.is_file():
                shutil.copyfile(path, dest / path.name)
    evidence = folders['native-originals']
    files = [seal(p) for p in sorted(evidence.rglob('*')) if p.is_file()]
    write(out / 'original-evidence-manifest.json', {'root': str(evidence), 'files': files, 'file_count': len(files), 'total_bytes': sum(row['bytes'] for row in files)})
    owner_path = out / 'combined-coord/owner-matched.json'
    owner = json.loads(owner_path.read_bytes()) if owner_path.exists() else {}
    source_unchanged = source == [seal(Path(row['path'])) for row in source]
    binary_unchanged = binary_matches()
    sdk_unchanged = sdk == sdk_files()
    okay = go_code == 0 and rust_code == 0 and owner.get('status') == 'passed' and owner.get('both_root_sets_retained') is True and owner.get('export_cache_retirement_checked_before_binding') is True and owner.get('source_objects', 0) >= 5480 and owner.get('export_objects', 0) >= 181 and owner.get('total_objects') == owner.get('source_objects', 0) + owner.get('export_objects', 0) and owner.get('compiler_inputs_imported') is True and owner.get('registered_sdk_files') == 5577 and owner.get('registered_sdk_bytes') == 166354960 and owner.get('test_only_installed_owner_bound') is True and not owner.get('installed_authority_claimed') and source_unchanged and binary_unchanged and sdk_unchanged
    result.update(status='passed' if okay else 'failed', go_exit_code=go_code, rust_exit_code=rust_code, seconds=time.monotonic() - started, original_inputs_unchanged=source_unchanged, test_binary_unchanged=binary_unchanged, registered_sdk_unchanged=sdk_unchanged, matched=owner, original_evidence_manifest=seal(out / 'original-evidence-manifest.json'), go_stdout=seal(out / 'go.stdout'), go_stderr=seal(out / 'go.stderr'), rust_stdout=seal(out / 'rust.stdout'), rust_stderr=seal(out / 'rust.stderr'))
    write(out / 'result.json', result)
    print(json.dumps({key: result[key] for key in ['status', 'go_exit_code', 'rust_exit_code', 'seconds', 'original_inputs_unchanged', 'test_binary_unchanged', 'registered_sdk_unchanged']}), flush=True)
    raise SystemExit(not okay)


if __name__ == '__main__':
    main()
