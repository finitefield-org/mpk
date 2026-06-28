# BitVec Ground Certificate Schema v0

Status: reviewed schema for TH-003. This document defines the v0 payload shape
for checked BitVec ground-normalization certificates. It is not a new MPK
certificate format version; the payload is intended to be stored inside the
`TheoryCertificate` table described by `CERT_V0.md` when TH-006 enables theory
proof nodes.

## Scope

The schema covers ground fixed-width bitvector expressions only. It has no
variables, binders, quantifiers, uninterpreted functions, solver status fields,
tactic traces, source locations, comments, or AI-generated hints.

The checker must recompute the normalized result with the TH-002 ground
evaluator. A certificate is accepted only when the encoded claim matches the
recomputed result and the surrounding `Theory` proof node requests a compatible
expected core type.

Supported widths are exactly:

- `8`
- `16`
- `32`
- `64`

Supported operations match the stable `Std.BitVec.BV{width}` hooks:

- unary: `not`, `neg`
- binary bitvector result: `and`, `or`, `xor`, `add`, `sub`, `mul`, `shl`,
  `lshr`, `ashr`
- unsigned comparisons: `ult`, `ule`, `ugt`, `uge`
- signed comparisons: `slt`, `sle`, `sgt`, `sge`

## Trust Boundary

The payload is a certificate, not an oracle answer. The checker must not trust:

- external solver `sat` or `unsat` results;
- a claimed normalized bitvector or boolean result;
- operation trace output fields;
- display names, source terms, or frontend metadata.

The checker may use the trace only as deterministic audit material. Acceptance
depends on decoding, recomputation, and equality with the final claim.

## Logical Layout

```text
BitVecGroundCertificate:
  format = "mpk.bitvec-ground.v0"
  expr: BitVecExpr
  trace: BitVecTraceStep[]
  result: BitVecResult
```

`expr` is the root expression. `trace` records a post-order operation trace.
`result` is the claimed normalized value for the root.

## Expression Schema

```text
BitVecExpr ::=
    Literal(width, bits)
  | Unary(op, value)
  | Binary(op, lhs, rhs)
  | Compare(op, lhs, rhs)

BitVecResult ::=
    BitVec(width, bits)
  | Bool(value)
```

Rules:

- `width` must be one of `8`, `16`, `32`, or `64`.
- `bits` is an unsigned integer normalized modulo `2^width`.
- Binary payloads encode exactly `width / 8` bytes, so decoded bits are already
  in range. Non-binary authoring tools must canonicalize larger source integers
  before encoding.
- `Unary` always consumes one `BitVecResult::BitVec` and returns a bitvector of
  the same width.
- `Binary` bitwise and arithmetic operations require equal operand widths and
  return a bitvector of that width.
- `shl`, `lshr`, and `ashr` return a bitvector with the lhs width. The rhs is a
  bitvector shift count and may have any supported width.
- `Compare` requires equal operand widths and returns `Bool`.
- Signed comparison and `ashr` interpret the same raw bits as two's-complement
  values. Signedness is an operation-level view, not a payload variant.

## Trace Schema

```text
BitVecTraceStep:
  step_id: u32
  expr_path: Path
  op: TraceOp
  inputs: BitVecResult[]
  output: BitVecResult

TraceOp ::=
    literal
  | not | neg
  | and | or | xor | add | sub | mul
  | shl | lshr | ashr
  | ult | ule | ugt | uge
  | slt | sle | sgt | sge

Path ::= PathSegment[]
PathSegment ::= UnaryValue | BinaryLhs | BinaryRhs | CompareLhs | CompareRhs
```

Trace rules:

- `step_id` values start at `0`, increase by one, and are unique.
- Steps are in post-order: children appear before the parent expression.
- The final step must correspond to `Root`.
- `output` for each step must equal the evaluator result for the expression at
  `expr_path`.
- The final step output must equal `result`.
- Missing, duplicate, out-of-order, or mismatched trace steps reject.

The trace is intentionally redundant. It lets future diagnostic tooling explain
normalization, but it does not reduce the recomputation requirement.

## Canonical Binary Payload

The canonical payload for `TheoryCertificate.payload` is:

```text
magic:             8 bytes = "MPKBVGC0"
format_tag:        u8 = 0
expr:              encoded BitVecExpr
trace_len:         minimal unsigned LEB128 u32
trace_steps:       encoded BitVecTraceStep[trace_len]
result:            encoded BitVecResult
```

All multi-byte fixed-width integer fields use big-endian byte order. All
variable-width unsigned integer fields use the minimal unsigned LEB128 rule from
`CERT_V0.md`.

### Encoded Expressions

```text
Expr tag:
  0x00 Literal
  0x01 Unary
  0x02 Binary
  0x03 Compare

Width tag:
  0x08 BV8
  0x10 BV16
  0x20 BV32
  0x40 BV64

Literal:
  expr_tag = 0x00
  width_tag
  bits: width / 8 bytes, big-endian

Unary:
  expr_tag = 0x01
  unary_op_tag
  value: Expr

Binary:
  expr_tag = 0x02
  binary_op_tag
  lhs: Expr
  rhs: Expr

Compare:
  expr_tag = 0x03
  comparison_op_tag
  lhs: Expr
  rhs: Expr
```

Operation tags:

```text
Unary:
  0x00 not
  0x01 neg

Binary:
  0x00 and
  0x01 or
  0x02 xor
  0x03 add
  0x04 sub
  0x05 mul
  0x06 shl
  0x07 lshr
  0x08 ashr

Comparison:
  0x00 ult
  0x01 ule
  0x02 ugt
  0x03 uge
  0x04 slt
  0x05 sle
  0x06 sgt
  0x07 sge
```

### Encoded Results

```text
Result tag:
  0x00 BitVec
  0x01 Bool

BitVec result:
  result_tag = 0x00
  width_tag
  bits: width / 8 bytes, big-endian

Bool result:
  result_tag = 0x01
  value: 0x00 false | 0x01 true
```

### Encoded Trace Steps

```text
TraceStep:
  step_id: minimal unsigned LEB128 u32
  path: encoded Path
  op_tag_domain: u8
  op_tag: u8
  input_len: u8
  inputs: Result[input_len]
  output: Result

op_tag_domain:
  0x00 literal
  0x01 unary
  0x02 binary
  0x03 comparison
```

`op_tag` is `0x00` for `literal`; otherwise it uses the operation tags defined
above for its domain. `input_len` must be `0` for literal, `1` for unary, and
`2` for binary or comparison.

Path encoding:

```text
Path segment:
  0x01 UnaryValue
  0x02 BinaryLhs
  0x03 BinaryRhs
  0x04 CompareLhs
  0x05 CompareRhs

Path:
  segment_count: minimal unsigned LEB128 u32
  segments: PathSegment[segment_count]
```

The root path has `segment_count = 0`. Child path segments are listed from root
to leaf.

## Resource Limits

The v0 checker must use deterministic limits:

- expression nodes: maximum `256`
- trace steps: exactly one step per expression node
- encoded payload size: implementation-defined release limit, rejected before
  decoding if exceeded
- no recursion or reduction path may accept by timeout

If an implementation uses an iterative decoder, it must preserve the same
post-order trace and node-count semantics.

## Rejection Conditions

Reject if:

- magic, format tag, enum tag, boolean byte, width tag, varint, or field order is
  invalid;
- expression node count exceeds the limit;
- a non-ground feature appears;
- operand result kinds do not match the operation;
- operand widths violate the operation rules;
- a trace step is missing, duplicated, out of order, or attached to the wrong
  path;
- a trace step input or output disagrees with recomputation;
- the final trace output does not match `result`;
- `result` disagrees with recomputing `expr`;
- trailing bytes remain after decoding.

## Example

BV8 `1 + 1 = 2` is represented as:

```text
expr =
  Binary(add,
    Literal(BV8, 0x01),
    Literal(BV8, 0x01))

trace =
  0: path BinaryLhs,  literal -> BitVec(BV8, 0x01)
  1: path BinaryRhs,  literal -> BitVec(BV8, 0x01)
  2: path Root,       add(BitVec(BV8, 0x01), BitVec(BV8, 0x01)) -> BitVec(BV8, 0x02)

result = BitVec(BV8, 0x02)
```

The checker accepts only after recomputing the root expression to
`BitVec(BV8, 0x02)` and confirming the trace and final claim.
