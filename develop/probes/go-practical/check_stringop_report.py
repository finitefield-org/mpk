#!/usr/bin/env python3
"""Retained operation/diagnostic evidence integrity; proof replay is separate."""
import json
from pathlib import PurePosixPath
import re
import sys

from check_balanced_report import resource
from entry_audit import read_json,require
import stringop_differential as native
from run_proof_pilot import sha
from run_stringop_probes import PARALLEL_GROUPS,BOUNDARY_SCOPE,CASE_SCOPE,COUNTS,PENDING,RECORD,REPORT_SCHEMA,ROOT,SCOPE,STATUS,declaration_count,depth,expected_cases,identifier,padding,proof_sources,verify_case

def check(report):
 require(report['schema']==REPORT_SCHEMA and report['work_item']=='GO-PRAC-T01-W03','stringop report schema')
 require(report['status']==STATUS,'invented W03 completion')
 require(report['scope']==SCOPE and report['pending']==PENDING,'lost production/ownership/operation pending scope')
 require(report['inputs']==proof_sources(),'stale source identity')
 require(report['boundary_probe_scope']==BOUNDARY_SCOPE,'padding promoted to source admission')
 require(report['deterministic_generations']==2,'missing deterministic generation')
 require(report['measurement_parallel_groups']==PARALLEL_GROUPS,'measurement concurrency scope')
 require(report['build']=='Rust release offline locked; Go 1.25.0 pinned host trimpath','checker build identity')
 require(report['host_toolchain_inventory']=={'files':14496,'sha256':'85d1dd7ca77c9263d83e25f05e4c32105738fdc002434d61b27671aee4e259bc'},'host toolchain inventory')
 require(report['measurement_child_ceiling_seconds']==60 and report['ceiling_scope']=='private measurement, not a profile budget','invented resource budget')
 require(report['host']['system']=='Darwin' and report['host']['machine']=='arm64','resource host units')
 require(set(report['checker_binaries'])=={'rust','go'} and all(re.fullmatch(r'[0-9a-f]{64}',h) for h in report['checker_binaries'].values()),'checker binary identity')
 require([g['count'] for g in report['stringop_groups']]==list(COUNTS),'stringop coverage')
 require([(r['case']['capacity'],r['case']['family'],r['case']['mutation']) for r in report['cases']]==expected_cases(),'case coverage')
 names=[r['case']['id'] for r in report['cases']]
 require(len(names)==len(set(names)),'shadowed candidate identity')
 require(sorted(p.name for p in RECORD.glob('*.mpcert'))==sorted(n+'.mpcert' for n in names),'retained certificate coverage')
 for g in report['stringop_groups']:
  require(g['scope']=='complete candidate group and shared ordinary library','stringop scope')
  for m in (g['first'],g['second']):
   require(m['exit_code']==0,'stringop failed');resource(m)
 outcomes={k:0 for k in ('accepted_zero_axiom','core_type_mismatch','candidate_fuel_failure_not_false_claim_evidence')}
 for row in report['cases']:
  c=row['case'];n,family,mutation=c['capacity'],c['family'],c['mutation']
  require(type(n) is int and type(c['count']) is int and type(mutation) is int,'candidate number types')
  require(c['count']==n and c['id']==identifier(n,family,mutation) and c['expected']==('accepted' if mutation==0 else 'rejected'),'candidate identity')
  require(c['address_depth']==depth(n) and c['padding_capacity']==padding(n),'semantic/padding identity')
  require(c['source_word_bits']==64 and c['length_bits']==64 and c['payload_bits']==8,'lost complete source/length/byte domain')
  require(c['quantified_storage_bits']==(8<<depth(n))*(1 if family.startswith('substring-') else 2),'lost complete finite byte domain')
  require(c['scope']==CASE_SCOPE,'overstated candidate scope')
  require(c['theorem']=='Probe.GoPractical.Word.StringOperationClaim'+family.replace('-','') and c['declarations']==declaration_count(n),'theorem identity')
  require(type(c['terms']) is int and c['terms']>0,'term count')
  require(all('Forall("'+key+'"' not in c['goal_debug'] for key in ('index_canonical','offset_canonical','result_nonnegative','no_wrap','fits')),'internal arithmetic/coverage fact promoted to caller assumption')
  raw=(RECORD/(c['id']+'.mpcert')).read_bytes()
  require(len(raw)==c['bytes'] and row['raw_sha256']==sha(raw),'certificate byte identity')
  require(set(row['observations'])=={'rust','go'},'missing independent checker')
  try:classification=verify_case(c,row['observations'])
  except RuntimeError as error:raise ValueError(str(error)) from error
  require(row['classification']==classification,'incorrect rejection classification')
  if classification=='accepted_zero_axiom':
   digest=sha(b'MPK-MODULE-CERT-0.1'+bytes([0])+raw)
   require(all(m['report']['hashes']['certificate']==digest for m in row['observations'].values()),'accepted checker/certificate identity')
  outcomes[classification]+=1
  for m in row['observations'].values():resource(m)
 require(report['outcomes']==outcomes and outcomes=={'accepted_zero_axiom':124,'core_type_mismatch':124,'candidate_fuel_failure_not_false_claim_evidence':0},'outcome counts')
 observed=report['stringop_observations'];native.check(observed,RECORD)
 require([w['case_id'] for w in observed['false_claim_witnesses']]==[r['case']['id'] for r in report['cases'] if r['case']['mutation']],'false-claim witness coverage')
 require(observed['go_measurement']['exit_code']==0,'Go stringop observer did not execute')
 require(re.fullmatch(r'[0-9a-f]{64}',observed['go_binary_sha256']) is not None,'Go observer binary identity')
 resource(observed['go_measurement'])
 return len(report['cases'])

if __name__=='__main__':
 try:
  n=check(read_json(ROOT,(RECORD/'observations.json').relative_to(ROOT).as_posix()))
  print(f'PASS: {n} stringop candidates; integrity only, W03 incomplete')
 except (OSError,KeyError,TypeError,ValueError) as error:
  print(f'String operation report rejected: {error}',file=sys.stderr);raise SystemExit(1)
