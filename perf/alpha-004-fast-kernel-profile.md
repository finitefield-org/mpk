# ALPHA-004 Fast Kernel Profile

Schema: `mpk.alpha004.fast_kernel_profile.v0`

Timings are local wall-clock measurements from the Rust fast-kernel profile harness. The workload is an ALPHA-002-sized source-free certificate shape; it does not claim to prove the ALPHA VC obligations.

## Workload

| Field | Value |
| --- | --- |
| ALPHA-002 manifest | `fixtures/vc-alpha/manifest.json` |
| Active VC member count | `33` |
| Profile module | `Bench.Alpha004.FastKernelProfile` |
| Input bytes | `2429` |
| Certificate hash | `79211f2c149b51cca590e2a56d2605e2aea48c837cc28f1304960e1579e6d383` |
| Declarations | `33` |
| Proof nodes | `33` |
| Terms | `2` |

## Stage timings

| Stage | Elapsed ms | Notes |
| --- | ---: | --- |
| decode | 0.183 | canonical decode and re-encode validation |
| typecheck | 0.496 | declaration translation and core checking |
| defeq | 0.035 | nested cache-instrumented conversion calls |
| proof-node checking | 0.146 | profile-gated proof-node traversal and checking |
| section recompute | 0.641 | export block, axiom report, and hash recomputation |
| total | 1.752 | end-to-end profile harness time |

## Cache and defeq metrics

| Scope | Operation | Calls | Hits | Misses | Elapsed ms |
| --- | --- | ---: | ---: | ---: | ---: |
| declarations | infer | 66 | 0 | 66 | 0.145 |
| declarations | whnf | 0 | 0 | 0 | 0.000 |
| declarations | defeq | 33 | 0 | 33 | 0.004 |
| declarations | check | 33 | 0 | 0 | 0.154 |
| proof nodes | infer | 66 | 64 | 2 | 0.076 |
| proof nodes | whnf | 0 | 0 | 0 | 0.000 |
| proof nodes | defeq | 33 | 32 | 1 | 0.030 |
| proof nodes | check | 33 | 0 | 0 | 0.080 |
| combined | infer | 132 | 64 | 68 | 0.222 |
| combined | whnf | 0 | 0 | 0 | 0.000 |
| combined | defeq | 66 | 32 | 34 | 0.035 |
| combined | check | 66 | 0 | 0 | 0.235 |

## Hotspots identified

1. `typecheck` at 0.496 ms.
2. `decode` at 0.183 ms.
3. `proof-node checking` at 0.146 ms.
4. `defeq` at 0.035 ms.

The optimization follow-up should start with the largest measured stage above, then inspect nested cache key construction plus defeq hit/miss costs before changing cache layout or proof-node locality. The profile keeps the trust boundary unchanged: it measures only canonical certificate decode, core declaration checking, cached defeq, and profile-gated proof-node checking.
