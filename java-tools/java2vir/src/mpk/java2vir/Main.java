package mpk.java2vir;

/** Fixed candidate launcher. The public MPK CLI still has no active Java route. */
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
        FrontendArguments.Request request;
        try { request = FrontendArguments.parse(args); }
        catch (IllegalArgumentException | FrontendFailure failure) {
            // A malformed invocation has no validated selection to echo.
            System.err.print("JAVA_FRONTEND_UNAVAILABLE\n");
            System.exit(2);
            return;
        }
        String phase = "metadata";
        try {
            RuntimePreflight.validate(request);
            phase = "capture";
            var snapshot = CapturedSnapshot.capture(java.nio.file.Path.of("/mpk/source"), request.selection());
            var result = JavaFrontend.process(snapshot, request.identity());
            phase = "emission";
            System.out.writeBytes(result.stdout());
            System.exit(result.exitCode());
        } catch (FrontendFailure failure) {
            System.out.writeBytes(Protocol.failure(request.selection(), failure));
            System.exit(failure.exitCode());
        } catch (VirtualMachineError error) {
            System.out.writeBytes(Protocol.failure(request.selection(), FrontendFailure.of("JAVA_FRONTEND_RESOURCE", phase)));
            System.exit(1);
        } catch (RuntimeException | Error error) {
            System.out.writeBytes(Protocol.failure(request.selection(), FrontendFailure.of("JAVA_FRONTEND_INTERNAL", phase)));
            System.exit(1);
        }
    }
}
