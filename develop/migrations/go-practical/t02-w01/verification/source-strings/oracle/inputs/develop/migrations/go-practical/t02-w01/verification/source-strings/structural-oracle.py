"""Independent complete source recipes, adapter bodies/statements and bit observations.

This observes whole certificates accepted by both unchanged cores. It does not
establish Go expression evaluation, signed int typing or context authority.
"""
from pathlib import Path
import functools, hashlib, importlib.util, json
ROOT=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location('source_layout_observer',ROOT.parent/'source-error-layouts/structural-oracle.py')
m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
f=m.f

def inspect(path):
 raw=path.read_bytes();source=json.loads(path.with_suffix('.sources.json').read_bytes());report=json.loads(path.with_suffix('.rust.json').read_bytes())
 assert report['verdict']=='accepted' and report['axiom_count']==0
 assert report['hashes']['certificate']==hashlib.sha256(b'MPK-MODULE-CERT-0.1\0'+raw).hexdigest()
 r=m.Sections(raw);assert raw[:7]==b'MPKCERT';r.at=7
 assert r.string()=='MPK-CERT-0.1' and r.string()=='MPK-Core-0.1'
 module=r.string();assert module==source['module']==report['module'];assert r.number()==0
 names=r.vec(r.string);levels=r.vec(r.level);terms=r.vec(r.term);assert r.number()==0
 declarations=r.vec(r.declaration);table={names[d[0]]:d for d in declarations}
 assert len(table)==len(declarations)==report['declaration_count']
 @functools.lru_cache(None)
 def level(i):
  assert levels[i][0]==0;return ('level',0)
 @functools.lru_cache(None)
 def term(i):
  tag,*args=terms[i]
  if tag==0:return ('sort',level(args[0]))
  if tag==1:return ('var',args[0])
  if tag==2:return ('const',names[declarations[args[0]][0]],tuple(level(j) for j in args[1]))
  if tag==3:return ('app',term(args[0]),tuple(term(j) for j in args[1]))
  if tag in [4,5]:return ('lam' if tag==4 else 'pi',term(args[0]),term(args[1]))
  return ('let',*(term(j) for j in args))
 def check(name,ty,body=None,law=False):
  d=table[name];assert d[1]==(2 if law else 1),(name,'kind')
  assert term(d[2])==m.frozen_ast(ty),(name,'type')
  if body is not None:assert term(d[3])==m.frozen_ast(body),(name,'body')
 parent=source['source_layout_module'];id=source['source_string_type_id'];cases=source['cases']
 assert cases==sorted(set(cases));keys={t['id']:t for t in source['types']};underlying=dict(source['underlying'])
 assert module==parent+'.Strings.T'+id
 current=id;seen=set()
 while keys[current]['kind']=='nominal':
  assert current not in seen;seen.add(current);current=underlying[current]
 assert keys[current]['kind']=='builtin' and keys[current]['name']=='string'
 def pair(i):return f.const(f'{parent}.T{i}.Carrier'),f.const(f'{parent}.T{i}.Zero')
 def product(parts):return f.product(parts,[0])
 # Original zero-size edges and every same-U source type are reconstructed.
 for i,k in keys.items():
  kind=k['kind']
  if kind=='nominal': expected=pair(underlying[i])
  elif kind=='builtin':
   n=k['name']
   if n=='bool':expected=f.boolean(),f.bit(False)
   elif n=='string':expected=product([f.word(64),f.vector(8,*f.word(8))])
   elif n=='error':expected=product([f.word(64),*[pair(c) for c in cases]])
   else:expected=f.word(k['width'])
  elif kind=='array':
   assert k['element'] in keys
   expected=(f.boolean(),f.bit(False)) if k['length']==0 else f.vector((k['length']-1).bit_length(),*pair(k['element']))
  elif kind=='slice':
   assert k['element'] in keys
   expected=product([(f.boolean(),f.bit(True)),(f.nat(),f.const('Std.Nat.zero')),f.vector(4,*product([f.word(8),f.word(64)])),f.word(64),f.word(64),f.word(64)])
  else:
   assert kind=='struct';expected=product([pair(field['type_id']) for field in k['fields']])
  ty,z=expected
  check(f'{parent}.T{i}.Carrier',{'kind':'sort0'},ty)
  check(f'{parent}.T{i}.Zero',ty,z)
  check(f'{parent}.T{i}.ZeroRecipe',f.eq(ty,pair(i)[1],z),f.refl(ty,z),True)
 def own(n,*xs):return f.app(f.const(module+'.'+n),*xs)
 def generic(n,*xs):return f.app(f.const('Mpk.Go.Practical.String.'+n),*xs)
 def g(n):return f.const('Mpk.Go.Practical.String.'+n)
 carrier,z=pair(id);word=f.word(64)[0];byte=f.word(8)[0];slots=f.vector(8,*f.word(8))[0];rawtype=product([f.word(64),f.vector(8,*f.word(8))])[0]
 s=f.var('s');v=f.var('m');math=lambda x:own('ToMath',x)
 check(module+'.ToMath',f.pi('s',carrier,rawtype),f.lam('s',carrier,s))
 check(module+'.ToSource',f.pi('m',rawtype,carrier),f.lam('m',rawtype,v))
 check(module+'.SourceRoundTrip',f.pi('s',carrier,f.eq(carrier,own('ToSource',math(s)),s)),law=True)
 check(module+'.MathRoundTrip',f.pi('m',rawtype,f.eq(rawtype,math(own('ToSource',v)),v)),law=True)
 check(module+'.ZeroToMath',f.eq(rawtype,math(z),g('Zero')),law=True)
 args=[('length',word),('data',slots)];length=f.var('length');data=f.var('data');made=own('Make',length,data)
 check(module+'.Make',f.bind('pi',args,carrier),f.bind('lam',args,own('ToSource',generic('Make',length,data))))
 check(module+'.MakeToMath',f.bind('pi',args,f.eq(rawtype,math(made),generic('Make',length,data))),law=True)
 seqtype=product([(f.nat(),f.bit(False)),(f.pi('_index',f.nat(),byte),f.bit(False))])[0]
 for n,ty in [('Length',word),('Data',slots),('Size',f.nat()),('Sequence',seqtype),*[(n,{'kind':'sort0'}) for n in ['LengthFits','Padding','Valid']]]:
  check(module+'.'+n,f.pi('s',carrier,ty),f.lam('s',carrier,generic(n,math(s))))
 check(module+'.Byte',f.bind('pi',[('s',carrier),('i',word)],byte),f.bind('lam',[('s',carrier),('i',word)],generic('Byte',math(s),f.var('i'))))
 for n,ns,nb in [('LengthBit',0,6),('SlotBit',8,3)]:
  ns_names=[f'slot{i}' for i in range(ns)];nb_names=[f'bit{i}' for i in range(nb)];names=ns_names+nb_names;bits=[f.var(n) for n in names]
  binders=[('s',carrier)]+[(n,f.boolean()) for n in names]
  check(module+'.'+n,f.bind('pi',binders,f.boolean()),f.bind('lam',binders,generic(n,math(s),*bits)))
  wanted=f.app(length,*bits) if ns==0 else f.app(f.app(data,*bits[:ns]),*bits[ns:])
  check(module+'.Make'+n,f.bind('pi',args+[(n,f.boolean()) for n in names],f.eq(f.boolean(),own(n,made,*bits),wanted)),law=True)
 for n,ty,want in [('Length',word,length),('Data',slots,data),('Size',f.nat(),f.app(f.const('Mpk.Go.Practical.Word.Numeric.Value6'),length))]:
  check(module+'.Make'+n,f.bind('pi',args,f.eq(ty,own(n,made),want)),law=True)
 check(module+'.ValidOfParts',f.bind('pi',[('s',carrier),('upper',own('LengthFits',s)),('padding',own('Padding',s))],own('Valid',s)),law=True)
 check(module+'.MakeValid',f.bind('pi',args+[('upper',own('LengthFits',made)),('padding',own('Padding',made))],own('Valid',made)),law=True)
 for n,predicate in [('ValidUpper','LengthFits'),('ValidPadding','Padding')]:
  check(module+'.'+n,f.bind('pi',[('s',carrier),('valid',own('Valid',s))],own(predicate,s)),law=True)
 for n,predicate in [('ZeroValid','Valid'),('ZeroPadding','Padding'),('ZeroFits','LengthFits')]:check(module+'.'+n,own(predicate,z),law=True)
 for n,observation,ty,want in [('ZeroLength','Length',word,generic('Length',g('Zero'))),('ZeroData','Data',slots,generic('Data',g('Zero'))),('ZeroSize','Size',f.nat(),f.const('Std.Nat.zero'))]:
  check(module+'.'+n,f.eq(ty,own(observation,z),want),law=True)
 i=f.var('i');numeric=lambda w:f.app(f.const('Mpk.Go.Practical.Word.Numeric.Value6'),w)
 lt=lambda a,b:f.app(f.const('Mpk.Go.Practical.Nat.Less'),a,b)
 within=f.eq(f.boolean(),lt(numeric(i),own('Size',s)),f.bit(True))
 guarded=[('s',carrier),('i',word),('valid',own('Valid',s)),('within',within)]
 check(module+'.ValidWithinCapacity',f.bind('pi',guarded,f.eq(f.boolean(),lt(numeric(i),f.const('Mpk.Go.Practical.FiniteVector.D8.Capacity')),f.bit(True))),law=True)
 address=[f.app(i,*[f.bit(bool(b&(1<<j))) for j in range(5,-1,-1)]) for b in range(7,-1,-1)]
 check(module+'.ValidByteWithinSlot',f.bind('pi',guarded,f.eq(byte,own('Byte',s,i),f.app(own('Data',s),*address))),law=True)
 # Independently inspect concrete validity; neither its bound nor unused slots
 # may become an arbitrary caller predicate or a pointwise-Eq strengthening.
 size=lambda x:generic('Size',x)
 upper=f.eq(f.boolean(),f.app(f.const('Mpk.Go.Practical.Nat.Less'),f.const('Mpk.Go.Practical.FiniteVector.D8.Capacity'),size(s)),f.bit(False))
 # The canonical natural library uses the fixed global name below.
 less_name='Mpk.Go.Practical.Nat.Less'
 # Discovering a name is not a validity oracle: require the frozen name.
 assert less_name in table
 ss=[f's{i}' for i in range(8)];slotvars=[f.var(n) for n in ss]
 index=f.app(f.const('Mpk.Go.Practical.FiniteVector.D8.Index'),*slotvars)
 outside=f.eq(f.boolean(),f.app(f.const(less_name),index,size(s)),f.bit(False))
 padding=f.bind('pi',[(n,f.boolean()) for n in ss],f.pi('_outside',outside,f.eq(byte,f.app(generic('Data',s),*slotvars),f.word(8)[1])))
 check('Mpk.Go.Practical.String.LengthFits',f.pi('s',rawtype,{'kind':'sort0'}),f.lam('s',rawtype,upper))
 check('Mpk.Go.Practical.String.Padding',f.pi('s',rawtype,{'kind':'sort0'}),f.lam('s',rawtype,padding))
 check('Mpk.Go.Practical.String.Valid',f.pi('s',rawtype,{'kind':'sort0'}),f.lam('s',rawtype,product([(upper,f.bit(False)),(padding,f.bit(False))])[0]))
 # Lazy evaluation of actual retained definition bodies. Types are never
 # evaluated as values; Bool eliminators select their actual concrete branch.
 def force(v):
  if isinstance(v,tuple) and v[0]=='thunk':return evaluate(v[1],v[2])
  return v
 def apply(fn,arg):
  fn=force(fn)
  if callable(fn):return fn(force(arg))
  assert fn[0]=='closure',fn
  return evaluate(fn[1],fn[2]+(arg,))
 def evaluate(ast,scope=()):
  k,*a=ast
  if k=='var':return force(scope[-1-a[0]])
  if k=='lam':return ('closure',a[1],scope)
  if k=='const':
   if a[0]=='Std.Bool.false':return False
   if a[0]=='Std.Bool.true':return True
   d=table[a[0]];assert d[1]==1,a[0]
   return evaluate(term(d[3]))
  assert k=='app',k
  head,xs=a
  if head[0]=='const' and head[1] in ['Std.Bool.cases','Std.Bool.rec']:
   want=4 if head[1]=='Std.Bool.cases' else 3;assert len(xs)==want
   condition=force(evaluate(xs[-1],scope));assert type(condition) is bool
   return evaluate(xs[(2 if condition else 1) if want==4 else (1 if condition else 0)],scope)
  value=evaluate(head,scope)
  for arg in xs:value=apply(value,('thunk',arg,scope))
  return value
 def call_observed(n,*args):
  d=table[module+'.'+n];assert d[1]==1
  value=evaluate(term(d[3]))
  for arg in args:value=apply(value,arg)
  return value
 def curry(count,fn,values=()):return fn(values) if count==0 else lambda b:curry(count-1,fn,values+(b,))
 def number(bits):
  n=0
  for b in bits:n=2*n+int(b)
  return n
 observations=0
 for inverse in [False,True]:
  value_word=0x8000000000000181 ^ (0xffffffffffffffff if inverse else 0)
  original=curry(6,lambda bs:bool(value_word&(1<<number(bs))))
  bytefn=lambda bs:bool(((number(bs[:8])*73+19)^(255 if inverse else 0))&(1<<number(bs[8:])))
  data_value=curry(11,bytefn)
  value=call_observed('Make',original,data_value)
  for address in range(64):
   bits=[bool(address&(1<<b)) for b in range(5,-1,-1)]
   assert force(call_observed('LengthBit',value,*bits))==bool(value_word&(1<<address));observations+=1
  for slot in range(256):
   for b in range(8):
    bits=[bool(slot&(1<<j)) for j in range(7,-1,-1)]+[bool(b&(1<<j)) for j in range(2,-1,-1)]
    assert force(call_observed('SlotBit',value,*bits))==bytefn(bits);observations+=1
 zero_value=evaluate(term(table[f'{parent}.T{id}.Zero'][3]))
 for address in range(64):
  bits=[bool(address&(1<<b)) for b in range(5,-1,-1)]
  assert force(call_observed('LengthBit',zero_value,*bits)) is False;observations+=1
 for slot in range(256):
  for b in range(8):
   bits=[bool(slot&(1<<j)) for j in range(7,-1,-1)]+[bool(b&(1<<j)) for j in range(2,-1,-1)]
   assert force(call_observed('SlotBit',zero_value,*bits)) is False;observations+=1
 return {'certificate':str(path),'module':module,'source_types':len(keys),'complete_error_cases':len(cases),'source_string_type':id,'exact_source_recipe_members':3*len(keys),'complete_length_and_slot_bit_observations':observations,'original_source_and_all_encoded_padding_preserved':True,'Go_expression_evaluation_or_context_authority':False}
