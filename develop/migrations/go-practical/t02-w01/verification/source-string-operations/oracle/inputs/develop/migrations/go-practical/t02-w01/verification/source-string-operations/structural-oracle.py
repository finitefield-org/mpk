"""Independent source-type layouts and source-bound captured-operand operations.

Reconstruct the original source carriers and the complete new operand-product
definitions and public theorem statements independently of the Rust producer.
No source expression execution or context authority is inferred.
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


def pt(types):
 if not types:return f.boolean()
 if len(types)==1:return types[0]
 n=len(types)//2
 return f.pi('_layout_tag',f.boolean(),f.rec_type(pt(types[:n]),pt(types[n:]),f.var('_layout_tag')))
def make(types,values):
 if not types:return f.bit(False)
 if len(types)==1:return values[0]
 n=len(types)//2;tag=f.var('_layout_tag')
 motive=f.lam('_layout_tag',f.boolean(),f.rec_type(pt(types[:n]),pt(types[n:]),tag))
 return f.lam('_layout_tag',f.boolean(),f.app(f.const('Std.Bool.cases'),motive,make(types[:n],values[:n]),make(types[n:],values[n:]),tag))
def get(types,value,index):
 if len(types)==1:return value
 n=len(types)//2
 projected=f.app(f.const('Mpk.Go.Practical.Product.Left' if index<n else 'Mpk.Go.Practical.Product.Right'),pt(types[:n]),pt(types[n:]),value)
 return get(types[:n] if index<n else types[n:],projected,index if index<n else index-n)

def inspect(path,meta=None):
 raw,module,table,term=decode(path);meta=meta or json.loads(path.with_suffix('.sources.json').read_bytes());report=json.loads(path.with_suffix('.rust.json').read_bytes())
 assert report['verdict']=='accepted' and report['axiom_count']==0 and report['module']==module==meta['module']
 assert report['hashes']['certificate']==hashlib.sha256(b'MPK-MODULE-CERT-0.1\0'+raw).hexdigest() and report['declaration_count']==len(table)
 parent=meta['source_module'];prior=meta['prior_module'];keys={k['id']:k for k in meta['types']};under=dict(meta['underlying']);cases=meta['cases'];checked=set();recipes=0
 def check(n,ty,body=None,law=False):
  d=table[n];assert d[1]==(2 if law else 1),(n,'kind');assert term(d[2])==m.frozen_ast(ty),(n,'type')
  if body is not None:assert term(d[3])==m.frozen_ast(body),(n,'body')
  checked.add(n)
 def c(n,*xs):return f.app(f.const(n),*xs)
 def own(n,*xs):return c(module+'.'+n,*xs)
 def old(n,*xs):return c(prior+'.'+n,*xs)
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
  elif kind=='array':expected=(f.boolean(),f.bit(False)) if k['length']==0 else f.vector((k['length']-1).bit_length(),*pair(k['element']))
  elif kind=='slice':expected=product([(f.boolean(),f.bit(True)),(f.nat(),f.const('Std.Nat.zero')),f.vector(4,*product([f.word(8),f.word(64)])),f.word(64),f.word(64),f.word(64)])
  else:assert kind=='struct';expected=product([pair(field['type_id']) for field in k['fields']])
  ty,z=expected;check(parent+'.T'+tid+'.Carrier',{'kind':'sort0'},ty);check(parent+'.T'+tid+'.Zero',ty,z);check(parent+'.T'+tid+'.ZeroRecipe',f.eq(ty,pair(tid)[1],z),f.refl(ty,z),True);recipes+=3
 if not any(k.get('name')=='error' for k in keys.values()):
  assert not cases and parent.startswith('Mpk.Go.Practical.SourceTypeLayout.') and not any(name.startswith(parent+'.T') and name.endswith('.Carrier') and name.split('.T')[-1].split('.')[0] not in keys for name in table)
  assert not any(name.startswith(parent+'.Nil') for name in table)
 types=[pair(i)[0] for i in meta['operand_types']];result=pair(meta['result'])[0];p=f.var('p');names=['s','i'];values=[f.var(names[i]) for i in range(len(types))];args=list(zip(names,types));input_type=pt(types);capture=own('Capture',*values);pa=[('p',input_type)]
 check(module+'.Operands',{'kind':'sort0'},input_type)
 length=meta['kind']=='Length';assert length or meta['kind']=='Index';assert len(types)==(1 if length else 2)
 def generic(n,*xs):return c('Mpk.Go.Practical.SourceOperation.'+n,*xs)
 if length:check(module+'.Capture',f.bind('pi',args,input_type),f.bind('lam',args,make(types,values)))
 else:
  A,B,C=[f.var(n) for n in ['A','B','C']];sort={'kind':'sort0'};gtypes=[A,B];ginput=pt(gtypes);gfty=f.pi('_a',A,f.pi('_b',B,C))
  ga=[('A',sort),('B',sort),('a',A),('b',B)]
  check('Mpk.Go.Practical.SourceOperation.Pair',f.bind('pi',ga,ginput),f.bind('lam',ga,make(gtypes,[f.var('a'),f.var('b')])))
  ga=[('A',sort),('B',sort),('C',sort),('f',gfty),('p',ginput)]
  check('Mpk.Go.Practical.SourceOperation.Uncurry',f.bind('pi',ga,C),f.bind('lam',ga,f.app(f.var('f'),get(gtypes,f.var('p'),0),get(gtypes,f.var('p'),1))))
  ga=[('A',sort),('B',sort),('C',sort),('f',gfty),('a',A),('b',B)]
  left=generic('Uncurry',A,B,C,f.var('f'),generic('Pair',A,B,f.var('a'),f.var('b')));right=f.app(f.var('f'),f.var('a'),f.var('b'))
  check('Mpk.Go.Practical.SourceOperation.PairApply',f.bind('pi',ga,f.eq(C,left,right)),f.bind('lam',ga,f.refl(C,right)),True)
  function=generic('Pair',*types);fty=f.bind('pi',args,input_type)
  check(module+'.Capture',fty,function);check(module+'.CaptureFunctionDefinition',f.eq(fty,own('Capture'),function),f.refl(fty,function),True)
 operands=[]
 for i,t in enumerate(types):
  name='Operand'+str(i);check(module+'.'+name,f.bind('pi',pa,t),f.bind('lam',pa,get(types,p,i)));check(module+'.CaptureOperand'+str(i),f.bind('pi',args,f.eq(t,own(name,capture),values[i])),law=True);operands.append(own(name,p))
 action='Len' if length else 'Read';strings=prior.rsplit('.GoLength.' if length else '.Index.',1)[0];predicate=c(strings+'.Valid',*operands) if length else old('Defined',*operands)
 if length:check(module+'.Result',f.bind('pi',pa,result),f.bind('lam',pa,old(action,*operands)))
 else:
  uncurry=generic('Uncurry',*types,result,old('Read'));fty=f.bind('pi',pa,result)
  check(module+'.Result',fty,uncurry);check(module+'.ResultFunctionDefinition',f.eq(fty,own('Result'),uncurry),f.refl(fty,uncurry),True)
 check(module+'.Domain',f.bind('pi',pa,{'kind':'sort0'}),f.bind('lam',pa,predicate))
 if length:check(module+'.CaptureResult',f.bind('pi',args,f.eq(result,own('Result',capture),old(action,*values))),law=True)
 else:check(module+'.CaptureResult',f.bind('pi',args,f.eq(result,f.app(uncurry,generic('Pair',*types,*values)),old('Read',*values))),f.bind('lam',args,generic('PairApply',*types,result,old('Read'),*values)),True)
 guard=pa+[('domain',own('Domain',p))]
 check(module+'.DomainWitness',f.bind('pi',guard,predicate),law=True);check(module+'.DomainOfWitness',f.bind('pi',pa+[('witness',predicate)],own('Domain',p)),law=True)
 width=64 if length else 8;word=f.word(width)[0];body=old('ToWord',own('Result',p)) if length else own('Result',p)
 if length:
  check(module+'.ResultWord',f.bind('pi',pa,word),f.bind('lam',pa,body))
  check(module+'.ResultWordDefinition',f.bind('pi',pa,f.eq(word,own('ResultWord',p),old('ToWord',old('Len',*operands)))),law=True)
 else:
  fty=f.bind('pi',pa,word);check(module+'.ResultWord',fty,own('Result'))
  check(module+'.ResultWordDefinition',f.eq(fty,own('ResultWord'),own('Result')),f.refl(fty,own('Result')),True)
 if length:
  intty=f.pi('_part',f.boolean(),f.nat())
  check(module+'.DomainInteger',f.bind('pi',guard,f.eq(intty,old('AsInteger',*operands),c('Mpk.Go.Practical.Int.FromNat',c(strings+'.Size',*operands)))),law=True)
 else:
  s,i=operands
  check(module+'.DomainNonnegative',f.bind('pi',guard,old('Nonnegative',i)),law=True)
  check(module+'.DomainWithin',f.bind('pi',guard,f.eq(f.boolean(),c('Mpk.Go.Practical.Nat.Less',old('Natural',i),c(strings+'.Size',s)),f.bit(True))),law=True)
  check(module+'.DomainInteger',f.bind('pi',guard,c('Mpk.Go.Practical.Int.Equivalent',old('Integer',i),c('Mpk.Go.Practical.Int.FromNat',old('Natural',i)))),law=True)
 actual={name for name in table if name.startswith(module+'.')};assert actual=={name for name in checked if name.startswith(module+'.')},'unexpected source-operation export'
 return {'certificate':str(path),'source_recipe_members':recipes,'operation_members':len(actual),'original_case_count':len(cases),'original_type_count':len(keys),'module':module,'source_expression_execution_or_context_authority':False}
