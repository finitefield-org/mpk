package main

import (
	"bytes"
	"fmt"
	"go/parser"
	"go/token"
	"go/version"
	"io"
	"path"
	"sort"
	"strconv"
	"strings"

	"golang.org/x/mod/modfile"
)

// An owned original compiler-source index, not an installed profile/context or
// a source-library adapter. Every member still needs original-source/proof
// correspondence before it can establish an application claim.
type practicalStdlibSources struct {
	location           practicalModuleLocation
	compilerSHA256     string
	distributionSHA256 string
	moduleSourceID     string
	goVersion          string
	packages           map[string]struct{}
}

func practicalCompilerSourceRole(name string, raw []byte) (string, error) {
	if !strings.HasSuffix(name, ".go") {
		switch path.Base(name) {
		case "go.mod":
			return "go_mod", nil
		case "go.sum":
			return "go_sum", nil
		}
		if strings.HasSuffix(name, "/vendor/modules.txt") {
			return "metadata", nil
		}
		return "auxiliary", nil
	}
	context, err := practicalTargetBuildContext()
	if err != nil {
		return "", err
	}
	fileName := path.Base(name)
	context.OpenFile = func(requested string) (io.ReadCloser, error) {
		if requested != path.Join(practicalBuildDirectory, fileName) {
			return nil, fmt.Errorf("compiler source selection requested another original file")
		}
		return io.NopCloser(bytes.NewReader(raw)), nil
	}
	matched, err := context.MatchFile(practicalBuildDirectory, fileName)
	if err != nil {
		return "", err
	}
	if !matched || strings.HasSuffix(fileName, "_test.go") {
		return "excluded_source", nil
	}
	file, err := parser.ParseFile(token.NewFileSet(), fileName, raw, parser.ImportsOnly|parser.SkipObjectResolution)
	if err != nil {
		return "go_file", nil
	} // retain original invalid source for the actual package checker
	if file.Name.Name == "documentation" {
		return "excluded_source", nil
	}
	for _, spec := range file.Imports {
		importPath, err := strconv.Unquote(spec.Path.Value)
		if err != nil {
			return "go_file", nil
		}
		if importPath == "C" {
			return "excluded_source", nil
		}
	}
	return "go_file", nil
}

func (session *practicalCaptureSession) capturePracticalStandardLibrary(compiler practicalCompilerImage, capability string) error {
	root, exists := session.roots[capability]
	if !exists || root.physical != compiler.selection.GoRoot {
		return session.reject(fmt.Errorf("standard-library source capability is not the original prepared compiler root"))
	}
	if err := compiler.revalidate(); err != nil {
		return session.reject(err)
	}
	for _, expected := range compiler.candidate.ToolchainInventory.Files {
		if !strings.HasPrefix(expected.Path, "go/src/") {
			continue
		}
		relative := strings.TrimPrefix(expected.Path, "go/")
		if expected.Executable {
			return session.reject(fmt.Errorf("compiler source inventory contains an executable member"))
		}
		buffer, err := session.readSourceClassified(capability, relative, "auxiliary", func(raw []byte) (string, error) { return practicalCompilerSourceRole(relative, raw) })
		if err != nil {
			return err
		}
		if buffer.object.RawSHA256 != expected.SHA256 || buffer.object.Bytes != expected.SizeBytes {
			return session.reject(fmt.Errorf("captured standard-library bytes differ from the original compiler inventory"))
		}
	}
	if err := compiler.revalidate(); err != nil {
		return session.reject(err)
	}
	return nil
}

func bindPracticalStandardLibrary(snapshot practicalSourceSnapshot, compiler practicalCompilerImage, capability string) (*practicalStdlibSources, error) {
	digest, err := hashTypedCanonicalJSON(bundleContentDomain, compiler.candidate.ToolchainInventory)
	if err != nil || digest != compiler.candidate.Toolchain.DistributionSHA256 || digest != compiler.selection.Toolchain.DistributionSHA256 {
		return nil, fmt.Errorf("standard-library compiler distribution differs from its complete original inventory")
	}
	index := &practicalStdlibSources{location: practicalModuleLocation{capability: capability, directory: "src"}, distributionSHA256: compiler.selection.Toolchain.DistributionSHA256, packages: map[string]struct{}{}}
	for _, component := range compiler.selection.Toolchain.Components {
		if component.Name == "go" && component.Release == practicalCompilerRelease {
			index.compilerSHA256 = component.BinarySHA256
		}
	}
	if !sha256Pattern.MatchString(index.compilerSHA256) || !sha256Pattern.MatchString(index.distributionSHA256) {
		return nil, fmt.Errorf("standard-library source has no complete prepared compiler identity")
	}
	expectedNames := map[string]bool{}
	for _, expected := range compiler.candidate.ToolchainInventory.Files {
		if !strings.HasPrefix(expected.Path, "go/src/") {
			continue
		}
		relative := strings.TrimPrefix(expected.Path, "go/")
		expectedNames[relative] = true
		id, captured := snapshot.paths[practicalCaptureKey(capability, relative)]
		if !captured {
			return nil, fmt.Errorf("original compiler source %s: %w", relative, errPracticalNotCaptured)
		}
		object := snapshot.buffers[id].object
		buffer, err := practicalSnapshotBuffer(snapshot, object)
		if err != nil {
			return nil, err
		}
		kind, err := practicalCompilerSourceRole(relative, buffer.data)
		if err != nil {
			return nil, err
		}
		if expected.Executable || object.Kind != kind || object.RawSHA256 != expected.SHA256 || object.Bytes != expected.SizeBytes || object.Capability != capability || object.Path != relative {
			return nil, fmt.Errorf("standard-library source identity/role differs from its original compiler inventory")
		}
		if strings.HasSuffix(relative, ".go") {
			directory := path.Dir(strings.TrimPrefix(relative, "src/"))
			if directory != "." {
				index.packages[directory] = struct{}{}
			}
		}
		if relative == "src/go.mod" {
			manifest, err := modfile.Parse(relative, buffer.data, nil)
			if err != nil || manifest.Module == nil || manifest.Module.Mod.Path != "std" || manifest.Go == nil || version.Compare("go"+manifest.Go.Version, practicalCompilerRelease) > 0 {
				return nil, fmt.Errorf("original standard-library module/language identity is incomplete")
			}
			index.goVersion = "go" + manifest.Go.Version
			index.moduleSourceID = object.ID
		}
	}
	for _, object := range snapshot.sourceObjects() {
		if object.Capability == capability && strings.HasPrefix(object.Path, "src/") && !expectedNames[object.Path] {
			return nil, fmt.Errorf("captured compiler source is absent from the complete original inventory")
		}
	}
	for key, entries := range snapshot.directories {
		owner, directory, ok := strings.Cut(key, "\x00")
		if !ok || owner != capability || directory != "src" && !strings.HasPrefix(directory, "src/") {
			continue
		}
		for _, entry := range entries {
			if entry.Type.IsDir() {
				continue
			}
			if entry.Type != 0 || !expectedNames[path.Join(directory, entry.Name)] {
				return nil, fmt.Errorf("captured compiler namespace differs from its complete original file inventory")
			}
		}
	}
	if index.moduleSourceID == "" || index.goVersion == "" {
		return nil, fmt.Errorf("original standard-library go.mod was not captured")
	}
	if len(index.packages) != len(compiler.selection.StandardPackagePaths) {
		return nil, fmt.Errorf("original standard-library package directory inventory differs")
	}
	for name := range compiler.selection.StandardPackagePaths {
		if _, exists := index.packages[name]; !exists {
			return nil, fmt.Errorf("standard-library package namespace differs from prepared compiler")
		}
	}
	return index, nil
}

func newPracticalSourceTypesWithStandardLibrary(snapshot practicalSourceSnapshot, discovery practicalModuleDiscovery, compiler practicalCompilerImage, capability string) (*practicalSourceTypes, error) {
	loader, err := newPracticalSourceTypes(snapshot, discovery, compiler)
	if err != nil {
		return nil, err
	}
	index, err := bindPracticalStandardLibrary(snapshot, compiler, capability)
	if err != nil {
		return nil, err
	}
	if index.compilerSHA256 != discovery.compilerSHA256 {
		return nil, fmt.Errorf("source module selection and standard library belong to different compilers")
	}
	loader.stdlib = index
	return loader, nil
}

func (index *practicalStdlibSources) contains(name string) bool {
	if index == nil {
		return false
	}
	_, exists := index.packages[name]
	return exists
}

func (index *practicalStdlibSources) resolve(name, from string) (string, error) {
	if !index.contains(from) {
		return "", fmt.Errorf("standard-library resolver has no original importing package")
	}
	for current := from; ; current = path.Dir(current) {
		candidate := path.Join(current, "vendor", name)
		if current == "." {
			candidate = "vendor/" + name
		}
		if index.contains(candidate) {
			return candidate, nil
		}
		if current == "." {
			break
		}
	}
	if index.contains(name) {
		return name, nil
	}
	return "", fmt.Errorf("original standard-library import %s: %w", name, errPracticalNotCaptured)
}

func (index *practicalStdlibSources) packagePaths() []string {
	result := make([]string, 0, len(index.packages))
	for name := range index.packages {
		result = append(result, name)
	}
	sort.Strings(result)
	return result
}
