"""Audit original source list, independently rebuilt binaries and receipts."""
from pathlib import Path
import hashlib
import importlib.util
import json
import sys

ROOT=Path(__file__).resolve().parent
sys.dont_write_bytecode=True
spec=importlib.util.spec_from_file_location('prior_snapshot_inspection',ROOT.parent/'analysis-namespace/inspect_snapshot.py');prior=importlib.util.module_from_spec(spec);spec.loader.exec_module(prior)


def seal(path):
 data=path.read_bytes();return {'path':str(path.resolve()),'bytes':len(data),'sha256':hashlib.sha256(data).hexdigest()}


def main():
 native=ROOT/'native';packet=json.loads((native/'native.json').read_bytes());old=json.loads((ROOT.parent/'analysis-namespace/native/native.json').read_bytes());sources={r['object']['id']:r for r in old['original_sources']}
 assert len(sources)==packet['original_sources']==213 and packet['frontend_go_files']==65 and packet['vendor_files']==133 and not packet['root_or_binary_authority_claimed']
 raw=bytes.fromhex(packet['snapshot_hex']);wire=json.loads(raw);assert prior.canonical(wire)==raw and prior.native_digest(wire)==wire['snapshot_sha256']
 body={k:v for k,v in wire.items() if k!='content_sha256'};assert hashlib.sha256(b'MPK-PRIVATE-GO-SOURCE-SNAPSHOT-INPUTS-1.0\0'+prior.canonical(body)).hexdigest()==wire['content_sha256']
 assert [o['id'] for o in wire['objects']]==packet['source_ids'] and set(packet['source_ids'])==set(sources)
 for obj in wire['objects']:assert sources[obj['id']]['object']==obj
 build=packet['build_objects'];assert len(build)==201 and [o['path'] for o in build]==sorted(o['path'] for o in build) and len({o['path'] for o in build})==201
 for obj in build:
  assert sources[obj['id']]['object']==obj
  data=(ROOT.parent/'analysis-namespace/native/original-bytes'/sources[obj['id']]['raw_file']).read_bytes()
  assert len(data)==obj['bytes'] and hashlib.sha256(data).hexdigest()==obj['raw_sha256']
  ident={k:v for k,v in obj.items() if k!='id'};assert hashlib.sha256(b'MPK-GO-SOURCE-OBJECT-1.0\0'+prior.canonical(ident)).hexdigest()==obj['id']
 rust=json.loads((ROOT/'list-tests/build-list.json').read_bytes());assert rust['status']=='passed' and rust['build_objects']==build and rust['snapshot_sha256']==wire['snapshot_sha256'] and not rust['original_root_or_registered_binary_proved']
 captured=json.loads((native/'build/captured-inputs.json').read_bytes());builds=json.loads((native/'build/reproduced-builds.json').read_bytes());assert captured['build_inputs']==build and builds['source_inputs']==201 and builds['vendor_files']==133 and builds['frontend_files']==65
 first=(native/'build/frontend-0.elf').read_bytes();second=(native/'build/frontend-1.elf').read_bytes();assert first==second and builds['byte_exact_equal'] and builds['independent_empty_caches']
 digest=hashlib.sha256(first).hexdigest();assert all(row['binary_sha256']==digest and row['bytes']==len(first) for row in builds['builds'])
 registered=ROOT.parent/'analysis-build-info/registered-go2vir.elf';registered_hash=hashlib.sha256(registered.read_bytes()).hexdigest()
 for name in ['prelaunch','native','go-format-write','go-checks','list-check','list-tests','list-clippy','rust-format']:
  result=json.loads((ROOT/name/'result.json').read_bytes());assert result['status']=='passed' and result.get('consumed_inputs_unchanged',True)
 output=(ROOT/'list-tests/stdout.log').read_text();assert output.count('test result: ok. 1 passed;')==2
 result={'status':'passed','driver':seal(Path(__file__).resolve()),'native':seal(native/'native.json'),'rust':seal(ROOT/'list-tests/build-list.json'),'go_build_receipt':seal(native/'build/reproduced-builds.json'),'build_binary':seal(native/'build/frontend-0.elf'),'registered_frontend':seal(registered),'original_sources':213,'frontend_go_files':65,'vendor_files':133,'build_inputs':201,'all_build_source_ids_and_bytes_recomputed':True,'go_rust_and_actual_build_source_lists_equal':True,'independent_full_binaries_identical':True,'built_binary_equals_registered_frontend':digest==registered_hash,'registered_source_to_binary_authority_proved':False,'original_root_authenticated':False,'native_Linux_release_acceptance':False,'whole_repository_gate_run':False,'full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'}
 out=ROOT/'inspection.json';assert not out.exists();out.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:result[k] for k in ['status','original_sources','build_inputs','independent_full_binaries_identical','built_binary_equals_registered_frontend']}))


if __name__=='__main__':main()
