//go:build go1.25 && (linux || darwin)

package main

import (
	"bytes"
	"encoding/hex"
	"encoding/json"
	"errors"
	"fmt"
	"os"
	"path/filepath"
	"reflect"
	"runtime"
	"testing"

	"golang.org/x/mod/module"
)

func TestPracticalCompilerInputsRequireOriginalImageOwner(t *testing.T) {
	for _, image := range []*practicalReplayImage{nil, {}, {closed: true}} {
		if input, err := image.compilerInputs(nil, nil); err == nil || input != nil {
			t.Fatal("compiler bytes accepted without original packages")
		}
		if input, err := image.compilerInputs(&practicalOriginalPackages{}, nil); err == nil || input != nil {
			t.Fatal("compiler bytes accepted without a live original image")
		}
	}
}

func TestPracticalCompilerInputsKeepWholeOriginalPreparedSDKAndPackages(t *testing.T) {
	practicalCompilerInputsKeepWholeOriginalPreparedSDKAndPackages(t, practicalPackageTestCompiler(t))
}

// This retains the very same whole application, dependency cache, SDK and Root
// selection as the original-owner test. The actual Linux SDK must also match the
// registered bundle, rather than being relabelled after producing its records.
func TestPracticalInstalledCompilerInputsKeepWholeRegisteredSDKAndPackages(t *testing.T) {
	if runtime.GOOS != "linux" || runtime.GOARCH != "amd64" {
		t.Skip("the registered SDK executes on Linux amd64")
	}
	prepared := practicalTestCompleteCompiler(t)
	candidate := practicalTestCompilerCandidateForBundle(t, prepared.ToolchainGoRootPath, "toolchain.go.go1.25.0.linux-amd64.candidate.v1")
	raw, err := os.ReadFile("../../release/bundles/candidates/go.json")
	if err != nil {
		t.Fatal(err)
	}
	var registered struct {
		Toolchains []struct {
			BundleID           string              `json:"bundle_id"`
			DistributionSHA256 string              `json:"distribution_sha256"`
			Components         []componentIdentity `json:"components"`
			Inventory          bundleInventory     `json:"inventory"`
		} `json:"toolchain_bundles"`
	}
	if err := json.Unmarshal(raw, &registered); err != nil || len(registered.Toolchains) != 1 {
		t.Fatal("registered Go toolchain", err)
	}
	want := registered.Toolchains[0]
	if !reflect.DeepEqual(candidate.Toolchain, toolchainIdentity{BundleID: want.BundleID, DistributionSHA256: want.DistributionSHA256, Components: want.Components}) || !reflect.DeepEqual(candidate.ToolchainInventory, want.Inventory) {
		t.Fatal("the complete original SDK differs from the registered bundle")
	}
	compiler, err := validatePracticalCompilerImage(candidate)
	if err != nil {
		t.Fatal(err)
	}
	practicalCompilerInputsKeepWholeOriginalPreparedSDKAndPackages(t, compiler)
}

func practicalCompilerInputsKeepWholeOriginalPreparedSDKAndPackages(t *testing.T, compiler practicalCompilerImage) {
	t.Helper()
	practicalContractTestOutput(t, "versioned-owned")
	roots, _ := practicalTestCaptureRoots(t)
	main, cache := practicalModuleLocation{capability: roots[0].Capability}, practicalModuleLocation{capability: roots[1].Capability}
	manifest := []byte("module example.com/policy\ngo 1.25.0\nrequire (\nexample.com/a v1.0.0\nexample.com/b v1.0.0 // indirect\nexample.com/c v1.1.0\nexample.com/leaf v1.1.0 // indirect\nexample.com/q v1.1.0 // indirect\nexample.com/z v1.0.0 // indirect\nexample.com/alias v1.0.0\n)\nexclude example.com/never v1.0.0\nreplace example.com/alias v1.0.0 => example.com/Actual v1.1.0\n")
	sums := ""
	for _, row := range []struct{ path, version, body string }{
		{"example.com/a", "v1.0.0", "go 1.16\nrequire (\nexample.com/c v1.0.0\nexample.com/c v1.0.0\nexample.com/never v1.0.0\n)\n"},
		{"example.com/b", "v1.0.0", "go 1.16\nrequire (\nexample.com/c v1.1.0\nexample.com/leaf v1.1.0\nexample.com/q v1.1.0\n)\n"},
		{"example.com/c", "v1.1.0", "go 1.22\n"},
		{"example.com/leaf", "v1.1.0", "go 1.22\n"},
		{"example.com/q", "v1.1.0", "go 1.22\n"},
		{"example.com/z", "v1.0.0", "go 1.22\nrequire example.com/q v1.0.0\n"},
		{"example.com/Actual", "v1.1.0", "go 1.22\n"},
	} {
		sums += writePracticalResolutionDependency(t, roots[1].Directory, module.Version{Path: row.path, Version: row.version}, []byte("module "+row.path+"\n"+row.body))
	}
	sums += practicalGraphTestModOnly(t, roots[1].Directory, module.Version{Path: "example.com/c", Version: "v1.0.0"}, []byte("module example.com/c\ngo 1.16\nfuture_directive ignored\nrequire (\nexample.com/leaf v1.0.0\nexample.com/a v1.0.0\n)\n"))
	sums += practicalGraphTestModOnly(t, roots[1].Directory, module.Version{Path: "example.com/leaf", Version: "v1.0.0"}, []byte("module example.com/leaf\n// original unselected zero-requirement leaf\n"))
	policy := []byte("package policy\nimport(a \"example.com/a\";c \"example.com/c\";alias \"example.com/alias\";\"net/http\")\nfunc Root(x uint16)uint16{return x+1}\nfunc Unused(r *http.Request,x int64)int64{go func(){select{}}();return a.Identity(c.Identity(alias.Identity(x)))}\nfunc init(){panic(\"original init must never execute\")}\n")
	writePracticalTestFile(t, filepath.Join(roots[0].Directory, "go.mod"), manifest, 0o600)
	writePracticalTestFile(t, filepath.Join(roots[0].Directory, "go.sum"), []byte(sums), 0o600)
	writePracticalTestFile(t, filepath.Join(roots[0].Directory, "policy.go"), policy, 0o600)
	practicalGraphTestOriginals(t, roots)
	roots = append(roots, practicalSourceRoot{Capability: "original-go-sdk", Directory: compiler.selection.GoRoot})
	practicalLiveRootComparisonStart(t, roots)
	session := practicalTestSession(t, roots)
	fail := func(err error) {
		var native *practicalGoDiscoveryFailure
		if errors.As(err, &native) {
			stdout, stderr := native.originalOutput()
			retainPracticalSyntaxEvidence(t, "graph-original-command-failure.json", map[string]any{"stdout": stdout, "stderr": stderr, "error": err.Error()})
		}
		t.Fatal(err)
	}
	discovery, err := session.discoverPracticalModules(main, cache, compiler.selection)
	if err != nil {
		fail(err)
	}
	retainPracticalSyntaxEvidence(t, "graph-original-modules.json", map[string]any{"stdout": discovery.output, "stderr": discovery.observation.diagnostics})
	discovery, err = session.capturePracticalModuleGraph(main, cache, discovery, compiler.selection)
	if err != nil {
		fail(err)
	}
	retainPracticalSyntaxEvidence(t, "graph-original-edges.json", map[string]any{"stdout": discovery.graph.output, "stderr": discovery.graph.diagnostics})
	if err := session.capturePracticalStandardLibrary(compiler, roots[2].Capability); err != nil {
		fail(err)
	}
	exports, err := session.capturePracticalPackageExportsObserved(compiler, discovery, roots[2].Capability, main, "example.com/policy", practicalLiveExportRootObserver(t))
	if err != nil {
		fail(err)
	}
	retainPracticalSyntaxEvidence(t, "graph-original-exports.json", map[string]any{"stdout": exports.output, "stderr": exports.diagnostics})
	sealed, err := session.sealPracticalReplaySources()
	if err != nil {
		fail(err)
	}
	retainPracticalStandardSnapshot(t, sealed.snapshot)
	retainPracticalStandardSnapshot(t, exports.snapshot)
	practicalLiveRootComparisonFinish(t, sealed.snapshot)
	image, err := materializePracticalReplaySources(t.TempDir(), []practicalReplaySources{sealed, exports.replay}, &compiler, roots[2].Capability)
	if err != nil {
		fail(err)
	}
	defer image.Close()
	for _, root := range roots {
		if err := removePracticalReplayImage(root.Directory); err != nil {
			fail(err)
		}
	}
	for _, snapshot := range []practicalSourceSnapshot{sealed.snapshot, exports.snapshot} {
		for _, buffer := range snapshot.buffers {
			clear(buffer.data)
		}
	}
	owner, err := image.originalPackages(main, cache, discovery, exports, roots[2].Capability)
	if err != nil {
		fail(err)
	}
	defer owner.Close()
	request, err := newPracticalCandidateRequest([]practicalCandidateSelector{{Package: "example.com/policy", Function: "Root"}})
	if err != nil {
		fail(err)
	}
	candidateInputs, err := image.candidateCompilerInputs(owner, request)
	if err != nil {
		fail(err)
	}
	defer candidateInputs.Close()
	selection := candidateInputs.selection
	if roots := selection.rootRefs(); len(roots) != 1 || roots[0] != practicalClosureTestRef(t, owner, "example.com/policy", "Root") {
		t.Fatal("candidate request did not select the original Root declaration")
	}
	compilerInputs := candidateInputs.compiler
	if !bytes.Equal(selection.selection.canonical, compilerInputs.exports.capture.selection.canonical) {
		t.Fatal("compiler inputs crossed the candidate's original source selection")
	}
	if err := image.Close(); err != nil {
		fail(err)
	}
	for _, root := range roots {
		if _, err := os.Lstat(root.Directory); !os.IsNotExist(err) {
			t.Fatal("original root survived", err)
		}
	}
	input, err := candidateInputs.moduleGraphInputs()
	if err != nil {
		fail(err)
	}
	packet := practicalCaptureInputEvidence(t, input.modules.exports.capture)
	packet["export_inputs_hex"] = hex.EncodeToString(input.modules.exports.canonical)
	packet["module_observations_hex"] = hex.EncodeToString(input.modules.canonical)
	packet["module_graph_inputs_hex"] = hex.EncodeToString(input.canonical)
	if !bytes.Equal(compilerInputs.exports.canonical, input.modules.exports.canonical) {
		t.Fatal("compiler/source export owners differ")
	}
	packet["compiler_inputs_hex"] = hex.EncodeToString(compilerInputs.canonical)
	packet["candidate_request_hex"] = hex.EncodeToString(selection.requestBytes())
	unselected, err := newPracticalCandidateRequest([]practicalCandidateSelector{{Package: "example.com/policy", Function: "Unused"}})
	if err != nil {
		fail(err)
	}
	packet["unselected_candidate_request_hex"] = hex.EncodeToString(unselected.requestBytes())
	files := []any{}
	totalBytes := int64(0)
	large := 0
	directory := os.Getenv("MPK_GO_PRACTICAL_SYNTAX_EVIDENCE")
	if directory != "" {
		if err := os.Mkdir(filepath.Join(directory, "original-compiler-files"), 0o700); err != nil {
			fail(err)
		}
	}
	for _, file := range compilerInputs.files {
		if file.entry.SizeBytes > maximumCandidateBytes {
			large++
		}
		if len(file.data) != int(file.entry.SizeBytes) || sha256Hex(file.data) != file.entry.SHA256 {
			t.Fatal("compiler bytes changed after image retirement")
		}
		name := file.entry.SHA256 + ".raw"
		if directory != "" {
			path := filepath.Join(directory, "original-compiler-files", name)
			if previous, err := os.ReadFile(path); err == nil {
				if !bytes.Equal(previous, file.data) {
					t.Fatal("compiler raw hash collision")
				}
			} else if !os.IsNotExist(err) {
				fail(err)
			} else {
				if err := os.WriteFile(path, file.data, 0o600); err != nil {
					fail(err)
				}
			}
		}
		files = append(files, map[string]any{"entry": file.entry, "raw_file": name})
		totalBytes += file.entry.SizeBytes
	}
	if large == 0 || len(files) != len(compiler.candidate.ToolchainInventory.Files) {
		t.Fatal("compiler universe was narrowed to source-file sizes")
	}
	packet["compiler_files"] = files
	retainPracticalSyntaxEvidence(t, "compiler-original-ownership.json", map[string]any{"files": len(files), "bytes": totalBytes, "files_larger_than_source_limit": large, "whole_compiler_survives_original_and_image_retirement": true, "source_file_limit_unchanged": maximumCandidateBytes, "source_aggregate_limit_unchanged": maximumCapturedBytes, "installed_or_compiler_correctness_authority": false})
	retainPracticalSyntaxEvidence(t, "complete-original-module-graph-inputs.json", packet)
	practicalLiveCombinedOwnerFinish(t, exports.buildCacheDirectory, len(sealed.snapshot.ids), len(exports.snapshot.ids), len(owner.inputs.snapshot.ids))
	if _, ok := owner.packages["net/http"]; !ok || len(owner.packages) < 181 {
		t.Fatal("original HTTP/package universe was reduced")
	}
	retainPracticalSyntaxEvidence(t, "graph-original-ownership.json", map[string]any{"packages": len(owner.packages), "source_objects": len(owner.inputs.snapshot.ids), "selected_modules": len(owner.modules), "initial_modules": len(owner.moduleGraph.initial), "final_modules": len(owner.moduleGraph.final), "original_and_image_roots_retired": true, "error": fmt.Sprint(err), "source_program_or_MVS_authority": false})
}
