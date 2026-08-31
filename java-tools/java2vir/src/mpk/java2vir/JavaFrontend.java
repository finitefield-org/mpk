package mpk.java2vir;

/** Private phase pipeline, intentionally not wired into the version-only Main. */
final class JavaFrontend {
    private JavaFrontend() {}
    record Result(int exitCode, byte[] stdout) {
        Result { stdout = stdout.clone(); }
        @Override public byte[] stdout() { return stdout.clone(); }
    }

    static Result process(CapturedSnapshot snapshot, JavaEmission.Identity identity) {
        String phase = "subset";
        try {
            JavaAdmission.Program admitted = JavaAdmission.analyze(snapshot);
            phase = "lowering";
            JavaIr.Program program = JavaLowering.lower(admitted);
            phase = "emission";
            return new Result(0, JavaEmission.emit(snapshot, program, identity));
        } catch (FrontendFailure failure) {
            return failure(snapshot.selection(), failure);
        } catch (VirtualMachineError error) {
            return failure(snapshot.selection(), FrontendFailure.of("JAVA_FRONTEND_RESOURCE", phase));
        } catch (RuntimeException | Error error) {
            return failure(snapshot.selection(), FrontendFailure.of("JAVA_FRONTEND_INTERNAL", phase));
        }
    }
    private static Result failure(Selection selection, FrontendFailure failure) {
        return new Result(failure.exitCode(), Protocol.failure(selection, failure));
    }
}
