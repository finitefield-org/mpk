package main

import (
	"crypto/sha256"
	"encoding/binary"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"sort"
	"strings"
)

// The origins are launcher-authorized names, not new source identities. Keep
// them with the sealed bytes so a kernel namespace can retain absolute and
// relative original replacement paths without rewriting a go.mod.
type practicalReplaySources struct {
	snapshot practicalSourceSnapshot
	origins  []practicalSourceRoot
	sealed   *practicalReplayBinding
}

type practicalReplayBinding struct {
	snapshot string
	origins  []practicalSourceRoot
}

func (session *practicalCaptureSession) sealPracticalReplaySources() (practicalReplaySources, error) {
	snapshot, err := session.seal()
	if err != nil {
		return practicalReplaySources{}, err
	}
	result := practicalReplaySources{snapshot: snapshot}
	for _, capability := range snapshot.roots {
		result.origins = append(result.origins, practicalSourceRoot{Capability: capability, Directory: session.roots[capability].physical})
	}
	result.sealed = &practicalReplayBinding{snapshot.contentSHA256, append([]practicalSourceRoot(nil), result.origins...)}
	return result, nil
}

type practicalReplayMount struct {
	capability string
	original   string
	private    string
}

type practicalReplayFile struct {
	original string
	objects  []practicalSourceObject
}

type practicalReplayDirectory struct {
	capability string
	relative   string
	entries    []practicalDirectoryEntry
}

// A byte image for the kernel sandbox, not a registered context or a proof.
// Only captured files are materialized. Complete original namespace records
// survive in inputs; unread names remain not-captured, never invented bytes.
// A native consumer must still bind its full consulted input/namespace closure.
type practicalReplayImage struct {
	root             string
	namespace        []practicalReplayDirectory
	mounts           []practicalReplayMount
	files            []practicalReplayFile
	compiler         *practicalCompilerImage
	compilerOriginal string
	contentSHA256    string
}

func practicalReplayAbsolute(name string) bool {
	return filepath.IsAbs(name) && filepath.Clean(name) == name && filepath.Dir(name) != name && !strings.ContainsRune(name, 0)
}

func practicalReplayWithin(root, name string) bool {
	return name == root || strings.HasPrefix(name, root+string(filepath.Separator))
}

func validatePracticalReplaySources(inputs []practicalReplaySources) error {
	if len(inputs) == 0 {
		return fmt.Errorf("replay byte image has no original captured inputs")
	}
	owners := map[string]bool{}
	identities := map[string]bool{}
	counters := captureCounters{}
	for _, input := range inputs {
		snapshot := input.snapshot
		if input.sealed == nil || snapshot.contentSHA256 == "" || snapshot.contentSHA256 != input.sealed.snapshot || snapshot.contentSHA256 != practicalSnapshotDigest(snapshot) || len(input.origins) != len(input.sealed.origins) || len(input.origins) != len(snapshot.roots) || len(snapshot.ids) != len(snapshot.buffers) || len(snapshot.ids) != len(snapshot.paths) {
			return fmt.Errorf("replay byte image needs intact original snapshots and root ownership")
		}
		for i, root := range input.origins {
			if root != input.sealed.origins[i] || !validPracticalLogicalPath(root.Capability) || root.Capability != snapshot.roots[i] || !practicalReplayAbsolute(root.Directory) || owners[root.Capability] || i > 0 && input.origins[i-1].Capability >= root.Capability {
				return fmt.Errorf("replay byte image has duplicate, crossed or invalid source roots")
			}
			owners[root.Capability] = true
		}
		for key, entries := range snapshot.directories {
			capability, relative, ok := strings.Cut(key, "\x00")
			if !ok || !snapshot.hasRoot(capability) || relative != "" && !validPracticalLogicalPath(relative) {
				return fmt.Errorf("replay namespace has an invalid original location")
			}
			if _, err := snapshot.readDirectory(capability, relative); err != nil {
				return err
			}
			for i, entry := range entries {
				if entry.Name == "" || entry.Name == "." || entry.Name == ".." || strings.ContainsAny(entry.Name, "/\x00") || entry.Type&^os.ModeType != 0 || i > 0 && entries[i-1].Name >= entry.Name {
					return fmt.Errorf("replay namespace has an invalid original entry")
				}
			}
			if err := counters.addDirectory(len(entries)); err != nil {
				return err
			}
		}
		for i, id := range snapshot.ids {
			buffer, exists := snapshot.buffers[id]
			if !exists || identities[id] || id != buffer.object.ID || i > 0 && snapshot.ids[i-1] >= id {
				return fmt.Errorf("replay byte image has duplicate or crossed source identities")
			}
			if _, err := practicalSnapshotBuffer(snapshot, buffer.object); err != nil {
				return err
			}
			if err := counters.admitCandidate(buffer.object.Bytes, buffer.object.Kind == "overlay", true); err != nil {
				return err
			}
			identities[id] = true
		}
	}
	return nil
}

// Use the full owned original namespace for absence/not-captured decisions.
// An unmaterialized opaque original name must never become evidence of absence.
func (image *practicalReplayImage) readCaptured(capability, relative, kind string) (practicalSourceBuffer, error) {
	namespace := practicalSourceSnapshot{directories: map[string][]practicalDirectoryEntry{}}
	for _, mount := range image.mounts {
		namespace.roots = append(namespace.roots, mount.capability)
	}
	sort.Strings(namespace.roots)
	for _, directory := range image.namespace {
		namespace.directories[practicalCaptureKey(directory.capability, directory.relative)] = directory.entries
	}
	entry, err := namespace.pathEntry(capability, relative)
	if err != nil {
		return practicalSourceBuffer{}, err
	}
	if entry.Type != 0 {
		return practicalSourceBuffer{}, fmt.Errorf("replay source is not an original regular namespace entry")
	}
	for _, file := range image.files {
		for _, object := range file.objects {
			if object.Capability != capability || object.Path != relative {
				continue
			}
			if object.Kind != kind {
				return practicalSourceBuffer{}, fmt.Errorf("replay source role differs from its original captured role")
			}
			reader, err := openPracticalSourceRoot(image.root)
			if err != nil {
				return practicalSourceBuffer{}, err
			}
			name, err := filepath.Rel(image.root, image.privatePath(file.original))
			if err != nil {
				_ = reader.Close()
				return practicalSourceBuffer{}, err
			}
			info, err := reader.Lstat(filepath.ToSlash(name))
			if err != nil || !info.Mode().IsRegular() || info.Mode()&os.ModeSymlink != 0 {
				_ = reader.Close()
				return practicalSourceBuffer{}, fmt.Errorf("private replay source is missing, linked or not regular")
			}
			raw, err := readPracticalRegularFile(reader, filepath.ToSlash(name), info, practicalSourceFileLimit(kind))
			closeErr := reader.Close()
			if err != nil || closeErr != nil {
				if err == nil {
					err = closeErr
				}
				return practicalSourceBuffer{}, err
			}
			buffer, err := newPracticalSourceBuffer(capability, relative, kind, raw)
			if err != nil || buffer.object != object {
				return practicalSourceBuffer{}, fmt.Errorf("private replay bytes differ from the full original source")
			}
			return buffer, nil
		}
	}
	return practicalSourceBuffer{}, fmt.Errorf("original replay input %s/%s: %w", capability, relative, errPracticalNotCaptured)
}

func (image *practicalReplayImage) privatePath(original string) string {
	return filepath.Join(image.root, "namespace", strings.TrimPrefix(original, string(filepath.Separator)))
}

func writePracticalReplayFile(filename string, raw []byte, executable bool) error {
	if err := os.MkdirAll(filepath.Dir(filename), 0o700); err != nil {
		return err
	}
	if existing, err := os.ReadFile(filename); err == nil {
		if sha256Hex(existing) != sha256Hex(raw) || len(existing) != len(raw) {
			return fmt.Errorf("overlapping original capabilities have different source bytes")
		}
		return nil
	} else if !os.IsNotExist(err) {
		return err
	}
	file, err := os.OpenFile(filename, os.O_WRONLY|os.O_CREATE|os.O_EXCL, 0o600)
	if err != nil {
		return err
	}
	writeErr := writeAll(file, raw)
	closeErr := file.Close()
	if writeErr != nil {
		return writeErr
	}
	if closeErr != nil {
		return closeErr
	}
	mode := os.FileMode(0o444)
	if executable {
		mode = 0o555
	}
	return os.Chmod(filename, mode)
}

// Compiler tools may exceed a source-object byte limit. Copy their already
// verified inventory as bounded streams, retaining the toolchain's own schema
// and limits instead of pretending binaries are Go source objects.
func copyPracticalReplayCompilerFile(root practicalRootReader, original, destination string, expected inventoryFile) error {
	before, err := root.Lstat(original)
	if err != nil || !releaseFileModeValid(before.Mode(), expected.Executable) || before.Size() != expected.SizeBytes {
		return fmt.Errorf("replay compiler input differs from its original inventory")
	}
	input, err := root.Open(original)
	if err != nil {
		return err
	}
	defer input.Close()
	opened, err := input.Stat()
	if err != nil || !sameFileState(before, opened) {
		return fmt.Errorf("replay compiler input changed while opening")
	}
	if err := os.MkdirAll(filepath.Dir(destination), 0o700); err != nil {
		return err
	}
	output, err := os.OpenFile(destination, os.O_WRONLY|os.O_CREATE|os.O_EXCL, 0o600)
	if err != nil {
		if os.IsExist(err) {
			raw, readErr := os.ReadFile(destination)
			if readErr == nil && int64(len(raw)) == expected.SizeBytes && sha256Hex(raw) == expected.SHA256 && !expected.Executable {
				return nil
			}
		}
		return err
	}
	digest := sha256.New()
	n, copyErr := io.Copy(io.MultiWriter(output, digest), io.LimitReader(input, expected.SizeBytes+1))
	closeErr := output.Close()
	if copyErr != nil || closeErr != nil || n != expected.SizeBytes || fmt.Sprintf("%x", digest.Sum(nil)) != expected.SHA256 {
		return fmt.Errorf("replay compiler copy differs from the full original bytes")
	}
	after, err := input.Stat()
	named, nameErr := root.Lstat(original)
	if err != nil || nameErr != nil || !sameFileState(before, after) || !sameFileState(before, named) {
		return fmt.Errorf("replay compiler input changed during copying")
	}
	mode := os.FileMode(0o444)
	if expected.Executable {
		mode = 0o555
	}
	return os.Chmod(destination, mode)
}

func materializePracticalReplaySources(parent string, inputs []practicalReplaySources, compiler *practicalCompilerImage, sdkCapability string) (*practicalReplayImage, error) {
	if err := validatePracticalReplaySources(inputs); err != nil {
		return nil, err
	}
	root, err := os.MkdirTemp(parent, "go-practical-replay-")
	if err != nil {
		return nil, err
	}
	root, err = filepath.EvalSymlinks(root)
	if err != nil {
		return nil, err
	}
	image := &practicalReplayImage{root: root}
	success := false
	defer func() {
		if !success {
			_ = image.Close()
		}
	}()
	byOriginal := map[string][]practicalSourceObject{}
	for _, input := range inputs {
		origins := map[string]string{}
		for _, origin := range input.origins {
			if practicalReplayWithin(origin.Directory, root) || practicalReplayWithin(root, origin.Directory) {
				return nil, fmt.Errorf("private replay storage overlaps original source capabilities")
			}
			origins[origin.Capability] = origin.Directory
			image.mounts = append(image.mounts, practicalReplayMount{origin.Capability, origin.Directory, image.privatePath(origin.Directory)})
			if err := os.MkdirAll(image.privatePath(origin.Directory), 0o700); err != nil {
				return nil, err
			}
		}
		for key, entries := range input.snapshot.directories {
			capability, relative, _ := strings.Cut(key, "\x00")
			image.namespace = append(image.namespace, practicalReplayDirectory{capability, relative, append([]practicalDirectoryEntry(nil), entries...)})
			original := filepath.Join(origins[capability], filepath.FromSlash(relative))
			if err := os.MkdirAll(image.privatePath(original), 0o700); err != nil {
				return nil, err
			}
		}
		for _, object := range input.snapshot.sourceObjects() {
			original := filepath.Join(origins[object.Capability], filepath.FromSlash(object.Path))
			raw, _ := input.snapshot.sourceBytes(object.ID)
			if err := writePracticalReplayFile(image.privatePath(original), raw, false); err != nil {
				return nil, err
			}
			byOriginal[original] = append(byOriginal[original], object)
		}
	}
	if compiler != nil {
		if err := compiler.revalidate(); err != nil {
			return nil, err
		}
		found := false
		for _, input := range inputs {
			for _, origin := range input.origins {
				if origin.Capability != sdkCapability {
					continue
				}
				if found || origin.Directory != compiler.selection.GoRoot {
					return nil, fmt.Errorf("replay compiler root crossed its original source capability")
				}
				if _, err := bindPracticalStandardLibrary(input.snapshot, *compiler, sdkCapability); err != nil {
					return nil, err
				}
				found = true
			}
		}
		if !found {
			return nil, fmt.Errorf("replay compiler needs its complete original source snapshot")
		}
		reader, err := openPracticalSourceRoot(compiler.selection.GoRoot)
		if err != nil {
			return nil, err
		}
		for _, expected := range compiler.candidate.ToolchainInventory.Files {
			relative := strings.TrimPrefix(expected.Path, "go/")
			original := filepath.Join(compiler.selection.GoRoot, filepath.FromSlash(relative))
			if err := copyPracticalReplayCompilerFile(reader, relative, image.privatePath(original), expected); err != nil {
				_ = reader.Close()
				return nil, err
			}
		}
		if err := reader.Close(); err != nil {
			return nil, err
		}
		if err := compiler.revalidate(); err != nil {
			return nil, err
		}
		copied := *compiler
		copied.candidate.ToolchainGoRootPath = image.privatePath(compiler.selection.GoRoot)
		image.compiler, image.compilerOriginal = &copied, compiler.selection.GoRoot
	}
	for original, objects := range byOriginal {
		image.files = append(image.files, practicalReplayFile{original, append([]practicalSourceObject(nil), objects...)})
	}
	sort.Slice(image.files, func(i, j int) bool { return image.files[i].original < image.files[j].original })
	sort.Slice(image.mounts, func(i, j int) bool {
		if image.mounts[i].original != image.mounts[j].original {
			return image.mounts[i].original < image.mounts[j].original
		}
		return image.mounts[i].capability < image.mounts[j].capability
	})
	sort.Slice(image.namespace, func(i, j int) bool {
		if image.namespace[i].capability != image.namespace[j].capability {
			return image.namespace[i].capability < image.namespace[j].capability
		}
		return image.namespace[i].relative < image.namespace[j].relative
	})
	hash := sha256.New()
	_, _ = hash.Write([]byte("MPK-GO-PRIVATE-REPLAY-IMAGE-1.0\x00"))
	add := func(value string) {
		var size [8]byte
		binary.LittleEndian.PutUint64(size[:], uint64(len(value)))
		_, _ = hash.Write(size[:])
		_, _ = hash.Write([]byte(value))
	}
	for _, input := range inputs {
		add(input.snapshot.contentSHA256)
		for _, origin := range input.origins {
			add(origin.Capability)
			add(origin.Directory)
		}
	}
	if image.compiler != nil {
		add(image.compiler.selection.Toolchain.DistributionSHA256)
	}
	image.contentSHA256 = fmt.Sprintf("%x", hash.Sum(nil))
	var directories []string
	if err := filepath.Walk(root, func(name string, info os.FileInfo, walkErr error) error {
		if walkErr != nil {
			return walkErr
		}
		if info.IsDir() {
			directories = append(directories, name)
		}
		return nil
	}); err != nil {
		return nil, err
	}
	for i := len(directories) - 1; i >= 0; i-- {
		if err := os.Chmod(directories[i], 0o555); err != nil {
			return nil, err
		}
	}
	if image.compiler != nil {
		checked, err := validatePracticalCompilerImage(image.compiler.candidate)
		if err != nil {
			return nil, err
		}
		image.compiler = &checked
	}
	success = true
	return image, nil
}

func (image *practicalReplayImage) Close() error {
	return removeSnapshot(image.root)
}
