"""Preserve the failed first preflight archive in reversible hexadecimal."""
from pathlib import Path
import hashlib
import json
import shutil
import sys
import types

ROOT=Path(__file__).resolve().parent
REPO=ROOT.parents[5]
source=ROOT/'preflight/inputs.tar.gz'
receipt=json.loads((ROOT/'preflight/inputs.json').read_bytes())['archive']
raw=source.read_bytes();assert len(raw)==receipt['bytes'] and hashlib.sha256(raw).hexdigest()==receipt['sha256']
scanner=REPO/'scripts/check-no-active-gir.sh';code=scanner.read_text().split("<<'PY'\n",1)[1].rsplit('\nPY',1)[0].rsplit('raise SystemExit(main())',1)[0]
module=types.ModuleType('live_preflight_archive_review');module.__file__=str(scanner);sys.modules[module.__name__]=module;sys.argv=[str(scanner),str(REPO),'strict'];exec(compile(code,str(scanner),'exec'),module.__dict__)
findings=[vars(item) for item in module.detect(str(source.relative_to(REPO)),raw.decode(errors='ignore'),module.load_matchers())];assert len(findings)==1
encoded=source.with_name(source.name+'.hex');assert not encoded.exists();encoded.write_bytes(raw.hex().encode('ascii'));assert bytes.fromhex(encoded.read_text())==raw
stored=Path('/private/tmp/mpk-live-preflight-original');assert not stored.exists();stored.mkdir(mode=0o700);stored=stored/source.name;shutil.move(source,stored);assert stored.read_bytes()==raw
findings_raw=(json.dumps(findings,indent=2)+'\n').encode();findings_file=ROOT/'preflight-scan-findings.json.hex';findings_file.write_bytes(findings_raw.hex().encode('ascii'));assert bytes.fromhex(findings_file.read_text())==findings_raw
record={'status':'passed','selection_reason':'Keep the complete failed preflight source archive and its one incidental compressed-byte text finding losslessly without changing scanner rules or exclusions.','historical_receipt':receipt,'encoded':{'path':str(encoded.resolve()),'bytes':encoded.stat().st_size,'sha256':hashlib.sha256(encoded.read_bytes()).hexdigest()},'local_original_copy':str(stored),'findings':{'count':len(findings),'raw_bytes':len(findings_raw),'raw_sha256':hashlib.sha256(findings_raw).hexdigest(),'encoded_path':str(findings_file.resolve())},'exact_recovery':True,'full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'}
(ROOT/'preflight-archive-storage.json').write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({'status':'passed','exact_recovery':True,'compressed_byte_findings':1}))
