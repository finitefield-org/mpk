package main

import (
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"strings"
)

type practicalNamespaceNode struct {
	kind     os.FileMode
	observed bool
	entries  []practicalDirectoryEntry
	objects  []practicalSourceObject
}

type practicalReplayExpectedFile struct {
	original   string
	size       int64
	digest     string
	executable bool
}

// Consistent knowledge of original physical names across all authorized
// capabilities. A known name does not imply its contents were captured.
// This private plan is not a registered context or a source proof.
type practicalReplayNamespace struct {
	root        string
	nodes       map[string]*practicalNamespaceNode
	files       map[string]practicalReplayExpectedFile
	directories map[string]bool
	origins     map[string]string
	captured    map[string]practicalSourceObject
	namespace   practicalSourceSnapshot
}

func (plan *practicalReplayNamespace) node(name string, kind os.FileMode) (*practicalNamespaceNode, error) {
	if old, exists := plan.nodes[name]; exists {
		if old.kind != kind {
			return nil, fmt.Errorf("original root aliases disagree on a native namespace type")
		}
		return old, nil
	}
	result := &practicalNamespaceNode{kind: kind}
	plan.nodes[name] = result
	return result, nil
}

func (plan *practicalReplayNamespace) directory(name string, materialized bool) error {
	for {
		if _, err := plan.node(name, os.ModeDir); err != nil {
			return err
		}
		if materialized {
			plan.directories[name] = true
		}
		parent := filepath.Dir(name)
		if parent == name {
			return nil
		}
		name = parent
	}
}

func (plan *practicalReplayNamespace) file(expected practicalReplayExpectedFile, object *practicalSourceObject) error {
	if err := plan.directory(filepath.Dir(expected.original), true); err != nil {
		return err
	}
	node, err := plan.node(expected.original, 0)
	if err != nil {
		return err
	}
	if old, exists := plan.files[expected.original]; exists && old != expected {
		return fmt.Errorf("original root aliases disagree on complete file identity or execution mode")
	}
	if object != nil {
		for _, old := range node.objects {
			if old.Kind != object.Kind || old.RawSHA256 != object.RawSHA256 || old.Bytes != object.Bytes {
				return fmt.Errorf("original root aliases disagree on the captured file role or bytes")
			}
		}
		node.objects = append(node.objects, *object)
	}
	plan.files[expected.original] = expected
	return nil
}

func reconcilePracticalReplayNamespace(inputs []practicalReplaySources, compiler *practicalCompilerImage) (*practicalReplayNamespace, error) {
	plan := &practicalReplayNamespace{
		nodes: map[string]*practicalNamespaceNode{}, files: map[string]practicalReplayExpectedFile{},
		directories: map[string]bool{}, origins: map[string]string{}, captured: map[string]practicalSourceObject{},
		namespace: practicalSourceSnapshot{directories: map[string][]practicalDirectoryEntry{}},
	}
	for _, input := range inputs {
		for _, origin := range input.origins {
			plan.origins[origin.Capability] = origin.Directory
			plan.namespace.roots = append(plan.namespace.roots, origin.Capability)
			if err := plan.directory(origin.Directory, true); err != nil {
				return nil, err
			}
		}
		for key, originalEntries := range input.snapshot.directories {
			capability, relative, _ := strings.Cut(key, "\x00")
			name := filepath.Join(plan.origins[capability], filepath.FromSlash(relative))
			if err := plan.directory(name, true); err != nil {
				return nil, err
			}
			node := plan.nodes[name]
			entries := append([]practicalDirectoryEntry(nil), originalEntries...)
			if node.observed {
				if len(entries) != len(node.entries) {
					return nil, fmt.Errorf("original root aliases disagree on a complete directory namespace")
				}
				for i := range entries {
					if entries[i] != node.entries[i] {
						return nil, fmt.Errorf("original root aliases disagree on a complete directory namespace")
					}
				}
			}
			node.observed, node.entries = true, entries
			plan.namespace.directories[key] = append([]practicalDirectoryEntry(nil), entries...)
			for _, entry := range entries {
				if _, err := plan.node(filepath.Join(name, entry.Name), entry.Type); err != nil {
					return nil, err
				}
			}
		}
		for _, object := range input.snapshot.sourceObjects() {
			name := filepath.Join(plan.origins[object.Capability], filepath.FromSlash(object.Path))
			expected := practicalReplayExpectedFile{original: name, size: object.Bytes, digest: object.RawSHA256}
			if err := plan.file(expected, &object); err != nil {
				return nil, err
			}
			plan.captured[practicalCaptureKey(object.Capability, object.Path)] = object
		}
	}
	if compiler != nil {
		if !practicalReplayAbsolute(compiler.selection.GoRoot) {
			return nil, fmt.Errorf("replay compiler has no original absolute root")
		}
		for _, member := range compiler.candidate.ToolchainInventory.Files {
			if !strings.HasPrefix(member.Path, "go/") || !validPracticalLogicalPath(strings.TrimPrefix(member.Path, "go/")) {
				return nil, fmt.Errorf("replay compiler member is outside its original root")
			}
			name := filepath.Join(compiler.selection.GoRoot, filepath.FromSlash(strings.TrimPrefix(member.Path, "go/")))
			if err := plan.file(practicalReplayExpectedFile{name, member.SizeBytes, member.SHA256, member.Executable}, nil); err != nil {
				return nil, err
			}
		}
	}
	// In particular, an independent nested capability may refine an unvisited
	// original directory, but cannot override observed absence in its parent.
	for name, node := range plan.nodes {
		parent := filepath.Dir(name)
		if parent == name {
			continue
		}
		directory := plan.nodes[parent]
		if directory == nil {
			continue
		}
		if directory.kind != os.ModeDir {
			return nil, fmt.Errorf("original replay namespace has a non-directory ancestor")
		}
		if !directory.observed {
			continue
		}
		child := filepath.Base(name)
		index := sort.Search(len(directory.entries), func(i int) bool { return directory.entries[i].Name >= child })
		if index == len(directory.entries) || directory.entries[index].Name != child || directory.entries[index].Type != node.kind {
			return nil, fmt.Errorf("original replay namespace contradicts a complete parent observation")
		}
	}
	sort.Strings(plan.namespace.roots)
	return plan, nil
}
