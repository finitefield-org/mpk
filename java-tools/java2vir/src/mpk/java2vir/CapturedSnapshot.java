package mpk.java2vir;

import java.io.IOException;
import java.nio.ByteBuffer;
import java.nio.file.DirectoryStream;
import java.nio.file.Files;
import java.nio.file.LinkOption;
import java.nio.file.Path;
import java.nio.file.SecureDirectoryStream;
import java.nio.file.StandardOpenOption;
import java.nio.file.attribute.BasicFileAttributeView;
import java.nio.file.attribute.BasicFileAttributes;
import java.nio.file.attribute.FileTime;
import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.HashSet;
import java.util.HexFormat;
import java.util.List;
import java.util.Locale;
import java.util.Map;
import java.util.Set;
import java.util.TreeMap;

/** Second capture boundary over the parent's private immutable source snapshot.
 * Host capture/openat2 and the read-only mount are owned by the shared native runner.
 * This adapter never opens a toolchain path or discovers an application dependency.
 */
final class CapturedSnapshot {
    static final class Input {
        private final String path;
        private final boolean source;
        private final byte[] bytes;
        private final String sha256;
        Input(String path, boolean source, byte[] bytes) {
            this.path = path;
            this.source = source;
            this.bytes = bytes.clone();
            try { sha256 = HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(this.bytes)); }
            catch (NoSuchAlgorithmException error) { throw FrontendFailure.of("JAVA_FRONTEND_INTERNAL", "capture"); }
        }
        String path() { return path; }
        boolean source() { return source; }
        byte[] bytes() { return bytes.clone(); }
        int size() { return bytes.length; }
        String sha256() { return sha256; }
    }
    private record Stat(Object key, boolean directory, long size, FileTime modified,
                        FileTime changed, int mode, int links) {}
    private record Pass(Map<String, Stat> metadata, Map<String, Input> files) {}
    private final Selection selection;
    private final Map<String, Input> files;

    private CapturedSnapshot(Selection selection, Map<String, Input> files) {
        this.selection = selection;
        this.files = Map.copyOf(files);
    }

    Selection selection() { return selection; }
    Input file(String path) {
        Input result = files.get(path);
        if (result == null) throw inventoryFailure();
        return result;
    }
    List<Input> inputs() { return new TreeMap<>(files).values().stream().toList(); }
    List<SourceText> sources() {
        return selection.sources().stream().map(path -> new SourceText(path, file(path).bytes())).toList();
    }

    static CapturedSnapshot capture(Path root, Selection selection) {
        if (!root.isAbsolute() || !root.equals(root.normalize())) throw pathFailure();
        var ancestors = new ArrayList<SecureDirectoryStream<Path>>();
        try {
            SecureDirectoryStream<Path> directory = secure(Files.newDirectoryStream(root.getRoot()));
            ancestors.add(directory);
            for (Path component : root) {
                BasicFileAttributes attributes = directory.getFileAttributeView(component,
                        BasicFileAttributeView.class, LinkOption.NOFOLLOW_LINKS).readAttributes();
                if (!attributes.isDirectory() || attributes.isSymbolicLink()) throw typeFailure();
                directory = directory.newDirectoryStream(component, LinkOption.NOFOLLOW_LINKS);
                ancestors.add(directory);
                if (!attributes.fileKey().equals(directory.getFileAttributeView(BasicFileAttributeView.class).readAttributes().fileKey()))
                    throw inventoryFailure();
            }
            Pass first = pass(root, directory, selection);
            Pass second = pass(root, directory, selection);
            if (!first.metadata().equals(second.metadata()) || !first.files().keySet().equals(second.files().keySet()))
                throw inventoryFailure();
            for (String path : first.files().keySet()) {
                if (!Arrays.equals(first.files().get(path).bytes, second.files().get(path).bytes)) throw inventoryFailure();
            }
            return new CapturedSnapshot(selection, first.files());
        } catch (FrontendFailure failure) {
            throw failure;
        } catch (IOException error) {
            throw inventoryFailure();
        } catch (VirtualMachineError error) {
            throw FrontendFailure.of("JAVA_FRONTEND_RESOURCE", "capture");
        } catch (RuntimeException error) {
            throw FrontendFailure.of("JAVA_FRONTEND_INTERNAL", "capture");
        } finally {
            IOException failed = null;
            for (int i = ancestors.size() - 1; i >= 0; i--) {
                try { ancestors.get(i).close(); } catch (IOException error) { failed = error; }
            }
            if (failed != null) throw FrontendFailure.of("JAVA_FRONTEND_INTERNAL", "capture");
        }
    }

    private static SecureDirectoryStream<Path> secure(DirectoryStream<Path> stream) throws IOException {
        if (stream instanceof SecureDirectoryStream<Path> secure) return secure;
        stream.close();
        throw FrontendFailure.of("JAVA_FRONTEND_INTERNAL", "capture");
    }

    private static Pass pass(Path root, SecureDirectoryStream<Path> directory, Selection selection) throws IOException {
        var metadata = new TreeMap<String, Stat>();
        var files = new TreeMap<String, Input>();
        // A DirectoryStream can only be iterated once; use a fresh descriptor each pass.
        try (var fresh = directory.newDirectoryStream(Path.of("."), LinkOption.NOFOLLOW_LINKS)) {
            walk(root, fresh, "", selection, selection.expected(), metadata, files, new HashSet<>());
        }
        if (!metadata.keySet().equals(selection.expected().keySet())) throw inventoryFailure();
        return new Pass(metadata, files);
    }

    private static void walk(Path root, SecureDirectoryStream<Path> directory, String relative,
                             Selection selection, Map<String, Boolean> expected, Map<String, Stat> metadata,
                             Map<String, Input> files, Set<Object> identities) throws IOException {
        Stat before = stat(root.resolve(relative), directory.getFileAttributeView(BasicFileAttributeView.class).readAttributes());
        var names = new ArrayList<String>();
        for (Path entry : directory) {
            String name = entry.getFileName().toString();
            FrontendLimits.check("snapshot_entries", (long) metadata.size() + names.size() + 1, "capture");
            names.add(name);
        }
        names.sort(String::compareTo);
        // Directory iteration order is not stable. Finish the bounded name
        // inventory before path errors can compete with the entry limit.
        var folded = new HashSet<String>();
        for (String name : names)
            if (!Selection.portablePath(name) || !folded.add(name.toLowerCase(Locale.ROOT))) throw pathFailure();
        for (String name : names) {
            String path = relative.isEmpty() ? name : relative + "/" + name;
            FrontendLimits.check("normalized_path_bytes", path.length(), "capture");
            var entry = Path.of(name);
            BasicFileAttributes attributes = directory.getFileAttributeView(entry,
                    BasicFileAttributeView.class, LinkOption.NOFOLLOW_LINKS).readAttributes();
            if (attributes.isSymbolicLink() || (!attributes.isDirectory() && !attributes.isRegularFile())) throw typeFailure();
            Stat state = stat(root.resolve(path), attributes);
            if (!state.directory() && (state.links() != 1 || !identities.add(state.key()))) throw typeFailure();
            if (!expected.containsKey(path) || expected.get(path) != state.directory()) throw inventoryFailure();
            FrontendLimits.check("snapshot_entries", (long) metadata.size() + 1, "capture");
            metadata.put(path, state);
            if (state.directory()) {
                try (var child = directory.newDirectoryStream(entry, LinkOption.NOFOLLOW_LINKS)) {
                    if (!state.key().equals(child.getFileAttributeView(BasicFileAttributeView.class).readAttributes().fileKey()))
                        throw inventoryFailure();
                    walk(root, child, path, selection, expected, metadata, files, identities);
                }
            } else {
                boolean source = selection.sources().contains(path);
                String kind = source ? "source" : "contract";
                FrontendLimits.check(kind + "_file_bytes", state.size(), "capture");
                long sourceTotal = state.size();
                long snapshotTotal = state.size();
                for (Input input : files.values()) {
                    snapshotTotal = FrontendLimits.add("snapshot_total_bytes", snapshotTotal, input.size(), "capture");
                    if (input.source() == source)
                        sourceTotal = FrontendLimits.add(kind + "_total_bytes", sourceTotal, input.size(), "capture");
                }
                FrontendLimits.check(kind + "_total_bytes", sourceTotal, "capture");
                FrontendLimits.check("snapshot_total_bytes", snapshotTotal, "capture");
                byte[] bytes = new byte[(int) state.size()];
                try (var channel = directory.newByteChannel(entry, Set.of(StandardOpenOption.READ, LinkOption.NOFOLLOW_LINKS))) {
                    if (channel.size() != state.size()) throw inventoryFailure();
                    ByteBuffer buffer = ByteBuffer.wrap(bytes);
                    while (buffer.hasRemaining()) if (channel.read(buffer) < 0) throw inventoryFailure();
                    if (channel.read(ByteBuffer.allocate(1)) != -1 || channel.size() != state.size()) throw inventoryFailure();
                }
                Stat after = stat(root.resolve(path), directory.getFileAttributeView(entry,
                        BasicFileAttributeView.class, LinkOption.NOFOLLOW_LINKS).readAttributes());
                if (!state.equals(after)) throw inventoryFailure();
                files.put(path, new Input(path, source, bytes));
            }
        }
        if (!before.equals(stat(root.resolve(relative), directory.getFileAttributeView(BasicFileAttributeView.class).readAttributes())))
            throw inventoryFailure();
    }

    private static Stat stat(Path path, BasicFileAttributes descriptorRelative) throws IOException {
        // Unix metadata supplies nlink/ctime absent from BasicFileAttributes. Never use
        // this path lookup for content; require identity with the no-follow descriptor view.
        Map<String, Object> unix = Files.readAttributes(path, "unix:*", LinkOption.NOFOLLOW_LINKS);
        if (descriptorRelative.fileKey() == null || !descriptorRelative.fileKey().equals(unix.get("fileKey"))
                || descriptorRelative.size() != (long) unix.get("size")
                || !descriptorRelative.lastModifiedTime().equals(unix.get("lastModifiedTime"))) throw inventoryFailure();
        return new Stat(descriptorRelative.fileKey(), descriptorRelative.isDirectory(), descriptorRelative.size(),
                descriptorRelative.lastModifiedTime(), (FileTime) unix.get("ctime"), (int) unix.get("mode"), (int) unix.get("nlink"));
    }

    private static FrontendFailure inventoryFailure() { return FrontendFailure.of("JAVA_CAPTURE_INVENTORY", "capture"); }
    private static FrontendFailure pathFailure() { return FrontendFailure.of("JAVA_CAPTURE_PATH", "capture"); }
    private static FrontendFailure typeFailure() { return FrontendFailure.of("JAVA_CAPTURE_FILE_TYPE", "capture"); }
}
