package main

import (
	"bufio"
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"strings"
)

const (
	registeredFrontendRoot    = "/mpk/frontend"
	registeredFrontendPath    = "bin/go2vir"
	registeredFrontendVersion = "go1.25.0-profile-v1-staging"
)

func buildRegisteredLauncherSelection(request lowerRequest) (launcherSelection, error) {
	frontendFiles, err := inventoryRegisteredRoot(registeredFrontendRoot, "")
	if err != nil || len(frontendFiles) != 1 || frontendFiles[0].Path != registeredFrontendPath {
		return launcherSelection{}, fail("frontend-error", "capture", "GO_FRONTEND_TOOLCHAIN", "registered frontend inventory is invalid")
	}
	frontendInventory := bundleInventory{
		Schema: bundleInventorySchema,
		Scope:  inventoryScope{Kind: "frontend_bundle", BundleID: request.FrontendBundleID},
		Files:  frontendFiles,
	}
	frontendBundleHash, err := hashTypedCanonicalJSON(bundleContentDomain, frontendInventory)
	if err != nil {
		return launcherSelection{}, fail("frontend-error", "capture", "GO_FRONTEND_TOOLCHAIN", "registered frontend identity cannot be computed")
	}

	goRoot := filepath.Join(logicalToolchain, "go")
	release, err := registeredGoRelease(goRoot)
	if err != nil {
		return launcherSelection{}, fail("frontend-error", "capture", "GO_FRONTEND_TOOLCHAIN", "registered Go release identity is invalid")
	}
	toolchainFiles, err := inventoryRegisteredRoot(goRoot, "go")
	if err != nil {
		return launcherSelection{}, fail("frontend-error", "capture", "GO_FRONTEND_TOOLCHAIN", "registered toolchain inventory is invalid")
	}
	toolchainInventory := bundleInventory{
		Schema: bundleInventorySchema,
		Scope:  inventoryScope{Kind: "toolchain_bundle", BundleID: request.ToolchainBundleID},
		Files:  toolchainFiles,
	}
	distributionHash, err := hashTypedCanonicalJSON(bundleContentDomain, toolchainInventory)
	if err != nil {
		return launcherSelection{}, fail("frontend-error", "capture", "GO_FRONTEND_TOOLCHAIN", "registered toolchain identity cannot be computed")
	}
	components, identities, err := registeredToolchainComponents(request.ToolchainBundleID, release, toolchainFiles)
	if err != nil {
		return launcherSelection{}, err
	}

	return launcherSelection{
		Registry: releaseRegistryIdentity{
			Schema:         "mpk.release.bundle_registry.v1",
			ID:             request.ReleaseRegistryID,
			RegistrySHA256: request.ReleaseRegistrySHA256,
		},
		Frontend: frontendIdentity{
			BundleID:            request.FrontendBundleID,
			Name:                "go2vir",
			Version:             registeredFrontendVersion,
			BinarySHA256:        frontendFiles[0].SHA256,
			SubordinateBinaries: []subordinateIdentity{},
		},
		Toolchain: toolchainIdentity{
			BundleID:           request.ToolchainBundleID,
			DistributionSHA256: distributionHash,
			Components:         identities,
		},
		Target: targetIdentity{
			ID:                    goTarget,
			PointerWidth:          goPointerWidth,
			LanguageConfiguration: fixedGoConfiguration(),
		},
		LimitProfileID:         limitProfileID,
		EnvironmentProfileID:   environmentProfileID,
		ArgumentProfileID:      argumentProfileID,
		FrontendRootPath:       registeredFrontendRoot,
		FrontendExecutablePath: registeredFrontendPath,
		FrontendInventory:      frontendInventory,
		FrontendBundleSHA256:   frontendBundleHash,
		ToolchainGoRootPath:    goRoot,
		ToolchainInventory:     toolchainInventory,
		ToolchainComponents:    components,
	}, nil
}

func inventoryRegisteredRoot(root, virtualPrefix string) ([]inventoryFile, error) {
	files, err := enumeratePhysicalInventory(root, virtualPrefix)
	if err != nil {
		return nil, err
	}
	for index := range files {
		relative := files[index].Path
		if virtualPrefix != "" {
			relative = strings.TrimPrefix(relative, virtualPrefix+"/")
		}
		digest, info, err := hashInventoryFile(filepath.Join(root, filepath.FromSlash(relative)), files[index].SizeBytes)
		if err != nil {
			return nil, err
		}
		files[index].SHA256 = digest
		files[index].Executable = executableClass(info.Mode())
	}
	return files, nil
}

func registeredGoRelease(goRoot string) (string, error) {
	file, err := os.Open(filepath.Join(goRoot, "VERSION"))
	if err != nil {
		return "", err
	}
	defer file.Close()
	scanner := bufio.NewScanner(file)
	if !scanner.Scan() || scanner.Text() != "go1.25.0" || scanner.Err() != nil {
		return "", fmt.Errorf("unexpected registered Go release")
	}
	return scanner.Text(), nil
}

func registeredToolchainComponents(bundleID, release string, files []inventoryFile) ([]candidateComponent, []componentIdentity, error) {
	contentFiles := make([]inventoryFile, 0, len(files))
	components := make([]candidateComponent, 0)
	for _, file := range files {
		if !file.Executable {
			contentFiles = append(contentFiles, file)
			continue
		}
		name := "go-tool-" + filepath.Base(file.Path)
		switch file.Path {
		case "go/bin/go":
			name = "go"
		case "go/bin/gofmt":
			name = "gofmt"
		}
		components = append(components, candidateComponent{
			Identity: componentIdentity{
				Kind:         "executable",
				Name:         name,
				Release:      release,
				BinarySHA256: file.SHA256,
			},
			Inventory: bundleInventory{
				Schema: bundleInventorySchema,
				Scope:  inventoryScope{Kind: "component", BundleID: bundleID, ComponentName: name},
				Files:  []inventoryFile{file},
			},
		})
	}
	contentInventory := bundleInventory{
		Schema: bundleInventorySchema,
		Scope:  inventoryScope{Kind: "component", BundleID: bundleID, ComponentName: "go-target-linux-amd64"},
		Files:  contentFiles,
	}
	contentHash, err := hashTypedCanonicalJSON(bundleContentDomain, contentInventory)
	if err != nil {
		return nil, nil, fail("frontend-error", "capture", "GO_FRONTEND_TOOLCHAIN", "registered target-library identity cannot be computed")
	}
	components = append(components, candidateComponent{
		Identity: componentIdentity{
			Kind:          "content",
			Name:          "go-target-linux-amd64",
			Release:       release,
			ContentSHA256: contentHash,
		},
		Inventory: contentInventory,
	})
	sort.Slice(components, func(left, right int) bool {
		return components[left].Identity.Name < components[right].Identity.Name
	})
	identities := make([]componentIdentity, len(components))
	for index := range components {
		identities[index] = components[index].Identity
	}
	return components, identities, nil
}
