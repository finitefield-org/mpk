//go:build go1.25 && (linux || darwin)

package main

import (
	"archive/zip"
	"bytes"
	"path/filepath"
	"testing"
)

func TestPracticalArchiveReturnedValuesCannotMutateSessionCache(t *testing.T) {
	for _, cached := range []bool{false, true} {
		for _, mutation := range []string{"member-map", "raw-bytes"} {
			name := "first/" + mutation
			if cached {
				name = "cached/" + mutation
			}
			t.Run(name, func(t *testing.T) {
				roots, _ := practicalTestCaptureRoots(t)
				files := practicalArchiveFixtureFiles(t)
				raw := practicalArchiveTestZIP(t, practicalArchiveTestEntries(practicalArchiveFixtureIdentity, files), zip.Store)
				writePracticalTestFile(t, filepath.Join(roots[0].Directory, "go.mod"), []byte("module example.com/app\ngo 1.25.0\n"), 0o600)
				writePracticalTestFile(t, filepath.Join(roots[0].Directory, "go.sum"), []byte(practicalArchiveFixtureIdentity.Path+" "+practicalArchiveFixtureIdentity.Version+" "+practicalArchiveFixtureSum+"\n"), 0o600)
				writePracticalTestFile(t, filepath.Join(roots[0].Directory, "module.zip"), raw, 0o600)
				session := practicalTestSession(t, roots)
				main, err := session.captureModule(practicalModuleLocation{capability: roots[0].Capability}, true)
				if err != nil {
					t.Fatal(err)
				}
				capture := func() practicalModuleArchive {
					archive, err := session.captureModuleArchive(roots[0].Capability, "module.zip", practicalArchiveFixtureIdentity, *main.sums)
					if err != nil {
						t.Fatal(err)
					}
					return archive
				}
				returned := capture()
				originalObject := returned.source.sourceObject()
				originalMembers := returned.memberInventory()
				if cached {
					returned = capture()
				}
				counters := session.counters
				changed := append(bytes.Clone(files[1].data), []byte("\n// caller-owned changed dependency\n")...)
				wrong, err := newPracticalSourceBuffer("unpacked", "dep.go", "go_file", changed)
				if err != nil {
					t.Fatal(err)
				}
				if mutation == "member-map" {
					returned.members["dep.go"] = practicalArchiveMember{Path: "dep.go", Bytes: uint64(len(changed)), RawSHA256: wrong.sourceObject().RawSHA256}
					delete(returned.members, "go.mod")
				} else {
					returned.source.data[0] ^= 0xff
				}
				fresh := capture()
				if fresh.verifyContent("dep.go", wrong) == nil {
					t.Fatal("mutating a returned archive authorized changed content through the session cache")
				}
				if fresh.source.sourceObject() != originalObject || !bytes.Equal(fresh.source.sourceBytes(), raw) {
					t.Fatal("returned archive bytes alias the original cached/captured input")
				}
				got := fresh.memberInventory()
				if len(got) != len(originalMembers) {
					t.Fatal("returned archive member deletion changed the cached original namespace")
				}
				for i := range got {
					if got[i] != originalMembers[i] {
						t.Fatal("returned archive metadata aliases the original cached inventory")
					}
				}
				if session.counters != counters {
					t.Fatal("owning repeated archive results changed capture budget charges")
				}
				snapshot, err := session.seal()
				if err != nil {
					t.Fatal(err)
				}
				stored, ok := snapshot.sourceBytes(originalObject.ID)
				if !ok || !bytes.Equal(stored, raw) {
					t.Fatal("a returned archive mutation reached the sealed original ZIP")
				}
			})
		}
	}
}
