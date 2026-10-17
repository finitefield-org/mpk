"""API envelope copy of the frozen narrow JSON algorithm.

Source artifacts retain their own 256MiB/768-container limits. Only the API
wrapper permits the existing 1MiB overhead and four additional containers.
"""
import hashlib
import json
import re

MAX_BYTES=269484032
MAX_DEPTH=772
MAX_STRING=1048576
SAFE=(1<<53)-1
INTEGER=re.compile(r'-?(?:0|[1-9][0-9]*)')


def need(ok,message):
    if not ok:raise ValueError(message)


def canonical(value):
    chunks=[];size=0;stack=[('value',value,0)]
    def emit(raw):
        nonlocal size
        need(len(raw)<=MAX_BYTES-size,'VC canonical byte budget');size+=len(raw);chunks.append(raw)
    def string(s):
        try:raw=s.encode('utf-8')
        except UnicodeError as e:raise ValueError('Unicode scalar JSON string') from e
        need(len(raw)<=MAX_STRING,'VC decoded string budget');return json.dumps(s,ensure_ascii=False,separators=(',',':')).encode()
    while stack:
        kind,v,depth=stack.pop()
        if kind=='raw':emit(v);continue
        if v is None:emit(b'null')
        elif type(v) is bool:emit(b'true' if v else b'false')
        elif type(v) is int:need(-SAFE<=v<=SAFE,'safe integral metadata');emit(str(v).encode())
        elif type(v) is str:emit(string(v))
        elif type(v) in {list,dict}:
            need(depth<MAX_DEPTH,'VC JSON container budget');depth+=1
            if type(v) is list:
                emit(b'[');stack.append(('raw',b']',depth))
                for i,x in reversed(list(enumerate(v))):
                    if i<len(v)-1:stack.append(('raw',b',',depth))
                    stack.append(('value',x,depth))
            else:
                need(all(type(k) is str for k in v),'JSON string keys')
                try:keys=sorted(v,key=lambda k:k.encode('utf-16-be'))
                except UnicodeError as e:raise ValueError('Unicode scalar object key') from e
                emit(b'{');stack.append(('raw',b'}',depth))
                for i,k in reversed(list(enumerate(keys))):
                    if i<len(keys)-1:stack.append(('raw',b',',depth))
                    stack.extend([('value',v[k],depth),('raw',b':',depth),('raw',string(k),depth)])
        else:raise ValueError('closed narrow VC JSON value')
    return b''.join(chunks)


def parse(raw):
    """Iterative grammar parser,including empty-container and duplicate checks."""
    need(type(raw) is bytes and len(raw)<=MAX_BYTES,'VC input byte budget')
    try:s=raw.decode('utf-8')
    except UnicodeError as e:raise ValueError('strict UTF8') from e
    pos=0;frames=[];root=None;has_root=False
    def attach(value):
        nonlocal root,has_root
        if not frames:need(not has_root,'one root');root=value;has_root=True;return
        f=frames[-1]
        if f[0]=='array':need(f[2] in ('first','value'),'array value position');f[1].append(value)
        else:
            need(f[2]=='value','object value position');key=f[3];need(key not in f[1],'duplicate VC JSON member');f[1][key]=value
        f[2]='comma'
    while True:
        while pos<len(s) and s[pos] in ' \t\r\n':pos+=1
        if not frames and has_root:need(pos==len(s),'no trailing JSON tokens');break
        need(pos<len(s),'complete JSON value');c=s[pos]
        if frames:
            f=frames[-1];state=f[2]
            if state=='comma':
                closing=']' if f[0]=='array' else '}'
                if c==closing:frames.pop();pos+=1;continue
                need(c==',','JSON comma');f[2]='value' if f[0]=='array' else 'key';pos+=1;continue
            if f[0]=='object' and state in ('first','key'):
                if state=='first' and c=='}':frames.pop();pos+=1;continue
                need(c=='"','object key')
                try:key,pos=json.decoder.scanstring(s,pos+1,True)
                except (ValueError,UnicodeError) as e:raise ValueError('JSON string') from e
                f[3]=key;f[2]='colon';continue
            if f[0]=='object' and state=='colon':need(c==':','JSON colon');f[2]='value';pos+=1;continue
            if f[0]=='array' and state=='first' and c==']':frames.pop();pos+=1;continue
        if c in '[{':
            need(len(frames)<MAX_DEPTH,'VC JSON container budget');v=[] if c=='[' else {};attach(v)
            frames.append(['array' if c=='[' else 'object',v,'first',None]);pos+=1
        elif c=='"':
            try:v,pos=json.decoder.scanstring(s,pos+1,True)
            except (ValueError,UnicodeError) as e:raise ValueError('JSON string') from e
            attach(v)
        elif c in '-0123456789':
            m=INTEGER.match(s,pos);need(m is not None,'JSON integral metadata');token=m.group();pos+=len(token);attach(int(token))
        else:
            values=[(t,v) for t,v in (('true',True),('false',False),('null',None)) if s.startswith(t,pos)]
            need(len(values)==1,'narrow JSON token');token,v=values[0];pos+=len(token);attach(v)
    canonical(root);return root


def digest(domain,value):
    need(type(domain) is str and re.fullmatch(r'[A-Z][A-Z0-9-]*-[0-9]+\.[0-9]+',domain),'ASCII hash domain')
    return hashlib.sha256(domain.encode()+b'\0'+canonical(value)).hexdigest()


def clone(value):return parse(canonical(value))


def seal(schema,domain,body,field):
    value={'schema':schema,**body};value[field]=digest(domain,value);return value
