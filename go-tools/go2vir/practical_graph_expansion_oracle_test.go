//go:build go1.25 && (linux || darwin)

package main

import (
	"bytes"
	"encoding/hex"
	"errors"
	"fmt"
	"os"
	"path/filepath"
	"slices"
	"sort"
	"strings"
	"testing"

	"golang.org/x/mod/module"
	"golang.org/x/mod/semver"
)

func practicalGraphIdentityRow(m module.Version) map[string]any {
	return map[string]any{"path": m.Path, "version_hex": hex.EncodeToString([]byte(m.Version))}
}

func practicalGraphExpansionRows(summaries []practicalGraphSummary) []any {
	rows := []any{}
	for _, s := range summaries {
		required := []any{}
		for _, m := range s.require {
			required = append(required, practicalGraphIdentityRow(m))
		}
		var source any
		if s.source != nil {
			source = map[string]any{"object": s.source.object, "raw_hex": hex.EncodeToString(s.source.sourceBytes())}
		}
		rows = append(rows, map[string]any{"identity": practicalGraphIdentityRow(s.identity), "go_version": s.goVersion, "pruned": s.pruned, "requirements": required, "source": source})
	}
	return rows
}

func TestPracticalOriginalGraphExpansionOracle(t *testing.T) {
	practicalContractTestOutput(t, "root-expansion")
	selection := practicalTestResolverSelection(t)
	type dep struct{ name, v, body string }
	base := []dep{{"a", "v1.0.0", "go 1.22\nrequire example.com/c v1.1.0\n"}, {"c", "v1.0.0", "go 1.22\n"}, {"c", "v1.1.0", "go 1.22\nrequire example.com/z v1.1.0\n"}, {"c", "v1.2.0", "go 1.22\nrequire example.com/unread v1.0.0\n"}, {"z", "v1.0.0", "go 1.22\n"}, {"z", "v1.1.0", "go 1.22\nrequire example.com/c v1.2.0\n"}}
	cases := []struct {
		name, goVersion, directives string
		deps                        []dep
	}{
		{"empty", "1.25.0", "", nil},
		{"language-1.22", "1.22", "", nil},
		{"language-1.23", "1.23", "", nil},
		{"language-1.24", "1.24", "", nil},
		{"self-root-is-discarded", "1.25.0", "require example.com/app v1.0.0\n", nil},
		{"discarded-lower-root-needs-no-manifest", "1.25.0", "require (\nexample.com/c v1.0.0\nexample.com/c v1.2.0\n)\n", []dep{{"c", "v1.2.0", "go 1.22\n"}}},
		{"cascade-transient", "1.25.0", "require (\nexample.com/a v1.0.0\nexample.com/c v1.0.0\nexample.com/z v1.0.0\n)\n", base},
		{"duplicate-consistent", "1.25.0", "require (\nexample.com/c v1.0.0\nexample.com/c v1.0.0\n)\n", base},
		{"duplicate-inconsistent", "1.25.0", "require (\nexample.com/c v1.0.0\nexample.com/c v1.2.0\nexample.com/z v1.0.0\n)\n", base},
		{"old-unprunes-modern", "1.25.0", "require example.com/old v1.0.0\n", []dep{{"old", "v1.0.0", "go 1.16\nrequire example.com/modern v1.0.0\n"}, {"modern", "v1.0.0", "go 1.22\nrequire example.com/leaf v1.0.0\n"}, {"leaf", "v1.0.0", "// no go directive, no outgoing edges\n"}}},
		{"cycle-upgrades", "1.25.0", "require (\nexample.com/a v1.0.0\nexample.com/c v1.0.0\n)\n", []dep{{"a", "v1.0.0", "go 1.16\nrequire example.com/c v1.1.0\n"}, {"c", "v1.0.0", "go 1.16\nrequire example.com/a v1.0.0\n"}, {"c", "v1.1.0", "go 1.16\nrequire example.com/a v1.0.0\n"}}},
		{"excluded-upgrade", "1.25.0", "require (\nexample.com/a v1.0.0\nexample.com/c v1.0.0\n)\nexclude example.com/c v1.1.0\n", base},
		{"go-root-upgrade", "1.22", "require example.com/a v1.0.0\n", []dep{{"a", "v1.0.0", "go 1.25.0\n"}}},
		{"toolchain-default", "1.25.0", "toolchain default\n", nil},
		{"toolchain-older", "1.25.0", "toolchain go1.24.0\n", nil},
		{"toolchain-equal-suffix", "1.25.0", "toolchain go1.25.0-original-" + string([]byte{255}) + "\n", nil},
		{"nonroot-is-not-promoted", "1.25.0", "require example.com/a v1.0.0\n", base},
		{"equal-real-labels", "1.25.0", "require (\nexample.com/equal v1.0.0+incompatible\nexample.com/equal v1.0.0\n)\n", []dep{{"equal", "v1.0.0+incompatible", "go 1.22\n"}, {"equal", "v1.0.0", "go 1.22\n"}}},
	}
	rows := []any{}
	for _, test := range cases {
		t.Run(test.name, func(t *testing.T) {
			practicalContractTestOutput(t, test.name)
			roots, _ := practicalTestCaptureRoots(t)
			manifest := []byte("module example.com/app\ngo " + test.goVersion + "\n" + test.directives)
			sums := ""
			for _, d := range test.deps {
				sums += practicalGraphTestModOnly(t, roots[1].Directory, module.Version{Path: "example.com/" + d.name, Version: d.v}, []byte("module example.com/"+d.name+"\n"+d.body))
			}
			writePracticalTestFile(t, filepath.Join(roots[0].Directory, "go.mod"), manifest, 0o600)
			writePracticalTestFile(t, filepath.Join(roots[0].Directory, "go.sum"), []byte(sums), 0o600)
			practicalGraphTestOriginals(t, roots)
			session := practicalTestSession(t, roots)
			main, err := session.captureModule(practicalModuleLocation{capability: roots[0].Capability}, true)
			if err != nil {
				t.Fatal(err)
			}
			cache := practicalModuleLocation{capability: roots[1].Capability}
			expanded, initial, final, err := practicalOriginalExpandedGraph(session, main, cache)
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
			observation, nativeErr := runPracticalGoObservation(selection, environment, roots[0].Directory, main.manifest.source.object.ID, practicalGoGraph, "")
			stdout, stderr := observationOutput(observation), []byte(nil)
			if observation != nil {
				stderr = observation.diagnostics
			}
			var failure *practicalGoDiscoveryFailure
			if errors.As(nativeErr, &failure) {
				stdout, stderr = failure.originalOutput()
			}
			retainPracticalSyntaxEvidence(t, "native-graph-command.json", map[string]any{"error": fmt.Sprint(nativeErr), "stdout": stdout, "stderr": stderr, "original_main_hex": hex.EncodeToString(manifest)})
			if nativeErr != nil {
				t.Fatal(nativeErr)
			}
			edges, err := practicalGraphEdges(observation.output, "example.com/app")
			if err != nil || !slices.Equal(expanded, edges[module.Version{Path: "example.com/app"}]) {
				t.Fatal("source-derived roots differ from native expansion", err, expanded, edges)
			}
			for name, original := range map[string][]byte{"go.mod": manifest, "go.sum": []byte(sums)} {
				actual, err := os.ReadFile(filepath.Join(roots[0].Directory, name))
				if err != nil || !bytes.Equal(original, actual) {
					t.Fatal("native graph rewrote original input", name, err)
				}
			}
			sealed, err := session.sealPracticalReplaySources()
			if err != nil {
				t.Fatal(err)
			}
			retainPracticalNamespaceInput(t, "whole-original-expansion-inputs.json", sealed)
			sources := []any{}
			for _, d := range test.deps {
				id := module.Version{Path: "example.com/" + d.name, Version: d.v}
				p, _ := module.EscapePath(id.Path)
				v, _ := module.EscapeVersion(id.Version)
				// Unread test files remain original inputs too; capture them only
				// after expansion, so they cannot hide a missing transient read.
				raw, err := os.ReadFile(filepath.Join(roots[1].Directory, "cache/download", p, "@v", v+".mod"))
				if err != nil {
					t.Fatal(err)
				}
				source, err := newPracticalSourceBuffer(roots[1].Capability, "cache/download/"+p+"/@v/"+v+".mod", "go_mod", raw)
				if err != nil {
					t.Fatal(err)
				}
				sources = append(sources, map[string]any{"module": practicalGraphIdentityRow(id), "object": source.object, "raw_hex": hex.EncodeToString(raw), "captured_during_expansion": slices.Contains(sealed.snapshot.ids, source.object.ID)})
			}
			rootRows := []any{}
			for _, r := range expanded {
				rootRows = append(rootRows, practicalGraphIdentityRow(r))
			}
			rows = append(rows, map[string]any{"case": test.name, "main": map[string]any{"object": main.manifest.source.object, "raw_hex": hex.EncodeToString(manifest)}, "sources": sources, "derived_roots": rootRows, "initial": practicalGraphExpansionRows(initial), "final": practicalGraphExpansionRows(final), "metadata_hex": hex.EncodeToString(observation.output), "diagnostics_hex": hex.EncodeToString(observation.diagnostics), "arguments": observation.arguments, "environment": observation.environment, "source_program_or_context_authority": false})
		})
	}
	retainPracticalSyntaxEvidence(t, "complete-original-root-expansion.json", map[string]any{"cases": rows})
}

func TestPracticalOriginalGraphOrderingOracle(t *testing.T) {
	practicalContractTestOutput(t, "root-ordering")
	comparisons := []any{}
	versions := []string{"v0.0.0", "v1.0.0-0", "v1.0.0-1", "v1.0.0-alpha", "v1.0.0-alpha.1", "v1.0.0-alpha.beta", "v1.0.0-beta.11", "v1.0.0-rc.1", "v1.0.0", "v1.0.0+incompatible", "v1.0.1", "v1.10.0", "v2.0.0", "v999999999999999999999999999999999999999.0.0", "v1.0.0-999999999999999999999999999999999999999"}
	for _, a := range versions {
		for _, b := range versions {
			comparisons = append(comparisons, map[string]any{"path": "example.com/version", "a_hex": hex.EncodeToString([]byte(a)), "b_hex": hex.EncodeToString([]byte(b)), "order": semver.Compare(a, b)})
		}
	}
	sorts := []any{}
	for _, n := range []int{0, 1, 2, 11, 12, 13, 49, 50, 51, 127, 128, 129, 513, 2048} {
		for _, pattern := range []string{"ascending", "descending", "equal", "saw", "organ", "random"} {
			before := []module.Version{}
			state := uint64(n + 1)
			for i := 0; i < n; i++ {
				key := i
				switch pattern {
				case "descending":
					key = n - i
				case "equal":
					key = 0
				case "saw":
					key = i % 7
				case "organ":
					key = min(i, n-i)
				case "random":
					state ^= state << 13
					state ^= state >> 7
					state ^= state << 17
					key = int(state % 17)
				}
				// Exact identities differ but each path's pair has equal semantic
				// versions. Their order exposes the actual unstable permutation.
				v := "v1.0.0"
				if i%2 == 0 {
					v += "+incompatible"
				}
				before = append(before, module.Version{Path: fmt.Sprintf("example.com/p%05d", key), Version: v})
			}
			after := slices.Clone(before)
			sort.Slice(after, func(i, j int) bool {
				if after[i].Path != after[j].Path {
					return after[i].Path < after[j].Path
				}
				return semver.Compare(after[i].Version, after[j].Version) < 0
			})
			encode := func(values []module.Version) []any {
				result := []any{}
				for _, m := range values {
					result = append(result, practicalGraphIdentityRow(m))
				}
				return result
			}
			sorts = append(sorts, map[string]any{"case": fmt.Sprintf("%s-%d", pattern, n), "before": encode(before), "after": encode(after)})
		}
	}
	retainPracticalSyntaxEvidence(t, "complete-original-root-ordering.json", map[string]any{"comparisons": comparisons, "sorts": sorts, "toolchain": practicalCompilerRelease, "version_labels": strings.Join(versions, "\n")})
}
