# Max64 End-to-End Example

## Go source

```go
package example

func Max64(a int64, b int64) int64 {
    if a < b {
        return b
    }
    return a
}
```

## Contract

```json
{
  "schema": "mpk.go.contract.v0",
  "function": "example.Max64",
  "ensures": [
    {"op": "signed_ge", "lhs": {"result": 0}, "rhs": {"var": "a"}},
    {"op": "signed_ge", "lhs": {"result": 0}, "rhs": {"var": "b"}},
    {"op": "or", "args": [
      {"op": "eq", "lhs": {"result": 0}, "rhs": {"var": "a"}},
      {"op": "eq", "lhs": {"result": 0}, "rhs": {"var": "b"}}
    ]}
  ]
}
```

## Conceptual theorem

```text
Theorem example.Max64.correct:
  forall a b : Go.I64,
    let r = GoModel.example.Max64 a b in
      signed_ge r a
    ∧ signed_ge r b
    ∧ (r = a ∨ r = b)
```

## Conceptual VCs

```text
VC_then:
  signed_lt a b -> r = b -> signed_ge r a ∧ signed_ge r b ∧ (r = a ∨ r = b)

VC_else:
  not (signed_lt a b) -> r = a -> signed_ge r a ∧ signed_ge r b ∧ (r = a ∨ r = b)
```

## Expected proof strategy

- Split conjunctions.
- Use branch condition for order facts.
- Use reflexivity for result equality.
- Use linear/signed-order theory certificate for comparisons.

## Trusted result

The example is accepted only after the resulting `.mpcert` checks under the Rust fast kernel and the Go reference checker. The Go source and contract sidecar are not themselves proof evidence.
