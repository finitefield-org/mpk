using System;
using System.Collections.Generic;
using System.Collections.Immutable;
using System.IO;
using System.Reflection;
using System.Security.Cryptography;
using System.Text;
using System.Text.Encodings.Web;
using System.Text.Json;
using System.Threading;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.Text;

namespace Mpk.CSharp2Vir;

internal static class FrozenRoslynRuntime
{
    private const string RoslynVersion = "5.6.0.0";

    internal static bool HasExactIdentity()
    {
        Assembly common = typeof(Compilation).Assembly;
        Assembly csharp = typeof(CSharpCompilation).Assembly;
        return HasIdentity(common, "Microsoft.CodeAnalysis")
            && HasIdentity(csharp, "Microsoft.CodeAnalysis.CSharp");
    }

    internal static void ValidateLoadedAssemblies()
    {
        _ = typeof(Compilation).Assembly;
        _ = typeof(CSharpCompilation).Assembly;

        int count = 0;
        bool common = false;
        bool csharp = false;
        foreach (Assembly assembly in AppDomain.CurrentDomain.GetAssemblies())
        {
            string? name = assembly.GetName().Name;
            if (name is null || !name.StartsWith("Microsoft.CodeAnalysis", StringComparison.Ordinal))
            {
                continue;
            }

            count++;
            if (string.Equals(name, "Microsoft.CodeAnalysis", StringComparison.Ordinal)
                && HasIdentity(assembly, name))
            {
                common = true;
            }
            else if (string.Equals(name, "Microsoft.CodeAnalysis.CSharp", StringComparison.Ordinal)
                && HasIdentity(assembly, name))
            {
                csharp = true;
            }
            else
            {
                throw FrontendFailure.Toolchain("release", "CSHARP_TOOLCHAIN_ROSLYN");
            }
        }

        if (count != 2 || !common || !csharp)
        {
            throw FrontendFailure.Toolchain("release", "CSHARP_TOOLCHAIN_ROSLYN");
        }
    }

    private static bool HasIdentity(Assembly assembly, string expectedName)
    {
        AssemblyName identity = assembly.GetName();
        return string.Equals(identity.Name, expectedName, StringComparison.Ordinal)
            && string.Equals(identity.Version?.ToString(), RoslynVersion, StringComparison.Ordinal);
    }
}

internal sealed class FrozenReferenceRecord
{
    internal FrozenReferenceRecord(string relativePath, string fullPath, long sizeBytes, string sha256)
    {
        RelativePath = relativePath;
        FullPath = fullPath;
        SizeBytes = sizeBytes;
        Sha256 = sha256;
    }

    internal string RelativePath { get; }

    internal string FullPath { get; }

    internal long SizeBytes { get; }

    internal string Sha256 { get; }
}

internal sealed class FrozenReferenceProjection
{
    internal const int ExpectedCount = 167;
    internal const long ExpectedTotalBytes = 6_046_008;
    internal const int ExpectedCanonicalBytes = 24_670;
    internal const string ExpectedInventorySha256 = "30623f64b7d85564260e62464e652bfaa89eb56e0e55193989bfb99538ba6cad";
    private static readonly byte[] InventoryDomain = Encoding.ASCII.GetBytes(
        "MPK-CSHARP-REFERENCE-INVENTORY-0.1\0");
    private readonly ImmutableArray<FrozenReferenceRecord> records;
    private readonly ImmutableArray<PortableExecutableReference> references;

    private FrozenReferenceProjection(
        ImmutableArray<FrozenReferenceRecord> records,
        ImmutableArray<PortableExecutableReference> references)
    {
        this.records = records;
        this.references = references;
    }

    internal int Count => records.Length;

    internal ImmutableArray<PortableExecutableReference> References => references;

    internal FrozenReferenceRecord RecordAt(int index) => records[index];

    internal static FrozenReferenceProjection Load(string referencePackRoot)
    {
        try
        {
            return LoadChecked(referencePackRoot);
        }
        catch (FrontendFailure)
        {
            throw;
        }
        catch (Exception error) when (
            error is ArgumentException
            || error is BadImageFormatException
            || error is IOException
            || error is NotSupportedException
            || error is OverflowException
            || error is UnauthorizedAccessException)
        {
            throw FrontendFailure.Toolchain("release", "CSHARP_TOOLCHAIN_REFERENCE");
        }
    }

    private static FrozenReferenceProjection LoadChecked(string referencePackRoot)
    {
        if (string.IsNullOrEmpty(referencePackRoot) || !Path.IsPathFullyQualified(referencePackRoot))
        {
            throw FrontendFailure.Toolchain("release", "CSHARP_TOOLCHAIN_REFERENCE");
        }

        var root = new DirectoryInfo(Path.GetFullPath(referencePackRoot));
        var referenceParent = new DirectoryInfo(Path.Combine(root.FullName, "ref"));
        var referenceDirectory = new DirectoryInfo(Path.Combine(referenceParent.FullName, "net10.0"));
        ValidateDirectory(root);
        ValidateDirectory(referenceParent);
        ValidateDirectory(referenceDirectory);

        var mutableRecords = new List<FrozenReferenceRecord>(ExpectedCount);
        foreach (FileSystemInfo entry in referenceDirectory.EnumerateFileSystemInfos("*.dll"))
        {
            if (entry is not FileInfo file
                || file.LinkTarget is not null
                || (file.Attributes & (FileAttributes.Directory | FileAttributes.ReparsePoint)) != 0
                || !file.Name.EndsWith(".dll", StringComparison.Ordinal)
                || file.Name.IndexOfAny(new[] { '/', '\\' }) >= 0)
            {
                throw FrontendFailure.Toolchain("release", "CSHARP_TOOLCHAIN_REFERENCE");
            }

            string relativePath = "ref/net10.0/" + file.Name;
            long sizeBytes = file.Length;
            string sha256;
            using (var stream = new FileStream(
                file.FullName,
                FileMode.Open,
                FileAccess.Read,
                FileShare.Read,
                64 * 1024,
                FileOptions.SequentialScan))
            {
                sha256 = Convert.ToHexString(SHA256.HashData(stream)).ToLowerInvariant();
                if (stream.Length != sizeBytes)
                {
                    throw FrontendFailure.Toolchain("release", "CSHARP_TOOLCHAIN_REFERENCE");
                }
            }

            mutableRecords.Add(new FrozenReferenceRecord(
                relativePath,
                Path.GetFullPath(file.FullName),
                sizeBytes,
                sha256));
        }

        mutableRecords.Sort(static (left, right) =>
            string.CompareOrdinal(left.RelativePath, right.RelativePath));
        ValidateInventory(mutableRecords);

        var references = ImmutableArray.CreateBuilder<PortableExecutableReference>(ExpectedCount);
        foreach (FrozenReferenceRecord record in mutableRecords)
        {
            PortableExecutableReference reference = MetadataReference.CreateFromFile(
                record.FullPath,
                MetadataReferenceProperties.Assembly,
                documentation: null);
            ValidateReference(reference, record.FullPath);
            references.Add(reference);
        }

        return new FrozenReferenceProjection(
            ImmutableArray.CreateRange(mutableRecords),
            references.MoveToImmutable());
    }

    private static void ValidateDirectory(DirectoryInfo directory)
    {
        if (!directory.Exists
            || directory.LinkTarget is not null
            || (directory.Attributes & FileAttributes.ReparsePoint) != 0)
        {
            throw FrontendFailure.Toolchain("release", "CSHARP_TOOLCHAIN_REFERENCE");
        }
    }

    private static void ValidateInventory(IReadOnlyList<FrozenReferenceRecord> records)
    {
        if (records.Count != ExpectedCount)
        {
            throw FrontendFailure.Toolchain("release", "CSHARP_TOOLCHAIN_REFERENCE");
        }

        long totalBytes = 0;
        string? previous = null;
        foreach (FrozenReferenceRecord record in records)
        {
            if (!record.RelativePath.StartsWith("ref/net10.0/", StringComparison.Ordinal)
                || record.RelativePath.AsSpan("ref/net10.0/".Length).Contains('/')
                || (previous is not null && string.CompareOrdinal(previous, record.RelativePath) >= 0))
            {
                throw FrontendFailure.Toolchain("release", "CSHARP_TOOLCHAIN_REFERENCE");
            }

            totalBytes = checked(totalBytes + record.SizeBytes);
            previous = record.RelativePath;
        }

        byte[] canonical = CanonicalInventory(records);
        var preimage = new byte[checked(InventoryDomain.Length + canonical.Length)];
        Buffer.BlockCopy(InventoryDomain, 0, preimage, 0, InventoryDomain.Length);
        Buffer.BlockCopy(canonical, 0, preimage, InventoryDomain.Length, canonical.Length);
        string inventorySha256 = Convert.ToHexString(SHA256.HashData(preimage)).ToLowerInvariant();
        if (totalBytes != ExpectedTotalBytes
            || canonical.Length != ExpectedCanonicalBytes
            || !string.Equals(inventorySha256, ExpectedInventorySha256, StringComparison.Ordinal))
        {
            throw FrontendFailure.Toolchain("release", "CSHARP_TOOLCHAIN_REFERENCE");
        }
    }

    private static byte[] CanonicalInventory(IReadOnlyList<FrozenReferenceRecord> records)
    {
        using var output = new MemoryStream(ExpectedCanonicalBytes);
        using (var writer = new Utf8JsonWriter(
            output,
            new JsonWriterOptions
            {
                Encoder = JavaScriptEncoder.UnsafeRelaxedJsonEscaping,
                Indented = false,
                SkipValidation = false,
            }))
        {
            writer.WriteStartArray();
            foreach (FrozenReferenceRecord record in records)
            {
                writer.WriteStartObject();
                writer.WriteString("path", record.RelativePath);
                writer.WriteString("sha256", record.Sha256);
                writer.WriteNumber("size_bytes", record.SizeBytes);
                writer.WriteEndObject();
            }

            writer.WriteEndArray();
        }

        return output.ToArray();
    }

    private static void ValidateReference(PortableExecutableReference reference, string expectedPath)
    {
        MetadataReferenceProperties properties = reference.Properties;
        if (!string.Equals(reference.FilePath, expectedPath, StringComparison.Ordinal)
            || !properties.Equals(MetadataReferenceProperties.Assembly))
        {
            throw FrontendFailure.Toolchain("release", "CSHARP_TOOLCHAIN_REFERENCE");
        }

        // The public factory overload above receives a null documentation
        // provider. Roslyn 5.6 does not expose that stored value publicly.
    }
}

internal sealed class RoslynSourceSession
{
    internal RoslynSourceSession(
        CSharpParseOptions parseOptions,
        ImmutableArray<SourceText> sourceTexts,
        ImmutableArray<SyntaxTree> syntaxTrees,
        ImmutableArray<Diagnostic> diagnostics)
    {
        ParseOptions = parseOptions;
        SourceTexts = sourceTexts;
        SyntaxTrees = syntaxTrees;
        Diagnostics = diagnostics;
    }

    internal CSharpParseOptions ParseOptions { get; }

    internal ImmutableArray<SourceText> SourceTexts { get; }

    internal ImmutableArray<SyntaxTree> SyntaxTrees { get; }

    internal ImmutableArray<Diagnostic> Diagnostics { get; }
}

internal sealed class RoslynCompilationSession
{
    internal RoslynCompilationSession(
        RoslynSourceSession source,
        FrozenReferenceProjection referenceProjection,
        CSharpCompilationOptions options,
        CSharpCompilation compilation,
        ImmutableArray<Diagnostic> diagnostics)
    {
        Source = source;
        ReferenceProjection = referenceProjection;
        Options = options;
        Compilation = compilation;
        Diagnostics = diagnostics;
    }

    internal RoslynSourceSession Source { get; }

    internal FrozenReferenceProjection ReferenceProjection { get; }

    internal CSharpCompilationOptions Options { get; }

    internal CSharpCompilation Compilation { get; }

    internal ImmutableArray<Diagnostic> Diagnostics { get; }
}

internal static class RoslynSessionFactory
{
    private static readonly UTF8Encoding StrictUtf8 = new UTF8Encoding(false, true);

    internal static RoslynSourceSession Parse(Selection selection, CapturedSourceSet sources)
    {
        FrozenRoslynRuntime.ValidateLoadedAssemblies();
        CSharpParseOptions parseOptions = CreateParseOptions();
        ValidateParseOptions(parseOptions);

        if (sources.Count != selection.Raw.Sources.Count)
        {
            throw FrontendFailure.Internal("source");
        }

        var sourceTexts = ImmutableArray.CreateBuilder<SourceText>(sources.Count);
        var syntaxTrees = ImmutableArray.CreateBuilder<SyntaxTree>(sources.Count);
        var diagnostics = ImmutableArray.CreateBuilder<Diagnostic>();
        for (int index = 0; index < sources.Count; index++)
        {
            CapturedSourceText source = sources.SourceAt(index);
            if (!string.Equals(
                source.NormalizedPath,
                selection.Raw.Sources[index],
                StringComparison.Ordinal))
            {
                throw FrontendFailure.Internal("source");
            }

            SourceText sourceText = SourceText.From(
                source.Text,
                StrictUtf8,
                SourceHashAlgorithm.Sha256);
            ValidateSourceText(sourceText, source.Text);
            SyntaxTree syntaxTree = CSharpSyntaxTree.ParseText(
                sourceText,
                parseOptions,
                source.NormalizedPath,
                CancellationToken.None);
            ValidateSyntaxTree(syntaxTree, sourceText, parseOptions, source.NormalizedPath);
            sourceTexts.Add(sourceText);
            syntaxTrees.Add(syntaxTree);
            diagnostics.AddRange(syntaxTree.GetDiagnostics(CancellationToken.None));
        }

        ImmutableArray<Diagnostic> collected = diagnostics.ToImmutable();
        if (HasActiveDiagnostic(collected))
        {
            throw FrontendFailure.SourceError("source", "CSHARP_SOURCE_PARSE");
        }

        return new RoslynSourceSession(
            parseOptions,
            sourceTexts.MoveToImmutable(),
            syntaxTrees.MoveToImmutable(),
            collected);
    }

    internal static RoslynCompilationSession Compile(
        Selection selection,
        RoslynSourceSession source,
        string referencePackRoot)
    {
        CSharpCompilationOptions options = CreateCompilationOptions();
        ValidateCompilationOptions(options);
        FrozenReferenceProjection referenceProjection = FrozenReferenceProjection.Load(referencePackRoot);

        CSharpCompilation compilation;
        try
        {
            compilation = CSharpCompilation.Create(
                selection.Raw.Compilation,
                source.SyntaxTrees,
                referenceProjection.References,
                options);
            ValidateCompilation(compilation, selection, source, referenceProjection, options);
        }
        catch (FrontendFailure)
        {
            throw;
        }
        catch (Exception error) when (
            error is ArgumentException
            || error is InvalidOperationException
            || error is NotSupportedException)
        {
            throw FrontendFailure.Toolchain("metadata", "CSHARP_TOOLCHAIN_ADAPTER");
        }

        ImmutableArray<Diagnostic> diagnostics;
        try
        {
            diagnostics = compilation.GetDiagnostics(CancellationToken.None);
        }
        catch (Exception error) when (
            error is ArgumentException
            || error is InvalidOperationException
            || error is NotSupportedException)
        {
            throw FrontendFailure.Toolchain("metadata", "CSHARP_TOOLCHAIN_ADAPTER");
        }

        if (HasActiveDiagnostic(diagnostics))
        {
            throw FrontendFailure.SourceError("metadata", "CSHARP_SOURCE_DIAGNOSTIC");
        }

        return new RoslynCompilationSession(
            source,
            referenceProjection,
            options,
            compilation,
            diagnostics);
    }

    internal static CSharpParseOptions CreateParseOptions()
    {
        return new CSharpParseOptions(
            languageVersion: LanguageVersion.CSharp14,
            documentationMode: DocumentationMode.None,
            kind: SourceCodeKind.Regular,
            preprocessorSymbols: Array.Empty<string>());
    }

    internal static void ValidateParseOptions(CSharpParseOptions options)
    {
        if (options.LanguageVersion != LanguageVersion.CSharp14
            || options.SpecifiedLanguageVersion != LanguageVersion.CSharp14
            || options.Kind != SourceCodeKind.Regular
            || options.SpecifiedKind != SourceCodeKind.Regular
            || options.DocumentationMode != DocumentationMode.None
            || HasAny(options.PreprocessorSymbolNames)
            || options.Features.Count != 0
            || !options.Errors.IsDefaultOrEmpty)
        {
            throw FrontendFailure.Toolchain("source", "CSHARP_TOOLCHAIN_OPTIONS");
        }
    }

    internal static CSharpCompilationOptions CreateCompilationOptions()
    {
        return new CSharpCompilationOptions(
            outputKind: OutputKind.DynamicallyLinkedLibrary,
            reportSuppressedDiagnostics: false,
            moduleName: null,
            mainTypeName: null,
            scriptClassName: "Script",
            usings: Array.Empty<string>(),
            optimizationLevel: OptimizationLevel.Release,
            checkOverflow: false,
            allowUnsafe: false,
            cryptoKeyContainer: null,
            cryptoKeyFile: null,
            cryptoPublicKey: ImmutableArray<byte>.Empty,
            delaySign: null,
            platform: Platform.X64,
            generalDiagnosticOption: ReportDiagnostic.Error,
            warningLevel: 4,
            specificDiagnosticOptions: Array.Empty<KeyValuePair<string, ReportDiagnostic>>(),
            concurrentBuild: false,
            deterministic: true,
            xmlReferenceResolver: null,
            sourceReferenceResolver: null,
            metadataReferenceResolver: null,
            assemblyIdentityComparer: AssemblyIdentityComparer.Default,
            strongNameProvider: null,
            publicSign: false,
            metadataImportOptions: MetadataImportOptions.Public,
            nullableContextOptions: NullableContextOptions.Disable);
    }

    internal static void ValidateCompilationOptions(CSharpCompilationOptions options)
    {
        if (options.OutputKind != OutputKind.DynamicallyLinkedLibrary
            || options.Platform != Platform.X64
            || options.OptimizationLevel != OptimizationLevel.Release
            || options.CheckOverflow
            || options.NullableContextOptions != NullableContextOptions.Disable
            || options.AllowUnsafe
            || !options.Deterministic
            || options.ConcurrentBuild
            || options.MetadataImportOptions != MetadataImportOptions.Public
            || options.GeneralDiagnosticOption != ReportDiagnostic.Error
            || options.WarningLevel != 4
            || options.ReportSuppressedDiagnostics
            || !options.SpecificDiagnosticOptions.IsEmpty
            || !options.Usings.IsEmpty
            || !ReferenceEquals(options.AssemblyIdentityComparer, AssemblyIdentityComparer.Default)
            || options.SourceReferenceResolver is not null
            || options.MetadataReferenceResolver is not null
            || options.XmlReferenceResolver is not null
            || options.StrongNameProvider is not null
            || options.SyntaxTreeOptionsProvider is not null
            || options.ModuleName is not null
            || options.MainTypeName is not null
            || !string.Equals(options.ScriptClassName, "Script", StringComparison.Ordinal)
            || options.CryptoKeyContainer is not null
            || options.CryptoKeyFile is not null
            || !options.CryptoPublicKey.IsEmpty
            || options.DelaySign is not null
            || options.PublicSign
            || !options.Errors.IsDefaultOrEmpty)
        {
            throw FrontendFailure.Toolchain("metadata", "CSHARP_TOOLCHAIN_OPTIONS");
        }

        // ReferencesSupersedeLowerVersions is an internal getter in Roslyn
        // 5.6. The exact public constructor above fixes its C# value to false;
        // the pinned assembly bytes and constructor behavior bind that value.
    }

    private static void ValidateSourceText(SourceText sourceText, string expectedText)
    {
        if (!ReferenceEquals(sourceText.Encoding, StrictUtf8)
            || sourceText.Encoding is not UTF8Encoding encoding
            || encoding.GetPreamble().Length != 0
            || encoding.CodePage != Encoding.UTF8.CodePage
            || sourceText.ChecksumAlgorithm != SourceHashAlgorithm.Sha256
            || sourceText.Length != expectedText.Length
            || !string.Equals(sourceText.ToString(), expectedText, StringComparison.Ordinal))
        {
            throw FrontendFailure.Toolchain("source", "CSHARP_TOOLCHAIN_ADAPTER");
        }
    }

    private static void ValidateSyntaxTree(
        SyntaxTree syntaxTree,
        SourceText sourceText,
        CSharpParseOptions parseOptions,
        string expectedPath)
    {
        if (!string.Equals(syntaxTree.FilePath, expectedPath, StringComparison.Ordinal)
            || !ReferenceEquals(syntaxTree.Options, parseOptions)
            || !syntaxTree.GetText(CancellationToken.None).ContentEquals(sourceText))
        {
            throw FrontendFailure.Toolchain("source", "CSHARP_TOOLCHAIN_ADAPTER");
        }
    }

    private static void ValidateCompilation(
        CSharpCompilation compilation,
        Selection selection,
        RoslynSourceSession source,
        FrozenReferenceProjection referenceProjection,
        CSharpCompilationOptions options)
    {
        if (!string.Equals(compilation.AssemblyName, selection.Raw.Compilation, StringComparison.Ordinal)
            || !ReferenceEquals(compilation.Options, options)
            || !string.Equals(compilation.Language, LanguageNames.CSharp, StringComparison.Ordinal)
            || !compilation.IsCaseSensitive
            || compilation.ScriptCompilationInfo is not null
            || !compilation.DirectiveReferences.IsEmpty)
        {
            throw FrontendFailure.Toolchain("metadata", "CSHARP_TOOLCHAIN_ADAPTER");
        }

        if (compilation.Options is not CSharpCompilationOptions observedOptions)
        {
            throw FrontendFailure.Toolchain("metadata", "CSHARP_TOOLCHAIN_ADAPTER");
        }

        ValidateCompilationOptions(observedOptions);
        int treeIndex = 0;
        foreach (SyntaxTree tree in compilation.SyntaxTrees)
        {
            if (treeIndex >= source.SyntaxTrees.Length
                || !ReferenceEquals(tree, source.SyntaxTrees[treeIndex]))
            {
                throw FrontendFailure.Toolchain("metadata", "CSHARP_TOOLCHAIN_ADAPTER");
            }

            treeIndex++;
        }

        if (treeIndex != source.SyntaxTrees.Length)
        {
            throw FrontendFailure.Toolchain("metadata", "CSHARP_TOOLCHAIN_ADAPTER");
        }

        int referenceIndex = 0;
        foreach (MetadataReference reference in compilation.References)
        {
            if (referenceIndex >= referenceProjection.References.Length
                || reference is CompilationReference
                || !ReferenceEquals(reference, referenceProjection.References[referenceIndex]))
            {
                throw FrontendFailure.Toolchain("metadata", "CSHARP_TOOLCHAIN_ADAPTER");
            }

            referenceIndex++;
        }

        if (referenceIndex != referenceProjection.Count)
        {
            throw FrontendFailure.Toolchain("metadata", "CSHARP_TOOLCHAIN_ADAPTER");
        }
    }

    private static bool HasActiveDiagnostic(ImmutableArray<Diagnostic> diagnostics)
    {
        foreach (Diagnostic diagnostic in diagnostics)
        {
            if (!diagnostic.IsSuppressed
                && (diagnostic.Severity == DiagnosticSeverity.Warning
                    || diagnostic.Severity == DiagnosticSeverity.Error))
            {
                return true;
            }
        }

        return false;
    }

    private static bool HasAny(IEnumerable<string> values)
    {
        using IEnumerator<string> enumerator = values.GetEnumerator();
        return enumerator.MoveNext();
    }
}
