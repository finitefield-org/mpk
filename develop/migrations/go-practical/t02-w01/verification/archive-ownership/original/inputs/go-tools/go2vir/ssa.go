package main

import (
	"fmt"

	"golang.org/x/tools/go/packages"
	"golang.org/x/tools/go/ssa"
	"golang.org/x/tools/go/ssa/ssautil"
)

// validateSSAAttribution builds SSA from the already captured, type-checked
// package graph. Lowering still follows the closed source forms so compiler
// temporaries never become public identities; this gate proves that the typed
// source accepted by the recognizer also has a complete compiler CFG.
func validateSSAAttribution(loaded packageLoadResult) (err error) {
	initial := make([]*packages.Package, 0, len(loaded.Packages))
	for _, value := range loaded.Packages {
		if value.packageValue == nil {
			return fmt.Errorf("loaded package lacks compiler state")
		}
		initial = append(initial, value.packageValue)
	}
	if len(initial) == 0 {
		return fmt.Errorf("loaded package graph is empty")
	}
	defer func() {
		if recovered := recover(); recovered != nil {
			err = fmt.Errorf("SSA construction failed")
		}
	}()
	program, packages := ssautil.AllPackages(initial, ssa.SanityCheckFunctions)
	program.Build()
	for _, value := range packages {
		if value == nil {
			return fmt.Errorf("SSA package graph is incomplete")
		}
	}
	return nil
}
