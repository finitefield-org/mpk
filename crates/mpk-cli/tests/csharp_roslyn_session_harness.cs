using System;
using System.Collections.Generic;
using System.Collections.Immutable;
using System.IO;
using System.Linq;
using System.Security.Cryptography;
using System.Text;
using System.Threading;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.CSharp.Syntax;
using Microsoft.CodeAnalysis.FlowAnalysis;
using Microsoft.CodeAnalysis.Operations;
using Microsoft.CodeAnalysis.Text;

namespace Mpk.CSharp2Vir;

internal static class RoslynSessionHarness
{
    private const string HelpersPath = "src/Helpers.cs";
    private const string PolicyPath = "src/Policy.cs";
    private const string HelpersSource =
        "namespace Example.Payment;\ninternal static class Helpers\n{\n    internal static long Identity(long value)\n    {\n        return value;\n    }\n}\n";
    private const string PolicySource =
        "namespace Example.Payment;\npublic static class Policy\n{\n    public static bool Approved(long reserve, long debit)\n    {\n        return reserve >= debit;\n    }\n}\n";

    public static int Main(string[] args)
    {
        if (args.Length != 1)
        {
            Console.Error.Write("CSHARP_ROSLYN_TEST_USAGE\n");
            return 1;
        }

        try
        {
            string referencePackRoot = args[0];
            OnlyFrozenRoslynAssembliesLoad();
            ExactSourceAndTreeSession();
            ExactCompilationAndReferences(referencePackRoot);
            DiagnosticPhaseOrdering(referencePackRoot);
            OptionAndReferenceDriftFailClosed(referencePackRoot);
            PublicSemanticApisAreExact(referencePackRoot);
            OnlyFrozenRoslynAssembliesLoad();
            return 0;
        }
        catch (HarnessFailure failure)
        {
            Console.Error.Write("CSHARP_ROSLYN_TEST_" + failure.Code + "\n");
            return 1;
        }
        catch (Exception)
        {
            Console.Error.Write("CSHARP_ROSLYN_TEST_UNEXPECTED\n");
            return 1;
        }
    }

    private static void OnlyFrozenRoslynAssembliesLoad()
    {
        FrozenRoslynRuntime.ValidateLoadedAssemblies();
        string[] roslynAssemblies = AppDomain.CurrentDomain.GetAssemblies()
            .Select(assembly => assembly.GetName().Name)
            .Where(name => name is not null && name.StartsWith("Microsoft.CodeAnalysis", StringComparison.Ordinal))
            .Select(name => name!)
            .OrderBy(name => name, StringComparer.Ordinal)
            .ToArray();
        Equal(
            "Microsoft.CodeAnalysis,Microsoft.CodeAnalysis.CSharp",
            string.Join(',', roslynAssemblies),
            "ROSLYN_ASSEMBLIES");
        Check(FrozenRoslynRuntime.HasExactIdentity(), "ROSLYN_IDENTITY");
    }

    private static void ExactSourceAndTreeSession()
    {
        Selection selection = BaselineSelection();
        RoslynSourceSession session = RoslynSessionFactory.Parse(selection, BaselineSources());
        RoslynSessionFactory.ValidateParseOptions(session.ParseOptions);
        Equal(LanguageVersion.CSharp14, session.ParseOptions.LanguageVersion, "PARSE_LANGUAGE");
        Equal(DocumentationMode.None, session.ParseOptions.DocumentationMode, "PARSE_DOCUMENTATION");
        Equal(SourceCodeKind.Regular, session.ParseOptions.Kind, "PARSE_KIND");
        Equal(0, session.ParseOptions.PreprocessorSymbolNames.Count(), "PARSE_SYMBOLS");
        Equal(0, session.ParseOptions.Features.Count, "PARSE_FEATURES");
        Equal(2, session.SourceTexts.Length, "SOURCE_TEXT_COUNT");
        Equal(2, session.SyntaxTrees.Length, "TREE_COUNT");
        Equal(0, session.Diagnostics.Length, "SYNTAX_DIAGNOSTICS");

        string[] expectedPaths = { HelpersPath, PolicyPath };
        string[] expectedSources = { HelpersSource, PolicySource };
        for (int index = 0; index < expectedPaths.Length; index++)
        {
            SourceText sourceText = session.SourceTexts[index];
            SyntaxTree syntaxTree = session.SyntaxTrees[index];
            Equal(expectedPaths[index], syntaxTree.FilePath, "TREE_PATH");
            Check(ReferenceEquals(syntaxTree.Options, session.ParseOptions), "TREE_OPTIONS");
            Check(syntaxTree.GetText(CancellationToken.None).ContentEquals(sourceText), "TREE_TEXT");
            Equal(SourceHashAlgorithm.Sha256, sourceText.ChecksumAlgorithm, "TEXT_CHECKSUM_ALGORITHM");
            Equal(expectedSources[index], sourceText.ToString(), "TEXT_CONTENT");
            Check(sourceText.Encoding is UTF8Encoding, "TEXT_ENCODING_TYPE");
            Equal(0, sourceText.Encoding!.GetPreamble().Length, "TEXT_ENCODING_PREAMBLE");
            byte[] expectedChecksum = SHA256.HashData(new UTF8Encoding(false, true).GetBytes(expectedSources[index]));
            Equal(
                Convert.ToHexString(expectedChecksum),
                Convert.ToHexString(sourceText.GetChecksum().AsSpan()),
                "TEXT_CHECKSUM");
        }
    }

    private static void ExactCompilationAndReferences(string referencePackRoot)
    {
        RoslynCompilationSession session = BaselineSession(referencePackRoot);
        RoslynSessionFactory.ValidateCompilationOptions(session.Options);
        Equal("payment-policy", session.Compilation.AssemblyName, "ASSEMBLY_NAME");
        Equal(OutputKind.DynamicallyLinkedLibrary, session.Options.OutputKind, "OPTION_OUTPUT");
        Equal(Platform.X64, session.Options.Platform, "OPTION_PLATFORM");
        Equal(OptimizationLevel.Release, session.Options.OptimizationLevel, "OPTION_OPTIMIZATION");
        Check(!session.Options.CheckOverflow, "OPTION_OVERFLOW");
        Equal(NullableContextOptions.Disable, session.Options.NullableContextOptions, "OPTION_NULLABLE");
        Check(!session.Options.AllowUnsafe, "OPTION_UNSAFE");
        Check(session.Options.Deterministic, "OPTION_DETERMINISTIC");
        Check(!session.Options.ConcurrentBuild, "OPTION_CONCURRENT");
        Equal(MetadataImportOptions.Public, session.Options.MetadataImportOptions, "OPTION_METADATA_IMPORT");
        Equal(ReportDiagnostic.Error, session.Options.GeneralDiagnosticOption, "OPTION_DIAGNOSTICS");
        Equal(4, session.Options.WarningLevel, "OPTION_WARNING_LEVEL");
        Check(!session.Options.ReportSuppressedDiagnostics, "OPTION_SUPPRESSED");
        Equal(0, session.Options.SpecificDiagnosticOptions.Count, "OPTION_SPECIFIC");
        Equal(0, session.Options.Usings.Length, "OPTION_USINGS");
        Check(ReferenceEquals(session.Options.AssemblyIdentityComparer, AssemblyIdentityComparer.Default), "OPTION_IDENTITY");
        Check(session.Options.SourceReferenceResolver is null, "OPTION_SOURCE_RESOLVER");
        Check(session.Options.MetadataReferenceResolver is null, "OPTION_METADATA_RESOLVER");
        Check(session.Options.XmlReferenceResolver is null, "OPTION_XML_RESOLVER");
        Check(session.Options.StrongNameProvider is null, "OPTION_STRONG_NAME_PROVIDER");
        Check(session.Options.SyntaxTreeOptionsProvider is null, "OPTION_TREE_PROVIDER");
        Check(session.Options.ModuleName is null, "OPTION_MODULE");
        Check(session.Options.MainTypeName is null, "OPTION_MAIN");
        Equal("Script", session.Options.ScriptClassName, "OPTION_SCRIPT");
        Check(session.Options.CryptoKeyContainer is null, "OPTION_KEY_CONTAINER");
        Check(session.Options.CryptoKeyFile is null, "OPTION_KEY_FILE");
        Check(session.Options.CryptoPublicKey.IsEmpty, "OPTION_PUBLIC_KEY");
        Check(session.Options.DelaySign is null, "OPTION_DELAY_SIGN");
        Check(!session.Options.PublicSign, "OPTION_PUBLIC_SIGN");
        Equal(0, session.Diagnostics.Length, "COMPILATION_DIAGNOSTICS");

        FrozenReferenceProjection projection = session.ReferenceProjection;
        Equal(FrozenReferenceProjection.ExpectedCount, projection.Count, "REFERENCE_COUNT");
        Equal(
            "ref/net10.0/Microsoft.CSharp.dll",
            projection.RecordAt(0).RelativePath,
            "REFERENCE_FIRST_PATH");
        Equal(
            "50ac73d1df3bd9c1ab431587efd4089c03b3cc373c5da382f97a565575966109",
            projection.RecordAt(0).Sha256,
            "REFERENCE_FIRST_HASH");
        Equal(
            "ref/net10.0/netstandard.dll",
            projection.RecordAt(projection.Count - 1).RelativePath,
            "REFERENCE_LAST_PATH");
        Equal(
            "ce09cef0d758196f2467f320d840c03202453efe9fb1920dddd24ea8277bbbb3",
            projection.RecordAt(projection.Count - 1).Sha256,
            "REFERENCE_LAST_HASH");

        for (int index = 0; index < projection.Count; index++)
        {
            PortableExecutableReference reference = projection.References[index];
            Equal(projection.RecordAt(index).FullPath, reference.FilePath, "REFERENCE_PATH");
            Equal(MetadataImageKind.Assembly, reference.Properties.Kind, "REFERENCE_KIND");
            Check(reference.Properties.Aliases.IsDefaultOrEmpty, "REFERENCE_ALIASES");
            Check(!reference.Properties.EmbedInteropTypes, "REFERENCE_INTEROP");
        }

        SyntaxTree[] compilationTrees = session.Compilation.SyntaxTrees.ToArray();
        Equal(2, compilationTrees.Length, "COMPILATION_TREE_COUNT");
        Check(ReferenceEquals(compilationTrees[0], session.Source.SyntaxTrees[0]), "COMPILATION_TREE_ZERO");
        Check(ReferenceEquals(compilationTrees[1], session.Source.SyntaxTrees[1]), "COMPILATION_TREE_ONE");
        Equal(projection.Count, session.Compilation.References.Count(), "COMPILATION_REFERENCE_COUNT");
        Check(session.Compilation.References.All(reference => reference is not CompilationReference), "COMPILATION_REFERENCE_KIND");
    }

    private static void DiagnosticPhaseOrdering(string referencePackRoot)
    {
        Selection selection = SingleSourceSelection();
        ExpectFailure(
            () => RoslynSessionFactory.Parse(
                selection,
                SingleSource("namespace Example;\npublic static class Broken { public static int M( { return 0; } }\n")),
            FrontendStatus.SourceError,
            "source",
            "CSHARP_SOURCE_PARSE");
        ExpectFailure(
            () => RoslynSessionFactory.Parse(
                selection,
                SingleSource("#warning frozen warning\nnamespace Example;\npublic static class Broken { public static int M() { return 0; } }\n")),
            FrontendStatus.SourceError,
            "source",
            "CSHARP_SOURCE_PARSE");

        RoslynSourceSession missingName = RoslynSessionFactory.Parse(
            selection,
            SingleSource("namespace Example;\npublic static class Broken { public static int M() { return Missing; } }\n"));
        ExpectFailure(
            () => RoslynSessionFactory.Compile(selection, missingName, referencePackRoot),
            FrontendStatus.SourceError,
            "metadata",
            "CSHARP_SOURCE_DIAGNOSTIC");

        RoslynSourceSession warning = RoslynSessionFactory.Parse(
            selection,
            SingleSource("namespace Example;\npublic static class Broken { public static int M() { int unused; return 0; } }\n"));
        ExpectFailure(
            () => RoslynSessionFactory.Compile(selection, warning, referencePackRoot),
            FrontendStatus.SourceError,
            "metadata",
            "CSHARP_SOURCE_DIAGNOSTIC");
    }

    private static void OptionAndReferenceDriftFailClosed(string referencePackRoot)
    {
        CSharpParseOptions parseOptions = RoslynSessionFactory.CreateParseOptions();
        ExpectFailure(
            () => RoslynSessionFactory.ValidateParseOptions(parseOptions.WithLanguageVersion(LanguageVersion.CSharp13)),
            FrontendStatus.FrontendError,
            "source",
            "CSHARP_TOOLCHAIN_OPTIONS");
        ExpectFailure(
            () => RoslynSessionFactory.ValidateParseOptions(parseOptions.WithDocumentationMode(DocumentationMode.Parse)),
            FrontendStatus.FrontendError,
            "source",
            "CSHARP_TOOLCHAIN_OPTIONS");
        ExpectFailure(
            () => RoslynSessionFactory.ValidateParseOptions(parseOptions.WithKind(SourceCodeKind.Script)),
            FrontendStatus.FrontendError,
            "source",
            "CSHARP_TOOLCHAIN_OPTIONS");
        ExpectFailure(
            () => RoslynSessionFactory.ValidateParseOptions(parseOptions.WithPreprocessorSymbols("DRIFT")),
            FrontendStatus.FrontendError,
            "source",
            "CSHARP_TOOLCHAIN_OPTIONS");
        ExpectFailure(
            () => RoslynSessionFactory.ValidateParseOptions(parseOptions.WithFeatures(new[]
            {
                new KeyValuePair<string, string>("drift", "true"),
            })),
            FrontendStatus.FrontendError,
            "source",
            "CSHARP_TOOLCHAIN_OPTIONS");

        CSharpCompilationOptions options = RoslynSessionFactory.CreateCompilationOptions();
        foreach (CSharpCompilationOptions mutation in new[]
        {
            options.WithOutputKind(OutputKind.ConsoleApplication),
            options.WithPlatform(Platform.AnyCpu),
            options.WithOptimizationLevel(OptimizationLevel.Debug),
            options.WithOverflowChecks(true),
            options.WithNullableContextOptions(NullableContextOptions.Enable),
            options.WithAllowUnsafe(true),
            options.WithDeterministic(false),
            options.WithConcurrentBuild(true),
            options.WithMetadataImportOptions(MetadataImportOptions.All),
            options.WithGeneralDiagnosticOption(ReportDiagnostic.Default),
            options.WithWarningLevel(3),
            options.WithReportSuppressedDiagnostics(true),
            options.WithSpecificDiagnosticOptions(
                ImmutableDictionary<string, ReportDiagnostic>.Empty.Add("CS0168", ReportDiagnostic.Suppress)),
            options.WithUsings("System"),
            options.WithModuleName("drift"),
            options.WithMainTypeName("Example.Program"),
            options.WithScriptClassName("Drift"),
            options.WithCryptoKeyContainer("drift"),
            options.WithCryptoKeyFile("drift.snk"),
            options.WithCryptoPublicKey(ImmutableArray.Create((byte)1)),
            options.WithDelaySign(true),
            options.WithPublicSign(true),
        })
        {
            ExpectFailure(
                () => RoslynSessionFactory.ValidateCompilationOptions(mutation),
                FrontendStatus.FrontendError,
                "metadata",
                "CSHARP_TOOLCHAIN_OPTIONS");
        }

        ExpectFailure(
            () => FrozenReferenceProjection.Load(System.IO.Path.Combine(referencePackRoot, "ref")),
            FrontendStatus.FrontendError,
            "release",
            "CSHARP_TOOLCHAIN_REFERENCE");
        WithMutatedReferenceProjection(referencePackRoot, mutatedRoot =>
        {
            ExpectFailure(
                () => FrozenReferenceProjection.Load(mutatedRoot),
                FrontendStatus.FrontendError,
                "release",
                "CSHARP_TOOLCHAIN_REFERENCE");
        });
    }

    private static void PublicSemanticApisAreExact(string referencePackRoot)
    {
        RoslynCompilationSession session = BaselineSession(referencePackRoot);
        SyntaxTree policyTree = session.Source.SyntaxTrees[1];
        SemanticModel semanticModel = RoslynPublicApi.GetSemanticModel(session, policyTree);
        CompilationUnitSyntax root = (CompilationUnitSyntax)policyTree.GetRoot(CancellationToken.None);
        MethodDeclarationSyntax method = root.DescendantNodes().OfType<MethodDeclarationSyntax>().Single();
        IMethodSymbol methodSymbol = RoslynPublicApi.GetDeclaredSymbol(semanticModel, method);
        Equal("Approved", methodSymbol.Name, "DECLARED_SYMBOL_NAME");
        Equal(SymbolKind.Method, methodSymbol.Kind, "DECLARED_SYMBOL_KIND");

        IdentifierNameSyntax reserve = method.DescendantNodes()
            .OfType<IdentifierNameSyntax>()
            .First(identifier => identifier.Identifier.ValueText == "reserve");
        SymbolInfo symbolInfo = RoslynPublicApi.GetSymbolInfo(semanticModel, reserve);
        Check(symbolInfo.Symbol is IParameterSymbol, "SYMBOL_INFO");
        TypeInfo typeInfo = RoslynPublicApi.GetTypeInfo(semanticModel, reserve);
        Equal(SpecialType.System_Int64, typeInfo.Type!.SpecialType, "TYPE_INFO");
        Conversion conversion = RoslynPublicApi.ClassifyConversion(
            semanticModel,
            reserve,
            session.Compilation.GetSpecialType(SpecialType.System_Int64),
            isExplicitInSource: false);
        Check(conversion.Exists && conversion.IsIdentity, "CONVERSION");
        Conversion explicitConversion = RoslynPublicApi.ClassifyConversion(
            semanticModel,
            reserve,
            session.Compilation.GetSpecialType(SpecialType.System_UInt32),
            isExplicitInSource: true);
        Check(explicitConversion.Exists && explicitConversion.IsExplicit, "EXPLICIT_CONVERSION");

        BinaryExpressionSyntax comparison = method.DescendantNodes()
            .OfType<BinaryExpressionSyntax>()
            .Single();
        IOperation? operation = RoslynPublicApi.GetOperation(semanticModel, comparison);
        Check(operation is IBinaryOperation, "BINARY_OPERATION");
        IMethodBodyOperation bodyOperation = RoslynPublicApi.GetMethodBodyOperation(semanticModel, method);
        ControlFlowGraph graph = RoslynPublicApi.CreateControlFlowGraph(bodyOperation);
        Check(graph.Blocks.Length >= 3, "CFG_BLOCKS");
        Check(ReferenceEquals(graph.OriginalOperation, bodyOperation), "CFG_ROOT");

        SyntaxTree foreignTree = CSharpSyntaxTree.ParseText("namespace Foreign;\n", cancellationToken: CancellationToken.None);
        ExpectFailure(
            () => RoslynPublicApi.GetSemanticModel(session, foreignTree),
            FrontendStatus.FrontendError,
            "lowering",
            "CSHARP_TOOLCHAIN_ADAPTER");
    }

    private static RoslynCompilationSession BaselineSession(string referencePackRoot)
    {
        Selection selection = BaselineSelection();
        RoslynSourceSession source = RoslynSessionFactory.Parse(selection, BaselineSources());
        return RoslynSessionFactory.Compile(selection, source, referencePackRoot);
    }

    private static Selection BaselineSelection()
    {
        return SelectionCodec.Validate(new RawSelection(
            "payment-policy",
            new[] { HelpersPath, PolicyPath },
            new[] { "contracts/approved.json" },
            new[] { "Example.Payment.Policy::Approved(i64,i64)->bool" }));
    }

    private static CapturedSourceSet BaselineSources()
    {
        return new CapturedSourceSet(new[]
        {
            new CapturedSourceText(HelpersPath, HelpersSource),
            new CapturedSourceText(PolicyPath, PolicySource),
        });
    }

    private static Selection SingleSourceSelection()
    {
        return SelectionCodec.Validate(new RawSelection(
            "diagnostic-case",
            new[] { PolicyPath },
            new[] { "contracts/diagnostic.json" },
            new[] { "Example.Broken::M()->i32" }));
    }

    private static CapturedSourceSet SingleSource(string source)
    {
        return new CapturedSourceSet(new[] { new CapturedSourceText(PolicyPath, source) });
    }

    private static void WithMutatedReferenceProjection(string referencePackRoot, Action<string> action)
    {
        string temporaryRoot = Path.Combine(
            Path.GetTempPath(),
            "mpk-csharp-reference-mutation-" + Guid.NewGuid().ToString("N"));
        string destination = Path.Combine(temporaryRoot, "ref", "net10.0");
        try
        {
            Directory.CreateDirectory(destination);
            string source = Path.Combine(referencePackRoot, "ref", "net10.0");
            foreach (string path in Directory.EnumerateFiles(source, "*.dll", SearchOption.TopDirectoryOnly))
            {
                File.Copy(path, Path.Combine(destination, Path.GetFileName(path)));
            }

            string mutation = Path.Combine(destination, "Microsoft.CSharp.dll");
            byte[] bytes = File.ReadAllBytes(mutation);
            bytes[0] ^= 0xff;
            File.WriteAllBytes(mutation, bytes);
            action(temporaryRoot);
        }
        finally
        {
            if (Directory.Exists(temporaryRoot))
            {
                Directory.Delete(temporaryRoot, recursive: true);
            }
        }
    }

    private static void ExpectFailure(
        Action action,
        FrontendStatus status,
        string phase,
        string code)
    {
        try
        {
            action();
        }
        catch (FrontendFailure failure)
        {
            Equal(status, failure.Status, code + "_STATUS");
            Equal(phase, failure.Phase, code + "_PHASE");
            Equal(code, failure.Code, code + "_CODE");
            return;
        }

        throw new HarnessFailure(code + "_ACCEPTED");
    }

    private static void Check(bool condition, string code)
    {
        if (!condition)
        {
            throw new HarnessFailure(code);
        }
    }

    private static void Equal<T>(T expected, T actual, string code)
    {
        if (!EqualityComparer<T>.Default.Equals(expected, actual))
        {
            throw new HarnessFailure(code);
        }
    }
}

internal sealed class HarnessFailure : Exception
{
    internal HarnessFailure(string code)
        : base(code)
    {
        Code = code;
    }

    internal string Code { get; }
}
