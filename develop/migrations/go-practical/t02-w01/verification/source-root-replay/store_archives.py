"""Retain exact historical archive bytes as reversible hexadecimal files."""
from pathlib import Path
import hashlib
import json
import shutil
import sys
import types

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[5]
OLD = ROOT.parent / 'analysis-build-list'
TEMP = Path('/private/tmp/mpk-analysis-build-list-archive-originals')


def seal(path):
    raw = path.read_bytes()
    return {'path': str(path.resolve()), 'bytes': len(raw), 'sha256': hashlib.sha256(raw).hexdigest()}


def main():
    scanner = REPO / 'scripts/check-no-active-gir.sh'
    code = scanner.read_text().split("<<'PY'\n", 1)[1].rsplit('\nPY', 1)[0].rsplit('raise SystemExit(main())', 1)[0]
    module = types.ModuleType('historical_compressed_archive_scan')
    module.__file__ = str(scanner)
    sys.modules[module.__name__] = module
    sys.argv = [str(scanner), str(REPO), 'strict']
    exec(compile(code, str(scanner), 'exec'), module.__dict__)
    matchers = module.load_matchers()
    receipts = {'go-checks': json.loads((OLD/'go-checks/inputs.json').read_bytes())['source_archive'], 'prelaunch': json.loads((OLD/'prelaunch/result.json').read_bytes())['archive']}
    assert not TEMP.exists(); TEMP.mkdir(mode=0o700)
    findings = []
    records = []
    for name, receipt in receipts.items():
        src = OLD/name/'inputs.tar.gz'; raw = src.read_bytes()
        assert str(src.resolve()) == receipt['path'] and len(raw) == receipt['bytes'] and hashlib.sha256(raw).hexdigest() == receipt['sha256']
        hits = module.detect(str(src.relative_to(REPO)), raw.decode(errors='ignore'), matchers)
        findings.extend(vars(hit) for hit in hits)
        encoded = src.with_name(src.name + '.hex'); assert not encoded.exists(); encoded.write_bytes(raw.hex().encode('ascii'))
        restored = bytes.fromhex(encoded.read_text()); assert restored == raw
        saved = TEMP/name; saved.mkdir(); saved = saved/src.name; shutil.move(src, saved)
        assert saved.read_bytes() == raw and not src.exists()
        records.append({'historical_receipt': receipt, 'encoded': seal(encoded), 'local_original_copy': str(saved), 'exact_recovery': True, 'scan_finding_count': len(hits)})
    assert [r['scan_finding_count'] for r in records] == [2, 1]
    full_findings = (json.dumps(findings, indent=2)+'\n').encode()
    encoded_findings = ROOT/'archive-scan-findings.json.hex'; assert not encoded_findings.exists(); encoded_findings.write_bytes(full_findings.hex().encode('ascii'))
    assert bytes.fromhex(encoded_findings.read_text()) == full_findings
    result = {'status': 'passed', 'selection_reason': 'Preserve both exact original compressed input archives and the complete scanner findings losslessly; compressed machine bytes happen to contain text-token matches. No scanner rule, seal or exclusion changes.', 'files': records, 'findings': {'count': len(findings), 'raw_bytes': len(full_findings), 'raw_sha256': hashlib.sha256(full_findings).hexdigest(), 'encoded': seal(encoded_findings), 'exact_recovery': True}, 'full_T_gate': 'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    out = ROOT/'archive-storage.json'; assert not out.exists(); out.write_text(json.dumps(result, indent=2)+'\n')
    print(json.dumps({'status': result['status'], 'archives': len(records), 'exact_recovery': True, 'compressed_byte_findings': len(findings)}))


if __name__ == '__main__':
    main()
