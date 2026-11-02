// Private complete loaded-syntax counts, including pinned standard packages.
package main

import (
	"encoding/json"
	"go/ast"
	"os"
	"sort"

	"golang.org/x/tools/go/packages"
)

func main() {
	var request struct {
		Root    string `json:"root"`
		Package string `json:"package"`
	}
	if err := json.NewDecoder(os.Stdin).Decode(&request); err != nil {
		panic(err)
	}
	loaded, err := packages.Load(&packages.Config{Dir: request.Root, Env: os.Environ(), Tests: false, BuildFlags: []string{"-mod=readonly", "-trimpath", "-p=1"}, Mode: packages.NeedName | packages.NeedFiles | packages.NeedCompiledGoFiles | packages.NeedImports | packages.NeedDeps | packages.NeedModule | packages.NeedSyntax | packages.NeedTypes | packages.NeedTypesInfo | packages.NeedTypesSizes}, request.Package)
	if err != nil {
		panic(err)
	}
	rows := []map[string]any{}
	diagnostics := []string{}
	total, standard, application := 0, 0, 0
	seen := map[string]bool{}
	packages.Visit(loaded, func(pkg *packages.Package) bool {
		if seen[pkg.ID] {
			return false
		}
		seen[pkg.ID] = true
		for _, e := range pkg.Errors {
			diagnostics = append(diagnostics, e.Error())
		}
		nodes := 0
		for _, file := range pkg.Syntax {
			ast.Inspect(file, func(n ast.Node) bool {
				if n != nil {
					nodes++
				}
				return true
			})
		}
		isStandard := pkg.Module == nil
		if isStandard {
			standard += nodes
		} else {
			application += nodes
		}
		total += nodes
		rows = append(rows, map[string]any{"package": pkg.PkgPath, "standard": isStandard, "syntax_files": len(pkg.Syntax), "syntax_nodes": nodes})
		return true
	}, nil)
	sort.Slice(rows, func(i, j int) bool { return rows[i]["package"].(string) < rows[j]["package"].(string) })
	sort.Strings(diagnostics)
	out := map[string]any{"schema": "mpk.go_practical.loaded_syntax_metrics.v0", "scope": "all loaded syntax, including standard library; original package never executed; not source/proof admission", "packages": rows, "diagnostics": diagnostics, "total_syntax_nodes": total, "standard_syntax_nodes": standard, "application_syntax_nodes": application}
	if err := json.NewEncoder(os.Stdout).Encode(out); err != nil {
		panic(err)
	}
}
