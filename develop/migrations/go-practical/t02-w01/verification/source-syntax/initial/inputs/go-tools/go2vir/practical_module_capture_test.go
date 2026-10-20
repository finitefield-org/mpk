//go:build go1.25 && (linux || darwin)

package main

import (
	"bytes"
	"context"
	"encoding/json"
	"io"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"strings"
	"testing"
	"time"
)

func writePracticalModuleTestFile(t *testing.T, name string, data []byte) {
	t.Helper()
	writePracticalTestFile(t, name, data, 0o600)
}

func TestPracticalModuleCaptureMatchesOriginalGoResolution(t *testing.T) {
	roots, _ := practicalTestCaptureRoots(t)
	mainRaw := []byte("// keep original paths and directives\nmodule example.com/app\n\ngo 1.25.0\nrequire example.com/dep v1.0.0\nreplace example.com/dep => ../../dep\n")
	depRaw := []byte("module example.com/dep\n\ngo 1.21\n")
	goRaw := []byte("package app\nfunc Selected() int64 { return 1 }\nfunc Unused(m map[string]int) *int { return nil }\n")
	writePracticalModuleTestFile(t, filepath.Join(roots[0].Directory, "nested", "go.mod"), mainRaw)
	writePracticalModuleTestFile(t, filepath.Join(roots[0].Directory, "nested", "go.sum"), nil)
	writePracticalModuleTestFile(t, filepath.Join(roots[0].Directory, "nested", "app.go"), goRaw)
	writePracticalModuleTestFile(t, filepath.Join(roots[1].Directory, "go.mod"), depRaw)
	session, err := newPracticalCaptureSession(roots)
	if err != nil {
		t.Fatal(err)
	}
	location := practicalModuleLocation{capability: roots[0].Capability, directory: "nested"}
	mainModule, err := session.captureModule(location, true)
	if err != nil {
		t.Fatal(err)
	}
	// Altering a returned parser tree cannot redirect an original replacement.
	mainModule.manifest.file.Replace[0].New.Path = "../../uncaptured"
	dep, err := session.captureLocalReplacement(mainModule, 0)
	if err != nil {
		t.Fatal(err)
	}
	if dep.location.capability != roots[1].Capability || dep.location.directory != "" ||
		dep.manifest.file.Module.Mod.Path != "example.com/dep" || dep.manifest.language != "go1.21" ||
		mainModule.sums == nil || len(mainModule.sums.entries) != 0 {
		t.Fatal("original local module, dependency language or empty checksum file lost")
	}
	source, err := session.readSource(location.capability, "nested/app.go", "go_file")
	if err != nil {
		t.Fatal(err)
	}
	if runtime.Version() != practicalCompilerRelease {
		t.Fatalf("differential requires exact %s, got %s", practicalCompilerRelease, runtime.Version())
	}
	filesystem, err := newLoaderFilesystem()
	if err != nil {
		t.Fatal(err)
	}
	defer filesystem.Close()
	selection := validatedLauncherSelection{GoRoot: runtime.GOROOT(), GoExecutable: filepath.Join(runtime.GOROOT(), "bin", "go")}
	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()
	command := exec.CommandContext(ctx, selection.GoExecutable, "list", "-mod=readonly", "-m", "-json", "all")
	command.Dir = filepath.Join(roots[0].Directory, "nested")
	command.Env = pinnedLoaderEnvironment(selection, filesystem)
	stderr := new(bytes.Buffer)
	command.Stderr = stderr
	raw, err := command.Output()
	if err != nil {
		t.Fatalf("original offline Go resolution: %v: %s", err, stderr.Bytes())
	}
	type resolved struct {
		Path, Version, GoVersion, Dir, GoMod string
		Replace                              *struct{ Path, Version, Dir, GoMod string }
	}
	decoder := json.NewDecoder(bytes.NewReader(raw))
	actual := make(map[string]resolved)
	for {
		var row resolved
		if err := decoder.Decode(&row); err == io.EOF {
			break
		} else if err != nil {
			t.Fatal(err)
		}
		if _, exists := actual[row.Path]; exists {
			t.Fatal("Go returned duplicate modules")
		}
		actual[row.Path] = row
	}
	mainFact, depFact := actual["example.com/app"], actual["example.com/dep"]
	if len(actual) != 2 || mainFact.GoVersion != "1.25.0" || mainFact.GoMod != filepath.Join(command.Dir, "go.mod") ||
		depFact.Version != "v1.0.0" || depFact.GoVersion != "1.21" || depFact.Replace == nil ||
		depFact.Replace.Path != "../../dep" || depFact.Replace.Dir != roots[1].Directory ||
		depFact.Replace.GoMod != filepath.Join(roots[1].Directory, "go.mod") {
		t.Fatalf("captured original metadata differs from Go resolution: %s", raw)
	}
	snapshot, err := session.seal()
	if err != nil {
		t.Fatal(err)
	}
	for _, expected := range []struct {
		id  string
		raw []byte
	}{
		{mainModule.manifest.source.object.ID, mainRaw}, {dep.manifest.source.object.ID, depRaw},
		{mainModule.sums.source.object.ID, nil}, {source.object.ID, goRaw},
	} {
		got, exists := snapshot.sourceBytes(expected.id)
		if !exists || !bytes.Equal(got, expected.raw) {
			t.Fatal("original manifest, checksum or full unused declaration was changed")
		}
	}
	t.Log("Go1.25.0 original offline graph: main go1.25.0; dependency v1.0.0, local ../../dep, go1.21; all4 original buffers sealed")
}

func TestPracticalModuleCaptureRejectsWorkspaceAndVendorInputs(t *testing.T) {
	for _, name := range []string{"go.work", "go.work.sum", "vendor"} {
		t.Run(name, func(t *testing.T) {
			roots, _ := practicalTestCaptureRoots(t)
			writePracticalModuleTestFile(t, filepath.Join(roots[0].Directory, "go.mod"), []byte("module example.com/app\ngo 1.25.0\n"))
			writePracticalModuleTestFile(t, filepath.Join(roots[0].Directory, name), nil)
			session, err := newPracticalCaptureSession(roots)
			if err != nil {
				t.Fatal(err)
			}
			if _, err := session.captureModule(practicalModuleLocation{capability: roots[0].Capability}, true); err == nil || session.failed == nil {
				t.Fatal("forbidden root input did not reject the transaction")
			}
		})
	}
}

func TestPracticalModuleCaptureRejectsUnboundAndLinkedReplacements(t *testing.T) {
	for _, target := range []string{"../outside", "./linked"} {
		t.Run(target, func(t *testing.T) {
			roots, _ := practicalTestCaptureRoots(t)
			raw := "module example.com/app\ngo 1.25.0\nrequire example.com/dep v1.0.0\nreplace example.com/dep => " + target + "\n"
			writePracticalModuleTestFile(t, filepath.Join(roots[0].Directory, "go.mod"), []byte(raw))
			writePracticalModuleTestFile(t, filepath.Join(roots[1].Directory, "go.mod"), []byte("module example.com/dep\ngo 1.22\n"))
			if err := os.Symlink(roots[1].Directory, filepath.Join(roots[0].Directory, "linked")); err != nil {
				t.Fatal(err)
			}
			session, err := newPracticalCaptureSession(roots)
			if err != nil {
				t.Fatal(err)
			}
			main, err := session.captureModule(practicalModuleLocation{capability: roots[0].Capability}, true)
			if err != nil {
				t.Fatal(err)
			}
			if _, err := session.captureLocalReplacement(main, 0); err == nil || session.failed == nil {
				t.Fatal("unbound or linked replacement accepted")
			}
			if _, err := session.seal(); err == nil {
				t.Fatal("ignored replacement rejection permitted sealing")
			}
		})
	}
}

func TestPracticalModuleCaptureChoosesExplicitMostSpecificRoot(t *testing.T) {
	roots, _ := practicalTestCaptureRoots(t)
	writePracticalModuleTestFile(t, filepath.Join(roots[0].Directory, "go.mod"), []byte("module example.com/app\ngo 1.25.0\nreplace example.com/dep => ../dep\n"))
	writePracticalModuleTestFile(t, filepath.Join(roots[1].Directory, "go.mod"), []byte("module example.com/dep\ngo 1.22\n"))
	// Duplicated broader roots cannot make the most-specific binding depend on
	// map iteration order. Equally specific duplicate roots must reject.
	parent := filepath.Dir(roots[0].Directory)
	for _, ambiguous := range []bool{false, true} {
		t.Run(map[bool]string{false: "more-specific", true: "ambiguous"}[ambiguous], func(t *testing.T) {
			bound := append([]practicalSourceRoot(nil), roots...)
			bound = append(bound, practicalSourceRoot{Capability: "broad-a", Directory: parent}, practicalSourceRoot{Capability: "broad-b", Directory: parent})
			if ambiguous {
				bound = append(bound, practicalSourceRoot{Capability: "duplicate-dep", Directory: roots[1].Directory})
			}
			session, err := newPracticalCaptureSession(bound)
			if err != nil {
				t.Fatal(err)
			}
			main, err := session.captureModule(practicalModuleLocation{capability: roots[0].Capability}, true)
			if err != nil {
				t.Fatal(err)
			}
			dep, err := session.captureLocalReplacement(main, 0)
			if ambiguous && err == nil || !ambiguous && (err != nil || dep.location.capability != roots[1].Capability) {
				t.Fatalf("incorrect capability choice: %+v, %v", dep.location, err)
			}
		})
	}
}

func TestPracticalModuleCaptureRequiresOriginalOwnerAndDirective(t *testing.T) {
	for _, invalid := range []string{"detached", "wrong-owner", "negative-index", "past-end", "versioned"} {
		t.Run(invalid, func(t *testing.T) {
			roots, _ := practicalTestCaptureRoots(t)
			raw := "module example.com/app\ngo 1.25.0\nreplace example.com/dep => ../dep\n"
			if invalid == "versioned" {
				raw = strings.Replace(raw, "../dep", "example.com/fork v1.0.0", 1)
			}
			writePracticalModuleTestFile(t, filepath.Join(roots[0].Directory, "go.mod"), []byte(raw))
			session, err := newPracticalCaptureSession(roots)
			if err != nil {
				t.Fatal(err)
			}
			location := practicalModuleLocation{capability: roots[0].Capability}
			var owner practicalCapturedModule
			if invalid == "detached" {
				owner.location = location
				owner.manifest, err = parsePracticalModuleManifest(practicalModuleTestBuffer(t, "go_mod", raw), true)
			} else {
				owner, err = session.captureModule(location, true)
			}
			if err != nil {
				t.Fatal(err)
			}
			index := 0
			switch invalid {
			case "wrong-owner":
				owner.location.capability = roots[1].Capability
			case "negative-index":
				index = -1
			case "past-end":
				index = 1
			}
			if _, err := session.captureLocalReplacement(owner, index); err == nil || session.failed == nil {
				t.Fatal("non-original owner/directive or unavailable versioned snapshot accepted")
			}
		})
	}
}
