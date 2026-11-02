package main

import (
	"fmt"
	"path/filepath"

	"golang.org/x/mod/module"
)

// Original collection and immutable image consumption share the complete
// binding rules. This interface neither selects module versions nor supplies
// registered context, compiler-execution or source-proof authority.
type practicalModuleInputs interface {
	readSource(string, string, string) (practicalSourceBuffer, error)
	readDirectory(string, string) ([]practicalDirectoryEntry, error)
	originalRoots() map[string]string
	reject(error) error
	originalModuleSums(practicalModuleSums) (practicalModuleSums, error)
	captureModuleArchive(string, string, module.Version, practicalModuleSums) (practicalModuleArchive, error)
	captureModule(practicalModuleLocation, bool) (practicalCapturedModule, error)
	captureLocalReplacement(practicalCapturedModule, int) (practicalCapturedModule, error)
	captureVersionedModule(practicalModuleLocation, module.Version, practicalModuleSums) (practicalVersionedModule, error)
	practicalPhysicalDirectory(practicalModuleLocation) (string, error)
}

func (session *practicalCaptureSession) originalRoots() map[string]string {
	roots := make(map[string]string, len(session.roots))
	for capability, root := range session.roots {
		roots[capability] = root.physical
	}
	return roots
}

func practicalModulePhysicalDirectory(inputs practicalModuleInputs, location practicalModuleLocation) (string, error) {
	if _, err := inputs.readDirectory(location.capability, location.directory); err != nil {
		return "", err
	}
	root, exists := inputs.originalRoots()[location.capability]
	if !exists || !practicalReplayAbsolute(root) || location.directory != "" && !validPracticalLogicalPath(location.directory) {
		return "", inputs.reject(fmt.Errorf("module location has no original authorized root"))
	}
	return filepath.Join(root, filepath.FromSlash(location.directory)), nil
}
