package mpk.java2vir;

import java.util.List;
import java.util.Map;

/** Closed T07 candidate identity. None of these values activates an installed MPK route. */
final class JavaRelease {
    private JavaRelease() {}
    static final String FRONTEND_ID = "frontend.java.java2vir.candidate.v1";
    static final String TOOLCHAIN_ID = "toolchain.java.temurin-25_0_4_1_1.candidate.v1";
    static final String DISTRIBUTION = "8f6c540278984d0a8f94f3d288ab94fa84fe165a023c2a376b26bfa955d0e8e1";
    static final String JAVA_SHA256 = "7380ce48ed5013735d2c8414db54adb8f981e7933ff594bd36f3baccddaafba3";
    static final String JDK_CONTENT = "43000475a958f37bc5859e9223bd5316dce9285abd0ac3c192cd2fe1fb7aac25";
    static final String NATIVE_CONTENT = "23730c576e8e618a2580cafef7e9535890b46b68eb274541f7806aea70f8be4e";
    static final List<String> PREFIX = List.of(
            "/mpk/toolchain/jdk/bin/java", "-Xint", "-Xshare:off", "-XX:+UseSerialGC", "-XX:ActiveProcessorCount=1",
            "-XX:+DisableAttachMechanism", "-XX:-UsePerfData", "-Xms32m", "-Xmx512m", "-Xss1m",
            "-Dfile.encoding=UTF-8", "-Duser.language=en", "-Duser.country=US", "-Duser.timezone=UTC",
            "-Djava.io.tmpdir=/mpk/tmp", "-Duser.home=/mpk/empty-home", "-Djava.library.path=/nonexistent",
            "-XX:ErrorFile=/mpk/tmp/hs_err.log", "-XX:-CreateCoredumpOnCrash", "-XX:-HeapDumpOnOutOfMemoryError",
            "--limit-modules", "java.base,java.compiler,jdk.compiler,jdk.zipfs",
            "--add-modules", "java.compiler,jdk.compiler,jdk.zipfs", "-cp", "/mpk/frontend/java2vir.jar", "mpk.java2vir.Main");
    static final Map<String, String> ENVIRONMENT = Map.of("HOME", "/mpk/empty-home", "TMPDIR", "/mpk/tmp",
            "PATH", "/nonexistent", "LANG", "C.UTF-8", "LC_ALL", "C.UTF-8", "TZ", "UTC");

    static List<Map<String, Object>> components() {
        return List.of(Map.of("kind", "executable", "name", "java", "release", "25.0.4.1+1", "binary_sha256", JAVA_SHA256),
                Map.of("kind", "content", "name", "jdk", "release", "25.0.4.1+1", "content_sha256", JDK_CONTENT),
                Map.of("kind", "content", "name", "native-runtime", "release", "glibc-2.36", "content_sha256", NATIVE_CONTENT));
    }
}
