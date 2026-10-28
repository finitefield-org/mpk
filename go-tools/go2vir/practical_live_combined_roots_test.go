//go:build go1.25 && (linux || darwin)

package main

import (
	"fmt"
	"os"
	"path/filepath"
	"testing"
)

// Check the actual path, including a dangling symbolic link, before claiming
// that the original export cache has been retired.
func practicalLiveCombinedExportRetired(path string) error {
	if !filepath.IsAbs(path) {
		return fmt.Errorf("original export cache path is not absolute")
	}
	if _, err := os.Lstat(path); os.IsNotExist(err) {
		return nil
	} else if err != nil {
		return fmt.Errorf("cannot inspect original export cache: %w", err)
	}
	return fmt.Errorf("original export cache still exists")
}

// Coordinate the existing complete original owner with an independent test
// observer that has retained both root sets since their live comparisons.
// This hook is absent from normal capture and provides no launcher authority.
func practicalLiveCombinedOwnerFinish(t *testing.T, exportCache string, sources, exports, total int) {
	t.Helper()
	directory := os.Getenv("MPK_GO_LIVE_COMBINED_COORD")
	if directory == "" {
		return
	}
	evidence := os.Getenv("MPK_GO_PRACTICAL_SYNTAX_EVIDENCE")
	if !filepath.IsAbs(directory) || !filepath.IsAbs(evidence) || sources == 0 || exports == 0 || sources+exports != total {
		t.Fatal("combined root observer requires the complete original owner")
	}
	if err := practicalLiveCombinedExportRetired(exportCache); err != nil {
		t.Fatal(err)
	}
	practicalLiveRootSignal(t, directory, "owner-ready.json", map[string]any{
		"owner_path":             filepath.Join(evidence, "complete-original-module-graph-inputs.json"),
		"source_directory":       evidence,
		"export_cache_directory": exportCache,
		"source_objects":         sources, "export_objects": exports, "total_objects": total,
		"original_roots_retired": true, "export_cache_retired": true,
	})
	matched := practicalLiveRootWait(t, directory, "owner-matched.json")
	if matched["source_objects"] != float64(sources) || matched["export_objects"] != float64(exports) || matched["total_objects"] != float64(total) || matched["both_root_sets_retained"] != true || matched["export_cache_retirement_checked_before_binding"] != true {
		t.Fatal("combined observer did not bind every original source and export object")
	}
	t.Log("both independently held original root sets match the complete original export owner")
}

func TestPracticalLiveCombinedExportRetirement(t *testing.T) {
	root := t.TempDir()
	missing := filepath.Join(root, "missing")
	if err := practicalLiveCombinedExportRetired(missing); err != nil {
		t.Fatal(err)
	}
	file := filepath.Join(root, "file")
	if err := os.WriteFile(file, []byte("original"), 0o600); err != nil {
		t.Fatal(err)
	}
	directory := filepath.Join(root, "directory")
	if err := os.Mkdir(directory, 0o700); err != nil {
		t.Fatal(err)
	}
	link := filepath.Join(root, "link")
	if err := os.Symlink(missing, link); err != nil {
		t.Fatal(err)
	}
	for _, path := range []string{"relative", file, directory, link} {
		if err := practicalLiveCombinedExportRetired(path); err == nil {
			t.Fatalf("existing or relative export cache accepted: %q", path)
		}
	}
}
