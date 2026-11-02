"""Keep early intended-positive type/fuel failures distinct from false claims."""
import json
from pathlib import Path
import re
import sys
from check_balanced_report import resource
from entry_audit import read_json,require
from run_proof_pilot import ROOT,sha

RECORD=ROOT/'develop/migrations/go-practical/t01-w03/affix-initial'
PREFIX='develop/probes/go-practical/affix-certificates/'
TYPE_SCOPE='intended positive private candidate rejected at core typing; diagnostic only, not a false semantic claim, W03 completion or public profile'
FUEL_SCOPE='intended positive fold construction exhausted unchanged checker fuel; resource/representation diagnostic, never a false semantic claim or profile bound'

def check():
    rows=0
    for folder,scope in [(RECORD,TYPE_SCOPE),(RECORD/'fuel',FUEL_SCOPE)]:
        metadata=read_json(ROOT,(folder/'scope.json').relative_to(ROOT).as_posix());require(metadata['scope']==scope,'affix initial scope')
        pins=read_json(ROOT,(folder/'sources.json').relative_to(ROOT).as_posix());names=[p['path'] for p in pins];require(len(names)==len(set(names)),'affix initial duplicate source')
        require({PREFIX+p for p in ('Cargo.toml','Cargo.lock','build.rs','src/main.rs','src/affix.rs','src/emit.rs')}.issubset(names),'affix initial source closure')
        for p in pins:
            path=Path(p['path']);require(not path.is_absolute() and '..' not in path.parts,'affix initial source path')
            actual=folder/'source'/p['path'].removeprefix(PREFIX) if p['path'].startswith(PREFIX) else ROOT/path
            require(sha(actual.read_bytes())==p['sha256'],'affix initial source identity')
        binary=read_json(ROOT,(folder/'binaries.json').relative_to(ROOT).as_posix());require(set(binary)=={'rust','go'} and all(re.fullmatch('[0-9a-f]{64}',h) for h in binary.values()),'affix initial binary identity')
        generation=read_json(ROOT,(folder/'generation.json').relative_to(ROOT).as_posix());require(generation['exit_code']==0,'affix initial generation')
        observations=read_json(ROOT,(folder/'observations.json').relative_to(ROOT).as_posix())
        if folder==RECORD:observations+=read_json(ROOT,(folder/'stage-observations.json').relative_to(ROOT).as_posix())
        require(len(observations)==(6 if folder==RECORD else 2),'affix initial diagnostic coverage')
        for row in observations:
            raw=(folder/row['path']).read_bytes();require(row['raw_sha256']==sha(raw),'affix initial certificate identity');require(set(row['observations'])=={'rust','go'},'affix initial independent checker')
            verdicts={m['report']['verdict'] for m in row['observations'].values()};require(len(verdicts)==1,'affix initial checker disagreement')
            accepted=verdicts=={'accepted'}
            for kind,m in row['observations'].items():
                resource(m);r=m['report'];require(m['exit_code']==(0 if accepted else 1),'affix initial child exit')
                if accepted:require(r.get('axiom_count',0 if kind=='go' else None)==0,'affix initial axiom')
                elif kind=='rust':require(json.loads(r['error_detail'])['code']==('CORE_TYPE_MISMATCH' if folder==RECORD else 'CORE_FUEL_EXHAUSTED'),'affix initial failure class')
                else:require((' inferred ' in r['error_detail'] and ' but expected ' in r['error_detail']) if folder==RECORD else r['error_detail']=='core reduction fuel exhausted','affix initial Go failure class')
            if accepted:require(row['observations']['rust']['report']['hashes']==row['observations']['go']['report']['hashes'],'affix initial accepted hashes')
            rows+=1
    require(rows==8,'affix initial total coverage');return rows
if __name__=='__main__':
    try:print(f'PASS: {check()} archived diagnostic rows; no W03 or semantic-negative promotion')
    except (OSError,KeyError,TypeError,ValueError) as error:print(f'Affix initial rejected: {error}',file=sys.stderr);raise SystemExit(1)
