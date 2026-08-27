using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading;
using System.Text.Json;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.Text;

namespace Mpk.CSharp2Vir;

internal sealed class CSharpSourceMapper
{
    private readonly Dictionary<string, Utf16SourceFileMap> files =
        new Dictionary<string, Utf16SourceFileMap>(StringComparer.Ordinal);

    internal CSharpSourceMapper(
        Selection selection,
        CapturedSourceSet capturedSources,
        RoslynCompilationSession session)
    {
        if (capturedSources.Count != selection.Raw.Sources.Count
            || session.Source.SourceTexts.Length != selection.Raw.Sources.Count
            || session.Source.SyntaxTrees.Length != selection.Raw.Sources.Count)
        {
            throw EmissionFailure.ExternalSource();
        }

        for (int index = 0; index < selection.Raw.Sources.Count; index++)
        {
            string path = selection.Raw.Sources[index];
            CapturedSourceText captured = capturedSources.SourceAt(index);
            SourceText sourceText = session.Source.SourceTexts[index];
            SyntaxTree tree = session.Source.SyntaxTrees[index];
            if (!string.Equals(path, captured.NormalizedPath, StringComparison.Ordinal)
                || !string.Equals(path, tree.FilePath, StringComparison.Ordinal)
                || !string.Equals(captured.Text, sourceText.ToString(), StringComparison.Ordinal)
                || !tree.GetText(CancellationToken.None).ContentEquals(sourceText)
                || !files.TryAdd(
                    path,
                    new Utf16SourceFileMap(path, captured.Text, sourceText, tree)))
            {
                throw EmissionFailure.ExternalSource();
            }
        }
    }

    internal MappedSourceOrigin Map(LoweredOrigin origin)
    {
        if (!files.TryGetValue(origin.NormalizedPath, out Utf16SourceFileMap? file)
            || !file.Owns(origin.SourceIdentity))
        {
            throw EmissionFailure.ExternalSource();
        }

        return file.Map(origin.Utf16Start, origin.Utf16End, crossCheckRoslyn: true);
    }

    internal static MappedSourceOrigin MapVector(string source, int utf16Start, int utf16End)
    {
        return new Utf16SourceFileMap("src/vector.cs", source, null, null)
            .Map(utf16Start, utf16End, crossCheckRoslyn: false);
    }

    private sealed class Utf16SourceFileMap
    {
        private readonly string path;
        private readonly string source;
        private readonly int[] utf8Boundaries;
        private readonly int[] lineStarts;
        private readonly SourceText? sourceText;
        private readonly SyntaxTree? syntaxTree;

        internal Utf16SourceFileMap(
            string path,
            string source,
            SourceText? sourceText,
            SyntaxTree? syntaxTree)
        {
            this.path = path;
            this.source = source;
            this.sourceText = sourceText;
            this.syntaxTree = syntaxTree;
            utf8Boundaries = BuildUtf8Boundaries(source);
            lineStarts = BuildLineStarts(source);
        }

        internal MappedSourceOrigin Map(
            int utf16Start,
            int utf16End,
            bool crossCheckRoslyn)
        {
            if (utf16Start < 0
                || utf16End <= utf16Start
                || utf16End > source.Length)
            {
                throw EmissionFailure.SourceRange();
            }

            int utf8Start = utf8Boundaries[utf16Start];
            int utf8End = utf8Boundaries[utf16End];
            if (utf8Start < 0 || utf8End < 0)
            {
                throw EmissionFailure.Utf16Boundary();
            }

            (int lineStart, int columnStart) = LineAndColumn(utf16Start);
            (int lineEnd, int columnEnd) = LineAndColumn(utf16End);
            if (crossCheckRoslyn)
            {
                CrossCheckRoslyn(
                    utf16Start,
                    utf16End,
                    lineStart,
                    columnStart,
                    lineEnd,
                    columnEnd);
            }

            return new MappedSourceOrigin(
                path,
                utf8Start,
                utf8End,
                lineStart,
                columnStart,
                lineEnd,
                columnEnd);
        }

        internal bool Owns(object? sourceIdentity)
        {
            return syntaxTree is not null && ReferenceEquals(syntaxTree, sourceIdentity);
        }

        private void CrossCheckRoslyn(
            int utf16Start,
            int utf16End,
            int lineStart,
            int columnStart,
            int lineEnd,
            int columnEnd)
        {
            if (sourceText is null
                || syntaxTree is null
                || sourceText.Length != source.Length)
            {
                throw EmissionFailure.ExternalSource();
            }

            FileLinePositionSpan roslyn;
            try
            {
                roslyn = syntaxTree.GetLineSpan(
                    TextSpan.FromBounds(utf16Start, utf16End),
                    CancellationToken.None);
            }
            catch (Exception error) when (
                error is ArgumentException
                || error is ArgumentOutOfRangeException
                || error is InvalidOperationException)
            {
                throw EmissionFailure.Adapter();
            }

            if (!roslyn.IsValid
                || !string.Equals(roslyn.Path, path, StringComparison.Ordinal)
                || roslyn.StartLinePosition.Line != lineStart
                || roslyn.StartLinePosition.Character != columnStart
                || roslyn.EndLinePosition.Line != lineEnd
                || roslyn.EndLinePosition.Character != columnEnd)
            {
                throw EmissionFailure.Adapter();
            }
        }

        private (int Line, int Column) LineAndColumn(int utf16Offset)
        {
            int found = Array.BinarySearch(lineStarts, utf16Offset);
            int line = found >= 0 ? found : ~found - 1;
            if (line < 0)
            {
                throw EmissionFailure.Internal();
            }

            return (line, utf16Offset - lineStarts[line]);
        }

        private static int[] BuildUtf8Boundaries(string source)
        {
            var boundaries = new int[source.Length + 1];
            Array.Fill(boundaries, -1);
            int utf16 = 0;
            int utf8 = 0;
            boundaries[0] = 0;
            while (utf16 < source.Length)
            {
                char current = source[utf16];
                if (char.IsHighSurrogate(current))
                {
                    if (utf16 + 1 >= source.Length
                        || !char.IsLowSurrogate(source[utf16 + 1]))
                    {
                        throw EmissionFailure.Utf16Boundary();
                    }

                    utf8 = checked(utf8 + 4);
                    utf16 += 2;
                    boundaries[utf16] = utf8;
                    continue;
                }

                if (char.IsLowSurrogate(current))
                {
                    throw EmissionFailure.Utf16Boundary();
                }

                utf8 = checked(utf8 + (current <= 0x7f ? 1 : current <= 0x7ff ? 2 : 3));
                utf16++;
                boundaries[utf16] = utf8;
            }

            return boundaries;
        }

        private static int[] BuildLineStarts(string source)
        {
            var starts = new List<int> { 0 };
            for (int index = 0; index < source.Length; index++)
            {
                if (source[index] == '\n')
                {
                    starts.Add(index + 1);
                }
            }

            return starts.ToArray();
        }
    }
}

internal static class CSharpSourceMapEmitter
{
    internal static CanonicalArtifact Emit(
        Selection selection,
        LoweredClosure closure,
        RoslynCompilationSession session,
        CapturedSourceSet sources,
        CanonicalArtifact vir)
    {
        if (!string.Equals(vir.Schema, CSharpEmissionProfiles.VirSchema, StringComparison.Ordinal))
        {
            throw EmissionFailure.Internal();
        }

        var mapper = new CSharpSourceMapper(selection, sources, session);
        SourceMapRecord[] records = BuildRecords(selection.Raw.Compilation, closure, mapper);
        byte[] payload = WriteMap(vir, records, null);
        string hash = EmissionCanonical.Hash(CSharpEmissionProfiles.SourceMapHashDomain, payload);
        byte[] canonical = WriteMap(vir, records, hash);
        return new CanonicalArtifact(CSharpEmissionProfiles.SourceMapSchema, hash, canonical);
    }

    private static SourceMapRecord[] BuildRecords(
        string unitId,
        LoweredClosure closure,
        CSharpSourceMapper mapper)
    {
        var records = new List<SourceMapRecord>();
        foreach (LoweredFunction function in closure.Functions.OrderBy(
            function => function.Id,
            StringComparer.Ordinal))
        {
            records.Add(SourceMapRecord.CreateFunction(
                unitId,
                function.Id,
                mapper.Map(function.Origin)));
            foreach (LoweredBlock block in function.Blocks)
            {
                foreach (LoweredInstruction instruction in block.Instructions)
                {
                    records.Add(SourceMapRecord.CreateInstruction(
                        unitId,
                        function.Id,
                        block.Label,
                        instruction.Id,
                        mapper.Map(instruction.Origin)));
                }
            }

            foreach (LoweredBlock block in function.Blocks)
            {
                records.Add(SourceMapRecord.CreateTerminator(
                    unitId,
                    function.Id,
                    block.Label,
                    mapper.Map(block.Terminator.Origin)));
            }
        }

        return records.ToArray();
    }

    private static byte[] WriteMap(
        CanonicalArtifact vir,
        IReadOnlyList<SourceMapRecord> records,
        string? hash)
    {
        return EmissionCanonical.Write(writer =>
        {
            writer.WriteStartObject();
            writer.WritePropertyName("entries");
            writer.WriteStartArray();
            foreach (SourceMapRecord record in records)
            {
                WriteRecord(writer, record);
            }

            writer.WriteEndArray();
            writer.WriteString("schema", CSharpEmissionProfiles.SourceMapSchema);
            EmissionCanonical.WriteSemanticContext(writer);
            writer.WriteString("source_ir_hash", vir.Sha256);
            writer.WriteString("source_ir_schema", vir.Schema);
            if (hash is not null)
            {
                writer.WriteString("source_map_hash", hash);
            }

            writer.WriteEndObject();
        }, "source_map_canonical_bytes", "emission");
    }

    private static void WriteRecord(Utf8JsonWriter writer, SourceMapRecord record)
    {
        writer.WriteStartObject();
        writer.WritePropertyName("origin");
        writer.WriteStartObject();
        writer.WriteNumber("end", record.Origin.Utf8End);
        writer.WriteString("input_kind", "source");
        writer.WriteString("kind", "source");
        writer.WriteString("normalized_path", record.Origin.NormalizedPath);
        writer.WriteNumber("start", record.Origin.Utf8Start);
        writer.WriteEndObject();
        writer.WritePropertyName("reference");
        writer.WriteStartObject();
        if (record.Block is not null)
        {
            writer.WriteString("block", record.Block);
        }

        writer.WriteString("function_id", record.FunctionId);
        if (record.Instruction is not null)
        {
            writer.WriteString("instruction", record.Instruction);
        }

        writer.WriteString("kind", record.Kind);
        writer.WriteString("unit_id", record.UnitId);
        writer.WriteEndObject();
        writer.WriteEndObject();
    }

    private sealed class SourceMapRecord
    {
        private SourceMapRecord(
            string kind,
            string unitId,
            string functionId,
            string? block,
            string? instruction,
            MappedSourceOrigin origin)
        {
            Kind = kind;
            UnitId = unitId;
            FunctionId = functionId;
            Block = block;
            Instruction = instruction;
            Origin = origin;
        }

        internal string Kind { get; }

        internal string UnitId { get; }

        internal string FunctionId { get; }

        internal string? Block { get; }

        internal string? Instruction { get; }

        internal MappedSourceOrigin Origin { get; }

        internal static SourceMapRecord CreateFunction(
            string unitId,
            string functionId,
            MappedSourceOrigin origin)
        {
            return new SourceMapRecord("function", unitId, functionId, null, null, origin);
        }

        internal static SourceMapRecord CreateInstruction(
            string unitId,
            string functionId,
            string block,
            string instruction,
            MappedSourceOrigin origin)
        {
            return new SourceMapRecord(
                "instruction",
                unitId,
                functionId,
                block,
                instruction,
                origin);
        }

        internal static SourceMapRecord CreateTerminator(
            string unitId,
            string functionId,
            string block,
            MappedSourceOrigin origin)
        {
            return new SourceMapRecord("terminator", unitId, functionId, block, null, origin);
        }
    }
}
