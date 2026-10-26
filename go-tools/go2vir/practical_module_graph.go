package main

import (
	"bytes"
	"fmt"
	"go/version"
	"path"
	"slices"
	"sort"
	"strings"

	"golang.org/x/mod/modfile"
	"golang.org/x/mod/module"
	"golang.org/x/mod/semver"
)

// Original native observations and original manifests for the initial roots
// and final pruned graph. Preparation derives roots and retains every manifest
// needed by intermediate expansion phases. Independent import remains Rust's
// responsibility; this is not a native filesystem-access trace.
type practicalModuleGraph struct {
	output, diagnostics            []byte
	observation, moduleObservation *practicalGoObservation
	initial, final                 []practicalGraphSummary
	cache                          practicalModuleLocation
}

type practicalGraphSummary struct {
	identity, replacement module.Version
	source                *practicalSourceBuffer // nil only for Go's virtual go/toolchain nodes
	goVersion             string
	pruned                bool
	require               []module.Version
}

func practicalGraphVersion(raw, mainPath string) (module.Version, error) {
	p, v, found := strings.Cut(raw, "@")
	m := module.Version{Path: p, Version: v}
	valid := false
	switch p {
	case "go":
		valid = found && version.IsValid("go"+v)
	case "toolchain":
		valid = found && (v == "default" || version.IsValid(v))
	default:
		valid = !found && p == mainPath || found && module.Check(p, v) == nil && module.CanonicalVersion(v) == v
	}
	if !valid || raw == "" {
		return module.Version{}, fmt.Errorf("original graph has a noncanonical module identity")
	}
	return m, nil
}

func practicalGraphName(m module.Version) string {
	if m.Version == "" {
		return m.Path
	}
	return m.Path + "@" + m.Version
}

// Preserve every original edge, including duplicates and order. Rebuilding the
// entire breadth-first output below also detects disconnected or reordered rows.
func practicalGraphEdges(raw []byte, mainPath string) (map[module.Version][]module.Version, error) {
	if len(raw) == 0 || len(raw) > maximumCandidateBytes || raw[len(raw)-1] != '\n' {
		return nil, fmt.Errorf("original graph output is empty, truncated or oversized")
	}
	edges := map[module.Version][]module.Version{}
	for _, line := range strings.Split(string(raw[:len(raw)-1]), "\n") {
		left, right, ok := strings.Cut(line, " ")
		if !ok || strings.ContainsAny(left+right, " \r\t") {
			return nil, fmt.Errorf("original graph row has invalid framing")
		}
		parent, err := practicalGraphVersion(left, mainPath)
		if err != nil {
			return nil, err
		}
		child, err := practicalGraphVersion(right, mainPath)
		if err != nil {
			return nil, err
		}
		edges[parent] = append(edges[parent], child)
	}
	if len(edges[module.Version{Path: mainPath}]) == 0 {
		return nil, fmt.Errorf("original graph lost its main root requirements")
	}
	return edges, nil
}

func practicalGraphOriginalRoots(main practicalCapturedModule) []module.Version {
	roots := []module.Version{}
	for _, r := range main.manifest.file.Require {
		if !practicalGraphExcluded(main, r.Mod) {
			roots = append(roots, r.Mod)
		}
	}
	roots = append(roots, module.Version{Path: "go", Version: main.manifest.file.Go.Version})
	if toolchain := main.manifest.file.Toolchain; toolchain != nil {
		roots = append(roots, module.Version{Path: "toolchain", Version: toolchain.Name})
	}
	// Native gover.ModSort uses sort.Slice without a raw-label tiebreak.
	// Inventory serialization has its own deterministic byte tiebreak below.
	sort.Slice(roots, func(i, j int) bool {
		if roots[i].Path != roots[j].Path {
			return roots[i].Path < roots[j].Path
		}
		return practicalGraphCompare(roots[i].Path, roots[i].Version, roots[j].Version) < 0
	})
	// loadModFile calls updatePrunedRoots before reading a graph if roots
	// contain a repeated path or the main path. With no loaded graph/package
	// roots, that step uses maxRootVersion and must not read discarded versions.
	selected := map[string]string{main.manifest.file.Module.Mod.Path: ""}
	for _, m := range roots {
		practicalGraphSelect(selected, m)
	}
	seen := map[string]bool{main.manifest.file.Module.Mod.Path: true}
	clean := []module.Version{}
	for _, m := range roots {
		if !seen[m.Path] {
			clean = append(clean, module.Version{Path: m.Path, Version: selected[m.Path]})
			seen[m.Path] = true
		}
	}
	return clean
}

func practicalGraphExcluded(main practicalCapturedModule, m module.Version) bool {
	for _, excluded := range main.manifest.file.Exclude {
		if excluded.Mod == m {
			return true
		}
	}
	return false
}

func practicalSortGraphVersions(modules []module.Version) {
	sort.SliceStable(modules, func(i, j int) bool {
		if modules[i].Path != modules[j].Path {
			return modules[i].Path < modules[j].Path
		}
		order := practicalGraphCompare(modules[i].Path, modules[i].Version, modules[j].Version)
		// Distinct original labels can have equal semantic versions (custom
		// toolchain suffixes and +incompatible). A map iteration must not choose
		// the private inventory's order; native graph output stays untouched.
		return order < 0 || order == 0 && modules[i].Version < modules[j].Version
	})
}

func practicalGraphCompare(modulePath, a, b string) int {
	switch modulePath {
	case "go":
		return version.Compare("go"+a, "go"+b)
	case "toolchain":
		return version.Compare(a, b)
	default:
		return semver.Compare(a, b)
	}
}

type practicalGraphPhase struct {
	summaries []practicalGraphSummary
	selected  map[string]string
}

func practicalGraphSelect(selected map[string]string, m module.Version) {
	old, ok := selected[m.Path]
	if !ok || old != "" && (m.Version == "" || practicalGraphCompare(m.Path, old, m.Version) < 0) {
		selected[m.Path] = m.Version
	}
}

func practicalOriginalGraphSummary(inputs practicalModuleInputs, main practicalCapturedModule, cache practicalModuleLocation, identity module.Version) (practicalGraphSummary, error) {
	result := practicalGraphSummary{identity: identity}
	if identity.Path == "go" || identity.Path == "toolchain" {
		if identity.Path == "go" && version.Compare("go"+identity.Version, "go1.21") >= 0 {
			result.require = []module.Version{{Path: "toolchain", Version: "go" + identity.Version}}
		}
		return result, nil
	}
	effective := identity
	directive, err := practicalReplacementDirective(main.manifest, identity)
	if err != nil {
		return result, inputs.reject(err)
	}
	if directive >= 0 {
		effective = main.manifest.file.Replace[directive].New
		result.replacement = effective
	}
	var source practicalSourceBuffer
	if effective.Version == "" {
		if directive < 0 {
			return result, inputs.reject(fmt.Errorf("graph-only local module has no original replacement directive"))
		}
		location, err := practicalLocalReplacementLocation(inputs, main.location, effective.Path)
		if err != nil {
			return result, err
		}
		source, err = inputs.readSource(location.capability, practicalModuleFile(location, "go.mod"), "go_mod")
		if err != nil {
			return result, err
		}
	} else {
		if main.sums == nil {
			return result, inputs.reject(fmt.Errorf("graph-only module has no original checksums"))
		}
		sums, err := inputs.originalModuleSums(*main.sums)
		if err != nil {
			return result, err
		}
		escapedPath, err := module.EscapePath(effective.Path)
		if err != nil {
			return result, inputs.reject(err)
		}
		escapedVersion, err := module.EscapeVersion(effective.Version)
		if err != nil {
			return result, inputs.reject(err)
		}
		// Only the original .mod is a graph input. An unselected version need
		// not have a ZIP, .info, expanded source directory or package export.
		source, err = inputs.readSource(cache.capability, path.Join(cache.directory, "cache/download", escapedPath, "@v", escapedVersion+".mod"), "go_mod")
		if err != nil {
			return result, err
		}
		if err := sums.verifyManifest(effective, source); err != nil {
			return result, inputs.reject(err)
		}
	}
	if _, err := source.canonicalObject(); err != nil {
		return result, inputs.reject(err)
	}
	// Native Go uses ParseLax for dependency graph summaries. Applying the
	// selected-module parser here would reject valid ignored directives in
	// an unselected old module and shrink the original graph universe.
	file, err := modfile.ParseLax(source.object.Path, source.sourceBytes(), nil)
	if err != nil {
		return result, inputs.reject(err)
	}
	if effective.Version != "" && (file.Module == nil || file.Module.Mod.Path != identity.Path && file.Module.Mod.Path != effective.Path) {
		return result, inputs.reject(fmt.Errorf("graph manifest declares a different original module path"))
	}
	result.source = &source
	if file.Go != nil {
		result.goVersion = file.Go.Version
		result.pruned = version.Compare("go"+file.Go.Version, "go1.17") >= 0
		if version.Compare("go"+file.Go.Version, practicalCompilerRelease) > 0 {
			return result, inputs.reject(fmt.Errorf("graph-only manifest requires an unavailable compiler"))
		}
	}
	for _, r := range file.Require {
		if !practicalGraphExcluded(main, r.Mod) {
			result.require = append(result.require, r.Mod)
		}
	}
	if result.goVersion != "" && version.Compare("go"+result.goVersion, "go1.21") >= 0 {
		result.require = append(result.require, module.Version{Path: "go", Version: result.goVersion})
	}
	return result, nil
}

// Loading is deduplicated by both identity and incoming pruning state. An old
// module can unprune an already visited modern module; cycles must terminate.
// A required leaf is retained even when it emits no native graph row.
func practicalOriginalGraphSummaries(inputs practicalModuleInputs, main practicalCapturedModule, cache practicalModuleLocation, roots []module.Version) ([]practicalGraphSummary, error) {
	phase, err := practicalOriginalGraphPhase(inputs, main, cache, roots)
	return phase.summaries, err
}

func practicalOriginalGraphPhase(inputs practicalModuleInputs, main practicalCapturedModule, cache practicalModuleLocation, roots []module.Version) (practicalGraphPhase, error) {
	type load struct {
		identity module.Version
		pruned   bool
	}
	queue := []load{}
	selected := map[string]string{main.manifest.file.Module.Mod.Path: ""}
	for _, root := range roots {
		queue = append(queue, load{root, true})
		practicalGraphSelect(selected, root)
	}
	seen := map[load]bool{}
	loaded := map[module.Version]practicalGraphSummary{}
	for index := 0; index < len(queue); index++ {
		item := queue[index]
		if seen[item] {
			continue
		}
		seen[item] = true
		summary, exists := loaded[item.identity]
		if !exists {
			var err error
			summary, err = practicalOriginalGraphSummary(inputs, main, cache, item.identity)
			if err != nil {
				return practicalGraphPhase{}, err
			}
			loaded[item.identity] = summary
			for _, r := range summary.require {
				practicalGraphSelect(selected, r)
			}
		}
		if !item.pruned || !summary.pruned {
			for _, r := range summary.require {
				queue = append(queue, load{r, item.pruned && summary.pruned})
			}
		}
	}
	ids := []module.Version{}
	for id := range loaded {
		ids = append(ids, id)
	}
	practicalSortGraphVersions(ids)
	result := make([]practicalGraphSummary, 0, len(ids))
	for _, id := range ids {
		result = append(result, loaded[id])
	}
	return practicalGraphPhase{summaries: result, selected: selected}, nil
}

// expandGraph/updatePrunedRoots without package/add roots: all admitted main
// languages (Go 1.22–1.25) are pruned. Do not add non-root paths or discard old
// input versions before their requirements have participated in selection.
// Each inconsistent iteration upgrades root versions or removes redundant
// paths; the finite original source universe supplies the termination bound.
func practicalOriginalExpandedGraph(inputs practicalModuleInputs, main practicalCapturedModule, cache practicalModuleLocation) ([]module.Version, []practicalGraphSummary, []practicalGraphSummary, error) {
	roots := practicalGraphOriginalRoots(main)
	current, err := practicalOriginalGraphPhase(inputs, main, cache, roots)
	if err != nil {
		return nil, nil, nil, err
	}
	initial := current.summaries
	for {
		consistent := true
		for _, root := range roots {
			if current.selected[root.Path] != root.Version {
				consistent = false
			}
		}
		if consistent {
			return roots, initial, current.summaries, nil
		}
		paths := map[string]bool{main.manifest.file.Module.Mod.Path: true}
		next := []module.Version{}
		for _, root := range roots {
			if !paths[root.Path] {
				v, ok := current.selected[root.Path]
				if !ok {
					return nil, nil, nil, inputs.reject(fmt.Errorf("original root is not selected"))
				}
				next = append(next, module.Version{Path: root.Path, Version: v})
				paths[root.Path] = true
			}
		}
		if slices.Equal(next, roots) {
			return nil, nil, nil, inputs.reject(fmt.Errorf("original root expansion made no progress"))
		}
		roots = next
		current, err = practicalOriginalGraphPhase(inputs, main, cache, roots)
		if err != nil {
			return nil, nil, nil, err
		}
	}
}

func practicalBindGraphOutput(inputs practicalModuleInputs, main practicalCapturedModule, cache practicalModuleLocation, discovery practicalModuleDiscovery, observation *practicalGoObservation) (*practicalModuleGraph, error) {
	if err := discovery.requireOriginalObservation(); err != nil {
		return nil, inputs.reject(err)
	}
	if err := observation.require(practicalGoGraph, discovery.compilerSHA256, discovery.mainSourceID, "", observationOutput(observation)); err != nil {
		return nil, inputs.reject(err)
	}
	previous := discovery.observation
	if observation.directory != previous.directory || observation.moduleCache != previous.moduleCache || observation.compilerRoot != previous.compilerRoot || observation.distributionSHA256 != previous.distributionSHA256 || main.manifest.source.object.ID != discovery.mainSourceID {
		return nil, inputs.reject(fmt.Errorf("native graph crossed its original module/compiler/root observation"))
	}
	cacheDirectory, err := inputs.practicalPhysicalDirectory(cache)
	if err != nil {
		return nil, err
	}
	if cacheDirectory != observation.moduleCache {
		return nil, inputs.reject(fmt.Errorf("native graph crossed its original cache capability"))
	}
	roots, initial, final, err := practicalOriginalExpandedGraph(inputs, main, cache)
	if err != nil {
		return nil, err
	}
	mainID := module.Version{Path: main.manifest.file.Module.Mod.Path}
	edges, err := practicalGraphEdges(observation.output, mainID.Path)
	if err != nil {
		return nil, inputs.reject(err)
	}
	if !slices.Equal(roots, edges[mainID]) {
		return nil, inputs.reject(fmt.Errorf("original graph roots differ from source-derived expansion"))
	}
	derived := map[module.Version][]module.Version{mainID: roots}
	for _, summary := range final {
		derived[summary.identity] = summary.require
	}
	var expected bytes.Buffer
	queue := []module.Version{mainID}
	seen := map[module.Version]bool{mainID: true}
	phase, err := practicalOriginalGraphPhase(inputs, main, cache, roots)
	if err != nil {
		return nil, err
	}
	selected := phase.selected
	delete(selected, "go")
	delete(selected, "toolchain")
	for index := 0; index < len(queue); index++ {
		parent := queue[index]
		for _, child := range derived[parent] {
			fmt.Fprintf(&expected, "%s %s\n", practicalGraphName(parent), practicalGraphName(child))
			if expected.Len() > maximumCandidateBytes {
				return nil, inputs.reject(fmt.Errorf("derived graph output exceeds its original byte bound"))
			}
			if !seen[child] {
				queue = append(queue, child)
				seen[child] = true
			}
		}
	}
	if !bytes.Equal(expected.Bytes(), observation.output) {
		return nil, inputs.reject(fmt.Errorf("complete original graph differs from original manifest requirements/pruning/order"))
	}
	if len(selected) != len(discovery.modules) {
		return nil, inputs.reject(fmt.Errorf("original graph and module list have different selected paths"))
	}
	for _, m := range discovery.modules {
		if v, ok := selected[m.identity.Path]; !ok || v != m.identity.Version {
			return nil, inputs.reject(fmt.Errorf("original graph maximum differs from original module-list version"))
		}
	}
	return &practicalModuleGraph{output: bytes.Clone(observation.output), diagnostics: bytes.Clone(observation.diagnostics), observation: observation, moduleObservation: discovery.observation, initial: initial, final: final, cache: cache}, nil
}

func observationOutput(observation *practicalGoObservation) []byte {
	if observation == nil {
		return nil
	}
	return observation.output
}

// Capture while original capabilities and the fixed compiler still exist.
// There is no caller graph, version-selection callback or command argument.
func (session *practicalCaptureSession) capturePracticalModuleGraph(mainLocation, cache practicalModuleLocation, discovery practicalModuleDiscovery, selection validatedLauncherSelection) (practicalModuleDiscovery, error) {
	if err := discovery.requireOriginalObservation(); err != nil {
		return practicalModuleDiscovery{}, session.reject(err)
	}
	main, err := session.captureModule(mainLocation, true)
	if err != nil {
		return practicalModuleDiscovery{}, err
	}
	directory, err := session.practicalPhysicalDirectory(mainLocation)
	if err != nil {
		return practicalModuleDiscovery{}, err
	}
	moduleCache, err := session.practicalPhysicalDirectory(cache)
	if err != nil {
		return practicalModuleDiscovery{}, err
	}
	digest, err := practicalGoCommandCompiler(selection)
	if err != nil || digest != discovery.compilerSHA256 || main.manifest.source.object.ID != discovery.mainSourceID || directory != discovery.observation.directory || moduleCache != discovery.observation.moduleCache || selection.GoRoot != discovery.observation.compilerRoot || selection.Toolchain.DistributionSHA256 != discovery.observation.distributionSHA256 {
		return practicalModuleDiscovery{}, session.reject(fmt.Errorf("graph preparation crossed original module/compiler ownership"))
	}
	// Retain all expansion inputs before execution, including versions absent
	// from both the initial and final loaded graph. Sealing revalidates them.
	if _, _, _, err := practicalOriginalExpandedGraph(session, main, cache); err != nil {
		return practicalModuleDiscovery{}, err
	}
	filesystem, err := newLoaderFilesystem()
	if err != nil {
		return practicalModuleDiscovery{}, session.reject(err)
	}
	defer filesystem.Close()
	environmentFS := filesystem
	environmentFS.goMod = moduleCache
	observation, err := runPracticalGoObservation(selection, environmentFS, directory, discovery.mainSourceID, practicalGoGraph, "")
	if err != nil {
		return practicalModuleDiscovery{}, session.reject(err)
	}
	graph, err := practicalBindGraphOutput(session, main, cache, discovery, observation)
	if err != nil {
		return practicalModuleDiscovery{}, session.reject(&practicalGoDiscoveryFailure{cause: err, output: bytes.Clone(observation.output), diagnostics: bytes.Clone(observation.diagnostics)})
	}
	discovery.graph = graph
	return discovery, nil
}

func (inputs *practicalImageInputs) restoreOriginalModuleGraph(main practicalCapturedModule, cache practicalModuleLocation, original practicalModuleDiscovery) (*practicalModuleGraph, error) {
	if original.graph == nil {
		return nil, nil
	}
	graph := original.graph
	if graph.moduleObservation != original.observation || !bytes.Equal(graph.output, observationOutput(graph.observation)) || graph.observation == nil || !bytes.Equal(graph.diagnostics, graph.observation.diagnostics) {
		return nil, inputs.reject(fmt.Errorf("original graph has no matching whole command/module observation"))
	}
	return practicalBindGraphOutput(inputs, main, cache, original, graph.observation)
}
