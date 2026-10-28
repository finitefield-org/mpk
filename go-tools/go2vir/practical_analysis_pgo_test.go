//go:build go1.25 && (linux || darwin)

package main

import (
	"bytes"
	"debug/buildinfo"
	"encoding/hex"
	"encoding/json"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"strings"
	"syscall"
	"testing"
)

// Automatic PGO consults the real Go compiler's default profile. Keep every
// original source and add the exact SDK profile only in a restored copy.
func TestPracticalAnalysisCapturesDefaultPGO(t *testing.T) {
	frontend := restoreRegisteredAnalysisOriginal(t)
	profile, err := os.ReadFile(filepath.Join(runtime.GOROOT(), "src", "cmd", "compile", "default.pgo"))
	if err != nil || len(profile) == 0 {
		t.Fatalf("read the real Go compiler profile: %v", err)
	}
	if err := os.WriteFile(filepath.Join(frontend, "default.pgo"), profile, 0o644); err != nil {
		t.Fatal(err)
	}
	roots := practicalAnalysisTestRoots(t, frontend)
	session := practicalTestSession(t, roots)
	inputs, err := session.capturePracticalAnalysisForPGO(practicalModuleLocation{capability: roots[0].Capability}, practicalModuleLocation{capability: roots[1].Capability}, practicalAnalysisPGOAuto)
	if err != nil {
		t.Fatal(err)
	}
	snapshot, err := session.seal()
	if err != nil {
		t.Fatal(err)
	}
	wire, err := snapshot.snapshotInputs()
	if err != nil {
		t.Fatal(err)
	}
	if inputs.pgoMode != practicalAnalysisPGOAuto || len(inputs.sources) != 48 || len(inputs.vendor) != 133 {
		t.Fatal("original registered analysis files were narrowed")
	}
	var objects []practicalSourceObject
	for _, source := range inputs.buildSources() {
		objects = append(objects, source.sourceObject())
	}
	var rows []struct {
		Object  practicalSourceObject `json:"object"`
		RawFile string                `json:"raw_file"`
	}
	out := os.Getenv("MPK_GO_ANALYSIS_PGO_OUTPUT")
	if out != "" {
		if err := os.MkdirAll(filepath.Join(out, "original-bytes"), 0o700); err != nil {
			t.Fatal(err)
		}
		for _, source := range wire.sources {
			raw := source.sourceBytes()
			name := source.object.RawSHA256 + ".raw"
			file := filepath.Join(out, "original-bytes", name)
			old, readErr := os.ReadFile(file)
			if os.IsNotExist(readErr) {
				if err := os.WriteFile(file, raw, 0o600); err != nil {
					t.Fatal(err)
				}
			} else if readErr != nil || !bytes.Equal(old, raw) {
				t.Fatal("original byte identity differs")
			}
			rows = append(rows, struct {
				Object  practicalSourceObject `json:"object"`
				RawFile string                `json:"raw_file"`
			}{source.object, name})
		}
		if err := os.WriteFile(filepath.Join(out, "original-profile.raw"), profile, 0o600); err != nil {
			t.Fatal(err)
		}
	}
	plans := make(map[string]string)
	for _, mode := range []string{"auto", "off"} {
		args := []string{"build", "-n", "-mod=vendor", "-trimpath", "-buildvcs=false", "-ldflags=-buildid="}
		if mode == "off" {
			args = append(args, "-pgo=off")
		}
		args = append(args, "-o", filepath.Join(t.TempDir(), "go2vir"), ".")
		command := exec.Command(filepath.Join(runtime.GOROOT(), "bin", "go"), args...)
		command.Dir = frontend
		command.Env = []string{"PATH=" + filepath.Join(runtime.GOROOT(), "bin") + ":/usr/bin:/bin", "HOME=" + t.TempDir(), "GOROOT=" + runtime.GOROOT(), "GOCACHE=" + os.Getenv("GOCACHE"), "GOENV=off", "GOFLAGS=", "GOWORK=off", "GOPROXY=off", "GOSUMDB=off", "GOTOOLCHAIN=local", "CGO_ENABLED=0", "GOOS=linux", "GOARCH=amd64", "GOAMD64=v1"}
		result, err := command.CombinedOutput()
		if err != nil {
			t.Fatalf("Go %s build plan: %v\n%s", mode, err, result)
		}
		plans[mode] = string(result)
		if out != "" {
			if err := os.WriteFile(filepath.Join(out, "plan-"+mode+".log"), result, 0o600); err != nil {
				t.Fatal(err)
			}
		}
	}
	if !strings.Contains(plans["auto"], "default.pgo") || strings.Contains(plans["off"], "default.pgo") {
		t.Fatal("actual Go plans did not distinguish automatic profile use from -pgo=off")
	}
	var archiveIDs []string
	for _, module := range inputs.modules {
		archiveIDs = append(archiveIDs, module.archive.source.object.ID)
	}
	if out != "" {
		packet := map[string]any{"snapshot_hex": hex.EncodeToString(wire.canonical), "source_ids": snapshot.ids, "original_sources": rows, "manifest_source_id": inputs.vendorManifest.object.ID, "archive_source_ids": archiveIDs, "build_objects": objects, "profile_sha256": sha256Hex(profile), "profile_bytes": len(profile), "native_auto_plan_uses_profile": true, "native_off_plan_uses_profile": false, "original_frontend_go_files": len(inputs.sources), "original_vendor_files": len(inputs.vendor), "application_proof_claimed": false}
		raw, err := json.MarshalIndent(packet, "", "  ")
		if err != nil {
			t.Fatal(err)
		}
		if err := os.WriteFile(filepath.Join(out, "native.json"), append(raw, '\n'), 0o600); err != nil {
			t.Fatal(err)
		}
	}
	captured, err := snapshot.readSource(roots[0].Capability, "default.pgo", "auxiliary")
	if err != nil || !bytes.Equal(captured.sourceBytes(), profile) {
		t.Fatalf("automatic PGO profile was not captured exactly: %v", err)
	}
	found := false
	for _, object := range objects {
		if object.Path == "default.pgo" && object.ID == captured.object.ID {
			found = true
		}
	}
	if !found || len(wire.sources) != 197 || len(objects) != 185 {
		t.Fatalf("complete original build list does not retain its PGO input: objects=%d build=%d", len(wire.sources), len(objects))
	}
}

// Keep each complete native snapshot and every original source byte so that
// the independent Rust reader can check ignored entries without substitutes.
func retainPracticalPGOControl(t *testing.T, out string, snapshot practicalSourceSnapshot, inputs practicalAnalysisInputs) {
	t.Helper()
	wire, err := snapshot.snapshotInputs()
	if err != nil {
		t.Fatal(err)
	}
	if err := os.MkdirAll(filepath.Join(out, "original-bytes"), 0o700); err != nil {
		t.Fatal(err)
	}
	var rows []struct {
		Object  practicalSourceObject `json:"object"`
		RawFile string                `json:"raw_file"`
	}
	for _, source := range wire.sources {
		name := source.object.RawSHA256 + ".raw"
		file := filepath.Join(out, "original-bytes", name)
		old, readErr := os.ReadFile(file)
		if os.IsNotExist(readErr) {
			if err := os.WriteFile(file, source.sourceBytes(), 0o600); err != nil {
				t.Fatal(err)
			}
		} else if readErr != nil || !bytes.Equal(old, source.sourceBytes()) {
			t.Fatal("original byte identity differs")
		}
		rows = append(rows, struct {
			Object  practicalSourceObject `json:"object"`
			RawFile string                `json:"raw_file"`
		}{source.object, name})
	}
	var archives []string
	for _, module := range inputs.modules {
		archives = append(archives, module.archive.source.object.ID)
	}
	var objects []practicalSourceObject
	for _, source := range inputs.buildSources() {
		objects = append(objects, source.sourceObject())
	}
	packet := map[string]any{"snapshot_hex": hex.EncodeToString(wire.canonical), "source_ids": snapshot.ids, "original_sources": rows, "manifest_source_id": inputs.vendorManifest.object.ID, "archive_source_ids": archives, "build_objects": objects, "pgo_mode": "off"}
	raw, err := json.MarshalIndent(packet, "", "  ")
	if err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(filepath.Join(out, "native.json"), append(raw, '\n'), 0o600); err != nil {
		t.Fatal(err)
	}
}

func TestPracticalAnalysisPGOOffPreservesIgnoredProfiles(t *testing.T) {
	profile, err := os.ReadFile(filepath.Join(runtime.GOROOT(), "src", "cmd", "compile", "default.pgo"))
	if err != nil || len(profile) == 0 {
		t.Fatalf("read the actual Go compiler profile: %v", err)
	}
	for _, name := range []string{"regular", "directory", "symlink", "oversized", "absent"} {
		t.Run(name, func(t *testing.T) {
			frontend := restoreRegisteredAnalysisOriginal(t)
			file := filepath.Join(frontend, "default.pgo")
			switch name {
			case "regular":
				if err := os.WriteFile(file, profile, 0o644); err != nil {
					t.Fatal(err)
				}
			case "directory":
				if err := os.Mkdir(file, 0o755); err != nil {
					t.Fatal(err)
				}
			case "symlink":
				target := filepath.Join(t.TempDir(), "original-profile")
				if err := os.WriteFile(target, profile, 0o644); err != nil {
					t.Fatal(err)
				}
				if err := os.Symlink(target, file); err != nil {
					t.Fatal(err)
				}
			case "oversized":
				f, err := os.Create(file)
				if err != nil {
					t.Fatal(err)
				}
				if err := f.Truncate(maximumCandidateBytes + 1); err != nil {
					t.Fatal(err)
				}
				if err := f.Close(); err != nil {
					t.Fatal(err)
				}
			}
			roots := practicalAnalysisTestRoots(t, frontend)
			location := practicalModuleLocation{capability: roots[0].Capability}
			download := practicalModuleLocation{capability: roots[1].Capability}
			session := practicalTestSession(t, roots)
			inputs, err := session.capturePracticalAnalysis(location, download)
			if err != nil {
				t.Fatal(err)
			}
			snapshot, err := session.seal()
			if err != nil {
				t.Fatal(err)
			}
			wire, err := snapshot.snapshotInputs()
			if err != nil {
				t.Fatal(err)
			}
			if inputs.pgoMode != practicalAnalysisPGOOff || inputs.defaultPGO != nil || len(inputs.sources) != 48 || len(inputs.vendor) != 133 || len(inputs.buildSources()) != 184 || len(wire.sources) != 196 {
				t.Fatal("PGO off must preserve every original input and ignore the extra entry")
			}
			if out := os.Getenv("MPK_GO_ANALYSIS_PGO_OUTPUT"); out != "" {
				control := filepath.Join(out, "controls", name)
				retainPracticalPGOControl(t, control, snapshot, inputs)
				if name == "oversized" {
					raw, err := os.ReadFile(file)
					if err != nil || len(raw) != maximumCandidateBytes+1 {
						t.Fatal("supplemental oversized input differs")
					}
					if err := os.WriteFile(filepath.Join(control, "original-ignored-profile.raw"), raw, 0o600); err != nil {
						t.Fatal(err)
					}
				}
			}
			autoSession := practicalTestSession(t, roots)
			auto, autoErr := autoSession.capturePracticalAnalysisForPGO(location, download, practicalAnalysisPGOAuto)
			switch name {
			case "regular":
				if autoErr != nil || auto.defaultPGO == nil || !bytes.Equal(auto.defaultPGO.sourceBytes(), profile) || len(auto.buildSources()) != 185 {
					t.Fatalf("automatic profile input: %v", autoErr)
				}
			case "absent":
				if autoErr != nil || auto.defaultPGO != nil || len(auto.buildSources()) != 184 {
					t.Fatalf("automatic build with no profile: %v", autoErr)
				}
			default:
				if autoErr == nil || autoSession.failed == nil {
					t.Fatal("automatic mode must reject an uncaptured or nonregular profile")
				}
			}
		})
	}
}

func TestPracticalAnalysisActualBuildInfoPGO(t *testing.T) {
	root := filepath.Join("..", "..", "develop", "migrations", "go-practical", "t02-w01", "verification", "analysis-pgo", "actual-build-original")
	for _, tc := range []struct {
		name     string
		accepted bool
	}{{"auto", false}, {"off", true}} {
		encoded, err := os.ReadFile(filepath.Join(root, tc.name+".elf.hex"))
		if err != nil {
			t.Fatal(err)
		}
		raw, err := hex.DecodeString(string(encoded))
		if err != nil {
			t.Fatal(err)
		}
		info, err := buildinfo.Read(bytes.NewReader(raw))
		if err != nil {
			t.Fatal(err)
		}
		if got := validatePracticalAnalysisBuildInfo(info, "linux_amd64") == nil; got != tc.accepted {
			t.Fatalf("actual %s binary build info accepted=%v, want %v", tc.name, got, tc.accepted)
		}
	}
}

// This supplemental root reader models an unrelated entry whose metadata
// cannot be read. It does not replace any original source or profile bytes.
type practicalUnreadablePGOSiblingRoot struct{ practicalRootReader }

func (root practicalUnreadablePGOSiblingRoot) Lstat(name string) (os.FileInfo, error) {
	if name == "unreadable.extra" {
		return nil, syscall.EACCES
	}
	return root.practicalRootReader.Lstat(name)
}

func TestPracticalAnalysisPGOIgnoresUnreadableSibling(t *testing.T) {
	frontend := restoreRegisteredAnalysisOriginal(t)
	profile, err := os.ReadFile(filepath.Join(runtime.GOROOT(), "src", "cmd", "compile", "default.pgo"))
	if err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(filepath.Join(frontend, "default.pgo"), profile, 0o644); err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(filepath.Join(frontend, "unreadable.extra"), []byte("additional ignored asset\n"), 0o644); err != nil {
		t.Fatal(err)
	}
	roots := practicalAnalysisTestRoots(t, frontend)
	session := practicalTestSession(t, roots)
	session.openRoot = func(name string) (practicalRootReader, error) {
		root, err := openPracticalSourceRoot(name)
		if err != nil {
			return nil, err
		}
		return practicalUnreadablePGOSiblingRoot{root}, nil
	}
	inputs, err := session.capturePracticalAnalysisForPGO(practicalModuleLocation{capability: roots[0].Capability}, practicalModuleLocation{capability: roots[1].Capability}, practicalAnalysisPGOAuto)
	if err != nil {
		t.Fatal(err)
	}
	snapshot, err := session.seal()
	if err != nil {
		t.Fatal(err)
	}
	wire, err := snapshot.snapshotInputs()
	if err != nil {
		t.Fatal(err)
	}
	if inputs.defaultPGO == nil || !bytes.Equal(inputs.defaultPGO.sourceBytes(), profile) || len(wire.sources) != 197 || len(inputs.buildSources()) != 185 {
		t.Fatal("unrelated unreadable metadata changed the complete original capture")
	}
}
