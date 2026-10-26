//go:build go1.25 && (linux || darwin)

package main

import (
	"bytes"
	"encoding/hex"
	"errors"
	"fmt"
	"io"
	"io/fs"
	"os"
	"path/filepath"
	"reflect"
	"strings"
	"testing"

	"golang.org/x/mod/module"
	"golang.org/x/mod/sumdb/dirhash"
)

func practicalGraphTestModOnly(t *testing.T, cache string, id module.Version, raw []byte) string {
	t.Helper()
	p, err := module.EscapePath(id.Path)
	if err != nil {
		t.Fatal(err)
	}
	v, err := module.EscapeVersion(id.Version)
	if err != nil {
		t.Fatal(err)
	}
	writePracticalTestFile(t, filepath.Join(cache, "cache/download", p, "@v", v+".mod"), raw, 0o600)
	sum, err := dirhash.Hash1([]string{"go.mod"}, func(string) (io.ReadCloser, error) { return io.NopCloser(bytes.NewReader(raw)), nil })
	if err != nil {
		t.Fatal(err)
	}
	return id.Path + " " + id.Version + "/go.mod " + sum + "\n"
}

func practicalGraphTestRows(summaries []practicalGraphSummary) []any {
	rows := []any{}
	for _, s := range summaries {
		var source any
		if s.source != nil {
			source = map[string]any{"object": s.source.object, "raw": s.source.sourceBytes()}
		}
		rows = append(rows, map[string]any{"identity": s.identity, "replacement": s.replacement, "source": source, "go_version": s.goVersion, "pruned": s.pruned, "requirements": s.require})
	}
	return rows
}

func practicalGraphTestOriginals(t *testing.T, roots []practicalSourceRoot) {
	t.Helper()
	rows := []any{}
	for _, root := range roots {
		err := filepath.WalkDir(root.Directory, func(name string, entry fs.DirEntry, failure error) error {
			if failure != nil {
				return failure
			}
			relative, err := filepath.Rel(root.Directory, name)
			if err != nil {
				return err
			}
			info, err := entry.Info()
			if err != nil {
				return err
			}
			row := map[string]any{"capability": root.Capability, "path": filepath.ToSlash(relative), "mode": uint32(info.Mode())}
			if entry.IsDir() {
				row["directory"] = true
			} else {
				if !info.Mode().IsRegular() {
					return fmt.Errorf("graph test original is not a regular file")
				}
				raw, err := os.ReadFile(name)
				if err != nil {
					return err
				}
				row["raw"] = raw
				row["sha256"] = sha256Hex(raw)
			}
			rows = append(rows, row)
			return nil
		})
		if err != nil {
			t.Fatal(err)
		}
	}
	retainPracticalSyntaxEvidence(t, sha256Hex([]byte(t.Name()))+".graph-generated-originals.json", map[string]any{"roots": roots, "whole_original_entries": rows})
}

func TestPracticalModuleGraphPreservesUnselectedOldLeavesCyclesAndPrunedInputs(t *testing.T) {
	selection := practicalTestResolverSelection(t)
	roots, _ := practicalTestCaptureRoots(t)
	main, cache := practicalModuleLocation{capability: roots[0].Capability}, practicalModuleLocation{capability: roots[1].Capability}
	manifest := []byte("module example.com/app\ngo 1.25.0\nrequire (\nexample.com/a v1.0.0\nexample.com/b v1.0.0\nexample.com/z v1.0.0\n)\nexclude example.com/never v1.0.0\n")
	sums := ""
	for _, row := range []struct{ path, version, body string }{
		{"example.com/a", "v1.0.0", "go 1.16\nrequire (\nexample.com/c v1.0.0\nexample.com/c v1.0.0\nexample.com/never v1.0.0\n)\n"},
		{"example.com/b", "v1.0.0", "go 1.16\nrequire (\nexample.com/c v1.1.0\nexample.com/leaf v1.1.0\nexample.com/q v1.1.0\n)\n"},
		{"example.com/c", "v1.1.0", "go 1.22\n"},
		{"example.com/leaf", "v1.1.0", "go 1.22\n"},
		{"example.com/q", "v1.1.0", "go 1.22\n"},
		{"example.com/z", "v1.0.0", "go 1.22\nrequire example.com/q v1.0.0\n"},
	} {
		sums += writePracticalResolutionDependency(t, roots[1].Directory, module.Version{Path: row.path, Version: row.version}, []byte("module "+row.path+"\n"+row.body))
	}
	// These original unselected inputs have no ZIP/info/source-tree surrogate.
	// ParseLax ignores the unknown directive in the old c version.
	old := module.Version{Path: "example.com/c", Version: "v1.0.0"}
	leaf := module.Version{Path: "example.com/leaf", Version: "v1.0.0"}
	sums += practicalGraphTestModOnly(t, roots[1].Directory, old, []byte("module example.com/c\ngo 1.16\nfuture_directive ignored\nrequire (\nexample.com/leaf v1.0.0\nexample.com/a v1.0.0\n)\n"))
	sums += practicalGraphTestModOnly(t, roots[1].Directory, leaf, []byte("module example.com/leaf\n// original zero-requirement old leaf\n"))
	writePracticalTestFile(t, filepath.Join(roots[0].Directory, "go.mod"), manifest, 0o600)
	writePracticalTestFile(t, filepath.Join(roots[0].Directory, "go.sum"), []byte(sums), 0o600)
	policy := []byte("package policy\nfunc Root(x int64)int64{return x}\nfunc Unused(m map[int]int)int{return len(m)}\n")
	writePracticalTestFile(t, filepath.Join(roots[0].Directory, "policy.go"), policy, 0o600)
	practicalGraphTestOriginals(t, roots)
	session := practicalTestSession(t, roots)
	if _, err := session.readSource(main.capability, "policy.go", "go_file"); err != nil {
		t.Fatal(err)
	}
	discovery, err := session.discoverPracticalModules(main, cache, selection)
	if err != nil {
		t.Fatal(err)
	}
	before, err := os.Stat(roots[1].Directory)
	if err != nil {
		t.Fatal(err)
	}
	discovery, err = session.capturePracticalModuleGraph(main, cache, discovery, selection)
	if err != nil {
		t.Fatal(err)
	}
	graph := discovery.graph
	seen := map[module.Version]practicalGraphSummary{}
	for _, s := range graph.final {
		seen[s.identity] = s
	}
	if seen[old].source == nil || seen[leaf].source == nil || len(seen[leaf].require) != 0 || seen[old].pruned || len(discovery.modules) != 7 {
		t.Fatal("unselected old/empty graph manifests were lost")
	}
	if _, ok := seen[module.Version{Path: "example.com/q", Version: "v1.0.0"}]; ok {
		t.Fatal("pruned original .mod was required")
	}
	if bytes.Contains(graph.output, []byte("example.com/leaf@v1.0.0 ")) || bytes.Contains(graph.output, []byte("example.com/never")) || !bytes.Contains(graph.output, []byte("example.com/z@v1.0.0 example.com/q@v1.0.0\n")) {
		t.Fatal("empty leaf, exclusions or pruned edges changed")
	}
	if bytes.Count(graph.output, []byte("example.com/a@v1.0.0 example.com/c@v1.0.0\n")) != 2 {
		t.Fatal("original duplicate requirement edges were discarded")
	}
	for _, s := range graph.final {
		if (s.identity.Path == "go" || s.identity.Path == "toolchain") && s.source != nil {
			t.Fatal("virtual graph source was manufactured")
		}
	}
	for name, expected := range map[string][]byte{"go.mod": manifest, "go.sum": []byte(sums), "policy.go": policy} {
		actual, err := os.ReadFile(filepath.Join(roots[0].Directory, name))
		if err != nil || !bytes.Equal(actual, expected) {
			t.Fatal("native graph rewrote original application input", name, err)
		}
	}
	after, err := os.Stat(roots[1].Directory)
	if err != nil || before.Mode() != after.Mode() {
		t.Fatal("graph scratch cleanup changed original cache permissions")
	}
	sealed, err := session.sealPracticalReplaySources()
	if err != nil {
		t.Fatal(err)
	}
	retainPracticalNamespaceInput(t, "graph-whole-original-inputs.json", sealed)
	retainPracticalSyntaxEvidence(t, sha256Hex([]byte(t.Name()))+".graph-whole-original-observation.json", map[string]any{"stdout": graph.output, "stderr": graph.diagnostics, "module_stdout": discovery.output, "module_stderr": discovery.observation.diagnostics, "arguments": graph.observation.arguments, "environment": graph.observation.environment, "initial": practicalGraphTestRows(graph.initial), "final": practicalGraphTestRows(graph.final), "selected_modules": len(discovery.modules), "independent_MVS_or_source_proof": false})
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
	for _, buffer := range sealed.snapshot.buffers {
		clear(buffer.data)
	}
	view, err := image.openVerifiedView()
	if err != nil {
		t.Fatal(err)
	}
	defer view.Close()
	inputs, err := view.restoreOriginalInputs()
	if err != nil {
		t.Fatal(err)
	}
	rebound, err := inputs.restoreOriginalDiscovery(main, cache, discovery)
	if err != nil || !bytes.Equal(rebound.graph.output, graph.output) || !reflect.DeepEqual(practicalGraphTestRows(rebound.graph.final), practicalGraphTestRows(graph.final)) {
		t.Fatal("immutable graph differs after original root retirement", err)
	}
	// Prepared parser views are not consumed during immutable reconstruction.
	graph.final[0].require = nil
	inputs, err = view.restoreOriginalInputs()
	if err != nil {
		t.Fatal(err)
	}
	again, err := inputs.restoreOriginalDiscovery(main, cache, discovery)
	if err != nil || !reflect.DeepEqual(practicalGraphTestRows(again.graph.final), practicalGraphTestRows(rebound.graph.final)) {
		t.Fatal("caller parser view changed immutable graph", err)
	}
	for _, control := range []string{"output", "diagnostics", "unobserved", "cross-command", "cross-module"} {
		t.Run(control, func(t *testing.T) {
			inputs, err := view.restoreOriginalInputs()
			if err != nil {
				t.Fatal(err)
			}
			candidate := discovery
			copyGraph := *graph
			candidate.graph = &copyGraph
			switch control {
			case "output":
				copyGraph.output = append(bytes.Clone(graph.output), '\n')
			case "diagnostics":
				copyGraph.diagnostics = []byte("caller diagnostic")
			case "unobserved":
				copyGraph.observation = nil
			case "cross-command":
				copyGraph.observation = discovery.observation
			case "cross-module":
				copyGraph.moduleObservation = nil
			}
			_, failure := inputs.restoreOriginalDiscovery(main, cache, candidate)
			retainPracticalSyntaxEvidence(t, "graph-control-"+control+".json", map[string]any{"candidate_output": copyGraph.output, "candidate_diagnostics": copyGraph.diagnostics, "error": fmt.Sprint(failure)})
			if failure == nil {
				t.Fatal("changed graph observation acquired original ownership")
			}
		})
	}
	// Even a coherently changed observation copy must agree with every original
	// dependency summary, native traversal order and selected module inventory.
	for _, control := range []string{"changed-requirement", "dropped-virtual-edge", "reordered-root-edges", "disconnected-edge"} {
		t.Run(control, func(t *testing.T) {
			inputs, err := view.restoreOriginalInputs()
			if err != nil {
				t.Fatal(err)
			}
			mainInput, err := inputs.captureModule(main, true)
			if err != nil {
				t.Fatal(err)
			}
			observation := *graph.observation
			switch control {
			case "changed-requirement":
				observation.output = bytes.ReplaceAll(graph.output, []byte("example.com/a@v1.0.0 example.com/c@v1.0.0\n"), []byte("example.com/a@v1.0.0 example.com/c@v1.1.0\n"))
			case "dropped-virtual-edge":
				observation.output = bytes.ReplaceAll(graph.output, []byte("go@1.25.0 toolchain@go1.25.0\n"), nil)
			case "reordered-root-edges":
				observation.output = bytes.ReplaceAll(graph.output, []byte("example.com/app example.com/a@v1.0.0\nexample.com/app example.com/b@v1.0.0\n"), []byte("example.com/app example.com/b@v1.0.0\nexample.com/app example.com/a@v1.0.0\n"))
			case "disconnected-edge":
				observation.output = append(bytes.Clone(graph.output), []byte("example.com/q@v1.0.0 go@1.22\n")...)
			}
			_, failure := practicalBindGraphOutput(inputs, mainInput, cache, rebound, &observation)
			retainPracticalSyntaxEvidence(t, "graph-coherent-"+control+".json", map[string]any{"whole_candidate_output": observation.output, "error": fmt.Sprint(failure)})
			if bytes.Equal(observation.output, graph.output) || failure == nil {
				t.Fatal("coherently changed graph accepted or control did not change bytes", control, failure)
			}
		})
	}
}

func TestPracticalModuleGraphRejectsChangedWholeEdges(t *testing.T) {
	mainPath := "example.com/app"
	for _, raw := range []string{"", mainPath + " go@1.25.0", mainPath + "\tgo@1.25.0\n", mainPath + "  go@1.25.0\n", mainPath + " go@invalid\n", mainPath + " example.com/dep@v1\n", mainPath + " example.com/dep@v1.0.0@other\n", "other go@1.25.0\n", mainPath + " go@1.25.0\r\n"} {
		_, err := practicalGraphEdges([]byte(raw), mainPath)
		retainPracticalSyntaxEvidence(t, sha256Hex([]byte(raw))+".graph-framing-control.json", map[string]any{"whole_output": []byte(raw), "error": fmt.Sprint(err)})
		if err == nil {
			t.Fatal("invalid graph framing/identity accepted", raw)
		}
	}
}

func TestPracticalModuleGraphHandoffSurvivesCompleteOwnerRetirement(t *testing.T) {
	practicalContractTestOutput(t, "graph-owned")
	owner, _ := practicalStorageTestOwnerWithGraph(t, map[string][]byte{
		"go.mod":          []byte("module example.com/policy\ngo 1.25.0\nrequire (\nexample.com/alias v1.0.0\nexample.com/shared v1.0.0\n)\nreplace (\nexample.com/alias => ./wildcard\nexample.com/alias v1.0.0 => ./local-a\nexample.com/shared => ./local-b\n)\n"),
		"policy.go":       []byte("package policy\nimport(a \"example.com/alias\";b \"example.com/shared\";\"net/http\")\nfunc Root(x uint16)uint16{return x+1}\nfunc Unused(r *http.Request)uint16{go func(){select{}}();return a.Value()+b.Value()}\nfunc init(){panic(\"original init must never execute\")}\n"),
		"local-a/go.mod":  []byte("module example.com/alias\ngo 1.22\n"),
		"local-a/a.go":    []byte("package a\nfunc Value()uint16{return 1}\n"),
		"local-b/go.mod":  []byte("module example.com/shared\ngo 1.23\n"),
		"local-b/b.go":    []byte("package b\nfunc Value()uint16{return 2}\n"),
		"wildcard/go.mod": []byte("module example.com/alias\ngo 1.24\n"),
	}, "example.com/policy", nil, true)
	defer owner.Close()
	roots := []practicalClosureRef{practicalClosureTestRef(t, owner, "example.com/policy", "Root")}
	input, err := owner.moduleGraphInputs(roots)
	if err != nil {
		t.Fatal(err)
	}
	packet := practicalCaptureInputEvidence(t, input.modules.exports.capture)
	packet["export_inputs_hex"] = hex.EncodeToString(input.modules.exports.canonical)
	packet["module_observations_hex"] = hex.EncodeToString(input.modules.canonical)
	packet["module_graph_inputs_hex"] = hex.EncodeToString(input.canonical)
	retainPracticalSyntaxEvidence(t, "complete-original-module-graph-inputs.json", packet)
	original := bytes.Clone(input.canonical)
	clear(input.canonical)
	clear(input.modules.canonical)
	clear(input.modules.exports.canonical)
	for _, source := range input.modules.exports.capture.snapshot.sources {
		clear(source.data)
	}
	again, err := owner.moduleGraphInputs(roots)
	if err != nil || !bytes.Equal(again.canonical, original) {
		t.Fatal("graph handoff aliases owner", err)
	}
	packages, objects := len(owner.packages), len(owner.inputs.snapshot.ids)
	if _, ok := owner.packages["net/http"]; !ok || packages < 180 {
		t.Fatal("unused original HTTP package universe disappeared")
	}
	owner.Close()
	if !bytes.Equal(again.canonical, original) {
		t.Fatal("graph handoff retired with original owner")
	}
	for _, closed := range []*practicalOriginalPackages{nil, owner} {
		if got, err := closed.moduleGraphInputs(roots); err == nil || got.canonical != nil {
			t.Fatal("absent/closed graph owner accepted")
		}
	}
	retainPracticalSyntaxEvidence(t, "graph-original-ownership.json", map[string]any{"packages": packages, "source_objects": objects, "unused_http_retained": true, "immutable_image_and_original_roots_retired": true, "detached_input_buffers": true, "independent_MVS_or_source_proof": false})
}

func TestPracticalModuleGraphRetainsVersionedReplacementAndInitialRootUpgrade(t *testing.T) {
	selection := practicalTestResolverSelection(t)
	for _, upgrade := range []bool{false, true} {
		t.Run(map[bool]string{false: "versioned-replacement", true: "initial-root-upgrade"}[upgrade], func(t *testing.T) {
			roots, _ := practicalTestCaptureRoots(t)
			main, cache := practicalModuleLocation{capability: roots[0].Capability}, practicalModuleLocation{capability: roots[1].Capability}
			manifest := "module example.com/app\ngo 1.25.0\nrequire example.com/alias v1.0.0\n"
			sums := ""
			if upgrade {
				manifest += "require example.com/b v1.0.0\n"
				sums += writePracticalResolutionDependency(t, roots[1].Directory, module.Version{Path: "example.com/b", Version: "v1.0.0"}, []byte("module example.com/b\ngo 1.22\nrequire example.com/alias v1.1.0\n"))
				sums += practicalGraphTestModOnly(t, roots[1].Directory, module.Version{Path: "example.com/alias", Version: "v1.0.0"}, []byte("module example.com/alias\ngo 1.16\nfuture_directive ignored\n"))
			} else {
				manifest += "replace example.com/alias v1.0.0 => example.com/Actual v1.1.0\n"
			}
			actual := "example.com/Actual"
			if upgrade {
				actual = "example.com/alias"
			}
			sums += writePracticalResolutionDependency(t, roots[1].Directory, module.Version{Path: actual, Version: "v1.1.0"}, []byte("module "+actual+"\ngo 1.22\n"))
			writePracticalTestFile(t, filepath.Join(roots[0].Directory, "go.mod"), []byte(manifest), 0o600)
			writePracticalTestFile(t, filepath.Join(roots[0].Directory, "go.sum"), []byte(sums), 0o600)
			practicalGraphTestOriginals(t, roots)
			session := practicalTestSession(t, roots)
			discovery, err := session.discoverPracticalModules(main, cache, selection)
			if upgrade {
				// Inconsistent original roots are rejected by the existing readonly
				// module-list command. Retain this case and the complete diagnostics;
				// it must not acquire an accepted graph owner via a looser command.
				var failure *practicalGoDiscoveryFailure
				if !errors.As(err, &failure) {
					t.Fatal("expected original readonly Go root-upgrade rejection", err)
				}
				stdout, stderr := failure.originalOutput()
				retainPracticalSyntaxEvidence(t, "graph-readonly-root-upgrade-rejection.json", map[string]any{"stdout": stdout, "stderr": stderr, "error": err.Error(), "accepted_owner": false})
				if !bytes.Contains(stderr, []byte("updates to go.mod needed")) {
					t.Fatal("readonly root-upgrade control failed for another reason", string(stderr))
				}
				fresh := practicalTestSession(t, roots)
				original, err := fresh.captureModule(main, true)
				if err != nil {
					t.Fatal(err)
				}
				initial, err := practicalOriginalGraphSummaries(fresh, original, cache, practicalGraphOriginalRoots(original))
				if err != nil {
					t.Fatal(err)
				}
				filesystem, err := newLoaderFilesystem()
				if err != nil {
					t.Fatal(err)
				}
				defer filesystem.Close()
				environment := filesystem
				environment.goMod = roots[1].Directory
				observed, err := runPracticalGoObservation(selection, environment, roots[0].Directory, original.manifest.source.object.ID, practicalGoGraph, "")
				if err != nil {
					t.Fatal(err)
				}
				found := false
				for _, s := range initial {
					if s.identity == (module.Version{Path: "example.com/alias", Version: "v1.0.0"}) && s.source != nil {
						found = true
					}
				}
				if !found || !bytes.Contains(observed.output, []byte("example.com/app example.com/alias@v1.1.0\n")) || bytes.Contains(observed.output, []byte("example.com/app example.com/alias@v1.0.0\n")) {
					t.Fatal("native expanded graph/initial original root changed")
				}
				for name, expected := range map[string][]byte{"go.mod": []byte(manifest), "go.sum": []byte(sums)} {
					actual, err := os.ReadFile(filepath.Join(roots[0].Directory, name))
					if err != nil || !bytes.Equal(actual, expected) {
						t.Fatal("native mod graph rewrote original rejected input", err)
					}
				}
				sealed, err := fresh.sealPracticalReplaySources()
				if err != nil {
					t.Fatal(err)
				}
				retainPracticalNamespaceInput(t, "graph-initial-rejected-original-inputs.json", sealed)
				retainPracticalSyntaxEvidence(t, "graph-rejected-native-root-expansion.json", map[string]any{"stdout": observed.output, "stderr": observed.diagnostics, "initial": practicalGraphTestRows(initial), "accepted_owner": false, "independent_MVS_or_source_proof": false})
				return
			}
			if err != nil {
				t.Fatal(err)
			}
			discovery, err = session.capturePracticalModuleGraph(main, cache, discovery, selection)
			if err != nil {
				t.Fatal(err)
			}
			graph := discovery.graph
			found := false
			for _, s := range graph.final {
				if s.identity.Path == "example.com/alias" && s.replacement.Path == actual && s.source != nil && strings.Contains(s.source.object.Path, "!actual") {
					found = true
				}
			}
			if !found {
				t.Fatal("graph versioned replacement crossed its original manifest")
			}
			sealed, err := session.sealPracticalReplaySources()
			if err != nil {
				t.Fatal(err)
			}
			retainPracticalNamespaceInput(t, "graph-whole-original-inputs.json", sealed)
			retainPracticalSyntaxEvidence(t, sha256Hex([]byte(t.Name()))+".graph-whole-original-observation.json", map[string]any{"stdout": graph.output, "stderr": graph.diagnostics, "module_stdout": discovery.output, "module_stderr": discovery.observation.diagnostics, "initial": practicalGraphTestRows(graph.initial), "final": practicalGraphTestRows(graph.final), "independent_MVS_or_source_proof": false})
		})
	}
}

func TestPracticalModuleGraphKeepsOriginalVirtualToolchainInputs(t *testing.T) {
	selection := practicalTestResolverSelection(t)
	for _, suggestion := range []string{"default", "go1.25.0", "go1.25.0-original"} {
		t.Run(suggestion, func(t *testing.T) {
			roots, _ := practicalTestCaptureRoots(t)
			manifest := []byte("module example.com/app\ngo 1.25.0\ntoolchain " + suggestion + "\n")
			writePracticalTestFile(t, filepath.Join(roots[0].Directory, "go.mod"), manifest, 0o600)
			practicalGraphTestOriginals(t, roots)
			session := practicalTestSession(t, roots)
			main, err := session.captureModule(practicalModuleLocation{capability: roots[0].Capability}, true)
			if err != nil {
				t.Fatal(err)
			}
			cache := practicalModuleLocation{capability: roots[1].Capability}
			initial, err := practicalOriginalGraphSummaries(session, main, cache, practicalGraphOriginalRoots(main))
			if err != nil {
				t.Fatal(err)
			}
			found := false
			for _, s := range initial {
				if s.source != nil {
					t.Fatal("virtual input acquired fabricated source")
				}
				if s.identity == (module.Version{Path: "toolchain", Version: suggestion}) {
					found = true
				}
			}
			if !found {
				t.Fatal("original explicit toolchain root was lost")
			}
			filesystem, err := newLoaderFilesystem()
			if err != nil {
				t.Fatal(err)
			}
			defer filesystem.Close()
			environment := filesystem
			environment.goMod = roots[1].Directory
			observation, err := runPracticalGoObservation(selection, environment, roots[0].Directory, main.manifest.source.object.ID, practicalGoGraph, "")
			if err != nil {
				t.Fatal(err)
			}
			edges, err := practicalGraphEdges(observation.output, "example.com/app")
			if err != nil {
				t.Fatal(err)
			}
			final, err := practicalOriginalGraphSummaries(session, main, cache, edges[module.Version{Path: "example.com/app"}])
			if err != nil {
				t.Fatal(err)
			}
			if !bytes.Contains(observation.output, []byte("go@1.25.0 toolchain@go1.25.0\n")) {
				t.Fatal("native virtual Go/toolchain requirement changed")
			}
			actual, err := os.ReadFile(filepath.Join(roots[0].Directory, "go.mod"))
			if err != nil || !bytes.Equal(actual, manifest) {
				t.Fatal("native graph rewrote original toolchain suggestion", err)
			}
			sealed, err := session.sealPracticalReplaySources()
			if err != nil {
				t.Fatal(err)
			}
			retainPracticalNamespaceInput(t, "graph-virtual-original-inputs.json", sealed)
			retainPracticalSyntaxEvidence(t, sha256Hex([]byte(t.Name()))+".graph-virtual-observation.json", map[string]any{"suggestion": suggestion, "stdout": observation.output, "stderr": observation.diagnostics, "initial": practicalGraphTestRows(initial), "final": practicalGraphTestRows(final), "independent_MVS_or_source_proof": false})
		})
	}
}
