//go:build go1.25 && (linux || darwin)

package main

import (
	"bytes"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"go/ast"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func practicalSelectionInputEvidence(t *testing.T, packet practicalSelectionInputs) map[string]any {
	t.Helper()
	result := practicalContractInputEvidence(t, packet.contracts)
	result["selection_hex"] = hex.EncodeToString(packet.canonical)
	return result
}

func TestPracticalSelectionInputsRequireOriginalOwnerAndRoots(t *testing.T) {
	root := practicalClosureRef{"example.com/policy", practicalSourceSpan{"missing", 0, 1}}
	for _, owner := range []*practicalOriginalPackages{nil, {}, {closed: true}, {inputs: &practicalImageInputs{}, loader: &practicalSourceTypes{}}} {
		for _, roots := range [][]practicalClosureRef{nil, {root}, make([]practicalClosureRef, 8193)} {
			packet, err := owner.selectionInputs(roots)
			if err == nil || packet.canonical != nil || packet.contracts.canonical != nil {
				t.Fatal("unowned or invalid root input accepted", err)
			}
		}
	}
}

func TestPracticalSelectionInputsMethodInitIsOrdinary(t *testing.T) {
	practicalContractTestOutput(t, "method-init")
	files := map[string][]byte{}
	for _, name := range []string{"go.mod", "policy.go"} {
		raw, err := os.ReadFile(filepath.Join("..", "..", "develop", "migrations", "go-practical", "t02-w01", "verification", "selection-inputs", "method-init-go-validity", "originals", name))
		if err != nil {
			t.Fatal(err)
		}
		files[name] = raw
	}
	owner, _ := practicalStorageTestOwner(t, files)
	defer owner.Close()
	var method, initializer practicalClosureRef
	for _, file := range owner.loader.packages["example.com/policy"].files {
		for _, declaration := range file.Decls {
			f, ok := declaration.(*ast.FuncDecl)
			if !ok || f.Name.Name != "init" {
				continue
			}
			tokenFile := owner.loader.syntax.fileSet.File(f.Pos())
			ref := practicalClosureRef{"example.com/policy", practicalSourceSpan{strings.TrimSuffix(tokenFile.Name(), ".go"), tokenFile.Offset(f.Pos()), tokenFile.Offset(f.End())}}
			if f.Recv == nil {
				initializer = ref
			} else {
				method = ref
			}
		}
	}
	packet, err := owner.selectionInputs([]practicalClosureRef{method})
	if err != nil {
		t.Fatal("ordinary method named init rejected", err)
	}
	if value, err := owner.selectionInputs([]practicalClosureRef{initializer}); err == nil || value.canonical != nil {
		t.Fatal("package initializer selected")
	}
	retainPracticalSyntaxEvidence(t, "selection-method-init.json", practicalSelectionInputEvidence(t, packet))
	retainPracticalSyntaxEvidence(t, "method-init-observations.json", map[string]any{"ordinary_method": practicalDispatchRef(method), "package_initializer": practicalDispatchRef(initializer), "application_initializers_executed": false})
}

func TestPracticalSelectionInputsOwnCompletePhysicalRoots(t *testing.T) {
	practicalContractTestOutput(t, "selection-owned")
	policy := "package policy\nimport \"example.com/policy/helper\"\ntype Count uint16\ntype Other uint16\nconst Limit=7\nfunc Root(x uint16)uint16{return helper.Work(x)}\nfunc Standalone(x uint16)uint16{return x}\nfunc(c Count)Work(x uint16)uint16{return uint16(c)+x}\nfunc(o Other)Work(x uint16)uint16{return uint16(o)-x}\nfunc Unused(c chan int,f func(int)int)int{return f(<-c)}\nfunc init(){panic(\"must not execute\")}\nfunc _(){}\n"
	// The exact frozen root-count boundary, with all ordinary original bodies.
	for i := 0; i < 8192; i++ {
		policy += fmt.Sprintf("func Bound%04d(){}\n", i)
	}
	annotation := func(function string, receiver any) []byte {
		data, err := json.Marshal(map[string]any{"schema": "mpk.go.practical.source_overlay.v1", "source": "policy.go", "function": map[string]any{"package": "example.com/policy", "function": function, "receiver": receiver}, "termination": "partial", "requires": []any{}, "ensures": []any{map[string]any{"op": "bool.literal", "value": true}}, "error_inputs": []any{}, "loops": []any{}, "modifies": []any{}})
		if err != nil {
			t.Fatal(err)
		}
		return data
	}
	files := map[string][]byte{
		"go.mod":                []byte("module example.com/policy\ngo 1.25.0\n"),
		"policy.go":             []byte(policy),
		"helper/helper.go":      []byte("package helper\nfunc Work(x uint16)uint16{return x+1}\nfunc Unused(m map[int]int)int{return m[0]}\n"),
		"contracts/root.json":   annotation("Root", nil),
		"contracts/count.json":  annotation("Work", map[string]any{"package": "example.com/policy", "type_name": "Count"}),
		"contracts/unused.json": annotation("Unused", nil),
	}
	owner, _ := practicalStorageTestOwnerWithOverlays(t, files, "example.com/policy", []string{"contracts/root.json", "contracts/count.json", "contracts/unused.json"})
	defer owner.Close()
	refs := map[string]practicalClosureRef{}
	observations := map[string]any{}
	boundary := []practicalClosureRef{}
	for pkg, loaded := range owner.loader.packages {
		for _, file := range loaded.files {
			for _, declaration := range file.Decls {
				function, ok := declaration.(*ast.FuncDecl)
				if !ok {
					continue
				}
				tokenFile := owner.loader.syntax.fileSet.File(function.Pos())
				ref := practicalClosureRef{pkg, practicalSourceSpan{strings.TrimSuffix(tokenFile.Name(), ".go"), tokenFile.Offset(function.Pos()), tokenFile.Offset(function.End())}}
				name := function.Name.Name
				if function.Recv != nil {
					name = function.Recv.List[0].Type.(*ast.Ident).Name + "." + name
				}
				name = pkg + "." + name
				refs[name] = ref
				observations[name] = practicalDispatchRef(ref)
				if strings.HasPrefix(function.Name.Name, "Bound") {
					boundary = append(boundary, ref)
				}
			}
		}
	}
	root := refs["example.com/policy.Root"]
	roots := []practicalClosureRef{root, refs["example.com/policy.Other.Work"], refs["example.com/policy.Count.Work"]}
	packet, err := owner.selectionInputs(roots)
	if err != nil {
		t.Fatal(err)
	}
	retainPracticalSyntaxEvidence(t, "selection-inputs.json", practicalSelectionInputEvidence(t, packet))
	retainPracticalSyntaxEvidence(t, "physical-root-observations.json", map[string]any{"functions": observations, "all_original_files": files, "application_initializers_executed": false})
	reverse := []practicalClosureRef{roots[2], roots[1], roots[0]}
	again, err := owner.selectionInputs(reverse)
	if err != nil || !bytes.Equal(packet.canonical, again.canonical) {
		t.Fatal("physical root ordering changed identity", err)
	}
	if len(boundary) != 8192 {
		t.Fatal("incomplete original boundary roots")
	}
	maximum, err := owner.selectionInputs(boundary)
	if err != nil {
		t.Fatal("exact root count rejected", err)
	}
	retainPracticalSyntaxEvidence(t, "selection-boundary-inputs.json", practicalSelectionInputEvidence(t, maximum))
	badStart, badEnd, badPackage, badSource := root, root, root, root
	badStart.span.start++
	badEnd.span.end++
	badPackage.packagePath = "example.com/policy/helper"
	badSource.span.sourceID = strings.Repeat("0", 64)
	controls := []struct {
		name  string
		roots []practicalClosureRef
	}{
		{"empty", nil}, {"duplicate", []practicalClosureRef{root, root}}, {"count-over", append(boundary, root)},
		{"interior-start", []practicalClosureRef{badStart}}, {"interior-end", []practicalClosureRef{badEnd}}, {"cross-package", []practicalClosureRef{badPackage}}, {"cross-source", []practicalClosureRef{badSource}},
		{"initializer", []practicalClosureRef{refs["example.com/policy.init"]}}, {"blank", []practicalClosureRef{refs["example.com/policy._"]}},
	}
	results := []any{}
	for _, control := range controls {
		if value, err := owner.selectionInputs(control.roots); err == nil || value.canonical != nil {
			t.Fatal(control.name, "accepted")
		}
		results = append(results, map[string]any{"control": control.name, "rejected": true})
	}
	canonical, contract := bytes.Clone(again.canonical), bytes.Clone(again.contracts.canonical)
	clear(packet.canonical)
	clear(packet.contracts.canonical)
	clear(packet.contracts.source.components.canonical)
	for i := range packet.contracts.source.components.sources {
		clear(packet.contracts.source.components.sources[i].data)
	}
	for i := range packet.contracts.overlays {
		clear(packet.contracts.overlays[i].data)
	}
	roots[0] = practicalClosureRef{}
	fresh, err := owner.selectionInputs(reverse)
	if err != nil || !bytes.Equal(fresh.canonical, canonical) || !bytes.Equal(fresh.contracts.canonical, contract) {
		t.Fatal("caller mutated the original selection universe", err)
	}
	owner.Close()
	if !bytes.Equal(again.canonical, canonical) {
		t.Fatal("selection packet retired with owner")
	}
	for _, overlay := range again.contracts.overlays {
		if !bytes.Equal(overlay.data, files[overlay.object.Path]) {
			t.Fatal("sidecar retired with owner")
		}
	}
	if value, err := owner.selectionInputs(reverse); err == nil || value.canonical != nil {
		t.Fatal("closed owner emitted new selection input")
	}
	retainPracticalSyntaxEvidence(t, "selection-ownership.json", map[string]any{"passed": true, "exact_root_count": 8192, "controls": results, "root_permutation_byte_identical": true, "returned_bytes_independent": true, "owner_retirement_keeps_inputs": true})
}
