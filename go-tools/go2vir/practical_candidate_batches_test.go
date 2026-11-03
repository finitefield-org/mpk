//go:build go1.25 && (linux || darwin)

package main

import (
	"bytes"
	"maps"
	"os"
	"reflect"
	"sort"
	"testing"
)

func TestPracticalGoBatchObservationsKeepOriginalPackageMetadata(t *testing.T) {
	compiler := practicalTestRegisteredCompiler(t)
	additional := []byte("func OtherRoot(x uint16) uint16 { return x * 2 }\n")
	extra := []practicalCandidateExtraFile{{name: "second/policy.go", data: []byte("package second\nfunc Select(x uint16) uint16 { return x + 3 }\n")}}
	practicalCompilerInputsWithCandidateHook(t, compiler, "Root", additional, nil, extra,
		func(session *practicalCaptureSession, discovery practicalModuleDiscovery, main practicalModuleLocation, compiler practicalCompilerImage, sdk string) {
			request := practicalTestCandidateRequest(t, []practicalCandidateSelector{
				{Package: "example.com/c", Function: "Identity"},
				{Package: "example.com/policy", Function: "Root"},
				{Package: "example.com/policy", Function: "OtherRoot"},
				{Package: "example.com/policy/second", Function: "Select"},
			})
			exports, err := session.capturePracticalCandidatePackageExports(compiler, discovery, sdk, main, request)
			if err != nil {
				t.Fatal(err)
			}
			sources, err := copyPracticalSourceSnapshot(session)
			if err != nil || len(sources.sourceObjects()) != 5517 || len(exports.packages) != 183 || len(exports.snapshot.sourceObjects()) != 182 {
				t.Fatal("complete original source/export universe changed", err)
			}
			retainPracticalStandardSnapshot(t, sources)
			retainPracticalStandardSnapshot(t, exports.snapshot)
			filesystem, err := newLoaderFilesystem()
			if err != nil {
				t.Fatal(err)
			}
			defer filesystem.Close()
			filesystem.goMod = discovery.observation.moduleCache
			directory := discovery.observation.directory
			whole, err := runPracticalGoCandidatePackageObservation(compiler.selection, filesystem, directory, discovery.mainSourceID, request)
			if err != nil {
				t.Fatal(err)
			}
			if err := whole.requireCandidatePackages(request, discovery.compilerSHA256, discovery.mainSourceID, whole.output); err != nil {
				t.Fatal(err)
			}
			wholeMetadata, err := decodePracticalPackageExports(whole.output)
			if err != nil || validatePracticalExportRoots(wholeMetadata, exports.requestedPackages) != nil {
				t.Fatal("whole native observation lost its actual selected roots", err)
			}
			wholeRows, err := decodePracticalCompleteObjects(whole.output)
			if err != nil || len(wholeRows) != 183 {
				t.Fatal("whole original native package output changed", err)
			}
			byPath := map[string]map[string]jsonValue{}
			archives := map[string][]byte{}
			for _, row := range wholeRows {
				name, ok := row["ImportPath"].(string)
				if !ok || byPath[name] != nil {
					t.Fatal("whole native output has an invalid or duplicate package")
				}
				byPath[name] = row
				if path, ok := row["Export"].(string); ok && path != "" {
					original, err := os.ReadFile(path)
					if err != nil {
						t.Fatal("original native compiler export unavailable", name, err)
					}
					archives[name] = original
				}
			}
			if len(archives) != 182 {
				t.Fatal("whole original native export closure changed")
			}
			retainPracticalSyntaxEvidence(t, "native-whole-cold.json", map[string]any{"stdout": whole.output, "stderr": whole.diagnostics})
			warm, err := runPracticalGoCandidatePackageObservation(compiler.selection, filesystem, directory, discovery.mainSourceID, request)
			if err != nil {
				t.Fatal(err)
			}
			if err := warm.requireCandidatePackages(request, discovery.compilerSHA256, discovery.mainSourceID, warm.output); err != nil {
				t.Fatal(err)
			}
			warmMetadata, err := decodePracticalPackageExports(warm.output)
			if err != nil || validatePracticalExportRoots(warmMetadata, exports.requestedPackages) != nil || !bytes.Equal(warm.diagnostics, whole.diagnostics) {
				t.Fatal("warm whole observation lost its original selected roots or diagnostics", err)
			}
			warmRows, err := decodePracticalCompleteObjects(warm.output)
			if err != nil || len(warmRows) != len(byPath) {
				t.Fatal("warm whole observation lost original packages", err)
			}
			retainPracticalSyntaxEvidence(t, "native-whole-warm.json", map[string]any{"stdout": warm.output, "stderr": warm.diagnostics})
			staleChanges := []any{}
			warmByPath := map[string]map[string]jsonValue{}
			for _, row := range warmRows {
				name, ok := row["ImportPath"].(string)
				cold := byPath[name]
				if !ok || cold == nil || warmByPath[name] != nil {
					t.Fatal("warm native output crossed original package membership", name)
				}
				left, right := maps.Clone(cold), maps.Clone(row)
				delete(left, "Stale")
				delete(left, "StaleReason")
				delete(right, "Stale")
				delete(right, "StaleReason")
				if !reflect.DeepEqual(left, right) {
					t.Fatalf("whole native metadata changed beyond stale state for %s", name)
				}
				if !reflect.DeepEqual(cold["Stale"], row["Stale"]) || !reflect.DeepEqual(cold["StaleReason"], row["StaleReason"]) {
					staleChanges = append(staleChanges, map[string]any{"package": name, "cold_stale": cold["Stale"], "cold_reason": cold["StaleReason"], "warm_stale": row["Stale"], "warm_reason": row["StaleReason"]})
				}
				warmByPath[name] = row
			}
			byPath = warmByPath
			retainPracticalSyntaxEvidence(t, "native-stale-changes.json", map[string]any{"packages": staleChanges})
			t.Logf("cold-to-warm metadata changed stale state for %d original packages", len(staleChanges))
			seen := map[string]bool{}
			for _, selected := range exports.requestedPackages {
				batch, err := runPracticalGoObservation(compiler.selection, filesystem, directory, discovery.mainSourceID, practicalGoPackages, selected)
				if err != nil {
					t.Fatal(err)
				}
				if batch.requireCandidatePackages(request, discovery.compilerSHA256, discovery.mainSourceID, batch.output) == nil {
					t.Fatal("a single-batch observation acquired the complete request")
				}
				if !bytes.Equal(batch.diagnostics, whole.diagnostics) {
					t.Fatal("native batch diagnostics changed")
				}
				retainPracticalSyntaxEvidence(t, "native-batch-"+sha256Hex([]byte(selected))+".json", map[string]any{"selected": selected, "stdout": batch.output, "stderr": batch.diagnostics})
				metadata, err := decodePracticalPackageExports(batch.output)
				if err != nil || validatePracticalExportRoot(metadata, selected) != nil {
					t.Fatal("native batch lost its actual selected root", selected, err)
				}
				rows, err := decodePracticalCompleteObjects(batch.output)
				if err != nil {
					t.Fatal(err)
				}
				for _, row := range rows {
					name, ok := row["ImportPath"].(string)
					original := byPath[name]
					if !ok || original == nil {
						t.Fatalf("batch introduced an unobserved package %q", name)
					}
					seen[name] = true
					left, right := maps.Clone(original), maps.Clone(row)
					delete(left, "DepOnly")
					delete(left, "Match")
					delete(right, "DepOnly")
					delete(right, "Match")
					if !reflect.DeepEqual(left, right) {
						fields := map[string]bool{}
						for key := range left {
							if !reflect.DeepEqual(left[key], right[key]) {
								fields[key] = true
							}
						}
						for key := range right {
							if !reflect.DeepEqual(left[key], right[key]) {
								fields[key] = true
							}
						}
						different := make([]string, 0, len(fields))
						for key := range fields {
							different = append(different, key)
						}
						sort.Strings(different)
						t.Fatalf("native package %s changed fields %v across batches", name, different)
					}
					if before, exists := archives[name]; exists {
						path, ok := row["Export"].(string)
						after, err := os.ReadFile(path)
						if !ok || err != nil || !bytes.Equal(before, after) {
							t.Fatalf("native compiler export changed across batches for %s: %v", name, err)
						}
					}
				}
			}
			if len(seen) != len(byPath) {
				t.Fatalf("native batch union lost original packages: %d of %d", len(seen), len(byPath))
			}
		})
}
