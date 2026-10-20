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
	"reflect"
	"testing"

	"golang.org/x/mod/module"
)

func TestPracticalImageInputsRestoreAllRolesAndOwnTheirOriginalNamespaces(t *testing.T) {
	roots, _ := practicalTestCaptureRoots(t)
	roles := []string{"go_file", "go_mod", "go_sum", "export_data", "overlay", "metadata", "auxiliary", "excluded_source"}
	for _, kind := range roles {
		raw := append([]byte("original-"+kind+"\r\n"), 0, 0xff, '\n')
		writePracticalTestFile(t, filepath.Join(roots[0].Directory, kind), raw, 0o600)
	}
	writePracticalTestFile(t, filepath.Join(roots[0].Directory, "opaque"), []byte("unread-original"), 0o600)
	session := practicalTestSession(t, roots)
	for _, kind := range roles {
		if _, err := session.readSource(roots[0].Capability, kind, kind); err != nil {
			t.Fatal(err)
		}
	}
	sealed, err := session.sealPracticalReplaySources()
	if err != nil {
		t.Fatal(err)
	}
	retainPracticalNamespaceInput(t, "namespace-image-all-roles.json", sealed)
	image, err := materializePracticalReplaySources(t.TempDir(), []practicalReplaySources{sealed}, nil, "")
	if err != nil {
		t.Fatal(err)
	}
	view, err := image.openVerifiedView()
	if err != nil {
		t.Fatal(err)
	}
	defer view.Close()
	defer image.Close()
	for _, root := range roots {
		if err := os.RemoveAll(root.Directory); err != nil {
			t.Fatal(err)
		}
	}
	for _, buffer := range sealed.snapshot.buffers {
		clear(buffer.data)
	}
	restored, err := view.restoreOriginalInputs()
	if err != nil {
		t.Fatal(err)
	}
	if restored.snapshot.contentSHA256 != sealed.snapshot.contentSHA256 {
		t.Fatal("original snapshot framing or namespace changed")
	}
	if err := view.Close(); err != nil {
		t.Fatal(err)
	}
	if err := image.Close(); err != nil {
		t.Fatal(err)
	}
	for _, kind := range roles {
		buffer, err := restored.readSource(roots[0].Capability, kind, kind)
		if err != nil || !bytes.Equal(buffer.data, append([]byte("original-"+kind+"\r\n"), 0, 0xff, '\n')) {
			t.Fatal("restored consumer borrowed retired data or changed raw bytes", err)
		}
	}
	projection, err := restored.snapshotForRoots([]string{roots[0].Capability})
	if err != nil {
		t.Fatal(err)
	}
	for _, buffer := range projection.buffers {
		clear(buffer.data)
	}
	rootsCopy := restored.originalRoots()
	delete(rootsCopy, roots[0].Capability)
	if _, err := restored.readSource(roots[0].Capability, "go_file", "go_file"); err != nil || restored.originalRoots()[roots[0].Capability] != roots[0].Directory {
		t.Fatal("projected source/root metadata mutated its independent owner", err)
	}
	_, failure := restored.readSource(roots[0].Capability, "opaque", "auxiliary")
	if failure == nil || !errors.Is(failure, errPracticalNotCaptured) || errors.Is(failure, fs.ErrNotExist) {
		t.Fatal("unread original input acquired bytes or absence authority", failure)
	}
	if _, err := restored.readSource(roots[0].Capability, "go_file", "go_file"); err != failure {
		t.Fatal("ignored image-input failure lost its transaction scope", err)
	}
}

func practicalImageModuleFixture(t *testing.T) (*practicalReplayImage, practicalReplaySources, practicalModuleDiscovery, []practicalSourceRoot) {
	t.Helper()
	roots, base := practicalTestCaptureRoots(t)
	roots = append(roots, practicalSourceRoot{"relative", filepath.Join(base, "relative")}, practicalSourceRoot{"absolute", filepath.Join(base, "absolute")})
	identity := module.Version{Path: "example.com/dep", Version: "v1.1.0"}
	sums := writePracticalResolutionDependency(t, roots[1].Directory, identity, []byte("module example.com/dep\ngo 1.22\n"))
	manifest := []byte("module example.com/app\ngo 1.25.0\nrequire (\nexample.com/dep v1.0.0\nexample.com/relative v1.0.0\nexample.com/absolute v1.0.0\n)\nreplace example.com/dep v1.0.0 => example.com/dep v1.1.0\nreplace example.com/relative => ../relative\nreplace example.com/absolute => " + roots[3].Directory + "\n")
	writePracticalTestFile(t, filepath.Join(roots[0].Directory, "go.mod"), manifest, 0o600)
	writePracticalTestFile(t, filepath.Join(roots[0].Directory, "go.sum"), []byte(sums), 0o600)
	for index, path := range map[int]string{2: "example.com/relative", 3: "example.com/absolute"} {
		writePracticalTestFile(t, filepath.Join(roots[index].Directory, "go.mod"), []byte("module "+path+"\ngo 1.22\n"), 0o600)
	}
	session := practicalTestSession(t, roots)
	discovery, err := session.discoverPracticalModules(practicalModuleLocation{capability: roots[0].Capability}, practicalModuleLocation{capability: roots[1].Capability}, practicalTestResolverSelection(t))
	if err != nil {
		t.Fatal(err)
	}
	sealed, err := session.sealPracticalReplaySources()
	if err != nil {
		t.Fatal(err)
	}
	retainPracticalNamespaceInput(t, "namespace-image-original-modules.json", sealed)
	retainPracticalSyntaxEvidence(t, sha256Hex([]byte(t.Name()))+".image-original-module-output.json", map[string]any{"stdout": discovery.output, "compiler_sha256": discovery.compilerSHA256})
	image, err := materializePracticalReplaySources(t.TempDir(), []practicalReplaySources{sealed}, nil, "")
	if err != nil {
		t.Fatal(err)
	}
	for _, root := range roots {
		if err := os.RemoveAll(root.Directory); err != nil {
			t.Fatal(err)
		}
	}
	return image, sealed, discovery, roots
}

func TestPracticalImageInputsRebindWholeOriginalModuleGraphAfterRetirement(t *testing.T) {
	image, sealed, discovery, roots := practicalImageModuleFixture(t)
	defer image.Close()
	view, err := image.openVerifiedView()
	if err != nil {
		t.Fatal(err)
	}
	defer view.Close()
	inputs, err := view.restoreOriginalInputs()
	if err != nil {
		t.Fatal(err)
	}
	modules, err := inputs.bindOriginalModuleOutput(practicalModuleLocation{capability: roots[0].Capability}, practicalModuleLocation{capability: roots[1].Capability}, discovery.output)
	if err != nil || len(modules) != 4 || !reflect.DeepEqual(modules, discovery.modules) {
		t.Fatal("immutable image binding differs from the complete original Go selection", err)
	}
	if inputs.snapshot.contentSHA256 != sealed.snapshot.contentSHA256 {
		t.Fatal("module consumer changed its original input snapshot")
	}
	// Caller parser/cache views cannot change later original-byte binding.
	main, err := inputs.captureModule(practicalModuleLocation{capability: roots[0].Capability}, true)
	if err != nil {
		t.Fatal(err)
	}
	original, err := inputs.originalModuleSums(*main.sums)
	if err != nil {
		t.Fatal(err)
	}
	originalRaw := original.source.sourceBytes()
	clear(original.entries)
	clear(original.source.data)
	againSums, err := inputs.originalModuleSums(*main.sums)
	if err != nil {
		t.Fatal(err)
	}
	_, sourceError := againSums.source.canonicalObject()
	retainPracticalSyntaxEvidence(t, "image-checksum-source-ownership.json", map[string]any{
		"original_object": original.source.object, "original_raw": originalRaw,
		"returned_raw": againSums.source.data, "error": fmt.Sprint(sourceError),
		"semantic_source_counterexample_claimed": false,
	})
	if sourceError != nil || !bytes.Equal(againSums.source.data, originalRaw) {
		t.Fatal("returned checksum source changed the cached original byte evidence", sourceError)
	}
	clear(modules[2].manifest.data)
	for _, resolved := range modules {
		if resolved.versioned != nil {
			clear(resolved.versioned.archive.members)
			clear(resolved.versioned.archive.source.data)
		}
	}
	again, err := inputs.bindOriginalModuleOutput(main.location, practicalModuleLocation{capability: roots[1].Capability}, discovery.output)
	if err != nil || !reflect.DeepEqual(again, discovery.modules) {
		t.Fatal("returned module/archive/checksum views changed original image authority", err)
	}
}

func TestPracticalImageInputsRejectAlteredOriginalModuleObservations(t *testing.T) {
	image, _, discovery, roots := practicalImageModuleFixture(t)
	defer image.Close()
	for _, control := range []string{"main-directory", "main-manifest", "language", "version", "replacement-path", "replacement-version", "versioned-directory", "checksum", "duplicate", "no-main", "unknown-field", "checksum-owner"} {
		t.Run(control, func(t *testing.T) {
			view, err := image.openVerifiedView()
			if err != nil {
				t.Fatal(err)
			}
			defer view.Close()
			inputs, err := view.restoreOriginalInputs()
			if err != nil {
				t.Fatal(err)
			}
			rows, err := decodePracticalGoModules(discovery.output)
			if err != nil {
				t.Fatal(err)
			}
			mainIndex, versionedIndex := -1, -1
			for i := range rows {
				if rows[i].Main {
					mainIndex = i
				}
				if rows[i].Replace != nil && rows[i].Replace.Version != "" {
					versionedIndex = i
				}
			}
			switch control {
			case "main-directory":
				rows[mainIndex].Dir += "/other"
			case "main-manifest":
				rows[mainIndex].GoMod += ".other"
			case "language":
				rows[versionedIndex].GoVersion = "1.24"
			case "version":
				rows[versionedIndex].Version = "v1.0.1"
			case "replacement-path":
				rows[versionedIndex].Replace.Path = "example.com/other"
			case "replacement-version":
				rows[versionedIndex].Replace.Version = "v1.2.0"
			case "versioned-directory":
				rows[versionedIndex].Replace.Dir = roots[0].Directory
				rows[versionedIndex].Dir = roots[0].Directory
			case "checksum":
				rows[versionedIndex].Replace.Sum = "h1:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA="
			case "duplicate":
				rows = append(rows, rows[versionedIndex])
			case "no-main":
				rows[mainIndex].Main = false
			}
			var raw bytes.Buffer
			for _, row := range rows {
				if err := json.NewEncoder(&raw).Encode(row); err != nil {
					t.Fatal(err)
				}
			}
			candidate := raw.Bytes()
			if control == "unknown-field" {
				candidate = bytes.Replace(candidate, []byte("{\"Path\""), []byte("{\"Unknown\":true,\"Path\""), 1)
			}
			var failure error
			if control == "checksum-owner" {
				main, err := inputs.captureModule(practicalModuleLocation{capability: roots[0].Capability}, true)
				if err != nil {
					t.Fatal(err)
				}
				main.sums.source.object.Capability = roots[1].Capability
				_, failure = bindPracticalGoModuleRows(inputs, main, practicalModuleLocation{capability: roots[1].Capability}, rows)
			} else {
				_, failure = inputs.bindOriginalModuleOutput(practicalModuleLocation{capability: roots[0].Capability}, practicalModuleLocation{capability: roots[1].Capability}, candidate)
			}
			retainPracticalSyntaxEvidence(t, "image-module-"+control+".json", map[string]any{"original_output": discovery.output, "candidate_output": candidate, "error": fmt.Sprint(failure), "semantic_source_counterexample_claimed": false})
			if failure == nil {
				t.Fatal("altered native observation acquired original image authority")
			}
			if _, err := inputs.bindOriginalModuleOutput(practicalModuleLocation{capability: roots[0].Capability}, practicalModuleLocation{capability: roots[1].Capability}, discovery.output); err != inputs.failed {
				t.Fatal("ignored binding failure did not retain transaction failure", err)
			}
		})
	}
}

func TestPracticalImageInputsRejectBackingMutationBeforeRestoration(t *testing.T) {
	image, original, _ := practicalViewTestImage(t)
	defer image.Close()
	view, err := image.openVerifiedView()
	if err != nil {
		t.Fatal(err)
	}
	defer view.Close()
	if err := os.Chmod(image.privatePath(original), 0o644); err != nil {
		t.Fatal(err)
	}
	if _, failure := view.restoreOriginalInputs(); failure == nil || errors.Is(failure, fs.ErrNotExist) {
		t.Fatal("changed backing supplied an apparently original snapshot", failure)
	}
	if err := os.Chmod(image.privatePath(original), 0o444); err != nil {
		t.Fatal(err)
	}
	if _, err := view.restoreOriginalInputs(); err == nil {
		t.Fatal("restored mode erased the failed original-input transaction")
	}
}

func TestPracticalModuleChecksumReturnedViewsCannotMutateOriginals(t *testing.T) {
	fixture := preparePracticalModuleSnapshotFixture(t, practicalArchiveFixtureIdentity, false)
	session, sums := capturePracticalModuleSnapshotFixture(t, fixture, fixture.mod)
	retainPracticalSyntaxEvidence(t, "image-collection-checksum-original.json", map[string]any{
		"original_files": fixture.files, "original_raw": sums.source.data,
		"original_object": sums.source.object, "roots": fixture.roots, "identity": fixture.identity,
	})
	first, err := session.originalModuleSums(sums)
	if err != nil {
		t.Fatal(err)
	}
	clear(first.source.data)
	clear(first.entries)
	again, err := session.originalModuleSums(sums)
	if err != nil {
		t.Fatal(err)
	}
	_, failure := again.source.canonicalObject()
	retainPracticalSyntaxEvidence(t, "image-collection-checksum-result.json", map[string]any{
		"original_object": sums.source.object, "original_raw": sums.source.data,
		"returned_raw": again.source.data, "error": fmt.Sprint(failure),
		"semantic_source_counterexample_claimed": false,
	})
	if failure != nil || !bytes.Equal(again.source.data, sums.source.data) || len(again.entries) != len(sums.entries) {
		t.Fatal("returned checksum views changed the collector's original byte/map authority", failure)
	}
	if _, err := session.captureVersionedModule(fixture.location, fixture.identity, sums); err != nil {
		t.Fatal("returned checksum views changed original archive verification", err)
	}
}
