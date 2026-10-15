import collections,hashlib,json,pathlib
b=pathlib.Path(__file__).parent;old=[json.loads(l) for l in (b/'vir-hash-replay-old/stdout.jsonl').read_text().splitlines()];new=[json.loads(l) for l in (b/'vir-hash-replay-new/stdout.jsonl').read_text().splitlines()];assert len(old)==len(new)==385
for name in ['old','new']:
 s=json.loads((b/f'vir-hash-replay-{name}/status.json').read_bytes());assert s['status']=='passed' and s['exit_code']==0 and s['source_input_binary_bytes_unchanged']
assert old[0]['revision']==4 and new[0]['revision']==5 and old[0]['foundation_sha256']=='230c708601f4b89feeae28af23da10ccb11eec998d990b5133cf9336369e15a2' and new[0]['foundation_sha256']=='99369543ab96e97e971118fe980322fe0b138f253a31ab7f8a9b08565bad0844'
rows=[];mapping={};counts=collections.Counter()
for a,c in zip(old[1:],new[1:]):
 assert a['corpus']==c['corpus'] and a['index']==c['index'] and a['result']==c['result'],(a['corpus'],a['index'])
 counts[a['result']]+=1
 if a['result']!='emitted':continue
 x=a['source_ir_sha256'];y=c['source_ir_sha256']
 if x in mapping:assert mapping[x]==y
 mapping[x]=y;rows.append(dict(corpus=a['corpus'],index=a['index'],old_id=a['id'],new_id=c['id'],old_source_ir_sha256=x,new_source_ir_sha256=y))
remaining=json.loads((b/'remaining-source-program-inventory.json').read_bytes());matched=[x for x in remaining if x['source_ir_sha256'] in mapping];unmatched=[x for x in remaining if x['source_ir_sha256'] not in mapping];receipt={'schema':'mpk.csharp_practical.t01_w09.actual_vir_identity_replay.v1','status':'passed_actual_predecessor_and_current_384_source_replays','rows':384,'outcomes':dict(counts),'distinct_emitted_vir_hash_rebindings':len(mapping),'remaining_metadata_program_occurrences_matched':len(matched),'remaining_metadata_program_occurrences_unmatched':len(unmatched),'identities':rows,'original_application_proof_ids_pending':987,'full_t01_gate':'deferred_to_final_W10','full_t06_gate':'deferred_to_W12'};(b/'actual-vir-hash-rebindings.json').write_text(json.dumps(mapping,sort_keys=True,indent=2)+'\n');(b/'vir-hash-replay-audit.json').write_text(json.dumps(receipt,sort_keys=True,indent=2)+'\n');(b/'remaining-programs-missing-vir-map.json').write_text(json.dumps(unmatched,sort_keys=True,indent=2)+'\n');print(json.dumps({k:v for k,v in receipt.items() if k!='identities'}))
