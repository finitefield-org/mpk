"""Classify literal scan positions by the actual closed certificate sections.

Raw findings remain visible. No proof byte, scanner rule or exclusion changes;
this scoped semantic audit does not claim the strict repository gate passed.
"""
from pathlib import Path
import codecs,hashlib,importlib.util,json,runpy,sys,types
ROOT=Path(__file__).resolve().parent;REPO=Path.cwd()
source=REPO/'scripts/check-no-active-gir.sh'
code=source.read_text().split("<<'PY'\n",1)[1].rsplit('\nPY',1)[0].rsplit('raise SystemExit(main())',1)[0]
module=types.ModuleType('source_error_representation_binary_review');module.__file__=str(source);sys.modules[module.__name__]=module;sys.argv=[str(source),str(REPO),'strict'];exec(compile(code,str(source),'exec'),module.__dict__)
matchers=module.load_matchers();assert len(matchers)==127
spec=importlib.util.spec_from_file_location('complete_certificate_section_reader',ROOT.parent/'source-error-layouts/structural-oracle.py');reader=importlib.util.module_from_spec(spec);spec.loader.exec_module(reader)


def inspect(path):
    raw=Path(path).read_bytes();assert raw[:7]==b'MPKCERT'
    section=reader.Sections(raw);section.at=7;strings=[];binary=[]
    def string(role):
        n=section.number();start=section.at;end=start+n
        assert end<=len(raw);value=raw[start:end].decode('utf-8');section.at=end
        strings.append({'role':role,'start':start,'end':end,'value':value});return value
    assert string('format')=='MPK-CERT-0.1' and string('core')=='MPK-Core-0.1'
    module_name=string('module');assert section.number()==0
    for _ in range(section.number()):string('name')
    for role,read in [('level',section.level),('term',section.term)]:
        for i in range(section.number()):
            start=section.at;read();binary.append((start,section.at,role,i))
    assert section.number()==0 # no separate proof table
    declarations=section.number()
    for i in range(declarations):
        start=section.at;section.declaration();binary.append((start,section.at,'declaration',i))
    assert section.number()==0 # no theory certificate payloads
    def hash_bytes(role,index):
        start=section.at;section.at+=32;assert section.at<=len(raw);binary.append((start,section.at,role,index))
    for i in range(section.number()):
        section.number();section.number();hash_bytes('export-hash',i)
    assert section.number()==0 # zero-axiom entry list
    for i in range(section.number()):
        string('dependency-declaration-name');hash_bytes('dependency-hash',i)
        assert section.number()==0 and section.number()==0
    assert [section.number() for _ in range(5)]==[0]*5
    assert section.byte()==0 # no source-manifest payload
    for i in range(3):hash_bytes('certificate-trailer-hash',i)
    assert section.at==len(raw)
    semantic_findings=module.detect(str(path),'\n'.join(s['value'] for s in strings),matchers)
    assert not semantic_findings
    text=raw.decode('utf-8',errors='ignore');decoder=codecs.getincrementaldecoder('utf-8')('ignore');positions=[];rebuilt=[]
    for i,b in enumerate(raw):
        value=decoder.decode(bytes([b]),False)
        assert len(value)<=1
        if value:positions.append(i-len(value.encode('utf-8'))+1);rebuilt.append(value)
    assert not decoder.decode(b'',True) and ''.join(rebuilt)==text
    observations=[]
    literal=module.detect(str(path),text,matchers)
    for finding in literal:
        matcher=next(m for m in matchers if m['id']==finding.matcher_id)
        assert matcher['scope']=='exact-global-token'
        for match in module.token_pattern(matcher['token']).finditer(text):
            points=positions[match.start():match.end()];assert points
            assert all(not any(s['start']<=p<s['end'] for s in strings) for p in points)
            sections=[next(({'role':role,'index':index,'start':a,'end':b} for a,b,role,index in binary if a<=p<b),None) for p in points]
            assert all(s is not None for s in sections)
            observations.append({'matcher_id':finding.matcher_id,'decoded_character_offset':match.start(),'decoded_bytes_hex':match.group().encode().hex(),'original_byte_positions':points,'raw_spanning_bytes_hex':raw[points[0]:points[-1]+1].hex(),'containing_nontext_sections':sections,'intersects_any_semantic_string':False})
    assert observations and len(observations)==len(literal)
    return {'raw_sha256':hashlib.sha256(raw).hexdigest(),'bytes':len(raw),'module':module_name,'declarations':declarations,'all_semantic_strings_scanned_with_same127_rules':True,'semantic_string_count':len(strings),'semantic_findings':[],'literal_nontext_findings':observations,'complete_file_parsed_to_EOF':True}

if __name__=='__main__':
    path=ROOT/'first-large/certificates/seventy-cases-call0-9ff960b8b522e8613d9385d27e82b59b032f79a26342c5306c162c2b5dc51183.mpcert'
    rust=json.loads(path.with_suffix('.rust.json').read_bytes());assert rust['verdict']=='accepted' and rust['axiom_count']==0
    go=json.loads((ROOT/'paired-large/result.json').read_bytes());paired=next(r for r in go['cases'] if r['certificate']['sha256']==hashlib.sha256(path.read_bytes()).hexdigest());assert paired['go_observation']['verdict']=='accepted' and paired['go_observation']['hashes']==rust['hashes']
    out=inspect(path);out.update(status='reviewed_nontext_binary_data_matches',original_certificate=str(path),both_cores_accept_exact_unchanged_full_stream=True,strict_whole_repository_gate_passed=False,no_scanner_rule_or_exclusion_changed=True,no_certificate_bytes_rewritten_or_archived_to_hide_findings=True,scope='Literal findings remain recorded. Only actual encoded integer/hash bytes outside every certificate string are classified as nontext data. Strict repository semantic handling remains a final-gate obligation.')
    (ROOT/'certificate-binary-data-review.json').write_text(json.dumps(out,indent=2)+'\n');print(json.dumps({'status':out['status'],'literal_data_matches':len(out['literal_nontext_findings']),'semantic_strings':out['semantic_string_count'],'semantic_findings':0}))
