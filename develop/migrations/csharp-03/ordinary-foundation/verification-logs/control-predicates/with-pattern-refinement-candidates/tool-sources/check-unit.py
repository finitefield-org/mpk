import json, subprocess, time
from pathlib import Path
from datetime import datetime, timezone
from hashlib import sha256
out=Path('/tmp/mpk-w09-pattern-candidates/unit-checks')
out.mkdir(parents=True,exist_ok=True)
source=Path('/tmp/mpk-w09-pattern-candidates/unit-certificates')
binaries={'go':Path('/tmp/mpk-w09-default-closed-proofs/go-checker'),'rust':Path('/Users/kazuyoshitoshiya/mpk/target/release/mpk')}
expected={'go':'d631f01eb0beaab7b83bebd53e9f66b02f3a6a8c04e173b3af0acb48b9b305a2','rust':'b5207abd1d92acc1fd8b92a995ed700e1a2d929e6254ffe20b6f9f5d868bac91'}
for backend, binary in binaries.items(): assert sha256(binary.read_bytes()).hexdigest()==expected[backend]
rows=[]
reports={}
for case in ('candidate','wrong-candidate'):
    data=(source/(case+'.mpcert')).read_bytes()
    cert_hash=sha256(b'MPK-MODULE-CERT-0.1\0'+data).hexdigest()
    for backend,binary in binaries.items():
        stem=case+'-'+backend
        cert=out/(stem+'.mpcert'); cert.write_bytes(data)
        command=[str(binary)]+(['check'] if backend=='rust' else [])+[str(cert)]
        start=time.monotonic(); started=datetime.now(timezone.utc).isoformat()
        result=subprocess.run(command,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
        (out/(stem+'.json')).write_bytes(result.stdout)
        (out/(stem+'.stderr')).write_bytes(result.stderr)
        report=json.loads(result.stdout)
        assert result.returncode==(0 if case=='candidate' else 1)
        assert report['verdict']==('accepted' if case=='candidate' else 'rejected')
        if case=='candidate':
            if backend=='go':
                core=report['report']; assert core['AxiomCount']==0
                hashes={key:bytes(core[field]).hex() for key,field in [('export','ExportHash'),('axiom_report','AxiomReportHash'),('certificate','CertificateHash')]}
            else:
                assert report['axiom_count']==0
                hashes=report['hashes']
            assert hashes['certificate']==cert_hash
            reports[backend]=hashes
        else:
            assert report.get('error_kind')=='core_check' if backend=='go' else report.get('error_code')=='KERNEL_CORE_CHECK'
            assert (report['certificate'] if backend=='go' else report['hashes']['certificate'])==cert_hash
        row=dict(case=case,backend=backend,command=command,started_at=started,finished_at=datetime.now(timezone.utc).isoformat(),elapsed_seconds=round(time.monotonic()-start,3),exit_code=result.returncode,certificate_sha256=cert_hash,input_file_sha256=sha256(data).hexdigest(),report_sha256=sha256(result.stdout).hexdigest(),stderr_sha256=sha256(result.stderr).hexdigest())
        rows.append(row)
        (out/'stages.json').write_text(json.dumps(rows,indent=2)+'\n')
        print(stem,result.returncode,flush=True)
assert reports['go']==reports['rust']
receipt=dict(status='passed',selection_reason='Check complete universal refinement fixtures at their exact named types with unchanged Go and Rust binaries. An incorrect proof can pass structural linkage but must reject at core checking; compare exact export, certificate and axiom-report hashes. Original source/application execution proofs remain pending.',binaries={k:dict(path=str(v),sha256=expected[k]) for k,v in binaries.items()},stages=rows,full_t_gate='deferred to T06-W12')
(out/'verification.json').write_text(json.dumps(receipt,indent=2)+'\n')
