package mpk.java2vir;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.LinkOption;
import java.nio.file.Path;
import java.nio.file.attribute.BasicFileAttributes;
import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.HexFormat;
import java.util.List;
import java.util.Map;

/** Defense in depth after the parent's descriptor, inventory and native isolation preflight. */
final class RuntimePreflight {
    private RuntimePreflight() {}
    static void validate(FrontendArguments.Request request) {
        if (!BuildIdentity.matches()) throw failure("COMPILER");
        if (!System.getenv().equals(JavaRelease.ENVIRONMENT)) throw failure("OPTIONS");
        for (var property : Map.ofEntries(Map.entry("os.name", "Linux"), Map.entry("os.arch", "amd64"),
                Map.entry("java.home", "/mpk/toolchain/jdk"), Map.entry("java.class.path", "/mpk/frontend/java2vir.jar"),
                Map.entry("user.dir", "/mpk/source"), Map.entry("user.home", "/mpk/empty-home"),
                Map.entry("java.io.tmpdir", "/mpk/tmp"), Map.entry("java.library.path", "/nonexistent"),
                Map.entry("file.encoding", "UTF-8"), Map.entry("user.language", "en"), Map.entry("user.country", "US"),
                Map.entry("user.timezone", "UTC")).entrySet())
            if (!property.getValue().equals(System.getProperty(property.getKey()))) throw failure("OPTIONS");
        try {
            var expected = new ArrayList<>(JavaRelease.PREFIX);
            expected.addAll(request.arguments());
            byte[] command = boundedProc("cmdline", 131072);
            if (command.length == 0 || command[command.length - 1] != 0) throw failure("OPTIONS");
            List<String> actual = Arrays.asList(new String(command, 0, command.length - 1, StandardCharsets.UTF_8).split("\u0000", -1));
            if (!actual.equals(expected)) throw failure("OPTIONS");
            Map<String, String> status = new java.util.HashMap<>();
            for (String line : new String(boundedProc("status", 65536), StandardCharsets.US_ASCII).split("\n")) {
                int colon = line.indexOf(':');
                if (colon > 0) status.put(line.substring(0, colon), line.substring(colon + 1).trim());
            }
            if (!"1".equals(status.get("NoNewPrivs")) || !"2".equals(status.get("Seccomp"))
                    || !"1".equals(status.get("Pid")) || !"".equals(status.get("Groups"))) throw failure("RUNTIME");
            for (String name : List.of("CapInh", "CapPrm", "CapEff", "CapBnd", "CapAmb"))
                if (!"0000000000000000".equals(status.get(name))) throw failure("RUNTIME");
            for (String name : List.of("Uid", "Gid"))
                if (!List.of("65534", "65534", "65534", "65534").equals(Arrays.asList(status.getOrDefault(name, "").split("\\s+")))) throw failure("RUNTIME");
            for (String name : List.of("/", "/proc", "/mpk/source", "/mpk/frontend", "/mpk/toolchain", "/mpk/native-runtime"))
                if (!Files.getFileStore(Path.of(name)).isReadOnly()) throw failure("RUNTIME");
            if (Runtime.getRuntime().availableProcessors() != 1 || Runtime.getRuntime().maxMemory() > 512L * 1024 * 1024)
                throw failure("RUNTIME");
            if (!Main.class.getProtectionDomain().getCodeSource().getLocation().toExternalForm().equals("file:/mpk/frontend/java2vir.jar")) throw failure("RUNTIME");
            verifyFile(Path.of("/mpk/frontend/java2vir.jar"), request.identity().frontendSha256(), 16L * 1024 * 1024);
            verifyFile(Path.of("/mpk/toolchain/jdk/bin/java"), JavaRelease.JAVA_SHA256, 1024 * 1024);
            verifyFile(Path.of("/mpk/toolchain/jdk/lib/modules"), "6b11db4c84e8ac3b9500397753e02ad03450e904fa8656eb8fd2a2197e536b57", 256L * 1024 * 1024);
            verifyFile(Path.of("/mpk/toolchain/jdk/lib/server/libjvm.so"), "5e0fb6e83a5676090f28ff0b453e9199df06af1ff90d473c1b9837e41096114c", 128L * 1024 * 1024);
            verifyFile(Path.of("/mpk/toolchain/jdk/release"), "44be64b383baa18668afefbe9a780ae3a9d730a066eaaa92500f77bd1e4b934c", 65536);
        } catch (IOException | SecurityException error) { throw failure("RUNTIME"); }
    }
    private static byte[] boundedProc(String name, int maximum) throws IOException {
        // /proc/self is the kernel-owned link inside the parent's private PID proc mount.
        try (var input = Files.newInputStream(Path.of("/proc/self/" + name))) {
            byte[] bytes = input.readNBytes(maximum + 1);
            if (bytes.length > maximum) throw failure("RUNTIME");
            return bytes;
        }
    }
    static void verifyFile(Path path, String expected, long maximum) throws IOException {
        var before = Files.readAttributes(path, BasicFileAttributes.class, LinkOption.NOFOLLOW_LINKS);
        if (!before.isRegularFile() || before.size() > maximum) throw failure("RUNTIME");
        try (var input = Files.newInputStream(path, LinkOption.NOFOLLOW_LINKS)) {
            MessageDigest digest = MessageDigest.getInstance("SHA-256");
            byte[] buffer = new byte[65536];
            long total = 0;
            for (int length; (length = input.read(buffer)) != -1;) {
                total += length;
                if (total > maximum || total > before.size()) throw failure("RUNTIME");
                digest.update(buffer, 0, length);
            }
            var after = Files.readAttributes(path, BasicFileAttributes.class, LinkOption.NOFOLLOW_LINKS);
            if (total != before.size() || !before.fileKey().equals(after.fileKey()) || before.size() != after.size()
                    || !before.lastModifiedTime().equals(after.lastModifiedTime())
                    || !HexFormat.of().formatHex(digest.digest()).equals(expected)) throw failure("RUNTIME");
        } catch (NoSuchAlgorithmException error) { throw failure("RUNTIME"); }
    }
    private static FrontendFailure failure(String code) { return FrontendFailure.of("JAVA_TOOLCHAIN_" + code, "metadata"); }
}
