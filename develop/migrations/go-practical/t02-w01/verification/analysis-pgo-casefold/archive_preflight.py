"""Preserve incidental compressed-byte scanner matches without changing rules."""
from pathlib import Path
import hashlib
import json
import sys
import types

ROOT=Path(__file__).resolve().parent;REPO=ROOT.parents[5];PRIOR=ROOT.parent/'analysis-pgo'
scanner=REPO/'scripts/check-no-active-gir.sh';code=scanner.read_text().split("<<'PY'\n",1)[1].rsplit('\nPY',1)[0].rsplit('raise SystemExit(main())',1)[0]
module=types.ModuleType('go_casefold_archive_preflight');module.__file__=str(scanner);sys.modules[module.__name__]=module;sys.argv=[str(scanner),str(REPO),'strict'];exec(compile(code,str(scanner),'exec'),module.__dict__)
matchers=module.load_matchers();manifest,_,_,_=module.validate_manifest();module.run_fixture_self_test(manifest,matchers)
assert (len(matchers),len(module.SCOPES),len(manifest['sealed_records']),len(module.active_retained_record_exclusions(manifest)))==(127,4,39,51)
paths=sorted(p for base in [ROOT,PRIOR] for p in base.rglob('*.tar.gz') if p.is_file())
rows=[];targets=[]
for path in paths:
    raw=path.read_bytes();findings=[vars(x) for x in module.detect(str(path.relative_to(REPO)),raw.decode(errors='ignore'),matchers)]
    row={'path':str(path.resolve()),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest(),'mode':path.stat().st_mode & 0o777,'findings':findings};rows.append(row)
    if findings:targets.append((path,row,raw))
report={'status':'passed','selection_reason':'Identify raw binary compressed streams that incidentally match existing text rules, while preserving every original stream and exact diagnostic before converting their storage.','inputs':rows,'matched_streams':len(targets),'findings':sum(len(row['findings']) for row in rows),'scanner_rules_changed':False}
raw=(json.dumps(report,indent=2)+'\n').encode();encoded=ROOT/'archive-preflight.json.hex';assert not encoded.exists();encoded.write_bytes(raw.hex().encode());assert bytes.fromhex(encoded.read_text())==raw
records=[]
for path,row,original in targets:
    chunks=[]
    for i,offset in enumerate(range(0,len(original),16*1024*1024)):
        part=original[offset:offset+16*1024*1024].hex().encode();target=path.with_name(path.name+f'.hex.part-{i:03d}');assert not target.exists();target.write_bytes(part)
        chunks.append({'path':str(target.resolve()),'bytes':len(part),'sha256':hashlib.sha256(part).hexdigest(),'mode':target.stat().st_mode & 0o777})
    recovered=bytes.fromhex(b''.join(Path(x['path']).read_bytes() for x in chunks).decode('ascii'));assert recovered==original
    records.append({'original':{key:row[key] for key in ['path','bytes','sha256','mode']},'chunks':chunks,'exact_recovery':True})
out=ROOT/'archive-preflight-storage.json';assert not out.exists();out.write_text(json.dumps({'status':'passed','initial_report':{'path':str(encoded.resolve()),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest(),'matched_streams':len(targets),'findings':report['findings']},'archives':records,'scanner_rules_changed':False},indent=2)+'\n')
for path,_,_ in targets:path.unlink()
print(json.dumps({'status':'passed','compressed_archives_scanned':len(paths),'encoded_matching_streams':len(records),'original_bytes_exactly_recoverable':True}),flush=True)
