#!/usr/bin/env python3
"""Measure selected existing proof recipes with one direct child at a time."""
import argparse
import json
from pathlib import Path
import platform
import tempfile
from run_proof_pilot import ROOT,HERE,TARGET,sha,json_bytes,timed,observe,semantic_expectation

RECORD=ROOT/'develop/migrations/go-practical/t01-w03/sequential-resources'
BASE=ROOT/'develop/migrations/go-practical/t01-w03'
SCOPE='representative upper/boundary recipe measurements with one direct child at a time in this runner; other host processes uncontrolled; not a guarantee for every source program or a frozen profile budget'
SELECTION=(
 ('halting-returns','returned-128-success-sum-valid','base','returned-generate',('128',)),
 ('halting-returns','returned-129-success-sum-valid','base','returned-generate',('129',)),
 ('reslicing','reslice-128-three-read-valid','base','reslice-generate',('128',)),
 ('bulk-copy','copy-128-snapshot-valid','base','copy-generate',('128',)),
 ('construction','construct-128-composed-borrowed-frame-valid','base','construct-generate',('128',)),
 ('construction','construct-129-composed-borrowed-frame-valid','base','construct-generate',('129',)),
 ('stringop','stringop-256-concat-right-read-valid','base','stringop-generate',('256',)),
 ('stringop','stringop-257-substring-read-valid','base','stringop-generate',('257',)),
 ('affix','affix-256-suffix-matched-read-valid','affix','affix-generate',('256',)),
 ('affix','affix-257-suffix-matched-read-valid','affix','affix-generate',('257',)),
 ('aggregate','aggregate-256-128-suffix-read-valid','aggregate','aggregate-generate',('256','128')),
 ('aggregate','aggregate-257-129-suffix-read-valid','aggregate','aggregate-generate',('257','129')),
 ('control-induction','control-invariant-entry-valid','control','control-generate',()),
 ('control-induction','control-absorbing-stop-valid','control','control-generate',()),
)
def files():
 reports={};inputs={};refs=[]
 for group,case,*_ in SELECTION:
  if group not in reports:
   path=BASE/group/'observations.json';r=json.loads(path.read_text());reports[group]=r;refs.append({'path':path.relative_to(ROOT).as_posix(),'sha256':sha(path.read_bytes())})
   for row in r['inputs']:
    if row['path'] in inputs and inputs[row['path']]!=row['sha256']:raise ValueError('resource prerequisite source crossing')
    inputs[row['path']]=row['sha256']
  r=reports[group];c=next(c for c in r['cases'] if c['case']['id']==case)
  if c['case']['expected']!='accepted' or c['classification']!='accepted_zero_axiom':raise ValueError('resource selection is not a checked positive recipe')
  path=BASE/group/(case+'.mpcert');refs.append({'path':path.relative_to(ROOT).as_posix(),'sha256':sha(path.read_bytes())})
 for path in (Path(__file__),HERE/'check_resource_envelope.py',BASE/'control-induction/local-resource-host.json'):
  inputs[path.relative_to(ROOT).as_posix()]=sha(path.read_bytes())
 if any(sha((ROOT/p).read_bytes())!=h for p,h in inputs.items()):raise ValueError('stale resource prerequisite source')
 return reports,[{'path':p,'sha256':h} for p,h in sorted(inputs.items())],sorted(refs,key=lambda r:r['path'])
def binary_paths():
 return {
  'base':TARGET/'release/go-practical-proof-probe',
  'affix':TARGET/'release/go-practical-affix-probe',
  'aggregate':TARGET/'release/go-practical-aggregate-probe',
  'control':Path('/private/tmp/mpk-go-practical-control-target/release/go-practical-control-probe'),
  'go':Path('/private/tmp/mpk-go-practical-checker-no-vcs'),
  'private_go':Path('/private/tmp/mpk-go-control-foundation-7/checker'),
 }
def measure():
 reports,inputs,refs=files();paths=binary_paths();pins={k:sha(v.read_bytes()) for k,v in paths.items()};rows=[]
 for group,identity,kind,command,args in SELECTION:
  r=reports[group]
  if r['checker_binaries']!={'rust':pins[kind],'go':pins['private_go' if kind=='control' else 'go']}:raise ValueError('resource binary differs from original measurement')
  original=next(c for c in r['cases'] if c['case']['id']==identity);c=original['case']
  with tempfile.TemporaryDirectory(prefix='mpk-sequential-recipe-',dir='/private/tmp') as temp:
   directory=Path(temp);construction=timed([paths[kind],command,ROOT,directory,*args]);path=directory/(identity+'.mpcert')
   if construction['exit_code'] or path.read_bytes()!=(BASE/group/path.name).read_bytes():raise ValueError('sequential generation changed the canonical candidate')
   obs={k:observe(exe,path) for k,exe in [('rust',paths[kind]),('go',paths['private_go' if kind=='control' else 'go'])]}
   semantic_expectation(c,obs,declarations=c['declarations'],module=original['observations']['rust']['report']['module'])
   if any(obs[k]['report']['hashes']!=original['observations'][k]['report']['hashes'] for k in obs):raise ValueError('sequential accepted hashes changed')
   row={'group':group,'id':identity,'checker_scope':'private proposed Nat.induct extension' if kind=='control' else 'unchanged production checker implementations','case':c,'raw_sha256':sha(path.read_bytes()),'construction_scope':'complete candidate group including shared ordinary library; not an isolated single-theorem producer','construction':construction,'observations':obs};rows.append(row)
   print(f"{identity}: construction {construction['wall_seconds']:.3f}s; Rust {obs['rust']['wall_seconds']:.3f}s, Go {obs['go']['wall_seconds']:.3f}s",flush=True)
  RECORD.mkdir(exist_ok=True);(RECORD/'attempt.json').write_bytes(json_bytes({'scope':SCOPE,'status':'incomplete sequential resource measurement','rows':rows,'inputs':inputs,'checker_binaries':pins}))
 if files()[1:]!=(inputs,refs) or pins!={k:sha(v.read_bytes()) for k,v in paths.items()}:raise ValueError('resource source/artifact/binary changed during measurement')
 maxima={key:max(r['case'][field] for r in rows) for key,field in [('certificate_bytes','bytes'),('terms','terms'),('declarations','declarations')]}
 maxima.update({stage:{field:max(r[stage][field] if stage=='construction' else r['observations'][stage][field] for r in rows) for field in ('wall_seconds','peak_rss_bytes')} for stage in ('construction','rust','go')})
 report={'schema':'mpk.go_practical.sequential_resources.v0','scope':SCOPE,'work_item':'GO-PRAC-T01-W03','inputs':inputs,'prerequisite_artifacts':refs,'checker_binaries':pins,'hardware_receipt':'develop/migrations/go-practical/t01-w03/control-induction/local-resource-host.json','host':{'system':platform.system(),'release':platform.release(),'machine':platform.machine()},'direct_child_parallelism':1,'child_ceiling_seconds':60,'ceiling_scope':'private measurement setting, not a profile budget','rows':rows,'maxima':maxima,'pending':['W04 exact semantic/transport/resource freeze and production executor assignment','T02–T06 production source/operation bindings and complete application gates']}
 (RECORD/'observations.json').write_bytes(json_bytes(report));(RECORD/'attempt.json').unlink();return report
def main():
 parser=argparse.ArgumentParser(description=__doc__);g=parser.add_mutually_exclusive_group(required=True);g.add_argument('--write',action='store_true');g.add_argument('--check',action='store_true');a=parser.parse_args()
 if a.write:measure()
 from check_resource_envelope import check
 print(f"PASS: {check(json.loads((RECORD/'observations.json').read_text()))} sequential representative resources; no frozen budget or full-profile completion")
if __name__=='__main__':main()
