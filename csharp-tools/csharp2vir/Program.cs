using System;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;

namespace Mpk.CSharp2Vir;

internal static class Program
{
    private const string RoslynVersion = "5.6.0.0";

    private static int Main(string[] args)
    {
        if (args.Length == 1 && string.Equals(args[0], "--version", StringComparison.Ordinal))
        {
            if (!HasFrozenRoslynIdentity())
            {
                Console.Error.Write("CSHARP_BUILD_ROSLYN_IDENTITY\n");
                return 70;
            }

            Console.Out.Write("csharp2vir 0.1.0 (Roslyn 5.6.0; .NET 10.0.11 profile)\n");
            return 0;
        }

        Console.Error.Write("CSHARP_FRONTEND_UNAVAILABLE\n");
        return 64;
    }

    private static bool HasFrozenRoslynIdentity()
    {
        string? common = typeof(Compilation).Assembly.GetName().Version?.ToString();
        string? csharp = typeof(CSharpCompilation).Assembly.GetName().Version?.ToString();
        return string.Equals(common, RoslynVersion, StringComparison.Ordinal)
            && string.Equals(csharp, RoslynVersion, StringComparison.Ordinal);
    }
}
