"""Independently replay the aggregate producer's declared Rust-only recipe."""
import json
import re
from pathlib import Path
import affix_build_sources as predecessor
from run_proof_pilot import HERE,TARGET

PACKAGE=HERE/'aggregate-certificates'
SCOPE='deterministic aggregate Rust candidate inputs; frozen predecessor files unchanged; no Go source rewriting or production binding'
inventory=predecessor.inventory
once=predecessor.once
def expected_sources():
 result={p:raw.decode() for p,raw in predecessor.expected_sources().items()}
 main=result['main.rs'].replace('affix-generate','aggregate-generate').replace('affix-trace','aggregate-trace')
 main=once(main,'assert_eq!(args.len(), 5, "aggregate-generate REPO_ROOT OUTPUT CAPACITY");','assert_eq!(args.len(), 6, "aggregate-generate REPO_ROOT OUTPUT BYTE_BOUND STORAGE_BOUND");')
 main=once(main,'word::generate_stringops(\n            Path::new(&args[2]),\n            Path::new(&args[3]),\n            args[4].parse().unwrap(),','word::generate_stringops(\n            Path::new(&args[2]),\n            Path::new(&args[3]),\n            args[4].parse::<usize>().unwrap().max(args[5].parse().unwrap()),')
 result['main.rs']=main
 access=predecessor.ACCESS
 result[access]=once(result[access],'assert!([0, 1, 31, 32, 33, 255, 256, 257].contains(&count));','assert!(count <= 257);')
 result[access]=once(result[access],'{PREFIX}.StringAffixCapacity{count}','{PREFIX}.AggregateCapacity{count}')
 recipe=(PACKAGE/'build.rs').read_text()
 payloads=[json.loads(s) for s in re.findall(r'text\.push_str\(\s*("(?:\\.|[^"\\])*")\s*\);',recipe)]
 if len(payloads)!=5:raise ValueError('aggregate recipe payload coverage')
 def payload(key):
  found=[p for p in payloads if key in p]
  if len(found)!=1:raise ValueError('aggregate recipe payload identity')
  return found[0]
 result[access]+=payload('fn aggregate_validity(')
 result[access.removesuffix('.rs')+'/validity.rs']+=payload('fn aggregate_install(')
 parent=predecessor.STRINGOPS.removesuffix('.rs')+'/affix'
 result[parent+'.rs']=once(result[parent+'.rs'],'mod emit;','#[allow(dead_code)]\nmod emit;\nmod aggregate;')
 result[parent+'.rs']=once(result[parent+'.rs'],'emit::generate(base, output, count);','aggregate::generate(base, output, count);')
 result[parent+'/emit.rs']=once(result[parent+'/emit.rs'],'fn proof_call(suf: bool, d: usize, f: &str) -> Typed','pub(super) fn proof_call(suf: bool, d: usize, f: &str) -> Typed')
 result[parent+'/aggregate.rs']=(PACKAGE/'src/aggregate.rs').read_text()
 return {p:raw.encode() for p,raw in result.items()}
def check_sources(record):
 if record['scope']!=SCOPE or record['files']!=inventory(expected_sources()):raise ValueError('aggregate generated source closure/identity')
 return len(record['files'])
def observe_generated(cargo_output):
 events=[json.loads(s) for s in cargo_output.splitlines() if s.startswith('{')]
 builds=[e for e in events if e.get('reason')=='build-script-executed' and 'go-practical-aggregate-probe@' in e.get('package_id','')]
 if len(builds)!=1:raise ValueError('aggregate Cargo build-script identity')
 output=Path(builds[0]['out_dir']).resolve()/'predecessor'
 if not output.is_relative_to(TARGET.resolve()) or not output.is_dir():raise ValueError('aggregate generated output path')
 observed={p.relative_to(output).as_posix():p.read_bytes() for p in output.rglob('*') if p.is_file()}
 record={'scope':SCOPE,'files':inventory(observed)};check_sources(record);return record
