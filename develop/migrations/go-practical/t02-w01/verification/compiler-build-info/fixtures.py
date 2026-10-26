"""Object layout and byte-exact build-info grammar probes, never source stubs."""
from pathlib import Path
import json, struct

GOOD=b'path\tcmd/compile\nbuild\tGOOS=linux\nbuild\tGOARCH=amd64\nbuild\tCGO_ENABLED=0\nbuild\t-compiler=gc\nbuild\t-buildmode=exe\nbuild\tGOAMD64=v1\n'
def varint(n):
 out=bytearray()
 while n>=128:out.append((n&127)|128);n>>=7
 out.append(n);return out
def blob(text=GOOD,version=b'go1.25.0',width=8,big=False,legacy=False):
 mod=b'A'*16+text+b'B'*16
 b=bytearray(0x600);b[:14]=b'\xff Go buildinf:';b[14]=width;b[15]=int(big) if legacy else 2
 if not legacy:
  body=varint(len(version))+version+varint(len(mod))+mod;b[32:32+len(body)]=body
 else:
  fmt=('>' if big else '<')+('Q' if width==8 else 'I')
  def put(at,n):struct.pack_into(fmt,b,at,n)
  put(16,0x1080);put(16+width,0x1090)
  put(0x80,0x1100);put(0x80+width,len(version));put(0x90,0x1200);put(0x90+width,len(mod))
  b[0x100:0x100+len(version)]=version;b[0x200:0x200+len(mod)]=mod
 return b
def object_file(kind,body,wide=True,big=False,section=True):
 raw=bytearray(0x200)+body;endian='>' if big else '<'
 def put(at,fmt,*values):struct.pack_into(endian+fmt,raw,at,*values)
 if kind=='elf':
  raw[:7]=b'\x7fELF'+bytes([2 if wide else 1,2 if big else 1,1]);put(16,'HHI',2,62 if wide else 3,1)
  if wide:
   put(24,'QQQIHHHHHH',0,64,0,0,64,56,1,64,0,0);put(64,'IIQQQQQQ',1,6,0x200,0x1000,0x1000,len(body),len(body),16)
  else:
   put(24,'IIIIHHHHHH',0,52,0,0,52,32,1,40,0,0);put(52,'IIIIIIII',1,0x200,0x1000,0x1000,len(body),len(body),6,16)
 elif kind=='macho':
  head=32 if wide else 28;seg=72 if wide else 56;sec=80 if wide else 68;size=seg+(sec if section else 0)
  put(0,'IIIIIII',0xfeedfacf if wide else 0xfeedface,0x100000c if wide else 7,0,2,1,size,0)
  put(head,'II16s',0x19 if wide else 1,size,b'__DATA')
  put(head+24,'QQQQ' if wide else 'IIII',0x1000,len(body),0x200,len(body))
  put(head+(56 if wide else 40),'IIII',3,3,int(section),0)
  if section:
   put(head+seg,'16s16s',b'__go_buildinfo',b'__DATA');put(head+seg+32,'QQ' if wide else 'II',0x1000,len(body))
 elif kind=='pe':
  endian='<';raw[:2]=b'MZ';put(60,'I',0x80);raw[0x80:0x84]=b'PE\0\0';size=240 if wide else 224
  put(0x84,'HHIIIHH',0x8664 if wide else 0x14c,1,0,0,0,size,0)
  put(0x98,'H',0x20b if wide else 0x10b);put(0x98+(24 if wide else 28),'Q' if wide else 'I',0)
  put(0x98+(108 if wide else 92),'I',16);at=0x98+size
  put(at,'8sIIIIIIHHI',b'.data',len(body),0x1000,len(body),0x200,0,0,0,0,0xc0000040)
 elif kind=='xcoff':
  endian='>';size=24 if wide else 20;symbols=len(raw);raw.extend(bytes(22));put(symbols+18,'I',4)
  if wide:put(0,'HHIQHHI',0x1f7,1,0,symbols,0,0,1)
  else:put(0,'HHIIIHH',0x1df,1,0,symbols,1,0,0)
  put(size,'8s',b'.data');word='Q' if wide else 'I';put(size+8,word*4,0x1000,0x1000,len(body),0x200);put(size+(64 if wide else 36),'I',0x40)
 elif kind=='plan9':
  endian='>';head=40 if wide else 32;put(0,'IIIIIIII',0x8a97 if wide else 0x1eb,0x200-head,len(body),0,0,0,0,0)
 else:raise AssertionError(kind)
 return raw
def generate(out):
 out=Path(out);out.mkdir(exist_ok=False);rows=[]
 def add(name,raw,form,**extra):
  file=name+'.bin';(out/file).write_bytes(raw);rows.append(dict(name=name,file=file,form=form,**extra))
 for kind in ['elf','macho','pe','xcoff','plan9']:
  for wide in [False,True]:
   for big in ([False,True] if kind in ['elf','macho'] else [kind in ['xcoff','plan9']]):
    for legacy in [False,True]:
     name=f'{kind}-{32 if not wide else 64}-{"be" if big else "le"}-{"pointer" if legacy else "inline"}'
     # Plan9 addresses use file offsets, not virtual load addresses. Its inline
     # layout is general; pointer layout is exercised through virtual mappings.
     if kind=='plan9' and legacy:continue
     add(name,object_file(kind,blob(width=8 if wide else 4,big=big,legacy=legacy),wide,big),kind)
 for wide in [False,True]:add(f'macho-fallback-{wide}',object_file('macho',blob(),wide,section=False),'macho-fallback')
 thin=object_file('macho',blob());fat=bytearray(0x1000)+thin;struct.pack_into('>IIIIIII',fat,0,0xcafebabe,1,0x100000c,0,0x1000,len(thin),12);add('fat-first',fat,'fat')
 for name,body in [('empty-version',blob(version=b'')),('wrong-version',blob(version=b'go1.24.0')),('wrong-tool',blob(text=GOOD.replace(b'cmd/compile',b'cmd/link'))),('duplicate-unknown',blob(text=GOOD+b'build\textra=x\nbuild\textra=x\n')),('bad-setting',blob(text=GOOD.replace(b'CGO_ENABLED=0',b'CGO_ENABLED=1'))),('malformed-text',blob(text=b'build\tno-equals\n'))]:
  add(name,object_file('elf',body),'binary-control')
 # Search the actual data range, including beyond its first one-MiB chunk.
 large=bytearray((1<<20)+16)+blob();add('late-magic',object_file('elf',large),'search')
 texts=[b'',GOOD,GOOD+b'build\textra=\n',GOOD+b'build\tGOEXPERIMENT=\n',GOOD+b'build\textra=x\nbuild\textra=y\n',b'path\tfirst\npath\tsecond\n',b'path\tignored-trailing',b'mod\tm\tv\nunknown\tvalue\n=>\tr\tv\ts\n',b'dep\td\tv\ts\npath\tp\nbuild\tk=v\n=>\tr\tv\ts\n',b'=>\tr\tv\ts\n',b'mod\tonly\n',b'dep\td\tv\textra\textra\n',b'mod\tm\tv\n=>\tr\tv\ts\n=>\tx\tv\ts\n']
 for value in [b'plain',b'',b'a=b',b'"a b"',b'`raw\rbytes`',b'"\\x00\\xff\\377"',b'"\\u3042\\U0001f600"',b'"bad\\q"',b'"\\400"',b'"\\ud800"',b'"\\U00110000"',b'"unterminated',b'"ok"trailing',b'a b',b'a\t',b'a\r',b'a`b',b'"\xff\xe3\x81"',b'`\xff\xe3\x81`',b'\xff\x00']:
  for key in [b'key',b'"quoted=key"',b'``',b'bad key',b'"bad\\q"',b'"escaped\\\"key"']:
   texts.append(b'build\t'+key+b'='+value+b'\n')
 texts.extend([b'build\t\n',b'build\t=x\n',b'build\tkey\n',b'build\t"key"\n',b'build\t"key"x=v\n',b'build\tkey=ignored',GOOD.replace(b'GOAMD64=v1',b'GOAMD64=v2'),GOOD.replace(b'GOOS=linux',b'GOOS=darwin'),GOOD+b'build\tGOEXPERIMENT=rangefunc\n'])
 for i,text in enumerate(texts):
  file=f'text-{i:03d}.raw';(out/file).write_bytes(text);rows.append(dict(name=f'text-{i:03d}',file=file,form='text'))
 (out/'inputs.json').write_text(json.dumps(rows,indent=2)+'\n');return rows
