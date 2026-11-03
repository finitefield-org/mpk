package main

import (
	"bytes"
	"context"
	"fmt"
	"os/exec"
	"path/filepath"
	"slices"
	"sort"
	"strings"
	"time"

	"golang.org/x/mod/module"
)

type practicalGoCommandKind uint8

const (
	practicalGoModules practicalGoCommandKind = iota + 1
	practicalGoPackages
	practicalGoGraph
)

// Produced only by the controlled command below, never by a caller's JSON or
// a self-hash of supplied metadata. These are preparation observations, not a
// kernel-isolation receipt, a registered context or a source/program proof.
type practicalGoObservation struct {
	kind                                practicalGoCommandKind
	compilerSHA256, distributionSHA256  string
	mainSourceID, requestedPackage      string
	requestedPackages                   []string
	candidateRequest                    []byte
	executable, directory, compilerRoot string
	buildCache, moduleCache             string
	arguments, environment              []string
	output, diagnostics                 []byte
}

func practicalGoCommandCompiler(selection validatedLauncherSelection) (string, error) {
	goHash, count := "", 0
	for _, component := range selection.Toolchain.Components {
		if component.Name == "go" {
			count++
			if component.Kind != "executable" || component.Release != practicalCompilerRelease {
				return "", fmt.Errorf("native preparation has a crossed Go compiler component")
			}
			goHash = component.BinarySHA256
		}
	}
	if count != 1 || !sha256Pattern.MatchString(goHash) || selection.Target.ID != goTarget || selection.Target.PointerWidth != goPointerWidth ||
		!practicalReplayAbsolute(selection.GoRoot) || selection.GoExecutable != filepath.Join(selection.GoRoot, "bin", executableName("go")) {
		return "", fmt.Errorf("native preparation needs the exact Go1.25 compiler and target")
	}
	return goHash, nil
}

func practicalGoExecutableMatches(selection validatedLauncherSelection, digest string) bool {
	root, err := openPracticalSourceRoot(selection.GoRoot)
	if err != nil {
		return false
	}
	info, err := root.Lstat("bin/" + executableName("go"))
	if err != nil || !info.Mode().IsRegular() {
		_ = root.Close()
		return false
	}
	raw, err := readPracticalRegularFile(root, "bin/"+executableName("go"), info, maximumCandidateBytes)
	closeErr := root.Close()
	return err == nil && closeErr == nil && sha256Hex(raw) == digest
}

func runPracticalGoObservation(selection validatedLauncherSelection, filesystem loaderFilesystem, directory, mainSourceID string, kind practicalGoCommandKind, requested string) (*practicalGoObservation, error) {
	return runPracticalGoObservationExact(selection, filesystem, directory, mainSourceID, kind, requested, nil, nil)
}

// Package arguments are derived only from the complete owned request. Multiple
// functions in one package still require only one exact go list pattern.
func practicalCandidatePackagePaths(request *practicalCandidateRequest) ([]string, error) {
	roots, err := practicalCandidateRequestRoots(request)
	if err != nil {
		return nil, err
	}
	seen := make(map[string]bool, len(roots))
	packages := make([]string, 0, len(roots))
	for _, root := range roots {
		if err := module.CheckImportPath(root.Package); err != nil {
			return nil, fmt.Errorf("candidate package is not an exact Go import path: %w", err)
		}
		// These spellings are Go command patterns, not exact list arguments.
		// Exact loading of a real path containing "..." needs a separate
		// source-root-based route and is not claimed by this preparation.
		if strings.Contains(root.Package, "...") || root.Package == "all" || root.Package == "std" || root.Package == "cmd" || root.Package == "tool" || root.Package == "work" {
			return nil, fmt.Errorf("candidate package cannot be passed as an exact Go list argument")
		}
		if !seen[root.Package] {
			seen[root.Package] = true
			packages = append(packages, root.Package)
		}
	}
	sort.Strings(packages)
	return packages, nil
}

func runPracticalGoCandidatePackageObservation(selection validatedLauncherSelection, filesystem loaderFilesystem, directory, mainSourceID string, request *practicalCandidateRequest) (*practicalGoObservation, error) {
	packages, err := practicalCandidatePackagePaths(request)
	if err != nil {
		return nil, err
	}
	requestBytes := request.requestBytes()
	observation, err := runPracticalGoObservationExact(selection, filesystem, directory, mainSourceID, practicalGoPackages, "", packages, requestBytes)
	if err != nil {
		return nil, err
	}
	if err := observation.requireCandidatePackages(request, observation.compilerSHA256, mainSourceID, observation.output); err != nil {
		return nil, err
	}
	return observation, nil
}

func runPracticalGoObservationExact(selection validatedLauncherSelection, filesystem loaderFilesystem, directory, mainSourceID string, kind practicalGoCommandKind, requested string, packages []string, candidate []byte) (*practicalGoObservation, error) {
	digest, err := practicalGoCommandCompiler(selection)
	if err != nil || !sha256Pattern.MatchString(mainSourceID) || !practicalReplayAbsolute(directory) {
		return nil, fmt.Errorf("native preparation has invalid original compiler/main/directory ownership")
	}
	arguments := []string{"list", "-mod=readonly"}
	switch kind {
	case practicalGoModules:
		if requested != "" || len(packages) != 0 || len(candidate) != 0 {
			return nil, fmt.Errorf("native module observation cannot take a package selector")
		}
		arguments = append(arguments, "-m", "-json", "all")
	case practicalGoPackages:
		if len(candidate) == 0 {
			if len(packages) != 0 || module.CheckImportPath(requested) != nil {
				return nil, fmt.Errorf("native package observation needs one exact original import path")
			}
			arguments = append(arguments, "-deps", "-export", "-compiled", "-json", requested)
		} else {
			if requested != "" || len(packages) == 0 {
				return nil, fmt.Errorf("native candidate observation needs its complete package set")
			}
			arguments = append(arguments, "-deps", "-export", "-compiled", "-json")
			arguments = append(arguments, packages...)
		}
	case practicalGoGraph:
		if requested != "" || len(packages) != 0 || len(candidate) != 0 {
			return nil, fmt.Errorf("native graph observation cannot take a selector")
		}
		// Go1.25's mod graph does not accept -mod. Its fixed offline
		// environment and original source/checksum capture remain required.
		arguments = []string{"mod", "graph"}
	default:
		return nil, fmt.Errorf("unknown native Go observation command")
	}
	if !practicalGoExecutableMatches(selection, digest) {
		return nil, fmt.Errorf("native preparation executable differs from its pinned identity")
	}
	ctx, cancel := context.WithTimeout(context.Background(), 120*time.Second)
	defer cancel()
	stdout := &practicalBoundedOutput{maximum: maximumCandidateBytes, cancel: cancel}
	stderr := &practicalBoundedOutput{maximum: maximumContractBytes, cancel: cancel}
	environment := pinnedLoaderEnvironment(selection, filesystem)
	command := exec.CommandContext(ctx, selection.GoExecutable, arguments...)
	command.Dir, command.Env = directory, environment
	command.Stdout, command.Stderr = stdout, stderr
	if err := command.Run(); err != nil {
		return nil, &practicalGoDiscoveryFailure{cause: err, output: bytes.Clone(stdout.Bytes()), diagnostics: bytes.Clone(stderr.Bytes())}
	}
	if !practicalGoExecutableMatches(selection, digest) {
		return nil, &practicalGoDiscoveryFailure{cause: fmt.Errorf("native Go executable changed during preparation"), output: bytes.Clone(stdout.Bytes()), diagnostics: bytes.Clone(stderr.Bytes())}
	}
	return &practicalGoObservation{
		kind: kind, compilerSHA256: digest, distributionSHA256: selection.Toolchain.DistributionSHA256,
		mainSourceID: mainSourceID, requestedPackage: requested, requestedPackages: append([]string(nil), packages...),
		candidateRequest: bytes.Clone(candidate), executable: selection.GoExecutable,
		directory: directory, compilerRoot: selection.GoRoot, buildCache: filesystem.goBuild, moduleCache: filesystem.goMod,
		arguments: append([]string(nil), arguments...), environment: append([]string(nil), environment...),
		output: bytes.Clone(stdout.Bytes()), diagnostics: bytes.Clone(stderr.Bytes()),
	}, nil
}

func (observation *practicalGoObservation) require(kind practicalGoCommandKind, compiler, mainSourceID, requested string, output []byte) error {
	if observation == nil || observation.kind != kind || observation.compilerSHA256 != compiler || observation.mainSourceID != mainSourceID ||
		observation.requestedPackage != requested || len(observation.requestedPackages) != 0 || len(observation.candidateRequest) != 0 || !bytes.Equal(observation.output, output) {
		return fmt.Errorf("native Go metadata has no matching original command observation")
	}
	return nil
}

func (observation *practicalGoObservation) requireCandidatePackages(request *practicalCandidateRequest, compiler, mainSourceID string, output []byte) error {
	packages, err := practicalCandidatePackagePaths(request)
	if err != nil {
		return fmt.Errorf("native package metadata has no matching complete candidate request")
	}
	arguments := append([]string{"list", "-mod=readonly", "-deps", "-export", "-compiled", "-json"}, packages...)
	if observation == nil || observation.kind != practicalGoPackages || observation.compilerSHA256 != compiler ||
		observation.mainSourceID != mainSourceID || observation.requestedPackage != "" || !slices.Equal(observation.requestedPackages, packages) ||
		!slices.Equal(observation.arguments, arguments) || !bytes.Equal(observation.candidateRequest, request.requestBytes()) || !bytes.Equal(observation.output, output) {
		return fmt.Errorf("native package metadata has no matching complete candidate request")
	}
	return nil
}

func (discovery practicalModuleDiscovery) requireOriginalObservation() error {
	return discovery.observation.require(practicalGoModules, discovery.compilerSHA256, discovery.mainSourceID, "", discovery.output)
}

func (exports practicalPackageExports) requireOriginalObservation() error {
	if err := exports.observation.require(practicalGoPackages, exports.compilerSHA256, exports.mainSourceID, exports.requestedPackage, exports.output); err != nil {
		return err
	}
	if exports.distributionSHA256 != exports.observation.distributionSHA256 || exports.buildCacheDirectory != exports.observation.buildCache ||
		!bytes.Equal(exports.diagnostics, exports.observation.diagnostics) {
		return fmt.Errorf("native export observation crossed original compiler/cache/diagnostic ownership")
	}
	return nil
}
