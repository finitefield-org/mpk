package main

import (
	"fmt"
	"go/parser"
	"go/token"
	"os"
	"path"
	"runtime/debug"
	"sort"
	"strings"

	"golang.org/x/mod/module"
)

const practicalFrontendModule = "github.com/finitefield-org/mpk/go-tools/go2vir"

type practicalAnalysisPin struct {
	identity module.Version
	sum      string
	modSum   string
	indirect bool
}

// These are the frontend's existing exact dependencies, not application module
// choices. A different analysis implementation requires a new checked context.
var practicalAnalysisPins = [...]practicalAnalysisPin{
	{module.Version{Path: "golang.org/x/mod", Version: "v0.27.0"}, "h1:kb+q2PyFnEADO2IEF935ehFUXlWiNjJWtRNgBLSfbxQ=", "h1:rWI627Fq0DEoudcK+MBkNkCe0EetEaDSwJJkCcjpazc=", false},
	{module.Version{Path: "golang.org/x/sync", Version: "v0.16.0"}, "h1:ycBJEhp9p4vXvUZNszeOq0kGTPghopOL8q0fq3vstxw=", "h1:1dzgHSNfp02xaA81J2MS99Qcpr2w7fw1gpm99rleRqA=", true},
	{module.Version{Path: "golang.org/x/tools", Version: "v0.36.0"}, "h1:kWS0uv/zsvHEle1LbV5LE8QujrxB3wfQyxHfhOk0Qkg=", "h1:WBDiHKJK8YgLHlcQPYQzNCkUxUypCaa5ZegCVutKm+s=", false},
}

type practicalAnalysisAssetDifference struct {
	sourceID      string
	archiveSHA256 string
	archiveBytes  uint64
	actualSHA256  string
	actualBytes   uint64
}

// The selected build mode determines whether Go consults default.pgo.
type practicalAnalysisPGOMode uint8

const (
	practicalAnalysisPGOOff practicalAnalysisPGOMode = iota
	practicalAnalysisPGOAuto
)

// Complete owned frontend/vendor build inputs and original dependency
// provenance. A registered context and a source-correlated build observation
// must still bind these inputs to an accepted executable.
type practicalAnalysisInputs struct {
	owner          practicalCapturedModule
	vendorManifest practicalSourceBuffer
	modules        []practicalVersionedModule
	sources        []practicalSourceBuffer
	vendor         []practicalSourceBuffer
	pgoMode        practicalAnalysisPGOMode
	defaultPGO     *practicalSourceBuffer
	differences    []practicalAnalysisAssetDifference
}

func practicalAnalysisPinByPath(modulePath string) (practicalAnalysisPin, bool) {
	for _, pin := range practicalAnalysisPins {
		if pin.identity.Path == modulePath {
			return pin, true
		}
	}
	return practicalAnalysisPin{}, false
}

func validatePracticalAnalysisManifest(manifest practicalModuleManifest, sums practicalModuleSums) error {
	file := manifest.file
	if file.Module == nil || file.Module.Mod.Path != practicalFrontendModule || file.Go == nil || file.Go.Version != "1.23.0" || file.Toolchain != nil || len(file.Require) != len(practicalAnalysisPins) || len(file.Replace)+len(file.Exclude)+len(file.Retract) != 0 {
		return fmt.Errorf("analysis frontend manifest is not the exact original pinned build module")
	}
	seen := map[string]bool{}
	for _, requirement := range file.Require {
		pin, exists := practicalAnalysisPinByPath(requirement.Mod.Path)
		if !exists || seen[requirement.Mod.Path] || requirement.Mod != pin.identity || requirement.Indirect != pin.indirect || sums.entries[pin.identity] != pin.sum || sums.entries[module.Version{Path: pin.identity.Path, Version: pin.identity.Version + "/go.mod"}] != pin.modSum {
			return fmt.Errorf("analysis frontend dependencies/checksums differ from the compiled pins")
		}
		seen[requirement.Mod.Path] = true
	}
	return nil
}

func parsePracticalAnalysisVendorManifest(source practicalSourceBuffer) (map[string]bool, error) {
	if source.object.Kind != "metadata" {
		return nil, fmt.Errorf("analysis vendor manifest requires original metadata")
	}
	if _, err := source.canonicalObject(); err != nil {
		return nil, err
	}
	packages, modules := map[string]bool{}, map[string]bool{}
	current, annotated, count := "", false, 0
	for _, raw := range strings.Split(string(source.data), "\n") {
		line := strings.TrimSpace(raw)
		if line == "" {
			continue
		}
		if strings.HasPrefix(line, "##") {
			if current == "" || annotated || line != "## explicit; go 1.23.0" {
				return nil, fmt.Errorf("analysis vendor module annotations differ")
			}
			annotated = true
			continue
		}
		if strings.HasPrefix(line, "#") {
			fields := strings.Fields(line)
			if current != "" && (!annotated || count == 0) || len(fields) != 3 || fields[0] != "#" {
				return nil, fmt.Errorf("analysis vendor module section is incomplete or replaced")
			}
			pin, exists := practicalAnalysisPinByPath(fields[1])
			if !exists || modules[fields[1]] || fields[2] != pin.identity.Version {
				return nil, fmt.Errorf("analysis vendor module version is not pinned")
			}
			current, annotated, count = fields[1], false, 0
			modules[current] = true
			continue
		}
		if current == "" || !annotated || packages[line] || module.CheckImportPath(line) != nil || line != current && !strings.HasPrefix(line, current+"/") || len(packages) >= maximumManifestInputs {
			return nil, fmt.Errorf("analysis vendor package has ambiguous or crossed ownership")
		}
		packages[line] = true
		count++
	}
	if len(modules) != len(practicalAnalysisPins) || !annotated || count == 0 {
		return nil, fmt.Errorf("analysis vendor manifest omits a complete pinned module")
	}
	return packages, nil
}

func validatePracticalAnalysisGoSource(source practicalSourceBuffer) error {
	file, err := parser.ParseFile(token.NewFileSet(), source.object.Path, source.data, parser.SkipObjectResolution|parser.ParseComments)
	if err != nil {
		return err
	}
	for _, group := range file.Comments {
		for _, comment := range group.List {
			if strings.HasPrefix(comment.Text, "//go:embed") {
				return fmt.Errorf("analysis build requires an explicit embedded-input inventory")
			}
		}
	}
	return nil
}

// Resolve the exact Go lookup through the captured root, then preserve the
// native directory spelling. A differently spelled unrelated file is ignored
// on a case-sensitive filesystem, just as Go ignores it.
func (session *practicalCaptureSession) capturePracticalDefaultPGO(location practicalModuleLocation, entries []practicalDirectoryEntry) (*practicalSourceBuffer, error) {
	var resolved string
	var observed os.FileInfo
	selectedRank := 0
	err := session.withRoot(location.capability, func(root practicalRootReader) error {
		requested := practicalModuleFile(location, "default.pgo")
		actual, err := root.Lstat(requested)
		if os.IsNotExist(err) {
			return nil
		}
		if err != nil {
			return err
		}
		// Prefer the exact or case-folded requested spelling, including when
		// another hard link happens to name the same underlying file.
		for _, entry := range entries {
			name := practicalModuleFile(location, entry.Name)
			info, err := root.Lstat(name)
			if err != nil {
				// An unrelated unreadable entry is not a Go PGO input.
				// The resolved entry must still be found and revalidated.
				continue
			}
			rank := 1
			if strings.EqualFold(entry.Name, "default.pgo") {
				rank = 2
			}
			if entry.Name == "default.pgo" {
				rank = 3
			}
			if os.SameFile(actual, info) && rank > selectedRank {
				resolved = name
				selectedRank = rank
			}
		}
		if resolved == "" {
			return fmt.Errorf("resolved Go PGO profile is absent from the original namespace")
		}
		observed = actual
		again, err := root.Lstat(requested)
		if err != nil || !sameFileState(actual, again) {
			return fmt.Errorf("Go PGO lookup changed during original namespace capture")
		}
		return nil
	})
	if err != nil || resolved == "" {
		return nil, err
	}
	source, err := session.readSource(location.capability, resolved, "auxiliary")
	if err != nil {
		return nil, err
	}
	captured, exists := session.files[practicalCaptureKey(location.capability, resolved)]
	if !exists || !sameFileState(observed, captured.info) {
		return nil, session.reject(fmt.Errorf("resolved Go PGO file changed before capture"))
	}
	err = session.withRoot(location.capability, func(root practicalRootReader) error {
		again, err := root.Lstat(practicalModuleFile(location, "default.pgo"))
		if err != nil || !sameFileState(observed, again) {
			return fmt.Errorf("resolved Go PGO file changed after capture")
		}
		return nil
	})
	if err != nil {
		return nil, err
	}
	return &source, nil
}

func (session *practicalCaptureSession) capturePracticalAnalysis(location, downloadRoot practicalModuleLocation) (practicalAnalysisInputs, error) {
	return session.capturePracticalAnalysisForPGO(location, downloadRoot, practicalAnalysisPGOOff)
}

func (session *practicalCaptureSession) capturePracticalAnalysisForPGO(location, downloadRoot practicalModuleLocation, pgoMode practicalAnalysisPGOMode) (practicalAnalysisInputs, error) {
	if pgoMode != practicalAnalysisPGOOff && pgoMode != practicalAnalysisPGOAuto {
		return practicalAnalysisInputs{}, session.reject(fmt.Errorf("unknown analysis PGO mode"))
	}
	owner, err := session.captureModule(location, false)
	if err != nil {
		return practicalAnalysisInputs{}, err
	}
	if owner.sums == nil {
		return practicalAnalysisInputs{}, session.reject(fmt.Errorf("analysis frontend has no original go.sum"))
	}
	if err := validatePracticalAnalysisManifest(owner.manifest, *owner.sums); err != nil {
		return practicalAnalysisInputs{}, session.reject(err)
	}
	result := practicalAnalysisInputs{owner: owner, pgoMode: pgoMode}
	archives := map[string]practicalModuleArchive{}
	for _, pin := range practicalAnalysisPins {
		captured, err := session.captureVersionedModule(downloadRoot, pin.identity, *owner.sums)
		if err != nil {
			return practicalAnalysisInputs{}, err
		}
		archives[pin.identity.Path] = captured.archive
		result.modules = append(result.modules, captured)
	}
	result.vendorManifest, err = session.readSource(location.capability, practicalModuleFile(location, "vendor/modules.txt"), "metadata")
	if err != nil {
		return practicalAnalysisInputs{}, err
	}
	declared, err := parsePracticalAnalysisVendorManifest(result.vendorManifest)
	if err != nil {
		return practicalAnalysisInputs{}, session.reject(err)
	}
	seen := map[string]bool{}
	var visit func(string) error
	visit = func(directory string) error {
		entries, err := session.readDirectory(location.capability, directory)
		if err != nil {
			return err
		}
		for _, entry := range entries {
			relative := path.Join(directory, entry.Name)
			if entry.Type.IsDir() {
				if err := visit(relative); err != nil {
					return err
				}
				continue
			}
			if relative == result.vendorManifest.object.Path {
				continue
			}
			logical := strings.TrimPrefix(relative, practicalModuleFile(location, "vendor")+"/")
			pin := practicalAnalysisPin{}
			for _, candidate := range practicalAnalysisPins {
				if strings.HasPrefix(logical, candidate.identity.Path+"/") {
					pin = candidate
					break
				}
			}
			if pin.identity.Path == "" {
				return fmt.Errorf("analysis vendor file is outside every pinned module")
			}
			kind := "auxiliary"
			if strings.HasSuffix(relative, ".go") {
				kind = "go_file"
			} else if entry.Name != "LICENSE" && entry.Name != "PATENTS" && entry.Name != "TODO" {
				return fmt.Errorf("analysis vendor file is an unmodeled compiler input")
			}
			source, err := session.readSource(location.capability, relative, kind)
			if err != nil {
				return err
			}
			archive := archives[pin.identity.Path]
			memberPath := strings.TrimPrefix(logical, pin.identity.Path+"/")
			member, exists := archive.members[memberPath]
			if !exists {
				return fmt.Errorf("analysis vendor input is not an original module member")
			}
			if kind == "go_file" {
				if !declared[path.Dir(logical)] || strings.HasSuffix(relative, "_test.go") {
					return fmt.Errorf("analysis vendor Go source is outside its declared build packages")
				}
				seen[path.Dir(logical)] = true
				if err := validatePracticalAnalysisGoSource(source); err != nil {
					return err
				}
			}
			if err := archive.verifyContent(memberPath, source); err != nil {
				if entry.Name != "TODO" {
					return err
				}
				// An existing uncompiled TODO has a different final LF. Preserve
				// both identities explicitly; never normalize any input bytes.
				result.differences = append(result.differences, practicalAnalysisAssetDifference{source.object.ID, member.RawSHA256, member.Bytes, source.object.RawSHA256, uint64(source.object.Bytes)})
			}
			result.vendor = append(result.vendor, source)
		}
		return nil
	}
	if err := visit(practicalModuleFile(location, "vendor")); err != nil {
		return practicalAnalysisInputs{}, session.reject(err)
	}
	if len(seen) != len(declared) {
		return practicalAnalysisInputs{}, session.reject(fmt.Errorf("analysis vendor manifest includes an absent package"))
	}
	entries, err := session.readDirectory(location.capability, location.directory)
	if err != nil {
		return practicalAnalysisInputs{}, err
	}
	if pgoMode == practicalAnalysisPGOAuto {
		result.defaultPGO, err = session.capturePracticalDefaultPGO(location, entries)
		if err != nil {
			return practicalAnalysisInputs{}, err
		}
	}
	for _, entry := range entries {
		for _, suffix := range []string{".s", ".S", ".h", ".c", ".cc", ".cpp", ".cxx", ".m", ".f", ".F", ".for", ".f90", ".syso", ".swig", ".swigcxx"} {
			if strings.HasSuffix(entry.Name, suffix) {
				return practicalAnalysisInputs{}, session.reject(fmt.Errorf("analysis frontend has an unmodeled non-Go compiler input"))
			}
		}
		if strings.HasSuffix(entry.Name, ".go") && !strings.HasSuffix(entry.Name, "_test.go") {
			source, err := session.readSource(location.capability, practicalModuleFile(location, entry.Name), "go_file")
			if err != nil {
				return practicalAnalysisInputs{}, err
			}
			if err := validatePracticalAnalysisGoSource(source); err != nil {
				return practicalAnalysisInputs{}, session.reject(err)
			}
			result.sources = append(result.sources, source)
		}
	}
	if len(result.sources) == 0 {
		return practicalAnalysisInputs{}, session.reject(fmt.Errorf("analysis frontend original source is absent"))
	}
	return result, nil
}

func (inputs practicalAnalysisInputs) buildSources() []practicalSourceBuffer {
	sources := []practicalSourceBuffer{inputs.owner.manifest.source, inputs.owner.sums.source, inputs.vendorManifest}
	if inputs.defaultPGO != nil {
		sources = append(sources, *inputs.defaultPGO)
	}
	sources = append(sources, inputs.sources...)
	sources = append(sources, inputs.vendor...)
	sort.Slice(sources, func(i, j int) bool { return sources[i].object.Path < sources[j].object.Path })
	return sources
}

func validatePracticalAnalysisBuildInfo(info *debug.BuildInfo, host string) error {
	if err := validatePracticalGoBuildInfo(info, practicalFrontendModule, host); err != nil {
		return err
	}
	if info.Main.Path != practicalFrontendModule || info.Main.Version != "(devel)" || info.Main.Sum != "" || info.Main.Replace != nil || len(info.Deps) != len(practicalAnalysisPins) {
		return fmt.Errorf("analysis frontend build module/dependency set differs")
	}
	seen := map[string]bool{}
	for _, dependency := range info.Deps {
		if dependency == nil {
			return fmt.Errorf("analysis frontend has an absent build dependency")
		}
		pin, exists := practicalAnalysisPinByPath(dependency.Path)
		if !exists || seen[dependency.Path] || dependency.Version != pin.identity.Version || dependency.Replace != nil || dependency.Sum != "" && dependency.Sum != pin.sum {
			return fmt.Errorf("analysis frontend build dependency differs from original pinned code")
		}
		seen[dependency.Path] = true
	}
	trimmed := false
	for _, setting := range info.Settings {
		if setting.Key == "-trimpath" {
			trimmed = setting.Value == "true"
		}
		for _, flag := range []string{"-asan", "-msan", "-race", "-cover", "-tags", "-gcflags", "-asmflags", "-gccgoflags", "-ldflags", "-pgo", "GOFIPS140"} {
			if setting.Key == flag && setting.Value != "" {
				return fmt.Errorf("analysis frontend has an unregistered build flag")
			}
		}
	}
	if !trimmed {
		return fmt.Errorf("analysis frontend build does not trim source paths")
	}
	return nil
}

// Byte and build-metadata validation only. The release build observation must
// additionally reproduce this binary from the exact captured buildSources.
func validatePracticalAnalysisExecutable(candidate launcherSelection, host string) error {
	if candidate.Frontend.Name != "go2vir" || len(candidate.Frontend.SubordinateBinaries) != 0 {
		return fmt.Errorf("analysis executable is not the complete selected frontend")
	}
	if err := validateFrontendCandidate(candidate); err != nil {
		return err
	}
	before, err := inspectPracticalPhysicalRoot(candidate.FrontendRootPath)
	if err != nil {
		return err
	}
	root, err := openPracticalSourceRoot(candidate.FrontendRootPath)
	if err != nil {
		return err
	}
	info, readErr := readPracticalGoBinaryBuildInfo(root, candidate.FrontendInventory.Files[0], candidate.FrontendExecutablePath)
	closeErr := root.Close()
	after, statErr := os.Lstat(candidate.FrontendRootPath)
	if readErr != nil || closeErr != nil || statErr != nil || !sameFileState(before, after) {
		return fmt.Errorf("analysis executable identity changed during validation")
	}
	return validatePracticalAnalysisBuildInfo(info, host)
}
