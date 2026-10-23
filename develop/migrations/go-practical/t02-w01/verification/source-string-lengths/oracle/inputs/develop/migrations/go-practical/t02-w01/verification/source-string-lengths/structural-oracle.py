"""Independent complete source/index definitions, statements and actual word DAG.

Nat/lookup observation acceleration is admitted only after exact type/body
comparison with the earlier accepted original source-string library. No new
axiom, function equality, source execution or context authority is inferred.
"""
from pathlib import Path
import functools,hashlib,importlib.util,json
ROOT=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location('frozen_source_layout',ROOT.parent/'source-error-layouts/structural-oracle.py')
m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);f=m.f

def decode(path):
 raw=path.read_bytes();r=m.Sections(raw);assert raw[:7]==b'MPKCERT';r.at=7
 assert r.string()=='MPK-CERT-0.1' and r.string()=='MPK-Core-0.1'
 module=r.string();assert r.number()==0
 name_table=r.vec(r.string);levels=r.vec(r.level);terms=r.vec(r.term);assert r.number()==0
 declarations=r.vec(r.declaration);table={name_table[d[0]]:d for d in declarations}
 @functools.lru_cache(None)
 def level(i):assert levels[i][0]==0;return ('level',0)
 @functools.lru_cache(None)
 def term(i):
  tag,*a=terms[i]
  if tag==0:return ('sort',level(a[0]))
  if tag==1:return ('var',a[0])
  if tag==2:return ('const',name_table[declarations[a[0]][0]],tuple(level(j) for j in a[1]))
  if tag==3:return ('app',term(a[0]),tuple(term(j) for j in a[1]))
  if tag in [4,5]:return ('lam' if tag==4 else 'pi',term(a[0]),term(a[1]))
  return ('let',*(term(j) for j in a))
 return raw,module,table,term

BASE_PATH=next((ROOT.parent/'source-strings/validity-parts/certificates').glob('nil-only-*.mpcert'))
BASE=decode(BASE_PATH)
assert json.loads(BASE_PATH.with_suffix('.rust.json').read_bytes())['verdict']=='accepted'

MATH='Mpk.Go.Practical.String.GoLength'
def inspect(path):
 raw,module,table,term=decode(path);meta=json.loads(path.with_suffix('.sources.json').read_bytes());report=json.loads(path.with_suffix('.rust.json').read_bytes())
 assert report['verdict']=='accepted' and report['axiom_count']==0 and report['module']==module==meta['module']
 assert report['hashes']['certificate']==hashlib.sha256(b'MPK-MODULE-CERT-0.1\0'+raw).hexdigest() and report['declaration_count']==len(table)
 checked=set()
 def check(n,ty,body=None,law=False):
  d=table[n];assert d[1]==(2 if law else 1),(n,'kind');assert term(d[2])==m.frozen_ast(ty),(n,'type')
  if body is not None:assert term(d[3])==m.frozen_ast(body),(n,'body')
  checked.add(n)
 def c(n,*xs):return f.app(f.const(n),*xs)
 def own(n,*xs):return c(module+'.'+n,*xs)
 def math(n,*xs):return c(MATH+'.'+n,*xs)
 def num(n,*xs):return c('Mpk.Go.Practical.Word.Numeric.'+n,*xs)
 def integer(n,*xs):return c('Mpk.Go.Practical.Int.'+n,*xs)
 parent=meta['source_layout_module'];strings=meta['string_module'];sid=meta['source_string_type_id'];rid=meta['source_result_type_id']
 keys={k['id']:k for k in meta['types']};under=dict(meta['underlying']);cases=meta['cases'];assert cases==sorted(set(cases))
 assert strings==parent+'.Strings.T'+sid and module==strings+'.GoLength.T'+rid
 assert keys[rid]['kind']=='builtin' and keys[rid]['name']=='int' and keys[rid]['width']==64 and keys[rid]['signed'] is True
 assert not any(k.get('name')=='uint8' for k in keys.values())
 def pair(i):return f.const(parent+'.T'+i+'.Carrier'),f.const(parent+'.T'+i+'.Zero')
 def product(parts):return f.product(parts,[0])
 for tid,k in keys.items():
  kind=k['kind']
  if kind=='nominal':expected=pair(under[tid])
  elif kind=='builtin':
   name=k['name']
   if name=='bool':expected=f.boolean(),f.bit(False)
   elif name=='string':expected=product([f.word(64),f.vector(8,*f.word(8))])
   elif name=='error':expected=product([f.word(64),*[pair(case) for case in cases]])
   else:expected=f.word(k['width'])
  elif kind=='array':
   assert k['element'] in keys;expected=(f.boolean(),f.bit(False)) if k['length']==0 else f.vector((k['length']-1).bit_length(),*pair(k['element']))
  elif kind=='slice':
   assert k['element'] in keys;expected=product([(f.boolean(),f.bit(True)),(f.nat(),f.const('Std.Nat.zero')),f.vector(4,*product([f.word(8),f.word(64)])),f.word(64),f.word(64),f.word(64)])
  else:assert kind=='struct';expected=product([pair(field['type_id']) for field in k['fields']])
  ty,z=expected;check(parent+'.T'+tid+'.Carrier',{'kind':'sort0'},ty);check(parent+'.T'+tid+'.Zero',ty,z);check(parent+'.T'+tid+'.ZeroRecipe',f.eq(ty,pair(tid)[1],z),f.refl(ty,z),True)
 def string(n,*xs):return c(strings+'.'+n,*xs)
 nat=f.nat();boolean=f.boolean();zero=f.const('Std.Nat.zero');one=c('Std.Nat.succ',zero);two=c('Mpk.Go.Practical.Nat.Add',one,one)
 def add(a,b):return c('Mpk.Go.Practical.Nat.Add',a,b)
 def mul(a,b):return c('Mpk.Go.Practical.Nat.Mul',a,b)
 def less(a,b):return c('Mpk.Go.Practical.Nat.Less',a,b)
 def numeral(n):
  value=zero
  for _ in range(n):value=c('Std.Nat.succ',value)
  return value
 def power(n):return num('PowerTwo',numeral(n))
 def half(d):return c('Mpk.Go.Practical.Word.SignedBounds.Half'+str(d))
 def value(w):return num('Value6',w)
 def signed(w):return num('W64.Signed',w)
 def top(w):return f.app(w,*[f.bit(True)]*6)
 wt=f.word(64)[0];it=f.pi('_part',boolean,nat);st=pair(sid)[0];rt=pair(rid)[0];s=f.var('s');w=f.var('word');n=f.var('n');x=f.var('w');sa=[('s',st)]
 length=c('Mpk.Go.Practical.String.Length',string('ToMath',s));cap=c('Mpk.Go.Practical.FiniteVector.D8.Capacity');size=string('Size',s);lw=own('ToWord',own('Len',s))
 check(module+'.ToWord',f.pi('n',rt,wt),f.lam('n',rt,n))
 check(module+'.FromWord',f.pi('w',wt,rt),f.lam('w',wt,x))
 check(module+'.ToFromWord',f.pi('w',wt,f.eq(wt,own('ToWord',own('FromWord',x)),x)),law=True)
 check(module+'.FromToWord',f.pi('n',rt,f.eq(rt,own('FromWord',own('ToWord',n)),n)),law=True)
 check(module+'.Len',f.bind('pi',sa,rt),f.bind('lam',sa,own('FromWord',string('Length',s))))
 check(module+'.LenDefinition',f.bind('pi',sa,f.eq(rt,own('Len',s),own('FromWord',length))),law=True)
 check(module+'.LenWord',f.bind('pi',sa,f.eq(wt,lw,length)),law=True)
 check(module+'.Natural',f.bind('pi',sa,nat),f.bind('lam',sa,value(lw)))
 check(module+'.LenNatural',f.bind('pi',sa,f.eq(nat,own('Natural',s),size)),law=True)
 check(module+'.AsInteger',f.bind('pi',sa,it),f.bind('lam',sa,signed(lw)))
 check(module+'.AsIntegerDefinition',f.bind('pi',sa,f.eq(it,own('AsInteger',s),signed(lw))),law=True)
 for prefix,predicate in [('Fits','LengthFits'),('Valid','Valid')]:
  args=sa+[('fits' if prefix=='Fits' else 'valid',string(predicate,s))]
  check(module+'.'+prefix+'Nonnegative',f.bind('pi',args,f.eq(boolean,top(lw),f.bit(False))),law=True)
  check(module+'.'+prefix+'Integer',f.bind('pi',args,f.eq(it,own('AsInteger',s),integer('FromNat',size))),law=True)
 generic_zero=c('Mpk.Go.Practical.String.Length',c('Mpk.Go.Practical.String.Zero'))
 check(module+'.ZeroWordToSource',f.eq(rt,own('FromWord',generic_zero),pair(rid)[1]),law=True)
 check(module+'.LenZero',f.eq(rt,own('Len',pair(sid)[1]),pair(rid)[1]),law=True)
 a,b,limit=f.var('a'),f.var('b'),f.var('c')
 check(MATH+'.AtMostBelow',f.bind('pi',[('a',nat),('b',nat),('c',nat),('upper',f.eq(boolean,less(b,a),f.bit(False))),('strict',f.eq(boolean,less(b,limit),f.bit(True)))],f.eq(boolean,less(a,limit),f.bit(True))),law=True)
 check(MATH+'.PositiveAdd',f.bind('pi',[('a',nat),('b',nat),('positive',f.eq(boolean,less(zero,b),f.bit(True)))],f.eq(boolean,less(a,add(a,b)),f.bit(True))),law=True)
 for d in range(7):check(MATH+'.HalfPower'+str(d),f.eq(nat,half(d),power((1<<d)-1)),law=True)
 rest=mul(mul(two,power(8)),c('Mpk.Go.Practical.Word.Shifts.PowerMinusOne',numeral(54)))
 check(MATH+'.Power63Gap',f.eq(nat,power(63),add(power(8),add(power(8),rest))),law=True)
 check(MATH+'.Power8LessPower63',f.eq(boolean,less(power(8),power(63)),f.bit(True)),law=True)
 check(MATH+'.CapacityBelowHalf',f.eq(boolean,less(cap,half(6)),f.bit(True)),law=True)
 check(MATH+'.BoundedSign',f.bind('pi',[('word',wt),('upper',f.eq(boolean,less(cap,value(w)),f.bit(False)))],f.eq(boolean,top(w),f.bit(False))),law=True)
 check(MATH+'.SignedAsNat',f.bind('pi',[('word',wt),('clear',f.eq(boolean,top(w),f.bit(False)))],f.eq(it,signed(w),integer('FromNat',value(w)))),law=True)
 assert {name for name in table if name.startswith(module+'.')}=={name for name in checked if name.startswith(module+'.')},'unexpected source-length export'
 # Only exact preexisting mathematical definitions are accelerated.
 for name in ['Mpk.Go.Practical.Nat.Add','Mpk.Go.Practical.Nat.Mul','Mpk.Go.Practical.Nat.Sub','Mpk.Go.Practical.Nat.Less']:
  d,old=table[name],BASE[2][name];assert d[1]==old[1]==1 and term(d[2])==BASE[3](old[2]) and term(d[3])==BASE[3](old[3]),(name,'prior primitive identity')
 const_cache={}
 def force(v):return evaluate(v[1],v[2]) if isinstance(v,tuple) and v[0]=='thunk' else v
 def prim(name,args=()):return ('primitive',name,args)
 def apply(fn,arg):
  fn=force(fn)
  if callable(fn):return fn(force(arg))
  if fn[0]=='closure':return evaluate(fn[1],fn[2]+(arg,))
  assert fn[0]=='primitive';name=fn[1];args=fn[2]+(arg,);arity=3 if name=='rec' else 1 if name=='succ' else 2
  if len(args)<arity:return prim(name,args)
  if name=='rec':
   value=force(args[0]);step=force(args[1]);count=force(args[2]);assert type(count) is int and 0<=count<=64
   for i in range(count):value=apply(apply(step,i),value)
   return value
  values=[force(a) for a in args];assert all(type(v) is int for v in values)
  if name=='succ':return values[0]+1
  if name=='add':return values[0]+values[1]
  if name=='mul':return values[0]*values[1]
  if name=='sub':return max(0,values[0]-values[1])
  assert name=='less';return values[0]<values[1]
 def evaluate(ast,scope=()):
  k,*a=ast
  if k=='var':return force(scope[-1-a[0]])
  if k=='lam':return ('closure',a[1],scope)
  if k=='const':
   name=a[0]
   if name in const_cache:return const_cache[name]
   if name=='Std.Bool.false':v=False
   elif name=='Std.Bool.true':v=True
   elif name=='Std.Nat.zero':v=0
   elif name=='Std.Nat.succ':v=prim('succ')
   elif name=='Std.Nat.rec':v=prim('rec')
   elif name in ['Mpk.Go.Practical.Nat.Add','Mpk.Go.Practical.Nat.Mul','Mpk.Go.Practical.Nat.Sub','Mpk.Go.Practical.Nat.Less']:v=prim(name.rsplit('.',1)[1].lower())
   else:d=table[name];assert d[1]==1,name;v=evaluate(term(d[3]))
   const_cache[name]=v;return v
  assert k=='app',k;head,args=a
  if head[0]=='const' and head[1] in ['Std.Bool.cases','Std.Bool.rec']:
   arity=4 if head[1]=='Std.Bool.cases' else 3;assert len(args)==arity
   condition=force(evaluate(args[-1],scope));assert type(condition) is bool
   return evaluate(args[(2 if condition else 1) if arity==4 else (1 if condition else 0)],scope)
  result=evaluate(head,scope)
  for arg in args:result=apply(result,('thunk',arg,scope))
  return result
 def call(name,*args):
  d=table[name];assert d[1]==1;v=evaluate(term(d[3]))
  for arg in args:v=apply(v,arg)
  return force(v)
 # Polynomial checkpoints are closed actual Nat equalities, not unchecked
 # emitter names or a replacement for the full kernel proof.
 checkpoints=0
 for name,d in table.items():
  if not name.startswith(MATH+'.') or name in checked:continue
  assert d[1]==2,(name,'extra non-theorem');ast=term(d[2]);assert ast[0]=='app' and ast[1][1]=='Std.Eq' and ast[2][0]==('const','Std.Nat',())
  assert evaluate(ast[2][1])==evaluate(ast[2][2]),name;checkpoints+=1
 def curry(count,fn,bits=()):return fn(bits) if count==0 else lambda bit:curry(count-1,fn,bits+(bit,))
 def number(bits):
  n=0
  for bit in bits:n=2*n+int(bit)
  return n
 def word(n):return curry(6,lambda bits:bool(n&(1<<number(bits))))
 def observe(v):
  result=0
  for bit in range(64):
   current=v
   for b in range(5,-1,-1):current=apply(current,bool(bit&(1<<b)))
   current=force(current);assert type(current) is bool
   if current:result|=1<<bit
  return result
 data=curry(11,lambda bits:False)
 mask=(1<<64)-1;values=set(range(257))|{mask,mask>>1}|{v for bit in range(64) for v in [1<<bit,mask^(1<<bit)]};observations=[]
 for n in sorted(values):
  original=call(strings+'.Make',word(n),data);actual=call(module+'.Len',original);asword=call(module+'.ToWord',actual);natural=call(module+'.Natural',original);asint=call(module+'.AsInteger',original);positive=force(apply(asint,False));negative=force(apply(asint,True))
  assert observe(asword)==natural==n and positive-negative==n-((1<<64) if n&(1<<63) else 0)
  if n<=256:assert positive-negative==n
  observations.append({'full_unsigned_header':n,'actual_integer':positive-negative,'fits_frozen_capacity':n<=256})
 return {'certificate':str(path),'module':module,'source_types':len(keys),'error_cases':len(cases),'new_source_members':len([name for name in checked if name.startswith(module+'.')]),'generic_public_statements':len([name for name in checked if name.startswith(MATH+'.')]),'closed_polynomial_checkpoints':checkpoints,'source_recipe_members':3*len(keys),'all257_valid_lengths_and_full64_raw_observations':observations,'full64_bit_observations':64*len(values),'source_uint8_type_required':False,'source_expression_or_context_authority':False}
