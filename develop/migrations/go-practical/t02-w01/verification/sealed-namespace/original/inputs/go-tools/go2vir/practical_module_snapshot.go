package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"path"
	"time"

	"golang.org/x/mod/module"
)

// Version selection and replacement ownership still come from Go's resolved
// graph. This type binds one already selected identity to original offline data.
type practicalVersionedModule struct {
	identity module.Version
	manifest practicalModuleManifest
	info     practicalSourceBuffer
	zipHash  practicalSourceBuffer
	archive  practicalModuleArchive
}

// The source remains an opaque original Go cache file. Decode Go's fields to
// check the version/time/origin types; unknown Go cache fields retain their raw
// identity and grant no MPK artifact or VCS authority.
func validatePracticalModuleInfo(source practicalSourceBuffer, identity module.Version) error {
	if source.object.Kind != "metadata" {
		return fmt.Errorf("module info requires original metadata bytes")
	}
	if _, err := source.canonicalObject(); err != nil {
		return err
	}
	if _, err := decodeStrictJSON(source.data); err != nil {
		return err
	}
	var info struct {
		Version string
		Time    time.Time
		Origin  *struct {
			VCS, URL, Subdir, Hash, TagPrefix, TagSum, Ref, RepoSum string
		}
	}
	if err := json.Unmarshal(source.data, &info); err != nil || info.Version != identity.Version {
		return fmt.Errorf("module info version or original Go metadata types differ")
	}
	return nil
}

func (session *practicalCaptureSession) captureVersionedModule(downloadRoot practicalModuleLocation, identity module.Version, suppliedSums practicalModuleSums) (practicalVersionedModule, error) {
	if module.Check(identity.Path, identity.Version) != nil || module.CanonicalVersion(identity.Version) != identity.Version ||
		downloadRoot.directory != "" && !validPracticalLogicalPath(downloadRoot.directory) {
		return practicalVersionedModule{}, session.reject(fmt.Errorf("invalid selected module or original download-cache location"))
	}
	sums, err := session.originalModuleSums(suppliedSums)
	if err != nil {
		return practicalVersionedModule{}, err
	}
	escapedPath, err := module.EscapePath(identity.Path)
	if err != nil {
		return practicalVersionedModule{}, session.reject(err)
	}
	escapedVersion, err := module.EscapeVersion(identity.Version)
	if err != nil {
		return practicalVersionedModule{}, session.reject(err)
	}
	prefix := path.Join(downloadRoot.directory, escapedPath, "@v", escapedVersion)
	modSource, err := session.readSource(downloadRoot.capability, prefix+".mod", "go_mod")
	if err != nil {
		return practicalVersionedModule{}, err
	}
	if err := sums.verifyManifest(identity, modSource); err != nil {
		return practicalVersionedModule{}, session.reject(err)
	}
	manifest, err := parsePracticalModuleManifest(modSource, false)
	if err != nil {
		return practicalVersionedModule{}, session.reject(err)
	}
	info, err := session.readSource(downloadRoot.capability, prefix+".info", "metadata")
	if err != nil {
		return practicalVersionedModule{}, err
	}
	if err := validatePracticalModuleInfo(info, identity); err != nil {
		return practicalVersionedModule{}, session.reject(err)
	}
	zipHash, err := session.readSource(downloadRoot.capability, prefix+".ziphash", "metadata")
	if err != nil {
		return practicalVersionedModule{}, err
	}
	if string(bytes.TrimSpace(zipHash.data)) != sums.entries[identity] || sums.entries[identity] == "" {
		return practicalVersionedModule{}, session.reject(fmt.Errorf("missing or stale original ZIP hash metadata"))
	}
	archive, err := session.captureModuleArchive(downloadRoot.capability, prefix+".zip", identity, suppliedSums)
	if err != nil {
		return practicalVersionedModule{}, err
	}
	if _, present := archive.members["go.mod"]; present {
		if err := archive.verifyContent("go.mod", modSource); err != nil {
			return practicalVersionedModule{}, session.reject(fmt.Errorf("module ZIP root manifest differs from original cached go.mod"))
		}
	}
	// Legacy ZIPs may lack go.mod. Retain the separately captured/checksummed
	// original .mod; never insert or rewrite a manifest inside the archive.
	return practicalVersionedModule{identity: identity, manifest: manifest, info: info, zipHash: zipHash, archive: archive}, nil
}
