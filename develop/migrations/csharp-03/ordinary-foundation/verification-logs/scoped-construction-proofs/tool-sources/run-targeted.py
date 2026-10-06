"""Test all 45 original contexts and 12 scoped fill/freeze source points with exact original W06 sequents, checked flow/receiver dependencies, strict imports, unchanged certificate prefixes and an actual-kernel typed ownership mutation. Include the legacy 442-proof allocation route and all 45 original operation pins because shared serialized operation/proof structures and domain-symbol lowering changed. Run crate Clippy and crate/support format. Generic ownership, original is_binding refinements, native execution and all 987 application proof IDs remain pending. The full T06 gate is deferred to T06-W12.
"""
import argparse
from datetime import datetime, timezone
from hashlib import sha256
import json
import os
from pathlib import Path
import re
import subprocess
import time


def now():
    return datetime.now(timezone.utc).isoformat()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--repo', type=Path, required=True)
    parser.add_argument('--reports', type=Path, required=True)
    parser.add_argument('--manifest', type=Path, required=True)
    parser.add_argument('--expected-certificates', type=Path)
    parser.add_argument('--target', type=Path, required=True)
    parser.add_argument('--cargo', default='cargo')
    args = parser.parse_args()
    args.reports.mkdir(parents=True, exist_ok=True)
    manifest = json.loads(args.manifest.read_bytes())
    expected_certificates = json.loads(args.expected_certificates.read_bytes()) if args.expected_certificates else None
    state = dict(status='preparing', supervisor_pid=os.getpid(), started_at=now(),
                 stages=[], selection_reason=__doc__, application_proofs_pending=987,
                 complete_application_assembly_pending=True,
                 full_t_gate='deferred to T06-W12')

    def persist():
        path = args.reports / 'status.tmp'
        path.write_text(json.dumps(state, indent=2, sort_keys=True) + '\n')
        os.replace(path, args.reports / 'status.json')

    def verify():
        for group in ('source_hashes', 'fixture_hashes'):
            for name, digest in manifest[group].items():
                assert sha256((args.repo / name).read_bytes()).hexdigest() == digest, name

    persist()
    try:
        verify()
        (args.reports / 'source-manifest.json').write_bytes(args.manifest.read_bytes())
        state.update(verified_source_hashes=len(manifest['source_hashes']),
                     verified_fixture_hashes=len(manifest['fixture_hashes']))
        env = os.environ.copy()
        env.update(CARGO_TARGET_DIR=str(args.target), CARGO_BUILD_JOBS='2', CARGO_INCREMENTAL='0',
                   CARGO_PROFILE_TEST_OPT_LEVEL='2', CARGO_PROFILE_TEST_DEBUG='0',
                   CARGO_PROFILE_TEST_DEBUG_ASSERTIONS='true', CARGO_PROFILE_TEST_OVERFLOW_CHECKS='true')
        for name in ('MPK_W09_CONTROL_PATTERN_ROUTE_OUTPUT', 'MPK_W09_CONTROL_PATTERN_SOURCE_OUTPUT', 'MPK_W09_PATTERN_OBSERVATION_OUTPUT',
                     'MPK_W09_CONTROL_PATTERN_CAPTURE_OUTPUT', 'MPK_W09_CONTROL_PATTERN_SCOPE_OUTPUT',
                     'MPK_W09_CONTROL_PREDICATE_INTEGRATED_OUTPUT', 'MPK_W09_CONTROL_PREDICATE_OUTPUT'):
            env.pop(name, None)
        env.pop('MPK_W09_PATTERN_PROOF_TYPES_OUTPUT', None)
        env.pop('MPK_W09_PATTERN_PROOF_TYPES_UNIT_OUTPUT', None)
        env.pop('MPK_W09_PACKED_PATTERN_OUTPUT', None)
        env.pop('MPK_W09_PACKED_PATTERN_UNIT_OUTPUT', None)
        env.pop('MPK_W09_PATTERN_CANDIDATE_UNIT_OUTPUT', None)
        env.pop('MPK_W09_CONTEXT_BOOLEAN_UNIT_OUTPUT', None)
        env.pop('MPK_W09_OWNERSHIP_PROOFS_OUT', None)
        env.pop('MPK_W09_DEFAULT_CLOSED_PROOFS_OUT', None)
        test = ['test', '-p', 'mpk-vc', '--test', 'csharp_practical_vc']
        tail = ['--', '--nocapture', '--test-threads=1']
        jobs = [
            ('scoped-construction-proofs', test + ['csharp_03_t06_w09_scoped_construction_operation_proofs_original_source'] + tail, 1),
            ('original-allocation-proofs', test + ['csharp_03_t06_w09_concrete_allocation_proofs_original_source'] + tail, 1),
            ('original-operation-pins', test + ['csharp_03_t06_w09_concrete_operations_original_source_certificates'] + tail, 1),
            ('clippy', ['clippy', '-p', 'mpk-vc', '--lib', '--tests', '--', '-D', 'warnings'], 0),
            ('format', ['fmt', '-p', 'mpk-vc', '--', '--check'], 0),
        ]
        env.pop('MPK_W09_CONCRETE_TYPES_OUT', None)
        env.pop('MPK_W09_CONCRETE_TYPE_PROOFS_OUT', None)
        env.pop('MPK_W09_SCOPED_SOURCE_FILTER', None)
        binaries = {}
        for label, arguments, count in jobs:
            verify()
            env['MPK_W09_SCOPED_CONSTRUCTION_PROOFS_OUT'] = str(args.reports / 'certificates')
            env['MPK_W09_CONCRETE_OPERATION_PROOFS_OUT'] = str(args.reports / 'legacy-certificates')
            state.update(status='running', stage=label)
            started = now()
            start = time.monotonic()
            log = args.reports / f'{label}.log'
            with log.open('wb') as out:
                process = subprocess.Popen([args.cargo, *arguments], cwd=args.repo, env=env,
                                           stdin=subprocess.DEVNULL, stdout=out, stderr=subprocess.STDOUT)
                state['process_pid'] = process.pid
                persist()
                code = process.wait()
            state['stages'].append(dict(stage=label, command=[args.cargo, *arguments],
                                       exit_code=code, started_at=started, finished_at=now(),
                                       elapsed_seconds=round(time.monotonic() - start, 3),
                                       log_sha256=sha256(log.read_bytes()).hexdigest()))
            persist()
            print(json.dumps(state['stages'][-1]), flush=True)
            assert code == 0, (label, code)
            if count:
                output = log.read_text()
                assert f'test result: ok. {count} passed; 0 failed;' in output, label
                for name in re.findall(r'Running [^\n]*\(([^\n]+)\)', output):
                    binaries[name] = sha256(Path(name).read_bytes()).hexdigest()
            verify()
        verify()
        support_command = ['rustfmt', '--edition', '2021', '--check', 'crates/mpk-vc/tests/support/csharp_practical_ordinary_concrete_operation_tests.rs', 'crates/mpk-vc/tests/support/csharp_practical_ordinary_scoped_construction_tests.rs']
        log = args.reports / 'support-format.log'
        with log.open('wb') as output:
            result = subprocess.run(support_command, cwd=args.repo, env=env, stdout=output, stderr=subprocess.STDOUT)
        state['stages'].append(dict(stage='support-format', command=support_command, exit_code=result.returncode,
                                   log_sha256=sha256(log.read_bytes()).hexdigest()))
        assert result.returncode == 0
        emitted = sorted((args.reports / 'certificates').glob('*-program.json'))
        assert len(emitted) == 45
        supplied = sum(len(json.loads(p.read_bytes())['candidates']) for p in emitted)
        pending_source = sum(len(json.loads(p.read_bytes())['pending_source_operation_ids']) for p in emitted)
        generic_pending = sum(len(json.loads(p.read_bytes())['generic_pending_operations']) for p in emitted)
        pending_ids = sum(len(json.loads(p.read_bytes())['pending_proof_ids']) for p in emitted)
        assert (supplied, pending_source, generic_pending, pending_ids) == (12, 0, 25, 987)
        state['exported_certificate_hashes'] = {str(p.relative_to(args.reports)): sha256(p.read_bytes()).hexdigest()
                                              for p in (args.reports / 'certificates').iterdir() if p.is_file()}
        actual_catalog = {Path(name).name: digest for name, digest in state['exported_certificate_hashes'].items()}
        assert len(actual_catalog) == 70
        if expected_certificates is not None:
            assert actual_catalog == expected_certificates
        (args.reports / 'expected-certificates.json').write_text(json.dumps(actual_catalog, indent=2, sort_keys=True) + '\n')
        state['legacy_certificate_hashes'] = {p.name: sha256(p.read_bytes()).hexdigest() for p in (args.reports / 'legacy-certificates').iterdir() if p.is_file()}
        assert len(state['legacy_certificate_hashes']) == 92
        verify()
        state['expected_certificates_sha256'] = sha256((args.reports / 'expected-certificates.json').read_bytes()).hexdigest()
        state['exported_certificates_match_local'] = True
        state.update(scoped_candidates=supplied, source_operations_pending=pending_source, generic_operations_pending=generic_pending, observed_operation_kinds=['fill','freeze'], source_ownership_pending=True)
        state.update(status='passed_targeted_tests_lint_format', finished_at=now(),
                     unique_tests_passed=3, test_binaries=binaries)
        persist()
    except BaseException as error:
        state.update(status='failed', finished_at=now(), error=repr(error))
        persist()
        raise


if __name__ == '__main__':
    main()
