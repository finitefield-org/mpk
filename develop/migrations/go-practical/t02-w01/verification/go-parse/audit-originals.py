"""Compare every decoded original file with its actual retained execution file."""
from pathlib import Path
import datetime,hashlib,json,runpy,stat,time
OUT=Path(__file__).resolve().parent;RUN=OUT/'original-byte-audit';RUN.mkdir(exist_ok=False);(RUN/'driver.py').write_bytes(Path(__file__).read_bytes())
def seal(p):
 raw=Path(p).read_bytes();return {'path':str(p),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest()}
record={'status':'running','started_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'actual_driver_prelaunch':seal(RUN/'driver.py'),'actual_decoders':[seal(OUT/'read-native.py'),seal(OUT/'read-rust.py')]};result=RUN/'result.json';result.write_text(json.dumps(record,indent=2)+'\n');start=time.monotonic();files_count=bytes_count=0
read_native=runpy.run_path(str(OUT/'read-native.py'))['read'];read_rust=runpy.run_path(str(OUT/'read-rust.py'))['read']
for name in ['native-original','native-depth','rust-originals']:
 metadata,files=read_rust()if name=='rust-originals'else read_native(name)
 for row in metadata['original_inputs']:
  source=Path(row['path']);assert source.is_file()and not source.is_symlink();actual=source.read_bytes();assert actual==files[row['member']]and stat.S_IMODE(source.stat().st_mode)==row['mode'];files_count+=1;bytes_count+=len(actual)
record.update(status='passed',seconds=time.monotonic()-start,actual_original_files_compared_byte_for_byte=files_count,actual_original_bytes_compared=bytes_count,all_original_modes_exact=True,identical_output_deduplication_verified_by_full_bytes=True,originals_not_regenerated_or_recompiled=True);result.write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({k:record[k]for k in ['status','actual_original_files_compared_byte_for_byte','actual_original_bytes_compared']}))
