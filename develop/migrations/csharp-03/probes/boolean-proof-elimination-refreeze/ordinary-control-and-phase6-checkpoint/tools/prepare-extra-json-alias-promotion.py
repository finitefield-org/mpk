import copy,hashlib,json,pathlib
b=pathlib.Path(__file__).parent;r=pathlib.Path('/private/tmp/mpk-w09-refreeze-consumer-continuation');out=b/'extra-json-alias-promotion-plan';out.mkdir(exist_ok=False);files=out/'files';h=lambda data:hashlib.sha256(data).hexdigest();receipt=json.loads((b/'extra-json-alias-comparison.json').read_bytes());assert not receipt['failures'];plan={x['label']:x for x in json.loads((b/'extra-json-alias-plan.json').read_bytes())['targets']};metrics={x['label']:x for x in json.loads((b/'extra-json-alias-regeneration/generated/generation.json').read_bytes())['programs']};docs={};original={};certs={};rows=[]
def at(doc,pointer):
 for key in pointer.split('/')[1:]:
  key=key.replace('~1','/').replace('~0','~');doc=doc[int(key)] if isinstance(doc,list) else doc[key]
 return doc
def set_at(doc,pointer,value):
 if not pointer:return copy.deepcopy(value)
 keys=pointer.split('/')[1:];parent=at(doc,'/'+ '/'.join(keys[:-1])) if len(keys)>1 else doc;key=keys[-1].replace('~1','/').replace('~0','~');parent[int(key) if isinstance(parent,list) else key]=copy.deepcopy(value)
 return doc
for row in receipt['programs']:
 if not row['occurrences']:continue
 label=row['label'];meta=b/'extra-json-alias-regeneration/generated'/(label+'.program.json');fresh=json.loads(meta.read_bytes());cert=b/'extra-json-alias-regeneration/generated'/(label+'.hex');assert h(meta.read_bytes())==row['new_metadata_raw_sha256'] and h(cert.read_bytes())==row['new_certificate_raw_sha256'];target=plan[label]
 for occurrence in row['occurrences']:
  assert occurrence['semantic_metadata_preserved'];path=occurrence['file'];pointer=occurrence['pointer'];source=next(x for x in target['occurrences'] if x['file']==path and x['pointer']==pointer)
  if path not in docs:original[path]=(r/path).read_bytes();docs[path]=json.loads(original[path])
  assert at(docs[path],pointer)==source['original_program'],(label,path,pointer)
  docs[path]=set_at(docs[path],pointer,fresh)
  if pointer.endswith('/program'):
   parent_pointer=pointer.rsplit('/',1)[0];parent=at(docs[path],parent_pointer)
   for key in ['certificate_sha256','terms','declarations']:
    if key in parent:parent[key]=metrics[label][key]
  for path in occurrence['original_certificate_files']:
   old=(r/path).read_bytes();raw=bytes.fromhex(old.decode());new=bytes.fromhex(cert.read_text());payload=cert.read_bytes().rstrip()+old[len(old.rstrip()):]
   if path in certs:assert certs[path]['new']==payload,(label,path)
   else:certs[path]={'old':old,'new':payload,'label':label,'byte_preserved':raw==new}
 rows.append({'label':label,'occurrences':len(row['occurrences']),'certificate_bytes_changed':any(not x['certificate_bytes_preserved'] for x in row['occurrences'])})
changes=[]
for path,doc in docs.items():
 root_occ=[row for row in receipt['programs'] for o in row['occurrences'] if o['file']==path and not o['pointer']]
 if root_occ:
  assert len(root_occ)==1;payload=(b/'extra-json-alias-regeneration/generated'/(root_occ[0]['label']+'.program.json')).read_bytes()
 else:payload=json.dumps(doc,sort_keys=True,ensure_ascii=False,indent=2).encode()+original[path][len(original[path].rstrip()):]
 assert json.loads(payload)==doc
 if payload!=original[path]:
  dest=files/path;dest.parent.mkdir(parents=True,exist_ok=True);dest.write_bytes(payload);changes.append({'path':path,'kind':'metadata','before_raw_sha256':h(original[path]),'raw_sha256':h(payload)})
for path,entry in certs.items():
 if entry['new']==entry['old']:continue
 dest=files/path;dest.parent.mkdir(parents=True,exist_ok=True);dest.write_bytes(entry['new']);changes.append({'path':path,'kind':'certificate','before_raw_sha256':h(entry['old']),'raw_sha256':h(entry['new']),'label':entry['label'],'certificate_bytes_preserved':entry['byte_preserved']})
result={'status':'reviewable_plan_pending_complete_dual_acceptance_and_owner_pin_tests','selection_reason':'Promote actual standalone producer metadata at its 178 validated occurrences, excluding nested parsers handled by whole-wrapper producers. Changed certificate graphs already compare exactly after independently replayed attachment/name mappings; both checkers and owner pins remain required.','programs':rows,'program_occurrences':sum(x['occurrences'] for x in rows),'changes':changes,'application_proof_ids_pending':987,'full_t01_gate':'deferred_to_final_W10','full_t06_gate':'deferred_to_W12'};(out/'plan.json').write_text(json.dumps(result,sort_keys=True,indent=2)+'\n');print('programs',len(rows),'occurrences',result['program_occurrences'],'metadata',sum(x['kind']=='metadata' for x in changes),'certificates',sum(x['kind']=='certificate' for x in changes))
