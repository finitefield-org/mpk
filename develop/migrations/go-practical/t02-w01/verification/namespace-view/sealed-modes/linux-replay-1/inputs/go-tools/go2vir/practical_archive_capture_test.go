//go:build go1.25 && (linux || darwin)

package main

import (
	"archive/zip"
	"bytes"
	"context"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"strings"
	"testing"
	"time"

	"golang.org/x/mod/module"
)

func TestPracticalArchiveCaptureSharesNamespaceBudgetAndOwnedInputs(t *testing.T) {
	for _, remaining := range []uint64{3, 4} {
		t.Run(map[uint64]string{3: "exceeds", 4: "inclusive"}[remaining], func(t *testing.T) {
			roots, _ := practicalTestCaptureRoots(t)
			raw := practicalArchiveTestZIP(t, practicalArchiveTestEntries(practicalArchiveFixtureIdentity, practicalArchiveFixtureFiles(t)), zip.Store)
			writePracticalTestFile(t, filepath.Join(roots[0].Directory, "go.mod"), []byte("module example.com/app\ngo 1.25.0\n"), 0o600)
			writePracticalTestFile(t, filepath.Join(roots[0].Directory, "go.sum"), []byte(practicalArchiveFixtureIdentity.Path+" "+practicalArchiveFixtureIdentity.Version+" "+practicalArchiveFixtureSum+"\n"), 0o600)
			writePracticalTestFile(t, filepath.Join(roots[0].Directory, "module.zip"), raw, 0o600)
			writePracticalTestFile(t, filepath.Join(roots[1].Directory, "module.zip"), raw, 0o600)
			session := practicalTestSession(t, roots)
			main, err := session.captureModule(practicalModuleLocation{capability: roots[0].Capability}, true)
			if err != nil {
				t.Fatal(err)
			}
			// Bind all physical namespaces/raw files first, then leave a small
			// remaining entry budget for actual archive namespace observations.
			for _, root := range roots {
				if _, err := session.readSource(root.Capability, "module.zip", "auxiliary"); err != nil {
					t.Fatal(err)
				}
			}
			session.counters.directoryEntries = maximumDirectoryEntries - remaining
			first, err := session.captureModuleArchive(roots[0].Capability, "module.zip", practicalArchiveFixtureIdentity, *main.sums)
			if err != nil || first.namespaceEntries != 2 {
				t.Fatalf("first actual archive: %v", err)
			}
			counters := session.counters
			// The caller's mutable parsed map is not checksum authority.
			main.sums.entries[practicalArchiveFixtureIdentity] = "caller mutation"
			if _, err := session.captureModuleArchive(roots[0].Capability, "module.zip", practicalArchiveFixtureIdentity, *main.sums); err != nil || session.counters != counters {
				t.Fatal("cached archive used caller metadata or double-charged an input")
			}
			second, err := session.captureModuleArchive(roots[1].Capability, "module.zip", practicalArchiveFixtureIdentity, *main.sums)
			if remaining == 3 {
				if err == nil || session.failed == nil {
					t.Fatal("archive namespace budget was reset per capability")
				}
				if _, err := session.seal(); err == nil {
					t.Fatal("ignored archive budget failure allowed sealing")
				}
				return
			}
			if err != nil || second.source.object.ID == first.source.object.ID || session.counters.directoryEntries != maximumDirectoryEntries {
				t.Fatal("inclusive cross-capability bound or original raw identity differs")
			}
			snapshot, err := session.seal()
			if err != nil {
				t.Fatal(err)
			}
			for _, archive := range []practicalModuleArchive{first, second} {
				got, exists := snapshot.sourceBytes(archive.source.object.ID)
				if !exists || !bytes.Equal(got, raw) {
					t.Fatal("original module archive bytes were rewritten or omitted")
				}
			}
		})
	}
}

func TestPracticalArchiveCaptureRejectsForgedOwnerAndUnavailableSnapshot(t *testing.T) {
	for _, name := range []string{"detached-owner", "forged-parsed-checksum", "missing-checksum", "missing-archive", "linked-archive", "crossed-cached-version"} {
		t.Run(name, func(t *testing.T) {
			roots, base := practicalTestCaptureRoots(t)
			raw := practicalArchiveTestZIP(t, practicalArchiveTestEntries(practicalArchiveFixtureIdentity, practicalArchiveFixtureFiles(t)), zip.Store)
			hash := practicalArchiveFixtureSum
			if name == "forged-parsed-checksum" {
				hash = "h1:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA="
			}
			sumBytes := []byte(practicalArchiveFixtureIdentity.Path + " " + practicalArchiveFixtureIdentity.Version + " " + hash + "\n")
			if name == "missing-checksum" {
				sumBytes = nil
			}
			writePracticalTestFile(t, filepath.Join(roots[0].Directory, "go.mod"), []byte("module example.com/app\ngo 1.25.0\n"), 0o600)
			writePracticalTestFile(t, filepath.Join(roots[0].Directory, "go.sum"), sumBytes, 0o600)
			if name == "linked-archive" {
				outside := filepath.Join(base, "outside.zip")
				writePracticalTestFile(t, outside, raw, 0o600)
				if err := os.Symlink(outside, filepath.Join(roots[0].Directory, "module.zip")); err != nil {
					t.Fatal(err)
				}
			} else if name != "missing-archive" {
				writePracticalTestFile(t, filepath.Join(roots[0].Directory, "module.zip"), raw, 0o600)
			}
			session := practicalTestSession(t, roots)
			main, err := session.captureModule(practicalModuleLocation{capability: roots[0].Capability}, true)
			if err != nil {
				t.Fatal(err)
			}
			sums := *main.sums
			if name == "detached-owner" {
				source, err := newPracticalSourceBuffer("detached", "go.sum", "go_sum", sumBytes)
				if err != nil {
					t.Fatal(err)
				}
				sums, err = parsePracticalModuleSums(source)
				if err != nil {
					t.Fatal(err)
				}
			}
			if name == "forged-parsed-checksum" {
				sums.entries[practicalArchiveFixtureIdentity] = practicalArchiveFixtureSum
			}
			identity := practicalArchiveFixtureIdentity
			if name == "crossed-cached-version" {
				if _, err := session.captureModuleArchive(roots[0].Capability, "module.zip", identity, sums); err != nil {
					t.Fatal(err)
				}
				identity.Version = "v1.0.1"
			}
			if _, err := session.captureModuleArchive(roots[0].Capability, "module.zip", identity, sums); err == nil || session.failed == nil {
				t.Fatal("forged owner/checksum or missing/crossed snapshot accepted")
			}
			if _, err := session.seal(); err == nil {
				t.Fatal("rejected archive transaction produced a source snapshot")
			}
		})
	}
}

func TestPracticalArchiveMatchesCurrentOfflineGoModuleVerification(t *testing.T) {
	if runtime.Version() != practicalCompilerRelease {
		t.Fatalf("exact compiler required for module checksum oracle: %s", runtime.Version())
	}
	files := practicalArchiveFixtureFiles(t)
	files[1].mode = os.ModeSymlink | 0o777
	raw := practicalArchiveTestZIP(t, practicalArchiveTestEntries(practicalArchiveFixtureIdentity, files), zip.Deflate)
	archive, err := parsePracticalModuleArchive(practicalArchiveFixtureIdentity, practicalArchiveTestBuffer(t, raw), practicalArchiveFixtureSum, 2)
	if err != nil {
		t.Fatal(err)
	}
	filesystem, err := newLoaderFilesystem()
	if err != nil {
		t.Fatal(err)
	}
	defer filesystem.Close()
	if err := os.Chmod(filesystem.goMod, 0o700); err != nil {
		t.Fatal(err)
	}
	escaped, err := module.EscapePath(practicalArchiveFixtureIdentity.Path)
	if err != nil {
		t.Fatal(err)
	}
	version := practicalArchiveFixtureIdentity.Version
	download := filepath.Join(filesystem.goMod, "cache", "download", escaped, "@v", version)
	unpacked := filepath.Join(filesystem.goMod, escaped+"@"+version)
	var modBytes, goBytes []byte
	for _, file := range files {
		writePracticalTestFile(t, filepath.Join(unpacked, file.name), file.data, 0o600)
		if file.name == "go.mod" {
			modBytes = file.data
		} else {
			goBytes = file.data
		}
	}
	writePracticalTestFile(t, download+".zip", raw, 0o600)
	writePracticalTestFile(t, download+".ziphash", []byte(practicalArchiveFixtureSum), 0o600)
	writePracticalTestFile(t, download+".mod", modBytes, 0o600)
	writePracticalTestFile(t, download+".info", []byte("{\"Version\":\"v1.0.0\",\"Time\":\"2020-01-02T03:04:05Z\"}\n"), 0o600)
	application := filepath.Join(filesystem.root, "original-app")
	mainMod := []byte("module example.com/app\n\ngo 1.25.0\nrequire example.com/pracdep v1.0.0\n")
	mainSums := []byte("example.com/pracdep v1.0.0 " + practicalArchiveFixtureSum + "\nexample.com/pracdep v1.0.0/go.mod h1:RNha+Ovptpv+50nN2NH8md8DuZZwwlLUzexiDm4kE8g=\n")
	writePracticalTestFile(t, filepath.Join(application, "go.mod"), mainMod, 0o600)
	writePracticalTestFile(t, filepath.Join(application, "go.sum"), mainSums, 0o600)
	selection := validatedLauncherSelection{GoRoot: runtime.GOROOT(), GoExecutable: filepath.Join(runtime.GOROOT(), "bin", "go")}
	for _, changed := range []bool{false, true} {
		if changed {
			goBytes = append(bytes.Clone(goBytes), '\n')
			writePracticalTestFile(t, filepath.Join(unpacked, "dep.go"), goBytes, 0o600)
		}
		original, err := newPracticalSourceBuffer("unpacked-original", "dep.go", "go_file", goBytes)
		if err != nil {
			t.Fatal(err)
		}
		contentErr := archive.verifyContent("dep.go", original)
		ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
		command := exec.CommandContext(ctx, selection.GoExecutable, "mod", "verify")
		command.Dir = application
		command.Env = pinnedLoaderEnvironment(selection, filesystem)
		output, goErr := command.CombinedOutput()
		cancel()
		if !changed && (goErr != nil || contentErr != nil || string(output) != "all modules verified\n") {
			t.Fatalf("original offline cache verification differs: %v / %v: %s", goErr, contentErr, output)
		}
		if changed && (goErr == nil || contentErr == nil || !strings.Contains(string(output), "dir has been modified")) {
			t.Fatalf("changed original cache was not rejected by both: %v / %v: %s", goErr, contentErr, output)
		}
		t.Logf("pinned offline Go module verification changed=%t: %s", changed, output)
	}
	for name, expected := range map[string][]byte{"go.mod": mainMod, "go.sum": mainSums} {
		actual, err := os.ReadFile(filepath.Join(application, name))
		if err != nil || !bytes.Equal(actual, expected) {
			t.Fatal("Go checksum oracle rewrote original application metadata")
		}
	}
}
