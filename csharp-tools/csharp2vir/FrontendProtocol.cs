using System;
using System.Collections.Generic;
using System.Text.Json;

namespace Mpk.CSharp2Vir;

internal static class CSharpFrontendFailureEmitter
{
    internal static byte[] Emit(LowerRequest request, FrontendFailure failure)
    {
        return Emit(request, failure, out _);
    }

    internal static byte[] Emit(
        LowerRequest request,
        FrontendFailure failure,
        out int exitCode)
    {
        FrontendIssue[] issues;
        try
        {
            var collector = new FrontendIssueCollector(failure.Phase);
            foreach (FrontendIssue issue in failure.Issues)
            {
                collector.Add(issue);
            }

            issues = collector.Freeze();
        }
        catch (FrontendFailure budget) when (
            string.Equals(
                budget.Code,
                "CSHARP_FRONTEND_DIAGNOSTIC_BUDGET",
                StringComparison.Ordinal))
        {
            failure = budget;
            issues = new[] { budget.Issues[0] };
        }

        exitCode = failure.ExitCode;
        byte[] envelope = WriteEnvelope(request, failure, issues);
        FrontendLimits.Validate(
            "frontend_stdout",
            checked((ulong)envelope.Length + 1),
            failure.Phase);
        var transport = new byte[checked(envelope.Length + 1)];
        Buffer.BlockCopy(envelope, 0, transport, 0, envelope.Length);
        transport[^1] = (byte)'\n';
        return transport;
    }

    private static byte[] WriteEnvelope(
        LowerRequest request,
        FrontendFailure failure,
        IReadOnlyList<FrontendIssue> issues)
    {
        return EmissionCanonical.Write(writer =>
        {
            writer.WriteStartObject();
            writer.WritePropertyName("diagnostics");
            writer.WriteStartArray();
            if (failure.Status != FrontendStatus.Rejected)
            {
                WriteIssues(writer, issues);
            }

            writer.WriteEndArray();
            writer.WriteString("phase", failure.Phase);
            writer.WritePropertyName("rejected_features");
            writer.WriteStartArray();
            if (failure.Status == FrontendStatus.Rejected)
            {
                WriteIssues(writer, issues);
            }

            writer.WriteEndArray();
            writer.WriteString("schema", CSharpEmissionProfiles.FrontendSchema);
            writer.WritePropertyName("selection");
            EmissionCanonical.WriteRaw(
                writer,
                SelectionCodec.CanonicalBytes(request.RawSelection));
            EmissionCanonical.WriteSemanticContext(writer);
            writer.WriteString("status", FrontendDiagnosticRegistry.StatusText(failure.Status));
            writer.WriteEndObject();
        }, "frontend_stdout", failure.Phase);
    }

    private static void WriteIssues(Utf8JsonWriter writer, IReadOnlyList<FrontendIssue> issues)
    {
        foreach (FrontendIssue issue in issues)
        {
            writer.WriteStartObject();
            writer.WriteString("code", issue.Code);
            if (issue.FunctionId is not null)
            {
                writer.WriteString("function_id", issue.FunctionId);
            }

            writer.WriteString("message", issue.Message);
            if (issue.Span is not null)
            {
                writer.WritePropertyName("span");
                writer.WriteStartObject();
                writer.WriteNumber("end", issue.Span.End);
                writer.WriteString("normalized_path", issue.Span.NormalizedPath);
                writer.WriteNumber("start", issue.Span.Start);
                writer.WriteEndObject();
            }

            writer.WriteEndObject();
        }
    }
}
