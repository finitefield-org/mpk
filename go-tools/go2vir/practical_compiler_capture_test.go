//go:build go1.25 && (linux || darwin)

package main

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"runtime"
	"sort"
	"strings"
	"testing"
)

func practicalTestCompilerCandidate(t *testing.T, root string) launcherSelection {
	return practicalTestCompilerCandidateForBundle(t, root, "toolchain.go.practical.prepared.test.v1")
}

func practicalTestCompilerCandidateForBundle(t *testing.T, root, id string) launcherSelection {
	t.Helper()
	files, err := inventoryFilesFromRoot(root, "go")
	if err != nil {
		t.Fatal(err)
	}
	inventory := bundleInventory{Schema: bundleInventorySchema, Scope: inventoryScope{Kind: "toolchain_bundle", BundleID: id}, Files: files}
	digest, err := hashTypedCanonicalJSON(bundleContentDomain, inventory)
	if err != nil {
		t.Fatal(err)
	}
	candidate := launcherSelection{Toolchain: toolchainIdentity{BundleID: id, DistributionSHA256: digest}, ToolchainGoRootPath: root, ToolchainInventory: inventory}
	content := []inventoryFile{}
	for _, file := range files {
		if !file.Executable {
			content = append(content, file)
			continue
		}
		name := "go-tool-" + filepath.Base(file.Path)
		if file.Path == "go/bin/go" || file.Path == "go/bin/gofmt" {
			name = filepath.Base(file.Path)
		}
		candidate.ToolchainComponents = append(candidate.ToolchainComponents, candidateComponent{Identity: componentIdentity{Kind: "executable", Name: name, Release: practicalCompilerRelease, BinarySHA256: file.SHA256}, Inventory: bundleInventory{Schema: bundleInventorySchema, Scope: inventoryScope{Kind: "component", BundleID: id, ComponentName: name}, Files: []inventoryFile{file}}})
	}
	contentInventory := bundleInventory{Schema: bundleInventorySchema, Scope: inventoryScope{Kind: "component", BundleID: id, ComponentName: "go-target-linux-amd64"}, Files: content}
	digest, err = hashTypedCanonicalJSON(bundleContentDomain, contentInventory)
	if err != nil {
		t.Fatal(err)
	}
	candidate.ToolchainComponents = append(candidate.ToolchainComponents, candidateComponent{Identity: componentIdentity{Kind: "content", Name: "go-target-linux-amd64", Release: practicalCompilerRelease, ContentSHA256: digest}, Inventory: contentInventory})
	sort.Slice(candidate.ToolchainComponents, func(i, j int) bool {
		return candidate.ToolchainComponents[i].Identity.Name < candidate.ToolchainComponents[j].Identity.Name
	})
	for _, component := range candidate.ToolchainComponents {
		candidate.Toolchain.Components = append(candidate.Toolchain.Components, component.Identity)
	}
	return candidate
}

func practicalTestCompleteCompiler(t *testing.T) launcherSelection {
	t.Helper()
	if runtime.Version() != practicalCompilerRelease {
		t.Fatalf("full prepared image test needs exact %s", practicalCompilerRelease)
	}
	source, err := filepath.EvalSymlinks(runtime.GOROOT())
	if err != nil {
		t.Fatal(err)
	}
	base, err := filepath.EvalSymlinks(t.TempDir())
	if err != nil {
		t.Fatal(err)
	}
	root := filepath.Join(base, "go")
	hostTools := "pkg/tool/" + runtime.GOOS + "_" + runtime.GOARCH + "/"
	// Match the existing release assembler: omit Go's own tests/testdata and
	// retain every original implementation/support byte. Only actual Go tools
	// keep executable mode; source build/generation scripts remain inert data.
	err = filepath.Walk(source, func(filename string, info os.FileInfo, walkErr error) error {
		if walkErr != nil {
			return walkErr
		}
		relative, err := filepath.Rel(source, filename)
		if err != nil {
			return err
		}
		if info.Mode()&os.ModeSymlink != 0 || !info.IsDir() && !info.Mode().IsRegular() {
			return fmt.Errorf("test compiler distribution has a link or special file")
		}
		if info.IsDir() {
			if relative == "test" || info.Name() == "testdata" {
				return filepath.SkipDir
			}
			return os.MkdirAll(filepath.Join(root, relative), 0o700)
		}
		if strings.HasSuffix(info.Name(), "_test.go") {
			return nil
		}
		destination := filepath.Join(root, relative)
		if err := copyToolchainFixturePath(filename, destination); err != nil {
			return err
		}
		mode := os.FileMode(0o444)
		logical := filepath.ToSlash(relative)
		if logical == "bin/go" || logical == "bin/gofmt" || strings.HasPrefix(logical, hostTools) {
			mode = 0o555
		}
		return os.Chmod(destination, mode)
	})
	if err != nil {
		t.Fatal(err)
	}
	directories := []string{}
	if err := filepath.Walk(root, func(filename string, info os.FileInfo, err error) error {
		if err == nil && info.IsDir() {
			directories = append(directories, filename)
		}
		return err
	}); err != nil {
		t.Fatal(err)
	}
	for index := len(directories) - 1; index >= 0; index-- {
		entries, err := os.ReadDir(directories[index])
		if err != nil {
			t.Fatal(err)
		}
		if len(entries) == 0 {
			if err := os.Remove(directories[index]); err != nil {
				t.Fatal(err)
			}
		}
	}
	sealTestReleaseRoot(t, root)
	return practicalTestCompilerCandidate(t, root)
}

func clonePracticalCompilerTestCandidate(candidate launcherSelection) launcherSelection {
	candidate.Toolchain.Components = append([]componentIdentity(nil), candidate.Toolchain.Components...)
	candidate.ToolchainInventory.Files = append([]inventoryFile(nil), candidate.ToolchainInventory.Files...)
	candidate.ToolchainComponents = append([]candidateComponent(nil), candidate.ToolchainComponents...)
	for index := range candidate.ToolchainComponents {
		candidate.ToolchainComponents[index].Inventory.Files = append([]inventoryFile(nil), candidate.ToolchainComponents[index].Inventory.Files...)
	}
	return candidate
}

func TestPracticalCompilerValidatesActualPreparedImageAndRejectsReauthorizedChanges(t *testing.T) {
	candidate := practicalTestCompleteCompiler(t)
	image, err := validatePracticalCompilerImage(candidate)
	if err != nil {
		t.Fatal(err)
	}
	if image.host != runtime.GOOS+"_"+runtime.GOARCH || image.selection.GoExecutable != filepath.Join(candidate.ToolchainGoRootPath, "bin", "go") || len(image.selection.StandardPackagePaths) < 100 {
		t.Fatal("complete image lost original host, executable or standard-library inventory")
	}
	var total int64
	for _, file := range candidate.ToolchainInventory.Files {
		total += file.SizeBytes
	}
	t.Logf("actual prepared Go1.25.0 image: files=%d bytes=%d components=%d standard source directories=%d distribution=%s; no native Linux or practical context receipt", len(candidate.ToolchainInventory.Files), total, len(candidate.ToolchainComponents), len(image.selection.StandardPackagePaths), candidate.Toolchain.DistributionSHA256)
	if destination := os.Getenv("MPK_GO_PRACTICAL_COMPILER_EVIDENCE"); destination != "" {
		record := struct {
			Host       string               `json:"host"`
			Toolchain  toolchainIdentity    `json:"toolchain"`
			Inventory  bundleInventory      `json:"inventory"`
			Components []candidateComponent `json:"components"`
		}{image.host, candidate.Toolchain, candidate.ToolchainInventory, candidate.ToolchainComponents}
		data, err := json.MarshalIndent(record, "", "  ")
		if err != nil {
			t.Fatal(err)
		}
		file, err := os.OpenFile(destination, os.O_CREATE|os.O_EXCL|os.O_WRONLY, 0o600)
		if err != nil {
			t.Fatal(err)
		}
		_, writeErr := file.Write(append(data, '\n'))
		closeErr := file.Close()
		if writeErr != nil || closeErr != nil {
			t.Fatal("cannot retain the complete original prepared compiler inventory")
		}
	}
	t.Run("caller-inventory-alias", func(t *testing.T) {
		candidate.ToolchainInventory.Files[0].SHA256 = strings.Repeat("0", 64)
		candidate.Toolchain.Components[0].Release = "go1.25.1"
		candidate.ToolchainComponents[0].Inventory.Files[0].SHA256 = strings.Repeat("1", 64)
		if err := image.revalidate(); err != nil {
			t.Fatalf("caller mutation changed the retained original compiler inventories: %v", err)
		}
	})
	t.Run("self-consistent-newer-release", func(t *testing.T) {
		changed := clonePracticalCompilerTestCandidate(image.candidate)
		for index := range changed.ToolchainComponents {
			changed.ToolchainComponents[index].Identity.Release = "go1.25.1"
			changed.Toolchain.Components[index].Release = "go1.25.1"
		}
		if _, err := validatePracticalCompilerImage(changed); err == nil || !strings.Contains(err.Error(), "exactly Go1.25.0") {
			t.Fatal("a reauthorized newer component release crossed the frozen compiler pin")
		}
	})
	t.Run("mixed-host-inventory", func(t *testing.T) {
		changed := clonePracticalCompilerTestCandidate(image.candidate)
		other := "linux_amd64"
		if image.host == other {
			other = "darwin_arm64"
		}
		changed.ToolchainInventory.Files = append(changed.ToolchainInventory.Files, inventoryFile{Path: "go/pkg/tool/" + other + "/compile", Executable: true, SizeBytes: 1, SHA256: strings.Repeat("1", 64)})
		sort.Slice(changed.ToolchainInventory.Files, func(i, j int) bool {
			return changed.ToolchainInventory.Files[i].Path < changed.ToolchainInventory.Files[j].Path
		})
		if _, err := validatePracticalCompilerImage(changed); err == nil || !strings.Contains(err.Error(), "mixed") {
			t.Fatal("mixed compiler host inventories accepted")
		}
	})
	t.Run("rehashed-specification", func(t *testing.T) {
		filename := filepath.Join(image.selection.GoRoot, "src/go/types/version.go")
		original, err := os.ReadFile(filename)
		if err != nil {
			t.Fatal(err)
		}
		write := func(data []byte) {
			t.Helper()
			if err := os.Chmod(filename, 0o600); err != nil {
				t.Fatal(err)
			}
			if err := os.WriteFile(filename, data, 0o600); err != nil {
				t.Fatal(err)
			}
			if err := os.Chmod(filename, 0o444); err != nil {
				t.Fatal(err)
			}
		}
		defer write(original)
		write(append(append([]byte(nil), original...), '\n'))
		changed := practicalTestCompilerCandidate(t, image.selection.GoRoot)
		if _, err := validatePracticalCompilerImage(changed); err == nil || !strings.Contains(err.Error(), "source witness") {
			t.Fatal("rehashing every inventory authorized a changed original compiler/version witness")
		}
	})
	t.Run("rehashed-wrong-tool-binary", func(t *testing.T) {
		filename := filepath.Join(image.selection.GoRoot, "pkg/tool", image.host, "compile")
		original, err := os.ReadFile(filename)
		if err != nil {
			t.Fatal(err)
		}
		other, err := os.ReadFile(filepath.Join(image.selection.GoRoot, "bin/gofmt"))
		if err != nil {
			t.Fatal(err)
		}
		write := func(data []byte) {
			t.Helper()
			if err := os.Chmod(filename, 0o700); err != nil {
				t.Fatal(err)
			}
			if err := os.WriteFile(filename, data, 0o700); err != nil {
				t.Fatal(err)
			}
			if err := os.Chmod(filename, 0o555); err != nil {
				t.Fatal(err)
			}
		}
		defer write(original)
		write(other)
		changed := practicalTestCompilerCandidate(t, image.selection.GoRoot)
		if _, err := validatePracticalCompilerImage(changed); err == nil || !strings.Contains(err.Error(), "tool identity") {
			t.Fatal("complete freshly rehashed image replaced the compiler with gofmt")
		}
	})
	t.Run("partial-original-test-fixture", func(t *testing.T) {
		partial := computeCurrentToolchainCandidate(t)
		if partial.err != nil {
			t.Fatal(partial.err)
		}
		if _, err := validatePracticalCompilerImage(launcherSelection{Toolchain: partial.identity, ToolchainGoRootPath: partial.root, ToolchainInventory: partial.inventory, ToolchainComponents: partial.components}); err == nil {
			t.Fatal("the old partial SDK fixture became complete practical compiler authority")
		}
	})
}
