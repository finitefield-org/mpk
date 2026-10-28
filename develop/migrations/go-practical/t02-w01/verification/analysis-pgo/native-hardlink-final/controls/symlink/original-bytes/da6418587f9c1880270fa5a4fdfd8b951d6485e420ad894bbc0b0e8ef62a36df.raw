package main

import (
	"bytes"
	"context"
	"fmt"
	"os/exec"
	"path/filepath"
	"time"

	"golang.org/x/mod/module"
)

type practicalGoCommandKind uint8

const (
	practicalGoModules practicalGoCommandKind = iota + 1
	practicalGoPackages
)

// Produced only by the controlled command below, never by a caller's JSON or
// a self-hash of supplied metadata. These are preparation observations, not a
// kernel-isolation receipt, a registered context or a source/program proof.
type practicalGoObservation struct {
	kind                                practicalGoCommandKind
	compilerSHA256, distributionSHA256  string
	mainSourceID, requestedPackage      string
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
	digest, err := practicalGoCommandCompiler(selection)
	if err != nil || !sha256Pattern.MatchString(mainSourceID) || !practicalReplayAbsolute(directory) {
		return nil, fmt.Errorf("native preparation has invalid original compiler/main/directory ownership")
	}
	arguments := []string{"list", "-mod=readonly"}
	switch kind {
	case practicalGoModules:
		if requested != "" {
			return nil, fmt.Errorf("native module observation cannot take a package selector")
		}
		arguments = append(arguments, "-m", "-json", "all")
	case practicalGoPackages:
		if module.CheckImportPath(requested) != nil {
			return nil, fmt.Errorf("native package observation needs one exact original import path")
		}
		arguments = append(arguments, "-deps", "-export", "-compiled", "-json", requested)
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
		mainSourceID: mainSourceID, requestedPackage: requested, executable: selection.GoExecutable,
		directory: directory, compilerRoot: selection.GoRoot, buildCache: filesystem.goBuild, moduleCache: filesystem.goMod,
		arguments: append([]string(nil), arguments...), environment: append([]string(nil), environment...),
		output: bytes.Clone(stdout.Bytes()), diagnostics: bytes.Clone(stderr.Bytes()),
	}, nil
}

func (observation *practicalGoObservation) require(kind practicalGoCommandKind, compiler, mainSourceID, requested string, output []byte) error {
	if observation == nil || observation.kind != kind || observation.compilerSHA256 != compiler || observation.mainSourceID != mainSourceID ||
		observation.requestedPackage != requested || !bytes.Equal(observation.output, output) {
		return fmt.Errorf("native Go metadata has no matching original command observation")
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
