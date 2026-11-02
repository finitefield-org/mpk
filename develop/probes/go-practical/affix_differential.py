"""Original-library and literal-body observations versus independent byte/bit loops."""
import json
import random
from byte_differential import equal_word,less_word
from order_differential import adder
from run_proof_pilot import json_bytes,sha,timed

CAPACITIES=(0,1,31,32,33,255,256,257)
ARTIFACTS=('affix-input.json','affix-output.json','affix-source-binding.json')
MASK=(1<<64)-1
SCOPE='original Go strings.HasPrefix/HasSuffix and exact literal bodies versus independent full64 and byte loops; no production adapter, whole-package admission or process resource theorem'
WITNESS_SCOPE='bounded-source, erased-match, optional-index-panic and unguarded-total-model witnesses remain distinct; original HasPrefix/HasSuffix never panic in this corpus'
BINDING_SCOPE='exact selected AST/source identities; private model correspondence only, no production adapter or whole-package admission'

def depth(n):return max(n-1,0).bit_length()
def padding(n):return 1<<depth(n)
def families(n):return ['range','length','binding','canonical','empty','matched-length']+(['longer','matched-index','matched-read','coverage'] if n else [])
def label(n,kind,f):
    if f=='length':return 'extra-byte'
    if f=='binding':return 'one-bit-comparison' if n else 'flipped-result'
    if f=='canonical':return 'prefix-suffix-confusion' if n>=2 else 'flipped-result'
    if f=='empty':return 'false-empty'
    if f=='matched-length':return 'erased-match' if n else 'flipped-result'
    return {'longer':'erased-length-guard','matched-index':'erased-index-guard','matched-read':'flipped-byte-equality','coverage':'flipped-word'}.get(f,'flipped-result')

def row(n,s=b'',p=b'',i=0):
    def backing(value,salt):return (value+bytes((j*73+salt)&255 for j in range(len(value),max(padding(n),len(value))))).hex()
    return dict(bound=n,source=backing(s,191),pattern=backing(p,239),source_length=len(s),pattern_length=len(p),index=str(i))
def guard_erasure_row(n):
    r=row(n,p=b'\x00');r['source']='00'*padding(n);return r
def inputs():
    rng=random.Random(0x4D504B4146464958);out=[]
    for n in CAPACITIES:
        values=[b'',b'\x00',b'\x01',b'\x80',b'\xff',b'a\x00x',b'\xc3\xa9',b'\xe3\x81\x82',b'\xc3(',bytes(rng.randrange(256) for _ in range(n)),b'\x00'*n,b'\xff'*n]
        values=list(dict.fromkeys(v for v in values if len(v)<=n))
        for s in values:
            patterns=list(dict.fromkeys(values+[s[:len(s)//2],s[len(s)//2:]]))
            for p in patterns:
                for i in dict.fromkeys((-1,0,len(p)-1,len(p),-(1<<63),(1<<63)-1)):out.append(row(n,s,p,i))
        out+=[row(n),row(n,b'\x00'*(padding(n)+1),b'\x00'*padding(n)+b'\xff',padding(n))]
        if n:out+=[row(n,b'\x00',b'\x00'),row(n,b'\x00',b'\x00',1),row(n,b'\x80',b'\x00'),row(n,p=b'\x00'),guard_erasure_row(n)]
        if n>=2:out.append(row(n,b'\x00\xff',b'\x00'))
    return list({json.dumps(r,sort_keys=True):r for r in out}.values())
def signed(x):return x-(1<<64) if x>>63 else x
def add(x,y):return adder(64,x&MASK,y&MASK,False)[0]
def sub(x,y):return adder(64,x&MASK,(y&MASK)^MASK,True)[0]
def read(a,i):return (True,a[i]) if i>=0 and less_word(64,i,len(a)) else (False,-1)
def finite_relation(r,kind,*,guard=True,one_bit=False):
    s,p=bytes.fromhex(r['source']),bytes.fromhex(r['pattern']);l,n=r['source_length'],r['pattern_length'];size=padding(r['bound'])
    if guard and less_word(64,l,n):return False
    start=sub(l,n) if kind=='suffix' else 0
    for j in range(size):
        if less_word(64,j,n):
            a,b=s[add(start,j)&(size-1)],p[j]
            if not equal_word(1 if one_bit else 8,a,b):return False
    return True
def model(r):
    s,p=bytes.fromhex(r['source']),bytes.fromhex(r['pattern']);s=[s[j] for j in range(r['source_length'])];p=[p[j] for j in range(r['pattern_length'])];i=int(r['index']);l,n=len(s),len(p)
    enough=not less_word(64,l,n);prefix=suffix=enough
    if enough:
        for j in range(n):prefix=prefix and equal_word(8,s[j],p[j]);suffix=suffix and equal_word(8,s[l-n+j],p[j])
    start=sub(l,n);mapped=add(start,i);pr,pv=read(p,i);sr,sv=read(s,i);tr,tv=read(s,signed(mapped))
    return dict(prefix=prefix,suffix=suffix,prefix_body=prefix,suffix_body=suffix,masked_prefix=prefix,masked_suffix=suffix,prefix_panicked=False,suffix_panicked=False,unconditional_prefix_panic=not enough,unconditional_suffix_panic=not enough,pattern_read=pr,pattern_value=pv,prefix_read=sr,prefix_value=sv,suffix_read=tr,suffix_value=tv,suffix_start=str(start),suffix_mapped=str(mapped),input_bound=l<=r['bound'] and n<=r['bound'],input_unchanged=True,query_covered=(i&MASK)==((i&MASK)&(padding(r['bound'])-1)))
def compare(cases,outputs):
    if cases!=inputs() or len(cases)!=len(outputs):raise ValueError('affix input coverage')
    keys=('prefix_matches','suffix_matches','empty_patterns','longer_patterns','different_prefix_suffix','matched_prefix_reads','matched_suffix_reads','high_bit_rejections','nul_patterns','invalid_utf8_inputs','multibyte_inputs','outside_bound','finite_projection_counterexamples','uncovered_source_reads')
    counts=dict.fromkeys(keys,0)
    for r,a in zip(cases,outputs):
        expected=model(r)
        if not isinstance(a,dict) or set(a)!=set(expected) or any(type(a[k]) is not type(v) for k,v in expected.items()):raise ValueError('affix observation field/type mismatch')
        if a!=expected:raise ValueError('Go/bit-loop affix mismatch')
        for kind in ('prefix','suffix'):
            rel=finite_relation(r,kind)
            if a['input_bound'] and rel!=a[kind]:raise ValueError('bounded affix relation disagrees with original Go')
            if a[kind] and a['pattern_read'] and (not a[kind+'_read'] or a[kind+'_value']!=a['pattern_value']):raise ValueError('matched actual index/full-byte law failed')
            counts['finite_projection_counterexamples']+=not a['input_bound'] and rel!=a[kind]
        counts['prefix_matches']+=a['prefix'];counts['suffix_matches']+=a['suffix'];counts['empty_patterns']+=r['pattern_length']==0;counts['longer_patterns']+=r['pattern_length']>r['source_length'];counts['different_prefix_suffix']+=a['prefix']!=a['suffix'];counts['matched_prefix_reads']+=a['prefix'] and a['pattern_read'];counts['matched_suffix_reads']+=a['suffix'] and a['pattern_read'];counts['high_bit_rejections']+=not a['prefix'] and a['input_bound'] and finite_relation(r,'prefix',one_bit=True);counts['nul_patterns']+=0 in bytes.fromhex(r['pattern'])[:r['pattern_length']]
        value=bytes.fromhex(r['source'])[:r['source_length']]
        try:text=value.decode('utf-8');counts['multibyte_inputs']+=len(text)<len(value)
        except UnicodeDecodeError:counts['invalid_utf8_inputs']+=1
        counts['outside_bound']+=not a['input_bound'];counts['uncovered_source_reads']+=not a['query_covered'] and a['prefix_read']
    if not all(counts.values()):raise ValueError('lost affix boundary/nonvacuity coverage: '+repr([k for k,v in counts.items() if not v]))
    return counts

def negative_witnesses(cases,outputs):
    out=[]
    for n in CAPACITIES:
        for kind in ('prefix','suffix'):
            for f in families(n):
                domain='bounded_source';r=row(n);actual,claimed=True,False
                if f=='length':actual,claimed=0,1
                elif f=='binding' and n:r=row(n,b'\x80',b'\x00');actual=False;claimed=True
                elif f=='canonical' and n>=2:r=row(n,b'\x00\xff',b'\x00');actual,claimed=kind=='prefix',kind=='suffix'
                elif f=='matched-length' and n:r=row(n,p=b'\x00');domain='erased_source_success';actual,claimed=False,True
                elif f=='longer':r=guard_erasure_row(n);domain='bounded_source_vs_unguarded_total_model';actual,claimed=False,True
                elif f=='matched-index':r=row(n,b'\x00',b'\x00',1);domain='optional_index_panic';actual,claimed=False,True
                elif f=='matched-read':r=row(n,b'\x00',b'\x00');actual,claimed=True,False
                elif f=='coverage':r=row(n,b'\x00',b'\x00');actual,claimed=0,MASK
                pos=cases.index(r);a=outputs[pos]
                if not a['input_bound'] or a['prefix_panicked'] or a['suffix_panicked']:raise ValueError('invalid affix witness source domain')
                if f=='binding' and n and (a[kind]!=actual or finite_relation(r,kind,one_bit=True)!=claimed):raise ValueError('lost complete-byte binding witness')
                if f=='canonical' and n>=2 and (a[kind]!=actual or a['prefix' if kind=='suffix' else 'suffix']!=claimed):raise ValueError('lost prefix/suffix offset witness')
                if f=='longer' and (a[kind] or not a['unconditional_'+kind+'_panic'] or not finite_relation(r,kind,guard=False)):raise ValueError('lost total-model guard-erasure witness')
                if f=='matched-length' and n and a[kind]:raise ValueError('erased match witness still succeeds')
                if f=='matched-index' and (not a[kind] or a['pattern_read'] or a[kind+'_read']):raise ValueError('lost optional-index panic witness')
                if f in ('matched-read','coverage') and (not a[kind] or not a['pattern_read'] or not a[kind+'_read']):raise ValueError('vacuous matched-byte/coverage witness')
                if type(actual) is not type(claimed) or actual==claimed:raise ValueError('affix false witness type/value')
                out.append(dict(case_id=f'affix-{n}-{kind}-{f}-{label(n,kind,f)}',observation_index=pos,domain=domain,actual=actual,claimed=claimed))
    return out

def check_binding(binding):
    if binding['scope']!=BINDING_SCOPE or binding['registered'] is not False or binding['go_version']!='go1.25.0' or binding['host']!='darwin/arm64':raise ValueError('affix source binding scope/compiler')
    expected=[]
    for package,path,digest in [('strings','src/strings/strings.go','ac1d74f15027b6c703ee13fd08f10b3ad337e61a79b874d403a10f3bee5bb4fb'),('internal/stringslite','src/internal/stringslite/strings.go','764f3227e30300d0227b8c971aa718032232aa16370535476dd8e7e830fa4985')]:
        for method,param in [('HasPrefix','prefix'),('HasSuffix','suffix')]:
            body=f'stringslite.{method}(s, {param})' if package=='strings' else (f'len(s) >= len({param}) && '+(f's[:len({param})] == {param}' if param=='prefix' else f's[len(s)-len({param}):] == {param}'))
            declaration=f'func {method}(s, {param} string) bool {{\n\treturn {body}\n}}'
            expected.append(dict(package=package,symbol=package+'.'+method,path=path,file_sha256=digest,declaration=declaration,declaration_sha256=sha(declaration.encode())))
    if json_bytes(binding['functions'])!=json_bytes(expected):raise ValueError('affix exact selected body/identity')
    return 4
def run(exe,directory,goroot):
    cases=inputs();path=directory/ARTIFACTS[0];path.write_bytes(json_bytes(cases));m=timed([exe,path])
    if m['exit_code']:raise RuntimeError('actual Go affix observer failed')
    raw=m.pop('stdout').encode();(directory/ARTIFACTS[1]).write_bytes(raw);outputs=json.loads(raw);counts=compare(cases,outputs)
    b=timed([exe,'--binding',goroot]);binding=b.pop('stdout').encode();(directory/ARTIFACTS[2]).write_bytes(binding)
    if b['exit_code'] or check_binding(json.loads(binding))!=4:raise ValueError('affix source binding observer')
    return dict(scope=SCOPE,cases=len(cases),outcomes=counts,input_sha256=sha(path.read_bytes()),output_sha256=sha(raw),binding_sha256=sha(binding),go_binary_sha256=sha(exe.read_bytes()),go_measurement=m,binding_measurement=b,witness_scope=WITNESS_SCOPE,false_claim_witnesses=negative_witnesses(cases,outputs))
def check(report,directory):
    a,b,c=((directory/f).read_bytes() for f in ARTIFACTS)
    if report['scope']!=SCOPE or report['witness_scope']!=WITNESS_SCOPE or report['input_sha256']!=sha(a) or report['output_sha256']!=sha(b) or report['binding_sha256']!=sha(c):raise ValueError('affix observation scope/byte identity')
    cases,outputs=json.loads(a),json.loads(b);counts=compare(cases,outputs);check_binding(json.loads(c))
    if report['cases']!=len(cases) or report['outcomes']!=counts or a!=json_bytes(inputs()) or json_bytes(report['false_claim_witnesses'])!=json_bytes(negative_witnesses(cases,outputs)):raise ValueError('affix witness/coverage identity')
    return len(cases)
