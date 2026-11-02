//go:build go1.25 && (linux || darwin)

package main

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"testing"
	"time"
)

// The optional local observer holds these exact test roots before the existing
// original capture begins. It is test instrumentation, not launcher authority.
func practicalLiveRootSignal(t *testing.T, directory, name string, value any) {
	t.Helper()
	raw, err := json.MarshalIndent(value, "", "  ")
	if err != nil {
		t.Fatal(err)
	}
	file, err := os.CreateTemp(directory, ".root-witness-")
	if err != nil {
		t.Fatal(err)
	}
	temporary := file.Name()
	defer os.Remove(temporary)
	if _, err = file.Write(append(raw, '\n')); err != nil {
		file.Close()
		t.Fatal(err)
	}
	if err = file.Close(); err != nil {
		t.Fatal(err)
	}
	if err = os.Link(temporary, filepath.Join(directory, name)); err != nil {
		t.Fatal(err)
	}
}

func practicalLiveRootWait(t *testing.T, directory, name string) map[string]any {
	t.Helper()
	deadline := time.Now().Add(30 * time.Minute)
	for time.Now().Before(deadline) {
		raw, err := os.ReadFile(filepath.Join(directory, name))
		if err == nil {
			var result map[string]any
			if err := json.Unmarshal(raw, &result); err != nil || result["status"] != "passed" {
				t.Fatalf("independent root observer rejected %s: %s (%v)", name, raw, err)
			}
			return result
		}
		if !os.IsNotExist(err) {
			t.Fatal(err)
		}
		time.Sleep(200 * time.Millisecond)
	}
	t.Fatalf("independent root observer did not produce %s", name)
	return nil
}

func practicalLiveRootComparisonStart(t *testing.T, roots []practicalSourceRoot) {
	t.Helper()
	directory := os.Getenv("MPK_GO_LIVE_ROOT_COORD")
	if directory == "" {
		return
	}
	if !filepath.IsAbs(directory) || len(roots) != 3 {
		t.Fatal("live comparison requires the full three original roots")
	}
	practicalLiveRootSignal(t, directory, "roots-ready.json", map[string]any{"roots": roots, "capture_started": false})
	opened := practicalLiveRootWait(t, directory, "opened.json")
	if opened["capabilities"] != float64(len(roots)) {
		t.Fatal("observer did not retain every original root")
	}
	t.Log("independent root handles opened before original capture")
}

func practicalLiveRootComparisonFinish(t *testing.T, snapshot practicalSourceSnapshot) {
	t.Helper()
	directory := os.Getenv("MPK_GO_LIVE_ROOT_COORD")
	if directory == "" {
		return
	}
	input, err := snapshot.snapshotInputs()
	if err != nil || len(input.sources) < 5480 {
		t.Fatal("original application and complete SDK snapshot unavailable", err)
	}
	evidence := os.Getenv("MPK_GO_PRACTICAL_SYNTAX_EVIDENCE")
	if evidence == "" {
		t.Fatal("original snapshot bytes have no evidence directory")
	}
	packet := practicalSnapshotInputEvidence(input, false)
	packet["source_directory"] = filepath.Join(evidence, "original-source-objects")
	packet["original_source_objects"] = len(input.sources)
	packet["native_snapshot_sha256"] = snapshot.contentSHA256
	practicalLiveRootSignal(t, directory, "snapshot-ready.json", packet)
	matched := practicalLiveRootWait(t, directory, "matched.json")
	if matched["original_source_objects"] != float64(len(input.sources)) || matched["native_snapshot_sha256"] != snapshot.contentSHA256 || matched["opened_before_capture"] != true {
		t.Fatal(fmt.Errorf("independent observer crossed the complete original capture: %v", matched))
	}
	t.Logf("independent original roots match all %d captured objects", len(input.sources))
}
