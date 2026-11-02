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

// Optional test coordination opens the actual generated export cache before
// Go captures it and matches the full sealed input while that root is live.
// The observer receives copied snapshot inputs, never the capture owner's maps.
func practicalLiveExportRootObserver(t *testing.T) *practicalExportCaptureObserver {
	t.Helper()
	directory := os.Getenv("MPK_GO_LIVE_EXPORT_ROOT_COORD")
	if directory == "" {
		return nil
	}
	if !filepath.IsAbs(directory) {
		t.Fatal("export root coordination path must be absolute")
	}
	return &practicalExportCaptureObserver{
		before: func(root practicalSourceRoot) error {
			if root.Capability != "original-go-package-exports" || !filepath.IsAbs(root.Directory) {
				return fmt.Errorf("generated export root is invalid")
			}
			practicalLiveRootSignal(t, directory, "roots-ready.json", map[string]any{"roots": []practicalSourceRoot{root}, "capture_started": false, "compiler_output_prepared": true})
			opened := practicalLiveRootWait(t, directory, "opened.json")
			if opened["capabilities"] != float64(1) || opened["opened_before_capture"] != true {
				return fmt.Errorf("export observer did not hold its root before capture")
			}
			return nil
		},
		after: func(inputs practicalSnapshotInputs) error {
			evidence := os.Getenv("MPK_GO_PRACTICAL_SYNTAX_EVIDENCE")
			if evidence == "" {
				return fmt.Errorf("original export buffers have no evidence directory")
			}
			root := filepath.Join(evidence, "original-source-objects")
			if err := os.MkdirAll(root, 0o700); err != nil {
				return err
			}
			for _, source := range inputs.sources {
				if source.object.Capability != "original-go-package-exports" || source.object.Kind != "export_data" {
					return fmt.Errorf("export observer received a crossed source object")
				}
				metadata, err := json.MarshalIndent(source.object, "", "  ")
				if err != nil {
					return err
				}
				for extension, data := range map[string][]byte{"raw": source.sourceBytes(), "json": append(metadata, '\n')} {
					file := filepath.Join(root, source.object.ID+"."+extension)
					previous, err := os.ReadFile(file)
					if err == nil {
						if !bytes.Equal(previous, data) {
							return fmt.Errorf("original export byte identity differs")
						}
						continue
					}
					if !os.IsNotExist(err) {
						return err
					}
					if err := os.WriteFile(file, data, 0o600); err != nil {
						return err
					}
				}
			}
			var wire struct {
				SnapshotSHA256 string `json:"snapshot_sha256"`
			}
			if err := json.Unmarshal(inputs.canonical, &wire); err != nil || wire.SnapshotSHA256 == "" {
				return fmt.Errorf("export snapshot identity is unavailable")
			}
			packet := practicalSnapshotInputEvidence(inputs, false)
			packet["source_directory"] = root
			packet["original_source_objects"] = len(inputs.sources)
			packet["native_snapshot_sha256"] = wire.SnapshotSHA256
			packet["copied_input_mutation_probe"] = os.Getenv("MPK_GO_LIVE_EXPORT_MUTATE_COPY") == "1"
			practicalLiveRootSignal(t, directory, "snapshot-ready.json", packet)
			matched := practicalLiveRootWait(t, directory, "matched.json")
			if matched["original_source_objects"] != float64(len(inputs.sources)) || matched["native_snapshot_sha256"] != wire.SnapshotSHA256 || matched["opened_before_capture"] != true {
				return fmt.Errorf("independent observer crossed the complete original export capture")
			}
			if os.Getenv("MPK_GO_LIVE_EXPORT_MUTATE_COPY") == "1" {
				if len(inputs.canonical) == 0 || len(inputs.sources) == 0 || len(inputs.sources[0].data) == 0 {
					return fmt.Errorf("copied export input mutation probe has no bytes")
				}
				inputs.canonical[0] ^= 1
				inputs.sources[0].data[0] ^= 1
				t.Log("independent observer mutated only copied export inputs")
			}
			return nil
		},
	}
}
