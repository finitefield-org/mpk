package mpk.java2vir;

import java.util.Map;
import javax.lang.model.type.TypeKind;
import javax.lang.model.type.TypeMirror;

/** The only source and contract value types in the frozen Java profile. */
enum ScalarType {
    BOOLEAN("boolean", TypeKind.BOOLEAN, 1), INT("int", TypeKind.INT, 1), LONG("long", TypeKind.LONG, 2);

    final String keyword;
    final TypeKind kind;
    final int slots;
    ScalarType(String keyword, TypeKind kind, int slots) {
        this.keyword = keyword;
        this.kind = kind;
        this.slots = slots;
    }
    boolean integer() { return this != BOOLEAN; }
    int width() { return this == INT ? 32 : 64; }
    Map<String, Object> vir() {
        return this == BOOLEAN ? Map.of("kind", "bool") : Map.of("kind", "bv", "signed", true, "width", width());
    }
    static ScalarType keyword(String text) {
        for (ScalarType type : values()) if (type.keyword.equals(text)) return type;
        throw FrontendFailure.of("JAVA_SUBSET_TYPE", "subset");
    }
    static ScalarType resolved(TypeMirror mirror) {
        TreeInventory.requireKnownType(mirror);
        for (ScalarType type : values()) if (type.kind == mirror.getKind()) return type;
        throw FrontendFailure.of("JAVA_SUBSET_TYPE", "subset");
    }
    static boolean conversion(ScalarType from, ScalarType to, String context) {
        return switch (context) {
            case "explicit_cast" -> from == to || from.integer() && to.integer();
            case "local_initializer", "local_assignment", "return" -> from == to || from == INT && to == LONG;
            case "call_argument", "binary_operand", "conditional_arm" -> from == to;
            default -> throw new IllegalArgumentException("unknown conversion context");
        };
    }
}
