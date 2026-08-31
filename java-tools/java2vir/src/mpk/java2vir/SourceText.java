package mpk.java2vir;

import java.net.URI;
import java.nio.ByteBuffer;
import java.nio.charset.CharacterCodingException;
import java.nio.charset.CodingErrorAction;
import java.nio.charset.StandardCharsets;
import java.util.Arrays;
import javax.tools.SimpleJavaFileObject;

/** Original, immutable source and its checked UTF-16 to UTF-8 boundary table. */
final class SourceText extends SimpleJavaFileObject {
    private final String path;
    private final String text;
    private final int[] boundaries;

    SourceText(String path, byte[] bytes) {
        super(sourceUri(path), Kind.SOURCE);
        this.path = path;
        if (!Selection.sourcePath(path)) throw FrontendFailure.of("JAVA_CAPTURE_PATH", "capture");
        FrontendLimits.check("source_file_bytes", bytes.length, "source");
        if (bytes.length == 0 || bytes[bytes.length - 1] != '\n') throw encoding();
        for (int i = 0; i < bytes.length; i++) {
            if (bytes[i] == 0 || bytes[i] == '\r'
                    || (bytes[i] == '\\' && i + 1 < bytes.length && bytes[i + 1] == 'u')) throw encoding();
        }
        try {
            text = StandardCharsets.UTF_8.newDecoder()
                    .onMalformedInput(CodingErrorAction.REPORT)
                    .onUnmappableCharacter(CodingErrorAction.REPORT)
                    .decode(ByteBuffer.wrap(bytes)).toString();
        } catch (CharacterCodingException error) {
            throw encoding();
        }
        if (text.charAt(0) == '\ufeff') throw encoding();
        boundaries = new int[text.length() + 1];
        Arrays.fill(boundaries, -1);
        int offset = 0;
        for (int i = 0; i < text.length();) {
            int scalar = text.codePointAt(i);
            if ((scalar >= 0xfdd0 && scalar <= 0xfdef) || (scalar & 0xffff) >= 0xfffe) throw encoding();
            boundaries[i] = offset;
            offset += scalar < 0x80 ? 1 : scalar < 0x800 ? 2 : scalar < 0x10000 ? 3 : 4;
            i += Character.charCount(scalar);
        }
        boundaries[text.length()] = offset;
        if (offset != bytes.length) throw encoding();
    }

    String path() { return path; }
    String text() { return text; }
    int byteLength() { return boundaries[text.length()]; }
    @Override public CharSequence getCharContent(boolean ignoreEncodingErrors) { return text; }

    private static URI sourceUri(String path) {
        if (path != null) FrontendLimits.check("normalized_path_bytes", path.length(), "capture");
        if (!Selection.sourcePath(path)) throw FrontendFailure.of("JAVA_CAPTURE_PATH", "capture");
        return URI.create("mpk-source:///" + path);
    }

    int byteOffset(long utf16) {
        if (utf16 < 0 || utf16 >= boundaries.length || boundaries[(int) utf16] < 0)
            throw FrontendFailure.of("JAVA_TOOLCHAIN_ADAPTER", "typecheck");
        return boundaries[(int) utf16];
    }

    FrontendFailure.Span span(long start, long end, boolean allowEmpty, String phase) {
        try {
            int first = byteOffset(start);
            int last = byteOffset(end);
            if (last < first || (!allowEmpty && last == first))
                throw FrontendFailure.of("JAVA_TOOLCHAIN_ADAPTER", phase);
            return first == last ? null : new FrontendFailure.Span(path, first, last);
        } catch (FrontendFailure error) {
            throw FrontendFailure.of("JAVA_TOOLCHAIN_ADAPTER", phase);
        }
    }

    private static FrontendFailure encoding() {
        return FrontendFailure.of("JAVA_SOURCE_ENCODING", "source");
    }
}
