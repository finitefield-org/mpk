"""Keep rejected positive attempts separate from semantic-negative evidence."""
import json
from pathlib import Path
import re
from run_proof_pilot import ROOT,sha
import run_affix_probes as predecessor
RECORD=ROOT/'develop/migrations/go-practical/t01-w03/aggregate-initial'
def check():
 read=lambda name:json.loads((RECORD/name).read_text())
 scope=read('scope.json')
 if scope!={'schema':'mpk.go_practical.aggregate_initial.v0','scope':'three actually observed intended-positive core type rejections; neither false-claim evidence nor W03 completion','correction':'checked negation congruence for literal bounds and explicit product-field amount updates; unchanged kernels/fuel/domains','predecessor':'f54d8bbbbee66d00c615f9894827e2b3ab34099a'}:raise ValueError('aggregate initial scope')
 pins=read('inputs.json');current={r['path']:r['sha256'] for r in predecessor.proof_sources()}
 archived={p.relative_to(RECORD/'source').as_posix():sha(p.read_bytes()) for p in (RECORD/'source').rglob('*') if p.is_file()}
 expected={**current,**archived}
 if pins!=[{'path':p,'sha256':h} for p,h in sorted(expected.items())]:raise ValueError('aggregate initial source pins')
 binaries=read('binaries.json')
 if set(binaries)!={'rust','go'} or any(re.fullmatch('[0-9a-f]{64}',h) is None for h in binaries.values()):raise ValueError('aggregate initial binary pins')
 rows=read('observations.json')
 if [r['case']['id'] for r in rows]!=[f'aggregate-1-1-{f}-valid' for f in ('layout-frame','full-cell','prefix-read')]:raise ValueError('aggregate initial observed coverage')
 for row in rows:
  c=row['case'];raw=(RECORD/(c['id']+'.mpcert')).read_bytes()
  if c['expected']!='accepted' or c['mutation']!=0 or len(raw)!=c['bytes'] or sha(raw)!=row['raw_sha256']:raise ValueError('aggregate initial candidate identity')
  if set(row['observations'])!={'rust','go'}:raise ValueError('aggregate initial checker coverage')
  for name,m in row['observations'].items():
   r=m['report'];error=r.get('error_detail')
   if m['exit_code']!=1 or r['verdict']!='rejected':raise ValueError('aggregate initial positive promoted')
   if name=='rust':
    if r['error_code']!='KERNEL_CORE_CHECK' or json.loads(error)['code']!='CORE_TYPE_MISMATCH':raise ValueError('aggregate initial rejection class')
   elif r['error_kind']!='core_check' or not error.startswith('term ') or ' inferred ' not in error or ' but expected ' not in error:raise ValueError('aggregate initial rejection class')
 return len(rows)
if __name__=='__main__':print(f'PASS: {check()} observed intended-positive rejections; no semantic-negative promotion')
