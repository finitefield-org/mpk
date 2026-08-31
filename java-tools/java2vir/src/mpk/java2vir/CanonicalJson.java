package mpk.java2vir;

import java.nio.charset.StandardCharsets;
import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.util.HexFormat;
import java.util.List;
import java.util.Map;
import java.util.TreeMap;

/** Integer-only JCS. Count before allocating bytes; never construct an unbounded JSON string. */
final class CanonicalJson {
    private CanonicalJson() {}
    static final class ByteCounter {
        private final String name;
        private long bytes;
        ByteCounter(String name) { this.name = name; }
        void append(long length) { bytes = FrontendLimits.add(name, bytes, length, "emission"); }
        long bytes() { return bytes; }
    }
    @FunctionalInterface private interface Sink { void append(int value); }
    private static final class Writer {
        final ByteCounter counter;
        final Sink sink;
        Writer(String limit, Sink sink) { counter = new ByteCounter(limit); this.sink = sink; }
        void byteValue(int value) { counter.append(1); if (sink != null) sink.append(value); }
        void ascii(String text) { for (int n = 0; n < text.length(); n++) byteValue(text.charAt(n)); }
        void string(String text) {
            byteValue('"');
            for (int n = 0; n < text.length(); n++) {
                int scalar = text.charAt(n);
                switch (scalar) {
                    case '"' -> ascii("\\\""); case '\\' -> ascii("\\\\");
                    case '\b' -> ascii("\\b"); case '\t' -> ascii("\\t"); case '\n' -> ascii("\\n");
                    case '\f' -> ascii("\\f"); case '\r' -> ascii("\\r");
                    default -> {
                        if (scalar < 0x20) {
                            ascii("\\u00"); byteValue(Character.forDigit(scalar >>> 4, 16)); byteValue(Character.forDigit(scalar & 15, 16));
                        } else {
                            if (Character.isSurrogate((char) scalar)) {
                                if (!Character.isHighSurrogate((char) scalar) || n + 1 == text.length()
                                        || !Character.isLowSurrogate(text.charAt(n + 1))) throw internal();
                                scalar = Character.toCodePoint((char) scalar, text.charAt(++n));
                            }
                            if (scalar < 0x80) byteValue(scalar);
                            else if (scalar < 0x800) { byteValue(0xc0 | scalar >>> 6); byteValue(0x80 | scalar & 63); }
                            else if (scalar < 0x10000) {
                                byteValue(0xe0 | scalar >>> 12); byteValue(0x80 | scalar >>> 6 & 63); byteValue(0x80 | scalar & 63);
                            } else {
                                byteValue(0xf0 | scalar >>> 18); byteValue(0x80 | scalar >>> 12 & 63);
                                byteValue(0x80 | scalar >>> 6 & 63); byteValue(0x80 | scalar & 63);
                            }
                        }
                    }
                }
            }
            byteValue('"');
        }
        void value(Object value, int depth) {
            if (depth > 256) throw internal();
            if (value instanceof String string) string(string);
            else if (value instanceof Boolean || value instanceof Integer || value instanceof Long) ascii(value.toString());
            else if (value instanceof List<?> list) {
                byteValue('['); boolean first = true;
                for (Object item : list) { if (!first) byteValue(','); first = false; value(item, depth + 1); }
                byteValue(']');
            } else if (value instanceof Map<?, ?> map) {
                var sorted = new TreeMap<String, Object>();
                for (var entry : map.entrySet()) {
                    if (!(entry.getKey() instanceof String key)) throw internal();
                    sorted.put(key, entry.getValue());
                }
                byteValue('{'); boolean first = true;
                for (var entry : sorted.entrySet()) {
                    if (!first) byteValue(','); first = false;
                    string(entry.getKey()); byteValue(':'); value(entry.getValue(), depth + 1);
                }
                byteValue('}');
            } else if (value == null) ascii("null");
            else throw internal();
        }
    }

    static long measure(Object value, String limit, boolean lf) {
        var writer = new Writer(limit, null);
        writer.value(value, 1);
        if (lf) writer.byteValue('\n');
        return writer.counter.bytes();
    }
    static byte[] encode(Object value, String limit, boolean lf) {
        long size = measure(value, limit, lf);
        byte[] bytes = new byte[Math.toIntExact(size)];
        int[] position = {0};
        var writer = new Writer(limit, next -> bytes[position[0]++] = (byte) next);
        writer.value(value, 1);
        if (lf) writer.byteValue('\n');
        if (position[0] != bytes.length) throw internal();
        return bytes;
    }
    static String hash(String domain, Object value, String limit) {
        try {
            MessageDigest digest = MessageDigest.getInstance("SHA-256");
            digest.update(domain.getBytes(StandardCharsets.US_ASCII)); digest.update((byte) 0);
            var writer = new Writer(limit, next -> digest.update((byte) next));
            writer.value(value, 1);
            return HexFormat.of().formatHex(digest.digest());
        } catch (NoSuchAlgorithmException error) { throw internal(); }
    }
    static Map<String, Object> artifact(Map<String, Object> payload, String hashField, String domain, String limit) {
        var result = new TreeMap<String, Object>(payload);
        if (result.containsKey(hashField)) throw internal();
        result.put(hashField, hash(domain, payload, limit));
        measure(result, limit, false);
        return Map.copyOf(result);
    }
    private static FrontendFailure internal() { return FrontendFailure.of("JAVA_FRONTEND_INTERNAL", "emission"); }
}
