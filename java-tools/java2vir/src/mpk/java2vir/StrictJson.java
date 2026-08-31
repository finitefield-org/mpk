package mpk.java2vir;

import java.nio.ByteBuffer;
import java.nio.charset.CharacterCodingException;
import java.nio.charset.CodingErrorAction;
import java.nio.charset.StandardCharsets;
import java.util.ArrayDeque;
import java.util.ArrayList;
import java.util.HashSet;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.TreeMap;

/** Narrowed strict JSON. The first pass validates the whole input without an
 * expression DOM, so bad JSON cannot be hidden by an earlier semantic error.
 * Views in the second pass retain only the bounded, closed contract members.
 */
final class StrictJson {
    private StrictJson() {}
    private static final int MAX_DEPTH = 768; // Shared strict JSON implementation ceiling.

    static Value validate(byte[] bytes) {
        FrontendLimits.check("contract_file_bytes", bytes.length, "subset");
        String text;
        try {
            text = StandardCharsets.UTF_8.newDecoder().onMalformedInput(CodingErrorAction.REPORT)
                    .onUnmappableCharacter(CodingErrorAction.REPORT).decode(ByteBuffer.wrap(bytes)).toString();
        } catch (CharacterCodingException error) { throw json(); }
        var cursor = new Cursor(text, 0, text.length());
        var stack = new ArrayDeque<Frame>();
        boolean value = true;
        for (;;) {
            if (value) {
                char next = cursor.peek();
                if (next == '{' || next == '[') {
                    if (stack.size() == MAX_DEPTH) throw json();
                    cursor.offset++;
                    stack.push(new Frame(next == '{'));
                } else if (next == '"') cursor.string();
                else if (next == 't') cursor.literal("true");
                else if (next == 'f') cursor.literal("false");
                else cursor.integer(); // null, floats and non-JSON spellings all reject.
                value = false;
            } else if (stack.isEmpty()) break;
            else {
                Frame frame = stack.peek();
                char close = frame.object ? '}' : ']';
                if (frame.state != 2 && cursor.take(close)) { stack.pop(); continue; }
                if (frame.state == 1) {
                    cursor.require(',');
                    frame.state = 2;
                    continue;
                }
                if (frame.object) {
                    String key = cursor.string();
                    if (!frame.keys.add(key)) throw json();
                    cursor.require(':');
                }
                frame.state = 1;
                value = true;
            }
        }
        cursor.whitespace();
        if (cursor.offset != text.length()) throw json();
        return new Value(text, 0, text.length());
    }

    private static final class Frame {
        final boolean object;
        final Set<String> keys;
        int state;
        Frame(boolean object) { this.object = object; keys = object ? new HashSet<>() : Set.of(); }
    }

    static final class Value {
        private final String text;
        private final int first;
        private final int last;
        private Value(String text, int first, int last) { this.text = text; this.first = first; this.last = last; }
        private Cursor cursor() { return new Cursor(text, first, last); }
        String raw() { return text.substring(first, last).strip(); }
        String string() {
            Cursor cursor = cursor();
            if (cursor.peek() != '"') throw shape();
            return cursor.string();
        }
        boolean bool() {
            String raw = raw();
            if (!raw.equals("true") && !raw.equals("false")) throw shape();
            return raw.equals("true");
        }
        Map<String, Value> fields(Set<String> allowed) {
            Cursor cursor = cursor();
            if (!cursor.take('{')) throw shape();
            var result = new TreeMap<String, Value>();
            if (!cursor.take('}')) for (;;) {
                String name = cursor.string();
                if (!allowed.contains(name)) throw shape();
                cursor.require(':');
                result.put(name, cursor.view());
                if (cursor.take('}')) break;
                cursor.require(',');
            }
            return Map.copyOf(result);
        }
        Map<String, Value> exact(Set<String> keys) {
            Map<String, Value> result = fields(keys);
            if (!result.keySet().equals(keys)) throw shape();
            return result;
        }
        List<Value> elements(int maximum, String code) {
            Cursor cursor = cursor();
            if (!cursor.take('[')) throw shape();
            var result = new ArrayList<Value>();
            if (!cursor.take(']')) for (;;) {
                if (result.size() == maximum) throw FrontendFailure.of(code, "subset");
                result.add(cursor.view());
                if (cursor.take(']')) break;
                cursor.require(',');
            }
            return List.copyOf(result);
        }
    }

    private static final class Cursor {
        final String text;
        final int end;
        int offset;
        Cursor(String text, int first, int last) { this.text = text; offset = first; end = last; }
        void whitespace() { while (offset < end && " \t\r\n".indexOf(text.charAt(offset)) >= 0) offset++; }
        char peek() { whitespace(); return offset < end ? text.charAt(offset) : '\0'; }
        boolean take(char c) { if (peek() != c) return false; offset++; return true; }
        void require(char c) { if (!take(c)) throw json(); }
        void literal(String value) {
            whitespace();
            if (offset + value.length() > end || !text.startsWith(value, offset)) throw json();
            offset += value.length();
        }
        void integer() {
            whitespace();
            int start = offset;
            if (offset < end && text.charAt(offset) == '-') offset++;
            if (offset >= end) throw json();
            char first = text.charAt(offset++);
            if (first < '0' || first > '9') throw json();
            if (first == '0') {
                if (offset < end && digit(text.charAt(offset))) throw json();
            } else while (offset < end && digit(text.charAt(offset))) offset++;
            if (offset < end && ".eE".indexOf(text.charAt(offset)) >= 0) throw json();
            long number;
            try { number = Long.parseLong(text.substring(start, offset)); }
            catch (NumberFormatException error) { throw json(); }
            if (number < -9007199254740991L || number > 9007199254740991L) throw json();
        }
        String string() {
            require('"');
            var out = new StringBuilder();
            while (offset < end) {
                char c = text.charAt(offset++);
                if (c == '"') return out.toString();
                if (c < 0x20) throw json();
                if (c == '\\') {
                    if (offset == end) throw json();
                    char escaped = text.charAt(offset++);
                    c = switch (escaped) {
                        case '"', '\\', '/' -> escaped;
                        case 'b' -> '\b'; case 'f' -> '\f'; case 'n' -> '\n'; case 'r' -> '\r'; case 't' -> '\t';
                        case 'u' -> hex();
                        default -> throw json();
                    };
                    if (Character.isHighSurrogate(c)) {
                        if (offset + 2 > end || !text.startsWith("\\u", offset)) throw json();
                        offset += 2;
                        char low = hex();
                        if (!Character.isLowSurrogate(low)) throw json();
                        out.append(c).append(low);
                        continue;
                    }
                    if (Character.isLowSurrogate(c)) throw json();
                } else if (Character.isHighSurrogate(c)) {
                    if (offset == end || !Character.isLowSurrogate(text.charAt(offset))) throw json();
                    out.append(c).append(text.charAt(offset++));
                    continue;
                } else if (Character.isLowSurrogate(c)) throw json();
                out.append(c);
            }
            throw json();
        }
        char hex() {
            if (offset + 4 > end) throw json();
            int value = 0;
            for (int i = 0; i < 4; i++) {
                char c = text.charAt(offset++);
                int digit = c >= '0' && c <= '9' ? c - '0' : c >= 'a' && c <= 'f' ? c - 'a' + 10 : c >= 'A' && c <= 'F' ? c - 'A' + 10 : -1;
                if (digit < 0) throw json();
                value = value * 16 + digit;
            }
            return (char) value;
        }
        // Input is already validated. Locate one value without retaining its children.
        Value view() {
            whitespace();
            int start = offset;
            int depth = 0;
            do {
                char c = peek();
                if (c == '"') string();
                else if (c == '{' || c == '[') { offset++; depth++; }
                else if (c == '}' || c == ']') { offset++; depth--; }
                else if (c == 't') literal("true");
                else if (c == 'f') literal("false");
                else if (c == ',' || c == ':') offset++;
                else integer();
            } while (depth != 0);
            return new Value(text, start, offset);
        }
    }
    private static boolean digit(char c) { return c >= '0' && c <= '9'; }
    private static FrontendFailure json() { return FrontendFailure.of("JAVA_CONTRACT_JSON", "subset"); }
    private static FrontendFailure shape() { return FrontendFailure.of("JAVA_CONTRACT_SHAPE", "subset"); }
}
