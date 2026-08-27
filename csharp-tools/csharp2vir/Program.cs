using System;

namespace Mpk.CSharp2Vir;

internal static class Program
{
    private static int Main(string[] args)
    {
        if (args.Length == 1 && string.Equals(args[0], "--version", StringComparison.Ordinal))
        {
            if (!FrozenRoslynRuntime.HasExactIdentity())
            {
                Console.Error.Write("CSHARP_BUILD_ROSLYN_IDENTITY\n");
                return 70;
            }

            Console.Out.Write("csharp2vir 0.1.0 (Roslyn 5.6.0; .NET 10.0.11 profile)\n");
            return 0;
        }

        LowerRequest request;
        try
        {
            request = CliParser.Parse(args);
        }
        catch (CliFailure)
        {
            Console.Error.Write("CSHARP_FRONTEND_USAGE\n");
            return 2;
        }
        catch (FrontendFailure failure)
        {
            return WriteFailure(failure);
        }
        catch (Exception)
        {
            return WriteFailure(FrontendFailure.Internal("capture"));
        }

        string phase = "capture";
        try
        {
            Selection selection = SelectionCodec.Validate(request.RawSelection);
            CapturedSnapshot snapshot = SnapshotCapture.Capture(request.SourceRoot, selection);
            phase = "source";
            CapturedSourceSet sources = SourceTransport.Validate(snapshot);
            RoslynSourceSession sourceSession = RoslynSessionFactory.Parse(selection, sources);
            phase = "metadata";
            RoslynCompilationSession compilationSession = RoslynSessionFactory.Compile(
                selection,
                sourceSession,
                System.IO.Path.Combine(FrontendConstants.ToolchainRoot, "reference-pack"));
            phase = "typecheck";
            SubsetClosure closure = CSharpSubset.Validate(selection, compilationSession);
            phase = "subset";
            ContractSet contracts = CSharpContracts.Attach(selection, snapshot, closure);
            phase = "lowering";
            LoweredClosure lowered = CSharpLowering.Lower(selection, closure, contracts);
            phase = "emission";
            EmittedFrontendSuccess success = CSharpFrontendSuccessEmitter.Emit(
                request,
                selection,
                snapshot,
                sources,
                compilationSession,
                closure,
                contracts,
                lowered);
            System.IO.Stream output = Console.OpenStandardOutput();
            output.Write(success.EnvelopeBytes);
            output.Flush();
            return 0;
        }
        catch (SelectionSyntaxFailure)
        {
            Console.Error.Write("CSHARP_FRONTEND_USAGE\n");
            return 2;
        }
        catch (FrontendFailure failure)
        {
            return WriteFailure(failure);
        }
        catch (Exception)
        {
            return WriteFailure(FrontendFailure.Internal(phase));
        }

    }

    private static int WriteFailure(FrontendFailure failure)
    {
        // T11 owns successor protocol serialization. Until then, failures are
        // diagnostic-only and stdout cannot be mistaken for a partial artifact.
        Console.Error.Write(failure.Code + "\n");
        return failure.ExitCode;
    }
}
