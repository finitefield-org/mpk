using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.IO;
using System.Security.Cryptography;
using System.Text;
using System.Text.Encodings.Web;
using System.Text.Json;

namespace Mpk.CSharp2Vir;

internal sealed class SelectionSyntaxFailure : Exception
{
    internal SelectionSyntaxFailure()
        : base("invalid C# selection")
    {
    }
}

internal sealed class CanonicalMethodId
{
    private readonly ReadOnlyCollection<string> parameterTypes;

    internal CanonicalMethodId(
        string canonical,
        string namespaceName,
        string staticType,
        string method,
        string[] parameterTypes,
        string resultType)
    {
        Canonical = canonical;
        NamespaceName = namespaceName;
        StaticType = staticType;
        Method = method;
        this.parameterTypes = Array.AsReadOnly(parameterTypes);
        ResultType = resultType;
    }

    internal string Canonical { get; }

    internal string NamespaceName { get; }

    internal string StaticType { get; }

    internal string Method { get; }

    internal IReadOnlyList<string> ParameterTypes => parameterTypes;

    internal string ResultType { get; }
}

internal sealed class Selection
{
    private readonly ReadOnlyCollection<CanonicalMethodId> parsedMethods;
    private readonly byte[] canonicalBytes;

    internal Selection(RawSelection raw, CanonicalMethodId[] parsedMethods, byte[] canonicalBytes, string sha256)
    {
        Raw = raw;
        this.parsedMethods = Array.AsReadOnly(parsedMethods);
        this.canonicalBytes = canonicalBytes;
        Sha256 = sha256;
    }

    internal RawSelection Raw { get; }

    internal IReadOnlyList<CanonicalMethodId> ParsedMethods => parsedMethods;

    internal ReadOnlySpan<byte> CanonicalBytes => canonicalBytes;

    internal string Sha256 { get; }
}

internal static class SelectionCodec
{
    private const int CompilationBytesMaximum = 64;
    internal const int SourceFilesMaximum = FrontendLimits.SourceFilesMaximum;
    internal const int ContractFilesMaximum = FrontendLimits.ContractFilesMaximum;
    internal const int SelectedMethodsMaximum = FrontendLimits.SelectedMethodsMaximum;
    private const int NormalizedPathBytesMaximum = FrontendLimits.NormalizedPathBytesMaximum;
    private const int CanonicalMethodIdBytesMaximum = FrontendLimits.CanonicalMethodIdBytesMaximum;
    private static readonly byte[] SelectionDomain = Encoding.ASCII.GetBytes("MPK-CSHARP-SELECTION-0.1\0");

    internal static Selection Validate(RawSelection raw)
    {
        ValidateCompilation(raw.Compilation);
        ValidateCount(raw.Sources.Count, 1, SourceFilesMaximum, "CSHARP_LIMIT_SOURCE_FILES");
        ValidateCount(raw.Contracts.Count, 1, ContractFilesMaximum, "CSHARP_LIMIT_CONTRACT_FILES");
        ValidateCount(raw.Methods.Count, 1, SelectedMethodsMaximum, "CSHARP_LIMIT_SELECTED_METHODS");

        ValidatePaths(raw.Sources, "src/", ".cs");
        ValidatePaths(raw.Contracts, "contracts/", ".json");
        ValidateCrossPathCollisions(raw.Sources, raw.Contracts);

        var parsedMethods = new CanonicalMethodId[raw.Methods.Count];
        string? previousMethod = null;
        for (int index = 0; index < raw.Methods.Count; index++)
        {
            string method = raw.Methods[index];
            if (Encoding.UTF8.GetByteCount(method) > CanonicalMethodIdBytesMaximum)
            {
                throw FrontendFailure.Rejected("capture", "CSHARP_LIMIT_CANONICAL_METHOD_ID_BYTES");
            }

            if (previousMethod is not null && string.CompareOrdinal(previousMethod, method) >= 0)
            {
                throw new SelectionSyntaxFailure();
            }

            parsedMethods[index] = ParseMethodId(method);
            previousMethod = method;
        }

        byte[] canonical = CanonicalBytes(raw);
        var preimage = new byte[checked(SelectionDomain.Length + canonical.Length)];
        Buffer.BlockCopy(SelectionDomain, 0, preimage, 0, SelectionDomain.Length);
        Buffer.BlockCopy(canonical, 0, preimage, SelectionDomain.Length, canonical.Length);
        string hash = Convert.ToHexString(SHA256.HashData(preimage)).ToLowerInvariant();
        return new Selection(raw, parsedMethods, canonical, hash);
    }

    internal static byte[] CanonicalBytes(RawSelection raw)
    {
        using var output = new MemoryStream();
        using (var writer = new Utf8JsonWriter(
            output,
            new JsonWriterOptions { Encoder = JavaScriptEncoder.UnsafeRelaxedJsonEscaping, Indented = false }))
        {
            WriteEnvelope(writer, raw);
        }

        return output.ToArray();
    }

    internal static void WriteEnvelope(Utf8JsonWriter writer, RawSelection raw)
    {
        writer.WriteStartObject();
        writer.WriteString("schema", FrontendConstants.SelectionSchema);
        writer.WritePropertyName("value");
        writer.WriteStartObject();
        writer.WriteString("compilation", raw.Compilation);
        WriteStringArray(writer, "contracts", raw.Contracts);
        WriteStringArray(writer, "methods", raw.Methods);
        WriteStringArray(writer, "sources", raw.Sources);
        writer.WriteEndObject();
        writer.WriteEndObject();
    }

    internal static CanonicalMethodId ParseMethodId(string value)
    {
        if (string.IsNullOrEmpty(value) || !IsAscii(value))
        {
            throw new SelectionSyntaxFailure();
        }

        int separator = value.IndexOf("::", StringComparison.Ordinal);
        int open = value.IndexOf('(', separator + 2);
        int arrow = value.IndexOf(")->", open + 1, StringComparison.Ordinal);
        if (separator <= 0
            || separator != value.LastIndexOf("::", StringComparison.Ordinal)
            || open <= separator + 2
            || arrow <= open
            || arrow + 3 >= value.Length
            || value.IndexOf(')', arrow + 1) >= 0
            || value.IndexOf('(', open + 1) >= 0)
        {
            throw new SelectionSyntaxFailure();
        }

        string qualifiedType = value.Substring(0, separator);
        string method = value.Substring(separator + 2, open - separator - 2);
        string parameters = value.Substring(open + 1, arrow - open - 1);
        string resultType = value.Substring(arrow + 3);
        string[] typeComponents = qualifiedType.Split('.');
        if (typeComponents.Length < 2
            || !AllIdentifiers(typeComponents)
            || !IsIdentifier(method)
            || !IsTypeToken(resultType))
        {
            throw new SelectionSyntaxFailure();
        }

        string[] parameterTypes;
        if (parameters.Length == 0)
        {
            parameterTypes = Array.Empty<string>();
        }
        else
        {
            parameterTypes = parameters.Split(',');
            if (Array.Exists(parameterTypes, parameter => !IsTypeToken(parameter)))
            {
                throw new SelectionSyntaxFailure();
            }
        }

        string namespaceName = string.Join('.', typeComponents, 0, typeComponents.Length - 1);
        return new CanonicalMethodId(
            value,
            namespaceName,
            typeComponents[^1],
            method,
            parameterTypes,
            resultType);
    }

    internal static bool IsPortablePath(string path)
    {
        if (string.IsNullOrEmpty(path)
            || path.Length > NormalizedPathBytesMaximum
            || !IsAscii(path)
            || path[0] == '/'
            || path[^1] == '/'
            || path.Contains('\\', StringComparison.Ordinal)
            || path.Contains(':', StringComparison.Ordinal)
            || path.Contains('\0', StringComparison.Ordinal)
            || path.StartsWith("file:", StringComparison.OrdinalIgnoreCase)
            || path.StartsWith("/mpk/", StringComparison.Ordinal))
        {
            return false;
        }

        string[] components = path.Split('/');
        foreach (string component in components)
        {
            if (component.Length == 0
                || component.Length > 255
                || component == "."
                || component == ".."
                || component[^1] == '.'
                || IsWindowsDeviceName(component))
            {
                return false;
            }

            foreach (char character in component)
            {
                if (!IsAsciiLetterOrDigit(character) && character != '.' && character != '_' && character != '-')
                {
                    return false;
                }
            }
        }

        return true;
    }

    private static void ValidateCompilation(string compilation)
    {
        if (string.IsNullOrEmpty(compilation)
            || compilation.Length > CompilationBytesMaximum
            || !IsAscii(compilation)
            || compilation[0] < 'a'
            || compilation[0] > 'z')
        {
            throw new SelectionSyntaxFailure();
        }

        bool separator = false;
        for (int index = 1; index < compilation.Length; index++)
        {
            char character = compilation[index];
            if ((character >= 'a' && character <= 'z') || (character >= '0' && character <= '9'))
            {
                separator = false;
            }
            else if ((character == '.' || character == '_' || character == '-') && !separator)
            {
                separator = true;
            }
            else
            {
                throw new SelectionSyntaxFailure();
            }
        }

        if (separator)
        {
            throw new SelectionSyntaxFailure();
        }
    }

    private static void ValidateCount(int count, int minimum, int maximum, string code)
    {
        if (count < minimum)
        {
            throw new SelectionSyntaxFailure();
        }

        if (count > maximum)
        {
            throw FrontendFailure.Rejected("capture", code);
        }
    }

    private static void ValidatePaths(IReadOnlyList<string> paths, string prefix, string suffix)
    {
        string? previous = null;
        var folded = new HashSet<string>(StringComparer.Ordinal);
        foreach (string path in paths)
        {
            if (Encoding.UTF8.GetByteCount(path) > NormalizedPathBytesMaximum)
            {
                throw FrontendFailure.Rejected("capture", "CSHARP_LIMIT_NORMALIZED_PATH_BYTES");
            }

            if (!IsPortablePath(path)
                || !path.StartsWith(prefix, StringComparison.Ordinal)
                || !path.EndsWith(suffix, StringComparison.Ordinal)
                || path.Length <= prefix.Length + suffix.Length
                || (previous is not null && string.CompareOrdinal(previous, path) >= 0)
                || !folded.Add(AsciiFold(path)))
            {
                throw FrontendFailure.Rejected("capture", "CSHARP_CAPTURE_PATH");
            }

            previous = path;
        }
    }

    private static void ValidateCrossPathCollisions(
        IReadOnlyList<string> sources,
        IReadOnlyList<string> contracts)
    {
        var folded = new HashSet<string>(StringComparer.Ordinal);
        foreach (string path in sources)
        {
            folded.Add(AsciiFold(path));
        }

        foreach (string path in contracts)
        {
            if (!folded.Add(AsciiFold(path)))
            {
                throw FrontendFailure.Rejected("capture", "CSHARP_CAPTURE_PATH");
            }
        }
    }

    private static void WriteStringArray(Utf8JsonWriter writer, string name, IReadOnlyList<string> values)
    {
        writer.WritePropertyName(name);
        writer.WriteStartArray();
        foreach (string value in values)
        {
            writer.WriteStringValue(value);
        }

        writer.WriteEndArray();
    }

    private static bool AllIdentifiers(string[] values)
    {
        foreach (string value in values)
        {
            if (!IsIdentifier(value))
            {
                return false;
            }
        }

        return true;
    }

    private static bool IsIdentifier(string value)
    {
        if (string.IsNullOrEmpty(value)
            || !((value[0] >= 'A' && value[0] <= 'Z')
                || (value[0] >= 'a' && value[0] <= 'z')
                || value[0] == '_'))
        {
            return false;
        }

        for (int index = 1; index < value.Length; index++)
        {
            char character = value[index];
            if (!IsAsciiLetterOrDigit(character) && character != '_')
            {
                return false;
            }
        }

        return true;
    }

    private static bool IsTypeToken(string value)
    {
        return value == "bool" || value == "i32" || value == "u32" || value == "i64" || value == "u64";
    }

    private static bool IsWindowsDeviceName(string component)
    {
        int dot = component.IndexOf('.');
        string stem = dot < 0 ? component : component.Substring(0, dot);
        string upper = stem.ToUpperInvariant();
        if (upper == "CON" || upper == "PRN" || upper == "AUX" || upper == "NUL")
        {
            return true;
        }

        return upper.Length == 4
            && (upper.StartsWith("COM", StringComparison.Ordinal) || upper.StartsWith("LPT", StringComparison.Ordinal))
            && upper[3] >= '1'
            && upper[3] <= '9';
    }

    private static string AsciiFold(string value)
    {
        return value.ToLowerInvariant();
    }

    private static bool IsAscii(string value)
    {
        foreach (char character in value)
        {
            if (character > 0x7f)
            {
                return false;
            }
        }

        return true;
    }

    private static bool IsAsciiLetterOrDigit(char character)
    {
        return (character >= 'A' && character <= 'Z')
            || (character >= 'a' && character <= 'z')
            || (character >= '0' && character <= '9');
    }
}
