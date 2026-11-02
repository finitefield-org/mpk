package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"go/ast"
	"go/types"
)

// The request and the physical roots are derived from the same original
// package owner. This is a private compiler observation, not launcher
// authority, a checked foundation, source admission or a program proof.
type practicalCandidateSourceSelection struct {
	request   []byte
	roots     []practicalClosureRef
	selection practicalSelectionInputs
}

func (selection practicalCandidateSourceSelection) rootRefs() []practicalClosureRef {
	return append([]practicalClosureRef(nil), selection.roots...)
}

func (selection practicalCandidateSourceSelection) requestBytes() []byte {
	return bytes.Clone(selection.request)
}

// Keep the candidate request and its independently captured compiler owner
// together. The caller cannot substitute physical roots into this handoff.
type practicalCandidateCompilerInputs struct {
	selection practicalCandidateSourceSelection
	compiler  *practicalCompilerInputs
}

func (inputs *practicalCandidateCompilerInputs) Close() {
	if inputs == nil {
		return
	}
	if inputs.compiler != nil {
		inputs.compiler.Close()
		inputs.compiler = nil
	}
	clear(inputs.selection.request)
	inputs.selection.request = nil
}

func (image *practicalReplayImage) candidateCompilerInputs(owner *practicalOriginalPackages, request *practicalCandidateRequest) (*practicalCandidateCompilerInputs, error) {
	if image == nil {
		return nil, fmt.Errorf("candidate compiler inputs require the original image")
	}
	selection, err := owner.candidateSourceSelection(request)
	if err != nil {
		return nil, err
	}
	compiler, err := image.compilerInputs(owner, selection.rootRefs())
	if err != nil {
		return nil, err
	}
	if !bytes.Equal(selection.selection.canonical, compiler.exports.capture.selection.canonical) {
		compiler.Close()
		return nil, fmt.Errorf("candidate compiler inputs crossed their original source selection")
	}
	return &practicalCandidateCompilerInputs{selection: selection, compiler: compiler}, nil
}

// Recover only the producer's own canonical request. Reconstructing all four
// complete buffers also rejects a zero, partially modified or crossed request.
func practicalCandidateRequestRoots(request *practicalCandidateRequest) ([]practicalCandidateSelector, error) {
	if request == nil {
		return nil, fmt.Errorf("candidate source selection requires an owned request")
	}
	var envelope struct {
		Schema string `json:"schema"`
		Value  struct {
			Roots []practicalCandidateSelector `json:"roots"`
		} `json:"value"`
	}
	if err := json.Unmarshal(request.selection, &envelope); err != nil {
		return nil, fmt.Errorf("candidate source selection has an invalid request: %w", err)
	}
	rebuilt, err := newPracticalCandidateRequest(envelope.Value.Roots)
	if err != nil || !bytes.Equal(request.registry, rebuilt.registry) ||
		!bytes.Equal(request.context, rebuilt.context) || !bytes.Equal(request.selection, rebuilt.selection) ||
		!bytes.Equal(request.request, rebuilt.request) {
		return nil, fmt.Errorf("candidate source selection lost its complete request identity")
	}
	return envelope.Value.Roots, nil
}

type practicalCandidateSourceKey struct {
	packagePath string
	function    string
	receiver    *types.TypeName
}

// Resolve raw names only against the owner's original type-checked packages.
// Aliases compare by their original named declaration, and pointer receivers
// do not satisfy a value-receiver selector. Every selected package is scanned
// once, including at the unchanged 8,192-root boundary.
func (owner *practicalOriginalPackages) candidateSourceSelection(request *practicalCandidateRequest) (practicalCandidateSourceSelection, error) {
	if owner == nil {
		return practicalCandidateSourceSelection{}, fmt.Errorf("candidate source selection requires the original package owner")
	}
	roots, err := practicalCandidateRequestRoots(request)
	if err != nil {
		return practicalCandidateSourceSelection{}, err
	}
	owner.mutex.Lock()
	defer owner.mutex.Unlock()
	if owner.closed || owner.inputs == nil || owner.loader == nil || owner.loader.syntax == nil {
		return practicalCandidateSourceSelection{}, fmt.Errorf("candidate source owner is closed or incomplete")
	}
	for _, failure := range []error{owner.inputs.failed, owner.loader.failed, owner.loader.syntax.failed} {
		if failure != nil {
			return practicalCandidateSourceSelection{}, failure
		}
	}
	wanted := make(map[practicalCandidateSourceKey]int, len(roots))
	names := make(map[string]map[string]bool)
	packages := make([]string, 0)
	for index, root := range roots {
		loaded := owner.loader.packages[root.Package]
		captured, present := owner.packages[root.Package]
		if loaded == nil || loaded.types == nil || loaded.info == nil || !present || captured.intrinsic {
			return practicalCandidateSourceSelection{}, fmt.Errorf("candidate root %d is outside the original typed packages", index)
		}
		key := practicalCandidateSourceKey{packagePath: root.Package, function: root.Function}
		if root.Receiver != nil {
			binding, ok := loaded.types.Scope().Lookup(root.Receiver.TypeName).(*types.TypeName)
			if !ok {
				return practicalCandidateSourceSelection{}, fmt.Errorf("candidate root %d has no original named receiver", index)
			}
			named, ok := types.Unalias(binding.Type()).(*types.Named)
			if !ok {
				return practicalCandidateSourceSelection{}, fmt.Errorf("candidate root %d has no value receiver", index)
			}
			key.receiver = named.Origin().Obj()
		}
		if first, exists := wanted[key]; exists {
			return practicalCandidateSourceSelection{}, fmt.Errorf("candidate roots %d and %d name the same original function", first, index)
		}
		wanted[key] = index
		if names[root.Package] == nil {
			names[root.Package] = make(map[string]bool)
			packages = append(packages, root.Package)
		}
		names[root.Package][root.Function] = true
	}
	refs := make([]practicalClosureRef, len(roots))
	found := make([]bool, len(roots))
	for _, packagePath := range packages {
		loaded := owner.loader.packages[packagePath]
		for _, file := range loaded.files {
			for _, declaration := range file.Decls {
				function, ok := declaration.(*ast.FuncDecl)
				if !ok || !names[packagePath][function.Name.Name] {
					continue
				}
				key := practicalCandidateSourceKey{packagePath: packagePath, function: function.Name.Name}
				if function.Recv != nil {
					object, ok := loaded.info.Defs[function.Name].(*types.Func)
					if !ok {
						return practicalCandidateSourceSelection{}, fmt.Errorf("candidate method lost its original type")
					}
					signature, ok := object.Type().(*types.Signature)
					if !ok || signature.Recv() == nil {
						return practicalCandidateSourceSelection{}, fmt.Errorf("candidate method lost its original receiver")
					}
					named, ok := types.Unalias(signature.Recv().Type()).(*types.Named)
					if !ok {
						continue
					}
					key.receiver = named.Origin().Obj()
				}
				index, selected := wanted[key]
				if !selected {
					continue
				}
				if found[index] {
					return practicalCandidateSourceSelection{}, fmt.Errorf("candidate root %d is ambiguous in the original package", index)
				}
				span, _, err := owner.loader.syntax.spanBytes(function)
				if err != nil {
					return practicalCandidateSourceSelection{}, err
				}
				refs[index] = practicalClosureRef{packagePath, span}
				found[index] = true
			}
		}
	}
	for index, ok := range found {
		if !ok {
			return practicalCandidateSourceSelection{}, fmt.Errorf("candidate root %d has no original value function or method", index)
		}
	}
	selection, err := owner.selectionInputsLocked(refs)
	if err != nil {
		return practicalCandidateSourceSelection{}, err
	}
	return practicalCandidateSourceSelection{request: bytes.Clone(request.request), roots: refs, selection: selection}, nil
}
