#!/usr/bin/env python3
"""Rebuild/replay a private proposed Nat-induction extension in two checkers."""
import argparse
import json
import os
from pathlib import Path
import platform
import shutil
import tempfile
import control_foundation_build as build
import control_differential as native
from run_proof_pilot import DEFAULT_GOROOT,HERE,ROOT,checked,json_bytes,observe,sha,timed,verify_host_toolchain,semantic_expectation
RECORD=ROOT/'develop/migrations/go-practical/t01-w03/control-induction'
TARGET=Path('/private/tmp/mpk-go-practical-control-target')
SCOPE='private proposed additive monomorphic Sort0 Nat.induct; independently extended mirrored Rust/Go checkers; arbitrary-Nat full-frame invariant/control proof schemas; production checkers and public profile unchanged'
PENDING=['W03 isolated resource envelope and cumulative exit review','freeze the additive core/foundation interface and executor/migration ownership in W04','production Go source/loop/variant/operation bindings in T02–T06']
FAMILIES=('invariant-entry','invariant-step','absorbing-stop','continue-post','break-exit','return-precedence')
VARIANTS=('legacy-no-induct','wrong-interface','nongenerated','wrong-zero-name','wrong-succ-type','nongenerated-zero','extra-constructor-before','extra-constructor-after','higher-universe-family','induct-levels-partial','family-levels','constructor-levels','prior-opaque-levels','prior-theorem-levels','unused-term-levels')
CASE_SCOPE='ordinary loop/control proof schema; entry and body preservation are proof obligations, never caller entry assumptions; production source and variant bindings remain due in T06'
INTERFACE_SCOPE='private proposed Nat.induct interface validation; malformed declaration rejection is distinct from semantic false-claim rejection'
def proof_sources():
 result={r['path']:r['sha256'] for r in build.sources()}
 for p in [Path(__file__),HERE/'check_control_report.py',HERE/'control_differential.py',HERE/'control_reference.go',HERE/'run_proof_pilot.py',HERE/'order_differential.py',HERE/'check_balanced_report.py']:
  result[p.relative_to(ROOT).as_posix()]=sha(p.read_bytes())
 return [{'path':p,'sha256':h} for p,h in sorted(result.items())]
def inventory(files):return [{'path':p,'sha256':sha(raw)} for p,raw in sorted(files.items())]
def classification(case,obs,interface=False):
 if set(obs)!={'rust','go'}:raise ValueError('control independent checker coverage')
 for k,v in obs.items():
  r=v['report']
  if r['verdict']!=case['expected']:raise ValueError('control unexpected verdict')
 if case['expected']=='accepted':
  semantic_expectation(case,obs,declarations=case['declarations'],module='Probe.GoPractical.ControlInduction');return 'accepted_zero_axiom'
 rust=json.loads(obs['rust']['report']['error_detail'])['code'];go=obs['go']['report']['error_detail']
 if any(v['exit_code']!=1 for v in obs.values()) or obs['rust']['report']['error_code']!='KERNEL_CORE_CHECK' or obs['go']['report']['error_kind']!='core_check':raise ValueError('control rejection boundary')
 if interface:
  expected='CORE_NOT_A_FUNCTION' if case['variant']=='wrong-succ-type' else 'CORE_INVALID_DECLARATION'
  if rust!=expected or (case['variant']=='wrong-succ-type' and 'non-pi function type' not in go) or (case['variant']!='wrong-succ-type' and not any(x in go for x in ('Nat induction','universe argument'))):raise ValueError('control malformed interface classification')
  return 'malformed_interface_rejection'
 if rust!='CORE_TYPE_MISMATCH' or not go.startswith('term ') or ' inferred ' not in go or ' but expected ' not in go:raise ValueError('control resource failure is not a false claim')
 return 'core_type_mismatch'
def main():
 parser=argparse.ArgumentParser(description=__doc__);g=parser.add_mutually_exclusive_group(required=True);g.add_argument('--write',action='store_true');g.add_argument('--check',action='store_true');args=parser.parse_args()
 if platform.system()!='Darwin' or platform.machine()!='arm64':raise RuntimeError('private host/resource units require Darwin arm64')
 before=proof_sources();pin=json.loads((HERE/'host-toolchain.json').read_text());toolchain=verify_host_toolchain(DEFAULT_GOROOT,pin)
 with tempfile.TemporaryDirectory(prefix='mpk-control-build-',dir='/private/tmp') as tmp:
  temp=Path(tmp);workspace=temp/'workspace';receipt=build.prepare(workspace)
  env={**os.environ,'CARGO_TARGET_DIR':str(TARGET)}
  built=checked([shutil.which('cargo'),'build','--release','--offline','--locked','--message-format=json','--manifest-path',str(workspace/'Cargo.toml'),'-p','go-practical-control-probe'],env=env)
  events=[json.loads(x) for x in built.splitlines() if x.startswith('{')];outputs=[e['out_dir'] for e in events if e.get('reason')=='build-script-executed' and 'go-practical-control-probe@' in e.get('package_id','')]
  if len(outputs)!=1:raise RuntimeError('control Cargo generated input identity')
  generated=Path(outputs[0])/'predecessor';observed={p.relative_to(generated).as_posix():p.read_bytes() for p in generated.rglob('*') if p.is_file()}
  if observed!=build.producer_files():raise RuntimeError('control generated producer recipe differs')
  for row in receipt['generated']:
   if sha((workspace/row['path']).read_bytes())!=row['sha256']:raise RuntimeError('control mirrored checker source changed')
  producer=TARGET/'release/go-practical-control-probe';reference=temp/'checker'
  go_env={**os.environ,'GOENV':'off','GOWORK':'off','GOTOOLCHAIN':'local','GOPROXY':'off','GOTELEMETRY':'off','CGO_ENABLED':'0','GOCACHE':'/private/tmp/mpk-go-practical-checker-cache'}
  checked([str(DEFAULT_GOROOT/'bin/go'),'build','-trimpath','-buildvcs=false','-o',str(reference),'./cmd/mpk-checker-ref'],cwd=workspace/'go-tools/mpk-checker-ref',env=go_env)
  binaries={'rust':sha(producer.read_bytes()),'go':sha(reference.read_bytes())}
  dirs=[temp/'first',temp/'second'];generations=[timed([producer,'control-generate',ROOT,p]) for p in dirs]
  if any(g['exit_code'] for g in generations):raise RuntimeError('control generation failed')
  names=sorted(p.name for p in dirs[0].iterdir())
  if names!=sorted(p.name for p in dirs[1].iterdir()) or any((dirs[0]/n).read_bytes()!=(dirs[1]/n).read_bytes() for n in names):raise RuntimeError('control generation not deterministic')
  expected=[];all_rows=[]
  for index,interface in [('index.json',False),('interfaces.json',True)]:
   rows=[];cases=json.loads((dirs[0]/index).read_text())['cases']
   for c in cases:
    path=dirs[0]/(c['id']+'.mpcert');raw=path.read_bytes();obs={k:observe(exe,path) for k,exe in [('rust',producer),('go',reference)]};kind=classification(c,obs,interface)
    row={'case':c,'classification':kind,'raw_sha256':sha(raw),'observations':obs};rows.append(row);print(c['id']+': '+kind,flush=True)
   all_rows.append(rows);expected.extend(c['id']+'.mpcert' for c in cases)
  if names!=sorted(expected+['index.json','interfaces.json']):raise RuntimeError('control candidate/index coverage')
  native_exe=temp/'observer';checked([str(DEFAULT_GOROOT/'bin/go'),'build','-trimpath','-buildvcs=false','-o',str(native_exe),str(HERE/'control_reference.go')],env=go_env);extra=native.run(native_exe,temp)
  if before!=proof_sources() or binaries!={'rust':sha(producer.read_bytes()),'go':sha(reference.read_bytes())}:raise RuntimeError('control source/binary changed during measurement')
  report={'schema':'mpk.go_practical.control_report.v0','work_item':'GO-PRAC-T01-W03','status':'private proposed induction/control feasibility measured; W03 cumulative exit review remains open','scope':SCOPE,'pending':PENDING,'inputs':before,'mirrored_checker_inputs':receipt['generated'],'generated_producer_inputs':inventory(observed),'checker_binaries':binaries,'host_toolchain_inventory':toolchain,'build':'private mirrored Rust release offline locked and Go1.25.0 trimpath; production unchanged','host':{'system':platform.system(),'release':platform.release(),'machine':platform.machine(),'cpu_count':os.cpu_count()},'child_ceiling_seconds':60,'ceiling_scope':'private measurements, not profile budgets','generations':generations,'cases':all_rows[0],'interface_cases':all_rows[1],'native_observations':extra,'outcomes':{'accepted_zero_axiom':6,'core_type_mismatch':6,'interface_accepted':2,'malformed_interface_rejection':13}}
  if args.write:
   RECORD.mkdir(exist_ok=True)
   for name in names:shutil.copyfile(dirs[0]/name,RECORD/name)
   for name in native.ARTIFACTS:shutil.copyfile(temp/name,RECORD/name)
   (RECORD/'observations.json').write_bytes(json_bytes(report))
  from check_control_report import check
  retained=json.loads((RECORD/'observations.json').read_text());check(retained)
  if retained['checker_binaries']!=binaries:raise RuntimeError('private checker binary identity differs from retained measurement')
  if [r['case'] for r in retained['cases']]!=[r['case'] for r in all_rows[0]] or [r['case'] for r in retained['interface_cases']]!=[r['case'] for r in all_rows[1]]:raise RuntimeError('control retained case index differs')
  if args.check and (any((RECORD/n).read_bytes()!=(dirs[0]/n).read_bytes() for n in names) or any((RECORD/n).read_bytes()!=(temp/n).read_bytes() for n in native.ARTIFACTS)):raise RuntimeError('control replay bytes differ')
  print('PASS: 12 ordinary claims and 15 proposed-interface cases in both private checkers; 576 original-Go observations; production unchanged',flush=True)
if __name__=='__main__':main()
