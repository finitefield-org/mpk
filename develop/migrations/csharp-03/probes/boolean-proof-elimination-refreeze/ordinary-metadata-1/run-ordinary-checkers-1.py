import hashlib,json,os,pathlib,subprocess,time
base=pathlib.Path(__file__).parent
repo=pathlib.Path('/private/tmp/mpk-w09-context-application-integration')
out=base/'ordinary-checkers-1'
plan=json.loads((base/'ordinary-checker-plan-1-final.json').read_bytes())
h=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
state={'status':'running','selection_reason':plan['selection_reason'],'binaries':plan['binaries'],'rust_checker_build':plan['rust_checker_build'],'stages':[],'full_t01_gate':plan['full_t01_gate'],'full_t06_gate':plan['full_t06_gate'],'original_application_proof_ids_pending':987,'supervisor_pid':os.getpid()}
def save(): (out/'status.json').write_text(json.dumps(state,sort_keys=True,indent=2)+'\n')
def verify():
    for p,sha in plan['source_hashes'].items():assert h(repo/p)==sha,p
    for name,row in plan['binaries'].items():assert h(pathlib.Path(row['path']))==row['sha256'],name
save()
try:
    verify()
    for row in plan['cases']:
        name=pathlib.Path(row['generated_path']).stem
        original=base/'ordinary-regeneration-1/goldens'/row['family']/row['generated_path']
        assert h(original)==row['generated_raw_sha256']
        data=bytes.fromhex(original.read_text());input_path=out/(name+'.mpcert');input_path.write_bytes(data)
        matches=[]
        for kind,binary in plan['binaries'].items():
            label=name+'-'+kind;state['stage']=label;t=time.monotonic()
            with (out/(label+'.json')).open('wb') as stdout,(out/(label+'.stderr.txt')).open('wb') as stderr:
                p=subprocess.Popen([binary['path'],'check' if kind=='rust' else 'verify',str(input_path)],stdout=stdout,stderr=stderr)
                state['process_pid']=p.pid;save();code=p.wait()
            result=json.loads((out/(label+'.json')).read_bytes())
            stage={'stage':label,'case':name,'backend':kind,'exit_code':code,'verdict':result['verdict'],'elapsed_seconds':time.monotonic()-t,'input_sha256':h(input_path),'report_sha256':h(out/(label+'.json')),'stderr_sha256':h(out/(label+'.stderr.txt'))}
            state['stages'].append(stage);save();assert code==0 and result['verdict']=='accepted',stage
            assert result['hashes']['certificate']==hashlib.sha256(b'MPK-MODULE-CERT-0.1\0'+data).hexdigest()
            if kind=='rust':assert all(x==0 for x in result['axiom_report']['summary'].values())
            assert result.get('axiom_count',0)==0
            matches.append({k:result[k] for k in ['module','hashes']});verify()
        assert matches[0]==matches[1],name
        print(name+': accepted by both with matching hashes',flush=True)
    state['status']='passed_all_15_changed_definition_certificates_zero_axioms_both_checkers'
except BaseException as e:
    state['status']='failed';state['error']=repr(e);raise
finally:save()
