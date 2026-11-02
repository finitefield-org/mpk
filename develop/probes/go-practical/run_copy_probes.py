#!/usr/bin/env python3
"""Measure full-word overlapping copy ingredients and computed source guards."""
import json
from pathlib import Path
import sys

import copy_differential
import run_prefix_probes as shared
from run_access_probes import declaration_count as access_declarations
from run_proof_pilot import DEFAULT_GOROOT,HERE,ROOT,checked,semantic_expectation,sha

RECORD=ROOT/'develop/migrations/go-practical/t01-w03/bulk-copy'
COUNTS=copy_differential.CAPACITIES
GENERATE_COMMAND='copy-generate'
INDEX_SCHEMA='mpk.go_practical.copy_pilot.v0'
REPORT_SCHEMA='mpk.go_practical.copy_report.v0'
ATTEMPT_SCHEMA='mpk.go_practical.copy_attempt.v0'
STATUS='general copy ingredients measured; W03 remains incomplete'
SCOPE='ordinary full64 min-length copy, computed descriptor validity to source/destination guards, immutable pre-state whole-cell reads and destination/other-region frames including overlap; production capture/ownership, remaining operations and aggregate closure stay pending'
PENDING=['production Go capture/model/ownership/operation bindings','bounded construction/append','remaining byte/control composition','aggregate boundary and resource freeze']
CASE_SCOPE='ordinary min-length copy with computed source/destination guards, immutable pre-state reads and whole-cell frames; production capture/ownership/remaining operations stay pending'
BOUNDARY_SCOPE='semantic storage bounds 0/1/8/32/127/128/129 are private workloads; power-of-two padding never raises admission or represents a Go panic or frozen budget'
EXTRA_FILES=copy_differential.ARTIFACTS
EXTRA_REPORT_KEY='copy_observations'

def proof_sources():
 result={r['path']:r['sha256'] for r in shared.proof_sources()}
 for p in (Path(__file__),HERE/'check_copy_report.py',HERE/'copy_differential.py',HERE/'copy_reference.go',HERE/'access_differential.py',HERE/'slice_differential.py',HERE/'order_differential.py',HERE/'byte_differential.py',HERE/'run_access_probes.py'):
  result[p.relative_to(ROOT).as_posix()]=sha(p.read_bytes())
 return [{'path':p,'sha256':h} for p,h in sorted(result.items())]

def depth(n):return max(n-1,0).bit_length()
def padding(n):return 1<<depth(n)
def declaration_count(n):return access_declarations(n)+195
def expected_cases():return [(n,f,m) for n in COUNTS for f in copy_differential.families(n) for m in range(2)]
def identifier(n,family,mutation):
 return f"copy-{n}-{family}-{'valid' if mutation==0 else copy_differential.label(family)}"

def verify_case(case,observations):
 semantic_expectation(case,observations,declarations=declaration_count(case['capacity']),module=f"Probe.GoPractical.Word.SliceBulkCopyCapacity{case['capacity']}")
 if case['expected']=='accepted':return 'accepted_zero_axiom'
 rust=json.loads(observations['rust']['report']['error_detail'])['code'];go=observations['go']['report']['error_detail']
 if rust=='CORE_TYPE_MISMATCH' and go.startswith('term ') and ' inferred ' in go and ' but expected ' in go:return 'core_type_mismatch'
 raise RuntimeError('copy proof did not fail at the intended type boundary; resource failures never establish a false claim')

def measure_extra(directory,env):
 exe=directory/'copy-observer'
 checked([str(DEFAULT_GOROOT/'bin/go'),'build','-trimpath','-buildvcs=false','-o',str(exe),str(HERE/'copy_reference.go')],env=env)
 return copy_differential.run(exe,directory)

def check_retained(report):
 from check_copy_report import check
 return check(report)

if __name__=='__main__':shared.main(sys.modules[__name__])
