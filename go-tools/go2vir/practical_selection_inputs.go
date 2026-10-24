package main

import (
	"fmt"
	"go/ast"
	"sort"
)

const practicalSelectionInputsDomain = "MPK-PRIVATE-GO-SOURCE-SELECTION-INPUTS-1.0"

type practicalSelectionRoot struct {
	PackageID string `json:"package_id"`
	SourceID  string `json:"source_object_id"`
	Start     int    `json:"start_byte"`
	End       int    `json:"end_byte"`
}

// Exact original physical function roots and the complete owned source/sidecar
// packet. No supplied AST, name resolver, type graph or callee facts enter it.
// This private input linkage does not authenticate an installed capture context.
type practicalSelectionInputs struct {
	contracts practicalContractInputs
	canonical []byte
}

func practicalSelectionRootLess(a, b practicalSelectionRoot) bool {
	if a.PackageID != b.PackageID {
		return a.PackageID < b.PackageID
	}
	if a.SourceID != b.SourceID {
		return a.SourceID < b.SourceID
	}
	if a.Start != b.Start {
		return a.Start < b.Start
	}
	return a.End < b.End
}

func (owner *practicalOriginalPackages) selectionInputs(roots []practicalClosureRef) (practicalSelectionInputs, error) {
	if owner == nil || len(roots) == 0 || len(roots) > 8192 {
		return practicalSelectionInputs{}, fmt.Errorf("selection inputs require original function roots within the frozen count")
	}
	owner.mutex.Lock()
	defer owner.mutex.Unlock()
	contracts, err := owner.contractInputsLocked()
	if err != nil {
		return practicalSelectionInputs{}, err
	}
	component, err := decodeStrictJSON(contracts.source.components.canonical)
	if err != nil {
		return practicalSelectionInputs{}, err
	}
	packages := map[string]string{}
	for _, row := range component.(map[string]jsonValue)["packages"].([]jsonValue) {
		p := row.(map[string]jsonValue)
		packages[p["import_path"].(string)] = p["id"].(string)
	}
	physical := make([]practicalSelectionRoot, 0, len(roots))
	functions := map[string]map[practicalSourceSpan]int{}
	for _, ref := range append([]practicalClosureRef(nil), roots...) {
		id := packages[ref.packagePath]
		if id == "" {
			return practicalSelectionInputs{}, fmt.Errorf("selection root is outside the original source package set")
		}
		if _, err := owner.storageOriginalSpan(ref.packagePath, ref.span); err != nil {
			return practicalSelectionInputs{}, err
		}
		if functions[ref.packagePath] == nil {
			functions[ref.packagePath] = map[practicalSourceSpan]int{}
			for _, file := range owner.loader.packages[ref.packagePath].files {
				for _, declaration := range file.Decls {
					function, ok := declaration.(*ast.FuncDecl)
					if !ok || function.Name.Name == "init" && function.Recv == nil || function.Name.Name == "_" {
						continue
					}
					span, _, err := owner.loader.syntax.spanBytes(function)
					if err != nil {
						return practicalSelectionInputs{}, err
					}
					functions[ref.packagePath][span]++
				}
			}
		}
		if functions[ref.packagePath][ref.span] != 1 {
			return practicalSelectionInputs{}, fmt.Errorf("selection root must be one complete original named function or method declaration")
		}
		physical = append(physical, practicalSelectionRoot{id, ref.span.sourceID, ref.span.start, ref.span.end})
	}
	sort.Slice(physical, func(i, j int) bool { return practicalSelectionRootLess(physical[i], physical[j]) })
	for i := 1; i < len(physical); i++ {
		if physical[i-1] == physical[i] {
			return practicalSelectionInputs{}, fmt.Errorf("selection inputs require distinct original physical roots")
		}
	}
	contract, err := decodeStrictJSON(contracts.canonical)
	if err != nil {
		return practicalSelectionInputs{}, err
	}
	body := map[string]any{
		"schema":                        "mpk.go.practical.source_selection_inputs.v1",
		"source_contract_inputs_sha256": contract.(map[string]jsonValue)["content_sha256"],
		"roots":                         physical,
	}
	digest, err := hashTypedCanonicalJSON(practicalSelectionInputsDomain, body)
	if err != nil {
		return practicalSelectionInputs{}, err
	}
	body["content_sha256"] = digest
	canonical, err := canonicalJSON(body)
	if err != nil {
		return practicalSelectionInputs{}, err
	}
	return practicalSelectionInputs{contracts: contracts, canonical: canonical}, nil
}
