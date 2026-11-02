//go:build go1.25 && (linux || darwin)

package main

import (
	"bytes"
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func TestPracticalOriginalPackagesRequireAnOwnedCompilerImage(t *testing.T) {
	for _, image := range []*practicalReplayImage{nil, {}} {
		if actual, err := image.originalPackages(practicalModuleLocation{}, practicalModuleLocation{}, practicalModuleDiscovery{}, practicalPackageExports{}, "sdk"); err == nil || actual != nil {
			t.Fatal("unowned package universe accepted", err)
		}
	}
}

func TestPracticalOriginalPackagesRetainWholePackagesAndRejectCrossedOwners(t *testing.T) {
	compiler := practicalPackageTestCompiler(t)
	roots, _ := practicalTestCaptureRoots(t)
	roots = append(roots, practicalSourceRoot{Capability: "original-go-sdk", Directory: compiler.selection.GoRoot})
	main, cache := practicalModuleLocation{capability: roots[0].Capability}, practicalModuleLocation{capability: roots[1].Capability}
	manifest := []byte("module example.com/policy\ngo 1.25.0\n")
	policy := []byte("package policy\r\nimport (\"example.com/policy/helper\"; \"unsafe\")\r\nfunc Root(x int64) int64 { return helper.Sub(x) }\r\nfunc Unused(c chan int, m map[int]int) int { return <-c + m[0] + int(unsafe.Sizeof(m)) }\r\nfunc init() { panic(\"application initialization must not execute\") }\r\n")
	helper := []byte("package helper\nfunc Sub(x int64) int64 { return x - 1 }\nfunc Unused(f func(int) int) int { return f(1) }\n")
	for name, raw := range map[string][]byte{"go.mod": manifest, "policy.go": policy, "helper/helper.go": helper} {
		writePracticalTestFile(t, filepath.Join(roots[0].Directory, name), raw, 0o600)
	}
	session := practicalTestSession(t, roots)
	discovery, err := session.discoverPracticalModules(main, cache, compiler.selection)
	if err != nil {
		t.Fatal(err)
	}
	if err := session.capturePracticalStandardLibrary(compiler, roots[2].Capability); err != nil {
		t.Fatal(err)
	}
	exports, err := session.capturePracticalPackageExports(compiler, discovery, roots[2].Capability, main, "example.com/policy")
	if err != nil {
		t.Fatal(err)
	}
	sealed, err := session.sealPracticalReplaySources()
	if err != nil {
		t.Fatal(err)
	}
	image, err := materializePracticalReplaySources(t.TempDir(), []practicalReplaySources{sealed, exports.replay}, &compiler, roots[2].Capability)
	if err != nil {
		t.Fatal(err)
	}
	defer image.Close()
	// The new consumer must have no access to the original application, module
	// cache or SDK. Retire the complete original byte buffers as well.
	for _, root := range roots {
		if err := removePracticalReplayImage(root.Directory); err != nil {
			t.Fatal(err)
		}
		if _, err := os.Lstat(root.Directory); !os.IsNotExist(err) {
			t.Fatal("original preparation path survived retirement", err)
		}
	}
	for _, snapshot := range []practicalSourceSnapshot{sealed.snapshot, exports.snapshot} {
		for _, buffer := range snapshot.buffers {
			clear(buffer.data)
		}
	}
	for _, control := range []string{"unobserved-modules", "unobserved-exports", "module-transport", "package-transport", "compiler-root", "package-directory", "stdlib-capability", "export-capability", "main-location", "cache-location", "selected-root", "export-snapshot", "package-membership", "source-membership", "package-imports"} {
		t.Run(control, func(t *testing.T) {
			original, candidate := discovery, exports
			mainLocation, cacheLocation, sdk := main, cache, roots[2].Capability
			switch control {
			case "unobserved-modules":
				original.observation = nil
			case "unobserved-exports":
				candidate.observation = nil
			case "module-transport":
				original.output = append(bytes.Clone(original.output), '\n')
			case "package-transport":
				candidate.output = append(bytes.Clone(candidate.output), '\n')
			case "compiler-root", "package-directory":
				observed := *candidate.observation
				candidate.observation = &observed
				if control == "compiler-root" {
					observed.compilerRoot += "-crossed"
				} else {
					observed.directory += "-crossed"
				}
			case "stdlib-capability":
				sdk = roots[0].Capability
			case "export-capability":
				candidate.buildCacheDirectory += "-crossed"
			case "main-location":
				mainLocation = cache
			case "cache-location":
				cacheLocation = main
			case "selected-root":
				candidate.requestedPackage = "example.com/policy/helper"
			case "export-snapshot":
				candidate.snapshot.contentSHA256 = strings.Repeat("a", 64)
			case "package-membership":
				candidate.packages = nil
			case "source-membership", "package-imports":
				candidate.packages = append([]practicalCapturedPackageExport(nil), candidate.packages...)
				for i := range candidate.packages {
					if candidate.packages[i].importPath == candidate.requestedPackage {
						if control == "source-membership" {
							candidate.packages[i].sources = nil
						} else {
							candidate.packages[i].imports = nil
						}
					}
				}
			}
			actual, failure := image.originalPackages(mainLocation, cacheLocation, original, candidate, sdk)
			if actual != nil {
				actual.Close()
			}
			retainPracticalSyntaxEvidence(t, "original-packages-"+control+".json", map[string]any{"error": fmt.Sprint(failure), "source_or_program_proof": false})
			if failure == nil || actual != nil || image.activeViews != 0 {
				t.Fatal("crossed original ownership accepted or leaked its image view", failure)
			}
		})
	}
	owned, err := image.originalPackages(main, cache, discovery, exports, roots[2].Capability)
	if err != nil {
		t.Fatal(err)
	}
	defer owned.Close()
	if image.activeViews != 0 {
		t.Fatal("completed package universe retained its source reader")
	}
	if err := image.Close(); err != nil {
		t.Fatal(err)
	}
	if owned.loader.packages["example.com/policy"].types.Scope().Lookup("Unused") == nil || owned.loader.packages["example.com/policy/helper"].types.Scope().Lookup("Unused") == nil {
		t.Fatal("unused original channel/map/callback bodies were stubbed or omitted")
	}
	inventory, err := owned.inventory()
	if err != nil || len(inventory) != 3 || inventory[0].importPath != "example.com/policy" || inventory[1].importPath != "example.com/policy/helper" || inventory[2].importPath != "unsafe" {
		t.Fatal("complete deterministic original package universe changed", err)
	}
	moduleOutput, packageOutput := bytes.Clone(owned.moduleOutput), bytes.Clone(owned.packageOutput)
	clear(discovery.output)
	clear(discovery.observation.output)
	clear(exports.output)
	clear(exports.observation.output)
	for i := range exports.packages {
		clear(exports.packages[i].sources)
		clear(exports.packages[i].imports)
		clear(exports.packages[i].importMap)
		if exports.packages[i].data != nil {
			exports.packages[i].data.ID = "crossed-after-construction"
		}
	}
	if !bytes.Equal(owned.moduleOutput, moduleOutput) || !bytes.Equal(owned.packageOutput, packageOutput) {
		t.Fatal("preparation caller changed owned original command bytes")
	}
	var observed []map[string]any
	for _, pkg := range inventory {
		var sources []map[string]any
		for _, object := range pkg.sources {
			buffer, err := owned.originalBytes(object)
			expected := policy
			if pkg.importPath == "example.com/policy/helper" {
				expected = helper
			} else if pkg.intrinsic {
				expected = bytes.Clone(buffer.data)
			}
			_, canonicalErr := buffer.canonicalObject()
			if err != nil || canonicalErr != nil || !bytes.Equal(buffer.data, expected) {
				t.Fatal("whole original source changed after parent retirement", err)
			}
			sources = append(sources, map[string]any{"object": object, "raw": bytes.Clone(buffer.data)})
			clear(buffer.data)
			again, err := owned.originalBytes(object)
			if err != nil || !bytes.Equal(again.data, expected) {
				t.Fatal("returned source bytes changed the package owner", err)
			}
		}
		var archive []byte
		if pkg.data != nil {
			buffer, err := owned.originalBytes(*pkg.data)
			if err != nil || validatePracticalExportArchive(buffer.data, pkg.buildID) != nil {
				t.Fatal("owned original compiler archive changed", err)
			}
			archive = buffer.sourceBytes()
		} else if !pkg.intrinsic || pkg.importPath != "unsafe" {
			t.Fatal("ordinary package lost original archive ownership")
		}
		observed = append(observed, map[string]any{"import_path": pkg.importPath, "name": pkg.name, "intrinsic": pkg.intrinsic, "sources": sources, "imports": pkg.imports, "import_map": pkg.importMap, "build_id": pkg.buildID, "export_object": pkg.data, "original_archive": archive})
	}
	for i := range inventory {
		clear(inventory[i].sources)
		clear(inventory[i].imports)
		clear(inventory[i].importMap)
		if inventory[i].data != nil {
			inventory[i].data.ID = "caller-mutated-view"
		}
	}
	again, err := owned.inventory()
	if err != nil || len(again[0].sources) != 1 || len(again[0].imports) != 2 || again[0].data.ID == "caller-mutated-view" {
		t.Fatal("returned inventory changed complete owned package metadata", err)
	}
	retainPracticalSyntaxEvidence(t, "original-packages-owned.json", map[string]any{"manifest": manifest, "policy": policy, "helper": helper, "modules": moduleOutput, "packages": packageOutput, "inventory": observed, "native_execution_receipt": false, "verification_closure_or_source_program_proof": false})
	bad := again[0].sources[0]
	bad.ID = strings.Repeat("f", 64)
	_, failed := owned.originalBytes(bad)
	if failed == nil {
		t.Fatal("crossed full source identity accepted")
	}
	if _, err := owned.inventory(); err != failed {
		t.Fatal("failed source transaction still exposed package facts", err)
	}
	owned.Close()
	owned.Close()
	if _, err := owned.inventory(); err == nil {
		t.Fatal("closed original package universe remained usable")
	}
	if _, err := owned.originalBytes(bad); err == nil || owned.packages != nil {
		t.Fatal("closed package universe retained source authority", err)
	}
}
