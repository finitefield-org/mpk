#!/usr/bin/env python3
"""Measure coupled struct fields, full-width descriptors and byte matches."""
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

import aggregate_differential as native
import run_affix_probes as predecessor
import run_prefix_probes as shared
from aggregate_build_sources import PACKAGE,observe_generated,check_sources
from run_proof_pilot import DEFAULT_GOROOT,HERE,ROOT,TARGET,checked,json_bytes,observe,semantic_expectation,sha,timed,verify_host_toolchain

RECORD=ROOT/'develop/migrations/go-practical/t01-w03/aggregate'
POINTS=native.POINTS
GENERATE_COMMAND='aggregate-generate'
INDEX_SCHEMA='mpk.go_practical.aggregate_pilot.v0'
REPORT_SCHEMA='mpk.go_practical.aggregate_report.v0'
ATTEMPT_SCHEMA='mpk.go_practical.aggregate_attempt.v0'
STATUS='coupled aggregate ingredients measured; W03 cumulative feasibility review remains open'
SCOPE='ordinary aggregate field/frame, actual descriptor address reconstruction and complete prefix/suffix byte observations in one model; source stores and all cell padding remain arbitrary; no production source-to-certificate claim'
PENDING=['W03 cumulative control/aggregate feasibility and resource-budget review before freeze','production capture/model/operation bindings in T02–T06']
CASE_SCOPE='private aggregate struct/descriptor/byte composition; complete stores and cell padding quantified; production source and admission remain pending'
BOUNDARY_SCOPE='byte/slice pairs 0/0,1/1,32/8,31/31,32/32,33/33,255/127,256/128,257/129 are private workloads; shared finite padding is not admission; complete heap address bits and cell padding remain quantified'
PARALLEL_GROUPS=3
EXTRA_FILES=native.ARTIFACTS
EXTRA_REPORT_KEY='aggregate_observations'

def proof_sources():
 result={r['path']:r['sha256'] for r in predecessor.proof_sources()}
 paths=[Path(__file__),HERE/'check_aggregate_report.py',HERE/'aggregate_differential.py',HERE/'aggregate_reference.go',HERE/'aggregate_build_sources.py',HERE/'order_differential.py',HERE/'byte_differential.py',PACKAGE/'Cargo.toml',PACKAGE/'Cargo.lock']
 paths.extend(PACKAGE.rglob('*.rs'))
 for p in paths:
  result[p.relative_to(ROOT).as_posix()]=sha(p.read_bytes())
 return [{'path':p,'sha256':h} for p,h in sorted(result.items())]

def depth(n):return max(n-1,0).bit_length()
def padding(n):return 1<<depth(n)
def declaration_count(n):return {0:593,1:593,31:655,32:655,33:653,255:649,256:649,257:675}[n]
def expected_cases():return [(b,s,f,m) for b,s in POINTS for f in native.families(b,s) for m in range(2)]
def identifier(b,s,f,m):return native.identifier(b,s,f,m)
def verify_case(case,observations):
 semantic_expectation(case,observations,declarations=declaration_count(case['common_bound']),module=f"Probe.GoPractical.Word.AggregateCapacity{case['common_bound']}")
 if case['expected']=='accepted':return 'accepted_zero_axiom'
 rust=json.loads(observations['rust']['report']['error_detail'])['code'];go=observations['go']['report']['error_detail']
 if rust=='CORE_TYPE_MISMATCH' and go.startswith('term ') and ' inferred ' in go and ' but expected ' in go:return 'core_type_mismatch'
 raise RuntimeError('aggregate claim did not fail at the intended type boundary; resource failures never establish a false claim')

def measure_extra(directory,env):
 exe=directory/'aggregate-observer'
 checked([str(DEFAULT_GOROOT/'bin/go'),'build','-trimpath','-buildvcs=false','-o',str(exe),str(HERE/'aggregate_reference.go')],env=env)
 return native.run(exe,directory)

def check_retained(report):
 from check_aggregate_report import check
 return check(report)

def main():
 parser=argparse.ArgumentParser(description=__doc__);mode=parser.add_mutually_exclusive_group(required=True);mode.add_argument('--write',action='store_true');mode.add_argument('--check',action='store_true');args=parser.parse_args()
 if platform.system()!='Darwin' or platform.machine()!='arm64':raise RuntimeError('pinned host and resource units require darwin/arm64')
 pin=json.loads((HERE/'host-toolchain.json').read_text());inventory=verify_host_toolchain(DEFAULT_GOROOT,pin)
 before=proof_sources()
 built=checked([shutil.which('cargo'),'build','--release','--offline','--locked','--message-format=json','--manifest-path',str(PACKAGE/'Cargo.toml')],env={**os.environ,'CARGO_TARGET_DIR':str(TARGET)})
 generated=observe_generated(built)
 if before!=proof_sources():raise RuntimeError('source changed while building the aggregate producer')
 producer=TARGET/'release/go-practical-aggregate-probe'
 with tempfile.TemporaryDirectory(prefix='mpk-go-aggregate-',dir='/private/tmp') as temporary:
  temp=Path(temporary);reference=temp/'mpk-checker-ref'
  env={**os.environ,'GOENV':'off','GOWORK':'off','GOTOOLCHAIN':'local','GOPROXY':'off','GOTELEMETRY':'off','CGO_ENABLED':'0','GOCACHE':'/private/tmp/mpk-go-practical-checker-cache'}
  checked([str(DEFAULT_GOROOT/'bin/go'),'build','-trimpath','-buildvcs=false','-o',str(reference),'./cmd/mpk-checker-ref'],cwd=ROOT/'go-tools/mpk-checker-ref',env=env)
  binaries={'rust':sha(producer.read_bytes()),'go':sha(reference.read_bytes())};lock=threading.Lock();state={'groups':{},'rows':{},'active':{}}
  if args.write:RECORD.mkdir(parents=True,exist_ok=True)
  def receipt():
   # Called under the lock; serialize each immutable observation before another update.
   if not args.write:return
   payload={'schema':ATTEMPT_SCHEMA,'status':'incomplete measurement','inputs':before,'aggregate_groups':[state['groups'][n] for n in POINTS if n in state['groups']],'cases':[state['rows'][key] for key in expected_cases() if key in state['rows']],'active':{str(n):v for n,v in sorted(state['active'].items())},'checker_binaries':binaries,'measurement_parallel_groups':PARALLEL_GROUPS}
   staging=RECORD/'attempt.json.tmp';staging.write_bytes(json_bytes(payload));staging.replace(RECORD/'attempt.json')
  with lock:receipt()
  def group(n):
   b,s=n
   first,second=temp/f'first-{n}',temp/f'second-{n}';one=timed([producer,GENERATE_COMMAND,ROOT,first,str(b),str(s)]);two=timed([producer,GENERATE_COMMAND,ROOT,second,str(b),str(s)])
   if one['exit_code'] or two['exit_code']:raise RuntimeError('aggregate failed; preserve the incomplete attempt')
   names=sorted(p.name for p in first.iterdir())
   if names!=sorted(p.name for p in second.iterdir()) or any((first/name).read_bytes()!=(second/name).read_bytes() for name in names):raise RuntimeError('aggregate generation is nondeterministic')
   index=json.loads((first/'index.json').read_text());ids=[c['id'] for c in index['cases']]
   if index['schema']!=INDEX_SCHEMA or len(ids)!=len(set(ids)) or names!=sorted([i+'.mpcert' for i in ids]+['index.json']):raise RuntimeError('aggregate index identity')
   g={'byte_bound':b,'slice_bound':s,'scope':'complete candidate group and shared ordinary library','first':one,'second':two}
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
    if args.check and (RECORD/path.name).read_bytes()!=raw:raise RuntimeError('retained aggregate certificate differs')
    rows.append(row);key=(b,s,case['family'],case['mutation'])
    with lock:
     if key in state['rows']:raise RuntimeError('shadowed aggregate case')
     state['rows'][key]=row;state['active'].pop(n,None);receipt();print(f"{case['id']}: {classification}; Rust {obs['rust']['wall_seconds']:.3f}s, Go {obs['go']['wall_seconds']:.3f}s",flush=True)
   return g,rows
  completed={}
  with ThreadPoolExecutor(max_workers=PARALLEL_GROUPS) as pool:
   futures={pool.submit(group,n):n for n in POINTS}
   for future in as_completed(futures):completed[futures[future]]=future.result()
  groups=[completed[n][0] for n in POINTS];rows=[r for n in POINTS for r in completed[n][1]]
  extra=measure_extra(temp,env)
  if before!=proof_sources():raise RuntimeError('source changed during aggregate measurement')
  if binaries!={'rust':sha(producer.read_bytes()),'go':sha(reference.read_bytes())}:raise RuntimeError('checker binary changed during aggregate measurement')
  report={'schema':REPORT_SCHEMA,'work_item':'GO-PRAC-T01-W03','status':STATUS,'scope':SCOPE,'pending':PENDING,'inputs':before,'generated_sources':generated,'host_toolchain_inventory':inventory,'checker_binaries':binaries,'host':{'system':platform.system(),'release':platform.release(),'machine':platform.machine(),'cpu_count':os.cpu_count(),'model':None,'physical_memory_bytes':None},'rustc':checked([shutil.which('rustc'),'--version']).strip(),'cargo':checked([shutil.which('cargo'),'--version']).strip(),'build':'Rust release offline locked; Go 1.25.0 pinned host trimpath','measurement_child_ceiling_seconds':60,'ceiling_scope':'private measurement, not a profile budget','boundary_probe_scope':BOUNDARY_SCOPE,'deterministic_generations':2,'measurement_parallel_groups':PARALLEL_GROUPS,'aggregate_groups':groups,'cases':rows,'outcomes':{key:sum(r['classification']==key for r in rows) for key in ('accepted_zero_axiom','core_type_mismatch','candidate_fuel_failure_not_false_claim_evidence')},EXTRA_REPORT_KEY:extra}
  if args.write:
   for name in EXTRA_FILES:shutil.copyfile(temp/name,RECORD/name)
   (RECORD/'observations.json').write_bytes(json_bytes(report));(RECORD/'attempt.json').unlink()
  retained=json.loads((RECORD/'observations.json').read_text());check_retained(retained)
  if [r['case'] for r in retained['cases']]!=[r['case'] for r in rows]:raise RuntimeError('retained aggregate case index differs')
  if args.check and any((RECORD/name).read_bytes()!=(temp/name).read_bytes() for name in EXTRA_FILES):raise RuntimeError('retained native aggregate bytes differ')
  print(f'PASS: {len(rows)} aggregate candidates; coupled whole-cell and complete matched-byte observations; W03 incomplete',flush=True)

if __name__=='__main__':main()
