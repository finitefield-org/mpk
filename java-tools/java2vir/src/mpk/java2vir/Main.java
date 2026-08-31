package mpk.java2vir;

/** Unregistered build candidate. Source parsing and VIR emission start later. */
public final class Main {
    private Main() {}

    public static void main(String[] args) {
        if (args.length == 1 && "--version".equals(args[0])) {
            if (!BuildIdentity.matches()) {
                System.err.print("JAVA_BUILD_IDENTITY\n");
                System.exit(70);
            }
            System.out.print("java2vir 0.1.0 (Temurin 25.0.4.1+1; inactive)\n");
            return;
        }
        // Never manufacture a frontend success envelope before T04-T06.
        System.err.print("JAVA_FRONTEND_UNAVAILABLE\n");
        System.exit(2);
    }
}
