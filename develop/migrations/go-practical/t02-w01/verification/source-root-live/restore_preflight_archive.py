"""Recover the exact first preflight archive and its original scan finding."""
from pathlib import Path
import hashlib
import json
import sys

ROOT=Path(__file__).resolve().parent
destination=Path(sys.argv[1]);assert destination.is_absolute() and not destination.exists();destination.mkdir(mode=0o700)
record=json.loads((ROOT/'preflight-archive-storage.json').read_bytes());assert record['status']=='passed'
raw=bytes.fromhex((ROOT/'preflight/inputs.tar.gz.hex').read_text());expected=record['historical_receipt'];assert len(raw)==expected['bytes'] and hashlib.sha256(raw).hexdigest()==expected['sha256']
archive=destination/'inputs.tar.gz';archive.write_bytes(raw);archive.chmod(expected['mode']);assert archive.read_bytes()==raw
findings=bytes.fromhex((ROOT/'preflight-scan-findings.json.hex').read_text());assert len(findings)==record['findings']['raw_bytes'] and hashlib.sha256(findings).hexdigest()==record['findings']['raw_sha256']
(destination/'scan-findings.json').write_bytes(findings)
print(json.dumps({'status':'passed','archive_bytes':len(raw),'archive_sha256':expected['sha256'],'finding_count':record['findings']['count'],'exact_recovery':True,'destination':str(destination)}))
