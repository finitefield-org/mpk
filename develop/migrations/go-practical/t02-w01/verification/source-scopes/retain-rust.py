"""Retain every actual emitted Rust file, including the original partial run."""
from pathlib import Path
import datetime,hashlib,json
OUT=Path(__file__).resolve().parent
sources={
 'original-partial':OUT/'paired/rust-selected/result.json',
 'exact-paired':OUT/'bound-receiver/rust-paired/result.json',
 'blockers':OUT/'bound-receiver/rust-blockers/result.json',
}
run=OUT/'rust-originals';run.mkdir(exist_ok=False);(run/'driver.py').write_bytes(Path(__file__).read_bytes())
rows=[]
for name,result in sources.items():
 record=json.loads(result.read_bytes());assert record['status']in {'passed','failed'}
 source=Path(record['environment']['MPK_GO_SOURCE_SCOPES_RUST_OUTPUT'])
 for p in sorted(source.rglob('*')):
  assert not p.is_symlink()
  if not p.is_file():continue
  raw=p.read_bytes();mode=p.stat().st_mode&0o777;target=run/name/p.relative_to(source);target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(raw);target.chmod(mode)
  assert target.read_bytes()==raw and target.stat().st_mode&0o777==mode
  rows.append({'original':str(p),'retained':str(target),'mode':mode,'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest()})
record={'status':'passed','observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'actual_driver_prelaunch_sha256':hashlib.sha256((run/'driver.py').read_bytes()).hexdigest(),'original_files':len(rows),'original_bytes':sum(r['bytes']for r in rows),'all_literal_original_bytes_and_modes_compared':True,'files':rows}
(run/'manifest.json').write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({k:record[k]for k in ['status','original_files','original_bytes']}))
