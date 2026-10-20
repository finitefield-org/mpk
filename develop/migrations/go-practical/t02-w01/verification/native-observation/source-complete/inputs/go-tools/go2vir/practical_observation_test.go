//go:build go1.25 && (linux || darwin)

package main

import (
	"bytes"
	"encoding/json"
	"errors"
	"fmt"
	"os"
	"path/filepath"
	"testing"
)

type practicalObservationFixture struct {
	compiler  practicalCompilerImage
	roots     []practicalSourceRoot
	discovery practicalModuleDiscovery
	exports   practicalPackageExports
	sealed    practicalReplaySources
	loader    *practicalSourceTypes
}

func preparePracticalObservationFixture(t *testing.T) practicalObservationFixture {
	t.Helper()
	compiler := practicalPackageTestCompiler(t)
	roots, _ := practicalTestCaptureRoots(t)
	roots = append(roots, practicalSourceRoot{Capability: "original-go-sdk", Directory: compiler.selection.GoRoot})
	writePracticalTestFile(t, filepath.Join(roots[0].Directory, "go.mod"), []byte("module example.com/policy\ngo 1.25.0\n"), 0o600)
	writePracticalTestFile(t, filepath.Join(roots[0].Directory, "policy.go"), []byte("package policy\nfunc Root(x int64) int64{return x}\nfunc Unused(x map[int]int)int{return len(x)}\n"), 0o600)
	session := practicalTestSession(t, roots)
	discovery, err := session.discoverPracticalModules(practicalModuleLocation{capability: roots[0].Capability}, practicalModuleLocation{capability: roots[1].Capability}, compiler.selection)
	if err != nil {
		t.Fatal(err)
	}
	if err := session.capturePracticalStandardLibrary(compiler, roots[2].Capability); err != nil {
		t.Fatal(err)
	}
	exports, err := session.capturePracticalPackageExports(compiler, discovery, roots[2].Capability, practicalModuleLocation{capability: roots[0].Capability}, "example.com/policy")
	retainPracticalSyntaxEvidence(t, sha256Hex([]byte(t.Name()))+".observation-original-output.json", map[string]any{
		"module_output": discovery.output, "package_output": exports.output, "diagnostics": exports.diagnostics, "error": fmt.Sprint(err),
	})
	if err != nil {
		t.Fatal(err)
	}
	sealed, err := session.sealPracticalReplaySources()
	if err != nil {
		t.Fatal(err)
	}
	retainPracticalStandardSnapshot(t, sealed.snapshot)
	retainPracticalStandardSnapshot(t, exports.snapshot)
	loader, err := newPracticalSourceTypesWithStandardLibrary(sealed.snapshot, discovery, compiler, roots[2].Capability)
	if err != nil {
		t.Fatal(err)
	}
	if _, err := loader.typecheck("example.com/policy"); err != nil {
		t.Fatal(err)
	}
	return practicalObservationFixture{compiler, roots, discovery, exports, sealed, loader}
}

func TestPracticalObservationOriginalExportBindsACompletePackageWithoutUnsafe(t *testing.T) {
	fixture := preparePracticalObservationFixture(t)
	rows, err := decodePracticalPackageExports(fixture.exports.output)
	if err != nil || len(rows) != 1 || rows[0].ImportPath != "example.com/policy" {
		t.Fatal("original no-import Go package observation changed", err)
	}
	failure := fixture.exports.bindOriginalSourceTypes(fixture.loader, fixture.discovery, fixture.compiler, fixture.roots[2].Capability)
	retainPracticalSyntaxEvidence(t, "observation-no-unsafe-binding.json", map[string]any{"native_packages": len(rows), "typed_packages": len(fixture.loader.packages), "error": fmt.Sprint(failure), "semantic_source_counterexample_claimed": false})
	if failure != nil {
		t.Fatal(failure)
	}
}

func TestPracticalObservationRequiresActualWholeOriginalModuleOutput(t *testing.T) {
	image, _, original, roots := practicalImageModuleFixture(t)
	defer image.Close()
	view, err := image.openVerifiedView()
	if err != nil {
		t.Fatal(err)
	}
	defer view.Close()
	for _, control := range []string{"unobserved-copy", "compatible-subset", "transport-replacement", "compiler", "main"} {
		t.Run(control, func(t *testing.T) {
			inputs, err := view.restoreOriginalInputs()
			if err != nil {
				t.Fatal(err)
			}
			candidate := original
			switch control {
			case "unobserved-copy":
				candidate.observation = nil
			case "compatible-subset":
				rows, err := decodePracticalGoModules(original.output)
				if err != nil {
					t.Fatal(err)
				}
				for _, row := range rows {
					if row.Main {
						candidate.output, err = json.Marshal(row)
						if err != nil {
							t.Fatal(err)
						}
					}
				}
			case "transport-replacement":
				candidate.output = append(bytes.Clone(original.output), '\n')
			case "compiler":
				candidate.compilerSHA256 = "different"
			case "main":
				candidate.mainSourceID = "different"
			}
			_, failure := inputs.restoreOriginalDiscovery(practicalModuleLocation{capability: roots[0].Capability}, practicalModuleLocation{capability: roots[1].Capability}, candidate)
			retainPracticalSyntaxEvidence(t, "observation-module-"+control+".json", map[string]any{"original_output": original.output, "candidate_output": candidate.output, "candidate_compiler": candidate.compilerSHA256, "candidate_main": candidate.mainSourceID, "candidate_has_observation": candidate.observation != nil, "error": fmt.Sprint(failure)})
			if failure == nil {
				t.Fatal("caller metadata acquired actual original observation ownership")
			}
		})
	}
	inputs, err := view.restoreOriginalInputs()
	if err != nil {
		t.Fatal(err)
	}
	if _, err := inputs.restoreOriginalDiscovery(practicalModuleLocation{capability: roots[0].Capability}, practicalModuleLocation{capability: roots[1].Capability}, original); err != nil {
		t.Fatal("actual complete original observation rejected", err)
	}
}

func TestPracticalObservationDoesNotExecuteCallerSelectedCommands(t *testing.T) {
	selection := practicalTestResolverSelection(t)
	filesystem, err := newLoaderFilesystem()
	if err != nil {
		t.Fatal(err)
	}
	defer filesystem.Close()
	root, err := filepath.EvalSymlinks(t.TempDir())
	if err != nil {
		t.Fatal(err)
	}
	for _, test := range []struct {
		kind      practicalGoCommandKind
		requested string
	}{{0, ""}, {255, ""}, {practicalGoModules, "example.com/policy"}, {practicalGoPackages, "-exec=anything"}, {practicalGoPackages, "example.com/..."}} {
		result, err := runPracticalGoObservation(selection, filesystem, root, sha256Hex([]byte("original-main")), test.kind, test.requested)
		var executed *practicalGoDiscoveryFailure
		if err == nil || result != nil || errors.As(err, &executed) {
			t.Fatal("caller command/selector acquired an actual Go observation", test, err)
		}
	}
	if _, err := os.Stat(filepath.Join(root, "go.mod")); !os.IsNotExist(err) {
		t.Fatal("invalid-command controls changed their empty original directory", err)
	}
}
