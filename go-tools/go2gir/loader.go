package main

import (
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"strings"

	"golang.org/x/tools/go/packages"
)

const packageLoadMode = packages.NeedName |
	packages.NeedFiles |
	packages.NeedCompiledGoFiles |
	packages.NeedImports |
	packages.NeedModule

type loadOptions struct {
	Dir string
	Env []string
}

type loadedPackage struct {
	ID              string   `json:"id"`
	PackagePath     string   `json:"package_path"`
	Name            string   `json:"name"`
	GoFiles         []string `json:"go_files"`
	CompiledGoFiles []string `json:"compiled_go_files"`
	Imports         []string `json:"imports"`
}

func loadPackages(packagePath string, options loadOptions) ([]loadedPackage, error) {
	config := pinnedPackageConfig(options)
	packagesLoaded, err := packages.Load(config, packagePath)
	if err != nil {
		return nil, fmt.Errorf("load Go packages: %w", err)
	}
	if len(packagesLoaded) == 0 {
		return nil, fmt.Errorf("load Go packages: no packages matched %q", packagePath)
	}

	var packageErrors []string
	loaded := make([]loadedPackage, 0, len(packagesLoaded))
	for _, pkg := range packagesLoaded {
		for _, packageError := range pkg.Errors {
			packageErrors = append(packageErrors, fmt.Sprintf("%s: %s", pkg.PkgPath, packageError.Msg))
		}
		loaded = append(loaded, summarizePackage(pkg, config.Dir))
	}
	if len(packageErrors) > 0 {
		sort.Strings(packageErrors)
		return nil, fmt.Errorf("load Go packages: %s", strings.Join(packageErrors, "; "))
	}

	sort.Slice(loaded, func(i, j int) bool {
		if loaded[i].PackagePath != loaded[j].PackagePath {
			return loaded[i].PackagePath < loaded[j].PackagePath
		}
		return loaded[i].ID < loaded[j].ID
	})
	return loaded, nil
}

func pinnedPackageConfig(options loadOptions) *packages.Config {
	dir := options.Dir
	if dir == "" {
		workingDir, err := os.Getwd()
		if err == nil {
			dir = workingDir
		}
	} else if absoluteDir, err := filepath.Abs(dir); err == nil {
		dir = absoluteDir
	}
	env := options.Env
	if env == nil {
		env = os.Environ()
	}

	return &packages.Config{
		Mode:       packageLoadMode,
		Dir:        dir,
		Env:        withPinnedGoEnv(env),
		BuildFlags: []string{"-mod=readonly"},
		Tests:      false,
	}
}

func withPinnedGoEnv(env []string) []string {
	pinned := map[string]string{
		"CGO_ENABLED": "0",
		"GO111MODULE": "on",
	}
	pinnedKeys := []string{"CGO_ENABLED", "GO111MODULE"}

	result := make([]string, 0, len(env)+len(pinned))
	for _, entry := range env {
		key, _, found := strings.Cut(entry, "=")
		if !found {
			continue
		}
		if _, ok := pinned[key]; ok {
			continue
		}
		result = append(result, entry)
	}
	for _, key := range pinnedKeys {
		result = append(result, key+"="+pinned[key])
	}
	return result
}

func summarizePackage(pkg *packages.Package, baseDir string) loadedPackage {
	imports := make([]string, 0, len(pkg.Imports))
	for importPath := range pkg.Imports {
		imports = append(imports, importPath)
	}
	sort.Strings(imports)

	return loadedPackage{
		ID:              pkg.ID,
		PackagePath:     pkg.PkgPath,
		Name:            pkg.Name,
		GoFiles:         normalizeFileList(baseDir, pkg.GoFiles),
		CompiledGoFiles: normalizeFileList(baseDir, pkg.CompiledGoFiles),
		Imports:         imports,
	}
}

func normalizeFileList(baseDir string, paths []string) []string {
	normalized := make([]string, 0, len(paths))
	for _, path := range paths {
		normalized = append(normalized, normalizePath(baseDir, path))
	}
	sort.Strings(normalized)
	return normalized
}

func normalizePath(baseDir string, path string) string {
	if baseDir != "" {
		if relative, err := filepath.Rel(baseDir, path); err == nil && !isParentRelativePath(relative) {
			return filepath.ToSlash(relative)
		}
	}
	return filepath.ToSlash(path)
}

func isParentRelativePath(path string) bool {
	return path == ".." || strings.HasPrefix(path, ".."+string(filepath.Separator))
}
