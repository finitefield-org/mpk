package mpk.java2vir;

import javax.tools.JavaCompiler;
import javax.tools.ToolProvider;

/** Runtime smoke check; byte-level identity is enforced by the build owner. */
final class BuildIdentity {
    private BuildIdentity() {}

    static boolean matches() {
        try {
            JavaCompiler compiler = ToolProvider.getSystemJavaCompiler();
            return "25.0.4.1+1-LTS".equals(System.getProperty("java.runtime.version"))
                    && "Eclipse Adoptium".equals(System.getProperty("java.vendor"))
                    && "25".equals(System.getProperty("java.specification.version"))
                    && compiler != null
                    && "com.sun.tools.javac.api.JavacTool".equals(compiler.getClass().getName());
        } catch (RuntimeException | LinkageError failure) {
            return false;
        }
    }
}
