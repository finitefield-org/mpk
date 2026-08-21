# GIR v0 Specification

Status: historical; retained only to document the removed pre-VIR interface.

GIR means **Go Verification IR**. It is an untrusted intermediate representation used to generate theorem obligations.

## Core warning

GIR is not proof evidence. GIR becomes meaningful only when the generated theorem statements are encoded in `.mpcert` and checked by MPK.

## Module

```text
GIRModule:
  schema_version
  module_path
  go_version
  frontend_version
  frontend_binary_sha256
  source_files
  packages
  type_decls
  const_decls
  function_decls
  source_hash
  gir_hash
```

## Function

```text
GIRFunction:
  id
  package
  name
  receiver?
  type_params?          # always rejected in MVP; retained only for deterministic rejected-feature diagnostics
  params
  results
  locals
  blocks
  contracts
  supported_features
  rejected_features
```

## Basic block

```text
GIRBlock:
  label
  parameters
  instructions
  terminator
```

## Instructions

```text
GIRInstr ::=
    Const
  | Copy
  | BinOp
  | UnaryOp
  | Convert
  | Phi
  | Field
  | Index
  | MakeStruct
  | MakeArray
  | CallStatic
  | Unsupported
```

MVP excludes heap-sensitive instructions such as general load/store, address-taking, pointer aliasing, map update, slice mutation, goroutine creation, channel operations, defer, panic/recover, reflection, unsafe, and cgo.

## Terminators

```text
GIRTerminator ::=
    Return(values)
  | Jump(label, args)
  | Branch(cond, then_label, else_label)
  | PanicUnsupported(reason)
```

## Integer modeling

Go fixed-width integers must be represented as bitvectors, not mathematical integers. Signed views are explicit interpretations of bitvectors.

```text
int64  -> BV64 with signed comparison/interpretation operators
uint64 -> BV64 with unsigned comparison/interpretation operators
```

## Contract attachment

Contracts are attached by fully qualified function identity:

```text
package_path.function_name
package_path.Type.method_name
```

Method contracts use `package_path.Type.method_name` only after `go2gir` has lowered the receiver to an explicit first argument. Interface dispatch and pointer-receiver aliasing remain rejected MVP features.

MVP contracts support:

- `requires`;
- `ensures`;
- `modifies` as empty for pure functions;
- loop invariant annotations by stable source location or explicit block id;
- decreases clause for total correctness where loops are enabled.

The contract sidecar format used by examples is untrusted JSON:

```json
{
  "schema": "mpk.go.contract.v0",
  "function": "example.Max64",
  "requires": [],
  "ensures": [
    {"op": "signed_ge", "lhs": {"result": 0}, "rhs": {"var": "a"}}
  ],
  "modifies": [],
  "loops": []
}
```

MVP expression atoms are `{"var": "<param-or-local>"}`, `{"result": <index>}`, boolean literals, and fixed-width integer literals tagged with width and signedness. MVP expression operators are `eq`, `not`, `and`, `or`, signed and unsigned integer comparisons, boolean connectives, supported bitvector arithmetic, and explicit conversions. Unknown operators, unresolved names, missing result indexes, non-empty `modifies`, or loop metadata that cannot be tied to a stable block id reject before GIR emission.

`requires`, `modifies`, and `loops` default to empty arrays when omitted. `ensures` must be present and non-empty for a function to produce verification-condition obligations; a contract with no postcondition is rejected by the VC generator rather than silently proving only runtime safety.

## Supported Go subset v0

Allowed:

- pure top-level functions;
- bool;
- int8/int16/int32/int64;
- uint8/uint16/uint32/uint64;
- fixed arrays;
- structs without pointers;
- local variables;
- if/else;
- return;
- static calls to supported pure functions;
- for loops only with explicit invariant metadata; variant/decreases metadata is required only when total correctness is claimed.

Rejected:

- unsafe;
- cgo;
- reflection;
- interface dynamic dispatch;
- goroutines;
- channels;
- defer;
- panic/recover;
- maps;
- mutable slices;
- pointer aliasing;
- floating point;
- complex numbers;
- generics in MVP;
- init functions;
- package-level mutable state.

## GIR validation

The `go2gir` tool must emit a rejected-feature report. Unsupported features fail closed and do not silently approximate semantics.
