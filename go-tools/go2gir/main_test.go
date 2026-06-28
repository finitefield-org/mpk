package main

import (
	"bytes"
	"encoding/base64"
	"encoding/binary"
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
	if result.Status != "gir-lowered" {
		t.Fatalf("status = %q, want gir-lowered", result.Status)
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
	if result.SSA == nil {
		t.Fatal("SSA dump missing")
	}
	identity := findSSAFunction(*result.SSA, got.PackagePath, "Identity")
	if identity == nil {
		t.Fatalf("SSA dump missing Identity function: %+v", result.SSA)
	}
	if identity.Signature != "func(value int64) int64" {
		t.Fatalf("Identity signature = %q, want func(value int64) int64", identity.Signature)
	}
	if !hasSSAInstructionContaining(*identity, "return") {
		t.Fatalf("Identity SSA instructions = %+v, want return instruction", identity.Blocks)
	}
	if result.GIR == nil {
		t.Fatal("GIR missing")
	}
	if result.GIR.GIRHash == "" {
		t.Fatal("GIR hash missing")
	}
	if result.GIREmission == nil {
		t.Fatal("GIR emission missing")
	}
	if result.GIREmission.Schema != girEmitSchema {
		t.Fatalf("GIR emission schema = %q, want %q", result.GIREmission.Schema, girEmitSchema)
	}
	if result.GIREmission.GIRHash != result.GIR.GIRHash {
		t.Fatalf("GIR emission hash = %q, want module hash %q", result.GIREmission.GIRHash, result.GIR.GIRHash)
	}
	identityGIR := findGIRFunction(*result.GIR, got.PackagePath, "Identity")
	if identityGIR == nil {
		t.Fatalf("GIR missing Identity function: %+v", result.GIR)
	}
	if !hasGIRTerminatorKind(*identityGIR, "Return") {
		t.Fatalf("Identity GIR blocks = %+v, want Return terminator", identityGIR.Blocks)
	}
}

func TestRunEmitsDeterministicGIRHash(t *testing.T) {
	first := runSuccessfulPackage(t, "./testdata/max64")
	second := runSuccessfulPackage(t, "./testdata/max64")

	if first.GIR == nil || second.GIR == nil {
		t.Fatal("GIR missing")
	}
	if first.GIR.GIRHash == "" {
		t.Fatal("GIR hash missing")
	}
	if first.GIR.GIRHash != second.GIR.GIRHash {
		t.Fatalf("GIR hash changed between runs: %q != %q", first.GIR.GIRHash, second.GIR.GIRHash)
	}
	if first.GIREmission == nil || second.GIREmission == nil {
		t.Fatal("GIR emission missing")
	}
	if first.GIREmission.Schema != girEmitSchema {
		t.Fatalf("GIR emission schema = %q, want %q", first.GIREmission.Schema, girEmitSchema)
	}
	if first.GIREmission.CanonicalJSON != second.GIREmission.CanonicalJSON {
		t.Fatalf("canonical JSON changed between runs:\n%s\n---\n%s", first.GIREmission.CanonicalJSON, second.GIREmission.CanonicalJSON)
	}
	if first.GIREmission.BinaryBase64 != second.GIREmission.BinaryBase64 {
		t.Fatal("canonical binary changed between runs")
	}
	if strings.Contains(first.GIREmission.CanonicalJSON, "gir_hash") {
		t.Fatalf("canonical hash input must exclude gir_hash: %s", first.GIREmission.CanonicalJSON)
	}

	binaryPayload, err := base64.StdEncoding.DecodeString(first.GIREmission.BinaryBase64)
	if err != nil {
		t.Fatalf("decode binary payload: %v", err)
	}
	if !bytes.HasPrefix(binaryPayload, []byte(girBinaryMagic)) {
		t.Fatalf("binary payload missing magic prefix: %x", binaryPayload[:min(len(binaryPayload), len(girBinaryMagic))])
	}
	if len(binaryPayload) < len(girBinaryMagic)+8 {
		t.Fatalf("binary payload too short: %d", len(binaryPayload))
	}
	canonicalJSONBytes := []byte(first.GIREmission.CanonicalJSON)
	declaredLength := binary.BigEndian.Uint64(binaryPayload[len(girBinaryMagic) : len(girBinaryMagic)+8])
	if declaredLength != uint64(len(canonicalJSONBytes)) {
		t.Fatalf("binary length = %d, want %d", declaredLength, len(canonicalJSONBytes))
	}
	if !bytes.Equal(binaryPayload[len(girBinaryMagic)+8:], canonicalJSONBytes) {
		t.Fatal("binary payload body does not match canonical JSON")
	}
	if got := hashGIRBinary(binaryPayload); got != first.GIR.GIRHash {
		t.Fatalf("binary hash = %q, want %q", got, first.GIR.GIRHash)
	}

	var canonical girCanonicalModule
	if err := json.Unmarshal([]byte(first.GIREmission.CanonicalJSON), &canonical); err != nil {
		t.Fatalf("decode canonical JSON: %v\n%s", err, first.GIREmission.CanonicalJSON)
	}
	if canonical.SchemaVersion != girSchemaVersion {
		t.Fatalf("canonical schema = %q, want %q", canonical.SchemaVersion, girSchemaVersion)
	}
	if len(canonical.Packages) != 1 {
		t.Fatalf("canonical packages = %d, want 1", len(canonical.Packages))
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

func TestRunRejectsUnsupportedFixtures(t *testing.T) {
	tests := []struct {
		name    string
		path    string
		feature string
		reason  string
	}{
		{
			name:    "map",
			path:    "./testdata/unsupported/map",
			feature: "maps",
			reason:  "maps are rejected by Go subset v0",
		},
		{
			name:    "goroutine",
			path:    "./testdata/unsupported/goroutine",
			feature: "goroutines",
			reason:  "goroutines are rejected by Go subset v0",
		},
		{
			name:    "generic",
			path:    "./testdata/unsupported/generic",
			feature: "generics",
			reason:  "generic functions are rejected by Go subset v0",
		},
		{
			name:    "pointer",
			path:    "./testdata/unsupported/pointer",
			feature: "pointers",
			reason:  "pointers are rejected by Go subset v0",
		},
		{
			name:    "untyped int expression",
			path:    "./testdata/unsupported/untypedint",
			feature: "GIR lowering",
			reason:  "type untyped int is not lowered by GO-005",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			var stdout bytes.Buffer
			var stderr bytes.Buffer

			exitCode := run([]string{tt.path}, &stdout, &stderr)
			if exitCode != 1 {
				t.Fatalf("exit code = %d, want 1; stdout=%s stderr=%s", exitCode, stdout.String(), stderr.String())
			}
			if stderr.Len() != 0 {
				t.Fatalf("stderr = %q, want empty", stderr.String())
			}

			var result cliResult
			if err := json.Unmarshal(stdout.Bytes(), &result); err != nil {
				t.Fatalf("decode stdout: %v\n%s", err, stdout.String())
			}
			if result.Status != "rejected" {
				t.Fatalf("status = %q, want rejected", result.Status)
			}
			if result.SSA != nil {
				t.Fatalf("SSA = %+v, want nil for rejected package", result.SSA)
			}
			if result.GIR != nil || result.GIREmission != nil {
				t.Fatalf("GIR output = %+v / %+v, want nil for rejected package", result.GIR, result.GIREmission)
			}
			if !hasRejectedFeature(result.RejectedFeatures, tt.feature, tt.reason) {
				t.Fatalf("rejected features = %+v, want %q / %q", result.RejectedFeatures, tt.feature, tt.reason)
			}
		})
	}
}

func TestRunLowersMax64ToGIR(t *testing.T) {
	var stdout bytes.Buffer
	var stderr bytes.Buffer

	exitCode := run([]string{"./testdata/max64"}, &stdout, &stderr)
	if exitCode != 0 {
		t.Fatalf("exit code = %d, want 0; stderr=%s stdout=%s", exitCode, stderr.String(), stdout.String())
	}
	if stderr.Len() != 0 {
		t.Fatalf("stderr = %q, want empty", stderr.String())
	}

	var result cliResult
	if err := json.Unmarshal(stdout.Bytes(), &result); err != nil {
		t.Fatalf("decode stdout: %v\n%s", err, stdout.String())
	}
	if result.Status != "gir-lowered" {
		t.Fatalf("status = %q, want gir-lowered", result.Status)
	}
	if result.GIR == nil {
		t.Fatal("GIR missing")
	}

	function := findGIRFunction(*result.GIR, "github.com/finitefield-org/mpk/go-tools/go2gir/testdata/max64", "Max64")
	if function == nil {
		t.Fatalf("GIR missing Max64: %+v", result.GIR)
	}
	if len(function.Params) != 2 {
		t.Fatalf("params = %+v, want 2", function.Params)
	}
	if len(function.Results) != 1 {
		t.Fatalf("results = %+v, want 1", function.Results)
	}
	if !hasGIRLocal(*function, "max") {
		t.Fatalf("locals = %+v, want max", function.Locals)
	}
	if !hasGIRInstruction(*function, "BinOp", "signed_gt") {
		t.Fatalf("Max64 GIR blocks = %+v, want signed_gt BinOp", function.Blocks)
	}
	if !hasGIRTerminatorKind(*function, "Branch") {
		t.Fatalf("Max64 GIR blocks = %+v, want Branch terminator", function.Blocks)
	}
	if !hasGIRReturnValue(*function, "max") {
		t.Fatalf("Max64 GIR blocks = %+v, want return for max", function.Blocks)
	}
}

func TestRunLowersStructAndArrayFixturesToGIR(t *testing.T) {
	var stdout bytes.Buffer
	var stderr bytes.Buffer

	exitCode := run([]string{"./testdata/structarray"}, &stdout, &stderr)
	if exitCode != 0 {
		t.Fatalf("exit code = %d, want 0; stderr=%s stdout=%s", exitCode, stderr.String(), stdout.String())
	}
	if stderr.Len() != 0 {
		t.Fatalf("stderr = %q, want empty", stderr.String())
	}

	var result cliResult
	if err := json.Unmarshal(stdout.Bytes(), &result); err != nil {
		t.Fatalf("decode stdout: %v\n%s", err, stdout.String())
	}
	if result.Status != "gir-lowered" {
		t.Fatalf("status = %q, want gir-lowered", result.Status)
	}
	if result.GIR == nil {
		t.Fatal("GIR missing")
	}

	packagePath := "github.com/finitefield-org/mpk/go-tools/go2gir/testdata/structarray"
	pickLeft := findGIRFunction(*result.GIR, packagePath, "PickLeft")
	if pickLeft == nil {
		t.Fatalf("GIR missing PickLeft: %+v", result.GIR)
	}
	if !hasGIRParamTypeKind(*pickLeft, "pair", "struct") {
		t.Fatalf("PickLeft params = %+v, want struct param", pickLeft.Params)
	}
	if !hasGIRInstructionKind(*pickLeft, "Field") {
		t.Fatalf("PickLeft GIR blocks = %+v, want Field instruction", pickLeft.Blocks)
	}

	makePair := findGIRFunction(*result.GIR, packagePath, "MakePair")
	if makePair == nil {
		t.Fatalf("GIR missing MakePair: %+v", result.GIR)
	}
	if !hasGIRInstructionKind(*makePair, "MakeStruct") {
		t.Fatalf("MakePair GIR blocks = %+v, want MakeStruct instruction", makePair.Blocks)
	}

	pickSecond := findGIRFunction(*result.GIR, packagePath, "PickSecond")
	if pickSecond == nil {
		t.Fatalf("GIR missing PickSecond: %+v", result.GIR)
	}
	if !hasGIRParamTypeKind(*pickSecond, "values", "array") {
		t.Fatalf("PickSecond params = %+v, want array param", pickSecond.Params)
	}
	if !hasGIRInstructionKind(*pickSecond, "Index") {
		t.Fatalf("PickSecond GIR blocks = %+v, want Index instruction", pickSecond.Blocks)
	}

	makeArray := findGIRFunction(*result.GIR, packagePath, "MakeArray")
	if makeArray == nil {
		t.Fatalf("GIR missing MakeArray: %+v", result.GIR)
	}
	if !hasGIRInstructionKind(*makeArray, "MakeArray") {
		t.Fatalf("MakeArray GIR blocks = %+v, want MakeArray instruction", makeArray.Blocks)
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

func runSuccessfulPackage(t *testing.T, path string) cliResult {
	t.Helper()

	var stdout bytes.Buffer
	var stderr bytes.Buffer
	exitCode := run([]string{path}, &stdout, &stderr)
	if exitCode != 0 {
		t.Fatalf("exit code = %d, want 0; stderr=%s stdout=%s", exitCode, stderr.String(), stdout.String())
	}
	if stderr.Len() != 0 {
		t.Fatalf("stderr = %q, want empty", stderr.String())
	}

	var result cliResult
	if err := json.Unmarshal(stdout.Bytes(), &result); err != nil {
		t.Fatalf("decode stdout: %v\n%s", err, stdout.String())
	}
	if result.Status != "gir-lowered" {
		t.Fatalf("status = %q, want gir-lowered", result.Status)
	}
	return result
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

func min(a int, b int) int {
	if a < b {
		return a
	}
	return b
}

func hasRejectedFeature(features []rejectedFeature, feature string, reason string) bool {
	for _, got := range features {
		if got.Feature == feature && got.Reason == reason && got.Location != "" {
			return true
		}
	}
	return false
}

func findGIRFunction(module girModule, packagePath string, functionName string) *girFunction {
	for _, pkg := range module.Packages {
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

func hasGIRInstruction(function girFunction, kind string, op string) bool {
	for _, block := range function.Blocks {
		for _, instruction := range block.Instructions {
			if instruction.Kind == kind && instruction.Op == op {
				return true
			}
		}
	}
	return false
}

func hasGIRInstructionKind(function girFunction, kind string) bool {
	for _, block := range function.Blocks {
		for _, instruction := range block.Instructions {
			if instruction.Kind == kind {
				return true
			}
		}
	}
	return false
}

func hasGIRLocal(function girFunction, name string) bool {
	for _, local := range function.Locals {
		if local.Name == name {
			return true
		}
	}
	return false
}

func hasGIRParamTypeKind(function girFunction, name string, kind string) bool {
	for _, param := range function.Params {
		if param.Name == name && param.Type.Kind == kind {
			return true
		}
	}
	return false
}

func hasGIRTerminatorKind(function girFunction, kind string) bool {
	for _, block := range function.Blocks {
		if block.Terminator.Kind == kind {
			return true
		}
	}
	return false
}

func hasGIRReturnValue(function girFunction, varName string) bool {
	for _, block := range function.Blocks {
		if block.Terminator.Kind != "Return" {
			continue
		}
		for _, value := range block.Terminator.Values {
			if value.Var == varName {
				return true
			}
		}
	}
	return false
}
