using System;

namespace Mpk.CSharp2Vir;

internal sealed class CanonicalArtifact
{
    private readonly byte[] canonicalBytes;

    internal CanonicalArtifact(string schema, string sha256, byte[] canonicalBytes)
    {
        Schema = schema;
        Sha256 = sha256;
        this.canonicalBytes = canonicalBytes;
    }

    internal string Schema { get; }

    internal string Sha256 { get; }

    internal ReadOnlySpan<byte> CanonicalBytes => canonicalBytes;
}

internal sealed class MappedSourceOrigin
{
    internal MappedSourceOrigin(
        string normalizedPath,
        int utf8Start,
        int utf8End,
        int lineStart,
        int columnStartUtf16,
        int lineEnd,
        int columnEndUtf16)
    {
        NormalizedPath = normalizedPath;
        Utf8Start = utf8Start;
        Utf8End = utf8End;
        LineStart = lineStart;
        ColumnStartUtf16 = columnStartUtf16;
        LineEnd = lineEnd;
        ColumnEndUtf16 = columnEndUtf16;
    }

    internal string NormalizedPath { get; }

    internal int Utf8Start { get; }

    internal int Utf8End { get; }

    internal int LineStart { get; }

    internal int ColumnStartUtf16 { get; }

    internal int LineEnd { get; }

    internal int ColumnEndUtf16 { get; }
}

internal sealed class EmittedFrontendSuccess
{
    private readonly byte[] envelopeBytes;

    internal EmittedFrontendSuccess(
        CanonicalArtifact vir,
        CanonicalArtifact sourceMap,
        CanonicalArtifact sourceManifest,
        byte[] envelopeBytes)
    {
        Vir = vir;
        SourceMap = sourceMap;
        SourceManifest = sourceManifest;
        this.envelopeBytes = envelopeBytes;
    }

    internal CanonicalArtifact Vir { get; }

    internal CanonicalArtifact SourceMap { get; }

    internal CanonicalArtifact SourceManifest { get; }

    internal ReadOnlySpan<byte> EnvelopeBytes => envelopeBytes;
}

internal static class EmissionFailure
{
    internal static FrontendFailure ExternalSource()
    {
        return FrontendFailure.Toolchain("emission", "CSHARP_SOURCE_MAP_EXTERNAL");
    }

    internal static FrontendFailure SourceRange()
    {
        return FrontendFailure.Toolchain("emission", "CSHARP_SOURCE_MAP_RANGE");
    }

    internal static FrontendFailure Utf16Boundary()
    {
        return FrontendFailure.Toolchain("emission", "CSHARP_SOURCE_MAP_UTF16");
    }

    internal static FrontendFailure Adapter()
    {
        return FrontendFailure.Toolchain("emission", "CSHARP_TOOLCHAIN_ADAPTER");
    }

    internal static FrontendFailure Internal()
    {
        return FrontendFailure.Internal("emission");
    }
}
