package poison;
import java.nio.file.*;
import java.util.Set;
import javax.annotation.processing.*;
import javax.lang.model.SourceVersion;
import javax.lang.model.element.TypeElement;
@SupportedAnnotationTypes("*")
@SupportedSourceVersion(SourceVersion.RELEASE_25)
public final class ProbeProcessor extends AbstractProcessor {
    @Override public boolean process(Set<? extends TypeElement> annotations, RoundEnvironment round) {
        try { Files.writeString(Path.of("/tmp/processor-executed"), "unexpected processor execution\n"); }
        catch (java.io.IOException e) { throw new java.io.UncheckedIOException(e); }
        throw new IllegalStateException("planted processor must never run");
    }
}
