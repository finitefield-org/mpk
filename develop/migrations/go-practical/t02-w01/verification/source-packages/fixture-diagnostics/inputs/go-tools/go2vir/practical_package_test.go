//go:build go1.25 && (linux || darwin)

package main

import (
	"bytes"
	"encoding/json"
	"errors"
	"go/types"
	"os"
	"os/exec"
	"path/filepath"
	"reflect"
	"sort"
	"strings"
	"testing"
)

func practicalPackageTestCompiler(t *testing.T) practicalCompilerImage {
	t.Helper()
	compiler, err := validatePracticalCompilerImage(practicalTestCompleteCompiler(t))
	if err != nil {
		t.Fatal(err)
	}
	return compiler
}

func practicalPackageTestCapture(t *testing.T, roots []practicalSourceRoot, compiler practicalCompilerImage, sources map[int]map[string][]byte, excluded map[string]bool) (practicalSourceSnapshot, practicalModuleDiscovery) {
	t.Helper()
	for index, files := range sources {
		for name, raw := range files {
			writePracticalTestFile(t, filepath.Join(roots[index].Directory, name), raw, 0o600)
		}
	}
	name := sha256Hex([]byte(t.Name()))
	retainPracticalSyntaxEvidence(t, name+".original-inputs.json", map[string]any{"test": t.Name(), "roots": roots, "original_sources": sources, "excluded_source_roles": excluded})
	session := practicalTestSession(t, roots)
	discovery, err := session.discoverPracticalModules(practicalModuleLocation{capability: roots[0].Capability}, practicalModuleLocation{capability: roots[1].Capability}, compiler.selection)
	if err != nil {
		var failure *practicalGoDiscoveryFailure
		if errors.As(err, &failure) {
			stdout, stderr := failure.originalOutput()
			retainPracticalSyntaxEvidence(t, name+".original-Go-failure.json", map[string]any{"test": t.Name(), "stdout": stdout, "stderr": stderr, "error": err.Error()})
		}
		t.Fatal(err)
	}
	for index, files := range sources {
		for name := range files {
			if !strings.HasSuffix(name, ".go") {
				continue
			}
			kind := "go_file"
			if excluded[roots[index].Capability+"/"+name] {
				kind = "excluded_source"
			}
			if _, err := session.readSource(roots[index].Capability, name, kind); err != nil {
				t.Fatal(err)
			}
		}
	}
	snapshot, err := session.seal()
	if err != nil {
		t.Fatal(err)
	}
	return snapshot, discovery
}

func TestPracticalPackageBuildUsesSealedTargetSelection(t *testing.T) {
	compiler := practicalPackageTestCompiler(t)
	roots, _ := practicalTestCaptureRoots(t)
	files := map[string][]byte{
		"go.mod":                []byte("module example.com/policy\ngo 1.25.0\n"),
		"policy.go":             []byte("package policy\nfunc Root(x int) int { return x }\n"),
		"target_linux_amd64.go": []byte("package policy\nconst Target = 1\n"),
		"other_windows.go":      []byte("package policy\nfunc NotSelected() { invalid ignored body }\n"),
		"tag.go":                []byte("//go:build unix && amd64.v1 && !amd64.v2 && goexperiment.dwarf5 && go1.25 && !go1.26 && !cgo\n\npackage policy\nconst Tags = 1\n"),
		"cgo.go":                []byte("package policy\nimport \"C\"\n"),
		"policy_test.go":        []byte("package policy\nfunc TestUnused() { missing package type checking }\n"),
	}
	excluded := map[string]bool{}
	for _, name := range []string{"other_windows.go", "cgo.go", "policy_test.go"} {
		excluded[roots[0].Capability+"/"+name] = true
	}
	snapshot, _ := practicalPackageTestCapture(t, roots, compiler, map[int]map[string][]byte{0: files}, excluded)
	filesystem, err := newLoaderFilesystem()
	if err != nil {
		t.Fatal(err)
	}
	defer filesystem.Close()
	command := exec.Command(compiler.selection.GoExecutable, "list", "-mod=readonly", "-json", ".")
	command.Dir = roots[0].Directory
	command.Env = pinnedLoaderEnvironment(compiler.selection, filesystem)
	raw, err := command.Output()
	if err != nil {
		t.Fatal(err)
	}
	var native struct {
		Name                                                        string
		GoFiles, IgnoredGoFiles, TestGoFiles, XTestGoFiles, Imports []string
	}
	if err := json.Unmarshal(raw, &native); err != nil {
		t.Fatal(err)
	}
	for _, root := range roots {
		if err := os.RemoveAll(root.Directory); err != nil {
			t.Fatal(err)
		}
	}
	build, err := snapshot.replayPackageBuild(practicalModuleLocation{capability: roots[0].Capability})
	if err != nil {
		t.Fatal(err)
	}
	var active, inactive []string
	for _, source := range build.active {
		active = append(active, source.Path)
	}
	for _, source := range build.excluded {
		inactive = append(inactive, source.Path)
	}
	expected := append(append(append([]string{}, native.IgnoredGoFiles...), native.TestGoFiles...), native.XTestGoFiles...)
	sort.Strings(expected)
	if build.name != native.Name || !reflect.DeepEqual(active, native.GoFiles) || !reflect.DeepEqual(inactive, expected) || !reflect.DeepEqual(build.imports, native.Imports) {
		t.Fatalf("closed target selection differs from actual Go: %v %v %+v", active, inactive, native)
	}
	retainPracticalSyntaxEvidence(t, "package-build.json", map[string]any{"native_raw": raw, "active": build.active, "excluded": build.excluded, "all_tags": build.allTags, "original_files": files})
}

func TestPracticalSourceTypesCheckOriginalBodiesAndNominalIdentity(t *testing.T) {
	compiler := practicalPackageTestCompiler(t)
	for _, absolute := range []bool{false, true} {
		t.Run(map[bool]string{false: "relative", true: "absolute"}[absolute], func(t *testing.T) {
			roots, _ := practicalTestCaptureRoots(t)
			replacement := "../dep"
			if absolute {
				replacement = roots[1].Directory
			}
			sources := map[int]map[string][]byte{
				0: {"go.mod": []byte("module example.com/app\ngo 1.22\nrequire example.com/dep v1.0.0\nreplace example.com/dep => " + replacement + "\n"), "policy.go": []byte("package policy\nimport \"example.com/dep\"\ntype Amount int64\ntype Alias = dep.Amount\nfunc Root(x Amount) dep.Amount {return dep.Identity(dep.Amount(x))}\nfunc Unused(m map[string]int) int {return len(m)}\n")},
				1: {"go.mod": []byte("module example.com/dep\ngo 1.23.0\n"), "dep.go": []byte("package dep\ntype Amount int64\nfunc Identity(x Amount) Amount {return x}\nfunc Unused(m map[int]int) int {return len(m)}\nfunc init() {panic(\"must never run\")}\n")},
			}
			snapshot, discovery := practicalPackageTestCapture(t, roots, compiler, sources, nil)
			for _, root := range roots {
				if err := os.RemoveAll(root.Directory); err != nil {
					t.Fatal(err)
				}
			}
			loader, err := newPracticalSourceTypes(snapshot, discovery, compiler)
			if err != nil {
				t.Fatal(err)
			}
			loaded, err := loader.typecheck("example.com/app")
			if err != nil {
				t.Fatal(err)
			}
			dep := loader.packages["example.com/dep"]
			if dep == nil || len(loaded.files) != 1 || len(dep.files) != 1 {
				t.Fatal("original typed dependency was omitted")
			}
			amount := loaded.types.Scope().Lookup("Amount").Type()
			foreign := dep.types.Scope().Lookup("Amount").Type()
			alias := loaded.types.Scope().Lookup("Alias").Type()
			if types.Identical(amount, foreign) || !types.Identical(types.Unalias(alias), foreign) {
				t.Fatal("original Go nominal or alias identity was collapsed")
			}
			for _, file := range loaded.files {
				if loaded.info.FileVersions[file] != "go1.22" {
					t.Fatal("original main file language changed")
				}
			}
			for _, file := range dep.files {
				if dep.info.FileVersions[file] != "go1.23.0" {
					t.Fatal("full dependency Go requirement was lost")
				}
			}
			if !bytes.Contains(sources[1]["dep.go"], []byte("panic")) || len(dep.files[0].Decls) != 4 {
				t.Fatal("original unused body/initializer was stubbed or dropped")
			}
			// Every compiler object that denotes an actual source identifier must
			// keep its complete original physical token, across both package trees.
			for _, pkg := range []*practicalTypedPackage{loaded, dep} {
				for id, object := range pkg.info.Defs {
					if object != nil {
						if actual, err := loader.syntax.identifier(id); err != nil || actual != object.Name() {
							t.Fatal("typed declaration lost its original source token", err)
						}
					}
				}
			}
		})
	}
}

func TestPracticalSourceTypesRetainAll81OriginalGoTypeResults(t *testing.T) {
	compiler := practicalPackageTestCompiler(t)
	raw, err := os.ReadFile("../../develop/migrations/go-practical/t01-w04/source-admission/source-vectors.json")
	if err != nil || sha256Hex(raw) != "81e7c48cb20d4c51c40b95edd00f54942420d3f3a6225e40159fd1c9d884f0cc" {
		t.Fatal("original catalog changed", err)
	}
	var catalog struct {
		Cases []struct {
			ID, Source, Language string
			Expected             string `json:"expected_status"`
		}
	}
	if err := json.Unmarshal(raw, &catalog); err != nil || len(catalog.Cases) != 81 {
		t.Fatal("original catalog incomplete", err)
	}
	var observations []map[string]any
	typed, typeErrors, sourceErrors := 0, 0, 0
	for _, c := range catalog.Cases {
		t.Run(c.ID, func(t *testing.T) {
			roots, _ := practicalTestCaptureRoots(t)
			sources := map[int]map[string][]byte{0: {"go.mod": []byte("module fixture/policy\ngo " + strings.TrimPrefix(c.Language, "go") + "\n"), "policy.go": []byte(c.Source)}}
			snapshot, discovery := practicalPackageTestCapture(t, roots, compiler, sources, nil)
			loader, err := newPracticalSourceTypes(snapshot, discovery, compiler)
			if err != nil {
				t.Fatal(err)
			}
			loaded, err := loader.typecheck("fixture/policy")
			original, readErr := snapshot.readSource(roots[0].Capability, "policy.go", "go_file")
			if readErr != nil {
				t.Fatal(readErr)
			}
			row := map[string]any{"id": c.ID, "source": original.object, "raw": original.data, "module_raw": sources[0]["go.mod"], "profile_admission_or_source_proof_claimed": false}
			if c.Expected == "source_error" || c.Expected == "type_error" {
				if err == nil {
					t.Fatal("original Go error in the complete package was removed")
				}
				row["error"] = err.Error()
				if c.Expected == "source_error" {
					sourceErrors++
				} else {
					typeErrors++
				}
			} else {
				if err != nil {
					t.Fatal(err)
				}
				if loaded.types.Path() != "fixture/policy" || len(loaded.files) != 1 {
					t.Fatal("original package identity or complete source was lost")
				}
				for _, v := range loaded.fileVersions {
					if v != c.Language {
						t.Fatalf("original effective file language %s != %s", v, c.Language)
					}
				}
				definitions := 0
				for id, obj := range loaded.info.Defs {
					if obj != nil {
						if _, err := loader.syntax.identifier(id); err != nil {
							t.Fatal(err)
						}
						definitions++
					}
				}
				row["file_versions"] = loaded.fileVersions
				row["definitions"] = definitions
				row["complete_syntax_nodes"] = loader.syntax.count
				typed++
			}
			observations = append(observations, row)
		})
	}
	if typed != 77 || typeErrors != 3 || sourceErrors != 1 {
		t.Fatalf("complete native Go type matrix changed: %d/%d/%d", typed, typeErrors, sourceErrors)
	}
	retainPracticalSyntaxEvidence(t, "all81-original-types.json", map[string]any{"typed": typed, "type_errors": typeErrors, "source_errors": sourceErrors, "cases": observations})
}

// Ensure the same complete source tree is used even for a function whose
// selected body would otherwise be trivial. No original declaration is erased.
func TestPracticalSourceTypesRejectErrorInUnusedDependencyBody(t *testing.T) {
	compiler := practicalPackageTestCompiler(t)
	roots, _ := practicalTestCaptureRoots(t)
	sources := map[int]map[string][]byte{0: {"go.mod": []byte("module example.com/app\ngo 1.25.0\nrequire example.com/dep v1.0.0\nreplace example.com/dep => ../dep\n"), "policy.go": []byte("package policy\nimport \"example.com/dep\"\nfunc Root(x int64) int64 {return dep.Identity(x)}\n")}, 1: {"go.mod": []byte("module example.com/dep\ngo 1.25.0\n"), "dep.go": []byte("package dep\nfunc Identity(x int64) int64 {return x}\nfunc Unused() int {return \"original type error\"}\n")}}
	snapshot, discovery := practicalPackageTestCapture(t, roots, compiler, sources, nil)
	loader, err := newPracticalSourceTypes(snapshot, discovery, compiler)
	if err != nil {
		t.Fatal(err)
	}
	if _, err := loader.typecheck("example.com/app"); err == nil || !strings.Contains(err.Error(), "original Go package type check") {
		t.Fatal("unselected original dependency type error disappeared", err)
	}
	if _, err := loader.typecheck("example.com/dep"); err == nil {
		t.Fatal("ignored original type error did not fail the whole transaction")
	}
}
