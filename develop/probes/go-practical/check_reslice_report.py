#!/usr/bin/env python3
"""Retained operation/diagnostic evidence integrity; proof replay is separate."""
import json
from pathlib import PurePosixPath
import re
import sys

from check_balanced_report import resource
from entry_audit import read_json,require
import reslice_differential as native
from run_proof_pilot import sha
from run_reslice_probes import BOUNDARY_SCOPE,CASE_SCOPE,COUNTS,PENDING,RECORD,REPORT_SCHEMA,ROOT,SCOPE,STATUS,declaration_count,depth,expected_cases,identifier,padding,proof_sources,verify_case

def check(report):
 require(report['schema']==REPORT_SCHEMA and report['work_item']=='GO-PRAC-T01-W03','reslice report schema')
 require(report['status']==STATUS,'invented W03 completion')
 require(report['scope']==SCOPE and report['pending']==PENDING,'lost production/ownership/operation pending scope')
 require(report['inputs']==proof_sources(),'stale source identity')
 require(report['boundary_probe_scope']==BOUNDARY_SCOPE,'padding promoted to source admission')
 require(report['deterministic_generations']==2,'missing deterministic generation')
 require(report['build']=='Rust release offline locked; Go 1.25.0 pinned host trimpath','checker build identity')
 require(report['host_toolchain_inventory']=={'files':14496,'sha256':'85d1dd7ca77c9263d83e25f05e4c32105738fdc002434d61b27671aee4e259bc'},'host toolchain inventory')
 require(report['measurement_child_ceiling_seconds']==60 and report['ceiling_scope']=='private measurement, not a profile budget','invented resource budget')
 require(report['host']['system']=='Darwin' and report['host']['machine']=='arm64','resource host units')
 require(set(report['checker_binaries'])=={'rust','go'} and all(re.fullmatch(r'[0-9a-f]{64}',h) for h in report['checker_binaries'].values()),'checker binary identity')
 require([g['count'] for g in report['construction_groups']]==list(COUNTS),'construction coverage')
 require([(r['case']['capacity'],r['case']['kind'],r['case']['family'],r['case']['mutation']) for r in report['cases']]==expected_cases(),'case coverage')
 names=[r['case']['id'] for r in report['cases']]
 require(len(names)==len(set(names)),'shadowed candidate identity')
 require(sorted(p.name for p in RECORD.glob('*.mpcert'))==sorted(n+'.mpcert' for n in names),'retained certificate coverage')
 for g in report['construction_groups']:
  require(g['scope']=='complete candidate group and shared ordinary library','construction scope')
  for m in (g['first'],g['second']):
   require(m['exit_code']==0,'construction failed');resource(m)
 outcomes={k:0 for k in ('accepted_zero_axiom','core_type_mismatch','candidate_fuel_failure_not_false_claim_evidence')}
 for row in report['cases']:
  c=row['case'];n,kind,family,mutation=c['capacity'],c['kind'],c['family'],c['mutation']
  require(type(n) is int and type(c['count']) is int and type(mutation) is int,'candidate number types')
  require(c['count']==n and c['id']==identifier(n,kind,family,mutation) and c['expected']==('accepted' if mutation==0 else 'rejected'),'candidate identity')
  require(c['address_depth']==depth(n) and c['padding_capacity']==padding(n),'semantic/padding identity')
  require(c['source_word_bits']==64 and c['descriptor_bits']==512 and c['element_bits']==128,'lost full source/descriptor/element domain')
  require(c['extent_intermediate_bits']==65,'lost computed widened extent')
  require(c['logical_region_count']==2 and c['source_pointer_bits_admitted'] is False,'region/source pointer scope')
  require(c['quantified_storage_bits']==128<<(depth(n)+1),'lost complete backing/padding domain')
  require(c['scope']==CASE_SCOPE,'overstated candidate scope')
  require(c['theorem']=='Probe.GoPractical.Word.SliceResliceClaim'+kind+family.replace('-','') and c['declarations']==declaration_count(n),'theorem identity')
  require(type(c['terms']) is int and c['terms']>0,'term count')
  require('Forall("index_canonical"' not in c['goal_debug'] and 'Forall("upper_canonical"' not in c['goal_debug'],'internal constructor fact promoted to caller assumption')
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
 require(report['outcomes']==outcomes and outcomes=={'accepted_zero_axiom':50,'core_type_mismatch':50,'candidate_fuel_failure_not_false_claim_evidence':0},'outcome counts')
 observed=report['reslice_observations'];native.check(observed,RECORD)
 require([w['case_id'] for w in observed['false_claim_witnesses']]==[r['case']['id'] for r in report['cases'] if r['case']['mutation']],'false-claim witness coverage')
 require(observed['go_measurement']['exit_code']==0,'Go reslice observer did not execute')
 require(re.fullmatch(r'[0-9a-f]{64}',observed['go_binary_sha256']) is not None,'Go observer binary identity')
 resource(observed['go_measurement'])
 return len(report['cases'])

def check_initial(finding=None):
 archive=RECORD.parent/'reslice-initial'
 finding=json.loads((archive/'finding.json').read_text()) if finding is None else finding
 require(finding['schema']=='mpk.go_practical.reslice_initial.v0' and finding['status']=='failed positive diagnostic; never false-claim evidence','initial positive diagnostic scope')
 require(finding['source_baseline']=='94752d34168df9aa71dc420126bb25fdf29bace5' and finding['source_snapshot_groups']==2 and finding['failed_positive_certificates']==3,'initial finding coverage')
 require(finding['expected']=='accepted' and finding['observed']=='both core type mismatch' and finding['localized_first_failure']=='SliceAccessValidityResliceNumberRestoreSum','initial failure meaning')
 for group in (archive,archive/'localization'):
  binaries=json.loads((group/'binaries.json').read_text())
  require(set(binaries)=={'rust','go'} and all(re.fullmatch(r'[0-9a-f]{64}',h) for h in binaries.values()),'initial checker identity')
  for item in json.loads((group/'sources.json').read_text()):
   path=PurePosixPath(item['path']);require(not path.is_absolute() and '..' not in path.parts,'initial source path')
   prefix='develop/probes/go-practical/certificates/src/'
   source=group/'source'/item['path'].removeprefix(prefix) if item['path'].startswith(prefix) else ROOT/path
   require(sha(source.read_bytes())==item['sha256'],'initial source identity')
 count=0
 for group,name in ((archive,'numeric-second.mpcert'),(archive/'localization','difference-nonnegative.mpcert')):
  rows=json.loads((group/'observations.json').read_text());require(len(rows)==1,'initial raw finding coverage')
  row=rows[0];raw=(group/name).read_bytes();require(sha(raw)==row['raw_sha256'],'initial certificate identity')
  require(set(row['observations'])=={'rust','go'},'initial independent checkers')
  for k,m in row['observations'].items():
   r=m['report'];require(m['exit_code']==1 and r['verdict']=='rejected','initial failed positive attempt')
   if k=='rust':
    require(r['error_code']=='KERNEL_CORE_CHECK' and json.loads(r['error_detail'])['code']=='CORE_TYPE_MISMATCH','initial failure class')
    require(r['hashes']['certificate']==sha(b'MPK-MODULE-CERT-0.1'+bytes([0])+raw),'initial checker/certificate identity')
   else:require(r['error_kind']=='core_check' and r['error_detail'].startswith('term ') and ' inferred ' in r['error_detail'] and ' but expected ' in r['error_detail'],'initial failure class')
   resource(m)
  count+=1
 rows=json.loads((archive/'localization/prefix-localization.json').read_text())
 require([r['path'] for r in rows]==['stage-reslice-sum-lower.mpcert','stage-reslice-sub-carry.mpcert','stage-reslice-restore-false-false.mpcert'],'initial localization coverage')
 for i,row in enumerate(rows):
  name=('reslice-sum-lower.mpcert','reslice-sub-carry.mpcert','restore-sum.mpcert')[i]
  raw=(archive/'localization'/name).read_bytes();require(sha(raw)==row['raw_sha256'],'initial localization byte identity')
  m=row['rust'];r=m['report'];resource(m)
  require(r['hashes']['certificate']==sha(b'MPK-MODULE-CERT-0.1'+bytes([0])+raw),'initial localization checker identity')
  require(m['exit_code']==(1 if i==2 else 0) and r['verdict']==('rejected' if i==2 else 'accepted'),'initial localization outcome')
  if i==2:
   require(json.loads(r['error_detail'])['code']=='CORE_TYPE_MISMATCH','initial localization failure class');count+=1
  else:require(r['axiom_count']==0,'initial localization control axioms')
 require(finding['sha256']==sha((archive/'numeric-second.mpcert').read_bytes()),'initial finding raw identity')
 return count

if __name__=='__main__':
 try:
  n=check(read_json(ROOT,(RECORD/'observations.json').relative_to(ROOT).as_posix()));initial=check_initial()
  print(f'PASS: {n} reslice candidates and {initial} retained failed-positive attempts; integrity only, W03 incomplete')
 except (OSError,KeyError,TypeError,ValueError) as error:
  print(f'Reslice report rejected: {error}',file=sys.stderr);raise SystemExit(1)
