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
            // A count refusal can occur before a complete request identity
            // exists. The staged runner validates that identity before launch.
            Console.Error.Write(failure.Code + "\n");
            return failure.ExitCode;
        }
        catch (Exception)
        {
            return 1;
        }

        string phase = "capture";
        try
        {
            FrontendLimits.ValidateArguments(args);
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
            return WriteFailure(request, failure);
        }
        catch (Exception)
        {
            return WriteFailure(request, FrontendFailure.Internal(phase));
        }

    }

    private static int WriteFailure(LowerRequest request, FrontendFailure failure)
    {
        try
        {
            byte[] transport = CSharpFrontendFailureEmitter.Emit(request, failure, out int exitCode);
            System.IO.Stream output = Console.OpenStandardOutput();
            output.Write(transport);
            output.Flush();
            return exitCode;
        }
        catch (Exception)
        {
            // A failure while constructing the bounded response stays
            // artifact-free and cannot be confused with a truncated envelope.
            return 1;
        }
    }
}
