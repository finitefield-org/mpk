package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"maps"
	"path"
	"path/filepath"
	"reflect"
	"sort"
	"strconv"
	"strings"

	"golang.org/x/mod/module"
)

// Original command metadata is retained in full. These selected fields do not
// form a public artifact schema or authorize an opaque source/library proof.
type practicalGoPackageExport struct {
	Dir, ImportPath, Name, Export, BuildID                                                                  string
	Standard, Goroot, Incomplete, BinaryOnly                                                                bool
	ForTest                                                                                                 string
	DepOnly                                                                                                 bool
	Match                                                                                                   []string
	GoFiles, CompiledGoFiles, CgoFiles, Imports                                                             []string
	IgnoredGoFiles, TestGoFiles, XTestGoFiles                                                               []string
	EmbedPatterns, EmbedFiles                                                                               []string
	CFiles, CXXFiles, MFiles, HFiles, FFiles, SFiles, SwigFiles, SwigCXXFiles, SysoFiles, IgnoredOtherFiles []string
	ImportMap                                                                                               map[string]string
	Module                                                                                                  *practicalGoListedModule
	Error                                                                                                   *struct{ Err string }
	DepsErrors                                                                                              []json.RawMessage
}

type practicalCapturedPackageExport struct {
	importPath string
	name       string
	buildID    string
	standard   bool
	intrinsic  bool
	sources    []practicalSourceObject
	embedded   []practicalSourceObject
	data       *practicalSourceObject
	imports    []string
	importMap  map[string]string
}

type practicalPackageExports struct {
	compilerSHA256      string
	distributionSHA256  string
	mainSourceID        string
	requestedPackage    string
	buildCacheDirectory string
	output, diagnostics []byte
	packages            []practicalCapturedPackageExport
	snapshot            practicalSourceSnapshot
	replay              practicalReplaySources
	observation         *practicalGoObservation
}

// A private observer may hold the generated cache root before export capture
// and independently compare its sealed snapshot while the root is still live.
// This is not launcher authorization or a compiler-correctness assertion.
type practicalExportCaptureObserver struct {
	before func(practicalSourceRoot) error
	after  func(practicalSnapshotInputs) error
}

func decodePracticalPackageExports(raw []byte) ([]practicalGoPackageExport, error) {
	if len(raw) > maximumCandidateBytes {
		return nil, fmt.Errorf("native package/export metadata exceeds its input byte bound")
	}
	decoder := json.NewDecoder(bytes.NewReader(raw))
	seen := map[string]bool{}
	var rows []practicalGoPackageExport
	for {
		var encoded json.RawMessage
		if err := decoder.Decode(&encoded); err == io.EOF {
			break
		} else if err != nil {
			return nil, err
		}
		if len(rows) >= maximumManifestInputs {
			return nil, fmt.Errorf("native package/export count exceeded")
		}
		if _, err := decodeStrictJSON(encoded); err != nil {
			return nil, err
		}
		var row practicalGoPackageExport
		if err := json.Unmarshal(encoded, &row); err != nil {
			return nil, err
		}
		if !validPracticalLogicalPath(row.ImportPath) || row.Name == "" || seen[row.ImportPath] || row.Incomplete || row.BinaryOnly || row.ForTest != "" || row.Error != nil || len(row.DepsErrors) != 0 || len(row.CgoFiles) != 0 {
			return nil, fmt.Errorf("native export metadata has missing/duplicate/failed/generated package state")
		}
		seen[row.ImportPath] = true
		rows = append(rows, row)
	}
	if len(rows) == 0 {
		return nil, fmt.Errorf("native export command returned no package")
	}
	return rows, nil
}

func practicalExportSourceLocation(compiler practicalCompilerImage, discovery practicalModuleDiscovery, sdkCapability string, row practicalGoPackageExport) (practicalModuleLocation, error) {
	if row.Standard || row.Goroot {
		if !row.Standard || !row.Goroot || row.Module != nil {
			return practicalModuleLocation{}, fmt.Errorf("standard export has crossed module/compiler ownership")
		}
		if _, exists := compiler.selection.StandardPackagePaths[row.ImportPath]; !exists {
			return practicalModuleLocation{}, fmt.Errorf("standard export belongs to an unrecorded compiler package")
		}
		return practicalModuleLocation{capability: sdkCapability, directory: "src/" + row.ImportPath}, nil
	}
	var owner *practicalResolvedModule
	for i := range discovery.modules {
		candidate := &discovery.modules[i]
		if row.ImportPath == candidate.identity.Path || strings.HasPrefix(row.ImportPath, candidate.identity.Path+"/") {
			if owner == nil || len(candidate.identity.Path) > len(owner.identity.Path) {
				owner = candidate
			}
		}
	}
	if owner == nil || row.Module == nil || row.Module.Path != owner.identity.Path || row.Module.Version != owner.identity.Version {
		return practicalModuleLocation{}, fmt.Errorf("source export has no exact original selected module owner")
	}
	var location practicalModuleLocation
	if owner.location != nil {
		location = *owner.location
	} else {
		if owner.versioned == nil {
			return location, fmt.Errorf("source export has no captured original module root")
		}
		var err error
		location, err = practicalVersionedSourceLocation(owner.manifest.object, owner.versioned.identity)
		if err != nil {
			return location, err
		}
	}
	relative := strings.TrimPrefix(strings.TrimPrefix(row.ImportPath, owner.identity.Path), "/")
	if relative != "" {
		location.directory += "/" + relative
		location.directory = strings.TrimPrefix(location.directory, "/")
	}
	return location, nil
}

func (session *practicalCaptureSession) capturePracticalPackageExports(compiler practicalCompilerImage, discovery practicalModuleDiscovery, sdkCapability string, mainLocation practicalModuleLocation, selectedPackage string) (practicalPackageExports, error) {
	return session.capturePracticalPackageExportsObserved(compiler, discovery, sdkCapability, mainLocation, selectedPackage, nil)
}

func (session *practicalCaptureSession) capturePracticalPackageExportsObserved(compiler practicalCompilerImage, discovery practicalModuleDiscovery, sdkCapability string, mainLocation practicalModuleLocation, selectedPackage string, observer *practicalExportCaptureObserver) (practicalPackageExports, error) {
	result := practicalPackageExports{compilerSHA256: discovery.compilerSHA256, distributionSHA256: compiler.selection.Toolchain.DistributionSHA256, mainSourceID: discovery.mainSourceID, requestedPackage: selectedPackage}
	if observer != nil && (observer.before == nil || observer.after == nil) {
		return result, session.reject(fmt.Errorf("export root observer is incomplete"))
	}
	if err := discovery.requireOriginalObservation(); err != nil {
		return result, session.reject(err)
	}
	if err := compiler.revalidate(); err != nil {
		return result, session.reject(err)
	}
	if discovery.observation.compilerRoot != compiler.selection.GoRoot || discovery.observation.distributionSHA256 != compiler.selection.Toolchain.DistributionSHA256 {
		return result, session.reject(fmt.Errorf("native export preparation crossed its original module compiler image"))
	}
	var main *practicalResolvedModule
	for i := range discovery.modules {
		if discovery.modules[i].manifest.object.ID == discovery.mainSourceID {
			if main != nil {
				return result, session.reject(fmt.Errorf("export preparation has duplicate main modules"))
			}
			main = &discovery.modules[i]
		}
	}
	if main == nil || main.location == nil || *main.location != mainLocation || selectedPackage != main.identity.Path && !strings.HasPrefix(selectedPackage, main.identity.Path+"/") {
		return result, session.reject(fmt.Errorf("export preparation selected a package outside the original main module"))
	}
	directory, err := session.practicalPhysicalDirectory(mainLocation)
	if err != nil {
		return result, err
	}
	if directory != discovery.observation.directory {
		return result, session.reject(fmt.Errorf("native export preparation crossed its original main directory"))
	}
	filesystem, err := newLoaderFilesystem()
	if err != nil {
		return result, session.reject(err)
	}
	defer filesystem.Close()
	// Resolve only our newly created private cache before giving its name to
	// Go. Caller source roots still require exact, link-free original paths.
	filesystem.goBuild, err = filepath.EvalSymlinks(filesystem.goBuild)
	if err != nil {
		return result, session.reject(err)
	}
	environmentFS := filesystem
	if cache, err := practicalExportOriginalCache(discovery); err != nil {
		return result, session.reject(err)
	} else if cache != nil {
		environmentFS.goMod, err = session.practicalPhysicalDirectory(*cache)
		if err != nil {
			return result, err
		}
	}
	result.observation, err = runPracticalGoObservation(compiler.selection, environmentFS, directory, discovery.mainSourceID, practicalGoPackages, selectedPackage)
	if err != nil {
		return result, session.reject(err)
	}
	result.output, result.diagnostics = bytes.Clone(result.observation.output), bytes.Clone(result.observation.diagnostics)
	result.buildCacheDirectory = filesystem.goBuild
	if err := compiler.revalidate(); err != nil {
		return result, session.reject(err)
	}
	rows, err := decodePracticalPackageExports(result.output)
	if err != nil {
		return result, session.reject(err)
	}
	if err := validatePracticalExportRoot(rows, selectedPackage); err != nil {
		return result, session.reject(err)
	}
	if observer != nil {
		if err := observer.before(practicalSourceRoot{Capability: "original-go-package-exports", Directory: filesystem.goBuild}); err != nil {
			return result, session.reject(err)
		}
	}
	// Only the command's private fresh cache is authorized for derived export
	// buffers. It is opened after generation, then sealed with normal rooted
	// source reads. The application/source roots remain the caller's originals.
	exports, err := newPracticalCaptureSession([]practicalSourceRoot{{Capability: "original-go-package-exports", Directory: filesystem.goBuild}})
	if err != nil {
		return result, session.reject(err)
	}
	selectedCount := 0
	for _, row := range rows {
		if row.ImportPath == selectedPackage {
			selectedCount++
		}
		location, err := practicalExportSourceLocation(compiler, discovery, sdkCapability, row)
		if err != nil {
			return result, session.reject(err)
		}
		bound, exists := session.roots[location.capability]
		if !exists {
			return result, session.reject(fmt.Errorf("export source owner is not an authorized original capability"))
		}
		expectedDir := filepath.Join(bound.physical, filepath.FromSlash(location.directory))
		if row.Dir != expectedDir {
			return result, session.reject(fmt.Errorf("native export source directory differs from its original captured owner"))
		}
		captured := practicalCapturedPackageExport{importPath: row.ImportPath, name: row.Name, buildID: row.BuildID, standard: row.Standard, imports: append([]string(nil), row.Imports...), importMap: map[string]string{}}
		for original, canonical := range row.ImportMap {
			captured.importMap[original] = canonical
		}
		for _, name := range row.GoFiles {
			if !validPracticalLogicalPath(name) || strings.Contains(name, "/") {
				return result, session.reject(fmt.Errorf("native export source is not an exact original Go filename"))
			}
			buffer, err := session.readSource(location.capability, path.Join(location.directory, name), "go_file")
			if err != nil {
				return result, err
			}
			captured.sources = append(captured.sources, buffer.object)
		}
		// Immutable go/build replay consults every original source candidate,
		// including headers and target/test exclusions. Capture them before
		// sealing; an active GoFiles-only packet cannot reconstruct selection.
		seen := map[string]bool{}
		for _, name := range row.GoFiles {
			seen[name] = true
		}
		for _, list := range []struct {
			names []string
			kind  string
		}{
			{row.IgnoredGoFiles, "excluded_source"}, {row.TestGoFiles, "excluded_source"}, {row.XTestGoFiles, "excluded_source"},
			{row.CFiles, "auxiliary"}, {row.CXXFiles, "auxiliary"}, {row.MFiles, "auxiliary"}, {row.HFiles, "auxiliary"}, {row.FFiles, "auxiliary"},
			{row.SFiles, "auxiliary"}, {row.SwigFiles, "auxiliary"}, {row.SwigCXXFiles, "auxiliary"}, {row.SysoFiles, "auxiliary"}, {row.IgnoredOtherFiles, "auxiliary"},
		} {
			for _, name := range list.names {
				if !validPracticalLogicalPath(name) || strings.Contains(name, "/") || seen[name] {
					return result, session.reject(fmt.Errorf("native package has an invalid or repeated original build input"))
				}
				seen[name] = true
				if _, err := session.readSource(location.capability, path.Join(location.directory, name), list.kind); err != nil {
					return result, err
				}
			}
		}
		embedded, err := session.captureEmbeddedSources(location, row.EmbedPatterns)
		if err != nil {
			return result, session.reject(err)
		}
		if strings.Join(embedded.files, "\x00") != strings.Join(row.EmbedFiles, "\x00") {
			return result, session.reject(fmt.Errorf("native embedded file list differs from original captured pattern selection"))
		}
		captured.embedded = embedded.objects
		if row.ImportPath == "unsafe" {
			if !row.Standard || row.Export != "" || row.BuildID != "" {
				return result, session.reject(fmt.Errorf("unsafe compiler intrinsic has fabricated export data"))
			}
			captured.intrinsic = true
		} else {
			if len(captured.sources) == 0 || row.Export == "" || row.BuildID == "" || len(row.BuildID) > 1024 || len(row.CompiledGoFiles) != len(row.GoFiles) {
				return result, session.reject(fmt.Errorf("native compiled source/export/build identity is incomplete"))
			}
			for i, compiled := range row.CompiledGoFiles {
				if filepath.IsAbs(compiled) {
					if filepath.Dir(compiled) != row.Dir {
						return result, session.reject(fmt.Errorf("native compiled source was generated outside the original package"))
					}
					compiled = filepath.Base(compiled)
				}
				if compiled != row.GoFiles[i] {
					return result, session.reject(fmt.Errorf("native compiler source list differs from original selected source files"))
				}
			}
			relative, err := filepath.Rel(filesystem.goBuild, row.Export)
			if err != nil || !filepath.IsAbs(row.Export) || filepath.Clean(row.Export) != row.Export || filepath.IsAbs(relative) || relative == ".." || strings.HasPrefix(relative, ".."+string(filepath.Separator)) {
				return result, session.reject(fmt.Errorf("native export file escaped the private compiler cache"))
			}
			buffer, err := exports.readSource("original-go-package-exports", filepath.ToSlash(relative), "export_data")
			if err != nil {
				return result, session.reject(err)
			}
			object := buffer.object
			captured.data = &object
		}
		result.packages = append(result.packages, captured)
	}
	if selectedCount != 1 {
		return result, session.reject(fmt.Errorf("native export preparation did not resolve the selected package exactly once"))
	}
	result.replay, err = exports.sealPracticalReplaySources()
	if err != nil {
		return result, session.reject(err)
	}
	result.snapshot = result.replay.snapshot
	if observer != nil {
		inputs, err := result.snapshot.snapshotInputs()
		if err != nil {
			return result, session.reject(err)
		}
		if err := observer.after(inputs); err != nil {
			return result, session.reject(err)
		}
	}
	sort.Slice(result.packages, func(i, j int) bool { return result.packages[i].importPath < result.packages[j].importPath })
	return result, nil
}

// The native export command must read the same original offline module cache
// as discovery. Derive its root from the already bound original archive/.mod
// identities; never replace it with an ambient cache or a downloading proxy.
func practicalExportOriginalCache(discovery practicalModuleDiscovery) (*practicalModuleLocation, error) {
	var selected *practicalModuleLocation
	for _, resolved := range discovery.modules {
		if resolved.versioned == nil {
			continue
		}
		identity := resolved.versioned.identity
		escaped, err := module.EscapePath(identity.Path)
		if err != nil {
			return nil, err
		}
		version, err := module.EscapeVersion(identity.Version)
		if err != nil {
			return nil, err
		}
		object := resolved.manifest.object
		suffix := path.Join("cache/download", escaped, "@v", version+".mod")
		prefix := ""
		if object.Path != suffix {
			if !strings.HasSuffix(object.Path, "/"+suffix) {
				return nil, fmt.Errorf("export module cache crossed its original download identity")
			}
			prefix = strings.TrimSuffix(object.Path, "/"+suffix)
		}
		location := practicalModuleLocation{capability: object.Capability, directory: prefix}
		if selected != nil && *selected != location {
			return nil, fmt.Errorf("export packages crossed original offline module caches")
		}
		selected = &location
	}
	return selected, nil
}

// Read the native archive's own build ID, rather than accepting the metadata
// label as the identity of arbitrary bytes. All archive members remain owned;
// this inspection does not turn an export signature into a source proof.
func validatePracticalExportArchive(raw []byte, expectedBuildID string) error {
	if len(raw) < 68 || string(raw[:8]) != "!<arch>\n" || expectedBuildID == "" {
		return fmt.Errorf("native export is not an original compiled Go archive")
	}
	first := true
	for offset := 8; offset < len(raw); {
		if len(raw)-offset < 60 {
			return fmt.Errorf("native export has a truncated archive member")
		}
		header := raw[offset : offset+60]
		size, err := strconv.ParseUint(strings.TrimSpace(string(header[48:58])), 10, 64)
		if err != nil || string(header[58:60]) != "`\n" || size > uint64(len(raw)-offset-60) {
			return fmt.Errorf("native export has an invalid archive member bound")
		}
		start := offset + 60
		end := start + int(size)
		if first {
			if strings.TrimSpace(string(header[:16])) != "__.PKGDEF" {
				return fmt.Errorf("native export has no original Go package definition")
			}
			object, rest, found := bytes.Cut(raw[start:end], []byte{'\n'})
			if !found || !bytes.HasPrefix(object, []byte("go object linux amd64 go1.25.0 GOAMD64=v1 X:")) {
				return fmt.Errorf("native export belongs to another compiler/target")
			}
			build, _, found := bytes.Cut(rest, []byte{'\n'})
			if !found || !bytes.HasPrefix(build, []byte("build id ")) {
				return fmt.Errorf("native export has no original build ID")
			}
			id, err := strconv.Unquote(string(build[len("build id "):]))
			if err != nil || id != expectedBuildID {
				return fmt.Errorf("native export bytes differ from the observed compiler build ID")
			}
			first = false
		}
		offset = end
		if size&1 != 0 {
			if offset >= len(raw) || raw[offset] != 0 {
				return fmt.Errorf("native export has invalid archive padding")
			}
			offset++
		}
	}
	return nil
}

func validatePracticalExportRoot(rows []practicalGoPackageExport, selectedPackage string) error {
	selected := 0
	for _, row := range rows {
		if row.ImportPath == selectedPackage {
			if row.DepOnly || len(row.Match) != 1 || row.Match[0] != selectedPackage {
				return fmt.Errorf("native export selected root differs from the original command")
			}
			selected++
		} else if !row.DepOnly || len(row.Match) != 0 {
			return fmt.Errorf("native export introduced an extra selected root")
		}
	}
	if selected != 1 {
		return fmt.Errorf("native export lost its original selected root")
	}
	return nil
}

// Preparation correspondence only. The complete compiler command must still
// be replayed against the immutable original module/package namespace before
// any registered context can acquire loader authority.
func (exports practicalPackageExports) bindOriginalSourceTypes(loader *practicalSourceTypes, discovery practicalModuleDiscovery, compiler practicalCompilerImage, sdkCapability string) error {
	if err := discovery.requireOriginalObservation(); err != nil {
		return err
	}
	if err := exports.requireOriginalObservation(); err != nil {
		return err
	}
	if loader == nil || loader.failed != nil || loader.syntax.failed != nil || loader.stdlib == nil || exports.compilerSHA256 != discovery.compilerSHA256 || exports.compilerSHA256 != loader.stdlib.compilerSHA256 || exports.distributionSHA256 != loader.stdlib.distributionSHA256 || exports.distributionSHA256 != compiler.selection.Toolchain.DistributionSHA256 || exports.mainSourceID != discovery.mainSourceID {
		return fmt.Errorf("export preparation crossed original compiler/module/source ownership")
	}
	if loader.packages[exports.requestedPackage] == nil || exports.snapshot.contentSHA256 == "" || exports.snapshot.contentSHA256 != practicalSnapshotDigest(exports.snapshot) || loader.syntax.snapshot.contentSHA256 != practicalSnapshotDigest(loader.syntax.snapshot) {
		return fmt.Errorf("export preparation needs intact original source/export snapshots")
	}
	rows, err := decodePracticalPackageExports(exports.output)
	if err != nil {
		return err
	}
	if err := validatePracticalExportRoot(rows, exports.requestedPackage); err != nil {
		return err
	}
	intrinsics := 0
	for _, row := range rows {
		if row.ImportPath == "unsafe" {
			intrinsics++
		}
	}
	if len(rows) != len(exports.packages) || len(rows) != len(loader.packages)+intrinsics {
		return fmt.Errorf("original compiler export/source package closure is incomplete")
	}
	byPath := map[string]practicalCapturedPackageExport{}
	for _, captured := range exports.packages {
		if _, exists := byPath[captured.importPath]; exists {
			return fmt.Errorf("duplicate original export package")
		}
		byPath[captured.importPath] = captured
	}
	exportIDs := map[string]bool{}
	for _, row := range rows {
		captured, exists := byPath[row.ImportPath]
		if !exists || captured.name != row.Name || captured.buildID != row.BuildID || captured.standard != row.Standard || !reflect.DeepEqual(captured.imports, row.Imports) || !maps.Equal(captured.importMap, row.ImportMap) {
			return fmt.Errorf("export correspondence differs from the original native observation")
		}
		location, err := practicalExportSourceLocation(compiler, discovery, sdkCapability, row)
		if err != nil {
			return err
		}
		sources, err := loader.syntax.snapshot.replayPackageBuild(location)
		if err != nil {
			return err
		}
		if sources.name != row.Name || !reflect.DeepEqual(sources.active, captured.sources) || len(row.GoFiles) != len(sources.active) {
			return fmt.Errorf("export source list differs from the complete original package")
		}
		if strings.Join(sources.embeds, "\x00") != strings.Join(row.EmbedPatterns, "\x00") {
			return fmt.Errorf("native embedded pattern list differs from original source headers")
		}
		embedded, err := loader.syntax.snapshot.replayEmbeddedSources(location, sources.embeds)
		if err != nil {
			return err
		}
		if strings.Join(embedded.files, "\x00") != strings.Join(row.EmbedFiles, "\x00") || !reflect.DeepEqual(embedded.objects, captured.embedded) {
			return fmt.Errorf("embedded file identities differ from original source/compiler selection")
		}
		for i, source := range sources.active {
			if path.Base(source.Path) != row.GoFiles[i] {
				return fmt.Errorf("export source filename differs from the original build selection")
			}
			if _, err := practicalSnapshotBuffer(loader.syntax.snapshot, source); err != nil {
				return err
			}
		}
		var imports []string
		expectedMap := map[string]string{}
		for _, original := range sources.imports {
			canonical := original
			if row.Standard {
				canonical, err = loader.stdlib.resolve(original, row.ImportPath)
				if err != nil {
					return err
				}
			}
			if canonical != original {
				expectedMap[original] = canonical
			}
			imports = append(imports, canonical)
		}
		// Native Go keeps the source-import order when rewriting SDK vendor
		// names. Sorting canonical paths here would change that observation.
		if strings.Join(imports, "\x00") != strings.Join(row.Imports, "\x00") || !maps.Equal(expectedMap, captured.importMap) {
			return fmt.Errorf("export import closure differs from original source imports")
		}
		if row.ImportPath == "unsafe" {
			if !captured.intrinsic || captured.data != nil || row.Export != "" || row.BuildID != "" || loader.packages["unsafe"] != nil {
				return fmt.Errorf("unsafe intrinsic was replaced by an opaque export package")
			}
			continue
		}
		typed := loader.packages[row.ImportPath]
		if typed == nil || typed.types.Name() != row.Name || typed.standardLibrary != row.Standard || !reflect.DeepEqual(typed.sources.active, captured.sources) || captured.intrinsic || captured.data == nil {
			return fmt.Errorf("export is not bound to the whole original source-checked package")
		}
		if len(row.CompiledGoFiles) != len(row.GoFiles) {
			return fmt.Errorf("export lost complete original compiler sources")
		}
		for i, name := range row.CompiledGoFiles {
			if filepath.IsAbs(name) {
				if filepath.Dir(name) != row.Dir {
					return fmt.Errorf("export compiled a generated source")
				}
				name = filepath.Base(name)
			}
			if name != row.GoFiles[i] {
				return fmt.Errorf("export compiled source differs from the original file")
			}
		}
		object := *captured.data
		if object.Kind != "export_data" || object.Capability != "original-go-package-exports" || filepath.Join(exports.buildCacheDirectory, filepath.FromSlash(object.Path)) != row.Export || exportIDs[object.ID] {
			return fmt.Errorf("export bytes crossed their original cache/package identity")
		}
		buffer, err := practicalSnapshotBuffer(exports.snapshot, object)
		if err != nil {
			return err
		}
		if err := validatePracticalExportArchive(buffer.sourceBytes(), row.BuildID); err != nil {
			return err
		}
		exportIDs[object.ID] = true
	}
	if len(exportIDs) != len(exports.snapshot.sourceObjects()) {
		return fmt.Errorf("unbound extra original export data")
	}
	return nil
}
