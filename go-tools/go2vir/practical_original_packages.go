package main

import (
	"bytes"
	"fmt"
	"maps"
	"sort"
	"sync"
)

// A privately reconstructed, source-typed package universe. It owns its bytes
// and compiler facts independently of preparation callers and the replay image.
// This is not a kernel-execution receipt, registered context, verification
// closure, or source/program proof. Those authorities remain separate.
type practicalOriginalPackages struct {
	mutex                                     sync.Mutex
	inputs                                    *practicalImageInputs
	loader                                    *practicalSourceTypes
	modules                                   []practicalResolvedModule
	moduleGraph                               *practicalModuleGraph
	packages                                  map[string]practicalCapturedPackageExport
	imageDigest, compilerDigest, mainSourceID string
	requestedPackage                          string
	moduleOutput, packageOutput               []byte
	moduleDiagnostics, packageDiagnostics     []byte
	closed                                    bool
}

// There is no caller-supplied typed package, source adapter, import callback or
// replacement command. Rebuild every original package, including unused bodies,
// from the owned image and bind the actual full source/export observation.
func (image *practicalReplayImage) originalPackages(main, cache practicalModuleLocation, discovery practicalModuleDiscovery, exports practicalPackageExports, sdkCapability string) (result *practicalOriginalPackages, failure error) {
	if image == nil || image.compiler == nil {
		return nil, fmt.Errorf("original package universe needs its complete owned compiler image")
	}
	if err := discovery.requireOriginalObservation(); err != nil {
		return nil, err
	}
	if err := exports.requireOriginalObservation(); err != nil {
		return nil, err
	}
	view, err := image.openVerifiedView()
	if err != nil {
		return nil, err
	}
	defer func() {
		if err := view.Close(); failure == nil && err != nil {
			if result != nil {
				result.Close()
			}
			result, failure = nil, err
		}
	}()
	if image.compilerOriginal != discovery.observation.compilerRoot ||
		exports.observation.compilerRoot != discovery.observation.compilerRoot ||
		exports.observation.directory != discovery.observation.directory ||
		exports.observation.distributionSHA256 != discovery.observation.distributionSHA256 ||
		image.compiler.selection.Toolchain.DistributionSHA256 != discovery.observation.distributionSHA256 ||
		view.plan.origins[sdkCapability] != image.compilerOriginal ||
		view.plan.origins["original-go-package-exports"] != exports.observation.buildCache {
		return nil, fmt.Errorf("original packages crossed their original compiler or export capability")
	}
	inputs, err := view.restoreOriginalInputs()
	if err != nil {
		return nil, err
	}
	rebound, err := inputs.restoreOriginalDiscovery(main, cache, discovery)
	if err != nil {
		return nil, err
	}
	// Restore export data from the image, never the preparation caller's buffers.
	// Keep its original whole snapshot identity, including its complete namespace.
	exportSnapshot, err := inputs.snapshotForRoots(exports.snapshot.roots)
	if err != nil || exportSnapshot.contentSHA256 != exports.snapshot.contentSHA256 {
		return nil, fmt.Errorf("original package universe lost its exact export snapshot")
	}
	exports.snapshot = exportSnapshot
	exportRoots := map[string]bool{}
	for _, capability := range exportSnapshot.roots {
		exportRoots[capability] = true
	}
	var sourceRoots []string
	for _, capability := range inputs.snapshot.roots {
		if !exportRoots[capability] {
			sourceRoots = append(sourceRoots, capability)
		}
	}
	sources, err := inputs.snapshotForRoots(sourceRoots)
	if err != nil {
		return nil, err
	}
	loader, err := newPracticalSourceTypesWithStandardLibrary(sources, rebound, *image.compiler, sdkCapability)
	if err != nil {
		return nil, err
	}
	if _, err := loader.typecheck(exports.requestedPackage); err != nil {
		return nil, err
	}
	if err := exports.bindOriginalSourceTypes(loader, rebound, *image.compiler, sdkCapability); err != nil {
		return nil, err
	}
	if err := view.check(); err != nil {
		return nil, err
	}
	result = &practicalOriginalPackages{
		inputs: inputs, loader: loader, modules: rebound.modules, moduleGraph: rebound.graph, packages: map[string]practicalCapturedPackageExport{},
		imageDigest: image.contentSHA256, compilerDigest: image.compiler.selection.Toolchain.DistributionSHA256,
		mainSourceID: rebound.mainSourceID, requestedPackage: exports.requestedPackage,
		moduleOutput: bytes.Clone(rebound.observation.output), packageOutput: bytes.Clone(exports.observation.output),
		moduleDiagnostics: bytes.Clone(rebound.observation.diagnostics), packageDiagnostics: bytes.Clone(exports.observation.diagnostics),
	}
	for _, pkg := range exports.packages {
		result.packages[pkg.importPath] = copyPracticalOriginalPackage(pkg)
	}
	return result, nil
}

func copyPracticalOriginalPackage(pkg practicalCapturedPackageExport) practicalCapturedPackageExport {
	pkg.sources = append([]practicalSourceObject(nil), pkg.sources...)
	pkg.embedded = append([]practicalSourceObject{}, pkg.embedded...)
	pkg.imports = append([]string(nil), pkg.imports...)
	pkg.importMap = maps.Clone(pkg.importMap)
	if pkg.data != nil {
		object := *pkg.data
		pkg.data = &object
	}
	return pkg
}

func (packages *practicalOriginalPackages) inventory() ([]practicalCapturedPackageExport, error) {
	packages.mutex.Lock()
	defer packages.mutex.Unlock()
	if packages.closed {
		return nil, fmt.Errorf("original package universe is closed")
	}
	if packages.inputs.failed != nil {
		return nil, packages.inputs.failed
	}
	names := make([]string, 0, len(packages.packages))
	for name := range packages.packages {
		names = append(names, name)
	}
	sort.Strings(names)
	result := make([]practicalCapturedPackageExport, 0, len(names))
	for _, name := range names {
		result = append(result, copyPracticalOriginalPackage(packages.packages[name]))
	}
	return result, nil
}

func (packages *practicalOriginalPackages) originalBytes(object practicalSourceObject) (practicalSourceBuffer, error) {
	packages.mutex.Lock()
	defer packages.mutex.Unlock()
	if packages.closed {
		return practicalSourceBuffer{}, fmt.Errorf("original package universe is closed")
	}
	buffer, err := packages.inputs.readSource(object.Capability, object.Path, object.Kind)
	if err != nil {
		return practicalSourceBuffer{}, err
	}
	if buffer.object != object {
		return practicalSourceBuffer{}, packages.inputs.reject(fmt.Errorf("original package byte request crossed its complete source identity"))
	}
	return buffer, nil
}

func (packages *practicalOriginalPackages) Close() {
	packages.mutex.Lock()
	defer packages.mutex.Unlock()
	if packages.closed {
		return
	}
	packages.closed = true
	for _, buffer := range packages.inputs.snapshot.buffers {
		clear(buffer.data)
	}
	clear(packages.moduleOutput)
	clear(packages.packageOutput)
	clear(packages.moduleDiagnostics)
	clear(packages.packageDiagnostics)
	if packages.moduleGraph != nil {
		clear(packages.moduleGraph.output)
		clear(packages.moduleGraph.diagnostics)
		for _, summaries := range [][]practicalGraphSummary{packages.moduleGraph.initial, packages.moduleGraph.final} {
			for _, summary := range summaries {
				if summary.source != nil {
					clear(summary.source.data)
				}
			}
		}
		packages.moduleGraph = nil
	}
	packages.inputs, packages.loader, packages.modules, packages.packages = nil, nil, nil, nil
}
