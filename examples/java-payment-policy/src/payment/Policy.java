package payment;

public interface Policy {
    public static int approvedReserve(boolean approved, int requested, int fallback) {
        int result = requested;
        if (approved) {
            result = requested;
        } else {
            result = fallback;
        }
        return result;
    }
}
