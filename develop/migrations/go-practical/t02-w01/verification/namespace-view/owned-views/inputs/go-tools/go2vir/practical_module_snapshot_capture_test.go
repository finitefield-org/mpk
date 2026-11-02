//go:build go1.25 && (linux || darwin)

package main

import (
	"archive/zip"
	"bytes"
	"io"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"golang.org/x/mod/module"
	"golang.org/x/mod/sumdb/dirhash"
)

type practicalModuleSnapshotFixture struct {
	roots    []practicalSourceRoot
	location practicalModuleLocation
	identity module.Version
	prefix   string
	files    map[string][]byte
	mod      []byte
	sum      string
}

func preparePracticalModuleSnapshotFixture(t *testing.T, identity module.Version, legacy bool) practicalModuleSnapshotFixture {
	t.Helper()
	roots, _ := practicalTestCaptureRoots(t)
	files := practicalArchiveFixtureFiles(t)
	files[0].data = []byte("module " + identity.Path + "\n\ngo 1.22\n")
	mod := files[0].data
	if legacy {
		mod = []byte("module " + identity.Path + "\n")
		files = files[1:]
	}
	raw := practicalArchiveTestZIP(t, practicalArchiveTestEntries(identity, files), zip.Deflate)
	sum := practicalArchiveTestSum(t, raw)
	escaped, err := module.EscapePath(identity.Path)
	if err != nil {
		t.Fatal(err)
	}
	version, err := module.EscapeVersion(identity.Version)
	if err != nil {
		t.Fatal(err)
	}
	return practicalModuleSnapshotFixture{roots: roots, location: practicalModuleLocation{capability: roots[1].Capability, directory: "cache/download"}, identity: identity,
		prefix: filepath.Join("cache", "download", escaped, "@v", version), mod: mod, sum: sum,
		files: map[string][]byte{".mod": mod, ".info": []byte(`{"Version":"` + identity.Version + `","Time":"2020-01-02T03:04:05Z"}`), ".zip": raw, ".ziphash": []byte(" \t" + sum + "\r\n")}}
}

func capturePracticalModuleSnapshotFixture(t *testing.T, fixture practicalModuleSnapshotFixture, manifestSumBytes []byte) (*practicalCaptureSession, practicalModuleSums) {
	t.Helper()
	manifestSum, err := dirhash.Hash1([]string{"go.mod"}, func(string) (io.ReadCloser, error) { return io.NopCloser(bytes.NewReader(manifestSumBytes)), nil })
	if err != nil {
		t.Fatal(err)
	}
	sums := fixture.identity.Path + " " + fixture.identity.Version + " " + fixture.sum + "\n" + fixture.identity.Path + " " + fixture.identity.Version + "/go.mod " + manifestSum + "\n"
	writePracticalTestFile(t, filepath.Join(fixture.roots[0].Directory, "go.mod"), []byte("module example.com/app\ngo 1.25.0\n"), 0o600)
	writePracticalTestFile(t, filepath.Join(fixture.roots[0].Directory, "go.sum"), []byte(sums), 0o600)
	for suffix, raw := range fixture.files {
		writePracticalTestFile(t, filepath.Join(fixture.roots[1].Directory, fixture.prefix+suffix), raw, 0o600)
	}
	session := practicalTestSession(t, fixture.roots)
	main, err := session.captureModule(practicalModuleLocation{capability: fixture.roots[0].Capability}, true)
	if err != nil {
		t.Fatal(err)
	}
	return session, *main.sums
}

func TestPracticalModuleSnapshotRetainsExactVersionedDownloadInputs(t *testing.T) {
	for _, identity := range []module.Version{practicalArchiveFixtureIdentity, {Path: "example.com/Upper/Dep/v2", Version: "v2.0.0"}, {Path: "example.com/legacy", Version: "v2.0.0+incompatible"}} {
		t.Run(identity.Path, func(t *testing.T) {
			fixture := preparePracticalModuleSnapshotFixture(t, identity, strings.HasSuffix(identity.Version, "+incompatible"))
			session, sums := capturePracticalModuleSnapshotFixture(t, fixture, fixture.mod)
			captured, err := session.captureVersionedModule(fixture.location, identity, sums)
			if err != nil {
				t.Fatal(err)
			}
			if captured.identity != identity || captured.manifest.file.Module.Mod.Path != identity.Path || captured.archive.sum != fixture.sum {
				t.Fatal("versioned source identities or literal original module path changed")
			}
			counters := session.counters
			if _, err := session.captureVersionedModule(fixture.location, identity, sums); err != nil || session.counters != counters {
				t.Fatal("repeated versioned metadata reads changed the capture inventory/budget")
			}
			snapshot, err := session.seal()
			if err != nil {
				t.Fatal(err)
			}
			for suffix, source := range map[string]practicalSourceBuffer{".mod": captured.manifest.source, ".info": captured.info, ".ziphash": captured.zipHash, ".zip": captured.archive.source} {
				raw, exists := snapshot.sourceBytes(source.object.ID)
				if !exists || !bytes.Equal(raw, fixture.files[suffix]) {
					t.Fatal("original download bytes, whitespace or source object omitted/rewritten")
				}
			}
		})
	}
}

func TestPracticalModuleSnapshotRetainsLegacyArchiveWithoutInsertedManifest(t *testing.T) {
	fixture := preparePracticalModuleSnapshotFixture(t, practicalArchiveFixtureIdentity, true)
	session, sums := capturePracticalModuleSnapshotFixture(t, fixture, fixture.mod)
	captured, err := session.captureVersionedModule(fixture.location, fixture.identity, sums)
	if err != nil {
		t.Fatal(err)
	}
	if _, inserted := captured.archive.members["go.mod"]; inserted || !bytes.Equal(captured.manifest.source.sourceBytes(), fixture.mod) {
		t.Fatal("legacy ZIP was rewritten to manufacture a root manifest")
	}
	if _, err := session.seal(); err != nil {
		t.Fatal(err)
	}
}

func TestPracticalModuleSnapshotRejectsMissingStaleOrInconsistentOriginals(t *testing.T) {
	for _, name := range []string{"missing-mod", "missing-info", "missing-ziphash", "missing-zip", "stale-mod-sum", "inconsistent-zip-manifest", "wrong-info-version", "stale-ziphash", "newer-module-patch"} {
		t.Run(name, func(t *testing.T) {
			fixture := preparePracticalModuleSnapshotFixture(t, practicalArchiveFixtureIdentity, false)
			sumBytes := fixture.mod
			if strings.HasPrefix(name, "missing-") {
				delete(fixture.files, "."+strings.TrimPrefix(name, "missing-"))
			}
			switch name {
			case "stale-mod-sum", "inconsistent-zip-manifest":
				fixture.files[".mod"] = append(bytes.Clone(fixture.mod), '\n')
				if name == "inconsistent-zip-manifest" {
					sumBytes = fixture.files[".mod"] // Both h1 values are individually correct.
				}
			case "wrong-info-version":
				fixture.files[".info"] = []byte(`{"Version":"v1.0.1"}`)
			case "stale-ziphash":
				fixture.files[".ziphash"] = []byte("h1:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=")
			case "newer-module-patch":
				fixture.files[".mod"] = []byte("module example.com/pracdep\ngo 1.25.1\n")
				sumBytes = fixture.files[".mod"]
			}
			session, sums := capturePracticalModuleSnapshotFixture(t, fixture, sumBytes)
			if _, err := session.captureVersionedModule(fixture.location, fixture.identity, sums); err == nil || session.failed == nil {
				t.Fatal("missing/stale/inconsistent original module snapshot accepted")
			}
			if _, err := session.seal(); err == nil {
				t.Fatal("ignored versioned capture error produced a snapshot")
			}
			if strings.HasPrefix(name, "missing-") {
				_, err := os.Lstat(filepath.Join(fixture.roots[1].Directory, fixture.prefix+"."+strings.TrimPrefix(name, "missing-")))
				if !os.IsNotExist(err) {
					t.Fatal("capture repaired or created missing offline metadata")
				}
			}
		})
	}
}

func TestPracticalModuleSnapshotUsesOriginalSumsAndRejectsLocationAliases(t *testing.T) {
	for _, name := range []string{"location-alias", "forged-manifest-sum"} {
		t.Run(name, func(t *testing.T) {
			fixture := preparePracticalModuleSnapshotFixture(t, practicalArchiveFixtureIdentity, false)
			session, sums := capturePracticalModuleSnapshotFixture(t, fixture, fixture.mod)
			if name == "location-alias" {
				fixture.location.directory = "cache/../cache/download"
			} else {
				// A source that actually has the original recorded checksum can
				// survive forged parsed metadata: only original bytes are authority.
				sums.entries[module.Version{Path: fixture.identity.Path, Version: fixture.identity.Version + "/go.mod"}] = "caller mutation"
				if _, err := session.captureVersionedModule(fixture.location, fixture.identity, sums); err != nil {
					t.Fatal("caller parsed map replaced original checksum authority")
				}
				return
			}
			if _, err := session.captureVersionedModule(fixture.location, fixture.identity, sums); err == nil {
				t.Fatal("cache location alias accepted")
			}
		})
	}
}
