package main

import (
	"errors"
	"go/ast"
	"go/parser"
	"go/token"
	"os"
	"path/filepath"
	"reflect"
	"sort"
	"strings"
	"testing"
)

func TestPreflightCapturesTheExactImportClosureAndInputs(t *testing.T) {
	root := copyPreflightFixture(t)
	request := preflightRequest()
	capture, err := captureSourceTree(root, request)
	if err != nil {
		t.Fatalf("capture valid source tree: %v", err)
	}
	if capture.ModulePath != "example.com/mpk/vector" || capture.SelectedPackage != request.Package {
		t.Fatalf("capture identity = %q/%q", capture.ModulePath, capture.SelectedPackage)
	}
	wantInputs := []string{
		"go.mod:build_manifest",
		"go.sum:lockfile",
		"helper/helper.go:source",
		"identity.go:source",
		"identity_contract.json:contract",
	}
	var gotInputs []string
	for _, input := range capture.Inputs {
		gotInputs = append(gotInputs, input.NormalizedPath+":"+input.Kind)
		if input.state.absolutePath != "" || input.state.info != nil {
			t.Fatal("successful capture retained an original-file capability")
		}
		if input.SHA256 != sha256Hex(input.Bytes) {
			t.Fatalf("captured digest mismatch for %s", input.NormalizedPath)
		}
	}
	if !reflect.DeepEqual(gotInputs, wantInputs) {
		t.Fatalf("captured inputs = %v, want %v", gotInputs, wantInputs)
	}
	wantPackages := []string{"example.com/mpk/vector", "example.com/mpk/vector/helper"}
	var gotPackages []string
	for _, packageRecord := range capture.Packages {
		gotPackages = append(gotPackages, packageRecord.ImportPath)
	}
	if !reflect.DeepEqual(gotPackages, wantPackages) {
		t.Fatalf("captured packages = %v, want %v", gotPackages, wantPackages)
	}
}

func TestPreflightRejectsClosedProfileViolations(t *testing.T) {
	tests := []struct {
		name   string
		mutate func(*testing.T, string, *lowerRequest)
		code   string
		status string
	}{
		{
			name: "workspace",
			mutate: func(t *testing.T, root string, _ *lowerRequest) {
				writeTestFile(t, filepath.Join(root, "go.work"), []byte("go 1.23\nuse .\n"), 0o600)
			},
			code: "GO_WORKSPACE_FORBIDDEN", status: "rejected",
		},
		{
			name: "dependency directive",
			mutate: func(t *testing.T, root string, _ *lowerRequest) {
				writeTestFile(t, filepath.Join(root, "go.mod"), []byte("module example.com/mpk/vector\n\ngo 1.23\n\nrequire example.com/external v1.0.0\n"), 0o600)
			},
			code: "GO_MODULE_POLICY", status: "rejected",
		},
		{
			name: "nonempty sum",
			mutate: func(t *testing.T, root string, _ *lowerRequest) {
				writeTestFile(t, filepath.Join(root, "go.sum"), []byte("dependency checksum\n"), 0o600)
			},
			code: "GO_MODULE_DEPENDENCY", status: "rejected",
		},
		{
			name: "intermediate nested module",
			mutate: func(t *testing.T, root string, _ *lowerRequest) {
				writeTestFile(t, filepath.Join(root, "identity.go"), []byte("package vector\n\nimport \"example.com/mpk/vector/nested/helper\"\n\nfunc Identity(value int8) int8 { return helper.Identity(value) }\n"), 0o600)
				if err := os.MkdirAll(filepath.Join(root, "nested", "helper"), 0o700); err != nil {
					t.Fatalf("create nested package: %v", err)
				}
				writeTestFile(t, filepath.Join(root, "nested", "go.mod"), []byte("module example.com/mpk/vector/nested\n\ngo 1.23\n"), 0o600)
				writeTestFile(t, filepath.Join(root, "nested", "helper", "helper.go"), []byte("package helper\n\nfunc Identity(value int8) int8 { return value }\n"), 0o600)
			},
			code: "GO_MODULE_POLICY", status: "rejected",
		},
		{
			name: "build constraint",
			mutate: func(t *testing.T, root string, _ *lowerRequest) {
				writeTestFile(t, filepath.Join(root, "identity.go"), []byte("//go:build linux\n\npackage vector\n"), 0o600)
			},
			code: "GO_BUILD_CONSTRAINT", status: "rejected",
		},
		{
			name: "late source directive",
			mutate: func(t *testing.T, root string, _ *lowerRequest) {
				writeTestFile(t, filepath.Join(root, "identity.go"), []byte("package vector\n\n//go:noinline\nfunc Identity(value int8) int8 { return value }\n"), 0o600)
			},
			code: "GO_SOURCE_DIRECTIVE", status: "rejected",
		},
		{
			name: "documentation package before imports",
			mutate: func(t *testing.T, root string, _ *lowerRequest) {
				writeTestFile(t, filepath.Join(root, "identity.go"), []byte("package documentation\n\nimport \"fmt\"\n"), 0o600)
			},
			code: "GO_BUILD_CONSTRAINT", status: "rejected",
		},
		{
			name: "auxiliary source",
			mutate: func(t *testing.T, root string, _ *lowerRequest) {
				writeTestFile(t, filepath.Join(root, "native.c"), []byte("int f(void) { return 0; }\n"), 0o600)
			},
			code: "GO_CGO_OR_AUX_SOURCE", status: "rejected",
		},
		{
			name: "contract set mismatch",
			mutate: func(_ *testing.T, _ string, request *lowerRequest) {
				request.Contracts = []string{}
			},
			code: "GO_CONTRACT_FUNCTION", status: "rejected",
		},
		{
			name: "standard library import",
			mutate: func(t *testing.T, root string, _ *lowerRequest) {
				writeTestFile(t, filepath.Join(root, "identity.go"), []byte("package vector\n\nimport \"fmt\"\n\nfunc Identity(value int8) int8 { fmt.Print(value); return value }\n"), 0o600)
			},
			code: "GO_SUBSET_IMPORT", status: "rejected",
		},
	}
	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			root := copyPreflightFixture(t)
			request := preflightRequest()
			test.mutate(t, root, &request)
			_, err := captureSourceTree(root, request)
			assertFrontendFailure(t, err, test.status, test.code)
		})
	}
}

func TestPreflightRejectsSymlinkAndConcurrentMutation(t *testing.T) {
	t.Run("symlink", func(t *testing.T) {
		root := copyPreflightFixture(t)
		target := filepath.Join(root, "target.go")
		writeTestFile(t, target, []byte("package vector\n"), 0o600)
		identity := filepath.Join(root, "identity.go")
		if err := os.Remove(identity); err != nil {
			t.Fatalf("remove identity fixture: %v", err)
		}
		if err := os.Symlink(target, identity); err != nil {
			t.Fatalf("create source symlink: %v", err)
		}
		_, err := captureSourceTree(root, preflightRequest())
		assertFrontendFailure(t, err, "rejected", "GO_CAPTURE_FILE_KIND")
	})

	t.Run("concurrent source mutation", func(t *testing.T) {
		root := copyPreflightFixture(t)
		observer := captureObserverFunc(func() error {
			writeTestFile(t, filepath.Join(root, "identity.go"), []byte("package vector\n\nfunc Identity(value int8) int8 { return value + 1 }\n"), 0o600)
			return nil
		})
		_, err := captureSourceTreeObserved(root, preflightRequest(), observer)
		assertFrontendFailure(t, err, "rejected", "GO_CAPTURE_CHANGED")
	})
}

func TestPreflightLimitCountersUseInclusiveCeilings(t *testing.T) {
	counters := captureCounters{
		candidates:         maximumCandidateEntries - 1,
		manifestInputs:     maximumManifestInputs - 1,
		capturedBytes:      maximumCapturedBytes - 1,
		contractCandidates: maximumContracts - 1,
		contractBytes:      maximumTotalContractBytes - 1,
	}
	if err := counters.admitCandidate(1, true, true); err != nil {
		t.Fatalf("inclusive candidate ceilings rejected: %v", err)
	}
	if err := counters.admitCandidate(0, false, false); err == nil {
		t.Fatal("candidate count above the ceiling was accepted")
	}
	directories := captureCounters{directories: maximumVisitedDirectories - 1, directoryEntries: maximumDirectoryEntries - 1}
	if err := directories.addDirectory(1); err != nil {
		t.Fatalf("inclusive directory ceilings rejected: %v", err)
	}
	if err := directories.addDirectory(0); err == nil {
		t.Fatal("directory count above the ceiling was accepted")
	}
}

func TestPreflightDirectoryEnumerationStopsAtTheSemanticLimit(t *testing.T) {
	root := t.TempDir()
	writeTestFile(t, filepath.Join(root, "first"), []byte("1"), 0o600)
	writeTestFile(t, filepath.Join(root, "second"), []byte("2"), 0o600)
	entries, overflow, err := readDirectoryState(root, 1)
	if err != nil {
		t.Fatalf("bounded directory enumeration: %v", err)
	}
	if !overflow || entries != nil {
		t.Fatalf("bounded enumeration = %#v/%t, want nil/true", entries, overflow)
	}
}

func TestPreflightSyntaxLimitIsInclusiveAcrossTheWholeClosure(t *testing.T) {
	file, err := parser.ParseFile(token.NewFileSet(), "input.go", []byte("package input\nfunc F() {}\n"), 0)
	if err != nil {
		t.Fatalf("parse syntax-limit fixture: %v", err)
	}
	count := uint64(0)
	for range ast.Preorder(file) {
		count++
	}
	files := []*ast.File{file, file}
	if err := countSyntaxNodes(files, count*2); err != nil {
		t.Fatalf("inclusive syntax-node ceiling rejected: %v", err)
	}
	err = countSyntaxNodes(files, count*2-1)
	assertFrontendFailure(t, err, "rejected", "GO_LIMIT_SYNTAX")
}

func TestPreflightRejectsCaseFoldedCapturedPathCollision(t *testing.T) {
	err := validateCapturedPathUniqueness([]string{"Identity.go", "identity.go"})
	assertFrontendFailure(t, err, "rejected", "GO_CAPTURE_PATH")
}

func preflightRequest() lowerRequest {
	request := loadFixtureRequestWithoutTesting()
	request.Package = "example.com/mpk/vector"
	request.Function = "example.com/mpk/vector.Identity"
	request.Contracts = []string{"identity_contract.json"}
	return request
}

func loadFixtureRequestWithoutTesting() lowerRequest {
	return lowerRequest{
		SourceRoot:                  logicalSourceRoot,
		SemanticProfile:             goSemanticProfile,
		Target:                      goTarget,
		FrontendBundleID:            "frontend.go.test.v0",
		FrontendSHA256:              strings.Repeat("1", 64),
		ReleaseRegistryID:           registryID,
		ReleaseRegistrySHA256:       strings.Repeat("2", 64),
		ToolchainBundleID:           "toolchain.go.test.v0",
		ToolchainRoot:               logicalToolchain,
		ToolchainDistributionSHA256: strings.Repeat("3", 64),
		Contracts:                   []string{},
	}
}

func copyPreflightFixture(t *testing.T) string {
	t.Helper()
	source := filepath.Join("testdata", "preflight", "valid")
	destination := t.TempDir()
	err := filepath.Walk(source, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}
		relative, err := filepath.Rel(source, path)
		if err != nil || relative == "." {
			return err
		}
		target := filepath.Join(destination, relative)
		if info.IsDir() {
			return os.MkdirAll(target, 0o700)
		}
		content, err := os.ReadFile(path)
		if err != nil {
			return err
		}
		return os.WriteFile(target, content, 0o600)
	})
	if err != nil {
		t.Fatalf("copy preflight fixture: %v", err)
	}
	return destination
}

func writeTestFile(t *testing.T, path string, content []byte, mode os.FileMode) {
	t.Helper()
	if err := os.WriteFile(path, content, mode); err != nil {
		t.Fatalf("write test file %s: %v", path, err)
	}
}

func assertFrontendFailure(t *testing.T, err error, status, code string) {
	t.Helper()
	if err == nil {
		t.Fatalf("operation succeeded, want %s/%s", status, code)
	}
	var failure *frontendFailure
	if !errors.As(err, &failure) {
		t.Fatalf("error = %T %v, want frontendFailure", err, err)
	}
	if failure.Status != status || failure.Code != code {
		t.Fatalf("failure = %s/%s, want %s/%s (%v)", failure.Status, failure.Code, status, code, err)
	}
}

func sortedKeys(values map[string]struct{}) []string {
	keys := make([]string, 0, len(values))
	for key := range values {
		keys = append(keys, key)
	}
	sort.Strings(keys)
	return keys
}
