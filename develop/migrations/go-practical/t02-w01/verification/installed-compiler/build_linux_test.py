"""Build the real private-boundary test executable in the retained Linux image."""
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
import tomllib

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[5]
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT))
from run_scoped import seal, write
sys.path.insert(0, str(REPO / 'develop/probes/go-practical'))
import policy_native_build


def main():
    out = ROOT / (sys.argv[1] if len(sys.argv) == 2 else 'linux-test-build')
    out.mkdir(exist_ok=False)
    paths = {Path(__file__).resolve(), ROOT / 'run_scoped.py', REPO / 'crates/mpk-cli/tests/go_practical_installed_compiler.rs'}
    paths.update(REPO / r['path'] for r in policy_native_build.linked_inputs())
    paths.update(REPO / 'develop/probes/go-practical' / n for n in ['policy_native_build.py', 'frontend_native_build.py', 'run_proof_pilot.py', 'check_scope_report.py', 'policy-native/Cargo.lock'])
    sources = [seal(p) for p in sorted(paths)]
    with tarfile.open(out / 'inputs.tar.gz', 'x:gz', dereference=True) as archive:
        for p in sorted(paths):
            archive.add(p, arcname=str(p.relative_to(REPO)), recursive=False)
    write(out / 'inputs.json', {'sources': sources, 'source_archive': seal(out / 'inputs.tar.gz'), 'selection_reason': 'Compile only the actual installed Go compiler integration test, including the exact production installed root, private SDK authority and independent full source/export/compiler importer. Retain all linked sources, reviewed registries, include resources, original root lock and complete private codec package. No whole gate, removed-language tests or native Linux release acceptance.'})
    report = {'status': 'running', 'started_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(), 'inputs': seal(out / 'inputs.json'), 'native_Linux_release_acceptance': False}
    write(out / 'result.json', report)
    calls = []
    original_run = subprocess.run

    def observed(*args, **kwargs):
        number = len(calls)
        entry = {'command': list(map(str, args[0]))}
        calls.append(entry)
        stdout, stderr = b'', b''
        try:
            result = original_run(*args, **kwargs)
            entry['exit_code'] = result.returncode
            stdout, stderr = result.stdout or b'', result.stderr or b''
        except subprocess.TimeoutExpired as error:
            entry.update(terminal='timeout', timeout_seconds=error.timeout)
            stdout, stderr = error.stdout or b'', error.stderr or b''
            raise
        finally:
            for name, data in [('stdout', stdout), ('stderr', stderr)]:
                path = out / f'command-{number:02d}-{name}.log'
                path.write_bytes(data)
                entry[name] = seal(path)
            write(out / 'commands.json', calls)
        return result

    subprocess.run = observed
    start = time.monotonic()
    try:
        prefix, audit = policy_native_build.prefix()
        test_environment = {'CARGO_PROFILE_TEST_OPT_LEVEL': '1', 'CARGO_PROFILE_TEST_DEBUG_ASSERTIONS': 'true', 'CARGO_PROFILE_TEST_OVERFLOW_CHECKS': 'true', 'CARGO_PROFILE_TEST_DEBUG': '0'}
        for name, value in test_environment.items():
            prefix[-1:-1] = ['--env', f'{name}={value}']
        report['test_environment'] = test_environment
        # The CLI test also compiles its existing dev-dependencies. Use only
        # exact root-lock cache archives; never resolve a newer dependency set.
        archives = []
        registry = Path.home() / '.cargo/registry/cache'
        for row in tomllib.loads((REPO / 'Cargo.lock').read_text())['package']:
            if 'source' not in row:
                continue
            assert row['source'] == 'registry+https://github.com/rust-lang/crates.io-index'
            candidates = [p for p in registry.glob(f'*/{row["name"]}-{row["version"]}.crate') if seal(p)['sha256'] == row['checksum']]
            assert candidates, f'exact cached crate missing: {row["name"]}-{row["version"]}'
            source = sorted(candidates)[0]
            destination = policy_native_build.CARGO / 'registry/cache' / source.parent.name / source.name
            destination.parent.mkdir(parents=True, exist_ok=True)
            if not destination.exists() or seal(destination)['sha256'] != row['checksum']:
                shutil.copyfile(source, destination)
            archives.append(seal(source))
        command = prefix + ['cargo', 'test', '--offline', '--locked', '-p', 'mpk-cli', '--test', 'go_practical_installed_compiler', '--no-run', '--message-format=json']
        result = subprocess.run(command, cwd=REPO, capture_output=True, check=False)
        assert result.returncode == 0, result.stderr.decode(errors='replace')
        binaries = [json.loads(line)['executable'] for line in result.stdout.splitlines() if json.loads(line).get('reason') == 'compiler-artifact' and json.loads(line).get('executable') and json.loads(line)['target']['name'] == 'go_practical_installed_compiler']
        assert len(binaries) == 1 and binaries[0].startswith('/target/')
        executable = policy_native_build.TARGET / Path(binaries[0]).relative_to('/target')
        assert executable.read_bytes().startswith(b'\x7fELF')
        report.update(status='passed', executable=seal(executable), container_executable=binaries[0], audit=audit, dependency_archives=archives)
    except BaseException as error:
        report.update(status='failed', error=repr(error))
        raise
    finally:
        subprocess.run = original_run
        report.update(seconds=time.monotonic() - start, consumed_inputs_unchanged=sources == [seal(p) for p in sorted(paths)], full_T_gate='Deferred to final T02-W04: ./scripts/check-fast.sh')
        write(out / 'result.json', report)
    print(json.dumps({k: report[k] for k in ['status', 'seconds', 'consumed_inputs_unchanged', 'executable']}))


if __name__ == '__main__':
    main()
