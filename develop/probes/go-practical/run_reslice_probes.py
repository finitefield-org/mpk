#!/usr/bin/env python3
"""Measure actual source-sized two/three-index slice operation ingredients."""
import json
from pathlib import Path
import sys

import reslice_differential
import run_prefix_probes as shared
from run_access_probes import declaration_count as access_declarations
from run_proof_pilot import DEFAULT_GOROOT,HERE,ROOT,checked,semantic_expectation,sha

RECORD=ROOT/'develop/migrations/go-practical/t01-w03/reslicing'
COUNTS=reslice_differential.CAPACITIES
KINDS=reslice_differential.KINDS
FAMILIES=reslice_differential.FAMILIES
GENERATE_COMMAND='reslice-generate'
INDEX_SCHEMA='mpk.go_practical.reslice_pilot.v0'
REPORT_SCHEMA='mpk.go_practical.reslice_report.v0'
ATTEMPT_SCHEMA='mpk.go_practical.reslice_attempt.v0'
STATUS='general reslice ingredients measured; W03 remains incomplete'
SCOPE='ordinary actual signed two/three-index guards preserve computed descriptor validity, exact capacity, mapped source guards and complete backing reads; production capture/ownership/remaining operation and aggregate closure stay pending'
PENDING=['production Go capture/model/ownership/operation bindings','overlapping copy and bounded construction/append','remaining byte/control composition','aggregate boundary and resource freeze']
CASE_SCOPE='ordinary actual signed two/three-index guards preserve computed validity, capacity and full-index/backing reads; production Go capture/ownership/remaining operations stay pending'
BOUNDARY_SCOPE='semantic storage bounds 0/1/8/32/127/128/129 are private workloads; power-of-two padding never raises admission or represents a Go panic or frozen budget'
EXTRA_FILES=reslice_differential.ARTIFACTS
EXTRA_REPORT_KEY='reslice_observations'

def proof_sources():
 result={r['path']:r['sha256'] for r in shared.proof_sources()}
 for p in (Path(__file__),HERE/'check_reslice_report.py',HERE/'reslice_differential.py',HERE/'reslice_reference.go',HERE/'access_differential.py',HERE/'slice_differential.py',HERE/'order_differential.py',HERE/'byte_differential.py',HERE/'run_access_probes.py'):
  result[p.relative_to(ROOT).as_posix()]=sha(p.read_bytes())
 return [{'path':p,'sha256':h} for p,h in sorted(result.items())]

def depth(n):return max(n-1,0).bit_length()
def padding(n):return 1<<depth(n)
def declaration_count(n):return access_declarations(n)+174
def expected_cases():return [(n,k,f,m) for n in COUNTS for k in KINDS for f in FAMILIES if n or f=='validity' for m in range(2)]
def identifier(n,kind,family,mutation):
 label='valid' if mutation==0 else {'validity':'erased-range-guard','capacity':'wrong-capacity','mapped-guard':'original-length-guard','read':'zeroed-read'}[family]
 return f'reslice-{n}-{kind}-{family}-{label}'

def verify_case(case,observations):
 semantic_expectation(case,observations,declarations=declaration_count(case['capacity']),module=f"Probe.GoPractical.Word.SliceResliceCapacity{case['capacity']}")
 if case['expected']=='accepted':return 'accepted_zero_axiom'
 rust=json.loads(observations['rust']['report']['error_detail'])['code'];go=observations['go']['report']['error_detail']
 if rust=='CORE_TYPE_MISMATCH' and go.startswith('term ') and ' inferred ' in go and ' but expected ' in go:return 'core_type_mismatch'
 raise RuntimeError('reslice proof did not fail at the intended type boundary; resource failures never establish a false claim')

def measure_extra(directory,env):
 exe=directory/'reslice-observer'
 checked([str(DEFAULT_GOROOT/'bin/go'),'build','-trimpath','-buildvcs=false','-o',str(exe),str(HERE/'reslice_reference.go')],env=env)
 return reslice_differential.run(exe,directory)

def check_retained(report):
 from check_reslice_report import check,check_initial
 check_initial()
 return check(report)

if __name__=='__main__':shared.main(sys.modules[__name__])
