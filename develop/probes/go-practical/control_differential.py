"""Independent complete-frame observations and nonvacuous loop-control witnesses."""
import json
from pathlib import Path
from order_differential import adder
from run_proof_pilot import json_bytes,sha,timed
MASK=(1<<64)-1
ARTIFACTS=('control-input.json','control-output.json')
SCOPE='original Go for/continue/break/return and full128 frames versus independent bit arithmetic; runtime observations are not arbitrary-Nat induction or production source binding'
def encode(pair):return dict(low=str(pair[0]),high=str(pair[1]))
def decode(d):return int(d['low']),int(d['high'])
def rest(s):return s[0]^128,s[1]^1
def post(s):return adder(64,s[0],2,False)[0],s[1]^(1<<63)
def step(s,bad):
 s=post(rest(s));return s[0]^int(bad),s[1]
def invariant(s):return s[0]&1==0
def inputs():
 out=[]
 for n in (0,1,2,31,32,33,127,128,129):
  for state in ((0,0),(MASK-1,MASK),(1,1<<63),(MASK,MASK)):
   for flags in range(8):
    for bad in (False,True):out.append(dict(n=n,state=encode(state),return_frame=encode((MASK,7)),break_frame=encode((2,1<<63)),returned=bool(flags&4),broken=bool(flags&2),continued=bool(flags&1),bad_step=bad))
 return out
def model(r):
 s=decode(r['state']);a=h=s
 for _ in range(r['n']):
  a=step(a,r['bad_step'])
  if invariant(h):h=step(h,r['bad_step'])
 if r['returned']:edge=decode(r['return_frame']);rest_calls=post_calls=0
 elif r['broken']:edge=decode(r['break_frame']);rest_calls=post_calls=0
 elif r['continued']:edge=post(s);rest_calls=0;post_calls=1
 else:edge=post(rest(s));rest_calls=post_calls=1
 return dict(iteration=encode(a),invariant=invariant(a),absorbed=encode(h),edge=encode(edge),rest_calls=rest_calls,post_calls=post_calls,input_unchanged=True)
def witnesses(cases,outputs):
 predicates={
  'invariant-entry':lambda r,a:r['n']==0 and not invariant(decode(r['state'])),
  'invariant-step':lambda r,a:r['n']==1 and invariant(decode(r['state'])) and r['bad_step'],
  'absorbing-stop':lambda r,a:r['n']==1 and invariant(decode(r['state'])),
  'continue-post':lambda r,a:not r['returned'] and not r['broken'] and r['continued'],
  'break-exit':lambda r,a:not r['returned'] and r['broken'] and a['edge']!=encode(post(decode(r['state']))),
  'return-precedence':lambda r,a:r['returned'] and r['broken'],
 }
 result=[]
 for f,predicate in predicates.items():
  i=next(i for i,(r,a) in enumerate(zip(cases,outputs)) if predicate(r,a));r,a=cases[i],outputs[i]
  if f.startswith('invariant-'):actual=a['invariant'];altered=True
  elif f=='absorbing-stop':actual=a['absorbed'];altered=r['state']
  elif f=='continue-post':actual=a['edge'];altered=encode(post(rest(decode(r['state']))))
  elif f=='break-exit':actual=a['edge'];altered=encode(post(decode(r['state'])))
  else:actual=a['edge'];altered=r['break_frame']
  if actual==altered:raise ValueError('vacuous control witness')
  result.append({'case_id':f'control-{f}-altered','input_index':i,'input_sha256':sha(json_bytes(r)),'actual':actual,'altered':altered})
 return result
def compare(cases,outputs):
 if cases!=inputs() or len(cases)!=len(outputs):raise ValueError('control input coverage')
 counts=dict.fromkeys(('invariant_preserved','bad_entry_refutations','bad_step_refutations','stopped_frames','continue_posts','break_exits','return_precedence'),0)
 for r,a in zip(cases,outputs):
  expected=model(r)
  if not isinstance(a,dict) or set(a)!=set(expected) or any(type(a[k]) is not type(v) for k,v in expected.items()) or a!=expected:raise ValueError('control original-Go/bit-model mismatch')
  positive=invariant(decode(r['state'])) and not r['bad_step']
  if positive and not a['invariant']:raise ValueError('control positive invariant refuted')
  if not invariant(decode(r['state'])) and a['absorbed']!=r['state']:raise ValueError('control halted full frame changed')
  counts['invariant_preserved']+=positive;counts['bad_entry_refutations']+=r['n']==0 and not a['invariant'];counts['bad_step_refutations']+=r['n']==1 and invariant(decode(r['state'])) and not a['invariant'];counts['stopped_frames']+=not invariant(decode(r['state']));counts['continue_posts']+=not r['returned'] and not r['broken'] and r['continued'];counts['break_exits']+=not r['returned'] and r['broken'];counts['return_precedence']+=r['returned'] and r['broken']
 if not all(counts.values()):raise ValueError('control nonvacuity coverage')
 return counts
def run(executable,directory):
 cases=inputs();raw=json_bytes(cases);p=directory/ARTIFACTS[0];p.write_bytes(raw);m=timed([executable,p]);out=m.pop('stdout').encode()
 if m['exit_code']!=0:raise ValueError('control native observer failed')
 (directory/ARTIFACTS[1]).write_bytes(out);outputs=json.loads(out)
 return dict(scope=SCOPE,cases=len(cases),counts=compare(cases,outputs),false_claim_witnesses=witnesses(cases,outputs),input_sha256=sha(raw),output_sha256=sha(out),go_binary_sha256=sha(Path(executable).read_bytes()),go_measurement=m)
def check(r,directory):
 raw=(directory/ARTIFACTS[0]).read_bytes();out=(directory/ARTIFACTS[1]).read_bytes();cases=json.loads(raw);outputs=json.loads(out)
 if r['scope']!=SCOPE or r['cases']!=len(cases) or r['input_sha256']!=sha(raw) or r['output_sha256']!=sha(out) or r['counts']!=compare(cases,outputs) or r['false_claim_witnesses']!=witnesses(cases,outputs):raise ValueError('control native scope/bytes/witnesses')
 return len(cases)
