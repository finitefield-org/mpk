package mpk.java2vir;

import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.security.MessageDigest;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.HexFormat;
import java.util.List;
import java.util.Map;
import java.util.concurrent.TimeUnit;

/** Private T07 owning executor, never packaged in the registered JAR. */
public final class RunnerTests {
    private RunnerTests() {}
    private static int assertions;
    private static void check(boolean condition, String label) {
        assertions++;
        if (!condition) throw new AssertionError(label);
    }
    private static void reject(String[] values, String label) {
        try { FrontendArguments.parse(values); }
        catch (IllegalArgumentException | FrontendFailure expected) { assertions++; return; }
        throw new AssertionError("accepted " + label);
    }
    public static void main(String[] arguments) throws Exception {
        check(arguments.length == 0, "no test options");
        String[] valid = Files.readAllLines(Path.of("/mpk/tests/runner-arguments.txt")).toArray(String[]::new);
        var request = FrontendArguments.parse(valid);
        check(request.selection().sources().equals(List.of("src/vector/Case.java")), "source selection");
        check(request.selection().contracts().equals(List.of("contracts/selected.json")), "contract selection");
        check(request.selection().methods().equals(List.of("vector.Case::f(int)->int")), "method selection");
        check(request.identity().distributionSha256().equals(JavaRelease.DISTRIBUTION), "registered distribution");
        for (int count = 0; count < valid.length; count++) reject(Arrays.copyOf(valid, count), "truncated " + count);
        for (int index = 0; index < valid.length; index++) {
            if (index > 0 && List.of("--compilation", "--source", "--contract", "--method", "--frontend-sha256", "--release-registry-sha256").contains(valid[index - 1])) continue;
            String[] changed = valid.clone(); changed[index] += "x";
            reject(changed, "fixed argument " + index);
        }
        var rejectedOptions = List.of("-javaagent:/host/agent.jar", "-agentlib:jdwp", "-agentpath:/host/native.so",
                "--java-home", "--class-path", "--module-path", "--patch-module", "-Xbootclasspath/a:/host",
                "-jar", "--source", "--processor", "-Xplugin:poison", "--restore", "--gradle", "--maven");
        for (String option : rejectedOptions) {
            var changed = new ArrayList<>(List.of(valid)); changed.add(option);
            reject(changed.toArray(String[]::new), option);
            changed.add(0, option); reject(changed.toArray(String[]::new), "prepend " + option);
        }
        for (String value : List.of("../Case.java", "/host/Case.java", "src/Case.class", "src/../Case.java", "src\\Case.java")) {
            String[] changed = valid.clone(); changed[List.of(valid).indexOf("--source") + 1] = value;
            reject(changed, "source path " + value);
        }
        for (String value : List.of("0".repeat(63), "0".repeat(65), "A".repeat(64), "x".repeat(64))) {
            String[] changed = valid.clone(); changed[List.of(valid).indexOf("--frontend-sha256") + 1] = value;
            reject(changed, "hash " + value);
        }
        var reordered = new ArrayList<>(List.of(valid));
        int source = reordered.indexOf("--source"), contract = reordered.indexOf("--contract");
        java.util.Collections.swap(reordered, source, contract);
        java.util.Collections.swap(reordered, source + 1, contract + 1);
        reject(reordered.toArray(String[]::new), "reordered argument groups");

        byte[] jar = Files.readAllBytes(Path.of("/work/java2vir.jar"));
        String hash = HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(jar));
        Path copy = Path.of("/work/verified.jar"); Files.write(copy, jar);
        RuntimePreflight.verifyFile(copy, hash, jar.length); assertions++;
        try { RuntimePreflight.verifyFile(copy, "0".repeat(64), jar.length); throw new AssertionError("changed hash accepted"); }
        catch (FrontendFailure expected) { check(expected.code().equals("JAVA_TOOLCHAIN_RUNTIME"), "changed hash"); }
        try { RuntimePreflight.verifyFile(copy, hash, jar.length - 1); throw new AssertionError("overrun accepted"); }
        catch (FrontendFailure expected) { check(expected.code().equals("JAVA_TOOLCHAIN_RUNTIME"), "file boundary"); }
        Path link = Path.of("/work/alias.jar"); Files.createSymbolicLink(link, copy);
        try { RuntimePreflight.verifyFile(link, hash, jar.length); throw new AssertionError("link accepted"); }
        catch (FrontendFailure expected) { check(expected.code().equals("JAVA_TOOLCHAIN_RUNTIME"), "nofollow identity"); }
        jar[jar.length - 1] ^= 1; Files.write(copy, jar);
        try { RuntimePreflight.verifyFile(copy, hash, jar.length); throw new AssertionError("modified jar accepted"); }
        catch (FrontendFailure expected) { check(expected.code().equals("JAVA_TOOLCHAIN_RUNTIME"), "modified jar"); }

        // Exercise the packaged Main with complete valid frontend arguments,
        // but an unregistered test classpath. /mpk/source is absent: metadata
        // must win before the capture error, with no partial artifacts.
        var command = new ArrayList<>(JavaRelease.PREFIX);
        command.set(command.indexOf("/mpk/frontend/java2vir.jar"), "/work/java2vir.jar");
        command.addAll(List.of(valid));
        var builder = new ProcessBuilder(command).directory(Path.of("/work").toFile());
        builder.environment().clear(); builder.environment().putAll(JavaRelease.ENVIRONMENT);
        Process child = builder.start();
        if (!child.waitFor(30, TimeUnit.SECONDS)) { child.destroyForcibly(); child.waitFor(); throw new AssertionError("Main timeout"); }
        byte[] stdout = child.getInputStream().readNBytes(65537), stderr = child.getErrorStream().readNBytes(65537);
        check(stdout.length <= 65536 && stderr.length == 0 && child.exitValue() == 1,
                "bounded metadata failure: exit=" + child.exitValue() + " stdout=" + new String(stdout, StandardCharsets.UTF_8)
                        + " stderr=" + new String(stderr, StandardCharsets.UTF_8));
        String envelope = new String(stdout, StandardCharsets.UTF_8);
        check(envelope.contains("JAVA_TOOLCHAIN_OPTIONS") && envelope.contains("\"phase\":\"metadata\"")
                && !envelope.contains("\"vir\"") && !envelope.contains("\"source_manifest\""), "release before source");
        System.out.write(CanonicalJson.encode(Map.of("schema", "mpk.java.runner_tests.v0", "assertions", assertions,
                "rejected_options", rejectedOptions, "precedence", "release_before_source", "envelope", envelope), "frontend_stdout", true));
    }
}
