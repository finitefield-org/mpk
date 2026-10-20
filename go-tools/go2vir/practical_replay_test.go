//go:build go1.25 && (linux || darwin)

package main

import (
	"bytes"
	"encoding/json"
	"errors"
	"fmt"
	"io/fs"
	"os"
	"path/filepath"
	"sort"
	"strings"
	"testing"

	"golang.org/x/mod/module"
)

func TestPracticalReplayImageOwnsOriginalBytesPathsAndNamespaces(t *testing.T) {
	roots, _ := practicalTestCaptureRoots(t)
	main := []byte("module example.com/app\ngo 1.25.0\nreplace example.com/dep => " + roots[1].Directory + "\n")
	source := []byte("package policy\nfunc Root(x int64) int64 {return x}\nfunc Unused(m map[int]int) int {return len(m)}\n")
	writePracticalTestFile(t, filepath.Join(roots[0].Directory, "go.mod"), main, 0o600)
	writePracticalTestFile(t, filepath.Join(roots[0].Directory, "policy.go"), source, 0o600)
	writePracticalTestFile(t, filepath.Join(roots[0].Directory, "opaque.bin"), []byte{0, 1, 2, 3}, 0o600)
	if err := os.Symlink("unread-original-target", filepath.Join(roots[0].Directory, "unrelated-link")); err != nil {
		t.Fatal(err)
	}
	session := practicalTestSession(t, roots)
	for _, input := range []struct{ path, kind string }{{"go.mod", "go_mod"}, {"policy.go", "go_file"}} {
		if _, err := session.readSource(roots[0].Capability, input.path, input.kind); err != nil {
			t.Fatal(err)
		}
	}
	sealed, err := session.sealPracticalReplaySources()
	if err != nil {
		t.Fatal(err)
	}
	image, err := materializePracticalReplaySources(t.TempDir(), []practicalReplaySources{sealed}, nil, "")
	if err != nil {
		t.Fatal(err)
	}
	defer image.Close()
	for _, root := range roots {
		if err := os.RemoveAll(root.Directory); err != nil {
			t.Fatal(err)
		}
	}
	for name, expected := range map[string][]byte{"go.mod": main, "policy.go": source} {
		kind := "go_file"
		if name == "go.mod" {
			kind = "go_mod"
		}
		actual, err := image.readCaptured(roots[0].Capability, name, kind)
		if err != nil || !bytes.Equal(actual.sourceBytes(), expected) {
			t.Fatal("original complete bytes or absolute replacement changed", name, err)
		}
	}
	if _, err := image.readCaptured(roots[0].Capability, "opaque.bin", "auxiliary"); !errors.Is(err, errPracticalNotCaptured) {
		t.Fatal("present unread source was relabeled as absent", err)
	}
	if _, err := image.readCaptured(roots[0].Capability, "absent.bin", "auxiliary"); !errors.Is(err, fs.ErrNotExist) {
		t.Fatal("observed absence changed category", err)
	}
	if _, err := image.readCaptured(roots[0].Capability, "unrelated-link", "auxiliary"); err == nil {
		t.Fatal("unread symbolic link acquired fabricated contents")
	}
	before := image.mounts[0]
	sealed.origins[0].Directory = "/changed-after-copy"
	for key, entries := range sealed.snapshot.directories {
		if len(entries) > 0 {
			entries[0].Name = "changed-after-copy"
			sealed.snapshot.directories[key] = entries
		}
	}
	if image.mounts[0] != before {
		t.Fatal("caller mutated owned replay origin")
	}
	if _, err := image.readCaptured(roots[0].Capability, "policy.go", "go_file"); err != nil {
		t.Fatal("caller mutated owned original replay namespace", err)
	}
	retainPracticalSyntaxEvidence(t, "replay-owned-bytes.json", map[string]any{"original_manifest": main, "original_application_bytes": source, "image_digest": image.contentSHA256, "materialized_files": len(image.files), "unread_original_name_is_absence": false, "registered_context_or_proof": false})
}

func TestPracticalReplayImageRejectsCrossedOriginsAndForgedSnapshots(t *testing.T) {
	roots, _ := practicalTestCaptureRoots(t)
	writePracticalTestFile(t, filepath.Join(roots[0].Directory, "policy.go"), []byte("package policy\n"), 0o600)
	session := practicalTestSession(t, roots)
	if _, err := session.readSource(roots[0].Capability, "policy.go", "go_file"); err != nil {
		t.Fatal(err)
	}
	sealed, err := session.sealPracticalReplaySources()
	if err != nil {
		t.Fatal(err)
	}
	for _, control := range []string{"missing-owner", "origin", "rehash", "crossed-path", "duplicate-input"} {
		t.Run(control, func(t *testing.T) {
			changed := sealed
			changed.origins = append([]practicalSourceRoot(nil), sealed.origins...)
			inputs := []practicalReplaySources{changed}
			switch control {
			case "missing-owner":
				inputs[0].sealed = nil
			case "origin":
				inputs[0].origins[0].Directory += "-other"
			case "rehash":
				inputs[0].snapshot.ids = nil
				inputs[0].snapshot.contentSHA256 = practicalSnapshotDigest(inputs[0].snapshot)
			case "crossed-path":
				inputs[0].snapshot.paths = map[string]string{}
			case "duplicate-input":
				inputs = append(inputs, changed)
			}
			if _, err := materializePracticalReplaySources(t.TempDir(), inputs, nil, ""); err == nil {
				t.Fatal("forged source/origin acquired original replay ownership")
			}
		})
	}
}

func TestPracticalReplayImageSharesFrozenBudgetsAcrossSnapshots(t *testing.T) {
	var inputs []practicalReplaySources
	for index := 0; index < 2; index++ {
		capability := fmt.Sprintf("budget/%d", index)
		snapshot := practicalSourceSnapshot{roots: []string{capability}, buffers: map[string]practicalSourceBuffer{}, paths: map[string]string{}, directories: map[string][]practicalDirectoryEntry{}}
		for file := 0; file < 9; file++ {
			name := fmt.Sprintf("input-%02d.bin", file)
			buffer, err := newPracticalSourceBuffer(capability, name, "auxiliary", make([]byte, maximumCandidateBytes))
			if err != nil {
				t.Fatal(err)
			}
			snapshot.ids = append(snapshot.ids, buffer.object.ID)
			snapshot.buffers[buffer.object.ID] = buffer
			snapshot.paths[practicalCaptureKey(capability, name)] = buffer.object.ID
			key := practicalCaptureKey(capability, "")
			snapshot.directories[key] = append(snapshot.directories[key], practicalDirectoryEntry{Name: name})
		}
		sort.Strings(snapshot.ids)
		snapshot.contentSHA256 = practicalSnapshotDigest(snapshot)
		origins := []practicalSourceRoot{{Capability: capability, Directory: filepath.Join(t.TempDir(), "original")}}
		input := practicalReplaySources{snapshot: snapshot, origins: origins, sealed: &practicalReplayBinding{snapshot.contentSHA256, append([]practicalSourceRoot(nil), origins...)}}
		if err := validatePracticalReplaySources([]practicalReplaySources{input}); err != nil {
			t.Fatal("individual original byte snapshot should fit its frozen bound", err)
		}
		inputs = append(inputs, input)
	}
	if err := validatePracticalReplaySources(inputs); err == nil {
		t.Fatal("combining snapshots reset the frozen aggregate byte limit")
	}
}

// The external local Linux harness mounts these exact owned bytes at the
// original paths, read-only and without network/host source mounts. The Go
// application is compiled, never executed; init bodies remain original.
func TestPracticalReplayPrepareWholeOriginalModuleAndCompilerImage(t *testing.T) {
	output := os.Getenv("MPK_GO_PRACTICAL_REPLAY_OUTPUT")
	if output == "" {
		t.Skip("requires explicit disposable kernel-replay output root")
	}
	compiler := practicalPackageTestCompiler(t)
	roots, base := practicalTestCaptureRoots(t)
	roots = append(roots,
		practicalSourceRoot{Capability: "original-go-sdk", Directory: compiler.selection.GoRoot},
		practicalSourceRoot{Capability: "local-relative", Directory: filepath.Join(base, "relative")},
		practicalSourceRoot{Capability: "local-absolute", Directory: filepath.Join(base, "absolute")})
	for _, root := range roots[3:] {
		if err := os.MkdirAll(root.Directory, 0o700); err != nil {
			t.Fatal(err)
		}
	}
	identity := module.Version{Path: "example.com/dep", Version: "v1.1.0"}
	sums := writePracticalResolutionDependency(t, roots[1].Directory, identity, []byte("module example.com/dep\ngo 1.22\n"))
	manifest := []byte("module example.com/app\ngo 1.25.0\nrequire (\nexample.com/dep v1.0.0\nexample.com/relative v1.0.0\nexample.com/absolute v1.0.0\n)\nreplace example.com/dep v1.0.0 => example.com/dep v1.1.0\nreplace example.com/relative => ../relative\nreplace example.com/absolute => " + roots[4].Directory + "\n")
	application := []byte("package policy\nimport(\"fmt\";\"strings\";\"net/http\";dep \"example.com/dep\";rel \"example.com/relative\";abs \"example.com/absolute\")\nfunc Root(x int64) int64 {return abs.Identity(rel.Identity(dep.Identity(x)))}\nfunc UnusedPrefix(s string) bool{return strings.HasPrefix(s,\"pay:\")}\nfunc UnusedHandler(w http.ResponseWriter,r *http.Request){fmt.Fprint(w,r.URL)}\n")
	files := map[int]map[string][]byte{
		0: {"go.mod": manifest, "go.sum": []byte(sums), "policy.go": application},
		3: {"go.mod": []byte("module example.com/relative\ngo 1.23.0\n"), "dep.go": []byte("package relative\nfunc Identity(x int64) int64{return x}\nfunc init(){panic(\"not executed\")}\n")},
		4: {"go.mod": []byte("module example.com/absolute\ngo 1.22\n"), "dep.go": []byte("package absolute\nfunc Identity(x int64) int64{return x}\n")},
	}
	for index, members := range files {
		for name, raw := range members {
			writePracticalTestFile(t, filepath.Join(roots[index].Directory, name), raw, 0o600)
		}
	}
	retainPracticalSyntaxEvidence(t, "replay-original-application.json", map[string]any{"roots": roots, "original_files": files, "versioned_identity": identity})
	session := practicalTestSession(t, roots)
	discovery, err := session.discoverPracticalModules(practicalModuleLocation{capability: roots[0].Capability}, practicalModuleLocation{capability: roots[1].Capability}, compiler.selection)
	if err != nil {
		var failure *practicalGoDiscoveryFailure
		if errors.As(err, &failure) {
			stdout, stderr := failure.originalOutput()
			retainPracticalSyntaxEvidence(t, "replay-native-original-module-failure.json", map[string]any{"stdout": stdout, "stderr": stderr, "error": err.Error()})
		}
		t.Fatal(err)
	}
	for index, members := range files {
		for name := range members {
			if strings.HasSuffix(name, ".go") {
				if _, err := session.readSource(roots[index].Capability, name, "go_file"); err != nil {
					t.Fatal(err)
				}
			}
		}
	}
	for _, name := range []string{"example.com/dep@v1.1.0/go.mod", "example.com/dep@v1.1.0/dep.go"} {
		kind := "go_file"
		if strings.HasSuffix(name, "go.mod") {
			kind = "go_mod"
		}
		if _, err := session.readSource(roots[1].Capability, name, kind); err != nil {
			t.Fatal(err)
		}
	}
	if err := session.capturePracticalStandardLibrary(compiler, roots[2].Capability); err != nil {
		t.Fatal(err)
	}
	before, err := copyPracticalSourceSnapshot(session)
	if err != nil {
		t.Fatal(err)
	}
	retainPracticalStandardSnapshot(t, before)
	exports, err := session.capturePracticalPackageExports(compiler, discovery, roots[2].Capability, practicalModuleLocation{capability: roots[0].Capability}, "example.com/app")
	if err != nil {
		var failure *practicalGoDiscoveryFailure
		if errors.As(err, &failure) {
			exports.output, exports.diagnostics = failure.originalOutput()
		}
	}
	retainPracticalSyntaxEvidence(t, "replay-native-original-export.json", map[string]any{"stdout": exports.output, "stderr": exports.diagnostics, "error": fmt.Sprint(err), "module_output": discovery.output})
	if err != nil {
		t.Fatal(err)
	}
	retainPracticalStandardSnapshot(t, exports.snapshot)
	sealed, err := session.sealPracticalReplaySources()
	if err != nil {
		t.Fatal(err)
	}
	retainPracticalStandardSnapshot(t, sealed.snapshot)
	replayInputs := []practicalReplaySources{sealed, exports.replay}
	image, err := materializePracticalReplaySources(output, replayInputs, &compiler, roots[2].Capability)
	if err != nil {
		t.Fatal(err)
	}
	// Deliberately retain the image for the independent second container.
	view, err := image.openVerifiedView()
	if err != nil {
		t.Fatal(err)
	}
	defer view.Close()
	if err := view.check(); err != nil {
		t.Fatal(err)
	}
	// Original roots are disposable fixture-owned directories, including the
	// separately cloned SDK. No original application/SDK path can supply data
	// to the following immutable module/source/export consumer.
	for _, root := range roots {
		if err := removePracticalReplayImage(root.Directory); err != nil {
			t.Fatal(err)
		}
		if _, err := os.Lstat(root.Directory); !os.IsNotExist(err) {
			t.Fatal("original producer root survived retirement", err)
		}
	}
	for _, input := range replayInputs {
		for _, buffer := range input.snapshot.buffers {
			clear(buffer.data)
		}
	}
	restored, err := view.restoreOriginalInputs()
	if err != nil {
		t.Fatal(err)
	}
	sources, err := restored.snapshotForRoots(sealed.snapshot.roots)
	if err != nil || sources.contentSHA256 != sealed.snapshot.contentSHA256 {
		t.Fatal("restored original source snapshot differs", err)
	}
	exportSnapshot, err := restored.snapshotForRoots(exports.snapshot.roots)
	if err != nil || exportSnapshot.contentSHA256 != exports.snapshot.contentSHA256 {
		t.Fatal("restored original export snapshot differs", err)
	}
	modules, err := restored.bindOriginalModuleOutput(practicalModuleLocation{capability: roots[0].Capability}, practicalModuleLocation{capability: roots[1].Capability}, discovery.output)
	if err != nil {
		t.Fatal(err)
	}
	rebound := practicalModuleDiscovery{output: bytes.Clone(discovery.output), compilerSHA256: discovery.compilerSHA256, mainSourceID: discovery.mainSourceID, modules: modules}
	loader, err := newPracticalSourceTypesWithStandardLibrary(sources, rebound, *image.compiler, roots[2].Capability)
	if err != nil {
		t.Fatal(err)
	}
	if _, err := loader.typecheck("example.com/app"); err != nil {
		t.Fatal(err)
	}
	exports.snapshot = exportSnapshot
	if err := exports.bindOriginalSourceTypes(loader, rebound, *image.compiler, roots[2].Capability); err != nil {
		t.Fatal(err)
	}
	if err := view.check(); err != nil {
		t.Fatal(err)
	}
	retainPracticalStandardSnapshot(t, restored.snapshot)
	var sourceSnapshots []map[string]any
	for _, input := range replayInputs {
		sourceSnapshots = append(sourceSnapshots, map[string]any{"digest": input.snapshot.contentSHA256, "roots": input.snapshot.roots, "origins": input.origins, "source_ids": input.snapshot.ids})
	}
	var mounts, inputs, namespaces []map[string]any
	for _, mount := range image.mounts {
		mounts = append(mounts, map[string]any{"capability": mount.capability, "original": mount.original, "private": mount.private})
	}
	for _, file := range image.files {
		inputs = append(inputs, map[string]any{"original": file.original, "private": image.privatePath(file.original), "objects": file.objects})
	}
	for _, directory := range image.namespace {
		namespaces = append(namespaces, map[string]any{"capability": directory.capability, "relative": directory.relative, "entries": directory.entries})
	}
	record := map[string]any{
		"image_root": image.root, "image_digest": image.contentSHA256, "mounts": mounts, "inputs": inputs, "original_namespaces": namespaces,
		"compiler_original_root": image.compilerOriginal, "compiler_distribution": compiler.selection.Toolchain.DistributionSHA256,
		"compiler_inventory": compiler.candidate.ToolchainInventory, "compiler_components": compiler.selection.Toolchain.Components,
		"main_original_directory": roots[0].Directory, "cache_original_directory": roots[1].Directory,
		"requested_package": "example.com/app", "original_module_output": discovery.output,
		"original_package_output": exports.output, "original_export_objects": exports.snapshot.sourceObjects(),
		"complete_source_typed_packages": len(loader.packages), "complete_syntax_nodes": loader.syntax.count,
		"registered_context_or_source_program_proof": false, "authoritative_selected_closure_replay_complete": false,
		"verified_materialized_files": len(view.plan.files), "reconciled_original_namespace_nodes": len(view.plan.nodes),
		"original_source_snapshots": sourceSnapshots, "restored_source_snapshot_digest": sources.contentSHA256,
		"restored_export_snapshot_digest":        exportSnapshot.contentSHA256,
		"original_roots_retired_before_consumer": true, "original_donor_buffers_cleared_before_consumer": true,
		"restored_original_module_bindings": len(modules), "restored_complete_original_objects": len(restored.snapshot.ids),
	}
	raw, err := json.MarshalIndent(record, "", "  ")
	if err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(filepath.Join(output, "replay-image.json"), append(raw, '\n'), 0o600); err != nil {
		t.Fatal(err)
	}
}
