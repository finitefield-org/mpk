# Original-source exception representation domains

The two positive sources define sealed exceptions with an empty source payload
and a payload containing int, Bool and char fields. Both are thrown directly by
an actual selected C# root. The third source attempts to retain System.Exception
in a field of another exception and rejects with
`CSHARP_PRACTICAL_TYPE/exception_value_api`, zero artifacts and no captured facts.
The existing frontend forbids escaping exception objects and exception-valued
APIs; the test does not relax that rule to manufacture a nested positive.

The private request test regenerates all three requests byte-for-byte. Responses
were freshly captured using the existing control-emission harness, which checks
capture determinism with a second capture. The ordinary data-phase harness
correctly rejects the two positive sources with `closed_type` because it does
not enable exception control; that initial route is recorded separately and is
not acceptance evidence. No frontend, build manifest or checker rule changed.

```sh
MPK_W09_EXCEPTION_REQUESTS_OUT=/tmp/mpk-w09-exception-domain-requests-v2.json \
  cargo test -p mpk-vc --test csharp_practical_vc \
  csharp_03_t06_w09_domain_exception_source_requests -- --nocapture

docker run --rm -i --platform linux/amd64 --network none \
  --security-opt seccomp=unconfined --cap-add SYS_ADMIN \
  --mount type=bind,source=/Users/kazuyoshitoshiya/mpk,target=/repo,readonly \
  --workdir /repo mpk-java-t10-gate:local \
  ./scripts/build-csharp-practical-frontend.sh --test-control-emission-requests \
  < /tmp/mpk-w09-exception-domain-requests-v2.json \
  > /tmp/mpk-w09-exception-domain-responses-v3.json
```

The source-level rejection is checked before the two accepted captures enter
ValidatedDataSource and normal VIR emission. The user-payload domain test
observes logical counts 2 and 5: a closed exception adds one, and its source
payload is itself a product with its own logical cell. Eight mutations reject
empty-product storage, sum/tag padding, unused active product roles, Bool/char
field padding and a built-in tag retaining payload bits. These are representation
domain checks; source constructor/public clauses and application VC proofs
remain separate work.
