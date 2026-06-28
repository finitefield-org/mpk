package main

import (
	"bytes"
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"runtime"
	"sort"

	"golang.org/x/tools/go/packages"
)

const (
	sourceManifestSchema   = "mpk.go.source_manifest.v0"
	sourceManifestLanguage = "go"
	frontendName           = "go2gir"
	frontendVersion        = "0.1.0"
)

type sourceManifest struct {
	Schema         string                 `json:"schema"`
	SourceLanguage string                 `json:"source_language"`
	GoVersion      string                 `json:"go_version"`
	Frontend       sourceManifestFrontend `json:"frontend"`
	SourceFiles    []sourceManifestFile   `json:"source_files"`
	SourceHash     string                 `json:"source_hash"`
	GIRHash        string                 `json:"gir_hash"`
}

type sourceManifestFrontend struct {
	Name         string `json:"name"`
	Version      string `json:"version"`
	BinarySHA256 string `json:"binary_sha256"`
}

type sourceManifestFile struct {
	Path   string `json:"path"`
	SHA256 string `json:"sha256"`
}

func buildSourceManifest(loaded packageLoadResult, girHash string) (sourceManifest, error) {
	sourceFiles, err := collectSourceManifestFiles(loaded.BaseDir, loaded.Packages)
	if err != nil {
		return sourceManifest{}, err
	}
	sourceHash, err := hashSourceManifestFiles(sourceFiles)
	if err != nil {
		return sourceManifest{}, err
	}
	frontendHash, err := currentExecutableSHA256()
	if err != nil {
		return sourceManifest{}, err
	}

	return sourceManifest{
		Schema:         sourceManifestSchema,
		SourceLanguage: sourceManifestLanguage,
		GoVersion:      runtime.Version(),
		Frontend: sourceManifestFrontend{
			Name:         frontendName,
			Version:      frontendVersion,
			BinarySHA256: frontendHash,
		},
		SourceFiles: sourceFiles,
		SourceHash:  sourceHash,
		GIRHash:     girHash,
	}, nil
}

func collectSourceManifestFiles(baseDir string, packagesLoaded []*packages.Package) ([]sourceManifestFile, error) {
	seen := make(map[string]sourceManifestFile)
	for _, pkg := range packagesLoaded {
		packageBaseDir := sourceManifestBaseDir(baseDir, pkg)
		for _, sourcePath := range manifestPackageSourceFiles(pkg) {
			manifestPath, resolvedPath, err := manifestSourcePath(packageBaseDir, sourcePath)
			if err != nil {
				return nil, err
			}
			if _, ok := seen[manifestPath]; ok {
				continue
			}
			hash, err := fileSHA256(resolvedPath)
			if err != nil {
				return nil, err
			}
			seen[manifestPath] = sourceManifestFile{
				Path:   manifestPath,
				SHA256: hash,
			}
		}
	}

	sourceFiles := make([]sourceManifestFile, 0, len(seen))
	for _, sourceFile := range seen {
		sourceFiles = append(sourceFiles, sourceFile)
	}
	sort.Slice(sourceFiles, func(i, j int) bool {
		return sourceFiles[i].Path < sourceFiles[j].Path
	})
	if len(sourceFiles) == 0 {
		return nil, fmt.Errorf("source manifest has no Go source files")
	}
	return sourceFiles, nil
}

func sourceManifestBaseDir(defaultBaseDir string, pkg *packages.Package) string {
	if pkg != nil && pkg.Module != nil && pkg.Module.Dir != "" {
		return pkg.Module.Dir
	}
	return defaultBaseDir
}

func manifestPackageSourceFiles(pkg *packages.Package) []string {
	seen := make(map[string]struct{}, len(pkg.GoFiles)+len(pkg.CompiledGoFiles))
	for _, path := range pkg.GoFiles {
		seen[path] = struct{}{}
	}
	for _, path := range pkg.CompiledGoFiles {
		seen[path] = struct{}{}
	}

	paths := make([]string, 0, len(seen))
	for path := range seen {
		paths = append(paths, path)
	}
	sort.Strings(paths)
	return paths
}

func manifestSourcePath(baseDir string, sourcePath string) (string, string, error) {
	if baseDir == "" {
		return "", "", fmt.Errorf("source manifest requires a package base directory")
	}
	baseResolved, err := filepath.EvalSymlinks(baseDir)
	if err != nil {
		return "", "", fmt.Errorf("resolve source manifest base directory %q: %w", baseDir, err)
	}

	path := sourcePath
	if !filepath.IsAbs(path) {
		path = filepath.Join(baseDir, path)
	}
	sourceResolved, err := filepath.EvalSymlinks(path)
	if err != nil {
		return "", "", fmt.Errorf("resolve source file %q: %w", sourcePath, err)
	}
	relative, err := filepath.Rel(baseResolved, sourceResolved)
	if err != nil {
		return "", "", fmt.Errorf("normalize source file %q: %w", sourcePath, err)
	}
	if relative == "." || filepath.IsAbs(relative) || isParentRelativePath(relative) {
		return "", "", fmt.Errorf("source file %q is outside package base directory", sourcePath)
	}
	return filepath.ToSlash(relative), sourceResolved, nil
}

func hashSourceManifestFiles(sourceFiles []sourceManifestFile) (string, error) {
	payload := struct {
		SourceFiles []sourceManifestFile `json:"source_files"`
	}{
		SourceFiles: sourceFiles,
	}
	encoded, err := canonicalJSONBytes(payload)
	if err != nil {
		return "", err
	}
	sum := sha256.Sum256(encoded)
	return hex.EncodeToString(sum[:]), nil
}

func currentExecutableSHA256() (string, error) {
	path, err := os.Executable()
	if err != nil {
		return "", fmt.Errorf("resolve frontend executable: %w", err)
	}
	return fileSHA256(path)
}

func fileSHA256(path string) (string, error) {
	content, err := os.ReadFile(path)
	if err != nil {
		return "", fmt.Errorf("read %q for SHA-256: %w", path, err)
	}
	sum := sha256.Sum256(content)
	return hex.EncodeToString(sum[:]), nil
}

func canonicalJSONBytes(value any) ([]byte, error) {
	var buffer bytes.Buffer
	encoder := json.NewEncoder(&buffer)
	encoder.SetEscapeHTML(false)
	if err := encoder.Encode(value); err != nil {
		return nil, err
	}
	return bytes.TrimSuffix(buffer.Bytes(), []byte("\n")), nil
}
