package main

import (
	"encoding/json"
	"fmt"
	"io"
	"os"
	"strings"
)

const (
	cliSchema = "mpk.go2gir.cli.v0"
	usage     = "usage: go2gir <package-path>\n"
)

type cliResult struct {
	Schema           string            `json:"schema"`
	Status           string            `json:"status"`
	PackagePath      string            `json:"package_path"`
	Packages         []loadedPackage   `json:"packages"`
	SSA              *ssaDump          `json:"ssa,omitempty"`
	RejectedFeatures []rejectedFeature `json:"rejected_features,omitempty"`
}

func main() {
	os.Exit(run(os.Args[1:], os.Stdout, os.Stderr))
}

func run(args []string, stdout io.Writer, stderr io.Writer) int {
	if len(args) == 1 && (args[0] == "-h" || args[0] == "--help") {
		fmt.Fprint(stdout, usage)
		return 0
	}

	if len(args) != 1 {
		fmt.Fprintf(stderr, "%sgo2gir requires exactly one package path\n", usage)
		return 2
	}

	packagePath := strings.TrimSpace(args[0])
	if packagePath == "" {
		fmt.Fprintf(stderr, "%sgo2gir package path must not be empty\n", usage)
		return 2
	}
	if strings.HasPrefix(packagePath, "-") {
		fmt.Fprintf(stderr, "%sunknown flag or invalid package path: %s\n", usage, packagePath)
		return 2
	}

	loaded, err := loadPackageSet(packagePath, loadOptions{})
	if err != nil {
		fmt.Fprintf(stderr, "%s%s\n", usage, err)
		return 1
	}
	rejectedFeatures := detectUnsupportedFeatures(loaded)
	if len(rejectedFeatures) > 0 {
		if err := encodeCLIResult(stdout, cliResult{
			Schema:           cliSchema,
			Status:           "rejected",
			PackagePath:      packagePath,
			Packages:         loaded.Summaries,
			RejectedFeatures: rejectedFeatures,
		}); err != nil {
			fmt.Fprintf(stderr, "encode go2gir result: %v\n", err)
			return 1
		}
		return 1
	}

	ssaResult, err := buildSSADump(loaded.Packages)
	if err != nil {
		fmt.Fprintf(stderr, "%s%s\n", usage, err)
		return 1
	}

	result := cliResult{
		Schema:      cliSchema,
		Status:      "ssa-built",
		PackagePath: packagePath,
		Packages:    loaded.Summaries,
		SSA:         &ssaResult,
	}
	if err := encodeCLIResult(stdout, result); err != nil {
		fmt.Fprintf(stderr, "encode go2gir result: %v\n", err)
		return 1
	}
	return 0
}

func encodeCLIResult(stdout io.Writer, result cliResult) error {
	encoder := json.NewEncoder(stdout)
	encoder.SetEscapeHTML(false)
	return encoder.Encode(result)
}
