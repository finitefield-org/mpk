package main

import (
	"fmt"
	"go/ast"
	"go/types"
	"path"
	"runtime"
	"sort"
	"strings"

	"golang.org/x/mod/module"
)

type practicalTypeModule struct {
	identity module.Version
	manifest practicalModuleManifest
	location practicalModuleLocation
	archive  *practicalModuleArchive
}

// Original source-checked compiler facts, not a nominal/source/program proof
// or a registered context. The complete module/compiler/export and immutable
// command replay still need their independent ownership bindings.
type practicalTypedPackage struct {
	importPath      string
	moduleSourceID  string
	sources         practicalPackageSources
	files           []*ast.File
	types           *types.Package
	info            *types.Info
	fileVersions    map[string]string
	standardLibrary bool
}

type practicalSourceTypes struct {
	syntax   *practicalSyntaxImage
	modules  []practicalTypeModule
	standard map[string]struct{}
	stdlib   *practicalStdlibSources
	packages map[string]*practicalTypedPackage
	checking map[string]bool
	failed   error
}

func (loader *practicalSourceTypes) reject(err error) error {
	if loader.failed == nil {
		loader.failed = err
	}
	return loader.failed
}

func practicalSnapshotBuffer(snapshot practicalSourceSnapshot, object practicalSourceObject) (practicalSourceBuffer, error) {
	buffer, err := snapshot.readSource(object.Capability, object.Path, object.Kind)
	if err != nil {
		return practicalSourceBuffer{}, err
	}
	if buffer.object != object {
		return practicalSourceBuffer{}, fmt.Errorf("module input differs from its original sealed source identity")
	}
	if _, err := buffer.canonicalObject(); err != nil {
		return practicalSourceBuffer{}, err
	}
	return buffer, nil
}

func practicalVersionedSourceLocation(manifest practicalSourceObject, identity module.Version) (practicalModuleLocation, error) {
	escapedPath, err := module.EscapePath(identity.Path)
	if err != nil {
		return practicalModuleLocation{}, err
	}
	escapedVersion, err := module.EscapeVersion(identity.Version)
	if err != nil {
		return practicalModuleLocation{}, err
	}
	suffix := path.Join("cache/download", escapedPath, "@v", escapedVersion+".mod")
	cache := ""
	if manifest.Path != suffix {
		if !strings.HasSuffix(manifest.Path, "/"+suffix) {
			return practicalModuleLocation{}, fmt.Errorf("versioned source root does not follow the original Go cache location")
		}
		cache = strings.TrimSuffix(manifest.Path, "/"+suffix)
	}
	return practicalModuleLocation{capability: manifest.Capability, directory: path.Join(cache, escapedPath+"@"+escapedVersion)}, nil
}

func newPracticalSourceTypes(snapshot practicalSourceSnapshot, discovery practicalModuleDiscovery, compiler practicalCompilerImage) (*practicalSourceTypes, error) {
	if err := discovery.requireOriginalObservation(); err != nil {
		return nil, err
	}
	if runtime.Version() != practicalCompilerRelease || compiler.selection.Target.ID != goTarget || compiler.selection.Target.PointerWidth != 64 || !sha256Pattern.MatchString(discovery.compilerSHA256) {
		return nil, fmt.Errorf("original source type checking requires the exact prepared Go1.25.0 target")
	}
	compilerMatch := false
	for _, component := range compiler.selection.Toolchain.Components {
		if component.Name == "go" && component.Release == practicalCompilerRelease && component.Kind == "executable" && component.BinarySHA256 == discovery.compilerSHA256 {
			compilerMatch = true
		}
	}
	if !compilerMatch {
		return nil, fmt.Errorf("source module discovery belongs to a different compiler")
	}
	syntax, err := newPracticalSyntaxImage(snapshot)
	if err != nil {
		return nil, err
	}
	rows, err := decodePracticalGoModules(discovery.output)
	if err != nil {
		return nil, err
	}
	if len(rows) != len(discovery.modules) {
		return nil, fmt.Errorf("original selected module inventory is incomplete")
	}
	byPath := map[string]practicalGoListedModule{}
	for _, row := range rows {
		byPath[row.Path] = row
	}
	loader := &practicalSourceTypes{syntax: syntax, standard: map[string]struct{}{}, packages: map[string]*practicalTypedPackage{}, checking: map[string]bool{}}
	for name := range compiler.selection.StandardPackagePaths {
		loader.standard[name] = struct{}{}
	}
	mainCount := 0
	var mainSums *practicalModuleSums
	for _, resolved := range discovery.modules {
		if resolved.manifest.object.ID != discovery.mainSourceID {
			continue
		}
		mainCount++
		if resolved.location == nil {
			return nil, fmt.Errorf("original main module has no captured root")
		}
		entries, err := snapshot.readDirectory(resolved.location.capability, resolved.location.directory)
		if err != nil {
			return nil, err
		}
		if practicalNamespaceHas(entries, "go.sum") {
			source, err := snapshot.readSource(resolved.location.capability, practicalModuleFile(*resolved.location, "go.sum"), "go_sum")
			if err != nil {
				return nil, err
			}
			sums, err := parsePracticalModuleSums(source)
			if err != nil {
				return nil, err
			}
			mainSums = &sums
		}
	}
	if mainCount != 1 {
		return nil, fmt.Errorf("source type checking requires one original main module")
	}
	seen := map[string]bool{}
	for _, resolved := range discovery.modules {
		row, exists := byPath[resolved.identity.Path]
		main := resolved.manifest.object.ID == discovery.mainSourceID
		if !exists || seen[resolved.identity.Path] || row.Version != resolved.identity.Version || row.Main != main || (row.Replace == nil) != (resolved.replacement.Path == "") {
			return nil, fmt.Errorf("original source module identity differs from the observed Go selection")
		}
		seen[resolved.identity.Path] = true
		if _, shadow := loader.standard[resolved.identity.Path]; shadow || strings.HasPrefix(resolved.identity.Path, "cmd/") {
			return nil, fmt.Errorf("source module shadows the original compiler namespace")
		}
		if row.Replace != nil && (row.Replace.Path != resolved.replacement.Path || row.Replace.Version != resolved.replacement.Version) {
			return nil, fmt.Errorf("source module replacement differs from original Go selection")
		}
		buffer, err := practicalSnapshotBuffer(snapshot, resolved.manifest.object)
		if err != nil {
			return nil, err
		}
		manifest, err := parsePracticalModuleManifest(buffer, main)
		if err != nil {
			return nil, err
		}
		selected := practicalTypeModule{identity: resolved.identity, manifest: manifest}
		fact := row
		if row.Replace != nil {
			fact = *row.Replace
		}
		if manifest.file.Go != nil && fact.GoVersion != manifest.file.Go.Version {
			return nil, fmt.Errorf("selected module language differs from its original go.mod")
		}
		if resolved.location != nil {
			selected.location = *resolved.location
			if resolved.versioned != nil || buffer.object.Capability != selected.location.capability || buffer.object.Path != practicalModuleFile(selected.location, "go.mod") {
				return nil, fmt.Errorf("source module manifest is crossed with another original root")
			}
		} else {
			if resolved.versioned == nil || mainSums == nil {
				return nil, fmt.Errorf("source versioned module has no original archive/checksum authority")
			}
			effective := resolved.identity
			if resolved.replacement.Path != "" {
				effective = resolved.replacement
			}
			if effective != resolved.versioned.identity || effective.Version == "" {
				return nil, fmt.Errorf("source archive differs from its original selected version")
			}
			if err := mainSums.verifyManifest(effective, buffer); err != nil {
				return nil, err
			}
			prefix := strings.TrimSuffix(buffer.object.Path, ".mod")
			info, err := snapshot.readSource(buffer.object.Capability, prefix+".info", "metadata")
			if err != nil {
				return nil, err
			}
			if err := validatePracticalModuleInfo(info, effective); err != nil {
				return nil, err
			}
			zipHash, err := snapshot.readSource(buffer.object.Capability, prefix+".ziphash", "metadata")
			if err != nil {
				return nil, err
			}
			if strings.TrimSpace(string(zipHash.data)) != mainSums.entries[effective] {
				return nil, fmt.Errorf("source module ZIP metadata has a stale checksum")
			}
			zip, err := snapshot.readSource(buffer.object.Capability, prefix+".zip", "auxiliary")
			if err != nil {
				return nil, err
			}
			archive, err := parsePracticalModuleArchive(effective, zip, mainSums.entries[effective], maximumDirectoryEntries)
			if err != nil {
				return nil, err
			}
			if _, present := archive.members["go.mod"]; present {
				if err := archive.verifyContent("go.mod", buffer); err != nil {
					return nil, err
				}
			}
			location, err := practicalVersionedSourceLocation(buffer.object, effective)
			if err != nil {
				return nil, err
			}
			selected.location, selected.archive = location, &archive
		}
		loader.modules = append(loader.modules, selected)
	}
	sort.Slice(loader.modules, func(i, j int) bool {
		return len(loader.modules[i].identity.Path) > len(loader.modules[j].identity.Path)
	})
	return loader, nil
}

type practicalSourceImporter struct {
	loader *practicalSourceTypes
	from   string
}

func (importer practicalSourceImporter) Import(name string) (*types.Package, error) {
	return importer.ImportFrom(name, "", 0)
}
func (importer practicalSourceImporter) ImportFrom(name, _ string, mode types.ImportMode) (*types.Package, error) {
	if mode != 0 {
		return nil, importer.loader.reject(fmt.Errorf("original source import has an unknown mode"))
	}
	if name == "unsafe" {
		if _, exists := importer.loader.standard[name]; exists {
			return types.Unsafe, nil
		}
	}
	if name == "builtin" {
		return nil, importer.loader.reject(fmt.Errorf("the Go predeclared universe is not an importable source package"))
	}
	for _, part := range strings.Split(name, "/") {
		if part == "vendor" {
			return nil, importer.loader.reject(fmt.Errorf("explicit vendor import is outside the original import namespace"))
		}
	}
	fromStandard := importer.loader.stdlib.contains(importer.from)
	if fromStandard {
		resolved, err := importer.loader.stdlib.resolve(name, importer.from)
		if err != nil {
			return nil, importer.loader.reject(err)
		}
		name = resolved
	}
	_, standard := importer.loader.standard[name]
	if standard && !importer.loader.stdlib.contains(name) {
		return nil, importer.loader.reject(fmt.Errorf("standard package %s needs captured original source/export data: %w", name, errPracticalNotCaptured))
	}
	parts := strings.Split(name, "/")
	for i, part := range parts {
		if part == "internal" {
			if standard && !fromStandard {
				return nil, importer.loader.reject(fmt.Errorf("original standard-library internal import crosses the compiler source root"))
			}
			parent := strings.Join(parts[:i], "/")
			if parent != "" && importer.from != parent && !strings.HasPrefix(importer.from, parent+"/") {
				return nil, importer.loader.reject(fmt.Errorf("original Go internal import crosses its parent namespace"))
			}
		}
	}
	loaded, err := importer.loader.typecheck(name)
	if err != nil {
		return nil, err
	}
	if loaded.types.Name() == "main" {
		return nil, importer.loader.reject(fmt.Errorf("original import resolves to a program package"))
	}
	return loaded.types, nil
}

func (loader *practicalSourceTypes) typecheck(importPath string) (*practicalTypedPackage, error) {
	if loader.failed != nil {
		return nil, loader.failed
	}
	if loaded, exists := loader.packages[importPath]; exists {
		return loaded, nil
	}
	if module.CheckImportPath(importPath) != nil || strings.HasPrefix(importPath, ".") {
		return nil, loader.reject(fmt.Errorf("original package import path is not canonical"))
	}
	if loader.checking[importPath] {
		return nil, loader.reject(fmt.Errorf("original source packages have an import cycle"))
	}
	if loader.stdlib.contains(importPath) {
		location := loader.stdlib.location
		location.directory = path.Join(location.directory, importPath)
		sources, err := loader.syntax.snapshot.replayPackageBuild(location)
		if err != nil {
			return nil, loader.reject(err)
		}
		return loader.checkOriginalPackage(importPath, loader.stdlib.moduleSourceID, loader.stdlib.goVersion, sources, true)
	}
	var owner *practicalTypeModule
	for i := range loader.modules {
		candidate := &loader.modules[i]
		if importPath == candidate.identity.Path || strings.HasPrefix(importPath, candidate.identity.Path+"/") {
			owner = candidate
			break
		}
	}
	if owner == nil {
		return nil, loader.reject(fmt.Errorf("original source package %s has no captured selected module: %w", importPath, errPracticalNotCaptured))
	}
	relative := strings.TrimPrefix(strings.TrimPrefix(importPath, owner.identity.Path), "/")
	location := owner.location
	if relative != "" {
		for _, part := range strings.Split(relative, "/") {
			location.directory = path.Join(location.directory, part)
			entries, err := loader.syntax.snapshot.readDirectory(location.capability, location.directory)
			if err != nil {
				return nil, loader.reject(err)
			}
			if practicalNamespaceHas(entries, "go.mod") {
				return nil, loader.reject(fmt.Errorf("source package crosses an unselected nested module boundary"))
			}
		}
	}
	sources, err := loader.syntax.snapshot.replayPackageBuild(location)
	if err != nil {
		return nil, loader.reject(err)
	}
	if owner.archive != nil {
		all := append(append(append([]practicalSourceObject{}, sources.active...), sources.excluded...), sources.auxiliary...)
		for _, object := range all {
			buffer, err := practicalSnapshotBuffer(loader.syntax.snapshot, object)
			if err != nil {
				return nil, loader.reject(err)
			}
			member := strings.TrimPrefix(object.Path, owner.location.directory+"/")
			if member == object.Path {
				return nil, loader.reject(fmt.Errorf("source package escaped its original archive root"))
			}
			if err := owner.archive.verifyContent(member, buffer); err != nil {
				return nil, loader.reject(err)
			}
		}
	}
	goVersion := "go1.16"
	if owner.manifest.file.Go != nil {
		goVersion = "go" + owner.manifest.file.Go.Version
	}
	return loader.checkOriginalPackage(importPath, owner.manifest.source.object.ID, goVersion, sources, false)
}

func (loader *practicalSourceTypes) checkOriginalPackage(importPath, moduleSourceID, goVersion string, sources practicalPackageSources, standard bool) (*practicalTypedPackage, error) {
	loader.checking[importPath] = true
	defer delete(loader.checking, importPath)
	loaded := &practicalTypedPackage{importPath: importPath, moduleSourceID: moduleSourceID, sources: sources, fileVersions: map[string]string{}, standardLibrary: standard}
	for _, source := range sources.active {
		parsed, err := loader.syntax.parse(source.Capability, source.Path)
		if err != nil {
			return nil, loader.reject(err)
		}
		if parsed.source.object != source || parsed.file.Name.Name != sources.name {
			return nil, loader.reject(fmt.Errorf("original selected source syntax differs from package build metadata"))
		}
		loaded.files = append(loaded.files, parsed.file)
	}
	loaded.info = &types.Info{Types: map[ast.Expr]types.TypeAndValue{}, Defs: map[*ast.Ident]types.Object{}, Uses: map[*ast.Ident]types.Object{}, Implicits: map[ast.Node]types.Object{}, Selections: map[*ast.SelectorExpr]*types.Selection{}, Scopes: map[ast.Node]*types.Scope{}, Instances: map[*ast.Ident]types.Instance{}, FileVersions: map[*ast.File]string{}}
	config := types.Config{GoVersion: goVersion, Sizes: types.SizesFor("gc", "amd64"), Importer: practicalSourceImporter{loader: loader, from: importPath}, IgnoreFuncBodies: false, FakeImportC: false, DisableUnusedImportCheck: false}
	checked, err := config.Check(importPath, loader.syntax.fileSet, loaded.files, loaded.info)
	if loader.failed != nil {
		return nil, loader.failed
	}
	if err != nil {
		return nil, loader.reject(fmt.Errorf("original Go package type check %s: %w", importPath, err))
	}
	if checked == nil || !checked.Complete() || len(loaded.info.FileVersions) != len(loaded.files) {
		return nil, loader.reject(fmt.Errorf("original package type checking returned incomplete metadata"))
	}
	loaded.types = checked
	for i, file := range loaded.files {
		loaded.fileVersions[sources.active[i].ID] = loaded.info.FileVersions[file]
	}
	loader.packages[importPath] = loaded
	return loaded, nil
}
