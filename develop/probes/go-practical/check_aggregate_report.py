#!/usr/bin/env python3
"""Verify retained aggregate evidence; kernel replay is a separate command."""
import re
import sys
from check_balanced_report import resource
from entry_audit import read_json,require
import aggregate_differential as native
from aggregate_build_sources import check_sources
from run_proof_pilot import sha
from run_aggregate_probes import PARALLEL_GROUPS,BOUNDARY_SCOPE,CASE_SCOPE,POINTS,PENDING,RECORD,REPORT_SCHEMA,ROOT,SCOPE,STATUS,declaration_count,depth,expected_cases,identifier,padding,proof_sources,verify_case

def check(report):
 require(report['schema']==REPORT_SCHEMA and report['work_item']=='GO-PRAC-T01-W03','aggregate schema')
 require(report['status']==STATUS and report['scope']==SCOPE and report['pending']==PENDING,'aggregate feasibility/production scope')
 require(report['inputs']==proof_sources(),'aggregate stale source identity')
 check_sources(report['generated_sources'])
 require(report['boundary_probe_scope']==BOUNDARY_SCOPE,'aggregate padding/admission scope')
 require(report['deterministic_generations']==2 and report['measurement_parallel_groups']==PARALLEL_GROUPS,'aggregate replay/concurrency scope')
 require(report['build']=='Rust release offline locked; Go 1.25.0 pinned host trimpath','aggregate build identity')
 require(report['host_toolchain_inventory']=={'files':14496,'sha256':'85d1dd7ca77c9263d83e25f05e4c32105738fdc002434d61b27671aee4e259bc'},'aggregate host toolchain')
 require(report['measurement_child_ceiling_seconds']==60 and report['ceiling_scope']=='private measurement, not a profile budget','aggregate private ceiling')
 require(report['host']['system']=='Darwin' and report['host']['machine']=='arm64','aggregate resource host units')
 require(set(report['checker_binaries'])=={'rust','go'} and all(re.fullmatch('[0-9a-f]{64}',v) for v in report['checker_binaries'].values()),'aggregate checker identities')
 require([(g['byte_bound'],g['slice_bound']) for g in report['aggregate_groups']]==list(POINTS),'aggregate workload coverage')
 require([(r['case']['byte_bound'],r['case']['slice_bound'],r['case']['family'],r['case']['mutation']) for r in report['cases']]==expected_cases(),'aggregate candidate coverage')
 names=[r['case']['id'] for r in report['cases']]
 require(len(names)==len(set(names)),'aggregate duplicate candidate')
 require(sorted(p.name for p in RECORD.glob('*.mpcert'))==sorted(n+'.mpcert' for n in names),'aggregate retained certificate coverage')
 for g in report['aggregate_groups']:
  require(g['scope']=='complete candidate group and shared ordinary library','aggregate group scope')
  for m in (g['first'],g['second']):
   require(m['exit_code']==0,'aggregate generation failed');resource(m)
 outcomes=dict.fromkeys(('accepted_zero_axiom','core_type_mismatch','candidate_fuel_failure_not_false_claim_evidence'),0)
 for row in report['cases']:
  c=row['case'];b,s,f,m=c['byte_bound'],c['slice_bound'],c['family'],c['mutation'];n=max(b,s);d=depth(n);cell_depth=2+max(6,d+3)
  require(all(type(c[k]) is int for k in ('byte_bound','slice_bound','mutation','common_bound','address_depth','padding_capacity','cell_depth','cell_bits','source_word_bits','payload_bits','heap_regions','heap_address_bits','observed_finite_heap_bits','declarations','terms','bytes')),'aggregate numeric types')
  require(c['id']==identifier(b,s,f,m) and c['expected']==('accepted' if m==0 else 'rejected'),'aggregate candidate identity')
  require(c['common_bound']==n and c['address_depth']==d and c['padding_capacity']==padding(n),'aggregate semantic/padding identity')
  require(c['cell_depth']==cell_depth and c['cell_bits']==1<<cell_depth,'aggregate complete cell/padding identity')
  require(c['source_word_bits']==64 and c['payload_bits']==8 and c['heap_regions']==2 and c['heap_address_bits']==64,'aggregate full source/heap domains')
  require(c['observed_finite_heap_bits']==2*padding(n)*(1<<cell_depth),'aggregate finite observable footprint')
  require(c['scope']==CASE_SCOPE and c['theorem']=='Probe.GoPractical.Word.AggregateClaim'+f.replace('-','') and c['declarations']==declaration_count(n),'aggregate theorem/scope identity')
  require(c['terms']>0 and all('Forall("'+key+'"' not in c['goal_debug'] for key in ('fits','no_wrap','source_nonnegative','pattern_nonnegative')),'aggregate caller fit/sign oracle')
  raw=(RECORD/(c['id']+'.mpcert')).read_bytes()
  require(c['bytes']==len(raw) and row['raw_sha256']==sha(raw),'aggregate certificate bytes')
  require(set(row['observations'])=={'rust','go'},'aggregate independent checker')
  try:classification=verify_case(c,row['observations'])
  except RuntimeError as error:raise ValueError(str(error)) from error
  require(row['classification']==classification,'aggregate rejection class')
  if classification=='accepted_zero_axiom':
   digest=sha(b'MPK-MODULE-CERT-0.1'+bytes([0])+raw)
   require(all(v['report']['hashes']['certificate']==digest for v in row['observations'].values()),'aggregate checker/certificate binding')
  outcomes[classification]+=1
  for v in row['observations'].values():resource(v)
 require(report['outcomes']==outcomes and outcomes=={'accepted_zero_axiom':49,'core_type_mismatch':49,'candidate_fuel_failure_not_false_claim_evidence':0},'aggregate outcome counts')
 observed=report['aggregate_observations'];native.check(observed,RECORD)
 require([w['case_id'] for w in observed['false_claim_witnesses']]==[r['case']['id'] for r in report['cases'] if r['case']['mutation']],'aggregate false witness coverage')
 require(observed['go_measurement']['exit_code']==0 and re.fullmatch('[0-9a-f]{64}',observed['go_binary_sha256']) is not None,'aggregate native execution')
 resource(observed['go_measurement'])
 return len(report['cases'])

if __name__=='__main__':
 try:print(f"PASS: {check(read_json(ROOT,(RECORD/'observations.json').relative_to(ROOT).as_posix()))} aggregate candidates; retained integrity only")
 except (OSError,KeyError,TypeError,ValueError) as error:
  print(f'Aggregate evidence rejected: {error}',file=sys.stderr);raise SystemExit(1)
