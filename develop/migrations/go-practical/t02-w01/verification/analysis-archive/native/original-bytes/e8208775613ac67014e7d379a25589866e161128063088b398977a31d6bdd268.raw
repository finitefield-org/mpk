package main

import (
	"fmt"
	"path"
	"path/filepath"
	"strings"
)

type practicalModuleLocation struct {
	capability string
	directory  string
}

type practicalCapturedModule struct {
	location practicalModuleLocation
	manifest practicalModuleManifest
	sums     *practicalModuleSums
}

func practicalModuleFile(location practicalModuleLocation, name string) string {
	return path.Join(location.directory, name)
}

func (session *practicalCaptureSession) captureModule(location practicalModuleLocation, main bool) (practicalCapturedModule, error) {
	return capturePracticalModule(session, location, main)
}

func capturePracticalModule(session practicalModuleInputs, location practicalModuleLocation, main bool) (practicalCapturedModule, error) {
	entries, err := session.readDirectory(location.capability, location.directory)
	if err != nil {
		return practicalCapturedModule{}, err
	}
	if main {
		for _, forbidden := range []string{"go.work", "go.work.sum", "vendor"} {
			if practicalNamespaceHas(entries, forbidden) {
				return practicalCapturedModule{}, session.reject(fmt.Errorf("root workspace or vendor input is outside the practical profile"))
			}
		}
	}
	source, err := session.readSource(location.capability, practicalModuleFile(location, "go.mod"), "go_mod")
	if err != nil {
		return practicalCapturedModule{}, err
	}
	manifest, err := parsePracticalModuleManifest(source, main)
	if err != nil {
		return practicalCapturedModule{}, session.reject(err)
	}
	captured := practicalCapturedModule{location: location, manifest: manifest}
	if practicalNamespaceHas(entries, "go.sum") {
		source, err := session.readSource(location.capability, practicalModuleFile(location, "go.sum"), "go_sum")
		if err != nil {
			return practicalCapturedModule{}, err
		}
		sums, err := parsePracticalModuleSums(source)
		if err != nil {
			return practicalCapturedModule{}, session.reject(err)
		}
		captured.sums = &sums
	}
	return captured, nil
}

// Resolve one original directive into an already authorized capability. This
// does not select a replacement/version; Go's resolved graph must still agree.
func (session *practicalCaptureSession) captureLocalReplacement(owner practicalCapturedModule, directive int) (practicalCapturedModule, error) {
	return capturePracticalLocalReplacement(session, owner, directive)
}

func capturePracticalLocalReplacement(session practicalModuleInputs, owner practicalCapturedModule, directive int) (practicalCapturedModule, error) {
	original, err := session.readSource(owner.location.capability, practicalModuleFile(owner.location, "go.mod"), "go_mod")
	if err != nil || original.object != owner.manifest.source.object {
		return practicalCapturedModule{}, session.reject(fmt.Errorf("replacement owner is not an original captured manifest"))
	}
	// Reparse owned original bytes rather than trusting a mutable parser tree.
	manifest, err := parsePracticalModuleManifest(original, owner.manifest.main)
	if err != nil || directive < 0 || directive >= len(manifest.file.Replace) {
		return practicalCapturedModule{}, session.reject(fmt.Errorf("invalid original replacement directive"))
	}
	replacement := manifest.file.Replace[directive].New
	if replacement.Version != "" {
		return practicalCapturedModule{}, session.reject(fmt.Errorf("replacement requires an offline versioned snapshot"))
	}
	location, err := practicalLocalReplacementLocation(session, owner.location, replacement.Path)
	if err != nil {
		return practicalCapturedModule{}, err
	}
	return session.captureModule(location, false)
}

// Both strict selected-module capture and lax graph-only summaries resolve
// the original main directive under the same authorized capability boundary.
func practicalLocalReplacementLocation(session practicalModuleInputs, owner practicalModuleLocation, replacement string) (practicalModuleLocation, error) {
	roots := session.originalRoots()
	ownerRoot := roots[owner.capability]
	target := replacement
	if !filepath.IsAbs(target) {
		target = filepath.Join(ownerRoot, filepath.FromSlash(owner.directory), target)
	}
	target = filepath.Clean(target)
	var selected practicalModuleLocation
	longest := -1
	ambiguous := false
	for capability, root := range roots {
		relative, err := filepath.Rel(root, target)
		if err != nil || relative == ".." || strings.HasPrefix(relative, ".."+string(filepath.Separator)) || filepath.IsAbs(relative) {
			continue
		}
		if len(root) < longest {
			continue
		}
		if len(root) == longest {
			ambiguous = true
			continue
		}
		if relative == "." {
			relative = ""
		}
		selected = practicalModuleLocation{capability: capability, directory: filepath.ToSlash(relative)}
		longest = len(root)
		ambiguous = false
	}
	if longest < 0 {
		return practicalModuleLocation{}, session.reject(fmt.Errorf("replacement escapes every authorized source capability"))
	}
	if ambiguous {
		return practicalModuleLocation{}, session.reject(fmt.Errorf("replacement root capability is ambiguous"))
	}
	return selected, nil
}
