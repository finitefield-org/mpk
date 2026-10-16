"""Replay the declared private extension recipe without executing Rust code."""
import json
from pathlib import Path
import re
from run_proof_pilot import HERE,ROOT,TARGET,sha

PACKAGE=HERE/'affix-certificates'
STRINGOPS='word/balanced/signed/bytes/bounds/slices/access/validity/reslice/stringops.rs'
ACCESS='word/balanced/signed/bytes/bounds/slices/access.rs'
COVERAGE='word/balanced/signed/bytes/bounds/coverage.rs'
SCOPE='deterministic generated private producer inputs; predecessor source files unchanged; no Go package extraction or production binding'

def once(text,old,new):
    if text.count(old)!=1:raise ValueError('affix predecessor recipe anchor')
    return text.replace(old,new,1)
def expected_sources():
    recipe=(PACKAGE/'build.rs').read_text()
    payloads=[json.loads(s) for s in re.findall(r'text\.push_str\(\s*("(?:\\.|[^"\\])*")\s*\);',recipe)]
    if len(payloads)!=3:raise ValueError('affix extension payload coverage')
    def payload(key):
        matches=[x for x in payloads if key in x]
        if len(matches)!=1:raise ValueError('ambiguous affix extension payload')
        return matches[0]
    result={}
    for p in sorted((HERE/'certificates/src').rglob('*.rs')):
        relative=p.relative_to(HERE/'certificates/src').as_posix();text=p.read_text()
        if relative=='main.rs':
            text=''.join(('//'+line[3:] if line.startswith('//!') else line)+'\n' for line in text.splitlines())
            builder=ROOT/'develop/migrations/csharp-03/probes/boolean-proof-elimination-logs/producer/builder.rs'
            text=once(text,'"../../../../migrations/csharp-03/probes/boolean-proof-elimination-logs/producer/builder.rs"',json.dumps(str(builder),ensure_ascii=False))
            text=text.replace('stringop-generate','affix-generate').replace('stringop-trace','affix-trace')
        elif relative==ACCESS:
            text=once(text,'{PREFIX}.StringOperationCapacity{count}','{PREFIX}.StringAffixCapacity{count}')+payload('fn affix_embedding(')
        elif relative==COVERAGE:text+=payload('fn install_affix_embedding(')
        elif relative==STRINGOPS:
            text=once(text,'mod emit;','mod affix;')
            text=once(text,'pub(super) fn install(p: &mut Producer, count: usize, trace: Option<&Path>)','fn install_predecessor(p: &mut Producer, count: usize, trace: Option<&Path>)')
            text=once(text,'emit::generate(base, output, count);','affix::emit(base, output, count);')+payload('install_predecessor(p, count, trace);')
        result[relative]=text.encode()
    parent=STRINGOPS.removesuffix('.rs')
    result[parent+'/affix.rs']=(PACKAGE/'src/affix.rs').read_bytes()
    result[parent+'/affix/emit.rs']=(PACKAGE/'src/emit.rs').read_bytes()
    return result
def inventory(files):return [{'path':p,'sha256':sha(raw)} for p,raw in sorted(files.items())]
def check_sources(record):
    if record['scope']!=SCOPE or record['files']!=inventory(expected_sources()):raise ValueError('affix generated source identity/closure')
    return len(record['files'])
def observe_generated(cargo_output):
    events=[json.loads(line) for line in cargo_output.splitlines() if line.startswith('{')]
    builds=[e for e in events if e.get('reason')=='build-script-executed' and 'go-practical-affix-probe@' in e.get('package_id','')]
    if len(builds)!=1:raise ValueError('affix Cargo build-script identity')
    output=Path(builds[0]['out_dir']).resolve()/'predecessor'
    if not output.is_relative_to(TARGET.resolve()) or not output.is_dir():raise ValueError('affix Cargo output path')
    files={p.relative_to(output).as_posix():p.read_bytes() for p in sorted(output.rglob('*')) if p.is_file()}
    record={'scope':SCOPE,'files':inventory(files)};check_sources(record);return record
