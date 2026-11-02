"""Bind the real handoff, complete native originals and prior checked proof bytes."""
from pathlib import Path
import base64,datetime,hashlib,json,runpy
ROOT=Path(__file__).resolve().parent;PRIOR=ROOT.parent/'source-invocations';OUT=ROOT/'original-comparison';OUT.mkdir(exist_ok=False)
def seal(p):
 b=Path(p).read_bytes();return {'path':str(p),'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest()}
checks=['native','native-replacement-corrected','derived','rust-controls','lint','go-vet','final-format','final-go-format'];runs={n:json.loads((ROOT/n/'result.json').read_bytes()) for n in checks};assert all(r['status']=='passed' for r in runs.values())
old=json.loads((PRIOR/'aggregate-paired/result.json').read_bytes());assert old['status']=='passed' and old['complete_original_call_interfaces']==32
ledger=json.loads((ROOT/'native-original-relocations.json').read_bytes());assert ledger['status']=='passed'
inputs=[Path(__file__),ROOT/'read-originals.py',PRIOR/'read-originals.py',ROOT/'native-original-relocations.json',ROOT/'corpus.json',ROOT/'derived/prelaunch-inputs.json',PRIOR/'corpus.json',PRIOR/'aggregate-paired/result.json',ROOT/'rust/result.json',ROOT/'rust/stdout.log',ROOT/'rust/observations/original-proof-replay.json',ROOT/'native-replacement-original/result.json',ROOT/'native-replacement-original/stdout.log']+[ROOT/n/'result.json' for n in checks]
for folder in [ROOT/r['archive'] for r in ledger['groups']]+[PRIOR/'native-output-original']:
 inputs.append(folder/'manifest.json');manifest=json.loads((folder/'manifest.json').read_bytes());inputs.extend(Path(c['encoded']['path']) for c in manifest['chunks'])
replay=json.loads((ROOT/'rust/observations/original-proof-replay.json').read_bytes());inputs.extend(Path(r['original_certificate']) for r in replay['complete_original_call_proofs']);rows=[]
for i,p in enumerate(inputs):
 target=OUT/'inputs'/f'{i:03d}.original';target.parent.mkdir(exist_ok=True);target.write_bytes(p.read_bytes());target.chmod(p.stat().st_mode&0o777);rows.append({**seal(p),'mode':p.stat().st_mode&0o777,'complete_prelaunch_snapshot':seal(target)})
receipt=OUT/'prelaunch-inputs.json';receipt.write_text(json.dumps(rows,indent=2)+'\n');(OUT/'driver.py').write_bytes(Path(__file__).read_bytes());record={'status':'running','actual_driver_prelaunch':seal(OUT/'driver.py'),'complete_prelaunch_inputs':seal(receipt),'started_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat()}
def save():(OUT/'result.json').write_text(json.dumps(record,indent=2)+'\n')
save()
try:
 read=runpy.run_path(str(ROOT/'read-originals.py'))['read'];_,native=read('native-complete-original');_,replacement=read('replacement-complete-original');_,failed=read('replacement-initial-original');_,prior=runpy.run_path(str(PRIOR/'read-originals.py'))['read']('native-output-original');corpus=json.loads((ROOT/'corpus.json').read_bytes());original=json.loads((PRIOR/'corpus.json').read_bytes())
 def packet(files,prefix,expected):
  raw_packet=json.loads(files[prefix+'unit-inputs.json']);raw=base64.b64decode(raw_packet['canonical_components'],validate=True);component=json.loads(raw)
  actual={'components_hex':raw.hex(),'sources':[{'object':s['object'],'raw_hex':base64.b64decode(s['raw'],validate=True).hex()} for s in raw_packet['sources']]};assert actual==expected['input']
  provenance=json.loads(files[prefix+'source-input-provenance.json']);assert provenance==expected['private_owner_provenance'] and provenance['registered_context_or_program_proof'] is False
  preimage={'image_sha256':provenance['image_sha256'],'compiler_sha256':provenance['compiler_sha256'],'main_source_object_id':provenance['main_source_object_id'],'requested_package':provenance['requested_package'],'source_components_sha256':component['content_sha256']}
  canonical=json.dumps(preimage,sort_keys=True,separators=(',',':')).encode();assert hashlib.sha256(b'MPK-PRIVATE-GO-SOURCE-INPUTS-1.0\0'+canonical).hexdigest()==provenance['source_inputs_sha256']
  assert len(component['types'])==1 and component['types'][0]['name']=='bool' and not component['declarations'] and not component['underlying'] and not component['aliases']
  assert component['package_roots']==[p['id'] for p in component['packages']]
  assert {s['id'] for s in component['source_objects']}=={s for p in component['packages'] for s in [p['module_source_object_id'],*p['source_objects']]}
  return component
 units=[]
 for name in ['pure','wide']:
  expected=next(u for u in corpus['units'] if u['name']==name);c=packet(native,name+'/',expected);assert expected['input']==next(u for u in original['units'] if u['name']==name)['input']
  key=lambda row:(row['source'],row['start'],row['end'])
  actual=json.loads(native[name+'/native-calls.json']);before=json.loads(prior[name+'/native-calls.json']);assert sorted(actual,key=key)==sorted(before,key=key)
  units.append({'unit':name,'complete_packages':len(c['packages']),'original_calls':len(actual),'all_native_caller_group_signature_and_target_facts_unchanged':True,'all_source_bytes_unchanged':True})
 c=packet(replacement,'',corpus['replacement']);assert len(c['packages'])==1;meta=json.loads(replacement['replacement-root.json']);assert meta['main_source_object_id']!=meta['requested_module_source_object_id'] and meta['requested_module_source_object_id']==c['packages'][0]['module_source_object_id']
 exports=json.loads(failed['storage-original-exports.json']);assert exports['error']=='<nil>' and json.loads((ROOT/'native-replacement-original/result.json').read_bytes())['status']=='failed'
 failed_output=(ROOT/'native-replacement-original/stdout.log').read_text();assert 'source inputs crossed the original selected main module' in failed_output
 stdout=(ROOT/'rust/stdout.log').read_text();assert '3 passed; 1 failed' in stdout
 for name in ['source_inputs_keep_a_selected_original_replacement_module','source_inputs_owned_handoff_preserves_every_original_call_proof','source_inputs_reject_missing_or_changed_original_bytes']:assert 'test '+name+' ... ok' in stdout
 assert replay['status']=='passed' and replay['kernel_verification_repeated'] is False and len(replay['complete_original_call_proofs'])==32
 for proof in replay['complete_original_call_proofs']:
  raw=Path(proof['original_certificate']).read_bytes();assert len(raw)==proof['bytes'] and hashlib.sha256(raw).hexdigest()==proof['raw_sha256'] and proof['whole_original_proof_bytes_equal']
 for row in rows:assert seal(row['path'])=={k:row[k] for k in ['path','bytes','sha256']} and Path(row['path']).stat().st_mode&0o777==row['mode']
 record.update(status='passed',whole_inputs_unchanged=True,units=units,whole_prior_proof_streams_replayed=32,source_type_counts=sorted({p['source_types'] for p in replay['complete_original_call_proofs']}),full_error_case_counts=sorted({p['error_cases'] for p in replay['complete_original_call_proofs']}),replacement_requested_module_differs_from_original_main=True,failed_owner_restriction_real_original_inputs_retained=True,all_four_Rust_roots_verified_across_affected_epochs=True,prior_full_named_union_kernel_evidence=seal(PRIOR/'aggregate-paired/result.json'),native_type_facts_used_as_constructor_inputs=False,kernel_matrix_repeated=False,source_program_context_or_native_Linux_proof_claimed=False,finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat());save();print(json.dumps({'status':'passed','whole_prior_proof_streams_replayed':32,'complete_native_calls':sum(u['original_calls'] for u in units),'replacement_module_retained':True}))
except BaseException:
 record['status']='failed';save();raise
