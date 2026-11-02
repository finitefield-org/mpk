"""Closed release graph/hash and non-promotable candidate controls."""
from release_contracts import *


def evaluate(native):
    m=ReleaseModel(native);out=[]
    for f,values in m.docs.items():
        for i,v in enumerate(values if type(values)is list else [values]):
            cases=[('same',v,True)]
            for k in FIELDS[f]:
                x=dict(v);del x[k];cases.append(('missing-'+k,x,False))
            x=dict(v);x['accepted']=True;cases.append(('extra',x,False))
            x=dict(v);x['schema']=SHARED[f][0].replace('.v3','.v2');cases.append(('reserved_v2',x,False))
            if f in HASH_FIELDS:
                x=dict(v);x[HASH_FIELDS[f]]='0'*64;cases.append(('wrong_hash',x,False))
                x=dict(v);x['accepted']=True;h=HASH_FIELDS[f];x[h]=digest(domain(f),{k:y for k,y in x.items()if k!=h});cases.append(('rehashed_promotion',x,False))
            for label,x,expected in cases:
                try:m.validate(f,x,i);ok=True
                except ValueError:ok=False
                need(ok==expected,'release vector '+f+label)
                out.append({'id':f+str(i)+'-'+label,'expected':expected,'classification':'metadata_only'if ok else'configuration_rejection'})
            for label,raw in [('no_LF',canonical(v)),('two_LF',wire(v)+b'\n'),('BOM',b'\xef\xbb\xbf'+wire(v)),('space',b' '+wire(v))]:
                try:m.import_json(f,raw,i);ok=True
                except ValueError:ok=False
                need(not ok,'release framing');out.append({'id':f+str(i)+'-'+label,'expected':False,'classification':'transport_rejection'})
    return m,out


def hashes(docs):
    rows=[]
    for f,values in docs.items():
        for i,v in enumerate(values if type(values)is list else[values]):
            h=HASH_FIELDS.get(f);pre={k:y for k,y in v.items()if k!=h};raw=canonical(pre)
            rows.append({'id':f+'-'+str(i),'family':f,'index':i,'accepted':True,'canonical_sha256':sha(raw),'bytes':len(raw),
                **({'domain':domain(f),'digest':v[h]}if h else{})})
    return rows


def hash_queries(docs,rows):
    out=[]
    for r in rows:
        f=r['family'];v=docs[f][r['index']]if type(docs[f])is list else docs[f]
        raw=canonical({k:y for k,y in v.items()if k!=HASH_FIELDS.get(f)})
        out.append({'mode':'hash','id':r['id'],'chunks':[raw[i:i+32768].hex()for i in range(0,len(raw),32768)],
            **({'domain':r['domain']}if'domain'in r else{})})
    return out


def documents(native):
    m,cases=evaluate(native)
    return {'schema.json':catalogue(),'graph.json':m.docs,'vectors.json':cases,'hash-vectors.json':hashes(m.docs)}
