"""Validate two distinct observed limitations of the unmodified Nat interface."""
import json
from run_proof_pilot import ROOT,sha
import run_affix_probes as predecessor
RECORD=ROOT/'develop/migrations/go-practical/t01-w03/control-initial'
def check():
 read=lambda name:json.loads((RECORD/name).read_text())
 scope=read('scope.json')
 if scope!={'scope':'one actually observed intended-positive type rejection under unchanged checkers; no general Nat induction currently admitted','finding':'Std.Nat.rec is Nat -> (Nat -> Nat -> Nat) -> Nat -> Nat; the candidate incorrectly assumed a dependent motive argument','private_amendment':'test additive monomorphic Sort0 Nat.induct against independently extended mirrored checkers; production unchanged'}:raise ValueError('initial control scope')
 pins=read('inputs.json');expected={r['path']:r['sha256'] for r in predecessor.proof_sources()}
 expected.update({p.relative_to(RECORD/'source').as_posix():sha(p.read_bytes()) for p in (RECORD/'source').rglob('*') if p.is_file()})
 if pins!=[{'path':p,'sha256':h} for p,h in sorted(expected.items())]:raise ValueError('initial control source identity')
 rows=read('observations.json')
 if len(rows)!=1 or rows[0]['case']['id']!='control-invariant-entry-valid':raise ValueError('initial control observed coverage')
 r=rows[0];raw=(RECORD/(r['case']['id']+'.mpcert')).read_bytes()
 if sha(raw)!=r['raw_sha256'] or len(raw)!=r['case']['bytes']:raise ValueError('initial control candidate bytes')
 if set(r['observations'])!={'rust','go'}:raise ValueError('initial control checker coverage')
 for k,v in r['observations'].items():
  rep=v['report'];error=rep['error_detail']
  if v['exit_code']!=1 or rep['verdict']!='rejected':raise ValueError('initial control result promotion')
  if k=='rust' and json.loads(error)['code']!='CORE_TYPE_MISMATCH':raise ValueError('initial control type rejection')
  if k=='go' and not (' inferred ' in error and ' but expected ' in error):raise ValueError('initial control type rejection')
 final=read('current-interface/observations.json');raw=(RECORD/'current-interface/control-invariant-entry-valid.mpcert').read_bytes()
 if final['scope']!='corrected proposed-interface candidate rejected by unchanged production checkers; separate private prototype required' or final['raw_sha256']!=sha(raw):raise ValueError('current interface initial identity/scope')
 if set(final['observations'])!={'rust','go'}:raise ValueError('current interface checker coverage')
 for k,v in final['observations'].items():
  if v['exit_code']!=1 or v['report']['verdict']!='rejected':raise ValueError('current interface admitted by production')
  if k=='rust' and json.loads(v['report']['error_detail'])['code']!='CORE_UNSUPPORTED_FEATURE':raise ValueError('current interface baseline class')
  if k=='go' and v['report']['error_detail']!='generated recursor name does not match family':raise ValueError('current interface baseline class')
 return 2
if __name__=='__main__':print(f'PASS: {check()} distinct original-checker diagnostics; no production induction admission')
