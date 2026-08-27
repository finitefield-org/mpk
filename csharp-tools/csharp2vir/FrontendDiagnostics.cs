using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Collections.Immutable;
using System.Globalization;
using System.Linq;
using System.Text;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.Text;

namespace Mpk.CSharp2Vir;

internal sealed class FrontendIssueSpan
{
    internal FrontendIssueSpan(string normalizedPath, int start, int end)
    {
        if (!SelectionCodec.IsPortablePath(normalizedPath) || start < 0 || end <= start)
        {
            throw new InvalidOperationException("invalid C# issue span");
        }

        NormalizedPath = normalizedPath;
        Start = start;
        End = end;
    }

    internal string NormalizedPath { get; }

    internal int Start { get; }

    internal int End { get; }
}

internal sealed class FrontendIssue
{
    internal FrontendIssue(
        string code,
        string message,
        string? functionId = null,
        FrontendIssueSpan? span = null)
    {
        if (string.IsNullOrEmpty(code)
            || string.IsNullOrEmpty(message)
            || message.Any(char.IsControl)
            || (functionId is not null
                && (functionId.Length == 0
                    || functionId.Any(character => character > 0x7f)
                    || Encoding.UTF8.GetByteCount(functionId) > FrontendLimits.CanonicalMethodIdBytesMaximum)))
        {
            throw new InvalidOperationException("invalid C# issue");
        }

        Code = code;
        Message = message;
        FunctionId = functionId;
        Span = span;
    }

    internal string Code { get; }

    internal string Message { get; }

    internal string? FunctionId { get; }

    internal FrontendIssueSpan? Span { get; }
}

internal sealed class FrontendDiagnosticDefinition
{
    internal FrontendDiagnosticDefinition(
        string code,
        FrontendStatus status,
        string phase)
    {
        Code = code;
        Status = status;
        Phase = phase;
    }

    internal string Code { get; }

    internal FrontendStatus Status { get; }

    internal string Phase { get; }
}

internal static class FrontendDiagnosticRegistry
{
    private static readonly ReadOnlyCollection<FrontendDiagnosticDefinition> definitions =
        Array.AsReadOnly(new[]
        {
            Rejected("CSHARP_CAPTURE_FILE_TYPE", "capture"),
            Rejected("CSHARP_CAPTURE_PATH", "capture"),
            Rejected("CSHARP_CAPTURE_INVENTORY", "capture"),
            Source("CSHARP_SOURCE_ENCODING", "source"),
            Source("CSHARP_SOURCE_PARSE", "source"),
            Source("CSHARP_SOURCE_DIAGNOSTIC", "metadata"),
            Toolchain("CSHARP_TOOLCHAIN_ARCHIVE", "release"),
            Toolchain("CSHARP_TOOLCHAIN_RUNTIME", "release"),
            Toolchain("CSHARP_TOOLCHAIN_ROSLYN", "release"),
            Toolchain("CSHARP_TOOLCHAIN_REFERENCE", "release"),
            Toolchain("CSHARP_TOOLCHAIN_OPTIONS", "owner-phase"),
            Toolchain("CSHARP_TOOLCHAIN_ADAPTER", "owner-phase"),
            Rejected("CSHARP_SUBSET_DECLARATION", "subset"),
            Rejected("CSHARP_SUBSET_TYPE", "typecheck"),
            Rejected("CSHARP_SUBSET_LITERAL", "typecheck"),
            Rejected("CSHARP_SUBSET_CONTROL_FLOW", "subset"),
            Rejected("CSHARP_SUBSET_OPERATION", "subset"),
            Rejected("CSHARP_SUBSET_OVERFLOW_CONTEXT", "subset"),
            Rejected("CSHARP_SUBSET_CHECKED_CONVERSION", "subset"),
            Rejected("CSHARP_SUBSET_CONVERSION", "subset"),
            Rejected("CSHARP_SUBSET_CALL", "subset"),
            Rejected("CSHARP_SUBSET_INITIALIZATION", "subset"),
            Rejected("CSHARP_SUBSET_PURITY", "subset"),
            Rejected("CSHARP_SUBSET_ABRUPT", "subset"),
            Rejected("CSHARP_CONTRACT_JSON", "subset"),
            Rejected("CSHARP_CONTRACT_SHAPE", "subset"),
            Rejected("CSHARP_CONTRACT_IDENTITY", "subset"),
            Rejected("CSHARP_CONTRACT_DUPLICATE", "subset"),
            Rejected("CSHARP_CONTRACT_MISSING", "subset"),
            Rejected("CSHARP_CONTRACT_UNUSED", "subset"),
            Rejected("CSHARP_CONTRACT_TYPE", "subset"),
            Rejected("CSHARP_CONTRACT_OPERATOR", "subset"),
            Rejected("CSHARP_CONTRACT_HASH", "subset"),
            Rejected("CSHARP_LOWERING_OPERATION", "lowering"),
            Rejected("CSHARP_LOWERING_CFG", "lowering"),
            Rejected("CSHARP_LOWERING_CHECK_MISSING", "lowering"),
            Rejected("CSHARP_LOWERING_CHECK_EXTRA", "lowering"),
            Rejected("CSHARP_LOWERING_CHECK_ORDER", "lowering"),
            Toolchain("CSHARP_SOURCE_MAP_EXTERNAL", "emission"),
            Toolchain("CSHARP_SOURCE_MAP_RANGE", "emission"),
            Toolchain("CSHARP_SOURCE_MAP_UTF16", "emission"),
            Toolchain("CSHARP_FRONTEND_OUTPUT_LIMIT", "owner-phase"),
            Toolchain("CSHARP_FRONTEND_DIAGNOSTIC_BUDGET", "owner-phase"),
            Toolchain("CSHARP_FRONTEND_INTERNAL", "owner-phase"),
        });

    internal static IReadOnlyList<FrontendDiagnosticDefinition> Definitions => definitions;

    internal static FrontendDiagnosticDefinition Find(string code)
    {
        foreach (FrontendDiagnosticDefinition definition in definitions)
        {
            if (string.Equals(definition.Code, code, StringComparison.Ordinal))
            {
                return definition;
            }
        }

        foreach (FrontendLimitDefinition limit in FrontendLimits.Definitions)
        {
            if (string.Equals(limit.Code, code, StringComparison.Ordinal)
                && limit.Disposition == FrontendLimitDisposition.Rejected)
            {
                return new FrontendDiagnosticDefinition(
                    code,
                    FrontendStatus.Rejected,
                    "owner-phase");
            }
        }

        throw new InvalidOperationException("unknown C# frontend diagnostic");
    }

    internal static void Validate(string code, FrontendStatus status, string phase)
    {
        FrontendDiagnosticDefinition definition = Find(code);
        if (definition.Status != status
            || !IsPhase(phase)
            || (!string.Equals(definition.Phase, "owner-phase", StringComparison.Ordinal)
                && !string.Equals(definition.Phase, phase, StringComparison.Ordinal)))
        {
            throw new InvalidOperationException("invalid C# frontend diagnostic owner");
        }
    }

    internal static void ValidateIssue(
        FrontendIssue issue,
        FrontendStatus status,
        string phase)
    {
        Validate(issue.Code, status, phase);
        if (string.Equals(issue.Code, "CSHARP_SOURCE_DIAGNOSTIC", StringComparison.Ordinal))
        {
            const string prefix = "C# compiler diagnostic ";
            if (!issue.Message.StartsWith(prefix, StringComparison.Ordinal)
                || !IsDiagnosticId(issue.Message.Substring(prefix.Length)))
            {
                throw new InvalidOperationException("invalid C# compiler diagnostic message");
            }

            return;
        }

        if (!string.Equals(issue.Message, PublicMessage(status, issue.Code), StringComparison.Ordinal))
        {
            throw new InvalidOperationException("invalid C# frontend diagnostic message");
        }
    }

    internal static string StatusText(FrontendStatus status)
    {
        return status switch
        {
            FrontendStatus.Rejected => "rejected",
            FrontendStatus.SourceError => "source-error",
            FrontendStatus.FrontendError => "frontend-error",
            _ => throw new InvalidOperationException("invalid C# frontend status"),
        };
    }

    internal static string PublicMessage(FrontendStatus status, string code)
    {
        if (code.StartsWith("CSHARP_LIMIT_", StringComparison.Ordinal))
        {
            return "C# profile limit exceeded";
        }

        return status switch
        {
            FrontendStatus.SourceError => "C# source is invalid",
            FrontendStatus.Rejected => "C# source is outside the frozen profile",
            FrontendStatus.FrontendError => "C# frontend failed closed",
            _ => throw new InvalidOperationException("invalid C# frontend status"),
        };
    }

    private static FrontendDiagnosticDefinition Rejected(string code, string phase)
    {
        return new FrontendDiagnosticDefinition(code, FrontendStatus.Rejected, phase);
    }

    private static FrontendDiagnosticDefinition Source(string code, string phase)
    {
        return new FrontendDiagnosticDefinition(code, FrontendStatus.SourceError, phase);
    }

    private static FrontendDiagnosticDefinition Toolchain(string code, string phase)
    {
        return new FrontendDiagnosticDefinition(code, FrontendStatus.FrontendError, phase);
    }

    private static bool IsPhase(string phase)
    {
        return phase is "release" or "capture" or "source" or "metadata"
            or "typecheck" or "subset" or "lowering" or "emission";
    }

    internal static bool IsDiagnosticId(string value)
    {
        return value.Length == 6
            && value[0] == 'C'
            && value[1] == 'S'
            && value[2] is >= '0' and <= '9'
            && value[3] is >= '0' and <= '9'
            && value[4] is >= '0' and <= '9'
            && value[5] is >= '0' and <= '9';
    }
}

internal sealed class FrontendIssueCollector
{
    private static readonly UTF8Encoding StrictUtf8 = new UTF8Encoding(false, true);
    private readonly string phase;
    private readonly List<FrontendIssue> issues = new List<FrontendIssue>();
    private ulong messageBytes;

    internal FrontendIssueCollector(string phase)
    {
        this.phase = phase;
    }

    internal void Add(FrontendIssue issue)
    {
        int bytes;
        try
        {
            bytes = StrictUtf8.GetByteCount(issue.Message);
        }
        catch (EncoderFallbackException)
        {
            throw FrontendLimits.Failure("diagnostic_message_bytes_each", phase);
        }

        FrontendLimits.Validate("diagnostic_message_bytes_each", (ulong)bytes, phase);
        FrontendLimits.Validate("normalized_issues", checked((ulong)issues.Count + 1), phase);
        ulong projected = FrontendLimits.Add(
            "diagnostic_message_bytes_total",
            messageBytes,
            (ulong)bytes,
            phase);
        issues.Add(issue);
        messageBytes = projected;
    }

    internal FrontendIssue[] Freeze()
    {
        FrontendIssue[] result = issues.ToArray();
        Array.Sort(result, ComparePublicIssues);
        return result;
    }

    private static int ComparePublicIssues(FrontendIssue left, FrontendIssue right)
    {
        FrontendIssueSpan? leftSpan = left.Span;
        FrontendIssueSpan? rightSpan = right.Span;
        int comparison = string.CompareOrdinal(
            leftSpan?.NormalizedPath ?? string.Empty,
            rightSpan?.NormalizedPath ?? string.Empty);
        if (comparison != 0)
        {
            return comparison;
        }

        comparison = (leftSpan?.Start ?? 0).CompareTo(rightSpan?.Start ?? 0);
        if (comparison != 0)
        {
            return comparison;
        }

        comparison = string.CompareOrdinal(left.Code, right.Code);
        if (comparison != 0)
        {
            return comparison;
        }

        comparison = string.CompareOrdinal(left.Message, right.Message);
        if (comparison != 0)
        {
            return comparison;
        }

        comparison = string.CompareOrdinal(
            left.FunctionId ?? string.Empty,
            right.FunctionId ?? string.Empty);
        return comparison != 0
            ? comparison
            : (leftSpan?.End ?? 0).CompareTo(rightSpan?.End ?? 0);
    }
}

internal static class FrontendDiagnosticNormalizer
{
    internal static FrontendFailure? Normalize(
        string phase,
        string publicCode,
        ImmutableArray<Diagnostic> diagnostics,
        ImmutableArray<SourceText> sourceTexts,
        ImmutableArray<SyntaxTree> syntaxTrees)
    {
        if (sourceTexts.Length != syntaxTrees.Length)
        {
            throw FrontendFailure.Toolchain(phase, "CSHARP_TOOLCHAIN_ADAPTER");
        }

        var sources = new DiagnosticSource[sourceTexts.Length];
        for (int index = 0; index < sources.Length; index++)
        {
            sources[index] = new DiagnosticSource(sourceTexts[index], syntaxTrees[index]);
        }

        var records = new List<DiagnosticRecord>(diagnostics.Length);
        foreach (Diagnostic diagnostic in diagnostics)
        {
            if (!FrontendDiagnosticRegistry.IsDiagnosticId(diagnostic.Id)
                || (int)diagnostic.Severity < (int)DiagnosticSeverity.Hidden
                || (int)diagnostic.Severity > (int)DiagnosticSeverity.Error)
            {
                throw FrontendFailure.Toolchain(phase, "CSHARP_TOOLCHAIN_ADAPTER");
            }

            string message;
            try
            {
                message = diagnostic.GetMessage(CultureInfo.InvariantCulture)
                    .Normalize(NormalizationForm.FormC);
            }
            catch (Exception error) when (
                error is ArgumentException || error is InvalidOperationException)
            {
                throw FrontendFailure.Toolchain(phase, "CSHARP_TOOLCHAIN_ADAPTER");
            }

            FrontendIssueSpan? span = TryMap(diagnostic.Location, sources);
            try
            {
                records.Add(new DiagnosticRecord(diagnostic, message, span));
            }
            catch (EncoderFallbackException)
            {
                throw FrontendFailure.Toolchain(phase, "CSHARP_TOOLCHAIN_ADAPTER");
            }
        }

        records.Sort(DiagnosticRecord.Compare);
        var collector = new FrontendIssueCollector(phase);
        foreach (DiagnosticRecord record in records)
        {
            if (record.Diagnostic.IsSuppressed
                || (record.Diagnostic.Severity != DiagnosticSeverity.Warning
                    && record.Diagnostic.Severity != DiagnosticSeverity.Error))
            {
                continue;
            }

            string message = string.Equals(publicCode, "CSHARP_SOURCE_DIAGNOSTIC", StringComparison.Ordinal)
                ? "C# compiler diagnostic " + record.Diagnostic.Id
                : FrontendDiagnosticRegistry.PublicMessage(FrontendStatus.SourceError, publicCode);
            collector.Add(new FrontendIssue(publicCode, message, span: record.Span));
        }

        FrontendIssue[] issues = collector.Freeze();
        return issues.Length == 0
            ? null
            : FrontendFailure.WithIssues(
                FrontendStatus.SourceError,
                phase,
                publicCode,
                issues);
    }

    private static FrontendIssueSpan? TryMap(Location location, IReadOnlyList<DiagnosticSource> sources)
    {
        if (location == Location.None || !location.IsInSource || location.SourceTree is null)
        {
            return null;
        }

        DiagnosticSource? source = null;
        for (int index = 0; index < sources.Count; index++)
        {
            if (ReferenceEquals(sources[index].Tree, location.SourceTree))
            {
                source = sources[index];
                break;
            }
        }

        if (source is null)
        {
            return null;
        }

        TextSpan span = location.SourceSpan;
        if (span.Start < 0
            || span.End <= span.Start
            || span.End > source.Text.Length
            || source.Utf8Boundaries[span.Start] < 0
            || source.Utf8Boundaries[span.End] < 0)
        {
            return null;
        }

        return new FrontendIssueSpan(
            source.Tree.FilePath,
            source.Utf8Boundaries[span.Start],
            source.Utf8Boundaries[span.End]);
    }

    private sealed class DiagnosticSource
    {
        internal DiagnosticSource(SourceText text, SyntaxTree tree)
        {
            Text = text;
            Tree = tree;
            Utf8Boundaries = BuildUtf8Boundaries(text.ToString());
        }

        internal SourceText Text { get; }

        internal SyntaxTree Tree { get; }

        internal int[] Utf8Boundaries { get; }

        private static int[] BuildUtf8Boundaries(string source)
        {
            var result = new int[source.Length + 1];
            Array.Fill(result, -1);
            int utf16 = 0;
            int utf8 = 0;
            result[0] = 0;
            while (utf16 < source.Length)
            {
                char current = source[utf16];
                if (char.IsHighSurrogate(current))
                {
                    if (utf16 + 1 >= source.Length || !char.IsLowSurrogate(source[utf16 + 1]))
                    {
                        return result;
                    }

                    utf16 += 2;
                    utf8 = checked(utf8 + 4);
                }
                else if (char.IsLowSurrogate(current))
                {
                    return result;
                }
                else
                {
                    utf16++;
                    utf8 = checked(utf8 + (current <= 0x7f ? 1 : current <= 0x7ff ? 2 : 3));
                }

                result[utf16] = utf8;
            }

            return result;
        }
    }

    private sealed class DiagnosticRecord
    {
        private static readonly UTF8Encoding StrictUtf8 = new UTF8Encoding(false, true);
        private readonly byte[] messageBytes;

        internal DiagnosticRecord(
            Diagnostic diagnostic,
            string message,
            FrontendIssueSpan? span)
        {
            Diagnostic = diagnostic;
            Span = span;
            messageBytes = StrictUtf8.GetBytes(message);
        }

        internal Diagnostic Diagnostic { get; }

        internal FrontendIssueSpan? Span { get; }

        internal static int Compare(DiagnosticRecord left, DiagnosticRecord right)
        {
            int comparison = string.CompareOrdinal(
                left.Span?.NormalizedPath ?? string.Empty,
                right.Span?.NormalizedPath ?? string.Empty);
            if (comparison != 0)
            {
                return comparison;
            }

            comparison = (left.Span?.Start ?? 0).CompareTo(right.Span?.Start ?? 0);
            if (comparison != 0)
            {
                return comparison;
            }

            comparison = (left.Span?.End ?? 0).CompareTo(right.Span?.End ?? 0);
            if (comparison != 0)
            {
                return comparison;
            }

            comparison = string.CompareOrdinal(left.Diagnostic.Id, right.Diagnostic.Id);
            if (comparison != 0)
            {
                return comparison;
            }

            comparison = left.Diagnostic.Severity.CompareTo(right.Diagnostic.Severity);
            return comparison != 0
                ? comparison
                : left.messageBytes.AsSpan().SequenceCompareTo(right.messageBytes);
        }
    }
}

internal static class FrontendPrecedence
{
    internal static FrontendFailure Select(IReadOnlyList<FrontendFailure> failures)
    {
        if (failures.Count == 0)
        {
            throw new ArgumentException("at least one C# frontend failure is required", nameof(failures));
        }

        FrontendFailure winner = failures[0];
        for (int index = 1; index < failures.Count; index++)
        {
            FrontendFailure candidate = failures[index];
            int rank = Rank(candidate).CompareTo(Rank(winner));
            if (rank < 0
                || (rank == 0 && string.CompareOrdinal(candidate.Code, winner.Code) < 0))
            {
                winner = candidate;
            }
        }

        return winner;
    }

    private static int Rank(FrontendFailure failure)
    {
        int phase = failure.Phase switch
        {
            "release" => 0,
            "capture" => 100,
            "source" => 200,
            "metadata" => 300,
            "typecheck" => 400,
            "subset" => 500,
            "lowering" => 600,
            "emission" => 700,
            _ => 1_000,
        };
        int within = failure.Code switch
        {
            "CSHARP_SOURCE_ENCODING" => 0,
            "CSHARP_TOOLCHAIN_OPTIONS" => 10,
            "CSHARP_SOURCE_PARSE" => 20,
            "CSHARP_SOURCE_DIAGNOSTIC" => 20,
            "CSHARP_SUBSET_TYPE" => 0,
            "CSHARP_SUBSET_LITERAL" => 1,
            "CSHARP_SUBSET_DECLARATION" => 0,
            "CSHARP_CONTRACT_HASH" => 20,
            "CSHARP_CONTRACT_MISSING" => 20,
            "CSHARP_LOWERING_OPERATION" => 0,
            "CSHARP_LOWERING_CFG" => 10,
            "CSHARP_LOWERING_CHECK_MISSING" => 20,
            "CSHARP_LOWERING_CHECK_EXTRA" => 20,
            "CSHARP_LOWERING_CHECK_ORDER" => 20,
            "CSHARP_SOURCE_MAP_EXTERNAL" => 0,
            "CSHARP_SOURCE_MAP_RANGE" => 0,
            "CSHARP_SOURCE_MAP_UTF16" => 0,
            "CSHARP_FRONTEND_OUTPUT_LIMIT" => 20,
            _ => 10,
        };
        return phase + within;
    }
}
