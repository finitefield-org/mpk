import hashlib,json,pathlib
root=pathlib.Path('/private/tmp/mpk-w09-identity-proofs');out=root/'checks';s=json.loads((out/'status.json').read_bytes())
def digest(p):return hashlib.sha256(p.read_bytes()).hexdigest()
assert s['status']=='passed' and len(s['stages'])==6 and len(s['reused_unchanged_certificate_contexts'])==44
for stage in s['stages']:
 stem='float-make-commutation-'+stage['kind'];b=stage['backend'];assert stage['input_file_sha256']==digest(out/(stem+'.mpcert'));assert stage['report_sha256']==digest(out/(stem+'-'+b+'.json'));assert stage['stderr_sha256']==digest(out/(stem+'-'+b+'.stderr.txt'));assert stage['binary_sha256']==digest(pathlib.Path(stage['command'][0]))
 expected=0 if stage['kind']=='positive' else 1;assert stage['exit_code']==expected
for kind,n in [('positive',2),('hash',2),('wrong',2)]:assert sum(t['kind']==kind for t in s['stages'])==n
r=dict(status='passed',terminal_exec_session=39495,fresh_stage_count=6,fresh_accepted=2,hash_rejections=2,typed_core_rejections=2,reused_unchanged_contexts=44,prior_dual_receipt_sha256=s['prior_dual_receipt_sha256'],status_sha256=digest(out/'status.json'))
(out/'final-audit.json').write_text(json.dumps(r,indent=2,sort_keys=True)+'\n');print(json.dumps(r))
