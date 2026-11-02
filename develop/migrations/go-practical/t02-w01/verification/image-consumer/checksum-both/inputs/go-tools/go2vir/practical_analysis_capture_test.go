//go:build go1.25 && (linux || darwin)

package main

import (
	"bytes"
	"context"
	"debug/buildinfo"
	"encoding/json"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"strings"
	"testing"
	"time"
)

func practicalAnalysisTestRoots(t *testing.T, frontend string) []practicalSourceRoot {
	t.Helper()
	root, err := filepath.EvalSymlinks(".")
	if err != nil {
		t.Fatal(err)
	}
	if !filepath.IsAbs(root) {
		root, err = filepath.Abs(root)
		if err != nil {
			t.Fatal(err)
		}
	}
	if frontend == "" {
		frontend = root
	}
	return []practicalSourceRoot{{Capability: "analysis-frontend", Directory: frontend}, {Capability: "analysis-archives", Directory: filepath.Join(root, "testdata/practical-analysis-cache")}}
}

func retainPracticalAnalysisEvidence(t *testing.T, name string, record any) {
	t.Helper()
	if directory := os.Getenv("MPK_GO_PRACTICAL_ANALYSIS_EVIDENCE"); directory != "" {
		data, err := json.MarshalIndent(record, "", "  ")
		if err != nil {
			t.Fatal(err)
		}
		file, err := os.OpenFile(filepath.Join(directory, name), os.O_WRONLY|os.O_CREATE|os.O_EXCL, 0o600)
		if err != nil {
			t.Fatal(err)
		}
		_, writeErr := file.Write(append(data, '\n'))
		closeErr := file.Close()
		if writeErr != nil || closeErr != nil {
			t.Fatal("cannot retain original analysis build evidence")
		}
	}
}

func TestPracticalAnalysisCapturedInputsRebuildTheSameActualFrontend(t *testing.T) {
	roots := practicalAnalysisTestRoots(t, "")
	session := practicalTestSession(t, roots)
	inputs, err := session.capturePracticalAnalysis(practicalModuleLocation{capability: roots[0].Capability}, practicalModuleLocation{capability: roots[1].Capability})
	if err != nil {
		t.Fatal(err)
	}
	if len(inputs.modules) != 3 || len(inputs.differences) != 1 || len(inputs.sources) == 0 {
		t.Fatal("original analysis closure or explicit uncompiled TODO difference omitted")
	}
	snapshot, err := session.seal()
	if err != nil {
		t.Fatal(err)
	}
	objects := []practicalSourceObject{}
	for _, source := range inputs.buildSources() {
		objects = append(objects, source.sourceObject())
	}
	differences := []map[string]any{}
	for _, difference := range inputs.differences {
		differences = append(differences, map[string]any{"source_id": difference.sourceID, "original_sha256": difference.archiveSHA256, "original_bytes": difference.archiveBytes, "actual_sha256": difference.actualSHA256, "actual_bytes": difference.actualBytes})
	}
	retainPracticalAnalysisEvidence(t, "captured-inputs.json", map[string]any{"build_inputs": objects, "snapshot_source_ids": snapshot.ids, "auxiliary_differences": differences, "application_proof_claimed": false})
	compiler, err := validatePracticalCompilerImage(practicalTestCompleteCompiler(t))
	if err != nil {
		t.Fatal(err)
	}
	results := []map[string]any{}
	var first []byte
	for iteration := 0; iteration < 2; iteration++ {
		// Each build has an independent empty compiler cache and a distinct
		// physical source root populated only from sealed owned input bytes.
		filesystem, err := newLoaderFilesystem()
		if err != nil {
			t.Fatal(err)
		}
		t.Cleanup(func() {
			if err := filesystem.Close(); err != nil {
				t.Errorf("remove private analysis build filesystem: %v", err)
			}
		})
		buildRoot := filepath.Join(filesystem.root, "original-frontend")
		for _, source := range inputs.buildSources() {
			raw, exists := snapshot.sourceBytes(source.object.ID)
			if !exists || sha256Hex(raw) != source.object.RawSHA256 {
				t.Fatal("a build input is not owned by the original sealed snapshot")
			}
			writePracticalTestFile(t, filepath.Join(buildRoot, filepath.FromSlash(source.object.Path)), raw, 0o444)
		}
		sealTestReleaseRoot(t, buildRoot)
		outputRoot := filepath.Join(filesystem.root, "built-frontend")
		if err := os.Mkdir(outputRoot, 0o700); err != nil {
			t.Fatal(err)
		}
		binary := filepath.Join(outputRoot, "go2vir")
		environment := pinnedLoaderEnvironment(compiler.selection, filesystem)
		for index, value := range environment {
			if strings.HasPrefix(value, "GOOS=") {
				environment[index] = "GOOS=" + runtime.GOOS
			} else if strings.HasPrefix(value, "GOARCH=") {
				environment[index] = "GOARCH=" + runtime.GOARCH
			}
		}
		ctx, cancel := context.WithTimeout(context.Background(), 4*time.Minute)
		command := exec.CommandContext(ctx, compiler.selection.GoExecutable, "build", "-mod=vendor", "-trimpath", "-buildvcs=false", "-pgo=off", "-o", binary, ".")
		command.Dir, command.Env = buildRoot, environment
		stdout := &practicalBoundedOutput{maximum: maximumCandidateBytes, cancel: cancel}
		stderr := &practicalBoundedOutput{maximum: maximumCandidateBytes, cancel: cancel}
		command.Stdout, command.Stderr = stdout, stderr
		started := time.Now()
		buildErr := command.Run()
		cancel()
		retainPracticalAnalysisEvidence(t, fmt.Sprintf("build-%d-output.json", iteration), map[string]any{"command": command.Args, "environment": environment, "seconds": time.Since(started).Seconds(), "stdout": stdout.String(), "stderr": stderr.String(), "error": fmt.Sprint(buildErr)})
		if buildErr != nil {
			t.Fatalf("original captured frontend did not build offline: %v\n%s\n%s", buildErr, stdout.String(), stderr.String())
		}
		if err := os.Chmod(binary, 0o555); err != nil {
			t.Fatal(err)
		}
		sealTestReleaseRoot(t, outputRoot)
		files, err := inventoryFilesFromRoot(outputRoot, "")
		if err != nil {
			t.Fatal(err)
		}
		inventory := bundleInventory{Schema: bundleInventorySchema, Scope: inventoryScope{Kind: "frontend_bundle", BundleID: "frontend.go.practical.analysis.test.v1"}, Files: files}
		digest, err := hashTypedCanonicalJSON(bundleContentDomain, inventory)
		if err != nil {
			t.Fatal(err)
		}
		candidate := launcherSelection{Frontend: frontendIdentity{BundleID: inventory.Scope.BundleID, Name: "go2vir", Version: "private-analysis-test", BinarySHA256: files[0].SHA256}, FrontendRootPath: outputRoot, FrontendExecutablePath: "go2vir", FrontendInventory: inventory, FrontendBundleSHA256: digest}
		physicalOutput, err := filepath.EvalSymlinks(outputRoot)
		if err != nil {
			t.Fatal(err)
		}
		if physicalOutput != outputRoot {
			if err := validatePracticalAnalysisExecutable(candidate, compiler.host); err == nil {
				t.Fatal("a private temporary path alias crossed original root authorization")
			}
		}
		candidate.FrontendRootPath = physicalOutput
		if err := validatePracticalAnalysisExecutable(candidate, compiler.host); err != nil {
			t.Fatal(err)
		}
		info, err := buildinfo.ReadFile(binary)
		if err != nil {
			t.Fatal(err)
		}
		raw, err := os.ReadFile(binary)
		if err != nil {
			t.Fatal(err)
		}
		if iteration == 0 {
			first = raw
		} else if !bytes.Equal(first, raw) {
			t.Fatal("independent fresh builds from identical owned source inputs differ")
		}
		results = append(results, map[string]any{"binary_sha256": sha256Hex(raw), "bytes": len(raw), "build_info": info, "frontend_inventory": inventory, "bundle_sha256": digest, "compiler_distribution_sha256": compiler.candidate.Toolchain.DistributionSHA256})
		t.Logf("fresh offline captured frontend build%d: bytes=%d SHA256=%s exact pinned analysis metadata accepted", iteration, len(raw), sha256Hex(raw))
	}
	if err := compiler.revalidate(); err != nil {
		t.Fatal(err)
	}
	retainPracticalAnalysisEvidence(t, "reproduced-builds.json", map[string]any{"builds": results, "byte_exact_equal": true, "independent_empty_caches": true, "source_inputs": len(objects), "vendor_files": len(inputs.vendor), "frontend_files": len(inputs.sources), "registered_practical_context": false, "native_Linux_acceptance_claimed": false})
}

func TestPracticalAnalysisCaptureRejectsAlteredAndUnclosedInputs(t *testing.T) {
	original, err := os.Getwd()
	if err != nil {
		t.Fatal(err)
	}
	for _, mutation := range []string{"go-sum", "replacement", "vendor-version", "vendor-body", "missing-package", "unknown-asset", "root-assembly", "embedded-input", "linked-source"} {
		t.Run(mutation, func(t *testing.T) {
			root, err := filepath.EvalSymlinks(t.TempDir())
			if err != nil {
				t.Fatal(err)
			}
			for _, name := range []string{"go.mod", "go.sum"} {
				raw, err := os.ReadFile(name)
				if err != nil {
					t.Fatal(err)
				}
				writePracticalTestFile(t, filepath.Join(root, name), raw, 0o600)
			}
			if err := copyToolchainFixturePath(filepath.Join(original, "vendor"), filepath.Join(root, "vendor")); err != nil {
				t.Fatal(err)
			}
			writePracticalTestFile(t, filepath.Join(root, "main.go"), []byte("package main\nfunc main() {}\n"), 0o600)
			change := func(name string, transform func([]byte) []byte) {
				t.Helper()
				filename := filepath.Join(root, name)
				raw, err := os.ReadFile(filename)
				if err != nil {
					t.Fatal(err)
				}
				if err := os.Chmod(filename, 0o600); err != nil {
					t.Fatal(err)
				}
				writePracticalTestFile(t, filename, transform(raw), 0o600)
			}
			switch mutation {
			case "go-sum":
				change("go.sum", func(raw []byte) []byte {
					return bytes.Replace(raw, []byte(practicalAnalysisPins[0].sum), []byte("h1:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA="), 1)
				})
			case "replacement":
				change("go.mod", func(raw []byte) []byte { return append(raw, []byte("\nreplace golang.org/x/tools => ../caller\n")...) })
			case "vendor-version":
				change("vendor/modules.txt", func(raw []byte) []byte { return bytes.Replace(raw, []byte("v0.36.0"), []byte("v0.36.1"), 1) })
			case "vendor-body":
				change("vendor/golang.org/x/tools/go/packages/packages.go", func(raw []byte) []byte { return append(raw, '\n') })
			case "missing-package":
				if err := os.Remove(filepath.Join(root, "vendor/golang.org/x/sync/errgroup/errgroup.go")); err != nil {
					t.Fatal(err)
				}
			case "unknown-asset":
				writePracticalTestFile(t, filepath.Join(root, "vendor/golang.org/x/tools/go/packages/caller.data"), []byte("caller"), 0o600)
			case "root-assembly":
				writePracticalTestFile(t, filepath.Join(root, "caller.s"), nil, 0o600)
			case "embedded-input":
				writePracticalTestFile(t, filepath.Join(root, "main.go"), []byte("package main\nimport _ \"embed\"\n//go:embed caller.data\nvar Data []byte\nfunc main() {}\n"), 0o600)
			case "linked-source":
				filename := filepath.Join(root, "vendor/golang.org/x/tools/go/packages/packages.go")
				if err := os.Remove(filename); err != nil {
					t.Fatal(err)
				}
				if err := os.Symlink(filepath.Join(original, "vendor/golang.org/x/tools/go/packages/packages.go"), filename); err != nil {
					t.Fatal(err)
				}
			}
			roots := practicalAnalysisTestRoots(t, root)
			session := practicalTestSession(t, roots)
			if _, err := session.capturePracticalAnalysis(practicalModuleLocation{capability: roots[0].Capability}, practicalModuleLocation{capability: roots[1].Capability}); err == nil || session.failed == nil {
				t.Fatal("altered or incomplete analysis input capture accepted")
			}
			if _, err := session.seal(); err == nil {
				t.Fatal("ignored analysis capture error sealed an authoritative snapshot")
			}
		})
	}
}
