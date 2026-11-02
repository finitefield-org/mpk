"""Complete retained corpus plus original ELF section/decompression observations."""
from pathlib import Path
import hashlib, json, struct, types, zlib
ROOT=Path(__file__).resolve().parent
OLD=ROOT.parent/'compiler-build-info/fixtures'
SDK=Path('/private/tmp/mpk-go125-20261025-download/go')

def raw_frame(data, *, last=True, checksum=False, window=None, dictionary=0, dictionary_value=0, fcs=None, block_kind=0):
 n=len(data) if fcs is None else fcs
 single=window is None
 width=1 if single and n<256 else 2 if 256<=n<65792 else 4 if n<(1<<32) else 8
 if not single and fcs is None:width=0
 flag={0:0,1:0,2:1,4:2,8:3}[width]
 descriptor=(flag<<6)|(32 if single else 0)|(4 if checksum else 0)|dictionary
 out=bytearray(b'\x28\xb5\x2f\xfd'+bytes([descriptor]))
 if not single:out.append(window)
 if dictionary:out.extend(dictionary_value.to_bytes(1<<(dictionary-1),'little'))
 if width:out.extend((n-(256 if width==2 else 0)).to_bytes(width,'little'))
 parts=[data[i:i+(128<<10)] for i in range(0,len(data),128<<10)] or [b'']
 for i,part in enumerate(parts):
  header=(len(part)<<3)|(block_kind<<1)|int(last and i==len(parts)-1)
  out.extend(header.to_bytes(3,'little'));out.extend(part if block_kind!=1 else part[:1] or b'\x00')
 return bytes(out)

def elf(body, payload, logical, *, wide=True,big=False,compressed=1,flags=0x800,kind=3,base=0,fallback=False,extra=False,extended=False):
 raw=bytearray(old.object_file('elf',body,wide,big));endian='>' if big else '<';word='Q' if wide else 'I';ss=64 if wide else 40
 def put(at,fmt,*v):struct.pack_into(endian+fmt,raw,at,*v)
 # A named section must actually select the read range unless fallback requested.
 put((64+4 if wide else 52+24),'I',6 if fallback else 5)
 table=(len(raw)+15)&~15;count=0xff01 if extended else 4 if extra else 3;name_index=count-1 if extended else 1
 raw.extend(bytes(table+ss*count-len(raw)))
 if wide:put(40,'Q',table);put(58,'HHH',ss,0 if extended else count,0xffff if extended else name_index)
 else:put(32,'I',table);put(46,'HHH',ss,0 if extended else count,0xffff if extended else name_index)
 if extended:
  put(table+(32 if wide else 20),word,count);put(table+(40 if wide else 24),'I',name_index)
 at=len(raw);header=struct.pack(endian+('IIQQ' if wide else 'III'),compressed,0,logical,1) if wide and flags&0x800 else struct.pack(endian+'III',compressed,logical,1) if flags&0x800 else b''
 raw.extend(header+payload)
 def section(index,name,typ,fl,addr,offset,size):
  p=table+ss*index;put(p,'II',name,typ);put(p+8,word*4,fl,addr,offset,size)
 section(name_index,base+1,kind,flags,0,at,len(header+payload))
 section(2,base+11,1,0,0x1000,0x200,len(body))
 if extra:section(3,base+25,1,0,0,0,0)
 return raw,dict(table=table,ss=ss,name_index=name_index,payload=at,wide=wide,big=big)

def main():
 global old
 source=ROOT.parent/'compiler-build-info/fixtures.py';old=types.ModuleType('unchanged_original_build_info_generator');exec(compile(source.read_bytes(),str(source),'exec'),old.__dict__)
 out=ROOT/'fixtures';out.mkdir(exist_ok=False)
 rows=json.loads((OLD/'inputs.json').read_bytes())
 for row in rows:
  raw=(OLD/row['file']).read_bytes();(out/row['file']).write_bytes(raw)
 def add(name,raw,want=None,**meta):
  path=name+'.bin';(out/path).write_bytes(raw);row=dict(name=name,file=path,form='binary-control',**meta)
  if want is not None:row['expect_accepted']=want
  rows.append(row)
 names=b'\x00.shstrtab\x00.go.buildinfo\x00.suffix\x00'
 original=json.loads((ROOT/'zstd/observations/complete-original-zstandard.json').read_bytes())['complete_original_streams']
 assert len(original)==3
 for wide in [False,True]:
  for big in [False,True]:
   label=f'{32 if not wide else 64}-{"be" if big else "le"}'
   body=old.blob(width=8 if wide else 4,big=big)
   def wrap(payload=names,logical=len(names),**kw):return elf(body,payload,logical,wide=wide,big=big,**kw)
   for level in [0,1,6,9]:
    raw,_=wrap(zlib.compress(names,level));add(f'zlib-{label}-level-{level}',raw,True)
   zipped=zlib.compress(names)
   for label2,payload,want in [('bad-adler',zipped[:-1]+bytes([zipped[-1]^1]),True),('no-adler',zipped[:-4],True),('short-deflate',zipped[:5],False),('bad-header',b'\x00\x00'+zipped[2:],False),('surplus',zipped+b'complete original surplus',True),('empty-dictionary',b'\x78\x20\x00\x00\x00\x01'+zipped[2:],True),('nonempty-dictionary',b'\x78\x20\x00\x00\x00\x02'+zipped[2:],False)]:
    raw,_=wrap(payload);add(f'zlib-{label}-{label2}',raw,want)
   for delta in [-1,1,100]:
    raw,_=wrap(zipped,logical=len(names)+delta);add(f'zlib-{label}-logical-{delta}',raw,False if delta>0 else True)
   for flags,kind,comp,want in [(0,3,0,True),(0x802,3,1,False),(0x800,1,1,False),(0x800,8,1,False),(0x800,3,99,False)]:
    raw,_=wrap(names if flags==0 else zipped,flags=flags,kind=kind,compressed=comp);add(f'section-{label}-{flags}-{kind}-{comp}',raw,want)
   for window in [None,0,0x68,0xff]:
    for dictionary in [0,1,2,3]:
     stream=raw_frame(names,window=window,dictionary=dictionary)
     raw,_=wrap(stream,compressed=2);add(f'zstd-{label}-window-{window}-dictionary-{dictionary}',raw,True)
   for label2,stream,want in [
    ('nonzero-dict',raw_frame(names,dictionary=1,dictionary_value=1),False),
    ('rle',raw_frame(bytes(len(names)),block_kind=1),True),
    ('missing-last-allowed-prefix',raw_frame(names,last=False,window=0x68),True),
    ('truncated',raw_frame(names)[:-1],False),
    ('reserved-descriptor',b'\x28\xb5\x2f\xfd\x28'+bytes([len(names)])+raw_frame(names)[6:],False),
    ('unused-descriptor',b'\x28\xb5\x2f\xfd\x30'+bytes([len(names)])+raw_frame(names)[6:],True),
    ('too-few-frame-bytes',raw_frame(names,fcs=len(names)+1),False),
    ('too-many-frame-bytes',raw_frame(names,fcs=len(names)-1),False),
    ('trailing-garbage',raw_frame(names)+b'complete original unread suffix',True),
    ('bad-checksum',raw_frame(names,checksum=True)+bytes(4),False),
    ('skippable',b'\x50\x2a\x4d\x18'+(8).to_bytes(4,'little')+b'original'+raw_frame(names),True),
    ('concatenated',raw_frame(names[:11])+raw_frame(names[11:]),True),
   ]:
    raw,_=wrap(stream,compressed=2,fallback=label2=='rle');add(f'zstd-{label}-{label2}',raw,want)
   for row in original:
    original_raw=bytes.fromhex(row['raw_hex']);decoded=bytes.fromhex(row['decoded_hex']);stream=original_raw+raw_frame(names)
    raw,_=wrap(stream,logical=len(decoded)+len(names),base=len(decoded),compressed=2)
    add('zstd-original-'+label+'-'+row['name'],raw,True,complete_original_zstandard_sha256=row['raw_sha256'],complete_original_decoded_sha256=row['decoded_sha256'])
   raw,meta=wrap(zipped,extra=True)
   for case,where,value,width in [('later-name-index',meta['table']+meta['ss']*3,0xffffffff,4),('zero-section-offset',40 if wide else 32,0,8 if wide else 4),('bad-name-table-type',meta['table']+meta['ss']+4,1,4),('short-compression-header',meta['table']+meta['ss']+(32 if wide else 20),1,8 if wide else 4)]:
    changed=bytearray(raw);struct.pack_into(('>' if big else '<')+('Q' if width==8 else 'I'),changed,where,value);add(f'elf-{label}-{case}',changed,False)
   for extended in [False,True]:
    raw,meta=wrap(zipped,extended=extended);add(f'elf-{label}-extended-{extended}',raw,True)
    if extended:
     for case,where,value in [('short-extended-count',meta['table']+(32 if wide else 20),10),('short-extended-names',meta['table']+(40 if wide else 24),1),('wrong-initial-type',meta['table']+4,1)]:
      changed=bytearray(raw);struct.pack_into(('>' if big else '<')+('Q' if wide and case=='short-extended-count' else 'I'),changed,where,value);add(f'elf-{label}-{case}',changed,False)
 # Multi-megabyte complete data: exercise repeated streaming, not a new limit.
 long=b'original auxiliary name/'*180000+names
 for comp,stream in [(1,zlib.compress(long,9)),(2,raw_frame(long,window=0x68))]:
  raw,_=elf(old.blob(),stream,len(long),base=len(long)-len(names),compressed=comp)
  add(f'complete-long-section-{comp}',raw,True,complete_expanded_bytes=len(long))
 (out/'inputs.json').write_text(json.dumps(rows,indent=2)+'\n')
 (out/'generation.json').write_text(json.dumps({'retained_original_fixture_cases':len(json.loads((OLD/'inputs.json').read_bytes())),'generated_cases':len(rows),'complete_sdk_zstandard_streams':3,'arbitrary_limits_added':False},indent=2)+'\n')
 print(json.dumps({'generated_cases':len(rows),'complete_original_streams':3}))
if __name__=='__main__':main()
