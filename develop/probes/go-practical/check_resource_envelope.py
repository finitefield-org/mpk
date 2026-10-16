"""Validate exact representative resource evidence and its limited scope."""
import re
from entry_audit import require
from check_balanced_report import resource
from measure_sequential_resources import ROOT,BASE,SCOPE,SELECTION,files
from run_proof_pilot import sha,semantic_expectation
def check(report):
 require(report['schema']=='mpk.go_practical.sequential_resources.v0' and report['work_item']=='GO-PRAC-T01-W03' and report['scope']==SCOPE,'sequential resource schema/scope')
 reports,inputs,refs=files();require(report['inputs']==inputs and report['prerequisite_artifacts']==refs,'sequential source/artifact identities')
 require(report['direct_child_parallelism']==1 and report['child_ceiling_seconds']==60 and report['ceiling_scope']=='private measurement setting, not a profile budget','sequential concurrency/budget scope')
 require(report['host']['system']=='Darwin' and report['host']['machine']=='arm64','sequential host units')
 require(report['hardware_receipt']=='develop/migrations/go-practical/t01-w03/control-induction/local-resource-host.json','sequential hardware receipt')
 require(report['pending']==['W04 exact semantic/transport/resource freeze and production executor assignment','T02–T06 production source/operation bindings and complete application gates'],'sequential remaining freeze/production scope')
 require(set(report['checker_binaries'])=={'base','affix','aggregate','control','go','private_go'} and all(re.fullmatch('[0-9a-f]{64}',h) for h in report['checker_binaries'].values()),'sequential checker pins')
 require([(r['group'],r['id']) for r in report['rows']]==[(g,i) for g,i,*_ in SELECTION],'sequential representative coverage')
 for row,selection in zip(report['rows'],SELECTION):
  group,identity,kind,_,_=selection;r=reports[group];original=next(c for c in r['cases'] if c['case']['id']==identity)
  require(row['case']==original['case'],'sequential complete candidate identity')
  require(row['checker_scope']==('private proposed Nat.induct extension' if kind=='control' else 'unchanged production checker implementations'),'sequential private/production checker scope')
  require(r['checker_binaries']=={'rust':report['checker_binaries'][kind],'go':report['checker_binaries']['private_go' if kind=='control' else 'go']},'sequential binary/prerequisite crossing')
  raw=(BASE/group/(identity+'.mpcert')).read_bytes();require(row['raw_sha256']==sha(raw) and len(raw)==row['case']['bytes'],'sequential candidate bytes')
  require(row['construction_scope']=='complete candidate group including shared ordinary library; not an isolated single-theorem producer','sequential construction scope')
  require(row['construction']['exit_code']==0,'sequential producer result');resource(row['construction'])
  require(set(row['observations'])=={'rust','go'},'sequential independent checkers')
  try:semantic_expectation(row['case'],row['observations'],declarations=row['case']['declarations'],module=original['observations']['rust']['report']['module'])
  except RuntimeError as e:raise ValueError(str(e)) from e
  for key,v in row['observations'].items():
   require(v['report']['hashes']==original['observations'][key]['report']['hashes'],'sequential accepted hashes');resource(v)
 maxima={key:max(r['case'][field] for r in report['rows']) for key,field in [('certificate_bytes','bytes'),('terms','terms'),('declarations','declarations')]}
 maxima.update({stage:{field:max(r[stage][field] if stage=='construction' else r['observations'][stage][field] for r in report['rows']) for field in ('wall_seconds','peak_rss_bytes')} for stage in ('construction','rust','go')})
 require(report['maxima']==maxima,'sequential observed maxima')
 return len(report['rows'])
