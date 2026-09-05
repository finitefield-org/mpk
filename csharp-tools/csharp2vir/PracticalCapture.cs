using System;
using System.Collections.Generic;
using System.Collections.Immutable;
using System.Collections.ObjectModel;
using System.IO;
using System.Linq;
using System.Security.Cryptography;
using System.Text;
using System.Text.Encodings.Web;
using System.Text.Json;
using System.Threading;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.CSharp.Syntax;
using Microsoft.CodeAnalysis.Operations;
using Microsoft.CodeAnalysis.Text;

namespace Mpk.CSharp2Vir;

// CSHARP-03-T03-W01 owns this private source gate.  It is deliberately not
// listed in csharp2vir.csproj or either installed build-input manifest yet.
// T07 is the sole owner of candidate materialization and registration.

internal enum PracticalCapturedInputKind
{
    Source,
    Sidecar,
    Project,
    Package,
    Binary,
    GeneratedSource,
    AnalyzerConfig,
    EditorConfig,
}

internal enum PracticalDiagnosticFamily
{
    CSHARP_PRACTICAL_PROTOCOL,
    CSHARP_PRACTICAL_LIMIT,
    CSHARP_PRACTICAL_DEPENDENCY,
    CSHARP_PRACTICAL_DECLARATION,
    CSHARP_PRACTICAL_TYPE,
    CSHARP_PRACTICAL_GENERIC,
    CSHARP_PRACTICAL_OBJECT,
    CSHARP_PRACTICAL_EFFECT,
}

internal sealed class PracticalCaptureFailure : Exception
{
    internal const string PublicMessage = "The selected construct is outside the frozen practical profile.";

    internal PracticalCaptureFailure(int phase, PracticalDiagnosticFamily family, string code)
        : base(PublicMessage)
    {
        Phase = phase;
        Family = family;
        Code = code;
    }

    internal int Phase { get; }

    internal PracticalDiagnosticFamily Family { get; }

    internal string Code { get; }

    internal int ArtifactCount => 0;
}

internal static class PracticalFailures
{
    internal static PracticalCaptureFailure Protocol(string code) =>
        new PracticalCaptureFailure(0, PracticalDiagnosticFamily.CSHARP_PRACTICAL_PROTOCOL, code);

    internal static PracticalCaptureFailure Limit(string code) =>
        new PracticalCaptureFailure(0, PracticalDiagnosticFamily.CSHARP_PRACTICAL_LIMIT, code);

    internal static PracticalCaptureFailure Dependency(string code) =>
        new PracticalCaptureFailure(1, PracticalDiagnosticFamily.CSHARP_PRACTICAL_DEPENDENCY, code);

    internal static PracticalCaptureFailure Declaration(string code) =>
        new PracticalCaptureFailure(2, PracticalDiagnosticFamily.CSHARP_PRACTICAL_DECLARATION, code);

    internal static PracticalCaptureFailure Type(string code) =>
        new PracticalCaptureFailure(2, PracticalDiagnosticFamily.CSHARP_PRACTICAL_TYPE, code);

    internal static PracticalCaptureFailure Generic(string code) =>
        new PracticalCaptureFailure(3, PracticalDiagnosticFamily.CSHARP_PRACTICAL_GENERIC, code);

    internal static PracticalCaptureFailure Object(string code) =>
        new PracticalCaptureFailure(6, PracticalDiagnosticFamily.CSHARP_PRACTICAL_OBJECT, code);

    internal static PracticalCaptureFailure Effect(string code) =>
        new PracticalCaptureFailure(7, PracticalDiagnosticFamily.CSHARP_PRACTICAL_EFFECT, code);
}

internal sealed class PracticalCapturedInput
{
    private readonly byte[] bytes;

    internal PracticalCapturedInput(PracticalCapturedInputKind kind, string normalizedPath, byte[] bytes)
    {
        Kind = kind;
        NormalizedPath = normalizedPath ?? throw new ArgumentNullException(nameof(normalizedPath));
        this.bytes = bytes is null ? throw new ArgumentNullException(nameof(bytes)) : (byte[])bytes.Clone();
        RawSha256 = Convert.ToHexString(SHA256.HashData(this.bytes)).ToLowerInvariant();
    }

    internal PracticalCapturedInputKind Kind { get; }

    internal string NormalizedPath { get; }

    internal int SizeBytes => bytes.Length;

    internal string RawSha256 { get; }

    internal byte[] CopyBytes() => (byte[])bytes.Clone();
}

internal sealed class PracticalSourceSelection
{
    private readonly ReadOnlyCollection<string> sourcePaths;
    private readonly ReadOnlyCollection<string> selectedRootIds;
    private readonly ReadOnlyCollection<string> sidecarPaths;

    internal PracticalSourceSelection(
        string schema,
        string compilationId,
        IEnumerable<string> sourcePaths,
        IEnumerable<string> selectedRootIds,
        IEnumerable<string> sidecarPaths)
    {
        Schema = schema ?? throw PracticalFailures.Protocol("selection_shape");
        CompilationId = compilationId ?? throw PracticalFailures.Protocol("selection_shape");
        this.sourcePaths = Array.AsReadOnly(Copy(
            sourcePaths,
            CSharpPracticalCapture.SourceFilesMaximum,
            "source_files"));
        this.selectedRootIds = Array.AsReadOnly(Copy(
            selectedRootIds,
            CSharpPracticalCapture.SelectedMethodsMaximum,
            "selected_methods"));
        this.sidecarPaths = Array.AsReadOnly(Copy(
            sidecarPaths,
            CSharpPracticalCapture.ContractFilesMaximum,
            "contract_files"));
    }

    internal string Schema { get; }

    internal string CompilationId { get; }

    internal IReadOnlyList<string> SourcePaths => sourcePaths;

    internal IReadOnlyList<string> SelectedRootIds => selectedRootIds;

    internal IReadOnlyList<string> SidecarPaths => sidecarPaths;

    private static string[] Copy(IEnumerable<string> values, int maximum, string limitCode)
    {
        if (values is null)
        {
            throw PracticalFailures.Protocol("selection_shape");
        }

        var copied = new List<string>(maximum);
        try
        {
            foreach (string value in values)
            {
                if (copied.Count == maximum)
                {
                    throw PracticalFailures.Limit(limitCode);
                }

                copied.Add(value);
            }
        }
        catch (PracticalCaptureFailure)
        {
            throw;
        }
        catch (Exception)
        {
            throw PracticalFailures.Protocol("selection_shape");
        }

        return copied.ToArray();
    }
}

internal sealed class PracticalSourceFile
{
    private readonly byte[] bytes;
    private readonly int[] utf8Offsets;

    internal PracticalSourceFile(int ordinal, string path, byte[] bytes, string text)
    {
        Ordinal = ordinal;
        Path = path;
        this.bytes = (byte[])bytes.Clone();
        Text = text;
        RawSha256 = Convert.ToHexString(SHA256.HashData(this.bytes)).ToLowerInvariant();
        utf8Offsets = BuildUtf8Offsets(text, this.bytes.Length);
    }

    internal int Ordinal { get; }

    internal string Path { get; }

    internal string Text { get; }

    internal int SizeBytes => bytes.Length;

    internal string RawSha256 { get; }

    internal byte[] CopyBytes() => (byte[])bytes.Clone();

    internal int ByteOffset(int characterOffset)
    {
        if (characterOffset < 0
            || characterOffset >= utf8Offsets.Length
            || utf8Offsets[characterOffset] < 0)
        {
            throw PracticalFailures.Protocol("source_span");
        }

        return utf8Offsets[characterOffset];
    }

    private static int[] BuildUtf8Offsets(string text, int expectedBytes)
    {
        var offsets = new int[checked(text.Length + 1)];
        int characterOffset = 0;
        int byteOffset = 0;
        while (characterOffset < text.Length)
        {
            offsets[characterOffset] = byteOffset;
            char character = text[characterOffset];
            if (char.IsHighSurrogate(character))
            {
                if (characterOffset + 1 >= text.Length
                    || !char.IsLowSurrogate(text[characterOffset + 1]))
                {
                    throw PracticalFailures.Protocol("source_encoding");
                }

                offsets[characterOffset + 1] = -1;
                byteOffset = checked(byteOffset + 4);
                characterOffset += 2;
            }
            else
            {
                if (char.IsLowSurrogate(character))
                {
                    throw PracticalFailures.Protocol("source_encoding");
                }

                byteOffset = checked(byteOffset + (character <= 0x7f ? 1 : character <= 0x7ff ? 2 : 3));
                characterOffset++;
            }
        }

        offsets[text.Length] = byteOffset;
        if (byteOffset != expectedBytes)
        {
            throw PracticalFailures.Protocol("source_encoding");
        }

        return offsets;
    }
}

internal sealed class PracticalSidecarFile
{
    private readonly byte[] bytes;

    internal PracticalSidecarFile(int ordinal, string path, byte[] bytes)
    {
        Ordinal = ordinal;
        Path = path;
        this.bytes = (byte[])bytes.Clone();
        RawSha256 = Convert.ToHexString(SHA256.HashData(this.bytes)).ToLowerInvariant();
    }

    internal int Ordinal { get; }

    internal string Path { get; }

    internal int SizeBytes => bytes.Length;

    internal string RawSha256 { get; }

    internal byte[] CopyBytes() => (byte[])bytes.Clone();
}

internal enum PracticalDeclarationKind
{
    Type,
    Constructor,
    Method,
    Property,
    Field,
    EnumMember,
}

internal sealed class PracticalDeclaration
{
    internal PracticalDeclaration(
        string id,
        PracticalDeclarationKind kind,
        int sourceOrdinal,
        int startByte,
        int endByte)
    {
        Id = id;
        Kind = kind;
        SourceOrdinal = sourceOrdinal;
        StartByte = startByte;
        EndByte = endByte;
    }

    internal string Id { get; }

    internal PracticalDeclarationKind Kind { get; }

    internal int SourceOrdinal { get; }

    internal int StartByte { get; }

    internal int EndByte { get; }
}

internal sealed class PracticalGraphEdge
{
    internal PracticalGraphEdge(string sourceId, string targetId)
    {
        SourceId = sourceId;
        TargetId = targetId;
    }

    internal string SourceId { get; }

    internal string TargetId { get; }
}

internal sealed class PracticalSourceClosure
{
    private readonly ReadOnlyCollection<PracticalSourceFile> sources;
    private readonly ReadOnlyCollection<PracticalSidecarFile> sidecars;
    private readonly ReadOnlyCollection<PracticalDeclaration> declarations;
    private readonly ReadOnlyCollection<PracticalDeclaration> reachableDeclarations;
    private readonly ReadOnlyCollection<PracticalGraphEdge> callEdges;
    private readonly ReadOnlyCollection<PracticalGraphEdge> typeEdges;

    internal PracticalSourceClosure(
        PracticalSourceFile[] sources,
        PracticalSidecarFile[] sidecars,
        PracticalDeclaration[] declarations,
        PracticalDeclaration[] reachableDeclarations,
        PracticalGraphEdge[] callEdges,
        PracticalGraphEdge[] typeEdges,
        int sourceDataExceptionTypeCount)
    {
        this.sources = Array.AsReadOnly(sources);
        this.sidecars = Array.AsReadOnly(sidecars);
        this.declarations = Array.AsReadOnly(declarations);
        this.reachableDeclarations = Array.AsReadOnly(reachableDeclarations);
        this.callEdges = Array.AsReadOnly(callEdges);
        this.typeEdges = Array.AsReadOnly(typeEdges);
        SourceDataExceptionTypeCount = sourceDataExceptionTypeCount;
    }

    internal IReadOnlyList<PracticalSourceFile> Sources => sources;

    internal IReadOnlyList<PracticalSidecarFile> Sidecars => sidecars;

    internal IReadOnlyList<PracticalDeclaration> Declarations => declarations;

    internal IReadOnlyList<PracticalDeclaration> ReachableDeclarations => reachableDeclarations;

    internal IReadOnlyList<PracticalGraphEdge> CallEdges => callEdges;

    internal IReadOnlyList<PracticalGraphEdge> TypeEdges => typeEdges;

    internal int SourceDataExceptionTypeCount { get; }
}

internal static class PracticalIdentity
{
    private static readonly byte[] DeclarationDomain = Encoding.ASCII.GetBytes(
        "MPK-CSHARP-DECLARATION-1.0\0");
    private static readonly byte[] ClosedInstanceDomain = Encoding.ASCII.GetBytes(
        "MPK-CSHARP-SEMANTIC-INSTANCE-1.0\0");

    internal static string SourceTypeId(string namespaceName, string name)
    {
        ValidateNamespace(namespaceName);
        ValidateIdentifier(name);
        return HashDeclaration("type", name, namespaceName, string.Empty, Array.Empty<string>(), string.Empty);
    }

    internal static string CallableId(
        string kind,
        string namespaceName,
        string ownerId,
        string name,
        IEnumerable<string> parameterTypeIds,
        string resultTypeId)
    {
        if (!string.Equals(kind, "method", StringComparison.Ordinal)
            && !string.Equals(kind, "constructor", StringComparison.Ordinal))
        {
            throw PracticalFailures.Protocol("identity_kind");
        }

        ValidateNamespace(namespaceName);
        ValidateIdentifier(name);
        ValidateSourceId(ownerId);
        string[] parameters = parameterTypeIds.ToArray();
        foreach (string parameter in parameters)
        {
            ValidateConcreteTypeId(parameter);
        }

        ValidateConcreteTypeId(resultTypeId);
        if (string.Equals(kind, "constructor", StringComparison.Ordinal)
            && !string.Equals(ownerId, resultTypeId, StringComparison.Ordinal))
        {
            throw PracticalFailures.Protocol("identity_shape");
        }

        return HashDeclaration(kind, name, namespaceName, ownerId, parameters, resultTypeId);
    }

    internal static string PrimitiveId(string token)
    {
        string id = "mpk.csharp.value." + token + ".v1";
        if (!IsPrimitiveId(id))
        {
            throw PracticalFailures.Protocol("identity_type_id");
        }

        return id;
    }

    internal static string ClosedInstanceId(string template, params string[] argumentTypeIds)
    {
        string templateId = template switch
        {
            "bounded_sequence" => "mpk.csharp.semantic.bounded_sequence.v1",
            "sequence_construction" => "mpk.csharp.semantic.sequence_construction.v1",
            "option" => "mpk.csharp.semantic.option.v1",
            "ordered_entry" => "mpk.csharp.semantic.ordered_entry.v1",
            "ordered_map" => "mpk.csharp.semantic.ordered_map.v1",
            "ordered_set" => "mpk.csharp.semantic.ordered_set.v1",
            "lookup" => "mpk.csharp.semantic.lookup.v1",
            "result" => "mpk.csharp.semantic.result.v1",
            "validation" => "mpk.csharp.semantic.validation.v1",
            "boundary_field" => "mpk.csharp.semantic.boundary_field.v1",
            _ => throw PracticalFailures.Protocol("identity_template"),
        };
        int arity = template is "ordered_entry" or "ordered_map" or "result" or "validation" ? 2 : 1;
        if (argumentTypeIds.Length != arity) { throw PracticalFailures.Protocol("identity_arity"); }
        foreach (string argument in argumentTypeIds) { ValidateConcreteTypeId(argument); }
        string json = "{\"arguments\":[\"" + string.Join("\",\"", argumentTypeIds)
            + "\"],\"template\":\"" + templateId + "\",\"version\":1}";
        byte[] payload = Encoding.ASCII.GetBytes(json);
        var preimage = new byte[checked(ClosedInstanceDomain.Length + payload.Length)];
        Buffer.BlockCopy(ClosedInstanceDomain, 0, preimage, 0, ClosedInstanceDomain.Length);
        Buffer.BlockCopy(payload, 0, preimage, ClosedInstanceDomain.Length, payload.Length);
        return "mpk.csharp.instance."
            + Convert.ToHexString(SHA256.HashData(preimage)).ToLowerInvariant();
    }

    internal static bool IsSourceId(string value)
    {
        const string Prefix = "mpk.csharp.source.";
        if (string.IsNullOrEmpty(value)
            || !value.StartsWith(Prefix, StringComparison.Ordinal)
            || value.Length != Prefix.Length + 64)
        {
            return false;
        }

        for (int index = Prefix.Length; index < value.Length; index++)
        {
            char character = value[index];
            if (!((character >= '0' && character <= '9')
                || (character >= 'a' && character <= 'f')))
            {
                return false;
            }
        }

        return true;
    }

    internal static void ValidateIdentifier(string value)
    {
        if (string.IsNullOrEmpty(value)
            || value.Length > 512
            || !IsAsciiIdentifierStart(value[0]))
        {
            throw PracticalFailures.Declaration("source_identifier");
        }

        for (int index = 1; index < value.Length; index++)
        {
            if (!IsAsciiIdentifierPart(value[index]))
            {
                throw PracticalFailures.Declaration("source_identifier");
            }
        }
    }

    internal static void ValidateNamespace(string value)
    {
        if (string.IsNullOrEmpty(value))
        {
            throw PracticalFailures.Declaration("source_namespace");
        }

        foreach (string component in value.Split('.'))
        {
            ValidateIdentifier(component);
        }
    }

    private static string HashDeclaration(
        string kind,
        string name,
        string namespaceName,
        string owner,
        IReadOnlyList<string> parameters,
        string result)
    {
        var json = new StringBuilder(256);
        json.Append("{\"kind\":\"").Append(kind)
            .Append("\",\"name\":\"").Append(name)
            .Append("\",\"namespace\":\"").Append(namespaceName)
            .Append("\",\"owner\":\"").Append(owner)
            .Append("\",\"parameter_type_ids\":[");
        for (int index = 0; index < parameters.Count; index++)
        {
            if (index != 0)
            {
                json.Append(',');
            }

            json.Append('"').Append(parameters[index]).Append('"');
        }

        json.Append("],\"result_type_id\":\"").Append(result).Append("\"}");
        byte[] payload = Encoding.UTF8.GetBytes(json.ToString());
        var preimage = new byte[checked(DeclarationDomain.Length + payload.Length)];
        Buffer.BlockCopy(DeclarationDomain, 0, preimage, 0, DeclarationDomain.Length);
        Buffer.BlockCopy(payload, 0, preimage, DeclarationDomain.Length, payload.Length);
        return "mpk.csharp.source."
            + Convert.ToHexString(SHA256.HashData(preimage)).ToLowerInvariant();
    }

    private static void ValidateSourceId(string value)
    {
        if (!IsSourceId(value))
        {
            throw PracticalFailures.Protocol("identity_source_id");
        }
    }

    private static void ValidateConcreteTypeId(string value)
    {
        if (IsSourceId(value)
            || IsPrimitiveId(value)
            || IsClosedInstanceId(value))
        {
            return;
        }

        throw PracticalFailures.Protocol("identity_type_id");
    }

    private static bool IsPrimitiveId(string value) => value is
        "mpk.csharp.value.bool.v1"
        or "mpk.csharp.value.i8.v1"
        or "mpk.csharp.value.u8.v1"
        or "mpk.csharp.value.i16.v1"
        or "mpk.csharp.value.u16.v1"
        or "mpk.csharp.value.i32.v1"
        or "mpk.csharp.value.u32.v1"
        or "mpk.csharp.value.i64.v1"
        or "mpk.csharp.value.u64.v1"
        or "mpk.csharp.value.char.v1"
        or "mpk.csharp.value.f32.v1"
        or "mpk.csharp.value.f64.v1"
        or "mpk.csharp.value.decimal.v1"
        or "mpk.csharp.value.string.v1"
        or "mpk.csharp.value.date.v1"
        or "mpk.csharp.value.time.v1"
        or "mpk.csharp.value.duration.v1"
        or "mpk.csharp.value.guid.v1"
        or "mpk.csharp.value.day_of_week.v1"
        or "mpk.csharp.value.unit.v1"
        or "mpk.csharp.value.parse_error.v1"
        or "mpk.csharp.value.instant.v1"
        or "mpk.csharp.value.exception.v1";

    private static bool IsClosedInstanceId(string value)
    {
        const string Prefix = "mpk.csharp.instance.";
        if (string.IsNullOrEmpty(value) || value.Length != Prefix.Length + 64
            || !value.StartsWith(Prefix, StringComparison.Ordinal))
        {
            return false;
        }

        for (int index = Prefix.Length; index < value.Length; index++)
        {
            char character = value[index];
            if (!((character >= '0' && character <= '9')
                || (character >= 'a' && character <= 'f')))
            {
                return false;
            }
        }

        return true;
    }

    private static bool IsAsciiIdentifierStart(char value) =>
        (value >= 'A' && value <= 'Z') || (value >= 'a' && value <= 'z') || value == '_';

    private static bool IsAsciiIdentifierPart(char value) =>
        IsAsciiIdentifierStart(value) || (value >= '0' && value <= '9');
}

internal static class CSharpPracticalCapture
{
    internal const string SelectionSchema = "mpk.selection.csharp_members.v1";
    internal const int SourceDataExceptionTypesMaximum = 128;
    internal const int SourceFilesMaximum = 256;
    internal const int ContractFilesMaximum = 128;
    internal const int SelectedMethodsMaximum = 32;
    private const int SourceFileBytesMaximum = 1_048_576;
    private const int SourceTotalBytesMaximum = 16_777_216;
    private const int ContractFileBytesMaximum = 1_048_576;
    private const int ContractTotalBytesMaximum = 8_388_608;
    private const int SyntaxNodesMaximum = 250_000;
    private const int MethodClosureMaximum = 128;
    private const long ReferenceTotalBytes = 6_046_008;
    private const int ReferenceCanonicalBytes = 24_670;
    private const string ReferenceInventorySha256 = "30623f64b7d85564260e62464e652bfaa89eb56e0e55193989bfb99538ba6cad";
    private static readonly UTF8Encoding StrictUtf8 = new UTF8Encoding(false, true);
    private static readonly byte[] ReferenceInventoryDomain = Encoding.ASCII.GetBytes(
        "MPK-CSHARP-REFERENCE-INVENTORY-0.1\0");
    private static readonly (string Id, DiagnosticSeverity Severity)[] IgnoredCompilerDiagnostics =
    {
        ("CS8019", DiagnosticSeverity.Hidden),
    };
    private static readonly string[] ReflectionAndCodeGenerationPrefixes =
    {
        "System.Activator",
        "System.CodeDom",
        "System.Delegate",
        "System.Linq",
        "System.Reflection",
        "System.Runtime.CompilerServices.RuntimeFeature",
        "System.Runtime.Loader",
        "System.Runtime.Serialization",
        "System.Type",
    };
    private static readonly string[] EffectAndConcurrencyPrefixes =
    {
        "System.Collections.Generic.IEnumerable",
        "System.Collections.Generic.IEnumerator",
        "System.Collections.IEnumerable",
        "System.Collections.IEnumerator",
        "System.Console",
        "System.Data",
        "System.DateTime",
        "System.DateTimeOffset",
        "System.Diagnostics.Debug",
        "System.Diagnostics.EventLog",
        "System.Diagnostics.Process",
        "System.Diagnostics.Stopwatch",
        "System.Diagnostics.Trace",
        "System.Environment",
        "System.IO",
        "System.Net",
        "System.Random",
        "System.Security.Cryptography.RandomNumberGenerator",
        "System.Threading",
        "System.Timers",
        "System.Transactions",
    };

    internal static PracticalSourceClosure Validate(
        PracticalSourceSelection selection,
        IEnumerable<PracticalCapturedInput> capturedInputs,
        ImmutableArray<MetadataReference> references,
        Action<CSharpCompilation>? validateDataDeclarations = null,
        Action<CSharpCompilation>? validateDataTypes = null,
        Action<CSharpCompilation>? validateDataLimits = null,
        Action<CSharpCompilation, PracticalSourceClosure>? validateConstruction = null)
    {
        try
        {
            if (selection is null)
            {
                throw PracticalFailures.Protocol("selection_shape");
            }

            ValidateRoslynRuntime();
            ValidateSelection(selection);
            PracticalCaptureSet captured = CaptureInputs(selection, capturedInputs);
            ImmutableArray<MetadataReference> pinnedReferences = ValidateReferences(references);
            RoslynState roslyn = CreateCompilation(selection, captured.Sources, pinnedReferences);
            ValidateSyntaxNodeLimit(roslyn);
            validateDataLimits?.Invoke(roslyn.Compilation);

            // The ordering below is the frozen diagnostic phase precedence.  A
            // later scan must never mask an earlier dependency/declaration/generic
            // finding merely because Roslyn happened to enumerate it first.
            ValidateDependencies(roslyn);
            ValidateCompilerDiagnostics(roslyn);
            ValidateGlobalDeclarationExclusions(roslyn);
            validateDataDeclarations?.Invoke(roslyn.Compilation);
            ValidateSynthesizedMarkers(roslyn);
            ValidateFrameworkApi(roslyn);
            validateDataTypes?.Invoke(roslyn.Compilation);
            ValidateGenerics(roslyn);
            PracticalSourceClosure closure = BuildClosure(
                selection,
                captured.Sources,
                captured.Sidecars,
                roslyn);
            validateConstruction?.Invoke(roslyn.Compilation, closure);
            ValidateEffectsAndConcurrency(roslyn);
            return closure;
        }
        catch (PracticalCaptureFailure)
        {
            throw;
        }
        catch (Exception)
        {
            throw PracticalFailures.Protocol("frontend_adapter");
        }
    }

    private static void ValidateSelection(PracticalSourceSelection selection)
    {
        if (!string.Equals(selection.Schema, SelectionSchema, StringComparison.Ordinal)
            || !IsCompilationId(selection.CompilationId))
        {
            throw PracticalFailures.Protocol("selection_shape");
        }

        ValidateCount(selection.SourcePaths.Count, 1, SourceFilesMaximum, "source_files");
        ValidateCount(selection.SelectedRootIds.Count, 1, SelectedMethodsMaximum, "selected_methods");
        ValidateCount(selection.SidecarPaths.Count, 0, ContractFilesMaximum, "contract_files");
        ValidateSortedPaths(selection.SourcePaths, "src/", ".cs");
        ValidateSortedPaths(selection.SidecarPaths, null, ".json");
        ValidateSortedIds(selection.SelectedRootIds);

        var folded = new HashSet<string>(StringComparer.Ordinal);
        foreach (string path in selection.SourcePaths.Concat(selection.SidecarPaths))
        {
            if (!folded.Add(path.ToLowerInvariant()))
            {
                throw PracticalFailures.Protocol("selection_path_collision");
            }
        }
    }

    private static PracticalCaptureSet CaptureInputs(
        PracticalSourceSelection selection,
        IEnumerable<PracticalCapturedInput> capturedInputs)
    {
        if (capturedInputs is null)
        {
            throw PracticalFailures.Protocol("input_inventory");
        }

        int expectedCount = checked(selection.SourcePaths.Count + selection.SidecarPaths.Count);
        var inputs = new List<PracticalCapturedInput>(expectedCount);
        foreach (PracticalCapturedInput input in capturedInputs)
        {
            if (inputs.Count == expectedCount)
            {
                throw PracticalFailures.Dependency("input_inventory");
            }

            inputs.Add(input);
        }

        if (inputs.Count != expectedCount)
        {
            throw PracticalFailures.Dependency("input_inventory");
        }

        var observed = new HashSet<string>(StringComparer.Ordinal);
        var byPath = new Dictionary<string, PracticalCapturedInput>(StringComparer.Ordinal);
        foreach (PracticalCapturedInput input in inputs)
        {
            if (input is null)
            {
                throw PracticalFailures.Dependency("input_inventory");
            }

            if (input.Kind != PracticalCapturedInputKind.Source
                && input.Kind != PracticalCapturedInputKind.Sidecar)
            {
                throw PracticalFailures.Dependency(InputKindCode(input.Kind));
            }

            string key = input.Kind.ToString() + "\0" + input.NormalizedPath;
            if (!observed.Add(key) || byPath.ContainsKey(input.NormalizedPath))
            {
                throw PracticalFailures.Dependency("input_inventory");
            }

            byPath.Add(input.NormalizedPath, input);
        }

        var sources = new PracticalSourceFile[selection.SourcePaths.Count];
        int sourceTotal = 0;
        for (int index = 0; index < selection.SourcePaths.Count; index++)
        {
            string path = selection.SourcePaths[index];
            if (!byPath.TryGetValue(path, out PracticalCapturedInput? input)
                || input.Kind != PracticalCapturedInputKind.Source)
            {
                throw PracticalFailures.Dependency("input_inventory");
            }

            byte[] bytes = input.CopyBytes();
            if (bytes.Length > SourceFileBytesMaximum)
            {
                throw PracticalFailures.Limit("source_file_bytes");
            }

            sourceTotal = CheckedAdd(sourceTotal, bytes.Length, SourceTotalBytesMaximum, "source_total_bytes");
            string text = DecodeSource(bytes);
            sources[index] = new PracticalSourceFile(index, path, bytes, text);
        }

        var sidecars = new PracticalSidecarFile[selection.SidecarPaths.Count];
        int sidecarTotal = 0;
        for (int index = 0; index < selection.SidecarPaths.Count; index++)
        {
            string path = selection.SidecarPaths[index];
            if (!byPath.TryGetValue(path, out PracticalCapturedInput? input)
                || input.Kind != PracticalCapturedInputKind.Sidecar)
            {
                throw PracticalFailures.Dependency("input_inventory");
            }

            byte[] bytes = input.CopyBytes();
            if (bytes.Length > ContractFileBytesMaximum)
            {
                throw PracticalFailures.Limit("contract_file_bytes");
            }

            sidecarTotal = CheckedAdd(
                sidecarTotal,
                bytes.Length,
                ContractTotalBytesMaximum,
                "contract_total_bytes");
            sidecars[index] = new PracticalSidecarFile(index, path, bytes);
        }

        return new PracticalCaptureSet(sources, sidecars);
    }

    private static string DecodeSource(byte[] bytes)
    {
        if (bytes.Length == 0
            || (bytes.Length >= 3 && bytes[0] == 0xef && bytes[1] == 0xbb && bytes[2] == 0xbf)
            || bytes[^1] != (byte)'\n'
            || Array.IndexOf(bytes, (byte)'\r') >= 0
            || Array.IndexOf(bytes, (byte)0) >= 0)
        {
            throw PracticalFailures.Protocol("source_encoding");
        }

        try
        {
            string text = StrictUtf8.GetString(bytes);
            foreach (char character in text)
            {
                if (character == '\u0085'
                    || character == '\u2028'
                    || character == '\u2029'
                    || character == '\ufffe'
                    || character == '\uffff')
                {
                    throw PracticalFailures.Protocol("source_encoding");
                }
            }

            return text;
        }
        catch (DecoderFallbackException)
        {
            throw PracticalFailures.Protocol("source_encoding");
        }
    }

    private static ImmutableArray<MetadataReference> ValidateReferences(
        ImmutableArray<MetadataReference> references)
    {
        if (references.IsDefault || references.Length != 167)
        {
            throw PracticalFailures.Dependency("ambient_reference");
        }

        var records = new List<PracticalReferenceRecord>(references.Length);
        try
        {
            foreach (MetadataReference reference in references)
            {
                if (reference is not PortableExecutableReference portable
                    || !portable.Properties.Equals(MetadataReferenceProperties.Assembly)
                    || string.IsNullOrEmpty(portable.FilePath))
                {
                    throw PracticalFailures.Dependency("ambient_reference");
                }

                var file = new FileInfo(portable.FilePath);
                string name = file.Name;
                if (!name.EndsWith(".dll", StringComparison.Ordinal)
                    || file.LinkTarget is not null
                    || (file.Attributes & FileAttributes.ReparsePoint) != 0
                    || IsMpkAssemblyName(name))
                {
                    throw PracticalFailures.Dependency(
                        IsMpkAssemblyName(name) ? "mpk_assembly" : "ambient_reference");
                }

                long fileLength = file.Length;
                if (fileLength < 0 || fileLength > ReferenceTotalBytes)
                {
                    throw PracticalFailures.Dependency("reference_projection");
                }

                byte[] image = new byte[checked((int)fileLength)];
                using (var stream = new FileStream(
                    file.FullName,
                    FileMode.Open,
                    FileAccess.Read,
                    FileShare.Read,
                    64 * 1024,
                    FileOptions.SequentialScan))
                {
                    int offset = 0;
                    while (offset < image.Length)
                    {
                        int read = stream.Read(image, offset, image.Length - offset);
                        if (read == 0)
                        {
                            break;
                        }

                        offset = checked(offset + read);
                    }

                    if (offset != image.Length
                        || stream.ReadByte() != -1
                        || stream.Length != fileLength)
                    {
                        throw PracticalFailures.Dependency("reference_projection");
                    }
                }

                records.Add(new PracticalReferenceRecord(
                    "ref/net10.0/" + name,
                    image.LongLength,
                    Convert.ToHexString(SHA256.HashData(image)).ToLowerInvariant(),
                    MetadataReference.CreateFromImage(
                        ImmutableArray.Create(image),
                        MetadataReferenceProperties.Assembly,
                        documentation: null,
                        filePath: "ref/net10.0/" + name)));
            }
        }
        catch (PracticalCaptureFailure)
        {
            throw;
        }
        catch (Exception error) when (
            error is ArgumentException
            or IOException
            or NotSupportedException
            or UnauthorizedAccessException
            or System.Security.SecurityException)
        {
            throw PracticalFailures.Dependency("reference_projection");
        }

        records.Sort((left, right) => string.CompareOrdinal(left.Path, right.Path));
        long totalBytes = 0;
        string? previous = null;
        foreach (PracticalReferenceRecord record in records)
        {
            if (previous is not null && string.CompareOrdinal(previous, record.Path) >= 0)
            {
                throw PracticalFailures.Dependency("ambient_reference");
            }

            totalBytes = checked(totalBytes + record.SizeBytes);
            previous = record.Path;
        }

        byte[] canonical = CanonicalReferenceInventory(records);
        var preimage = new byte[checked(ReferenceInventoryDomain.Length + canonical.Length)];
        Buffer.BlockCopy(ReferenceInventoryDomain, 0, preimage, 0, ReferenceInventoryDomain.Length);
        Buffer.BlockCopy(canonical, 0, preimage, ReferenceInventoryDomain.Length, canonical.Length);
        string inventorySha256 = Convert.ToHexString(SHA256.HashData(preimage)).ToLowerInvariant();
        if (totalBytes != ReferenceTotalBytes
            || canonical.Length != ReferenceCanonicalBytes
            || !string.Equals(inventorySha256, ReferenceInventorySha256, StringComparison.Ordinal))
        {
            throw PracticalFailures.Dependency("reference_projection");
        }

        return records.Select(record => (MetadataReference)record.Reference).ToImmutableArray();
    }

    private static byte[] CanonicalReferenceInventory(IReadOnlyList<PracticalReferenceRecord> records)
    {
        using var output = new MemoryStream(ReferenceCanonicalBytes);
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
            foreach (PracticalReferenceRecord record in records)
            {
                writer.WriteStartObject();
                writer.WriteString("path", record.Path);
                writer.WriteString("sha256", record.Sha256);
                writer.WriteNumber("size_bytes", record.SizeBytes);
                writer.WriteEndObject();
            }
            writer.WriteEndArray();
        }

        return output.ToArray();
    }

    private static RoslynState CreateCompilation(
        PracticalSourceSelection selection,
        PracticalSourceFile[] sources,
        ImmutableArray<MetadataReference> references)
    {
        var parseOptions = new CSharpParseOptions(
            (LanguageVersion)1400,
            DocumentationMode.None,
            SourceCodeKind.Regular,
            preprocessorSymbols: Array.Empty<string>());
        if (parseOptions.LanguageVersion != (LanguageVersion)1400
            || parseOptions.DocumentationMode != DocumentationMode.None
            || parseOptions.Kind != SourceCodeKind.Regular
            || parseOptions.PreprocessorSymbolNames.Any()
            || parseOptions.Features.Count != 0)
        {
            throw PracticalFailures.Protocol("roslyn_parse_options");
        }
        var trees = ImmutableArray.CreateBuilder<SyntaxTree>(sources.Length);
        foreach (PracticalSourceFile source in sources)
        {
            trees.Add(CSharpSyntaxTree.ParseText(
                source.Text,
                parseOptions,
                source.Path,
                Encoding.UTF8,
                CancellationToken.None));
        }

        var options = new CSharpCompilationOptions(
            OutputKind.DynamicallyLinkedLibrary,
            optimizationLevel: OptimizationLevel.Release,
            checkOverflow: true,
            allowUnsafe: false,
            platform: Platform.X64,
            warningLevel: 9999,
            generalDiagnosticOption: ReportDiagnostic.Error,
            nullableContextOptions: NullableContextOptions.Enable,
            concurrentBuild: false,
            deterministic: true,
            metadataImportOptions: MetadataImportOptions.Public);
        if (options.OutputKind != OutputKind.DynamicallyLinkedLibrary
            || options.OptimizationLevel != OptimizationLevel.Release
            || !options.CheckOverflow
            || options.AllowUnsafe
            || options.Platform != Platform.X64
            || options.WarningLevel != 9999
            || options.GeneralDiagnosticOption != ReportDiagnostic.Error
            || options.NullableContextOptions != NullableContextOptions.Enable
            || options.ConcurrentBuild
            || !options.Deterministic
            || options.MetadataImportOptions != MetadataImportOptions.Public
            || options.SpecificDiagnosticOptions.Count != 0)
        {
            throw PracticalFailures.Protocol("roslyn_compilation_options");
        }
        CSharpCompilation compilation = CSharpCompilation.Create(
            selection.CompilationId,
            trees.MoveToImmutable(),
            references,
            options);
        return new RoslynState(compilation, compilation.SyntaxTrees.ToImmutableArray());
    }

    private static void ValidateDependencies(RoslynState state)
    {
        foreach (SyntaxTree tree in state.Trees)
        {
            SemanticModel model = state.Compilation.GetSemanticModel(tree, ignoreAccessibility: false);
            SyntaxNode root = tree.GetRoot(CancellationToken.None);
            foreach (SyntaxNode node in root.DescendantNodesAndSelf(descendIntoTrivia: false))
            {
                if (node is UsingDirectiveSyntax usingDirective
                    && IsMpkNamespaceName(usingDirective.Name?.ToString() ?? string.Empty))
                {
                    throw PracticalFailures.Dependency("mpk_namespace");
                }

                if (node is BaseNamespaceDeclarationSyntax namespaceDeclaration
                    && IsMpkNamespaceName(namespaceDeclaration.Name.ToString()))
                {
                    throw PracticalFailures.Dependency("mpk_namespace");
                }

                if (node is NameSyntax nameSyntax
                    && IsMpkDependencyName(
                        model,
                        nameSyntax,
                        nameSyntax.ToString(),
                        state.Compilation))
                {
                    throw PracticalFailures.Dependency("mpk_symbol");
                }

                if (node is AttributeSyntax attribute)
                {
                    ISymbol? symbol = model.GetSymbolInfo(attribute, CancellationToken.None).Symbol;
                    if (IsMpkDependencySymbol(symbol, state.Compilation)
                        || IsMpkDependencyName(
                            model,
                            attribute.Name,
                            attribute.Name.ToString(),
                            state.Compilation))
                    {
                        throw PracticalFailures.Dependency("mpk_attribute");
                    }
                }

                ISymbol? referenced = ReferencedSymbol(model, node);
                if (IsMpkDependencySymbol(referenced, state.Compilation))
                {
                    throw PracticalFailures.Dependency("mpk_symbol");
                }
            }
        }

        foreach (IAssemblySymbol assembly in state.Compilation.SourceModule.ReferencedAssemblySymbols)
        {
            if (IsMpkAssemblyName(assembly.Identity.Name))
            {
                throw PracticalFailures.Dependency("mpk_assembly");
            }
        }
    }

    private static void ValidateCompilerDiagnostics(RoslynState state)
    {
        foreach (Diagnostic diagnostic in state.Compilation.GetDiagnostics(CancellationToken.None))
        {
            if (diagnostic.IsSuppressed)
            {
                continue;
            }

            if (!IsCompilerDiagnosticId(diagnostic.Id))
            {
                throw PracticalFailures.Protocol("compiler_diagnostic_identity");
            }

            ValidateDiagnosticLocation(diagnostic, state);
            if (diagnostic.Severity == DiagnosticSeverity.Error
                || diagnostic.Severity == DiagnosticSeverity.Warning)
            {
                throw PracticalFailures.Declaration("compiler_diagnostic");
            }

            if (!IgnoredCompilerDiagnostics.Contains((diagnostic.Id, diagnostic.Severity)))
            {
                throw PracticalFailures.Declaration("compiler_diagnostic");
            }
        }
    }

    private static bool IsCompilerDiagnosticId(string id)
    {
        if (id.Length != 6 || id[0] != 'C' || id[1] != 'S')
        {
            return false;
        }

        for (int index = 2; index < id.Length; index++)
        {
            if (!char.IsAsciiDigit(id[index]))
            {
                return false;
            }
        }

        return true;
    }

    private static void ValidateDiagnosticLocation(Diagnostic diagnostic, RoslynState state)
    {
        Location location = diagnostic.Location;
        if (location == Location.None || !location.IsInSource)
        {
            return;
        }

        SyntaxTree? tree = location.SourceTree;
        if (tree is null || !state.ContainsTree(tree))
        {
            return;
        }

        TextSpan span = location.SourceSpan;
        int length = tree.GetText(CancellationToken.None).Length;
        if (span.Start < 0 || span.End < span.Start || span.End > length)
        {
            throw PracticalFailures.Protocol("compiler_diagnostic_location");
        }
    }

    private static void ValidateGlobalDeclarationExclusions(RoslynState state)
    {
        foreach (SyntaxTree tree in state.Trees)
        {
            SemanticModel model = state.Compilation.GetSemanticModel(tree, ignoreAccessibility: false);
            SyntaxNode root = tree.GetRoot(CancellationToken.None);
            ValidateDirectives(root);
            ValidateIdentifierTokens(root);
            foreach (SyntaxNode node in root.DescendantNodesAndSelf(descendIntoTrivia: false))
            {
                if (node is AttributeSyntax)
                {
                    throw PracticalFailures.Declaration("source_attribute");
                }

                if (node is DelegateDeclarationSyntax
                    || node is AnonymousFunctionExpressionSyntax
                    || node is QueryExpressionSyntax
                    || node is TypeOfExpressionSyntax
                    || node is FunctionPointerTypeSyntax)
                {
                    throw PracticalFailures.Declaration("delegate_dynamic_or_runtime_codegen");
                }

                if (node is RecordDeclarationSyntax
                    || node is InterfaceDeclarationSyntax
                    || node is EventDeclarationSyntax
                    || node is EventFieldDeclarationSyntax
                    || node is IndexerDeclarationSyntax
                    || node is OperatorDeclarationSyntax
                    || node is ConversionOperatorDeclarationSyntax
                    || node is DestructorDeclarationSyntax
                    || node is LocalFunctionStatementSyntax)
                {
                    throw PracticalFailures.Declaration("unsupported_declaration");
                }

                if (node is ClassDeclarationSyntax { ParameterList: not null }
                    or StructDeclarationSyntax { ParameterList: not null })
                {
                    throw PracticalFailures.Declaration("unsupported_declaration");
                }

                if (node is MemberDeclarationSyntax member
                    && HasPartialModifier(member))
                {
                    throw PracticalFailures.Declaration("partial_declaration");
                }

                if (node is BaseTypeDeclarationSyntax typeDeclaration
                    && typeDeclaration.Parent is TypeDeclarationSyntax)
                {
                    throw PracticalFailures.Declaration("nested_type");
                }

                if (node is DirectiveTriviaSyntax directive
                    && !IsExactFileWideNullableEnable(directive, root))
                {
                    throw PracticalFailures.Declaration("source_directive");
                }

                ISymbol? symbol = ReferencedSymbol(model, node);
                if (IsReflectionOrCodeGeneration(symbol)
                    || IsDelegateType(ReferencedType(model, node)))
                {
                    throw PracticalFailures.Declaration("delegate_dynamic_or_runtime_codegen");
                }

                if (node is ExpressionSyntax expression)
                {
                    TypeInfo type = model.GetTypeInfo(expression, CancellationToken.None);
                    if (type.Type?.TypeKind == TypeKind.Dynamic
                        || type.ConvertedType?.TypeKind == TypeKind.Dynamic
                        || (symbol is IMethodSymbol
                            && type.ConvertedType?.TypeKind == TypeKind.Delegate))
                    {
                        throw PracticalFailures.Declaration("delegate_dynamic_or_runtime_codegen");
                    }
                }
            }
        }
    }

    private static void ValidateIdentifierTokens(SyntaxNode root)
    {
        foreach (SyntaxToken token in root.DescendantTokens(descendIntoTrivia: false))
        {
            if (!token.IsKind(SyntaxKind.IdentifierToken))
            {
                continue;
            }

            if (!string.Equals(token.Text, token.ValueText, StringComparison.Ordinal))
            {
                throw PracticalFailures.Declaration("source_identifier");
            }

            PracticalIdentity.ValidateIdentifier(token.Text);
        }
    }

    private static void ValidateSyntaxNodeLimit(RoslynState state)
    {
        int syntaxNodes = 0;
        foreach (SyntaxTree tree in state.Trees)
        {
            foreach (SyntaxNode _ in tree.GetRoot(CancellationToken.None)
                .DescendantNodesAndSelf(descendIntoTrivia: false))
            {
                syntaxNodes = CheckedAdd(syntaxNodes, 1, SyntaxNodesMaximum, "syntax_nodes");
            }
        }
    }

    private static void ValidateSynthesizedMarkers(RoslynState state)
    {
        foreach (SyntaxTree tree in state.Trees)
        {
            SemanticModel model = state.Compilation.GetSemanticModel(tree, ignoreAccessibility: false);
            foreach (PropertyDeclarationSyntax syntax in tree.GetRoot(CancellationToken.None)
                .DescendantNodes(descendIntoTrivia: false)
                .OfType<PropertyDeclarationSyntax>())
            {
                if (model.GetDeclaredSymbol(syntax, CancellationToken.None) is not IPropertySymbol property)
                {
                    throw PracticalFailures.Declaration("synthesized_marker");
                }

                bool sourceInit = syntax.AccessorList?.Accessors.Any(accessor =>
                    accessor.IsKind(SyntaxKind.InitAccessorDeclaration)) == true;
                bool sourceRequired = syntax.Modifiers.Any(SyntaxKind.RequiredKeyword);
                if (property.IsRequired != sourceRequired
                    || (property.SetMethod?.IsInitOnly == true) != sourceInit)
                {
                    throw PracticalFailures.Declaration("synthesized_marker");
                }

                if (sourceInit)
                {
                    IMethodSymbol setter = property.SetMethod
                        ?? throw PracticalFailures.Declaration("synthesized_marker");
                    ImmutableArray<CustomModifier> modifiers = setter.ReturnTypeCustomModifiers;
                    if (modifiers.Length != 1
                        || modifiers[0].IsOptional
                        || !IsExactCompilerMarker(
                            modifiers[0].Modifier,
                            "System.Runtime.CompilerServices.IsExternalInit"))
                    {
                        throw PracticalFailures.Declaration("synthesized_marker");
                    }
                }

                ValidateMarkerAttributes(property.GetAttributes());
            }

            foreach (FieldDeclarationSyntax syntax in tree.GetRoot(CancellationToken.None)
                .DescendantNodes(descendIntoTrivia: false)
                .OfType<FieldDeclarationSyntax>())
            {
                bool sourceRequired = syntax.Modifiers.Any(SyntaxKind.RequiredKeyword);
                foreach (VariableDeclaratorSyntax variable in syntax.Declaration.Variables)
                {
                    if (model.GetDeclaredSymbol(variable, CancellationToken.None) is not IFieldSymbol field
                        || field.IsRequired != sourceRequired)
                    {
                        throw PracticalFailures.Declaration("synthesized_marker");
                    }

                    ValidateMarkerAttributes(field.GetAttributes());
                }
            }

            foreach (TypeDeclarationSyntax syntax in tree.GetRoot(CancellationToken.None)
                .DescendantNodes(descendIntoTrivia: false)
                .OfType<TypeDeclarationSyntax>())
            {
                if (model.GetDeclaredSymbol(syntax, CancellationToken.None) is not INamedTypeSymbol type)
                {
                    throw PracticalFailures.Declaration("synthesized_marker");
                }

                bool sourceRequired = syntax.Members.Any(member => member switch
                {
                    PropertyDeclarationSyntax property => property.Modifiers.Any(SyntaxKind.RequiredKeyword),
                    FieldDeclarationSyntax field => field.Modifiers.Any(SyntaxKind.RequiredKeyword),
                    _ => false,
                });
                bool semanticRequired = type.GetMembers().Any(member => member switch
                {
                    IPropertySymbol property => property.IsRequired,
                    IFieldSymbol field => field.IsRequired,
                    _ => false,
                });
                if (semanticRequired != sourceRequired)
                {
                    throw PracticalFailures.Declaration("synthesized_marker");
                }

                ValidateMarkerAttributes(type.GetAttributes());
            }
        }
    }

    private static void ValidateMarkerAttributes(ImmutableArray<AttributeData> attributes)
    {
        foreach (AttributeData attribute in attributes)
        {
            INamedTypeSymbol? type = attribute.AttributeClass;
            string name = type?.ToDisplayString(SymbolDisplayFormat.CSharpErrorMessageFormat)
                ?? string.Empty;
            if (name is "System.Runtime.CompilerServices.RequiredMemberAttribute"
                or "System.Runtime.CompilerServices.CompilerFeatureRequiredAttribute"
                or "System.Diagnostics.CodeAnalysis.SetsRequiredMembersAttribute")
            {
                if (type is null
                    || attribute.ApplicationSyntaxReference is not null
                    || !IsExactCompilerMarker(type, name))
                {
                    throw PracticalFailures.Declaration("synthesized_marker");
                }
            }
        }
    }

    private static bool IsExactCompilerMarker(ITypeSymbol type, string expectedName) =>
        string.Equals(
            type.ToDisplayString(SymbolDisplayFormat.CSharpErrorMessageFormat),
            expectedName,
            StringComparison.Ordinal)
        && type.DeclaringSyntaxReferences.IsEmpty
        && string.Equals(type.ContainingAssembly?.Identity.Name, "System.Runtime", StringComparison.Ordinal);

    private static void ValidateGenerics(RoslynState state)
    {
        foreach (SyntaxTree tree in state.Trees)
        {
            SemanticModel model = state.Compilation.GetSemanticModel(tree, ignoreAccessibility: false);
            SyntaxNode root = tree.GetRoot(CancellationToken.None);
            foreach (SyntaxNode node in root.DescendantNodesAndSelf(descendIntoTrivia: false))
            {
                if (node is TypeParameterListSyntax
                    || node is TypeParameterConstraintClauseSyntax
                    || node is TypeArgumentListSyntax)
                {
                    throw PracticalFailures.Generic("user_or_constructed_generic");
                }

                if (node is NullableTypeSyntax nullable)
                {
                    ValidateNullableValueType(nullable, model);
                    continue;
                }

                if (node is TypeSyntax typeSyntax)
                {
                    ITypeSymbol? type = model.GetTypeInfo(typeSyntax, CancellationToken.None).Type;
                    RejectDirectGenericType(type);
                    ValidateSourceVisibleType(
                        type,
                        state.Compilation,
                        IsWithinEffectReference(typeSyntax, model),
                        IsWithinIntrinsicConstantReference(typeSyntax, model));
                }

                if (node is InvocationExpressionSyntax invocation
                    && model.GetSymbolInfo(invocation, CancellationToken.None).Symbol is IMethodSymbol method
                    && (method.IsGenericMethod || method.OriginalDefinition.IsGenericMethod))
                {
                    throw PracticalFailures.Generic("generic_method");
                }

                if (node is ExpressionSyntax expression)
                {
                    TypeInfo information = model.GetTypeInfo(expression, CancellationToken.None);
                    RejectDirectGenericType(information.Type);
                    RejectDirectGenericType(information.ConvertedType);
                }
            }
        }
    }

    private static void ValidateEffectsAndConcurrency(RoslynState state)
    {
        foreach (SyntaxTree tree in state.Trees)
        {
            SemanticModel model = state.Compilation.GetSemanticModel(tree, ignoreAccessibility: false);
            foreach (SyntaxNode node in tree.GetRoot(CancellationToken.None).DescendantNodesAndSelf(descendIntoTrivia: false))
            {
                if (node is AwaitExpressionSyntax
                    || node is YieldStatementSyntax
                    || node is ForEachStatementSyntax
                    || node is ForEachVariableStatementSyntax
                    || node is LockStatementSyntax)
                {
                    throw PracticalFailures.Effect("external_effect_or_concurrency");
                }

                if (node is MethodDeclarationSyntax methodDeclaration
                    && methodDeclaration.Modifiers.Any(SyntaxKind.AsyncKeyword))
                {
                    throw PracticalFailures.Effect("external_effect_or_concurrency");
                }

                if (node is FieldDeclarationSyntax fieldDeclaration
                    && fieldDeclaration.Modifiers.Any(SyntaxKind.VolatileKeyword))
                {
                    throw PracticalFailures.Effect("external_effect_or_concurrency");
                }

                ISymbol? symbol = ReferencedSymbol(model, node);
                if (IsEffectOrConcurrency(symbol))
                {
                    throw PracticalFailures.Effect("external_effect_or_concurrency");
                }
            }
        }
    }

    private static void ValidateFrameworkApi(RoslynState state)
    {
        foreach (SyntaxTree tree in state.Trees)
        {
            SemanticModel model = state.Compilation.GetSemanticModel(tree, ignoreAccessibility: false);
            foreach (SyntaxNode node in tree.GetRoot(CancellationToken.None).DescendantNodesAndSelf(descendIntoTrivia: false))
            {
                ISymbol? symbol = ReferencedSymbol(model, node);
                if (IsEffectOrConcurrency(symbol))
                {
                    continue;
                }

                // Constructed types and generic methods belong to the frozen
                // phase-3 generic family.  Do not let the earlier framework-
                // API firewall relabel them as a phase-2 type failure.
                if (RequiresGenericDiagnostic(symbol))
                {
                    continue;
                }

                if (symbol is IMethodSymbol externalMethod
                    && !IsSourceSymbol(externalMethod, state.Compilation)
                    && externalMethod.MethodKind != MethodKind.BuiltinOperator
                    && externalMethod.MethodKind != MethodKind.Conversion
                    && (!IsAllowlistedFrameworkMember(externalMethod)
                        && !IsOutcomeThrowConstructor(externalMethod, node)
                        || !HasExactIntrinsicArguments(externalMethod, node, model)))
                {
                    throw PracticalFailures.Type("framework_api");
                }

                if (symbol is IPropertySymbol externalProperty
                    && !IsSourceSymbol(externalProperty, state.Compilation)
                    && !IsAllowlistedFrameworkProperty(externalProperty))
                {
                    throw PracticalFailures.Type("framework_api");
                }

                if (symbol is IFieldSymbol externalField
                    && !IsSourceSymbol(externalField, state.Compilation)
                    && !IsAllowlistedFrameworkField(externalField, node, model))
                {
                    throw PracticalFailures.Type("framework_api");
                }
            }
        }
    }

    private static void ValidateRoslynRuntime()
    {
        System.Reflection.AssemblyName common = typeof(Compilation).Assembly.GetName();
        System.Reflection.AssemblyName csharp = typeof(CSharpCompilation).Assembly.GetName();
        if (!string.Equals(common.Name, "Microsoft.CodeAnalysis", StringComparison.Ordinal)
            || !string.Equals(csharp.Name, "Microsoft.CodeAnalysis.CSharp", StringComparison.Ordinal)
            || !string.Equals(common.Version?.ToString(), "5.6.0.0", StringComparison.Ordinal)
            || !string.Equals(csharp.Version?.ToString(), "5.6.0.0", StringComparison.Ordinal))
        {
            throw PracticalFailures.Protocol("roslyn_runtime");
        }
    }

    private static PracticalSourceClosure BuildClosure(
        PracticalSourceSelection selection,
        PracticalSourceFile[] sources,
        PracticalSidecarFile[] sidecars,
        RoslynState state)
    {
        var model = new PracticalDeclarationModel(state, sources);
        model.Collect();
        model.ResolveIdentities();
        model.ResolveEdges();
        return model.Close(selection, sidecars);
    }

    private static void ValidateNullableValueType(NullableTypeSyntax syntax, SemanticModel model)
    {
        ITypeSymbol? symbol = model.GetTypeInfo(syntax, CancellationToken.None).Type;
        if (symbol?.IsReferenceType == true)
        {
            RejectDirectGenericType(symbol);
            ValidateSourceVisibleType(
                symbol,
                model.Compilation,
                IsWithinEffectReference(syntax, model),
                IsWithinIntrinsicConstantReference(syntax, model));
            return;
        }

        if (symbol is not INamedTypeSymbol named
            || !IsExactNullable(named)
            || named.TypeArguments.Length != 1
            || !named.TypeArguments[0].IsValueType
            || named.TypeArguments[0] is INamedTypeSymbol nested && IsExactNullable(nested)
            || !IsAdmittedNullablePayload(named.TypeArguments[0])
            || syntax.ElementType is GenericNameSyntax)
        {
            throw PracticalFailures.Generic("nullable_shape");
        }
    }

    private static void RejectDirectGenericType(ITypeSymbol? type)
    {
        if (type is INamedTypeSymbol named && named.IsGenericType)
        {
            if (IsExactNullable(named))
            {
                return;
            }

            throw PracticalFailures.Generic("constructed_type");
        }

        if (type?.TypeKind == TypeKind.TypeParameter)
        {
            throw PracticalFailures.Generic("type_parameter");
        }
    }

    private static void ValidateSourceVisibleType(
        ITypeSymbol? type,
        Compilation compilation,
        bool withinEffectReference,
        bool withinIntrinsicConstantReference)
    {
        if (type is null)
        {
            return;
        }

        RejectDirectGenericType(type);

        if (type is IArrayTypeSymbol array)
        {
            if (!array.IsSZArray || array.Rank != 1)
            {
                throw PracticalFailures.Type("closed_type");
            }

            ValidateSourceVisibleType(
                array.ElementType,
                compilation,
                withinEffectReference,
                withinIntrinsicConstantReference);
            return;
        }

        if (IsSourceSymbol(type, compilation)
            || IsAdmittedPredefinedType(type)
            || FrameworkTypeToken(type) is not null
            || withinIntrinsicConstantReference && IsIntrinsicConstantCarrier(type))
        {
            return;
        }

        if (type is INamedTypeSymbol nullable && IsExactNullable(nullable))
        {
            ValidateSourceVisibleType(
                nullable.TypeArguments[0],
                compilation,
                withinEffectReference,
                withinIntrinsicConstantReference);
            return;
        }

        // Effect protocols retain their frozen phase-7 diagnostic instead of
        // being masked by the earlier closed-type firewall.
        if (withinEffectReference || IsEffectOrConcurrency(type))
        {
            return;
        }

        throw PracticalFailures.Type("closed_type");
    }

    private static bool IsWithinEffectReference(TypeSyntax syntax, SemanticModel model)
    {
        foreach (SyntaxNode node in syntax.AncestorsAndSelf())
        {
            if (IsEffectOrConcurrency(ReferencedSymbol(model, node)))
            {
                return true;
            }

            if (node is StatementSyntax or MemberDeclarationSyntax)
            {
                break;
            }
        }

        return false;
    }

    private static bool IsWithinIntrinsicConstantReference(
        TypeSyntax syntax,
        SemanticModel model)
    {
        foreach(var creation in syntax.AncestorsAndSelf().OfType<ObjectCreationExpressionSyntax>()) {
            if(model.GetSymbolInfo(creation).Symbol is IMethodSymbol constructor&&IsOutcomeThrowConstructor(constructor,creation)){return true;}
        }
        foreach (MemberAccessExpressionSyntax access in syntax.AncestorsAndSelf()
            .OfType<MemberAccessExpressionSyntax>())
        {
            if (model.GetSymbolInfo(access, CancellationToken.None).Symbol is IMethodSymbol method
                && IsAllowlistedFloatingMember(method)) { return true; }
            if (model.GetSymbolInfo(access, CancellationToken.None).Symbol is IFieldSymbol field
                && IsAllowlistedFrameworkField(field, access, model))
            {
                return true;
            }
        }

        return false;
    }

    // W12 source helpers may construct these exact exceptions only as the
    // immediate operand of throw. This admits no exception-valued API/catch.
    private static bool IsOutcomeThrowConstructor(IMethodSymbol method,SyntaxNode syntax) =>
        method.MethodKind==MethodKind.Constructor&&method.Parameters.Length==0
        && (IsExactSystemRuntimeType(method.ContainingType,"InvalidOperationException")
            || IsExactSystemRuntimeType(method.ContainingType,"ArgumentException"))
        && syntax is ObjectCreationExpressionSyntax {Parent:ThrowExpressionSyntax or ThrowStatementSyntax};

    private static bool IsIntrinsicConstantCarrier(ITypeSymbol type) =>
        IsExactSystemRuntimeType(type, "StringComparison")
        || IsExactSystemRuntimeType(type, "MidpointRounding")
        || IsExactSystemRuntimeType(type, "Math")
        || IsExactSystemRuntimeType(type, "MathF")
        || IsExactSystemRuntimeType(type,"InvalidOperationException")
        || IsExactSystemRuntimeType(type,"ArgumentException");

    private static bool IsAdmittedPredefinedType(ITypeSymbol type) => type.SpecialType is
        SpecialType.System_Void
        or SpecialType.System_Boolean
        or SpecialType.System_SByte
        or SpecialType.System_Byte
        or SpecialType.System_Int16
        or SpecialType.System_UInt16
        or SpecialType.System_Int32
        or SpecialType.System_UInt32
        or SpecialType.System_Int64
        or SpecialType.System_UInt64
        or SpecialType.System_Char
        or SpecialType.System_Single
        or SpecialType.System_Double
        or SpecialType.System_Decimal
        or SpecialType.System_String;

    private static string? FrameworkTypeToken(ITypeSymbol type)
    {
        return type.MetadataName switch
        {
            "DateOnly" when IsExactSystemRuntimeType(type, "DateOnly") => "date",
            "TimeOnly" when IsExactSystemRuntimeType(type, "TimeOnly") => "time",
            "TimeSpan" when IsExactSystemRuntimeType(type, "TimeSpan") => "duration",
            "Guid" when IsExactSystemRuntimeType(type, "Guid") => "guid",
            "DayOfWeek" when IsExactSystemRuntimeType(type, "DayOfWeek") => "day_of_week",
            _ => null,
        };
    }

    private static bool IsExactSystemRuntimeType(ITypeSymbol type, string metadataName, int arity = 0) =>
        type is INamedTypeSymbol named
        && named.Arity == arity
        && string.Equals(named.MetadataName, metadataName, StringComparison.Ordinal)
        && named.DeclaringSyntaxReferences.IsEmpty
        && string.Equals(named.ContainingNamespace.ToDisplayString(), "System", StringComparison.Ordinal)
        && string.Equals(named.ContainingAssembly?.Identity.Name, "System.Runtime", StringComparison.Ordinal);

    private static ISymbol? ReferencedSymbol(SemanticModel model, SyntaxNode node)
    {
        if (node is not ExpressionSyntax
            && node is not TypeSyntax
            && node is not AttributeSyntax
            && node is not BaseTypeSyntax
            && node is not ConstructorInitializerSyntax)
        {
            return null;
        }

        SymbolInfo information = model.GetSymbolInfo(node, CancellationToken.None);
        return information.Symbol ?? information.CandidateSymbols.FirstOrDefault();
    }

    private static ITypeSymbol? ReferencedType(SemanticModel model, SyntaxNode node)
    {
        if (node is not ExpressionSyntax && node is not TypeSyntax)
        {
            return null;
        }

        TypeInfo information = model.GetTypeInfo(node, CancellationToken.None);
        return information.ConvertedType ?? information.Type;
    }

    private static bool IsMpkSymbol(ISymbol? symbol)
    {
        if (symbol is null)
        {
            return false;
        }

        string namespaceName = symbol is INamespaceSymbol namespaceSymbol
            ? namespaceSymbol.ToDisplayString()
            : symbol.ContainingNamespace?.ToDisplayString() ?? string.Empty;
        return IsMpkAssemblyName(symbol.ContainingAssembly?.Identity.Name ?? string.Empty)
            || IsMpkNamespaceName(namespaceName);
    }

    private static bool IsMpkDependencyName(
        SemanticModel model,
        SyntaxNode node,
        string sourceSpelling,
        Compilation compilation)
    {
        if (!IsMpkNamespaceName(sourceSpelling))
        {
            return false;
        }

        SymbolInfo information = model.GetSymbolInfo(node, CancellationToken.None);
        if (information.Symbol is not null)
        {
            return IsMpkDependencySymbol(information.Symbol, compilation);
        }

        if (!information.CandidateSymbols.IsEmpty)
        {
            return information.CandidateSymbols.Any(symbol =>
                IsMpkDependencySymbol(symbol, compilation));
        }

        // Preserve dependency-phase precedence for an unresolved Mpk or
        // Mpk.* spelling, without reserving that spelling for a resolved
        // application-owned local, parameter, member, or type.
        return true;
    }

    private static bool IsMpkDependencySymbol(ISymbol? symbol, Compilation compilation) =>
        symbol is not null
        && !IsSourceSymbol(symbol, compilation)
        && IsMpkSymbol(symbol);

    private static bool IsMpkNamespaceName(string value) =>
        string.Equals(value, "Mpk", StringComparison.OrdinalIgnoreCase)
        || value.StartsWith("Mpk.", StringComparison.OrdinalIgnoreCase);

    private static bool IsMpkAssemblyName(string value) =>
        IsMpkNamespaceName(value)
        || value.StartsWith("Mpk-", StringComparison.OrdinalIgnoreCase)
        || value.StartsWith("mpk_", StringComparison.OrdinalIgnoreCase);

    private static bool IsReflectionOrCodeGeneration(ISymbol? symbol)
    {
        if (symbol is IMethodSymbol method
            && string.Equals(method.Name, "GetType", StringComparison.Ordinal)
            && method.Parameters.Length == 0
            && method.ContainingType.SpecialType == SpecialType.System_Object)
        {
            return true;
        }

        string name = QualifiedOwner(symbol);
        return HasPrefix(name, ReflectionAndCodeGenerationPrefixes)
            || symbol?.ContainingAssembly?.Identity.Name.Contains("CodeAnalysis", StringComparison.Ordinal) == true;
    }

    private static bool IsEffectOrConcurrency(ISymbol? symbol)
    {
        string name = QualifiedOwner(symbol);
        if (HasPrefix(name, EffectAndConcurrencyPrefixes))
        {
            return true;
        }

        return name.StartsWith("System.DateTime.Now", StringComparison.Ordinal)
            || name.StartsWith("System.DateTime.UtcNow", StringComparison.Ordinal)
            || name.StartsWith("System.DateTime.Today", StringComparison.Ordinal)
            || name.StartsWith("System.DateTimeOffset.Now", StringComparison.Ordinal)
            || name.StartsWith("System.DateTimeOffset.UtcNow", StringComparison.Ordinal)
            || name.StartsWith("System.TimeProvider", StringComparison.Ordinal)
            || name.StartsWith("System.Guid.NewGuid", StringComparison.Ordinal)
            || name.StartsWith("System.GC", StringComparison.Ordinal);
    }

    private static string QualifiedOwner(ISymbol? symbol)
    {
        if (symbol is null)
        {
            return string.Empty;
        }

        ISymbol owner = symbol is INamedTypeSymbol ? symbol : symbol.ContainingType ?? symbol;
        string value = owner.ToDisplayString(SymbolDisplayFormat.CSharpErrorMessageFormat);
        return symbol is INamedTypeSymbol ? value : value + "." + symbol.Name;
    }

    private static bool HasPrefix(string value, IEnumerable<string> prefixes)
    {
        foreach (string prefix in prefixes)
        {
            if (string.Equals(value, prefix, StringComparison.Ordinal)
                || value.StartsWith(prefix + ".", StringComparison.Ordinal)
                || value.StartsWith(prefix + "<", StringComparison.Ordinal))
            {
                return true;
            }
        }

        return false;
    }

    private static bool IsDelegateType(ITypeSymbol? type) => type?.TypeKind == TypeKind.Delegate;

    private static bool RequiresGenericDiagnostic(ISymbol? symbol) => symbol switch
    {
        IMethodSymbol method => method.IsGenericMethod
            || method.OriginalDefinition.IsGenericMethod
            || method.ContainingType.IsGenericType && !IsExactNullable(method.ContainingType),
        IPropertySymbol property => property.ContainingType.IsGenericType
            && !IsExactNullable(property.ContainingType),
        IFieldSymbol field => field.ContainingType.IsGenericType
            && !IsExactNullable(field.ContainingType),
        INamedTypeSymbol type => type.IsGenericType && !IsExactNullable(type),
        _ => false,
    };

    private static bool IsExactNullable(INamedTypeSymbol type)
    {
        INamedTypeSymbol definition = type.OriginalDefinition;
        return IsExactSystemRuntimeType(definition, "Nullable`1", arity: 1)
            && definition.TypeKind == TypeKind.Struct
            && definition.TypeParameters.Length == 1
            && definition.TypeParameters[0].HasValueTypeConstraint;
    }

    private static bool IsAdmittedNullablePayload(ITypeSymbol type)
    {
        if (type.ContainingAssembly is not null && !type.DeclaringSyntaxReferences.IsEmpty)
        {
            return type.TypeKind is TypeKind.Struct or TypeKind.Enum;
        }

        if (type.SpecialType is SpecialType.System_Boolean
            or SpecialType.System_SByte
            or SpecialType.System_Byte
            or SpecialType.System_Int16
            or SpecialType.System_UInt16
            or SpecialType.System_Int32
            or SpecialType.System_UInt32
            or SpecialType.System_Int64
            or SpecialType.System_UInt64
            or SpecialType.System_Char
            or SpecialType.System_Single
            or SpecialType.System_Double
            or SpecialType.System_Decimal)
        {
            return true;
        }

        return FrameworkTypeToken(type) is not null;
    }

    private static bool IsSourceSymbol(ISymbol symbol, Compilation compilation) =>
        SymbolEqualityComparer.Default.Equals(symbol.ContainingAssembly, compilation.Assembly);

    private static bool IsAllowlistedFrameworkMember(IMethodSymbol method)
    {
        if (IsAllowlistedFloatingMember(method)) { return true; }

        if (method.ContainingType.SpecialType == SpecialType.System_String)
        {
            return IsAllowlistedStringMember(method);
        }

        if (IsExactNullable(method.ContainingType))
        {
            return method.Name == "GetValueOrDefault"
                && !method.IsStatic
                && method.ReturnType.Equals(method.ContainingType.TypeArguments[0], SymbolEqualityComparer.Default)
                && (method.Parameters.Length == 0
                    || (method.Parameters.Length == 1
                        && method.Parameters[0].Type.Equals(
                            method.ContainingType.TypeArguments[0],
                            SymbolEqualityComparer.Default)));
        }

        if (IsExactSystemRuntimeType(method.ContainingType, "Guid"))
        {
            return method.Name is "op_Equality" or "op_Inequality"
                && method.IsStatic
                && method.ReturnType.SpecialType == SpecialType.System_Boolean
                && method.Parameters.Length == 2
                && method.Parameters.All(parameter =>
                    IsExactSystemRuntimeType(parameter.Type, "Guid"));
        }

        return method.ContainingType.SpecialType == SpecialType.System_Decimal
            && IsAllowlistedDecimalMember(method);
    }

    private static bool HasExactIntrinsicArguments(
        IMethodSymbol method,
        SyntaxNode syntax,
        SemanticModel model)
    {
        IParameterSymbol[] intrinsicParameters = method.Parameters
            .Where(parameter => IsIntrinsicConstantCarrier(parameter.Type))
            .ToArray();
        if (intrinsicParameters.Length == 0)
        {
            return true;
        }

        // A parenthesized invocation can itself resolve to the method symbol.
        // Find the actual call before checking its exact intrinsic arguments.
        while (syntax is ParenthesizedExpressionSyntax parentheses) { syntax = parentheses.Expression; }
        InvocationExpressionSyntax? invocation = syntax.AncestorsAndSelf()
            .OfType<InvocationExpressionSyntax>()
            .FirstOrDefault(candidate =>
                model.GetSymbolInfo(candidate, CancellationToken.None).Symbol is IMethodSymbol candidateMethod
                && SymbolEqualityComparer.Default.Equals(candidateMethod, method));
        if (invocation is null
            || model.GetOperation(invocation, CancellationToken.None) is not IInvocationOperation operation)
        {
            return false;
        }

        foreach (IParameterSymbol parameter in intrinsicParameters)
        {
            IArgumentOperation? argument = operation.Arguments.FirstOrDefault(candidate =>
                candidate.Parameter is not null
                && SymbolEqualityComparer.Default.Equals(candidate.Parameter, parameter));
            if (argument is null
                || !IsExactIntrinsicFieldValue(argument.Value, out IFieldSymbol? field)
                || field is null
                || !IsAllowlistedIntrinsicConstant(field))
            {
                return false;
            }
        }

        return true;
    }

    private static bool IsAllowlistedStringMember(IMethodSymbol method)
    {
        bool ReturnsString() => method.ReturnType.SpecialType == SpecialType.System_String;
        bool ReturnsBoolean() => method.ReturnType.SpecialType == SpecialType.System_Boolean;
        bool IsString(int index) => method.Parameters[index].Type.SpecialType == SpecialType.System_String;
        bool IsInt32(int index) => method.Parameters[index].Type.SpecialType == SpecialType.System_Int32;
        bool IsOrdinal(int index) => IsExactSystemRuntimeType(
            method.Parameters[index].Type,
            "StringComparison");

        return method.Name switch
        {
            "Equals" when method.IsStatic =>
                ReturnsBoolean()
                && (method.Parameters.Length == 2
                    && IsString(0)
                    && IsString(1)
                    || method.Parameters.Length == 3
                    && IsString(0)
                    && IsString(1)
                    && IsOrdinal(2)),
            "Equals" =>
                ReturnsBoolean()
                && method.Parameters.Length == 2
                && IsString(0)
                && IsOrdinal(1),
            "Compare" =>
                method.IsStatic
                && method.ReturnType.SpecialType == SpecialType.System_Int32
                && method.Parameters.Length == 3
                && IsString(0)
                && IsString(1)
                && IsOrdinal(2),
            "Concat" =>
                method.IsStatic
                && ReturnsString()
                && method.Parameters.Length is >= 2 and <= 4
                && method.Parameters.All(parameter =>
                    parameter.Type.SpecialType == SpecialType.System_String),
            "IsNullOrEmpty" =>
                method.IsStatic
                && ReturnsBoolean()
                && method.Parameters.Length == 1
                && IsString(0),
            "Contains" or "StartsWith" or "EndsWith" =>
                !method.IsStatic
                && ReturnsBoolean()
                && method.Parameters.Length == 2
                && IsString(0)
                && IsOrdinal(1),
            "Substring" =>
                !method.IsStatic
                && ReturnsString()
                && method.Parameters.Length == 2
                && IsInt32(0)
                && IsInt32(1),
            _ => false,
        };
    }

    private static bool IsAllowlistedFloatingMember(IMethodSymbol method)
    {
        if (!method.IsStatic || method.IsGenericMethod) { return false; }
        SpecialType scalar = method.ContainingType.SpecialType;
        if (scalar is SpecialType.System_Single or SpecialType.System_Double)
        {
            return method.Name is "IsNaN" or "IsInfinity" or "IsFinite"
                && method.ReturnType.SpecialType == SpecialType.System_Boolean
                && method.Parameters.Length == 1
                && method.Parameters[0].Type.SpecialType == scalar;
        }
        scalar = IsExactSystemRuntimeType(method.ContainingType, "MathF")
            ? SpecialType.System_Single : IsExactSystemRuntimeType(method.ContainingType, "Math")
                ? SpecialType.System_Double : SpecialType.None;
        return scalar != SpecialType.None
            && method.ReturnType.SpecialType == scalar
            && (method.Name == "Abs" && method.Parameters.Length == 1
                || method.Name is "Min" or "Max" && method.Parameters.Length == 2)
            && method.Parameters.All(p => p.Type.SpecialType == scalar);
    }

    private static bool IsAllowlistedDecimalMember(IMethodSymbol method)
    {
        if (!method.IsStatic
            || method.ReturnType.SpecialType != SpecialType.System_Decimal
            || method.Parameters.Length == 0
            || method.Parameters[0].Type.SpecialType != SpecialType.System_Decimal)
        {
            return false;
        }

        if (method.Name is "Truncate" or "Floor" or "Ceiling")
        {
            return method.Parameters.Length == 1;
        }

        if (method.Name != "Round")
        {
            return false;
        }

        return method.Parameters.Length == 1
            || method.Parameters.Length == 2
                && (method.Parameters[1].Type.SpecialType == SpecialType.System_Int32
                    || IsExactSystemRuntimeType(method.Parameters[1].Type, "MidpointRounding"))
            || method.Parameters.Length == 3
                && method.Parameters[1].Type.SpecialType == SpecialType.System_Int32
                && IsExactSystemRuntimeType(method.Parameters[2].Type, "MidpointRounding");
    }

    private static bool IsAllowlistedFrameworkProperty(IPropertySymbol property)
    {
        if (IsExactNullable(property.ContainingType))
        {
            return property.Name is "HasValue" or "Value";
        }

        if (property.ContainingType.SpecialType == SpecialType.System_String && property.IsIndexer)
        {
            return !property.IsStatic && property.Type.SpecialType == SpecialType.System_Char
                && property.Parameters.Length == 1
                && property.Parameters[0].Type.SpecialType == SpecialType.System_Int32;
        }
        return property.Name == "Length"
            && !property.IsStatic
            && property.Type.SpecialType == SpecialType.System_Int32
            && property.Parameters.Length == 0
            && (property.ContainingType.SpecialType == SpecialType.System_String
                || property.ContainingType.SpecialType == SpecialType.System_Array);
    }

    private static bool IsAllowlistedFrameworkField(
        IFieldSymbol field,
        SyntaxNode syntax,
        SemanticModel model)
    {
        if (IsExactSystemRuntimeType(field.ContainingType, "StringComparison"))
        {
            return IsAllowlistedIntrinsicConstant(field)
                && IsExactIntrinsicArgument(field, syntax, model, IsAllowlistedStringMember);
        }

        if (IsExactSystemRuntimeType(field.ContainingType, "MidpointRounding"))
        {
            return IsAllowlistedIntrinsicConstant(field)
                && IsExactIntrinsicArgument(field, syntax, model, IsAllowlistedDecimalMember);
        }

        if (IsExactSystemRuntimeType(field.ContainingType, "DayOfWeek"))
        {
            return field.Name is "Sunday" or "Monday" or "Tuesday" or "Wednesday"
                or "Thursday" or "Friday" or "Saturday";
        }

        return IsExactSystemRuntimeType(field.ContainingType, "Guid")
            && field.Name == "Empty";
    }

    private static bool IsAllowlistedIntrinsicConstant(IFieldSymbol field) =>
        (IsExactSystemRuntimeType(field.ContainingType, "StringComparison")
            && field.Name == "Ordinal")
        || (IsExactSystemRuntimeType(field.ContainingType, "MidpointRounding")
            && (field.Name is "ToEven" or "AwayFromZero" or "ToZero"
                or "ToNegativeInfinity" or "ToPositiveInfinity"));

    private static bool IsExactIntrinsicArgument(
        IFieldSymbol field,
        SyntaxNode syntax,
        SemanticModel model,
        Func<IMethodSymbol, bool> allowlistedMethod)
    {
        ArgumentSyntax? argument = syntax.AncestorsAndSelf().OfType<ArgumentSyntax>().FirstOrDefault();
        if (argument is null
            || model.GetOperation(argument, CancellationToken.None) is not IArgumentOperation operation
            || operation.Parameter is null
            || !SymbolEqualityComparer.Default.Equals(operation.Parameter.Type, field.ContainingType)
            || argument.Parent?.Parent is not InvocationExpressionSyntax invocation
            || model.GetSymbolInfo(invocation, CancellationToken.None).Symbol is not IMethodSymbol method)
        {
            return false;
        }

        return IsExactIntrinsicFieldValue(operation.Value, out IFieldSymbol? referenced)
            && SymbolEqualityComparer.Default.Equals(referenced, field)
            && allowlistedMethod(method);
    }

    private static bool IsExactIntrinsicFieldValue(
        IOperation operation,
        out IFieldSymbol? field)
    {
        IOperation value = operation;
        while (value is IConversionOperation conversion
            && conversion.IsImplicit
            && !conversion.Conversion.IsUserDefined)
        {
            value = conversion.Operand;
        }

        field = (value as IFieldReferenceOperation)?.Field;
        return field is not null;
    }

    private static bool IsExactFileWideNullableEnable(DirectiveTriviaSyntax directive, SyntaxNode root)
    {
        if (directive is not NullableDirectiveTriviaSyntax nullable
            || !nullable.IsActive
            || !nullable.SettingToken.IsKind(SyntaxKind.EnableKeyword)
            || nullable.TargetToken.Kind() != SyntaxKind.None)
        {
            return false;
        }

        SyntaxToken firstToken = root.GetFirstToken(includeZeroWidth: true);
        return nullable.Span.End <= firstToken.SpanStart;
    }

    private static void ValidateDirectives(SyntaxNode root)
    {
        foreach (SyntaxTrivia trivia in root.DescendantTrivia(descendIntoTrivia: true))
        {
            if (trivia.GetStructure() is DirectiveTriviaSyntax directive
                && !IsExactFileWideNullableEnable(directive, root))
            {
                throw PracticalFailures.Declaration("source_directive");
            }
        }
    }

    private static bool HasPartialModifier(MemberDeclarationSyntax member) => member switch
    {
        BaseTypeDeclarationSyntax type => type.Modifiers.Any(SyntaxKind.PartialKeyword),
        MethodDeclarationSyntax method => method.Modifiers.Any(SyntaxKind.PartialKeyword),
        _ => false,
    };

    private static string InputKindCode(PracticalCapturedInputKind kind) => kind switch
    {
        PracticalCapturedInputKind.Project => "project_input",
        PracticalCapturedInputKind.Package => "package_input",
        PracticalCapturedInputKind.Binary => "binary_input",
        PracticalCapturedInputKind.GeneratedSource => "generated_source",
        PracticalCapturedInputKind.AnalyzerConfig => "analyzer_config",
        PracticalCapturedInputKind.EditorConfig => "editor_config",
        _ => "input_inventory",
    };

    private static void ValidateCount(int count, int minimum, int maximum, string code)
    {
        if (count < minimum)
        {
            throw PracticalFailures.Protocol("selection_shape");
        }

        if (count > maximum)
        {
            throw PracticalFailures.Limit(code);
        }
    }

    private static void ValidateSortedPaths(IReadOnlyList<string> paths, string? prefix, string suffix)
    {
        string? previous = null;
        foreach (string path in paths)
        {
            if (!IsPortablePath(path)
                || (prefix is not null && !path.StartsWith(prefix, StringComparison.Ordinal))
                || !path.EndsWith(suffix, StringComparison.Ordinal)
                || (previous is not null && string.CompareOrdinal(previous, path) >= 0))
            {
                throw PracticalFailures.Protocol("selection_path");
            }

            previous = path;
        }
    }

    private static void ValidateSortedIds(IReadOnlyList<string> ids)
    {
        string? previous = null;
        foreach (string id in ids)
        {
            if (!PracticalIdentity.IsSourceId(id)
                || (previous is not null && string.CompareOrdinal(previous, id) >= 0))
            {
                throw PracticalFailures.Protocol("selected_root_id");
            }

            previous = id;
        }
    }

    private static bool IsPortablePath(string path)
    {
        if (string.IsNullOrEmpty(path)
            || path.Length > 1_024
            || path[0] == '/'
            || path[^1] == '/'
            || path.Contains('\\', StringComparison.Ordinal)
            || path.Contains(':', StringComparison.Ordinal)
            || !path.All(character => character <= 0x7f))
        {
            return false;
        }

        foreach (string component in path.Split('/'))
        {
            if (component.Length == 0
                || component.Length > 255
                || component is "." or ".."
                || component[^1] == '.'
                || IsWindowsDeviceName(component)
                || component.Any(character =>
                    !char.IsAsciiLetterOrDigit(character)
                    && character is not '.' and not '_' and not '-'))
            {
                return false;
            }
        }

        return true;
    }

    private static bool IsWindowsDeviceName(string component)
    {
        string stem = component.Split('.')[0].ToUpperInvariant();
        if (stem is "CON" or "PRN" or "AUX" or "NUL" or "CLOCK$")
        {
            return true;
        }

        return stem.Length == 4
            && (stem.StartsWith("COM", StringComparison.Ordinal)
                || stem.StartsWith("LPT", StringComparison.Ordinal))
            && stem[3] >= '1'
            && stem[3] <= '9';
    }

    private static bool IsCompilationId(string value)
    {
        if (string.IsNullOrEmpty(value)
            || value.Length > 64
            || value[0] < 'a'
            || value[0] > 'z')
        {
            return false;
        }

        bool separator = false;
        for (int index = 1; index < value.Length; index++)
        {
            char character = value[index];
            if ((character >= 'a' && character <= 'z') || char.IsAsciiDigit(character))
            {
                separator = false;
            }
            else if ((character is '.' or '_' or '-') && !separator)
            {
                separator = true;
            }
            else
            {
                return false;
            }
        }

        return !separator;
    }

    private static int CheckedAdd(int current, int increment, int maximum, string code)
    {
        int candidate;
        try
        {
            candidate = checked(current + increment);
        }
        catch (OverflowException)
        {
            throw PracticalFailures.Limit(code);
        }

        if (candidate > maximum)
        {
            throw PracticalFailures.Limit(code);
        }

        return candidate;
    }

    private sealed class PracticalCaptureSet
    {
        internal PracticalCaptureSet(
            PracticalSourceFile[] sources,
            PracticalSidecarFile[] sidecars)
        {
            Sources = sources;
            Sidecars = sidecars;
        }

        internal PracticalSourceFile[] Sources { get; }

        internal PracticalSidecarFile[] Sidecars { get; }
    }

    private sealed class RoslynState
    {
        private readonly HashSet<SyntaxTree> treeSet;

        internal RoslynState(CSharpCompilation compilation, ImmutableArray<SyntaxTree> trees)
        {
            Compilation = compilation;
            Trees = trees;
            treeSet = new HashSet<SyntaxTree>(trees);
        }

        internal CSharpCompilation Compilation { get; }

        internal ImmutableArray<SyntaxTree> Trees { get; }

        internal bool ContainsTree(SyntaxTree tree) => treeSet.Contains(tree);
    }

    private sealed class PracticalReferenceRecord
    {
        internal PracticalReferenceRecord(
            string path,
            long sizeBytes,
            string sha256,
            PortableExecutableReference reference)
        {
            Path = path;
            SizeBytes = sizeBytes;
            Sha256 = sha256;
            Reference = reference;
        }

        internal string Path { get; }

        internal long SizeBytes { get; }

        internal string Sha256 { get; }

        internal PortableExecutableReference Reference { get; }
    }

    private sealed class SourceTypeRecord
    {
        internal SourceTypeRecord(
            BaseTypeDeclarationSyntax syntax,
            INamedTypeSymbol symbol,
            SemanticModel model,
            int sourceOrdinal)
        {
            Syntax = syntax;
            Symbol = symbol;
            Model = model;
            SourceOrdinal = sourceOrdinal;
        }

        internal BaseTypeDeclarationSyntax Syntax { get; }

        internal INamedTypeSymbol Symbol { get; }

        internal SemanticModel Model { get; }

        internal int SourceOrdinal { get; }

        internal string Id { get; set; } = string.Empty;

        internal bool IsDataOrException => !Symbol.IsStatic;
    }

    private sealed class SourceCallableRecord
    {
        private readonly HashSet<SourceCallableRecord> calleeSet = new HashSet<SourceCallableRecord>();
        private readonly HashSet<SourceTypeRecord> bodyTypeSet = new HashSet<SourceTypeRecord>();

        internal SourceCallableRecord(
            SyntaxNode syntax,
            IMethodSymbol symbol,
            SemanticModel model,
            SourceTypeRecord owner,
            PracticalDeclarationKind kind,
            string identityName)
        {
            Syntax = syntax;
            Symbol = symbol;
            Model = model;
            Owner = owner;
            Kind = kind;
            IdentityName = identityName;
        }

        internal SyntaxNode Syntax { get; }

        internal IMethodSymbol Symbol { get; }

        internal SemanticModel Model { get; }

        internal SourceTypeRecord Owner { get; }

        internal PracticalDeclarationKind Kind { get; }

        internal string IdentityName { get; }

        internal string Id { get; set; } = string.Empty;

        internal List<SourceCallableRecord> Callees { get; } = new List<SourceCallableRecord>();

        internal List<SourceTypeRecord> BodyTypes { get; } = new List<SourceTypeRecord>();

        internal void AddCallee(SourceCallableRecord callee)
        {
            if (calleeSet.Add(callee))
            {
                Callees.Add(callee);
            }
        }

        internal void AddBodyType(SourceTypeRecord type)
        {
            if (bodyTypeSet.Add(type))
            {
                BodyTypes.Add(type);
            }
        }
    }

    private sealed class PracticalDeclarationModel
    {
        private readonly RoslynState state;
        private readonly PracticalSourceFile[] sources;
        private readonly List<SourceTypeRecord> types = new List<SourceTypeRecord>();
        private readonly List<SourceCallableRecord> callables = new List<SourceCallableRecord>();
        private readonly List<PracticalDeclaration> storedDeclarations = new List<PracticalDeclaration>();
        private readonly List<PracticalGraphEdge> typeEdges = new List<PracticalGraphEdge>();
        private readonly Dictionary<ISymbol, SourceTypeRecord> typesBySymbol =
            new Dictionary<ISymbol, SourceTypeRecord>(SymbolEqualityComparer.Default);
        private readonly Dictionary<ISymbol, SourceCallableRecord> callablesBySymbol =
            new Dictionary<ISymbol, SourceCallableRecord>(SymbolEqualityComparer.Default);
        private readonly Dictionary<string, SourceCallableRecord> callablesById =
            new Dictionary<string, SourceCallableRecord>(StringComparer.Ordinal);
        private readonly Dictionary<SourceTypeRecord, List<SourceTypeRecord>> typeTargetsBySource =
            new Dictionary<SourceTypeRecord, List<SourceTypeRecord>>();
        private readonly Dictionary<SourceTypeRecord, List<SourceCallableRecord>> propertiesByOwner =
            new Dictionary<SourceTypeRecord, List<SourceCallableRecord>>();
        private readonly HashSet<(string Source, string Target)> typeEdgeKeys =
            new HashSet<(string Source, string Target)>();

        internal PracticalDeclarationModel(RoslynState state, PracticalSourceFile[] sources)
        {
            this.state = state;
            this.sources = sources;
        }

        internal void Collect()
        {
            for (int ordinal = 0; ordinal < state.Trees.Length; ordinal++)
            {
                SyntaxTree tree = state.Trees[ordinal];
                SemanticModel semanticModel = state.Compilation.GetSemanticModel(tree, false);
                SyntaxNode root = tree.GetRoot(CancellationToken.None);
                foreach (BaseTypeDeclarationSyntax declaration in root.DescendantNodes().OfType<BaseTypeDeclarationSyntax>())
                {
                    if (semanticModel.GetDeclaredSymbol(declaration, CancellationToken.None) is not INamedTypeSymbol symbol)
                    {
                        throw PracticalFailures.Declaration("declared_symbol");
                    }

                    PracticalIdentity.ValidateNamespace(symbol.ContainingNamespace.ToDisplayString());
                    PracticalIdentity.ValidateIdentifier(symbol.Name);
                    if (typesBySymbol.ContainsKey(symbol))
                    {
                        throw PracticalFailures.Declaration("partial_declaration");
                    }

                    var retained = new SourceTypeRecord(declaration, symbol, semanticModel, ordinal);
                    types.Add(retained);
                    typesBySymbol.Add(symbol, retained);
                }
            }

            foreach (SourceTypeRecord owner in types)
            {
                foreach (MemberDeclarationSyntax member in Members(owner.Syntax))
                {
                    switch (member)
                    {
                        case MethodDeclarationSyntax method:
                            AddCallable(owner, method, owner.Model.GetDeclaredSymbol(method), PracticalDeclarationKind.Method, method.Identifier.ValueText);
                            break;
                        case ConstructorDeclarationSyntax constructor:
                            AddCallable(owner, constructor, owner.Model.GetDeclaredSymbol(constructor), PracticalDeclarationKind.Constructor, owner.Symbol.Name);
                            break;
                        case PropertyDeclarationSyntax property:
                            AddProperty(owner, property);
                            break;
                        case FieldDeclarationSyntax field:
                            foreach (VariableDeclaratorSyntax variable in field.Declaration.Variables)
                            {
                                if (owner.Model.GetDeclaredSymbol(variable, CancellationToken.None) is not IFieldSymbol symbol)
                                {
                                    throw PracticalFailures.Declaration("declared_symbol");
                                }

                                storedDeclarations.Add(StoredDeclaration(owner, symbol, PracticalDeclarationKind.Field, variable.Span));
                            }
                            break;
                        case EnumMemberDeclarationSyntax enumMember:
                            if (owner.Model.GetDeclaredSymbol(enumMember, CancellationToken.None) is not IFieldSymbol enumSymbol)
                            {
                                throw PracticalFailures.Declaration("declared_symbol");
                            }

                            storedDeclarations.Add(StoredDeclaration(
                                owner,
                                enumSymbol,
                                PracticalDeclarationKind.EnumMember,
                                enumMember.Span));
                            break;
                        default:
                            throw PracticalFailures.Declaration("unsupported_declaration");
                    }
                }
            }
        }

        internal void ResolveIdentities()
        {
            foreach (SourceTypeRecord type in types)
            {
                type.Id = PracticalIdentity.SourceTypeId(
                    type.Symbol.ContainingNamespace.ToDisplayString(),
                    type.Symbol.Name);
            }

            foreach (SourceCallableRecord callable in callables)
            {
                var parameters = new List<string>();
                foreach (IParameterSymbol parameter in callable.Symbol.Parameters)
                {
                    parameters.Add(TypeId(parameter.Type));
                }

                string result = callable.Kind == PracticalDeclarationKind.Constructor
                    ? callable.Owner.Id
                    : TypeId(callable.Symbol.ReturnType);
                callable.Id = PracticalIdentity.CallableId(
                    callable.Kind == PracticalDeclarationKind.Constructor ? "constructor" : "method",
                    callable.Owner.Symbol.ContainingNamespace.ToDisplayString(),
                    callable.Owner.Id,
                    callable.IdentityName,
                    parameters,
                    result);
            }

            EnsureUniqueIdentities();
        }

        internal void ResolveEdges()
        {
            foreach (SourceTypeRecord type in types)
            {
                foreach (ITypeSymbol dependency in StructuralDependencies(type))
                {
                    SourceTypeRecord? target = FindType(Unwrap(dependency));
                    if (target is not null)
                    {
                        AddTypeEdge(type, target);
                    }
                }
            }

            foreach (SourceCallableRecord callable in callables)
            {
                foreach (SyntaxNode node in CallableBodyNodes(callable.Syntax, callable.Model))
                {
                    ISymbol? symbol = ReferencedSymbol(callable.Model, node);
                    if (node is InvocationExpressionSyntax
                            or ObjectCreationExpressionSyntax
                            or ImplicitObjectCreationExpressionSyntax
                            or ConstructorInitializerSyntax
                        && symbol is IMethodSymbol method)
                    {
                        SourceCallableRecord? callee = FindCallable(method);
                        if (callee is not null)
                        {
                            callable.AddCallee(callee);
                        }
                    }
                    else if (symbol is IPropertySymbol property && property.GetMethod is not null)
                    {
                        SourceCallableRecord? getter = FindCallable(property.GetMethod);
                        if (getter is not null)
                        {
                            callable.AddCallee(getter);
                        }
                    }

                    ITypeSymbol? usedType = ReferencedType(callable.Model, node);
                    SourceTypeRecord? used = FindType(Unwrap(usedType));
                    if (used is not null)
                    {
                        callable.AddBodyType(used);
                    }
                }

                callable.Callees.Sort((left, right) => string.CompareOrdinal(left.Id, right.Id));
            }

        }

        internal PracticalSourceClosure Close(
            PracticalSourceSelection selection,
            PracticalSidecarFile[] sidecars)
        {
            var roots = new List<SourceCallableRecord>();
            foreach (string rootId in selection.SelectedRootIds)
            {
                if (!callablesById.TryGetValue(rootId, out SourceCallableRecord? root))
                {
                    throw PracticalFailures.Declaration("selected_root_missing");
                }

                if (root.Kind is not PracticalDeclarationKind.Method
                        and not PracticalDeclarationKind.Constructor
                    || root.Kind == PracticalDeclarationKind.Constructor
                        && root.Symbol.MethodKind != MethodKind.Constructor)
                {
                    throw PracticalFailures.Declaration("selected_root_kind");
                }

                roots.Add(root);
            }

            var reachableCallables = new HashSet<SourceCallableRecord>();
            var reachableTypes = new HashSet<SourceTypeRecord>();
            var callEdges = new List<PracticalGraphEdge>();
            var callEdgeKeys = new HashSet<(string Source, string Target)>();
            foreach (SourceCallableRecord root in roots)
            {
                VisitCallable(
                    root,
                    reachableCallables,
                    reachableTypes,
                    callEdges,
                    callEdgeKeys,
                    onTypeReached: null);
            }

            ExpandTypeClosure(reachableTypes, reachableCallables, callEdges, callEdgeKeys);
            int sourceDataExceptionTypeCount = 0;
            foreach (SourceTypeRecord type in reachableTypes.Where(type => type.IsDataOrException))
            {
                sourceDataExceptionTypeCount = RetainSourceDataExceptionType(
                    sourceDataExceptionTypeCount);
            }

            RejectCallCycles(reachableCallables);
            if (reachableCallables.Count != callables.Count || reachableTypes.Count != types.Count)
            {
                throw PracticalFailures.Declaration("dead_declaration");
            }

            RejectTypeCycles();

            var all = new List<PracticalDeclaration>();
            foreach (SourceTypeRecord type in types)
            {
                all.Add(Declaration(type));
            }
            foreach (SourceCallableRecord callable in callables)
            {
                all.Add(Declaration(callable));
            }
            all.AddRange(storedDeclarations);
            all.Sort(CompareDeclaration);

            PracticalDeclaration[] reachable = all.ToArray();
            callEdges.Sort(CompareEdge);
            typeEdges.Sort(CompareEdge);
            return new PracticalSourceClosure(
                sources,
                sidecars,
                all.ToArray(),
                reachable,
                callEdges.ToArray(),
                typeEdges.ToArray(),
                sourceDataExceptionTypeCount);
        }

        private void AddCallable(
            SourceTypeRecord owner,
            SyntaxNode syntax,
            IMethodSymbol? symbol,
            PracticalDeclarationKind kind,
            string identityName)
        {
            IMethodSymbol admitted = symbol
                ?? throw PracticalFailures.Declaration("callable_shape");
            if (admitted.IsAbstract
                || admitted.IsExtern
                || admitted.ExplicitInterfaceImplementations.Length != 0)
            {
                throw PracticalFailures.Declaration("callable_shape");
            }

            PracticalIdentity.ValidateIdentifier(identityName);
            var callable = new SourceCallableRecord(
                syntax,
                admitted,
                owner.Model,
                owner,
                kind,
                identityName);
            if (!callablesBySymbol.TryAdd(admitted, callable))
            {
                throw PracticalFailures.Declaration("declaration_identity_collision");
            }

            callables.Add(callable);
            if (kind == PracticalDeclarationKind.Property)
            {
                if (!propertiesByOwner.TryGetValue(owner, out List<SourceCallableRecord>? properties))
                {
                    properties = new List<SourceCallableRecord>();
                    propertiesByOwner.Add(owner, properties);
                }

                properties.Add(callable);
            }
        }

        private void AddProperty(SourceTypeRecord owner, PropertyDeclarationSyntax property)
        {
            if (owner.Model.GetDeclaredSymbol(property, CancellationToken.None) is not IPropertySymbol symbol
                || symbol.SetMethod is { IsInitOnly: false })
            {
                throw PracticalFailures.Declaration("property_shape");
            }

            IMethodSymbol getter = symbol.GetMethod
                ?? throw PracticalFailures.Declaration("property_shape");

            storedDeclarations.Add(StoredDeclaration(owner, symbol, PracticalDeclarationKind.Property, property.Span));
            AddCallable(owner, property, getter, PracticalDeclarationKind.Property, "get_" + symbol.Name);
        }

        private PracticalDeclaration StoredDeclaration(
            SourceTypeRecord owner,
            ISymbol symbol,
            PracticalDeclarationKind kind,
            TextSpan span)
        {
            string id = owner.Symbol.ContainingNamespace.ToDisplayString()
                + "." + owner.Symbol.Name + "." + symbol.Name;
            return new PracticalDeclaration(
                id,
                kind,
                owner.SourceOrdinal,
                ByteOffset(owner.SourceOrdinal, span.Start),
                ByteOffset(owner.SourceOrdinal, span.End));
        }

        private PracticalDeclaration Declaration(SourceTypeRecord type) =>
            new PracticalDeclaration(
                type.Id,
                PracticalDeclarationKind.Type,
                type.SourceOrdinal,
                ByteOffset(type.SourceOrdinal, type.Syntax.SpanStart),
                ByteOffset(type.SourceOrdinal, type.Syntax.Span.End));

        private PracticalDeclaration Declaration(SourceCallableRecord callable) =>
            new PracticalDeclaration(
                callable.Id,
                callable.Kind,
                callable.Owner.SourceOrdinal,
                ByteOffset(callable.Owner.SourceOrdinal, callable.Syntax.SpanStart),
                ByteOffset(callable.Owner.SourceOrdinal, callable.Syntax.Span.End));

        private int ByteOffset(int ordinal, int characterOffset) =>
            sources[ordinal].ByteOffset(characterOffset);

        private static int RetainSourceDataExceptionType(int current)
        {
            // This is the one frozen source_data_exception_types retention
            // site: checked increment first, compare to the inclusive 128
            // ceiling, and only then let the caller retain the candidate.
            int candidate;
            try
            {
                candidate = checked(current + 1);
            }
            catch (OverflowException)
            {
                throw PracticalFailures.Limit("source_data_exception_types");
            }

            if (candidate > SourceDataExceptionTypesMaximum)
            {
                throw PracticalFailures.Limit("source_data_exception_types");
            }

            return candidate;
        }

        private void EnsureUniqueIdentities()
        {
            var ids = new HashSet<string>(StringComparer.Ordinal);
            foreach (string id in types.Select(type => type.Id))
            {
                if (!ids.Add(id))
                {
                    throw PracticalFailures.Declaration("declaration_identity_collision");
                }
            }

            foreach (SourceCallableRecord callable in callables)
            {
                if (!ids.Add(callable.Id)
                    || !callablesById.TryAdd(callable.Id, callable))
                {
                    throw PracticalFailures.Declaration("declaration_identity_collision");
                }
            }
        }

        private void VisitCallable(
            SourceCallableRecord callable,
            HashSet<SourceCallableRecord> reachableCallables,
            HashSet<SourceTypeRecord> reachableTypes,
            List<PracticalGraphEdge> callEdges,
            HashSet<(string Source, string Target)> callEdgeKeys,
            Action<SourceTypeRecord>? onTypeReached)
        {
            if (reachableCallables.Contains(callable))
            {
                return;
            }

            if (reachableCallables.Count >= MethodClosureMaximum)
            {
                throw PracticalFailures.Limit("method_closure");
            }

            reachableCallables.Add(callable);
            AddReachableType(reachableTypes, callable.Owner, onTypeReached);
            AddSignatureTypes(callable.Symbol, reachableTypes, onTypeReached);
            foreach (SourceTypeRecord bodyType in callable.BodyTypes)
            {
                AddReachableType(reachableTypes, bodyType, onTypeReached);
            }

            foreach (SourceCallableRecord callee in callable.Callees)
            {
                AddGraphEdge(callEdges, callEdgeKeys, callable.Id, callee.Id);
                VisitCallable(
                    callee,
                    reachableCallables,
                    reachableTypes,
                    callEdges,
                    callEdgeKeys,
                    onTypeReached);
            }
        }

        private void ExpandTypeClosure(
            HashSet<SourceTypeRecord> reachableTypes,
            HashSet<SourceCallableRecord> reachableCallables,
            List<PracticalGraphEdge> callEdges,
            HashSet<(string Source, string Target)> callEdgeKeys)
        {
            var pending = new Queue<SourceTypeRecord>(
                reachableTypes.OrderBy(type => type.Id, StringComparer.Ordinal));
            void Enqueue(SourceTypeRecord type) => pending.Enqueue(type);
            while (pending.Count != 0)
            {
                SourceTypeRecord type = pending.Dequeue();
                foreach (SourceTypeRecord target in TypeTargets(type))
                {
                    AddReachableType(reachableTypes, target, Enqueue);
                }

                if (propertiesByOwner.TryGetValue(
                    type,
                    out List<SourceCallableRecord>? properties))
                {
                    foreach (SourceCallableRecord property in properties)
                    {
                        VisitCallable(
                            property,
                            reachableCallables,
                            reachableTypes,
                            callEdges,
                            callEdgeKeys,
                            Enqueue);
                    }
                }
            }
        }

        private static void RejectCallCycles(IEnumerable<SourceCallableRecord> reachable)
        {
            var visited = new HashSet<SourceCallableRecord>();
            var visiting = new HashSet<SourceCallableRecord>();
            foreach (SourceCallableRecord callable in reachable)
            {
                VisitCall(callable, visited, visiting);
            }
        }

        private static void VisitCall(
            SourceCallableRecord callable,
            HashSet<SourceCallableRecord> visited,
            HashSet<SourceCallableRecord> visiting)
        {
            if (visiting.Contains(callable))
            {
                throw PracticalFailures.Declaration("call_cycle");
            }

            if (!visited.Add(callable))
            {
                return;
            }

            visiting.Add(callable);
            foreach (SourceCallableRecord callee in callable.Callees)
            {
                VisitCall(callee, visited, visiting);
            }
            visiting.Remove(callable);
        }

        private void AddSignatureTypes(
            IMethodSymbol method,
            HashSet<SourceTypeRecord> reachable,
            Action<SourceTypeRecord>? onTypeReached)
        {
            AddIfSource(method.ReturnType, reachable, onTypeReached);
            foreach (IParameterSymbol parameter in method.Parameters)
            {
                AddIfSource(parameter.Type, reachable, onTypeReached);
            }
        }

        private void AddIfSource(
            ITypeSymbol symbol,
            HashSet<SourceTypeRecord> reachable,
            Action<SourceTypeRecord>? onTypeReached)
        {
            SourceTypeRecord? found = FindType(Unwrap(symbol));
            if (found is not null)
            {
                AddReachableType(reachable, found, onTypeReached);
            }
        }

        private static void AddReachableType(
            HashSet<SourceTypeRecord> reachable,
            SourceTypeRecord type,
            Action<SourceTypeRecord>? onTypeReached)
        {
            if (reachable.Add(type))
            {
                onTypeReached?.Invoke(type);
            }
        }

        private IEnumerable<SourceTypeRecord> TypeTargets(SourceTypeRecord source) =>
            typeTargetsBySource.TryGetValue(source, out List<SourceTypeRecord>? targets)
                ? targets
                : Array.Empty<SourceTypeRecord>();

        private void RejectTypeCycles()
        {
            var visited = new HashSet<SourceTypeRecord>();
            var visiting = new HashSet<SourceTypeRecord>();
            foreach (SourceTypeRecord type in types)
            {
                VisitType(type, visited, visiting);
            }
        }

        private void VisitType(
            SourceTypeRecord type,
            HashSet<SourceTypeRecord> visited,
            HashSet<SourceTypeRecord> visiting)
        {
            if (visiting.Contains(type))
            {
                throw PracticalFailures.Type("type_cycle");
            }

            if (!visited.Add(type))
            {
                return;
            }

            visiting.Add(type);
            foreach (SourceTypeRecord target in TypeTargets(type))
            {
                VisitType(target, visited, visiting);
            }
            visiting.Remove(type);
        }

        private void AddTypeEdge(SourceTypeRecord source, SourceTypeRecord target)
        {
            if (!typeEdgeKeys.Add((source.Id, target.Id)))
            {
                return;
            }

            typeEdges.Add(new PracticalGraphEdge(source.Id, target.Id));
            if (!typeTargetsBySource.TryGetValue(source, out List<SourceTypeRecord>? targets))
            {
                targets = new List<SourceTypeRecord>();
                typeTargetsBySource.Add(source, targets);
            }

            targets.Add(target);
        }

        private static void AddGraphEdge(
            List<PracticalGraphEdge> edges,
            HashSet<(string Source, string Target)> keys,
            string source,
            string target)
        {
            if (!keys.Add((source, target)))
            {
                return;
            }

            edges.Add(new PracticalGraphEdge(source, target));
        }

        private IEnumerable<ITypeSymbol> StructuralDependencies(SourceTypeRecord type)
        {
            if (type.Symbol.BaseType is not null && IsSourceSymbol(type.Symbol.BaseType, state.Compilation))
            {
                yield return type.Symbol.BaseType;
            }

            foreach (ISymbol member in type.Symbol.GetMembers())
            {
                if (member.IsImplicitlyDeclared)
                {
                    continue;
                }

                if (member is IFieldSymbol field && !field.IsStatic)
                {
                    yield return field.Type;
                }
                else if (member is IPropertySymbol property && !property.IsStatic)
                {
                    yield return property.Type;
                }
            }
        }

        private string TypeId(ITypeSymbol symbol)
        {
            var wrappers = new List<string>();
            while (true)
            {
                if (symbol is IArrayTypeSymbol array)
                {
                    if (!array.IsSZArray || array.Rank != 1)
                    {
                        throw PracticalFailures.Type("closed_type");
                    }

                    wrappers.Add("bounded_sequence");
                    symbol = array.ElementType;
                    continue;
                }

                if (symbol is INamedTypeSymbol nullable && IsExactNullable(nullable))
                {
                    wrappers.Add("option");
                    symbol = nullable.TypeArguments[0];
                    continue;
                }

                break;
            }

            SourceTypeRecord? source = FindType(symbol);
            string id;
            if (source is not null)
            {
                id = source.Id;
            }
            else
            {
                string? token = symbol.SpecialType switch
                {
                    SpecialType.System_Void => "unit",
                    SpecialType.System_Boolean => "bool",
                    SpecialType.System_SByte => "i8",
                    SpecialType.System_Byte => "u8",
                    SpecialType.System_Int16 => "i16",
                    SpecialType.System_UInt16 => "u16",
                    SpecialType.System_Int32 => "i32",
                    SpecialType.System_UInt32 => "u32",
                    SpecialType.System_Int64 => "i64",
                    SpecialType.System_UInt64 => "u64",
                    SpecialType.System_Char => "char",
                    SpecialType.System_Single => "f32",
                    SpecialType.System_Double => "f64",
                    SpecialType.System_Decimal => "decimal",
                    SpecialType.System_String => "string",
                    _ => null,
                };
                string? frameworkToken = FrameworkTypeToken(symbol);
                if (token is null && frameworkToken is null)
                {
                    throw PracticalFailures.Type("closed_type");
                }

                id = PracticalIdentity.PrimitiveId(token ?? frameworkToken!);
            }

            for (int index = wrappers.Count - 1; index >= 0; index--)
            {
                id = PracticalIdentity.ClosedInstanceId(wrappers[index], id);
            }

            return id;
        }

        private SourceTypeRecord? FindType(ITypeSymbol? symbol)
        {
            if (symbol is null)
            {
                return null;
            }

            return typesBySymbol.TryGetValue(symbol, out SourceTypeRecord? type) ? type : null;
        }

        private SourceCallableRecord? FindCallable(IMethodSymbol symbol)
        {
            if (callablesBySymbol.TryGetValue(symbol, out SourceCallableRecord? callable))
            {
                return callable;
            }

            IMethodSymbol definition = symbol.ReducedFrom ?? symbol.OriginalDefinition;
            return callablesBySymbol.TryGetValue(definition, out callable) ? callable : null;
        }

        private static ITypeSymbol? Unwrap(ITypeSymbol? symbol)
        {
            while (true)
            {
                if (symbol is INamedTypeSymbol named
                    && IsExactNullable(named)
                    && named.TypeArguments.Length == 1)
                {
                    symbol = named.TypeArguments[0];
                    continue;
                }

                if (symbol is IArrayTypeSymbol array)
                {
                    symbol = array.ElementType;
                    continue;
                }

                return symbol;
            }
        }

        private static IEnumerable<MemberDeclarationSyntax> Members(BaseTypeDeclarationSyntax syntax) => syntax switch
        {
            TypeDeclarationSyntax type => type.Members,
            EnumDeclarationSyntax enumeration => enumeration.Members,
            _ => Array.Empty<MemberDeclarationSyntax>(),
        };

        private static IEnumerable<SyntaxNode> CallableBodyNodes(
            SyntaxNode syntax,
            SemanticModel model) =>
            syntax.DescendantNodes(
                    descendIntoChildren: node => !IsNameofExpression(node, model),
                    descendIntoTrivia: false)
                .Where(node => !IsNameofExpression(node, model));

        private static bool IsNameofExpression(SyntaxNode node, SemanticModel model) =>
            node is InvocationExpressionSyntax invocation
            && invocation.Expression is IdentifierNameSyntax identifier
            && string.Equals(identifier.Identifier.ValueText, "nameof", StringComparison.Ordinal)
            && model.GetOperation(invocation, CancellationToken.None) is INameOfOperation;

        private static int CompareDeclaration(PracticalDeclaration left, PracticalDeclaration right)
        {
            int id = string.CompareOrdinal(left.Id, right.Id);
            if (id != 0)
            {
                return id;
            }

            int kind = left.Kind.CompareTo(right.Kind);
            if (kind != 0)
            {
                return kind;
            }

            return left.StartByte.CompareTo(right.StartByte);
        }

        private static int CompareEdge(PracticalGraphEdge left, PracticalGraphEdge right)
        {
            int source = string.CompareOrdinal(left.SourceId, right.SourceId);
            return source != 0 ? source : string.CompareOrdinal(left.TargetId, right.TargetId);
        }
    }
}
