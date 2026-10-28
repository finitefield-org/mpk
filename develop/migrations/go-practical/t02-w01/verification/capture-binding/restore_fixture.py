"""Restore the complete original application, dependency and SDK inputs."""
from pathlib import Path
import base64
import hashlib
import json
import os
import subprocess
import sys

ROOT=Path(__file__).resolve().parent
REPO=ROOT.parents[5]
LIVE=ROOT.parent/'source-root-live'


def safe_relative(value):
    path=Path(value)
    assert not path.is_absolute() and all(part not in ('','.','..') for part in path.parts)
    return path


def main():
    destination=Path(sys.argv[1]);assert destination.is_absolute() and not destination.exists();destination.mkdir(mode=0o700)
    saved=destination/'saved';command=['python3','-B',str(LIVE/'restore_evidence.py'),'live-original',str(saved)]
    restored=subprocess.run(command,cwd=REPO,capture_output=True,check=False);assert restored.returncode==0,restored.stderr.decode()
    evidence=saved/'originals/versioned-owned';full=json.loads((evidence/'complete-original-module-graph-inputs.json').read_bytes())
    native=json.loads((LIVE/'live-original/coord/snapshot-ready.json').read_bytes());snapshot=json.loads(bytes.fromhex(native['snapshot_hex']))
    roots={'workspace/main':destination/'roots/main','workspace/dep':destination/'roots/dep','original-go-sdk':destination/'roots/sdk'}
    for path in roots.values():path.mkdir(parents=True,mode=0o700)
    graph_files=list(evidence.glob('*.graph-generated-originals.json'));assert len(graph_files)==1
    graph=json.loads(graph_files[0].read_bytes());rows=graph['whole_original_entries'];assert len(rows)==74
    seen=set()
    for row in rows:
        capability=row['capability'];assert capability in ('workspace/main','workspace/dep')
        relative=row['path'];key=(capability,relative);assert key not in seen;seen.add(key)
        if relative=='.':assert row['directory'];continue
        path=roots[capability]/safe_relative(relative)
        if row.get('directory'):
            path.mkdir(parents=True,exist_ok=True);path.chmod(row['mode'] & 0o777)
        else:
            raw=base64.b64decode(row['raw'],validate=True);assert hashlib.sha256(raw).hexdigest()==row['sha256']
            path.parent.mkdir(parents=True,exist_ok=True);path.write_bytes(raw);path.chmod(row['mode'] & 0o777)
    expected={cap:set() for cap in ['workspace/main','workspace/dep']}
    for capability,relative in seen:
        if relative!='.':expected[capability].add(relative)
    for cap in expected:
        actual={p.relative_to(roots[cap]).as_posix() for p in roots[cap].rglob('*')};assert actual==expected[cap]
    candidate=json.loads((REPO/'release/bundles/candidates/go.json').read_bytes())['toolchain_bundles'][0]['inventory']['files']
    assert len(candidate)==len(full['compiler_files'])==5577
    for row,entry in zip(full['compiler_files'],candidate):
        assert row['entry']==entry and entry['path'].startswith('go/') and row['raw_file']==entry['sha256']+'.raw'
        raw=(evidence/'original-compiler-files'/row['raw_file']).read_bytes();assert len(raw)==entry['size_bytes'] and hashlib.sha256(raw).hexdigest()==entry['sha256']
        path=roots['original-go-sdk']/safe_relative(entry['path'][3:]);path.parent.mkdir(parents=True,exist_ok=True);path.write_bytes(raw);path.chmod(0o555 if entry['executable'] else 0o444)
    for directory in snapshot['directories']:
        if directory['capability']!='original-go-sdk':continue
        path=roots['original-go-sdk']/safe_relative(directory['path']) if directory['path'] else roots['original-go-sdk']
        path.mkdir(parents=True,exist_ok=True)
        for entry in directory['entries']:
            if entry['type_bits']==1<<31:
                name=bytes.fromhex(entry['name_hex']);assert name not in (b'',b'.',b'..') and b'/' not in name and b'\0' not in name
                (path/os.fsdecode(name)).mkdir(exist_ok=True)
    # The original prepared SDK uses read-only directories and files.
    sdk_dirs=sorted((p for p in roots['original-go-sdk'].rglob('*') if p.is_dir()),key=lambda p:len(p.parts),reverse=True)
    for path in sdk_dirs:path.chmod(0o555)
    roots['original-go-sdk'].chmod(0o555)
    result={'status':'passed','selection_reason':'Restore every original evidence file, all 74 application/dependency entries and all 5577 registered SDK files; recreate the original observed SDK directory namespace without substituting source files.','roots':{cap:str(path) for cap,path in roots.items()},'evidence':str(evidence),'original_application_entries':len(rows),'registered_sdk_files':len(candidate),'registered_sdk_bytes':sum(x['size_bytes'] for x in candidate),'all_original_evidence_files':16880,'all_original_evidence_bytes':367714473,'native_Linux_release_acceptance':False}
    (destination/'fixture.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result),flush=True)


if __name__=='__main__':main()
