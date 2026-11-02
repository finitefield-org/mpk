//go:build go1.25 && (linux || darwin)

package main

import (
	"bytes"
	"encoding/hex"
	"os"
	"path/filepath"
	"testing"

	"golang.org/x/mod/module"
)

func TestPracticalModuleObservationsKeepOriginalReplacementGraph(t *testing.T) {
	practicalContractTestOutput(t, "modules-owned")
	owner, _ := practicalStorageTestOwnerWithOverlays(t, map[string][]byte{
		"go.mod":          []byte("module example.com/policy\ngo 1.25.0\nrequire (\nexample.com/alias v1.0.0\nexample.com/shared v1.0.0\n)\nreplace (\nexample.com/alias => ./wildcard\nexample.com/alias v1.0.0 => \"\\u002e/local-a\" // exact version wins\nexample.com/shared => ./local-b\nexample.com/policy => ./wildcard\nexample.com/unselected => ./wildcard\nexample.com/unselected => ./wildcard\n)\n"),
		"policy.go":       []byte("package policy\nimport(a \"example.com/alias\";b \"example.com/shared\";\"net/http\")\nfunc Root(x uint16)uint16{return x+1}\nfunc Unused(r *http.Request)uint16{go func(){select{}}();return a.Value()+b.Value()}\nfunc init(){panic(\"original initialization must never execute\")}\n"),
		"local-a/go.mod":  []byte("module example.com/alias\ngo 1.22\n"),
		"local-a/a.go":    []byte("package a\nfunc Value()uint16{return 1}\n"),
		"local-b/go.mod":  []byte("module example.com/shared\ngo 1.23\n"),
		"local-b/b.go":    []byte("package b\nfunc Value()uint16{return 2}\n"),
		"wildcard/go.mod": []byte("module example.com/alias\ngo 1.24\n"),
	}, "example.com/policy", nil)
	defer owner.Close()
	roots := []practicalClosureRef{practicalClosureTestRef(t, owner, "example.com/policy", "Root")}
	input, err := owner.moduleObservationInputs(roots)
	if err != nil {
		t.Fatal(err)
	}
	packet := practicalCaptureInputEvidence(t, input.exports.capture)
	packet["export_inputs_hex"] = hex.EncodeToString(input.exports.canonical)
	packet["module_observations_hex"] = hex.EncodeToString(input.canonical)
	retainPracticalSyntaxEvidence(t, "complete-original-module-observations.json", packet)
	original, originalExports := bytes.Clone(input.canonical), bytes.Clone(input.exports.canonical)
	clear(input.canonical)
	clear(input.exports.canonical)
	for i := range input.exports.capture.snapshot.sources {
		clear(input.exports.capture.snapshot.sources[i].data)
	}
	again, err := owner.moduleObservationInputs(roots)
	if err != nil || !bytes.Equal(original, again.canonical) || !bytes.Equal(originalExports, again.exports.canonical) {
		t.Fatal("module observation handoff aliases original owner", err)
	}
	count, packages, objects := len(owner.modules), len(owner.packages), len(owner.inputs.snapshot.ids)
	owner.Close()
	if !bytes.Equal(original, again.canonical) || !bytes.Equal(originalExports, again.exports.canonical) {
		t.Fatal("module observation input retired with owner")
	}
	if got, err := owner.moduleObservationInputs(roots); err == nil || got.canonical != nil {
		t.Fatal("closed owner accepted")
	}
	var absent *practicalOriginalPackages
	if got, err := absent.moduleObservationInputs(roots); err == nil || got.canonical != nil {
		t.Fatal("nil owner accepted")
	}
	retainPracticalSyntaxEvidence(t, "original-module-observation-ownership.json", map[string]any{"selected_modules": count, "packages": packages, "complete_original_objects": objects, "returned_buffers_independent": true, "owner_retirement_keeps_handoff": true, "closed_and_nil_owners_rejected": true, "source_or_context_program_proof": false})
}

func TestPracticalModuleObservationsKeepOriginalVersionedReplacement(t *testing.T) {
	practicalContractTestOutput(t, "versioned-owned")
	compiler := practicalPackageTestCompiler(t)
	roots, _ := practicalTestCaptureRoots(t)
	roots = append(roots, practicalSourceRoot{Capability: "original-go-sdk", Directory: compiler.selection.GoRoot})
	main, cache := practicalModuleLocation{capability: roots[0].Capability}, practicalModuleLocation{capability: roots[1].Capability}
	identity := module.Version{Path: "example.com/Dep", Version: "v1.2.0"}
	sums := writePracticalResolutionDependency(t, roots[1].Directory, identity, []byte("module example.com/Dep\ngo 1.22\n"))
	manifest := []byte("module example.com/policy\ngo 1.25.0\nrequire example.com/Dep v1.1.0\nreplace example.com/Dep v1.1.0 => example.com/Dep v1.2.0\n")
	policy := []byte("package policy\nimport dep \"example.com/Dep\"\nfunc Root(x uint16)uint16{return x+1}\nfunc Unused(x int64)int64{return dep.Identity(x)}\nfunc init(){panic(\"original initialization must never execute\")}\n")
	writePracticalTestFile(t, filepath.Join(roots[0].Directory, "go.mod"), manifest, 0o600)
	writePracticalTestFile(t, filepath.Join(roots[0].Directory, "go.sum"), []byte(sums), 0o600)
	writePracticalTestFile(t, filepath.Join(roots[0].Directory, "policy.go"), policy, 0o600)
	retainPracticalSyntaxEvidence(t, "module-original-inputs.json", map[string]any{"roots": roots, "manifest": manifest, "checksums": sums, "policy": policy})
	session := practicalTestSession(t, roots)
	discovery, err := session.discoverPracticalModules(main, cache, compiler.selection)
	if err != nil {
		t.Fatal(err)
	}
	retainPracticalSyntaxEvidence(t, "module-original-discovery.json", map[string]any{"stdout": discovery.output, "stderr": discovery.observation.diagnostics})
	if err := session.capturePracticalStandardLibrary(compiler, roots[2].Capability); err != nil {
		t.Fatal(err)
	}
	exports, err := session.capturePracticalPackageExports(compiler, discovery, roots[2].Capability, main, "example.com/policy")
	if err != nil {
		t.Fatal(err)
	}
	retainPracticalSyntaxEvidence(t, "module-original-exports.json", map[string]any{"stdout": exports.output, "stderr": exports.diagnostics})
	sealed, err := session.sealPracticalReplaySources()
	if err != nil {
		t.Fatal(err)
	}
	retainPracticalStandardSnapshot(t, sealed.snapshot)
	retainPracticalStandardSnapshot(t, exports.snapshot)
	image, err := materializePracticalReplaySources(t.TempDir(), []practicalReplaySources{sealed, exports.replay}, &compiler, roots[2].Capability)
	if err != nil {
		t.Fatal(err)
	}
	defer image.Close()
	for _, root := range roots {
		if err := removePracticalReplayImage(root.Directory); err != nil {
			t.Fatal(err)
		}
		if _, err := os.Lstat(root.Directory); !os.IsNotExist(err) {
			t.Fatal("original preparation root survived", err)
		}
	}
	for _, snapshot := range []practicalSourceSnapshot{sealed.snapshot, exports.snapshot} {
		for _, source := range snapshot.buffers {
			clear(source.data)
		}
	}
	owner, err := image.originalPackages(main, cache, discovery, exports, roots[2].Capability)
	if err != nil {
		t.Fatal(err)
	}
	defer owner.Close()
	input, err := owner.moduleObservationInputs([]practicalClosureRef{practicalClosureTestRef(t, owner, "example.com/policy", "Root")})
	if err != nil {
		t.Fatal(err)
	}
	packet := practicalCaptureInputEvidence(t, input.exports.capture)
	packet["export_inputs_hex"] = hex.EncodeToString(input.exports.canonical)
	packet["module_observations_hex"] = hex.EncodeToString(input.canonical)
	retainPracticalSyntaxEvidence(t, "complete-original-module-observations.json", packet)
}
