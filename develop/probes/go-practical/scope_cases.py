"""Original Go snippets and draft expectations; no profile admission executor.

All snippets are type-checked intact. The selected root and profile outcome are
design vectors, not observations made by a production practical frontend.
"""
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
HERE = Path(__file__).resolve().parent
RECORD = ROOT / "develop/migrations/go-practical/t01-w04/scope-observations"
VECTORS = ROOT / "develop/migrations/go-practical/t01-w04/source-vectors.json"
SCOPE = ("private original-Go type/runtime observations for W04 draft vectors; "
         "no practical frontend admission, source/model proof, Linux release or public-profile acceptance")
PENDING = [
    "W04 exact operation/overlay/wire/limit/migration freeze and review",
    "T02–T06 production source/type/closure/error/slice/loop bindings and both-checker application proofs",
    "W05 complete T01 gate; T07 installed-release activation",
]


def case(identity, source, rule, outcome="accepted_model", *, status="ok",
         root="F", error_contains=None, types=None, calls=None, contract=None):
    return {
        "id": identity, "source": "package p\n" + source.strip() + "\n",
        "selected_root": root, "draft_rule": rule, "draft_outcome": outcome,
        "contract_context": contract or {},
        "native_expectation": {
            "status": status, "error_contains": error_contains,
            "type_assertions": types or {}, "call_assertions": calls or [],
        },
    }


CASES = [
    case("root-slice-aggregate", """
type Line struct { Amount int64; SKU string }
type Rows []Line
type Root struct { Lines Rows; Nested [2]Rows }
func F(r Root) int { c := r; a := c.Nested; return len(c.Lines)+cap(a[0]) }
""", "root-aggregate",
         types={"Rows": {"alias": False}, "Root": {"alias": False}}),
    case("root-descriptor-rebind", """
type Root struct { Values []int64 }
func F(r Root) Root { c := r; c.Values = []int64{23}; return c }
""", "root-aggregate"),
    case("root-fixed-array-of-slices", """
func F(r [2][]int64) int { c := r; return len(c[0])+cap(c[1]) }
""", "root-aggregate"),
    case("plain-struct-slice-cell", """
type Line struct { Amount int64; SKU string; Flags [2]bool }
func F(s []Line) int64 { if len(s)==0 { return 0 }; return s[0].Amount }
""", "plain-cell"),
    case("nested-slice-cell", """
func F(s [][]int64) int { return len(s) }
""", "plain-cell", "source_rejection"),
    case("nested-struct-slice-cell", """
type Box struct { Values []int64 }
func F(s []Box) int { return len(s) }
""", "plain-cell", "source_rejection"),
    case("nested-array-slice-cell", """
func F(s [][2][]int64) int { return len(s) }
""", "plain-cell", "source_rejection"),
    case("recursive-slice-element", """
type Node struct { Children []Node }
func F(n Node) int { return len(n.Children) }
""", "closed-type-graph", "source_rejection"),
    case("named-and-alias-types", """
type Amount int
type Status bool
type Label string
type Rows []Amount
type Alias = Amount
type Bytes = []byte
func F(n Alias, r Rows, b Bytes, label Label, ok Status) (Amount, Rows) {
    if ok && len(label)>0 && len(b)>0 { return Amount(n)+Amount(b[0]), append(r, Rows{}...) }
    return n, r
}
""", "nominal",
         types={"Amount": {"alias": False, "underlying": "int"},
                "Status": {"alias": False, "underlying": "bool"},
                "Label": {"alias": False, "underlying": "string"},
                "Alias": {"alias": True, "underlying": "int"},
                "Bytes": {"alias": True, "underlying": "[]byte"}}),
    case("nominal-assignment-confusion", """
type A int64
type B int64
func F(a A) B { return a }
""", "nominal", "source_rejection", status="source_type_error",
         error_contains="cannot use a"),
    case("integer-constant-overflow", """
func F() int8 { return 128 }
""", "nominal", "source_rejection", status="source_type_error",
         error_contains="overflows"),
    case("uintptr-is-outside-v1", """
func F(n uintptr) uintptr { return n }
""", "excluded-type", "source_rejection"),
    case("float-is-outside-v1", """
func F(n float64) float64 { return n }
""", "excluded-type", "source_rejection"),
    case("bool-value-error", """
type Flag bool
func (Flag) Error() string { return "flag" }
func F() error { return Flag(false) }
""", "closed-error",
         types={"Flag": {"value_error": True, "pointer_error": True}},
         calls=[{"text": "Flag(false)", "kind": "conversion",
                 "target": "mpk.private.w04/bool-value-error.Flag"}]),
    case("integer-value-error", """
type Code int64
func (Code) Error() string { return "code" }
func F() (int64, error) { return 7, Code(0) }
""", "closed-error",
         types={"Code": {"value_error": True, "pointer_error": True}}),
    case("plain-struct-error", """
type Failure struct { Code int64; Message string }
func (Failure) Error() string { return "failure" }
func F() error { return Failure{} }
""", "closed-error",
         types={"Failure": {"value_error": True, "pointer_error": True}}),
    case("root-slice-error", """
type Failure struct { Values []int64 }
func (Failure) Error() string { return "failure" }
func F(s []int64) error { return Failure{Values:s} }
""", "closed-error-root-payload",
         types={"Failure": {"value_error": True, "pointer_error": True}}),
    case("plain-closed-error-cells", """
type Code int64
func (Code) Error() string { return "code" }
func F(s []error) int { return len(s) }
""", "plain-closed-error-cell", contract={"entry_error_types": ["Code"]}),
    case("reference-closed-error-cells", """
type Failure struct { Values []int64 }
func (Failure) Error() string { return "failure" }
func F(s []error) int { return len(s) }
""", "plain-closed-error-cell", "source_rejection",
         contract={"entry_error_types": ["Failure"]}),
    case("recursive-error-payload", """
type Failure struct { Cause error }
func (Failure) Error() string { return "failure" }
func F() error { return Failure{} }
""", "closed-type-graph", "source_rejection"),
    case("string-concrete-error", """
type Failure string
func (Failure) Error() string { return string(Failure("")) }
func F() error { return Failure("") }
""", "closed-error-kind", "source_rejection",
         types={"Failure": {"value_error": True, "pointer_error": True}}),
    case("pointer-concrete-error", """
type Code int64
func (Code) Error() string { return "code" }
func F() error { var p *Code; return p }
""", "pointer-error", "source_rejection"),
    case("wrong-error-result-type", """
type Message string
type Code int64
func (Code) Error() Message { return "" }
func F() error { return Code(0) }
""", "closed-error", "source_rejection", status="source_type_error",
         error_contains="does not implement error"),
    case("boxing-signature-only", """
type Code int64
var calls int
func (Code) Error() string { calls++; return "code" }
func F() error { return Code(0) }
""", "signature-versus-body",
         calls=[{"text": "Code(0)", "kind": "conversion",
                 "target": "mpk.private.w04/boxing-signature-only.Code"}]),
    case("direct-error-body-call", """
type Code int64
func (Code) Error() string { return "code" }
func F(c Code) string { return c.Error() }
""", "direct-error-call",
         calls=[{"text": "c.Error()", "kind": "direct_method",
                 "target": "(mpk.private.w04/direct-error-body-call.Code).Error"}]),
    case("closed-error-body-call", """
type Code int64
type Flag bool
func (Code) Error() string { return "code" }
func (Flag) Error() string { return "flag" }
func F(e error) string { if e == nil { return "" }; return e.Error() }
""", "closed-error-call",
         contract={"entry_error_types": ["Code", "Flag"]},
         calls=[{"text": "e.Error()", "kind": "interface_invoke", "target": "(error).Error"}]),
    case("closed-error-call-without-nil-guard", """
type Code int64
func (Code) Error() string { return "code" }
func F(e error) string { return e.Error() }
""", "closed-error-call", "accepted_model_pending_safety",
         contract={"entry_error_types": ["Code"], "entry_allows_nil": True},
         calls=[{"text": "e.Error()", "kind": "interface_invoke", "target": "(error).Error"}]),
    case("closed-error-body-global-effect", """
type Code int64
var calls int
func (Code) Error() string { calls++; return "code" }
func F(e error) string { if e == nil { return "" }; return e.Error() }
""", "signature-versus-body", "source_rejection",
         contract={"entry_error_types": ["Code"]},
         calls=[{"text": "e.Error()", "kind": "interface_invoke", "target": "(error).Error"}]),
    case("unclosed-error-input", """
func F(e error) string { if e == nil { return "" }; return e.Error() }
""", "closed-error-universe", "configuration_rejection",
         calls=[{"text": "e.Error()", "kind": "interface_invoke", "target": "(error).Error"}]),
    case("error-alias-input", """
type Code int64
type Alias = error
func (Code) Error() string { return "code" }
func F(e Alias) string { if e == nil { return "" }; return e.Error() }
""", "closed-error-call", contract={"entry_error_types": ["Code"]},
         types={"Alias": {"alias": True, "value_error": True}},
         calls=[{"text": "e.Error()", "kind": "interface_invoke", "target": "(error).Error"}]),
    case("defined-error-interface", """
type Other error
func F(e Other) bool { return e==nil }
""", "open-interface", "source_rejection",
         types={"Other": {"alias": False, "value_error": True}}),
    case("anonymous-error-interface", """
func F(e interface { Error() string }) bool { return e==nil }
""", "open-interface", "source_rejection"),
    case("array-concrete-error", """
type Failure [1]int64
func (Failure) Error() string { return "failure" }
func F() error { return Failure{} }
""", "closed-error-kind", "source_rejection",
         types={"Failure": {"value_error": True, "pointer_error": True}}),
    case("empty-error-entry-universe", """
func F(e error) bool { return e==nil }
""", "closed-error-universe", contract={"entry_error_types": []}),
    case("open-interface-call", """
type Message interface { Text() string }
func F(m Message) string { return m.Text() }
""", "open-interface", "source_rejection",
         calls=[{"text": "m.Text()", "kind": "interface_invoke",
                 "target": "(mpk.private.w04/open-interface-call.Message).Text"}]),
    case("error-to-error-comparison", """
type Code int64
func (Code) Error() string { return "code" }
func F(a,b error) bool { return a==b }
""", "error-comparison", "source_rejection",
         contract={"entry_error_types": ["Code"]}),
    case("error-comparison-to-typed-nil", """
type Code int64
func (Code) Error() string { return "code" }
func F(e error) bool { return e==error(nil) }
""", "error-comparison", "source_rejection",
         contract={"entry_error_types": ["Code"]}),
    case("root-slice-struct-comparison", """
type Root struct { Values []int64 }
func F(a,b Root) bool { return a==b }
""", "source-comparability", "source_rejection", status="source_type_error",
         error_contains="cannot be compared"),
    case("zero-slice-nil-test", """
func F(s []int64) (bool,int,int) { return s==nil,len(s),cap(s) }
""", "zero-capacity"),
    case("zero-slice-construction", """
type Rows []int64
func F() (Rows, bool) { s:=make(Rows,0,0); return s,s==nil }
""", "zero-capacity"),
    case("zero-slice-tail-view", """
func F(s []int64) []int64 { return s[len(s):len(s):len(s)] }
""", "zero-capacity"),
    case("positive-capacity-empty-view", """
func F(s []int64) int64 {
    t := s[:0:cap(s)]
    if cap(t)==0 { return 0 }
    return t[:1][0]
}
""", "positive-capacity-empty"),
    case("zero-count-borrowed-copy", """
func F(dst,src []int64) int { return copy(dst[:0:0],src) }
""", "zero-count-effects"),
    case("zero-count-named-append", """
type Rows []int64
func F(s Rows) Rows { return append(s,Rows{}...) }
""", "zero-count-effects"),
    case("positive-append-to-zero", """
func F() []int64 { return append(make([]int64,0,0),41) }
""", "append-domain", "accepted_model_pending_domain"),
    case("unused-unsupported-body", """
func unused() map[string]int { return map[string]int{"a":1} }
func F(n int64) int64 { return n }
""", "two-closures"),
    case("reachable-unsupported-body", """
func helper() map[string]int { return map[string]int{"a":1} }
func F() int { return len(helper()) }
""", "two-closures", "source_rejection",
         calls=[{"text": "helper()", "kind": "static_function", "occurrence": 1,
                 "target": "mpk.private.w04/reachable-unsupported-body.helper"}]),
    case("unused-type-error", """
func unused() int { return "wrong" }
func F(n int64) int64 { return n }
""", "original-type-check", "source_rejection", status="source_type_error",
         error_contains="cannot use"),
    case("source-parse-error", """
func F( { }
""", "original-type-check", "source_rejection", status="source_parse_error",
         error_contains="expected"),
]


RULE_OWNERS = {
    "root-aggregate": ("GO-V1-R006", "types"),
    "plain-cell": ("GO-V1-R013", "types"),
    "closed-type-graph": ("GO-V1-R006", "types"),
    "nominal": ("GO-V1-R006", "types"),
    "excluded-type": ("GO-V1-R006", "types"),
    "closed-error": ("GO-V1-R009", "errors"),
    "closed-error-root-payload": ("GO-V1-R009", "errors"),
    "plain-closed-error-cell": ("GO-V1-R009", "errors"),
    "closed-error-kind": ("GO-V1-R009", "errors"),
    "pointer-error": ("GO-V1-R009", "errors"),
    "signature-versus-body": ("GO-V1-R004", "closure"),
    "direct-error-call": ("GO-V1-R009", "errors"),
    "closed-error-call": ("GO-V1-R009", "errors"),
    "closed-error-universe": ("GO-V1-R009", "errors"),
    "open-interface": ("GO-V1-R009", "errors"),
    "error-comparison": ("GO-V1-R009", "errors"),
    "source-comparability": ("GO-V1-R006", "types"),
    "zero-capacity": ("GO-V1-R013", "slices"),
    "positive-capacity-empty": ("GO-V1-R013", "slices"),
    "zero-count-effects": ("GO-V1-R015", "construction"),
    "append-domain": ("GO-V1-R015", "construction"),
    "two-closures": ("GO-V1-R004", "closure"),
    "original-type-check": ("GO-V1-R001", "capture"),
}


def vectors():
    return {
        "schema": "mpk.go_practical.w04_source_vectors_draft.v0",
        "work_item": "GO-PRAC-T01-W04",
        "status": "draft design expectations; not executed practical-profile admission",
        "native_scope": "intact original snippets under Go1.25.0 go/types with gc/amd64 sizes; no imports in these snippets",
        "cases": [
            {**row, "requirement": RULE_OWNERS[row["draft_rule"]][0],
             "primary_component": RULE_OWNERS[row["draft_rule"]][1],
             "primary_production_executor": "go-tools/go2vir/practical_" + RULE_OWNERS[row["draft_rule"]][1] + ".go",
             "primary_test_owner": "go-tools/go2vir/practical_" + RULE_OWNERS[row["draft_rule"]][1] + "_test.go"}
            for row in CASES
        ],
    }
