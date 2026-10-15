def main():
    from pathlib import Path
    from hashlib import sha256
    import concurrent.futures,json,subprocess,time
    import os
    from datetime import datetime,timezone
    root=Path('/private/tmp/mpk-w09-foundation-proofs')
    source=root/'local/certificates'; out=root/'checks';out.mkdir(exist_ok=True)
    repo=Path('/private/tmp/mpk-w09-packed-pattern-proofs')
    manifest=json.loads((root/'source-manifest.json').read_bytes())
    for group in ('source_hashes','fixture_hashes'):
     for n,h in manifest[group].items(): assert sha256((repo/n).read_bytes()).hexdigest()==h,n
    prior=repo/'develop/migrations/csharp-03/ordinary-foundation/verification-logs/construction-storage-types/source-manifest.json'
    assert manifest['fixture_hashes']==json.loads(prior.read_bytes())['fixture_hashes']
    old=repo/'develop/migrations/csharp-03/ordinary-foundation/verification-logs/concrete-allocation-proofs/attempt-5/certificates'
    def catalog(p): return {x.name:sha256(x.read_bytes()).hexdigest() for x in p.iterdir() if x.is_file()}
    assert catalog(root/'local/operation-certificates')==catalog(old)
    bins={'go':Path('/private/tmp/mpk-w09-scoped-construction/go-checker'),'rust':Path('/Users/kazuyoshitoshiya/mpk/target/release/mpk')}
    binary_hashes={k:sha256(p.read_bytes()).hexdigest() for k,p in bins.items()}
    assert binary_hashes['rust']=='b5207abd1d92acc1fd8b92a995ed700e1a2d929e6254ffe20b6f9f5d868bac91'
    positive=sorted(p for p in source.glob('*.hex') if not p.stem.endswith('-wrong'))
    assert len(positive)==45
    aliases={}
    unique={}
    for p in positive:
     data=bytes.fromhex(p.read_text()); h=sha256(data).hexdigest()
     aliases.setdefault(h,[]).append(p.stem)
     unique.setdefault(h,p)
    positive=list(unique.values())
    roots=[json.loads(p.read_bytes()) for p in source.glob('*.json')]
    assert len(roots)==45
    assert sum(len(m['types']['proofs']) for m in roots)==87
    assert sum(len(m['operations']['proofs']) for m in roots)==442
    assert sum(len(m['supplied_binding_sequent_ids']) for m in roots)==529
    assert sum(len(m['remaining_binding_sequent_ids']) for m in roots)==458
    assert sum(len(m['pending_proof_ids']) for m in roots)==987
    state=dict(status='running',supervisor_pid=os.getpid(),started_at=datetime.now(timezone.utc).isoformat(),stages=[],binaries=binary_hashes,source_contexts=45,
     supplied_type_proofs=87,supplied_operation_proofs=442,supplied_binding_sequents=529,
     remaining_binding_sequents=458,application_proofs_pending=987,generic_operations_pending=25,
     source_certificate_aliases=aliases,distinct_positive_certificates=len(positive),
     full_t_gate='deferred to T06-W12')
    def persist():
     p=out/'status.tmp';p.write_text(json.dumps(state,indent=2,sort_keys=True)+'\n');p.replace(out/'status.json')
    def run(b,stem):
     p=out/(stem+'.mpcert'); command=[str(bins[b]),'verify' if b=='go' else 'check',str(p)]
     start=time.monotonic()
     with (out/(stem+'-'+b+'.json')).open('wb') as o,(out/(stem+'-'+b+'.stderr.txt')).open('wb') as e: proc=subprocess.run(command,stdout=o,stderr=e)
     report=json.loads((out/(stem+'-'+b+'.json')).read_bytes())
     return b,proc.returncode,report,round(time.monotonic()-start,3),command
    persist()
    for p in positive:
     data=bytes.fromhex(p.read_text()); jobs=[('positive',data),('hash',data[:-1]+bytes([data[-1]^1]))]
     wrongs=[source/(name+'-wrong.hex') for name in aliases[sha256(data).hexdigest()]]
     wrongs=[wrong for wrong in wrongs if wrong.exists()]
     assert len(wrongs)<=1
     if wrongs: jobs.append(('wrong',bytes.fromhex(wrongs[0].read_text())))
     for kind,data in jobs:
      stem=p.stem+'-'+kind; (out/(stem+'.mpcert')).write_bytes(data)
      state['stage']=stem;persist(); accepted={}
      with concurrent.futures.ThreadPoolExecutor(max_workers=2) as pool: results=list(pool.map(lambda b:run(b,stem),bins))
      h=sha256(b'MPK-MODULE-CERT-0.1\0'+data).hexdigest()
      for b,code,r,elapsed,command in results:
       assert code==(0 if kind=='positive' else 1),(stem,b,code)
       assert r['verdict']==('accepted' if kind=='positive' else 'rejected')
       if kind=='positive':
        assert r.get('axiom_count',0)==0 and r['hashes']['certificate']==h
        if b=='rust': assert all(v==0 for v in r['axiom_report']['summary'].values())
        accepted[b]=(r['module'],r['declaration_count'],r.get('axiom_count',0),r['hashes'])
       else:
        assert r['error_kind' if b=='go' else 'error_code']==({'hash':'hash_mismatch','wrong':'core_check'}[kind] if b=='go' else {'hash':'KERNEL_HASH_MISMATCH','wrong':'KERNEL_CORE_CHECK'}[kind])
       state['stages'].append(dict(backend=b,case=p.stem,kind=kind,command=command,exit_code=code,elapsed_seconds=elapsed,input_file_sha256=sha256(data).hexdigest(),certificate_sha256=h,binary_sha256=binary_hashes[b],report_sha256=sha256((out/(stem+'-'+b+'.json')).read_bytes()).hexdigest(),stderr_sha256=sha256((out/(stem+'-'+b+'.stderr.txt')).read_bytes()).hexdigest()))
      if kind=='positive': assert accepted['go']==accepted['rust'],stem
      persist();print(stem,'passed both checkers',flush=True)
    assert len(state['stages'])==len(positive)*4+2
    state.update(status='passed',finished_at=datetime.now(timezone.utc).isoformat(),stage_count=len(state['stages']),positive_stages=len(positive)*2,hash_rejections=len(positive)*2,typed_core_rejections=2,legacy_operation_exported_files_unchanged=92)
    persist();(out/'verification.json').write_bytes((out/'status.json').read_bytes())

if __name__ == '__main__':
    try:
        main()
    except BaseException as error:
        from pathlib import Path
        from datetime import datetime, timezone
        import json
        out = Path('/private/tmp/mpk-w09-foundation-proofs/checks')
        out.mkdir(exist_ok=True)
        path = out/'status.json'
        state = json.loads(path.read_bytes()) if path.exists() else {}
        state.update(status='failed', error=repr(error), finished_at=datetime.now(timezone.utc).isoformat())
        temporary = out/'status.tmp'
        temporary.write_text(json.dumps(state, indent=2, sort_keys=True)+'\n')
        temporary.replace(path)
        raise
