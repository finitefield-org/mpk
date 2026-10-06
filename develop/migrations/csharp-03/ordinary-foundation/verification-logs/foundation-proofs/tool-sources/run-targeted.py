"""Test the unified original concrete-type/operation proof assembly in all 45 original source contexts. Independently match all complete W06 sequents and original checked operation prefixes; check named universal type/equation operands, strict metadata and cross-context imports, and a typed false concrete-type mutation in the actual Rust kernel. Include the existing private-storage type proof and allocation proof exporters to confirm both consumed component routes remain unchanged. Run crate Clippy and crate/support format. All 25 generic operations, original refinements, native/application scopes and 987 original application proof IDs remain pending. Defer the full T06 gate to T06-W12.
"""
import argparse
from datetime import datetime, timezone
from hashlib import sha256
import json
import os
from pathlib import Path
import re
import subprocess
import shutil
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
    parser.add_argument('--reuse-unchanged-components', type=Path)
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
            ('foundation-proofs', test + ['csharp_03_t06_w09_foundation_proof_assembly_original_source'] + tail, 1),
            ('original-allocation-proofs', test + ['csharp_03_t06_w09_concrete_allocation_proofs_original_source'] + tail, 1),
            ('original-storage-type-proofs', test + ['csharp_03_t06_w09_construction_type_proofs_original_source'] + tail, 1),
            ('clippy', ['clippy', '-p', 'mpk-vc', '--lib', '--tests', '--', '-D', 'warnings'], 0),
            ('format', ['fmt', '-p', 'mpk-vc', '--', '--check'], 0),
        ]
        if args.reuse_unchanged_components:
            previous = json.loads((args.reuse_unchanged_components/'status.json').read_bytes())
            previous_manifest = json.loads((args.reuse_unchanged_components/'source-manifest.json').read_bytes())
            assert previous['status'] == 'passed_targeted_tests_lint_format' and previous['unique_tests_passed'] == 3
            assert previous_manifest['fixture_hashes'] == manifest['fixture_hashes']
            assert previous_manifest['source_hashes'].keys() == manifest['source_hashes'].keys()
            changed = [name for name,digest in manifest['source_hashes'].items() if previous_manifest['source_hashes'][name] != digest]
            assert set(changed) == {'crates/mpk-vc/src/csharp_practical_ordinary_foundation_proofs.rs', 'crates/mpk-vc/tests/support/csharp_practical_ordinary_foundation_proof_tests.rs'}
            for binary,digest in previous['test_binaries'].items():
                assert sha256((args.reuse_unchanged_components/'test-binaries'/Path(binary).name).read_bytes()).hexdigest() == digest
            for stage in previous['stages']:
                assert stage['exit_code'] == 0
                assert sha256((args.reuse_unchanged_components/(stage['stage']+'.log')).read_bytes()).hexdigest() == stage['log_sha256']
            for folder in ('type-certificates','operation-certificates'):
                shutil.copytree(args.reuse_unchanged_components/folder,args.reports/folder)
            jobs = [job for job in jobs if job[0] not in ('original-allocation-proofs','original-storage-type-proofs')]
            state['reused_unchanged_component_receipt'] = dict(path=str(args.reuse_unchanged_components),
                status_sha256=sha256((args.reuse_unchanged_components/'status.json').read_bytes()).hexdigest(),
                source_manifest_sha256=sha256((args.reuse_unchanged_components/'source-manifest.json').read_bytes()).hexdigest(),
                changed_files=changed,unique_tests_reused=2,
                reason='The final change only checks the unified adapter cumulative transformer count and asserts that count in its new source test. Original standalone type/operation exporters and their transitive source files are unchanged. Repeat the affected assembly test, lint and format; preserve the two passing component tests and exact exports.')
        env.pop('MPK_W09_CONCRETE_TYPES_OUT', None)
        env.pop('MPK_W09_CONCRETE_TYPE_PROOFS_OUT', None)
        env.pop('MPK_W09_FOUNDATION_SOURCE_FILTER', None)
        env.pop('MPK_W09_SCOPED_SOURCE_FILTER', None)
        binaries = {}
        for label, arguments, count in jobs:
            verify()
            env['MPK_W09_FOUNDATION_PROOFS_OUT'] = str(args.reports / 'certificates')
            env['MPK_W09_CONCRETE_TYPE_PROOFS_OUT'] = str(args.reports / 'type-certificates')
            env['MPK_W09_CONCRETE_OPERATION_PROOFS_OUT'] = str(args.reports / 'operation-certificates')
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
        support_command = ['rustfmt', '--edition', '2021', '--check', 'crates/mpk-vc/tests/support/csharp_practical_ordinary_binding_relation_tests.rs', 'crates/mpk-vc/tests/support/csharp_practical_ordinary_foundation_proof_tests.rs']
        log = args.reports / 'support-format.log'
        with log.open('wb') as output:
            result = subprocess.run(support_command, cwd=args.repo, env=env, stdout=output, stderr=subprocess.STDOUT)
        state['stages'].append(dict(stage='support-format', command=support_command, exit_code=result.returncode,
                                   log_sha256=sha256(log.read_bytes()).hexdigest()))
        assert result.returncode == 0
        emitted = sorted((args.reports / 'certificates').glob('*.json'))
        assert len(emitted) == 45
        metadata = [json.loads(p.read_bytes()) for p in emitted]
        types = sum(len(m['types']['proofs']) for m in metadata)
        operations = sum(len(m['operations']['proofs']) for m in metadata)
        supplied = sum(len(m['supplied_binding_sequent_ids']) for m in metadata)
        remaining = sum(len(m['remaining_binding_sequent_ids']) for m in metadata)
        pending = sum(len(m['pending_proof_ids']) for m in metadata)
        assert (types, operations, supplied, remaining, pending) == (87, 442, 529, 458, 987)
        assert sum(len(m['operations']['pending_operations']) for m in metadata) == 25
        assert all(m['proof_check_pending'] and m['application_scope_pending'] for m in metadata)
        actual_catalog = {p.name: sha256(p.read_bytes()).hexdigest() for p in (args.reports / 'certificates').iterdir() if p.is_file()}
        assert len(actual_catalog) == 91
        if expected_certificates is not None:
            assert actual_catalog == expected_certificates
        state['exported_certificate_hashes'] = {'certificates/' + name: digest for name, digest in actual_catalog.items()}
        (args.reports / 'expected-certificates.json').write_text(json.dumps(actual_catalog, indent=2, sort_keys=True) + '\n')
        legacy = [('type-certificates', 'construction-storage-types/attempt-2/certificates'), ('operation-certificates', 'concrete-allocation-proofs/attempt-5/certificates')]
        for folder, expected_folder in legacy:
            catalog = {p.name: sha256(p.read_bytes()).hexdigest() for p in (args.reports / folder).iterdir() if p.is_file()}
            published = args.repo / 'develop/migrations/csharp-03/ordinary-foundation/verification-logs' / expected_folder
            assert len(catalog) == 92
            assert catalog == {p.name: sha256(p.read_bytes()).hexdigest() for p in published.iterdir() if p.is_file()}
            state[folder.replace('-', '_') + '_hashes'] = catalog
        verify()
        state.update(supplied_type_proofs=types, supplied_operation_proofs=operations,
                     supplied_binding_sequents=supplied, remaining_binding_sequents=remaining,
                     original_application_proofs_pending=pending, generic_operations_pending=25,
                     source_contexts=45, legacy_certificate_bytes_unchanged=True,
                     exported_certificates_match_local=True,
                     expected_certificates_sha256=sha256((args.reports/'expected-certificates.json').read_bytes()).hexdigest())
        state.update(status='passed_targeted_tests_lint_format', finished_at=now(),
                     unique_tests_passed=1 if args.reuse_unchanged_components else 3, test_binaries=binaries)
        persist()
    except BaseException as error:
        state.update(status='failed', finished_at=now(), error=repr(error))
        persist()
        raise


if __name__ == '__main__':
    main()
