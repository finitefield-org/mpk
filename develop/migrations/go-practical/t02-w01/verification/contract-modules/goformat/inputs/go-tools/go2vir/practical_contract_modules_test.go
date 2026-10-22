//go:build go1.25 && (linux || darwin)

package main

import (
	"fmt"
	"go/types"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func TestPracticalContractModuleOriginalContexts(t *testing.T) {
	if output := os.Getenv("MPK_GO_PRACTICAL_SYNTAX_EVIDENCE"); output != "" {
		output = filepath.Join(output, "contract-modules")
		if err := os.Mkdir(output, 0700); err != nil {
			t.Fatal(err)
		}
		t.Setenv("MPK_GO_PRACTICAL_SYNTAX_EVIDENCE", output)
	}
	source := `package policy
import h "example.com/policy/helper"
type Imported=h.Code
type First uint8
type FirstAlias=First
func(First)Error()string{return Link0(nil)}
type Second uint16
func(Second)Error()string{return "second"}
type Rich struct{Cells []uint8}
func(Rich)Error()string{return "rich"}
type Dormant uint8
var Ambient="ambient"
func(Dormant)Error()string{return Ambient}
type Recv struct{A uint8}
type RecvAlias=Recv
type MapRecv map[string]int
type MapRecvAlias=MapRecv
func(m MapRecv)Unused(e error)error{_=m;return e}
func(r Recv)Entry(e error)string{_=r;return Worker(e)}
func Root(xs []error,e error)(string,error){_=xs;return Worker(e),e}
func Worker(e error)string{for i:=0;i<1;i++{_=i};return Helper(e)}
func Helper(e error)string{return e.Error()}
func Link0(e error)string{_=e;return "link"}
func BoxOnly()error{return Dormant(0)}
func Unselected(m map[string]int)map[string]int{return m}
func BadCycle(){BadCycle()}
//line absent.go:999
func Loops(xs []uint8){for i:=0;i<1;i++{_=i};for _,v:=range xs{_=v}}
func BoundLoop(){for i:=0;i<1;i++{_=i}}
`
	for i := 0; i < 70; i++ {
		source += fmt.Sprintf("type Grow%d uint8\nfunc(Grow%d)Error()string{return Next%d(nil)}\nfunc Next%d(e error)string{_=e;return \"next\"}\n", i, i, i, i)
	}
	source += "func Grow()string{var e error=Grow0(0);return e.Error()}\nfunc Many()uint16{return H000()}\n"
	for i := 0; i < 512; i++ {
		body := "return 1"
		if i < 511 {
			body = fmt.Sprintf("return H%03d()", i+1)
		}
		source += fmt.Sprintf("func H%03d()uint16{%s}\n", i, body)
	}
	for i := 0; i < 8; i++ {
		body := ""
		if i == 0 {
			for j := 1; j < 8; j++ {
				body += fmt.Sprintf("Bound%d();", j)
			}
		}
		source += fmt.Sprintf("func Bound%d(){%s}\n", i, body)
	}
	owner, _ := practicalStorageTestOwner(t, map[string][]byte{
		"go.mod":           []byte("module example.com/policy\ngo 1.25.0\n"),
		"policy.go":        []byte(strings.ReplaceAll(source, "\n", "\r\n")),
		"helper/helper.go": []byte("package helper\ntype Code uint8\nfunc(Code)Error()string{return \"helper\"}\n"),
	})
	defer owner.Close()
	retainPracticalSyntaxEvidence(t, "inputs.json", practicalBoxingInput(t, owner))
	selector := func(name string, receiver any) map[string]any {
		return map[string]any{"package": "example.com/policy", "function": name, "receiver": receiver}
	}
	ordinary := func(name string) map[string]any { return selector(name, nil) }
	method := func(name, receiver string) map[string]any {
		return selector(name, map[string]any{"package": "example.com/policy", "type_name": receiver})
	}
	entry := func(input int, path []any, cases ...string) any {
		return map[string]any{"input": input, "path": path, "cases": cases}
	}
	annotation := func(s map[string]any, entries []any, total bool) map[string]any {
		termination := "partial"
		if total {
			termination = "total"
		}
		return map[string]any{"schema": "mpk.go.practical.source_overlay.v1", "source": "policy.go", "function": s, "termination": termination, "requires": []any{}, "ensures": []any{map[string]any{"op": "bool.literal", "value": true}}, "error_inputs": entries, "loops": []any{}, "modifies": []any{}}
	}
	worker := func(cases ...string) map[string]any {
		return annotation(ordinary("Worker"), []any{entry(0, []any{}, cases...)}, false)
	}
	unused := annotation(ordinary("Unselected"), []any{}, false)
	unused["requires"] = []any{map[string]any{"op": "source.zero", "type": "example.com/policy.Absent"}}
	unusedReceiver := annotation(method("Unused", "MapRecvAlias"), []any{}, false)
	unusedReceiver["requires"] = []any{map[string]any{"op": "local", "name": "ghost"}}
	dormant := annotation(method("Error", "Dormant"), []any{}, false)
	dormant["requires"] = []any{map[string]any{"op": "local", "name": "ghost"}}
	type testCase struct {
		name      string
		roots     []map[string]any
		overlays  []map[string]any
		entry     []string
		rejection string
	}
	cases := []testCase{
		{"defaults", []map[string]any{ordinary("Root")}, nil, nil, ""},
		{"callee-annotation", []map[string]any{ordinary("Root")}, []map[string]any{worker("example.com/policy.FirstAlias", "example.com/policy.Second")}, []string{"First", "Second"}, ""},
		{"root-annotation", []map[string]any{ordinary("Root")}, []map[string]any{annotation(ordinary("Root"), []any{entry(0, []any{map[string]any{"kind": "elements"}}, "example.com/policy.First"), entry(1, []any{}, "example.com/policy.Second")}, false)}, []string{"First", "Second"}, ""},
		{"total-loops", []map[string]any{ordinary("Loops")}, []map[string]any{annotation(ordinary("Loops"), []any{}, true)}, nil, ""},
		{"receiver-alias", []map[string]any{method("Entry", "RecvAlias")}, []map[string]any{worker("example.com/policy.First", "example.com/policy.Second")}, []string{"First", "Second"}, ""},
		{"signature-only", []map[string]any{ordinary("BoxOnly")}, []map[string]any{dormant}, nil, ""},
		{"captured-unselected", []map[string]any{ordinary("Loops")}, []map[string]any{unused, unusedReceiver}, nil, ""},
		{"growth-seventy", []map[string]any{ordinary("Grow")}, nil, nil, ""},
		{"many-helpers", []map[string]any{ordinary("Many")}, nil, nil, ""},
		{"multiple-roots", []map[string]any{ordinary("Root"), ordinary("Loops")}, []map[string]any{worker("example.com/policy.First", "example.com/policy.Second")}, []string{"First", "Second"}, ""},
		{"target-effect", []map[string]any{ordinary("Root")}, []map[string]any{worker("example.com/policy.Dormant")}, []string{"Dormant"}, "selection"},
		{"global-nonplain", []map[string]any{ordinary("Root")}, []map[string]any{worker("example.com/policy.Rich")}, []string{"Rich"}, "domain"},
		{"cycle", []map[string]any{ordinary("BadCycle")}, nil, nil, "selection"},
		{"selected-map", []map[string]any{ordinary("Unselected")}, []map[string]any{unused}, nil, "selection"},
		{"boundaries", []map[string]any{ordinary("Bound0")}, nil, nil, ""},
		{"generated-budget", []map[string]any{ordinary("BoundLoop")}, nil, nil, ""},
		{"selected-map-receiver", []map[string]any{method("Unused", "MapRecvAlias")}, []map[string]any{unusedReceiver}, nil, "selection"},
	}
	for i := 0; i < 69; i++ {
		cases[7].overlays = append(cases[7].overlays, annotation(ordinary(fmt.Sprintf("Next%d", i)), []any{entry(0, []any{}, fmt.Sprintf("example.com/policy.Grow%d", i+1))}, false))
		cases[7].entry = append(cases[7].entry, fmt.Sprintf("Grow%d", i+1))
	}
	directory, err := filepath.EvalSymlinks(t.TempDir())
	if err != nil {
		t.Fatal(err)
	}
	paths := [][]string{}
	for i, row := range cases {
		prepared := []string{}
		for j, raw := range row.overlays {
			bytes, err := canonicalJSON(raw)
			if err != nil {
				t.Fatal(err)
			}
			path := fmt.Sprintf("contracts/%02d-%03d.json", i, j)
			writePracticalTestFile(t, filepath.Join(directory, path), append(bytes, '\n'), 0600)
			prepared = append(prepared, path)
		}
		paths = append(paths, prepared)
	}
	session := practicalTestSession(t, []practicalSourceRoot{{Capability: "overlays", Directory: directory}})
	for i, row := range cases {
		roots := []practicalClosureRef{}
		for _, s := range row.roots {
			r, _, _ := practicalOverlayFunction(t, owner, s)
			roots = append(roots, r)
		}
		entryTypes := []types.Type{}
		for _, name := range row.entry {
			entryTypes = append(entryTypes, owner.loader.packages["example.com/policy"].types.Scope().Lookup(name).Type())
		}
		original := []any{}
		for _, path := range paths[i] {
			buffer, err := session.readSource("overlays", path, "overlay")
			if err != nil {
				t.Fatal(err)
			}
			original = append(original, map[string]any{"object": buffer.object, "raw": buffer.sourceBytes()})
		}
		var native any
		// Map results are valid original Go but excluded before source closure
		// classification; the separate native closure observer expects carriers.
		if !strings.HasPrefix(row.name, "selected-map") {
			native = practicalNativeErrorClosure(t, owner, roots, entryTypes)
		}
		retainPracticalSyntaxEvidence(t, fmt.Sprintf("case-%02d.json", i), map[string]any{"name": row.name, "roots": row.roots, "overlays": original, "native": native, "expected_profile_rejection": row.rejection})
	}
	snapshot, err := session.seal()
	if err != nil {
		t.Fatal(err)
	}
	retainPracticalStandardSnapshot(t, snapshot)
	if err := removePracticalReplayImage(directory); err != nil {
		t.Fatal(err)
	}
	retainPracticalSyntaxEvidence(t, "controls.json", map[string]any{"cases": len(cases), "growth_cases": 70, "helpers": 512, "original_sidecar_namespace_removed": true, "constructor_receives_native_facts": false})
}
