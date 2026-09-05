using System;
using System.Collections.Generic;
using System.Collections.Immutable;
using System.IO;
using System.Linq;
using System.Text;
using Microsoft.CodeAnalysis;

namespace Mpk.CSharp2Vir;

internal static class PracticalSyntaxHarness
{
    private static ImmutableArray<MetadataReference> references;
    private static string currentTest = "bootstrap";

    public static int Main(string[] arguments)
    {
        try
        {
            if (arguments.Length != 1)
            {
                throw new HarnessFailure("ARGUMENTS");
            }

            references = LoadReferences(arguments[0]);
            RunTest(
                nameof(ExpressionBodiesNormalizeExactly),
                ExpressionBodiesNormalizeExactly);
            RunTest(
                nameof(VarLocalsExposeExactTypes),
                VarLocalsExposeExactTypes);
            RunTest(
                nameof(NamespaceImportsAndNullableNormalizeExactly),
                NamespaceImportsAndNullableNormalizeExactly);
            RunTest(
                nameof(InferenceFailuresAreClosed),
                InferenceFailuresAreClosed);
            RunTest(
                nameof(ImportFormsAreClosed),
                ImportFormsAreClosed);
            RunTest(
                nameof(DirectiveFormsAreClosed),
                DirectiveFormsAreClosed);
            RunTest(
                nameof(ForeachRemainsDeferred),
                ForeachRemainsDeferred);
            RunTest(
                nameof(ArtifactsAreDeterministicImmutableAndSanitized),
                ArtifactsAreDeterministicImmutableAndSanitized);
            return 0;
        }
        catch (HarnessFailure failure)
        {
            Console.Error.Write(
                "CSHARP_PRACTICAL_SYNTAX_TEST_"
                + currentTest + "_" + failure.Code + "\n");
            return 1;
        }
        catch (Exception error)
        {
            string detail = error is PracticalCaptureFailure practical
                ? practical.Family + "_" + practical.Code
                : error.GetType().Name;
            Console.Error.Write(
                "CSHARP_PRACTICAL_SYNTAX_TEST_UNEXPECTED_"
                + currentTest + "_" + detail + "\n");
            return 1;
        }
    }

    private static void RunTest(string name, Action test)
    {
        currentTest = name;
        test();
    }

    private static void ExpressionBodiesNormalizeExactly()
    {
        const string Block =
            "namespace Business;\n"
            + "public sealed class Box\n"
            + "{\n"
            + "    private readonly int value;\n"
            + "    public Box(int value) { this.value = value; Entry.Ignore(value); }\n"
            + "    public int Value { get { return value; } }\n"
            + "}\n"
            + "public static class Entry\n"
            + "{\n"
            + "    internal static void Ignore(int value) { _ = value; }\n"
            + "    private static int Identity(int value) { return value; }\n"
            + "    public static int Run(int input)\n"
            + "    {\n"
            + "        return Identity(new Box(input).Value);\n"
            + "    }\n"
            + "}\n";
        const string PropertyAndMethodArrows =
            "namespace Business;\n"
            + "public sealed class Box\n"
            + "{\n"
            + "    private readonly int value;\n"
            + "    public Box(int value) { this.value = value; Entry.Ignore(value); }\n"
            + "    public int Value => value;\n"
            + "}\n"
            + "public static class Entry\n"
            + "{\n"
            + "    internal static void Ignore(int value) => _ = value;\n"
            + "    private static int Identity(int value) => value;\n"
            + "    public static int Run(int input) => Identity(new Box(input).Value);\n"
            + "}\n";
        const string GetterArrow =
            "namespace Business;\n"
            + "public sealed class Box\n"
            + "{\n"
            + "    private readonly int value;\n"
            + "    public Box(int value) { this.value = value; Entry.Ignore(value); }\n"
            + "    public int Value { get => value; }\n"
            + "}\n"
            + "public static class Entry\n"
            + "{\n"
            + "    internal static void Ignore(int value) { _ = value; }\n"
            + "    private static int Identity(int value) { return value; }\n"
            + "    public static int Run(int input)\n"
            + "    {\n"
            + "        return Identity(new Box(input).Value);\n"
            + "    }\n"
            + "}\n";

        PracticalNormalizedSyntax block = RunSingle(Block);
        AssertEquivalent(block, RunSingle(PropertyAndMethodArrows), "MEMBER_ARROWS");
        AssertEquivalent(block, RunSingle(GetterArrow), "GETTER_ARROW");
        Equal(5, block.Callables.Count, "ARROW_CALLABLE_COUNT");

        Expect(
            PracticalDiagnosticFamily.CSHARP_PRACTICAL_DECLARATION,
            "expression_body_kind",
            () => RunSingle(
                "namespace Business;\npublic sealed class Box { private int value; public Box(int input) => value = input; public int Read() { return value; } } public static class Entry { public static int Run(int input) { return new Box(input).Read(); } }\n"));
        Expect(
            PracticalDiagnosticFamily.CSHARP_PRACTICAL_DECLARATION,
            "property_shape",
            () => RunSingle(
                "namespace Business;\npublic static class Entry { public static int Value { set => _ = value; } public static int Run(int input) { Value = input; return input; } }\n"));
        Expect(
            PracticalDiagnosticFamily.CSHARP_PRACTICAL_DECLARATION,
            "unsupported_declaration",
            () => RunSingle(
                "namespace Business;\npublic static class Entry { public static int Run(int input) { int Local(int value) => value; return Local(input); } }\n"));
        Expect(
            PracticalDiagnosticFamily.CSHARP_PRACTICAL_DECLARATION,
            "expression_body_kind",
            () => RunSingle(
                "namespace Business;\npublic sealed class Box { private int value; public Box(int input) { value = input; } public int Value { get { return value; } init => this.value = value; } } public static class Entry { public static int Run(int input) { return new Box(input).Value; } }\n"));
    }

    private static void VarLocalsExposeExactTypes()
    {
        currentTest = nameof(VarLocalsExposeExactTypes) + "_Value";
        const string Explicit =
            "namespace Business;\n"
            + "public static class Entry\n"
            + "{\n"
            + "    public static int Run(int input)\n"
            + "    {\n"
            + "        int number = input + 1;\n"
            + "        int[] values = new int[] { number, input };\n"
            + "        int? maybe = (int?)values[0];\n"
            + "        return maybe ?? 0;\n"
            + "    }\n"
            + "}\n";
        const string Inferred =
            "namespace Business;\n"
            + "public static class Entry\n"
            + "{\n"
            + "    public static int Run(int input)\n"
            + "    {\n"
            + "        var number = input + 1;\n"
            + "        var values = new[] { number, input };\n"
            + "        var maybe = (int?)values[0];\n"
            + "        return maybe ?? 0;\n"
            + "    }\n"
            + "}\n";

        PracticalNormalizedSyntax explicitResult = RunSingle(Explicit);
        PracticalNormalizedSyntax inferredResult = RunSingle(Inferred);
        AssertEquivalent(explicitResult, inferredResult, "VAR_EXPLICIT");
        Equal(3, inferredResult.ExactTypes.Count, "VAR_TYPE_COUNT");
        Equal(Value("i32"), inferredResult.ExactTypes[0].Type.Id, "VAR_I32");
        Equal(
            PracticalIdentity.ClosedInstanceId("bounded_sequence", Value("i32")),
            inferredResult.ExactTypes[1].Type.Id,
            "VAR_ARRAY");
        Equal(
            PracticalIdentity.ClosedInstanceId("option", Value("i32")),
            inferredResult.ExactTypes[2].Type.Id,
            "VAR_NULLABLE_VALUE");

        const string ExplicitNullable =
            "namespace Business;\n"
            + "public static class Entry\n"
            + "{\n"
            + "    public static int Run(string? input)\n"
            + "    {\n"
            + "        string? copy = input;\n"
            + "        return copy is null ? 0 : copy.Length;\n"
            + "    }\n"
            + "}\n";
        const string InferredNullable =
            "namespace Business;\n"
            + "public static class Entry\n"
            + "{\n"
            + "    public static int Run(string? input)\n"
            + "    {\n"
            + "        var copy = input;\n"
            + "        return copy is null ? 0 : copy.Length;\n"
            + "    }\n"
            + "}\n";
        currentTest = nameof(VarLocalsExposeExactTypes) + "_Reference";
        PracticalNormalizedSyntax nullable = RunSingle(
            InferredNullable,
            new[] { Value("string") });
        AssertEquivalent(
            RunSingle(ExplicitNullable, new[] { Value("string") }),
            nullable,
            "VAR_NULLABLE_EXPLICIT");
        Equal(1, nullable.ExactTypes.Count, "VAR_NULLABLE_COUNT");
        Equal("annotated", nullable.ExactTypes[0].Type.Nullability, "VAR_NULLABILITY");

        const string ExplicitRecursiveNullable =
            "namespace Business;\n"
            + "public static class Entry\n"
            + "{\n"
            + "    public static int Run(string? input)\n"
            + "    {\n"
            + "        string?[] values = new string?[] { input };\n"
            + "        string? copy = values[0];\n"
            + "        return copy is null ? 0 : copy.Length;\n"
            + "    }\n"
            + "}\n";
        const string InferredRecursiveNullable =
            "namespace Business;\n"
            + "public static class Entry\n"
            + "{\n"
            + "    public static int Run(string? input)\n"
            + "    {\n"
            + "        var values = new[] { input };\n"
            + "        var copy = values[0];\n"
            + "        return copy is null ? 0 : copy.Length;\n"
            + "    }\n"
            + "}\n";
        PracticalNormalizedSyntax recursiveNullable = RunSingle(
            InferredRecursiveNullable,
            new[] { Value("string") });
        AssertEquivalent(
            RunSingle(ExplicitRecursiveNullable, new[] { Value("string") }),
            recursiveNullable,
            "VAR_RECURSIVE_NULLABLE_EXPLICIT");
        Equal(2, recursiveNullable.ExactTypes.Count, "VAR_RECURSIVE_COUNT");
        Equal(
            "annotated",
            recursiveNullable.ExactTypes[0].Type.Arguments[0].Nullability,
            "VAR_RECURSIVE_ELEMENT_NULLABILITY");
    }

    private static void NamespaceImportsAndNullableNormalizeExactly()
    {
        const string FullyQualified =
            "namespace Business\n"
            + "{\n"
            + "    public static class Entry\n"
            + "    {\n"
            + "        public static int Run(System.DateOnly date, System.Guid id, System.DayOfWeek day)\n"
            + "        {\n"
            + "            return id == System.Guid.Empty && day == System.DayOfWeek.Monday ? 1 : 0;\n"
            + "        }\n"
            + "    }\n"
            + "}\n";
        const string CompilationImport =
            "using System;\n"
            + "namespace Business\n"
            + "{\n"
            + "    public static class Entry\n"
            + "    {\n"
            + "        public static int Run(DateOnly date, Guid id, DayOfWeek day)\n"
            + "        {\n"
            + "            return id == Guid.Empty && day == DayOfWeek.Monday ? 1 : 0;\n"
            + "        }\n"
            + "    }\n"
            + "}\n";
        const string NamespaceImport =
            "namespace Business\n"
            + "{\n"
            + "    using System;\n"
            + "    public static class Entry\n"
            + "    {\n"
            + "        public static int Run(DateOnly date, Guid id, DayOfWeek day)\n"
            + "        {\n"
            + "            return id == Guid.Empty && day == DayOfWeek.Monday ? 1 : 0;\n"
            + "        }\n"
            + "    }\n"
            + "}\n";
        string[] parameterTypes =
        {
            Value("date"),
            Value("guid"),
            Value("day_of_week"),
        };
        PracticalNormalizedSyntax qualified = RunSingle(FullyQualified, parameterTypes);
        AssertEquivalent(
            qualified,
            RunSingle(CompilationImport, parameterTypes),
            "COMPILATION_IMPORT");
        AssertEquivalent(
            qualified,
            RunSingle(NamespaceImport, parameterTypes),
            "NAMESPACE_IMPORT");

        const string Ordinary =
            "namespace Business;\n"
            + "public static class Entry { public static int Run(int input) { return input; } }\n";
        AssertEquivalent(
            RunSingle(Ordinary),
            RunSingle("#nullable enable\n" + Ordinary),
            "NULLABLE_ENABLE");
        AssertEquivalent(
            RunSingle(Ordinary),
            RunSingle("// retained application comment\n#nullable enable\n" + Ordinary),
            "NULLABLE_ENABLE_AFTER_COMMENT");

        const string QualifiedIntrinsics =
            "namespace Business;\n"
            + "public static class Entry\n"
            + "{\n"
            + "    public static int Run(string value)\n"
            + "    {\n"
            + "        decimal rounded = decimal.Round(1m, 0, System.MidpointRounding.ToEven);\n"
            + "        return string.Equals(value, \"ok\", System.StringComparison.Ordinal) && rounded == 1m ? 1 : 0;\n"
            + "    }\n"
            + "}\n";
        const string ImportedIntrinsics =
            "using System;\n"
            + "namespace Business;\n"
            + "public static class Entry\n"
            + "{\n"
            + "    public static int Run(string value)\n"
            + "    {\n"
            + "        var rounded = decimal.Round(1m, 0, MidpointRounding.ToEven);\n"
            + "        return string.Equals(value, \"ok\", StringComparison.Ordinal) && rounded == 1m ? 1 : 0;\n"
            + "    }\n"
            + "}\n";
        AssertEquivalent(
            RunSingle(QualifiedIntrinsics, new[] { Value("string") }),
            RunSingle(ImportedIntrinsics, new[] { Value("string") }),
            "INTRINSIC_IMPORT_AND_VAR");
    }

    private static void InferenceFailuresAreClosed()
    {
        Expect(
            PracticalDiagnosticFamily.CSHARP_PRACTICAL_DECLARATION,
            "compiler_diagnostic",
            () => RunSingle(
                "namespace Business;\npublic static class Entry { public static int Run(int input) { var left = input, right = 1; return left + right; } }\n"));
        Expect(
            PracticalDiagnosticFamily.CSHARP_PRACTICAL_TYPE,
            "target_typed_inference",
            () => RunSingle(
                "namespace Business;\npublic static class Entry { public static int Run(int input) { var values = new[] { input, 1L }; return values.Length; } }\n"));
        Expect(
            PracticalDiagnosticFamily.CSHARP_PRACTICAL_DECLARATION,
            "var_shape",
            () => RunSingle(
                "namespace Business;\npublic static class Entry { public static int Run(int input) { int value = input; ref var alias = ref value; return alias; } }\n"));
        Expect(
            PracticalDiagnosticFamily.CSHARP_PRACTICAL_TYPE,
            "target_typed_inference",
            () => RunSingle(
                "namespace Business;\npublic static class Entry { private static int Identity(int value) { return value; } public static int Run(int input) { var value = Identity(default); return value; } }\n"));
        Expect(
            PracticalDiagnosticFamily.CSHARP_PRACTICAL_TYPE,
            "target_typed_inference",
            () => RunSingle(
                "namespace Business;\npublic static class Entry { private static int[] Identity(int[] value) { return value; } public static int Run(int input) { var values = Identity([input]); return values.Length; } }\n"));
        Expect(
            PracticalDiagnosticFamily.CSHARP_PRACTICAL_TYPE,
            "target_typed_inference",
            () => RunSingle(
                "namespace Business;\npublic sealed class Item { public Item(int value) { Value = value; } public int Value { get; } } public static class Entry { private static Item Identity(Item value) { return value; } public static int Run(int input) { var item = Identity(new(input)); return item.Value; } }\n"));
        Expect(
            PracticalDiagnosticFamily.CSHARP_PRACTICAL_TYPE,
            "target_typed_inference",
            () => RunSingle(
                "namespace Business;\npublic static class Entry { public static int Run(int input) { var item = new { Value = input }; return item.Value; } }\n"));
        Expect(
            PracticalDiagnosticFamily.CSHARP_PRACTICAL_DECLARATION,
            "delegate_dynamic_or_runtime_codegen",
            () => RunSingle(
                "namespace Business;\npublic static class Entry { public static int Run(int input) { dynamic value = input; return value; } }\n"));
        Expect(
            PracticalDiagnosticFamily.CSHARP_PRACTICAL_DECLARATION,
            "compiler_diagnostic",
            () => RunSingle(
                "using Left;\nusing Right;\nnamespace Left { public sealed class Token { } }\nnamespace Right { public sealed class Token { } }\nnamespace Business { public static class Entry { public static int Run(int input) { Token value = new Token(); return input; } } }\n"));
        Expect(
            PracticalDiagnosticFamily.CSHARP_PRACTICAL_DECLARATION,
            "compiler_diagnostic",
            () => RunSingle(
                "namespace Business;\npublic sealed class var { public var(int value) { Value = value; } public int Value { get; } } public static class Entry { public static int Run(int input) { var item = new var(input); return item.Value; } }\n"));
        Expect(
            PracticalDiagnosticFamily.CSHARP_PRACTICAL_DECLARATION,
            "compiler_diagnostic",
            () => RunSingle(
                "namespace Business;\npublic static class Entry { public static int Run(string? input) { var value = input; return value.Length; } }\n",
                new[] { Value("string") }));
        Expect(
            PracticalDiagnosticFamily.CSHARP_PRACTICAL_TYPE,
            "framework_api",
            () => RunSingle(
                "namespace Business;\npublic static class Entry { public static int Run(int input) { var value = new System.Uri(\"https://example.invalid\"); return input; } }\n"));
    }

    private static void ImportFormsAreClosed()
    {
        foreach (string source in new[]
        {
            "global using System;\nnamespace Business;\npublic static class Entry { public static int Run(int input) { return input; } }\n",
            "using Alias = Business.Helpers;\nnamespace Business;\npublic static class Helpers { public static int Identity(int input) { return input; } } public static class Entry { public static int Run(int input) { return Alias.Identity(input); } }\n",
            "using static Business.Helpers;\nnamespace Business;\npublic static class Helpers { public static int Identity(int input) { return input; } } public static class Entry { public static int Run(int input) { return Identity(input); } }\n",
            "using global::System;\nnamespace Business;\npublic static class Entry { public static int Run(int input) { return input; } }\n",
        })
        {
            Expect(
                PracticalDiagnosticFamily.CSHARP_PRACTICAL_DECLARATION,
                "using_directive",
                () => RunSingle(source));
        }

        Expect(
            PracticalDiagnosticFamily.CSHARP_PRACTICAL_DECLARATION,
            "compiler_diagnostic",
            () => RunSingle(
                "extern alias External;\nnamespace Business;\npublic static class Entry { public static int Run(int input) { return input; } }\n"));
        Expect(
            PracticalDiagnosticFamily.CSHARP_PRACTICAL_DECLARATION,
            "compiler_diagnostic",
            () => RunSingle(
                "using System;\nusing System;\nnamespace Business;\npublic static class Entry { public static int Run(int input) { return input; } }\n"));
        Expect(
            PracticalDiagnosticFamily.CSHARP_PRACTICAL_DECLARATION,
            "compiler_diagnostic",
            () => RunSingle(
                "using Missing.Namespace;\nnamespace Business;\npublic static class Entry { public static int Run(int input) { return input; } }\n"));
        Expect(
            PracticalDiagnosticFamily.CSHARP_PRACTICAL_DEPENDENCY,
            "mpk_namespace",
            () => RunSingle(
                "using Mpk;\nnamespace Mpk { public sealed class Marker { } }\nnamespace Business { public static class Entry { public static int Run(int input) { Marker marker = new Marker(); return input; } } }\n"));
        Expect(
            PracticalDiagnosticFamily.CSHARP_PRACTICAL_EFFECT,
            "external_effect_or_concurrency",
            () => RunSingle(
                "namespace Business;\npublic static class Entry { public static int Run(int input) { using (var stream = new System.IO.MemoryStream()) { return input; } } }\n"));
        Expect(
            PracticalDiagnosticFamily.CSHARP_PRACTICAL_EFFECT,
            "external_effect_or_concurrency",
            () => RunSingle(
                "namespace Business;\npublic static class Entry { public static int Run(int input) { using var stream = new System.IO.MemoryStream(); return input; } }\n"));

        PracticalSourceSelection selection = Selection(
            new[] { "src/Entry.cs", "src/ImplicitUsings.g.cs" },
            new[] { DefaultRootId() });
        Expect(
            PracticalDiagnosticFamily.CSHARP_PRACTICAL_DEPENDENCY,
            "generated_source",
            () => CSharpPracticalSyntaxNormalizer.Normalize(
                selection,
                new[]
                {
                    Source("src/Entry.cs", DefaultSource()),
                    new PracticalCapturedInput(
                        PracticalCapturedInputKind.GeneratedSource,
                        "src/ImplicitUsings.g.cs",
                        Utf8("global using System;\n")),
                },
                references));
    }

    private static void DirectiveFormsAreClosed()
    {
        const string Body =
            "namespace Business;\n"
            + "public static class Entry { public static int Run(int input) { return input; } }\n";
        foreach (string source in new[]
        {
            "#nullable disable\n" + Body,
            "#nullable restore\n" + Body,
            "#nullable enable annotations\n" + Body,
            "#nullable enable warnings\n" + Body,
            "#nullable enable\n#nullable enable\n" + Body,
            "namespace Business;\n#nullable enable\npublic static class Entry { public static int Run(int input) { return input; } }\n",
            "#if true\n" + Body + "#endif\n",
            "#define FEATURE\n" + Body,
            "#undef FEATURE\n" + Body,
            "#pragma warning disable\n" + Body,
            "#line 1 \"mapped.cs\"\n" + Body,
            "#region practical\n" + Body + "#endregion\n",
        })
        {
            Expect(
                PracticalDiagnosticFamily.CSHARP_PRACTICAL_DECLARATION,
                "source_directive",
                () => RunSingle(source));
        }

        foreach (string source in new[]
        {
            "#error STOP\n" + Body,
            "#warning STOP\n" + Body,
        })
        {
            Expect(
                PracticalDiagnosticFamily.CSHARP_PRACTICAL_DECLARATION,
                "compiler_diagnostic",
                () => RunSingle(source));
        }
    }

    private static void ForeachRemainsDeferred()
    {
        foreach (string declaration in new[] { "int value", "var value" })
        {
            Expect(
                PracticalDiagnosticFamily.CSHARP_PRACTICAL_EFFECT,
                "external_effect_or_concurrency",
                () => RunSingle(
                    "namespace Business;\npublic static class Entry { public static int Run(int input) { int total = 0; foreach ("
                    + declaration
                    + " in new int[] { input }) { total += value; } return total; } }\n"));
        }
    }

    private static void ArtifactsAreDeterministicImmutableAndSanitized()
    {
        var sources = new Dictionary<string, string>(StringComparer.Ordinal)
        {
            ["src/Entry.cs"] =
                "using Business.Work;\nnamespace Business;\npublic static class Entry { public static int Run(int input) => Worker.Step(input); }\n",
            ["src/Worker.cs"] =
                "namespace Business.Work;\npublic static class Worker { public static int Step(int input) { var next = input + 1; return next; } }\n",
        };
        string root = DefaultRootId();
        PracticalNormalizedSyntax first = Run(sources, new[] { root });
        PracticalNormalizedSyntax second = Run(
            new Dictionary<string, string>(sources.Reverse(), StringComparer.Ordinal),
            new[] { root },
            reverseReferences: true);
        AssertEquivalent(first, second, "DETERMINISTIC_ORDER");

        byte[] artifact = first.CopyCanonicalBytes();
        byte original = artifact[0];
        artifact[0] ^= 0xff;
        Equal(original, first.CopyCanonicalBytes()[0], "IMMUTABLE_ARTIFACT");
        byte[] body = first.Callables[0].CopyBodyBytes();
        byte bodyOriginal = body[0];
        body[0] ^= 0xff;
        Equal(bodyOriginal, first.Callables[0].CopyBodyBytes()[0], "IMMUTABLE_BODY");

        PracticalCaptureFailure failure = CaptureFailure(() => RunSingle(
            "namespace Customer.Secret;\n"
            + "public static class Entry { public static int Run(int input) { var pair = new { SecretMember = input }; return pair.SecretMember; } }\n",
            namespaceName: "Customer.Secret"));
        Equal(0, failure.ArtifactCount, "FAILURE_ARTIFACT_COUNT");
        Equal(PracticalCaptureFailure.PublicMessage, failure.Message, "PUBLIC_MESSAGE");
        Check(!failure.Message.Contains("Customer", StringComparison.Ordinal), "NO_NAMESPACE");
        Check(!failure.Message.Contains("SecretMember", StringComparison.Ordinal), "NO_MEMBER");
        Check(!failure.Message.Contains("src/", StringComparison.Ordinal), "NO_PATH");
    }

    private static void AssertEquivalent(
        PracticalNormalizedSyntax expected,
        PracticalNormalizedSyntax actual,
        string code)
    {
        Equal(expected.SemanticSha256, actual.SemanticSha256, code + "_HASH");
        Check(
            expected.CopyCanonicalBytes().SequenceEqual(actual.CopyCanonicalBytes()),
            code + "_BYTES");
        Equal(expected.Callables.Count, actual.Callables.Count, code + "_CALLABLES");
        Equal(expected.ExactTypes.Count, actual.ExactTypes.Count, code + "_TYPES");
        for (int index = 0; index < expected.Callables.Count; index++)
        {
            Equal(expected.Callables[index].Id, actual.Callables[index].Id, code + "_ID");
            Check(
                expected.Callables[index].CopyBodyBytes().SequenceEqual(
                    actual.Callables[index].CopyBodyBytes()),
                code + "_BODY");
        }
    }

    private static PracticalNormalizedSyntax RunSingle(
        string source,
        string[]? parameterTypes = null,
        string? resultType = null,
        string namespaceName = "Business")
    {
        string root = MethodId(
            namespaceName,
            "Entry",
            "Run",
            parameterTypes ?? new[] { Value("i32") },
            resultType ?? Value("i32"));
        return Run(
            new Dictionary<string, string>(StringComparer.Ordinal)
            {
                ["src/Entry.cs"] = source,
            },
            new[] { root });
    }

    private static PracticalNormalizedSyntax Run(
        IReadOnlyDictionary<string, string> sourceByPath,
        string[] roots,
        bool reverseReferences = false)
    {
        string[] paths = sourceByPath.Keys
            .OrderBy(value => value, StringComparer.Ordinal)
            .ToArray();
        PracticalCapturedInput[] inputs = paths
            .Select(path => Source(path, sourceByPath[path]))
            .Reverse()
            .ToArray();
        ImmutableArray<MetadataReference> selectedReferences = reverseReferences
            ? references.Reverse().ToImmutableArray()
            : references;
        return CSharpPracticalSyntaxNormalizer.Normalize(
            Selection(paths, roots),
            inputs,
            selectedReferences);
    }

    private static PracticalSourceSelection Selection(string[] paths, string[] roots) =>
        new PracticalSourceSelection(
            CSharpPracticalCapture.SelectionSchema,
            "business",
            paths,
            roots.OrderBy(value => value, StringComparer.Ordinal),
            Array.Empty<string>());

    private static PracticalCapturedInput Source(string path, string source) =>
        new PracticalCapturedInput(
            PracticalCapturedInputKind.Source,
            path,
            Utf8(source));

    private static string DefaultSource() =>
        "namespace Business;\n"
        + "public static class Entry { public static int Run(int input) { return input; } }\n";

    private static string DefaultRootId() =>
        MethodId("Business", "Entry", "Run", new[] { Value("i32") }, Value("i32"));

    private static string MethodId(
        string namespaceName,
        string type,
        string method,
        IEnumerable<string> parameters,
        string result)
    {
        string owner = PracticalIdentity.SourceTypeId(namespaceName, type);
        return PracticalIdentity.CallableId(
            "method",
            namespaceName,
            owner,
            method,
            parameters,
            result);
    }

    private static string Value(string token) => PracticalIdentity.PrimitiveId(token);

    private static byte[] Utf8(string value) =>
        new UTF8Encoding(false, true).GetBytes(value);

    private static ImmutableArray<MetadataReference> LoadReferences(string referenceRoot)
    {
        string directory = Path.Combine(referenceRoot, "ref", "net10.0");
        string[] paths = Directory.EnumerateFiles(
                directory,
                "*.dll",
                SearchOption.TopDirectoryOnly)
            .OrderBy(path => Path.GetFileName(path), StringComparer.Ordinal)
            .ToArray();
        Equal(167, paths.Length, "REFERENCE_COUNT");
        return paths
            .Select(path => MetadataReference.CreateFromFile(path))
            .ToImmutableArray<MetadataReference>();
    }

    private static void Expect(
        PracticalDiagnosticFamily family,
        string code,
        Action action)
    {
        PracticalCaptureFailure failure = CaptureFailure(action);
        Equal(
            family,
            failure.Family,
            "FAILURE_FAMILY_" + code + "_GOT_" + failure.Family + "_" + failure.Code);
        Equal(code, failure.Code, "FAILURE_CODE_" + code + "_GOT_" + failure.Code);
        Equal(ExpectedPhase(family), failure.Phase, "FAILURE_PHASE_" + code);
        Equal(0, failure.ArtifactCount, "FAILURE_ARTIFACTS_" + code);
    }

    private static void ExpectOneOf(
        IReadOnlyCollection<PracticalDiagnosticFamily> families,
        Action action)
    {
        PracticalCaptureFailure failure = CaptureFailure(action);
        Check(
            families.Contains(failure.Family),
            "FAILURE_FAMILY_GOT_" + failure.Family + "_" + failure.Code);
        Equal(ExpectedPhase(failure.Family), failure.Phase, "FAILURE_PHASE");
        Equal(0, failure.ArtifactCount, "FAILURE_ARTIFACTS");
    }

    private static PracticalCaptureFailure CaptureFailure(Action action)
    {
        try
        {
            action();
        }
        catch (PracticalCaptureFailure failure)
        {
            return failure;
        }

        throw new HarnessFailure("EXPECTED_FAILURE");
    }

    private static int ExpectedPhase(PracticalDiagnosticFamily family) => family switch
    {
        PracticalDiagnosticFamily.CSHARP_PRACTICAL_PROTOCOL => 0,
        PracticalDiagnosticFamily.CSHARP_PRACTICAL_LIMIT => 0,
        PracticalDiagnosticFamily.CSHARP_PRACTICAL_DEPENDENCY => 1,
        PracticalDiagnosticFamily.CSHARP_PRACTICAL_DECLARATION => 2,
        PracticalDiagnosticFamily.CSHARP_PRACTICAL_TYPE => 2,
        PracticalDiagnosticFamily.CSHARP_PRACTICAL_GENERIC => 3,
        PracticalDiagnosticFamily.CSHARP_PRACTICAL_EFFECT => 7,
        _ => throw new HarnessFailure("UNKNOWN_PHASE"),
    };

    private static void Check(bool condition, string code)
    {
        if (!condition)
        {
            throw new HarnessFailure(code);
        }
    }

    private static void Equal<T>(T expected, T actual, string code)
        where T : notnull
    {
        if (!EqualityComparer<T>.Default.Equals(expected, actual))
        {
            throw new HarnessFailure(code);
        }
    }

    private sealed class HarnessFailure : Exception
    {
        internal HarnessFailure(string code)
        {
            Code = code;
        }

        internal string Code { get; }
    }
}
