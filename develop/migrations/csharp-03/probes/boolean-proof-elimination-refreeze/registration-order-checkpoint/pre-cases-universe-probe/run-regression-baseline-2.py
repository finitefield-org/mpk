import pathlib,json,hashlib,time,subprocess
b=pathlib.Path(__file__).parent;out=b/'regression-baseline-2';out.mkdir(exist_ok=False);h=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();bins={'rust':pathlib.Path('/private/tmp/mpk-w09-bool-cases-target/debug/mpk'),'go':pathlib.Path('/private/tmp/mpk-w09-bool-cases-reports/go-local-4/go-checker')};state={'status':'running','selection_reason':'Reproduce preceding-type/value/proof universe-argument acceptance and verify legacy, unused-term and other-family controls before the registration-order fix. Only the later-use case should already reject.','binary_sha256':{k:h(p) for k,p in bins.items()},'producer_sha256':{p.name:h(p) for p in [b/'regression-main.rs',b/'builder.rs']},'stages':[]}
try:
 for inp in sorted((b/'regression-inputs-2').glob('*.mpcert')):
  label=inp.stem;wanted='rejected' if label.startswith('later-') else 'accepted'
  for kind,exe in bins.items():
   name=f'{label}-{kind}';cmd=[str(exe),'check' if kind=='rust' else 'verify',str(inp)];start=time.monotonic()
   with (out/f'{name}.json').open('wb') as stdout,(out/f'{name}.stderr.txt').open('wb') as stderr:p=subprocess.run(cmd,stdout=stdout,stderr=stderr)
   report=json.loads((out/f'{name}.json').read_bytes());row=dict(stage=name,command=cmd,exit_code=p.returncode,verdict=report['verdict'],wanted_baseline_verdict=wanted,input_sha256=h(inp),report_sha256=h(out/f'{name}.json'),stderr_sha256=h(out/f'{name}.stderr.txt'),elapsed_seconds=time.monotonic()-start);state['stages'].append(row);(out/'status.json').write_text(json.dumps(state,indent=2,sort_keys=True)+'\n');print(name,p.returncode,report['verdict'],flush=True);assert report['verdict']==wanted and p.returncode==(1 if wanted=='rejected' else 0),row
   if wanted=='rejected' and kind=='rust':assert report['error_code']=='KERNEL_CORE_CHECK',report
 state['status']='reproduced_prior_use_gap_with_legacy_unused_and_other_family_controls'
except BaseException as error:state['status']='failed';state['error']=repr(error);raise
finally:(out/'status.json').write_text(json.dumps(state,indent=2,sort_keys=True)+'\n')
