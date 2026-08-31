package mpk.java2vir;

import java.io.IOException;
import java.util.ArrayList;
import java.util.Collections;
import java.util.IdentityHashMap;
import java.util.Iterator;
import java.util.List;
import java.util.Map;
import java.util.ServiceLoader;
import java.util.Set;
import java.util.TreeMap;
import javax.tools.FileObject;
import javax.tools.ForwardingJavaFileManager;
import javax.tools.JavaFileObject;
import javax.tools.StandardJavaFileManager;
import javax.tools.StandardLocation;

/** Closes the application view. --release uses a separate javac platform manager;
 * its closure comes from the pinned JDK and native filesystem boundary, not this wrapper.
 */
final class ClosedFileManager extends ForwardingJavaFileManager<StandardJavaFileManager> {
    private final Set<Location> system = Collections.newSetFromMap(new IdentityHashMap<>());
    private final Set<FileObject> issued = Collections.newSetFromMap(new IdentityHashMap<>());
    private final Set<FileObject> sources = Collections.newSetFromMap(new IdentityHashMap<>());
    private final Map<String, Long> calls = new TreeMap<>();
    private String phase = "metadata";
    private boolean closed;
    private int outputAttempts;
    private int systemFiles;

    ClosedFileManager(StandardJavaFileManager base, List<SourceText> sources) throws IOException {
        super(base);
        this.sources.addAll(sources);
        system.add(StandardLocation.SYSTEM_MODULES);
        for (StandardLocation location : List.of(StandardLocation.CLASS_PATH, StandardLocation.SOURCE_PATH,
                StandardLocation.MODULE_PATH, StandardLocation.UPGRADE_MODULE_PATH,
                StandardLocation.ANNOTATION_PROCESSOR_PATH, StandardLocation.ANNOTATION_PROCESSOR_MODULE_PATH))
            base.setLocationFromPaths(location, List.of());
    }

    void phase(String phase) { this.phase = phase; }
    boolean closed() { return closed; }
    int outputAttempts() { return outputAttempts; }
    int systemFiles() { return systemFiles; }
    Map<String, Long> calls() { return Map.copyOf(calls); }
    private boolean allowed(Location location) { return system.contains(location); }
    private FrontendFailure failure() { return FrontendFailure.of("JAVA_TOOLCHAIN_FILE_MANAGER", phase); }

    private void touch(String operation, Location location) {
        if (closed) throw failure();
        String label = location == null ? "-" : location instanceof StandardLocation || allowed(location)
                ? location.getName() : "unknown";
        calls.merge(operation + ":" + label, 1L, Math::addExact);
    }

    JavaFileObject verifySystem(JavaFileObject file) {
        if (file == null) return null;
        var location = file.toUri();
        if (!location.equals(location.normalize()) || location.getRawQuery() != null || location.getRawFragment() != null
                || file.getKind() != JavaFileObject.Kind.CLASS) throw failure();
        String uri = location.toASCIIString();
        if (!uri.startsWith("jrt:/") && !uri.startsWith("jar:file:/mpk/toolchain/jdk/lib/ct.sym!/")) throw failure();
        issued.add(file);
        systemFiles++;
        return file;
    }

    private void requireIssued(FileObject file) { if (!issued.contains(file)) throw failure(); }

    @Override public boolean hasLocation(Location location) {
        touch("hasLocation", location);
        return allowed(location) && fileManager.hasLocation(location);
    }
    @Override public Iterable<JavaFileObject> list(Location location, String pkg,
                                                  Set<JavaFileObject.Kind> kinds, boolean recurse) throws IOException {
        touch("list", location);
        if (!allowed(location)) return List.of();
        Iterable<JavaFileObject> files = fileManager.list(location, pkg, kinds, recurse);
        return () -> new Iterator<>() {
            private final Iterator<JavaFileObject> iterator = files.iterator();
            @Override public boolean hasNext() { if (closed) throw failure(); return iterator.hasNext(); }
            @Override public JavaFileObject next() { if (closed) throw failure(); return verifySystem(iterator.next()); }
        };
    }
    @Override public JavaFileObject getJavaFileForInput(Location location, String name, JavaFileObject.Kind kind) throws IOException {
        touch("getJavaFileForInput", location);
        return allowed(location) ? verifySystem(fileManager.getJavaFileForInput(location, name, kind)) : null;
    }
    @Override public FileObject getFileForInput(Location location, String pkg, String name) {
        touch("getFileForInput", location);
        return null;
    }
    @Override public String inferBinaryName(Location location, JavaFileObject file) {
        touch("inferBinaryName", location);
        if (!allowed(location)) return null;
        requireIssued(file);
        return fileManager.inferBinaryName(location, file);
    }
    @Override public boolean contains(Location location, FileObject file) throws IOException {
        touch("contains", location);
        if (!allowed(location)) return false;
        requireIssued(file);
        return fileManager.contains(location, file);
    }
    @Override public ClassLoader getClassLoader(Location location) { touch("getClassLoader", location); return null; }
    @Override public <S> ServiceLoader<S> getServiceLoader(Location location, Class<S> service) {
        touch("getServiceLoader", location);
        throw failure();
    }
    @Override public String inferModuleName(Location location) throws IOException {
        touch("inferModuleName", location);
        return allowed(location) ? fileManager.inferModuleName(location) : null;
    }
    @Override public Iterable<Set<Location>> listLocationsForModules(Location location) throws IOException {
        touch("listLocationsForModules", location);
        if (!allowed(location)) return List.of();
        var result = new ArrayList<Set<Location>>();
        for (Set<Location> group : fileManager.listLocationsForModules(location)) {
            system.addAll(group);
            result.add(Set.copyOf(group));
        }
        return List.copyOf(result);
    }
    @Override public Location getLocationForModule(Location location, String module) throws IOException {
        touch("getLocationForModuleName", location);
        if (!allowed(location)) return null;
        Location result = fileManager.getLocationForModule(location, module);
        if (result != null) system.add(result);
        return result;
    }
    @Override public Location getLocationForModule(Location location, JavaFileObject file) throws IOException {
        touch("getLocationForModuleFile", location);
        if (!allowed(location)) return null;
        requireIssued(file);
        Location result = fileManager.getLocationForModule(location, file);
        if (result != null) system.add(result);
        return result;
    }
    @Override public JavaFileObject getJavaFileForOutput(Location location, String name,
                                                        JavaFileObject.Kind kind, FileObject sibling) {
        touch("getJavaFileForOutput", location); outputAttempts++; throw failure();
    }
    @Override public FileObject getFileForOutput(Location location, String pkg, String name, FileObject sibling) {
        touch("getFileForOutput", location); outputAttempts++; throw failure();
    }
    @Override public JavaFileObject getJavaFileForOutputForOriginatingFiles(Location location, String name,
            JavaFileObject.Kind kind, FileObject... origins) {
        touch("getJavaFileForOutputForOriginatingFiles", location); outputAttempts++; throw failure();
    }
    @Override public FileObject getFileForOutputForOriginatingFiles(Location location, String pkg,
                                                                   String name, FileObject... origins) {
        touch("getFileForOutputForOriginatingFiles", location); outputAttempts++; throw failure();
    }
    @Override public boolean isSameFile(FileObject left, FileObject right) {
        touch("isSameFile", null);
        if (sources.contains(left) || sources.contains(right)) return left == right;
        requireIssued(left); requireIssued(right);
        return fileManager.isSameFile(left, right);
    }
    @Override public boolean handleOption(String option, Iterator<String> remaining) {
        touch("handleOption", null);
        if (!option.equals("--multi-release") && !option.equals("-encoding")) return false;
        if (!remaining.hasNext()) throw FrontendFailure.of("JAVA_TOOLCHAIN_OPTIONS", "metadata");
        String value = remaining.next();
        if (!value.equals(option.equals("-encoding") ? "UTF-8" : "25"))
            throw FrontendFailure.of("JAVA_TOOLCHAIN_OPTIONS", "metadata");
        return fileManager.handleOption(option, List.of(value).iterator());
    }
    @Override public int isSupportedOption(String option) {
        touch("isSupportedOption", null);
        return option.equals("--multi-release") || option.equals("-encoding") ? fileManager.isSupportedOption(option) : -1;
    }
    @Override public void flush() throws IOException { touch("flush", null); fileManager.flush(); }
    @Override public void close() throws IOException {
        if (!closed) { closed = true; fileManager.close(); }
    }
}
