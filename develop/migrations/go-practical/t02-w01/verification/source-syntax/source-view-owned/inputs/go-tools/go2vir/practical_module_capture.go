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
	key := practicalCaptureKey(owner.location.capability, practicalModuleFile(owner.location, "go.mod"))
	original, exists := session.files[key]
	if !exists || original.buffer.object != owner.manifest.source.object {
		return practicalCapturedModule{}, session.reject(fmt.Errorf("replacement owner is not an original captured manifest"))
	}
	// Reparse owned original bytes rather than trusting a mutable parser tree.
	manifest, err := parsePracticalModuleManifest(original.buffer, owner.manifest.main)
	if err != nil || directive < 0 || directive >= len(manifest.file.Replace) {
		return practicalCapturedModule{}, session.reject(fmt.Errorf("invalid original replacement directive"))
	}
	replacement := manifest.file.Replace[directive].New
	if replacement.Version != "" {
		return practicalCapturedModule{}, session.reject(fmt.Errorf("replacement requires an offline versioned snapshot"))
	}
	ownerRoot := session.roots[owner.location.capability]
	target := replacement.Path
	if !filepath.IsAbs(target) {
		target = filepath.Join(ownerRoot.physical, filepath.FromSlash(owner.location.directory), target)
	}
	target = filepath.Clean(target)
	var selected practicalModuleLocation
	longest := -1
	ambiguous := false
	for capability, root := range session.roots {
		relative, err := filepath.Rel(root.physical, target)
		if err != nil || relative == ".." || strings.HasPrefix(relative, ".."+string(filepath.Separator)) || filepath.IsAbs(relative) {
			continue
		}
		if len(root.physical) < longest {
			continue
		}
		if len(root.physical) == longest {
			ambiguous = true
			continue
		}
		if relative == "." {
			relative = ""
		}
		selected = practicalModuleLocation{capability: capability, directory: filepath.ToSlash(relative)}
		longest = len(root.physical)
		ambiguous = false
	}
	if longest < 0 {
		return practicalCapturedModule{}, session.reject(fmt.Errorf("replacement escapes every authorized source capability"))
	}
	if ambiguous {
		return practicalCapturedModule{}, session.reject(fmt.Errorf("replacement root capability is ambiguous"))
	}
	return session.captureModule(selected, false)
}
