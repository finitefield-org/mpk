//go:build go1.25 && (linux || darwin)

package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"testing"
)

func TestPracticalReplayBaselineComparesWholeMetadataAndArchiveBytes(t *testing.T) {
	fixture := preparePracticalObservationFixture(t)
	if err := fixture.exports.bindOriginalSourceTypes(fixture.loader, fixture.discovery, fixture.compiler, fixture.roots[2].Capability); err != nil {
		t.Fatal(err)
	}
	image, err := materializePracticalReplaySources(t.TempDir(), []practicalReplaySources{fixture.sealed, fixture.exports.replay}, &fixture.compiler, fixture.roots[2].Capability)
	if err != nil {
		t.Fatal(err)
	}
	defer image.Close()
	for _, control := range []string{"identity", "canonical-transport", "module-membership", "module-field", "package-field", "unmodelled-root", "unknown-field", "cache-escape", "archive-bytes", "archive-missing", "module-diagnostics", "package-diagnostics"} {
		t.Run(control, func(t *testing.T) {
			baseline, err := image.originalReplayBaseline(fixture.discovery, fixture.exports)
			if err != nil {
				t.Fatal(err)
			}
			defer baseline.Close()
			cacheDirectory, err := filepath.EvalSymlinks(t.TempDir())
			if err != nil {
				t.Fatal(err)
			}
			modules, err := decodePracticalCompleteObjects(fixture.discovery.output)
			if err != nil {
				t.Fatal(err)
			}
			packages, err := decodePracticalCompleteObjects(fixture.exports.output)
			if err != nil {
				t.Fatal(err)
			}
			var actualFile string
			for _, row := range packages {
				name, _ := row["ImportPath"].(string)
				object, hasArchive := baseline.archives[name]
				if !hasArchive {
					continue
				}
				relative := object.Path
				actualFile = filepath.Join(cacheDirectory, filepath.FromSlash(relative))
				buffer, err := baseline.view.readCaptured(object.Capability, object.Path, object.Kind)
				if err != nil {
					t.Fatal(err)
				}
				writePracticalTestFile(t, actualFile, buffer.data, 0o600)
				row["Export"] = filepath.Join("/observed-derived-cache", filepath.FromSlash(relative))
			}
			var moduleDiagnostics, packageDiagnostics []byte
			switch control {
			case "module-membership":
				modules = append(modules, modules[0])
			case "module-field":
				modules[0]["GoVersion"] = "1.24"
			case "package-field":
				delete(packages[0], "GoFiles")
			case "unmodelled-root":
				if _, exists := packages[0]["Root"]; !exists {
					t.Fatal("original native Root field was not present for its full-metadata control")
				}
				packages[0]["Root"] = "/another-root"
			case "unknown-field":
				packages[0]["CallerExtra"] = true
			case "cache-escape":
				packages[0]["Export"] = "/outside-derived-cache/archive"
			case "archive-bytes":
				raw, err := os.ReadFile(actualFile)
				if err != nil {
					t.Fatal(err)
				}
				raw[len(raw)-1] ^= 1
				if err := os.WriteFile(actualFile, raw, 0o600); err != nil {
					t.Fatal(err)
				}
			case "archive-missing":
				if err := os.Remove(actualFile); err != nil {
					t.Fatal(err)
				}
			case "module-diagnostics":
				moduleDiagnostics = []byte("unexpected module diagnostic\n")
			case "package-diagnostics":
				packageDiagnostics = []byte("unexpected package diagnostic\n")
			}
			encode := func(rows []map[string]jsonValue) []byte {
				t.Helper()
				var output bytes.Buffer
				for _, row := range rows {
					if control == "canonical-transport" {
						raw, err := canonicalJSONValue(row)
						if err != nil {
							t.Fatal(err)
						}
						output.Write(raw)
						output.WriteByte('\n')
					} else if err := json.NewEncoder(&output).Encode(row); err != nil {
						t.Fatal(err)
					}
				}
				return output.Bytes()
			}
			moduleOutput, packageOutput := encode(modules), encode(packages)
			result, failure := baseline.compare(moduleOutput, packageOutput, moduleDiagnostics, packageDiagnostics,
				"/observed-derived-cache", practicalSourceRoot{Capability: "comparison-fixture-exports", Directory: cacheDirectory})
			if result.actual.contentSHA256 != "" {
				retainPracticalStandardSnapshot(t, result.actual)
			}
			retainPracticalSyntaxEvidence(t, "observation-comparison-"+control+".json", map[string]any{
				"module_output": moduleOutput, "package_output": packageOutput, "module_diagnostics": moduleDiagnostics, "package_diagnostics": packageDiagnostics,
				"modules": result.modules, "packages": result.packages, "exports": result.exports, "export_bytes": result.bytes,
				"actual_objects": result.actual.sourceObjects(), "error": fmt.Sprint(failure),
				"fixture_copies_original_archives": true, "independent_native_command_claimed": false,
			})
			positive := control == "identity" || control == "canonical-transport"
			if positive != (failure == nil) {
				t.Fatal("full original comparison verdict differs", control, failure)
			}
			if positive && (result.modules != 1 || result.packages != 1 || result.exports != 1) {
				t.Fatal("comparison lost an original complete object")
			}
			if failure != nil {
				if _, again := baseline.compare(fixture.discovery.output, fixture.exports.output, nil, nil, "/observed-derived-cache", practicalSourceRoot{Capability: "comparison-fixture-exports", Directory: cacheDirectory}); again != failure {
					t.Fatal("ignored comparison failure did not remain sticky", again)
				}
			}
			if err := image.Close(); err == nil {
				t.Fatal("image deletion escaped a live baseline lease")
			}
			if err := baseline.Close(); err != nil {
				t.Fatal(err)
			}
			if _, err := baseline.compare(nil, nil, nil, nil, "/observed-derived-cache", practicalSourceRoot{}); err == nil {
				t.Fatal("closed original baseline regained comparison authority")
			}
		})
	}
}
