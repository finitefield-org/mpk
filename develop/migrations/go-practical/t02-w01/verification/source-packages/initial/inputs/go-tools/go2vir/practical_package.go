package main

import (
	"bytes"
	"fmt"
	"go/build"
	"io"
	"io/fs"
	"path"
	"runtime"
	"sort"
	"strconv"
	"strings"
	"time"
)

// Default Linux/amd64 Go1.25.0 experiments from the exact original
// internal/buildcfg/exp.go, plus GOAMD64=v1. In particular dwarf5 is enabled
// on the target even when this frontend is built for the Darwin development
// host. Neither build.Default nor ambient tags/experiments select source files.
var practicalTargetToolTags = [...]string{
	"goexperiment.regabiwrappers", "goexperiment.regabiargs", "goexperiment.aliastypeparams",
	"goexperiment.swissmap", "goexperiment.synchashtriemap", "goexperiment.dwarf5", "amd64.v1",
}

// Private reconstruction from original sealed input. A selected Go source list
// is not a verification closure or a source/profile proof. Compiler/module/
// export/context ownership must still bind the eventual package loader.
type practicalPackageSources struct {
	location  practicalModuleLocation
	name      string
	active    []practicalSourceObject
	excluded  []practicalSourceObject
	auxiliary []practicalSourceObject
	imports   []string
	allTags   []string
	embeds    []string
}

type practicalBuildFileInfo struct {
	name string
	mode fs.FileMode
	size int64
}

func (info practicalBuildFileInfo) Name() string       { return info.name }
func (info practicalBuildFileInfo) Size() int64        { return info.size }
func (info practicalBuildFileInfo) Mode() fs.FileMode  { return info.mode }
func (info practicalBuildFileInfo) ModTime() time.Time { return time.Time{} }
func (info practicalBuildFileInfo) IsDir() bool        { return info.mode.IsDir() }
func (info practicalBuildFileInfo) Sys() any           { return nil }

const practicalBuildDirectory = "/mpk-sealed-package"

func practicalLexicalSubdir(root, directory string) (string, bool) {
	if directory == root {
		return ".", true
	}
	prefix := strings.TrimSuffix(root, "/") + "/"
	if strings.HasPrefix(directory, prefix) {
		return strings.TrimPrefix(directory, prefix), true
	}
	return "", false
}

func practicalTargetBuildContext() (build.Context, error) {
	if runtime.Version() != practicalCompilerRelease {
		return build.Context{}, fmt.Errorf("sealed practical package selection requires the exact Go1.25.0 library")
	}
	context := build.Context{
		GOOS: "linux", GOARCH: "amd64", Compiler: "gc", CgoEnabled: false,
		GOROOT: "/mpk-sealed-toolchain", GOPATH: "/mpk-sealed-gopath", Dir: practicalBuildDirectory,
		ToolTags: append([]string(nil), practicalTargetToolTags[:]...),
		JoinPath: path.Join, IsAbsPath: path.IsAbs, HasSubdir: practicalLexicalSubdir,
		SplitPathList: func(value string) []string {
			if value == "" {
				return nil
			}
			return strings.Split(value, ":")
		},
	}
	for minor := 1; minor <= 25; minor++ {
		context.ReleaseTags = append(context.ReleaseTags, "go1."+strconv.Itoa(minor))
	}
	return context, nil
}

func (snapshot practicalSourceSnapshot) buildSource(location practicalModuleLocation, name string) (practicalSourceBuffer, error) {
	if !validPracticalLogicalPath(name) || strings.Contains(name, "/") {
		return practicalSourceBuffer{}, fmt.Errorf("package build source is not an exact original child name")
	}
	relative := path.Join(location.directory, name)
	entry, err := snapshot.pathEntry(location.capability, relative)
	if err != nil {
		return practicalSourceBuffer{}, err
	}
	if entry.Type != 0 {
		return practicalSourceBuffer{}, fmt.Errorf("package build source is linked or not a regular file")
	}
	id, exists := snapshot.paths[practicalCaptureKey(location.capability, relative)]
	if !exists {
		return practicalSourceBuffer{}, fmt.Errorf("package build input %s/%s: %w", location.capability, relative, errPracticalNotCaptured)
	}
	buffer := snapshot.buffers[id]
	if _, err := buffer.canonicalObject(); err != nil {
		return practicalSourceBuffer{}, err
	}
	if buffer.object.Capability != location.capability || buffer.object.Path != relative {
		return practicalSourceBuffer{}, fmt.Errorf("package build source has a crossed original location")
	}
	if strings.HasSuffix(name, ".go") {
		if buffer.object.Kind != "go_file" && buffer.object.Kind != "excluded_source" {
			return practicalSourceBuffer{}, fmt.Errorf("package Go candidate has a non-source captured role")
		}
	} else if buffer.object.Kind != "auxiliary" {
		return practicalSourceBuffer{}, fmt.Errorf("package auxiliary input has a different original role")
	}
	return practicalSourceBuffer{object: buffer.object, data: buffer.sourceBytes()}, nil
}

func (snapshot practicalSourceSnapshot) replayPackageBuild(location practicalModuleLocation) (practicalPackageSources, error) {
	result := practicalPackageSources{location: location}
	if snapshot.contentSHA256 == "" || snapshot.contentSHA256 != practicalSnapshotDigest(snapshot) {
		return result, fmt.Errorf("package build selection requires an intact sealed namespace")
	}
	if location.directory != "" && !validPracticalLogicalPath(location.directory) {
		return result, fmt.Errorf("package directory is not a canonical original location")
	}
	context, err := practicalTargetBuildContext()
	if err != nil {
		return result, err
	}
	entries, err := snapshot.readDirectory(location.capability, location.directory)
	if err != nil {
		return result, err
	}
	var readErr error
	context.IsDir = func(name string) bool { return name == practicalBuildDirectory }
	context.ReadDir = func(name string) ([]fs.FileInfo, error) {
		if name != practicalBuildDirectory {
			return nil, fmt.Errorf("package build tried an unbound virtual directory")
		}
		info := make([]fs.FileInfo, len(entries))
		for i, entry := range entries {
			// Only names/type bits are consulted by go/build. Byte sizes in the
			// returned records always come from owned source objects, not this
			// private directory adapter for unread/unrelated native assets.
			info[i] = practicalBuildFileInfo{name: entry.Name, mode: entry.Type}
		}
		return info, nil
	}
	context.OpenFile = func(name string) (io.ReadCloser, error) {
		if path.Dir(name) != practicalBuildDirectory {
			readErr = fmt.Errorf("package build tried an unbound virtual source path")
			return nil, readErr
		}
		buffer, err := snapshot.buildSource(location, path.Base(name))
		if err != nil {
			if readErr == nil {
				readErr = err
			}
			return nil, err
		}
		return io.NopCloser(bytes.NewReader(buffer.data)), nil
	}
	// Non-nil filesystem hooks also disable go/build's external go-command
	// module fallback. Every path callback is fixed and closed over the seal.
	observed, buildErr := context.ImportDir(practicalBuildDirectory, 0)
	if readErr != nil {
		return result, readErr
	}
	if buildErr != nil {
		return result, fmt.Errorf("original package build selection: %w", buildErr)
	}
	if observed == nil || observed.Name == "" || len(observed.GoFiles) == 0 || len(observed.CgoFiles) != 0 || len(observed.InvalidGoFiles) != 0 {
		return result, fmt.Errorf("original package has no complete non-cgo selected Go source set")
	}
	result.name = observed.Name
	result.imports = append([]string(nil), observed.Imports...)
	result.allTags = append([]string(nil), observed.AllTags...)
	result.embeds = append([]string(nil), observed.EmbedPatterns...)
	selected := make(map[string]bool, len(observed.GoFiles))
	for _, name := range observed.GoFiles {
		selected[name] = true
	}
	goNames := append([]string(nil), observed.GoFiles...)
	goNames = append(goNames, observed.IgnoredGoFiles...)
	goNames = append(goNames, observed.TestGoFiles...)
	goNames = append(goNames, observed.XTestGoFiles...)
	sort.Strings(goNames)
	for i, name := range goNames {
		if i > 0 && goNames[i-1] == name {
			return result, fmt.Errorf("original Go build lists one source more than once")
		}
		buffer, err := snapshot.buildSource(location, name)
		if err != nil {
			return result, err
		}
		if selected[name] {
			if buffer.object.Kind != "go_file" {
				return result, fmt.Errorf("selected original Go source has an excluded captured role")
			}
			result.active = append(result.active, buffer.object)
		} else {
			if buffer.object.Kind != "excluded_source" {
				return result, fmt.Errorf("excluded original Go source has a selected captured role")
			}
			result.excluded = append(result.excluded, buffer.object)
		}
	}
	auxiliary := [][]string{observed.CFiles, observed.CXXFiles, observed.MFiles, observed.HFiles, observed.FFiles, observed.SFiles, observed.SwigFiles, observed.SwigCXXFiles, observed.SysoFiles, observed.IgnoredOtherFiles}
	seen := map[string]bool{}
	for _, list := range auxiliary {
		for _, name := range list {
			if seen[name] {
				return result, fmt.Errorf("original auxiliary source appears more than once")
			}
			seen[name] = true
			buffer, err := snapshot.buildSource(location, name)
			if err != nil {
				return result, err
			}
			result.auxiliary = append(result.auxiliary, buffer.object)
		}
	}
	sort.Slice(result.auxiliary, func(i, j int) bool { return result.auxiliary[i].Path < result.auxiliary[j].Path })
	return result, nil
}
