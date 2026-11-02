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

def inspect(path):
 raw,module,table,term=decode(path);meta=json.loads(path.with_suffix('.sources.json').read_bytes());report=json.loads(path.with_suffix('.rust.json').read_bytes())
 assert report['verdict']=='accepted' and report['axiom_count']==0 and report['module']==module==meta['module']
 assert report['hashes']['certificate']==hashlib.sha256(b'MPK-MODULE-CERT-0.1\0'+raw).hexdigest()
 assert report['declaration_count']==len(table)
 checked=set()
 def check(n,ty,body=None,law=False):
  d=table[n];assert d[1]==(2 if law else 1),(n,'kind');assert term(d[2])==m.frozen_ast(ty),(n,'type')
  if body is not None:assert term(d[3])==m.frozen_ast(body),(n,'body')
  checked.add(n)
 parent=meta['source_layout_module'];strings=meta['string_module'];string_id=meta['source_string_type_id'];index_id=meta['source_index_type_id'];byte_id=meta['source_byte_type_id']
 keys={k['id']:k for k in meta['types']};underlying=dict(meta['underlying']);cases=meta['cases'];assert cases==sorted(set(cases))
 assert strings==parent+'.Strings.T'+string_id and module==strings+'.Index.T'+index_id
 def shape(i):
  seen=set()
  while i in underlying:assert i not in seen;seen.add(i);i=underlying[i]
  return keys[i]
 key=shape(index_id);assert key['kind']=='builtin' and key['name'] in ['int','uint','int8','uint8','int16','uint16','int32','uint32','int64','uint64']
 width=key['width'];signed=key['signed'];assert (width,signed)==(meta['width'],meta['signed']) and width in [8,16,32,64]
 assert shape(string_id)['name']=='string' and keys[byte_id]['name']=='uint8' and keys[byte_id]['width']==8 and keys[byte_id]['signed'] is False
 def c(n,*xs):return f.app(f.const(n),*xs)
 def own(n,*xs):return c(module+'.'+n,*xs)
 def string(n,*xs):return c(strings+'.'+n,*xs)
 def num(n,*xs):return c('Mpk.Go.Practical.Word.Numeric.'+n,*xs)
 def integer(n,*xs):return c('Mpk.Go.Practical.Int.'+n,*xs)
 def pair(i):return f.const(parent+'.T'+i+'.Carrier'),f.const(parent+'.T'+i+'.Zero')
 def product(parts):return f.product(parts,[0])
 # Every original type/U/zero-size edge remains, including unrelated payloads.
 for tid,k in keys.items():
  kind=k['kind']
  if kind=='nominal':expected=pair(underlying[tid])
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
 i=f.var('i');s=f.var('s');it=pair(index_id)[0];st=pair(string_id)[0];bt=pair(byte_id)[0];word=f.word(width)[0];word64=f.word(64)[0];byte=f.word(8)[0];inttype=f.pi('_part',f.boolean(),f.nat())
 ia=[('i',it)];si=[('s',st),('i',it)];guarded=si+[('defined',own('Defined',s,i))]
 negative=c('Mpk.Go.Practical.Word.Numeric.Bit',f.app(own('ToWord',i),*[f.bit(True)]*((width-1).bit_length())))
 sign=f.app(own('ToWord',i),*[f.bit(True)]*((width-1).bit_length())) if signed else f.bit(False)
 natural=own('Natural',i);converted=c(f'Mpk.Go.Practical.Word.W{width}.UnsignedTo64',own('ToWord',i));iv=num(f'W{width}.Signed',own('ToWord',i)) if signed else integer('FromNat',natural)
 check(module+'.ToWord',f.bind('pi',ia,word),f.bind('lam',ia,i))
 check(module+'.To64',f.bind('pi',ia,word64),f.bind('lam',ia,converted))
 check(module+'.Natural',f.bind('pi',ia,f.nat()),f.bind('lam',ia,num('Value'+str((width-1).bit_length()),own('ToWord',i))))
 check(module+'.Negative',f.bind('pi',ia,f.boolean()),f.bind('lam',ia,sign))
 nn=f.eq(f.boolean(),own('Negative',i),f.bit(False))
 check(module+'.Nonnegative',f.bind('pi',ia,{'kind':'sort0'}),f.bind('lam',ia,nn))
 check(module+'.Integer',f.bind('pi',ia,inttype),f.bind('lam',ia,iv))
 check(module+'.IntegerDefinition',f.bind('pi',ia,f.eq(inttype,own('Integer',i),iv)),law=True)
 check(module+'.To64Definition',f.bind('pi',ia,f.eq(word64,own('To64',i),converted)),law=True)
 check(module+'.Natural64',f.bind('pi',ia,f.eq(f.nat(),num('Value6',own('To64',i)),natural)),law=True)
 for member,want in [('Positive',natural),('Negative',c('Mpk.Go.Practical.Nat.Mul',num('Weight'+str((width-1).bit_length())),negative) if signed else f.const('Std.Nat.zero'))]:
  check(module+'.'+member+'Value',f.bind('pi',ia,f.eq(f.nat(),integer(member,own('Integer',i)),want)),law=True)
 nonneg=ia+[('nonnegative',own('Nonnegative',i))]
 check(module+'.NonnegativeNegativeZero',f.bind('pi',nonneg,f.eq(f.nat(),integer('Negative',own('Integer',i)),f.const('Std.Nat.zero'))),law=True)
 equivalent=integer('Equivalent',own('Integer',i),integer('FromNat',natural))
 check(module+'.NonnegativeInteger',f.bind('pi',nonneg,equivalent),law=True)
 within=f.eq(f.boolean(),c('Mpk.Go.Practical.Nat.Less',natural,string('Size',s)),f.bit(True))
 clauses=[string('Valid',s),own('Nonnegative',i),within]
 check(module+'.Defined',f.bind('pi',si,{'kind':'sort0'}),f.bind('lam',si,product([(t,f.bit(False)) for t in clauses])[0]))
 for n,t in zip(['DefinedValid','DefinedNonnegative','DefinedWithin'],clauses):check(module+'.'+n,f.bind('pi',guarded,t),law=True)
 check(module+'.DefinedOfWitnesses',f.bind('pi',si+[('valid',clauses[0]),('nonnegative',clauses[1]),('within',clauses[2])],own('Defined',s,i)),law=True)
 check(module+'.Within64',f.bind('pi',guarded,f.eq(f.boolean(),c('Mpk.Go.Practical.Nat.Less',num('Value6',own('To64',i)),string('Size',s)),f.bit(True))),law=True)
 check(module+'.Read',f.bind('pi',si,bt),f.bind('lam',si,string('Byte',s,own('To64',i))))
 address=[f.app(own('To64',i),*[f.bit(bool(b&(1<<j))) for j in range(5,-1,-1)]) for b in range(7,-1,-1)]
 check(module+'.ReadWithinSlot',f.bind('pi',guarded,f.eq(byte,own('Read',s,i),f.app(string('Data',s),*address))),law=True)
 check(module+'.DefinedInteger',f.bind('pi',guarded,equivalent),law=True)
 if signed:check(module+'.RejectNegative',f.bind('pi',guarded+[('negative',f.eq(f.boolean(),own('Negative',i),f.bit(True)))],f.eq(f.boolean(),f.bit(False),f.bit(True))),law=True)
 else:assert module+'.RejectNegative' not in table
 assert {n for n in table if n.startswith(module+'.')}=={n for n in checked if n.startswith(module+'.')}
 # Exact prior primitive identities precede every accelerated observation.
 for name in ['Mpk.Go.Practical.Nat.Add','Mpk.Go.Practical.Nat.Mul','Mpk.Go.Practical.Nat.Less','Mpk.Go.Practical.FiniteVector.D8.Lookup']:
  d,old=table[name],BASE[2][name];assert d[1]==old[1]==1 and term(d[2])==BASE[3](old[2]) and term(d[3])==BASE[3](old[3]),(name,'prior primitive identity')
 const_cache={}
 def force(v):return evaluate(v[1],v[2]) if isinstance(v,tuple) and v[0]=='thunk' else v
 def prim(name,args=()):return ('primitive',name,args)
 def apply(fn,arg):
  fn=force(fn)
  if callable(fn):return fn(force(arg))
  if fn[0]=='closure':return evaluate(fn[1],fn[2]+(arg,))
  assert fn[0]=='primitive';name=fn[1];args=fn[2]+(arg,);arity=3 if name=='lookup' else 1 if name=='succ' else 2
  if len(args)<arity:return prim(name,args)
  if name=='lookup':
   data,index=force(args[1]),force(args[2]);assert type(index) is int and 0<=index<256
   for b in range(7,-1,-1):data=apply(data,bool(index&(1<<b)))
   return data
  values=[force(a) for a in args]
  assert all(type(v) is int for v in values)
  if name=='succ':return values[0]+1
  if name=='add':return values[0]+values[1]
  if name=='mul':return values[0]*values[1]
  assert name=='less';return values[0]<values[1]
 def evaluate(ast,scope=()):
  k,*a=ast
  if k=='var':return force(scope[-1-a[0]])
  if k=='lam':return ('closure',a[1],scope)
  if k=='const':
   name=a[0]
   if name in const_cache:return const_cache[name]
   if name=='Std.Bool.false':value=False
   elif name=='Std.Bool.true':value=True
   elif name=='Std.Nat.zero':value=0
   elif name=='Std.Nat.succ':value=prim('succ')
   elif name in ['Mpk.Go.Practical.Nat.Add','Mpk.Go.Practical.Nat.Mul','Mpk.Go.Practical.Nat.Less']:value=prim(name.rsplit('.',1)[1].lower())
   elif name=='Mpk.Go.Practical.FiniteVector.D8.Lookup':value=prim('lookup')
   else:d=table[name];assert d[1]==1,name;value=evaluate(term(d[3]))
   const_cache[name]=value;return value
  assert k=='app',k
  head,args=a
  if head[0]=='const' and head[1] in ['Std.Bool.cases','Std.Bool.rec']:
   arity=4 if head[1]=='Std.Bool.cases' else 3;assert len(args)==arity
   condition=force(evaluate(args[-1],scope));assert type(condition) is bool
   return evaluate(args[(2 if condition else 1) if arity==4 else (1 if condition else 0)],scope)
  value=evaluate(head,scope)
  for arg in args:value=apply(value,('thunk',arg,scope))
  return value
 def call(name,*args):
  d=table[name];assert d[1]==1;value=evaluate(term(d[3]))
  for arg in args:value=apply(value,arg)
  return force(value)
 def curry(count,fn,bits=()):return fn(bits) if count==0 else lambda bit:curry(count-1,fn,bits+(bit,))
 def number(bits):
  result=0
  for bit in bits:result=2*result+int(bit)
  return result
 def word_value(w,n):return curry((w-1).bit_length(),lambda bits:bool(n&(1<<number(bits))))
 def observe_word(value,w):
  result=0
  for bit in range(w):
   v=value
   for b in range((w-1).bit_length()-1,-1,-1):v=apply(v,bool(bit&(1<<b)))
   v=force(v);assert type(v) is bool
   if v:result|=1<<bit
  return result
 mask=(1<<width)-1
 values=set(range(256)) if width==8 else {0,1,127,128,255,256,257,mask,mask>>1,1<<(width-1)}|{x for b in range(width) for x in [1<<b,mask^(1<<b)]}
 data=curry(11,lambda bits:bool(((number(bits[:8])*73+19)&255)&(1<<number(bits[8:]))))
 string_value=call(strings+'.Make',word_value(64,256),data)
 reads=0;observations=[]
 for original in sorted(values):
  index=word_value(width,original);actual_natural=call(module+'.Natural',index);actual_negative=call(module+'.Negative',index);actual64=observe_word(call(module+'.To64',index),64)
  actual_integer=call(module+'.Integer',index);positive=force(apply(actual_integer,False));negative_part=force(apply(actual_integer,True))
  negative=bool(signed and original&(1<<(width-1)))
  assert type(actual_natural) is int and actual_natural==original and actual64==original
  assert type(actual_negative) is bool and actual_negative==negative
  assert positive-negative_part==original-((1<<width) if negative else 0)
  if not negative and original<256:
   assert observe_word(call(module+'.Read',string_value,index),8)==(original*73+19)&255;reads+=1
  observations.append({'raw_unsigned':original,'signed_negative':negative,'actual_integer':positive-negative_part,'full64_zero_extension':actual64})
 return {'certificate':str(path),'module':module,'index_width':width,'signed':signed,'source_types':len(keys),'error_cases':len(cases),'new_exact_members':len([n for n in checked if n.startswith(module+'.')]),'source_recipe_members':3*len(keys),'observed_integer_inputs':observations,'full64_bit_observations':64*len(values),'complete_byte_bit_observations':8*reads,'typed_defined_predicate_keeps_actual_sign_and_complete_value':True,'source_expression_or_context_authority':False}
