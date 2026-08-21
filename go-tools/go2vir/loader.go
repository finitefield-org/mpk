package main

import (
	"fmt"
	"go/ast"
	"go/types"
	"os"
	"path/filepath"
	"sort"
	"strings"
	"sync"

	"golang.org/x/tools/go/packages"
)

const packageLoadMode = packages.NeedName |
	packages.NeedFiles |
	packages.NeedCompiledGoFiles |
	packages.NeedImports |
	packages.NeedDeps |
	packages.NeedModule |
	packages.NeedTypes |
	packages.NeedSyntax |
	packages.NeedTypesInfo |
	packages.NeedTypesSizes

var processPathMutex sync.Mutex

type loadedPackage struct {
	PackagePath     string
	Name            string
	GoFiles         []string
	CompiledGoFiles []string
	Imports         []string
	packageValue    *packages.Package
}

type packageLoadResult struct {
	Packages []loadedPackage
}

type loaderFilesystem struct {
	root      string
	goBuild   string
	goMod     string
	goPath    string
	home      string
	temporary string
}

func loadCapturedPackages(capture sourceCapture, snapshot sourceSnapshot, selection validatedLauncherSelection) (packageLoadResult, error) {
	if !snapshot.matchesCapture(capture) {
		return packageLoadResult{}, fail("frontend-error", "capture", "GO_FRONTEND_INTERNAL", "private source snapshot identity is inconsistent")
	}
	for _, packageRecord := range capture.Packages {
		if _, shadowsToolchain := selection.StandardPackagePaths[packageRecord.ImportPath]; shadowsToolchain || strings.HasPrefix(packageRecord.ImportPath, "cmd/") {
			return packageLoadResult{}, fail("rejected", "source", "GO_SUBSET_IMPORT", "source module shadows the selected toolchain namespace")
		}
	}
	loaderFS, err := newLoaderFilesystem()
	if err != nil {
		return packageLoadResult{}, fail("frontend-error", "capture", "GO_FRONTEND_SANDBOX", "private loader filesystem cannot be created")
	}
	defer loaderFS.Close()
	configuration := pinnedPackageConfig(snapshot, selection, loaderFS)

	packagesLoaded, loadErr, pathErr := loadPackagesWithPinnedProcessPath(configuration, selection.GoExecutable, capture.SelectedPackage)
	if pathErr != nil {
		return packageLoadResult{}, fail("frontend-error", "capture", "GO_FRONTEND_SANDBOX", "pinned Go executable path cannot be installed")
	}
	if loadErr != nil {
		return packageLoadResult{}, failWithCause("frontend-error", "typecheck", "GO_FRONTEND_TOOLCHAIN", "pinned Go package loading failed", loadErr)
	}
	if len(packagesLoaded) != 1 || packagesLoaded[0].PkgPath != capture.SelectedPackage {
		return packageLoadResult{}, fail("source-error", "capture", "GO_PACKAGE_AMBIGUOUS", "selected package did not resolve exactly once")
	}

	all := flattenPackageGraph(packagesLoaded[0])
	capturedByImport := make(map[string]capturedPackage, len(capture.Packages))
	for _, packageRecord := range capture.Packages {
		capturedByImport[packageRecord.ImportPath] = packageRecord
	}
	loaded := make([]loadedPackage, 0, len(all))
	typedSyntax := make([]*ast.File, 0)
	for _, packageValue := range all {
		capturedPackage, expected := capturedByImport[packageValue.PkgPath]
		if !expected {
			return packageLoadResult{}, fail("rejected", "source", "GO_SUBSET_IMPORT", "loader resolved an unrecorded package")
		}
		if packageValue.Module == nil || packageValue.Module.Path != capture.ModulePath {
			return packageLoadResult{}, fail("rejected", "metadata", "GO_MODULE_DEPENDENCY", "loader resolved a package outside the captured module")
		}
		if len(packageValue.Errors) != 0 {
			return packageLoadResult{}, fail("source-error", "typecheck", "GO_TYPECHECK", "captured Go package does not type check")
		}
		if packageValue.Name != capturedPackage.Name || packageValue.Types == nil || packageValue.TypesSizes == nil {
			return packageLoadResult{}, fail("frontend-error", "typecheck", "GO_FRONTEND_TOOLCHAIN", "loader package identity is incomplete")
		}
		if packageValue.TypesSizes.Sizeof(types.Typ[types.Uintptr]) != goPointerWidth/8 {
			return packageLoadResult{}, fail("frontend-error", "typecheck", "GO_FRONTEND_TOOLCHAIN", "loader pointer width differs from the selected target")
		}
		goFiles, normalizeErr := normalizeSnapshotFiles(snapshot, packageValue.GoFiles)
		if normalizeErr != nil {
			return packageLoadResult{}, fail("rejected", "source", "GO_BUILD_CONSTRAINT", "loader returned a source outside the private snapshot")
		}
		compiledFiles, normalizeErr := normalizeSnapshotFiles(snapshot, packageValue.CompiledGoFiles)
		if normalizeErr != nil {
			return packageLoadResult{}, fail("rejected", "source", "GO_BUILD_CONSTRAINT", "loader returned a compiled source outside the private snapshot")
		}
		if !equalStrings(goFiles, capturedPackage.Sources) || !equalStrings(compiledFiles, capturedPackage.Sources) || !equalStrings(goFiles, compiledFiles) || len(packageValue.IgnoredFiles) != 0 {
			return packageLoadResult{}, fail("rejected", "source", "GO_BUILD_CONSTRAINT", "loader file inventory differs from the captured package")
		}
		imports := make([]string, 0, len(packageValue.Imports))
		for importPath := range packageValue.Imports {
			imports = append(imports, importPath)
		}
		sort.Strings(imports)
		if !equalStrings(imports, capturedPackage.Imports) {
			return packageLoadResult{}, fail("rejected", "source", "GO_SUBSET_IMPORT", "loader import graph differs from captured syntax")
		}
		typedSyntax = append(typedSyntax, packageValue.Syntax...)
		loaded = append(loaded, loadedPackage{
			PackagePath:     packageValue.PkgPath,
			Name:            packageValue.Name,
			GoFiles:         goFiles,
			CompiledGoFiles: compiledFiles,
			Imports:         imports,
			packageValue:    packageValue,
		})
	}
	if len(loaded) != len(capture.Packages) {
		return packageLoadResult{}, fail("rejected", "source", "GO_SUBSET_IMPORT", "loader omitted a captured package")
	}
	if err := countSyntaxNodes(typedSyntax, maximumTypedSyntaxNodes); err != nil {
		return packageLoadResult{}, err
	}
	sort.Slice(loaded, func(left, right int) bool { return loaded[left].PackagePath < loaded[right].PackagePath })
	return packageLoadResult{Packages: loaded}, nil
}

func loadPackagesWithPinnedProcessPath(configuration *packages.Config, goExecutable, pattern string) (loaded []*packages.Package, loadErr, pathErr error) {
	processPathMutex.Lock()
	defer processPathMutex.Unlock()

	previousPath, hadPath := os.LookupEnv("PATH")
	if err := os.Setenv("PATH", filepath.Dir(goExecutable)); err != nil {
		return nil, nil, err
	}
	defer func() {
		if hadPath {
			pathErr = os.Setenv("PATH", previousPath)
		} else {
			pathErr = os.Unsetenv("PATH")
		}
	}()
	loaded, loadErr = packages.Load(configuration, pattern)
	return loaded, loadErr, nil
}

func pinnedPackageConfig(snapshot sourceSnapshot, selection validatedLauncherSelection, filesystem loaderFilesystem) *packages.Config {
	return &packages.Config{
		Mode:       packageLoadMode,
		Dir:        snapshot.Root,
		Env:        pinnedLoaderEnvironment(selection, filesystem),
		BuildFlags: []string{"-mod=readonly"},
		Tests:      false,
		Overlay:    nil,
		ParseFile:  nil,
		Logf:       nil,
	}
}

func pinnedLoaderEnvironment(selection validatedLauncherSelection, filesystem loaderFilesystem) []string {
	values := map[string]string{
		"CGO_ENABLED":      "0",
		"GO111MODULE":      "on",
		"GOAMD64":          "v1",
		"GOARCH":           "amd64",
		"GOCACHE":          filesystem.goBuild,
		"GODEBUG":          "",
		"GOENV":            "off",
		"GOEXPERIMENT":     "",
		"GOFLAGS":          "",
		"GOMAXPROCS":       "1",
		"GOMODCACHE":       filesystem.goMod,
		"GONOPROXY":        "",
		"GONOSUMDB":        "",
		"GOOS":             "linux",
		"GOPACKAGESDRIVER": "off",
		"GOPATH":           filesystem.goPath,
		"GOPRIVATE":        "",
		"GOPROXY":          "off",
		"GOROOT":           selection.GoRoot,
		"GOSUMDB":          "off",
		"GOTELEMETRY":      "off",
		"GOTOOLCHAIN":      "local",
		"GOVCS":            "*:off",
		"GOWORK":           "off",
		"HOME":             filesystem.home,
		"LANG":             "C",
		"LC_ALL":           "C",
		"PATH":             filepath.Dir(selection.GoExecutable),
		"TMPDIR":           filesystem.temporary,
		"TZ":               "UTC",
	}
	keys := make([]string, 0, len(values))
	for key := range values {
		keys = append(keys, key)
	}
	sort.Strings(keys)
	environment := make([]string, 0, len(keys))
	for _, key := range keys {
		environment = append(environment, key+"="+values[key])
	}
	return environment
}

func newLoaderFilesystem() (loaderFilesystem, error) {
	root, err := os.MkdirTemp("", "go2vir-loader-")
	if err != nil {
		return loaderFilesystem{}, err
	}
	filesystem := loaderFilesystem{
		root:      root,
		goBuild:   filepath.Join(root, "cache", "go-build"),
		goMod:     filepath.Join(root, "cache", "go-mod"),
		goPath:    filepath.Join(root, "gopath"),
		home:      filepath.Join(root, "empty", "home"),
		temporary: filepath.Join(root, "tmp"),
	}
	allDirectories := []string{
		filesystem.root,
		filepath.Join(filesystem.root, "cache"),
		filesystem.goBuild,
		filesystem.goMod,
		filesystem.goPath,
		filepath.Join(filesystem.root, "empty"),
		filesystem.home,
		filesystem.temporary,
	}
	for _, path := range allDirectories {
		if err := os.MkdirAll(path, 0o700); err != nil {
			_ = os.RemoveAll(root)
			return loaderFilesystem{}, err
		}
		if err := os.Chmod(path, 0o700); err != nil {
			_ = os.RemoveAll(root)
			return loaderFilesystem{}, err
		}
	}
	for _, path := range []string{filesystem.goMod, filesystem.goPath, filesystem.home} {
		if err := os.Chmod(path, 0o500); err != nil {
			_ = os.RemoveAll(root)
			return loaderFilesystem{}, err
		}
	}
	return filesystem, nil
}

func (filesystem loaderFilesystem) Close() error {
	for _, path := range []string{filesystem.goMod, filesystem.goPath, filesystem.home} {
		_ = os.Chmod(path, 0o700)
	}
	return os.RemoveAll(filesystem.root)
}

func flattenPackageGraph(root *packages.Package) []*packages.Package {
	seen := make(map[string]*packages.Package)
	var visit func(*packages.Package)
	visit = func(packageValue *packages.Package) {
		if packageValue == nil {
			return
		}
		if _, exists := seen[packageValue.ID]; exists {
			return
		}
		seen[packageValue.ID] = packageValue
		imports := make([]string, 0, len(packageValue.Imports))
		for path := range packageValue.Imports {
			imports = append(imports, path)
		}
		sort.Strings(imports)
		for _, path := range imports {
			visit(packageValue.Imports[path])
		}
	}
	visit(root)
	result := make([]*packages.Package, 0, len(seen))
	for _, packageValue := range seen {
		result = append(result, packageValue)
	}
	sort.Slice(result, func(left, right int) bool {
		if result[left].PkgPath != result[right].PkgPath {
			return result[left].PkgPath < result[right].PkgPath
		}
		return result[left].ID < result[right].ID
	})
	return result
}

func normalizeSnapshotFiles(snapshot sourceSnapshot, files []string) ([]string, error) {
	normalized := make([]string, 0, len(files))
	for _, path := range files {
		value, err := snapshot.normalizedPath(path)
		if err != nil {
			return nil, err
		}
		if input, exists := snapshot.inputs[value]; !exists || input.Kind != sourceInputKind {
			return nil, fmt.Errorf("loader file is not a captured source input")
		}
		normalized = append(normalized, value)
	}
	sort.Strings(normalized)
	for index := 1; index < len(normalized); index++ {
		if normalized[index-1] == normalized[index] {
			return nil, fmt.Errorf("loader returned a duplicate source file")
		}
	}
	return normalized, nil
}
