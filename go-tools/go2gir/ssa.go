package main

import (
	"fmt"
	"go/types"
	"sort"

	"golang.org/x/tools/go/packages"
	"golang.org/x/tools/go/ssa"
	"golang.org/x/tools/go/ssa/ssautil"
)

type ssaDump struct {
	Packages []ssaPackageDump `json:"packages"`
}

type ssaPackageDump struct {
	PackagePath string            `json:"package_path"`
	Name        string            `json:"name"`
	Functions   []ssaFunctionDump `json:"functions"`
}

type ssaFunctionDump struct {
	Name      string         `json:"name"`
	Signature string         `json:"signature"`
	Blocks    []ssaBlockDump `json:"blocks"`
}

type ssaBlockDump struct {
	Index        int      `json:"index"`
	Comment      string   `json:"comment,omitempty"`
	Instructions []string `json:"instructions"`
}

func buildSSADump(packagesLoaded []*packages.Package) (ssaDump, error) {
	program, ssaPackages := ssautil.Packages(packagesLoaded, 0)
	program.Build()

	packagesDump := make([]ssaPackageDump, 0, len(ssaPackages))
	for index, ssaPackage := range ssaPackages {
		if ssaPackage == nil {
			return ssaDump{}, fmt.Errorf("build SSA: package %s could not be constructed", packageLabel(packagesLoaded, index))
		}
		packagesDump = append(packagesDump, dumpSSAPackage(ssaPackage))
	}

	sort.Slice(packagesDump, func(i, j int) bool {
		if packagesDump[i].PackagePath != packagesDump[j].PackagePath {
			return packagesDump[i].PackagePath < packagesDump[j].PackagePath
		}
		return packagesDump[i].Name < packagesDump[j].Name
	})

	return ssaDump{Packages: packagesDump}, nil
}

func dumpSSAPackage(ssaPackage *ssa.Package) ssaPackageDump {
	functions := make([]ssaFunctionDump, 0)
	for _, member := range ssaPackage.Members {
		function, ok := member.(*ssa.Function)
		if !ok || function.Synthetic != "" {
			continue
		}
		functions = append(functions, dumpSSAFunction(function))
	}

	sort.Slice(functions, func(i, j int) bool {
		if functions[i].Name != functions[j].Name {
			return functions[i].Name < functions[j].Name
		}
		return functions[i].Signature < functions[j].Signature
	})

	return ssaPackageDump{
		PackagePath: packagePath(ssaPackage.Pkg),
		Name:        packageName(ssaPackage.Pkg),
		Functions:   functions,
	}
}

func dumpSSAFunction(function *ssa.Function) ssaFunctionDump {
	blocks := make([]ssaBlockDump, 0, len(function.Blocks))
	for _, block := range function.Blocks {
		instructions := make([]string, 0, len(block.Instrs))
		for _, instruction := range block.Instrs {
			instructions = append(instructions, instruction.String())
		}
		blocks = append(blocks, ssaBlockDump{
			Index:        block.Index,
			Comment:      block.Comment,
			Instructions: instructions,
		})
	}

	sort.Slice(blocks, func(i, j int) bool {
		return blocks[i].Index < blocks[j].Index
	})

	return ssaFunctionDump{
		Name:      function.Name(),
		Signature: signatureString(function.Signature),
		Blocks:    blocks,
	}
}

func packageLabel(packagesLoaded []*packages.Package, index int) string {
	if index < 0 || index >= len(packagesLoaded) || packagesLoaded[index] == nil {
		return fmt.Sprintf("#%d", index)
	}
	if packagesLoaded[index].PkgPath != "" {
		return packagesLoaded[index].PkgPath
	}
	if packagesLoaded[index].ID != "" {
		return packagesLoaded[index].ID
	}
	return fmt.Sprintf("#%d", index)
}

func packagePath(pkg *types.Package) string {
	if pkg == nil {
		return ""
	}
	return pkg.Path()
}

func packageName(pkg *types.Package) string {
	if pkg == nil {
		return ""
	}
	return pkg.Name()
}

func signatureString(signature *types.Signature) string {
	if signature == nil {
		return ""
	}
	return signature.String()
}
