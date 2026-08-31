package mpk.java2vir;

/** Private phase boundary consumed by the future lowering stage. */
final class JavaAdmission {
    private JavaAdmission() {}
    record Program(JavaSubset.Closure closure, JavaContracts.ContractSet contracts) {}

    static Program analyze(CapturedSnapshot snapshot) {
        String phase = "source";
        try {
            try (var compiler = CompilerSession.analyze(snapshot)) {
                phase = "subset";
                JavaSubset.Closure closure = JavaSubset.admit(compiler, snapshot.selection());
                JavaContracts.ContractSet contracts = JavaContracts.attach(snapshot, closure);
                return new Program(closure, contracts);
            }
        } catch (FrontendFailure failure) {
            throw failure;
        } catch (VirtualMachineError error) {
            throw FrontendFailure.of("JAVA_FRONTEND_RESOURCE", phase);
        } catch (RuntimeException | Error error) {
            throw FrontendFailure.of("JAVA_FRONTEND_INTERNAL", phase);
        }
    }
}
