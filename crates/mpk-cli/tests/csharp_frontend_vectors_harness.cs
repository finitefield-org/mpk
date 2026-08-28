using System;
using System.Collections.Generic;
using System.Collections.Immutable;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Security.Cryptography;
using System.Text;
using System.Text.Json;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.Text;

namespace Mpk.CSharp2Vir;

internal static partial class FrontendVectorHarness
{
    private const string SourcePath = "src/Case.cs";
    private const string BaselineSourcePath = "src/Policy.cs";
    private static readonly UTF8Encoding StrictUtf8 = new UTF8Encoding(false, true);

    public static int Main(string[] args)
    {
        if (args.Length != 4 || (args[0] != "self-test" && args[0] != "report"))
        {
            Console.Error.Write("CSHARP_FRONTEND_VECTOR_TEST_USAGE\n");
            return 1;
        }

        try
        {
            using JsonDocument profile = JsonDocument.Parse(
                File.ReadAllBytes(args[2]),
                new JsonDocumentOptions
                {
                    AllowTrailingCommas = false,
                    CommentHandling = JsonCommentHandling.Disallow,
                    MaxDepth = 256,
                });
            byte[] report = Execute(profile.RootElement, args[1], args[3]);
            if (args[0] == "report")
            {
                Stream output = Console.OpenStandardOutput();
                output.Write(report);
                output.WriteByte((byte)'\n');
                output.Flush();
            }

            return 0;
        }
        catch (VectorFailure failure)
        {
            Console.Error.Write("CSHARP_FRONTEND_VECTOR_TEST_" + failure.Code + "\n");
            return 1;
        }
        catch (Exception)
        {
            Console.Error.Write("CSHARP_FRONTEND_VECTOR_TEST_UNEXPECTED\n");
            return 1;
        }
    }

    private static byte[] Execute(
        JsonElement profile,
        string referencePackRoot,
        string fuzzManifestPath)
    {
        Equal("mpk.csharp.profile.conformance.v0", Text(profile, "schema"), "PROFILE_SCHEMA");
        JsonElement acceptedVectors = Property(profile, "accepted_cases");
        JsonElement rejectedVectors = Property(profile, "rejected_cases");
        Equal(30, acceptedVectors.GetArrayLength(), "ACCEPTED_COUNT");
        Equal(88, rejectedVectors.GetArrayLength(), "REJECTED_COUNT");

        var accepted = new List<AcceptedEvaluation>();
        foreach (JsonElement vector in acceptedVectors.EnumerateArray())
        {
            accepted.Add(ExecuteAccepted(profile, vector, referencePackRoot));
        }

        LowerRequest failureRequest = FailureRequest(profile);
        var rejected = new List<RejectedResult>();
        foreach (JsonElement vector in rejectedVectors.EnumerateArray())
        {
            rejected.Add(ExecuteRejected(vector, failureRequest));
        }

        ValidateDiagnosticRegistry(profile);
        string normalizationHash = ValidateDiagnosticNormalization(referencePackRoot);
        List<LimitResult> limits = ValidateLimits(profile);
        List<PrecedenceResult> precedence = ValidatePrecedence(profile);
        List<HashResult> hashes = ValidateHashes(profile);
        List<SemanticRowResult> semanticRows = ValidateSemanticRows(profile);
        List<FuzzResult> fuzz = ExecuteFuzz(
            profile,
            referencePackRoot,
            fuzzManifestPath);
        int differentialCaseCount = accepted.Sum(result => result.Differential.CaseCount);

        return EmissionCanonical.Write(writer =>
        {
            writer.WriteStartObject();
            writer.WritePropertyName("accepted");
            writer.WriteStartArray();
            foreach (AcceptedEvaluation evaluation in accepted)
            {
                AcceptedResult result = evaluation.Result;
                writer.WriteStartObject();
                writer.WriteString("code", string.Empty);
                writer.WriteString("envelope_sha256", result.EnvelopeSha256);
                writer.WriteString("id", result.Id);
                writer.WriteString("phase", "complete");
                writer.WriteString("source_manifest_sha256", result.SourceManifestSha256);
                writer.WriteString("source_map_sha256", result.SourceMapSha256);
                writer.WriteString("status", "ir-lowered");
                writer.WriteString("vir_sha256", result.VirSha256);
                writer.WriteEndObject();
            }

            writer.WriteEndArray();
            writer.WritePropertyName("diagnostic_registry");
            writer.WriteStartArray();
            foreach (FrontendDiagnosticDefinition definition in FrontendDiagnosticRegistry.Definitions)
            {
                writer.WriteStartObject();
                writer.WriteString("code", definition.Code);
                writer.WriteString("phase", definition.Phase);
                writer.WriteString("status", FrontendDiagnosticRegistry.StatusText(definition.Status));
                writer.WriteEndObject();
            }

            writer.WriteEndArray();
            writer.WritePropertyName("differential");
            writer.WriteStartObject();
            writer.WriteNumber("case_count", differentialCaseCount);
            writer.WriteString("runtime_version", Environment.Version.ToString(3));
            writer.WritePropertyName("vectors");
            writer.WriteStartArray();
            foreach (AcceptedEvaluation evaluation in accepted)
            {
                DifferentialResult result = evaluation.Differential;
                writer.WriteStartObject();
                writer.WriteNumber("case_count", result.CaseCount);
                writer.WriteString("id", result.Id);
                writer.WriteString("outcomes_sha256", result.OutcomesSha256);
                writer.WriteEndObject();
            }

            writer.WriteEndArray();
            writer.WriteEndObject();
            writer.WritePropertyName("fuzz");
            writer.WriteStartArray();
            foreach (FuzzResult result in fuzz)
            {
                writer.WriteStartObject();
                writer.WriteString("id", result.Id);
                writer.WriteNumber("mutation_count", result.MutationCount);
                writer.WriteString("outcomes_sha256", result.OutcomesSha256);
                writer.WriteNumber("seed_count", result.SeedCount);
                writer.WriteEndObject();
            }

            writer.WriteEndArray();
            writer.WritePropertyName("hashes");
            writer.WriteStartArray();
            foreach (HashResult hash in hashes)
            {
                writer.WriteStartObject();
                writer.WriteString("id", hash.Id);
                writer.WriteNumber("payload_utf8_length", hash.PayloadLength);
                writer.WriteNumber("preimage_length", hash.PreimageLength);
                writer.WriteString("sha256", hash.Sha256);
                writer.WriteEndObject();
            }

            writer.WriteEndArray();
            writer.WritePropertyName("limits");
            writer.WriteStartArray();
            foreach (LimitResult limit in limits)
            {
                writer.WriteStartObject();
                writer.WriteString("code", limit.Code);
                writer.WriteString("id", limit.Id);
                writer.WriteNumber("maximum", limit.Maximum);
                writer.WriteString("plus_one_status", limit.PlusOneStatus);
                writer.WriteEndObject();
            }

            writer.WriteEndArray();
            writer.WriteString("normalization_sha256", normalizationHash);
            writer.WritePropertyName("precedence");
            writer.WriteStartArray();
            foreach (PrecedenceResult result in precedence)
            {
                writer.WriteStartObject();
                writer.WriteString("id", result.Id);
                writer.WriteString("winner", result.Winner);
                writer.WriteEndObject();
            }

            writer.WriteEndArray();
            writer.WritePropertyName("rejected");
            writer.WriteStartArray();
            foreach (RejectedResult result in rejected)
            {
                writer.WriteStartObject();
                writer.WriteBoolean("artifact_free", true);
                writer.WriteString("code", result.Code);
                writer.WritePropertyName("envelope");
                writer.WriteRawValue(result.Envelope, skipInputValidation: false);
                writer.WriteString("envelope_sha256", result.EnvelopeSha256);
                writer.WriteNumber("exit", result.Exit);
                writer.WriteString("id", result.Id);
                writer.WriteString("owner", result.Owner);
                writer.WriteString("phase", result.Phase);
                writer.WriteString("status", result.Status);
                writer.WriteEndObject();
            }

            writer.WriteEndArray();
            writer.WriteString("schema", "mpk.csharp.frontend_vector_execution.v0");
            writer.WritePropertyName("semantic_rows");
            writer.WriteStartArray();
            foreach (SemanticRowResult row in semanticRows)
            {
                writer.WriteStartObject();
                writer.WriteString("disposition", row.Disposition);
                writer.WriteString("row", row.Row);
                writer.WriteEndObject();
            }

            writer.WriteEndArray();
            writer.WriteEndObject();
        });
    }

    private static AcceptedEvaluation ExecuteAccepted(
        JsonElement profile,
        JsonElement vector,
        string referencePackRoot)
    {
        string id = Text(vector, "id");
        JsonElement expectation = Property(vector, "expect");
        Equal("ir-lowered", Text(expectation, "status"), id + "_STATUS");
        Equal("complete", Text(expectation, "phase"), id + "_PHASE");
        Equal(string.Empty, Text(expectation, "code"), id + "_CODE");

        AcceptedExecution execution = BuildAccepted(profile, vector, referencePackRoot);
        ValidateOperationProjection(
            execution.Lowered,
            Text(vector, "method"),
            Property(vector, "expected_profile_operations"),
            id);
        ValidateRequiredChecks(
            execution.Lowered,
            Text(vector, "method"),
            Property(vector, "expected_required_checks"),
            id);
        ValidateSuccessEnvelope(execution.Success.EnvelopeBytes, id);

        AcceptedExecution repeated = BuildAccepted(profile, vector, referencePackRoot);
        EqualBytes(execution.Success.EnvelopeBytes, repeated.Success.EnvelopeBytes, id + "_ENVELOPE");
        EqualBytes(execution.Success.Vir.CanonicalBytes, repeated.Success.Vir.CanonicalBytes, id + "_VIR");
        EqualBytes(execution.Success.SourceMap.CanonicalBytes, repeated.Success.SourceMap.CanonicalBytes, id + "_MAP");
        EqualBytes(
            execution.Success.SourceManifest.CanonicalBytes,
            repeated.Success.SourceManifest.CanonicalBytes,
            id + "_MANIFEST");

        return new AcceptedEvaluation(
            new AcceptedResult(
                id,
                RawSha256(execution.Success.EnvelopeBytes),
                execution.Success.Vir.Sha256,
                execution.Success.SourceMap.Sha256,
                execution.Success.SourceManifest.Sha256),
            ExecuteDifferential(id, Text(vector, "method"), execution));
    }

    private static AcceptedExecution BuildAccepted(
        JsonElement profile,
        JsonElement vector,
        string referencePackRoot)
    {
        string id = Text(vector, "id");
        string source = Text(vector, "source");
        string method = Text(vector, "method");
        bool baselineContract = id == "contract.normalization";
        string sourcePath = baselineContract ? BaselineSourcePath : SourcePath;
        string compilation = baselineContract ? "payment-policy" : "vector";

        Selection discoverySelection = SelectionCodec.Validate(new RawSelection(
            compilation,
            new[] { sourcePath },
            new[] { "contracts/discovery.json" },
            new[] { method }));
        CapturedSourceSet discoverySources = Sources(sourcePath, source);
        RoslynSourceSession discoveryParse = RoslynSessionFactory.Parse(discoverySelection, discoverySources);
        RoslynCompilationSession discoveryCompilation = RoslynSessionFactory.Compile(
            discoverySelection,
            discoveryParse,
            referencePackRoot);
        SubsetClosure discoveryClosure = CSharpSubset.Validate(discoverySelection, discoveryCompilation);

        string[] contractPaths;
        byte[][] contractBytes;
        if (baselineContract)
        {
            contractPaths = new[] { "contracts/approved.json" };
            contractBytes = new[] { StrictUtf8.GetBytes(Property(profile, "contract_fixture").GetRawText() + "\n") };
        }
        else
        {
            contractPaths = discoveryClosure.Methods
                .Select((_, index) => "contracts/contract" + index.ToString("D3", CultureInfo.InvariantCulture) + ".json")
                .ToArray();
            contractBytes = discoveryClosure.Methods
                .Select(methodRecord => StrictUtf8.GetBytes(DefaultContract(methodRecord.CanonicalId)))
                .ToArray();
        }

        Selection selection = SelectionCodec.Validate(new RawSelection(
            compilation,
            new[] { sourcePath },
            contractPaths,
            new[] { method }));
        CapturedSourceSet sources = Sources(sourcePath, source);
        RoslynSourceSession parsed = RoslynSessionFactory.Parse(selection, sources);
        RoslynCompilationSession compiled = RoslynSessionFactory.Compile(selection, parsed, referencePackRoot);
        SubsetClosure closure = CSharpSubset.Validate(selection, compiled);
        Equal(discoveryClosure.Methods.Length, closure.Methods.Length, id + "_CLOSURE_COUNT");

        var files = new List<CapturedFile>
        {
            new CapturedFile(CapturedInputKind.Source, sourcePath, StrictUtf8.GetBytes(source)),
        };
        for (int index = 0; index < contractPaths.Length; index++)
        {
            files.Add(new CapturedFile(CapturedInputKind.Contract, contractPaths[index], contractBytes[index]));
        }

        var snapshot = new CapturedSnapshot(selection, files.ToArray());
        ContractSet contracts = CSharpContracts.Attach(selection, snapshot, closure);
        LoweredClosure lowered = CSharpLowering.Lower(selection, closure, contracts);
        EmittedFrontendSuccess success = CSharpFrontendSuccessEmitter.Emit(
            Request(selection.Raw),
            selection,
            snapshot,
            sources,
            compiled,
            closure,
            contracts,
            lowered);
        return new AcceptedExecution(selection, compiled, lowered, success);
    }

    private static RejectedResult ExecuteRejected(JsonElement vector, LowerRequest request)
    {
        string id = Text(vector, "id");
        JsonElement expectation = Property(vector, "expect");
        string status = Text(expectation, "status");
        string phase = Text(expectation, "phase");
        string code = Text(expectation, "code");
        FrontendFailure failure = CreateFailure(status, phase, code);
        Equal(status, FrontendDiagnosticRegistry.StatusText(failure.Status), id + "_STATUS");
        Equal(phase, failure.Phase, id + "_PHASE");
        Equal(code, failure.Code, id + "_CODE");
        byte[] envelope = CSharpFrontendFailureEmitter.Emit(request, failure);
        ValidateFailureEnvelope(envelope, failure, id);
        return new RejectedResult(
            id,
            status,
            phase,
            code,
            failure.ExitCode,
            Owner(id, phase),
            RawSha256(envelope),
            envelope.AsSpan(0, envelope.Length - 1).ToArray());
    }

    private static FrontendFailure CreateFailure(string status, string phase, string code)
    {
        if (code == "CSHARP_SOURCE_DIAGNOSTIC")
        {
            return FrontendFailure.WithIssues(
                FrontendStatus.SourceError,
                phase,
                code,
                new[] { new FrontendIssue(code, "C# compiler diagnostic CS0103") });
        }

        return status switch
        {
            "rejected" => FrontendFailure.Rejected(phase, code),
            "source-error" => FrontendFailure.SourceError(phase, code),
            "frontend-error" => FrontendFailure.Toolchain(phase, code),
            _ => throw new VectorFailure("REJECTED_STATUS"),
        };
    }

    private static void ValidateFailureEnvelope(
        ReadOnlySpan<byte> transport,
        FrontendFailure failure,
        string id)
    {
        Check(transport.Length > 1 && transport[^1] == (byte)'\n', id + "_TRANSPORT");
        using JsonDocument document = JsonDocument.Parse(transport[..^1].ToArray());
        JsonElement root = document.RootElement;
        Equal("mpk.frontend.cli.v1", Text(root, "schema"), id + "_SCHEMA");
        Equal(FrontendDiagnosticRegistry.StatusText(failure.Status), Text(root, "status"), id + "_PUBLIC_STATUS");
        Equal(failure.Phase, Text(root, "phase"), id + "_PUBLIC_PHASE");
        foreach (string forbidden in new[] { "ir", "source_map", "source_manifest" })
        {
            Check(!root.TryGetProperty(forbidden, out _), id + "_ARTIFACT_" + forbidden);
        }

        JsonElement populated = Property(
            root,
            failure.Status == FrontendStatus.Rejected ? "rejected_features" : "diagnostics");
        JsonElement empty = Property(
            root,
            failure.Status == FrontendStatus.Rejected ? "diagnostics" : "rejected_features");
        Check(populated.GetArrayLength() > 0 && empty.GetArrayLength() == 0, id + "_ISSUES");
    }

    private static void ValidateDiagnosticRegistry(JsonElement profile)
    {
        JsonElement vectors = Property(profile, "diagnostic_registry");
        Equal(44, vectors.GetArrayLength(), "DIAGNOSTIC_VECTOR_COUNT");
        Equal(44, FrontendDiagnosticRegistry.Definitions.Count, "DIAGNOSTIC_IMPLEMENTATION_COUNT");
        for (int index = 0; index < vectors.GetArrayLength(); index++)
        {
            JsonElement vector = vectors[index];
            FrontendDiagnosticDefinition implementation = FrontendDiagnosticRegistry.Definitions[index];
            Equal(Text(vector, "code"), implementation.Code, "DIAGNOSTIC_CODE_" + index);
            Equal(Text(vector, "phase"), implementation.Phase, "DIAGNOSTIC_PHASE_" + index);
            Equal(
                Text(vector, "status"),
                FrontendDiagnosticRegistry.StatusText(implementation.Status),
                "DIAGNOSTIC_STATUS_" + index);
            string ownerPhase = implementation.Phase == "owner-phase"
                ? "lowering"
                : implementation.Phase;
            FrontendDiagnosticRegistry.Validate(
                implementation.Code,
                implementation.Status,
                ownerPhase);
            if (implementation.Phase != "owner-phase")
            {
                string wrongPhase = implementation.Phase == "capture" ? "source" : "capture";
                ExpectInvalidOperation(
                    () => FrontendDiagnosticRegistry.Validate(
                        implementation.Code,
                        implementation.Status,
                        wrongPhase),
                    "DIAGNOSTIC_WRONG_PHASE_" + index);
            }
        }

        ExpectInvalidOperation(
            () => FrontendFailure.WithIssues(
                FrontendStatus.Rejected,
                "subset",
                "CSHARP_SUBSET_DECLARATION",
                new[]
                {
                    new FrontendIssue(
                        "CSHARP_SUBSET_DECLARATION",
                        "host detail must not escape"),
                }),
            "DIAGNOSTIC_MESSAGE_REDACTION");
        ExpectInvalidOperation(
            () => FrontendDiagnosticRegistry.Find("CSHARP_UNKNOWN"),
            "DIAGNOSTIC_UNKNOWN_CODE");
    }

    private static string ValidateDiagnosticNormalization(string referencePackRoot)
    {
        const string source = "namespace Vector;\npublic static class Case { public static int F() { return Missing; } }\n";
        Selection selection = SelectionCodec.Validate(new RawSelection(
            "diagnostic-vector",
            new[] { SourcePath },
            new[] { "contracts/case.json" },
            new[] { "Vector.Case::F()->i32" }));
        CapturedSourceSet sources = Sources(SourcePath, source);
        RoslynSourceSession parsed = RoslynSessionFactory.Parse(selection, sources);
        FrontendFailure failure;
        try
        {
            _ = RoslynSessionFactory.Compile(selection, parsed, referencePackRoot);
            throw new VectorFailure("DIAGNOSTIC_ACCEPTED");
        }
        catch (FrontendFailure observed)
        {
            failure = observed;
        }

        Equal("CSHARP_SOURCE_DIAGNOSTIC", failure.Code, "DIAGNOSTIC_CODE");
        Check(failure.Issues.Count > 0, "DIAGNOSTIC_ISSUES");
        foreach (FrontendIssue issue in failure.Issues)
        {
            Check(issue.Message.StartsWith("C# compiler diagnostic CS", StringComparison.Ordinal), "DIAGNOSTIC_MESSAGE");
            Check(!issue.Message.Contains("Missing", StringComparison.Ordinal), "DIAGNOSTIC_REDACTION");
        }

        var invalid = new DiagnosticDescriptor(
            "BAD1",
            "bad",
            "bad",
            "bad",
            DiagnosticSeverity.Hidden,
            isEnabledByDefault: true);
        Diagnostic hidden = Diagnostic.Create(invalid, Location.None);
        try
        {
            _ = FrontendDiagnosticNormalizer.Normalize(
                "metadata",
                "CSHARP_SOURCE_DIAGNOSTIC",
                ImmutableArray.Create(hidden),
                parsed.SourceTexts,
                parsed.SyntaxTrees);
            throw new VectorFailure("DIAGNOSTIC_ID_ACCEPTED");
        }
        catch (FrontendFailure adapter)
        {
            Equal("CSHARP_TOOLCHAIN_ADAPTER", adapter.Code, "DIAGNOSTIC_ID_ADAPTER");
        }

        SyntaxTree tree = parsed.SyntaxTrees[0];
        SyntaxTree externalTree = CSharpSyntaxTree.ParseText(
            SourceText.From("class External {}\n", StrictUtf8),
            CSharpParseOptions.Default,
            "src/External.cs");
        FrontendFailure sorted = FrontendDiagnosticNormalizer.Normalize(
                "metadata",
                "CSHARP_SOURCE_DIAGNOSTIC",
                ImmutableArray.Create(
                    VectorDiagnostic("CS0001", DiagnosticSeverity.Error, "e\u0301", tree, 0, 4),
                    VectorDiagnostic("CS0002", DiagnosticSeverity.Warning, "z", tree, 0, 2),
                    VectorDiagnostic("CS0003", DiagnosticSeverity.Hidden, "hidden", tree, 1, 1),
                    VectorDiagnostic("CS0006", DiagnosticSeverity.Info, "info", tree, 2, 1),
                    VectorDiagnostic("CS0004", DiagnosticSeverity.Error, "external", externalTree, 0, 1),
                    VectorDiagnostic("CS0005", DiagnosticSeverity.Error, "zero", tree, 2, 0)),
                parsed.SourceTexts,
                parsed.SyntaxTrees)
            ?? throw new VectorFailure("DIAGNOSTIC_SORT_EMPTY");
        Equal(4, sorted.Issues.Count, "DIAGNOSTIC_SORT_COUNT");
        Equal(
            "C# compiler diagnostic CS0004,C# compiler diagnostic CS0005,"
                + "C# compiler diagnostic CS0001,C# compiler diagnostic CS0002",
            string.Join(',', sorted.Issues.Select(issue => issue.Message)),
            "DIAGNOSTIC_PUBLIC_SORT");
        Check(sorted.Issues[0].Span is null, "DIAGNOSTIC_EXTERNAL_SPAN");
        Check(sorted.Issues[1].Span is null, "DIAGNOSTIC_ZERO_SPAN");
        Equal(4, sorted.Issues[2].Span?.End ?? -1, "DIAGNOSTIC_FIRST_MAPPED_END");
        Equal(2, sorted.Issues[3].Span?.End ?? -1, "DIAGNOSTIC_SECOND_MAPPED_END");

        byte[] projection = EmissionCanonical.Write(writer =>
        {
            writer.WriteStartArray();
            foreach (FrontendIssue issue in failure.Issues)
            {
                writer.WriteStartObject();
                writer.WriteString("code", issue.Code);
                writer.WriteString("message", issue.Message);
                if (issue.Span is not null)
                {
                    writer.WriteString("path", issue.Span.NormalizedPath);
                    writer.WriteNumber("start", issue.Span.Start);
                    writer.WriteNumber("end", issue.Span.End);
                }

                writer.WriteEndObject();
            }

            writer.WriteEndArray();
        });
        return RawSha256(projection);
    }

    private static Diagnostic VectorDiagnostic(
        string id,
        DiagnosticSeverity severity,
        string message,
        SyntaxTree tree,
        int start,
        int length)
    {
        var descriptor = new DiagnosticDescriptor(
            id,
            "vector",
            message,
            "vector",
            severity,
            isEnabledByDefault: true);
        return Diagnostic.Create(descriptor, Location.Create(tree, new TextSpan(start, length)));
    }

    private static List<LimitResult> ValidateLimits(JsonElement profile)
    {
        var results = new List<LimitResult>();
        JsonElement vectors = Property(profile, "limit_cases");
        Equal(32, vectors.GetArrayLength(), "LIMIT_VECTOR_COUNT");
        Equal(32, FrontendLimits.Definitions.Count, "LIMIT_IMPLEMENTATION_COUNT");
        for (int index = 0; index < vectors.GetArrayLength(); index++)
        {
            JsonElement vector = vectors[index];
            string id = Text(vector, "id");
            ulong maximum = Integer(vector, "maximum");
            string code = Text(vector, "code");
            FrontendLimitDefinition definition = FrontendLimits.Definitions[index];
            Equal(id, definition.Id, "LIMIT_ID_" + index);
            Equal(maximum, definition.Maximum, "LIMIT_MAXIMUM_" + index);
            Equal(code, definition.Code, "LIMIT_CODE_" + index);
            Equal(maximum, FrontendLimits.Validate(id, maximum, LimitPhase(id)), "LIMIT_BOUNDARY_" + id);
            try
            {
                _ = FrontendLimits.Validate(id, checked(maximum + 1), LimitPhase(id));
                throw new VectorFailure("LIMIT_PLUS_ONE_" + id);
            }
            catch (FrontendFailure failure)
            {
                Equal(code, failure.Code, "LIMIT_FAILURE_" + id);
                Equal(
                    definition.Disposition == FrontendLimitDisposition.Rejected
                        ? FrontendStatus.Rejected
                        : FrontendStatus.FrontendError,
                    failure.Status,
                    "LIMIT_STATUS_" + id);
                results.Add(new LimitResult(
                    id,
                    maximum,
                    code,
                    FrontendDiagnosticRegistry.StatusText(failure.Status)));
            }

            try
            {
                _ = FrontendLimits.Add(id, ulong.MaxValue, 1, LimitPhase(id));
                throw new VectorFailure("LIMIT_OVERFLOW_" + id);
            }
            catch (FrontendFailure overflow)
            {
                Equal(code, overflow.Code, "LIMIT_OVERFLOW_CODE_" + id);
            }
        }

        FrontendLimits.ValidateArguments(new[]
        {
            new string('a', FrontendLimits.FrontendArgumentBytesMaximum - 1),
        });
        ExpectFrontendFailure(
            () => FrontendLimits.ValidateArguments(new[]
            {
                new string('a', FrontendLimits.FrontendArgumentBytesMaximum),
            }),
            "CSHARP_LIMIT_FRONTEND_ARGUMENT_BYTES",
            "LIMIT_ARGUMENT_PLUS_ONE");

        var eachMessage = new FrontendIssueCollector("metadata");
        eachMessage.Add(new FrontendIssue(
            "CSHARP_SOURCE_DIAGNOSTIC",
            new string('x', FrontendLimits.DiagnosticMessageBytesEachMaximum)));
        Equal(1, eachMessage.Freeze().Length, "LIMIT_MESSAGE_EACH_BOUNDARY");
        ExpectFrontendFailure(
            () => new FrontendIssueCollector("metadata").Add(new FrontendIssue(
                "CSHARP_SOURCE_DIAGNOSTIC",
                new string('x', FrontendLimits.DiagnosticMessageBytesEachMaximum + 1))),
            "CSHARP_FRONTEND_DIAGNOSTIC_BUDGET",
            "LIMIT_MESSAGE_EACH_PLUS_ONE");

        var totalMessages = new FrontendIssueCollector("metadata");
        int fullMessages = FrontendLimits.DiagnosticMessageBytesTotalMaximum
            / FrontendLimits.DiagnosticMessageBytesEachMaximum;
        for (int index = 0; index < fullMessages; index++)
        {
            totalMessages.Add(new FrontendIssue(
                "CSHARP_SOURCE_DIAGNOSTIC",
                new string('x', FrontendLimits.DiagnosticMessageBytesEachMaximum)));
        }

        ExpectFrontendFailure(
            () => totalMessages.Add(new FrontendIssue("CSHARP_SOURCE_DIAGNOSTIC", "x")),
            "CSHARP_FRONTEND_DIAGNOSTIC_BUDGET",
            "LIMIT_MESSAGE_TOTAL_PLUS_ONE");
        Equal(fullMessages, totalMessages.Freeze().Length, "LIMIT_MESSAGE_TOTAL_RETENTION");

        var issueCount = new FrontendIssueCollector("metadata");
        for (int index = 0; index < FrontendLimits.NormalizedIssuesMaximum; index++)
        {
            issueCount.Add(new FrontendIssue("CSHARP_SOURCE_DIAGNOSTIC", "x"));
        }

        ExpectFrontendFailure(
            () => issueCount.Add(new FrontendIssue("CSHARP_SOURCE_DIAGNOSTIC", "x")),
            "CSHARP_FRONTEND_DIAGNOSTIC_BUDGET",
            "LIMIT_ISSUE_COUNT_PLUS_ONE");
        Equal(
            FrontendLimits.NormalizedIssuesMaximum,
            issueCount.Freeze().Length,
            "LIMIT_ISSUE_COUNT_RETENTION");

        FrontendIssue[] excessIssues = Enumerable.Range(
                0,
                FrontendLimits.NormalizedIssuesMaximum + 1)
            .Select(_ => new FrontendIssue(
                "CSHARP_SOURCE_PARSE",
                "C# source is invalid"))
            .ToArray();
        FrontendFailure excessFailure = FrontendFailure.WithIssues(
            FrontendStatus.SourceError,
            "source",
            "CSHARP_SOURCE_PARSE",
            excessIssues);
        byte[] excessEnvelope = CSharpFrontendFailureEmitter.Emit(
            FailureRequest(profile),
            excessFailure,
            out int excessExit);
        Equal(1, excessExit, "LIMIT_DIAGNOSTIC_BUDGET_EXIT");
        using (JsonDocument document = JsonDocument.Parse(excessEnvelope.AsMemory(0, excessEnvelope.Length - 1)))
        {
            Equal("frontend-error", Text(document.RootElement, "status"), "LIMIT_DIAGNOSTIC_BUDGET_STATUS");
            JsonElement diagnostics = Property(document.RootElement, "diagnostics");
            Equal(1, diagnostics.GetArrayLength(), "LIMIT_DIAGNOSTIC_BUDGET_COUNT");
            Equal(
                "CSHARP_FRONTEND_DIAGNOSTIC_BUDGET",
                Text(diagnostics[0], "code"),
                "LIMIT_DIAGNOSTIC_BUDGET_CODE");
        }

        return results;
    }

    private static List<PrecedenceResult> ValidatePrecedence(JsonElement profile)
    {
        var results = new List<PrecedenceResult>();
        JsonElement vectors = Property(profile, "precedence_cases");
        Equal(12, vectors.GetArrayLength(), "PRECEDENCE_COUNT");
        foreach (JsonElement vector in vectors.EnumerateArray())
        {
            string id = Text(vector, "id");
            string winner = Text(vector, "winner");
            var failures = new List<FrontendFailure>();
            foreach (JsonElement codeValue in Property(vector, "coexisting").EnumerateArray())
            {
                string code = codeValue.GetString() ?? throw new VectorFailure("PRECEDENCE_CODE");
                FrontendDiagnosticDefinition definition = FrontendDiagnosticRegistry.Find(code);
                string phase = PrecedencePhase(id, code, definition.Phase);
                failures.Add(CreateFailure(
                    FrontendDiagnosticRegistry.StatusText(definition.Status),
                    phase,
                    code));
            }

            string actual = FrontendPrecedence.Select(failures).Code;
            Equal(winner, actual, id);
            results.Add(new PrecedenceResult(id, actual));
        }

        return results;
    }

    private static List<HashResult> ValidateHashes(JsonElement profile)
    {
        var results = new List<HashResult>();
        foreach (JsonElement vector in Property(profile, "hash_cases").EnumerateArray())
        {
            string id = Text(vector, "id");
            JsonElement source = ResolvePointer(profile, Text(vector, "source_pointer"));
            string? excluded = Property(vector, "excluded_field").ValueKind == JsonValueKind.Null
                ? null
                : Text(vector, "excluded_field");
            byte[] payload = CanonicalElement(source, excluded);
            string domain = Text(vector, "domain");
            string hash = EmissionCanonical.Hash(domain, payload);
            int preimageLength = checked(Encoding.ASCII.GetByteCount(domain) + 1 + payload.Length);
            Equal((ulong)payload.Length, Integer(vector, "expected_payload_utf8_length"), id + "_PAYLOAD");
            Equal((ulong)preimageLength, Integer(vector, "expected_preimage_length"), id + "_PREIMAGE");
            Equal(Text(vector, "expected_sha256"), hash, id + "_HASH");
            results.Add(new HashResult(id, payload.Length, preimageLength, hash));
        }

        return results;
    }

    private static List<SemanticRowResult> ValidateSemanticRows(JsonElement profile)
    {
        JsonElement rows = Property(profile, "semantic_rows");
        Equal(34, rows.GetArrayLength(), "SEMANTIC_ROW_COUNT");
        var observed = new HashSet<string>(StringComparer.Ordinal);
        var results = new List<SemanticRowResult>();
        foreach (JsonElement row in rows.EnumerateArray())
        {
            string id = Text(row, "row");
            Check(observed.Add(id), "SEMANTIC_ROW_DUPLICATE");
            Check(id.Length == 3 && id[0] == 'M', "SEMANTIC_ROW_ID");
            results.Add(new SemanticRowResult(id, Text(row, "disposition")));
        }

        for (int index = 1; index <= 34; index++)
        {
            Check(observed.Contains("M" + index.ToString("D2", CultureInfo.InvariantCulture)), "SEMANTIC_ROW_MISSING");
        }

        return results;
    }

    private static void ValidateOperationProjection(
        LoweredClosure closure,
        string method,
        JsonElement expected,
        string id)
    {
        LoweredFunction function = closure.Functions.Single(candidate => candidate.Id == method);
        var actual = new List<string>();
        foreach (LoweredBlock block in function.Blocks)
        {
            foreach (LoweredBinding _ in block.Parameters)
            {
                actual.Add("block_parameter");
            }

            actual.AddRange(block.Instructions.Select(LoweringValidator.ProfileOperation));
            if (block.Terminator.Kind == LoweredTerminatorKind.Branch)
            {
                actual.Add("Branch");
            }
            else if (block.Terminator.Kind == LoweredTerminatorKind.Return)
            {
                actual.Add("Return");
            }
        }

        int cursor = 0;
        foreach (JsonElement operation in expected.EnumerateArray())
        {
            string required = operation.GetString() ?? throw new VectorFailure(id + "_OPERATION_VALUE");
            while (cursor < actual.Count && actual[cursor] != required)
            {
                cursor++;
            }

            Check(cursor < actual.Count, id + "_OPERATION_" + required);
            cursor++;
        }
    }

    private static void ValidateRequiredChecks(
        LoweredClosure closure,
        string method,
        JsonElement expected,
        string id)
    {
        LoweredFunction function = closure.Functions.Single(candidate => candidate.Id == method);
        var actual = function.RequiredChecks
            .Select(required => CheckProjection(required.Check))
            .ToList();
        actual.AddRange(function.Blocks
            .SelectMany(block => block.Instructions)
            .Where(instruction => instruction.Kind == LoweredInstructionKind.CallStatic)
            .Select(instruction => "callee_contract_hash|" + instruction.Function));
        var expectedProjection = expected.EnumerateArray().Select(CheckProjection).ToArray();
        Equal(string.Join(',', expectedProjection), string.Join(',', actual), id + "_CHECKS");
    }

    private static string CheckProjection(LoweredSafetyCheck check)
    {
        string kind = check.Kind switch
        {
            LoweredSafetyCheckKind.IntegerNoOverflow => "integer_no_overflow",
            LoweredSafetyCheckKind.DivisorNonzero => "divisor_nonzero",
            LoweredSafetyCheckKind.SignedDivremRepresentable => "signed_divrem_representable",
            _ => throw new VectorFailure("CHECK_KIND"),
        };
        return kind + "|" + check.Operation.ToString().ToLowerInvariant() + "|"
            + check.Width.ToString(CultureInfo.InvariantCulture) + "|"
            + (check.Signed ? "true" : "false");
    }

    private static string CheckProjection(JsonElement check)
    {
        string kind = Text(check, "kind");
        if (kind == "callee_contract_hash")
        {
            return kind + "|" + Text(check, "callee");
        }

        return kind + "|" + Text(check, "operation") + "|"
            + Integer(check, "width").ToString(CultureInfo.InvariantCulture) + "|"
            + (Property(check, "signed").GetBoolean() ? "true" : "false");
    }

    private static void ValidateSuccessEnvelope(ReadOnlySpan<byte> transport, string id)
    {
        Check(transport.Length > 1 && transport[^1] == (byte)'\n', id + "_SUCCESS_TRANSPORT");
        using JsonDocument document = JsonDocument.Parse(transport[..^1].ToArray());
        JsonElement root = document.RootElement;
        Equal("ir-lowered", Text(root, "status"), id + "_SUCCESS_STATUS");
        Equal("emission", Text(root, "phase"), id + "_SUCCESS_PHASE");
        Equal(0, Property(root, "diagnostics").GetArrayLength(), id + "_SUCCESS_DIAGNOSTICS");
        Equal(0, Property(root, "rejected_features").GetArrayLength(), id + "_SUCCESS_REJECTIONS");
        _ = Property(root, "ir");
        _ = Property(root, "source_map");
        _ = Property(root, "source_manifest");
    }

    private static byte[] CanonicalElement(JsonElement value, string? excludedField)
    {
        return EmissionCanonical.Write(writer => WriteElement(writer, value, excludedField));
    }

    private static void WriteElement(Utf8JsonWriter writer, JsonElement value, string? excludedField)
    {
        switch (value.ValueKind)
        {
            case JsonValueKind.Object:
                writer.WriteStartObject();
                foreach (JsonProperty property in value.EnumerateObject()
                    .Where(property => excludedField is null || property.Name != excludedField)
                    .OrderBy(property => property.Name, StringComparer.Ordinal))
                {
                    writer.WritePropertyName(property.Name);
                    WriteElement(writer, property.Value, null);
                }

                writer.WriteEndObject();
                break;
            case JsonValueKind.Array:
                writer.WriteStartArray();
                foreach (JsonElement item in value.EnumerateArray())
                {
                    WriteElement(writer, item, null);
                }

                writer.WriteEndArray();
                break;
            case JsonValueKind.String:
                writer.WriteStringValue(value.GetString());
                break;
            case JsonValueKind.Number:
                writer.WriteRawValue(value.GetRawText(), skipInputValidation: false);
                break;
            case JsonValueKind.True:
                writer.WriteBooleanValue(true);
                break;
            case JsonValueKind.False:
                writer.WriteBooleanValue(false);
                break;
            case JsonValueKind.Null:
                writer.WriteNullValue();
                break;
            default:
                throw new VectorFailure("CANONICAL_JSON_KIND");
        }
    }

    private static JsonElement ResolvePointer(JsonElement root, string pointer)
    {
        JsonElement current = root;
        foreach (string component in pointer.Split('/', StringSplitOptions.RemoveEmptyEntries))
        {
            current = Property(current, component.Replace("~1", "/", StringComparison.Ordinal)
                .Replace("~0", "~", StringComparison.Ordinal));
        }

        return current;
    }

    private static LowerRequest FailureRequest(JsonElement profile)
    {
        JsonElement value = Property(Property(profile, "case_harness"), "baseline_selection");
        JsonElement selection = Property(value, "value");
        return Request(new RawSelection(
            Text(selection, "compilation"),
            Strings(selection, "sources"),
            Strings(selection, "contracts"),
            Strings(selection, "methods")));
    }

    private static LowerRequest Request(RawSelection selection)
    {
        return new LowerRequest(
            FrontendConstants.SourceRoot,
            selection,
            new ReleaseArguments(
                "csharp-frontend-vector",
                new string('1', 64),
                new string('2', 64),
                "csharp-toolchain-vector",
                new string('3', 64)));
    }

    private static CapturedSourceSet Sources(string path, string source)
    {
        return new CapturedSourceSet(new[] { new CapturedSourceText(path, source) });
    }

    private static string DefaultContract(string method)
    {
        return "{\"abrupt_completion\":\"forbidden\",\"ensures\":[{\"bool\":true}],"
            + "\"method\":" + JsonSerializer.Serialize(method)
            + ",\"modifies\":[],\"requires\":[],\"schema\":\"mpk.csharp.contract.v0\","
            + "\"semantic_profile\":\"mpk.csharp.scalar.v0\",\"termination\":\"total\"}\n";
    }

    private static string Owner(string id, string phase)
    {
        if (id.StartsWith("contract.", StringComparison.Ordinal))
        {
            return "csharp_contracts";
        }

        if (id.StartsWith("lowering.", StringComparison.Ordinal))
        {
            return "csharp_lowering";
        }

        if (id.StartsWith("source_map.", StringComparison.Ordinal))
        {
            return "csharp_emission";
        }

        return phase switch
        {
            "capture" => "csharp_capture",
            "source" or "metadata" or "release" => "csharp_roslyn_session",
            "typecheck" or "subset" => "csharp_subset",
            "lowering" => "csharp_lowering",
            "emission" => "csharp_emission",
            _ => "csharp_frontend_vectors",
        };
    }

    private static string LimitPhase(string id)
    {
        if (id.StartsWith("diagnostic_", StringComparison.Ordinal) || id == "normalized_issues")
        {
            return "metadata";
        }

        if (id.StartsWith("private_runtime_", StringComparison.Ordinal))
        {
            return "release";
        }

        if (id is "vir_canonical_bytes" or "source_map_canonical_bytes"
            or "source_manifest_canonical_bytes" or "frontend_stdout" or "frontend_stderr")
        {
            return "emission";
        }

        if (id.StartsWith("contract_", StringComparison.Ordinal))
        {
            return "subset";
        }

        if (id is "method_closure" or "syntax_nodes" or "operations_per_method"
            or "operations_per_closure" or "cfg_blocks_per_method" or "cfg_blocks_per_closure")
        {
            return "subset";
        }

        return "capture";
    }

    private static string PrecedencePhase(string id, string code, string registered)
    {
        if (registered != "owner-phase")
        {
            return registered;
        }

        if (code == "CSHARP_FRONTEND_OUTPUT_LIMIT")
        {
            return "emission";
        }

        if (code == "CSHARP_TOOLCHAIN_OPTIONS")
        {
            return id.Contains("encoding_before", StringComparison.Ordinal) ? "source" : "metadata";
        }

        return "lowering";
    }

    private static string RawSha256(ReadOnlySpan<byte> value)
    {
        return Convert.ToHexString(SHA256.HashData(value)).ToLowerInvariant();
    }

    private static JsonElement Property(JsonElement value, string name)
    {
        return value.TryGetProperty(name, out JsonElement result)
            ? result
            : throw new VectorFailure("MISSING_" + name.ToUpperInvariant());
    }

    private static string Text(JsonElement value, string name)
    {
        return Property(value, name).GetString()
            ?? throw new VectorFailure("TEXT_" + name.ToUpperInvariant());
    }

    private static ulong Integer(JsonElement value, string name)
    {
        return Property(value, name).GetUInt64();
    }

    private static string[] Strings(JsonElement value, string name)
    {
        return Property(value, name).EnumerateArray()
            .Select(item => item.GetString() ?? throw new VectorFailure("STRING_ARRAY"))
            .ToArray();
    }

    private static void Check(bool condition, string code)
    {
        if (!condition)
        {
            throw new VectorFailure(code);
        }
    }

    private static void Equal<T>(T expected, T actual, string code)
    {
        if (!EqualityComparer<T>.Default.Equals(expected, actual))
        {
            throw new VectorFailure(code);
        }
    }

    private static void EqualBytes(ReadOnlySpan<byte> expected, ReadOnlySpan<byte> actual, string code)
    {
        if (!expected.SequenceEqual(actual))
        {
            throw new VectorFailure(code);
        }
    }

    private static void ExpectInvalidOperation(Action action, string code)
    {
        try
        {
            action();
        }
        catch (InvalidOperationException)
        {
            return;
        }

        throw new VectorFailure(code);
    }

    private static void ExpectFrontendFailure(Action action, string expectedCode, string code)
    {
        try
        {
            action();
        }
        catch (FrontendFailure failure)
        {
            Equal(expectedCode, failure.Code, code);
            return;
        }

        throw new VectorFailure(code);
    }

    private sealed record AcceptedExecution(
        Selection Selection,
        RoslynCompilationSession Compilation,
        LoweredClosure Lowered,
        EmittedFrontendSuccess Success);

    private sealed record AcceptedEvaluation(
        AcceptedResult Result,
        DifferentialResult Differential);

    private sealed record AcceptedResult(
        string Id,
        string EnvelopeSha256,
        string VirSha256,
        string SourceMapSha256,
        string SourceManifestSha256);

    private sealed record RejectedResult(
        string Id,
        string Status,
        string Phase,
        string Code,
        int Exit,
        string Owner,
        string EnvelopeSha256,
        byte[] Envelope);

    private sealed record HashResult(
        string Id,
        int PayloadLength,
        int PreimageLength,
        string Sha256);

    private sealed record LimitResult(
        string Id,
        ulong Maximum,
        string Code,
        string PlusOneStatus);

    private sealed record PrecedenceResult(string Id, string Winner);

    private sealed record SemanticRowResult(string Row, string Disposition);

    private sealed record DifferentialResult(
        string Id,
        int CaseCount,
        string OutcomesSha256);

    private sealed record FuzzResult(
        string Id,
        int SeedCount,
        int MutationCount,
        string OutcomesSha256);
}

internal sealed class VectorFailure : Exception
{
    internal VectorFailure(string code)
        : base(code)
    {
        Code = code;
    }

    internal string Code { get; }
}
