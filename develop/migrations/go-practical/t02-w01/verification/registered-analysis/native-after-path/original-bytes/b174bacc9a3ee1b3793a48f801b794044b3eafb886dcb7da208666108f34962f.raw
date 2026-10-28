package main

import (
	"crypto/sha256"
	"encoding/hex"
	"fmt"
	"io"
	"os"
	pathpkg "path"
	"path/filepath"
	"reflect"
	"sort"
	"strings"
)

const (
	bundleInventorySchema  = "mpk.release.bundle_inventory.v0"
	bundleContentDomain    = "MPK-BUNDLE-CONTENT-0.1"
	limitProfileID         = "mpk.vir.limits.v0"
	environmentProfileID   = "mpk.go.frontend_environment.v0"
	argumentProfileID      = "mpk.go.frontend_arguments.v0"
	maximumBundleFiles     = 262_144
	maximumComponents      = 8_192
	maximumBundleFileBytes = int64(4_294_967_296)
	maximumBundleBytes     = int64(34_359_738_368)
)

type releaseRegistryIdentity struct {
	Schema         string `json:"schema"`
	ID             string `json:"id"`
	RegistrySHA256 string `json:"registry_sha256"`
}

type frontendIdentity struct {
	BundleID            string                `json:"bundle_id"`
	Name                string                `json:"name"`
	Version             string                `json:"version"`
	BinarySHA256        string                `json:"binary_sha256"`
	SubordinateBinaries []subordinateIdentity `json:"subordinate_binaries"`
}

type subordinateIdentity struct {
	Name         string `json:"name"`
	Version      string `json:"version"`
	BinarySHA256 string `json:"binary_sha256"`
}

type componentIdentity struct {
	Kind          string `json:"kind"`
	Name          string `json:"name"`
	Release       string `json:"release"`
	BinarySHA256  string `json:"binary_sha256,omitempty"`
	ContentSHA256 string `json:"content_sha256,omitempty"`
}

type toolchainIdentity struct {
	BundleID           string              `json:"bundle_id"`
	DistributionSHA256 string              `json:"distribution_sha256"`
	Components         []componentIdentity `json:"components"`
}

type targetIdentity struct {
	ID                    string               `json:"id"`
	PointerWidth          int64                `json:"pointer_width"`
	LanguageConfiguration goFixedConfiguration `json:"language_configuration"`
}

type goFixedConfiguration struct {
	Kind                 string   `json:"kind"`
	Compiler             string   `json:"compiler"`
	CgoEnabled           bool     `json:"cgo_enabled"`
	Go111Module          string   `json:"go111module"`
	ModuleMode           string   `json:"module_mode"`
	WorkspaceMode        string   `json:"workspace_mode"`
	Tests                bool     `json:"tests"`
	BuildTags            []string `json:"build_tags"`
	EnvironmentProfileID string   `json:"environment_profile_id"`
	ArgumentProfileID    string   `json:"argument_profile_id"`
}

type inventoryScope struct {
	Kind          string `json:"kind"`
	BundleID      string `json:"bundle_id"`
	ComponentName string `json:"component_name,omitempty"`
}

type inventoryFile struct {
	Path       string `json:"path"`
	Executable bool   `json:"executable"`
	SizeBytes  int64  `json:"size_bytes"`
	SHA256     string `json:"sha256"`
}

type bundleInventory struct {
	Schema string          `json:"schema"`
	Scope  inventoryScope  `json:"scope"`
	Files  []inventoryFile `json:"files"`
}

type candidateComponent struct {
	Identity  componentIdentity
	Inventory bundleInventory
}

type launcherSelection struct {
	Registry               releaseRegistryIdentity
	Frontend               frontendIdentity
	Toolchain              toolchainIdentity
	Target                 targetIdentity
	LimitProfileID         string
	EnvironmentProfileID   string
	ArgumentProfileID      string
	FrontendRootPath       string
	FrontendExecutablePath string
	FrontendInventory      bundleInventory
	FrontendBundleSHA256   string
	ToolchainGoRootPath    string
	ToolchainInventory     bundleInventory
	ToolchainComponents    []candidateComponent
}

type validatedLauncherSelection struct {
	Registry             releaseRegistryIdentity
	Frontend             frontendIdentity
	Toolchain            toolchainIdentity
	Target               targetIdentity
	LimitProfileID       string
	GoRoot               string
	GoExecutable         string
	StandardPackagePaths map[string]struct{}
}

func fixedGoConfiguration() goFixedConfiguration {
	return goFixedConfiguration{
		Kind:                 "go",
		Compiler:             "gc",
		CgoEnabled:           false,
		Go111Module:          "on",
		ModuleMode:           "readonly",
		WorkspaceMode:        "off",
		Tests:                false,
		BuildTags:            []string{},
		EnvironmentProfileID: environmentProfileID,
		ArgumentProfileID:    argumentProfileID,
	}
}

func validateLauncherSelection(request lowerRequest, candidate launcherSelection) (validatedLauncherSelection, error) {
	if candidate.Registry.Schema != "mpk.release.bundle_registry.v1" || candidate.Registry.ID != request.ReleaseRegistryID || candidate.Registry.RegistrySHA256 != request.ReleaseRegistrySHA256 {
		return validatedLauncherSelection{}, fail("frontend-error", "capture", "GO_FRONTEND_TOOLCHAIN", "release registry selection does not match the launcher request")
	}
	if candidate.Frontend.BundleID != request.FrontendBundleID || candidate.Frontend.BinarySHA256 != request.FrontendSHA256 || candidate.Frontend.Name != "go2vir" || len(candidate.Frontend.SubordinateBinaries) != 0 {
		return validatedLauncherSelection{}, fail("frontend-error", "capture", "GO_FRONTEND_TOOLCHAIN", "frontend bundle selection does not match the launcher request")
	}
	if candidate.Toolchain.BundleID != request.ToolchainBundleID || candidate.Toolchain.DistributionSHA256 != request.ToolchainDistributionSHA256 {
		return validatedLauncherSelection{}, fail("frontend-error", "capture", "GO_FRONTEND_TOOLCHAIN", "toolchain bundle selection does not match the launcher request")
	}
	if candidate.Target.ID != request.Target || candidate.Target.PointerWidth != goPointerWidth || !reflect.DeepEqual(candidate.Target.LanguageConfiguration, fixedGoConfiguration()) {
		return validatedLauncherSelection{}, fail("frontend-error", "capture", "GO_FRONTEND_TOOLCHAIN", "target selection does not match the Go semantic profile")
	}
	if candidate.LimitProfileID != limitProfileID || candidate.EnvironmentProfileID != environmentProfileID || candidate.ArgumentProfileID != argumentProfileID {
		return validatedLauncherSelection{}, fail("frontend-error", "capture", "GO_FRONTEND_TOOLCHAIN", "frontend profile identities are not exact")
	}
	if err := validateFrontendCandidate(candidate); err != nil {
		return validatedLauncherSelection{}, err
	}
	standardPackages, goExecutable, err := validateToolchainCandidate(candidate)
	if err != nil {
		return validatedLauncherSelection{}, err
	}
	return validatedLauncherSelection{
		Registry:             candidate.Registry,
		Frontend:             candidate.Frontend,
		Toolchain:            candidate.Toolchain,
		Target:               candidate.Target,
		LimitProfileID:       candidate.LimitProfileID,
		GoRoot:               candidate.ToolchainGoRootPath,
		GoExecutable:         goExecutable,
		StandardPackagePaths: standardPackages,
	}, nil
}

func validateFrontendCandidate(candidate launcherSelection) error {
	if !validReleaseVersion(candidate.Frontend.Version) || !validPortablePath(candidate.FrontendExecutablePath) {
		return fail("frontend-error", "capture", "GO_FRONTEND_TOOLCHAIN", "frontend executable identity is invalid")
	}
	if candidate.FrontendInventory.Schema != bundleInventorySchema || candidate.FrontendInventory.Scope != (inventoryScope{Kind: "frontend_bundle", BundleID: candidate.Frontend.BundleID}) {
		return fail("frontend-error", "capture", "GO_FRONTEND_TOOLCHAIN", "frontend inventory scope is invalid")
	}
	if err := validateInventoryShape(candidate.FrontendInventory.Files); err != nil {
		return err
	}
	if err := validatePhysicalInventory(candidate.FrontendRootPath, "", candidate.FrontendInventory.Files); err != nil {
		return err
	}
	digest, err := hashTypedCanonicalJSON(bundleContentDomain, candidate.FrontendInventory)
	if err != nil || digest != candidate.FrontendBundleSHA256 {
		return fail("frontend-error", "capture", "GO_FRONTEND_TOOLCHAIN", "frontend inventory content identity is invalid")
	}
	matching := 0
	executables := 0
	for _, file := range candidate.FrontendInventory.Files {
		if file.Executable {
			executables++
		}
		if file.Path == candidate.FrontendExecutablePath && file.SHA256 == candidate.Frontend.BinarySHA256 && file.Executable {
			matching++
		}
	}
	if matching != 1 || executables != 1 {
		return fail("frontend-error", "capture", "GO_FRONTEND_TOOLCHAIN", "frontend executable is not uniquely represented by its inventory")
	}
	return nil
}

func validateToolchainCandidate(candidate launcherSelection) (map[string]struct{}, string, error) {
	if len(candidate.ToolchainComponents) == 0 || len(candidate.ToolchainComponents) > maximumComponents {
		return nil, "", fail("frontend-error", "capture", "GO_FRONTEND_TOOLCHAIN", "toolchain component count is invalid")
	}
	wantScope := inventoryScope{Kind: "toolchain_bundle", BundleID: candidate.Toolchain.BundleID}
	if candidate.ToolchainInventory.Schema != bundleInventorySchema || candidate.ToolchainInventory.Scope != wantScope {
		return nil, "", fail("frontend-error", "capture", "GO_FRONTEND_TOOLCHAIN", "toolchain inventory scope is invalid")
	}
	if err := validateInventoryShape(candidate.ToolchainInventory.Files); err != nil {
		return nil, "", err
	}
	if err := validatePhysicalInventory(candidate.ToolchainGoRootPath, "go", candidate.ToolchainInventory.Files); err != nil {
		return nil, "", err
	}
	distributionDigest, err := hashTypedCanonicalJSON(bundleContentDomain, candidate.ToolchainInventory)
	if err != nil || distributionDigest != candidate.Toolchain.DistributionSHA256 {
		return nil, "", fail("frontend-error", "capture", "GO_FRONTEND_TOOLCHAIN", "toolchain distribution identity is invalid")
	}

	serializedEntries := uint64(len(candidate.ToolchainInventory.Files))
	componentByName := make(map[string]candidateComponent, len(candidate.ToolchainComponents))
	partition := make(map[string]string, len(candidate.ToolchainInventory.Files))
	for _, component := range candidate.ToolchainComponents {
		if len(component.Identity.Name) < 1 || len(component.Identity.Name) > 128 || !idPattern.MatchString(component.Identity.Name) || !validReleaseVersion(component.Identity.Release) {
			return nil, "", fail("frontend-error", "capture", "GO_FRONTEND_TOOLCHAIN", "toolchain component identity is invalid")
		}
		serializedEntries += uint64(len(component.Inventory.Files))
		if serializedEntries > maximumBundleFiles {
			return nil, "", fail("frontend-error", "capture", "GO_FRONTEND_TOOLCHAIN", "toolchain serialized inventory limit is exceeded")
		}
		if _, duplicate := componentByName[component.Identity.Name]; duplicate {
			return nil, "", fail("frontend-error", "capture", "GO_FRONTEND_TOOLCHAIN", "toolchain component name is duplicated")
		}
		wantComponentScope := inventoryScope{Kind: "component", BundleID: candidate.Toolchain.BundleID, ComponentName: component.Identity.Name}
		if component.Inventory.Schema != bundleInventorySchema || component.Inventory.Scope != wantComponentScope {
			return nil, "", fail("frontend-error", "capture", "GO_FRONTEND_TOOLCHAIN", "toolchain component inventory scope is invalid")
		}
		if err := validateInventoryShape(component.Inventory.Files); err != nil {
			return nil, "", err
		}
		digest, hashErr := hashTypedCanonicalJSON(bundleContentDomain, component.Inventory)
		if hashErr != nil {
			return nil, "", hashErr
		}
		switch component.Identity.Kind {
		case "executable":
			if len(component.Inventory.Files) != 1 || !component.Inventory.Files[0].Executable || component.Identity.BinarySHA256 != component.Inventory.Files[0].SHA256 || component.Identity.ContentSHA256 != "" {
				return nil, "", fail("frontend-error", "capture", "GO_FRONTEND_TOOLCHAIN", "toolchain executable component is invalid")
			}
		case "content":
			if component.Identity.ContentSHA256 != digest || component.Identity.BinarySHA256 != "" {
				return nil, "", fail("frontend-error", "capture", "GO_FRONTEND_TOOLCHAIN", "toolchain content component is invalid")
			}
		default:
			return nil, "", fail("frontend-error", "capture", "GO_FRONTEND_TOOLCHAIN", "unknown toolchain component kind")
		}
		for _, file := range component.Inventory.Files {
			if _, duplicate := partition[file.Path]; duplicate {
				return nil, "", fail("frontend-error", "capture", "GO_FRONTEND_TOOLCHAIN", "toolchain component inventories overlap")
			}
			partition[file.Path] = component.Identity.Name
		}
		componentByName[component.Identity.Name] = component
	}
	if len(candidate.Toolchain.Components) != len(candidate.ToolchainComponents) {
		return nil, "", fail("frontend-error", "capture", "GO_FRONTEND_TOOLCHAIN", "toolchain component projection is incomplete")
	}
	for index, identity := range candidate.Toolchain.Components {
		if index > 0 && candidate.Toolchain.Components[index-1].Name >= identity.Name {
			return nil, "", fail("frontend-error", "capture", "GO_FRONTEND_TOOLCHAIN", "toolchain component projection is not sorted")
		}
		component, exists := componentByName[identity.Name]
		if !exists || component.Identity != identity {
			return nil, "", fail("frontend-error", "capture", "GO_FRONTEND_TOOLCHAIN", "toolchain component projection differs from the candidate inventory")
		}
	}
	if len(partition) != len(candidate.ToolchainInventory.Files) {
		return nil, "", fail("frontend-error", "capture", "GO_FRONTEND_TOOLCHAIN", "toolchain components do not partition the root inventory")
	}
	for _, file := range candidate.ToolchainInventory.Files {
		componentName, exists := partition[file.Path]
		if !exists {
			return nil, "", fail("frontend-error", "capture", "GO_FRONTEND_TOOLCHAIN", "toolchain inventory file has no component")
		}
		componentFiles := componentByName[componentName].Inventory.Files
		index := sort.Search(len(componentFiles), func(index int) bool { return componentFiles[index].Path >= file.Path })
		if index >= len(componentFiles) || componentFiles[index] != file {
			return nil, "", fail("frontend-error", "capture", "GO_FRONTEND_TOOLCHAIN", "toolchain component entry differs from the root inventory")
		}
	}

	goExecutableVirtual := "go/bin/go"
	rootFiles := candidate.ToolchainInventory.Files
	index := sort.Search(len(rootFiles), func(index int) bool { return rootFiles[index].Path >= goExecutableVirtual })
	if index >= len(rootFiles) || rootFiles[index].Path != goExecutableVirtual || !rootFiles[index].Executable {
		return nil, "", fail("frontend-error", "capture", "GO_FRONTEND_TOOLCHAIN", "allowlisted Go executable is absent")
	}
	goComponent, exists := componentByName["go"]
	if !exists || goComponent.Identity.Kind != "executable" || goComponent.Identity.BinarySHA256 != rootFiles[index].SHA256 {
		return nil, "", fail("frontend-error", "capture", "GO_FRONTEND_TOOLCHAIN", "Go executable component identity is invalid")
	}
	standardPackages := make(map[string]struct{})
	for _, file := range rootFiles {
		if strings.HasPrefix(file.Path, "go/src/") && strings.HasSuffix(file.Path, ".go") {
			relative := strings.TrimPrefix(file.Path, "go/src/")
			directory := filepath.ToSlash(filepath.Dir(relative))
			if directory != "." {
				standardPackages[directory] = struct{}{}
			}
		}
	}
	return standardPackages, filepath.Join(candidate.ToolchainGoRootPath, "bin", executableName("go")), nil
}

func validateInventoryShape(files []inventoryFile) error {
	if len(files) == 0 || len(files) > maximumBundleFiles {
		return fail("frontend-error", "capture", "GO_FRONTEND_TOOLCHAIN", "bundle inventory file count is invalid")
	}
	seenFolded := make(map[string]struct{}, len(files))
	declaredBytes := uint64(0)
	for index, file := range files {
		if !validPortablePath(file.Path) || file.SizeBytes < 0 || file.SizeBytes > maximumBundleFileBytes || !sha256Pattern.MatchString(file.SHA256) {
			return fail("frontend-error", "capture", "GO_FRONTEND_TOOLCHAIN", "bundle inventory contains an invalid file")
		}
		declaredBytes += uint64(file.SizeBytes)
		if declaredBytes > uint64(maximumBundleBytes) {
			return fail("frontend-error", "capture", "GO_FRONTEND_TOOLCHAIN", "bundle declared byte limit is exceeded")
		}
		if index > 0 && files[index-1].Path >= file.Path {
			return fail("frontend-error", "capture", "GO_FRONTEND_TOOLCHAIN", "bundle inventory paths are not strictly sorted")
		}
		folded := strings.ToLower(file.Path)
		if _, duplicate := seenFolded[folded]; duplicate {
			return fail("frontend-error", "capture", "GO_FRONTEND_TOOLCHAIN", "bundle inventory path case-collides")
		}
		seenFolded[folded] = struct{}{}
	}
	return nil
}

func validatePhysicalInventory(root, virtualPrefix string, files []inventoryFile) error {
	rootInfo, err := os.Lstat(root)
	if err != nil || rootInfo.Mode()&os.ModeSymlink != 0 || !rootInfo.IsDir() || !releaseDirectoryModeValid(rootInfo.Mode()) {
		return fail("frontend-error", "capture", "GO_FRONTEND_TOOLCHAIN", "bundle root is not a regular directory")
	}
	physicalFiles, err := enumeratePhysicalInventory(root, virtualPrefix)
	if err != nil || len(physicalFiles) != len(files) {
		return fail("frontend-error", "capture", "GO_FRONTEND_TOOLCHAIN", "bundle root inventory is incomplete")
	}
	for index := range files {
		if physicalFiles[index].Path != files[index].Path || physicalFiles[index].SizeBytes != files[index].SizeBytes || physicalFiles[index].Executable != files[index].Executable {
			return fail("frontend-error", "capture", "GO_FRONTEND_TOOLCHAIN", "bundle root namespace differs from its inventory")
		}
	}
	for _, file := range files {
		relative := file.Path
		if virtualPrefix != "" {
			prefix := virtualPrefix + "/"
			if !strings.HasPrefix(relative, prefix) {
				return fail("frontend-error", "capture", "GO_FRONTEND_TOOLCHAIN", "toolchain inventory escapes its logical root")
			}
			relative = strings.TrimPrefix(relative, prefix)
		}
		absolute := filepath.Join(root, filepath.FromSlash(relative))
		digest, info, readErr := hashInventoryFile(absolute, file.SizeBytes)
		if readErr != nil || digest != file.SHA256 || executableClass(info.Mode()) != file.Executable || !releaseFileModeValid(info.Mode(), file.Executable) {
			return fail("frontend-error", "capture", "GO_FRONTEND_TOOLCHAIN", "bundle inventory file differs from its selected identity")
		}
	}
	finalRootInfo, rootErr := os.Lstat(root)
	finalFiles, inventoryErr := enumeratePhysicalInventory(root, virtualPrefix)
	if rootErr != nil || !sameFileState(rootInfo, finalRootInfo) || inventoryErr != nil || !reflect.DeepEqual(physicalFiles, finalFiles) {
		return fail("frontend-error", "capture", "GO_FRONTEND_TOOLCHAIN", "bundle root changed while validating its inventory")
	}
	return nil
}

func enumeratePhysicalInventory(root, virtualPrefix string) ([]inventoryFile, error) {
	files := make([]inventoryFile, 0)
	directories := make(map[string]struct{})
	err := filepath.Walk(root, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}
		if path == root {
			return nil
		}
		if info.Mode()&os.ModeSymlink != 0 || !info.IsDir() && !info.Mode().IsRegular() {
			return fmt.Errorf("bundle root contains a link or special file")
		}
		if info.IsDir() {
			if !releaseDirectoryModeValid(info.Mode()) {
				return fmt.Errorf("bundle directory mode is not sealed")
			}
			relative, err := filepath.Rel(root, path)
			if err != nil {
				return err
			}
			directories[filepath.ToSlash(relative)] = struct{}{}
			return nil
		}
		if !releaseFileModeValid(info.Mode(), executableClass(info.Mode())) {
			return fmt.Errorf("bundle file mode is not sealed")
		}
		relative, err := filepath.Rel(root, path)
		if err != nil {
			return err
		}
		normalized := filepath.ToSlash(relative)
		if virtualPrefix != "" {
			normalized = virtualPrefix + "/" + normalized
		}
		files = append(files, inventoryFile{Path: normalized, Executable: executableClass(info.Mode()), SizeBytes: info.Size()})
		return nil
	})
	if err != nil {
		return nil, err
	}
	impliedDirectories := make(map[string]struct{})
	prefix := ""
	if virtualPrefix != "" {
		prefix = virtualPrefix + "/"
	}
	for _, file := range files {
		relative := strings.TrimPrefix(file.Path, prefix)
		for directory := pathpkg.Dir(relative); directory != "."; directory = pathpkg.Dir(directory) {
			impliedDirectories[directory] = struct{}{}
		}
	}
	if !reflect.DeepEqual(directories, impliedDirectories) {
		return nil, fmt.Errorf("bundle root contains an undeclared directory")
	}
	sort.Slice(files, func(left, right int) bool { return files[left].Path < files[right].Path })
	return files, nil
}

func hashInventoryFile(path string, expectedSize int64) (string, os.FileInfo, error) {
	before, err := os.Lstat(path)
	if err != nil || before.Mode()&os.ModeSymlink != 0 || !before.Mode().IsRegular() || before.Size() != expectedSize {
		return "", nil, fmt.Errorf("inventory path is not the expected regular file")
	}
	file, err := openRegularNoFollow(path)
	if err != nil {
		return "", nil, err
	}
	defer file.Close()
	hasher := sha256.New()
	read, err := io.Copy(hasher, io.LimitReader(file, expectedSize+1))
	if err != nil || read != expectedSize {
		return "", nil, fmt.Errorf("inventory file size changed while reading")
	}
	after, err := file.Stat()
	if err != nil || !sameFileState(before, after) {
		return "", nil, fmt.Errorf("inventory file changed while hashing")
	}
	current, err := os.Lstat(path)
	if err != nil || !sameFileState(after, current) {
		return "", nil, fmt.Errorf("inventory path changed while hashing")
	}
	return hex.EncodeToString(hasher.Sum(nil)), after, nil
}

func executableClass(mode os.FileMode) bool {
	return mode.Perm()&0o111 != 0
}

func releaseDirectoryModeValid(mode os.FileMode) bool {
	return mode.IsDir() && mode.Perm() == 0o555 && mode&(os.ModeSetuid|os.ModeSetgid|os.ModeSticky) == 0
}

func releaseFileModeValid(mode os.FileMode, executable bool) bool {
	want := os.FileMode(0o444)
	if executable {
		want = 0o555
	}
	return mode.IsRegular() && mode.Perm() == want && mode&(os.ModeSetuid|os.ModeSetgid|os.ModeSticky) == 0
}

func executableName(base string) string {
	if filepath.Separator == '\\' {
		return base + ".exe"
	}
	return base
}

func validReleaseVersion(value string) bool {
	if len(value) < 1 || len(value) > 128 || value[0] == ' ' || value[len(value)-1] == ' ' {
		return false
	}
	for _, character := range []byte(value) {
		if character < 0x20 || character > 0x7e || character == '\\' || character == '/' {
			return false
		}
	}
	return true
}

func hashTypedCanonicalJSON(domain string, value any) (string, error) {
	canonical, err := canonicalJSON(value)
	if err != nil {
		return "", err
	}
	strict, err := decodeStrictJSON(canonical)
	if err != nil {
		return "", err
	}
	return hashCanonicalJSON(domain, strict)
}
