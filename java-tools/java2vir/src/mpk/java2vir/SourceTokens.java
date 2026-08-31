package mpk.java2vir;

/** Raw spelling cursor, not a Java parser. Admission always uses the public trees.
 * Comments and Java whitespace are skipped without normalizing identifier bytes.
 * Tokens are consumed on demand; a file-sized second token inventory is not built.
 */
final class SourceTokens {
    private final String text;
    private final int end;
    private int offset;

    SourceTokens(TreeInventory.Node node) {
        node.source().span(node.start(), node.end(), false, "typecheck");
        text = node.source().text();
        offset = Math.toIntExact(node.start());
        end = Math.toIntExact(node.end());
    }

    private SourceTokens(String text, int first, int last) {
        this.text = text;
        offset = first;
        end = last;
    }

    void triviaTo(long position, String code) {
        if (position < offset || position > end) throw adapter();
        new SourceTokens(text, offset, Math.toIntExact(position)).expect("", code);
        offset = Math.toIntExact(position);
    }

    void seek(long position) {
        if (position < offset || position > end) throw adapter();
        offset = Math.toIntExact(position);
    }

    String next() {
        for (;;) {
            while (offset < end && " \t\n\f".indexOf(text.charAt(offset)) >= 0) offset++;
            if (offset + 1 >= end || text.charAt(offset) != '/') break;
            if (text.charAt(offset + 1) == '/') {
                offset += 2;
                while (offset < end && text.charAt(offset) != '\n') offset++;
            } else if (text.charAt(offset + 1) == '*') {
                int close = text.indexOf("*/", offset + 2);
                if (close < 0 || close + 2 > end) throw adapter();
                offset = close + 2;
            } else break;
        }
        if (offset == end) return "";
        int first = offset++;
        char c = text.charAt(first);
        if ("(){}[];,.=+-*/%&|^!~<>?:@".indexOf(c) >= 0) return text.substring(first, offset);
        // Keep non-ASCII and identifier-ignorable characters in the raw token.
        while (offset < end && " \t\n\f(){}[];,.=+-*/%&|^!~<>?:@".indexOf(text.charAt(offset)) < 0) offset++;
        return text.substring(first, offset);
    }

    void expect(String wanted, String code) {
        if (!next().equals(wanted)) throw FrontendFailure.of(code, "subset");
    }

    private static FrontendFailure adapter() { return FrontendFailure.of("JAVA_TOOLCHAIN_ADAPTER", "typecheck"); }
}
