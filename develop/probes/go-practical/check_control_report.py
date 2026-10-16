#!/usr/bin/env python3
"""Retained proposed Nat-induction evidence integrity, not production admission."""
import json
import re
import sys
import control_foundation_build as build
import control_differential as native
from check_balanced_report import resource
from entry_audit import read_json,require
from run_proof_pilot import sha
from run_control_probes import ROOT,RECORD,SCOPE,PENDING,FAMILIES,VARIANTS,CASE_SCOPE,INTERFACE_SCOPE,proof_sources,classification,inventory
def check(report):
 require(report['schema']=='mpk.go_practical.control_report.v0' and report['work_item']=='GO-PRAC-T01-W03','control schema')
 require(report['status']=='private proposed induction/control feasibility measured; W03 cumulative exit review remains open' and report['scope']==SCOPE and report['pending']==PENDING,'private/production/completion scope')
 require(report['inputs']==proof_sources(),'control stale source inputs')
 require(report['mirrored_checker_inputs']==inventory(build.files()),'control mirrored checker identity')
 require(report['generated_producer_inputs']==inventory(build.producer_files()),'control generated producer identity')
 require(report['build']=='private mirrored Rust release offline locked and Go1.25.0 trimpath; production unchanged','control build scope')
 require(report['host_toolchain_inventory']=={'files':14496,'sha256':'85d1dd7ca77c9263d83e25f05e4c32105738fdc002434d61b27671aee4e259bc'},'control toolchain')
 require(report['host']['system']=='Darwin' and report['host']['machine']=='arm64','control host resource units')
 require(report['child_ceiling_seconds']==60 and report['ceiling_scope']=='private measurements, not profile budgets','control ceiling scope')
 require(set(report['checker_binaries'])=={'rust','go'} and all(re.fullmatch('[0-9a-f]{64}',h) for h in report['checker_binaries'].values()),'control private checker pins')
 require(report['checker_binaries']['go']!='bc21d279807f5994a16287a532db65cac3bbb170bfa4d0ca5c6729825e24c62c','private extension reported as production checker')
 require(len(report['generations'])==2,'control deterministic generation count')
 for g in report['generations']:require(g['exit_code']==0,'control generation result');resource(g)
 require([(r['case']['family'],r['case']['mutation']) for r in report['cases']]==[(f,m) for f in FAMILIES for m in range(2)],'control case coverage')
 require([r['case']['variant'] for r in report['interface_cases']]==list(VARIANTS),'control interface coverage')
 names=[];outcomes=dict.fromkeys(('accepted_zero_axiom','core_type_mismatch','interface_accepted','malformed_interface_rejection'),0)
 for interface,rows in [(False,report['cases']),(True,report['interface_cases'])]:
  for row in rows:
   c=row['case'];name=c['id'];names.append(name)
   require(all(type(c[k]) is int and c[k]>0 for k in ('declarations','terms','bytes')),'control numeric types')
   if interface:
    variant=c['variant'];require(name=='induction-interface-'+variant and c['scope']==INTERFACE_SCOPE,'control interface identity/scope')
    require(c['expected']==('accepted' if variant in ('legacy-no-induct','unused-term-levels') else 'rejected'),'control interface expected boundary')
    count=22 if variant=='legacy-no-induct' else 24 if variant in ('extra-constructor-before','extra-constructor-after','induct-levels-partial','family-levels','constructor-levels','prior-opaque-levels','prior-theorem-levels') else 25
    require(c['declarations']==count,'control interface declaration identity')
   else:
    f,m=c['family'],c['mutation'];require(type(m) is int and m in (0,1),'control mutation type')
    require(name==f"control-{f}-{'valid' if m==0 else 'altered'}" and c['expected']==('accepted' if m==0 else 'rejected'),'control claim identity')
    require(c['scope']==CASE_SCOPE and c['frame_bits']==128,'control complete frame/production scope')
    domain='arbitrary genuine Std.Nat; no finite unrolling' if f.startswith('invariant-') or f=='absorbing-stop' else 'single source-shaped abrupt edge; no iteration-count premise'
    require(c['iteration_domain']==domain,'control iteration domain')
    require(c['theorem']=='Probe.GoPractical.Word.ControlClaim'+f.replace('-',''),'control theorem identity')
    require(c['declarations']==26,'control claim declaration identity')
   raw=(RECORD/(name+'.mpcert')).read_bytes();require(c['bytes']==len(raw) and row['raw_sha256']==sha(raw),'control certificate byte identity')
   try:kind=classification(c,row['observations'],interface)
   except (RuntimeError,KeyError) as e:raise ValueError(str(e)) from e
   require(row['classification']==kind,'control rejection classification')
   if kind=='accepted_zero_axiom':
    digest=sha(b'MPK-MODULE-CERT-0.1'+bytes([0])+raw);require(all(v['report']['hashes']['certificate']==digest for v in row['observations'].values()),'control accepted certificate binding')
   outcomes['interface_accepted' if interface and kind=='accepted_zero_axiom' else kind]+=1
   for v in row['observations'].values():resource(v)
 require(len(names)==len(set(names)) and sorted(p.name for p in RECORD.glob('*.mpcert'))==sorted(n+'.mpcert' for n in names),'control retained unique certificate coverage')
 require(report['outcomes']==outcomes and outcomes=={'accepted_zero_axiom':6,'core_type_mismatch':6,'interface_accepted':2,'malformed_interface_rejection':13},'control outcome coverage')
 observed=report['native_observations'];native.check(observed,RECORD);require(observed['go_measurement']['exit_code']==0,'control original-Go execution');resource(observed['go_measurement'])
 require([r['case_id'] for r in observed['false_claim_witnesses']]==[r['case']['id'] for r in report['cases'] if r['case']['mutation']],'control false witness scope')
 return len(names)
if __name__=='__main__':
 try:print(f"PASS: {check(read_json(ROOT,(RECORD/'observations.json').relative_to(ROOT).as_posix()))} private proposed-control/interface candidates; production unchanged")
 except (OSError,KeyError,TypeError,ValueError) as e:print(f'Control evidence rejected: {e}',file=sys.stderr);raise SystemExit(1)
