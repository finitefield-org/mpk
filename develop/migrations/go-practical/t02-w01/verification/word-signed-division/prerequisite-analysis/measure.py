"""Read sealed ordinary tables for dependency planning, never checker authority."""
from pathlib import Path
import datetime,hashlib,json
root=Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
def seal(p):
 p=Path(p);b=p.read_bytes();return {'path':str(p),'sha256':hashlib.sha256(b).hexdigest(),'bytes':len(b)}
class Tables:
 def __init__(self,record):
  self.record=record;assert seal(record['path'])==record
  self.raw=Path(record['path']).read_bytes();assert self.raw[:7]==b'MPKCERT';self.pos=7
  assert self.string()=='MPK-CERT-0.1'and self.string()=='MPK-Core-0.1';self.module=self.string();assert self.uint()==0
  self.names=[self.string()for _ in range(self.uint())]
  self.levels=[]
  for _ in range(self.uint()):
   tag=self.byte();assert tag in range(4);self.levels.append((tag,[self.uint()for _ in range([0,1,2,1][tag])]))
  self.terms=[]
  for _ in range(self.uint()):
   tag=self.byte();assert tag in range(7)
   if tag in [0,1]:args=[self.uint()]
   elif tag in [2,3]:args=[self.uint(),[self.uint()for _ in range(self.uint())]]
   else:args=[self.uint()for _ in range(3 if tag==6 else 2)]
   self.terms.append((tag,args))
  assert self.uint()==0;self.declarations=[]
  for _ in range(self.uint()):
   name=self.uint();tag=self.byte();assert tag in [1,2,3,4,5]
   args=[self.uint()for _ in range(1 if tag==3 else 2)]
   if tag in [1,4,5]:args.append(self.byte())
   self.declarations.append((self.names[name],tag,args))
  self.index={name:i for i,(name,_,_)in enumerate(self.declarations)};assert len(self.index)==len(self.declarations)
 def byte(self):
  v=self.raw[self.pos];self.pos+=1;return v
 def uint(self):
  value=shift=0
  for _ in range(10):
   b=self.byte();value|=(b&127)<<shift
   if b<128:return value
   shift+=7
  raise AssertionError('bounded minimal-varint planning input')
 def string(self):
  n=self.uint();s=self.raw[self.pos:self.pos+n];assert len(s)==n;self.pos+=n;return s.decode()
 def closure(self,roots):
  pending=[self.index[n]for n in roots];pending.extend(i for i,(_,tag,_)in enumerate(self.declarations)if tag in [3,4,5]);decls=set();terms=set()
  while pending:
   i=pending.pop()
   if i in decls:continue
   decls.add(i);_,tag,args=self.declarations[i]
   nodes=args[:2]if tag in [1,2]else args[:1]
   if tag in [4,5]:pending.append(args[1])
   nodes=list(nodes)
   while nodes:
    n=nodes.pop()
    if n in terms:continue
    terms.add(n);kind,a=self.terms[n]
    if kind==2:pending.append(a[0])
    elif kind==3:nodes.extend([a[0],*a[1]])
    elif kind in [4,5,6]:nodes.extend(a)
  return {self.declarations[i][0]for i in decls},terms
 def fingerprints(self):
  def h(v):return hashlib.sha256(json.dumps(v,separators=(',',':'),ensure_ascii=False).encode()).hexdigest()
  levels=[]
  for tag,a in self.levels:
   levels.append(h([tag,[]if tag==0 else[self.names[a[0]]]if tag==3 else[levels[i]for i in a]]))
  terms=[]
  for tag,a in self.terms:
   if tag==0:v=[levels[a[0]]]
   elif tag==1:v=a
   elif tag==2:v=[self.declarations[a[0]][0],[levels[i]for i in a[1]]]
   elif tag==3:v=[terms[a[0]],[terms[i]for i in a[1]]]
   else:v=[terms[i]for i in a]
   terms.append(h([tag,v]))
  result={}
  for name,tag,a in self.declarations:
   if tag in [1,2]:v=[terms[a[0]],terms[a[1]],*a[2:]]
   elif tag in [4,5]:v=[terms[a[0]],self.declarations[a[1]][0],a[2]]
   else:v=[terms[a[0]]]
   result[name]=h([tag,v])
  return result
sources=[]
for name in ['word-division','word-signed-bounds']:
 p=root.parent.parent/name/'verification.json';v=json.loads(p.read_text());sources.append(Tables(v['fresh_component']['certificate']))
a,b=sources;fa,fb=a.fingerprints(),b.fingerprints();conflicts=[n for n in fa.keys()&fb.keys()if fa[n]!=fb[n]];assert not conflicts,conflicts
aroots=[];broots=[]
for w in [8,16,32,64]:
 aroots.extend(f'Mpk.Go.Practical.Word.Division.W{w}.{n}'for n in ['UnsignedDivisionConservation','UnsignedRemainderBound'])
 broots.extend(f'Mpk.Go.Practical.Word.SignedMagnitude.W{w}.{n}'for n in ['SignedMagnitudeEquivalent','MagnitudeDivideDefined','NegationValueSum'])
 broots.extend(f'Mpk.Go.Practical.Word.SignedBounds.W{w}.{n}'for n in ['MagnitudeHalfBound','MagnitudeHalfIsMinimum','MagnitudeSignIsMinimum','UnsignedHalfLess'])
 broots.extend(f'Mpk.Go.Practical.Word.Numeric.W{w}.{n}'for n in ['SignedPositive','SignedNegative'])
for n in ['Make','Negate','FromNat','Mul','Add','Equivalent','PositiveOfMake','NegativeOfMake','PositiveOfFromNat','NegativeOfFromNat','PositiveOfNegate','NegativeOfNegate','PositiveOfAdd','NegativeOfAdd','PositiveOfMul','NegativeOfMul']:
 broots.append('Mpk.Go.Practical.Int.'+n)
ac,at=a.closure(aroots);bc,bt=b.closure(broots)
scenarios={}
for label,excluded,replacement in [
 ('half_bound_and_unsigned_threshold_only',['MagnitudeHalfIsMinimum','MagnitudeSignIsMinimum'],[]),
 ('half_bound_and_forward_minimum',['MagnitudeHalfIsMinimum','MagnitudeSignIsMinimum'],['MagnitudeHalfReflect']),
 ('half_bound_only',['MagnitudeHalfIsMinimum','MagnitudeSignIsMinimum','UnsignedHalfLess'],[]),
 ('unconditional_signed_conservation',['MagnitudeHalfBound','MagnitudeHalfIsMinimum','MagnitudeSignIsMinimum','UnsignedHalfLess'],[]),
]:
 roots=[n for n in broots if not any(n.endswith('.'+suffix)for suffix in excluded)]
 roots.extend(f'Mpk.Go.Practical.Word.SignedBounds.W{w}.{n}'for w in [8,16,32,64]for n in replacement)
 names,nodes=b.closure(roots);scenarios[label]={'signed_declarations':len(names),'union':len(ac|names),'remaining':4096-len(ac|names),'roots':roots}
record={'schema':'mpk.go_practical.signed_division_dependency_planning.v1','status':'measured_structural_inputs_not_checker_authority','observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'sealed_original_certificates':[c.record for c in sources],'parser_sources':[seal('crates/mpk-cert/src/decode.rs'),seal('crates/mpk-cert/src/binary_tags.rs')],'whole_declarations':[len(c.declarations)for c in sources],'whole_name_union':len(a.index.keys()|b.index.keys()),'shared_named_declarations':len(fa.keys()&fb.keys()),'shared_named_type_body_proof_level_family_fingerprint_conflicts':conflicts,'unsigned_final_roots':aroots,'signed_magnitude_bound_roots':broots,'unsigned_closed_declarations':len(ac),'unsigned_closed_terms':len(at),'signed_closed_declarations':len(bc),'signed_closed_terms':len(bt),'selected_name_union':len(ac|bc),'selected_common_names':len(ac&bc),'remaining_declaration_slots_before_new_source_defs_and_laws':4096-len(ac|bc),'selected_names':sorted(ac|bc),'root_scenarios':scenarios,'scope':'A small independent reader traverses only original sealed ordinary tables to plan compiled-in roots and compare normalized named-DAG fingerprints. The full canonical bytes were accepted in prior checkpoint evidence. This measurement neither creates/accepts a certificate nor grants an import/source/registry/context/application proof. Actual new producer and every selected proof still require current both-checker acceptance within unchanged budgets. No omitted suffix is treated as newly validated.'}
(root/'result.json').write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({k:record[k]for k in ['whole_declarations','whole_name_union','shared_named_declarations','unsigned_closed_declarations','signed_closed_declarations','selected_name_union','remaining_declaration_slots_before_new_source_defs_and_laws']}))

print(json.dumps({k:{x:v[x]for x in ['signed_declarations','union','remaining']}for k,v in scenarios.items()}))
