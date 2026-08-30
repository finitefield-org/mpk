import java.nio.file.Files;
import java.nio.file.Path;
import java.net.NetworkInterface;
import java.util.Collections;
import java.util.List;
import java.util.Map;
import java.util.TreeMap;
import java.util.TreeSet;

/** Disposable T01 measurement code, never a frontend or proof-acceptance path. */
public final class RuntimeProbe {
    private static String quote(String value) {
        return "\"" + value.replace("\\", "\\\\").replace("\"", "\\\"")
                .replace("\n", "\\n").replace("\r", "\\r").replace("\t", "\\t") + "\"";
    }

    private static String strings(Iterable<String> values) {
        StringBuilder result = new StringBuilder("[");
        for (String value : values) {
            if (result.length() > 1) result.append(',');
            result.append(quote(value));
        }
        return result.append(']').toString();
    }

    public static void main(String[] args) throws Exception {
        TreeMap<String, String> report = new TreeMap<>();
        TreeMap<String, Boolean> checks = new TreeMap<>();
        report.put("java_runtime_version", quote(System.getProperty("java.runtime.version")));
        report.put("os_arch", quote(System.getProperty("os.arch")));
        checks.put("pinned_runtime", "25.0.4.1+1-LTS".equals(System.getProperty("java.runtime.version")));
        checks.put("one_processor", Runtime.getRuntime().availableProcessors() == 1);
        checks.put("heap_ceiling", Runtime.getRuntime().maxMemory() <= 536870912L);
        checks.put("environment_closed", System.getenv().equals(Map.of(
                "HOME", "/mpk/empty-home", "TMPDIR", "/mpk/tmp", "PATH", "/nonexistent",
                "LANG", "C.UTF-8", "LC_ALL", "C.UTF-8", "TZ", "UTC")));
        String status = Files.readString(Path.of("/proc/self/status"));
        checks.put("nonroot_uid", status.lines().anyMatch(s -> s.matches("Uid:\\s+65534\\s+65534\\s+65534\\s+65534")));
        checks.put("no_capabilities", status.lines().anyMatch(s -> s.matches("CapEff:\\s+0+")));
        checks.put("no_new_privileges", status.lines().anyMatch(s -> s.matches("NoNewPrivs:\\s+1")));
        TreeSet<String> interfaces = new TreeSet<>();
        for (NetworkInterface item : Collections.list(NetworkInterface.getNetworkInterfaces())) {
            interfaces.add(item.getName());
        }
        report.put("network_interfaces", strings(interfaces));
        checks.put("loopback_only", interfaces.equals(new TreeSet<>(List.of("lo"))));
        checks.put("no_host_home", !Files.exists(Path.of("/Users")) && !Files.exists(Path.of("/root")));
        checks.put("no_host_configuration", !Files.exists(Path.of("/etc")) && !Files.exists(Path.of("/sys")));
        String mounts = Files.readString(Path.of("/proc/self/mountinfo"));
        checks.put("private_tmpfs_noswap", mounts.lines().anyMatch(s -> s.contains(" /mpk/tmp ")
                && s.contains("noexec") && s.contains(" - tmpfs ") && s.contains("noswap")));
        checks.put("readonly_proc", mounts.lines().anyMatch(s -> s.contains(" /proc ro,") && s.contains(" - proc ")));
        checks.put("readonly_jdk", mounts.lines().anyMatch(s -> s.contains(" /mpk/toolchain/jdk ro,")));
        checks.put("readonly_root", mounts.lines().anyMatch(s -> s.contains(" / ro,")));
        Path temporary = Path.of("/mpk/tmp/runtime-probe");
        Files.writeString(temporary, "bounded temporary storage\n");
        checks.put("temporary_write", Files.readString(temporary).equals("bounded temporary storage\n"));
        Files.delete(temporary);
        TreeSet<String> nativeFiles = new TreeSet<>();
        for (String line : Files.readAllLines(Path.of("/proc/self/maps"))) {
            int start = line.indexOf('/');
            if (start >= 0) {
                String path = line.substring(start);
                if (path.startsWith("/mpk/toolchain/jdk/") || path.startsWith("/lib/")) {
                    if (path.contains(".so")) nativeFiles.add(path);
                } else if (path.startsWith("/lib64/")) nativeFiles.add(path);
            }
        }
        report.put("loaded_native_files", strings(nativeFiles));
        StringBuilder checkJson = new StringBuilder("{");
        for (var item : checks.entrySet()) {
            if (checkJson.length() > 1) checkJson.append(',');
            checkJson.append(quote(item.getKey())).append(':').append(item.getValue());
        }
        report.put("checks", checkJson.append('}').toString());
        StringBuilder output = new StringBuilder("{");
        for (var item : report.entrySet()) {
            if (output.length() > 1) output.append(',');
            output.append(quote(item.getKey())).append(':').append(item.getValue());
        }
        System.out.println(output.append('}'));
        if (checks.containsValue(false)) System.exit(1);
    }
}
