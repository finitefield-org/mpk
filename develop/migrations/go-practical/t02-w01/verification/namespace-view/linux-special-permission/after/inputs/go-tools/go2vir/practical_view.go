package main

import (
	"crypto/sha256"
	"errors"
	"fmt"
	"io"
	"io/fs"
	"os"
	"path/filepath"
	"sync"
)

// An owned image-reader transaction. It does not assert that a kernel sandbox
// executed Go, register a context, or establish source/program semantics.
// Materialized bytes and original namespace knowledge are checked separately.
type practicalReplayView struct {
	mutex  sync.Mutex
	owner  *practicalReplayImage
	plan   *practicalReplayNamespace
	root   practicalRootReader
	failed error
	closed bool
}

func verifyPracticalReplayMaterialization(root practicalRootReader, plan *practicalReplayNamespace) error {
	directories := map[string]bool{".": true}
	for name := range plan.directories {
		directories[filepath.Join("namespace", name[1:])] = true
	}
	files := map[string]practicalReplayExpectedFile{}
	for name, expected := range plan.files {
		files[filepath.Join("namespace", name[1:])] = expected
	}
	seenFiles, seenDirectories := 0, 0
	err := filepath.Walk(plan.root, func(name string, info os.FileInfo, walkErr error) error {
		if walkErr != nil {
			return fmt.Errorf("private replay image is no longer intact")
		}
		relative, err := filepath.Rel(plan.root, name)
		if err != nil {
			return err
		}
		if info.Mode()&os.ModeSymlink != 0 {
			return fmt.Errorf("private replay image contains a symbolic link")
		}
		if info.IsDir() {
			if !directories[relative] || info.Mode().Perm() != 0o555 || info.Mode()&(os.ModeSetuid|os.ModeSetgid|os.ModeSticky) != 0 {
				return fmt.Errorf("private replay directory is extra or no longer sealed")
			}
			seenDirectories++
			return nil
		}
		expected, exists := files[relative]
		if !exists || !releaseFileModeValid(info.Mode(), expected.executable) || info.Size() != expected.size {
			return fmt.Errorf("private replay file is extra, changed, special or no longer sealed")
		}
		before, err := root.Lstat(filepath.ToSlash(relative))
		if err != nil || !sameFileState(info, before) {
			return fmt.Errorf("private replay file changed before opening")
		}
		file, err := root.Open(filepath.ToSlash(relative))
		if err != nil {
			return fmt.Errorf("private replay file cannot be opened")
		}
		opened, openErr := file.Stat()
		if openErr != nil || !sameFileState(before, opened) {
			_ = file.Close()
			return fmt.Errorf("private replay file changed while opening")
		}
		hash := sha256.New()
		n, readErr := io.Copy(hash, io.LimitReader(file, expected.size+1))
		after, afterErr := file.Stat()
		named, nameErr := root.Lstat(filepath.ToSlash(relative))
		closeErr := file.Close()
		if readErr != nil || afterErr != nil || nameErr != nil || closeErr != nil || n != expected.size || fmt.Sprintf("%x", hash.Sum(nil)) != expected.digest || !sameFileState(before, after) || !sameFileState(before, named) {
			return fmt.Errorf("private replay file no longer matches its complete original identity")
		}
		seenFiles++
		return nil
	})
	if err != nil {
		return err
	}
	if seenFiles != len(files) || seenDirectories != len(directories) {
		return fmt.Errorf("private replay image lost an original materialized input")
	}
	return nil
}

func (image *practicalReplayImage) openVerifiedView() (*practicalReplayView, error) {
	image.mutex.Lock()
	defer image.mutex.Unlock()
	if image.closed || image.plan == nil {
		return nil, fmt.Errorf("replay byte image has no live original ownership")
	}
	root, err := openPracticalSourceRoot(image.plan.root)
	if err != nil {
		return nil, fmt.Errorf("private replay image cannot be opened")
	}
	if err := verifyPracticalReplayMaterialization(root, image.plan); err != nil {
		_ = root.Close()
		return nil, err
	}
	image.activeViews++
	return &practicalReplayView{owner: image, plan: image.plan, root: root}, nil
}

func (view *practicalReplayView) reject(err error) error {
	if view.failed == nil {
		view.failed = err
	}
	return view.failed
}

func (view *practicalReplayView) ready() error {
	if view.closed {
		return fmt.Errorf("original replay consumer view is closed")
	}
	return view.failed
}

func (view *practicalReplayView) readCaptured(capability, relative, kind string) (practicalSourceBuffer, error) {
	view.mutex.Lock()
	defer view.mutex.Unlock()
	if err := view.ready(); err != nil {
		return practicalSourceBuffer{}, err
	}
	if !validPracticalSourceKind(kind) {
		return practicalSourceBuffer{}, view.reject(fmt.Errorf("unknown original replay source role"))
	}
	entry, err := view.plan.namespace.pathEntry(capability, relative)
	if err != nil {
		if !errors.Is(err, fs.ErrNotExist) {
			err = view.reject(err)
		}
		return practicalSourceBuffer{}, err
	}
	if entry.Type != 0 {
		return practicalSourceBuffer{}, view.reject(fmt.Errorf("original replay source is linked or not regular"))
	}
	object, exists := view.plan.captured[practicalCaptureKey(capability, relative)]
	if !exists {
		return practicalSourceBuffer{}, view.reject(fmt.Errorf("original replay input %s/%s: %w", capability, relative, errPracticalNotCaptured))
	}
	if object.Kind != kind {
		return practicalSourceBuffer{}, view.reject(fmt.Errorf("original replay source has a crossed captured role"))
	}
	original := filepath.Join(view.plan.origins[capability], filepath.FromSlash(relative))
	name := filepath.ToSlash(filepath.Join("namespace", original[1:]))
	info, err := view.root.Lstat(name)
	if err != nil || !releaseFileModeValid(info.Mode(), false) {
		return practicalSourceBuffer{}, view.reject(fmt.Errorf("private replay source is missing, linked or not regular"))
	}
	raw, err := readPracticalRegularFile(view.root, name, info, practicalSourceFileLimit(kind))
	if err != nil {
		return practicalSourceBuffer{}, view.reject(fmt.Errorf("private replay source changed during its bounded read"))
	}
	buffer, err := newPracticalSourceBuffer(capability, relative, kind, raw)
	if err != nil || buffer.object != object {
		return practicalSourceBuffer{}, view.reject(fmt.Errorf("private replay source differs from its whole original identity"))
	}
	return buffer, nil
}

func (view *practicalReplayView) check() error {
	view.mutex.Lock()
	defer view.mutex.Unlock()
	if err := view.ready(); err != nil {
		return err
	}
	if err := verifyPracticalReplayMaterialization(view.root, view.plan); err != nil {
		return view.reject(err)
	}
	return nil
}

func (view *practicalReplayView) Close() error {
	view.mutex.Lock()
	defer view.mutex.Unlock()
	if view.closed {
		return nil
	}
	err := view.root.Close()
	view.closed = true
	view.owner.mutex.Lock()
	view.owner.activeViews--
	view.owner.mutex.Unlock()
	return err
}
