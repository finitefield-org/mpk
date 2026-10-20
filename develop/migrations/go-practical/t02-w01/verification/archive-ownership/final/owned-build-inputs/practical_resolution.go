package main

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"os/exec"
	"path"
	"path/filepath"
	"sort"
	"time"

	"golang.org/x/mod/module"
)

// A preparation observation, not a registered context or a verified immutable
// loader result. The production consumer must replay Go on the sealed namespace.
type practicalModuleDiscovery struct {
	output         []byte
	compilerSHA256 string
	mainSourceID   string
	modules        []practicalResolvedModule
}

type practicalResolvedModule struct {
	identity    module.Version
	replacement module.Version
	manifest    practicalSourceBuffer
	location    *practicalModuleLocation
	versioned   *practicalVersionedModule
}

// Fields of the pinned Go1.25 `list -m -json all` result. Other command modes
// and caller adapters are not accepted as a discovery result.
type practicalGoListedModule struct {
	Path, Version, Dir, GoMod, GoVersion, Sum, GoModSum string
	Main, Indirect                                      bool
	Time                                                *time.Time
	Replace                                             *practicalGoListedModule
	Query, Deprecated                                   string
	Versions, Retracted                                 []string
	Update                                              *practicalGoListedModule
	Origin                                              json.RawMessage
	Reuse                                               bool
	Error                                               *struct{ Err string }
}

type practicalBoundedOutput struct {
	bytes.Buffer
	maximum int
	cancel  context.CancelFunc
}

func (output *practicalBoundedOutput) Write(data []byte) (int, error) {
	if len(data) > output.maximum-output.Len() {
		output.cancel()
		return 0, fmt.Errorf("Go module discovery output exceeds its input bound")
	}
	return output.Buffer.Write(data)
}

func decodePracticalGoModules(raw []byte) ([]practicalGoListedModule, error) {
	if len(raw) > maximumCandidateBytes {
		return nil, fmt.Errorf("Go module output exceeds its metadata byte bound")
	}
	decoder := json.NewDecoder(bytes.NewReader(raw))
	rows := []practicalGoListedModule{}
	seen := map[string]bool{}
	for {
		var encoded json.RawMessage
		if err := decoder.Decode(&encoded); err == io.EOF {
			break
		} else if err != nil {
			return nil, err
		}
		if len(rows) >= maximumManifestInputs {
			return nil, fmt.Errorf("Go module output exceeds its module count bound")
		}
		if _, err := decodeStrictJSON(encoded); err != nil {
			return nil, err
		}
		var row practicalGoListedModule
		closed := json.NewDecoder(bytes.NewReader(encoded))
		closed.DisallowUnknownFields()
		if err := closed.Decode(&row); err != nil {
			return nil, err
		}
		if row.Path == "" || seen[row.Path] || row.Error != nil || row.Replace != nil && (row.Replace.Replace != nil || row.Replace.Error != nil) {
			return nil, fmt.Errorf("Go module output has duplicate, empty or chained module identities")
		}
		seen[row.Path] = true
		rows = append(rows, row)
	}
	if len(rows) == 0 {
		return nil, fmt.Errorf("Go returned no resolved main module")
	}
	return rows, nil
}

func (session *practicalCaptureSession) practicalPhysicalDirectory(location practicalModuleLocation) (string, error) {
	if _, err := session.readDirectory(location.capability, location.directory); err != nil {
		return "", err
	}
	return filepath.Join(session.roots[location.capability].physical, filepath.FromSlash(location.directory)), nil
}

func practicalReplacementDirective(manifest practicalModuleManifest, identity module.Version) (int, error) {
	// This is the original directive lookup, not version selection. Go's
	// replacement rule checks an exact version before a path-wide directive.
	exact, wildcard := -1, -1
	for index, directive := range manifest.file.Replace {
		if directive.Old.Path != identity.Path {
			continue
		}
		if directive.Old.Version == identity.Version {
			if exact >= 0 {
				return -1, fmt.Errorf("duplicate exact original replacement directive")
			}
			exact = index
		} else if directive.Old.Version == "" {
			if wildcard >= 0 {
				return -1, fmt.Errorf("duplicate original path-wide replacement directive")
			}
			wildcard = index
		}
	}
	if exact >= 0 {
		return exact, nil
	}
	return wildcard, nil
}

func (session *practicalCaptureSession) bindPracticalGoModuleRows(main practicalCapturedModule, cache practicalModuleLocation, rows []practicalGoListedModule) ([]practicalResolvedModule, error) {
	original, exists := session.files[practicalCaptureKey(main.location.capability, practicalModuleFile(main.location, "go.mod"))]
	if !exists || original.buffer.object != main.manifest.source.object {
		return nil, session.reject(fmt.Errorf("resolved module owner is not an original captured main manifest"))
	}
	if main.sums != nil {
		checksums, exists := session.files[practicalCaptureKey(main.location.capability, practicalModuleFile(main.location, "go.sum"))]
		if !exists || checksums.buffer.object != main.sums.source.object {
			return nil, session.reject(fmt.Errorf("resolved module checksums do not belong to the original main module"))
		}
	}
	manifest, err := parsePracticalModuleManifest(original.buffer, true)
	if err != nil {
		return nil, session.reject(err)
	}
	mainDirectory, err := session.practicalPhysicalDirectory(main.location)
	if err != nil {
		return nil, err
	}
	if cache.directory != "" && !validPracticalLogicalPath(cache.directory) {
		return nil, session.reject(fmt.Errorf("module cache location is not canonical"))
	}
	modules := make([]practicalResolvedModule, 0, len(rows))
	mainCount := 0
	seen := map[string]bool{}
	for _, row := range rows {
		if seen[row.Path] || row.Error != nil || row.Replace != nil && row.Replace.Error != nil {
			return nil, session.reject(fmt.Errorf("multiple selected versions of one module path"))
		}
		seen[row.Path] = true
		identity := module.Version{Path: row.Path, Version: row.Version}
		if row.Main {
			mainCount++
			if row.Path != manifest.file.Module.Mod.Path || row.Version != "" || row.Replace != nil || row.Dir != mainDirectory || row.GoMod != filepath.Join(mainDirectory, "go.mod") || row.GoVersion != manifest.file.Go.Version {
				return nil, session.reject(fmt.Errorf("resolved main module differs from its original captured input"))
			}
			location := main.location
			modules = append(modules, practicalResolvedModule{identity: identity, manifest: original.buffer, location: &location})
			continue
		}
		if module.Check(identity.Path, identity.Version) != nil || module.CanonicalVersion(identity.Version) != identity.Version {
			return nil, session.reject(fmt.Errorf("resolved dependency module version is not canonical"))
		}
		directiveIndex, err := practicalReplacementDirective(manifest, identity)
		if err != nil {
			return nil, session.reject(err)
		}
		if (directiveIndex >= 0) != (row.Replace != nil) {
			return nil, session.reject(fmt.Errorf("resolved replacement differs from the original main module directive"))
		}
		effective := identity
		fact := row
		resolved := practicalResolvedModule{identity: identity}
		if directiveIndex >= 0 {
			directive := manifest.file.Replace[directiveIndex]
			if row.Replace.Path != directive.New.Path || row.Replace.Version != directive.New.Version || row.Replace.Main || row.Replace.Replace != nil {
				return nil, session.reject(fmt.Errorf("resolved replacement target is not the original exact directive"))
			}
			effective, fact = directive.New, *row.Replace
			resolved.replacement = effective
			if effective.Version == "" {
				captured, err := session.captureLocalReplacement(main, directiveIndex)
				if err != nil {
					return nil, err
				}
				directory, err := session.practicalPhysicalDirectory(captured.location)
				if err != nil {
					return nil, err
				}
				if fact.Dir != directory || fact.GoMod != filepath.Join(directory, "go.mod") || row.Dir != directory || row.GoMod != fact.GoMod {
					return nil, session.reject(fmt.Errorf("Go resolved a local replacement outside its captured original location"))
				}
				if captured.manifest.file.Go != nil && (fact.GoVersion != captured.manifest.file.Go.Version || row.GoVersion != fact.GoVersion) {
					return nil, session.reject(fmt.Errorf("local replacement language metadata differs"))
				}
				resolved.location, resolved.manifest = &captured.location, captured.manifest.source
				modules = append(modules, resolved)
				continue
			}
		}
		if main.sums == nil {
			return nil, session.reject(fmt.Errorf("resolved versioned dependency has no original go.sum"))
		}
		download := practicalModuleLocation{capability: cache.capability, directory: path.Join(cache.directory, "cache/download")}
		captured, err := session.captureVersionedModule(download, effective, *main.sums)
		if err != nil {
			return nil, err
		}
		physicalCache, err := session.practicalPhysicalDirectory(cache)
		if err != nil {
			return nil, err
		}
		goMod := filepath.Join(session.roots[cache.capability].physical, filepath.FromSlash(captured.manifest.source.object.Path))
		escapedPath, _ := module.EscapePath(effective.Path)
		escapedVersion, _ := module.EscapeVersion(effective.Version)
		expectedDirectory := filepath.Join(physicalCache, filepath.FromSlash(escapedPath)+"@"+escapedVersion)
		if fact.GoMod != goMod || row.GoMod != goMod || fact.Dir != "" && fact.Dir != expectedDirectory || row.Dir != fact.Dir {
			return nil, session.reject(fmt.Errorf("resolved versioned source/manifest location differs from original snapshot"))
		}
		if captured.manifest.file.Go != nil && (fact.GoVersion != captured.manifest.file.Go.Version || row.GoVersion != fact.GoVersion) {
			return nil, session.reject(fmt.Errorf("resolved module language metadata differs from original go.mod"))
		}
		sums, err := session.originalModuleSums(*main.sums)
		if err != nil {
			return nil, err
		}
		if fact.Sum != "" && fact.Sum != captured.archive.sum || fact.GoModSum != "" && fact.GoModSum != sums.entries[module.Version{Path: effective.Path, Version: effective.Version + "/go.mod"}] {
			return nil, session.reject(fmt.Errorf("Go checksum observations differ from original captured checksums"))
		}
		resolved.versioned, resolved.manifest = &captured, captured.manifest.source
		modules = append(modules, resolved)
	}
	if mainCount != 1 {
		return nil, session.reject(fmt.Errorf("Go module graph must contain exactly one original main module"))
	}
	sort.Slice(modules, func(i, j int) bool { return modules[i].identity.Path < modules[j].identity.Path })
	return modules, nil
}

// The existing validated launcher supplies only compiler identity here. It
// grants no practical-profile registration or foundation/source proof authority.
func (session *practicalCaptureSession) discoverPracticalModules(mainLocation, cache practicalModuleLocation, selection validatedLauncherSelection) (practicalModuleDiscovery, error) {
	goHash := ""
	for _, component := range selection.Toolchain.Components {
		if component.Name == "go" && component.Kind == "executable" && component.Release == practicalCompilerRelease {
			goHash = component.BinarySHA256
		}
	}
	if !sha256Pattern.MatchString(goHash) || selection.Target.ID != goTarget || selection.Target.PointerWidth != goPointerWidth || selection.GoExecutable != filepath.Join(selection.GoRoot, "bin", executableName("go")) {
		return practicalModuleDiscovery{}, session.reject(fmt.Errorf("module discovery requires the exact validated Go1.25 compiler/target"))
	}
	checkExecutable := func() bool {
		root, err := openPracticalSourceRoot(selection.GoRoot)
		if err != nil {
			return false
		}
		info, err := root.Lstat("bin/" + executableName("go"))
		if err != nil || !info.Mode().IsRegular() {
			_ = root.Close()
			return false
		}
		data, err := readPracticalRegularFile(root, "bin/"+executableName("go"), info, maximumCandidateBytes)
		closeErr := root.Close()
		return err == nil && closeErr == nil && sha256Hex(data) == goHash
	}
	if !checkExecutable() {
		return practicalModuleDiscovery{}, session.reject(fmt.Errorf("selected Go executable differs from its validated identity"))
	}
	main, err := session.captureModule(mainLocation, true)
	if err != nil {
		return practicalModuleDiscovery{}, err
	}
	for index, directive := range main.manifest.file.Replace {
		if directive.New.Version == "" {
			if _, err := session.captureLocalReplacement(main, index); err != nil {
				return practicalModuleDiscovery{}, err
			}
		}
	}
	directory, err := session.practicalPhysicalDirectory(mainLocation)
	if err != nil {
		return practicalModuleDiscovery{}, err
	}
	moduleCache, err := session.practicalPhysicalDirectory(cache)
	if err != nil {
		return practicalModuleDiscovery{}, err
	}
	filesystem, err := newLoaderFilesystem()
	if err != nil {
		return practicalModuleDiscovery{}, session.reject(err)
	}
	defer filesystem.Close()
	environmentFS := filesystem
	environmentFS.goMod = moduleCache
	ctx, cancel := context.WithTimeout(context.Background(), 120*time.Second)
	defer cancel()
	stdout := &practicalBoundedOutput{maximum: maximumCandidateBytes, cancel: cancel}
	stderr := &practicalBoundedOutput{maximum: maximumContractBytes, cancel: cancel}
	command := exec.CommandContext(ctx, selection.GoExecutable, "list", "-mod=readonly", "-m", "-json", "all")
	command.Dir, command.Env = directory, pinnedLoaderEnvironment(selection, environmentFS)
	command.Stdout, command.Stderr = stdout, stderr
	if err := command.Run(); err != nil {
		return practicalModuleDiscovery{}, session.reject(fmt.Errorf("original offline Go module discovery failed: %w", err))
	}
	if !checkExecutable() {
		return practicalModuleDiscovery{}, session.reject(fmt.Errorf("selected Go executable changed during module discovery"))
	}
	rows, err := decodePracticalGoModules(stdout.Bytes())
	if err != nil {
		return practicalModuleDiscovery{}, session.reject(err)
	}
	modules, err := session.bindPracticalGoModuleRows(main, cache, rows)
	if err != nil {
		return practicalModuleDiscovery{}, err
	}
	return practicalModuleDiscovery{output: bytes.Clone(stdout.Bytes()), compilerSHA256: goHash, mainSourceID: main.manifest.source.object.ID, modules: modules}, nil
}
