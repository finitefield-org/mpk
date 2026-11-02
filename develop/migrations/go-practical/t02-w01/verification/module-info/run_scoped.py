"""Retain exact inputs and outputs for the installed Go compiler boundary."""
from pathlib import Path
import datetime
import hashlib
import json
import os
import shutil
import subprocess
import sys
import tarfile
import time

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[5]
SDK = Path('/private/tmp/mpk-installed-compiler-205-original-go-output/toolchain')
HOST_GO = Path('/private/tmp/mpk-go125-20261025-download/go')
RUST = ['crates/mpk-vc/src/go_practical_type_keys.rs', 'crates/mpk-vc/src/go_practical_type_keys/module_observations.rs', 'crates/mpk-vc/src/go_practical_type_keys/module_observations/data.rs', 'crates/mpk-vc/src/go_practical_type_keys/module_observations/data/archive.rs', 'crates/mpk-vc/src/go_practical_type_keys/module_observations/data/info.rs', 'crates/mpk-vc/src/go_practical_type_keys/analysis_vendor.rs', 'crates/mpk-vc/tests/go_practical_module_data.rs']
GO = ['go-tools/go2vir/practical_module_info_differential_test.go']


def seal(path):
    path = Path(path)
    digest = hashlib.sha256()
    size = 0
    with path.open('rb') as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b''):
            digest.update(block)
            size += len(block)
    return {'path': str(path.resolve()), 'bytes': size, 'sha256': digest.hexdigest(), 'mode': path.stat().st_mode & 0o777}


def write(path, value):
    path.write_text(json.dumps(value, indent=2) + '\n')


def sdk_files():
    candidate = json.loads((REPO / 'release/bundles/candidates/go.json').read_bytes())['toolchain_bundles'][0]
    rows = []
    for entry in candidate['inventory']['files']:
        actual = seal(SDK / entry['path'])
        assert (actual['bytes'], actual['sha256'], bool(actual['mode'] & 0o111)) == (entry['size_bytes'], entry['sha256'], entry['executable'])
        rows.append(actual)
    assert {str(p.relative_to(SDK)) for p in SDK.rglob('*') if p.is_file()} == {e['path'] for e in candidate['inventory']['files']}
    return rows


def main():
    epoch, mode = sys.argv[1:]
    assert epoch.replace('-', '').isalnum()
    assert REPO == Path.cwd()
    out = ROOT / epoch
    out.mkdir(exist_ok=False)
    files = {Path(__file__).resolve()}
    files.update(REPO / p for p in RUST + GO)
    files.add(REPO / 'crates/mpk-cli/Cargo.toml')
    if mode.startswith('vc-'):
        files.update(REPO / 'crates/mpk-vc/tests' / name for name in ['go_practical_compiler_build_info.rs', 'go_practical_compiler_elf.rs', 'go_practical_compiler_inputs.rs'])
        files.add(ROOT / 'registered-go2vir.elf')
        if (ROOT / 'native/native.json').exists():
            files.add(ROOT / 'native/native.json')
    if mode.startswith('vendor-'):
        files.add(REPO / 'crates/mpk-vc/tests/go_practical_type_keys.rs')
        if mode == 'vendor-tests':
            files.add(ROOT / 'native/native.json')
            files.update(p for p in (ROOT / 'native/original-bytes').rglob('*') if p.is_file())
    if mode.startswith('archive-'):
        files.add(REPO / 'crates/mpk-vc/tests/go_practical_module_data.rs')
        if mode in ['archive-tests', 'archive-final-tests']:
            files.add(ROOT / 'native/native.json')
            files.update(p for p in (ROOT / 'native/original-bytes').rglob('*') if p.is_file())
        if mode == 'archive-tests':
            files.add(ROOT.parent / 'analysis-vendor/native/native.json')
            files.update(p for p in (ROOT.parent / 'analysis-vendor/native/original-bytes').rglob('*') if p.is_file())
    if mode in ['consumer-tests', 'generic-tests']:
        directories = ['analysis-info/native', 'analysis-module/native', 'analysis-namespace/native', 'analysis-archive/native'] if mode == 'consumer-tests' else ['module-info/native']
        for directory in directories:
            files.update(p for p in (ROOT.parent / directory).rglob('*') if p.is_file() and p.name not in ['inputs.tar.gz', 'inputs.json', 'result.json'])
        files.update(REPO / 'crates/mpk-vc/tests' / name for name in ['go_practical_analysis_info.rs', 'go_practical_analysis_module.rs', 'go_practical_analysis_namespace.rs', 'go_practical_analysis_archives.rs'])
    if mode in ['generic-tests', 'owner-tests', 'rebind-tests']:
        files.add(ROOT / 'original-inputs.json')
        files.add(ROOT.parent / 'snapshot-inputs/preserve-originals.py')
    if mode == 'info-tests':
        for directory in ['analysis-info/native', 'analysis-module/native', 'analysis-namespace/native', 'analysis-archive/native']:
            files.update(p for p in (ROOT.parent / directory).rglob('*') if p.is_file() and p.name not in ['inputs.tar.gz', 'inputs.json', 'result.json'])
        files.add(REPO / 'crates/mpk-vc/tests/go_practical_analysis_namespace.rs')
        files.add(REPO / 'crates/mpk-vc/tests/go_practical_analysis_archives.rs')
    if mode == 'fold-tests':
        files.add(ROOT.parent / 'module-data/native/native-originals/module-data-oracle/original-simple-fold.json')
    if mode == 'module-tests':
        for directory in ['analysis-module/native', 'analysis-namespace/native', 'analysis-archive/native']:
            files.update(p for p in (ROOT.parent / directory).rglob('*') if p.is_file() and p.name not in ['inputs.tar.gz', 'inputs.json', 'result.json'])
        files.add(REPO / 'crates/mpk-vc/tests/go_practical_analysis_namespace.rs')
        files.add(REPO / 'crates/mpk-vc/tests/go_practical_analysis_archives.rs')
    if mode == 'manifest-tests':
        files.add(REPO / 'crates/mpk-vc/tests/go_practical_module_manifests.rs')
        files.add(ROOT.parent / 'module-manifests/native-toolchain/native-originals/module-manifests/complete-original-module-manifests.json')
    if mode == 'sums-tests':
        files.add(REPO / 'crates/mpk-vc/tests/go_practical_module_data.rs')
        files.add(ROOT.parent / 'module-data/native/native-originals/module-data-oracle/original-module-sums.json')
    if mode == 'budget-tests':
        for directory in ['analysis-budget/native', 'analysis-namespace/native', 'analysis-archive/native']:
            files.update(p for p in (ROOT.parent / directory).rglob('*') if p.is_file() and p.name not in ['inputs.tar.gz', 'inputs.json', 'result.json'])
        files.add(REPO / 'crates/mpk-vc/tests/go_practical_analysis_namespace.rs')
        files.add(REPO / 'crates/mpk-vc/tests/go_practical_analysis_archives.rs')
    if mode in ['namespace-tests', 'snapshot-tests', 'budget-tests', 'module-tests', 'manifest-tests', 'sums-tests', 'info-tests', 'fold-tests', 'generic-tests', 'owner-tests', 'rebind-tests', 'consumer-tests']:
        for directory in ['analysis-namespace/native', 'analysis-archive/native', 'snapshot-inputs/native/native-originals/namespace']:
            originals_root = ROOT.parent / directory
            files.update(p for p in originals_root.rglob('*') if p.is_file() and p.name not in ['inputs.tar.gz', 'inputs.json', 'result.json'])
        files.add(REPO / 'crates/mpk-vc/tests/go_practical_analysis_archives.rs')
        files.add(REPO / 'crates/mpk-vc/tests/go_practical_snapshot_inputs.rs')
    if mode not in ['rust-format-write', 'rust-format', 'go-format-write', 'go-format']:
        sys.dont_write_bytecode = True
        sys.path.insert(0, str(REPO / 'develop/probes/go-practical'))
        from frontend_native_build import linked_inputs
        files.update(REPO / r['path'] for r in linked_inputs())
        for folder in ['go-tools/go2vir', 'go-tools/mpk-checker-ref']:
            files.update(p for p in (REPO / folder).rglob('*') if p.is_file())
        files.update(REPO / p for p in ['develop/probes/go-practical/frontend_native_build.py', 'release/bundles/candidates/go.json'])
    originals = set()
    if mode == 'vc-tests':
        for directory in ['compiler-build-info/fixtures', 'compiler-elf/fixtures', 'compiler-inputs/native/native-originals/versioned-owned']:
            originals.update(p for p in (ROOT.parent / directory).rglob('*') if p.is_file())
    if mode == 'vendor-tests':
        originals.update(p for p in (ROOT.parent / 'type-keys/native-components').rglob('*') if p.is_file())
    if mode == 'archive-tests':
        for directory in ['module-data/native/native-originals', 'module-data/native-compat/native-originals', 'module-observations/native/native-originals']:
            originals.update(p for p in (ROOT.parent / directory).rglob('*') if p.is_file())
        originals.add(ROOT.parent / 'snapshot-inputs/preserve-originals.py')
    if mode in ['generic-tests', 'owner-tests', 'rebind-tests']:
        owners = ['versioned-owned', 'modules-owned'] if mode == 'owner-tests' else ['versioned-owned']
        for owner in owners:
            originals.update(p for p in (ROOT.parent / 'module-observations/native/native-originals' / owner).rglob('*') if p.is_file())
    before_originals = [seal(p) for p in sorted(originals)]
    sources = [seal(p) for p in sorted(files)]
    with tarfile.open(out / 'inputs.tar.gz', 'x:gz', dereference=True) as archive:
        for p in sorted(files):
            archive.add(p, arcname=str(p.relative_to(REPO)), recursive=False)
    installed_sdk = sdk_files() if mode == 'native-owner' else []
    write(out / 'inputs.json', {'sources': sources, 'source_archive': seal(out / 'inputs.tar.gz'), 'whole_registered_SDK': installed_sdk, 'complete_original_test_inputs': before_originals})
    command = {
        'generic-check': ['cargo', 'check', '--offline', '--locked', '-p', 'mpk-vc', '--test', 'go_practical_module_data', '--test', 'go_practical_analysis_info'],
        'generic-tests': ['cargo', 'test', '--offline', '--locked', '-p', 'mpk-vc', '--test', 'go_practical_module_data', 'original_application_module_info_matches_native_go_with_complete_capture'],
        'owner-tests': ['cargo', 'test', '--offline', '--locked', '-p', 'mpk-vc', '--test', 'go_practical_module_data', 'actual_original_versioned_and_local_module_universes_verify_without_host_inputs'],
        'rebind-tests': ['cargo', 'test', '--offline', '--locked', '-p', 'mpk-vc', '--test', 'go_practical_module_data', 'coherently_resealed_auxiliary_data_cannot_replace_original_module_contents'],
        'consumer-tests': ['cargo', 'test', '--offline', '--locked', '-p', 'mpk-vc', '--test', 'go_practical_analysis_info', '--test', 'go_practical_analysis_module', '--test', 'go_practical_analysis_namespace', '--test', 'go_practical_analysis_archives'],
        'generic-clippy': ['cargo', 'clippy', '--offline', '--locked', '-p', 'mpk-vc', '--lib', '--test', 'go_practical_module_data', '--test', 'go_practical_analysis_info', '--test', 'go_practical_analysis_module', '--test', 'go_practical_analysis_namespace', '--test', 'go_practical_analysis_archives', '--', '-D', 'warnings'],
        'info-check': ['cargo', 'check', '--offline', '--locked', '-p', 'mpk-vc', '--test', 'go_practical_analysis_info'],
        'info-tests': ['cargo', 'test', '--offline', '--locked', '-p', 'mpk-vc', '--test', 'go_practical_analysis_info', '--test', 'go_practical_analysis_module', '--test', 'go_practical_analysis_namespace', '--test', 'go_practical_analysis_archives'],
        'fold-tests': ['cargo', 'test', '--offline', '--locked', '-p', 'mpk-vc', '--lib', 'whole_pinned_go_unicode_fold_agrees_for_every_scalar'],
        'info-clippy': ['cargo', 'clippy', '--offline', '--locked', '-p', 'mpk-vc', '--lib', '--test', 'go_practical_analysis_info', '--test', 'go_practical_analysis_module', '--test', 'go_practical_analysis_namespace', '--test', 'go_practical_analysis_archives', '--', '-D', 'warnings'],
        'module-check': ['cargo', 'check', '--offline', '--locked', '-p', 'mpk-vc', '--test', 'go_practical_analysis_module'],
        'module-tests': ['cargo', 'test', '--offline', '--locked', '-p', 'mpk-vc', '--test', 'go_practical_analysis_module', '--test', 'go_practical_analysis_namespace', '--test', 'go_practical_analysis_archives'],
        'manifest-tests': ['cargo', 'test', '--offline', '--locked', '-p', 'mpk-vc', '--test', 'go_practical_module_manifests', 'every_complete_original_manifest'],
        'sums-tests': ['cargo', 'test', '--offline', '--locked', '-p', 'mpk-vc', '--test', 'go_practical_module_data', 'every_original_go_sum_line'],
        'module-clippy': ['cargo', 'clippy', '--offline', '--locked', '-p', 'mpk-vc', '--lib', '--test', 'go_practical_analysis_module', '--test', 'go_practical_analysis_namespace', '--test', 'go_practical_analysis_archives', '--test', 'go_practical_module_manifests', '--test', 'go_practical_module_data', '--', '-D', 'warnings'],
        'budget-check': ['cargo', 'check', '--offline', '--locked', '-p', 'mpk-vc', '--test', 'go_practical_analysis_budget'],
        'budget-tests': ['cargo', 'test', '--offline', '--locked', '-p', 'mpk-vc', '--test', 'go_practical_analysis_budget', '--test', 'go_practical_analysis_namespace', '--test', 'go_practical_analysis_archives'],
        'budget-clippy': ['cargo', 'clippy', '--offline', '--locked', '-p', 'mpk-vc', '--lib', '--test', 'go_practical_analysis_budget', '--test', 'go_practical_analysis_namespace', '--test', 'go_practical_analysis_archives', '--', '-D', 'warnings'],
        'namespace-check': ['cargo', 'check', '--offline', '--locked', '-p', 'mpk-vc', '--test', 'go_practical_analysis_namespace'],
        'namespace-tests': ['cargo', 'test', '--offline', '--locked', '-p', 'mpk-vc', '--test', 'go_practical_analysis_namespace', '--test', 'go_practical_analysis_archives'],
        'snapshot-tests': ['cargo', 'test', '--offline', '--locked', '-p', 'mpk-vc', '--test', 'go_practical_snapshot_inputs', 'snapshot_inputs_'],
        'namespace-clippy': ['cargo', 'clippy', '--offline', '--locked', '-p', 'mpk-vc', '--lib', '--test', 'go_practical_analysis_namespace', '--test', 'go_practical_analysis_archives', '--test', 'go_practical_snapshot_inputs', '--', '-D', 'warnings'],
        'rust-format-write': ['rustfmt', '--edition=2021', '--config', 'skip_children=true', *RUST],
        'rust-format': ['rustfmt', '--edition=2021', '--check', '--config', 'skip_children=true', *RUST],
        'go-format-write': [str(HOST_GO / 'bin/gofmt'), '-w', *GO],
        'go-format': [str(HOST_GO / 'bin/gofmt'), '-l', *GO],
        'host-check': ['cargo', 'check', '--offline', '--locked', '-p', 'mpk-cli', '--lib'],
        'host-test': ['cargo', 'test', '--offline', '--locked', '-p', 'mpk-cli', '--test', 'go_practical_installed_compiler'],
        'host-clippy': ['cargo', 'clippy', '--offline', '--locked', '-p', 'mpk-cli', '--lib', '--test', 'go_practical_installed_compiler', '--', '-D', 'warnings'],
        'vc-check': ['cargo', 'check', '--offline', '--locked', '-p', 'mpk-vc', '--test', 'go_practical_analysis_build_info'],
        'vc-tests': ['cargo', 'test', '--offline', '--locked', '-p', 'mpk-vc', '--test', 'go_practical_analysis_build_info', '--test', 'go_practical_compiler_build_info', '--test', 'go_practical_compiler_elf', '--test', 'go_practical_compiler_inputs'],
        'vc-clippy': ['cargo', 'clippy', '--offline', '--locked', '-p', 'mpk-vc', '--lib', '--test', 'go_practical_analysis_build_info', '--test', 'go_practical_compiler_build_info', '--test', 'go_practical_compiler_elf', '--test', 'go_practical_compiler_inputs', '--', '-D', 'warnings'],
        'vendor-check': ['cargo', 'check', '--offline', '--locked', '-p', 'mpk-vc', '--test', 'go_practical_analysis_vendor'],
        'vendor-tests': ['cargo', 'test', '--offline', '--locked', '-p', 'mpk-vc', '--test', 'go_practical_analysis_vendor', '--test', 'go_practical_type_keys'],
        'vendor-clippy': ['cargo', 'clippy', '--offline', '--locked', '-p', 'mpk-vc', '--lib', '--test', 'go_practical_analysis_vendor', '--test', 'go_practical_type_keys', '--', '-D', 'warnings'],
        'archive-check': ['cargo', 'check', '--offline', '--locked', '-p', 'mpk-vc', '--test', 'go_practical_analysis_archives'],
        'archive-tests': ['cargo', 'test', '--offline', '--locked', '-p', 'mpk-vc', '--test', 'go_practical_analysis_archives', '--test', 'go_practical_analysis_vendor', '--test', 'go_practical_module_data'],
        'archive-clippy': ['cargo', 'clippy', '--offline', '--locked', '-p', 'mpk-vc', '--lib', '--test', 'go_practical_analysis_archives', '--test', 'go_practical_analysis_vendor', '--test', 'go_practical_module_data', '--', '-D', 'warnings'],
        'archive-final-tests': ['cargo', 'test', '--offline', '--locked', '-p', 'mpk-vc', '--test', 'go_practical_analysis_archives'],
        'archive-final-clippy': ['cargo', 'clippy', '--offline', '--locked', '-p', 'mpk-vc', '--lib', '--test', 'go_practical_analysis_archives', '--', '-D', 'warnings'],
    }.get(mode)
    changes = {'CARGO_NET_OFFLINE': 'true', 'CARGO_INCREMENTAL': '0', 'CARGO_PROFILE_DEV_DEBUG': '0'}
    if mode in ['vc-tests', 'vendor-tests', 'archive-tests', 'archive-final-tests', 'namespace-tests', 'snapshot-tests', 'budget-tests', 'module-tests', 'manifest-tests', 'sums-tests', 'info-tests', 'fold-tests', 'generic-tests', 'owner-tests', 'rebind-tests', 'consumer-tests']:
        changes.update({'CARGO_PROFILE_TEST_OPT_LEVEL': '1', 'CARGO_PROFILE_TEST_DEBUG': '0', 'CARGO_PROFILE_TEST_DEBUG_ASSERTIONS': 'true', 'CARGO_PROFILE_TEST_OVERFLOW_CHECKS': 'true', 'RUST_TEST_THREADS': '1'})
    if mode == 'native-owner':
        evidence = out / 'native-originals'
        evidence.mkdir()
        cache = Path('/private/tmp/mpk-installed-compiler-205-go-cache')
        cache.mkdir(exist_ok=True)
        image = 'docker.io/library/golang@sha256:ebd54034f076819b3054f155db53660ded951612bb4dfd277f933e62059e5d5a'
        command = ['/usr/local/bin/docker', 'run', '--pull=never', '--rm', '--platform', 'linux/amd64', '--network', 'none', '--read-only', '--cap-drop', 'ALL', '--security-opt', 'no-new-privileges', '--user', f'{os.getuid()}:{os.getgid()}', '--tmpfs', '/tmp:rw,exec,nosuid,nodev,size=2g,mode=1777', '--mount', f'type=bind,src={REPO},dst=/workspace,readonly', '--mount', f'type=bind,src={SDK},dst=/registered-sdk,readonly', '--mount', f'type=bind,src={evidence},dst=/evidence', '--mount', f'type=bind,src={cache},dst=/cache', '--workdir', '/workspace/go-tools/go2vir']
        go_env = {'GOROOT': '/registered-sdk/go', 'GOTOOLCHAIN': 'local', 'GOENV': 'off', 'GOWORK': 'off', 'GOFLAGS': '-mod=vendor', 'GOPROXY': 'off', 'GOSUMDB': 'off', 'CGO_ENABLED': '0', 'GOOS': 'linux', 'GOARCH': 'amd64', 'GOAMD64': 'v1', 'GOMAXPROCS': '1', 'GOCACHE': '/cache/build', 'GOMODCACHE': '/cache/mod', 'MPK_GO_PRACTICAL_SYNTAX_EVIDENCE': '/evidence'}
        for k, v in go_env.items():
            command.extend(['--env', f'{k}={v}'])
        command.extend([image, '/registered-sdk/go/bin/go', 'test', '-json', '-count=1', '-timeout=40m', '-run', '^TestPracticalInstalledCompilerInputsKeepWholeRegisteredSDKAndPackages$'])
    assert command is not None
    report = {'status': 'running', 'started_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(), 'command': command, 'environment': changes, 'inputs': seal(out / 'inputs.json'), 'selection_reason': 'Check generic original application module-info validation and its directly affected analysis/module consumers, retaining the complete original application, SDK and archive inputs. A caller-supplied snapshot is not authenticated as an original filesystem; source syntax and binary/context authority remain separate. Preserve original frontend/source and compiler inputs; do not infer a practical profile, source correspondence or native Linux release acceptance. Formats cover changed files only.', 'full_T_gate': 'Deferred to final T02-W04: ./scripts/check-fast.sh', 'native_Linux_release_acceptance': False}
    write(out / 'result.json', report)
    start = time.monotonic()
    with (out / 'stdout.log').open('xb') as stdout, (out / 'stderr.log').open('xb') as stderr:
        code = subprocess.run(command, cwd=REPO, env={**os.environ, **changes}, stdout=stdout, stderr=stderr, check=False).returncode
    unchanged = sources == [seal(p) for p in sorted(files)]
    sdk_unchanged = installed_sdk == sdk_files() if installed_sdk else True
    formatter = mode.endswith('-format-write')
    originals_unchanged = before_originals == [seal(p) for p in sorted(originals)]
    passed = code == 0 and (unchanged or formatter) and sdk_unchanged and originals_unchanged
    if mode == 'go-format':
        passed = passed and not (out / 'stdout.log').read_bytes()
    report.update(status='passed' if passed else 'failed', exit_code=code, seconds=time.monotonic() - start, consumed_inputs_unchanged=unchanged, complete_original_test_inputs_unchanged=originals_unchanged, expected_formatter_mutation=formatter, whole_registered_SDK_unchanged=sdk_unchanged, stdout=seal(out / 'stdout.log'), stderr=seal(out / 'stderr.log'))
    write(out / 'result.json', report)
    print(json.dumps({k: report[k] for k in ['status', 'exit_code', 'seconds', 'consumed_inputs_unchanged', 'whole_registered_SDK_unchanged']}))
    raise SystemExit(not passed)


if __name__ == '__main__':
    main()
