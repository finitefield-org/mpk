# ALPHA-004 Fast Kernel Profile

Schema: `mpk.alpha004.fast_kernel_profile.v0`

Timings are local wall-clock measurements from the Rust fast-kernel profile harness. The workload is an ALPHA-002-sized source-free certificate shape; it does not claim to prove the ALPHA VC obligations.

## Workload

| Field | Value |
| --- | --- |
| ALPHA-002 manifest | `fixtures/vc-alpha/manifest.json` |
| ALPHA-002 obligation count | `1056` |
| Profile module | `Bench.Alpha004.FastKernelProfile` |
| Input bytes | `74781` |
| Certificate hash | `cfb06076ba6afa7360b156458c8eb5b8727f202fb401e5c5d3f6e3ea1e92158c` |
| Declarations | `1056` |
| Proof nodes | `1056` |
| Terms | `2` |

## Stage timings

| Stage | Elapsed ms | Notes |
| --- | ---: | --- |
| decode | 6.186 | canonical decode and re-encode validation |
| typecheck | 972.883 | declaration translation and core checking |
| defeq | 528.272 | nested cache-instrumented conversion calls |
| proof-node checking | 1539.133 | profile-gated proof-node traversal and checking |
| section recompute | 8.270 | export block, axiom report, and hash recomputation |
| total | 2529.916 | end-to-end profile harness time |

## Cache and defeq metrics

| Scope | Operation | Calls | Hits | Misses | Elapsed ms |
| --- | --- | ---: | ---: | ---: | ---: |
| declarations | infer | 2112 | 0 | 2112 | 276.770 |
| declarations | whnf | 0 | 0 | 0 | 0.000 |
| declarations | defeq | 1056 | 0 | 1056 | 37.632 |
| declarations | check | 1056 | 0 | 0 | 686.075 |
| proof nodes | infer | 2112 | 2110 | 2 | 978.311 |
| proof nodes | whnf | 0 | 0 | 0 | 0.000 |
| proof nodes | defeq | 1056 | 1055 | 1 | 490.640 |
| proof nodes | check | 1056 | 0 | 0 | 1026.117 |
| combined | infer | 4224 | 2110 | 2114 | 1255.082 |
| combined | whnf | 0 | 0 | 0 | 0.000 |
| combined | defeq | 2112 | 1055 | 1057 | 528.272 |
| combined | check | 2112 | 0 | 0 | 1712.192 |

## Hotspots identified

1. `proof-node checking` at 1539.133 ms.
2. `typecheck` at 972.883 ms.
3. `defeq` at 528.272 ms.
4. `decode` at 6.186 ms.

The optimization follow-up should start with the largest measured stage above, then inspect nested cache key construction plus defeq hit/miss costs before changing cache layout or proof-node locality. The profile keeps the trust boundary unchanged: it measures only canonical certificate decode, core declaration checking, cached defeq, and profile-gated proof-node checking.
