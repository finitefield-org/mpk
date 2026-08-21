package main

import (
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"sort"
	"strings"
	"testing"
)

type testToolchainCandidate struct {
	root       string
	inventory  bundleInventory
	components []candidateComponent
	identity   toolchainIdentity
	err        error
}

func buildTestLauncherSelection(t *testing.T) (lowerRequest, launcherSelection) {
	t.Helper()
	toolchain := computeCurrentToolchainCandidate(t)
	if toolchain.err != nil {
		t.Fatalf("construct current toolchain candidate: %v", toolchain.err)
	}

	frontendRoot := t.TempDir()
	frontendExecutable := filepath.Join(frontendRoot, "go2vir")
	build := exec.Command(
		filepath.Join(runtime.GOROOT(), "bin", "go"),
		"build", "-mod=readonly", "-trimpath", "-buildvcs=false", "-ldflags=-buildid=", "-o", frontendExecutable, ".",
	)
	build.Dir = "."
	build.Env = fixedTestBuildEnvironment()
	if output, err := build.CombinedOutput(); err != nil {
		t.Fatalf("freshly build current go2vir candidate: %v: %s", err, output)
	}
	if err := os.Chmod(frontendExecutable, 0o555); err != nil {
		t.Fatalf("seal current go2vir candidate executable: %v", err)
	}
	sealTestReleaseRoot(t, frontendRoot)
	frontendFiles, err := inventoryFilesFromRoot(frontendRoot, "")
	if err != nil {
		t.Fatalf("inventory current go2vir candidate: %v", err)
	}
	frontendInventory := bundleInventory{
		Schema: bundleInventorySchema,
		Scope:  inventoryScope{Kind: "frontend_bundle", BundleID: "frontend.go.test.v0"},
		Files:  frontendFiles,
	}
	frontendBundleHash, err := hashTypedCanonicalJSON(bundleContentDomain, frontendInventory)
	if err != nil {
		t.Fatalf("hash current go2vir candidate inventory: %v", err)
	}
	frontendSHA := frontendFiles[0].SHA256
	registrySHA := sha256Hex([]byte("go2vir-unregistered-test-registry-v0"))
	request := lowerRequest{
		SourceRoot:                  logicalSourceRoot,
		Package:                     "example.com/mpk/vector",
		SemanticProfile:             goSemanticProfile,
		Target:                      goTarget,
		Function:                    "example.com/mpk/vector.Identity",
		FrontendBundleID:            "frontend.go.test.v0",
		FrontendSHA256:              frontendSHA,
		ReleaseRegistryID:           registryID,
		ReleaseRegistrySHA256:       registrySHA,
		ToolchainBundleID:           toolchain.identity.BundleID,
		ToolchainRoot:               logicalToolchain,
		ToolchainDistributionSHA256: toolchain.identity.DistributionSHA256,
		Contracts:                   []string{"identity_contract.json"},
	}
	candidate := launcherSelection{
		Registry: releaseRegistryIdentity{
			Schema:         "mpk.release.bundle_registry.v0",
			ID:             registryID,
			RegistrySHA256: registrySHA,
		},
		Frontend: frontendIdentity{
			BundleID:            request.FrontendBundleID,
			Name:                "go2vir",
			Version:             "test-current",
			BinarySHA256:        frontendSHA,
			SubordinateBinaries: []subordinateIdentity{},
		},
		Toolchain:              toolchain.identity,
		Target:                 targetIdentity{ID: goTarget, PointerWidth: goPointerWidth, LanguageConfiguration: fixedGoConfiguration()},
		LimitProfileID:         limitProfileID,
		EnvironmentProfileID:   environmentProfileID,
		ArgumentProfileID:      argumentProfileID,
		FrontendRootPath:       frontendRoot,
		FrontendExecutablePath: "go2vir",
		FrontendInventory:      frontendInventory,
		FrontendBundleSHA256:   frontendBundleHash,
		ToolchainGoRootPath:    toolchain.root,
		ToolchainInventory:     toolchain.inventory,
		ToolchainComponents:    toolchain.components,
	}
	return request, candidate
}

func computeCurrentToolchainCandidate(t *testing.T) testToolchainCandidate {
	t.Helper()
	installedRoot := runtime.GOROOT()
	root := filepath.Join(t.TempDir(), "go")
	for _, relative := range []string{
		"bin/go",
		"VERSION",
		"go.env",
		"pkg/include",
		"pkg/tool/" + runtime.GOOS + "_" + runtime.GOARCH,
		"src/builtin/builtin.go",
		"src/unsafe/unsafe.go",
	} {
		source := filepath.Join(installedRoot, filepath.FromSlash(relative))
		destination := filepath.Join(root, filepath.FromSlash(relative))
		if err := copyToolchainFixturePath(source, destination); err != nil {
			return testToolchainCandidate{err: err}
		}
	}
	sealTestReleaseRoot(t, root)
	files, err := inventoryFilesFromRoot(root, "go")
	if err != nil {
		return testToolchainCandidate{err: err}
	}
	bundleID := "toolchain.go.test.v0"
	rootInventory := bundleInventory{
		Schema: bundleInventorySchema,
		Scope:  inventoryScope{Kind: "toolchain_bundle", BundleID: bundleID},
		Files:  files,
	}
	distributionHash, err := hashTypedCanonicalJSON(bundleContentDomain, rootInventory)
	if err != nil {
		return testToolchainCandidate{err: err}
	}
	goIndex := sort.Search(len(files), func(index int) bool { return files[index].Path >= "go/bin/go" })
	if goIndex >= len(files) || files[goIndex].Path != "go/bin/go" || !files[goIndex].Executable {
		return testToolchainCandidate{err: fmt.Errorf("current GOROOT has no go/bin/go")}
	}
	components := make([]candidateComponent, 0)
	contentFiles := make([]inventoryFile, 0, len(files))
	for _, file := range files {
		if !file.Executable {
			contentFiles = append(contentFiles, file)
			continue
		}
		name := "go-tool-" + filepath.Base(file.Path)
		if file.Path == "go/bin/go" {
			name = "go"
		}
		inventory := bundleInventory{
			Schema: bundleInventorySchema,
			Scope:  inventoryScope{Kind: "component", BundleID: bundleID, ComponentName: name},
			Files:  []inventoryFile{file},
		}
		components = append(components, candidateComponent{
			Identity:  componentIdentity{Kind: "executable", Name: name, Release: runtime.Version(), BinarySHA256: file.SHA256},
			Inventory: inventory,
		})
	}
	contentInventory := bundleInventory{
		Schema: bundleInventorySchema,
		Scope:  inventoryScope{Kind: "component", BundleID: bundleID, ComponentName: "go-stdlib"},
		Files:  contentFiles,
	}
	contentHash, err := hashTypedCanonicalJSON(bundleContentDomain, contentInventory)
	if err != nil {
		return testToolchainCandidate{err: err}
	}
	components = append(components, candidateComponent{
		Identity:  componentIdentity{Kind: "content", Name: "go-stdlib", Release: runtime.Version(), ContentSHA256: contentHash},
		Inventory: contentInventory,
	})
	sort.Slice(components, func(left, right int) bool {
		return components[left].Identity.Name < components[right].Identity.Name
	})
	identities := make([]componentIdentity, len(components))
	for index := range components {
		identities[index] = components[index].Identity
	}
	return testToolchainCandidate{
		root:       root,
		inventory:  rootInventory,
		components: components,
		identity:   toolchainIdentity{BundleID: bundleID, DistributionSHA256: distributionHash, Components: identities},
	}
}

func fixedTestBuildEnvironment() []string {
	overrides := map[string]string{
		"CGO_ENABLED": "0",
		"GOARCH":      runtime.GOARCH,
		"GOENV":       "off",
		"GOFLAGS":     "",
		"GOOS":        runtime.GOOS,
		"GOPROXY":     "off",
		"GOSUMDB":     "off",
		"GOTOOLCHAIN": "local",
		"GOWORK":      "off",
	}
	environment := make([]string, 0, len(os.Environ())+len(overrides))
	for _, entry := range os.Environ() {
		name, _, found := strings.Cut(entry, "=")
		if _, overridden := overrides[name]; found && overridden {
			continue
		}
		environment = append(environment, entry)
	}
	keys := make([]string, 0, len(overrides))
	for name := range overrides {
		keys = append(keys, name)
	}
	sort.Strings(keys)
	for _, name := range keys {
		environment = append(environment, name+"="+overrides[name])
	}
	return environment
}

func copyToolchainFixturePath(source, destination string) error {
	info, err := os.Lstat(source)
	if err != nil {
		return err
	}
	if info.Mode()&os.ModeSymlink != 0 {
		return fmt.Errorf("fixed toolchain fixture source contains a symlink")
	}
	if info.IsDir() {
		return filepath.Walk(source, func(path string, entryInfo os.FileInfo, walkErr error) error {
			if walkErr != nil {
				return walkErr
			}
			relative, err := filepath.Rel(source, path)
			if err != nil {
				return err
			}
			target := filepath.Join(destination, relative)
			if entryInfo.IsDir() {
				return os.MkdirAll(target, 0o700)
			}
			return copyToolchainFixtureFile(path, target, entryInfo)
		})
	}
	return copyToolchainFixtureFile(source, destination, info)
}

func copyToolchainFixtureFile(source, destination string, info os.FileInfo) error {
	if !info.Mode().IsRegular() {
		return fmt.Errorf("fixed toolchain fixture source is not regular")
	}
	if err := os.MkdirAll(filepath.Dir(destination), 0o700); err != nil {
		return err
	}
	content, err := os.ReadFile(source)
	if err != nil {
		return err
	}
	mode := os.FileMode(0o444)
	if executableClass(info.Mode()) {
		mode = 0o555
	}
	if err := os.WriteFile(destination, content, mode); err != nil {
		return err
	}
	return os.Chmod(destination, mode)
}

func inventoryFilesFromRoot(root, virtualPrefix string) ([]inventoryFile, error) {
	metadata, err := enumeratePhysicalInventory(root, virtualPrefix)
	if err != nil {
		return nil, err
	}
	for index := range metadata {
		relative := metadata[index].Path
		if virtualPrefix != "" {
			relative = relative[len(virtualPrefix)+1:]
		}
		absolute := filepath.Join(root, filepath.FromSlash(relative))
		digest, info, err := hashInventoryFile(absolute, metadata[index].SizeBytes)
		if err != nil {
			return nil, err
		}
		metadata[index].SHA256 = digest
		metadata[index].Executable = executableClass(info.Mode())
	}
	return metadata, nil
}

func sealTestReleaseRoot(t *testing.T, root string) {
	t.Helper()
	if err := filepath.Walk(root, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}
		if info.IsDir() {
			return os.Chmod(path, 0o555)
		}
		return nil
	}); err != nil {
		t.Fatalf("seal test release root: %v", err)
	}
	t.Cleanup(func() {
		_ = filepath.Walk(root, func(path string, info os.FileInfo, err error) error {
			if err != nil {
				return nil
			}
			if info.IsDir() {
				return os.Chmod(path, 0o700)
			}
			return os.Chmod(path, 0o600)
		})
	})
}
