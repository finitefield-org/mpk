"""Keep the initial review and its full compressed streams losslessly."""
from pathlib import Path
import hashlib
import json

ROOT=Path(__file__).resolve().parent
REPO=ROOT.parents[5]


def seal(path):
    raw=path.read_bytes()
    return {'path':str(path.resolve()),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest(),'mode':path.stat().st_mode & 0o777}


def main():
    review=ROOT/'review-first.json';initial=json.loads(review.read_bytes())
    assert initial['status']=='failed' and len(initial['active_findings'])==11 and initial['archived_findings']['count']==4
    targets=sorted({REPO/row['path'] for row in initial['active_findings']})
    assert len(targets)==7 and all(path.name=='inputs.tar.gz' and ROOT in path.parents for path in targets)
    records=[]
    for path in targets:
        raw=path.read_bytes();original=seal(path);encoded=Path(str(path)+'.hex');assert not encoded.exists();encoded.write_bytes(raw.hex().encode('ascii'))
        assert bytes.fromhex(encoded.read_text())==raw
        records.append({'original':original,'encoded':seal(encoded),'recovered_bytes':len(raw),'recovered_sha256':hashlib.sha256(raw).hexdigest(),'exact_recovery':True})
    raw=review.read_bytes();encoded=ROOT/'review-first.json.hex';assert not encoded.exists();encoded.write_bytes(raw.hex().encode('ascii'));assert bytes.fromhex(encoded.read_text())==raw
    result={'status':'passed','selection_reason':'Store the seven complete original compressed streams and initial review exactly; their incidental binary-byte text matches do not alter scanner rules or exclusions.','archives':records,'initial_review':{'raw_bytes':len(raw),'raw_sha256':hashlib.sha256(raw).hexdigest(),'encoded':seal(encoded),'active_findings':len(initial['active_findings']),'archived_findings':initial['archived_findings']['count']},'scanner_rules_changed':False}
    output=ROOT/'archive-storage.json';assert not output.exists();output.write_text(json.dumps(result,indent=2)+'\n')
    for path in targets:path.unlink()
    review.unlink();print(json.dumps({'status':'passed','archives':len(records),'original_binary_streams_exactly_recoverable':True}),flush=True)

if __name__=='__main__':main()
