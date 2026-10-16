#!/usr/bin/env python3
"""Measure guarded full-byte substring and bounded concatenation ingredients."""
import argparse
from concurrent.futures import ThreadPoolExecutor,as_completed
import json
import os
from pathlib import Path
import platform
import shutil
import sys
import tempfile
import threading

import stringop_differential as native
import run_prefix_probes as shared
from run_proof_pilot import DEFAULT_GOROOT,HERE,ROOT,TARGET,checked,json_bytes,observe,semantic_expectation,sha,timed,verify_host_toolchain

RECORD=ROOT/'develop/migrations/go-practical/t01-w03/stringop'
COUNTS=native.CAPACITIES
GENERATE_COMMAND='stringop-generate'
INDEX_SCHEMA='mpk.go_practical.string_operation_pilot.v0'
REPORT_SCHEMA='mpk.go_practical.stringop_report.v0'
ATTEMPT_SCHEMA='mpk.go_practical.stringop_attempt.v0'
STATUS='substring and concat ingredients measured; W03 remains incomplete'
SCOPE='ordinary signed substring guards, full-byte source/result reads and input-tail normalization; concat derives carry from actual lengths and result nonnegativity from an explicit output bound; full64 index reconstruction from semantic bounds; production capture/effect/operation binding and remaining string/control/application closure stay pending'
PENDING=['production Go capture/model/effect/operation bindings','prefix/suffix, remaining byte/control/aggregate and complete application composition','aggregate boundary and resource freeze']
CASE_SCOPE='ordinary signed substring guards, full-byte reads and input-tail normalization; concat length carry, explicit output bound, source-index lifting and full-byte/tail preservation; production source/effect/operation binding and remaining string laws stay pending'
BOUNDARY_SCOPE='semantic byte bounds 0/1/31/32/33/255/256/257 are private workloads; power-of-two padding is not admission; exceeding concat bound can succeed in Go; child ceilings are not frozen budgets'
PARALLEL_GROUPS=3
EXTRA_FILES=native.ARTIFACTS
EXTRA_REPORT_KEY='stringop_observations'

def proof_sources():
 result={r['path']:r['sha256'] for r in shared.proof_sources()}
 for p in (Path(__file__),HERE/'check_stringop_report.py',HERE/'stringop_differential.py',HERE/'stringop_reference.go',HERE/'order_differential.py',HERE/'byte_differential.py'):
  result[p.relative_to(ROOT).as_posix()]=sha(p.read_bytes())
 return [{'path':p,'sha256':h} for p,h in sorted(result.items())]

def depth(n):return max(n-1,0).bit_length()
def padding(n):return 1<<depth(n)
def declaration_count(n):return {0:534,1:534,31:571,32:571,33:564,255:550,256:550,257:571}[n]
def expected_cases():return [(n,f,m) for n in COUNTS for f in native.families(n) for m in range(2)]
def identifier(n,family,mutation):return f"stringop-{n}-{family}-{'valid' if mutation==0 else native.label(n,family)}"

def verify_case(case,observations):
 semantic_expectation(case,observations,declarations=declaration_count(case['capacity']),module=f"Probe.GoPractical.Word.StringOperationCapacity{case['capacity']}")
 if case['expected']=='accepted':return 'accepted_zero_axiom'
 rust=json.loads(observations['rust']['report']['error_detail'])['code'];go=observations['go']['report']['error_detail']
 if rust=='CORE_TYPE_MISMATCH' and go.startswith('term ') and ' inferred ' in go and ' but expected ' in go:return 'core_type_mismatch'
 raise RuntimeError('stringop proof did not fail at the intended type boundary; resource failures never establish a false claim')

def measure_extra(directory,env):
 exe=directory/'stringop-observer'
 checked([str(DEFAULT_GOROOT/'bin/go'),'build','-trimpath','-buildvcs=false','-o',str(exe),str(HERE/'stringop_reference.go')],env=env)
 return native.run(exe,directory)

def check_retained(report):
 from check_stringop_report import check
 return check(report)

def main():
 parser=argparse.ArgumentParser(description=__doc__);mode=parser.add_mutually_exclusive_group(required=True);mode.add_argument('--write',action='store_true');mode.add_argument('--check',action='store_true');args=parser.parse_args()
 if platform.system()!='Darwin' or platform.machine()!='arm64':raise RuntimeError('pinned host and resource units require darwin/arm64')
 pin=json.loads((HERE/'host-toolchain.json').read_text());inventory=verify_host_toolchain(DEFAULT_GOROOT,pin)
 checked([shutil.which('cargo'),'build','--release','--offline','--locked','--manifest-path',str(HERE/'certificates/Cargo.toml')],env={**os.environ,'CARGO_TARGET_DIR':str(TARGET)})
 producer=TARGET/'release/go-practical-proof-probe';before=proof_sources()
 with tempfile.TemporaryDirectory(prefix='mpk-go-stringop-',dir='/private/tmp') as temporary:
  temp=Path(temporary);reference=temp/'mpk-checker-ref'
  env={**os.environ,'GOENV':'off','GOWORK':'off','GOTOOLCHAIN':'local','GOPROXY':'off','GOTELEMETRY':'off','CGO_ENABLED':'0','GOCACHE':'/private/tmp/mpk-go-practical-checker-cache'}
  checked([str(DEFAULT_GOROOT/'bin/go'),'build','-trimpath','-buildvcs=false','-o',str(reference),'./cmd/mpk-checker-ref'],cwd=ROOT/'go-tools/mpk-checker-ref',env=env)
  binaries={'rust':sha(producer.read_bytes()),'go':sha(reference.read_bytes())};lock=threading.Lock();state={'groups':{},'rows':{},'active':{}}
  if args.write:RECORD.mkdir(parents=True,exist_ok=True)
  def receipt():
   # Called under the lock; serialize each immutable observation before another update.
   if not args.write:return
   payload={'schema':ATTEMPT_SCHEMA,'status':'incomplete measurement','inputs':before,'stringop_groups':[state['groups'][n] for n in COUNTS if n in state['groups']],'cases':[state['rows'][key] for key in expected_cases() if key in state['rows']],'active':{str(n):v for n,v in sorted(state['active'].items())},'checker_binaries':binaries,'measurement_parallel_groups':PARALLEL_GROUPS}
   staging=RECORD/'attempt.json.tmp';staging.write_bytes(json_bytes(payload));staging.replace(RECORD/'attempt.json')
  with lock:receipt()
  def group(n):
   first,second=temp/f'first-{n}',temp/f'second-{n}';one=timed([producer,GENERATE_COMMAND,ROOT,first,str(n)]);two=timed([producer,GENERATE_COMMAND,ROOT,second,str(n)])
   if one['exit_code'] or two['exit_code']:raise RuntimeError('stringop failed; preserve the incomplete attempt')
   names=sorted(p.name for p in first.iterdir())
   if names!=sorted(p.name for p in second.iterdir()) or any((first/name).read_bytes()!=(second/name).read_bytes() for name in names):raise RuntimeError('stringop generation is nondeterministic')
   index=json.loads((first/'index.json').read_text());ids=[c['id'] for c in index['cases']]
   if index['schema']!=INDEX_SCHEMA or len(ids)!=len(set(ids)) or names!=sorted([i+'.mpcert' for i in ids]+['index.json']):raise RuntimeError('stringop index identity')
   g={'count':n,'scope':'complete candidate group and shared ordinary library','first':one,'second':two}
   with lock:state['groups'][n]=g;receipt()
   rows=[]
   for case in index['cases']:
    path=first/(case['id']+'.mpcert');raw=path.read_bytes()
    if args.write:shutil.copyfile(path,RECORD/path.name)
    obs={}
    for k,exe in (('rust',producer),('go',reference)):
     with lock:state['active'][n]={'case':case,'observations':dict(obs),'raw_sha256':sha(raw),'checking':k};receipt()
     obs[k]=observe(exe,path)
    classification=verify_case(case,obs);row={'case':case,'classification':classification,'raw_sha256':sha(raw),'observations':obs}
    if args.check and (RECORD/path.name).read_bytes()!=raw:raise RuntimeError('retained stringop certificate differs')
    rows.append(row);key=(n,case['family'],case['mutation'])
    with lock:
     if key in state['rows']:raise RuntimeError('shadowed stringop case')
     state['rows'][key]=row;state['active'].pop(n,None);receipt();print(f"{case['id']}: {classification}; Rust {obs['rust']['wall_seconds']:.3f}s, Go {obs['go']['wall_seconds']:.3f}s",flush=True)
   return g,rows
  completed={}
  with ThreadPoolExecutor(max_workers=PARALLEL_GROUPS) as pool:
   futures={pool.submit(group,n):n for n in COUNTS}
   for future in as_completed(futures):completed[futures[future]]=future.result()
  groups=[completed[n][0] for n in COUNTS];rows=[r for n in COUNTS for r in completed[n][1]]
  extra=measure_extra(temp,env)
  if before!=proof_sources():raise RuntimeError('source changed during stringop measurement')
  if binaries!={'rust':sha(producer.read_bytes()),'go':sha(reference.read_bytes())}:raise RuntimeError('checker executable changed during stringop measurement')
  report={'schema':REPORT_SCHEMA,'work_item':'GO-PRAC-T01-W03','status':STATUS,'scope':SCOPE,'pending':PENDING,'inputs':before,'host_toolchain_inventory':inventory,'checker_binaries':binaries,'host':{'system':platform.system(),'release':platform.release(),'machine':platform.machine(),'cpu_count':os.cpu_count(),'model':None,'physical_memory_bytes':None},'rustc':checked([shutil.which('rustc'),'--version']).strip(),'cargo':checked([shutil.which('cargo'),'--version']).strip(),'build':'Rust release offline locked; Go 1.25.0 pinned host trimpath','measurement_child_ceiling_seconds':60,'ceiling_scope':'private measurement, not a profile budget','boundary_probe_scope':BOUNDARY_SCOPE,'deterministic_generations':2,'measurement_parallel_groups':PARALLEL_GROUPS,'stringop_groups':groups,'cases':rows,'outcomes':{key:sum(r['classification']==key for r in rows) for key in ('accepted_zero_axiom','core_type_mismatch','candidate_fuel_failure_not_false_claim_evidence')},EXTRA_REPORT_KEY:extra}
  if args.write:
   for name in EXTRA_FILES:shutil.copyfile(temp/name,RECORD/name)
   (RECORD/'observations.json').write_bytes(json_bytes(report));(RECORD/'attempt.json').unlink()
  retained=json.loads((RECORD/'observations.json').read_text());check_retained(retained)
  if [r['case'] for r in retained['cases']]!=[r['case'] for r in rows]:raise RuntimeError('retained stringop case index differs')
  if args.check and any((RECORD/name).read_bytes()!=(temp/name).read_bytes() for name in EXTRA_FILES):raise RuntimeError('retained native stringop bytes differ')
  print(f'PASS: {len(rows)} stringop candidates; all guards/output bounds explicit; W03 incomplete',flush=True)

if __name__=='__main__':main()
