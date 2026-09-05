using System;
using System.Collections.Generic;
using System.Collections.Immutable;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Runtime.Loader;
using System.Text;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.CSharp.Syntax;

namespace Mpk.CSharp2Vir;

internal static class PracticalTypesHarness
{
    private static ImmutableArray<MetadataReference> references;
    private static string currentTest = "bootstrap";
    public static int Main(string[] arguments)
    {
        try
        {
            Check(arguments.Length == 1, "ARGUMENTS");
            references = Directory.EnumerateFiles(Path.Combine(arguments[0], "ref", "net10.0"), "*.dll")
                .OrderBy(path => path, StringComparer.Ordinal).Select(path => MetadataReference.CreateFromFile(path))
                .ToImmutableArray<MetadataReference>();
            Equal(167, references.Length, "REFERENCES");
            RunTest(nameof(DeclarationAndMemberMatrix), DeclarationAndMemberMatrix);
            RunTest(nameof(EnumCarrierMatrix), EnumCarrierMatrix);
            RunTest(nameof(RecursiveDefaults), RecursiveDefaults);
            RunTest(nameof(ForbiddenDeclarations), ForbiddenDeclarations);
            RunTest(nameof(IdentityMutationAndEnumEscapes), IdentityMutationAndEnumEscapes);
            RunTest(nameof(SourceExceptionsAreClassificationOnly), SourceExceptionsAreClassificationOnly);
            RunTest(nameof(FrozenLimitsAreInclusive), FrozenLimitsAreInclusive);
            RunTest(nameof(ArtifactsAndPrecedence), ArtifactsAndPrecedence);
            RunTest(nameof(SourceRuntimeDifferential), SourceRuntimeDifferential);
            return 0;
        }
        catch (Exception error)
        {
            string code = error is HarnessFailure harness ? harness.Code
                : error is PracticalCaptureFailure practical ? practical.Family + "_" + practical.Code
                : error.GetType().Name;
            Console.Error.Write("CSHARP_PRACTICAL_TYPES_TEST_" + currentTest + "_" + code + "\n");
            return 1;
        }
    }

    private static void RunTest(string name, Action test) { currentTest = name; test(); }
    private static string Id(string name) => PracticalIdentity.SourceTypeId("Business", name);
    private static string Value(string name) => PracticalIdentity.PrimitiveId(name);
    private static PracticalDataType Type(PracticalDataTypes result, string name) => result.Types.Single(type => type.Id == Id(name));
    private static string Entry(string declarations, string body = "return input;", string parameters = "int input", string result = "int") =>
        "namespace Business;\n" + declarations + "\npublic static class Entry { public static " + result
        + " Run(" + parameters + ") { " + body + " } }\n";

    private static PracticalDataTypes Run(string source, string[]? parameters = null, string? result = null,
        bool reverseReferences = false, bool sidecar = false)
    {
        string root = PracticalIdentity.CallableId("method", "Business", Id("Entry"), "Run",
            parameters ?? new[] { Value("i32") }, result ?? Value("i32"));
        var inputs = new List<PracticalCapturedInput>
        {
            new(PracticalCapturedInputKind.Source, "src/Entry.cs", Encoding.UTF8.GetBytes(source)),
        };
        if (sidecar) { inputs.Add(new(PracticalCapturedInputKind.Sidecar, "contracts/value.json", Encoding.UTF8.GetBytes("{}"))); }
        return CSharpPracticalDataTypes.Validate(new PracticalSourceSelection(
            CSharpPracticalCapture.SelectionSchema, "business", new[] { "src/Entry.cs" }, new[] { root },
            sidecar ? new[] { "contracts/value.json" } : Array.Empty<string>()), inputs,
            reverseReferences ? references.Reverse().ToImmutableArray() : references);
    }

    private static void DeclarationAndMemberMatrix()
    {
        string source = Entry("public readonly struct Amount { public readonly int Major, Minor; public int Total => Major + Minor; }\n"
            + "public sealed class Item { public readonly Amount Amount; public int Count { get; } public int Optional { get; init; } public required int Required { get; init; } }",
            "return input.Count;", "Item input");
        PracticalDataTypes result = Run(source, new[] { Id("Item") });
        Equal(2, result.Types.Count, "DATA_COUNT");
        Equal(Id("Amount"), result.Types[0].Id, "DEPENDENCY_ORDER");
        PracticalDataType amount = Type(result, "Amount");
        Equal("readonly_struct", amount.Kind, "STRUCT");
        Check(amount.DefaultEligible, "STRUCT_DEFAULT");
        Equal("Major,Minor,Total", string.Join(',', amount.Members.Select(member => member.Name)), "MEMBER_ORDER");
        Equal(2, amount.DefaultValue!.Members.Count, "COMPUTED_GETTER_NOT_STORAGE");
        PracticalDataType item = Type(result, "Item");
        Equal("sealed_class", item.Kind, "CLASS");
        Check(!item.DefaultEligible, "CLASS_DEFAULT");
        Check(item.Members.Last().Required, "REQUIRED_RETAINED");
        Equal(1, item.StructuralDepth, "DEPTH");
        string explicitGetter = source.Replace("public int Total => Major + Minor;", "public int Total { get { return Major + Minor; } }");
        Check(result.CopyCanonicalBytes().SequenceEqual(Run(explicitGetter, new[] { Id("Item") }).CopyCanonicalBytes()), "GETTER_EQUIVALENCE");
        foreach (string access in new[] { "public", "internal", "private" })
        {
            Run(Entry("public readonly struct Data { " + access + " readonly int Field; public Data(int field) { Field = field; } public int Read() { return Field; } }",
                "return new Data(input).Read();"));
        }
    }

    private static void EnumCarrierMatrix()
    {
        (string Carrier, string Min, string Max)[] carriers =
        {
            ("sbyte", "-128", "127"), ("byte", "0", "255"), ("short", "-32768", "32767"),
            ("ushort", "0", "65535"), ("int", "-2147483648", "2147483647"),
            ("uint", "0", "4294967295"), ("long", "-9223372036854775808", "9223372036854775807"),
            ("ulong", "0", "18446744073709551615"),
        };
        foreach (var carrier in carriers)
        {
            string source = Entry("public enum State : " + carrier.Carrier + " { Zero = 0, Alias = 0, Min = "
                + carrier.Min + ", Max = " + carrier.Max + " }", "return input;", "State input", "State");
            PracticalDataType type = Type(Run(source, new[] { Id("State") }, Id("State")), "State");
            Equal("enum", type.Kind, "ENUM_KIND");
            Check(type.DefaultEligible, "ENUM_ZERO");
            Equal(carrier.Min, type.EnumMembers[2].Value, "EXACT_MIN");
            Equal(carrier.Max, type.EnumMembers[3].Value, "EXACT_MAX");
            Equal("0", type.EnumMembers[1].Value, "ALIAS");
        }
        PracticalDataType implicitValues = Type(Run(Entry("public enum State { First, Second, Alias = Second, Last }",
            "return input;", "State input", "State"), new[] { Id("State") }, Id("State")), "State");
        Equal("0,1,1,2", string.Join(',', implicitValues.EnumMembers.Select(member => member.Value)), "IMPLICIT_ENUM_VALUES");
        PracticalDataTypes day = Run(Entry("", "return input;", "System.DayOfWeek input", "System.DayOfWeek"),
            new[] { Value("day_of_week") }, Value("day_of_week"));
        Equal(Value("day_of_week"), day.Types.Single().Id, "FRAMEWORK_ENUM");
        Equal("0,1,2,3,4,5,6", string.Join(',', day.Types.Single().EnumMembers.Select(member => member.Value)), "DAYS");
    }

    private static void RecursiveDefaults()
    {
        string declarations = "public enum State { None, Ready }\npublic readonly struct Leaf { public readonly State State; public readonly int Count; }\n"
            + "public readonly struct Data { public readonly Leaf Leaf; public readonly string? Text; public readonly int[]? Values; }";
        string source = Entry(declarations, "return default(Data);", "int input", "Data");
        PracticalDataType data = Type(Run(source, result: Id("Data")), "Data");
        Check(data.DefaultEligible, "RECURSIVE_ELIGIBLE");
        Equal("product", data.DefaultValue!.Kind, "EXPANDED_PRODUCT");
        Equal("enum", data.DefaultValue.Members[0].Members[0].Kind, "EXPANDED_ENUM");
        Equal("none", data.DefaultValue.Members[1].Kind, "NULLABLE_REFERENCE_NONE");
        Equal("none", data.DefaultValue.Members[2].Kind, "NULLABLE_ARRAY_NONE");
        Reject(Entry(declarations.Replace("None, Ready", "Ready = 1"), "return default(Data);", result: "Data"),
            "ineligible_default", result: Id("Data"));
        Reject(Entry("public readonly struct Data { public readonly string Text; }", "return default(Data);", result: "Data"),
            "ineligible_default", result: Id("Data"));
        Reject(Entry("public readonly struct Data { public readonly int[] Values; }", "return new Data();", result: "Data"),
            "ineligible_default", result: Id("Data"));
        currentTest = nameof(RecursiveDefaults) + "_ExplicitCtor";
        Reject(Entry("public readonly struct Data { public required int Value { get; init; } }",
            "return default(Data);", result: "Data"), "ineligible_default", result: Id("Data"));
        string explicitCtor = Entry("public readonly struct Data { public readonly int Value; public Data() { Value = 7; } }",
            "Data value = new Data(); return value.Value + default(Data).Value;");
        Check(Type(Run(explicitCtor), "Data").DefaultEligible, "DEFAULT_DOES_NOT_CALL_CONSTRUCTOR");
        currentTest = nameof(RecursiveDefaults) + "_NullableEnum";
        Run(Entry("public enum State { Ready = 1 }", "return default(State?);", result: "State?"),
            result: PracticalIdentity.ClosedInstanceId("option", Id("State")));
        Reject(Entry("public enum State { Ready = 1 }", "return default(State);", result: "State"),
            "ineligible_default", result: Id("State"));
        Reject(Entry("public readonly struct Data { public readonly int Value; }", "return default;", result: "Data"),
            "target_typed_default", result: Id("Data"));
        currentTest = nameof(RecursiveDefaults) + "_ClassDefault";
        Run(Entry("public enum State { Ready = 1 }", "return default(State?);", result: "State?"),
            result: PracticalIdentity.ClosedInstanceId("option", Id("State")), sidecar: true);
        PracticalDataType pendingEnum = Type(Run(Entry("public enum State { None, Ready }",
            "return input;", "State input", "State"), new[] { Id("State") }, Id("State"), sidecar: true), "State");
        Check(!pendingEnum.DefaultEligible && pendingEnum.DefaultInvariantPending, "ENUM_PENDING_DEFAULT_FACT");
        string classDefault = Entry("public sealed class Data { }", "return default(Data?);", result: "Data?");
        Run(classDefault, result: Id("Data"));
        currentTest = nameof(RecursiveDefaults) + "_PendingInvariant";
        PracticalDataTypes pending = Run(Entry("public readonly struct Data { public readonly int Value; }",
            "return input.Value;", "Data input"), new[] { Id("Data") }, sidecar: true);
        Check(!Type(pending, "Data").DefaultEligible && Type(pending, "Data").DefaultInvariantPending,
            "OPAQUE_SIDECAR_IS_NOT_INVARIANT_PROOF");
        Reject(Entry("public readonly struct Data { public readonly int Value; }", "return default(Data);", result: "Data"),
            "default_invariant_pending", result: Id("Data"), sidecar: true);
    }

    private static void ForbiddenDeclarations()
    {
        string[] declarations =
        {
            "public struct Data { public int Value; }",
            "public class Data { }",
            "public sealed class Data : object { }",
            "public sealed class Data { public int Value; }",
            "public sealed class Data { public static int Value; }",
            "public sealed class Data { public const int Value = 1; }",
            "public sealed class Data { public static int Value { get; } }",
            "public sealed class Data { static Data() { } }",
            "public sealed class Data { public int Value { get; set; } }",
            "public sealed class Data { private readonly int value; public int Value { get { return value; } init { this.value = value; } } }",
            "public sealed class Data { public readonly int Value = 1; }",
            "public sealed class Data { public int Value { get; } = 1; }",
            "public readonly record struct Data(int Value);",
            "public sealed record Data(int Value);",
            "public partial class Data { }",
            "public sealed class Data { public sealed class Nested { } }",
            "public interface Data { }",
            "public sealed class Data : System.IDisposable { public void Dispose() { } }",
            "public sealed class Data { public int this[int index] => index; }",
            "public sealed class Data { ~Data() { } }",
            "public sealed class Data { public static Data operator +(Data left, Data right) { return left; } }",
            "public sealed class Data { public static implicit operator int(Data value) { return 0; } }",
            "public sealed class Data { public event System.Action? Changed; public void Raise() { Changed?.Invoke(); } }",
            "public readonly ref struct Data { }",
            "public sealed class Data { public readonly System.WeakReference Reference; }",
            "[System.Runtime.InteropServices.StructLayout(System.Runtime.InteropServices.LayoutKind.Explicit)] public readonly struct Data { [System.Runtime.InteropServices.FieldOffset(0)] public readonly int Value; }",
            "public sealed class Data { public readonly int Value; public Data() : base() { } }",
        };
        for (int index = 0; index < declarations.Length; index++)
        {
            Reject(Entry(declarations[index], "return 0;", "Data input"), parameters: new[] { Id("Data") }, label: "DECL_" + index);
        }
        Reject(Entry("public static class Dead { public const int Value = 1; }"), "data_modifier");
    }

    private static void IdentityMutationAndEnumEscapes()
    {
        const string Enumeration = "public enum State { None, Ready }";
        foreach (string expression in new[] { "(State)input", "(State)0", "0", "(State)unchecked((byte)255)" })
        { Reject(Entry(Enumeration, "return " + expression + ";", result: "State"), "enum_conversion", result: Id("State")); }
        foreach (string body in new[] { "return input + 1;", "return input | State.Ready;", "return ~input;", "input++; return input;", "input |= State.Ready; return input;" })
        { Reject(Entry(Enumeration, body, "State input", "State"), "enum_arithmetic", new[] { Id("State") }, Id("State")); }
        Reject(Entry(Enumeration, "return (State?)input;", result: "State?"), "enum_conversion",
            result: PracticalIdentity.ClosedInstanceId("option", Id("State")));
        Reject(Entry(Enumeration, "return 0;", result: "State?"), "enum_conversion",
            result: PracticalIdentity.ClosedInstanceId("option", Id("State")));
        Reject(Entry("public enum State { Ready = 1 }", "return new State();", result: "State"),
            "ineligible_default", result: Id("State"));
        Reject(Entry(Enumeration, "return (int)input;", "State input"), "enum_conversion", new[] { Id("State") });
        Run(Entry(Enumeration, "return input == State.Ready ? 1 : 0;", "State input"), new[] { Id("State") });
        Reject(Entry("", "return (System.DayOfWeek)input;", result: "System.DayOfWeek"), "enum_conversion", result: Value("day_of_week"));
        foreach (string body in new[] { "return input == input ? 1 : 0;", "return input != null ? 1 : 0;", "return object.ReferenceEquals(input, input) ? 1 : 0;", "return input.GetHashCode();", "return input.GetType() == typeof(Data) ? 1 : 0;" })
        { Reject(Entry("public sealed class Data { }", body, "Data input"), parameters: new[] { Id("Data") }); }
        Reject(Entry("public readonly struct Data { public readonly int Value; public void Reset() { this = default(Data); } }",
            "input.Reset(); return input.Value;", "Data input"), "compiler_diagnostic", new[] { Id("Data") });
        Reject(Entry("public sealed class Data { public readonly int[] Values; public Data(int[] values) { Values = values; } public int Mutate() { Values[0] = 1; return Values[0]; } }",
            "return new Data(new[] { input }).Mutate();"), "reachable_mutation");
        foreach (string body in new[]
        {
            "int[] alias = Values; alias[0] = 1; return alias[0];",
            "int[] alias = new int[1]; int[] second = alias; alias = Values; second = alias; second[0]++; return second[0];",
            "int[][] alias = new[] { Values }; alias[0][0] = 1; return alias[0][0];",
            "int[][] alias = new int[1][]; alias[0] = Values; alias[0][0] = 1; return alias[0][0];",
        })
        {
            Reject(Entry("public sealed class Data { public readonly int[] Values; public Data(int[] values) { Values = values; } public int Mutate() { " + body + " } }",
                "return new Data(new[] { input }).Mutate();"), "reachable_mutation");
        }
        Reject(Entry("public sealed class Data { public readonly int[] Values; public Data(int[] values) { Values = values; } }",
            "input.Values[0] = 1; return input.Values[0];", "Data input"), "reachable_mutation", new[] { Id("Data") });
        Reject(Entry("public sealed class Data { public readonly int[] Values; public Data(int[] values) { Values = values; } public static void Change(ref int value) { value = 3; } }",
            "Data value = new Data(new[] { input }); Data.Change(ref value.Values[0]); return value.Values[0];"),
            "identity_or_reference_escape");
        const string ArrayValue = "public sealed class Data { public readonly int[] Values; public Data(int[] values) { Values = values; } }";
        foreach (string body in new[]
        {
            "int[] values = new[] { input }; Data value = new Data(values); values[0]++; return value.Values[0];",
            "int[] values = new[] { input }; int[] alias = values; Data value = new Data(alias); values[0] = 3; return value.Values[0];",
        })
        { Reject(Entry(ArrayValue, body), "published_construction_mutation"); }
        Run(Entry(ArrayValue, "Data value = new Data(new[] { input }); int[] fresh = new[] { value.Values[0] }; fresh[0]++; return fresh[0];"));
        Run(Entry(ArrayValue, "int[] values = new[] { input }; values[0] = 3; Data value = new Data(values); return value.Values[0];"));
        Run(Entry("public sealed class Data { public int Make(int count) { int[] fresh = new int[count]; fresh[0] = 7; return fresh[0]; } }",
            "return input.Make(1);", "Data input"), new[] { Id("Data") });
        Reject(Entry("public readonly struct Data { public readonly int Value; }", "return input with { };", "Data input", "Data"),
            "identity_or_reference_escape", new[] { Id("Data") }, Id("Data"));
        Reject(Entry("public sealed class A { public B? Next { get; } } public sealed class B { public A? Next { get; } }",
            "return 0;", "A input"), "type_cycle", new[] { Id("A") });
    }

    private static void SourceExceptionsAreClassificationOnly()
    {
        foreach (string declaration in new[]
        {
            "public sealed class Failure : System.Exception { }",
            "public class Failure : System.Exception { }", // classification is not sealed admission
            "public sealed class Failure : System.IO.IOException { }", // not an admitted base claim
            "public class Base : System.Exception { } public sealed class Failure : Base { }",
        })
        {
            CSharpCompilation compilation = Compile(Entry(declaration));
            INamedTypeSymbol symbol = compilation.GetTypeByMetadataName("Business.Failure")!;
            PracticalSourceExceptionCandidate candidate = CSharpPracticalDataTypes.ClassifySourceException(symbol)
                ?? throw new HarnessFailure("MISSING_EXCEPTION_HANDOFF");
            Equal(Id("Failure"), candidate.SourceId, "EXCEPTION_SOURCE_ID");
            Check(candidate.Declaration.GetSyntax() is ClassDeclarationSyntax { BaseList: not null }, "ORIGINAL_BASE_CLAUSE");
            // No candidate is admitted or emitted on the W03 ordinary-data path.
            Reject(Entry(declaration, "return 0;", "Failure input"), parameters: new[] { Id("Failure") });
        }
        CSharpCompilation ordinary = Compile(Entry("public sealed class Failure { }"));
        Check(CSharpPracticalDataTypes.ClassifySourceException(ordinary.GetTypeByMetadataName("Business.Failure")!) is null,
            "NAME_DOES_NOT_CLASSIFY_EXCEPTION");
    }

    private static void FrozenLimitsAreInclusive()
    {
        foreach (int count in new[] { 31, 32, 33 })
        {
            string fields = string.Join(" ", Enumerable.Range(0, count).Select(index => "public readonly int F" + index + ";"));
            string source = Entry("public readonly struct Data { " + fields + " }", "return input.F0;", "Data input");
            if (count == 33)
            {
                Reject(source, "fields_properties_per_type", new[] { Id("Data") });
                Reject(source.Replace("readonly struct", "struct").Replace("readonly int", "int"),
                    "fields_properties_per_type", new[] { Id("Data") });
                Reject(source.Replace("return input.F0;", "Mpk.Missing(); return input.F0;"),
                    "fields_properties_per_type", new[] { Id("Data") });
            }
            else { Equal(count, Type(Run(source, new[] { Id("Data") }), "Data").Members.Count, "MEMBER_LIMIT"); }
        }
        foreach (int count in new[] { 31, 32, 33 })
        {
            string members = "public readonly int A, B; " + string.Join(" ", Enumerable.Range(2, count - 2)
                .Select(index => "public int P" + index + " { get; }"));
            string source = Entry("public readonly struct Data { " + members + " }", "return input.A;", "Data input");
            if (count == 33) { Reject(source, "fields_properties_per_type", new[] { Id("Data") }); }
            else { Equal(count, Type(Run(source, new[] { Id("Data") }), "Data").Members.Count, "MIXED_MEMBER_LIMIT"); }
        }
        foreach (int depth in new[] { 15, 16, 17 })
        {
            var declarations = new StringBuilder("public readonly struct T0 { public readonly int Value; }\n");
            for (int index = 1; index <= depth; index++)
            { declarations.Append("public readonly struct T").Append(index).Append(" { public readonly T").Append(index - 1).Append(" Value; }\n"); }
            string name = "T" + depth;
            string source = Entry(declarations.ToString(), "return 0;", name + " input");
            if (depth == 17) { Reject(source, "structural_type_nesting", new[] { Id(name) }); }
            else { Equal(depth, Type(Run(source, new[] { Id(name) }), name).StructuralDepth, "DEPTH_LIMIT"); }
        }
    }

    private static void ArtifactsAndPrecedence()
    {
        string source = Entry("public readonly struct Data { public readonly int Value; }", "return input.Value;", "Data input");
        PracticalDataTypes first = Run(source, new[] { Id("Data") });
        PracticalDataTypes second = Run(source, new[] { Id("Data") }, reverseReferences: true);
        Check(first.CopyCanonicalBytes().SequenceEqual(second.CopyCanonicalBytes()), "DETERMINISM");
        byte[] changed = first.CopyCanonicalBytes(); changed[0] ^= 1;
        Check(first.CopyCanonicalBytes()[0] != changed[0], "IMMUTABLE_BYTES");
        Equal(0, first.ArtifactCount, "PRIVATE_HANDOFF");
        Reject(Entry("public sealed class Data { public int Value; }", "System.Console.WriteLine(input); return 0;", "Data input"),
            "data_field", new[] { Id("Data") });
        Reject(Entry("public readonly struct Generic<T> { } public enum State { None, Ready }",
            "return (State)input;", result: "State"), "enum_conversion", result: Id("State"));
        Reject(Entry("public sealed class Generic<T> { } public readonly struct Data { public readonly Generic<int>? Value; } public enum State { None, Ready }",
            "return (State)input;", result: "State"), "enum_conversion", result: Id("State"));
        Reject(Entry("public struct Data { }", "System.Collections.Generic.List<int>? values = null; return values == null ? 0 : 1;", "Data input"),
            "data_type_kind", new[] { Id("Data") });
    }

    private static CSharpCompilation Compile(string source) => CSharpCompilation.Create("differential_" + Guid.NewGuid().ToString("N"),
        new[] { CSharpSyntaxTree.ParseText(source, new CSharpParseOptions((LanguageVersion)1400)) }, references,
        new CSharpCompilationOptions(OutputKind.DynamicallyLinkedLibrary, checkOverflow: true,
            nullableContextOptions: NullableContextOptions.Enable, optimizationLevel: OptimizationLevel.Release));

    private static void SourceRuntimeDifferential()
    {
        string source = Entry("public enum State : ulong { None, Alias = 0, Max = 18446744073709551615 } "
            + "public readonly struct Data { public readonly State State; public readonly int Count; public Data() { State = State.Max; Count = 7; } }",
            "Data constructed = new Data(); _ = constructed.Count; return default(Data);", result: "Data");
        PracticalDataType data = Type(Run(source, result: Id("Data")), "Data");
        CSharpCompilation compilation = Compile(source);
        using var pe = new MemoryStream();
        Check(compilation.Emit(pe).Success, "RUNTIME_EMIT"); pe.Position = 0;
        var context = new AssemblyLoadContext("practical-types-differential", isCollectible: true);
        try
        {
            Assembly assembly = context.LoadFromStream(pe);
            object actual = assembly.GetType("Business.Entry")!.GetMethod("Run")!.Invoke(null, new object[] { 0 })!;
            Equal(data.DefaultValue!.Members[0].Scalar,
                Convert.ToUInt64(actual.GetType().GetField("State")!.GetValue(actual), CultureInfo.InvariantCulture).ToString(CultureInfo.InvariantCulture), "RUNTIME_ENUM_ZERO");
            Equal(data.DefaultValue.Members[1].Scalar,
                actual.GetType().GetField("Count")!.GetValue(actual)!.ToString()!, "RUNTIME_STRUCT_ZERO");
            object constructed = Activator.CreateInstance(actual.GetType())!;
            Equal(7, (int)constructed.GetType().GetField("Count")!.GetValue(constructed)!, "RUNTIME_CONSTRUCTOR_DISTINCT");
            Type runtimeEnum = assembly.GetType("Business.State")!;
            PracticalDataType enumType = Type(Run(source, result: Id("Data")), "State");
            foreach (PracticalEnumMember member in enumType.EnumMembers)
            {
                Equal(member.Value, Convert.ToUInt64(runtimeEnum.GetField(member.Name)!.GetValue(null), CultureInfo.InvariantCulture)
                    .ToString(CultureInfo.InvariantCulture), "RUNTIME_ENUM_CARRIER");
            }
        }
        finally { context.Unload(); }
    }

    private static void Reject(string source, string? code = null, string[]? parameters = null,
        string? result = null, string label = "REJECT", bool sidecar = false)
    {
        try { Run(source, parameters, result, sidecar: sidecar); }
        catch (PracticalCaptureFailure failure)
        {
            Equal(0, failure.ArtifactCount, label + "_ARTIFACTS");
            Equal(PracticalCaptureFailure.PublicMessage, failure.Message, label + "_MESSAGE");
            if (code is not null) { Equal(code, failure.Code, label + "_EXPECTED_" + code + "_ACTUAL_" + failure.Code); }
            Check(failure.Code is not "frontend_adapter" and not "syntax_normalizer", label + "_UNEXPECTED_ADAPTER_FAILURE");
            return;
        }
        throw new HarnessFailure(label + "_EXPECTED_FAILURE_" + code);
    }
    private static void Check(bool condition, string code) { if (!condition) { throw new HarnessFailure(code); } }
    private static void Equal<T>(T expected, T actual, string code) where T : notnull => Check(EqualityComparer<T>.Default.Equals(expected, actual), code);
    private sealed class HarnessFailure : Exception
    {
        internal HarnessFailure(string code) { Code = code; }
        internal string Code { get; }
    }
}
