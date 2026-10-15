import copy,hashlib,json,pathlib
b=pathlib.Path(__file__).parent;r=pathlib.Path('/private/tmp/mpk-w09-refreeze-consumer-continuation');out=b/'standalone-document-alias-plan';out.mkdir(exist_ok=False);files=out/'files';mapping=json.loads((b/'context-name-rebindings-9.json').read_bytes());plan={x['label']:x for x in json.loads((b/'program-alias-plan.json').read_bytes())['targets']};receipt=json.loads((b/'program-alias-comparison-4.json').read_bytes());assert not receipt['failures'];docs={};before={};aliases={};rows=[];h=lambda p:hashlib.sha256(p).hexdigest()
def at(d,ptr):
 for k in ptr.split('/')[1:]:d=d[int(k)] if isinstance(d,list) else d[k.replace('~1','/').replace('~0','~')]
 return d
for row in receipt['programs']:
 target=plan[row['label']]
 for o in row['occurrences']:
  if not o['pointer'].endswith('/program'):continue
  path=o['file'];ptr=o['pointer'].rsplit('/',1)[0]
  if path not in docs:before[path]=(r/path).read_bytes();docs[path]=json.loads(before[path])
  parent=at(docs[path],ptr)
  if not isinstance(parent.get('id'),str) or not parent['id'].startswith('document-'):continue
  old_id=parent['id'];old_hash=old_id.removeprefix('document-');new_hash=mapping.get(old_hash)
  if new_hash is None or old_hash==new_hash:continue
  assert target['source']['request_path'].endswith('boundary-output/source-requests.json'),target
  assert target['source']['id']==new_hash,(old_id,target['source']['id'],new_hash)
  new_id='document-'+new_hash;fresh_meta=b/'program-alias-regeneration/generated'/(row['label']+'.program.json');fresh_hex=b/'program-alias-regeneration/generated'/(row['label']+'.hex');assert parent['program']==json.loads(fresh_meta.read_bytes())
  old_cert=(r/pathlib.Path(path).parent/(old_id+'.hex'));assert old_cert.is_file(),old_cert
  assert bytes.fromhex(old_cert.read_text())==bytes.fromhex(fresh_hex.read_text()),old_cert
  dest=str(pathlib.Path(path).parent/(new_id+'.hex'));payload=old_cert.read_bytes()
  if dest in aliases:assert aliases[dest]==payload
  aliases[dest]=payload;parent['id']=new_id;rows.append({'file':path,'pointer':ptr+'/id','old_id':old_id,'new_id':new_id,'original_hex_path':str(old_cert.relative_to(r)),'new_hex_path':dest,'actual_program':row['label'],'byte_sha256':h(bytes.fromhex(payload.decode())),'certificate_bytes_preserved':True})
changes=[]
for path,doc in docs.items():
 if json.loads(before[path])==doc:continue
 payload=json.dumps(doc,sort_keys=True,ensure_ascii=False,indent=2).encode()+b'\n';dest=files/path;dest.parent.mkdir(parents=True,exist_ok=True);dest.write_bytes(payload);changes.append({'path':path,'kind':'metadata','before_raw_sha256':h(before[path]),'raw_sha256':h(payload)})
for path,payload in aliases.items():
 dest=files/path;dest.parent.mkdir(parents=True,exist_ok=True);dest.write_bytes(payload);changes.append({'path':path,'kind':'byte-identical-alias','before_raw_sha256':h((r/path).read_bytes()) if (r/path).exists() else None,'raw_sha256':h(payload)})
result={'status':'reviewable_actual_source_id_and_byte_identical_document_aliases','selection_reason':'A selected JSON product pin failed because enclosing document labels retained pre-freeze IDs while the exact regenerated source program uses the approved canonical document IDs. Require the independently replayed request-ID mapping, the exact producer metadata and certificate equality before updating only the enclosing document ID and adding its byte-identical hex alias. Original labels remain available.','changes':changes,'aliases':rows,'application_proof_ids_pending':987,'full_t01_gate':'deferred_to_final_W10','full_t06_gate':'deferred_to_W12'};(out/'plan.json').write_text(json.dumps(result,sort_keys=True,indent=2)+'\n');print('metadata',sum(x['kind']=='metadata' for x in changes),'aliases',len(aliases),'occurrences',len(rows))
