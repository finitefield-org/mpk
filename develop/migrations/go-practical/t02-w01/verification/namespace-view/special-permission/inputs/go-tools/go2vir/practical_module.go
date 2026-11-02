package main

import (
	"crypto/sha256"
	"encoding/base64"
	"fmt"
	"go/version"
	"strings"
	"unicode/utf8"

	"golang.org/x/mod/modfile"
	"golang.org/x/mod/module"
)

const practicalCompilerRelease = "go1.25.0"

// Parsed original inputs, not a selected module graph or a registered context.
// Keep the parser tree private: Go's resolver, not this parser, selects versions.
type practicalModuleManifest struct {
	source   practicalSourceBuffer
	file     *modfile.File
	language string
	main     bool
}

type practicalModuleSums struct {
	source  practicalSourceBuffer
	entries map[module.Version]string
}

func practicalProofLanguage(value string) bool {
	switch value {
	case "go1.22", "go1.23", "go1.24", "go1.25":
		return true
	}
	return false
}

func parsePracticalModuleManifest(source practicalSourceBuffer, main bool) (practicalModuleManifest, error) {
	if source.object.Kind != "go_mod" {
		return practicalModuleManifest{}, fmt.Errorf("module manifest requires original go_mod bytes")
	}
	if _, err := source.canonicalObject(); err != nil {
		return practicalModuleManifest{}, err
	}
	parsed, err := modfile.Parse(source.object.Path, source.sourceBytes(), nil)
	if err != nil || parsed.Module == nil || module.CheckImportPath(parsed.Module.Mod.Path) != nil ||
		oneOf(parsed.Module.Mod.Path, "main", "all", "std", "cmd") {
		return practicalModuleManifest{}, fmt.Errorf("invalid original module manifest")
	}
	// Additional directive families need a frozen disposition before admission.
	if len(parsed.Godebug) != 0 || len(parsed.Tool) != 0 || len(parsed.Ignore) != 0 {
		return practicalModuleManifest{}, fmt.Errorf("module manifest contains an unmodeled directive")
	}
	if main && parsed.Go == nil {
		return practicalModuleManifest{}, fmt.Errorf("main module requires an explicit go directive")
	}
	language := ""
	if parsed.Go != nil {
		required := "go" + parsed.Go.Version
		if !version.IsValid(required) || version.Compare(required, practicalCompilerRelease) > 0 {
			return practicalModuleManifest{}, fmt.Errorf("module requires an unavailable compiler version")
		}
		// Check the full patch requirement before deriving the language minor.
		language = version.Lang(required)
	}
	if main && !practicalProofLanguage(language) {
		return practicalModuleManifest{}, fmt.Errorf("main module language is outside the practical profile")
	}
	if parsed.Toolchain != nil {
		required := parsed.Toolchain.Name
		if required != "default" && (!version.IsValid(required) || main && version.Compare(required, practicalCompilerRelease) > 0) {
			return practicalModuleManifest{}, fmt.Errorf("module toolchain suggestion is unavailable or invalid")
		}
		// Dependency toolchain suggestions do not select the main toolchain.
	}
	return practicalModuleManifest{source: source, file: parsed, language: language, main: main}, nil
}

func parsePracticalModuleSums(source practicalSourceBuffer) (practicalModuleSums, error) {
	if source.object.Kind != "go_sum" {
		return practicalModuleSums{}, fmt.Errorf("module checksums require original go_sum bytes")
	}
	if _, err := source.canonicalObject(); err != nil {
		return practicalModuleSums{}, err
	}
	if !utf8.Valid(source.data) {
		return practicalModuleSums{}, fmt.Errorf("module checksums are not UTF8")
	}
	sums := practicalModuleSums{source: source, entries: make(map[module.Version]string)}
	for _, line := range strings.Split(string(source.data), "\n") {
		fields := strings.Fields(line)
		if len(fields) == 0 {
			continue
		}
		if len(fields) != 3 {
			return practicalModuleSums{}, fmt.Errorf("invalid module checksum line")
		}
		path, rawVersion, hash := fields[0], fields[1], fields[2]
		moduleVersion := strings.TrimSuffix(rawVersion, "/go.mod")
		if module.Check(path, moduleVersion) != nil || module.CanonicalVersion(moduleVersion) != moduleVersion ||
			!strings.HasPrefix(hash, "h1:") {
			return practicalModuleSums{}, fmt.Errorf("invalid module checksum identity")
		}
		decoded, err := base64.StdEncoding.Strict().DecodeString(strings.TrimPrefix(hash, "h1:"))
		if err != nil || len(decoded) != sha256.Size || "h1:"+base64.StdEncoding.EncodeToString(decoded) != hash {
			return practicalModuleSums{}, fmt.Errorf("invalid module h1 checksum")
		}
		key := module.Version{Path: path, Version: rawVersion}
		if old, exists := sums.entries[key]; exists && old != hash {
			return practicalModuleSums{}, fmt.Errorf("conflicting checksums for one module version")
		}
		// Identical duplicate lines are harmless; retain every original byte.
		sums.entries[key] = hash
	}
	return sums, nil
}

func (sums practicalModuleSums) verifyManifest(identity module.Version, manifest practicalSourceBuffer) error {
	if module.Check(identity.Path, identity.Version) != nil || module.CanonicalVersion(identity.Version) != identity.Version {
		return fmt.Errorf("invalid versioned manifest identity")
	}
	if manifest.object.Kind != "go_mod" {
		return fmt.Errorf("manifest checksum requires original go_mod bytes")
	}
	if _, err := manifest.canonicalObject(); err != nil {
		return err
	}
	// Go's dirhash.Hash1 hashes the sorted "hex  name\n" summary. A module
	// manifest has one member named go.mod, independent of its cache filename.
	summary := sha256.Sum256([]byte(manifest.object.RawSHA256 + "  go.mod\n"))
	expected := "h1:" + base64.StdEncoding.EncodeToString(summary[:])
	if sums.entries[module.Version{Path: identity.Path, Version: identity.Version + "/go.mod"}] != expected {
		return fmt.Errorf("missing or stale original module manifest checksum")
	}
	return nil
}
