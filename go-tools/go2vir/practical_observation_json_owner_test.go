//go:build go1.25 && (linux || darwin)

package main

import (
	"encoding/hex"
	"errors"
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"golang.org/x/mod/module"
)

func TestPracticalObservationJSONKeepsLargeOriginalGraphAndExportOwner(t *testing.T) {
	practicalContractTestOutput(t, "versioned-owned")
	candidate := practicalTestCompleteCompiler(t)
	base, err := filepath.EvalSymlinks(t.TempDir())
	if err != nil {
		t.Fatal(err)
	}
	newRoot := filepath.Join(base, strings.Repeat("original-sdk-", 14), strings.Repeat("physical-observation-", 10), "go")
	if err := os.MkdirAll(filepath.Dir(newRoot), 0o700); err != nil {
		t.Fatal(err)
	}
	// Darwin requires write permission on a moved directory to update its
	// parent entry. Only this disposable fixture root is unsealed; original
	// file bytes/modes stay fixed and the root is resealed before validation.
	if err := os.Chmod(candidate.ToolchainGoRootPath, 0o700); err != nil {
		t.Fatal(err)
	}
	if err := os.Rename(candidate.ToolchainGoRootPath, newRoot); err != nil {
		t.Fatal(err)
	}
	sealTestReleaseRoot(t, newRoot)
	candidate.ToolchainGoRootPath = newRoot
	compiler, err := validatePracticalCompilerImage(candidate)
	if err != nil {
		t.Fatal(err)
	}
	roots, _ := practicalTestCaptureRoots(t)
	main, cache := practicalModuleLocation{capability: roots[0].Capability}, practicalModuleLocation{capability: roots[1].Capability}
	manifest := []byte("module example.com/policy\ngo 1.25.0\nrequire (\nexample.com/a v1.0.0\nexample.com/b v1.0.0 // indirect\nexample.com/c v1.1.0\nexample.com/leaf v1.1.0 // indirect\nexample.com/q v1.1.0 // indirect\nexample.com/z v1.0.0 // indirect\nexample.com/alias v1.0.0\n)\nexclude example.com/never v1.0.0\nreplace example.com/alias v1.0.0 => example.com/Actual v1.1.0\n")
	sums := ""
	for _, row := range []struct{ path, version, body string }{
		{"example.com/a", "v1.0.0", "go 1.16\nrequire (\n" + strings.Repeat("example.com/c v1.0.0\n", 15000) + "example.com/never v1.0.0\n)\n"},
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
	exports, err := session.capturePracticalPackageExports(compiler, discovery, roots[2].Capability, main, "example.com/policy")
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
	if err := image.Close(); err != nil {
		fail(err)
	}
	for _, root := range roots {
		if _, err := os.Lstat(root.Directory); !os.IsNotExist(err) {
			t.Fatal("original root survived", err)
		}
	}
	input, err := owner.moduleGraphInputs([]practicalClosureRef{practicalClosureTestRef(t, owner, "example.com/policy", "Root")})
	if err != nil {
		fail(err)
	}
	for _, record := range [][]byte{input.modules.exports.canonical, input.modules.canonical, input.canonical} {
		if _, err := decodePracticalObservationJSON(record); err != nil {
			fail(err)
		}
	}
	packet := practicalCaptureInputEvidence(t, input.modules.exports.capture)
	packet["export_inputs_hex"] = hex.EncodeToString(input.modules.exports.canonical)
	packet["module_observations_hex"] = hex.EncodeToString(input.modules.canonical)
	packet["module_graph_inputs_hex"] = hex.EncodeToString(input.canonical)
	retainPracticalSyntaxEvidence(t, "complete-original-module-graph-inputs.json", packet)
	if len(owner.moduleGraph.output)*2 <= maximumStringBytes || len(owner.packageOutput)*2 <= maximumStringBytes {
		t.Fatal("original native output did not cross the former JSON limit", len(owner.moduleGraph.output), len(owner.packageOutput))
	}
	if _, ok := owner.packages["net/http"]; !ok || len(owner.packages) < 181 {
		t.Fatal("original HTTP/package universe was reduced")
	}
	retainPracticalSyntaxEvidence(t, "graph-original-ownership.json", map[string]any{"packages": len(owner.packages), "source_objects": len(owner.inputs.snapshot.ids), "selected_modules": len(owner.modules), "initial_modules": len(owner.moduleGraph.initial), "final_modules": len(owner.moduleGraph.final), "original_and_image_roots_retired": true, "error": fmt.Sprint(err), "original_graph_output_bytes": len(owner.moduleGraph.output), "original_package_output_bytes": len(owner.packageOutput), "common_string_bytes_unchanged": maximumStringBytes, "source_program_or_MVS_authority": false})
}
