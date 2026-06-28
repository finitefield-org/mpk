package main

import (
	"bytes"
	"encoding/json"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func TestRunAcceptsPackagePath(t *testing.T) {
	var stdout bytes.Buffer
	var stderr bytes.Buffer

	exitCode := run([]string{"./testdata/samplepkg"}, &stdout, &stderr)
	if exitCode != 0 {
		t.Fatalf("exit code = %d, want 0; stderr=%s", exitCode, stderr.String())
	}
	if stderr.Len() != 0 {
		t.Fatalf("stderr = %q, want empty", stderr.String())
	}

	var result cliResult
	if err := json.Unmarshal(stdout.Bytes(), &result); err != nil {
		t.Fatalf("decode stdout: %v\n%s", err, stdout.String())
	}
	if result.Schema != cliSchema {
		t.Fatalf("schema = %q, want %q", result.Schema, cliSchema)
	}
	if result.Status != "ssa-built" {
		t.Fatalf("status = %q, want ssa-built", result.Status)
	}
	if result.PackagePath != "./testdata/samplepkg" {
		t.Fatalf("package path = %q, want ./testdata/samplepkg", result.PackagePath)
	}
	if len(result.Packages) != 1 {
		t.Fatalf("package count = %d, want 1: %+v", len(result.Packages), result.Packages)
	}
	got := result.Packages[0]
	if got.Name != "samplepkg" {
		t.Fatalf("package name = %q, want samplepkg", got.Name)
	}
	if got.PackagePath != "github.com/finitefield-org/mpk/go-tools/go2gir/testdata/samplepkg" {
		t.Fatalf("package path = %q", got.PackagePath)
	}
	if !contains(got.GoFiles, "testdata/samplepkg/sample.go") {
		t.Fatalf("go files = %v, want sample.go", got.GoFiles)
	}
	identity := findSSAFunction(result.SSA, got.PackagePath, "Identity")
	if identity == nil {
		t.Fatalf("SSA dump missing Identity function: %+v", result.SSA)
	}
	if identity.Signature != "func(value int64) int64" {
		t.Fatalf("Identity signature = %q, want func(value int64) int64", identity.Signature)
	}
	if !hasSSAInstructionContaining(*identity, "return") {
		t.Fatalf("Identity SSA instructions = %+v, want return instruction", identity.Blocks)
	}
}

func TestRunRejectsMissingPackagePath(t *testing.T) {
	var stdout bytes.Buffer
	var stderr bytes.Buffer

	exitCode := run(nil, &stdout, &stderr)
	if exitCode != 2 {
		t.Fatalf("exit code = %d, want 2", exitCode)
	}
	if stdout.Len() != 0 {
		t.Fatalf("stdout = %q, want empty", stdout.String())
	}
	if !strings.Contains(stderr.String(), "requires exactly one package path") {
		t.Fatalf("stderr = %q, want package path error", stderr.String())
	}
}

func TestRunRejectsExtraArguments(t *testing.T) {
	var stdout bytes.Buffer
	var stderr bytes.Buffer

	exitCode := run([]string{"./one", "./two"}, &stdout, &stderr)
	if exitCode != 2 {
		t.Fatalf("exit code = %d, want 2", exitCode)
	}
	if !strings.Contains(stderr.String(), "requires exactly one package path") {
		t.Fatalf("stderr = %q, want package path error", stderr.String())
	}
}

func TestRunPrintsUsage(t *testing.T) {
	var stdout bytes.Buffer
	var stderr bytes.Buffer

	exitCode := run([]string{"--help"}, &stdout, &stderr)
	if exitCode != 0 {
		t.Fatalf("exit code = %d, want 0", exitCode)
	}
	if stdout.String() != usage {
		t.Fatalf("stdout = %q, want %q", stdout.String(), usage)
	}
	if stderr.Len() != 0 {
		t.Fatalf("stderr = %q, want empty", stderr.String())
	}
}

func TestLoadPackagesUsesPinnedSettings(t *testing.T) {
	loaded, err := loadPackages("./testdata/samplepkg", loadOptions{
		Dir: filepath.FromSlash("."),
		Env: append(os.Environ(),
			"CGO_ENABLED=1",
			"GO111MODULE=off",
		),
	})
	if err != nil {
		t.Fatalf("load packages: %v", err)
	}
	if len(loaded) != 1 {
		t.Fatalf("package count = %d, want 1", len(loaded))
	}
	if loaded[0].Name != "samplepkg" {
		t.Fatalf("package name = %q, want samplepkg", loaded[0].Name)
	}
	if !contains(loaded[0].CompiledGoFiles, "testdata/samplepkg/sample.go") {
		t.Fatalf("compiled go files = %v, want sample.go", loaded[0].CompiledGoFiles)
	}
}

func TestBuildSSADumpForSamplePackage(t *testing.T) {
	loaded, err := loadPackageSet("./testdata/samplepkg", loadOptions{
		Dir: filepath.FromSlash("."),
	})
	if err != nil {
		t.Fatalf("load package set: %v", err)
	}

	dump, err := buildSSADump(loaded.Packages)
	if err != nil {
		t.Fatalf("build SSA dump: %v", err)
	}

	function := findSSAFunction(dump, loaded.Summaries[0].PackagePath, "Identity")
	if function == nil {
		t.Fatalf("SSA dump missing Identity function: %+v", dump)
	}
	if !hasSSAInstructionContaining(*function, "return") {
		t.Fatalf("Identity SSA instructions = %+v, want return instruction", function.Blocks)
	}
}

func TestPinnedPackageConfigUsesFixedSettings(t *testing.T) {
	config := pinnedPackageConfig(loadOptions{
		Dir: filepath.FromSlash("."),
		Env: []string{
			"CGO_ENABLED=1",
			"GO111MODULE=off",
			"PATH=/bin",
		},
	})

	if config.Mode != packageLoadMode {
		t.Fatalf("mode = %v, want %v", config.Mode, packageLoadMode)
	}
	if config.Tests {
		t.Fatal("tests = true, want false")
	}
	if len(config.BuildFlags) != 1 || config.BuildFlags[0] != "-mod=readonly" {
		t.Fatalf("build flags = %v, want [-mod=readonly]", config.BuildFlags)
	}
	if got := envValue(config.Env, "CGO_ENABLED"); got != "0" {
		t.Fatalf("CGO_ENABLED = %q, want 0", got)
	}
	if got := envValue(config.Env, "GO111MODULE"); got != "on" {
		t.Fatalf("GO111MODULE = %q, want on", got)
	}
	if got := envValue(config.Env, "PATH"); got != "/bin" {
		t.Fatalf("PATH = %q, want /bin", got)
	}
}

func contains(values []string, want string) bool {
	for _, value := range values {
		if value == want {
			return true
		}
	}
	return false
}

func envValue(env []string, key string) string {
	prefix := key + "="
	for _, entry := range env {
		if strings.HasPrefix(entry, prefix) {
			return strings.TrimPrefix(entry, prefix)
		}
	}
	return ""
}

func findSSAFunction(dump ssaDump, packagePath string, functionName string) *ssaFunctionDump {
	for _, pkg := range dump.Packages {
		if pkg.PackagePath != packagePath {
			continue
		}
		for index := range pkg.Functions {
			if pkg.Functions[index].Name == functionName {
				return &pkg.Functions[index]
			}
		}
	}
	return nil
}

func hasSSAInstructionContaining(function ssaFunctionDump, substring string) bool {
	for _, block := range function.Blocks {
		for _, instruction := range block.Instructions {
			if strings.Contains(instruction, substring) {
				return true
			}
		}
	}
	return false
}
