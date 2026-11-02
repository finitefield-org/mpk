"""Independent inspection of whole native inputs, content hashes and replay."""
from pathlib import Path
import base64,hashlib,io,json,re,tarfile,tomllib,zipfile,struct,zlib
ROOT=Path(__file__).resolve().parent
INPUTS={}
def seal(p):
 b=p.read_bytes();return {'path':str(p),'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest(),'mode':p.stat().st_mode&0o777}
def raw(p):INPUTS[str(p)]=seal(p);return p.read_bytes()
def read(p):return json.loads(raw(p))
def check_object(row,data):assert len(data)==row['bytes'] and hashlib.sha256(data).hexdigest()==row['raw_sha256']
def main():
 out=ROOT/'inspection.json';assert not out.exists()
 verification=read(ROOT/'verification.json')
 epochs={p.parent.name:read(p) for p in ROOT.glob('*/result.json')}
 for name in verification['passing_check_epochs']:assert epochs[name]['status']=='passed'
 failed=[name for name,r in epochs.items() if r['status']=='failed']
 assert sorted(failed)==['build-original','rust-original']
 assert 'E0689' in raw(ROOT/'build-original/stderr.log').decode() and 'expanded.checked_add' in raw(ROOT/'build-original/stderr.log').decode()
 for r in epochs.values():
  assert r['external_originals_unchanged']
  assert r['consumed_inputs_unchanged'] or r['expected_formatter_mutation'] or r.get('expected_lock_resolution_mutation')
 deps=read(ROOT/'dependency-sources/manifest.json');lock=tomllib.loads(raw(Path.cwd()/'Cargo.lock').decode())
 for package in deps['packages']:
  archive=package['archive'];data=raw(Path(archive['retained_path']));assert len(data)==archive['bytes'] and hashlib.sha256(data).hexdigest()==archive['sha256']
  assert next(r for r in lock['package'] if r['name']==package['name'] and r['version']==package['version'])['checksum']==archive['sha256']
  with tarfile.open(fileobj=io.BytesIO(data),mode='r:gz') as packed:
   members={m.name.removeprefix(package['name']+'-'+package['version']+'/'):packed.extractfile(m).read() for m in packed.getmembers() if m.isfile()}
  for source in package['sources']:
   data=raw(Path(source['retained_path']));assert len(data)==source['bytes'] and hashlib.sha256(data).hexdigest()==source['sha256']
   name=str(Path(source['retained_path']).relative_to(ROOT/'dependency-sources'/(package['name']+'-'+package['version'])))
   if name in members:assert members[name]==data
   else:assert name in ['.cargo-ok','.cargo-checksum.json']
 native=ROOT/'native/native-originals/module-data-oracle'
 original_sums=read(native/'original-module-sums.json')['cases']
 rust_sums=read(ROOT/'rust-original/observations/data/all-original-sum-observations.json')['cases']
 assert len(original_sums)==1291 and len(original_sums)==len(rust_sums)
 for original,observed in zip(original_sums,rust_sums):
  data=base64.b64decode(original['raw'],validate=True);check_object(original['object'],data)
  assert original['case']==observed['case'] and original['accepted']==observed['accepted']
 archives=read(native/'original-module-archives.json')['cases']+read(ROOT/'native-compat/native-originals/module-data-compat/original-zip-compatibility.json')['cases']
 rust_archives=read(ROOT/'rust-original/observations/data/all-original-archive-observations.json')['cases'];assert len(archives)==len(rust_archives)==39
 expanded=0
 for original,observed in zip(archives,rust_archives):
  data=base64.b64decode(original['raw'],validate=True);check_object(original['object'],data)
  assert original['case']==observed['case'] and original['accepted']==observed['accepted']
  if not original['accepted']:continue
  # Independently hash original member contents with Python's ZIP directory
  # reader and raw DEFLATE decoder. Apply the native signed-size/directory
  # content view; Python's separate CRC policy is not the Go Hash1 definition.
  with zipfile.ZipFile(io.BytesIO(data)) as archive:
   last={i.filename:i for i in archive.infolist()};summary=hashlib.sha256()
   for name in sorted(i.filename for i in archive.infolist()):
    digest=hashlib.sha256()
    info=last[name]
    if not name.endswith('/'):
     name_len,extra_len=struct.unpack_from('<HH',data,info.header_offset+26)
     start=info.header_offset+30+name_len+extra_len
     size=0 if info.compress_size>=1<<63 else info.compress_size
     compressed=data[start:start+size]
     if info.compress_type==0:digest.update(compressed);expanded+=len(compressed)
     else:
      assert info.compress_type==8
      decoder=zlib.decompressobj(-15);rest=compressed
      while True:
       chunk=decoder.decompress(rest,65536);digest.update(chunk);expanded+=len(chunk)
       rest=decoder.unconsumed_tail
       if decoder.eof:break
       assert chunk or rest
    summary.update((digest.hexdigest()+'  '+name+'\n').encode())
   assert 'h1:'+base64.b64encode(summary.digest()).decode()==original['sum']
 by_case={r['case']:r for r in archives}
 assert by_case['full-expanded-500MiB']['accepted'] and by_case['full-expanded-500MiB']['members'][0]['Bytes']==500<<20
 assert by_case['full-namespace-131072']['accepted'] and by_case['full-namespace-131072']['namespace_entries']==131072 and not by_case['namespace-one-short']['accepted']
 fold=read(native/'original-simple-fold.json');assert fold['unicode_version']=='15.0.0' and len(fold['pairs'])==1454
 generation=read(ROOT/'fold-generation.json');assert hashlib.sha256(fold['rust_source'].encode()).hexdigest()==generation['generated_source']['sha256']
 generated=Path.cwd()/'crates/mpk-vc/src/go_practical_type_keys/module_observations/data/archive/fold.rs'
 pairs=[list(map(int,p)) for p in re.findall(r'\((\d+),\s*(\d+)\)',raw(generated).decode())];assert pairs==fold['pairs']
 assert '1 passed; 0 failed' in raw(ROOT/'fold-original/stdout.log').decode()
 owners=read(ROOT/'rust-original/observations/data/whole-original-offline-module-data.json')['owners'];assert len(owners)==2
 for owner in owners:
  directory=ROOT.parent/'module-observations/native/native-originals'/owner['owner'];packet=read(directory/'complete-original-module-observations.json')
  assert owner['source_objects']==len(packet['snapshot_sources']) and owner['original_selected_Root_contracts']==1
  for source in packet['snapshot_sources']:check_object(source['object'],raw(directory/'original-source-objects'/source['raw_file']))
 controls=read(ROOT/'controls-remaining/observations/data/coherent-original-module-data-controls.json')['cases'];assert len(controls)==11 and sum(r['accepted'] for r in controls)==3
 analysis=read(ROOT/'failed-control-analysis.json');assert analysis['cases'][0]['decoded_equal_original'] and not analysis['cases'][1]['decoded_equal_original']
 by_control={r['case']:r for r in controls}
 assert by_control['archive-encoding-variant']['complete_changed_raw_hex']==analysis['cases'][0]['whole_changed_archive_hex']
 assert by_control['archive-body']['complete_changed_raw_hex']==analysis['cases'][1]['whole_changed_archive_hex']
 assert 'archive-body None' in raw(ROOT/'rust-original/stderr.log').decode()
 original=read(ROOT.parent/'module-observations/native/native-originals/versioned-owned/complete-original-module-observations.json');capture=json.loads(bytes.fromhex(original['capture_hex']))
 for row in controls:
  data=bytes.fromhex(row['complete_changed_raw_hex']);check_object(row['changed_object'],data)
  changed_capture=json.loads(bytes.fromhex(row['changed_capture_hex']));changed_export=json.loads(bytes.fromhex(row['changed_export_hex']));changed_module=json.loads(bytes.fromhex(row['changed_module_hex']))
  assert row['same_original_source_selection_and_Root_contract'] and changed_capture['selection_inputs_sha256']==capture['selection_inputs_sha256']
  assert changed_export['source_capture_inputs_sha256']==changed_capture['content_sha256'] and changed_module['source_export_inputs_sha256']==changed_export['content_sha256']
 proofs=read(ROOT/'controls-remaining/observations/sources/original-proof-replay.json')['complete_original_call_proofs'];assert len(proofs)==32
 for row in proofs:
  data=raw(Path(row['original_certificate']));assert len(data)==row['bytes'] and hashlib.sha256(data).hexdigest()==row['raw_sha256']
 units=read(ROOT/'controls-remaining/observations/snapshots/all-original-capture-units.json')['units'];assert len(units)==2 and all(u['original_roots']==43 and u['all_source_components_and_bytes_equal'] for u in units)
 result={'status':'passed','driver':seal(Path(__file__).resolve()),'inputs':list(INPUTS.values()),'source_epochs':len(epochs),'preserved_failed_epochs':failed,'original_sum_cases':len(original_sums),'original_archive_cases':len(archives),'independently_streamed_positive_archive_bytes':expanded,'complete_fold_pairs':len(pairs),'unicode_scalar_domain':1112064,'original_module_owners':owners,'coherent_data_controls':len(controls),'positive_resealed_controls':3,'original_proof_streams':32,'original_roots_per_unit':43,'original_error_universes':sorted({p['error_cases'] for p in proofs}),'full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'}
 out.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:result[k] for k in ['status','source_epochs','original_sum_cases','original_archive_cases','coherent_data_controls']}))
if __name__=='__main__':main()
