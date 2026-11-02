"""Verify the final error controls and retain the prior full regression scope."""
from pathlib import Path
import hashlib
import json
import tarfile

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[5]
TEST = REPO / 'crates/mpk-vc/tests/go_practical_analysis_archives.rs'


def seal(path):
    data = path.read_bytes()
    return {'path': str(path.resolve()), 'bytes': len(data), 'sha256': hashlib.sha256(data).hexdigest(), 'mode': path.stat().st_mode & 0o777}


def main():
    previous = json.loads((ROOT / 'archive-selected/result.json').read_bytes())
    assert previous['status'] == 'passed' and previous['consumed_inputs_unchanged'] and previous['complete_original_test_inputs_unchanged']
    with tarfile.open(ROOT / 'archive-selected/inputs.tar.gz') as archive:
        prior = archive.extractfile('crates/mpk-vc/tests/go_practical_analysis_archives.rs').read().decode()
    prior = prior.replace('result.unwrap().supplied_files().len()', 'result.as_ref().unwrap().supplied_files().len()')
    old = '''        }
    }
}

#[test]
fn original_owners_reject_duplicates_and_accept_original_nested_roots()'''
    new = '''        }
        let error = match name {
            "changed-vendor-go" => Some(GoAnalysisVendorArchiveError::Content),
            "missing-only-package-file" => Some(GoAnalysisVendorArchiveError::MissingPackage),
            "unknown-asset" => Some(GoAnalysisVendorArchiveError::Source),
            "changed-archive" => Some(GoAnalysisVendorArchiveError::Archive),
            _ => None,
        };
        if let Some(error) = error {
            assert_eq!(result.as_ref().unwrap_err(), &error, "{name}");
        }
    }
}

#[test]
fn original_owners_reject_duplicates_and_accept_original_nested_roots()'''
    assert prior.count(old) == 1 and prior.replace(old, new) == TEST.read_text()
    inputs = json.loads((ROOT / 'archive-selected/inputs.json').read_bytes())
    changed = {str(TEST.resolve()), str((ROOT / 'run_scoped.py').resolve())}
    stable = [row for row in inputs['sources'] if row['path'] not in changed]
    assert all(seal(Path(row['path'])) == row for row in stable)
    for name in ['archive-controls-final', 'archive-clippy-controls-final', 'rust-format-controls-check']:
        result = json.loads((ROOT / name / 'result.json').read_bytes())
        assert result['status'] == 'passed' and result['consumed_inputs_unchanged']
    assert 'test result: ok. 2 passed;' in (ROOT / 'archive-controls-final/stdout.log').read_text()
    result = {'status': 'passed', 'driver': seal(Path(__file__).resolve()), 'previous_full_regression': seal(ROOT / 'archive-selected/result.json'), 'final_controls': seal(ROOT / 'archive-controls-final/result.json'), 'final_clippy': seal(ROOT / 'archive-clippy-controls-final/result.json'), 'final_format': seal(ROOT / 'rust-format-controls-check/result.json'), 'unchanged_prior_sources_checked': len(stable), 'test_change_only_adds_error_class_assertions': True, 'original_full_regression_not_repeated': True, 'native_Linux_release_acceptance': False, 'whole_repository_gate_run': False, 'full_T_gate': 'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    out = ROOT / 'control-inspection.json'
    assert not out.exists()
    out.write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps({key: result[key] for key in ['status', 'unchanged_prior_sources_checked', 'test_change_only_adds_error_class_assertions']}))


if __name__ == '__main__':
    main()
