"""Registered shared JSON limits for structured bodies; no64-depth pilot cap."""
import hashlib
import json
import re
from run_proof_pilot import ROOT

CAPS={r['id']:r['maximum'] for r in json.loads((ROOT/'develop/migrations/go-practical/t01-w04/structural-limits/limits.json').read_bytes())['counters']}
SAFE=(1<<53)-1


def need(ok,message):
    if not ok:raise ValueError(message)


def canonical(value):
    chunks=[];size=0;stack=[('value',value,0)]
    def emit(raw):
        nonlocal size
        need(len(raw)<=CAPS['shared.canonical_json_bytes']-size,'canonical body JSON byte limit');size+=len(raw);chunks.append(raw)
    def string(s):
        try:raw=s.encode('utf-8')
        except UnicodeError as e:raise ValueError('Unicode scalar JSON string') from e
        need(len(raw)<=CAPS['shared.string_bytes'],'decoded body JSON string limit')
        return json.dumps(s,ensure_ascii=False,separators=(',',':')).encode('utf-8')
    while stack:
        kind,v,depth=stack.pop()
        if kind=='raw':emit(v);continue
        if v is None:emit(b'null')
        elif type(v) is bool:emit(b'true' if v else b'false')
        elif type(v) is int:need(-SAFE<=v<=SAFE,'safe integral metadata; full words remain strings');emit(str(v).encode())
        elif type(v) is str:emit(string(v))
        elif type(v) in {list,dict}:
            need(depth<CAPS['shared.json_nesting'],'body JSON container nesting')
            depth+=1
            if type(v) is list:
                emit(b'[');stack.append(('raw',b']',depth))
                for i,child in reversed(list(enumerate(v))):
                    if i<len(v)-1:stack.append(('raw',b',',depth))
                    stack.append(('value',child,depth))
            else:
                need(all(type(k) is str for k in v),'JSON string keys')
                try:keys=sorted(v,key=lambda k:k.encode('utf-16-be'))
                except UnicodeError as e:raise ValueError('Unicode scalar object name') from e
                emit(b'{');stack.append(('raw',b'}',depth))
                for i,k in reversed(list(enumerate(keys))):
                    if i<len(keys)-1:stack.append(('raw',b',',depth))
                    stack.append(('value',v[k],depth));stack.append(('raw',b':',depth));stack.append(('raw',string(k),depth))
        else:raise ValueError('closed narrow JSON value')
    return b''.join(chunks)


def parse(raw):
    need(type(raw) is bytes and len(raw)<=CAPS['shared.input_json_bytes'],'raw body JSON byte limit')
    def pairs(rows):
        out={}
        for k,v in rows:need(k not in out,'duplicate body JSON member');out[k]=v
        return out
    def number(_):raise ValueError('narrow integral JSON only')
    try:value=json.loads(raw.decode('utf-8'),object_pairs_hook=pairs,parse_float=number,parse_constant=number)
    except (UnicodeError,json.JSONDecodeError,RecursionError) as e:raise ValueError('strict body UTF8/JSON') from e
    canonical(value);return value


def digest(domain,value):
    need(type(domain) is str and re.fullmatch(r'[A-Z][A-Z0-9-]*-[0-9]+\.[0-9]+',domain),'exact ASCII hash domain')
    return hashlib.sha256(domain.encode('ascii')+b'\0'+canonical(value)).hexdigest()


def seal(schema,domain,body,field='body_hash'):
    value={'schema':schema,**body};value[field]=digest(domain,value);return value
