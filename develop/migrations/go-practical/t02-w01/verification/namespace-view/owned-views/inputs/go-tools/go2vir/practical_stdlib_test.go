//go:build go1.25 && (linux || darwin)

package main

import (
	"bytes"
	"encoding/json"
	"errors"
	"fmt"
	"os"
	"path"
	"path/filepath"
	"sort"
	"strings"
	"testing"
)

func retainPracticalStandardSnapshot(t *testing.T, snapshot practicalSourceSnapshot) {
	t.Helper()
	directory := os.Getenv("MPK_GO_PRACTICAL_SYNTAX_EVIDENCE")
	if directory == "" {
		return
	}
	root := filepath.Join(directory, "original-source-objects")
	if err := os.MkdirAll(root, 0o700); err != nil {
		t.Fatal(err)
	}
	for _, object := range snapshot.sourceObjects() {
		raw, exists := snapshot.sourceBytes(object.ID)
		if !exists {
			t.Fatal("captured original source bytes missing")
		}
		metadata, err := json.MarshalIndent(object, "", "  ")
		if err != nil {
			t.Fatal(err)
		}
		for extension, data := range map[string][]byte{"raw": raw, "json": append(metadata, '\n')} {
			name := filepath.Join(root, object.ID+"."+extension)
			if previous, err := os.ReadFile(name); err == nil {
				if !bytes.Equal(previous, data) {
					t.Fatal("a different original source reused a captured identity")
				}
				continue
			}
			if err := os.WriteFile(name, data, 0o600); err != nil {
				t.Fatal(err)
			}
		}
	}
}

func TestPracticalStandardLibraryOriginalSourcesTypecheckWithNoFilesystemFallback(t *testing.T) {
	compiler := practicalPackageTestCompiler(t)
	roots, _ := practicalTestCaptureRoots(t)
	roots = append(roots, practicalSourceRoot{Capability: "original-go-sdk", Directory: compiler.selection.GoRoot})
	policy := []byte("package policy\nimport (\"fmt\";\"strings\";\"net/http\")\nfunc Root(s string) bool {return strings.HasPrefix(s,\"pay:\")}\nfunc Unused(x any) string {return fmt.Sprint(x)}\nfunc UnusedHandler(w http.ResponseWriter,r *http.Request) {fmt.Fprint(w,r.URL)}\n")
	manifest := []byte("module example.com/app\ngo 1.25.0\n")
	writePracticalTestFile(t, filepath.Join(roots[0].Directory, "go.mod"), manifest, 0o600)
	writePracticalTestFile(t, filepath.Join(roots[0].Directory, "policy.go"), policy, 0o600)
	retainPracticalSyntaxEvidence(t, "stdlib-application-original.json", map[string]any{"roots": roots, "go_mod": manifest, "original_application_bytes": policy})
	session := practicalTestSession(t, roots)
	discovery, err := session.discoverPracticalModules(practicalModuleLocation{capability: roots[0].Capability}, practicalModuleLocation{capability: roots[1].Capability}, compiler.selection)
	if err != nil {
		t.Fatal(err)
	}
	if _, err := session.readSource(roots[0].Capability, "policy.go", "go_file"); err != nil {
		t.Fatal(err)
	}
	if err := session.capturePracticalStandardLibrary(compiler, roots[2].Capability); err != nil {
		t.Fatal(err)
	}
	// Preserve all original input buffers even if the native generation/capture
	// fails. Final seal below still independently revalidates the originals.
	beforeExport, err := copyPracticalSourceSnapshot(session)
	if err != nil {
		t.Fatal(err)
	}
	retainPracticalStandardSnapshot(t, beforeExport)
	exports, err := session.capturePracticalPackageExports(compiler, discovery, roots[2].Capability, practicalModuleLocation{capability: roots[0].Capability}, "example.com/app")
	if err != nil {
		var failure *practicalGoDiscoveryFailure
		if errors.As(err, &failure) {
			exports.output, exports.diagnostics = failure.originalOutput()
		}
	}
	retainPracticalSyntaxEvidence(t, "stdlib-native-export.json", map[string]any{"arguments": []string{"list", "-mod=readonly", "-deps", "-export", "-compiled", "-json", "example.com/app"}, "stdout": exports.output, "stderr": exports.diagnostics, "error": fmt.Sprint(err)})
	if err != nil {
		t.Fatal(err)
	}
	retainPracticalStandardSnapshot(t, exports.snapshot)
	snapshot, err := session.seal()
	if err != nil {
		t.Fatal(err)
	}
	retainPracticalStandardSnapshot(t, snapshot)
	rows, err := decodePracticalPackageExports(exports.output)
	if err != nil {
		t.Fatal(err)
	}
	native := map[string]practicalGoPackageExport{}
	for _, row := range rows {
		native[row.ImportPath] = row
	}
	for _, root := range roots[:2] {
		if err := os.RemoveAll(root.Directory); err != nil {
			t.Fatal(err)
		}
	}
	// Deny access to the original SDK after the snapshot boundary. The loader
	// below accepts only owned source objects and compiler identity metadata.
	compiler.selection.GoRoot = filepath.Join(roots[0].Directory, "removed-sdk")
	compiler.candidate.ToolchainGoRootPath = compiler.selection.GoRoot
	loader, err := newPracticalSourceTypesWithStandardLibrary(snapshot, discovery, compiler, roots[2].Capability)
	if err != nil {
		t.Fatal(err)
	}
	checked, err := loader.typecheck("example.com/app")
	if err != nil {
		t.Fatal(err)
	}
	if checked.types.Scope().Lookup("Root") == nil || checked.types.Scope().Lookup("Unused") == nil {
		t.Fatal("original application source was stubbed or partially typed")
	}
	if loader.packages["fmt"] == nil || loader.packages["strings"] == nil {
		t.Fatal("whole original standard dependencies were not source checked")
	}
	if loader.packages["net/http"] == nil {
		t.Fatal("unselected original HTTP handler lost its complete source-typing dependency")
	}
	var observed []map[string]any
	names := make([]string, 0, len(loader.packages))
	for name := range loader.packages {
		names = append(names, name)
	}
	sort.Strings(names)
	for _, name := range names {
		pkg := loader.packages[name]
		row, exists := native[name]
		if !exists {
			t.Fatalf("source loader introduced a non-native package %s", name)
		}
		var files []string
		for _, source := range pkg.sources.active {
			files = append(files, path.Base(source.Path))
		}
		if strings.Join(files, "\x00") != strings.Join(row.GoFiles, "\x00") || pkg.types.Name() != row.Name {
			t.Fatalf("original source files/package identity differ for %s", name)
		}
		if pkg.standardLibrary {
			for original, canonical := range row.ImportMap {
				resolved, err := loader.stdlib.resolve(original, name)
				if err != nil || resolved != canonical {
					t.Fatalf("original SDK vendor/import mapping differs for %s: %s -> %s, native %s, %v", name, original, resolved, canonical, err)
				}
			}
		}
		observed = append(observed, map[string]any{"import_path": name, "standard": pkg.standardLibrary, "active": pkg.sources.active, "excluded": pkg.sources.excluded, "file_versions": pkg.fileVersions, "imports": pkg.sources.imports})
	}
	if len(native) != len(loader.packages)+1 {
		t.Fatalf("native source closure differs (only unsafe intrinsic lacks an AST): native%d source%d", len(native), len(loader.packages))
	}
	if err := exports.bindOriginalSourceTypes(loader, discovery, compiler, roots[2].Capability); err != nil {
		t.Fatal(err)
	}
	var exportObservations []map[string]any
	for _, pkg := range exports.packages {
		exportObservations = append(exportObservations, map[string]any{"import_path": pkg.importPath, "name": pkg.name, "standard": pkg.standard, "intrinsic": pkg.intrinsic, "build_id": pkg.buildID, "sources": pkg.sources, "export": pkg.data, "imports": pkg.imports, "import_map": pkg.importMap})
	}
	retainPracticalSyntaxEvidence(t, "stdlib-original-exports.json", map[string]any{"compiler": exports.compilerSHA256, "distribution": exports.distributionSHA256, "main_source_id": exports.mainSourceID, "requested_package": exports.requestedPackage, "build_cache_original_directory": exports.buildCacheDirectory, "export_snapshot_digest": exports.snapshot.contentSHA256, "export_objects": exports.snapshot.sourceObjects(), "packages": exportObservations, "source_profile_or_application_proof_claimed": false})
	for _, control := range []string{"drop", "duplicate", "compiler", "module", "selected", "selected-dependency", "build-id", "build-id-rebound", "source", "imports", "archive", "archive-rehashed", "target-rehashed", "metadata"} {
		t.Run("export-"+control, func(t *testing.T) {
			changed := exports
			changed.packages = append([]practicalCapturedPackageExport(nil), exports.packages...)
			index := -1
			for i, pkg := range changed.packages {
				if pkg.importPath == "example.com/app" {
					index = i
				}
			}
			if index < 0 {
				t.Fatal("original compiled application missing")
			}
			pkg := &changed.packages[index]
			var changedArchive []byte
			switch control {
			case "drop":
				changed.packages = changed.packages[:len(changed.packages)-1]
			case "duplicate":
				changed.packages[index] = changed.packages[0]
			case "compiler":
				changed.compilerSHA256 = strings.Repeat("b", 64)
			case "module":
				changed.mainSourceID = strings.Repeat("c", 64)
			case "selected":
				changed.requestedPackage = "example.com/missing"
			case "selected-dependency":
				changed.requestedPackage = "fmt"
			case "build-id":
				pkg.buildID += "altered"
			case "build-id-rebound":
				original, _ := json.Marshal(pkg.buildID)
				pkg.buildID += "altered"
				altered, _ := json.Marshal(pkg.buildID)
				changed.output = bytes.Replace(exports.output, original, altered, 1)
			case "source":
				pkg.sources = nil
			case "imports":
				pkg.imports = []string{"unsafe"}
			case "metadata":
				changed.output = bytes.Replace(exports.output, []byte(`"BuildID": "`), []byte(`"BuildID": "altered`), 1)
			default:
				old := exports.snapshot.buffers[pkg.data.ID]
				data := old.sourceBytes()
				if control == "target-rehashed" {
					data = bytes.Replace(data, []byte("go object linux amd64"), []byte("go object other amd64"), 1)
				} else {
					data[0] = '?'
				}
				changedArchive = data
				buffer, err := newPracticalSourceBuffer(old.object.Capability, old.object.Path, old.object.Kind, data)
				if err != nil {
					t.Fatal(err)
				}
				changed.snapshot.buffers = make(map[string]practicalSourceBuffer, len(exports.snapshot.buffers))
				changed.snapshot.paths = make(map[string]string, len(exports.snapshot.paths))
				changed.snapshot.ids = append([]string(nil), exports.snapshot.ids...)
				for id, original := range exports.snapshot.buffers {
					changed.snapshot.buffers[id] = original
				}
				for key, id := range exports.snapshot.paths {
					changed.snapshot.paths[key] = id
				}
				delete(changed.snapshot.buffers, old.object.ID)
				changed.snapshot.buffers[buffer.object.ID] = buffer
				changed.snapshot.paths[practicalCaptureKey(buffer.object.Capability, buffer.object.Path)] = buffer.object.ID
				for i, id := range changed.snapshot.ids {
					if id == old.object.ID {
						changed.snapshot.ids[i] = buffer.object.ID
					}
				}
				sort.Strings(changed.snapshot.ids)
				changed.snapshot.contentSHA256 = practicalSnapshotDigest(changed.snapshot)
				if control != "archive" {
					object := buffer.object
					pkg.data = &object
				}
			}
			err := changed.bindOriginalSourceTypes(loader, discovery, compiler, roots[2].Capability)
			var packages []map[string]any
			for _, p := range changed.packages {
				packages = append(packages, map[string]any{"import_path": p.importPath, "name": p.name, "build_id": p.buildID, "standard": p.standard, "intrinsic": p.intrinsic, "sources": p.sources, "export": p.data, "imports": p.imports, "import_map": p.importMap})
			}
			retainPracticalSyntaxEvidence(t, "export-control-"+control+".json", map[string]any{"compiler": changed.compilerSHA256, "main_source_id": changed.mainSourceID, "requested_package": changed.requestedPackage, "native_output": changed.output, "packages": packages, "export_objects": changed.snapshot.sourceObjects(), "changed_archive": changedArchive, "error": fmt.Sprint(err), "semantic_source_counterexample_claimed": false})
			if err == nil {
				t.Fatal("altered export/module/source/compiler acquired original correspondence")
			}
		})
	}
	retainPracticalSyntaxEvidence(t, "stdlib-source-typing.json", map[string]any{"compiler_distribution": compiler.selection.Toolchain.DistributionSHA256, "go_module_discovery_raw": discovery.output, "snapshot_objects": snapshot.sourceObjects(), "typed_packages": observed, "complete_syntax_nodes": loader.syntax.count, "source_profile_or_application_proof_claimed": false})
	for _, control := range []string{"missing", "role", "content", "extra", "compiler"} {
		t.Run(control, func(t *testing.T) {
			changed := snapshot
			changed.buffers = make(map[string]practicalSourceBuffer, len(snapshot.buffers))
			for id, buffer := range snapshot.buffers {
				changed.buffers[id] = buffer
			}
			changed.paths = make(map[string]string, len(snapshot.paths))
			for key, id := range snapshot.paths {
				changed.paths[key] = id
			}
			changed.directories = make(map[string][]practicalDirectoryEntry, len(snapshot.directories))
			for key, entries := range snapshot.directories {
				changed.directories[key] = append([]practicalDirectoryEntry(nil), entries...)
			}
			changed.ids = append([]string(nil), snapshot.ids...)
			candidate := compiler
			key := practicalCaptureKey(roots[2].Capability, "src/strings/strings.go")
			oldID := changed.paths[key]
			old := changed.buffers[oldID]
			if control == "compiler" {
				candidate.selection.Toolchain.DistributionSHA256 = strings.Repeat("a", 64)
			} else if control == "extra" {
				buffer, err := newPracticalSourceBuffer(roots[2].Capability, "src/strings/extra.go", "go_file", []byte("package strings\n"))
				if err != nil {
					t.Fatal(err)
				}
				changed.buffers[buffer.object.ID] = buffer
				changed.paths[practicalCaptureKey(buffer.object.Capability, buffer.object.Path)] = buffer.object.ID
				changed.ids = append(changed.ids, buffer.object.ID)
				parent := practicalCaptureKey(roots[2].Capability, "src/strings")
				changed.directories[parent] = append(changed.directories[parent], practicalDirectoryEntry{Name: "extra.go"})
				sort.Slice(changed.directories[parent], func(i, j int) bool { return changed.directories[parent][i].Name < changed.directories[parent][j].Name })
			} else {
				delete(changed.buffers, oldID)
				delete(changed.paths, key)
				var ids []string
				for _, id := range changed.ids {
					if id != oldID {
						ids = append(ids, id)
					}
				}
				changed.ids = ids
				if control != "missing" {
					kind, data := old.object.Kind, old.sourceBytes()
					if control == "role" {
						kind = "excluded_source"
					} else {
						data = append(data, '\n')
					}
					buffer, err := newPracticalSourceBuffer(old.object.Capability, old.object.Path, kind, data)
					if err != nil {
						t.Fatal(err)
					}
					changed.buffers[buffer.object.ID] = buffer
					changed.paths[key] = buffer.object.ID
					changed.ids = append(changed.ids, buffer.object.ID)
				}
			}
			sort.Strings(changed.ids)
			changed.contentSHA256 = practicalSnapshotDigest(changed)
			_, err := newPracticalSourceTypesWithStandardLibrary(changed, discovery, candidate, roots[2].Capability)
			if err == nil {
				t.Fatal("rehashed altered standard-library inputs acquired original compiler authority")
			}
			if control == "missing" && !errors.Is(err, errPracticalNotCaptured) {
				t.Fatal("missing SDK input changed its failure category", err)
			}
		})
	}
}
