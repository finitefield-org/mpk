using System;
using System.Collections.Generic;
using System.Globalization;
using System.Linq;
using System.Text;
using System.Text.Json;

namespace Mpk.CSharp2Vir;

internal static class EmissionHarness
{
    internal const string SourcePath = "src/Case.cs";
    internal const string RootMethod = "Vector.Calls::F(i32)->i32";
    internal const string CalleeMethod = "Vector.Calls::G(i32)->i32";
    internal const string Source =
        "namespace Vector;\n"
        + "public static class Calls\n"
        + "{\n"
        + "    public static int F(int x)\n"
        + "    {\n"
        + "        int y = G(x);\n"
        + "        y = unchecked(y + 1);\n"
        + "        if (y > 0)\n"
        + "        {\n"
        + "            y = unchecked(y - 1);\n"
        + "        }\n"
        + "\n"
        + "        return y;\n"
        + "    }\n"
        + "\n"
        + "    private static int G(int x) { return checked(x * 2); }\n"
        + "}\n";
    internal const string RootContractPath = "contracts/f.json";
    internal const string CalleeContractPath = "contracts/g.json";
    internal static readonly byte[] SourceBytes = Encoding.UTF8.GetBytes(Source);
    internal static readonly byte[] RootContractBytes = ContractBytes(RootMethod);
    internal static readonly byte[] CalleeContractBytes = ContractBytes(CalleeMethod);

    public static int Main(string[] args)
    {
        if (args.Length != 2
            || (args[0] != "self-test" && args[0] != "emit"))
        {
            Console.Error.Write("CSHARP_EMISSION_TEST_USAGE\n");
            return 1;
        }

        try
        {
            if (args[0] == "emit")
            {
                EmissionFixture fixture = BuildFixture(args[1]);
                System.IO.Stream output = Console.OpenStandardOutput();
                output.Write(fixture.Success.EnvelopeBytes);
                output.Flush();
                return 0;
            }

            StaticCallsAreCalleeFirstAndLeftToRight(args[1]);
            StableIdsIgnoreRoslynOrdinalsAndCaptures(args[1]);
            SourceMapVectorsAreExact();
            ArtifactsAreCanonicalCompleteAndDeterministic(args[1]);
            OwnedCompiledProfilesAreExact();
            SourceMapFailuresAreClosed(args[1]);
            return 0;
        }
        catch (HarnessFailure failure)
        {
            Console.Error.Write("CSHARP_EMISSION_TEST_" + failure.Code + "\n");
            return 1;
        }
        catch (Exception)
        {
            Console.Error.Write("CSHARP_EMISSION_TEST_UNEXPECTED\n");
            return 1;
        }
    }

    private static void StaticCallsAreCalleeFirstAndLeftToRight(string referencePackRoot)
    {
        const string source =
            "namespace Vector;\npublic static class Order\n{\n"
            + "    public static int F(int x, int y) { return H(G(x), G(y)); }\n"
            + "    private static int G(int x) { return x; }\n"
            + "    private static int H(int x, int y) { return unchecked(x + y); }\n"
            + "}\n";
        string[] methods =
        {
            "Vector.Order::F(i32,i32)->i32",
            "Vector.Order::G(i32)->i32",
            "Vector.Order::H(i32,i32)->i32",
        };
        EmissionFixture fixture = Build(
            referencePackRoot,
            "call-order",
            source,
            methods[0],
            methods);
        Equal(
            string.Join(",", new[] { methods[1], methods[2], methods[0] }),
            string.Join(',', fixture.Lowered.Functions.Select(function => function.Id)),
            "CALLEE_FIRST");
        LoweredFunction caller = fixture.Function(methods[0]);
        LoweredInstruction[] calls = caller.Blocks
            .SelectMany(block => block.Instructions)
            .Where(instruction => instruction.Kind == LoweredInstructionKind.CallStatic)
            .ToArray();
        Equal(3, calls.Length, "CALL_COUNT");
        Equal("t0,t1,t2", string.Join(',', calls.Select(call => call.Id)), "CALL_IDS");
        Equal(
            string.Join(",", new[] { methods[1], methods[1], methods[2] }),
            string.Join(',', calls.Select(call => call.Function)),
            "CALL_ORDER");
        foreach (LoweredInstruction call in calls)
        {
            LoweredFunction callee = fixture.Function(
                call.Function ?? throw new HarnessFailure("CALL_TARGET"));
            Equal(callee.ContractHash, call.ContractHash, "CALL_CONTRACT_HASH");
        }

        Check(caller.Features.Contains(LoweredFeature.CallStatic), "CALL_FEATURE");
        Equal("CallStatic", LoweringValidator.ProfileOperation(calls[0]), "CALL_MAPPING");
        ExpectFailure(
            () => LoweringValidator.Validate(ReplaceCallHash(fixture.Lowered, caller.Id, calls[0].Id)),
            FrontendStatus.Rejected,
            "lowering",
            "CSHARP_LOWERING_OPERATION");

        LoweredFunction signatureFunction = fixture.Function(methods[1]);
        var signatureMismatch = new LoweredFunction(
            "Vector.Order::G(u32)->i32",
            signatureFunction.Name,
            signatureFunction.ContractHash,
            signatureFunction.Origin,
            signatureFunction.Parameters.ToArray(),
            signatureFunction.Results.ToArray(),
            signatureFunction.Locals.ToArray(),
            signatureFunction.Blocks.ToArray(),
            signatureFunction.RequiredChecks.ToArray(),
            signatureFunction.Features.ToArray());
        ExpectFailure(
            () => LoweringValidator.Validate(signatureMismatch),
            FrontendStatus.Rejected,
            "lowering",
            "CSHARP_LOWERING_OPERATION");
    }

    private static void StableIdsIgnoreRoslynOrdinalsAndCaptures(string referencePackRoot)
    {
        const string compact =
            "namespace Vector;\npublic static class Stable\n{\n"
            + "    public static int F(bool c, int x, int y) { return G(c ? x : y); }\n"
            + "    private static int G(int x) { return x; }\n"
            + "}\n";
        const string shifted =
            "// 😀 UTF-16 and trivia must not become IDs.\n"
            + "namespace Vector;\npublic static class Stable\n{\n"
            + "    public static int F(bool c, int x, int y)\n"
            + "    {\n"
            + "        return G(c ? x : y);\n"
            + "    }\n"
            + "    private static int G(int x) { return x; }\n"
            + "}\n";
        string[] methods =
        {
            "Vector.Stable::F(bool,i32,i32)->i32",
            "Vector.Stable::G(i32)->i32",
        };
        EmissionFixture left = Build(referencePackRoot, "stable-ids", compact, methods[0], methods);
        EmissionFixture right = Build(referencePackRoot, "stable-ids", shifted, methods[0], methods);
        Equal(IdFingerprint(left.Lowered), IdFingerprint(right.Lowered), "STABLE_IDS");
        Check(
            !left.Success.SourceMap.CanonicalBytes.SequenceEqual(
                right.Success.SourceMap.CanonicalBytes),
            "ORIGINS_REMAIN_SOURCE_SENSITIVE");
    }

    private static void SourceMapVectorsAreExact()
    {
        CheckMap("a\nx\n", 2, 3, 2, 3, 1, 0, 1, 1, "MAP_ASCII");
        CheckMap("// é\n", 3, 4, 3, 5, 0, 3, 0, 4, "MAP_BMP");
        CheckMap("// 😀\n", 3, 5, 3, 7, 0, 3, 0, 5, "MAP_SURROGATE");
        ExpectFailure(
            () => CSharpSourceMapper.MapVector("// 😀\n", 4, 5),
            FrontendStatus.FrontendError,
            "emission",
            "CSHARP_SOURCE_MAP_UTF16");
        ExpectFailure(
            () => CSharpSourceMapper.MapVector("x\n", 0, 0),
            FrontendStatus.FrontendError,
            "emission",
            "CSHARP_SOURCE_MAP_RANGE");
        ExpectFailure(
            () => CSharpSourceMapper.MapVector("x\n", 0, 3),
            FrontendStatus.FrontendError,
            "emission",
            "CSHARP_SOURCE_MAP_RANGE");
    }

    private static void ArtifactsAreCanonicalCompleteAndDeterministic(string referencePackRoot)
    {
        EmissionFixture first = BuildFixture(referencePackRoot);
        EmissionFixture second = BuildFixture(referencePackRoot);
        Check(first.Success.EnvelopeBytes.SequenceEqual(second.Success.EnvelopeBytes), "DETERMINISM");
        ReadOnlySpan<byte> transport = first.Success.EnvelopeBytes;
        Check(transport.Length > 1 && transport[^1] == (byte)'\n', "ENVELOPE_LF");
        Check(!transport[..^1].Contains((byte)'\n'), "ENVELOPE_SINGLE_LINE");

        using JsonDocument document = JsonDocument.Parse(transport[..^1].ToArray());
        JsonElement root = document.RootElement;
        Equal("mpk.frontend.cli.v1", root.GetProperty("schema").GetString(), "ENVELOPE_SCHEMA");
        Equal("ir-lowered", root.GetProperty("status").GetString(), "ENVELOPE_STATUS");
        Equal("emission", root.GetProperty("phase").GetString(), "ENVELOPE_PHASE");
        Equal(0, root.GetProperty("diagnostics").GetArrayLength(), "ENVELOPE_DIAGNOSTICS");
        Equal(0, root.GetProperty("rejected_features").GetArrayLength(), "ENVELOPE_REJECTIONS");

        JsonElement vir = root.GetProperty("ir").GetProperty("value");
        Equal(first.Success.Vir.Sha256, vir.GetProperty("vir_hash").GetString(), "VIR_HASH");
        JsonElement functions = vir.GetProperty("units")[0].GetProperty("functions");
        Equal(CalleeMethod, functions[0].GetProperty("id").GetString(), "VIR_CALLEE_FIRST");
        Equal(RootMethod, functions[1].GetProperty("id").GetString(), "VIR_CALLER_SECOND");
        JsonElement call = functions[1].GetProperty("blocks")[0]
            .GetProperty("instructions")[0];
        Equal("CallStatic", call.GetProperty("kind").GetString(), "VIR_CALL_KIND");
        Equal(CalleeMethod, call.GetProperty("function").GetString(), "VIR_CALL_TARGET");
        Equal(
            functions[0].GetProperty("contracts").GetProperty("contract_hash").GetString(),
            call.GetProperty("contract_hash").GetString(),
            "VIR_CALL_HASH_LINK");

        JsonElement sourceMap = root.GetProperty("source_map");
        int expectedEntries = 0;
        foreach (LoweredFunction function in first.Lowered.Functions)
        {
            expectedEntries++;
            foreach (LoweredBlock block in function.Blocks)
            {
                expectedEntries += block.Instructions.Count + 1;
            }
        }
        Equal(
            expectedEntries,
            sourceMap.GetProperty("entries").GetArrayLength(),
            "SOURCE_MAP_TOTAL");
        Check(expectedEntries > 5, "SOURCE_MAP_NONTRIVIAL");
        foreach (JsonElement entry in sourceMap.GetProperty("entries").EnumerateArray())
        {
            Equal("source", entry.GetProperty("origin").GetProperty("kind").GetString(), "SOURCE_ORIGIN");
        }

        JsonElement manifest = root.GetProperty("source_manifest");
        Equal(3, manifest.GetProperty("inputs").GetArrayLength(), "MANIFEST_INPUTS");
        Equal("contract", manifest.GetProperty("inputs")[0].GetProperty("kind").GetString(), "MANIFEST_ORDER_0");
        Equal("contract", manifest.GetProperty("inputs")[1].GetProperty("kind").GetString(), "MANIFEST_ORDER_1");
        Equal("source", manifest.GetProperty("inputs")[2].GetProperty("kind").GetString(), "MANIFEST_ORDER_2");
        Equal("compilation", manifest.GetProperty("units")[0].GetProperty("kind").GetString(), "MANIFEST_UNIT");
        Check(!manifest.TryGetProperty("vc_hash", out _), "MANIFEST_FRONTEND_STAGE");
    }

    private static void OwnedCompiledProfilesAreExact()
    {
        const string expected =
            "[{\"envelope\":{\"contract_id\":\"mpk.profile.manifest.csharp_scalar.v0\","
            + "\"profile_entry_sha256\":\"d4cc54a5364af848af845a9044d0f5aec962e6e509554e9a9b75a7a9f0b6e7ac\","
            + "\"value\":{\"input_kinds\":[\"contract\",\"source\"],\"source_extension\":\".cs\","
            + "\"unit_kind\":\"compilation\"}},\"field\":\"manifest\"},"
            + "{\"envelope\":{\"contract_id\":\"mpk.profile.source_map.csharp_scalar.v0\","
            + "\"profile_entry_sha256\":\"d4cc54a5364af848af845a9044d0f5aec962e6e509554e9a9b75a7a9f0b6e7ac\","
            + "\"value\":{\"encoding\":\"utf-8\",\"offset_unit\":\"utf8-byte\",\"synthetic_reasons\":[]}},"
            + "\"field\":\"source_map\"},{\"envelope\":{\"contract_id\":\"mpk.profile.vir.csharp_scalar.v0\","
            + "\"profile_entry_sha256\":\"d4cc54a5364af848af845a9044d0f5aec962e6e509554e9a9b75a7a9f0b6e7ac\","
            + "\"value\":{\"operation_profile_id\":\"mpk.csharp.vir_operations.v0\","
            + "\"source_map_profile_id\":\"mpk.csharp.source_map.v0\","
            + "\"vir_limit_profile_id\":\"mpk.vir.limits.v0\"}},\"field\":\"vir\"}]";
        Equal(
            expected,
            Encoding.UTF8.GetString(CSharpEmissionProfiles.CanonicalOwnedContracts()),
            "COMPILED_PROFILES");
    }

    private static void SourceMapFailuresAreClosed(string referencePackRoot)
    {
        EmissionFixture fixture = BuildFixture(referencePackRoot);
        var mapper = new CSharpSourceMapper(
            fixture.Selection,
            fixture.Sources,
            fixture.Compilation);
        ExpectFailure(
            () => mapper.Map(new LoweredOrigin("src/External.cs", 0, 1)),
            FrontendStatus.FrontendError,
            "emission",
            "CSHARP_SOURCE_MAP_EXTERNAL");
        ExpectFailure(
            () => mapper.Map(new LoweredOrigin(SourcePath, 0, 1, new object())),
            FrontendStatus.FrontendError,
            "emission",
            "CSHARP_SOURCE_MAP_EXTERNAL");
    }

    private static EmissionFixture BuildFixture(string referencePackRoot)
    {
        return Build(
            referencePackRoot,
            "call-case",
            Source,
            RootMethod,
            new[] { RootMethod, CalleeMethod },
            new[] { RootContractPath, CalleeContractPath },
            new[] { RootContractBytes, CalleeContractBytes });
    }

    private static EmissionFixture Build(
        string referencePackRoot,
        string compilation,
        string source,
        string selectedMethod,
        string[] closureMethods,
        string[]? contractPaths = null,
        byte[][]? contractBytes = null)
    {
        string[] methods = { selectedMethod };
        string[] paths = contractPaths ?? closureMethods
            .Select((_, index) => "contracts/c" + index.ToString("D3", CultureInfo.InvariantCulture) + ".json")
            .ToArray();
        byte[][] bytes = contractBytes ?? closureMethods.Select(ContractBytes).ToArray();
        if (paths.Length != bytes.Length || paths.Length != closureMethods.Length)
        {
            throw new HarnessFailure("CONTRACT_FIXTURE");
        }

        Array.Sort(paths, bytes, StringComparer.Ordinal);
        var raw = new RawSelection(compilation, new[] { SourcePath }, paths, methods);
        Selection selection = SelectionCodec.Validate(raw);
        var files = new List<CapturedFile>
        {
            new CapturedFile(CapturedInputKind.Source, SourcePath, Encoding.UTF8.GetBytes(source)),
        };
        for (int index = 0; index < paths.Length; index++)
        {
            files.Add(new CapturedFile(CapturedInputKind.Contract, paths[index], bytes[index]));
        }

        var snapshot = new CapturedSnapshot(selection, files.ToArray());
        CapturedSourceSet sources = SourceTransport.Validate(snapshot);
        RoslynSourceSession sourceSession = RoslynSessionFactory.Parse(selection, sources);
        RoslynCompilationSession compilationSession = RoslynSessionFactory.Compile(
            selection,
            sourceSession,
            referencePackRoot);
        SubsetClosure closure = CSharpSubset.Validate(selection, compilationSession);
        ContractSet contracts = CSharpContracts.Attach(selection, snapshot, closure);
        LoweredClosure lowered = CSharpLowering.Lower(selection, closure, contracts);
        var request = new LowerRequest(
            FrontendConstants.SourceRoot,
            raw,
            new ReleaseArguments(
                "frontend.csharp.test.v0",
                new string('a', 64),
                new string('1', 64),
                "toolchain.csharp.test.v0",
                new string('b', 64)));
        EmittedFrontendSuccess success = CSharpFrontendSuccessEmitter.Emit(
            request,
            selection,
            snapshot,
            sources,
            compilationSession,
            closure,
            contracts,
            lowered);
        return new EmissionFixture(
            request,
            selection,
            snapshot,
            sources,
            compilationSession,
            closure,
            contracts,
            lowered,
            success);
    }

    private static byte[] ContractBytes(string method)
    {
        return Encoding.UTF8.GetBytes(
            "{\"abrupt_completion\":\"forbidden\",\"ensures\":[{\"bool\":true}],"
            + "\"method\":\"" + method + "\",\"modifies\":[],\"requires\":[],"
            + "\"schema\":\"mpk.csharp.contract.v0\","
            + "\"semantic_profile\":\"mpk.csharp.scalar.v0\",\"termination\":\"total\"}\n");
    }

    private static LoweredClosure ReplaceCallHash(
        LoweredClosure closure,
        string functionId,
        string instructionId)
    {
        LoweredFunction[] functions = closure.Functions.Select(function =>
        {
            if (!string.Equals(function.Id, functionId, StringComparison.Ordinal))
            {
                return function;
            }

            LoweredBlock[] blocks = function.Blocks.Select(block => new LoweredBlock(
                block.Label,
                block.Parameters.ToArray(),
                block.Instructions.Select(instruction =>
                    string.Equals(instruction.Id, instructionId, StringComparison.Ordinal)
                        ? new LoweredInstruction(
                            instruction.Id,
                            instruction.Kind,
                            instruction.Type,
                            instruction.Target,
                            instruction.UnaryOperator,
                            instruction.BinaryOperator,
                            instruction.ConversionForm,
                            instruction.OverflowContext,
                            instruction.IsShiftCountMask,
                            instruction.Operands.ToArray(),
                            instruction.SafetyChecks.ToArray(),
                            instruction.Origin,
                            instruction.Function,
                            new string('0', 64))
                        : instruction).ToArray(),
                block.Terminator)).ToArray();
            return new LoweredFunction(
                function.Id,
                function.Name,
                function.ContractHash,
                function.Origin,
                function.Parameters.ToArray(),
                function.Results.ToArray(),
                function.Locals.ToArray(),
                blocks,
                function.RequiredChecks.ToArray(),
                function.Features.ToArray());
        }).ToArray();
        return new LoweredClosure(closure.SelectionSha256, functions);
    }

    private static string IdFingerprint(LoweredClosure closure)
    {
        var builder = new StringBuilder();
        foreach (LoweredFunction function in closure.Functions)
        {
            builder.Append(function.Id).Append('|').Append(function.Name).Append('|')
                .Append(string.Join(',', function.Parameters.Select(binding => binding.Id))).Append('|')
                .Append(string.Join(',', function.Results.Select(binding => binding.Id))).Append('|')
                .Append(string.Join(',', function.Locals.Select(binding => binding.Id)));
            foreach (LoweredBlock block in function.Blocks)
            {
                builder.Append('|').Append(block.Label).Append('(')
                    .Append(string.Join(',', block.Parameters.Select(binding => binding.Id))).Append(')')
                    .Append('[').Append(string.Join(',', block.Instructions.Select(instruction =>
                        instruction.Id + ":" + instruction.Kind + ":" + instruction.Function))).Append(']')
                    .Append("->").Append(block.Terminator.Kind).Append(':')
                    .Append(block.Terminator.FalseTarget).Append(':')
                    .Append(block.Terminator.TrueTarget);
            }
        }

        return builder.ToString();
    }

    private static void CheckMap(
        string source,
        int start,
        int end,
        int utf8Start,
        int utf8End,
        int lineStart,
        int columnStart,
        int lineEnd,
        int columnEnd,
        string code)
    {
        MappedSourceOrigin mapped = CSharpSourceMapper.MapVector(source, start, end);
        Equal(utf8Start, mapped.Utf8Start, code + "_UTF8_START");
        Equal(utf8End, mapped.Utf8End, code + "_UTF8_END");
        Equal(lineStart, mapped.LineStart, code + "_LINE_START");
        Equal(columnStart, mapped.ColumnStartUtf16, code + "_COLUMN_START");
        Equal(lineEnd, mapped.LineEnd, code + "_LINE_END");
        Equal(columnEnd, mapped.ColumnEndUtf16, code + "_COLUMN_END");
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
            if (failure.Status == status
                && string.Equals(failure.Phase, phase, StringComparison.Ordinal)
                && string.Equals(failure.Code, code, StringComparison.Ordinal))
            {
                return;
            }

            throw new HarnessFailure(code + "_WRONG_FAILURE");
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

    private sealed class EmissionFixture
    {
        internal EmissionFixture(
            LowerRequest request,
            Selection selection,
            CapturedSnapshot snapshot,
            CapturedSourceSet sources,
            RoslynCompilationSession compilation,
            SubsetClosure closure,
            ContractSet contracts,
            LoweredClosure lowered,
            EmittedFrontendSuccess success)
        {
            Request = request;
            Selection = selection;
            Snapshot = snapshot;
            Sources = sources;
            Compilation = compilation;
            Closure = closure;
            Contracts = contracts;
            Lowered = lowered;
            Success = success;
        }

        internal LowerRequest Request { get; }

        internal Selection Selection { get; }

        internal CapturedSnapshot Snapshot { get; }

        internal CapturedSourceSet Sources { get; }

        internal RoslynCompilationSession Compilation { get; }

        internal SubsetClosure Closure { get; }

        internal ContractSet Contracts { get; }

        internal LoweredClosure Lowered { get; }

        internal EmittedFrontendSuccess Success { get; }

        internal LoweredFunction Function(string id)
        {
            return Lowered.Functions.Single(function =>
                string.Equals(function.Id, id, StringComparison.Ordinal));
        }
    }

    private sealed class HarnessFailure : Exception
    {
        internal HarnessFailure(string code)
            : base(code)
        {
            Code = code;
        }

        internal string Code { get; }
    }
}
