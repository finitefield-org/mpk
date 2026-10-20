package main

import (
	"archive/zip"
	"bytes"
	"crypto/sha256"
	"fmt"
	"hash"
	"io"
	"math"
	"path"
	"sort"
	"strings"

	"golang.org/x/mod/module"
	"golang.org/x/mod/sumdb/dirhash"
)

// Go's module ZIP protocol bound, separate from captured raw file limits.
const practicalGoModuleZIPBytes = 500 << 20

type practicalArchiveMember struct {
	Path      string
	Bytes     uint64
	RawSHA256 string
}

type practicalModuleArchive struct {
	identity         module.Version
	source           practicalSourceBuffer
	sum              string
	members          map[string]practicalArchiveMember
	namespaceEntries uint64
}

// The session owns its verified cache. Returned handles independently own both
// derived member metadata and original bytes, including on the first read.
func copyPracticalModuleArchive(archive practicalModuleArchive) practicalModuleArchive {
	owned := archive
	owned.source = practicalSourceBuffer{object: archive.source.object, data: archive.source.sourceBytes()}
	owned.members = make(map[string]practicalArchiveMember, len(archive.members))
	for name, member := range archive.members {
		owned.members[name] = member
	}
	return owned
}

type practicalArchiveNameKey struct {
	parent uint32
	folded string
}

type practicalArchiveName struct {
	spelling  string
	directory bool
	id        uint32
}

// Component keys implement Go's Unicode simple-fold path collision rule.
// A trie avoids recursive checks and quadratic copies of long asset paths.
func checkPracticalArchiveName(names map[practicalArchiveNameKey]practicalArchiveName, relative string, directory bool, maximum int) error {
	parent := uint32(0)
	parts := strings.Split(relative, "/")
	for index, spelling := range parts {
		isDirectory := directory || index < len(parts)-1
		key := practicalArchiveNameKey{parent: parent, folded: foldPracticalArchiveName(spelling)}
		if old, exists := names[key]; exists {
			if old.spelling != spelling || old.directory != isDirectory || !isDirectory {
				return fmt.Errorf("module ZIP contains a case, file/directory or duplicate file collision")
			}
			parent = old.id
			continue
		}
		if len(names) >= maximum {
			return fmt.Errorf("module ZIP namespace exceeds remaining capture entries")
		}
		parent = uint32(len(names) + 1)
		names[key] = practicalArchiveName{spelling: spelling, directory: isDirectory, id: parent}
	}
	return nil
}

type practicalArchiveHashReader struct {
	io.Reader
	closer io.Closer
	limit  *io.LimitedReader
	hash   hash.Hash
	finish func(string, error)
}

func (reader *practicalArchiveHashReader) Close() error {
	err := reader.closer.Close()
	if reader.limit.N != 1 {
		err = fmt.Errorf("module ZIP member length differs from its original header")
	}
	reader.finish(fmt.Sprintf("%x", reader.hash.Sum(nil)), err)
	return err
}

// Inspect original owned ZIP bytes without extracting a loader filesystem.
// Hash1 binds original member contents, including unselected package bodies.
// The source object separately retains the exact complete ZIP encoding.
func parsePracticalModuleArchive(identity module.Version, source practicalSourceBuffer, expectedSum string, remainingEntries int) (practicalModuleArchive, error) {
	if module.Check(identity.Path, identity.Version) != nil || module.CanonicalVersion(identity.Version) != identity.Version {
		return practicalModuleArchive{}, fmt.Errorf("invalid original module ZIP identity")
	}
	if source.object.Kind != "auxiliary" || !strings.HasPrefix(expectedSum, "h1:") {
		return practicalModuleArchive{}, fmt.Errorf("module ZIP requires original auxiliary bytes and a recorded h1 checksum")
	}
	if _, err := source.canonicalObject(); err != nil {
		return practicalModuleArchive{}, err
	}
	reader, err := zip.NewReader(bytes.NewReader(source.data), int64(len(source.data)))
	if err != nil {
		return practicalModuleArchive{}, err
	}
	if len(reader.File) > remainingEntries {
		return practicalModuleArchive{}, fmt.Errorf("module ZIP headers exceed remaining capture entries")
	}
	archive := practicalModuleArchive{identity: identity, source: source, members: make(map[string]practicalArchiveMember)}
	prefix := identity.Path + "@" + identity.Version + "/"
	names := make(map[practicalArchiveNameKey]practicalArchiveName)
	headerPaths := make(map[string]bool)
	files := make(map[string]*zip.File)
	ordered := make([]string, 0, len(reader.File))
	var expanded uint64
	for _, file := range reader.File {
		if !strings.HasPrefix(file.Name, prefix) || file.UncompressedSize64 > math.MaxInt64-1 {
			return practicalModuleArchive{}, fmt.Errorf("module ZIP contains an invalid prefix or size")
		}
		// Go's ZIP reader rejects uncompressed directory data. Check every
		// original header, including entries shadowed by HashZip's last-name
		// lookup. Zero-uncompressed compressed directory metadata remains valid.
		if strings.HasSuffix(file.Name, "/") && file.UncompressedSize64 != 0 {
			return practicalModuleArchive{}, fmt.Errorf("module ZIP directory contains uncompressed data")
		}
		files[file.Name] = file
		ordered = append(ordered, file.Name)
		relative := strings.TrimPrefix(file.Name, prefix)
		if relative == "" {
			continue
		}
		directory := strings.HasSuffix(relative, "/")
		if directory {
			relative = strings.TrimSuffix(relative, "/")
		}
		if path.Clean(relative) != relative || module.CheckFilePath(relative) != nil {
			return practicalModuleArchive{}, fmt.Errorf("module ZIP contains a noncanonical path")
		}
		if err := checkPracticalArchiveName(names, relative, directory, remainingEntries); err != nil {
			return practicalModuleArchive{}, err
		}
		headerPaths[relative] = true
		if directory {
			continue
		}
		if strings.EqualFold(path.Base(relative), "go.mod") && relative != "go.mod" {
			return practicalModuleArchive{}, fmt.Errorf("module ZIP go.mod is not an exact root manifest")
		}
		if file.UncompressedSize64 > practicalGoModuleZIPBytes-expanded {
			return practicalModuleArchive{}, fmt.Errorf("module ZIP content exceeds the Go protocol bound")
		}
		expanded += file.UncompressedSize64
		if oneOf(relative, "go.mod", "LICENSE") && file.UncompressedSize64 > maximumCandidateBytes {
			return practicalModuleArchive{}, fmt.Errorf("module ZIP manifest or license exceeds the Go protocol bound")
		}
		archive.members[relative] = practicalArchiveMember{Path: relative, Bytes: file.UncompressedSize64}
	}
	// Unique complete namespace, plus repeated/root header observations. Implicit
	// parent directories count as entries even when their headers are absent.
	archive.namespaceEntries = uint64(len(names) + len(reader.File) - len(headerPaths))
	if archive.namespaceEntries > uint64(remainingEntries) {
		return practicalModuleArchive{}, fmt.Errorf("module ZIP namespace exceeds remaining capture entries")
	}
	var closeError error
	sum, err := dirhash.Hash1(ordered, func(name string) (io.ReadCloser, error) {
		file := files[name]
		stream, err := file.Open()
		if err != nil {
			return nil, err
		}
		limit := &io.LimitedReader{R: stream, N: int64(file.UncompressedSize64) + 1}
		digest := sha256.New()
		return &practicalArchiveHashReader{Reader: io.TeeReader(limit, digest), closer: stream, limit: limit, hash: digest, finish: func(hash string, err error) {
			if err != nil {
				closeError = err
			}
			relative := strings.TrimPrefix(name, prefix)
			if member, exists := archive.members[relative]; exists {
				member.RawSHA256 = hash
				archive.members[relative] = member
			}
		}}, nil
	})
	if err != nil || closeError != nil {
		return practicalModuleArchive{}, fmt.Errorf("module ZIP member bytes or CRC differ from their original headers")
	}
	if sum != expectedSum {
		return practicalModuleArchive{}, fmt.Errorf("missing or stale original module archive checksum")
	}
	archive.sum = sum
	return archive, nil
}

func (archive practicalModuleArchive) memberInventory() []practicalArchiveMember {
	members := make([]practicalArchiveMember, 0, len(archive.members))
	for _, member := range archive.members {
		members = append(members, member)
	}
	sort.Slice(members, func(i, j int) bool { return members[i].Path < members[j].Path })
	return members
}

// Content linkage only. Capability/module/package authority is independently
// required before an original source object can belong to a loaded package.
func (archive practicalModuleArchive) verifyContent(relative string, source practicalSourceBuffer) error {
	if _, err := source.canonicalObject(); err != nil {
		return err
	}
	member, exists := archive.members[relative]
	if !exists || source.object.RawSHA256 != member.RawSHA256 || uint64(source.object.Bytes) != member.Bytes {
		return fmt.Errorf("captured file differs from its original module archive member")
	}
	return nil
}

func (session *practicalCaptureSession) originalModuleSums(sums practicalModuleSums) (practicalModuleSums, error) {
	owner, exists := session.files[practicalCaptureKey(sums.source.object.Capability, sums.source.object.Path)]
	if !exists || owner.buffer.object != sums.source.object || sums.source.object.Kind != "go_sum" {
		return practicalModuleSums{}, session.reject(fmt.Errorf("module checksums are not an original captured input"))
	}
	originalSums, cached := session.archiveSums[owner.buffer.object.ID]
	if !cached {
		var err error
		originalSums, err = parsePracticalModuleSums(owner.buffer)
		if err != nil {
			return practicalModuleSums{}, session.reject(err)
		}
		if session.archiveSums == nil {
			session.archiveSums = make(map[string]practicalModuleSums)
		}
		session.archiveSums[owner.buffer.object.ID] = originalSums
	}
	// The supplied parsed map is not authority. Cache a separate original-byte
	// reconstruction once, without reparsing a large go.sum for every module.
	return copyPracticalModuleSums(originalSums), nil
}

func (session *practicalCaptureSession) captureModuleArchive(capability, relative string, identity module.Version, sums practicalModuleSums) (practicalModuleArchive, error) {
	originalSums, err := session.originalModuleSums(sums)
	if err != nil {
		return practicalModuleArchive{}, err
	}
	expectedSum := originalSums.entries[identity]
	source, err := session.readSource(capability, relative, "auxiliary")
	if err != nil {
		return practicalModuleArchive{}, err
	}
	if old, exists := session.archives[source.object.ID]; exists {
		if old.identity != identity || old.sum != expectedSum {
			return practicalModuleArchive{}, session.reject(fmt.Errorf("cached module ZIP identity or checksum changed"))
		}
		return copyPracticalModuleArchive(old), nil
	}
	archive, err := parsePracticalModuleArchive(identity, source, expectedSum, maximumDirectoryEntries-int(session.counters.directoryEntries))
	if err != nil {
		return practicalModuleArchive{}, session.reject(err)
	}
	session.counters.directoryEntries += archive.namespaceEntries
	if session.archives == nil {
		session.archives = make(map[string]practicalModuleArchive)
	}
	session.archives[source.object.ID] = copyPracticalModuleArchive(archive)
	return copyPracticalModuleArchive(archive), nil
}
